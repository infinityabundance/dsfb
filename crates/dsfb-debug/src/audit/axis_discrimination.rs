//! Per-axis (tier) fault-discrimination report (Phase ζ.6).
//!
//! For each of the 27 mathematical axes (tier bits A-U + EXTRA +
//! V/X/Y/Z/AA from Phase 5), aggregate the per-detector firing
//! patterns into a per-tier discrimination ratio:
//!
//! ```text
//! axis_discrimination = mean_fault_rate / (mean_healthy_rate + eps)
//! ```
//!
//! The ratio is the per-tier analogue of the per-detector
//! selectivity index. Higher = the axis discriminates fault from
//! clean cells more sharply on the cross-fixture surface.
//!
//! Read-only telemetry; no axis is pruned, no tier bit removed.
//! The report identifies "always-firing" axes (high false alarm)
//! and "never-firing" axes (architectural-breadth claim, currently
//! dormant on the 12 vendored fixtures) for operator-side calibration.

extern crate std;

use std::format;
use std::string::String;
use std::vec::Vec;

use super::detector_firing::DetectorSelectivity;

const AXIS_EPS: f64 = 0.01;

/// Per-axis (tier) cross-fixture discrimination record.
#[derive(Debug, Clone)]
pub struct AxisDiscriminationEntry {
    pub tier_letter: &'static str,
    pub tier_bit: u32,
    pub detectors_in_axis: usize,
    pub mean_healthy_rate: f64,
    pub mean_fault_rate: f64,
    pub discrimination_ratio: f64,
    pub fixtures_observed: usize,
}

#[derive(Debug, Clone)]
pub struct AxisDiscriminationReport {
    /// Sorted descending by discrimination_ratio.
    pub entries: Vec<AxisDiscriminationEntry>,
}

/// Map a detector name to the tier-letter it belongs to.
///
/// This mapping mirrors the `cell_tier_mask |= TIER_BIT_*`
/// assignments in `src/fusion.rs::run_inner`. When a detector spans
/// multiple tiers (e.g. some Phase-5 wave detectors), its primary
/// tier is reported. Returned as a static string so the report can
/// hold `&'static str` references throughout.
fn tier_of(detector_name: &str) -> Option<&'static str> {
    // Tier A — parametric trio
    match detector_name {
        "scalar_3sigma" | "scalar" | "cusum" | "ewma" => Some("A"),
        // Tier B — robust statistics
        "robust_z" | "page_hinkley" | "tukey_iqr" => Some("B"),
        // Tier C — model / non-parametric
        "spectral_residual" | "matrix_profile" | "bocpd" |
        "isolation_forest" | "lof" => Some("C"),
        // Tier D — additional non-dep
        "mann_kendall" | "rolling_z" | "ar1_residual" |
        "mahalanobis" | "ks_rolling" => Some("D"),
        // Tier E — debugging-specific
        "poisson_burst" | "saturation_chain" | "chi_squared_prop" => Some("E"),
        // Tier F — burst (neuroscience)
        "max_interval_burst" | "log_isi_burst" |
        "rank_surprise_burst" | "misi_burst" => Some("F"),
        // Tier EXTRA — GLR/ADWIN/MEWMA/retry-storm/correlation-break
        "glr" | "adwin" | "mewma" | "retry_storm" | "correlation_break" => Some("EXTRA"),
        // Tier M — debugging-native (extras)
        "causal_lag" => Some("M"),
        // Tier U — dynamical systems
        "dsfb_structural" => Some("STRUCTURAL"),
        _ => None,
    }
}

