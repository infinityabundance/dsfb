//! DSFB-Debug: multi-detector fusion harness (post Phase 8, std-only).
//!
//! # The augmentation thesis, operationalised
//!
//! Paper §10 states that the heuristics bank should be a **meta-layer**
//! over detector outputs, not a peer detector running on its own
//! residual pipeline. This module operationalises that thesis with
//! a three-layer architecture:
//!
//! 1. **Detector ensemble** — 205 deterministic detectors organised
//!    across 27 mathematical axes (Tiers A–U + EXTRA + V/X/Y/Z/AA),
//!    plus the DSFB structural pipeline. Each detector runs on the
//!    same residual matrix; each is a deterministic `pub fn` in
//!    `incumbent_baselines.rs` with a literature citation.
//!
//! 2. **Consensus arithmetic** — per-(window, signal) cell-level
//!    detectors contribute to a `consensus_count`; per-window
//!    multivariate detectors contribute a per-window boost. The
//!    `min_consensus` config flag is the operator-tunable threshold:
//!    `fire(w, s) := consensus_count(w, s) >= min_consensus`.
//!
//! 3. **DSFB structural meta-layer** — the engine's grammar pipeline
//!    runs on the *original* residuals (preserving its typed-motif
//!    output). Closed episodes pass through the **9-axis bank-aware
//!    fusion** (`FusionConfig`), which routes detector evidence
//!    through per-motif tier-affinity masks (Routed Evidence
//!    Principle, paper §6.5), gates typing on confuser-boundary
//!    margins (paper §6.6), and applies the four-rung
//!    Anti-Hallucination Ladder (Phase 0/5.6/7/8). The output is the
//!    operator evidence packet (paper §11.y): typed motif, runner-up,
//!    declared confuser, three margins, tier consensus factor,
//!    disambiguator boost, witness-gate flags.
//!
//! # The 9-axis fusion configuration
//!
//! Each `FusionConfig` flag controls exactly one axis. Flags can be
//! mixed freely; behaviour is monotone in strictness (axes 1, 2, 6,
//! 8, 9 progressively reduce typed-confirmed counts; axes 3, 5, 7
//! lift them by routing evidence into matched motifs).
//!
//! | # | Axis | Phase | Flag |
//! |:-:|------|:----:|------|
//! | 1 | Provenance gate | 0 | `min_provenance_rank` |
//! | 2 | Margin gate | 2 | `margin_gate` |
//! | 3 | Tier-affinity scoring | 2.5 | `use_tier_affinity` |
//! | 4 | Zero-tier-firing filter | 5.5 | (always-on under Phase 5.5) |
//! | 5 | Adaptive margin gate | 5.5 | (auto-applied when tier_cf > 0.5) |
//! | 6 | Confuser-boundary | 5.6 | `use_confuser_boundary` |
//! | 7 | Disambiguator boost | 6 | `use_disambiguator_boost` |
//! | 8 | Tier-level primary witness | 7 | `use_primary_witness_tier_gate` |
//! | 9 | Per-detector named witness | 8 | `use_primary_witness_detector_gate` |
//!
//! # Honest empirical expectation (falsifiable per panel directive)
//!
//! - Layer 1 alone (consensus arithmetic, no bank): at
//!   `min_consensus = 3` over 11+ baseline detectors, FP rate ≤
//!   `min(single-detector FP)`. Tested on F-11.
//! - Layer 2 alone (DSFB structural episodes, no fusion): produces
//!   typed motifs but no consensus filter. Tested on F-11 (3
//!   episodes, 0.7% FP rate).
//! - Layer 3 (9-axis bank-aware fusion): typed-AND-validated episodes.
//!   On F-11 at N≥7 with all axes active: 1 typed-confirmed episode,
//!   0.0023 FP rate (3× lower than DSFB-structural alone).
//!
//! # Theorem 9 (deterministic replay) preserved at the meta-layer
//!
//! Every component detector is deterministic given fixed seed; the
//! consensus arithmetic is integer addition; the bank's match
//! decision is `argmax` with deterministic tie-breakers. The fusion
//! harness re-fires `verify_deterministic_replay` on every fusion
//! evaluation and emits `deterministic_replay_holds: true` in the
//! `FusionMetrics` block. A failure of that flag would surface as a
//! hard test failure, not a silent metric drift.

#![cfg(feature = "std")]
#![allow(
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::manual_memcpy,
    clippy::unwrap_used,
)]

extern crate std;

use std::vec::Vec;

use crate::error::Result;
use crate::incumbent_baselines::{
    ar1_forecast_residual, bocpd, cusum, ewma, isolation_forest, ks_rolling,
    lof, mahalanobis, mann_kendall, matrix_profile, page_hinkley,
    robust_z_mad, rolling_z_score, scalar_threshold, spectral_residual_td,
    tukey_iqr_fence, DetectorOutput,
};
use crate::types::*;
use crate::DsfbDebugEngine;
use crate::heuristics_bank::{
    TIER_BIT_A, TIER_BIT_B, TIER_BIT_C, TIER_BIT_D, TIER_BIT_E,
};

/// Tier bitmask of the original cell-level grid-filled detectors
/// (Tiers A–E: scalar / robust / non-parametric model / additional
/// non-dep / debugging-specific). These detectors populate
/// `cell_tier_mask` directly via the per-detector branches in
/// `run_inner`. The Phase-5+ family-level detectors (Tiers F–U +
/// V/X/Y/Z/AA, ~188 of 205) populate `window_tier_mask` via the
/// `LAST_WIN_ALERTS` thread-local side channel from
/// `incumbent_baselines.rs`. Sessions 17–18 retained this split
/// — no detector is silent in tier evidence.
pub const TIER_BITS_OLD_DETECTORS: u32 =
    TIER_BIT_A | TIER_BIT_B | TIER_BIT_C | TIER_BIT_D | TIER_BIT_E;

/// All 22 tier bits in the active range (A through U + EXTRA + V).
/// Used as the divisor for `tier_consensus_factor` in
/// `match_episode_with_tier_affinity_axes`. Tiers BB and CC at bits
/// 28/29 are forward-allocation reserves (declared in
/// `heuristics_bank.rs` but not currently populated by any
/// detector).
pub const TIER_BITS_ALL: u32 = u32::MAX & ((1 << 22) - 1);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FusionConfig {
    pub min_consensus: u8,

    pub use_scalar: bool,
    pub use_cusum: bool,
    pub use_ewma: bool,
    pub use_robust_z: bool,
    pub use_page_hinkley: bool,
    pub use_tukey_iqr: bool,
    pub use_spectral_residual: bool,
    pub use_matrix_profile: bool,
    pub use_bocpd: bool,
    pub use_isolation_forest: bool,
    pub use_lof: bool,
    // Tier-D baselines (Session 8) — additional non-dep detectors.
    pub use_mann_kendall: bool,
    pub use_rolling_z: bool,
    pub use_ar1_residual: bool,
    pub use_mahalanobis: bool,
    pub use_ks_rolling: bool,
    // Tier-E baselines (Session 8 — debugging-specific statistical).
    pub use_poisson_burst: bool,
    pub use_saturation_chain: bool,
    pub use_chi_squared_prop: bool,
    // Tier-F baselines (Session 8 — neuroscience burst detectors translated
    // to inter-event-interval domain).
    pub use_max_interval_burst: bool,
    pub use_log_isi_burst: bool,
    pub use_rank_surprise_burst: bool,
    pub use_misi_burst: bool,
    pub use_dsfb_structural: bool,

    // Phase 5 family-level flags. Each enables a family of detectors
    // implemented in `incumbent_baselines.rs`. Family-flag granularity is
    // a deliberate trade-off — see `docs/fusion_design.md` for the per-
    // detector list. Each family's detectors contribute their standalone
    // metric to `per_detector`; cell-level / window-level consensus
    // contribution is documented per-family in fusion_design.md.
    pub use_tier_g_concept_drift: bool,
    pub use_tier_h_distribution_shift: bool,
    pub use_tier_i_robust_nonparametric: bool,
    pub use_tier_j_forecast_residual: bool,
    pub use_tier_k_frequency: bool,
    pub use_tier_l_multivariate: bool,
    pub use_tier_m_debugging_native: bool,
    pub use_tier_n_offline_cpd: bool,
    pub use_tier_o_rare_changepoint: bool,
    pub use_tier_p_streaming_sequential: bool,
    pub use_tier_q_concept_drift_rarer: bool,
    pub use_tier_r_robust_depth: bool,
    pub use_tier_s_count_event: bool,
    pub use_tier_t_info_theoretic: bool,
    pub use_tier_u_dynamical_systems: bool,

    // Phase 5 wave (Session 9) — new families V-AA.
    pub use_tier_v_industrial_fdd: bool,
    pub use_tier_x_climate_homogeneity: bool,
    pub use_tier_y_dispersion_rank: bool,
    pub use_tier_z_circular: bool,
    pub use_tier_aa_nonlinear_ts: bool,

    // Per-family hyperparameters.
    pub family_default_win_n: usize,    // default rolling-window size for new detectors
    pub family_default_k: f64,          // default k-sigma threshold

    // === Phase 1 bank-aware-fusion (panel directions #1, #4, #7) ===
    /// If true, the typed-confirmation gate consults each motif's
    /// per-provenance + per-correlation-count effective consensus
    /// threshold (`HeuristicsBank::effective_min_consensus`) instead of
    /// the global `min_consensus`. FieldValidated motifs trust experience
    /// and lower the threshold by 1; FrameworkDesign motifs require an
    /// extra +1; multi-service motifs (`min_correlation_count >= 3`) get
    /// an additional +1 regardless of provenance.
    pub use_bank_aware_consensus: bool,
    /// Episodes whose top-vs-runner-up `MatchConfidence.margin` falls
    /// below this gate are reported as `ambiguous_typed_episodes` rather
    /// than `consensus_confirmed_typed_episodes`. Default 0.30 — match
    /// dominates by 30% of top-score. Set to 0.0 to disable margin
    /// gating (every typed episode passes).
    pub margin_gate: f64,
    /// Phase 2 (Direction #2 — tier-affinity matrix). When true, the
    /// typed-confirmation step uses
    /// `bank.match_episode_with_tier_affinity(...)` instead of
    /// `match_episode_with_consensus(...)`. The new method scores each
    /// candidate motif using a tier-restricted consensus computed from
    /// per-cell + per-window tier-fired bitmasks (only the 23 old
    /// grid-filled detectors contribute tier bits; the 138 new
    /// family-level detectors remain in per_detector but do not
    /// populate tier_mask in this revision). Determinism preserved.
    pub use_tier_affinity: bool,

    // ===== Phase η.4 — full 9-axis ablation toggles =====
    //
    // Each flag controls one of the six remaining bank-aware fusion
    // axes (4-9 of the 9-axis ladder). Default `true` preserves
    // pre-Phase-η.4 behaviour exactly. Setting any flag to `false`
    // disables that single axis for ablation studies.
    //
    // Axes 1-3 are already toggleable via `use_bank_aware_consensus`
    // (axis 1), `margin_gate=0.0` (axis 2), and `use_tier_affinity`
    // (axis 3).
    /// Axis 4 — Zero-tier-firing filter. Inside the bank's
    /// tier-affinity matcher, motifs whose affinity tiers are all
    /// silent in the episode range are filtered out. Disabling
    /// admits motifs even when no affinity tiers fired.
    pub use_zero_tier_filter: bool,
    /// Axis 5 — Adaptive margin gate (Path 3). Halves the margin
    /// gate when `tier_consensus_factor > 0.5`. Disabling restores
    /// the constant `margin_gate` regardless of tier evidence.
    pub use_adaptive_margin_gate: bool,
    /// Axis 6 — Confuser-boundary adjudication. When the matched
    /// motif declares a confuser, the typed-confirmation gate
    /// requires the margin against the confuser to exceed the
    /// motif's `margin_vs_confuser_threshold`. Disabling vacuously
    /// passes confuser-margin checks.
    pub use_confuser_boundary: bool,
    /// Axis 7 — Structural disambiguator boost (Phase 6). Inside
    /// the bank's tier-affinity matcher, the matched motif's score
    /// receives a multiplicative boost proportional to the fraction
    /// of distinguishing tiers (motif affinity AND NOT confuser
    /// affinity) firing in the episode. Disabling skips the boost.
    pub use_disambiguator_boost: bool,
    /// Axis 8 — Tier-level primary witness gate (Phase 7). Inside
    /// the bank's tier-affinity matcher, motifs whose declared
    /// `primary_witness_tiers` are silent in the episode range are
    /// filtered out. Disabling admits motifs without their named
    /// witness tiers firing.
    pub use_primary_witness_tier_gate: bool,
    /// Axis 9 — Per-detector named witness gate (Phase 8). At the
    /// fusion-side typed-confirmation step, the matched motif's
    /// `primary_witness_detectors` (named detectors) must include
    /// at least one that fired in the episode range. Disabling
    /// vacuously passes the named-detector witness check.
    pub use_primary_witness_detector_gate: bool,

    pub scalar_k: f64,
    pub cusum_h: f64,
    pub ewma_lambda: f64,
    pub ewma_l: f64,
    pub robust_z_k: f64,
    pub page_hinkley_lambda: f64,
    pub page_hinkley_delta: f64,
    pub tukey_iqr_k: f64,
    pub sr_rolling_n: usize,
    pub sr_k: f64,
    pub mp_seq_len: usize,
    pub mp_k: f64,
    pub bocpd_run_length: f64,
    pub bocpd_theta: f64,
    pub iso_n_trees: usize,
    pub iso_sample_size: usize,
    pub iso_seed: u64,
    pub lof_k: usize,
    pub lof_theta: f64,

    // Tier-D detector hyperparameters.
    pub mk_win_n: usize,
    pub mk_z_alpha: f64,
    pub rolling_z_win_n: usize,
    pub rolling_z_k: f64,
    pub ar1_k: f64,
    pub mahalanobis_k: f64,
    pub ks_win_n: usize,
    pub ks_crit_d: f64,

    pub poisson_k: f64,
    pub saturation_chain_k: f64,
    pub saturation_chain_n: usize,
    pub chi_sq_win_n: usize,
    pub chi_sq_crit: f64,

    pub burst_event_k: f64,
    pub mi_max_start_isi: usize,
    pub mi_max_burst_isi: usize,
    pub mi_min_n_in_burst: usize,
    pub log_isi_min_n_in_burst: usize,
    pub rs_win_n_isis: usize,
    pub rs_z_alpha: f64,
    pub misi_rolling_n: usize,
    pub misi_factor_k: f64,

    /// Phase ζ.5 (Session 17) — per-detector consensus weight overrides.
    ///
    /// Sparse list of `(detector_name, weight)` entries; detector names
    /// not listed default to weight 1 (unit consensus contribution,
    /// preserving Phase 8 behaviour). Weight 0 fully suppresses the
    /// detector's contribution to both `cell_consensus` and the per-
    /// cell / per-window tier-evidence bitmasks consulted by the bank's
    /// affinity scoring. Weight 2 boosts the detector's increment.
    ///
    /// Default `None` ⇒ unit weights everywhere (no behavioural
    /// change). Set to `Some(canonical_calibrated_weight_overrides())`
    /// for the audit-derived calibrated weights; the LO-CV harness
    /// gates the calibrated default by `audit::loo_cv::refinement_passes_gate`
    /// before promoting it to the canonical default.
    ///
    /// Theorem 9 preservation: weights are static; the lookup is a
    /// linear scan of a fixed-size const slice; no floating-point or
    /// non-deterministic operation is introduced.
    pub detector_weight_overrides: Option<&'static [(&'static str, u8)]>,
}

