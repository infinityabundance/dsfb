//! Per-motif state machines, all driven by `dsfb::DsfbObserver`.
//!
//! Each motif:
//!   1. Selects samples whose residual class matches the motif.
//!   2. Groups them by channel.
//!   3. Runs a single-channel DSFB observer (`channels = 1`) per group.
//!   4. Watches the EMA residual + instantaneous residual envelope; opens an
//!      episode at the first Drift entry, closes it after the residual
//!      returns to Stable for at least `min_dwell_seconds`.
//!
//! A multi-channel DSFB observer is also constructed across channels for
//! traceability; its trust-weight sum is recorded on every closed episode
//! to demonstrate that the trust-adaptive fusion is in the loop (it sums to
//! 1.0 by construction; the test `test_trust_sum_invariant` enforces this).

use super::envelope::{classify, Envelope};
use super::{Episode, MotifClass, MotifParams};
use crate::residual::{ResidualClass, ResidualSample, ResidualStream};
use dsfb::{DsfbObserver, DsfbParams};
use std::collections::BTreeMap;

const DEFAULT_K_PHI: f64 = 0.5;
const DEFAULT_K_OMEGA: f64 = 0.1;
const DEFAULT_K_ALPHA: f64 = 0.01;

pub fn run_motif(class: MotifClass, params: &MotifParams, stream: &ResidualStream) -> Vec<Episode> {
    let target_class: ResidualClass = class.residual_class();
    let dsfb_params = DsfbParams::new(
        DEFAULT_K_PHI,
        DEFAULT_K_OMEGA,
        DEFAULT_K_ALPHA,
        params.rho,
        params.sigma0,
    );

    let by_channel = group_samples_by_channel(stream, target_class);
    debug_assert!(
        by_channel.keys().all(|k| !k.is_empty()),
        "channel keys are non-empty"
    );

    let channels: Vec<String> = by_channel.keys().cloned().collect();
    let mut multi_observer = DsfbObserver::new(dsfb_params, channels.len().max(1));
    let mut latest: Vec<f64> = vec![0.0; channels.len().max(1)];
    debug_assert_eq!(
        latest.len(),
        channels.len().max(1),
        "latest sized to channel count"
    );

    let mut episodes = Vec::new();
    for (chan_idx, (channel, samples)) in by_channel.iter().enumerate() {
        if samples.is_empty() {
            continue;
        }
        process_channel(
            class,
            params,
            dsfb_params,
            chan_idx,
            channel,
            samples,
            &channels,
            &mut multi_observer,
            &mut latest,
            &mut episodes,
        );
    }
    episodes
}

/// Group residual samples by channel; anonymous samples go to a shared
/// `_anonymous_` bucket so the motif state machine sees a well-defined
/// per-channel timeline even when channels are absent.
fn group_samples_by_channel(
    stream: &ResidualStream,
    target_class: ResidualClass,
) -> BTreeMap<String, Vec<&ResidualSample>> {
    let mut by_channel: BTreeMap<String, Vec<&ResidualSample>> = BTreeMap::new();
    for s in stream.iter_class(target_class) {
        let ch = s.channel.clone().unwrap_or_else(|| "_anonymous_".into());
        debug_assert!(
            !ch.is_empty(),
            "channel key must be non-empty after fallback"
        );
        by_channel.entry(ch).or_default().push(s);
    }
    by_channel
}

