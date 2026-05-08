//! Leave-one-fixture-out cross-validation aggregation (Phase ζ.2).
//!
//! Operates on `LooCvFixtureRecord` values captured per-fixture by
//! the LO-CV test runner (`tests/loo_cross_validation.rs`); produces
//! cross-fixture mean / median / stddev aggregates plus per-fixture
//! deltas relative to the leave-one-out training mean.
//!
//! The LO-CV protocol (panel discipline P3 / P9 / P16):
//!
//! 1. For each fixture i ∈ [0..N], compute per-fixture metrics
//!    (RSCR, FP rate, fault recall, replay-holds count, episode
//!    count) under a fixed `FusionConfig`.
//! 2. The "training mean for held-out i" is the mean over the other
//!    N-1 fixtures.
//! 3. The "test value at i" is the per-fixture metric of fixture i.
//! 4. The cross-fixture aggregate stddev (over all N fixtures) is
//!    the LO-1 reliability proxy.
//!
//! No bank mutation, no synthetic-data generation. Theorem 9
//! preservation: every fixture's per-fixture metrics include
//! `deterministic_replay_holds`; the aggregate counts the number of
//! fixtures where replay was verified.

extern crate std;

use std::collections::BTreeMap;
use std::format;
use std::string::String;
use std::vec::Vec;

/// Per-fixture record captured during a LO-CV pass.
#[derive(Debug, Clone)]
pub struct LooCvFixtureRecord {
    pub fixture_name: &'static str,
    pub rscr: f64,
    pub clean_window_fp_rate: f64,
    pub fault_recall: f64,
    pub raw_alert_count: u64,
    pub fusion_episode_count: u64,
    pub consensus_confirmed_typed_episodes: u64,
    pub deterministic_replay_holds: bool,
}

/// Cross-fixture aggregate over LO-CV records.
#[derive(Debug, Clone)]
pub struct LooCvAggregate {
    pub fixtures_observed: usize,
    pub fixtures_with_replay_holds: usize,
    pub mean_rscr: f64,
    pub stddev_rscr: f64,
    pub mean_clean_window_fp_rate: f64,
    pub stddev_clean_window_fp_rate: f64,
    pub mean_fault_recall: f64,
    pub stddev_fault_recall: f64,
    pub total_raw_alerts: u64,
    pub total_episodes: u64,
    pub total_typed_episodes: u64,
    /// Phase ζ.9 — informational delta for bank-aware Layer-3 typing.
    /// Captured per-fixture; the aggregate sums them. Useful for
    /// detecting refinement effects that don't show up in L-2 metrics
    /// (e.g. family-tier detector filtering changes window_tier_mask
    /// → bank affinity scoring → typed-confirmed count, but not
    /// cell_consensus → L-2 FP rate).
    pub total_consensus_confirmed: u64,
    /// LO-CV per-fixture deltas: for each fixture i, the value of
    /// the metric at i minus the mean over the other N-1 fixtures.
    /// Keyed by metric name → Vec<(fixture_name, delta)>.
    pub per_fixture_deltas: BTreeMap<&'static str, Vec<(&'static str, f64)>>,
}

/// Convenience: run a LO-CV pass over a slice of per-fixture records.
///
/// The runner itself (the LO-CV iteration that calls
/// `run_fusion_evaluation` once per fixture) lives in
/// `tests/loo_cross_validation.rs`; this function takes the
/// already-captured records and produces the aggregate.
pub fn run_loo_cv(records: &[LooCvFixtureRecord]) -> LooCvAggregate {
    aggregate_loo_cv(records)
}

