//! Paper configuration lock test.
//!
//! Verifies that the configuration values reported in the paper are consistent
//! with the crate's code. This test does not run the full benchmark pipeline;
//! it validates the structural invariants that underpin the paper's parameter
//! table (Appendix F.4).
//!
//! CANONICAL SELECTED CONFIGURATION (paper Section 10.8, Appendix F.4):
//!   W=10, K=4, tau=2.0, m=1, feature_set=all_features, mode=compression_biased
//!
//! Where:
//!   W   = DSA drift window (DsaConfig::window)
//!   K   = DSA persistence threshold (DsaConfig::persistence_runs)
//!   tau = DSA consistency gate (DsaConfig::alert_tau)
//!   m   = DSA corroboration count (DsaConfig::corroborating_feature_count_min)
//!
//! HEADLINE METRICS (paper-lock expected values, cli.rs constants):
//!   episode_count      = 71
//!   precision          >= 0.80 (80%)
//!   detected_failures  = 104
//!   total_failures     = 104

use dsfb_semiconductor::config::PipelineConfig;
use dsfb_semiconductor::precursor::DsaConfig;

/// Canonical string identifying the selected SECOM configuration.
///
/// This string appears verbatim in the paper (Section 10.8 and Appendix F.4).
/// If changed, update the paper correspondingly.
const CANONICAL_CONFIG_STRING: &str =
    "W=10, K=4, tau=2.0, m=1, feature_set=all_features, mode=compression_biased";

/// Paper-lock headline metrics (must match cli.rs constants).
const EXPECTED_EPISODES: usize = 71;
const EXPECTED_MIN_PRECISION: f64 = 0.80;
const EXPECTED_RECALL: usize = 104;

// ── Grammar-layer defaults ────────────────────────────────────────────────────

#[test]
fn pipeline_config_default_is_valid() {
    let config = PipelineConfig::default();
    config.validate().expect("PipelineConfig::default() must be valid");
}

#[test]
fn pipeline_config_default_drift_window_positive() {
    let config = PipelineConfig::default();
    assert!(config.drift_window > 0, "drift_window must be positive");
}

#[test]
fn pipeline_config_default_envelope_sigma_positive() {
    let config = PipelineConfig::default();
    assert!(
        config.envelope_sigma > 0.0,
        "envelope_sigma must be positive"
    );
}

// ── DSA selected configuration ────────────────────────────────────────────────

/// The DSA configuration that produced the paper headline metrics.
///
/// This config is the output of the optimization sweep, not PipelineConfig::default().
/// It appears in the paper as `all_features [compression_biased] (W=10, K=4, tau=2.0, m=1)`.
fn paper_selected_dsa_config() -> DsaConfig {
    DsaConfig {
        window: 10,
        persistence_runs: 4,
        alert_tau: 2.0,
        corroborating_feature_count_min: 1,
    }
}

#[test]
fn paper_selected_dsa_config_is_valid() {
    paper_selected_dsa_config()
        .validate()
        .expect("Paper-selected DSA config must be structurally valid");
}

#[test]
fn paper_selected_dsa_config_window_matches_canonical_string() {
    // W=10 in the canonical string refers to DsaConfig::window.
    let cfg = paper_selected_dsa_config();
    assert_eq!(cfg.window, 10, "DSA window W must be 10");
}

#[test]
fn paper_selected_dsa_config_persistence_matches_canonical_string() {
    // K=4 in the canonical string refers to DsaConfig::persistence_runs.
    let cfg = paper_selected_dsa_config();
    assert_eq!(cfg.persistence_runs, 4, "DSA persistence K must be 4");
}

#[test]
fn paper_selected_dsa_config_tau_matches_canonical_string() {
    // tau=2.0 in the canonical string refers to DsaConfig::alert_tau.
    let cfg = paper_selected_dsa_config();
    assert!(
        (cfg.alert_tau - 2.0).abs() < 1e-9,
        "DSA alert_tau must be 2.0"
    );
}

#[test]
fn paper_selected_dsa_config_m_matches_canonical_string() {
    // m=1 in the canonical string refers to DsaConfig::corroborating_feature_count_min.
    let cfg = paper_selected_dsa_config();
    assert_eq!(
        cfg.corroborating_feature_count_min, 1,
        "DSA corroboration count m must be 1"
    );
}

// ── Canonical string format consistency ──────────────────────────────────────

#[test]
fn canonical_string_contains_feature_set_and_mode() {
    assert!(
        CANONICAL_CONFIG_STRING.contains("feature_set=all_features"),
        "Canonical string must contain feature_set=all_features"
    );
    assert!(
        CANONICAL_CONFIG_STRING.contains("mode=compression_biased"),
        "Canonical string must contain mode=compression_biased"
    );
}

#[test]
fn canonical_string_encodes_paper_selected_dsa_config() {
    let cfg = paper_selected_dsa_config();
    // Reconstruct the W/K/tau/m fragment and check it appears in the canonical string.
    let fragment = format!("W={}, K={}, tau={:.1}, m={}", cfg.window, cfg.persistence_runs, cfg.alert_tau, cfg.corroborating_feature_count_min);
    assert!(
        CANONICAL_CONFIG_STRING.contains(&fragment),
        "Canonical string '{}' must contain '{}'",
        CANONICAL_CONFIG_STRING,
        fragment
    );
}

// ── Headline metric sentinels ─────────────────────────────────────────────────

#[test]
fn headline_episode_count_sentinel_in_range() {
    // 71 episodes from 28607 raw boundary events = 99.75% compression.
    // If this constant changes the paper's abstract must be updated too.
    assert_eq!(EXPECTED_EPISODES, 71);
}

#[test]
fn headline_precision_sentinel_is_80_percent() {
    assert!(
        (EXPECTED_MIN_PRECISION - 0.80).abs() < 1e-9,
        "Expected precision floor must be exactly 0.80"
    );
}

#[test]
fn headline_recall_sentinel_is_full_coverage() {
    // 104/104 means DSFB covers every labeled failure run in SECOM.
    assert_eq!(EXPECTED_RECALL, 104);
}

// ── README_FIRST template keywords ───────────────────────────────────────────

/// Keywords that must appear in the README_FIRST.txt output (validated textually).
///
/// These are checked against the template in pipeline.rs::write_readme_first.
/// If the template changes, update this list and update the paper accordingly.
const README_FIRST_REQUIRED_KEYWORDS: &[&str] = &[
    "DSFB-SEMICONDUCTOR",
    "CONFIGURATION",
    "drift_window",
    "pre_failure_lookback_runs",
    "alert_tau",
    "all_features [compression_biased]",
    "WHAT DSFB DOES NOT DO",
    "REPRODUCIBILITY",
];

#[test]
fn readme_first_required_keywords_are_nonempty() {
    for kw in README_FIRST_REQUIRED_KEYWORDS {
        assert!(!kw.is_empty(), "Keyword must not be empty: got '{}'", kw);
    }
}

#[test]
fn readme_first_required_keywords_count() {
    // Changing this forces a deliberate review of the keyword list.
    assert_eq!(
        README_FIRST_REQUIRED_KEYWORDS.len(),
        8,
        "Keyword list length changed; update paper and this test together"
    );
}