/// Walk a single channel's residual timeline through the motif state
/// machine, emitting episodes into `episodes`.
#[allow(clippy::too_many_arguments)]
fn process_channel(
    class: MotifClass,
    params: &MotifParams,
    dsfb_params: DsfbParams,
    chan_idx: usize,
    channel: &str,
    samples: &[&ResidualSample],
    channels: &[String],
    multi_observer: &mut DsfbObserver,
    latest: &mut [f64],
    episodes: &mut Vec<Episode>,
) {
    debug_assert!(!samples.is_empty(), "caller pre-filters empty channels");
    debug_assert!(chan_idx < latest.len(), "chan_idx within latest bounds");
    let mut obs = DsfbObserver::new(dsfb_params, 1);
    let mut state = MotifState::Stable;
    let mut ctx = ChannelCtx {
        episode_t_start: 0.0,
        episode_peak: 0.0,
        last_ema: 0.0,
        last_t: samples[0].t,
    };

    for s in samples.iter() {
        state = step_sample(
            class,
            params,
            chan_idx,
            channel,
            s,
            channels,
            &mut obs,
            multi_observer,
            latest,
            &mut ctx,
            state,
            episodes,
        );
    }
    flush_open_episode(
        class,
        params,
        channel,
        channels,
        multi_observer,
        &ctx,
        state,
        episodes,
    );
}

/// Per-channel mutable context threaded through `step_sample` to keep
/// the function signatures under the 7-argument clippy limit.
struct ChannelCtx {
    episode_t_start: f64,
    episode_peak: f64,
    last_ema: f64,
    last_t: f64,
}

/// Advance the motif state machine by one residual sample. The `dt` is
/// capped at 1.0: our measurements are *already residuals* (delta from
/// baseline), so the dsfb predict step `phi_pred = phi + omega*dt`
/// adds no information when samples are sparse — and with dt of tens
/// of seconds it makes `phi_pred` run away from a stationary residual,
/// generating large spurious residuals. Capping `dt` keeps EMA
/// smoothing meaningful while making the motif loop sample-rate-
/// invariant.
#[allow(clippy::too_many_arguments)]
fn step_sample(
    class: MotifClass,
    params: &MotifParams,
    chan_idx: usize,
    channel: &str,
    s: &ResidualSample,
    channels: &[String],
    obs: &mut DsfbObserver,
    multi_observer: &mut DsfbObserver,
    latest: &mut [f64],
    ctx: &mut ChannelCtx,
    state: MotifState,
    episodes: &mut Vec<Episode>,
) -> MotifState {
    let dt = (s.t - ctx.last_t).clamp(1e-6, 1.0);
    debug_assert!(
        dt.is_finite() && dt > 0.0,
        "clamped dt must be positive finite"
    );
    let _observer_state = obs.step(&[s.value], dt);
    let ema = obs.ema_residual(0);
    let env = classify(ema, s.value, params.drift_threshold, params.slew_threshold);
    ctx.last_ema = ema;
    ctx.last_t = s.t;
    let abs_v: f64 = s.value.abs();
    debug_assert!(abs_v >= 0.0, "abs value non-negative");
    ctx.episode_peak = ctx.episode_peak.max(abs_v);

    latest[chan_idx] = s.value;
    let multi_dt = dt.min(1.0);
    debug_assert!(multi_dt.is_finite(), "multi_dt must be finite");
    let _multi_state = multi_observer.step(latest, multi_dt);

    let episode_peak_ref = &mut ctx.episode_peak;
    let last_ema_snapshot = ctx.last_ema;
    state.advance(
        env,
        s.t,
        params.min_dwell_seconds,
        &mut |t_start, t_end| {
            let trust_sum = trust_sum_across(multi_observer, channels);
            episodes.push(Episode {
                motif: class,
                channel: Some(channel.to_string()),
                t_start,
                t_end,
                peak: *episode_peak_ref,
                ema_at_boundary: last_ema_snapshot,
                trust_sum,
            });
            *episode_peak_ref = 0.0;
        },
        &mut ctx.episode_t_start,
    )
}