/// Compute the LO-CV aggregate from per-fixture records.
pub fn aggregate_loo_cv(records: &[LooCvFixtureRecord]) -> LooCvAggregate {
    let n = records.len();
    if n == 0 {
        return LooCvAggregate {
            fixtures_observed: 0,
            fixtures_with_replay_holds: 0,
            mean_rscr: 0.0,
            stddev_rscr: 0.0,
            mean_clean_window_fp_rate: 0.0,
            stddev_clean_window_fp_rate: 0.0,
            mean_fault_recall: 0.0,
            stddev_fault_recall: 0.0,
            total_raw_alerts: 0,
            total_episodes: 0,
            total_typed_episodes: 0,
            total_consensus_confirmed: 0,
            per_fixture_deltas: BTreeMap::new(),
        };
    }
    let nf = n as f64;

    let mean_rscr = records.iter().map(|r| r.rscr).sum::<f64>() / nf;
    let mean_fp = records.iter().map(|r| r.clean_window_fp_rate).sum::<f64>() / nf;
    let mean_recall = records.iter().map(|r| r.fault_recall).sum::<f64>() / nf;

    let stddev_rscr = (records.iter()
        .map(|r| (r.rscr - mean_rscr).powi(2))
        .sum::<f64>() / nf).sqrt();
    let stddev_fp = (records.iter()
        .map(|r| (r.clean_window_fp_rate - mean_fp).powi(2))
        .sum::<f64>() / nf).sqrt();
    let stddev_recall = (records.iter()
        .map(|r| (r.fault_recall - mean_recall).powi(2))
        .sum::<f64>() / nf).sqrt();

    let total_raw = records.iter().map(|r| r.raw_alert_count).sum::<u64>();
    let total_eps = records.iter().map(|r| r.fusion_episode_count).sum::<u64>();
    let total_typed = records.iter()
        .map(|r| r.consensus_confirmed_typed_episodes).sum::<u64>();
    let replay_holds = records.iter()
        .filter(|r| r.deterministic_replay_holds).count();

    // Per-fixture deltas: for each fixture i, value(i) - mean(others).
    // Using the closed-form: delta_i = (value(i) - mean) * n / (n-1).
    let mut per_fixture_deltas: BTreeMap<&'static str, Vec<(&'static str, f64)>>
        = BTreeMap::new();
    if n > 1 {
        let scale = nf / (nf - 1.0);
        for r in records {
            let delta_rscr = (r.rscr - mean_rscr) * scale;
            let delta_fp = (r.clean_window_fp_rate - mean_fp) * scale;
            let delta_recall = (r.fault_recall - mean_recall) * scale;
            per_fixture_deltas.entry("rscr").or_default()
                .push((r.fixture_name, delta_rscr));
            per_fixture_deltas.entry("clean_window_fp_rate").or_default()
                .push((r.fixture_name, delta_fp));
            per_fixture_deltas.entry("fault_recall").or_default()
                .push((r.fixture_name, delta_recall));
        }
    }

    LooCvAggregate {
        fixtures_observed: n,
        fixtures_with_replay_holds: replay_holds,
        mean_rscr,
        stddev_rscr,
        mean_clean_window_fp_rate: mean_fp,
        stddev_clean_window_fp_rate: stddev_fp,
        mean_fault_recall: mean_recall,
        stddev_fault_recall: stddev_recall,
        total_raw_alerts: total_raw,
        total_episodes: total_eps,
        total_typed_episodes: total_typed,
        total_consensus_confirmed: total_typed, // alias; refined-vs-baseline delta visible in render
        per_fixture_deltas,
    }
}

/// LO-CV refinement-acceptance gate.
///
/// Returns `true` iff every metric's post-refinement LO-CV mean is
/// within the user-locked tolerance of the baseline LO-CV mean.
///
/// Tolerance per Session-17 user choice: `ε = 0.5 × baseline-stddev`,
/// no regression on any of {rscr, clean_window_fp_rate, fault_recall,
/// replay_holds_count}.
pub fn refinement_passes_gate(
    baseline: &LooCvAggregate,
    refined: &LooCvAggregate,
) -> RefinementGateVerdict {
    let mut reasons: Vec<String> = Vec::new();

    // RSCR — higher is better.
    let rscr_floor = baseline.mean_rscr - 0.5 * baseline.stddev_rscr;
    if refined.mean_rscr < rscr_floor {
        reasons.push(format!(
            "RSCR regressed: refined mean {:.4} < baseline mean {:.4} − 0.5·stddev = {:.4}",
            refined.mean_rscr, baseline.mean_rscr, rscr_floor));
    }

    // FP rate — lower is better; floor is baseline + 0.5·stddev.
    let fp_ceiling = baseline.mean_clean_window_fp_rate
        + 0.5 * baseline.stddev_clean_window_fp_rate;
    if refined.mean_clean_window_fp_rate > fp_ceiling {
        reasons.push(format!(
            "FP rate regressed: refined mean {:.4} > baseline mean {:.4} + 0.5·stddev = {:.4}",
            refined.mean_clean_window_fp_rate, baseline.mean_clean_window_fp_rate, fp_ceiling));
    }

    // Fault recall — higher is better.
    let recall_floor = baseline.mean_fault_recall - 0.5 * baseline.stddev_fault_recall;
    if refined.mean_fault_recall < recall_floor {
        reasons.push(format!(
            "Fault recall regressed: refined mean {:.4} < baseline mean {:.4} − 0.5·stddev = {:.4}",
            refined.mean_fault_recall, baseline.mean_fault_recall, recall_floor));
    }

    // Replay holds — count must not decrease.
    if refined.fixtures_with_replay_holds < baseline.fixtures_with_replay_holds {
        reasons.push(format!(
            "Replay-holds regressed: refined {} < baseline {}",
            refined.fixtures_with_replay_holds, baseline.fixtures_with_replay_holds));
    }

    if reasons.is_empty() {
        RefinementGateVerdict::Accept
    } else {
        RefinementGateVerdict::Reject { reasons }
    }
}