impl FusionConfig {
    pub const ALL_DEFAULT: FusionConfig = FusionConfig {
        min_consensus: 3,
        use_scalar: true,
        use_cusum: true,
        use_ewma: true,
        use_robust_z: true,
        use_page_hinkley: true,
        use_tukey_iqr: true,
        use_spectral_residual: true,
        use_matrix_profile: true,
        use_bocpd: true,
        use_isolation_forest: true,
        use_lof: true,
        use_mann_kendall: true,
        use_rolling_z: true,
        use_ar1_residual: true,
        use_mahalanobis: true,
        use_ks_rolling: true,
        use_dsfb_structural: true,
        scalar_k: 3.0,
        cusum_h: 4.0,
        ewma_lambda: 0.2,
        ewma_l: 3.0,
        robust_z_k: 3.0,
        page_hinkley_lambda: 50.0,
        page_hinkley_delta: 0.005,
        tukey_iqr_k: 1.5,
        sr_rolling_n: 8,
        sr_k: 3.0,
        mp_seq_len: 4,
        mp_k: 3.0,
        bocpd_run_length: 100.0,
        bocpd_theta: 0.5,
        iso_n_trees: 16,
        iso_sample_size: 64,
        iso_seed: 0x9E3779B97F4A7C15,
        lof_k: 5,
        lof_theta: 1.5,
        mk_win_n: 20,
        mk_z_alpha: 1.96,
        rolling_z_win_n: 30,
        rolling_z_k: 3.0,
        ar1_k: 3.0,
        mahalanobis_k: 3.0,
        ks_win_n: 20,
        ks_crit_d: 0.4,
        use_poisson_burst: true,
        use_saturation_chain: true,
        use_chi_squared_prop: true,
        use_max_interval_burst: true,
        use_log_isi_burst: true,
        use_rank_surprise_burst: true,
        use_misi_burst: true,
        poisson_k: 4.0,
        saturation_chain_k: 2.0,
        saturation_chain_n: 4,
        chi_sq_win_n: 10,
        chi_sq_crit: 3.84,
        burst_event_k: 1.0,
        mi_max_start_isi: 5,
        mi_max_burst_isi: 8,
        mi_min_n_in_burst: 3,
        log_isi_min_n_in_burst: 3,
        rs_win_n_isis: 5,
        rs_z_alpha: 1.96,
        misi_rolling_n: 10,
        misi_factor_k: 0.3,
        // Phase 5 family flags — all enabled by default.
        use_tier_g_concept_drift: true,
        use_tier_h_distribution_shift: true,
        use_tier_i_robust_nonparametric: true,
        use_tier_j_forecast_residual: true,
        use_tier_k_frequency: true,
        use_tier_l_multivariate: true,
        use_tier_m_debugging_native: true,
        use_tier_n_offline_cpd: true,
        use_tier_o_rare_changepoint: true,
        use_tier_p_streaming_sequential: true,
        use_tier_q_concept_drift_rarer: true,
        use_tier_r_robust_depth: true,
        use_tier_s_count_event: true,
        use_tier_t_info_theoretic: true,
        use_tier_u_dynamical_systems: true,
        // Phase 5 wave — all enabled by default.
        use_tier_v_industrial_fdd: true,
        use_tier_x_climate_homogeneity: true,
        use_tier_y_dispersion_rank: true,
        use_tier_z_circular: true,
        use_tier_aa_nonlinear_ts: true,
        family_default_win_n: 30,
        family_default_k: 4.0,
        use_bank_aware_consensus: true,
        margin_gate: 0.30,
        use_tier_affinity: true,
        // Phase η.4 — full 9-axis ablation toggles. Defaults preserve
        // pre-Phase-η.4 behaviour exactly.
        use_zero_tier_filter: true,
        use_adaptive_margin_gate: true,
        use_confuser_boundary: true,
        use_disambiguator_boost: true,
        use_primary_witness_tier_gate: true,
        use_primary_witness_detector_gate: true,
        detector_weight_overrides: None,
    };

    pub const ALL_FOUR_DEFAULT: FusionConfig = FusionConfig {
        min_consensus: 2,
        use_robust_z: false,
        use_page_hinkley: false,
        use_tukey_iqr: false,
        use_spectral_residual: false,
        use_matrix_profile: false,
        use_bocpd: false,
        use_isolation_forest: false,
        use_lof: false,
        use_mann_kendall: false,
        use_rolling_z: false,
        use_ar1_residual: false,
        use_mahalanobis: false,
        use_ks_rolling: false,
        use_poisson_burst: false,
        use_saturation_chain: false,
        use_chi_squared_prop: false,
        use_max_interval_burst: false,
        use_log_isi_burst: false,
        use_rank_surprise_burst: false,
        use_misi_burst: false,
        use_tier_g_concept_drift: false,
        use_tier_h_distribution_shift: false,
        use_tier_i_robust_nonparametric: false,
        use_tier_j_forecast_residual: false,
        use_tier_k_frequency: false,
        use_tier_l_multivariate: false,
        use_tier_m_debugging_native: false,
        use_tier_n_offline_cpd: false,
        use_tier_o_rare_changepoint: false,
        use_tier_p_streaming_sequential: false,
        use_tier_q_concept_drift_rarer: false,
        use_tier_r_robust_depth: false,
        use_tier_s_count_event: false,
        use_tier_t_info_theoretic: false,
        use_tier_u_dynamical_systems: false,
        use_tier_v_industrial_fdd: false,
        use_tier_x_climate_homogeneity: false,
        use_tier_y_dispersion_rank: false,
        use_tier_z_circular: false,
        use_tier_aa_nonlinear_ts: false,
        ..Self::ALL_DEFAULT
    };

    /// Phase ζ.5 — per-detector consensus weight lookup.
    ///
    /// Returns the weight for the named detector. Default: 1 (unit
    /// weight). When `detector_weight_overrides` is `Some(slice)`,
    /// looks up the name; falls back to 1 if not found in the list.
    /// Weight 0 fully suppresses the detector's contribution (skip
    /// `cell_consensus` increment, skip `cell_tier_mask` /
    /// `window_tier_mask` update, skip `all_detector_alerts` insert).
    ///
    /// Linear scan; the override list is bounded (≤ 205 entries).
    /// Deterministic: const-time given the input slice.
    pub fn weight_for(&self, name: &str) -> u8 {
        if let Some(overrides) = self.detector_weight_overrides {
            for (n, w) in overrides {
                if *n == name { return *w; }
            }
        }
        1
    }

    pub fn detectors_used(&self) -> u8 {
        ((self.use_scalar as u8)
            + (self.use_cusum as u8)
            + (self.use_ewma as u8)
            + (self.use_robust_z as u8)
            + (self.use_page_hinkley as u8)
            + (self.use_tukey_iqr as u8)
            + (self.use_spectral_residual as u8)
            + (self.use_matrix_profile as u8)
            + (self.use_bocpd as u8)
            + (self.use_isolation_forest as u8)
            + (self.use_lof as u8)
            + (self.use_mann_kendall as u8)
            + (self.use_rolling_z as u8)
            + (self.use_ar1_residual as u8)
            + (self.use_mahalanobis as u8)
            + (self.use_ks_rolling as u8)
            + (self.use_poisson_burst as u8)
            + (self.use_saturation_chain as u8)
            + (self.use_chi_squared_prop as u8)
            + (self.use_max_interval_burst as u8)
            + (self.use_log_isi_burst as u8)
            + (self.use_rank_surprise_burst as u8)
            + (self.use_misi_burst as u8)
            + (self.use_dsfb_structural as u8)).saturating_add(self.tier_detector_count())
    }

    /// Count detectors enabled across the 15 Session-8.6 family flags.
    /// Each family multiplied by its detector count. Capped to u8 max.
    pub fn tier_detector_count(&self) -> u8 {
        let n: u32 = (if self.use_tier_g_concept_drift { 9 } else { 0 })
            + (if self.use_tier_h_distribution_shift { 10 } else { 0 })
            + (if self.use_tier_i_robust_nonparametric { 10 } else { 0 })
            + (if self.use_tier_j_forecast_residual { 10 } else { 0 })
            + (if self.use_tier_k_frequency { 10 } else { 0 })
            + (if self.use_tier_l_multivariate { 9 } else { 0 })
            + (if self.use_tier_m_debugging_native { 18 } else { 0 })
            + (if self.use_tier_n_offline_cpd { 8 } else { 0 })
            + (if self.use_tier_o_rare_changepoint { 10 } else { 0 })
            + (if self.use_tier_p_streaming_sequential { 9 } else { 0 })
            + (if self.use_tier_q_concept_drift_rarer { 10 } else { 0 })
            + (if self.use_tier_r_robust_depth { 8 } else { 0 })
            + (if self.use_tier_s_count_event { 3 } else { 0 })
            + (if self.use_tier_t_info_theoretic { 6 } else { 0 })
            + (if self.use_tier_u_dynamical_systems { 8 } else { 0 })
            + (if self.use_tier_v_industrial_fdd { 8 } else { 0 })
            + (if self.use_tier_x_climate_homogeneity { 10 } else { 0 })
            + (if self.use_tier_y_dispersion_rank { 10 } else { 0 })
            + (if self.use_tier_z_circular { 10 } else { 0 })
            + (if self.use_tier_aa_nonlinear_ts { 5 } else { 0 });
        n.min(255) as u8
    }
}

#[derive(Debug, Clone)]
pub struct FusionMetrics {
    pub fixture_name: &'static str,
    pub min_consensus: u8,
    pub detectors_used: u8,

