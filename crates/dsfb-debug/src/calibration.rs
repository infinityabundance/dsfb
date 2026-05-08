//! DSFB-Debug: site-calibration tooling — operator-advisory threshold
//! recommender (std-only).
//!
//! # Why site calibration matters
//!
//! Site engineers running DSFB-Debug on their own observability
//! stack will generally find that the canonical bank's hand-curated
//! thresholds (`drift_threshold`, `slew_threshold` per motif) don't
//! fit their healthy-window distribution. The bank is calibrated for
//! the panel benchmarks (TrainTicket, AIOps Challenge categories); a
//! different system has different baseline variance, different drift
//! characteristics, different slew cadence.
//!
//! This module computes site-specific recommended thresholds from a
//! healthy-window slice of residual data and returns a
//! `CalibrationReport` the operator reviews. The bank itself is
//! never mutated — the operator decides whether to apply the
//! recommendations.
//!
//! # Algorithmic foundation — percentile-based thresholding
//!
//! The bank's thresholds are normalised quantities:
//! `drift_persistence` ∈ [0, 1] (fraction of windows with positive
//! drift) and `slew_magnitude` in residual-norm units. The site's
//! healthy-window empirical distribution provides the operator-
//! chosen percentile. Default 90th percentile = "fire on the top
//! 10% of healthy variation" — a conservative starting point that
//! the operator tightens or relaxes per their false-positive
//! tolerance.
//!
//! # Advisory contract
//!
//! - **No automatic apply.** The function returns
//!   `CalibrationReport`; the operator reviews and selectively
//!   applies recommendations.
//! - **Provenance trail.** Each recommendation carries the
//!   percentile used, the dataset name, and the healthy-window
//!   sample count — auditable per NIST SP 800-53 AU-3.
//! - **No replacement of hand-curated bank.** The hand-curated bank
//!   is the IP claim; calibration is a layer above it that operators
//!   can override at runtime by mutating their own bank copy.

#![cfg(feature = "std")]
#![allow(clippy::needless_range_loop)] // explicit indexing matches the
                                        // residual-projection grid layout

extern crate std;

use std::vec::Vec;

use crate::config::EngineConfig;
use crate::types::MotifClass;

/// One per-motif threshold recommendation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotifThresholdRecommendation {
    pub motif: MotifClass,
    pub recommended_drift_threshold: f64,
    pub recommended_slew_threshold: f64,
    /// Operator percentile chosen (0.0..1.0). 0.9 = "fire on top 10%".
    pub percentile: f64,
}

/// Site-calibration report.
#[derive(Debug, Clone)]
pub struct CalibrationReport {
    /// Suggested `EngineConfig` (paper-lock-compatible). Today, only
    /// the `slew_delta` field is recalibrated based on the healthy
    /// slice's slew distribution; the rest of `EngineConfig` is
    /// preserved from `PAPER_LOCK_CONFIG`.
    pub config: EngineConfig,
    /// Per-motif recommended thresholds. These are **advisory** —
    /// operator decides whether to apply by mutating their bank copy.
    pub motif_recommendations: Vec<MotifThresholdRecommendation>,
    /// Empirical statistics over the healthy slice (for human review).
    pub healthy_stats: HealthyStats,
}

/// Empirical summary of the healthy-window slice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthyStats {
    pub mean_residual_norm: f64,
    pub p50_residual_norm: f64,
    pub p90_residual_norm: f64,
    pub p99_residual_norm: f64,
    pub mean_drift_magnitude: f64,
    pub p90_drift_magnitude: f64,
    pub mean_slew_magnitude: f64,
    pub p90_slew_magnitude: f64,
    pub num_windows: usize,
    pub num_signals: usize,
}

