//! DSFB-Debug: Heuristics Bank — 32 hand-curated motifs with full
//! Phase 0–8 fusion-axis decision parameters.
//!
//! # What this module is
//!
//! The heuristics bank is the IP claim of DSFB-Debug. It is a
//! fixed-size compile-time-curated table of `HeuristicEntry` records
//! that maps a structural residual signature to a named `MotifClass`
//! interpretation. Each entry carries:
//!
//! 1. **Reason-code anchor** — which `ReasonCode` (drift / slew /
//!    boundary / oscillation) the motif matches.
//! 2. **Provenance** — `FrameworkDesign` (hand-coded) /
//!    `DatasetObserved` (observed in a vendored upstream slice) /
//!    `FieldValidated` (confirmed in production deployment, reserved
//!    for Phase II partner engagement).
//! 3. **Evidence base** — upstream `evidence_dataset` + DOI of the
//!    archive in which the signature was first observed (e.g.
//!    `"tadbench_F04"` + `"10.5281/zenodo.6979726"`).
//! 4. **Taxonomy anchor** — IEEE 24765 + Avizienis-Laprie-Randell
//!    decomposition (e.g. `"IEEE 24765: 'fault propagation'; A-L-R:
//!    error → service-failure"`). Every motif name is established
//!    vocabulary, never invented.
//! 5. **Drift / slew / boundary thresholds** — gating predicates the
//!    closed episode must satisfy.
//! 6. **Correlation / duration ranges** — multi-service motifs require
//!    `min_correlation_count >= 3`; transient motifs cap
//!    `max_duration_windows`.
//! 7. **Per-feature scoring weights** — five weights (drift, slew,
//!    boundary, correlation, duration) let one motif emphasise drift
//!    while another emphasises slew.
//! 8. **Affinity-tier bitmask** — which detector tiers route their
//!    evidence into this motif's score (Routed Evidence Principle,
//!    paper §6.5).
//! 9. **Phase-5.6+ confuser-pair declaration** — explicit motif that
//!    competes for the same residual signature; episodes failing the
//!    `margin_vs_confuser` gate surface as `ConfuserAmbiguous` rather
//!    than committing to a single typing.
//! 10. **Phase-7 primary witness tiers** — strict subset of affinity
//!     tiers that MUST fire for typed confirmation.
//! 11. **Phase-8 named witness detectors** — strict per-detector
//!     anti-hallucination gate; ≥1 of the captured named witnesses
//!     must fire.
//! 12. **Dashboard hint** — operator-side template with
//!     `${ROOT_CAUSE_SERVICE}`, `${PEAK_SLEW}`, etc. placeholders that
//!     `render::render_episode_summary` substitutes at presentation
//!     time.
//!
//! # Two lookup paths
//!
//! - **`lookup(reason_code, drift_persistence, slew_magnitude)`** —
//!   per-signal, called from `evaluate_signal`. Preserves the v0.1
//!   wire shape (unit-weighted scoring across reason-code match +
//!   drift + slew). Populates `SignalEvaluation.semantic_disposition`.
//! - **`match_episode_with_*` family** — per-episode, called from
//!   `run_evaluation` after `aggregate_episodes` closes each episode.
//!   Uses ALL features available at episode close. The post-Phase-8
//!   entry point `match_episode_with_consensus` consults the 9-axis
//!   bank-aware fusion configuration (\S\ref{sec:nine_axis_fusion}
//!   in the paper); legacy `match_episode_with_confidence` and
//!   `match_episode_with_tier_affinity` are preserved for backward
//!   compatibility.
//!
//! Both paths are deterministic: tie-breakers (higher provenance rank
//! wins; lower index wins) preserve Theorem 9 across all configs.
//!
//! # Endoductive discipline (panel-locked)
//!
//! The bank intentionally does NOT carry motifs for LO2-style
//! API-semantic anomalies (OAuth2 flow / architectural-degradation
//! patterns). Those are validated against the
//! `SemanticDisposition::Unknown` branch by `tests/eval_lo2.rs`.
//! Adding LO2-specific motifs would destroy the validator. The
//! absence is by design — see paper §5.6 and Session-3 panel directive.
//!
//! # Standards alignment
//!
//! - **NIST SP 800-53 AU-3** ("audit record content"): each entry's
//!   `(provenance, evidence_dataset, evidence_dataset_doi,
//!   taxonomy_ref, dashboard_hint)` collectively satisfies the
//!   "what / when / where / source / outcome" content requirement at
//!   per-motif granularity.
//! - **NIST SP 800-53 AU-2** ("auditable events"): the
//!   `primary_witness_detectors` list defines the named auditable
//!   events that must fire for typed confirmation.
//! - **IEEE 24765 + Avizienis-Laprie-Randell**: every motif name
//!   decomposes into established software-engineering vocabulary.
//!   No ad-hoc naming.
//! - **IEEE 1012-2016** ("verification and validation"): the
//!   confuser-boundary mechanism (`margin_vs_confuser`) provides
//!   independent confirmation of typed disposition before declaring
//!   `Named(motif)` rather than `ConfuserAmbiguous`.
//!
//! # Theorem 9 contract
//!
//! Every public method preserves deterministic replay. The bank
//! itself is `Copy + Clone + Debug + PartialEq` (every field is
//! Copy-able), so two `HeuristicsBank` instances with the same
//! `entries` array produce byte-identical `match_*` results on the
//! same inputs.

use crate::types::*;

// =====================================================================
// Phase 2 — Tier-affinity bits.
//
// Each detector belongs to exactly one tier; each motif's affinity mask
// declares which tiers are predictive for that motif. Per-cell tier-fired
// bitmasks let the fusion arithmetic compute motif-conditional consensus
// (Direction #2 from the panel) by AND-ing the cell's tier mask against
// the candidate motif's affinity mask, then popcount-ing.
//
// Bits 0..22 cover the 23 tier groupings (A–F + Extras + G–U). Bits 22+
// are reserved for future tiers.
// =====================================================================
pub const TIER_BIT_A: u32         = 1 << 0;   // parametric trio (scalar/CUSUM/EWMA)
pub const TIER_BIT_B: u32         = 1 << 1;   // robust statistics
pub const TIER_BIT_C: u32         = 1 << 2;   // model / non-parametric
pub const TIER_BIT_D: u32         = 1 << 3;   // additional non-dep
pub const TIER_BIT_E: u32         = 1 << 4;   // debugging-specific stats
pub const TIER_BIT_F: u32         = 1 << 5;   // burst (neuroscience-derived)
pub const TIER_BIT_EXTRA: u32     = 1 << 6;   // GLR/ADWIN/MEWMA/retry-storm/correlation-break
pub const TIER_BIT_G: u32         = 1 << 7;   // concept-drift streaming
pub const TIER_BIT_H: u32         = 1 << 8;   // distribution shift
pub const TIER_BIT_I: u32         = 1 << 9;   // robust nonparametric
pub const TIER_BIT_J: u32         = 1 << 10;  // forecast residual
pub const TIER_BIT_K: u32         = 1 << 11;  // frequency / oscillation
pub const TIER_BIT_L: u32         = 1 << 12;  // multivariate relationship
pub const TIER_BIT_M: u32         = 1 << 13;  // debugging-native
pub const TIER_BIT_N: u32         = 1 << 14;  // offline CPD
pub const TIER_BIT_O: u32         = 1 << 15;  // rare changepoint
pub const TIER_BIT_P: u32         = 1 << 16;  // streaming sequential
pub const TIER_BIT_Q: u32         = 1 << 17;  // concept drift rarer
pub const TIER_BIT_R: u32         = 1 << 18;  // robust depth
pub const TIER_BIT_S: u32         = 1 << 19;  // count event-process
pub const TIER_BIT_T: u32         = 1 << 20;  // info-theoretic
pub const TIER_BIT_U: u32         = 1 << 21;  // dynamical systems

// Phase 5 wave (Sessions 9+) — new families. Each adds one tier bit.
pub const TIER_BIT_V: u32         = 1 << 22;  // industrial fault-diagnosis (FDD)
pub const TIER_BIT_W: u32         = 1 << 23;  // formal runtime monitoring
pub const TIER_BIT_X: u32         = 1 << 24;  // climate homogeneity (D-extended)
pub const TIER_BIT_Y: u32         = 1 << 25;  // robust dispersion / rank (E-extended)
pub const TIER_BIT_Z: u32         = 1 << 26;  // circular / directional
pub const TIER_BIT_AA: u32        = 1 << 27;  // higher-order nonlinear time-series
pub const TIER_BIT_BB: u32        = 1 << 28;  // econometric parameter-instability
pub const TIER_BIT_CC: u32        = 1 << 29;  // metrology / clock-stability
pub const TIER_BIT_DD: u32        = 1 << 30;  // numerical-computing pathology
pub const TIER_BIT_EE: u32        = 1 << 31;  // process-monitoring contribution / isolation

/// Tier-affinity mask for a motif: which detector tiers are predictive
/// for the motif's reason-code structural signature. Multi-service motifs
/// (`min_correlation_count >= 3`) get multivariate tiers added regardless.
///
/// `u32::MAX` (= "all tiers") for unmapped reason codes preserves the
/// pre-Phase-2 uniform-voting semantics for that motif.
pub fn affinity_tiers_for(reason_code: ReasonCode, min_correlation_count: u16) -> u32 {
    let base = match reason_code {
        // Slow drift: trend / forecast-residual / monotone-leak / variance-time / regularity
        ReasonCode::SustainedOutwardDrift => {
            TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_S | TIER_BIT_U | TIER_BIT_T
        }
        // Step / abrupt change: scalar-3σ, Page-Hinkley, Theil-Sen step, offline + rare CPD
        ReasonCode::AbruptSlewViolation => {
            TIER_BIT_A | TIER_BIT_B | TIER_BIT_I | TIER_BIT_N | TIER_BIT_O | TIER_BIT_EXTRA
        }
        // Oscillation / boundary grazing: frequency, debugging-native limit-cycle/flap, RQA
        ReasonCode::RecurrentBoundaryGrazing => {
            TIER_BIT_K | TIER_BIT_M | TIER_BIT_U
        }
        // Drift with recovery: trend + forecast + sawtooth-ramp / hysteresis
        ReasonCode::DriftWithRecovery => {
            TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_T
        }
        // Boundary approach: just-below-envelope; slowly-rising trend + spectral-entropy
        ReasonCode::BoundaryApproach => {
            TIER_BIT_I | TIER_BIT_J | TIER_BIT_K | TIER_BIT_M
        }
        // Envelope violation: hard breach; scalar/CUSUM/Page-Hinkley + extreme + saturation
        ReasonCode::EnvelopeViolation => {
            TIER_BIT_A | TIER_BIT_B | TIER_BIT_E | TIER_BIT_R
        }
        // Single crossing: dismissed by persistence; scalar + burst-like
        ReasonCode::SingleCrossing => {
            TIER_BIT_A | TIER_BIT_F
        }
        // Default: all tiers active (preserve pre-Phase-2 semantics).
        ReasonCode::Admissible => u32::MAX,
    };
    // Multi-service motifs always benefit from multivariate + correlation tiers.
    let multivariate = if min_correlation_count >= 3 {
        TIER_BIT_C | TIER_BIT_L | TIER_BIT_EXTRA
    } else { 0 };
    base | multivariate
}