    pub raw_alert_count: u64,
    pub consensus_alert_count: u64,
    pub consensus_alert_windows: u64,
    pub fusion_episode_count: u64,
    pub fusion_rscr: f64,
    pub fusion_fault_recall: f64,
    pub fusion_clean_window_fp_rate: f64,

    pub consensus_confirmed_typed_episodes: u64,
    pub consensus_filtered_out_episodes: u64,
    pub consensus_confirmed_clean_fp_rate: f64,

    /// Bank-aware-fusion (Phase 1) — typed episodes whose match-
    /// confidence margin is below `FusionConfig::margin_gate`. These are
    /// reported separately from `consensus_confirmed_typed_episodes` so
    /// the operator can route low-margin matches to a "needs-review"
    /// queue rather than commit to one motif label. Counted only when
    /// the per-motif consensus gate already passed.
    pub ambiguous_typed_episodes: u64,
    /// Bank-aware-fusion — episodes filtered out specifically by the
    /// per-motif consensus check (over and above the global filter).
    /// Distinguishes "your motif needs more agreement than this episode
    /// gathered" from "no detector agreement at all".
    pub bank_aware_filtered_out: u64,
    /// Phase 5.6 — episodes that beat the runner-up motif but failed the
    /// confuser-margin gate (top motif's score did not exceed its
    /// declared confuser by `margin_vs_confuser_threshold`). Routed to a
    /// "needs disambiguation" operator queue rather than confidently
    /// typed. Disjoint from `ambiguous_typed_episodes`.
    pub confuser_ambiguous_episodes: u64,
    /// Phase 5.6 — operator-cost objective:
    /// `typed_confirmed - 0.5*ambiguous - 0.5*confuser_ambiguous - 10*FP_rate`.
    /// Higher is better. Captures the trade-off between high-confidence
    /// typing, ambiguity surface, and false-positive cost.
    pub operator_score: f64,

    pub deterministic_replay_holds: bool,

    pub per_detector: Vec<DetectorOutput>,
    pub dsfb_structural: Option<BenchmarkMetrics>,

    /// Per-episode bank-aware confidence — one entry per
    /// consensus-confirmed typed DSFB episode, ordered by episode id.
    /// Each entry is the result of
    /// `bank.match_episode_with_consensus(...)` with the per-episode
    /// max consensus count threaded through. Operators reading this
    /// see "this episode is CascadingTimeoutSlew with margin 0.74,
    /// runner-up DeploymentRegressionSlew, max consensus 8/12".
    pub per_episode_confidence: Vec<MatchConfidence>,

    /// Phase ζ.4 — per-episode observed tier-firing bitmask.
    ///
    /// One entry per closed episode (parallel to
    /// `per_episode_confidence`). Each value is the OR over all
    /// per-cell `cell_tier_mask[w*S+s]` and per-window
    /// `window_tier_mask[w]` bits during that episode's window range.
    /// Used by `audit::motif_refinement` to compare hand-curated
    /// `affinity_tiers` against the empirically-observed tier
    /// signature on each typed episode.
    pub per_episode_tier_mask: Vec<u32>,

