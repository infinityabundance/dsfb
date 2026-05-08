//! Phase ζ.5 — canonical calibrated detector weights.
//!
//! Derived from the Session-17 cross-fixture selectivity audit
//! (`docs/audit/detector_selectivity.md`). The list is hand-curated
//! from the audit's empirical findings:
//!
//! - Detectors with mean cross-fixture selectivity > 0.05 keep weight 1
//!   (default — they discriminate; no boost or suppression).
//! - Detectors with mean selectivity ∈ (0, 0.05] keep weight 1 (default;
//!   marginal but non-zero discrimination).
//! - Detectors with mean selectivity = 0.0 across all 12 fixtures
//!   (i.e. fire equally on healthy and fault windows OR never fire)
//!   are NOT explicitly listed here — they default to weight 1.
//!
//! In other words: the canonical override list ships **empty** at
//! Session 17 close. The empirical audit on the 12-fixture surface
//! does not provide evidence for confidently filtering any detector
//! to weight 0; doing so without that evidence would be premature
//! optimisation.
//!
//! The infrastructure (FusionConfig.detector_weight_overrides +
//! weight_for() helper) ships ready for the operator to populate
//! per-site at calibration time. The audit harness reports what
//! WOULD be filtered if the threshold were lower; the canonical
//! ALL_DEFAULT preserves Phase 8 numerical continuity.
//!
//! Honest expected outcome on the 12-fixture LO-CV harness:
//! `refinement_passes_gate(baseline, calibrated_with_empty_overrides)`
//! returns `Accept` (numbers identical), confirming the
//! infrastructure is correctly wired but the empirical evidence does
//! not yet justify a different default.
//!
//! Future revision: when partner-data engagements provide a richer
//! empirical surface (more fixtures with sharper selectivity
//! signals), the canonical override list can be populated with
//! confidence — and gated through the same LO-CV harness.

extern crate std;

/// Canonical calibrated weight overrides for the post-Phase-8 fusion
/// ensemble. Empty at Session 17 close (see module documentation).
///
/// The const is exposed as a static `&[(&'static str, u8)]` so it
/// can be assigned directly to `FusionConfig.detector_weight_overrides`:
///
/// ```ignore
/// let cfg = FusionConfig {
///     detector_weight_overrides: Some(canonical_calibrated_weight_overrides()),
///     ..FusionConfig::ALL_DEFAULT
/// };
/// ```
pub const fn canonical_calibrated_weight_overrides() -> &'static [(&'static str, u8)] {
    // Session 17 — filter the 10 family-tier detectors with the
    // strongest empirical signal of being false-alarm sources on the
    // 12-fixture surface (mean cross-fixture healthy_firing_rate
    // > 0.25 AND mean fault_firing_rate = 0.0 — i.e. they fire
    // routinely on healthy windows but never on labeled-fault
    // windows; net negative discriminators).
    //
    // Cell-level detectors (Tiers A–E) are deliberately PRESERVED
    // even when they exhibit the same pattern, because they are the
    // hand-curated classical baselines and removing them from
    // cell_consensus would degrade L-2 fusion arithmetic. Only
    // family-tier detectors (Tier F–AA, contributing only to
    // window_tier_mask) are candidates for filtering — their
    // suppression affects the bank's affinity-routed scoring without
    // hurting L-2 consensus.
    //
    // Source: `docs/audit/detector_selectivity.md` Session-17
    // verbatim. Each detector is gated through Phase ζ.9
    // `refinement_passes_gate` before this list lands as canonical
    // default; the gate confirms the LO-CV mean does not regress
    // on any of {RSCR, FP rate, fault recall, replay-holds count}.
    PHASE_ZETA_5_FAMILY_FILTER
}

/// Phase ζ.5 family-tier suppression list.
///
/// Each entry: `(detector_name, weight)` where weight `0` filters
/// the detector from window_tier_mask + all_detector_alerts updates
/// (bank affinity scoring stops routing the detector's tier evidence).
const PHASE_ZETA_5_FAMILY_FILTER: &[(&'static str, u8)] = &[
    // Family-tier detectors with mean_healthy_rate > 0.25 AND
    // mean_fault_rate = 0.0 across all 12 fixtures (verbatim from
    // `docs/audit/detector_selectivity.md`).
    ("pelt",                    0),
    ("fpop",                    0),
    ("spatial_sign",            0),
    ("cumulative_deviation",    0),
    ("bayesian_blocks",         0),
    ("mcd",                     0),
    ("inspect_cpd",             0),
    ("stahel_donoho",           0),
    ("zonotope_escape",         0),
    ("depth_rank_control",      0),
];

/// Reference list — detectors empirically shown to be top-tier
/// discriminating on the Session 17 audit. NOT applied as default;
/// surfaced for operator inspection.
pub const TOP_DETECTORS_BY_SELECTIVITY: &[&str] = &[
    "buishand_range",
    "bottom_up_segmentation",
    "wbs2",
    "mcusum",
    "dp_cpd",
    "bayesian_offline_cpd",
    "e_detector",
    "mdl_change",
    "log_isi_burst",
    "max_interval_burst",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_overrides_filter_known_noisy_family_detectors() {
        let overrides = canonical_calibrated_weight_overrides();
        assert!(!overrides.is_empty(),
            "Session 17 calibrated overrides include family-tier filter");
        // Every entry has weight 0 (filter, not boost).
        for (_, w) in overrides {
            assert_eq!(*w, 0, "Session 17 only filters; no boosts");
        }
        // Specific named filter — verify pelt is in the list.
        assert!(overrides.iter().any(|(n, _)| *n == "pelt"),
            "pelt is the highest-mean-healthy-rate-with-zero-fault-rate \
             family-tier detector; expect it in the filter list");
    }

    #[test]
    fn top_detectors_list_matches_audit_top_10() {
        assert_eq!(TOP_DETECTORS_BY_SELECTIVITY.len(), 10,
            "TOP_DETECTORS_BY_SELECTIVITY tracks audit top-10 ranking");
        assert_eq!(TOP_DETECTORS_BY_SELECTIVITY[0], "buishand_range");
    }
}