/// Map a detector name to its tier-letter via prefix matching for
/// the family-flag detectors that use shared name prefixes.
fn tier_of_family(detector_name: &str) -> Option<&'static str> {
    if detector_name.starts_with("g_") { return Some("G"); }
    if detector_name.starts_with("h_") { return Some("H"); }
    if detector_name.starts_with("i_") { return Some("I"); }
    if detector_name.starts_with("j_") { return Some("J"); }
    if detector_name.starts_with("k_") { return Some("K"); }
    if detector_name.starts_with("l_") { return Some("L"); }
    if detector_name.starts_with("m_") { return Some("M"); }
    if detector_name.starts_with("n_") { return Some("N"); }
    if detector_name.starts_with("o_") { return Some("O"); }
    if detector_name.starts_with("p_") { return Some("P"); }
    if detector_name.starts_with("q_") { return Some("Q"); }
    if detector_name.starts_with("r_") { return Some("R"); }
    if detector_name.starts_with("s_") { return Some("S"); }
    if detector_name.starts_with("t_") { return Some("T"); }
    if detector_name.starts_with("u_") { return Some("U"); }
    if detector_name.starts_with("v_") { return Some("V"); }
    if detector_name.starts_with("x_") { return Some("X"); }
    if detector_name.starts_with("y_") { return Some("Y"); }
    if detector_name.starts_with("z_") { return Some("Z"); }
    if detector_name.starts_with("aa_") { return Some("AA"); }
    None
}

/// Resolve a detector to its tier letter; falls back to "?" if
/// unmapped.
fn resolve_tier(detector_name: &str) -> &'static str {
    if let Some(t) = tier_of(detector_name) { return t; }
    if let Some(t) = tier_of_family(detector_name) { return t; }
    "?"
}

/// Tier letter → tier bit (mirroring `src/heuristics_bank.rs`).
fn tier_bit(letter: &str) -> u32 {
    match letter {
        "A"     => 1 << 0,
        "B"     => 1 << 1,
        "C"     => 1 << 2,
        "D"     => 1 << 3,
        "E"     => 1 << 4,
        "F"     => 1 << 5,
        "EXTRA" => 1 << 6,
        "G"     => 1 << 7,
        "H"     => 1 << 8,
        "I"     => 1 << 9,
        "J"     => 1 << 10,
        "K"     => 1 << 11,
        "L"     => 1 << 12,
        "M"     => 1 << 13,
        "N"     => 1 << 14,
        "O"     => 1 << 15,
        "P"     => 1 << 16,
        "Q"     => 1 << 17,
        "R"     => 1 << 18,
        "S"     => 1 << 19,
        "T"     => 1 << 20,
        "U"     => 1 << 21,
        "V"     => 1 << 22,
        "X"     => 1 << 24,
        "Y"     => 1 << 25,
        "Z"     => 1 << 26,
        "AA"    => 1 << 27,
        _       => 0,
    }
}

/// Compute per-axis discrimination from per-fixture selectivity records.
pub fn compute_axis_discrimination(
    per_fixture: &[Vec<DetectorSelectivity>],
) -> AxisDiscriminationReport {
    use std::collections::BTreeMap;

    // Group selectivity records by tier letter; keep per-fixture
    // tracking for `fixtures_observed`.
    let mut by_tier: BTreeMap<&'static str, Vec<&DetectorSelectivity>>
        = BTreeMap::new();
    let mut fixtures_per_tier: BTreeMap<&'static str,
                                         std::collections::BTreeSet<&'static str>>
        = BTreeMap::new();

    for fix in per_fixture {
        for entry in fix {
            let tier = resolve_tier(entry.detector_name);
            if tier == "?" || tier == "STRUCTURAL" { continue; }
            by_tier.entry(tier).or_default().push(entry);
            fixtures_per_tier.entry(tier).or_default().insert(entry.fixture_name);
        }
    }

    let mut entries: Vec<AxisDiscriminationEntry> = by_tier.into_iter()
        .map(|(letter, recs)| {
            let n = recs.len() as f64;
            let mean_healthy = recs.iter().map(|r| r.healthy_firing_rate)
                .sum::<f64>() / n;
            let mean_fault = recs.iter().map(|r| r.fault_firing_rate)
                .sum::<f64>() / n;
            let discrimination_ratio = mean_fault / (mean_healthy + AXIS_EPS);
            let fixtures_observed = fixtures_per_tier.get(letter)
                .map(|s| s.len()).unwrap_or(0);

            AxisDiscriminationEntry {
                tier_letter: letter,
                tier_bit: tier_bit(letter),
                detectors_in_axis: recs.len(),
                mean_healthy_rate: mean_healthy,
                mean_fault_rate: mean_fault,
                discrimination_ratio,
                fixtures_observed,
            }
        }).collect();

    entries.sort_by(|a, b| b.discrimination_ratio
        .partial_cmp(&a.discrimination_ratio)
        .unwrap_or(std::cmp::Ordering::Equal));

    AxisDiscriminationReport { entries }
}