#[derive(Debug, Clone)]
pub enum RefinementGateVerdict {
    Accept,
    Reject { reasons: Vec<String> },
}

// ===== Phase η.2 — K-fold cross-validation aggregate =====

/// K-fold CV record: one entry per fold.
///
/// For each fold, the test set is the held-out subset of fixtures
/// and the train set is the rest. The harness records the aggregate
/// over the test set for that fold.
#[derive(Debug, Clone)]
pub struct KFoldFoldRecord {
    pub fold_index: usize,
    pub test_fixtures: Vec<&'static str>,
    pub test_aggregate: LooCvAggregate,
}

/// K-fold cross-validation aggregate. Captures per-fold test
/// metrics + the cross-fold mean / stddev.
#[derive(Debug, Clone)]
pub struct KFoldCvAggregate {
    pub k: usize,
    pub fixtures_per_fold: usize,
    pub folds: Vec<KFoldFoldRecord>,
    pub cross_fold_mean_rscr: f64,
    pub cross_fold_stddev_rscr: f64,
    pub cross_fold_mean_fp: f64,
    pub cross_fold_stddev_fp: f64,
    pub cross_fold_mean_recall: f64,
    pub cross_fold_stddev_recall: f64,
    pub all_folds_replay_holds: bool,
}

/// Run K-fold cross-validation over a slice of records.
///
/// Records are partitioned into K folds in their declared order.
/// Each fold's aggregate is computed independently. The cross-fold
/// mean / stddev is reported as the K-fold CV result, complementing
/// the LO-1 CV from `aggregate_loo_cv`.
///
/// Constraints: K >= 2, n >= K (at least one record per fold).
pub fn aggregate_kfold_cv(records: &[LooCvFixtureRecord], k: usize) -> KFoldCvAggregate {
    let n = records.len();
    if k < 2 || n < k {
        return KFoldCvAggregate {
            k, fixtures_per_fold: 0, folds: Vec::new(),
            cross_fold_mean_rscr: 0.0, cross_fold_stddev_rscr: 0.0,
            cross_fold_mean_fp: 0.0, cross_fold_stddev_fp: 0.0,
            cross_fold_mean_recall: 0.0, cross_fold_stddev_recall: 0.0,
            all_folds_replay_holds: false,
        };
    }

    let fpf = (n + k - 1) / k;  // ceiling division — last fold may be smaller

    let mut folds: Vec<KFoldFoldRecord> = Vec::with_capacity(k);
    let mut all_replay = true;

    for fold_idx in 0..k {
        let start = fold_idx * fpf;
        let end = ((fold_idx + 1) * fpf).min(n);
        if start >= n { continue; }
        let test_set: Vec<LooCvFixtureRecord> = records[start..end].to_vec();
        let test_names: Vec<&'static str> = test_set.iter()
            .map(|r| r.fixture_name).collect();
        let agg = aggregate_loo_cv(&test_set);
        if agg.fixtures_with_replay_holds != agg.fixtures_observed {
            all_replay = false;
        }
        folds.push(KFoldFoldRecord {
            fold_index: fold_idx,
            test_fixtures: test_names,
            test_aggregate: agg,
        });
    }

    let kf = folds.len() as f64;
    let mean_rscr = folds.iter().map(|f| f.test_aggregate.mean_rscr).sum::<f64>() / kf;
    let mean_fp = folds.iter().map(|f| f.test_aggregate.mean_clean_window_fp_rate).sum::<f64>() / kf;
    let mean_recall = folds.iter().map(|f| f.test_aggregate.mean_fault_recall).sum::<f64>() / kf;
    let stddev_rscr = (folds.iter()
        .map(|f| (f.test_aggregate.mean_rscr - mean_rscr).powi(2))
        .sum::<f64>() / kf).sqrt();
    let stddev_fp = (folds.iter()
        .map(|f| (f.test_aggregate.mean_clean_window_fp_rate - mean_fp).powi(2))
        .sum::<f64>() / kf).sqrt();
    let stddev_recall = (folds.iter()
        .map(|f| (f.test_aggregate.mean_fault_recall - mean_recall).powi(2))
        .sum::<f64>() / kf).sqrt();

    KFoldCvAggregate {
        k, fixtures_per_fold: fpf,
        folds,
        cross_fold_mean_rscr: mean_rscr,
        cross_fold_stddev_rscr: stddev_rscr,
        cross_fold_mean_fp: mean_fp,
        cross_fold_stddev_fp: stddev_fp,
        cross_fold_mean_recall: mean_recall,
        cross_fold_stddev_recall: stddev_recall,
        all_folds_replay_holds: all_replay,
    }
}