    /// Phase ζ.8 — per-episode top-K detector firings (top 5).
    ///
    /// One entry per closed episode (parallel to
    /// `per_episode_confidence`). Each entry: list of
    /// `(detector_name, fired_window_count)` pairs sorted descending
    /// by fired count, capped at 5. Used by
    /// `audit::motif_refinement` to compare hand-curated
    /// `named_witnesses` against the empirically-observed top
    /// witnesses on each typed episode.
    pub per_episode_top_witnesses: Vec<Vec<(&'static str, u64)>>,
}

pub fn run_fusion_evaluation<const S: usize, const M: usize>(
    engine: &DsfbDebugEngine<S, M>,
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    config: &FusionConfig,
    fixture_name: &'static str,
) -> Result<FusionMetrics> {
    let metrics = run_inner(engine, data, num_signals, num_windows,
        healthy_window_end, fault_labels, config, fixture_name)?;
    let replay = run_inner(engine, data, num_signals, num_windows,
        healthy_window_end, fault_labels, config, fixture_name)?;
    let det = (metrics.fusion_episode_count == replay.fusion_episode_count)
        && (metrics.consensus_alert_count == replay.consensus_alert_count)
        && (metrics.consensus_confirmed_typed_episodes
            == replay.consensus_confirmed_typed_episodes)
        && (metrics.ambiguous_typed_episodes
            == replay.ambiguous_typed_episodes)
        && (metrics.bank_aware_filtered_out
            == replay.bank_aware_filtered_out)
        && (metrics.confuser_ambiguous_episodes
            == replay.confuser_ambiguous_episodes)
        && (metrics.consensus_alert_windows == replay.consensus_alert_windows)
        // Phase ζ.4 + ζ.8 — telemetry must replay identically.
        && (metrics.per_episode_tier_mask == replay.per_episode_tier_mask)
        && (metrics.per_episode_top_witnesses == replay.per_episode_top_witnesses);
    let mut out = metrics;
    out.deterministic_replay_holds = det;
    Ok(out)
}

fn run_inner<const S: usize, const M: usize>(
    engine: &DsfbDebugEngine<S, M>,
    data: &[f64],
    num_signals: usize,
    num_windows: usize,
    healthy_window_end: usize,
    fault_labels: &[bool],
    config: &FusionConfig,
    fixture_name: &'static str,
) -> Result<FusionMetrics> {
    let pred_w = engine.config().episode_precision_window;
    let mut per_detector: Vec<DetectorOutput> = Vec::new();
    let total = num_signals * num_windows;

    // Cell-level (per-(w, s)) consensus grid: each cell stores the
    // count of cell-level detectors that fired.
    let mut cell_consensus = std::vec![0_u8; total];
    // Window-level consensus boost from multivariate detectors.
    let mut window_boost = std::vec![0_u8; num_windows];

    // Phase 8 — per-detector window-alert capture for primary-witness
    // checks. Map keyed by detector_name; values are Vec<bool> over
    // num_windows. BTreeMap preserves deterministic iteration order
    // (Theorem 9 across replay).
    let mut all_detector_alerts: std::collections::BTreeMap<&'static str, Vec<bool>>
        = std::collections::BTreeMap::new();

    // Helper closure: derive per-window alerts from a cell-level grid
    // by OR-ing across signals. Used by old-detector branches.
    let cell_grid_to_win_alerts = |grid: &[bool]| -> Vec<bool> {
        let mut wa = std::vec![false; num_windows];
        for w in 0..num_windows {
            for s in 0..num_signals {
                let idx = w * num_signals + s;
                if idx < grid.len() && grid[idx] { wa[w] = true; break; }
            }
        }
        wa
    };
    // Phase 2: per-cell + per-window tier-fired bitmasks. Each bit
    // represents one tier (A–F + Extra + G–U). Populated by the
    // grid-filled old detectors only (see `fusion_design.md` honest
    // note); the 138 family-level new detectors contribute their
    // standalone metric to per_detector but not to tier_mask in this
    // revision.
    let mut cell_tier_mask  = std::vec![0_u32; total];
    let mut window_tier_mask = std::vec![0_u32; num_windows];

    let _ = TIER_BIT_A; // imports are at module level via const TIER_BITS_OLD_DETECTORS
    if config.use_scalar {
        let out = scalar_threshold(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_scalar_grid(data, num_signals, num_windows, healthy_window_end,
            config.scalar_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_A; }
        }
    }
    if config.use_cusum {
        let out = cusum(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.cusum_h);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_cusum_grid(data, num_signals, num_windows, healthy_window_end,
            config.cusum_h, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_A; }
        }
        all_detector_alerts.insert("cusum", cell_grid_to_win_alerts(&grid));
    }
    if config.use_ewma {
        let out = ewma(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.ewma_lambda, config.ewma_l);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_ewma_grid(data, num_signals, num_windows, healthy_window_end,
            config.ewma_lambda, config.ewma_l, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_A; }
        }
    }
    if config.use_robust_z {
        let out = robust_z_mad(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.robust_z_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_robust_z_grid(data, num_signals, num_windows, healthy_window_end,
            config.robust_z_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_B; }
        }
    }
    if config.use_page_hinkley {
        let out = page_hinkley(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.page_hinkley_lambda,
            config.page_hinkley_delta);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_page_hinkley_grid(data, num_signals, num_windows, healthy_window_end,
            config.page_hinkley_lambda, config.page_hinkley_delta, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_B; }
        }
        all_detector_alerts.insert("page_hinkley", cell_grid_to_win_alerts(&grid));
    }
    if config.use_tukey_iqr {
        let out = tukey_iqr_fence(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w, config.tukey_iqr_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_tukey_grid(data, num_signals, num_windows, healthy_window_end,
            config.tukey_iqr_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_B; }
        }
    }
    if config.use_spectral_residual {
        let out = spectral_residual_td(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w,
            config.sr_rolling_n, config.sr_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_sr_grid(data, num_signals, num_windows, healthy_window_end,
            config.sr_rolling_n, config.sr_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_C; }
        }
    }
    // Window-level multivariate detectors → contribute to window_boost.
    if config.use_matrix_profile {
        let out = matrix_profile(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w,
            config.mp_seq_len, config.mp_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
    }
    if config.use_bocpd {
        let out = bocpd(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.bocpd_run_length, config.bocpd_theta);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_bocpd_grid(data, num_signals, num_windows, healthy_window_end,
            config.bocpd_run_length, config.bocpd_theta, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_C; }
        }
    }
    if config.use_isolation_forest {
        let out = isolation_forest(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w,
            config.iso_n_trees, config.iso_sample_size, config.iso_seed);
        let __w = config.weight_for(out.detector_name);
        let _ = out;
        let mut win_grid = std::vec![false; num_windows];
        fill_iso_window_grid(data, num_signals, num_windows,
            healthy_window_end, config.iso_n_trees, config.iso_sample_size,
            config.iso_seed, &mut win_grid);
        for w in 0..num_windows {
            if win_grid[w] && __w > 0 { window_boost[w] += __w; window_tier_mask[w] |= TIER_BIT_C; }
        }
    }
    if config.use_lof {
        let out = lof(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.lof_k, config.lof_theta);
        let __w = config.weight_for(out.detector_name);
        let _ = out;
        let mut win_grid = std::vec![false; num_windows];
        fill_lof_window_grid(data, num_signals, num_windows,
            healthy_window_end, config.lof_k, config.lof_theta, &mut win_grid);
        for w in 0..num_windows {
            if win_grid[w] && __w > 0 { window_boost[w] += __w; window_tier_mask[w] |= TIER_BIT_C; }
        }
    }
    // Tier-D detectors (Session 8).
    if config.use_mann_kendall {
        let out = mann_kendall(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.mk_win_n, config.mk_z_alpha);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_mk_grid(data, num_signals, num_windows, config.mk_win_n,
            config.mk_z_alpha, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_D; }
        }
        all_detector_alerts.insert("mann_kendall", cell_grid_to_win_alerts(&grid));
    }
    if config.use_rolling_z {
        let out = rolling_z_score(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w,
            config.rolling_z_win_n, config.rolling_z_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_rolling_z_grid(data, num_signals, num_windows,
            config.rolling_z_win_n, config.rolling_z_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_D; }
        }
    }
    if config.use_ar1_residual {
        let out = ar1_forecast_residual(data, num_signals, num_windows,
            healthy_window_end, fault_labels, pred_w, config.ar1_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_ar1_grid(data, num_signals, num_windows, healthy_window_end,
            config.ar1_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_D; }
        }
    }
    if config.use_mahalanobis {
        let out = mahalanobis(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.mahalanobis_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut win_grid = std::vec![false; num_windows];
        fill_mahalanobis_window_grid(data, num_signals, num_windows,
            healthy_window_end, config.mahalanobis_k, &mut win_grid);
        for w in 0..num_windows {
            if win_grid[w] && __w > 0 { window_boost[w] += __w; window_tier_mask[w] |= TIER_BIT_D; }
        }
    }
    if config.use_ks_rolling {
        let out = ks_rolling(data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.ks_win_n, config.ks_crit_d);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_ks_grid(data, num_signals, num_windows, healthy_window_end,
            config.ks_win_n, config.ks_crit_d, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_D; }
        }
    }
    // Tier-E debugging-specific detectors (Session 8).
    if config.use_poisson_burst {
        let out = crate::incumbent_baselines::poisson_burst(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.poisson_k);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_poisson_grid(data, num_signals, num_windows, healthy_window_end,
            config.poisson_k, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_E; }
        }
        all_detector_alerts.insert("poisson_burst", cell_grid_to_win_alerts(&grid));
    }
    if config.use_saturation_chain {
        let out = crate::incumbent_baselines::saturation_chain(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.saturation_chain_k,
            config.saturation_chain_n);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_saturation_chain_grid(data, num_signals, num_windows,
            healthy_window_end, config.saturation_chain_k,
            config.saturation_chain_n, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_E; }
        }
    }
    if config.use_chi_squared_prop {
        let out = crate::incumbent_baselines::chi_squared_proportion(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.chi_sq_win_n, config.chi_sq_crit);
        per_detector.push(out);
        let __w = config.weight_for(out.detector_name);
        let mut grid = std::vec![false; total];
        fill_chi_sq_grid(data, num_signals, num_windows, healthy_window_end,
            config.chi_sq_win_n, config.chi_sq_crit, &mut grid);
        for i in 0..total {
            if grid[i] && __w > 0 { cell_consensus[i] += __w; cell_tier_mask[i] |= TIER_BIT_E; }
        }
    }
    // Tier-F neuroscience-derived burst detectors. Each operates on
    // events (windows where |x-μ|>burst_event_k·σ); contribution to
    // per-(w, s) consensus is approximated via the detector's own
    // alert-window output (which already covers the burst span).
    if config.use_max_interval_burst {
        let out = crate::incumbent_baselines::max_interval_burst(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.burst_event_k,
            config.mi_max_start_isi, config.mi_max_burst_isi,
            config.mi_min_n_in_burst);
        per_detector.push(out);
    }
    if config.use_log_isi_burst {
        let out = crate::incumbent_baselines::log_isi_burst(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.burst_event_k,
            config.log_isi_min_n_in_burst);
        per_detector.push(out);
    }
    if config.use_rank_surprise_burst {
        let out = crate::incumbent_baselines::rank_surprise_burst(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.burst_event_k,
            config.rs_win_n_isis, config.rs_z_alpha);
        per_detector.push(out);
    }
    if config.use_misi_burst {
        let out = crate::incumbent_baselines::misi_burst(
            data, num_signals, num_windows, healthy_window_end,
            fault_labels, pred_w, config.burst_event_k,
            config.misi_rolling_n, config.misi_factor_k);
        per_detector.push(out);
    }

    // ================================================================
    // SESSION 8.6 — TIER G–U family-level invocations.
    // Each family flag triggers all detectors in that family. Each
    // detector's standalone metric is recorded in `per_detector`. Cell-
    // and window-level consensus contribution from these detectors is
    // documented as a deferred follow-up in `docs/fusion_design.md`
    // (run_inner currently uses the existing 16-cell + 4-window
    // consensus; the new 138 detectors are reported but do not vote
    // in the consensus arithmetic — this is honest, not deferred —
    // they remain visible in per-detector output for operator audit).
    // ================================================================
    use crate::incumbent_baselines as ib;
    let win_n = config.family_default_win_n;
    let k = config.family_default_k;

    // Phase 4 — fold each detector's per-window alerts into window_tier_mask
    // via the LAST_WIN_ALERTS thread-local side channel. Each family flag
    // pushes its detectors' standalone metrics AND populates tier evidence
    // at the window level.
    macro_rules! push_tier {
        ($out:expr, $tier:expr) => {
            {
                let __out = $out; // forces evaluation BEFORE LAST_WIN_ALERTS read
                let __name: &'static str = __out.detector_name;
                // Phase ζ.5 — consult per-detector weight override.
                // Weight 0 suppresses the detector's contribution to
                // window_tier_mask + all_detector_alerts entirely.
                // The standalone DetectorOutput is still pushed for
                // audit telemetry (so the selectivity audit still
                // sees this detector's behaviour even when filtered).
                let __family_w: u8 = config.weight_for(__name);
                per_detector.push(__out);
                if __family_w > 0 {
                    ib::LAST_WIN_ALERTS.with(|c| {
                        let buf = c.borrow();
                        let lim = num_windows.min(buf.len());
                        for w in 0..lim {
                            if buf[w] { window_tier_mask[w] |= $tier; }
                        }
                        // Phase 8: capture per-detector window alerts for
                        // primary-witness check at typed-confirmation time.
                        let mut wa = std::vec![false; num_windows];
                        for w in 0..lim { wa[w] = buf[w]; }
                        all_detector_alerts.insert(__name, wa);
                    });
                }
            }
        };
    }
    use crate::heuristics_bank::{
        TIER_BIT_G, TIER_BIT_H, TIER_BIT_I, TIER_BIT_J, TIER_BIT_K,
        TIER_BIT_L, TIER_BIT_M, TIER_BIT_N, TIER_BIT_O, TIER_BIT_P,
        TIER_BIT_Q, TIER_BIT_R, TIER_BIT_S, TIER_BIT_T, TIER_BIT_U,
    };

    if config.use_tier_g_concept_drift {
        push_tier!(ib::shiryaev_roberts(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 50.0), TIER_BIT_G);
        push_tier!(ib::ddm(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5), TIER_BIT_G);
        push_tier!(ib::eddm(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5), TIER_BIT_G);
        push_tier!(ib::hddm_a(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, 0.001), TIER_BIT_G);
        push_tier!(ib::hddm_w(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, 0.05, 0.001), TIER_BIT_G);
        push_tier!(ib::stepd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, win_n, 6.63), TIER_BIT_G);
        push_tier!(ib::ecdd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, 0.2, 2.5), TIER_BIT_G);
        push_tier!(ib::kswin(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 100, 0.005), TIER_BIT_G);
        push_tier!(ib::fhddm(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, 25, 1e-7), TIER_BIT_G);
    }

    if config.use_tier_h_distribution_shift {
        push_tier!(ib::wasserstein_1d(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.3), TIER_BIT_H);
        push_tier!(ib::jensen_shannon(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.1), TIER_BIT_H);
        push_tier!(ib::kl_divergence(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_H);
        push_tier!(ib::psi(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.25), TIER_BIT_H);
        push_tier!(ib::anderson_darling(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_H);
        push_tier!(ib::cramer_von_mises(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_H);
        push_tier!(ib::energy_distance(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.3), TIER_BIT_H);
        push_tier!(ib::mmd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.05), TIER_BIT_H);
        push_tier!(ib::bhattacharyya(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.2), TIER_BIT_H);
        push_tier!(ib::hellinger(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.3), TIER_BIT_H);
    }

    if config.use_tier_i_robust_nonparametric {
        push_tier!(ib::median_absolute_slope(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 5.0), TIER_BIT_I);
        push_tier!(ib::theil_sen_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_I);
        push_tier!(ib::sen_slope_changepoint(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.05), TIER_BIT_I);
        push_tier!(ib::moods_median_rolling(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 6.63), TIER_BIT_I);
        push_tier!(ib::brown_forsythe(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 4.0), TIER_BIT_I);
        push_tier!(ib::levene_variance(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 4.0), TIER_BIT_I);
        push_tier!(ib::sign_test_drift(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_I);
        push_tier!(ib::runs_test(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_I);
        push_tier!(ib::wald_wolfowitz_two_sample(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_I);
        push_tier!(ib::sequential_rank(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_I);
    }

    if config.use_tier_j_forecast_residual {
        push_tier!(ib::ses_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.3, k), TIER_BIT_J);
        push_tier!(ib::holt_linear(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.3, 0.1, k), TIER_BIT_J);
        push_tier!(ib::holt_winters(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.3, 0.1, 0.3, 24, k), TIER_BIT_J);
        push_tier!(ib::ar2_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_J);
        push_tier!(ib::arima_simplified(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_J);
        push_tier!(ib::kalman_innovation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.01, 1.0, k), TIER_BIT_J);
        push_tier!(ib::savitzky_golay_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_J);
        push_tier!(ib::stl_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 24, 51, k), TIER_BIT_J);
        push_tier!(ib::prophet_simplified(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 24, k), TIER_BIT_J);
        push_tier!(ib::naive_seasonal(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 24, k), TIER_BIT_J);
    }

    if config.use_tier_k_frequency {
        push_tier!(ib::fft_band_energy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 0.3), TIER_BIT_K);
        push_tier!(ib::welch_psd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, k), TIER_BIT_K);
        push_tier!(ib::wavelet_haar(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, k), TIER_BIT_K);
        push_tier!(ib::autocorrelation_peak(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 0.3), TIER_BIT_K);
        push_tier!(ib::lomb_scargle(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, k), TIER_BIT_K);
        push_tier!(ib::zero_crossing_rate(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.2), TIER_BIT_K);
        push_tier!(ib::dominant_frequency_drift(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 3), TIER_BIT_K);
        push_tier!(ib::spectral_entropy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 0.5), TIER_BIT_K);
        push_tier!(ib::cepstral_simplified(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 0.5), TIER_BIT_K);
        push_tier!(ib::phase_locking(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 64, 0.5), TIER_BIT_K);
    }

    if config.use_tier_l_multivariate {
        push_tier!(ib::hotelling_t2(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_L);
        push_tier!(ib::mcusum(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0, 0.5), TIER_BIT_L);
        push_tier!(ib::pca_reconstruction(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k, 2), TIER_BIT_L);
        push_tier!(ib::robust_pca(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k, 2), TIER_BIT_L);
        push_tier!(ib::correlation_matrix_distance(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_L);
        push_tier!(ib::partial_correlation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.4), TIER_BIT_L);
        push_tier!(ib::graph_laplacian(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_L);
        push_tier!(ib::canonical_correlation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.4), TIER_BIT_L);
        push_tier!(ib::mutual_information(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 8, 0.5), TIER_BIT_L);
    }

    if config.use_tier_m_debugging_native {
        push_tier!(ib::flap(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 8), TIER_BIT_M);
        push_tier!(ib::sawtooth_ramp(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 5.0), TIER_BIT_M);
        push_tier!(ib::deadband_stuck(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.1), TIER_BIT_M);
        push_tier!(ib::quantization(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 5), TIER_BIT_M);
        push_tier!(ib::plateau(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.05), TIER_BIT_M);
        push_tier!(ib::counter_wrap(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_M);
        push_tier!(ib::monotone_leak(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.85), TIER_BIT_M);
        push_tier!(ib::hysteresis(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 1.5), TIER_BIT_M);
        push_tier!(ib::limit_cycle(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.7), TIER_BIT_M);
        push_tier!(ib::ping_pong(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 25), TIER_BIT_M);
        push_tier!(ib::backpressure(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_M);
        push_tier!(ib::causal_lag(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 5), TIER_BIT_M);
        push_tier!(ib::fan_out(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 3.0, 3), TIER_BIT_M);
        push_tier!(ib::fan_in(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, 4), TIER_BIT_M);
        push_tier!(ib::phase_slip(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 1.5), TIER_BIT_M);
        push_tier!(ib::jitter_bloom(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_M);
        push_tier!(ib::tail_thickening(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 50, 0.10), TIER_BIT_M);
        push_tier!(ib::burst_after_silence(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 20, k), TIER_BIT_M);
    }

    if config.use_tier_n_offline_cpd {
        push_tier!(ib::pelt(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_N);
        push_tier!(ib::binary_segmentation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_N);
        push_tier!(ib::bottom_up_segmentation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 10), TIER_BIT_N);
        push_tier!(ib::window_based_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.5), TIER_BIT_N);
        push_tier!(ib::dynamic_programming_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 10), TIER_BIT_N);
        push_tier!(ib::kernel_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.1), TIER_BIT_N);
        push_tier!(ib::piecewise_linear_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.05), TIER_BIT_N);
        push_tier!(ib::bayesian_offline_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_N);
    }

    if config.use_tier_o_rare_changepoint {
        push_tier!(ib::mosum(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_O);
        push_tier!(ib::narrowest_over_threshold(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_O);
        push_tier!(ib::wbs2(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k, 100), TIER_BIT_O);
        push_tier!(ib::seeded_bs(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_O);
        push_tier!(ib::smuce(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 3.5), TIER_BIT_O);
        push_tier!(ib::fdr_seg(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.05), TIER_BIT_O);
        push_tier!(ib::fpop(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_O);
        push_tier!(ib::tguh(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 3.0), TIER_BIT_O);
        push_tier!(ib::inspect_cpd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_O);
        push_tier!(ib::double_cusum_bs(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_O);
    }

    if config.use_tier_p_streaming_sequential {
        push_tier!(ib::e_detector(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0, 1.0), TIER_BIT_P);
        push_tier!(ib::conformal_martingale(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100.0), TIER_BIT_P);
        push_tier!(ib::exchangeability_martingale(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100.0), TIER_BIT_P);
        push_tier!(ib::power_martingale(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100.0, 0.92), TIER_BIT_P);
        push_tier!(ib::mixture_martingale(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100.0), TIER_BIT_P);
        push_tier!(ib::mixture_sprt(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100.0), TIER_BIT_P);
        push_tier!(ib::scan_statistic(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_P);
        push_tier!(ib::higher_criticism(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_P);
        push_tier!(ib::berk_jones(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_P);
    }

    if config.use_tier_q_concept_drift_rarer {
        push_tier!(ib::mddm_a(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, win_n, 1e-7), TIER_BIT_Q);
        push_tier!(ib::mddm_e(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, win_n, 1e-7), TIER_BIT_Q);
        push_tier!(ib::mddm_g(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.5, win_n, 1e-7), TIER_BIT_Q);
        push_tier!(ib::lfr(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_Q);
        push_tier!(ib::fpdd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 6.63), TIER_BIT_Q);
        push_tier!(ib::optwin(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 4.0), TIER_BIT_Q);
        push_tier!(ib::seqdrift2(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 100, win_n, 3.0), TIER_BIT_Q);
        push_tier!(ib::d3_drift(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.7), TIER_BIT_Q);
        push_tier!(ib::quanttree(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 8, win_n, 5.0), TIER_BIT_Q);
        push_tier!(ib::nn_dvi(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_Q);
    }

    if config.use_tier_r_robust_depth {
        push_tier!(ib::halfspace_depth(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.05), TIER_BIT_R);
        push_tier!(ib::projection_depth(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_R);
        push_tier!(ib::stahel_donoho(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, k), TIER_BIT_R);
        push_tier!(ib::mcd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_R);
        push_tier!(ib::spatial_sign(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.5), TIER_BIT_R);
        push_tier!(ib::s_estimator_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_R);
        push_tier!(ib::depth_rank_control(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.05), TIER_BIT_R);
        push_tier!(ib::outlyingness_median_polish(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_R);
    }

    if config.use_tier_s_count_event {
        push_tier!(ib::bayesian_blocks(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 4.0), TIER_BIT_S);
        push_tier!(ib::index_of_dispersion(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 3.0), TIER_BIT_S);
        push_tier!(ib::allan_variance(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 5, 3.0), TIER_BIT_S);
    }

    if config.use_tier_t_info_theoretic {
        push_tier!(ib::mdl_change(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 5.0), TIER_BIT_T);
        push_tier!(ib::ncd(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.6), TIER_BIT_T);
        push_tier!(ib::lempel_ziv(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_T);
        push_tier!(ib::transfer_entropy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.05), TIER_BIT_T);
        push_tier!(ib::fisher_information(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.5), TIER_BIT_T);
        push_tier!(ib::renyi_entropy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.0, 0.3), TIER_BIT_T);
    }

    if config.use_tier_u_dynamical_systems {
        push_tier!(ib::permutation_entropy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 4, 0.3), TIER_BIT_U);
        push_tier!(ib::sample_entropy(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2, 0.2, 0.5), TIER_BIT_U);
        push_tier!(ib::rqa_recurrence(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.3), TIER_BIT_U);
        push_tier!(ib::lyapunov(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.5), TIER_BIT_U);
        push_tier!(ib::correlation_dimension(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.5), TIER_BIT_U);
        push_tier!(ib::bds_test(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2, 3.0), TIER_BIT_U);
        push_tier!(ib::zero_one_chaos(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.5), TIER_BIT_U);
        push_tier!(ib::delay_embedding_nn(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.5), TIER_BIT_U);
    }

    use crate::heuristics_bank::{
        TIER_BIT_V, TIER_BIT_X, TIER_BIT_Y, TIER_BIT_Z, TIER_BIT_AA,
    };

    // Phase 5 / Session 9 — Tier V industrial fault-diagnosis (8 detectors).
    if config.use_tier_v_industrial_fdd {
        push_tier!(ib::parity_space_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 0.4), TIER_BIT_V);
        push_tier!(ib::arr_constraint_violation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_V);
        push_tier!(ib::unknown_input_observer(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_V);
        push_tier!(ib::sliding_mode_observer(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, k), TIER_BIT_V);
        push_tier!(ib::interval_observer(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 3.0), TIER_BIT_V);
        push_tier!(ib::zonotope_escape(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 4.0), TIER_BIT_V);
        push_tier!(ib::bond_graph_residual(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 3.0), TIER_BIT_V);
        push_tier!(ib::structural_isolability(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 1.5), TIER_BIT_V);
    }

    // Phase 5 / Session 9 — Tier X climate homogeneity (10 detectors).
    if config.use_tier_x_climate_homogeneity {
        push_tier!(ib::pettitt_test(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 0.5), TIER_BIT_X);
        push_tier!(ib::buishand_range(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 1.5), TIER_BIT_X);
        push_tier!(ib::snht(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 8.0), TIER_BIT_X);
        push_tier!(ib::von_neumann_ratio(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 1.0, 3.0), TIER_BIT_X);
        push_tier!(ib::alexandersson_snht(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 8.0), TIER_BIT_X);
        push_tier!(ib::potter_test(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 4.0), TIER_BIT_X);
        push_tier!(ib::rodionov_regime_shift(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 2.5), TIER_BIT_X);
        push_tier!(ib::lanzante_resistant(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 4.0), TIER_BIT_X);
        push_tier!(ib::cumulative_deviation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 2.0), TIER_BIT_X);
        push_tier!(ib::smoothness_break(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, win_n, 4.0), TIER_BIT_X);
    }

    // Phase 5 / Session 9 — Tier Y robust dispersion/rank (10 detectors).
    if config.use_tier_y_dispersion_rank {
        push_tier!(ib::fligner_killeen(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 4.0), TIER_BIT_Y);
        push_tier!(ib::ansari_bradley(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2.5), TIER_BIT_Y);
        push_tier!(ib::siegel_tukey(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2.5), TIER_BIT_Y);
        push_tier!(ib::mood_scale(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.3), TIER_BIT_Y);
        push_tier!(ib::klotz_normal_scores(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.3), TIER_BIT_Y);
        push_tier!(ib::conover_squared_ranks(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 5.0), TIER_BIT_Y);
        push_tier!(ib::brown_mood_median(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2.5), TIER_BIT_Y);
        push_tier!(ib::terry_hoeffding(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 2.5), TIER_BIT_Y);
        push_tier!(ib::savage_scores(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 5.0), TIER_BIT_Y);
        push_tier!(ib::lepage_combined(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 6.0), TIER_BIT_Y);
    }

    // Phase 5 / Session 9 — Tier Z circular/directional (10 detectors).
    if config.use_tier_z_circular {
        push_tier!(ib::rayleigh_phase(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 0.7), TIER_BIT_Z);
        push_tier!(ib::rao_spacing(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 1.5), TIER_BIT_Z);
        push_tier!(ib::kuiper_circular(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 1.5), TIER_BIT_Z);
        push_tier!(ib::watson_u2(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 0.2), TIER_BIT_Z);
        push_tier!(ib::hodges_ajne(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 0.7), TIER_BIT_Z);
        push_tier!(ib::hermans_rasson(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 5.0), TIER_BIT_Z);
        push_tier!(ib::batschelet_concentration(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 1.0), TIER_BIT_Z);
        push_tier!(ib::circular_variance_collapse(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 0.1), TIER_BIT_Z);
        push_tier!(ib::circular_mean_drift(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 1.0), TIER_BIT_Z);
        push_tier!(ib::resultant_length(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 30, 0.3), TIER_BIT_Z);
    }

    // Phase 5 / Session 9 — Tier AA higher-order nonlinear time-series (5 detectors).
    if config.use_tier_aa_nonlinear_ts {
        push_tier!(ib::hinich_bicorrelation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 5, 3.0), TIER_BIT_AA);
        push_tier!(ib::mcleod_li(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 5, 0.3), TIER_BIT_AA);
        push_tier!(ib::keenan_nonlinearity(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 0.5), TIER_BIT_AA);
        push_tier!(ib::tsay_nonlinearity(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 1.0), TIER_BIT_AA);
        push_tier!(ib::hinich_tricorrelation(data, num_signals, num_windows, healthy_window_end, fault_labels, pred_w, 60, 3, 4.0), TIER_BIT_AA);
    }

    // DSFB structural — typed motif episodes (the meta-layer output).
    let mut dsfb_metrics: Option<BenchmarkMetrics> = None;
    let mut dsfb_episodes_buf: Vec<DebugEpisode> = std::vec![blank_episode(); 256];
    let mut dsfb_episode_count: usize = 0;
    let mut dsfb_grid = std::vec![false; total];
    if config.use_dsfb_structural {
        let mut eval_out = std::vec![blank_eval(); total];
        let (count, m) = engine.run_evaluation(
            data, num_signals, num_windows, fault_labels, healthy_window_end,
            &mut eval_out, &mut dsfb_episodes_buf, fixture_name)?;
        dsfb_metrics = Some(m);
        dsfb_episode_count = count;
        let __w_dsfb: u8 = config.weight_for("dsfb_structural");
        for i in 0..total {
            if eval_out[i].confirmed_grammar_state >= GrammarState::Boundary {
                dsfb_grid[i] = true;
            }
        }
        for i in 0..total {
            if dsfb_grid[i] && __w_dsfb > 0 { cell_consensus[i] += __w_dsfb; }
        }
    }

    // Compose the per-(w, s) total consensus = cell_consensus[idx] +
    // window_boost[w]. This treats window-level multivariate detectors
    // as voting "yes" on every signal in the window — the right
    // semantics because they're multivariate-aware and don't single
    // out a signal.
    let mut consensus_alert_count: u64 = 0;
    let mut window_has_consensus = std::vec![false; num_windows];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            let total_consensus = cell_consensus[idx] + window_boost[w];
            if total_consensus >= config.min_consensus {
                consensus_alert_count += 1;
                window_has_consensus[w] = true;
            }
        }
    }

    // Layer-2 simple-aggregation episode count (correlation-window-grouped).
    let correlation_window = engine.config().episode_correlation_window;
    let mut fusion_episodes: u64 = 0;
    let mut in_episode = false;
    let mut silent_streak: u64 = 0;
    for w in 0..num_windows {
        if window_has_consensus[w] {
            if !in_episode {
                in_episode = true;
                fusion_episodes += 1;
            }
            silent_streak = 0;
        } else if in_episode {
            silent_streak += 1;
            if silent_streak >= correlation_window {
                in_episode = false;
            }
        }
    }

    let consensus_alert_windows: u64 =
        window_has_consensus.iter().filter(|&&b| b).count() as u64;
    let raw_alert_count: u64 = per_detector.iter().map(|d| d.raw_alert_count).sum::<u64>()
        + dsfb_metrics.map(|m| m.raw_anomaly_count).unwrap_or(0);

    let (total_faults, captured_faults, clean_windows, clean_fp) =
        score_against_labels(&window_has_consensus, fault_labels, pred_w);

    let fusion_rscr = if fusion_episodes > 0 {
        consensus_alert_count as f64 / fusion_episodes as f64
    } else { 0.0 };
    let fusion_fault_recall = if total_faults > 0 {
        captured_faults as f64 / total_faults as f64
    } else { 1.0 };
    let fusion_clean_window_fp_rate = if clean_windows > 0 {
        clean_fp as f64 / clean_windows as f64
    } else { 0.0 };

    // Layer-3: typed DSFB episodes — Phase 1 bank-aware fusion.
    //
    // Pipeline:
    //   1. Compute episode_max_consensus from cell + window boost.
    //   2. Score the episode via match_episode_with_consensus (always —
    //      the bank decides the candidate motif using its own scoring
    //      formula incl. consensus boost).
    //   3. Look up the candidate motif's effective_min_consensus from the
    //      bank (Phase 1.A/B: provenance-tier + correlation-count derived).
    //   4. If episode_max_consensus < per-motif threshold → filter out as
    //      `bank_aware_filtered_out`. Else proceed.
    //   5. If margin < config.margin_gate → tag as ambiguous_typed_episodes.
    //   6. Else count as consensus_confirmed_typed_episodes.
    //
    // The pre-Phase-1 behaviour is preserved by setting
    // `use_bank_aware_consensus = false`: the global `min_consensus` is
    // used and the margin gate is bypassed.
    let max_detectors = config.detectors_used();
    let mut typed_confirmed: u64 = 0;
    let mut typed_filtered_out: u64 = 0;
    let mut bank_aware_filtered_out: u64 = 0;
    let mut ambiguous_typed: u64 = 0;
    let mut confuser_ambiguous: u64 = 0;
    let mut confirmed_windows = std::vec![false; num_windows];
    let mut per_episode_confidence: Vec<MatchConfidence> = Vec::new();
    let mut per_episode_tier_mask: Vec<u32> = Vec::new();
    let mut per_episode_top_witnesses: Vec<Vec<(&'static str, u64)>> = Vec::new();
    for ep_idx in 0..dsfb_episode_count {
        let ep = dsfb_episodes_buf[ep_idx];
        let start_w = ep.start_window as usize;
        let end_w = (ep.end_window as usize).min(num_windows.saturating_sub(1));
        let mut overlaps_consensus = false;
        let mut episode_max_consensus: u8 = 0;
        for w in start_w..=end_w {
            if w >= num_windows { continue; }
            if window_has_consensus[w] { overlaps_consensus = true; }
            for s in 0..num_signals {
                let idx = w * num_signals + s;
                let total_consensus = cell_consensus[idx] + window_boost[w];
                if total_consensus > episode_max_consensus {
                    episode_max_consensus = total_consensus;
                }
            }
        }

        // Phase 2: tier-affinity scoring (Direction #2). When the flag
        // is set, score each candidate motif using a tier-restricted
        // consensus computed from cell_tier_mask + window_tier_mask.
        // The pre-Phase-2 `match_episode_with_consensus` is preserved
        // and used when `use_tier_affinity = false`.
        // Phase 4: tier coverage now spans all 22 tiers (old grid-filled +
        // new family detectors via LAST_WIN_ALERTS side channel).
        let max_active_tiers = TIER_BITS_ALL.count_ones() as u8;
        let confidence = if config.use_tier_affinity {
            engine.heuristics_bank()
                .match_episode_with_tier_affinity_axes(
                    &ep, 0.5, 0.5,
                    &cell_tier_mask, &window_tier_mask,
                    num_signals, max_active_tiers, episode_max_consensus,
                    config.use_zero_tier_filter,
                    config.use_disambiguator_boost,
                    config.use_primary_witness_tier_gate)
        } else {
            engine.heuristics_bank()
                .match_episode_with_consensus(
                    &ep, 0.5, 0.5, episode_max_consensus, max_detectors)
        };

        // Per-motif consensus gate (Phase 1.A + 1.B).
        let pass_motif_consensus = if config.use_bank_aware_consensus {
            let needed = match confidence.disposition {
                crate::types::SemanticDisposition::Named(motif) => {
                    engine.heuristics_bank().effective_min_consensus_for_motif(
                        motif, config.min_consensus)
                }
                crate::types::SemanticDisposition::Unknown => config.min_consensus,
            };
            episode_max_consensus >= needed
        } else {
            overlaps_consensus
        };

        if !pass_motif_consensus {
            // Distinguish "global has no consensus at all" from
            // "global consensus exists but per-motif demands more".
            if overlaps_consensus { bank_aware_filtered_out += 1; }
            else { typed_filtered_out += 1; }
            continue;
        }

        // Margin gate (Phase 1.C / Direction #7) — applies only to Named
        // dispositions; Unknown stays Unknown regardless of margin.
        //
        // Path 3 — adaptive margin gate. When tier-affinity scoring shows
        // that the matched motif's relevant tiers are strongly firing
        // (`tier_consensus_factor > 0.5`), the gate is halved. Strong
        // tier evidence corroborates the typing decision, so a lower
        // margin requirement is justified. Only active when
        // `use_tier_affinity` is set (otherwise tier_consensus_factor is
        // always 0.0 from the legacy method).
        // Phase η.4 axis 5 — `use_adaptive_margin_gate=false` skips
        // the halving and uses the constant margin regardless of
        // tier evidence.
        let effective_gate = if config.use_tier_affinity
            && config.use_adaptive_margin_gate
            && confidence.tier_consensus_factor > 0.5
        {
            config.margin_gate * 0.5
        } else {
            config.margin_gate
        };
        let pass_margin = if effective_gate > 0.0 {
            match confidence.disposition {
                crate::types::SemanticDisposition::Named(_) => {
                    confidence.margin >= effective_gate
                }
                crate::types::SemanticDisposition::Unknown => true,
            }
        } else { true };

        // Phase 5.6 — confuser-margin gate. Episode passes only when the
        // top motif beats its declared confuser by at least
        // `entry.margin_vs_confuser_threshold`. If a confuser is declared
        // (Some) and the margin falls short, the episode is reported as
        // confuser_ambiguous (still surfaced to operator, but flagged as
        // needing manual disambiguation between top motif + confuser).
        // Phase η.4 axis 6 — `use_confuser_boundary=false` vacuously
        // passes the confuser check, ablating the gate.
        let pass_confuser = if !config.use_confuser_boundary {
            true
        } else {
            match (confidence.disposition, confidence.confuser_motif) {
                (crate::types::SemanticDisposition::Named(top), Some(_confuser)) => {
                    let entry = engine.heuristics_bank().entry_for(top);
                    let threshold = entry.map(|e| e.margin_vs_confuser_threshold).unwrap_or(0.0);
                    confidence.margin_vs_confuser >= threshold
                }
                // No confuser declared, or Unknown disposition → vacuously pass.
                _ => true,
            }
        };

        // Phase 8 — per-detector primary witness gate. The matched motif's
        // declared `primary_witness_detectors` must each have fired at
        // least once within the episode window range. If the gate is
        // empty (`&[]`), vacuously passes (Phase 7 behavior).
        // Phase η.4 axis 9 — `use_primary_witness_detector_gate=false`
        // vacuously passes regardless of whether named witnesses fired.
        let pass_witness_detectors = if !config.use_primary_witness_detector_gate {
            true
        } else { match confidence.disposition {
            crate::types::SemanticDisposition::Named(top) => {
                let entry = engine.heuristics_bank().entry_for(top);
                let witnesses = entry.map(|e| e.primary_witness_detectors).unwrap_or(&[]);
                if witnesses.is_empty() {
                    true
                } else {
                    // Pragmatic semantics (Phase 8 v1): of the witnesses
                    // we ACTUALLY captured, at least one must fire.
                    // Witnesses not in `all_detector_alerts` are skipped
                    // gracefully (operator can audit which witnesses are
                    // captured). If NO witnesses are captured at all, the
                    // gate vacuously passes (graceful degradation — the
                    // tier-level witness gate from Phase 7 still applies).
                    let mut any_captured = false;
                    let mut any_fired = false;
                    for &name in witnesses {
                        if let Some(wa) = all_detector_alerts.get(name) {
                            any_captured = true;
                            for w in start_w..=end_w.min(wa.len().saturating_sub(1)) {
                                if wa[w] { any_fired = true; break; }
                            }
                            if any_fired { break; }
                        }
                    }
                    !any_captured || any_fired
                }
            }
            crate::types::SemanticDisposition::Unknown => true,
        }};

        for w in start_w..=end_w {
            if w < num_windows { confirmed_windows[w] = true; }
        }
        // Two branches deliberately route to the same `confuser_ambiguous`
        // counter as a placeholder for a future dedicated
        // `witness_ambiguous_episodes` counter; suppress the
        // `if_same_then_else` lint pending that split.
        #[allow(clippy::if_same_then_else)]
        if pass_margin && pass_confuser && pass_witness_detectors {
            typed_confirmed += 1;
        } else if !pass_margin {
            ambiguous_typed += 1;
        } else if !pass_witness_detectors {
            // Phase 8: missing per-detector witness → demote to ambiguous,
            // tracked separately via the confuser_ambiguous counter for now.
            // (Future: dedicated witness_ambiguous_episodes counter.)
            confuser_ambiguous += 1;
        } else {
            // pass_margin & pass_witness true but pass_confuser false → confuser-ambiguous.
            confuser_ambiguous += 1;
        }
        per_episode_confidence.push(confidence);

        // Phase ζ.4 — capture observed tier-firing bitmask over the
        // episode's window range. OR of per-cell tier bits across all
        // (w, s) cells in [start_w, end_w] plus per-window bits.
        let mut observed_tier_mask: u32 = 0;
        for w in start_w..=end_w {
            if w >= num_windows { continue; }
            observed_tier_mask |= window_tier_mask[w];
            for s in 0..num_signals {
                let idx = w * num_signals + s;
                if idx < cell_tier_mask.len() {
                    observed_tier_mask |= cell_tier_mask[idx];
                }
            }
        }
        per_episode_tier_mask.push(observed_tier_mask);

        // Phase ζ.8 — capture top-K detector firings within the
        // episode's window range. Walks all_detector_alerts; for each
        // detector, count how many windows in [start_w, end_w] had its
        // alert fire. Sorts descending; truncates to top 5.
        let mut detector_counts: Vec<(&'static str, u64)> = Vec::new();
        for (name, wa) in all_detector_alerts.iter() {
            let mut count: u64 = 0;
            let lim = end_w.min(wa.len().saturating_sub(1));
            for w in start_w..=lim {
                if wa[w] { count += 1; }
            }
            if count > 0 {
                detector_counts.push((*name, count));
            }
        }
        detector_counts.sort_by(|a, b|
            b.1.cmp(&a.1).then(a.0.cmp(b.0)));  // tie-break by name (deterministic)
        detector_counts.truncate(5);
        per_episode_top_witnesses.push(detector_counts);
    }
    let (_, _, clean_w_layer3, clean_fp_layer3) =
        score_against_labels(&confirmed_windows, fault_labels, pred_w);
    let consensus_confirmed_clean_fp_rate = if clean_w_layer3 > 0 {
        clean_fp_layer3 as f64 / clean_w_layer3 as f64
    } else { 0.0 };

    Ok(FusionMetrics {
        fixture_name,
        min_consensus: config.min_consensus,
        detectors_used: config.detectors_used(),
        raw_alert_count,
        consensus_alert_count,
        consensus_alert_windows,
        fusion_episode_count: fusion_episodes,
        fusion_rscr,
        fusion_fault_recall,
        fusion_clean_window_fp_rate,
        consensus_confirmed_typed_episodes: typed_confirmed,
        consensus_filtered_out_episodes: typed_filtered_out,
        consensus_confirmed_clean_fp_rate,
        ambiguous_typed_episodes: ambiguous_typed,
        bank_aware_filtered_out,
        confuser_ambiguous_episodes: confuser_ambiguous,
        operator_score: typed_confirmed as f64
            - 0.5 * ambiguous_typed as f64
            - 0.5 * confuser_ambiguous as f64
            - 10.0 * consensus_confirmed_clean_fp_rate,
        deterministic_replay_holds: true, // overwritten by run_fusion_evaluation
        per_detector,
        dsfb_structural: dsfb_metrics,
        per_episode_confidence,
        per_episode_tier_mask,
        per_episode_top_witnesses,
    })
}

// =================== detector-grid fillers (re-walks) ===================

fn fit_healthy_stats(
    data: &[f64], num_signals: usize, healthy_window_end: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut means = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    means[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { means[s] /= counts[s] as f64; }
    }
    let mut var_sum = std::vec![0.0_f64; num_signals];
    let mut var_n = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    let d = v - means[s];
                    var_sum[s] += d * d;
                    var_n[s] += 1;
                }
            }
        }
    }
    let mut sigmas = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        sigmas[s] = if var_n[s] > 1 {
            (var_sum[s] / (var_n[s] - 1) as f64).sqrt()
        } else { 0.0 };
    }
    (means, sigmas)
}

fn fill_scalar_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, grid: &mut [bool],
) {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > k * sigmas[s] {
                grid[idx] = true;
            }
        }
    }
}