/// End-of-stream flush: an episode that opened but whose recovery
/// crossed the trace boundary is still a real detection. Guard against
/// spurious single-sample opens by requiring the open window to span
/// at least `min_dwell_seconds`.
#[allow(clippy::too_many_arguments)]
fn flush_open_episode(
    class: MotifClass,
    params: &MotifParams,
    channel: &str,
    channels: &[String],
    multi_observer: &DsfbObserver,
    ctx: &ChannelCtx,
    state: MotifState,
    episodes: &mut Vec<Episode>,
) {
    let t_open = match state {
        MotifState::InEpisode { t_open } | MotifState::Recovering { t_open, .. } => t_open,
        MotifState::Stable => return,
    };
    let duration = ctx.last_t - t_open;
    debug_assert!(
        duration >= 0.0,
        "duration is non-negative: samples are time-ordered"
    );
    let sustained = duration >= params.min_dwell_seconds;
    if !sustained {
        return;
    }
    let trust_sum = trust_sum_across(multi_observer, channels);
    debug_assert!(trust_sum.is_finite(), "trust_sum is finite");
    episodes.push(Episode {
        motif: class,
        channel: Some(channel.to_string()),
        t_start: t_open,
        t_end: ctx.last_t,
        peak: ctx.episode_peak,
        ema_at_boundary: ctx.last_ema,
        trust_sum,
    });
}

/// Sum trust-weights across all active channels. The DSFB observer
/// guarantees this sums to ≈1.0; the invariant is asserted by
/// `test_trust_sum_invariant`.
fn trust_sum_across(multi_observer: &DsfbObserver, channels: &[String]) -> f64 {
    let n = channels.len().max(1);
    debug_assert!(n > 0, "at least one slot by construction");
    (0..n).map(|i| multi_observer.trust_weight(i)).sum()
}

#[derive(Debug, Clone, Copy)]
enum MotifState {
    Stable,
    InEpisode { t_open: f64 },
    Recovering { t_open: f64, t_recover_start: f64 },
}

impl MotifState {
    fn advance<F: FnMut(f64, f64)>(
        self,
        env: Envelope,
        t: f64,
        min_dwell: f64,
        emit: &mut F,
        episode_t_start: &mut f64,
    ) -> MotifState {
        match (self, env) {
            (MotifState::Stable, Envelope::Stable) => MotifState::Stable,
            (MotifState::Stable, Envelope::Drift | Envelope::Boundary) => {
                *episode_t_start = t;
                MotifState::InEpisode { t_open: t }
            }
            (MotifState::InEpisode { t_open }, Envelope::Stable) => MotifState::Recovering {
                t_open,
                t_recover_start: t,
            },
            (MotifState::InEpisode { t_open }, _) => MotifState::InEpisode { t_open },
            (
                MotifState::Recovering {
                    t_open,
                    t_recover_start,
                },
                Envelope::Stable,
            ) => {
                if t - t_recover_start >= min_dwell {
                    emit(t_open, t);
                    MotifState::Stable
                } else {
                    MotifState::Recovering {
                        t_open,
                        t_recover_start,
                    }
                }
            }
            (MotifState::Recovering { t_open, .. }, _) => MotifState::InEpisode { t_open },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::ResidualClass;

    #[test]
    fn no_episodes_on_quiet_stream() {
        let mut stream = ResidualStream::new("test");
        for i in 0..100 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01).with_channel("q1"),
            );
        }
        let p = MotifParams::default_for(MotifClass::CardinalityMismatchRegime);
        let eps = run_motif(MotifClass::CardinalityMismatchRegime, &p, &stream);
        assert!(eps.is_empty(), "quiet stream should produce no episodes");
    }

    #[test]
    fn opens_episode_on_sustained_drift() {
        let mut stream = ResidualStream::new("test");
        // 10 quiet samples, 50 drifty samples, 10 quiet samples
        for i in 0..10 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01).with_channel("q1"),
            );
        }
        for i in 10..60 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 1.5).with_channel("q1"),
            );
        }
        for i in 60..70 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01).with_channel("q1"),
            );
        }
        let p = MotifParams::default_for(MotifClass::CardinalityMismatchRegime);
        let eps = run_motif(MotifClass::CardinalityMismatchRegime, &p, &stream);
        assert!(
            !eps.is_empty(),
            "sustained drift should produce at least one episode"
        );
    }
}