/// Render the K-fold CV aggregate as markdown.
pub fn render_kfold_cv_md(agg: &KFoldCvAggregate) -> String {
    let mut out = String::new();
    out.push_str(&format!("# K-fold cross-validation (K = {}) — Phase η.2\n\n", agg.k));
    out.push_str("Source: Phase η.2 K-fold harness (`src/audit/loo_cv.rs::aggregate_kfold_cv`).\n\n");
    out.push_str(&format!("**Folds:** {}\n", agg.folds.len()));
    out.push_str(&format!("**Fixtures per fold:** {} (last fold may be smaller)\n", agg.fixtures_per_fold));
    out.push_str(&format!("**All folds Theorem 9 replay holds:** {}\n\n", agg.all_folds_replay_holds));

    out.push_str("## Per-fold test-set aggregates\n\n");
    out.push_str("| Fold | Test fixtures | RSCR | FP rate | Fault recall | Replay |\n");
    out.push_str("|-----:|---------------|-----:|--------:|-------------:|:------:|\n");
    for f in &agg.folds {
        out.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {} / {} |\n",
            f.fold_index,
            f.test_fixtures.iter().map(|s| format!("`{}`", s))
                .collect::<Vec<_>>().join(", "),
            f.test_aggregate.mean_rscr,
            f.test_aggregate.mean_clean_window_fp_rate,
            f.test_aggregate.mean_fault_recall,
            f.test_aggregate.fixtures_with_replay_holds,
            f.test_aggregate.fixtures_observed,
        ));
    }

    out.push_str("\n## Cross-fold aggregate\n\n");
    out.push_str("| Metric | Cross-fold mean | Cross-fold stddev |\n");
    out.push_str("|--------|----------------:|------------------:|\n");
    out.push_str(&format!("| RSCR | {:.4} | {:.4} |\n",
        agg.cross_fold_mean_rscr, agg.cross_fold_stddev_rscr));
    out.push_str(&format!("| Clean-window FP rate | {:.4} | {:.4} |\n",
        agg.cross_fold_mean_fp, agg.cross_fold_stddev_fp));
    out.push_str(&format!("| Fault recall | {:.4} | {:.4} |\n",
        agg.cross_fold_mean_recall, agg.cross_fold_stddev_recall));

    out.push_str("\n## Honest empirical reading\n\n");
    out.push_str("K-fold CV averages over multiple test-set configurations\n");
    out.push_str("(in contrast to LO-1 which holds out one fixture at a time).\n");
    out.push_str("With N = ");
    let total_fixtures: usize = agg.folds.iter()
        .map(|f| f.test_aggregate.fixtures_observed).sum();
    out.push_str(&format!("{} fixtures and K = {}, ", total_fixtures, agg.k));
    out.push_str("each fold tests on a multi-fixture set (lower variance per\n");
    out.push_str("fold than LO-1, but fewer folds than LO-1 — the two views\n");
    out.push_str("triangulate the cross-validation noise floor).\n");
    out
}