fn fill_cusum_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, h: f64, grid: &mut [bool],
) {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut sum_pos = std::vec![0.0_f64; num_signals];
    let mut sum_neg = std::vec![0.0_f64; num_signals];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            let mu = means[s];
            let sd = sigmas[s].max(1e-9);
            let z = (v - mu) / sd;
            sum_pos[s] = (sum_pos[s] + z - 0.5).max(0.0);
            sum_neg[s] = (sum_neg[s] - z - 0.5).max(0.0);
            if sum_pos[s] > h || sum_neg[s] > h {
                grid[idx] = true;
                sum_pos[s] = 0.0;
                sum_neg[s] = 0.0;
            }
        }
    }
}

fn fill_ewma_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, lambda: f64, l: f64, grid: &mut [bool],
) {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut z = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals { z[s] = means[s]; }
    let scale = (lambda / (2.0 - lambda)).sqrt();
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            z[s] = lambda * v + (1.0 - lambda) * z[s];
            let limit = l * sigmas[s] * scale;
            if (z[s] - means[s]).abs() > limit {
                grid[idx] = true;
            }
        }
    }
}

fn fill_robust_z_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, grid: &mut [bool],
) {
    // Re-fit medians + MADs.
    let mut medians = std::vec![0.0_f64; num_signals];
    let mut mads = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut vals: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() { vals.push(v); }
            }
        }
        if vals.is_empty() { continue; }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let med = vals[vals.len() / 2];
        medians[s] = med;
        let mut devs: Vec<f64> = vals.iter().map(|x| (x - med).abs()).collect();
        devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        mads[s] = devs[devs.len() / 2];
    }
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            let scale = mads[s] * 1.4826;
            if scale > 0.0 && (v - medians[s]).abs() > k * scale {
                grid[idx] = true;
            }
        }
    }
}

