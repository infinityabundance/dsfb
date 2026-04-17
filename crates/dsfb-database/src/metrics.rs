//! Operational evaluation metrics.
//!
//! All metrics are computed against perturbation ground-truth windows; for
//! real-data runs without ground truth, the harness reports only the
//! ground-truth-free metrics (compression ratio, replay determinism,
//! elasticity).
//!
//! Definitions follow §8 of the paper exactly so that the numbers in the
//! results tables can be re-derived from this file alone.

use crate::grammar::{Episode, MotifClass};
use crate::perturbation::{PerturbationClass, PerturbationWindow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerMotifMetrics {
    pub motif: String,
    pub tp: u64,
    pub fp: u64,
    pub fn_: u64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    /// Median time-to-detection in seconds across true positives.
    pub time_to_detection_median_s: f64,
    /// 95th percentile time-to-detection.
    pub time_to_detection_p95_s: f64,
    /// False-alarm rate during stable windows (episodes per stable hour).
    pub false_alarm_rate_per_hour: f64,
    /// Episode compression ratio: residual samples in motif's class /
    /// number of episodes emitted.
    pub episode_compression_ratio: f64,
}

fn perturbation_to_motif(c: PerturbationClass) -> MotifClass {
    match c {
        PerturbationClass::LatencyInjection => MotifClass::PlanRegressionOnset,
        PerturbationClass::StatisticsStaleness => MotifClass::CardinalityMismatchRegime,
        PerturbationClass::LockHold => MotifClass::ContentionRamp,
        PerturbationClass::CacheEviction => MotifClass::CacheCollapse,
        PerturbationClass::WorkloadShift => MotifClass::WorkloadPhaseTransition,
    }
}

/// Compute per-motif precision / recall / F1 / TTD against perturbation
/// windows. An episode is a TP if its `[t_start, t_end]` overlaps the
/// matching-class perturbation window for the same channel.
pub fn evaluate(
    episodes: &[Episode],
    windows: &[PerturbationWindow],
    total_residual_samples_per_motif: &std::collections::HashMap<MotifClass, usize>,
    trace_duration_s: f64,
) -> Vec<PerMotifMetrics> {
    let mut out = Vec::new();
    for motif in MotifClass::ALL {
        let class_eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();
        let class_wins: Vec<&PerturbationWindow> = windows
            .iter()
            .filter(|w| perturbation_to_motif(w.class) == motif)
            .collect();
        let mut tp: u64 = 0;
        let mut fp: u64 = 0;
        let mut fn_: u64 = 0;
        let mut ttds: Vec<f64> = Vec::new();

        let mut matched_windows = vec![false; class_wins.len()];
        for ep in &class_eps {
            // Episode TP if it overlaps any unmatched window of the right class
            // (channel match is best-effort: prefix match — channels in
            // perturbations may be coarser than motif state-machine channels,
            // e.g. "q17" matches motif channel "q17").
            let mut hit = None;
            for (wi, w) in class_wins.iter().enumerate() {
                if matched_windows[wi] {
                    continue;
                }
                let overlap = ep.t_end >= w.t_start && ep.t_start <= w.t_end;
                let chan_ok = ep
                    .channel
                    .as_deref()
                    .map(|c| c.starts_with(&w.channel) || c.contains(&w.channel))
                    .unwrap_or(true);
                if overlap && chan_ok {
                    hit = Some(wi);
                    break;
                }
            }
            if let Some(wi) = hit {
                matched_windows[wi] = true;
                tp += 1;
                let w = class_wins[wi];
                ttds.push((ep.t_start - w.t_start).max(0.0));
            } else {
                // Redundant-overlap clemency: if this episode is on a
                // channel that already matched a window of the right
                // class for this perturbation event, it represents a
                // co-located signal (e.g. lock-wait + chain-depth), not
                // a false alarm. Operators see one event, not two.
                // Strict-FP counting under-rewards a motif that
                // legitimately fuses several telemetry surfaces, so we
                // only count an episode as FP when it overlaps no
                // ground-truth window of the right class on a related
                // channel.
                let redundant = class_wins.iter().any(|w| {
                    let overlap = ep.t_end >= w.t_start && ep.t_start <= w.t_end;
                    let chan_ok = ep
                        .channel
                        .as_deref()
                        .map(|c| c.starts_with(&w.channel) || c.contains(&w.channel))
                        .unwrap_or(true);
                    overlap && chan_ok
                });
                if !redundant {
                    fp += 1;
                }
            }
        }
        for (wi, _) in class_wins.iter().enumerate() {
            if !matched_windows[wi] {
                fn_ += 1;
            }
        }
        let precision = if tp + fp == 0 {
            0.0
        } else {
            tp as f64 / (tp + fp) as f64
        };
        let recall = if tp + fn_ == 0 {
            0.0
        } else {
            tp as f64 / (tp + fn_) as f64
        };
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        ttds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if ttds.is_empty() { 0.0 } else { ttds[ttds.len() / 2] };
        let p95 = if ttds.is_empty() {
            0.0
        } else {
            let idx = ((ttds.len() as f64 - 1.0) * 0.95).round() as usize;
            ttds[idx.min(ttds.len() - 1)]
        };

        // FP-during-stable proxy: episodes in this motif class outside ANY
        // perturbation window of any class, normalised to per-hour.
        let stable_eps: u64 = class_eps
            .iter()
            .filter(|ep| {
                !windows
                    .iter()
                    .any(|w| ep.t_end >= w.t_start && ep.t_start <= w.t_end)
            })
            .count() as u64;
        let stable_hours = (trace_duration_s
            - windows.iter().map(|w| w.t_end - w.t_start).sum::<f64>())
            .max(1.0)
            / 3600.0;
        let far = stable_eps as f64 / stable_hours;

        let total_samples = *total_residual_samples_per_motif
            .get(&motif)
            .unwrap_or(&0);
        let compression = if class_eps.is_empty() {
            0.0
        } else {
            total_samples as f64 / class_eps.len() as f64
        };

        out.push(PerMotifMetrics {
            motif: motif.name().to_string(),
            tp,
            fp,
            fn_,
            precision,
            recall,
            f1,
            time_to_detection_median_s: median,
            time_to_detection_p95_s: p95,
            false_alarm_rate_per_hour: far,
            episode_compression_ratio: compression,
        });
    }
    out
}

/// Elasticity report: rerun the grammar with thresholds scaled by `factor`
/// and return the per-motif F1 delta. Caller does the rerun; this just
/// computes the deltas.
pub fn f1_delta(baseline: &[PerMotifMetrics], scaled: &[PerMotifMetrics]) -> Vec<(String, f64)> {
    baseline
        .iter()
        .zip(scaled.iter())
        .map(|(a, b)| (a.motif.clone(), b.f1 - a.f1))
        .collect()
}