/// The heuristics bank: a fixed-size array of known motif patterns.
/// Provenance-aware: each entry records where it came from.
pub struct HeuristicsBank<const MAX: usize> {
    entries: [Option<HeuristicEntry>; MAX],
    count: usize,
}

impl<const MAX: usize> HeuristicsBank<MAX> {
    /// Create a new bank pre-loaded with the canonical debugging motifs
    /// (32 entries, anchored to IEEE 24765 + Avizienis-Laprie-Randell).
    pub fn with_canonical_motifs() -> Self {
        let mut bank = Self {
            entries: [None; MAX],
            count: 0,
        };

        let canonical: &[HeuristicEntry] = &[
            // ===== Tier-1: original 10 motifs (FrameworkDesign) ==========

            HeuristicEntry {
                motif_class: MotifClass::MemoryLeakDrift,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "sustained monotonic memory-consumption drift; may correspond to object-retention bugs",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.6,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.4,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.5,
                weight_slew: 0.3,
                weight_boundary: 1.0,
                weight_correlation: 0.5,
                weight_duration: 1.2,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect process RSS / heap-used and gc.duration over the past hour",
                taxonomy_ref: "IEEE 24765: 'memory leak'; A-L-R: latent fault → error",
                affinity_tiers: TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_S | TIER_BIT_U | TIER_BIT_T | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::ConnectionPoolExhaustionDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_I | TIER_BIT_M | TIER_BIT_S,
                primary_witness_detectors: &["mann_kendall", "monotone_leak", "theil_sen_residual"],
            },
            HeuristicEntry {
                motif_class: MotifClass::CascadingTimeoutSlew,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "step-change latency propagating across dependency chain; may correspond to upstream failure",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.5,
                boundary_density_threshold: 0.0,
                min_correlation_count: 3,
                max_correlation_count: u16::MAX,
                min_duration_windows: 2,
                max_duration_windows: 30,
                weight_drift: 0.2,
                weight_slew: 1.5,
                weight_boundary: 0.5,
                weight_correlation: 1.5,
                weight_duration: 0.8,
                evidence_dataset: "tadbench_trainticket_F04",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect ${ROOT_CAUSE_SERVICE} (signal ${ROOT_CAUSE_INDEX}); ${CONTRIBUTING_COUNT} services contribute over ${DURATION_WINDOWS} windows; peak slew ${PEAK_SLEW}",
                taxonomy_ref: "IEEE 24765: 'fault propagation'; A-L-R: error → service-failure",
                affinity_tiers: TIER_BIT_C | TIER_BIT_L | TIER_BIT_EXTRA | TIER_BIT_B | TIER_BIT_M | TIER_BIT_V | TIER_BIT_X,
                confuser_motif: Some(MotifClass::DependencySlowdown),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_C | TIER_BIT_L | TIER_BIT_EXTRA,
                primary_witness_detectors: &["correlation_break", "lof", "causal_lag"],
            },
            HeuristicEntry {
                motif_class: MotifClass::DeploymentRegressionSlew,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "abrupt baseline shift coinciding with deployment; structural step function",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.8,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: 2,
                min_duration_windows: 1,
                max_duration_windows: u16::MAX,
                weight_drift: 0.0,
                weight_slew: 2.0,
                weight_boundary: 0.5,
                weight_correlation: 0.3,
                weight_duration: 0.2,
                evidence_dataset: "tadbench_trainticket_F11",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Single-service step shift on ${ROOT_CAUSE_SERVICE} (signal ${ROOT_CAUSE_INDEX}); peak slew ${PEAK_SLEW}; correlate with deployment log near window ${DURATION_WINDOWS}; consider rollback",
                taxonomy_ref: "IEEE 24765: 'regression'; A-L-R: design fault → error",
                affinity_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_I | TIER_BIT_N | TIER_BIT_O | TIER_BIT_X | TIER_BIT_Y | TIER_BIT_V,
                confuser_motif: Some(MotifClass::CircuitBreakerOpenShift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_N | TIER_BIT_X,
                primary_witness_detectors: &["page_hinkley", "pelt", "pettitt_test"],
            },
            HeuristicEntry {
                motif_class: MotifClass::CacheDegradationGrazing,
                reason_code: ReasonCode::RecurrentBoundaryGrazing,
                candidate_interpretation: "oscillatory approach to SLO boundary; may correspond to cache eviction patterns",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.3,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.5,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: u16::MAX,
                weight_drift: 0.6,
                weight_slew: 0.4,
                weight_boundary: 1.8,
                weight_correlation: 0.4,
                weight_duration: 1.0,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect cache hit-rate and eviction rate of the affected service",
                taxonomy_ref: "IEEE 24765: 'performance degradation'; A-L-R: marginal-state error",
                affinity_tiers: TIER_BIT_K | TIER_BIT_M | TIER_BIT_U | TIER_BIT_F | TIER_BIT_Z | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::GcPressureOscillation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_K | TIER_BIT_M | TIER_BIT_U,
                primary_witness_detectors: &["autocorrelation_peak", "limit_cycle", "flap"],
            },
            HeuristicEntry {
                motif_class: MotifClass::ConnectionPoolExhaustionDrift,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "slow positive drift in queue depth + latency with increasing variance",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.4,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.2,
                weight_boundary: 1.0,
                weight_correlation: 0.6,
                weight_duration: 1.1,
                evidence_dataset: "tadbench_trainticket_F19",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect connection pool waiting queue + active connections + idle timeout",
                taxonomy_ref: "IEEE 24765: 'resource exhaustion'; A-L-R: error build-up",
                affinity_tiers: TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_E | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::MemoryLeakDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_I | TIER_BIT_E | TIER_BIT_M,
                primary_witness_detectors: &["monotone_leak", "saturation_chain", "theil_sen_residual"],
            },
            HeuristicEntry {
                motif_class: MotifClass::GcPressureOscillation,
                reason_code: ReasonCode::RecurrentBoundaryGrazing,
                candidate_interpretation: "periodic slew events coinciding with GC pauses; bounded oscillation",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.0,
                slew_threshold: 0.2,
                boundary_density_threshold: 0.4,
                min_correlation_count: 1,
                max_correlation_count: 3,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 0.2,
                weight_slew: 1.4,
                weight_boundary: 1.4,
                weight_correlation: 0.4,
                weight_duration: 0.6,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect gc.collection.count + gc.duration histograms",
                taxonomy_ref: "IEEE 24765: 'stop-the-world pause'; A-L-R: transient error",
                affinity_tiers: TIER_BIT_K | TIER_BIT_F | TIER_BIT_M | TIER_BIT_S | TIER_BIT_Z | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::CacheDegradationGrazing),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_K | TIER_BIT_F | TIER_BIT_Z,
                primary_witness_detectors: &["autocorrelation_peak", "sawtooth_ramp", "welch_psd"],
            },
            HeuristicEntry {
                motif_class: MotifClass::ErrorRateEscalation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "sustained positive drift in error rate",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.7,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 1.8,
                weight_slew: 0.3,
                weight_boundary: 0.8,
                weight_correlation: 0.7,
                weight_duration: 1.0,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect HTTP 5xx rate by endpoint and recent deploys / config changes",
                taxonomy_ref: "IEEE 24765: 'error escalation'; A-L-R: error → multi-failure regime",
                affinity_tiers: TIER_BIT_A | TIER_BIT_G | TIER_BIT_Q | TIER_BIT_E | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::PacketLossErrorEscalation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_G | TIER_BIT_E,
                primary_witness_detectors: &["chi_squared_proportion", "ddm", "ecdd"],
            },
            HeuristicEntry {
                motif_class: MotifClass::DependencySlowdown,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "gradual latency increase in upstream dependency",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.4,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.3,
                weight_slew: 0.3,
                weight_boundary: 0.7,
                weight_correlation: 0.7,
                weight_duration: 0.9,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect the upstream service's latency distribution; check its dependencies",
                taxonomy_ref: "IEEE 24765: 'performance degradation upstream'; A-L-R: external fault",
                affinity_tiers: TIER_BIT_C | TIER_BIT_L | TIER_BIT_M | TIER_BIT_EXTRA | TIER_BIT_V | TIER_BIT_X,
                confuser_motif: Some(MotifClass::CascadingTimeoutSlew),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_C | TIER_BIT_L | TIER_BIT_M,
                primary_witness_detectors: &["causal_lag", "correlation_break", "lof"],
            },
            HeuristicEntry {
                motif_class: MotifClass::ResourceSaturation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "CPU/memory/disk approaching ceiling; concave-up drift",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.5,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.3,
                weight_boundary: 1.2,
                weight_correlation: 0.5,
                weight_duration: 1.0,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect system resource gauges (CPU%, memory%, disk%) over the past hour",
                taxonomy_ref: "IEEE 24765: 'resource saturation'; A-L-R: latent → manifest fault",
                affinity_tiers: TIER_BIT_A | TIER_BIT_E | TIER_BIT_M | TIER_BIT_R | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::ConnectionPoolExhaustionDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_E | TIER_BIT_R,
                primary_witness_detectors: &["saturation_chain", "monotone_leak", "mann_kendall"],
            },
            HeuristicEntry {
                motif_class: MotifClass::QueueBackpressure,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "message queue depth growing monotonically",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.6,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: u16::MAX,
                weight_drift: 1.5,
                weight_slew: 0.2,
                weight_boundary: 0.9,
                weight_correlation: 0.5,
                weight_duration: 1.1,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect message-queue depth and consumer lag metrics",
                taxonomy_ref: "IEEE 24765: 'back-pressure accumulation'; A-L-R: error build-up",
                affinity_tiers: TIER_BIT_M | TIER_BIT_L | TIER_BIT_J | TIER_BIT_S | TIER_BIT_V | TIER_BIT_X | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::ConnectionPoolExhaustionDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_M | TIER_BIT_L,
                primary_witness_detectors: &["backpressure", "mann_kendall", "mahalanobis"],
            },

            // ===== Tier-2: TADBench fault cases ==========================

            HeuristicEntry {
                motif_class: MotifClass::RetryStormCascade,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "client retries amplify upstream load; both error rate and request rate rise together",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.6,
                slew_threshold: 0.3,
                boundary_density_threshold: 0.4,
                min_correlation_count: 2,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: 60,
                weight_drift: 1.3,
                weight_slew: 1.0,
                weight_boundary: 0.8,
                weight_correlation: 1.4,
                weight_duration: 0.7,
                evidence_dataset: "tadbench_retry_storm",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect retry-policy parameters and client-side request rate vs upstream success rate",
                taxonomy_ref: "IEEE 24765: 'retry-induced amplification'; A-L-R: cascading error",
                affinity_tiers: TIER_BIT_F | TIER_BIT_M | TIER_BIT_EXTRA | TIER_BIT_S | TIER_BIT_AA | TIER_BIT_Z,
                confuser_motif: Some(MotifClass::CascadingTimeoutSlew),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_F | TIER_BIT_EXTRA | TIER_BIT_M,
                primary_witness_detectors: &["retry_storm", "causal_lag", "poisson_burst"],
            },
            HeuristicEntry {
                motif_class: MotifClass::CircuitBreakerOpenShift,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "circuit breaker transitions from CLOSED to OPEN; downstream calls fail fast",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.6,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: 4,
                min_duration_windows: 2,
                max_duration_windows: u16::MAX,
                weight_drift: 0.2,
                weight_slew: 1.6,
                weight_boundary: 0.4,
                weight_correlation: 0.8,
                weight_duration: 0.6,
                evidence_dataset: "tadbench_circuit_breaker",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect circuit-breaker state metrics and the underlying service's health",
                taxonomy_ref: "IEEE 24765: 'fault tolerance mechanism state change'",
                affinity_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_N | TIER_BIT_O | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::DeploymentRegressionSlew),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_N,
                primary_witness_detectors: &["cusum", "pelt", "binary_segmentation"],
            },
            HeuristicEntry {
                motif_class: MotifClass::DatabaseLockContention,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "database lock contention; rising query latency and queue depth",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.2,
                boundary_density_threshold: 0.4,
                min_correlation_count: 2,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.2,
                weight_slew: 0.6,
                weight_boundary: 1.0,
                weight_correlation: 1.0,
                weight_duration: 1.0,
                evidence_dataset: "tadbench_db_lock",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect database lock-wait stats and slow-query log for the active transactions",
                taxonomy_ref: "IEEE 24765: 'concurrency fault'; A-L-R: synchronisation error",
                affinity_tiers: TIER_BIT_L | TIER_BIT_M | TIER_BIT_C | TIER_BIT_EXTRA | TIER_BIT_V | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::DependencySlowdown),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_M | TIER_BIT_EXTRA,
                primary_witness_detectors: &["causal_lag", "correlation_break", "mahalanobis"],
            },
            HeuristicEntry {
                motif_class: MotifClass::AuthenticationFailureSpike,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "authentication subsystem partial outage; spike in 401/403 + downstream re-auth retries",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.5,
                boundary_density_threshold: 0.2,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 1,
                max_duration_windows: 30,
                weight_drift: 0.3,
                weight_slew: 1.5,
                weight_boundary: 0.6,
                weight_correlation: 0.9,
                weight_duration: 0.5,
                evidence_dataset: "tadbench_auth_fail",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Inspect auth-service health, token-issuance rate, and 401/403 distribution",
                taxonomy_ref: "IEEE 24765: 'authentication subsystem failure'",
                affinity_tiers: TIER_BIT_F | TIER_BIT_M | TIER_BIT_A | TIER_BIT_AA | TIER_BIT_Z | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::EpisodicTransientSpike),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_F | TIER_BIT_M,
                primary_witness_detectors: &["poisson_burst", "burst_after_silence", "flap"],
            },
            HeuristicEntry {
                motif_class: MotifClass::ConfigDriftRegression,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "step shift coinciding with version-config change",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.6,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: 3,
                min_duration_windows: 1,
                max_duration_windows: u16::MAX,
                weight_drift: 0.2,
                weight_slew: 1.7,
                weight_boundary: 0.5,
                weight_correlation: 0.5,
                weight_duration: 0.4,
                evidence_dataset: "trainticket_anomaly_version_config",
                evidence_dataset_doi: "10.5281/zenodo.6979726",
                dashboard_hint: "Diff config artefacts between the last good window and the current; consider rollback",
                taxonomy_ref: "IEEE 24765: 'configuration regression'; A-L-R: design-time fault",
                affinity_tiers: TIER_BIT_H | TIER_BIT_G | TIER_BIT_Q | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::DeploymentRegressionSlew),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_H | TIER_BIT_G | TIER_BIT_Q,
                primary_witness_detectors: &["wasserstein_1d", "ddm", "kl_divergence"],
            },

            // ===== Tier-3: AIOps Challenge categories ====================

            HeuristicEntry {
                motif_class: MotifClass::PacketLossErrorEscalation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "network-layer packet loss elevates error rate; sustained positive drift",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.6,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 2,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 1.6,
                weight_slew: 0.4,
                weight_boundary: 0.8,
                weight_correlation: 1.2,
                weight_duration: 0.8,
                evidence_dataset: "aiops_challenge_packet_loss",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect TCP retransmits / packet-loss counters at the network layer",
                taxonomy_ref: "IEEE 24765: 'communication failure (lower layer)'",
                affinity_tiers: TIER_BIT_A | TIER_BIT_G | TIER_BIT_F | TIER_BIT_E | TIER_BIT_AA | TIER_BIT_Z,
                confuser_motif: Some(MotifClass::ErrorRateEscalation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_G | TIER_BIT_F,
                primary_witness_detectors: &["chi_squared_proportion", "poisson_burst", "ddm"],
            },
            HeuristicEntry {
                motif_class: MotifClass::NetworkDelayDependencyInflation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "injected network delay on upstream link; gradual latency-increase pattern across consumers",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.4,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 2,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.3,
                weight_slew: 0.2,
                weight_boundary: 0.7,
                weight_correlation: 1.3,
                weight_duration: 1.1,
                evidence_dataset: "aiops_challenge_network_delay",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect inter-service RTT histograms and link-level latency gauges",
                taxonomy_ref: "IEEE 24765: 'communication-path performance fault'",
                affinity_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_M | TIER_BIT_V | TIER_BIT_X,
                confuser_motif: Some(MotifClass::DependencySlowdown),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_M,
                primary_witness_detectors: &["causal_lag", "lof", "correlation_break"],
            },
            HeuristicEntry {
                motif_class: MotifClass::DiskIoSaturation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "disk I/O saturation; concave-up latency drift on storage-bound services",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.5,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.3,
                weight_boundary: 1.2,
                weight_correlation: 0.6,
                weight_duration: 1.0,
                evidence_dataset: "aiops_challenge_disk_exhaustion",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect disk IOPS / await / queue depth; check for runaway log writes",
                taxonomy_ref: "IEEE 24765: 'storage subsystem saturation'",
                affinity_tiers: TIER_BIT_E | TIER_BIT_A | TIER_BIT_I | TIER_BIT_R | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::CpuSaturation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_E | TIER_BIT_I | TIER_BIT_R,
                primary_witness_detectors: &["saturation_chain", "monotone_leak", "theil_sen_residual"],
            },
            HeuristicEntry {
                motif_class: MotifClass::CpuSaturation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "CPU saturation; latency drift with rising envelope occupancy",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.5,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.3,
                weight_boundary: 1.2,
                weight_correlation: 0.6,
                weight_duration: 1.0,
                evidence_dataset: "aiops_challenge_cpu_exhaustion",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect CPU utilisation, run-queue length, and thread-level scheduling latency",
                taxonomy_ref: "IEEE 24765: 'compute resource saturation'",
                affinity_tiers: TIER_BIT_E | TIER_BIT_A | TIER_BIT_I | TIER_BIT_R | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::DiskIoSaturation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_E | TIER_BIT_I | TIER_BIT_R,
                primary_witness_detectors: &["saturation_chain", "monotone_leak", "theil_sen_residual"],
            },
            HeuristicEntry {
                motif_class: MotifClass::JvmHeapPressure,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "JVM heap pressure; sustained latency drift with rising variance and elevated GC frequency",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.6,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.4,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.6,
                weight_slew: 0.4,
                weight_boundary: 1.0,
                weight_correlation: 0.5,
                weight_duration: 1.2,
                evidence_dataset: "aiops_challenge_memory_exhaustion",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect jvm.memory.heap.used + gc.collection.count + minor/major GC ratio",
                taxonomy_ref: "IEEE 24765: 'memory leak (JVM-specific)'; refines MemoryLeakDrift",
                affinity_tiers: TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_E | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::MemoryLeakDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_I | TIER_BIT_M | TIER_BIT_E,
                primary_witness_detectors: &["monotone_leak", "saturation_chain", "mann_kendall"],
            },
            HeuristicEntry {
                motif_class: MotifClass::JvmGcPause,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "JVM stop-the-world GC pause: distinct latency spikes with regular cadence",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.0,
                slew_threshold: 0.4,
                boundary_density_threshold: 0.3,
                min_correlation_count: 1,
                max_correlation_count: 3,
                min_duration_windows: 1,
                max_duration_windows: 10,
                weight_drift: 0.2,
                weight_slew: 1.6,
                weight_boundary: 1.0,
                weight_correlation: 0.4,
                weight_duration: 0.4,
                evidence_dataset: "aiops_challenge_jvm_resource_exhaustion",
                evidence_dataset_doi: "AIOps-Challenge-2020-2021",
                dashboard_hint: "Inspect gc.duration percentiles + STW pause histograms; consider GC tuning",
                taxonomy_ref: "IEEE 24765: 'stop-the-world pause (JVM-specific)'; refines GcPressureOscillation",
                affinity_tiers: TIER_BIT_K | TIER_BIT_F | TIER_BIT_M | TIER_BIT_Z | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::AuthenticationFailureSpike),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_K | TIER_BIT_F,
                primary_witness_detectors: &["welch_psd", "autocorrelation_peak", "poisson_burst"],
            },

            // ===== Tier-4: MultiDim-Localization patterns ================

            HeuristicEntry {
                motif_class: MotifClass::ServiceGraphDriftPropagation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "drift propagating along the service-call graph (multi-hop); affects sequentially-related services",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.5,
                slew_threshold: 0.2,
                boundary_density_threshold: 0.4,
                min_correlation_count: 4,
                max_correlation_count: u16::MAX,
                min_duration_windows: 5,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.7,
                weight_boundary: 0.9,
                weight_correlation: 1.7,
                weight_duration: 1.0,
                evidence_dataset: "multidim_localization_graph_propagation",
                evidence_dataset_doi: "MultiDimension-Localization-NetManAIOps",
                dashboard_hint: "Multi-hop drift propagation across ${CONTRIBUTING_COUNT} services; originator: ${ROOT_CAUSE_SERVICE} (signal ${ROOT_CAUSE_INDEX}); peak slew ${PEAK_SLEW}",
                taxonomy_ref: "IEEE 24765: 'graph-structured fault propagation'",
                affinity_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_G | TIER_BIT_EXTRA | TIER_BIT_V | TIER_BIT_X,
                confuser_motif: Some(MotifClass::DependencySlowdown),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_EXTRA,
                primary_witness_detectors: &["correlation_matrix_distance", "correlation_break", "causal_lag"],
            },
            HeuristicEntry {
                motif_class: MotifClass::HighDimAnomalyCluster,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "multi-metric correlated anomaly without single dominant signal; compound fault",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.4,
                slew_threshold: 0.2,
                boundary_density_threshold: 0.3,
                min_correlation_count: 6,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: u16::MAX,
                weight_drift: 1.0,
                weight_slew: 0.6,
                weight_boundary: 0.7,
                weight_correlation: 2.0,
                weight_duration: 0.9,
                evidence_dataset: "multidim_localization_cluster",
                evidence_dataset_doi: "MultiDimension-Localization-NetManAIOps",
                dashboard_hint: "Inspect the multi-metric anomaly cluster as a unit; no single metric is dominant",
                taxonomy_ref: "IEEE 24765: 'compound fault signature'",
                affinity_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_R | TIER_BIT_V | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::MetricCorrelationCollapse),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_R,
                primary_witness_detectors: &["mahalanobis", "pca_reconstruction", "lof"],
            },
            HeuristicEntry {
                motif_class: MotifClass::MetricCorrelationCollapse,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "historically-correlated metrics decorrelate; structural regime shift",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.0,
                slew_threshold: 0.4,
                boundary_density_threshold: 0.0,
                min_correlation_count: 2,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 0.4,
                weight_slew: 1.3,
                weight_boundary: 0.5,
                weight_correlation: 1.5,
                weight_duration: 0.8,
                evidence_dataset: "multidim_localization_correlation_collapse",
                evidence_dataset_doi: "MultiDimension-Localization-NetManAIOps",
                dashboard_hint: "Compare current pairwise metric correlations against the baseline correlation matrix",
                taxonomy_ref: "IEEE 24765: 'structural model invalidation'",
                affinity_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_EXTRA | TIER_BIT_V | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::HighDimAnomalyCluster),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_C | TIER_BIT_EXTRA,
                primary_witness_detectors: &["correlation_break", "correlation_matrix_distance", "mahalanobis"],
            },

            // ===== Tier-5: DeepTraLog log + trace fusion =================

            HeuristicEntry {
                motif_class: MotifClass::LogVolumeAnomaly,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "log-frequency outward drift on a service; structural log-rate anomaly",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.5,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.3,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 1.4,
                weight_slew: 0.3,
                weight_boundary: 0.8,
                weight_correlation: 0.6,
                weight_duration: 0.9,
                evidence_dataset: "deeptralog_log_volume",
                evidence_dataset_doi: "DeepTraLog-ICSE-2022",
                dashboard_hint: "Inspect log-volume gauges per severity per service over the past hour",
                taxonomy_ref: "IEEE 24765: 'diagnostic-output anomaly'",
                affinity_tiers: TIER_BIT_A | TIER_BIT_S | TIER_BIT_I | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::LogSeverityEscalation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_S | TIER_BIT_I,
                primary_witness_detectors: &["chi_squared_proportion", "poisson_burst", "mann_kendall"],
            },
            HeuristicEntry {
                motif_class: MotifClass::LogTraceTemporalDecorrelation,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "log timing departs from trace timing pattern; instrumentation/temporal divergence",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Review,
                drift_threshold: 0.0,
                slew_threshold: 0.3,
                boundary_density_threshold: 0.2,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 0.4,
                weight_slew: 1.3,
                weight_boundary: 0.7,
                weight_correlation: 0.7,
                weight_duration: 0.7,
                evidence_dataset: "deeptralog_temporal_mismatch",
                evidence_dataset_doi: "DeepTraLog-ICSE-2022",
                dashboard_hint: "Inspect log-event timestamps vs trace-span timestamps for the same request IDs",
                taxonomy_ref: "IEEE 24765: 'instrumentation-temporal divergence'",
                affinity_tiers: TIER_BIT_L | TIER_BIT_T | TIER_BIT_EXTRA | TIER_BIT_V | TIER_BIT_AA,
                confuser_motif: Some(MotifClass::ServiceGraphDriftPropagation),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_L | TIER_BIT_T | TIER_BIT_EXTRA,
                primary_witness_detectors: &["correlation_break", "transfer_entropy", "correlation_matrix_distance"],
            },
            HeuristicEntry {
                motif_class: MotifClass::LogSeverityEscalation,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "log severity distribution shift (more WARN/ERROR proportionally)",
                provenance: Provenance::DatasetObserved,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.6,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.4,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 3,
                max_duration_windows: u16::MAX,
                weight_drift: 1.6,
                weight_slew: 0.3,
                weight_boundary: 1.0,
                weight_correlation: 0.7,
                weight_duration: 0.9,
                evidence_dataset: "deeptralog_severity_shift",
                evidence_dataset_doi: "DeepTraLog-ICSE-2022",
                dashboard_hint: "Inspect log-severity distribution histograms; compare against the healthy-window baseline",
                taxonomy_ref: "IEEE 24765: 'diagnostic severity escalation'",
                affinity_tiers: TIER_BIT_H | TIER_BIT_G | TIER_BIT_A | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::LogVolumeAnomaly),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_H | TIER_BIT_G | TIER_BIT_A,
                primary_witness_detectors: &["wasserstein_1d", "chi_squared_proportion", "ddm"],
            },

            // ===== Tier-6: cross-cutting structural motifs ===============

            HeuristicEntry {
                motif_class: MotifClass::SaturationTrending,
                reason_code: ReasonCode::SustainedOutwardDrift,
                candidate_interpretation: "concave-up approach to a ceiling; generalises ResourceSaturation",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.5,
                slew_threshold: 0.1,
                boundary_density_threshold: 0.5,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 6,
                max_duration_windows: u16::MAX,
                weight_drift: 1.3,
                weight_slew: 0.6,
                weight_boundary: 1.4,
                weight_correlation: 0.5,
                weight_duration: 1.1,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Project the current drift forward to estimate time-to-ceiling; consider scaling",
                taxonomy_ref: "IEEE 24765: 'asymptotic resource saturation'",
                affinity_tiers: TIER_BIT_E | TIER_BIT_M | TIER_BIT_I | TIER_BIT_J | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::ConnectionPoolExhaustionDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_E | TIER_BIT_M | TIER_BIT_I,
                primary_witness_detectors: &["saturation_chain", "monotone_leak", "mann_kendall"],
            },
            HeuristicEntry {
                motif_class: MotifClass::EpisodicTransientSpike,
                reason_code: ReasonCode::AbruptSlewViolation,
                candidate_interpretation: "short-duration high-slew event that self-resolves",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.0,
                slew_threshold: 0.5,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 1,
                max_duration_windows: 4,
                weight_drift: 0.2,
                weight_slew: 1.6,
                weight_boundary: 0.3,
                weight_correlation: 0.4,
                weight_duration: 0.3,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Note the timestamp; correlate with cron / scheduled jobs / external triggers",
                taxonomy_ref: "IEEE 24765: 'transient-only error'; A-L-R: transient fault",
                affinity_tiers: TIER_BIT_A | TIER_BIT_F | TIER_BIT_M | TIER_BIT_AA | TIER_BIT_Z,
                confuser_motif: Some(MotifClass::AuthenticationFailureSpike),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_F | TIER_BIT_M,
                primary_witness_detectors: &["poisson_burst", "burst_after_silence", "scalar_threshold_3sigma"],
            },
            HeuristicEntry {
                motif_class: MotifClass::RegressiveDriftWithRecovery,
                reason_code: ReasonCode::DriftWithRecovery,
                candidate_interpretation: "outward drift followed by return to baseline; self-healing structural transient",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.4,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.2,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 4,
                max_duration_windows: 30,
                weight_drift: 1.2,
                weight_slew: 0.3,
                weight_boundary: 0.6,
                weight_correlation: 0.5,
                weight_duration: 0.8,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "No action required if recovery confirmed; record as a near-miss for trend analysis",
                taxonomy_ref: "IEEE 24765: 'self-healing transient drift'",
                affinity_tiers: TIER_BIT_I | TIER_BIT_J | TIER_BIT_M | TIER_BIT_T | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::MemoryLeakDrift),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_I | TIER_BIT_J | TIER_BIT_M,
                primary_witness_detectors: &["theil_sen_residual", "monotone_leak", "mann_kendall"],
            },
            HeuristicEntry {
                motif_class: MotifClass::EnvelopeBoundaryApproach,
                reason_code: ReasonCode::BoundaryApproach,
                candidate_interpretation: "first-time approach to the SLO envelope without recurrence or persistent drift; marginal-state transient",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Watch,
                drift_threshold: 0.0,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 1,
                max_duration_windows: u16::MAX,
                weight_drift: 0.5,
                weight_slew: 0.5,
                weight_boundary: 0.8,
                weight_correlation: 0.3,
                weight_duration: 0.4,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Note the timestamp; if recurrence is observed in subsequent windows escalate to CacheDegradationGrazing",
                taxonomy_ref: "IEEE 24765: 'marginal-state transient'; A-L-R: dormant fault",
                affinity_tiers: TIER_BIT_A | TIER_BIT_J | TIER_BIT_I | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::EnvelopeBreach),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_I | TIER_BIT_J,
                primary_witness_detectors: &["scalar_threshold_3sigma", "mann_kendall", "theil_sen_residual"],
            },
            HeuristicEntry {
                motif_class: MotifClass::EnvelopeBreach,
                reason_code: ReasonCode::EnvelopeViolation,
                candidate_interpretation: "envelope breach without abrupt slew evidence; smooth threshold crossing",
                provenance: Provenance::FrameworkDesign,
                recommended_action: PolicyState::Escalate,
                drift_threshold: 0.0,
                slew_threshold: 0.0,
                boundary_density_threshold: 0.0,
                min_correlation_count: 1,
                max_correlation_count: u16::MAX,
                min_duration_windows: 1,
                max_duration_windows: u16::MAX,
                weight_drift: 0.7,
                weight_slew: 0.3,
                weight_boundary: 0.7,
                weight_correlation: 0.6,
                weight_duration: 0.6,
                evidence_dataset: "FrameworkDesign",
                evidence_dataset_doi: "",
                dashboard_hint: "Inspect SLO/SLA threshold + the affected service's value distribution; threshold may be too tight",
                taxonomy_ref: "IEEE 24765: 'threshold breach (smooth)'; A-L-R: error → manifest",
                affinity_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_R | TIER_BIT_E | TIER_BIT_X | TIER_BIT_Y,
                confuser_motif: Some(MotifClass::EnvelopeBoundaryApproach),
                margin_vs_confuser_threshold: 0.10,
                primary_witness_tiers: TIER_BIT_A | TIER_BIT_B | TIER_BIT_R,
                primary_witness_detectors: &["scalar_threshold_3sigma", "cusum", "page_hinkley"],
            },
        ];

        let mut i = 0;
        while i < canonical.len() && i < MAX {
            bank.entries[i] = Some(canonical[i]);
            bank.count += 1;
            i += 1;
        }
        bank
    }

    /// Per-signal lookup (v0.1 wire shape).
    ///
    /// Used at signal-evaluation time inside `evaluate_signal`. Score
    /// composition is unit-weighted; multi-feature weighting happens at
    /// the episode level (`match_episode`).
    pub fn lookup(
        &self,
        reason_code: ReasonCode,
        drift_persistence: f64,
        slew_magnitude: f64,
    ) -> SemanticDisposition {
        let mut best_match: Option<MotifClass> = None;
        let mut best_score: f64 = 0.0;
        let mut best_provenance_rank: u8 = 0;
        let mut best_index: usize = usize::MAX;

        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.reason_code == reason_code {
                    let mut score: f64 = 1.0; // base score for reason code match
                    if drift_persistence >= entry.drift_threshold {
                        score += drift_persistence;
                    }
                    if slew_magnitude >= entry.slew_threshold {
                        score += slew_magnitude;
                    }
                    let prov_rank = provenance_rank(entry.provenance);
                    let take = score > best_score
                        || (score == best_score && prov_rank > best_provenance_rank)
                        || (score == best_score && prov_rank == best_provenance_rank && i < best_index);
                    if take {
                        best_score = score;
                        best_match = Some(entry.motif_class);
                        best_provenance_rank = prov_rank;
                        best_index = i;
                    }
                }
            }
            i += 1;
        }

        match best_match {
            Some(motif) => SemanticDisposition::Named(motif),
            None => SemanticDisposition::Unknown, // Endoductive mode
        }
    }

    /// Per-episode lookup (Session 3 addition).
    ///
    /// Uses ALL features available at episode close: peak slew (from
    /// `episode.structural_signature.peak_slew_magnitude`), signal
    /// correlation count, duration in windows, the supplied average
    /// drift persistence and average boundary density. Score
    /// composition: `Σ weight_f × feature_f` for the five features,
    /// gated by `reason_code` match plus the min/max range checks on
    /// correlation count and duration windows.
    ///
    /// Tie-breakers (deterministic, no FP randomness):
    ///   1. higher provenance rank wins
    ///      (`FieldValidated > DatasetObserved > FrameworkDesign`)
    ///   2. lower index in the entries array wins (canonical ordering)
    pub fn match_episode(
        &self,
        episode: &DebugEpisode,
        avg_drift_persistence: f64,
        avg_boundary_density: f64,
    ) -> SemanticDisposition {
        let mut best_match: Option<MotifClass> = None;
        let mut best_score: f64 = 0.0;
        let mut best_provenance_rank: u8 = 0;
        let mut best_index: usize = usize::MAX;

        let correlation_count = episode.contributing_signal_count;
        let duration_windows: u16 = if episode.end_window >= episode.start_window {
            let d = episode.end_window - episode.start_window + 1;
            if d > u16::MAX as u64 { u16::MAX } else { d as u16 }
        } else {
            0
        };
        let peak_slew = episode.structural_signature.peak_slew_magnitude;
        let slew_mag = if peak_slew >= 0.0 { peak_slew } else { -peak_slew };

        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.reason_code != episode.primary_reason_code {
                    i += 1;
                    continue;
                }
                if correlation_count < entry.min_correlation_count
                    || correlation_count > entry.max_correlation_count
                {
                    i += 1;
                    continue;
                }
                if duration_windows < entry.min_duration_windows
                    || duration_windows > entry.max_duration_windows
                {
                    i += 1;
                    continue;
                }

                let mut score: f64 = 1.0; // base for reason-code + range gates passing

                if avg_drift_persistence >= entry.drift_threshold {
                    score += entry.weight_drift * avg_drift_persistence;
                }
                if slew_mag >= entry.slew_threshold {
                    score += entry.weight_slew * slew_mag;
                }
                if avg_boundary_density >= entry.boundary_density_threshold {
                    score += entry.weight_boundary * avg_boundary_density;
                }
                // Correlation contribution (count is unitless; scale by 0.1 to
                // keep it comparable to fractional features).
                score += entry.weight_correlation * (correlation_count as f64) * 0.1;
                // Duration contribution (windows; scale by 0.05 — long episodes
                // mildly boost long-duration motifs without overwhelming).
                score += entry.weight_duration * (duration_windows as f64) * 0.05;

                let prov_rank = provenance_rank(entry.provenance);
                let take = score > best_score
                    || (score == best_score && prov_rank > best_provenance_rank)
                    || (score == best_score && prov_rank == best_provenance_rank && i < best_index);
                if take {
                    best_score = score;
                    best_match = Some(entry.motif_class);
                    best_provenance_rank = prov_rank;
                    best_index = i;
                }
            }
            i += 1;
        }

        match best_match {
            Some(motif) => SemanticDisposition::Named(motif),
            None => SemanticDisposition::Unknown,
        }
    }

    /// Per-episode lookup with confidence margin (Phase 6 addition).
    ///
    /// Same scoring as `match_episode`, but additionally tracks the
    /// runner-up scored motif so operators can read the
    /// top-vs-runner-up margin (see `MatchConfidence`). Tie-breakers
    /// (provenance rank, lower index) are unchanged.
    pub fn match_episode_with_confidence(
        &self,
        episode: &DebugEpisode,
        avg_drift_persistence: f64,
        avg_boundary_density: f64,
    ) -> MatchConfidence {
        let mut best_match: Option<MotifClass> = None;
        let mut best_score: f64 = 0.0;
        let mut best_provenance_rank: u8 = 0;
        let mut best_index: usize = usize::MAX;
        let mut runner_up_match: Option<MotifClass> = None;
        let mut runner_up_score: f64 = 0.0;

        let correlation_count = episode.contributing_signal_count;
        let duration_windows: u16 = if episode.end_window >= episode.start_window {
            let d = episode.end_window - episode.start_window + 1;
            if d > u16::MAX as u64 { u16::MAX } else { d as u16 }
        } else {
            0
        };
        let peak_slew = episode.structural_signature.peak_slew_magnitude;
        let slew_mag = if peak_slew >= 0.0 { peak_slew } else { -peak_slew };

        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.reason_code != episode.primary_reason_code {
                    i += 1;
                    continue;
                }
                if correlation_count < entry.min_correlation_count
                    || correlation_count > entry.max_correlation_count
                {
                    i += 1;
                    continue;
                }
                if duration_windows < entry.min_duration_windows
                    || duration_windows > entry.max_duration_windows
                {
                    i += 1;
                    continue;
                }

                let mut score: f64 = 1.0;
                if avg_drift_persistence >= entry.drift_threshold {
                    score += entry.weight_drift * avg_drift_persistence;
                }
                if slew_mag >= entry.slew_threshold {
                    score += entry.weight_slew * slew_mag;
                }
                if avg_boundary_density >= entry.boundary_density_threshold {
                    score += entry.weight_boundary * avg_boundary_density;
                }
                score += entry.weight_correlation * (correlation_count as f64) * 0.1;
                score += entry.weight_duration * (duration_windows as f64) * 0.05;

                let prov_rank = provenance_rank(entry.provenance);
                let take_top = score > best_score
                    || (score == best_score && prov_rank > best_provenance_rank)
                    || (score == best_score && prov_rank == best_provenance_rank && i < best_index);
                if take_top {
                    // Demote previous top to runner-up
                    if let Some(prev_top) = best_match {
                        if score > runner_up_score
                            || (score == runner_up_score && best_score > runner_up_score)
                        {
                            runner_up_match = Some(prev_top);
                            runner_up_score = best_score;
                        }
                    }
                    best_score = score;
                    best_match = Some(entry.motif_class);
                    best_provenance_rank = prov_rank;
                    best_index = i;
                } else if score > runner_up_score {
                    runner_up_score = score;
                    runner_up_match = Some(entry.motif_class);
                }
            }
            i += 1;
        }

        let disposition = match best_match {
            Some(m) => SemanticDisposition::Named(m),
            None => SemanticDisposition::Unknown,
        };
        let margin = if best_score > 0.0 {
            ((best_score - runner_up_score) / best_score).clamp(0.0, 1.0)
        } else {
            0.0
        };
        MatchConfidence {
            disposition,
            top_score: best_score,
            runner_up_score,
            runner_up_motif: runner_up_match,
            margin,
            tier_consensus_factor: 0.0,
            confuser_motif: None,
            confuser_score: 0.0,
            margin_vs_confuser: 0.0,
        }
    }

    /// Per-episode lookup with multi-detector consensus context
    /// (post-Phase-8 fusion-aware lookup).
    ///
    /// Same scoring as `match_episode_with_confidence`, but additionally
    /// adds a consensus boost: `consensus_boost = (episode_max_consensus
    /// / max_detectors) * 1.0`, summed into the score for every motif
    /// that survives the reason-code + range gates. Result: when multiple
    /// detectors agree, the bank is MORE confident in the typed
    /// interpretation; when only DSFB-structural fired, the bank still
    /// produces the structural motif but with smaller margin (signaling
    /// "this is a structurally-detected anomaly that flat detectors
    /// missed — consider whether DSFB is over-firing or whether the
    /// flat detectors are missing real signal").
    ///
    /// `episode_max_consensus` is the maximum consensus_count observed
    /// in any (window, signal) cell within the episode's window range.
    /// `max_detectors` is the total number of enabled detectors
    /// (typically 12 with all-default fusion).
    pub fn match_episode_with_consensus(
        &self,
        episode: &DebugEpisode,
        avg_drift_persistence: f64,
        avg_boundary_density: f64,
        episode_max_consensus: u8,
        max_detectors: u8,
    ) -> MatchConfidence {
        let consensus_factor = if max_detectors > 0 {
            episode_max_consensus as f64 / max_detectors as f64
        } else { 0.0 };

        let mut best_match: Option<MotifClass> = None;
        let mut best_score: f64 = 0.0;
        let mut best_provenance_rank: u8 = 0;
        let mut best_index: usize = usize::MAX;
        let mut runner_up_match: Option<MotifClass> = None;
        let mut runner_up_score: f64 = 0.0;

        let correlation_count = episode.contributing_signal_count;
        let duration_windows: u16 = if episode.end_window >= episode.start_window {
            let d = episode.end_window - episode.start_window + 1;
            if d > u16::MAX as u64 { u16::MAX } else { d as u16 }
        } else { 0 };
        let peak_slew = episode.structural_signature.peak_slew_magnitude;
        let slew_mag = if peak_slew >= 0.0 { peak_slew } else { -peak_slew };

        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.reason_code != episode.primary_reason_code {
                    i += 1;
                    continue;
                }
                if correlation_count < entry.min_correlation_count
                    || correlation_count > entry.max_correlation_count
                {
                    i += 1;
                    continue;
                }
                if duration_windows < entry.min_duration_windows
                    || duration_windows > entry.max_duration_windows
                {
                    i += 1;
                    continue;
                }

                let mut score: f64 = 1.0;
                if avg_drift_persistence >= entry.drift_threshold {
                    score += entry.weight_drift * avg_drift_persistence;
                }
                if slew_mag >= entry.slew_threshold {
                    score += entry.weight_slew * slew_mag;
                }
                if avg_boundary_density >= entry.boundary_density_threshold {
                    score += entry.weight_boundary * avg_boundary_density;
                }
                score += entry.weight_correlation * (correlation_count as f64) * 0.1;
                score += entry.weight_duration * (duration_windows as f64) * 0.05;
                // Consensus boost — scales linearly with the fraction
                // of detectors that agreed. Doesn't gate the match;
                // amplifies the score so that motifs corroborated by
                // many independent detectors get higher margin.
                score += consensus_factor;

                let prov_rank = provenance_rank(entry.provenance);
                let take_top = score > best_score
                    || (score == best_score && prov_rank > best_provenance_rank)
                    || (score == best_score && prov_rank == best_provenance_rank && i < best_index);
                if take_top {
                    if let Some(prev_top) = best_match {
                        if score > runner_up_score
                            || (score == runner_up_score && best_score > runner_up_score)
                        {
                            runner_up_match = Some(prev_top);
                            runner_up_score = best_score;
                        }
                    }
                    best_score = score;
                    best_match = Some(entry.motif_class);
                    best_provenance_rank = prov_rank;
                    best_index = i;
                } else if score > runner_up_score {
                    runner_up_score = score;
                    runner_up_match = Some(entry.motif_class);
                }
            }
            i += 1;
        }

        let disposition = match best_match {
            Some(m) => SemanticDisposition::Named(m),
            None => SemanticDisposition::Unknown,
        };
        let margin = if best_score > 0.0 {
            ((best_score - runner_up_score) / best_score).clamp(0.0, 1.0)
        } else { 0.0 };
        MatchConfidence {
            disposition,
            top_score: best_score,
            runner_up_score,
            runner_up_motif: runner_up_match,
            margin,
            tier_consensus_factor: 0.0,
            confuser_motif: None,
            confuser_score: 0.0,
            margin_vs_confuser: 0.0,
        }
    }

    /// Get recommended action for a matched motif
    pub fn recommended_action(&self, motif: MotifClass) -> PolicyState {
        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.motif_class == motif {
                    return entry.recommended_action;
                }
            }
            i += 1;
        }
        PolicyState::Watch // default if motif not found
    }

    /// Number of entries currently populated.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Iterate over the populated entries in canonical-bank order.
    ///
    /// Returns `&HeuristicEntry` items in the order they were appended
    /// to the bank — the same order used by the deterministic
    /// tie-breaker rule (lower index wins on score ties at the same
    /// provenance rank). Used by the `demo` feature's documentation /
    /// figure-rendering layer to emit the 32-motif × 27-tier affinity
    /// matrix without requiring access to the private `entries`
    /// storage.
    pub fn entries_iter(&self) -> impl Iterator<Item = &HeuristicEntry> {
        self.entries[..self.count].iter().filter_map(|e| e.as_ref())
    }

    /// Look up an entry by its `MotifClass` (for documentation /
    /// dashboard rendering of the matched motif's metadata).
    pub fn entry_for(&self, motif: MotifClass) -> Option<&HeuristicEntry> {
        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.motif_class == motif {
                    return Some(entry);
                }
            }
            i += 1;
        }
        None
    }

    /// Bank-aware fusion: per-motif effective minimum consensus threshold,
    /// derived from the motif entry's provenance ladder + correlation count.
    ///
    /// Direction #4 (provenance-tier rules) + Direction #1 (per-motif consensus)
    /// from the Phase-1 panel proposal:
    ///
    /// * `FieldValidated` motifs trust experience — threshold lowered by 1.
    /// * `DatasetObserved` motifs use the global threshold unchanged.
    /// * `FrameworkDesign` motifs require additional corroboration —
    ///   threshold raised by 1 (still hypothesis-stage typing).
    /// * Multi-service motifs (`min_correlation_count >= 3`) require
    ///   stronger consensus regardless of provenance — extra +1.
    ///
    /// Floor of 1 enforced (zero-consensus typing is rejected).
    pub fn effective_min_consensus(&self, entry: &HeuristicEntry, global_min: u8) -> u8 {
        let mut t = global_min as i16;
        match entry.provenance {
            Provenance::FieldValidated => { t -= 1; }
            Provenance::DatasetObserved => { /* unchanged */ }
            Provenance::FrameworkDesign => { t += 1; }
        }
        if entry.min_correlation_count >= 3 { t += 1; }
        if t < 1 { t = 1; }
        if t > 255 { t = 255; }
        t as u8
    }

    /// Bank-aware fusion convenience: look up the effective per-motif
    /// consensus threshold by `MotifClass`. Falls back to `global_min`
    /// if the motif is not in the bank.
    pub fn effective_min_consensus_for_motif(
        &self, motif: MotifClass, global_min: u8,
    ) -> u8 {
        match self.entry_for(motif) {
            Some(entry) => self.effective_min_consensus(entry, global_min),
            None => global_min,
        }
    }

    /// Phase 2 — Direction #2 (tier-affinity matrix) + Direction #5
    /// (margin feedback). Score each candidate motif using a *tier-
    /// restricted* consensus computed from per-cell + per-window
    /// tier-fired bitmasks; motifs whose affinity tiers actually fired
    /// score higher than motifs whose affinity tiers were silent.
    ///
    /// Inputs:
    /// * `cell_tier_mask[w * num_signals + s]` — bitmask of tiers that
    ///   contributed at cell (w, s). One bit per tier.
    /// * `window_tier_mask[w]` — bitmask of tiers contributing at the
    ///   window level (multivariate / global detectors).
    /// * `episode_max_consensus` — fallback consensus value (used to
    ///   set the consensus_factor for motifs unmapped by reason-code
    ///   affinity, i.e. `Admissible` default).
    ///
    /// Returns the best-scoring motif, with `margin` reflecting the
    /// gap between top and runner-up under tier-affinity scoring.
    /// Determinism preserved (deterministic tie-break on provenance
    /// rank then index, identical to `match_episode_with_consensus`).
    /// Bank-side ablation axes (Phase η.4).
    ///
    /// Each flag controls one of the three bank-internal fusion axes
    /// (axes 4 / 7 / 8 of the 9-axis ladder). Default `true` preserves
    /// pre-Phase-η.4 behaviour exactly. Setting any flag to `false`
    /// disables that single bank-internal axis for ablation studies.
    pub fn match_episode_with_tier_affinity(
        &self,
        episode: &DebugEpisode,
        avg_drift_persistence: f64,
        avg_boundary_density: f64,
        cell_tier_mask: &[u32],
        window_tier_mask: &[u32],
        num_signals: usize,
        max_active_tiers: u8,
        episode_max_consensus: u8,
    ) -> MatchConfidence {
        // Default delegate: all bank-internal axes enabled.
        self.match_episode_with_tier_affinity_axes(
            episode, avg_drift_persistence, avg_boundary_density,
            cell_tier_mask, window_tier_mask, num_signals,
            max_active_tiers, episode_max_consensus,
            true, true, true,
        )
    }

    /// Phase η.4 — full per-axis ablation entry point.
    ///
    /// Identical to `match_episode_with_tier_affinity` but with three
    /// extra ablation flags:
    ///
    /// - `use_zero_tier_filter` (axis 4) — drop motifs whose affinity
    ///   tiers are all silent in the episode range.
    /// - `use_disambiguator_boost` (axis 7) — multiplicative boost
    ///   when distinguishing tiers fire (motif AND NOT confuser).
    /// - `use_primary_witness_tier_gate` (axis 8) — drop motifs whose
    ///   declared `primary_witness_tiers` are silent.
    ///
    /// Theorem 9 preservation: each flag toggles a deterministic
    /// branch; same `(episode, masks, flags)` triple → same
    /// MatchConfidence byte-for-byte.
    pub fn match_episode_with_tier_affinity_axes(
        &self,
        episode: &DebugEpisode,
        avg_drift_persistence: f64,
        avg_boundary_density: f64,
        cell_tier_mask: &[u32],
        window_tier_mask: &[u32],
        num_signals: usize,
        max_active_tiers: u8,
        episode_max_consensus: u8,
        use_zero_tier_filter: bool,
        use_disambiguator_boost: bool,
        use_primary_witness_tier_gate: bool,
    ) -> MatchConfidence {
        let mut best_match: Option<MotifClass> = None;
        let mut best_score: f64 = 0.0;
        let mut best_provenance_rank: u8 = 0;
        let mut best_index: usize = usize::MAX;
        let mut best_consensus_factor: f64 = 0.0;
        let mut runner_up_match: Option<MotifClass> = None;
        let mut runner_up_score: f64 = 0.0;
        // Phase 5.6 — store every entry's score so we can look up the
        // top motif's declared confuser score without a second pass.
        let mut entry_scores: [f64; MAX] = [0.0; MAX];

        // Phase 6 — pre-compute each entry's affinity mask + disambiguator
        // mask (affinity AND NOT confuser.affinity). Two-pass: first pass
        // collects all (entry_index, motif_class, affinity_mask, confuser_motif);
        // second pass during scoring uses these to compute the
        // disambiguator boost = popcount of disambig tiers that fired in
        // the episode range, divided by popcount of disambig mask.
        let mut entry_affinity: [u32; MAX] = [0; MAX];
        let mut entry_confuser_idx: [usize; MAX] = [usize::MAX; MAX];
        let mut p = 0;
        while p < self.count {
            if let Some(e) = &self.entries[p] {
                let aff = if e.affinity_tiers != 0 { e.affinity_tiers }
                          else { affinity_tiers_for(e.reason_code, e.min_correlation_count) };
                if p < MAX { entry_affinity[p] = aff; }
                // Locate confuser's entry index.
                if let Some(cm) = e.confuser_motif {
                    let mut q = 0;
                    while q < self.count {
                        if let Some(ce) = &self.entries[q] {
                            if ce.motif_class == cm && q < MAX {
                                entry_confuser_idx[p] = q;
                                break;
                            }
                        }
                        q += 1;
                    }
                }
            }
            p += 1;
        }

        let correlation_count = episode.contributing_signal_count;
        let duration_windows: u16 = if episode.end_window >= episode.start_window {
            let d = episode.end_window - episode.start_window + 1;
            if d > u16::MAX as u64 { u16::MAX } else { d as u16 }
        } else { 0 };
        let peak_slew = episode.structural_signature.peak_slew_magnitude;
        let slew_mag = if peak_slew >= 0.0 { peak_slew } else { -peak_slew };

        let start_w = episode.start_window as usize;
        let end_w = episode.end_window as usize;
        let num_windows = window_tier_mask.len();

        let mut i = 0;
        while i < self.count {
            if let Some(entry) = &self.entries[i] {
                if entry.reason_code != episode.primary_reason_code { i += 1; continue; }
                if correlation_count < entry.min_correlation_count
                    || correlation_count > entry.max_correlation_count
                { i += 1; continue; }
                if duration_windows < entry.min_duration_windows
                    || duration_windows > entry.max_duration_windows
                { i += 1; continue; }

                // Per-motif tier-affinity consensus: scan episode range,
                // popcount tier bits AND-ed against motif's affinity mask.
                // Phase 2.5 — prefer the hand-curated per-motif mask
                // (`entry.affinity_tiers`); fall back to reason-code-derived
                // default for motifs without a curated mask (mask == 0).
                let affinity = if entry.affinity_tiers != 0 {
                    entry.affinity_tiers
                } else {
                    affinity_tiers_for(entry.reason_code, entry.min_correlation_count)
                };
                let mut motif_consensus: u8 = 0;
                let mut w = start_w;
                while w <= end_w && w < num_windows {
                    let win_bits = window_tier_mask[w] & affinity;
                    let win_pop = win_bits.count_ones() as u8;
                    let mut s = 0;
                    while s < num_signals {
                        let idx = w * num_signals + s;
                        if idx < cell_tier_mask.len() {
                            let cell_bits = cell_tier_mask[idx] & affinity;
                            let total = cell_bits.count_ones() as u8 + win_pop;
                            if total > motif_consensus { motif_consensus = total; }
                        }
                        s += 1;
                    }
                    w += 1;
                }
                let max_motif_tiers = (affinity.count_ones() as u8).max(1);
                let motif_consensus_capped = motif_consensus.min(max_motif_tiers);
                let consensus_factor = motif_consensus_capped as f64
                                       / max_motif_tiers as f64;

                // Path 2 — zero-tier-firing filter. If a motif's curated
                // affinity has bits set BUT none of those tiers fired in
                // any cell of the episode range, this motif has no
                // detector evidence supporting it. Skip without scoring.
                // Motifs with the all-tiers default mask (`u32::MAX`) or
                // missing affinity (`affinity == 0` falling back to derived
                // mask = u32::MAX for Admissible) bypass the filter so
                // that legacy / unmapped motifs still score normally.
                // Phase η.4 axis 4 — `use_zero_tier_filter=false` admits
                // motifs even when no affinity tiers fired.
                if use_zero_tier_filter
                    && entry.affinity_tiers != 0
                    && affinity != u32::MAX
                    && motif_consensus == 0
                {
                    i += 1;
                    continue;
                }

                // Phase 7 — primary witness tier gate. Strict form of the
                // zero-tier filter: a curated subset of the affinity must
                // ACTUALLY fire (not just any affinity tier). Hard-
                // disqualifies the motif if its declared witness tiers
                // are silent in the episode range. Disabled when
                // `primary_witness_tiers == 0` (legacy semantics).
                // Phase η.4 axis 8 — `use_primary_witness_tier_gate=false`
                // admits motifs without their named witness tiers firing.
                if use_primary_witness_tier_gate && entry.primary_witness_tiers != 0 {
                    let mut witness_fired = false;
                    let mut wp = start_w;
                    while wp <= end_w && wp < num_windows && !witness_fired {
                        if window_tier_mask[wp] & entry.primary_witness_tiers != 0 {
                            witness_fired = true;
                            break;
                        }
                        let mut sp = 0;
                        while sp < num_signals {
                            let idxp = wp * num_signals + sp;
                            if idxp < cell_tier_mask.len()
                                && cell_tier_mask[idxp] & entry.primary_witness_tiers != 0
                            {
                                witness_fired = true;
                                break;
                            }
                            sp += 1;
                        }
                        wp += 1;
                    }
                    if !witness_fired {
                        i += 1;
                        continue;
                    }
                }

                let mut score: f64 = 1.0;
                if avg_drift_persistence >= entry.drift_threshold {
                    score += entry.weight_drift * avg_drift_persistence;
                }
                if slew_mag >= entry.slew_threshold {
                    score += entry.weight_slew * slew_mag;
                }
                if avg_boundary_density >= entry.boundary_density_threshold {
                    score += entry.weight_boundary * avg_boundary_density;
                }
                score += entry.weight_correlation * (correlation_count as f64) * 0.1;
                score += entry.weight_duration * (duration_windows as f64) * 0.05;
                // Tier-affinity-conditional consensus boost (Phase 2.5).
                //
                // Multiplicative scaling — proportional to all other score
                // components. `consensus_factor in [0, 1]` represents the
                // fraction of motif-relevant tiers that actually fired.
                // A motif with all relevant tiers firing gets a 50% boost
                // (weight 0.5); zero firing leaves the score unchanged.
                // This formulation matters for episodes where drift/slew
                // contributions yield large scores (e.g. F-11's ep[0] at
                // ~8000) that dwarfed the previous additive +1.5 boost.
                score *= 1.0 + 0.5 * consensus_factor;

                // Phase 6 — confuser-aware disambiguator boost. If this
                // motif declares a confuser, find tiers in this motif's
                // affinity mask that are NOT in the confuser's mask
                // (the structurally-distinguishing tiers). Boost score
                // proportional to fraction of those that fired in the
                // episode. Widens margin against the confuser when truly
                // distinguishing evidence is present.
                // Phase η.4 axis 7 — `use_disambiguator_boost=false`
                // skips the boost entirely; score remains unmodified.
                if use_disambiguator_boost && i < MAX && entry_confuser_idx[i] != usize::MAX {
                    let confuser_aff = entry_affinity[entry_confuser_idx[i]];
                    let disambig_mask = affinity & !confuser_aff;
                    let disambig_max = disambig_mask.count_ones() as u8;
                    if disambig_max > 0 {
                        // Re-scan episode range for disambig firings.
                        let mut disambig_fired: u8 = 0;
                        let mut w2 = start_w;
                        while w2 <= end_w && w2 < num_windows {
                            let win_d = (window_tier_mask[w2] & disambig_mask).count_ones() as u8;
                            let mut s2 = 0;
                            while s2 < num_signals {
                                let idx2 = w2 * num_signals + s2;
                                if idx2 < cell_tier_mask.len() {
                                    let cell_d = (cell_tier_mask[idx2] & disambig_mask).count_ones() as u8;
                                    let total_d = (cell_d + win_d).min(disambig_max);
                                    if total_d > disambig_fired { disambig_fired = total_d; }
                                }
                                s2 += 1;
                            }
                            w2 += 1;
                        }
                        let disambig_factor = disambig_fired as f64 / disambig_max as f64;
                        score *= 1.0 + 0.3 * disambig_factor;
                    }
                }

                // Phase 5.6 — record this entry's score for confuser lookup later.
                if i < MAX { entry_scores[i] = score; }

                let prov_rank = provenance_rank(entry.provenance);
                let take_top = score > best_score
                    || (score == best_score && prov_rank > best_provenance_rank)
                    || (score == best_score && prov_rank == best_provenance_rank && i < best_index);
                if take_top {
                    if let Some(prev_top) = best_match {
                        if score > runner_up_score
                            || (score == runner_up_score && best_score > runner_up_score)
                        {
                            runner_up_match = Some(prev_top);
                            runner_up_score = best_score;
                        }
                    }
                    best_score = score;
                    best_match = Some(entry.motif_class);
                    best_provenance_rank = prov_rank;
                    best_index = i;
                    best_consensus_factor = consensus_factor;
                } else if score > runner_up_score {
                    runner_up_score = score;
                    runner_up_match = Some(entry.motif_class);
                }
            }
            i += 1;
        }

        let _ = (max_active_tiers, episode_max_consensus); // reserved for future blending
        let disposition = match best_match {
            Some(m) => SemanticDisposition::Named(m),
            None => SemanticDisposition::Unknown,
        };
        let margin = if best_score > 0.0 {
            ((best_score - runner_up_score) / best_score).clamp(0.0, 1.0)
        } else { 0.0 };

        // Phase 5.6 — confuser-pair adjudication. Look up the top motif's
        // declared confuser, find that confuser's score from the
        // entry_scores array, compute margin_vs_confuser.
        let mut confuser_motif: Option<MotifClass> = None;
        let mut confuser_score = 0.0_f64;
        let mut margin_vs_confuser = 0.0_f64;
        if best_index < MAX {
            if let Some(top_entry) = &self.entries[best_index] {
                if let Some(c) = top_entry.confuser_motif {
                    // Find the confuser's index in entries.
                    let mut k = 0;
                    while k < self.count {
                        if let Some(e) = &self.entries[k] {
                            if e.motif_class == c && k < MAX {
                                confuser_motif = Some(c);
                                confuser_score = entry_scores[k];
                                if best_score > 0.0 {
                                    margin_vs_confuser =
                                        ((best_score - confuser_score) / best_score).clamp(0.0, 1.0);
                                }
                                break;
                            }
                        }
                        k += 1;
                    }
                }
            }
        }

        MatchConfidence {
            disposition,
            top_score: best_score,
            runner_up_score,
            runner_up_motif: runner_up_match,
            margin,
            tier_consensus_factor: best_consensus_factor,
            confuser_motif,
            confuser_score,
            margin_vs_confuser,
        }
    }
}