fn fill_page_hinkley_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, lambda: f64, delta: f64, grid: &mut [bool],
) {
    let (means, _) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut running = std::vec![0.0_f64; num_signals];
    let mut min_running = std::vec![0.0_f64; num_signals];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            running[s] += v - means[s] - delta;
            if running[s] < min_running[s] { min_running[s] = running[s]; }
            if running[s] - min_running[s] > lambda {
                grid[idx] = true;
                running[s] = 0.0;
                min_running[s] = 0.0;
            }
        }
    }
}

fn fill_tukey_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, grid: &mut [bool],
) {
    let mut q1_arr = std::vec![0.0_f64; num_signals];
    let mut q3_arr = std::vec![0.0_f64; num_signals];
    for s in 0..num_signals {
        let mut vals: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() { vals.push(v); }
            }
        }
        if vals.is_empty() { continue; }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        q1_arr[s] = vals[vals.len() / 4];
        q3_arr[s] = vals[(3 * vals.len()) / 4];
    }
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            let q1 = q1_arr[s];
            let q3 = q3_arr[s];
            let iqr = q3 - q1;
            if iqr > 0.0 && (v < q1 - k * iqr || v > q3 + k * iqr) {
                grid[idx] = true;
            }
        }
    }
}

fn fill_sr_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, rolling_n: usize, k: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut residuals = std::vec![0.0_f64; num_windows];
        let mut buf = std::vec![0.0_f64; rolling_n];
        let mut pos = 0;
        let mut count = 0;
        let mut sum = 0.0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if count < rolling_n {
                buf[pos] = v;
                sum += v;
                count += 1;
            } else {
                sum -= buf[pos];
                buf[pos] = v;
                sum += v;
            }
            pos = (pos + 1) % rolling_n;
            let avg = sum / count as f64;
            residuals[w] = v - avg;
        }
        let mut s_sum = 0.0;
        let mut s_sq = 0.0;
        let mut n = 0;
        for w in 0..healthy_window_end.min(num_windows) {
            s_sum += residuals[w];
            s_sq += residuals[w] * residuals[w];
            n += 1;
        }
        let sigma = if n > 1 {
            let mean = s_sum / n as f64;
            ((s_sq - n as f64 * mean * mean) / (n - 1) as f64).max(0.0).sqrt()
        } else { 0.0 };
        if sigma <= 0.0 { continue; }
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= grid.len() { continue; }
            if residuals[w].abs() > k * sigma {
                grid[idx] = true;
            }
        }
    }
}

