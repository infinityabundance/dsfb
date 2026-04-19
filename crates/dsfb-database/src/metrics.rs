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
    debug_assert!(
        trace_duration_s.is_finite(),
        "trace_duration_s must be finite"
    );
    debug_assert!(
        trace_duration_s >= 0.0,
        "trace_duration_s must be non-negative"
    );

    let mut out = Vec::with_capacity(MotifClass::ALL.len());
    for motif in MotifClass::ALL {
        let class_eps: Vec<&Episode> = episodes.iter().filter(|e| e.motif == motif).collect();
        let class_wins: Vec<&PerturbationWindow> = windows
            .iter()
            .filter(|w| perturbation_to_motif(w.class) == motif)
            .collect();

        let (tp, fp, fn_, ttds) = count_confusion(&class_eps, &class_wins);
        debug_assert_eq!(
            fn_ as usize + tp as usize,
            class_wins.len(),
            "every window must be either matched (tp) or missed (fn)"
        );

        let (precision, recall, f1) = precision_recall_f1(tp, fp, fn_);
        debug_assert!(
            (0.0..=1.0).contains(&precision),
            "precision must be in [0,1]"
        );
        debug_assert!((0.0..=1.0).contains(&recall), "recall must be in [0,1]");
        debug_assert!((0.0..=1.0).contains(&f1), "f1 must be in [0,1]");

        let (median, p95) = ttd_percentiles(ttds);
        let far = false_alarm_rate_per_hour(&class_eps, windows, trace_duration_s);
        let compression = compression_ratio(
            &class_eps,
            total_residual_samples_per_motif
                .get(&motif)
                .copied()
                .unwrap_or(0),
        );

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
    debug_assert_eq!(
        out.len(),
        MotifClass::ALL.len(),
        "one row per motif is the invariant relied on by the report layer"
    );
    out
}

/// One episode overlaps a perturbation window iff their intervals
/// overlap **and** the episode channel prefix/contains the window
/// channel (operator channels may be coarser than motif channels).
fn episode_matches_window(ep: &Episode, w: &PerturbationWindow) -> bool {
    let overlap = ep.t_end >= w.t_start && ep.t_start <= w.t_end;
    let chan_ok = ep
        .channel
        .as_deref()
        .map(|c| c.starts_with(&w.channel) || c.contains(&w.channel))
        .unwrap_or(true);
    overlap && chan_ok
}

/// Walk the episode / window lists and compute TP / FP / FN counts plus
/// the set of time-to-detection measurements (seconds from window open
/// to episode open for matched windows). Implements the
/// redundant-overlap clemency rule documented in §8 of the paper.
fn count_confusion(
    class_eps: &[&Episode],
    class_wins: &[&PerturbationWindow],
) -> (u64, u64, u64, Vec<f64>) {
    let mut tp: u64 = 0;
    let mut fp: u64 = 0;
    let mut fn_: u64 = 0;
    let mut ttds: Vec<f64> = Vec::new();
    let mut matched_windows = vec![false; class_wins.len()];

    for ep in class_eps {
        let hit = class_wins
            .iter()
            .enumerate()
            .find(|(wi, w)| !matched_windows[*wi] && episode_matches_window(ep, w))
            .map(|(wi, _)| wi);
        if let Some(wi) = hit {
            matched_windows[wi] = true;
            tp += 1;
            let w = class_wins[wi];
            ttds.push((ep.t_start - w.t_start).max(0.0));
        } else if !class_wins.iter().any(|w| episode_matches_window(ep, w)) {
            // Redundant-overlap clemency: count FP only when the
            // episode overlaps *no* window of the right class on a
            // related channel (redundant co-located detections are
            // not false alarms per §8).
            fp += 1;
        }
    }
    for matched in &matched_windows {
        if !matched {
            fn_ += 1;
        }
    }
    (tp, fp, fn_, ttds)
}

/// Canonical precision / recall / F1 definitions with zero-guards. At
/// the boundary (tp+fp=0 or tp+fn=0) the score collapses to 0 — that
/// is the §8 convention rather than NaN.
fn precision_recall_f1(tp: u64, fp: u64, fn_: u64) -> (f64, f64, f64) {
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
    (precision, recall, f1)
}

/// Median and 95th-percentile TTD from a TTD list. Empty lists produce
/// (0.0, 0.0) by §8 convention.
fn ttd_percentiles(mut ttds: Vec<f64>) -> (f64, f64) {
    if ttds.is_empty() {
        return (0.0, 0.0);
    }
    ttds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = ttds[ttds.len() / 2];
    let idx = ((ttds.len() as f64 - 1.0) * 0.95).round() as usize;
    let p95 = ttds[idx.min(ttds.len() - 1)];
    debug_assert!(
        median.is_finite() && p95.is_finite(),
        "percentiles finite on finite input"
    );
    (median, p95)
}