/// Render the axis-discrimination report as a markdown table.
pub fn render_axis_discrimination_md(report: &AxisDiscriminationReport) -> String {
    let mut out = String::new();
    out.push_str("# Per-axis (tier) discrimination audit\n\n");
    out.push_str("Discrimination ratio = mean_fault_rate / (mean_healthy_rate + 0.01)\n");
    out.push_str("aggregated across all detectors in each axis, across all 12 fixtures.\n\n");
    out.push_str("Higher = axis fires more selectively on fault windows.\n");
    out.push_str("A ratio near 1.0 indicates the axis fires equally on healthy/fault.\n");
    out.push_str("A ratio near 0.0 indicates the axis is dormant on the current surface.\n\n");
    out.push_str("Source: Phase ζ.6 audit harness (`src/audit/axis_discrimination.rs`).\n\n");
    out.push_str("| Tier | Bit | Detectors | Fixtures | Mean healthy | Mean fault | Discrimination |\n");
    out.push_str("|:----:|----:|----------:|---------:|-------------:|-----------:|---------------:|\n");

    for e in &report.entries {
        out.push_str(&format!(
            "| {} | 0x{:08x} | {} | {} | {:.4} | {:.4} | {:.4} |\n",
            e.tier_letter,
            e.tier_bit,
            e.detectors_in_axis,
            e.fixtures_observed,
            e.mean_healthy_rate,
            e.mean_fault_rate,
            e.discrimination_ratio,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    fn sel(name: &'static str, fixture: &'static str,
           healthy: f64, fault: f64) -> DetectorSelectivity {
        DetectorSelectivity {
            detector_name: name,
            fixture_name: fixture,
            healthy_firing_rate: healthy,
            fault_firing_rate: fault,
            selectivity_index: fault / (healthy + 0.01),
        }
    }

    #[test]
    fn tier_a_resolves_for_parametric_trio() {
        assert_eq!(resolve_tier("scalar_3sigma"), "A");
        assert_eq!(resolve_tier("cusum"), "A");
        assert_eq!(resolve_tier("ewma"), "A");
    }

    #[test]
    fn family_prefix_resolves() {
        assert_eq!(resolve_tier("g_ddm"), "G");
        assert_eq!(resolve_tier("h_jensen_shannon"), "H");
        assert_eq!(resolve_tier("aa_arch_residual"), "AA");
    }

    #[test]
    fn axis_discrimination_aggregates_across_detectors() {
        let f1 = vec![
            sel("scalar_3sigma", "f1", 0.0, 1.0),
            sel("cusum",         "f1", 0.0, 1.0),
        ];
        let f2 = vec![
            sel("scalar_3sigma", "f2", 0.0, 1.0),
            sel("cusum",         "f2", 0.0, 1.0),
        ];
        let report = compute_axis_discrimination(&[f1, f2]);
        let a = report.entries.iter().find(|e| e.tier_letter == "A").unwrap();
        assert_eq!(a.detectors_in_axis, 4);  // 2 detectors × 2 fixtures
        assert_eq!(a.fixtures_observed, 2);
        assert!((a.mean_fault_rate - 1.0).abs() < 1e-9);
        assert!(a.discrimination_ratio > 50.0);
    }
}