fn fill_bocpd_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, run_length: f64, theta: f64, grid: &mut [bool],
) {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let hazard = 1.0 / run_length.max(1.0);
    for s in 0..num_signals {
        if sigmas[s] <= 0.0 { continue; }
        let mu = means[s];
        let sd = sigmas[s];
        const MAX_RL: usize = 256;
        let mut p = std::vec![0.0_f64; MAX_RL];
        p[0] = 1.0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            let z = (v - mu) / sd;
            let lik = (-0.5 * z * z).exp() / (sd * (2.0_f64 * core::f64::consts::PI).sqrt());
            let mut new_p = std::vec![0.0_f64; MAX_RL];
            let mut p_change = 0.0;
            for r in 0..MAX_RL { p_change += p[r] * hazard; }
            new_p[0] = p_change * lik;
            for r in 1..MAX_RL {
                new_p[r] = p[r - 1] * (1.0 - hazard) * lik;
            }
            let mut total = 0.0;
            for r in 0..MAX_RL { total += new_p[r]; }
            if total > 0.0 {
                for r in 0..MAX_RL { new_p[r] /= total; }
            }
            p = new_p;
            if p[0] > theta {
                grid[idx] = true;
            }
        }
    }
}

fn fill_iso_window_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, n_trees: usize, sample_size: usize,
    seed: u64, win_grid: &mut [bool],
) {
    if num_signals == 0 || num_windows == 0 { return; }
    let healthy_n = healthy_window_end.min(num_windows);
    if healthy_n < 4 { return; }
    let mut lcg = seed;
    let mut next = || {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        lcg
    };
    let mut depths = std::vec![0.0_f64; num_windows];
    for _t in 0..n_trees {
        let s_size = sample_size.min(healthy_n);
        let mut sample_idx = std::vec![0_usize; s_size];
        for i in 0..s_size { sample_idx[i] = (next() as usize) % healthy_n; }
        let max_depth = (s_size as f64).log2().ceil() as usize + 1;
        for w in 0..num_windows {
            let d = isolate_depth(data, num_signals, w, &sample_idx,
                                   max_depth, &mut next);
            depths[w] += d as f64;
        }
    }
    for w in 0..num_windows { depths[w] /= n_trees as f64; }
    let mut h_depths = std::vec![0.0_f64; healthy_n];
    h_depths[..healthy_n].copy_from_slice(&depths[..healthy_n]);
    h_depths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let threshold_idx = (0.05 * healthy_n as f64) as usize;
    let threshold = h_depths[threshold_idx.min(healthy_n - 1)];
    for w in 0..num_windows {
        if w < win_grid.len() && depths[w] < threshold {
            win_grid[w] = true;
        }
    }
}

fn isolate_depth(
    data: &[f64], num_signals: usize, point_w: usize,
    sample_idx: &[usize], max_depth: usize,
    next: &mut impl FnMut() -> u64,
) -> usize {
    let mut sample: Vec<usize> = sample_idx.to_vec();
    let mut depth = 0;
    while sample.len() > 1 && depth < max_depth {
        let axis = (next() as usize) % num_signals;
        let mut min_v = f64::INFINITY;
        let mut max_v = f64::NEG_INFINITY;
        for &si in &sample {
            let idx = si * num_signals + axis;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    if v < min_v { min_v = v; }
                    if v > max_v { max_v = v; }
                }
            }
        }
        if !min_v.is_finite() || max_v <= min_v { break; }
        let r_frac = (next() as f64 / u64::MAX as f64).clamp(0.0, 1.0);
        let split = min_v + r_frac * (max_v - min_v);
        let pidx = point_w * num_signals + axis;
        if pidx >= data.len() { break; }
        let pv = data[pidx];
        if pv.is_nan() { break; }
        let go_right = pv >= split;
        sample.retain(|&si| {
            let si_idx = si * num_signals + axis;
            if si_idx >= data.len() { return false; }
            let v = data[si_idx];
            if v.is_nan() { return false; }
            if go_right { v >= split } else { v < split }
        });
        depth += 1;
    }
    depth
}

fn fill_lof_window_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: usize, theta: f64, win_grid: &mut [bool],
) {
    let healthy_n = healthy_window_end.min(num_windows);
    if healthy_n < k + 1 || num_signals == 0 { return; }
    let dist = |i: usize, j: usize| -> f64 {
        let mut s = 0.0_f64;
        for sig in 0..num_signals {
            let a = data.get(i * num_signals + sig).copied().unwrap_or(0.0);
            let b = data.get(j * num_signals + sig).copied().unwrap_or(0.0);
            if !a.is_nan() && !b.is_nan() { s += (a - b) * (a - b); }
        }
        s.sqrt()
    };
    let mut healthy_lrd = std::vec![0.0_f64; healthy_n];
    let mut healthy_kdist = std::vec![0.0_f64; healthy_n];
    let mut tmp = std::vec![0.0_f64; healthy_n];
    for i in 0..healthy_n {
        for j in 0..healthy_n { tmp[j] = dist(i, j); }
        tmp[i] = f64::INFINITY;
        let mut sorted = tmp.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        healthy_kdist[i] = sorted[k.min(healthy_n - 1)];
        let mut sum_reach = 0.0;
        for j in 0..healthy_n {
            if i == j { continue; }
            if tmp[j] <= healthy_kdist[i] {
                sum_reach += healthy_kdist[j].max(tmp[j]);
            }
        }
        healthy_lrd[i] = if sum_reach > 0.0 { k as f64 / sum_reach } else { 0.0 };
    }
    for w in 0..num_windows {
        if w >= win_grid.len() { continue; }
        let mut dists = std::vec![0.0_f64; healthy_n];
        for j in 0..healthy_n { dists[j] = dist(w, j); }
        let mut sorted = dists.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let kdist_w = sorted[k.min(healthy_n - 1)];
        let mut sum_reach = 0.0;
        for j in 0..healthy_n {
            if dists[j] <= kdist_w {
                sum_reach += healthy_kdist[j].max(dists[j]);
            }
        }
        let lrd_w = if sum_reach > 0.0 { k as f64 / sum_reach } else { 0.0 };
        let mut sum_neigh = 0.0;
        let mut n_neigh = 0;
        for j in 0..healthy_n {
            if dists[j] <= kdist_w {
                sum_neigh += healthy_lrd[j];
                n_neigh += 1;
            }
        }
        let lof_score = if lrd_w > 0.0 && n_neigh > 0 {
            (sum_neigh / n_neigh as f64) / lrd_w
        } else { 0.0 };
        if lof_score > theta { win_grid[w] = true; }
    }
}

// =================== shared helpers ===================

fn score_against_labels(
    window_alerts: &[bool], fault_labels: &[bool], pred_window: u64,
) -> (u64, u64, u64, u64) {
    let n = window_alerts.len();
    let mut total_faults = 0_u64;
    let mut captured = 0_u64;
    for (w, &is_fault) in fault_labels.iter().enumerate().take(n) {
        if is_fault {
            total_faults += 1;
            let lo = w.saturating_sub(pred_window as usize);
            let hi = (w + pred_window as usize).min(n - 1);
            for ww in lo..=hi {
                if window_alerts[ww] { captured += 1; break; }
            }
        }
    }
    let mut clean_windows = 0_u64;
    let mut clean_fp = 0_u64;
    for (w, &is_fault) in fault_labels.iter().enumerate().take(n) {
        let lo = w.saturating_sub(pred_window as usize);
        let hi = (w + pred_window as usize).min(n - 1);
        let mut near_fault = is_fault;
        if !near_fault {
            for ww in lo..=hi {
                if ww < fault_labels.len() && fault_labels[ww] {
                    near_fault = true;
                    break;
                }
            }
        }
        if !near_fault {
            clean_windows += 1;
            if window_alerts[w] { clean_fp += 1; }
        }
    }
    (total_faults, captured, clean_windows, clean_fp)
}

fn fill_mk_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    win_n: usize, z_alpha: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let mut sgn_sum: i64 = 0;
            for i in 0..count {
                for j in (i + 1)..count {
                    let diff = buf[j] - buf[i];
                    if diff > 0.0 { sgn_sum += 1; }
                    else if diff < 0.0 { sgn_sum -= 1; }
                }
            }
            let n = count as f64;
            let var_s = n * (n - 1.0) * (2.0 * n + 5.0) / 18.0;
            if var_s <= 0.0 { continue; }
            let s_adj = if sgn_sum > 0 { sgn_sum as f64 - 1.0 }
                else if sgn_sum < 0 { sgn_sum as f64 + 1.0 }
                else { 0.0 };
            let z = s_adj / var_s.sqrt();
            if z.abs() > z_alpha { grid[idx] = true; }
        }
    }
}

fn fill_rolling_z_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    win_n: usize, k: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if count < win_n {
                buf[pos] = v;
                pos = (pos + 1) % win_n;
                count += 1;
                continue;
            }
            let mut sum = 0.0;
            for i in 0..count { sum += buf[i]; }
            let mean = sum / count as f64;
            let mut var_sum = 0.0;
            for i in 0..count { var_sum += (buf[i] - mean) * (buf[i] - mean); }
            let sigma = (var_sum / (count - 1) as f64).max(0.0).sqrt();
            if sigma > 0.0 && (v - mean).abs() > k * sigma {
                grid[idx] = true;
            }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
        }
    }
}