/// Render the LO-CV baseline aggregate as markdown.
pub fn render_loo_cv_baseline_md(agg: &LooCvAggregate, label: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Leave-one-fixture-out cross-validation: {}\n\n", label));
    out.push_str("Source: Phase ζ.2 LO-CV harness (`src/audit/loo_cv.rs`).\n\n");
    out.push_str(&format!("**Fixtures observed:** {}\n", agg.fixtures_observed));
    out.push_str(&format!("**Fixtures with deterministic replay verified:** {} / {}\n\n",
        agg.fixtures_with_replay_holds, agg.fixtures_observed));
    out.push_str("## Cross-fixture aggregate\n\n");
    out.push_str("| Metric | Mean | Stddev | LO-CV gate floor (mean−0.5·stddev) |\n");
    out.push_str("|--------|-----:|-------:|----------------------------------:|\n");
    out.push_str(&format!("| RSCR | {:.4} | {:.4} | {:.4} |\n",
        agg.mean_rscr, agg.stddev_rscr,
        agg.mean_rscr - 0.5 * agg.stddev_rscr));
    out.push_str(&format!("| Clean-window FP rate | {:.4} | {:.4} | (ceiling: {:.4}) |\n",
        agg.mean_clean_window_fp_rate, agg.stddev_clean_window_fp_rate,
        agg.mean_clean_window_fp_rate + 0.5 * agg.stddev_clean_window_fp_rate));
    out.push_str(&format!("| Fault recall | {:.4} | {:.4} | {:.4} |\n",
        agg.mean_fault_recall, agg.stddev_fault_recall,
        agg.mean_fault_recall - 0.5 * agg.stddev_fault_recall));
    out.push_str(&format!("\n**Total raw alerts:** {}\n", agg.total_raw_alerts));
    out.push_str(&format!("**Total episodes:** {}\n", agg.total_episodes));
    out.push_str(&format!("**Total typed-confirmed episodes:** {}\n\n",
        agg.total_typed_episodes));

    out.push_str("## Per-fixture LO-CV deltas\n\n");
    out.push_str("Delta = (fixture metric) − (mean over other N-1 fixtures).\n");
    out.push_str("Large positive deltas indicate the fixture pulls the mean upward;\n");
    out.push_str("large negative deltas indicate the fixture pulls it downward.\n\n");
    for metric in ["rscr", "clean_window_fp_rate", "fault_recall"] {
        if let Some(deltas) = agg.per_fixture_deltas.get(metric) {
            out.push_str(&format!("### {}\n\n", metric));
            out.push_str("| Fixture | Delta |\n");
            out.push_str("|---------|------:|\n");
            for (fix, d) in deltas {
                out.push_str(&format!("| `{}` | {:+.4} |\n", fix, d));
            }
            out.push_str("\n");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    fn record(name: &'static str, rscr: f64, fp: f64, recall: f64) -> LooCvFixtureRecord {
        LooCvFixtureRecord {
            fixture_name: name,
            rscr,
            clean_window_fp_rate: fp,
            fault_recall: recall,
            raw_alert_count: 1,
            fusion_episode_count: 1,
            consensus_confirmed_typed_episodes: 0,
            deterministic_replay_holds: true,
        }
    }

    #[test]
    fn empty_records_produce_zero_aggregate() {
        let agg = aggregate_loo_cv(&[]);
        assert_eq!(agg.fixtures_observed, 0);
        assert_eq!(agg.mean_rscr, 0.0);
    }

    #[test]
    fn aggregate_computes_mean_and_stddev() {
        let recs = vec![
            record("a", 1.0, 0.10, 0.5),
            record("b", 2.0, 0.20, 0.7),
            record("c", 3.0, 0.30, 0.9),
        ];
        let agg = aggregate_loo_cv(&recs);
        assert_eq!(agg.fixtures_observed, 3);
        assert!((agg.mean_rscr - 2.0).abs() < 1e-9);
        // stddev of [1, 2, 3] = sqrt(2/3) ≈ 0.8165
        assert!((agg.stddev_rscr - 0.8164965809277260).abs() < 1e-9);
    }

    #[test]
    fn refinement_gate_accepts_within_tolerance() {
        let base = aggregate_loo_cv(&[
            record("a", 1.0, 0.10, 0.5),
            record("b", 2.0, 0.20, 0.7),
            record("c", 3.0, 0.30, 0.9),
        ]);
        // Refinement: same fixtures, same numbers — no regression.
        let refined = base.clone();
        match refinement_passes_gate(&base, &refined) {
            RefinementGateVerdict::Accept => {},
            RefinementGateVerdict::Reject { reasons } => panic!("should accept: {:?}", reasons),
        }
    }

    #[test]
    fn refinement_gate_rejects_recall_regression() {
        let base = aggregate_loo_cv(&[
            record("a", 1.0, 0.10, 0.7),
            record("b", 2.0, 0.20, 0.7),
            record("c", 3.0, 0.30, 0.7),
        ]);
        // mean_recall = 0.7, stddev = 0; floor = 0.7. Refinement to 0.6 fails.
        let refined = aggregate_loo_cv(&[
            record("a", 1.0, 0.10, 0.6),
            record("b", 2.0, 0.20, 0.6),
            record("c", 3.0, 0.30, 0.6),
        ]);
        match refinement_passes_gate(&base, &refined) {
            RefinementGateVerdict::Reject { reasons } => {
                assert!(reasons.iter().any(|r| r.contains("Fault recall")));
            },
            _ => panic!("should reject"),
        }
    }
}
