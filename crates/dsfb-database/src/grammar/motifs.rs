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

pub fn run_motif(
    class: MotifClass,
    params: &MotifParams,
    stream: &ResidualStream,
) -> Vec<Episode> {
    let target_class: ResidualClass = class.residual_class();
    let dsfb_params = DsfbParams::new(
        DEFAULT_K_PHI,
        DEFAULT_K_OMEGA,
        DEFAULT_K_ALPHA,
        params.rho,
        params.sigma0,
    );

    // Group samples by channel; channels are operator-meaningful (qclass,
    // wait_event, cache_id, bucket_id, …).
    let mut by_channel: BTreeMap<String, Vec<&ResidualSample>> = BTreeMap::new();
    for s in stream.iter_class(target_class) {
        let ch = s.channel.clone().unwrap_or_else(|| "_anonymous_".into());
        by_channel.entry(ch).or_default().push(s);
    }

    // For trust-sum traceability we run a single multi-channel observer
    // across the union of channels at each timestep. To keep this cheap we
    // sample at the most recent residual per channel.
    let channels: Vec<String> = by_channel.keys().cloned().collect();
    let mut multi_observer = DsfbObserver::new(dsfb_params, channels.len().max(1));
    let mut latest: Vec<f64> = vec![0.0; channels.len().max(1)];

    let mut episodes = Vec::new();
    for (chan_idx, (channel, samples)) in by_channel.iter().enumerate() {
        if samples.is_empty() {
            continue;
        }
        let mut obs = DsfbObserver::new(dsfb_params, 1);
        let mut state = MotifState::Stable;
        let mut episode_t_start: f64 = 0.0;
        let mut episode_peak: f64 = 0.0;
        let mut last_ema: f64 = 0.0;
        let mut last_t: f64 = samples[0].t;

        for s in samples {
            // Cap dt at 1.0: our measurements are *already residuals*
            // (delta from baseline), so the dsfb predict step
            // `phi_pred = phi + omega*dt` adds no information when
            // samples are sparse — and with dt of tens of seconds it
            // makes phi_pred run away from a stationary residual,
            // generating large spurious residuals on every channel.
            // Capping dt keeps EMA smoothing meaningful while making
            // the motif loop sample-rate-invariant, which is what an
            // operator on `pg_stat_statements`-style telemetry needs.
            let dt = (s.t - last_t).clamp(1e-6, 1.0);
            let _ = obs.step(&[s.value], dt);
            let ema = obs.ema_residual(0);
            let env = classify(
                ema,
                s.value,
                params.drift_threshold,
                params.slew_threshold,
            );
            last_ema = ema;
            last_t = s.t;
            let abs_v: f64 = s.value.abs();
            episode_peak = episode_peak.max(abs_v);

            // multi-channel observer trace: only update its slot for this channel
            // — keeps trust-weights coherent across all channels. Same
            // dt cap as above to keep the multi-observer well-behaved.
            latest[chan_idx] = s.value;
            let multi_dt = dt.min(1.0);
            let _ = multi_observer.step(&latest, multi_dt);

            state = state.advance(env, s.t, params.min_dwell_seconds, &mut |t_start, t_end| {
                let trust_sum: f64 = (0..channels.len().max(1))
                    .map(|i| multi_observer.trust_weight(i))
                    .sum();
                episodes.push(Episode {
                    motif: class,
                    channel: Some(channel.clone()),
                    t_start,
                    t_end,
                    peak: episode_peak,
                    ema_at_boundary: last_ema,
                    trust_sum,
                });
                episode_peak = 0.0;
            }, &mut episode_t_start);
        }

        // End-of-stream flush: an episode that opened but whose recovery
        // crossed the trace boundary is still a real detection. Guard
        // against spurious single-sample opens by requiring the open
        // window to span at least `min_dwell_seconds` AND the EMA at
        // boundary to remain above the drift threshold — this is the
        // structural counterpart to the in-loop close condition and
        // keeps the false-positive rate bounded.
        if let MotifState::InEpisode { t_open } | MotifState::Recovering { t_open, .. } = state {
            let duration = last_t - t_open;
            // Flush any open episode whose t_open span is at least one
            // dwell wide. The EMA may have decayed below the drift
            // threshold by the trace boundary (legitimate recovery), so
            // we do *not* gate on ema_at_boundary — that's the role of
            // the duration check.
            let sustained = duration >= params.min_dwell_seconds;
            if sustained {
                let trust_sum: f64 = (0..channels.len().max(1))
                    .map(|i| multi_observer.trust_weight(i))
                    .sum();
                episodes.push(Episode {
                    motif: class,
                    channel: Some(channel.clone()),
                    t_start: t_open,
                    t_end: last_t,
                    peak: episode_peak,
                    ema_at_boundary: last_ema,
                    trust_sum,
                });
            }
        }
    }
    episodes
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
            (MotifState::InEpisode { t_open }, Envelope::Stable) => {
                MotifState::Recovering {
                    t_open,
                    t_recover_start: t,
                }
            }
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
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01)
                    .with_channel("q1"),
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
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01)
                    .with_channel("q1"),
            );
        }
        for i in 10..60 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 1.5)
                    .with_channel("q1"),
            );
        }
        for i in 60..70 {
            stream.push(
                ResidualSample::new(i as f64, ResidualClass::Cardinality, 0.01)
                    .with_channel("q1"),
            );
        }
        let p = MotifParams::default_for(MotifClass::CardinalityMismatchRegime);
        let eps = run_motif(MotifClass::CardinalityMismatchRegime, &p, &stream);
        assert!(!eps.is_empty(), "sustained drift should produce at least one episode");
    }
}