/// False-alarm rate in episodes-per-stable-hour. "Stable" = trace time
/// outside *any* perturbation window of *any* class.
fn false_alarm_rate_per_hour(
    class_eps: &[&Episode],
    windows: &[PerturbationWindow],
    trace_duration_s: f64,
) -> f64 {
    let stable_eps: u64 = class_eps
        .iter()
        .filter(|ep| {
            !windows
                .iter()
                .any(|w| ep.t_end >= w.t_start && ep.t_start <= w.t_end)
        })
        .count() as u64;
    let stable_hours =
        (trace_duration_s - windows.iter().map(|w| w.t_end - w.t_start).sum::<f64>()).max(1.0)
            / 3600.0;
    debug_assert!(stable_hours > 0.0, "stable_hours lower-clamped to 1s/3600");
    stable_eps as f64 / stable_hours
}

/// Episode-compression ratio: input residual samples per emitted
/// episode. Zero-episode streams report zero (matches §8's
/// "no episodes ⇒ no compression report" convention).
fn compression_ratio(class_eps: &[&Episode], total_samples: usize) -> f64 {
    if class_eps.is_empty() {
        0.0
    } else {
        total_samples as f64 / class_eps.len() as f64
    }
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

/// Cross-signal agreement per episode and per-motif mean.
///
/// For each emitted episode `E` of motif class `c` on channel `ch` over
/// `[t_s, t_e]`, define cross-signal agreement as the fraction of *other*
/// motif classes `c' ≠ c` that also emit at least one episode overlapping
/// `[t_s, t_e]` on any channel. Range `[0, 1]`; zero = isolated signal,
/// one = full grammar coverage.
///
/// This is **not** a ground-truth correlation and **not** a causal score.
/// High agreement can arise from a common exogenous stressor *or* from
/// the grammar's overlapping sensitivity; low agreement is not evidence
/// of isolation either. It is a structural co-occurrence measurement
/// whose sole purpose is to quantify the multi-signal coupling that
/// distinguishes DSFB episodes from per-series change-point reports.
///
/// See paper §7 for the matching limitation statement.
pub fn cross_signal_agreement(episodes: &[Episode]) -> Vec<(MotifClass, f64)> {
    let mut per_motif: Vec<(MotifClass, Vec<f64>)> =
        MotifClass::ALL.iter().map(|m| (*m, Vec::new())).collect();
    for ep in episodes {
        let other_classes_with_overlap = MotifClass::ALL
            .iter()
            .filter(|m| **m != ep.motif)
            .filter(|m| {
                episodes
                    .iter()
                    .any(|e| e.motif == **m && e.t_end >= ep.t_start && e.t_start <= ep.t_end)
            })
            .count();
        let agreement = other_classes_with_overlap as f64 / (MotifClass::ALL.len() - 1) as f64;
        debug_assert!(
            (0.0..=1.0).contains(&agreement),
            "cross-signal agreement must be in [0,1]"
        );
        per_motif
            .iter_mut()
            .find(|(m, _)| *m == ep.motif)
            .map(|(_, v)| v.push(agreement));
    }
    per_motif
        .into_iter()
        .map(|(m, vs)| {
            let mean = if vs.is_empty() {
                0.0
            } else {
                vs.iter().sum::<f64>() / vs.len() as f64
            };
            (m, mean)
        })
        .collect()
}

/// Stability-under-perturbation scalar metric.
///
/// Given per-(motif, scale) F1 rows from the stress sweep, compute the
/// normalised area-under-the-F1 curve over scales in the operating
/// envelope window `[0.5, 1.5]` (around the canonical scale of 1.0). The
/// scalar collapses the F1-vs-scale curve to a single number in `[0, 1]`
/// that answers "how robust is this motif's detection across a ±50%
/// perturbation-magnitude window?" — higher is more stable.
///
/// Trapezoidal integration on the subset of provided scales that fall in
/// the window; normalisation divides by the window width so the result
/// is the mean F1 across the interval. Motifs with no samples in the
/// window return 0.0.
///
/// This is a re-read of the existing `tpcds.stress.csv` data, not a new
/// experiment. See paper §6 (operating envelope) for the companion
/// narrative.
pub fn stability_under_perturbation(
    stress_rows: &[(f64, String, f64)],
) -> std::collections::HashMap<String, f64> {
    let (lo, hi) = (0.5_f64, 1.5_f64);
    let mut by_motif: std::collections::BTreeMap<String, Vec<(f64, f64)>> =
        std::collections::BTreeMap::new();
    for (scale, motif, f1) in stress_rows {
        if *scale >= lo && *scale <= hi && f1.is_finite() {
            by_motif
                .entry(motif.clone())
                .or_default()
                .push((*scale, *f1));
        }
    }
    let mut out = std::collections::HashMap::new();
    for (motif, mut pts) in by_motif {
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        if pts.len() < 2 {
            out.insert(motif, 0.0);
            continue;
        }
        let mut auc = 0.0_f64;
        for pair in pts.windows(2) {
            let (x0, y0) = pair[0];
            let (x1, y1) = pair[1];
            auc += 0.5 * (y0 + y1) * (x1 - x0);
        }
        let width = pts.last().unwrap().0 - pts.first().unwrap().0;
        let norm = if width > 0.0 { auc / width } else { 0.0 };
        debug_assert!(norm.is_finite(), "stability AUC finite");
        out.insert(motif, norm.clamp(0.0, 1.0));
    }
    out
}