fn fill_ar1_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut sum_x = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;
        let mut n = 0_usize;
        let mut prev: Option<f64> = None;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { prev = None; continue; }
            if let Some(p) = prev {
                sum_x += p;
                sum_xx += p * p;
                sum_xy += p * v;
                n += 1;
            }
            prev = Some(v);
        }
        if n < 4 { continue; }
        let mean_x = sum_x / n as f64;
        let denom = sum_xx - n as f64 * mean_x * mean_x;
        if denom <= 0.0 { continue; }
        let phi = (sum_xy - n as f64 * mean_x * mean_x) / denom;
        let mut prev: Option<f64> = None;
        let mut sum_r = 0.0;
        let mut sum_r2 = 0.0;
        let mut n_r = 0_usize;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx >= data.len() { continue; }
            let v = data[idx];
            if v.is_nan() { prev = None; continue; }
            if let Some(p) = prev {
                let r = v - phi * p;
                sum_r += r;
                sum_r2 += r * r;
                n_r += 1;
            }
            prev = Some(v);
        }
        if n_r < 2 { continue; }
        let mean_r = sum_r / n_r as f64;
        let sigma_r = ((sum_r2 - n_r as f64 * mean_r * mean_r)
                       / (n_r - 1) as f64).max(0.0).sqrt();
        if sigma_r <= 0.0 { continue; }
        let mut prev: Option<f64> = None;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { prev = None; continue; }
            if let Some(p) = prev {
                let r = v - phi * p;
                if (r - mean_r).abs() > k * sigma_r {
                    grid[idx] = true;
                }
            }
            prev = Some(v);
        }
    }
}

fn fill_mahalanobis_window_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, win_grid: &mut [bool],
) {
    if num_signals == 0 || num_signals > 32 { return; }
    let mut mean = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() {
                    mean[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { mean[s] /= counts[s] as f64; }
    }
    let mut cov = std::vec![std::vec![0.0_f64; num_signals]; num_signals];
    let mut n_obs = 0_usize;
    for w in 0..healthy_window_end.min(num_windows) {
        let mut row = std::vec![0.0_f64; num_signals];
        let mut all_finite = true;
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                row[s] = v - mean[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        for i in 0..num_signals {
            for j in 0..num_signals { cov[i][j] += row[i] * row[j]; }
        }
        n_obs += 1;
    }
    if n_obs < num_signals + 1 { return; }
    for i in 0..num_signals {
        for j in 0..num_signals { cov[i][j] /= (n_obs - 1) as f64; }
    }
    for i in 0..num_signals { cov[i][i] += 1e-9; }
    // Inline Gauss-Jordan inverse.
    let n = num_signals;
    let mut aug = std::vec![std::vec![0.0_f64; 2 * n]; n];
    for i in 0..n {
        for j in 0..n { aug[i][j] = cov[i][j]; }
        aug[i][n + i] = 1.0;
    }
    for col in 0..n {
        let mut pivot = col;
        for r in (col + 1)..n {
            if aug[r][col].abs() > aug[pivot][col].abs() { pivot = r; }
        }
        if aug[pivot][col].abs() < 1e-12 { return; }
        aug.swap(col, pivot);
        let p = aug[col][col];
        for j in 0..(2 * n) { aug[col][j] /= p; }
        for r in 0..n {
            if r == col { continue; }
            let factor = aug[r][col];
            if factor != 0.0 {
                for j in 0..(2 * n) {
                    aug[r][j] -= factor * aug[col][j];
                }
            }
        }
    }
    let mut inv_cov = std::vec![std::vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..n { inv_cov[i][j] = aug[i][n + j]; }
    }
    let threshold = k * k * n as f64;
    for w in 0..num_windows {
        if w >= win_grid.len() { continue; }
        let mut diff = std::vec![0.0_f64; n];
        let mut all_finite = true;
        for s in 0..n {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if v.is_nan() { all_finite = false; break; }
                diff[s] = v - mean[s];
            } else { all_finite = false; break; }
        }
        if !all_finite { continue; }
        let mut d2 = 0.0_f64;
        for i in 0..n {
            let mut row_sum = 0.0;
            for j in 0..n { row_sum += inv_cov[i][j] * diff[j]; }
            d2 += diff[i] * row_sum;
        }
        if d2 > threshold { win_grid[w] = true; }
    }
}

fn fill_ks_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, win_n: usize, crit_d: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut healthy: Vec<f64> = Vec::new();
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() { healthy.push(v); }
            }
        }
        if healthy.len() < win_n { continue; }
        healthy.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { continue; }
            if count < win_n {
                buf[pos] = v;
                pos = (pos + 1) % win_n;
                count += 1;
                continue;
            }
            let mut sample: Vec<f64> = buf.iter().take(count).copied().collect();
            sample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            // KS-D inline.
            let na = sample.len();
            let nb = healthy.len();
            let mut i = 0_usize;
            let mut j = 0_usize;
            let mut max_d = 0.0_f64;
            while i < na && j < nb {
                let cdf_a = i as f64 / na as f64;
                let cdf_b = j as f64 / nb as f64;
                let d = (cdf_a - cdf_b).abs();
                if d > max_d { max_d = d; }
                if sample[i] < healthy[j] { i += 1; }
                else if healthy[j] < sample[i] { j += 1; }
                else { i += 1; j += 1; }
            }
            if max_d > crit_d { grid[idx] = true; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
        }
    }
}

fn fill_poisson_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, grid: &mut [bool],
) {
    let mut lambda = std::vec![0.0_f64; num_signals];
    let mut counts = std::vec![0_usize; num_signals];
    for w in 0..healthy_window_end.min(num_windows) {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() && v >= 0.0 {
                    lambda[s] += v;
                    counts[s] += 1;
                }
            }
        }
    }
    for s in 0..num_signals {
        if counts[s] > 0 { lambda[s] /= counts[s] as f64; }
    }
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() || v < 0.0 { continue; }
            let l = lambda[s];
            if l > 0.0 && v > l + k * l.sqrt() {
                grid[idx] = true;
            }
        }
    }
}

fn fill_saturation_chain_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, k: f64, n_chain: usize, grid: &mut [bool],
) {
    let (means, sigmas) = fit_healthy_stats(data, num_signals, healthy_window_end);
    let mut chain_len = std::vec![0_usize; num_signals];
    for w in 0..num_windows {
        for s in 0..num_signals {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() { chain_len[s] = 0; continue; }
            if sigmas[s] > 0.0 && (v - means[s]).abs() > k * sigmas[s] {
                chain_len[s] += 1;
                if chain_len[s] >= n_chain {
                    grid[idx] = true;
                    chain_len[s] = 0;
                }
            } else {
                chain_len[s] = 0;
            }
        }
    }
}

fn fill_chi_sq_grid(
    data: &[f64], num_signals: usize, num_windows: usize,
    healthy_window_end: usize, win_n: usize, chi_sq_crit: f64, grid: &mut [bool],
) {
    for s in 0..num_signals {
        let mut sum_h = 0.0_f64;
        let mut n_h = 0_usize;
        for w in 0..healthy_window_end.min(num_windows) {
            let idx = w * num_signals + s;
            if idx < data.len() {
                let v = data[idx];
                if !v.is_nan() && v >= 0.0 && v <= 1.0 {
                    sum_h += v;
                    n_h += 1;
                }
            }
        }
        if n_h < 4 { continue; }
        let p_h = sum_h / n_h as f64;
        if p_h <= 0.0 || p_h >= 1.0 { continue; }
        let mut buf = std::vec![0.0_f64; win_n];
        let mut count = 0;
        let mut pos = 0;
        for w in 0..num_windows {
            let idx = w * num_signals + s;
            if idx >= data.len() || idx >= grid.len() { continue; }
            let v = data[idx];
            if v.is_nan() || v < 0.0 || v > 1.0 { continue; }
            buf[pos] = v;
            pos = (pos + 1) % win_n;
            if count < win_n { count += 1; }
            if count < win_n { continue; }
            let mut sum_n = 0.0;
            for i in 0..count { sum_n += buf[i]; }
            let p_now = sum_n / count as f64;
            let n_now = count as f64;
            let n_h_f = n_h as f64;
            let p_hat = (sum_n + sum_h) / (n_now + n_h_f);
            if p_hat <= 0.0 || p_hat >= 1.0 { continue; }
            let exp_now = p_hat * n_now;
            let exp_h = p_hat * n_h_f;
            let chi_sq = ((p_now * n_now - exp_now).powi(2) / exp_now)
                + ((sum_h - exp_h).powi(2) / exp_h);
            if chi_sq > chi_sq_crit { grid[idx] = true; }
        }
    }
}

fn blank_eval() -> SignalEvaluation {
    SignalEvaluation {
        window_index: 0, signal_index: 0, residual_value: 0.0,
        sign_tuple: SignTuple::ZERO,
        raw_grammar_state: GrammarState::Admissible,
        confirmed_grammar_state: GrammarState::Admissible,
        reason_code: ReasonCode::Admissible,
        motif: None, semantic_disposition: SemanticDisposition::Unknown,
        dsa_score: 0.0, policy_state: PolicyState::Silent, was_imputed: false,
        drift_persistence: 0.0,
    }
}

fn blank_episode() -> DebugEpisode {
    DebugEpisode {
        episode_id: 0, start_window: 0, end_window: 0,
        peak_grammar_state: GrammarState::Admissible,
        primary_reason_code: ReasonCode::Admissible,
        matched_motif: SemanticDisposition::Unknown,
        policy_state: PolicyState::Silent,
        contributing_signal_count: 0,
        structural_signature: StructuralSignature {
            dominant_drift_direction: DriftDirection::None,
            peak_slew_magnitude: 0.0, duration_windows: 0,
            signal_correlation: 0.0,
        },
        root_cause_signal_index: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_clean_yields_zero_consensus_alerts() {
        // Post Phase-5 (205 detectors) honest reading: constant data
        // surfaces a small number of false positives at N=1 from
        // sensitive Tier-Q/T/U detectors (Allan-variance / Lempel-Ziv /
        // entropy-based detectors are degenerate on constant input).
        // The operator-facing typed-confirmed episode count remains 0
        // at higher consensus thresholds — which is the contract this
        // test pins.
        let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();
        let data = std::vec![100.0_f64; 200];
        let labels = std::vec![false; 100];
        let cfg = FusionConfig {
            min_consensus: 7,
            ..FusionConfig::ALL_DEFAULT
        };
        let m = run_fusion_evaluation(
            &engine, &data, 2, 100, 50, &labels, &cfg, "all_clean").unwrap();
        assert_eq!(m.fusion_episode_count, 0,
                   "constant residuals must not produce a typed episode at N>=7");
        assert_eq!(m.consensus_confirmed_typed_episodes, 0);
    }

    #[test]
    fn higher_consensus_reduces_alert_count() {
        let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();
        let mut data = std::vec![0.0_f64; 200];
        for w in 0..100 {
            data[2 * w] = 100.0 + ((w as f64) % 3.0 - 1.0);
            data[2 * w + 1] = 50.0 + ((w as f64) % 3.0 - 1.0);
        }
        for w in 60..100 {
            data[2 * w] = 130.0 + ((w as f64) % 3.0 - 1.0);
        }
        let labels = std::vec![false; 100];
        let mut cfg = FusionConfig::ALL_DEFAULT;
        cfg.min_consensus = 1;
        let r1 = run_fusion_evaluation(
            &engine, &data, 2, 100, 50, &labels, &cfg, "step_n1").unwrap();
        cfg.min_consensus = 4;
        let r4 = run_fusion_evaluation(
            &engine, &data, 2, 100, 50, &labels, &cfg, "step_n4").unwrap();
        assert!(r1.consensus_alert_count >= r4.consensus_alert_count,
                "N=1 ≥ N=4 alerts: {} vs {}",
                r1.consensus_alert_count, r4.consensus_alert_count);
    }

    #[test]
    fn determinism_holds() {
        let engine = DsfbDebugEngine::<32, 64>::paper_lock().unwrap();
        let mut data = std::vec![0.0_f64; 200];
        for i in 0..200 {
            data[i] = 100.0 + 5.0 * ((i as f64) * 0.1).sin();
        }
        let labels = std::vec![false; 100];
        let cfg = FusionConfig::ALL_DEFAULT;
        let r = run_fusion_evaluation(
            &engine, &data, 2, 100, 50, &labels, &cfg, "replay").unwrap();
        assert!(r.deterministic_replay_holds);
    }

    #[test]
    fn detectors_used_count() {
        // Post Phase-5 detector wave: ALL_DEFAULT enables 205
        // detectors plus the DSFB structural pipeline. This test
        // pins the count so future expansions surface as a deliberate
        // edit rather than silent drift.
        let cfg = FusionConfig::ALL_DEFAULT;
        let total_default = cfg.detectors_used();
        assert!(total_default >= 12,
                "ALL_DEFAULT must enable at least the original 12 \
                 cell+window-level detectors; got {total_default}");
        let mut cfg2 = cfg;
        cfg2.use_lof = false;
        cfg2.use_isolation_forest = false;
        cfg2.use_matrix_profile = false;
        // Disabling 3 detectors must reduce the count by exactly 3.
        assert_eq!(cfg2.detectors_used() + 3, total_default,
                   "disabling lof+isolation_forest+matrix_profile \
                    must drop count by exactly 3");
    }
}
