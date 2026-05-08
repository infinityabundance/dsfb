//! Per-detector firing-pattern audit (Phase ζ.1 / ζ.3).
//!
//! Consumes the `DetectorOutput` slice already captured during a
//! `run_fusion_evaluation` call and produces a per-detector
//! selectivity index. The selectivity index is the ratio of
//! fault-firing-rate to healthy-firing-rate, with an additive
//! lower-bound on the denominator to prevent division-by-zero:
//!
//! ```text
//! selectivity_index = (captured_faults / total_faults)
//!                   / ((clean_window_false_alerts / clean_windows) + eps)
//! ```
//!
//! eps = 0.01 (1% per-window false-alarm floor) is a hand-picked
//! Laplace-style smoothing constant; documented for reviewer
//! reproducibility. Higher selectivity_index = more discriminating
//! detector on this fixture.
//!
//! Cross-fixture aggregation produces mean / stddev across the 12
//! vendored fixtures; the cross-fixture stddev is the LO-1
//! reliability proxy that informs Phase ζ.5 weighted-consensus
//! calibration.
//!
//! Read-only: zero engine mutation. Theorem 9 preserved.

extern crate std;

use std::collections::BTreeMap;
use std::format;
use std::string::String;
use std::vec::Vec;

use crate::incumbent_baselines::DetectorOutput;

const SELECTIVITY_EPS: f64 = 0.01;

/// Per-(detector, fixture) selectivity record.
///
/// Computed from a single `DetectorOutput`. All rates are bounded
/// [0, 1]; selectivity_index is bounded above by `1.0 / SELECTIVITY_EPS`
/// (= 100.0) when fault_firing_rate = 1.0 and healthy_firing_rate = 0.
#[derive(Debug, Clone)]
pub struct DetectorSelectivity {
    pub detector_name: &'static str,
    pub fixture_name: &'static str,
    /// (clean_window_false_alerts) / (clean_windows). 0 if no clean windows.
    pub healthy_firing_rate: f64,
    /// (captured_faults) / (total_faults). 0 if no labelled faults.
    pub fault_firing_rate: f64,
    /// Selectivity = fault / (healthy + ε). Higher = more discriminating.
    pub selectivity_index: f64,
}

/// Compute per-detector selectivity records on a single fixture.
///
/// Input `per_detector` is the `FusionMetrics::per_detector` field
/// from a completed fusion run; `fixture_name` is the manifest
/// short-name for cross-fixture aggregation.
pub fn compute_detector_selectivity_per_fixture(
    per_detector: &[DetectorOutput],
    fixture_name: &'static str,
) -> Vec<DetectorSelectivity> {
    per_detector.iter().map(|d| {
        let healthy_firing_rate = if d.clean_windows > 0 {
            d.clean_window_false_alerts as f64 / d.clean_windows as f64
        } else {
            0.0
        };
        let fault_firing_rate = if d.total_faults > 0 {
            d.captured_faults as f64 / d.total_faults as f64
        } else {
            0.0
        };
        let selectivity_index =
            fault_firing_rate / (healthy_firing_rate + SELECTIVITY_EPS);

        DetectorSelectivity {
            detector_name: d.detector_name,
            fixture_name,
            healthy_firing_rate,
            fault_firing_rate,
            selectivity_index,
        }
    }).collect()
}