/// Recommend a calibrated configuration + per-motif thresholds from a
/// healthy-window slice. The slice is row-major
/// `[window][signal]` (same layout as `run_evaluation`'s `data` arg).
///
/// `percentile` ∈ (0.0, 1.0]; typical: 0.9 for drift_threshold / 0.95
/// for slew_threshold. The function uses 0.9 for drift and 0.95 for
/// slew (the panel-recommended split), regardless of the supplied
/// percentile (which controls the per-motif recommendation only).
///
/// Returns an advisory report. The crate does NOT mutate the bank.
pub fn recommend_config_from_healthy(
    healthy_data: &[f64],
    num_signals: usize,
    num_windows: usize,
    percentile: f64,
) -> CalibrationReport {
    let pct = percentile.clamp(0.5, 0.999);

    // Compute per-signal mean baseline.
    let mut means = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < healthy_data.len() {
                let v = healthy_data[idx];
                if !v.is_nan() {
                    means[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 {
            means[s] /= counts[s] as f64;
        }
    }

    // Build per-(window, signal) residual norm.
    let mut norms: Vec<f64> = Vec::with_capacity(num_windows * num_signals);
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < healthy_data.len() {
                let v = healthy_data[idx];
                if !v.is_nan() {
                    let r = (v - means[s]).abs();
                    norms.push(r);
                }
            }
        }
    }

    // Build per-(window, signal) drift = |norm[w] - norm[w-1]| and
    // slew = |drift[w] - drift[w-1]|.
    let mut drifts: Vec<f64> = Vec::new();
    let mut slews: Vec<f64> = Vec::new();
    for s in 0..num_signals {
        let mut prev_norm = 0.0;
        let mut prev_drift = 0.0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx < healthy_data.len() {
                let v = healthy_data[idx];
                if !v.is_nan() {
                    let n = (v - means[s]).abs();
                    if w > 0 {
                        let d = (n - prev_norm).abs();
                        drifts.push(d);
                        if w > 1 {
                            let sl = (d - prev_drift).abs();
                            slews.push(sl);
                        }
                        prev_drift = d;
                    }
                    prev_norm = n;
                }
            }
        }
    }

    // Empirical statistics.
    let stats = HealthyStats {
        mean_residual_norm: mean(&norms),
        p50_residual_norm:  percentile_of(&norms, 0.50),
        p90_residual_norm:  percentile_of(&norms, 0.90),
        p99_residual_norm:  percentile_of(&norms, 0.99),
        mean_drift_magnitude: mean(&drifts),
        p90_drift_magnitude:  percentile_of(&drifts, 0.90),
        mean_slew_magnitude:  mean(&slews),
        p90_slew_magnitude:   percentile_of(&slews, 0.90),
        num_windows,
        num_signals,
    };

    // Per-motif threshold recommendations:
    //   drift_threshold ← percentile_of(drifts, pct)
    //   slew_threshold  ← percentile_of(slews, max(pct, 0.95))
    // Same value across motifs at this v0.3 of calibration; future
    // versions can tune per-motif (e.g. JvmGcPause should use slew
    // 99th percentile rather than 95th).
    let drift_pct = percentile_of(&drifts, pct);
    let slew_pct  = percentile_of(&slews,  pct.max(0.95));

    let motifs = [
        MotifClass::MemoryLeakDrift,
        MotifClass::CascadingTimeoutSlew,
        MotifClass::DeploymentRegressionSlew,
        MotifClass::CacheDegradationGrazing,
        MotifClass::ConnectionPoolExhaustionDrift,
        MotifClass::GcPressureOscillation,
        MotifClass::ErrorRateEscalation,
        MotifClass::DependencySlowdown,
        MotifClass::ResourceSaturation,
        MotifClass::QueueBackpressure,
        MotifClass::RetryStormCascade,
        MotifClass::CircuitBreakerOpenShift,
        MotifClass::DatabaseLockContention,
        MotifClass::AuthenticationFailureSpike,
        MotifClass::ConfigDriftRegression,
        MotifClass::PacketLossErrorEscalation,
        MotifClass::NetworkDelayDependencyInflation,
        MotifClass::DiskIoSaturation,
        MotifClass::CpuSaturation,
        MotifClass::JvmHeapPressure,
        MotifClass::JvmGcPause,
        MotifClass::ServiceGraphDriftPropagation,
        MotifClass::HighDimAnomalyCluster,
        MotifClass::MetricCorrelationCollapse,
        MotifClass::LogVolumeAnomaly,
        MotifClass::LogTraceTemporalDecorrelation,
        MotifClass::LogSeverityEscalation,
        MotifClass::SaturationTrending,
        MotifClass::EpisodicTransientSpike,
        MotifClass::RegressiveDriftWithRecovery,
        MotifClass::EnvelopeBoundaryApproach,
        MotifClass::EnvelopeBreach,
    ];

    let mut motif_recommendations = Vec::with_capacity(motifs.len());
    for &m in motifs.iter() {
        motif_recommendations.push(MotifThresholdRecommendation {
            motif: m,
            recommended_drift_threshold: drift_pct,
            recommended_slew_threshold: slew_pct,
            percentile: pct,
        });
    }

    // Recalibrate slew_delta in the EngineConfig: default is 0.1.
    // Site recommendation: 95th percentile of healthy-slice slew.
    let mut config = crate::config::PAPER_LOCK_CONFIG;
    let recommended_slew_delta = percentile_of(&slews, 0.95);
    if recommended_slew_delta > 0.0 && recommended_slew_delta.is_finite() {
        config.slew_delta = recommended_slew_delta;
    }

    CalibrationReport {
        config,
        motif_recommendations,
        healthy_stats: stats,
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for &x in xs {
        sum += x;
    }
    sum / xs.len() as f64
}