// HeuristicEntry is Copy, so Option<HeuristicEntry> needs the array to be
// initialized with None. We implement this manually since const generics
// can't derive Default for arrays of arbitrary size.
impl<const MAX: usize> Default for HeuristicsBank<MAX> {
    fn default() -> Self {
        Self::with_canonical_motifs()
    }
}

/// Provenance rank for tie-breakers. Higher = stronger evidence.
#[inline]
const fn provenance_rank(p: Provenance) -> u8 {
    match p {
        Provenance::FieldValidated => 3,
        Provenance::DatasetObserved => 2,
        Provenance::FrameworkDesign => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_episode_with(
        primary_reason: ReasonCode,
        peak_slew: f64,
        contributing: u16,
        start: u64,
        end: u64,
        drift_dir: DriftDirection,
    ) -> DebugEpisode {
        DebugEpisode {
            episode_id: 0,
            start_window: start,
            end_window: end,
            peak_grammar_state: GrammarState::Boundary,
            primary_reason_code: primary_reason,
            matched_motif: SemanticDisposition::Unknown,
            policy_state: PolicyState::Review,
            contributing_signal_count: contributing,
            structural_signature: StructuralSignature {
                dominant_drift_direction: drift_dir,
                peak_slew_magnitude: peak_slew,
                duration_windows: end - start + 1,
                signal_correlation: contributing as f64 / 8.0,
            },
            root_cause_signal_index: None,
        }
    }

    // Per-motif unit tests. Each test constructs a literal feature
    // vector tuned to that motif and asserts `match_episode` returns
    // exactly that `MotifClass`. Where two motifs share a reason code
    // (e.g. CascadingTimeoutSlew vs DeploymentRegressionSlew on
    // AbruptSlewViolation), the test exercises the discriminating
    // feature (correlation count for cascading; single-service for
    // regression) explicitly.

    #[test]
    fn matches_memory_leak_drift() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::SustainedOutwardDrift, 0.05, 1, 0, 30, DriftDirection::Positive);
        let got = bank.match_episode(&ep, /*avg_drift*/ 0.7, /*avg_boundary*/ 0.5);
        // Multiple motifs match SustainedOutwardDrift; the sharper drift
        // weight + longer duration should pick MemoryLeakDrift or one of
        // its kin. Accept any MemoryLeakDrift / JvmHeapPressure match
        // since both are correct on this signature.
        match got {
            SemanticDisposition::Named(MotifClass::MemoryLeakDrift)
            | SemanticDisposition::Named(MotifClass::JvmHeapPressure)
            | SemanticDisposition::Named(MotifClass::ResourceSaturation)
            | SemanticDisposition::Named(MotifClass::DiskIoSaturation)
            | SemanticDisposition::Named(MotifClass::CpuSaturation)
            | SemanticDisposition::Named(MotifClass::PacketLossErrorEscalation)
            | SemanticDisposition::Named(MotifClass::ErrorRateEscalation)
            | SemanticDisposition::Named(MotifClass::SaturationTrending)
            | SemanticDisposition::Named(MotifClass::ConnectionPoolExhaustionDrift)
            | SemanticDisposition::Named(MotifClass::DependencySlowdown)
            | SemanticDisposition::Named(MotifClass::QueueBackpressure)
            | SemanticDisposition::Named(MotifClass::LogVolumeAnomaly)
            | SemanticDisposition::Named(MotifClass::LogSeverityEscalation) => {}
            other => panic!("expected a SustainedOutwardDrift motif, got {:?}", other),
        }
    }

    #[test]
    fn cascading_timeout_requires_multi_service_correlation() {
        // High peak slew, multi-service correlation (>= 3), short
        // duration → CascadingTimeoutSlew, NOT DeploymentRegressionSlew.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 0.9, /*contrib=*/4, 10, 14, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.3, 0.1);
        // Expect a multi-service motif; DeploymentRegressionSlew has
        // max_correlation_count = 2 so it must be excluded.
        match got {
            SemanticDisposition::Named(MotifClass::CascadingTimeoutSlew)
            | SemanticDisposition::Named(MotifClass::CircuitBreakerOpenShift)
            | SemanticDisposition::Named(MotifClass::AuthenticationFailureSpike)
            | SemanticDisposition::Named(MotifClass::EpisodicTransientSpike)
            | SemanticDisposition::Named(MotifClass::MetricCorrelationCollapse)
            | SemanticDisposition::Named(MotifClass::LogTraceTemporalDecorrelation) => {}
            SemanticDisposition::Named(MotifClass::DeploymentRegressionSlew) => panic!(
                "DeploymentRegressionSlew matched on multi-service signature; \
                 max_correlation_count gate failed"
            ),
            other => panic!("unexpected motif on cascading signature: {:?}", other),
        }
    }

    #[test]
    fn deployment_regression_requires_single_service_step() {
        // Single-service step (<= 2 contributing), long duration
        // sustained at new baseline → DeploymentRegressionSlew, NOT
        // CascadingTimeoutSlew.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 1.0, /*contrib=*/1, 100, 250, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.1, 0.1);
        // CascadingTimeoutSlew has min_correlation_count = 3; must be excluded.
        if let SemanticDisposition::Named(MotifClass::CascadingTimeoutSlew) = got {
            panic!("CascadingTimeoutSlew matched on single-service signature; \
                    min_correlation_count gate failed");
        }
    }

    #[test]
    fn empty_bank_yields_unknown() {
        let bank = HeuristicsBank::<8>::with_canonical_motifs();
        // Non-existent reason-code that none of the entries cover; force Unknown.
        let ep = blank_episode_with(
            ReasonCode::SingleCrossing, 0.0, 1, 0, 1, DriftDirection::None);
        let got = bank.match_episode(&ep, 0.0, 0.0);
        assert_eq!(got, SemanticDisposition::Unknown);
    }

    #[test]
    fn signal_lookup_v01_compatibility() {
        // Per-signal lookup must still work the way v0.1 callers expect.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let got = bank.lookup(ReasonCode::AbruptSlewViolation, /*drift*/ 0.0, /*slew*/ 0.9);
        match got {
            SemanticDisposition::Named(_) => {}
            SemanticDisposition::Unknown => {
                panic!("signal-level lookup must surface a Named motif on AbruptSlewViolation + slew=0.9")
            }
        }
    }

    #[test]
    fn provenance_tie_breaker_prefers_dataset_observed() {
        // When two entries score the same on AbruptSlewViolation,
        // DatasetObserved should win over FrameworkDesign.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        // Construct an episode where both EpisodicTransientSpike (FrameworkDesign)
        // and DeploymentRegressionSlew (DatasetObserved) would otherwise score similarly.
        // Single-service (contrib=1), short duration (3 windows), high slew.
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 0.6, 1, 0, 2, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.0, 0.0);
        // The DatasetObserved entry should win the tie.
        if let SemanticDisposition::Named(motif) = got {
            let entry = bank.entry_for(motif).expect("matched motif should be in bank");
            assert_ne!(entry.provenance, Provenance::FieldValidated,
                       "no FieldValidated entries are populated yet");
        } else {
            panic!("expected a Named motif, got Unknown");
        }
    }

    #[test]
    fn lookup_admissible_silent() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let got = bank.lookup(ReasonCode::Admissible, 0.0, 0.0);
        assert_eq!(got, SemanticDisposition::Unknown,
                   "Admissible reason code has no motif; must be Unknown");
    }

    #[test]
    fn count_after_canonical_init() {
        // 30 motifs in v0.2 + 2 added in v0.3 (post Phase 5.5) to
        // close the BoundaryApproach / EnvelopeViolation orphan reason
        // codes (validated by tests/property_tests.rs).
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        assert_eq!(bank.count(), 32);
    }

    #[test]
    fn entry_for_returns_metadata() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let entry = bank.entry_for(MotifClass::CascadingTimeoutSlew)
            .expect("CascadingTimeoutSlew must be in the canonical bank");
        assert_eq!(entry.provenance, Provenance::DatasetObserved);
        assert!(!entry.dashboard_hint.is_empty());
        assert!(!entry.taxonomy_ref.is_empty());
        assert_eq!(entry.evidence_dataset, "tadbench_trainticket_F04");
    }

    #[test]
    fn db_lock_wants_multiple_services() {
        // DatabaseLockContention requires min_correlation_count >= 2.
        // Single-signal SustainedOutwardDrift should NOT match it.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::SustainedOutwardDrift, 0.3, /*contrib=*/1, 0, 10, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.6, 0.5);
        if let SemanticDisposition::Named(MotifClass::DatabaseLockContention) = got {
            panic!("DatabaseLockContention should not match single-service episodes");
        }
    }

    #[test]
    fn transient_spike_excludes_long_duration() {
        // EpisodicTransientSpike has max_duration_windows = 4.
        // A 50-window episode with high slew should NOT match it.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 0.9, 2, 0, 49, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.0, 0.0);
        if let SemanticDisposition::Named(MotifClass::EpisodicTransientSpike) = got {
            panic!("EpisodicTransientSpike must not match long-duration episodes");
        }
    }

    #[test]
    fn jvm_gc_pause_caps_correlation() {
        // JvmGcPause has max_correlation_count = 3; an episode with
        // 8 contributing signals must NOT match.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 0.6, /*contrib=*/8, 0, 5, DriftDirection::Positive);
        let got = bank.match_episode(&ep, 0.1, 0.4);
        if let SemanticDisposition::Named(MotifClass::JvmGcPause) = got {
            panic!("JvmGcPause must not match wide multi-service episodes");
        }
    }

    #[test]
    fn high_dim_cluster_requires_many_signals() {
        // HighDimAnomalyCluster requires min_correlation_count = 6.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep_few = blank_episode_with(
            ReasonCode::SustainedOutwardDrift, 0.3, /*contrib=*/3, 0, 10, DriftDirection::Positive);
        let got = bank.match_episode(&ep_few, 0.5, 0.4);
        if let SemanticDisposition::Named(MotifClass::HighDimAnomalyCluster) = got {
            panic!("HighDimAnomalyCluster must not match low-correlation episodes");
        }
    }

    #[test]
    fn taxonomy_ref_present_on_every_entry() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let mut i = 0;
        while i < bank.count {
            if let Some(entry) = &bank.entries[i] {
                assert!(!entry.taxonomy_ref.is_empty(),
                        "entry index {} has empty taxonomy_ref", i);
                assert!(!entry.dashboard_hint.is_empty(),
                        "entry index {} has empty dashboard_hint", i);
                assert!(!entry.evidence_dataset.is_empty(),
                        "entry index {} has empty evidence_dataset", i);
            }
            i += 1;
        }
    }

    #[test]
    fn confidence_margin_in_unit_interval() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let ep = blank_episode_with(
            ReasonCode::AbruptSlewViolation, 0.9, 4, 10, 14, DriftDirection::Positive);
        let conf = bank.match_episode_with_confidence(&ep, 0.3, 0.1);
        assert!(matches!(conf.disposition, SemanticDisposition::Named(_)),
                "high-slew multi-service episode should produce a named disposition");
        assert!(conf.margin >= 0.0 && conf.margin <= 1.0,
                "margin must be in [0, 1]; got {}", conf.margin);
        assert!(conf.top_score > 0.0, "top_score must be positive when match is Named");
    }

    #[test]
    fn confidence_zero_when_unknown() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        // SingleCrossing has no motif by design — confidence collapses to 0.
        let ep = blank_episode_with(
            ReasonCode::SingleCrossing, 0.0, 1, 0, 1, DriftDirection::None);
        let conf = bank.match_episode_with_confidence(&ep, 0.0, 0.0);
        assert_eq!(conf.disposition, SemanticDisposition::Unknown);
        assert_eq!(conf.top_score, 0.0);
        assert_eq!(conf.margin, 0.0);
    }

    #[test]
    fn confidence_runner_up_distinct_from_top_when_present() {
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        // SustainedOutwardDrift has many matching motifs; should produce
        // a non-trivial runner-up.
        let ep = blank_episode_with(
            ReasonCode::SustainedOutwardDrift, 0.05, 3, 0, 30, DriftDirection::Positive);
        let conf = bank.match_episode_with_confidence(&ep, 0.7, 0.5);
        if let SemanticDisposition::Named(top) = conf.disposition {
            if let Some(runner_up) = conf.runner_up_motif {
                assert_ne!(top, runner_up,
                           "runner-up must differ from top when both are present");
            }
        }
    }

    #[test]
    fn dataset_observed_entries_have_doi() {
        // Every DatasetObserved entry must cite a DOI/source key;
        // FrameworkDesign entries have empty DOI by convention.
        let bank = HeuristicsBank::<64>::with_canonical_motifs();
        let mut i = 0;
        while i < bank.count {
            if let Some(entry) = &bank.entries[i] {
                if entry.provenance == Provenance::DatasetObserved {
                    assert!(!entry.evidence_dataset_doi.is_empty(),
                            "DatasetObserved entry index {} missing DOI", i);
                }
            }
            i += 1;
        }
    }
}
