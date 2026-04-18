//! Phase-C4: Prometheus / OpenMetrics exposition of live engine state.
//!
//! This module emits the OpenMetrics 1.0 text format a Prometheus
//! scraper will happily consume. It is **intentionally** a pure
//! function over the in-memory episode list and (optionally) the
//! streaming ingestor's counters — no HTTP, no async runtime, no
//! hyper / axum / tokio dependency. The operator runbook (§Phase-C6
//! README) documents three wiring options that cost less than 100
//! lines each: plain `std::net::TcpListener` loop, `tiny_http` crate,
//! or a sidecar `textfile` collector read by `node_exporter`.
//!
//! Metric catalogue (the names follow the Prometheus
//! `NAMESPACE_SUBSYSTEM_UNIT` convention):
//!
//!   * `dsfb_episodes_total{motif="..."}` — counter, total episodes
//!     emitted per motif class since process start.
//!   * `dsfb_episode_peak_last{motif="..."}` — gauge, `|peak|` of the
//!     most recently closed episode per motif. `NaN` when no episode
//!     has closed yet.
//!   * `dsfb_episode_trust_sum_last{motif="..."}` — gauge, trust-sum
//!     observed at the most recent episode boundary. Should stay in
//!     `[0.99, 1.01]`; deviations indicate an observer bug.
//!   * `dsfb_streaming_residuals_staged` — gauge, current reorder
//!     buffer occupancy.
//!   * `dsfb_streaming_residuals_flushed_total` — counter, samples
//!     moved from the reorder buffer into the canonical stream.
//!   * `dsfb_streaming_residuals_dropped_out_of_window_total` —
//!     counter, samples dropped because they arrived more than one
//!     reorder-window behind the flushed frontier. A non-zero value
//!     is an alertable condition (telemetry-pipeline is worse than
//!     the configured window).
//!
//! The text format is deterministic given the same inputs — helpful
//! for unit-testing and for diffing scrapes across deploys.

use crate::grammar::{Episode, MotifClass};
use crate::streaming::StreamingIngestor;
use std::fmt::Write;

/// Snapshot of counters an exporter can publish directly. Constructed
/// from an [`Episode`] slice (typically the output of one
/// `MotifEngine::run` or the accumulated set of closed episodes in a
/// long-running daemon).
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Per-motif total episode count.
    pub per_motif_count: [u64; MotifClass::ALL.len()],
    /// Per-motif last-episode peak `|residual|` (NaN when none).
    pub per_motif_last_peak: [f64; MotifClass::ALL.len()],
    /// Per-motif last-episode trust sum (NaN when none).
    pub per_motif_last_trust_sum: [f64; MotifClass::ALL.len()],
    /// Residuals currently held in a streaming ingestor's buffer.
    pub streaming_staged: u64,
    /// Residuals the streaming ingestor has flushed to its stream.
    pub streaming_flushed: u64,
    /// Residuals the streaming ingestor dropped because they fell
    /// outside the reorder window.
    pub streaming_dropped_out_of_window: u64,
}

impl MetricsSnapshot {
    /// Build a snapshot from an [`Episode`] slice. Order matters only
    /// in that the *last* episode per motif (by insertion order)
    /// determines `last_peak` and `last_trust_sum`.
    pub fn from_episodes(episodes: &[Episode]) -> Self {
        let mut snap = MetricsSnapshot::default();
        for v in snap.per_motif_last_peak.iter_mut() {
            *v = f64::NAN;
        }
        for v in snap.per_motif_last_trust_sum.iter_mut() {
            *v = f64::NAN;
        }
        for e in episodes {
            let idx = motif_index(e.motif);
            snap.per_motif_count[idx] += 1;
            snap.per_motif_last_peak[idx] = e.peak;
            snap.per_motif_last_trust_sum[idx] = e.trust_sum;
        }
        snap
    }

    /// Fold the streaming ingestor's counters into an existing
    /// snapshot. The ingestor is borrowed immutably — callers can
    /// continue to push samples into it after this call.
    pub fn with_streaming(mut self, ing: &StreamingIngestor) -> Self {
        self.streaming_staged = ing.staged() as u64;
        self.streaming_flushed = ing.flushed() as u64;
        self.streaming_dropped_out_of_window = ing.dropped_out_of_window();
        self
    }
}

fn motif_index(m: MotifClass) -> usize {
    // `MotifClass::ALL` is the declared ordering used by the metrics
    // and episode-emission paths. We locate the motif by linear scan
    // — there are five elements, so the cost is negligible and the
    // match stays deny-list-safe under future additions.
    MotifClass::ALL
        .iter()
        .position(|x| *x == m)
        .expect("MotifClass::ALL covers every motif")
}