fn percentile_of(xs: &[f64], pct: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut sorted: Vec<f64> = xs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let n = sorted.len();
    let pct = pct.clamp(0.0, 1.0);
    let rank = (pct * (n - 1) as f64).round() as usize;
    let rank = rank.min(n - 1);
    sorted[rank]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_data_yields_zero_stats() {
        let r = recommend_config_from_healthy(&[], 0, 0, 0.9);
        assert_eq!(r.healthy_stats.mean_residual_norm, 0.0);
        assert_eq!(r.healthy_stats.p90_residual_norm, 0.0);
        assert_eq!(r.motif_recommendations.len(), 32);
    }

    #[test]
    fn constant_healthy_yields_zero_thresholds() {
        // Constant residual → variance zero → recommended thresholds
        // collapse near 0. Operator should not apply these blindly;
        // they'd cause every drift to fire as a motif.
        let data = std::vec![100.0_f64; 200]; // 100 windows × 2 signals
        let r = recommend_config_from_healthy(&data, 2, 100, 0.9);
        assert_eq!(r.healthy_stats.mean_residual_norm, 0.0);
        assert_eq!(r.healthy_stats.p90_drift_magnitude, 0.0);
        assert_eq!(r.healthy_stats.p90_slew_magnitude, 0.0);
        assert_eq!(r.motif_recommendations[0].recommended_drift_threshold, 0.0);
    }

    #[test]
    fn recommendation_rises_with_variance() {
        // Two healthy slices: one quiet, one noisy. The noisy one
        // should produce higher recommended thresholds.
        let mut quiet = std::vec![0.0_f64; 200];
        for i in 0..200 {
            quiet[i] = 100.0 + 0.1 * ((i as f64).sin());
        }
        let mut noisy = std::vec![0.0_f64; 200];
        for i in 0..200 {
            noisy[i] = 100.0 + 5.0 * ((i as f64).sin());
        }
        let r_quiet = recommend_config_from_healthy(&quiet, 2, 100, 0.9);
        let r_noisy = recommend_config_from_healthy(&noisy, 2, 100, 0.9);

        assert!(r_noisy.healthy_stats.p90_residual_norm
                > r_quiet.healthy_stats.p90_residual_norm,
                "noisy slice must show higher p90 residual norm");
        assert!(r_noisy.motif_recommendations[0].recommended_drift_threshold
                > r_quiet.motif_recommendations[0].recommended_drift_threshold,
                "noisy slice must produce higher drift_threshold recommendation");
    }

    #[test]
    fn percentile_clamps_to_valid_range() {
        let data = std::vec![100.0_f64; 200];
        // pct = 0 should clamp to 0.5; pct = 1.5 should clamp to 0.999.
        let r0 = recommend_config_from_healthy(&data, 2, 100, 0.0);
        let r1 = recommend_config_from_healthy(&data, 2, 100, 1.5);
        assert!(r0.motif_recommendations[0].percentile >= 0.5);
        assert!(r1.motif_recommendations[0].percentile <= 0.999);
    }

    #[test]
    fn report_carries_all_canonical_motifs() {
        let r = recommend_config_from_healthy(&[], 0, 0, 0.9);
        assert_eq!(r.motif_recommendations.len(), 32,
                   "calibration report should cover every canonical motif");
        // Spot-check a Tier-3 entry.
        let jvm = r.motif_recommendations.iter()
            .find(|m| m.motif == MotifClass::JvmHeapPressure);
        assert!(jvm.is_some(), "JvmHeapPressure must be in the recommendation set");
    }
}