/// Cross-fixture aggregate per detector.
///
/// Produced by `aggregate_detector_audit` from per-fixture selectivity
/// records. The `top_3_fixtures` and `bottom_3_fixtures` lists are
/// the fixture-names where the detector ranked most / least
/// selective.
#[derive(Debug, Clone)]
pub struct CrossFixtureDetectorEntry {
    pub detector_name: &'static str,
    pub fixtures_observed: usize,
    pub mean_selectivity: f64,
    pub stddev_selectivity: f64,
    pub mean_healthy_rate: f64,
    pub mean_fault_rate: f64,
    /// (fixture_name, selectivity) pairs, top-3 by selectivity.
    pub top_3_fixtures: Vec<(&'static str, f64)>,
    /// (fixture_name, selectivity) pairs, bottom-3 by selectivity.
    pub bottom_3_fixtures: Vec<(&'static str, f64)>,
}

#[derive(Debug, Clone)]
pub struct CrossFixtureDetectorReport {
    /// Sorted descending by mean_selectivity. The most discriminating
    /// detectors come first; the noisiest come last.
    pub entries: Vec<CrossFixtureDetectorEntry>,
}

/// Aggregate per-fixture selectivity records into a cross-fixture
/// report.
///
/// Input is a slice of `Vec<DetectorSelectivity>` — one outer entry
/// per fixture. Detectors are matched by `detector_name`; missing
/// detectors on a fixture (e.g. when a fixture-specific feature flag
/// disabled them) are skipped — `fixtures_observed` reflects only
/// fixtures where the detector actually ran.
pub fn aggregate_detector_audit(
    per_fixture: &[Vec<DetectorSelectivity>],
) -> CrossFixtureDetectorReport {
    let mut by_name: BTreeMap<&'static str, Vec<&DetectorSelectivity>> = BTreeMap::new();
    for fix in per_fixture {
        for entry in fix {
            by_name.entry(entry.detector_name).or_default().push(entry);
        }
    }

    let mut entries: Vec<CrossFixtureDetectorEntry> = by_name.into_iter()
        .map(|(name, recs)| {
            let n = recs.len();
            let mean_sel = recs.iter().map(|r| r.selectivity_index)
                .sum::<f64>() / n as f64;
            let var_sel = recs.iter()
                .map(|r| (r.selectivity_index - mean_sel).powi(2))
                .sum::<f64>() / n as f64;
            let stddev_sel = var_sel.sqrt();
            let mean_healthy = recs.iter().map(|r| r.healthy_firing_rate)
                .sum::<f64>() / n as f64;
            let mean_fault = recs.iter().map(|r| r.fault_firing_rate)
                .sum::<f64>() / n as f64;

            let mut sorted = recs.clone();
            sorted.sort_by(|a, b| b.selectivity_index
                .partial_cmp(&a.selectivity_index)
                .unwrap_or(std::cmp::Ordering::Equal));
            let top_3: Vec<(&'static str, f64)> = sorted.iter().take(3)
                .map(|r| (r.fixture_name, r.selectivity_index)).collect();
            let bottom_3: Vec<(&'static str, f64)> = sorted.iter().rev().take(3)
                .map(|r| (r.fixture_name, r.selectivity_index)).collect();

            CrossFixtureDetectorEntry {
                detector_name: name,
                fixtures_observed: n,
                mean_selectivity: mean_sel,
                stddev_selectivity: stddev_sel,
                mean_healthy_rate: mean_healthy,
                mean_fault_rate: mean_fault,
                top_3_fixtures: top_3,
                bottom_3_fixtures: bottom_3,
            }
        }).collect();

    entries.sort_by(|a, b| b.mean_selectivity
        .partial_cmp(&a.mean_selectivity)
        .unwrap_or(std::cmp::Ordering::Equal));

    CrossFixtureDetectorReport { entries }
}

/// Render the cross-fixture selectivity report as a markdown table.
///
/// Output format: full ranked table with columns
/// (rank, name, mean_selectivity, stddev_selectivity,
/// mean_healthy_rate, mean_fault_rate, fixtures_observed,
/// top_3_fixtures).
///
/// Numbers are formatted to 4 decimal places; nothing rounded for
/// display purposes (academic-honesty discipline).
pub fn render_detector_selectivity_md(report: &CrossFixtureDetectorReport) -> String {
    let mut out = String::new();
    out.push_str("# Per-detector selectivity audit\n\n");
    out.push_str("Selectivity index = fault_firing_rate / (healthy_firing_rate + 0.01).\n");
    out.push_str("Higher = more discriminating on the cross-fixture surface.\n\n");
    out.push_str("Source: Phase ζ.1 audit harness (`src/audit/detector_firing.rs`).\n");
    out.push_str("Data: verbatim from `DetectorOutput` per fixture run.\n\n");
    out.push_str("| Rank | Detector | Mean selectivity | Stddev | Mean healthy | Mean fault | N fix | Top fixture |\n");
    out.push_str("|-----:|----------|-----------------:|-------:|-------------:|-----------:|------:|-------------|\n");

    for (i, e) in report.entries.iter().enumerate() {
        let top_fix = e.top_3_fixtures.first()
            .map(|(n, _)| *n).unwrap_or("--");
        out.push_str(&format!(
            "| {} | `{}` | {:.4} | {:.4} | {:.4} | {:.4} | {} | {} |\n",
            i + 1,
            e.detector_name,
            e.mean_selectivity,
            e.stddev_selectivity,
            e.mean_healthy_rate,
            e.mean_fault_rate,
            e.fixtures_observed,
            top_fix,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;
    use crate::incumbent_baselines::DetectorOutput;

    fn mock_detector(
        name: &'static str,
        clean_window_false_alerts: u64,
        clean_windows: u64,
        captured_faults: u64,
        total_faults: u64,
    ) -> DetectorOutput {
        DetectorOutput {
            detector_name: name,
            raw_alert_count: clean_window_false_alerts + captured_faults,
            alerts_per_signal: [0; 32],
            alert_windows: clean_window_false_alerts + captured_faults,
            episode_count: clean_window_false_alerts + captured_faults,
            captured_faults,
            total_faults,
            clean_window_false_alerts,
            clean_windows,
        }
    }

    #[test]
    fn perfect_detector_high_selectivity() {
        // 0 false alerts on 100 healthy windows, 10 of 10 faults caught.
        // selectivity = 1.0 / (0.0 + 0.01) = 100.0
        let det = mock_detector("perfect", 0, 100, 10, 10);
        let v = compute_detector_selectivity_per_fixture(&[det], "test_fixture");
        assert_eq!(v.len(), 1);
        assert!((v[0].selectivity_index - 100.0).abs() < 1e-9);
        assert!((v[0].fault_firing_rate - 1.0).abs() < 1e-9);
        assert!(v[0].healthy_firing_rate.abs() < 1e-9);
    }

    #[test]
    fn noisy_detector_low_selectivity() {
        // Fires on every healthy window AND every fault window.
        // healthy_rate = 1.0; fault_rate = 1.0; selectivity = 1.0/1.01 ≈ 0.99
        let det = mock_detector("noisy", 100, 100, 10, 10);
        let v = compute_detector_selectivity_per_fixture(&[det], "test_fixture");
        assert!(v[0].selectivity_index < 1.0);
        assert!(v[0].selectivity_index > 0.9);
    }

    #[test]
    fn aggregation_orders_by_selectivity() {
        let det_perfect = mock_detector("perfect", 0, 100, 10, 10);
        let det_noisy = mock_detector("noisy", 100, 100, 10, 10);
        let f1 = compute_detector_selectivity_per_fixture(
            &[det_perfect.clone(), det_noisy.clone()], "fixture1");
        let f2 = compute_detector_selectivity_per_fixture(
            &[det_perfect.clone(), det_noisy.clone()], "fixture2");

        let report = aggregate_detector_audit(&[f1, f2]);
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.entries[0].detector_name, "perfect");
        assert_eq!(report.entries[1].detector_name, "noisy");
        assert!(report.entries[0].mean_selectivity
                > report.entries[1].mean_selectivity);
    }

    #[test]
    fn aggregation_handles_missing_detector_per_fixture() {
        // Fixture 1 has detectors A, B; fixture 2 has only A.
        // Detector A appears in 2 fixtures; B in 1.
        let f1 = vec![
            DetectorSelectivity { detector_name: "a", fixture_name: "f1",
                healthy_firing_rate: 0.0, fault_firing_rate: 1.0,
                selectivity_index: 100.0 },
            DetectorSelectivity { detector_name: "b", fixture_name: "f1",
                healthy_firing_rate: 0.5, fault_firing_rate: 0.5,
                selectivity_index: 0.98 },
        ];
        let f2 = vec![
            DetectorSelectivity { detector_name: "a", fixture_name: "f2",
                healthy_firing_rate: 0.1, fault_firing_rate: 1.0,
                selectivity_index: 9.09 },
        ];
        let report = aggregate_detector_audit(&[f1, f2]);
        let a = report.entries.iter().find(|e| e.detector_name == "a").unwrap();
        let b = report.entries.iter().find(|e| e.detector_name == "b").unwrap();
        assert_eq!(a.fixtures_observed, 2);
        assert_eq!(b.fixtures_observed, 1);
    }

    #[test]
    fn deterministic_replay_same_inputs_same_outputs() {
        // Theorem 9 preservation at the audit-harness level.
        let det = mock_detector("d", 5, 100, 8, 10);
        let v1 = compute_detector_selectivity_per_fixture(&[det.clone()], "fix");
        let v2 = compute_detector_selectivity_per_fixture(&[det.clone()], "fix");
        assert_eq!(v1.len(), v2.len());
        assert!((v1[0].selectivity_index - v2[0].selectivity_index).abs() < 1e-15);
    }
}