/// Render a [`MetricsSnapshot`] as OpenMetrics 1.0 text. The output
/// ends with `# EOF\n` as required by the OpenMetrics spec. The
/// function is deterministic: same input → byte-identical output.
pub fn render_openmetrics(snap: &MetricsSnapshot) -> String {
    let mut out = String::with_capacity(2048);
    // Counters
    writeln!(
        out,
        "# HELP dsfb_episodes_total Total motif episodes emitted since process start."
    )
    .unwrap();
    writeln!(out, "# TYPE dsfb_episodes_total counter").unwrap();
    for (i, motif) in MotifClass::ALL.iter().enumerate() {
        writeln!(
            out,
            "dsfb_episodes_total{{motif=\"{}\"}} {}",
            motif.name(),
            snap.per_motif_count[i]
        )
        .unwrap();
    }
    // Last-episode peak (gauge)
    writeln!(
        out,
        "# HELP dsfb_episode_peak_last Peak |residual| of the most recently closed episode per motif."
    )
    .unwrap();
    writeln!(out, "# TYPE dsfb_episode_peak_last gauge").unwrap();
    for (i, motif) in MotifClass::ALL.iter().enumerate() {
        writeln!(
            out,
            "dsfb_episode_peak_last{{motif=\"{}\"}} {}",
            motif.name(),
            fmt_f64(snap.per_motif_last_peak[i])
        )
        .unwrap();
    }
    // Last-episode trust sum (gauge)
    writeln!(
        out,
        "# HELP dsfb_episode_trust_sum_last Trust-sum observed at the most recent episode boundary."
    )
    .unwrap();
    writeln!(out, "# TYPE dsfb_episode_trust_sum_last gauge").unwrap();
    for (i, motif) in MotifClass::ALL.iter().enumerate() {
        writeln!(
            out,
            "dsfb_episode_trust_sum_last{{motif=\"{}\"}} {}",
            motif.name(),
            fmt_f64(snap.per_motif_last_trust_sum[i])
        )
        .unwrap();
    }
    // Streaming counters
    writeln!(
        out,
        "# HELP dsfb_streaming_residuals_staged Residuals currently held in the streaming reorder buffer."
    )
    .unwrap();
    writeln!(out, "# TYPE dsfb_streaming_residuals_staged gauge").unwrap();
    writeln!(
        out,
        "dsfb_streaming_residuals_staged {}",
        snap.streaming_staged
    )
    .unwrap();
    writeln!(
        out,
        "# HELP dsfb_streaming_residuals_flushed_total Residuals moved from the reorder buffer to the canonical stream."
    )
    .unwrap();
    writeln!(out, "# TYPE dsfb_streaming_residuals_flushed_total counter").unwrap();
    writeln!(
        out,
        "dsfb_streaming_residuals_flushed_total {}",
        snap.streaming_flushed
    )
    .unwrap();
    writeln!(
        out,
        "# HELP dsfb_streaming_residuals_dropped_out_of_window_total Residuals dropped because they arrived outside the reorder window."
    )
    .unwrap();
    writeln!(
        out,
        "# TYPE dsfb_streaming_residuals_dropped_out_of_window_total counter"
    )
    .unwrap();
    writeln!(
        out,
        "dsfb_streaming_residuals_dropped_out_of_window_total {}",
        snap.streaming_dropped_out_of_window
    )
    .unwrap();
    out.push_str("# EOF\n");
    out
}

/// Format an `f64` the way OpenMetrics wants: `NaN` stays `NaN`,
/// positive/negative infinity become `+Inf` / `-Inf`, everything else
/// goes through `{:.6}` so the representation is bounded in length
/// and deterministic across platforms.
fn fmt_f64(x: f64) -> String {
    if x.is_nan() {
        "NaN".to_string()
    } else if x.is_infinite() {
        if x > 0.0 {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else {
        format!("{:.6}", x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::Episode;

    fn ep(motif: MotifClass, peak: f64, trust: f64) -> Episode {
        Episode {
            motif,
            channel: None,
            t_start: 0.0,
            t_end: 1.0,
            peak,
            ema_at_boundary: 0.0,
            trust_sum: trust,
        }
    }

    #[test]
    fn empty_snapshot_has_zero_counts_and_nan_peaks() {
        let snap = MetricsSnapshot::from_episodes(&[]);
        for c in snap.per_motif_count {
            assert_eq!(c, 0);
        }
        for p in snap.per_motif_last_peak {
            assert!(p.is_nan());
        }
        let text = render_openmetrics(&snap);
        assert!(text.contains("dsfb_episodes_total{motif=\"plan_regression_onset\"} 0"));
        assert!(text.contains("dsfb_episode_peak_last{motif=\"plan_regression_onset\"} NaN"));
        assert!(text.ends_with("# EOF\n"));
    }

    #[test]
    fn counts_and_last_peak_track_episode_order() {
        let eps = vec![
            ep(MotifClass::PlanRegressionOnset, 0.5, 1.0),
            ep(MotifClass::PlanRegressionOnset, 0.8, 1.0),
            ep(MotifClass::CacheCollapse, 0.3, 1.0),
        ];
        let snap = MetricsSnapshot::from_episodes(&eps);
        let idx_plan = motif_index(MotifClass::PlanRegressionOnset);
        let idx_cache = motif_index(MotifClass::CacheCollapse);
        assert_eq!(snap.per_motif_count[idx_plan], 2);
        assert_eq!(snap.per_motif_count[idx_cache], 1);
        assert!((snap.per_motif_last_peak[idx_plan] - 0.8).abs() < 1e-12);
        assert!((snap.per_motif_last_peak[idx_cache] - 0.3).abs() < 1e-12);
    }

    #[test]
    fn openmetrics_output_is_deterministic() {
        let eps = vec![
            ep(MotifClass::ContentionRamp, 1.2, 1.0),
            ep(MotifClass::WorkloadPhaseTransition, 0.4, 1.0),
        ];
        let snap = MetricsSnapshot::from_episodes(&eps);
        let a = render_openmetrics(&snap);
        let b = render_openmetrics(&snap);
        assert_eq!(a, b);
    }

    #[test]
    fn streaming_fold_propagates_counters() {
        let mut ing = crate::streaming::StreamingIngestor::with_window("t", 1.0);
        ing.push(crate::residual::ResidualSample::new(
            0.0,
            crate::residual::ResidualClass::PlanRegression,
            0.1,
        ));
        let snap = MetricsSnapshot::from_episodes(&[]).with_streaming(&ing);
        let text = render_openmetrics(&snap);
        assert!(text.contains("dsfb_streaming_residuals_staged 1"));
        assert!(text.contains("dsfb_streaming_residuals_dropped_out_of_window_total 0"));
    }
}
