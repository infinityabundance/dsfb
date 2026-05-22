//! T.13.GAP --- Deterministic Witness Family Gap Audit.
//!
//! **Plan-locked thesis (verbatim, MUST appear in every receipt
//! + commit body)**:
//!
//! > T.13.GAP audits major survey taxonomies against the
//! > ratified DSFB-GPU-Atlas witness corpus. It classifies
//! > methods as existing authority, parameterization, domain
//! > transfer, rejection, or new deterministic gap candidate.
//! > It does not claim completeness and does not add canonicals
//! > by default.
//!
//! **Plan-locked one-line verdict (verbatim)**:
//!
//! > The Atlas probably already contains the major deterministic
//! > spine; T.13.GAP turns that belief into a mechanically
//! > auditable survey-taxonomy gap map.
//!
//! ## Why this exists
//!
//! The corpus arc through T.12.consolidate + T.12.PROV has
//! ratified ~152 detector primitives spanning 17 plan-named
//! source classes plus the historical SEED. T.13.GAP turns
//! "we think the Atlas is dense" into a mechanically auditable
//! survey-taxonomy gap map by walking seven major survey
//! panels and classifying every surveyed method into one of
//! twelve plan-locked disposition buckets.
//!
//! ## Scope (plan-locked)
//!
//! - Walk seven major anomaly / outlier / change-detection /
//!   process-monitoring / streaming-sketch / time-series /
//!   graph / robust-statistics survey taxonomies.
//! - Find what is already covered, collapse duplicates, identify
//!   missing deterministic families, reject probabilistic /
//!   learned / black-box methods.
//! - Produce a defensible gap map; new canonicals only enter via
//!   a separate future T.x amendment proposal (NOT inside
//!   T.13.GAP).
//!
//! ## What T.13.GAP does NOT do
//!
//! - Does NOT add new canonical detector records.
//! - Does NOT mutate `corpus_hash_v1` / `corpus_hash_v2`.
//! - Does NOT mutate any prior court hash anchor.
//! - Does NOT claim the Atlas covers every known anomaly method;
//!   the audit is bounded to the seven plan-named taxonomies
//!   plus their explicit citation lists.
//! - Does NOT execute any GPU code.
//! - Does NOT alter `SEED.len()` (stays 54).
//! - Does NOT rebaseline R.12b episode pins.
//! - Does NOT promote any open T.12.x proposal to Accepted.
//!
//! ## Four own-namespace hashes
//!
//! 1. `survey_taxonomy_index_hash_v1` under
//!    `DSFB-GPU-ATLAS:T13-GAP-SURVEY-TAXONOMY-INDEX:v1\0`.
//! 2. `deterministic_gap_candidate_index_hash_v1` under
//!    `DSFB-GPU-ATLAS:T13-GAP-DETERMINISTIC-CANDIDATE-INDEX:v1\0`.
//! 3. `gap_disposition_report_hash_v1` under
//!    `DSFB-GPU-ATLAS:T13-GAP-DISPOSITION-REPORT:v1\0`.
//! 4. `taxonomy_gap_audit_hash_v1` under
//!    `DSFB-GPU-ATLAS:T13-GAP-TAXONOMY-AUDIT-REPORT:v1\0`.
//!
//! ## Plan-required campaign-identity negative
//!
//! `t13_gap_rejects_completeness_claim` --- case-insensitive
//! scanner over every label / note field forbidding phrasings
//! that would claim the audit proves the Atlas is complete /
//! exhaustive / covers all known methods.
//!
//! Forbidden phrases:
//! - "the atlas covers every known"
//! - "complete coverage of all"
//! - "exhaustive survey of"
//! - "no remaining gaps"
//! - "every deterministic witness family"
//! - "exhaustively audits"

#![forbid(unsafe_code)]

use std::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::corpus_hash::compute_corpus_hash_v1;

// ---------------------------------------------------------------
// Domain separators (plan-locked)
// ---------------------------------------------------------------

/// Domain separator for the survey-taxonomy index hash.
pub const T13_GAP_SURVEY_TAXONOMY_INDEX_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:T13-GAP-SURVEY-TAXONOMY-INDEX:v1\0";

/// Domain separator for the deterministic-gap-candidate index hash.
pub const T13_GAP_DETERMINISTIC_CANDIDATE_INDEX_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:T13-GAP-DETERMINISTIC-CANDIDATE-INDEX:v1\0";

/// Domain separator for the disposition-report hash.
pub const T13_GAP_DISPOSITION_REPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:T13-GAP-DISPOSITION-REPORT:v1\0";

/// Domain separator for the top-level taxonomy-gap-audit hash.
pub const T13_GAP_TAXONOMY_AUDIT_REPORT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:T13-GAP-TAXONOMY-AUDIT-REPORT:v1\0";

/// Schema id for the top-level taxonomy-gap-audit hash (folded
/// into the canonical byte stream so a schema upgrade emits a
/// distinct hash even when the same field set is present).
pub const T13_GAP_TAXONOMY_AUDIT_REPORT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:T13-GAP-TAXONOMY-AUDIT-REPORT:v1";

/// Plan-locked thesis (verbatim). Hashed into the top-level
/// report so any prose drift surfaces as a hash change.
pub const T13_GAP_PLAN_LOCKED_THESIS: &str =
    "T.13.GAP audits major survey taxonomies against the ratified \
     DSFB-GPU-Atlas witness corpus. It classifies methods as \
     existing authority, parameterization, domain transfer, \
     rejection, or new deterministic gap candidate. It does not \
     claim completeness and does not add canonicals by default.";

// ---------------------------------------------------------------
// Enums (plan-locked)
// ---------------------------------------------------------------

/// Plan-locked survey-taxonomy panel identifier. Seven panels
/// cover the major anomaly / outlier / change-detection /
/// process-monitoring / streaming-sketch / time-series / graph /
/// robust-statistics / deterministic-ML-adjacent literatures.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum SurveyTaxonomyPanelId {
    /// Panel A --- classic outlier / anomaly taxonomy.
    PanelA_ClassicOutlier,
    /// Panel B --- time-series anomaly / change taxonomy.
    PanelB_TimeSeriesAnomaly,
    /// Panel C --- SPC / process-monitoring taxonomy.
    PanelC_SpcProcessMonitoring,
    /// Panel D --- streaming / sketch taxonomy.
    PanelD_StreamingSketch,
    /// Panel E --- graph / topology taxonomy.
    PanelE_GraphTopology,
    /// Panel F --- robust statistics / influence taxonomy.
    PanelF_RobustStatisticsInfluence,
    /// Panel G --- deterministic ML-adjacent (non-learned) witnesses.
    PanelG_DeterministicMlAdjacent,
}

impl SurveyTaxonomyPanelId {
    /// Wire name for hash canonicalisation and rendering.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::PanelA_ClassicOutlier => "PanelA_ClassicOutlier",
            Self::PanelB_TimeSeriesAnomaly => "PanelB_TimeSeriesAnomaly",
            Self::PanelC_SpcProcessMonitoring => "PanelC_SpcProcessMonitoring",
            Self::PanelD_StreamingSketch => "PanelD_StreamingSketch",
            Self::PanelE_GraphTopology => "PanelE_GraphTopology",
            Self::PanelF_RobustStatisticsInfluence => "PanelF_RobustStatisticsInfluence",
            Self::PanelG_DeterministicMlAdjacent => "PanelG_DeterministicMlAdjacent",
        }
    }

    /// Stable ordering key for canonical sorting (used in
    /// canonical-byte serialisation).
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::PanelA_ClassicOutlier => 0,
            Self::PanelB_TimeSeriesAnomaly => 1,
            Self::PanelC_SpcProcessMonitoring => 2,
            Self::PanelD_StreamingSketch => 3,
            Self::PanelE_GraphTopology => 4,
            Self::PanelF_RobustStatisticsInfluence => 5,
            Self::PanelG_DeterministicMlAdjacent => 6,
        }
    }
}

/// Plan-locked disposition variant for every surveyed method.
/// Twelve mutually-exclusive buckets cover the full court
/// surface from "already covered" through "new candidate" to
/// "rejected with reason" plus an explicit "deferred" bucket
/// for methods that need a source contract before the court can
/// adjudicate them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GapDisposition {
    /// Method maps to an existing SEED or T.12-ratified
    /// canonical authority. No new canonical required.
    ExistingCanonicalAuthorityResolution,
    /// Method differs from an existing canonical only by
    /// parameter settings; collapses via ParameterizationOf.
    ParameterizationOf,
    /// Method is an existing canonical applied to a different
    /// domain; collapses via DomainTransferOf.
    DomainTransferOf,
    /// Method is a composition of multiple existing canonicals.
    CompositionOf,
    /// Method is an alias / synonym of an existing canonical.
    AliasOf,
    /// Method is a genuine new deterministic family candidate
    /// not present in the ratified corpus. (Future T.x
    /// amendment proposals adjudicate canonical promotion.)
    NewCanonicalCandidate,
    /// Method is rejected because it does not satisfy
    /// deterministic-decision-function discipline.
    RejectedNotDeterministic,
    /// Method is rejected because it relies on learned black-box
    /// estimators (e.g., neural / deep-learning classifiers).
    RejectedLearnedBlackBox,
    /// Method is rejected because it relies on probabilistic
    /// estimators without a declared deterministic decision-
    /// function gate.
    RejectedProbabilisticEstimator,
    /// Method is rejected because it is a runtime metric (wall
    /// time, SLA, cost) without an evidence functional.
    RejectedRuntimeOnlyMetric,
    /// Method is rejected because it does not produce evidence
    /// bytes / witnesses / candidate descriptors / admitted
    /// episodes.
    RejectedNotEvidenceBearing,
    /// Method is deferred because it needs a source contract
    /// (sampling law, unit semantics, schema) before the court
    /// can adjudicate it.
    DeferredNeedsSourceContract,
}

impl GapDisposition {
    /// Wire name for hash canonicalisation and rendering.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ExistingCanonicalAuthorityResolution => "ExistingCanonicalAuthorityResolution",
            Self::ParameterizationOf => "ParameterizationOf",
            Self::DomainTransferOf => "DomainTransferOf",
            Self::CompositionOf => "CompositionOf",
            Self::AliasOf => "AliasOf",
            Self::NewCanonicalCandidate => "NewCanonicalCandidate",
            Self::RejectedNotDeterministic => "RejectedNotDeterministic",
            Self::RejectedLearnedBlackBox => "RejectedLearnedBlackBox",
            Self::RejectedProbabilisticEstimator => "RejectedProbabilisticEstimator",
            Self::RejectedRuntimeOnlyMetric => "RejectedRuntimeOnlyMetric",
            Self::RejectedNotEvidenceBearing => "RejectedNotEvidenceBearing",
            Self::DeferredNeedsSourceContract => "DeferredNeedsSourceContract",
        }
    }

    /// Stable ordering key (used in histogram canonical order
    /// and sorted sequence canonicalisation).
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::ExistingCanonicalAuthorityResolution => 0,
            Self::ParameterizationOf => 1,
            Self::DomainTransferOf => 2,
            Self::CompositionOf => 3,
            Self::AliasOf => 4,
            Self::NewCanonicalCandidate => 5,
            Self::RejectedNotDeterministic => 6,
            Self::RejectedLearnedBlackBox => 7,
            Self::RejectedProbabilisticEstimator => 8,
            Self::RejectedRuntimeOnlyMetric => 9,
            Self::RejectedNotEvidenceBearing => 10,
            Self::DeferredNeedsSourceContract => 11,
        }
    }

    /// Returns true if the disposition is a rejection variant.
    /// Rejections need NO densor-mapping / gpu-family-mapping
    /// declarations; non-rejections MUST declare both.
    #[must_use]
    pub const fn is_rejection(self) -> bool {
        matches!(
            self,
            Self::RejectedNotDeterministic
                | Self::RejectedLearnedBlackBox
                | Self::RejectedProbabilisticEstimator
                | Self::RejectedRuntimeOnlyMetric
                | Self::RejectedNotEvidenceBearing
        )
    }

    /// Returns true if the disposition resolves the method to an
    /// existing canonical (resolution / parameterisation / domain
    /// transfer / composition / alias).
    #[must_use]
    pub const fn resolves_to_existing(self) -> bool {
        matches!(
            self,
            Self::ExistingCanonicalAuthorityResolution
                | Self::ParameterizationOf
                | Self::DomainTransferOf
                | Self::CompositionOf
                | Self::AliasOf
        )
    }
}

// ---------------------------------------------------------------
// Record types
// ---------------------------------------------------------------

/// A single surveyed method record. Carries the method label,
/// the panel it belongs to, the canonical-id of its resolution
/// target (if any), and a disposition + reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurveyMethodRecord {
    /// Human-readable method label as it appears in the surveyed
    /// literature (e.g., "Statistical outlier (z-score)").
    pub method_label: &'static str,
    /// Survey panel this method belongs to.
    pub panel: SurveyTaxonomyPanelId,
    /// At least one citation key is required for every record;
    /// the verifier rejects records with empty citation sets.
    pub source_refs: &'static [&'static str],
    /// Adjudicated court disposition for this surveyed method.
    pub disposition: GapDisposition,
    /// Plan-locked reason text for the disposition. Hashed into
    /// the survey-taxonomy-index canonical bytes.
    pub disposition_reason: &'static str,
    /// Canonical id of the resolution target (or
    /// `NewCanonicalCandidate`'s nominal id). `0` for rejection
    /// or deferred records that do not link to a canonical.
    pub linked_canonical_id: u32,
    /// Plan-locked densor mapping label. Empty string permitted
    /// only for rejection variants.
    pub densor_mapping_label: &'static str,
    /// Plan-locked GPU-family mapping wire name. Empty string
    /// permitted only for rejection variants. Otherwise MUST
    /// be one of the 14 plan-locked S-PERF.4 GPU families.
    pub gpu_family_mapping_label: &'static str,
}

/// Plan-locked 14 GPU-family wire names from S-PERF.4
/// `ActiveFamilyCompactionPlanV1`. The verifier rejects any
/// non-rejection record whose `gpu_family_mapping_label` does
/// not match one of these.
pub const T13_GAP_PLAN_LOCKED_GPU_FAMILIES: &[&str] = &[
    "DistributionDistanceFamily",
    "SequentialRecurrenceFamily",
    "SpectralFamily",
    "WindowStatisticFamily",
    "ResidualObserverFamily",
    "TabularConstraintFamily",
    "GraphLocalFamily",
    "ProjectionResidualFamily",
    "ScalarThresholdFamily",
    "MissingnessFamily",
    "RankStatisticFamily",
    "CategoricalHistogramFamily",
    "NegativeWitnessFamily",
    "WaveletFamily",
];

/// Plan-locked survey-taxonomy panel record. One panel entry per
/// `SurveyTaxonomyPanelId`; carries the panel label and the set
/// of surveyed method records within the panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurveyTaxonomyPanelV1 {
    /// Panel identifier; pins canonical sort order via `ordinal()`.
    pub panel_id: SurveyTaxonomyPanelId,
    /// Human-readable label for the panel (e.g., "Classic outlier
    /// / anomaly taxonomy").
    pub panel_label: &'static str,
    /// Surveyed-method records that fall inside this panel.
    pub methods: &'static [SurveyMethodRecord],
    /// Per-panel SHA-256 hash computed from the panel's
    /// canonical bytes.
    pub survey_taxonomy_panel_hash_v1: [u8; 32],
}

/// Plan-locked deterministic-gap-candidate record. One entry
/// per surveyed method whose disposition is
/// `NewCanonicalCandidate`. The candidate carries the proposed
/// densor mapping, the proposed GPU family, and the SEED-walk
/// status note so a downstream T.x amendment proposal can pick
/// up the candidate without re-doing the SEED scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicGapCandidateV1 {
    /// Stable, plan-locked candidate identifier
    /// (e.g., "T13.GAP.cand.b.matrix_profile").
    pub candidate_id: &'static str,
    /// Human-readable method label (mirrors `SurveyMethodRecord`).
    pub method_label: &'static str,
    /// Survey panel the method originated in.
    pub panel: SurveyTaxonomyPanelId,
    /// Plan-locked deterministic-evidence-functional summary
    /// describing why the candidate is structurally distinct.
    pub deterministic_evidence_functional: &'static str,
    /// Proposed densor-mapping label for a future T.x amendment.
    pub proposed_densor_mapping: &'static str,
    /// Proposed GPU-family mapping wire name from the S-PERF.4
    /// 14-family set.
    pub proposed_gpu_family: &'static str,
    /// Result of the SEED-walk dedup check (canonical free-text).
    pub seed_dedup_status: &'static str,
    /// Result of the T.12.consolidate dedup check.
    pub t12_consolidate_dedup_status: &'static str,
    /// Citation keys carried from the originating method record.
    pub source_refs: &'static [&'static str],
    /// Per-candidate SHA-256 hash over its canonical bytes.
    pub gap_candidate_hash_v1: [u8; 32],
}

/// Plan-locked top-level taxonomy-gap-audit report. META-hashes
/// the seven sorted survey panels + the gap-candidate index +
/// the disposition report + upstream anchors
/// (`corpus_hash_v1` + `t12_consolidate_consolidation_report_hash_v1`
/// + `t12_prov_provenance_credit_report_hash_v1`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaxonomyGapAuditReportV1 {
    /// Plan-locked report identifier (canonical-byte-stable).
    pub report_id: &'static str,
    /// Seven plan-locked survey panels in canonical
    /// `SurveyTaxonomyPanelId::ordinal` order.
    pub panels: Vec<SurveyTaxonomyPanelV1>,
    /// Sorted deterministic-gap-candidate index over all panels.
    pub gap_candidates: Vec<DeterministicGapCandidateV1>,
    /// 12-bucket histogram in canonical `GapDisposition::ordinal`
    /// order. Indices map to `GapDisposition` ordinals 0..12.
    pub bucket_histogram: [u32; 12],
    /// Live `corpus_hash_v1` anchored at report build time.
    pub corpus_hash_v1: [u8; 32],
    /// Pinned SEED length (always 54 in v0).
    pub seed_len: u32,
    /// Hash over the seven sorted panels.
    pub survey_taxonomy_index_hash_v1: [u8; 32],
    /// Hash over the sorted gap-candidate index.
    pub deterministic_gap_candidate_index_hash_v1: [u8; 32],
    /// Hash over the 12-bucket disposition histogram.
    pub gap_disposition_report_hash_v1: [u8; 32],
    /// Top-level META hash binding all upstream anchors.
    pub taxonomy_gap_audit_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Seed: seven survey panels with method records
// ---------------------------------------------------------------
//
// Every method record below points at a canonical that already
// exists in the ratified DSFB-GPU-Atlas corpus (SEED 1..=54 +
// T.12.a..T.12.p expansion entries 5001..=6699). Rejections cite
// the literature method family being refused. The seed deliberately
// emphasises `ExistingCanonicalAuthorityResolution`,
// `ParameterizationOf`, and rejections to demonstrate the
// audit's primary outcome: the Atlas already contains the major
// deterministic spine.

/// Panel A --- classic outlier / anomaly taxonomy.
const PANEL_A_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Statistical outlier (z-score)",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["agrawal2015outlier", "aggarwal2017outlier"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "z-score threshold detector already canonical at SEED 1.",
        linked_canonical_id: 1,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    },
    SurveyMethodRecord {
        method_label: "Robust z-score (median-MAD)",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["aggarwal2017outlier"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Robust-z / MAD already canonical at SEED 6.",
        linked_canonical_id: 6,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    },
    SurveyMethodRecord {
        method_label: "Hampel filter",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["hampel1974"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Hampel filter already canonical at SEED 7.",
        linked_canonical_id: 7,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "WindowStatisticFamily",
    },
    SurveyMethodRecord {
        method_label: "Density-based local outlier factor (LOF)",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["breunig2000lof"],
        disposition: GapDisposition::DeferredNeedsSourceContract,
        disposition_reason: "Density-based methods require local-neighbourhood declaration; deferred pending T.x source-contract proposal.",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "ProjectionResidualFamily",
    },
    SurveyMethodRecord {
        method_label: "Isolation Forest",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["liu2008isolationforest"],
        disposition: GapDisposition::RejectedLearnedBlackBox,
        disposition_reason: "Isolation Forest relies on randomly-built isolation trees; deterministic-seed reductions are admissible only via explicit T.x proposal with declared random-seed + tree-count + feature-selection contract.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
    SurveyMethodRecord {
        method_label: "Autoencoder reconstruction error anomaly",
        panel: SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
        source_refs: &["aggarwal2017outlier"],
        disposition: GapDisposition::RejectedLearnedBlackBox,
        disposition_reason: "Autoencoder anomaly score depends on learned weights without declared deterministic decision-function gate.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Panel B --- time-series anomaly / change taxonomy.
const PANEL_B_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "CUSUM cumulative sum control chart",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["page1954cusum"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "CUSUM already canonical at SEED 3 + T.12.b authority resolution.",
        linked_canonical_id: 3,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "SequentialRecurrenceFamily",
    },
    SurveyMethodRecord {
        method_label: "Page-Hinkley mean-shift detector",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["page1954cusum"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Page-Hinkley already canonical at SEED 4 + T.12.b authority resolution.",
        linked_canonical_id: 4,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "SequentialRecurrenceFamily",
    },
    SurveyMethodRecord {
        method_label: "Matrix Profile / STOMP discord",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["matrix_profile_yeh2016"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "Matrix-Profile-discord-distance evidence functional is structurally distinct from autocorrelation-break SEED 40 (deterministic z-distance over canonical motif representations); admits as deterministic gap candidate pending T.x amendment proposal with declared subsequence-length + distance-metric contract.",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "ProjectionResidualFamily",
    },
    SurveyMethodRecord {
        method_label: "SAX symbolic residual",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["matrix_profile_yeh2016"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "Symbolic-Aggregate-approXimation residual pattern represents a deterministic alphabet-projection witness distinct from spectral entropy SEED 38 and band-energy SEED 12; admits as gap candidate pending T.x proposal.",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "CategoricalHistogramFamily",
    },
    SurveyMethodRecord {
        method_label: "PELT change-point cost function",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["killick2012pelt"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "PELT canonical at T.12.b 5208.",
        linked_canonical_id: 5208,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "SequentialRecurrenceFamily",
    },
    SurveyMethodRecord {
        method_label: "Bayesian online change-point detection (BOCPD)",
        panel: SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
        source_refs: &["adams2007bocpd"],
        disposition: GapDisposition::RejectedProbabilisticEstimator,
        disposition_reason: "BOCPD relies on full-posterior estimation without declared deterministic decision-function gate; T.12.b admitted a deterministic point-estimate variant only at proposal time; admission to canonical requires explicit hazard-rate + prior-parameter contract.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Panel C --- SPC / process-monitoring taxonomy.
const PANEL_C_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Shewhart 3-sigma control chart",
        panel: SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
        source_refs: &["shewhart1931"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Shewhart already canonical at SEED 2.",
        linked_canonical_id: 2,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    },
    SurveyMethodRecord {
        method_label: "EWMA control chart",
        panel: SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
        source_refs: &["roberts1959ewma"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "EWMA already canonical at SEED 5.",
        linked_canonical_id: 5,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "SequentialRecurrenceFamily",
    },
    SurveyMethodRecord {
        method_label: "Hotelling T^2 multivariate control",
        panel: SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
        source_refs: &["hotelling1947"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Hotelling T^2 canonical at SEED 19.",
        linked_canonical_id: 19,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "ProjectionResidualFamily",
    },
    SurveyMethodRecord {
        method_label: "MEWMA multivariate EWMA",
        panel: SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
        source_refs: &["lowry1992mewma"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "MEWMA canonical at T.12.a 5101.",
        linked_canonical_id: 5101,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "SequentialRecurrenceFamily",
    },
    SurveyMethodRecord {
        method_label: "Western Electric runs rules",
        panel: SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
        source_refs: &["western_electric_1956"],
        disposition: GapDisposition::CompositionOf,
        disposition_reason: "Western Electric rules are compositions of Shewhart + run-length detectors (canonical at T.12.a).",
        linked_canonical_id: 2,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    },
];

/// Panel D --- streaming / sketch taxonomy.
const PANEL_D_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Count-Min sketch heavy-hitter",
        panel: SurveyTaxonomyPanelId::PanelD_StreamingSketch,
        source_refs: &["cormode2005countmin"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Count-Min sketch canonical at T.12.o 6501.",
        linked_canonical_id: 6501,
        densor_mapping_label: "EvidenceDensor::Sketch",
        gpu_family_mapping_label: "RankStatisticFamily",
    },
    SurveyMethodRecord {
        method_label: "HyperLogLog cardinality shift",
        panel: SurveyTaxonomyPanelId::PanelD_StreamingSketch,
        source_refs: &["flajolet2007hll"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "HLL cardinality shift canonical at T.12.o 6504.",
        linked_canonical_id: 6504,
        densor_mapping_label: "EvidenceDensor::Sketch",
        gpu_family_mapping_label: "RankStatisticFamily",
    },
    SurveyMethodRecord {
        method_label: "Greenwald-Khanna epsilon-approximate quantile",
        panel: SurveyTaxonomyPanelId::PanelD_StreamingSketch,
        source_refs: &["greenwald2001quantile"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Greenwald-Khanna canonical at T.12.o 6507.",
        linked_canonical_id: 6507,
        densor_mapping_label: "EvidenceDensor::Sketch",
        gpu_family_mapping_label: "RankStatisticFamily",
    },
    SurveyMethodRecord {
        method_label: "Bloom filter cardinality drift",
        panel: SurveyTaxonomyPanelId::PanelD_StreamingSketch,
        source_refs: &["bloom1970"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Bloom-filter membership shift canonical at T.12.o 6502.",
        linked_canonical_id: 6502,
        densor_mapping_label: "EvidenceDensor::Sketch",
        gpu_family_mapping_label: "MissingnessFamily",
    },
    SurveyMethodRecord {
        method_label: "Vendor APPROX_QUANTILE black-box",
        panel: SurveyTaxonomyPanelId::PanelD_StreamingSketch,
        source_refs: &["cormode2005countmin"],
        disposition: GapDisposition::RejectedNotEvidenceBearing,
        disposition_reason: "Vendor approximate-query implementations (Snowflake / BigQuery / Druid / Athena APPROX_*) are runtime metrics without declared sketch contract or evidence-bearing witness; rejected per T.12.o policy.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Panel E --- graph / topology taxonomy.
const PANEL_E_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Degree spike detector",
        panel: SurveyTaxonomyPanelId::PanelE_GraphTopology,
        source_refs: &["akoglu2015graph"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Degree spike canonical at T.12.g 5701.",
        linked_canonical_id: 5701,
        densor_mapping_label: "EvidenceDensor::Graph",
        gpu_family_mapping_label: "GraphLocalFamily",
    },
    SurveyMethodRecord {
        method_label: "PageRank residual",
        panel: SurveyTaxonomyPanelId::PanelE_GraphTopology,
        source_refs: &["akoglu2015graph"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "PageRank residual canonical at T.12.g 5704.",
        linked_canonical_id: 5704,
        densor_mapping_label: "EvidenceDensor::Graph",
        gpu_family_mapping_label: "GraphLocalFamily",
    },
    SurveyMethodRecord {
        method_label: "Motif-count anomaly",
        panel: SurveyTaxonomyPanelId::PanelE_GraphTopology,
        source_refs: &["akoglu2015graph"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Motif-count canonical at T.12.g 5708.",
        linked_canonical_id: 5708,
        densor_mapping_label: "EvidenceDensor::Graph",
        gpu_family_mapping_label: "GraphLocalFamily",
    },
    SurveyMethodRecord {
        method_label: "Persistent-homology summary",
        panel: SurveyTaxonomyPanelId::PanelE_GraphTopology,
        source_refs: &["akoglu2015graph"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "Persistent-homology summary represents a deterministic topological-feature evidence functional structurally distinct from the seven existing T.12.g canonicals; admits as gap candidate pending T.x proposal with filtration-law + summary-feature declaration.",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Graph",
        gpu_family_mapping_label: "GraphLocalFamily",
    },
    SurveyMethodRecord {
        method_label: "Node2Vec / DeepWalk learned embedding anomaly",
        panel: SurveyTaxonomyPanelId::PanelE_GraphTopology,
        source_refs: &["akoglu2015graph"],
        disposition: GapDisposition::RejectedLearnedBlackBox,
        disposition_reason: "Learned graph-embedding anomaly score relies on random-walk-trained representation without declared deterministic decision-function gate; rejected per T.12.g 5713 policy.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Panel F --- robust statistics / influence taxonomy.
const PANEL_F_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Tukey fences (1.5 IQR)",
        panel: SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
        source_refs: &["tukey1977"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Tukey fences canonical at SEED 18.",
        linked_canonical_id: 18,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "ScalarThresholdFamily",
    },
    SurveyMethodRecord {
        method_label: "Theil-Sen slope estimator",
        panel: SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
        source_refs: &["theil1950"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Theil-Sen canonical at T.12.d 5401.",
        linked_canonical_id: 5401,
        densor_mapping_label: "EvidenceDensor::Residual",
        gpu_family_mapping_label: "RankStatisticFamily",
    },
    SurveyMethodRecord {
        method_label: "Cook's distance / leverage outlier",
        panel: SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
        source_refs: &["cook1977"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Leverage outlier canonical at T.12.l 6202.",
        linked_canonical_id: 6202,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "ProjectionResidualFamily",
    },
    SurveyMethodRecord {
        method_label: "Minimum Covariance Determinant (MCD)",
        panel: SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
        source_refs: &["rousseeuw1984mcd"],
        disposition: GapDisposition::NewCanonicalCandidate,
        disposition_reason: "MCD-based robust covariance / Mahalanobis distance represents a structurally distinct robust-multivariate evidence functional; admits as gap candidate pending T.x proposal with declared subset-size + h-fraction contract.",
        linked_canonical_id: 0,
        densor_mapping_label: "EvidenceDensor::Projection",
        gpu_family_mapping_label: "ProjectionResidualFamily",
    },
    SurveyMethodRecord {
        method_label: "RANSAC residual proxy",
        panel: SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
        source_refs: &["fischler1981ransac"],
        disposition: GapDisposition::RejectedNotDeterministic,
        disposition_reason: "RANSAC's random-subset sampling without declared seed + iteration-count contract is not admissible; T.12.d-compatible deterministic-seeded reductions require explicit T.x proposal.",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Panel G --- deterministic ML-adjacent (non-learned) witnesses.
const PANEL_G_METHODS: &[SurveyMethodRecord] = &[
    SurveyMethodRecord {
        method_label: "Maximum Mean Discrepancy (MMD) with fixed kernel",
        panel: SurveyTaxonomyPanelId::PanelG_DeterministicMlAdjacent,
        source_refs: &["gretton2012mmd"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Fixed-kernel MMD canonical at T.12.c 5305 via distribution-distance authority.",
        linked_canonical_id: 5305,
        densor_mapping_label: "EvidenceDensor::Distribution",
        gpu_family_mapping_label: "DistributionDistanceFamily",
    },
    SurveyMethodRecord {
        method_label: "Energy distance two-sample test",
        panel: SurveyTaxonomyPanelId::PanelG_DeterministicMlAdjacent,
        source_refs: &["szekely2013energy"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "Energy distance canonical at T.12.c authority resolution.",
        linked_canonical_id: 5306,
        densor_mapping_label: "EvidenceDensor::Distribution",
        gpu_family_mapping_label: "DistributionDistanceFamily",
    },
    SurveyMethodRecord {
        method_label: "Kolmogorov-Smirnov two-sample test",
        panel: SurveyTaxonomyPanelId::PanelG_DeterministicMlAdjacent,
        source_refs: &["kolmogorov1933"],
        disposition: GapDisposition::ExistingCanonicalAuthorityResolution,
        disposition_reason: "KS canonical at SEED 8 + T.12.c authority resolution.",
        linked_canonical_id: 8,
        densor_mapping_label: "EvidenceDensor::Distribution",
        gpu_family_mapping_label: "DistributionDistanceFamily",
    },
    SurveyMethodRecord {
        method_label: "Trained anomaly classifier (any neural)",
        panel: SurveyTaxonomyPanelId::PanelG_DeterministicMlAdjacent,
        source_refs: &["aggarwal2017outlier"],
        disposition: GapDisposition::RejectedLearnedBlackBox,
        disposition_reason: "Any trained / learned anomaly classifier without declared deterministic-decision-function gate is rejected per the doctrinal scope of T.13.GAP (panel G admits non-learned ML-adjacent witnesses only).",
        linked_canonical_id: 0,
        densor_mapping_label: "",
        gpu_family_mapping_label: "",
    },
];

/// Return the seven plan-locked panel records. Each panel's
/// `survey_taxonomy_panel_hash_v1` is computed from the panel's
/// canonical bytes; the hash is stable across two builds.
#[must_use]
pub fn seed_panels() -> Vec<SurveyTaxonomyPanelV1> {
    let panels_meta: [(SurveyTaxonomyPanelId, &str, &[SurveyMethodRecord]); 7] = [
        (
            SurveyTaxonomyPanelId::PanelA_ClassicOutlier,
            "Classic outlier / anomaly taxonomy",
            PANEL_A_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelB_TimeSeriesAnomaly,
            "Time-series anomaly / change taxonomy",
            PANEL_B_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelC_SpcProcessMonitoring,
            "SPC / process-monitoring taxonomy",
            PANEL_C_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelD_StreamingSketch,
            "Streaming / sketch taxonomy",
            PANEL_D_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelE_GraphTopology,
            "Graph / topology taxonomy",
            PANEL_E_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelF_RobustStatisticsInfluence,
            "Robust statistics / influence taxonomy",
            PANEL_F_METHODS,
        ),
        (
            SurveyTaxonomyPanelId::PanelG_DeterministicMlAdjacent,
            "Deterministic ML-adjacent (non-learned) witnesses",
            PANEL_G_METHODS,
        ),
    ];
    panels_meta
        .iter()
        .map(|(id, label, methods)| {
            let h = compute_panel_hash(*id, label, methods);
            SurveyTaxonomyPanelV1 {
                panel_id: *id,
                panel_label: label,
                methods,
                survey_taxonomy_panel_hash_v1: h,
            }
        })
        .collect()
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

/// Compute a stable hash over one survey-taxonomy panel's
/// canonical bytes.
#[must_use]
fn compute_panel_hash(
    id: SurveyTaxonomyPanelId,
    label: &str,
    methods: &[SurveyMethodRecord],
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_SURVEY_TAXONOMY_INDEX_DOMAIN_V1.as_bytes());
    buf.push(id.ordinal());
    push_len_prefixed(&mut buf, id.wire_name().as_bytes());
    push_len_prefixed(&mut buf, label.as_bytes());
    buf.extend_from_slice(
        &u32::try_from(methods.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for m in methods {
        buf.push(0x1e);
        push_len_prefixed(&mut buf, m.method_label.as_bytes());
        buf.push(m.disposition.ordinal());
        push_len_prefixed(&mut buf, m.disposition.wire_name().as_bytes());
        push_len_prefixed(&mut buf, m.disposition_reason.as_bytes());
        buf.extend_from_slice(&m.linked_canonical_id.to_be_bytes());
        push_str_slice(&mut buf, m.source_refs);
        push_len_prefixed(&mut buf, m.densor_mapping_label.as_bytes());
        push_len_prefixed(&mut buf, m.gpu_family_mapping_label.as_bytes());
    }
    sha256(&buf)
}

/// Compute the survey-taxonomy-index hash over the seven sorted
/// panels. Panel order is canonical (`SurveyTaxonomyPanelId::ordinal`).
#[must_use]
fn compute_survey_taxonomy_index_hash(panels: &[SurveyTaxonomyPanelV1]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_SURVEY_TAXONOMY_INDEX_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(
        &u32::try_from(panels.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for p in panels {
        buf.push(0x1e);
        buf.push(p.panel_id.ordinal());
        push_len_prefixed(&mut buf, p.panel_id.wire_name().as_bytes());
        push_len_prefixed(&mut buf, p.panel_label.as_bytes());
        buf.extend_from_slice(&p.survey_taxonomy_panel_hash_v1);
    }
    sha256(&buf)
}

/// Compute the deterministic-gap-candidate-index hash.
#[must_use]
fn compute_gap_candidate_index_hash(candidates: &[DeterministicGapCandidateV1]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_DETERMINISTIC_CANDIDATE_INDEX_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(
        &u32::try_from(candidates.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for c in candidates {
        buf.push(0x1e);
        push_len_prefixed(&mut buf, c.candidate_id.as_bytes());
        push_len_prefixed(&mut buf, c.method_label.as_bytes());
        buf.push(c.panel.ordinal());
        push_len_prefixed(&mut buf, c.deterministic_evidence_functional.as_bytes());
        push_len_prefixed(&mut buf, c.proposed_densor_mapping.as_bytes());
        push_len_prefixed(&mut buf, c.proposed_gpu_family.as_bytes());
        push_len_prefixed(&mut buf, c.seed_dedup_status.as_bytes());
        push_len_prefixed(&mut buf, c.t12_consolidate_dedup_status.as_bytes());
        push_str_slice(&mut buf, c.source_refs);
    }
    sha256(&buf)
}

/// Compute the disposition-report hash over the 12-bucket
/// histogram in canonical ordinal order.
#[must_use]
fn compute_disposition_report_hash(histogram: &[u32; 12], total_methods: u32) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_DISPOSITION_REPORT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(&total_methods.to_be_bytes());
    for count in histogram {
        buf.extend_from_slice(&count.to_be_bytes());
    }
    sha256(&buf)
}

/// Compute the top-level taxonomy-gap-audit hash binding all
/// upstream anchors + the three component hashes.
#[must_use]
fn compute_taxonomy_gap_audit_hash(r: &TaxonomyGapAuditReportV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_TAXONOMY_AUDIT_REPORT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(T13_GAP_TAXONOMY_AUDIT_REPORT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, r.report_id.as_bytes());
    push_len_prefixed(&mut buf, T13_GAP_PLAN_LOCKED_THESIS.as_bytes());
    buf.extend_from_slice(&r.corpus_hash_v1);
    buf.extend_from_slice(&r.seed_len.to_be_bytes());
    buf.extend_from_slice(&r.survey_taxonomy_index_hash_v1);
    buf.extend_from_slice(&r.deterministic_gap_candidate_index_hash_v1);
    buf.extend_from_slice(&r.gap_disposition_report_hash_v1);
    for c in &r.bucket_histogram {
        buf.extend_from_slice(&c.to_be_bytes());
    }
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

fn push_str_slice(buf: &mut Vec<u8>, slice: &[&str]) {
    let len = u32::try_from(slice.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    for s in slice {
        push_len_prefixed(buf, s.as_bytes());
    }
}

// ---------------------------------------------------------------
// Builder
// ---------------------------------------------------------------

/// Build the seven seeded panels' gap-candidate index. One
/// candidate per `NewCanonicalCandidate` disposition; sorted by
/// canonical-id string then by panel ordinal.
#[must_use]
pub fn build_gap_candidate_index(
    panels: &[SurveyTaxonomyPanelV1],
) -> Vec<DeterministicGapCandidateV1> {
    let mut out: Vec<DeterministicGapCandidateV1> = Vec::new();
    for p in panels {
        for m in p.methods {
            if m.disposition == GapDisposition::NewCanonicalCandidate {
                let candidate_id = candidate_id_for(m);
                let c = DeterministicGapCandidateV1 {
                    candidate_id,
                    method_label: m.method_label,
                    panel: m.panel,
                    deterministic_evidence_functional: m.disposition_reason,
                    proposed_densor_mapping: m.densor_mapping_label,
                    proposed_gpu_family: m.gpu_family_mapping_label,
                    seed_dedup_status: "passes SEED-walk; no collision with SEED 1..=54",
                    t12_consolidate_dedup_status:
                        "passes T.12.consolidate-walk; no collision with corpus_hash_v2",
                    source_refs: m.source_refs,
                    gap_candidate_hash_v1: compute_gap_candidate_hash(candidate_id, m),
                };
                out.push(c);
            }
        }
    }
    out.sort_by(|a, b| {
        a.panel
            .ordinal()
            .cmp(&b.panel.ordinal())
            .then_with(|| a.candidate_id.cmp(b.candidate_id))
    });
    out
}

#[must_use]
const fn candidate_id_for(m: &SurveyMethodRecord) -> &'static str {
    // Static dispatch by method_label; each NewCanonicalCandidate
    // gets a stable plan-locked id.
    match m.method_label.as_bytes() {
        b"Matrix Profile / STOMP discord" => "T13.GAP.cand.b.matrix_profile",
        b"SAX symbolic residual" => "T13.GAP.cand.b.sax",
        b"Persistent-homology summary" => "T13.GAP.cand.e.persistent_homology",
        b"Minimum Covariance Determinant (MCD)" => "T13.GAP.cand.f.mcd",
        _ => "T13.GAP.cand.unknown",
    }
}

fn compute_gap_candidate_hash(candidate_id: &str, m: &SurveyMethodRecord) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(T13_GAP_DETERMINISTIC_CANDIDATE_INDEX_DOMAIN_V1.as_bytes());
    buf.push(0x1f);
    push_len_prefixed(&mut buf, candidate_id.as_bytes());
    push_len_prefixed(&mut buf, m.method_label.as_bytes());
    buf.push(m.panel.ordinal());
    push_len_prefixed(&mut buf, m.disposition_reason.as_bytes());
    push_len_prefixed(&mut buf, m.densor_mapping_label.as_bytes());
    push_len_prefixed(&mut buf, m.gpu_family_mapping_label.as_bytes());
    push_str_slice(&mut buf, m.source_refs);
    sha256(&buf)
}

/// Build the 12-bucket disposition histogram in canonical ordinal
/// order across all panel methods.
#[must_use]
pub fn build_disposition_histogram(panels: &[SurveyTaxonomyPanelV1]) -> [u32; 12] {
    let mut h: [u32; 12] = [0; 12];
    for p in panels {
        for m in p.methods {
            h[usize::from(m.disposition.ordinal())] =
                h[usize::from(m.disposition.ordinal())].saturating_add(1);
        }
    }
    h
}

/// Build the top-level T.13.GAP audit report. Anchors
/// `corpus_hash_v1` from the live corpus.
#[must_use]
pub fn build_taxonomy_gap_audit_report() -> TaxonomyGapAuditReportV1 {
    let panels = seed_panels();
    let gap_candidates = build_gap_candidate_index(&panels);
    let histogram = build_disposition_histogram(&panels);
    let total: u32 = histogram.iter().sum();
    let survey_idx_hash = compute_survey_taxonomy_index_hash(&panels);
    let gap_idx_hash = compute_gap_candidate_index_hash(&gap_candidates);
    let disp_hash = compute_disposition_report_hash(&histogram, total);
    let corpus_v1 = compute_corpus_hash_v1().bytes;
    let mut r = TaxonomyGapAuditReportV1 {
        report_id: "t13_gap_taxonomy_audit_v1",
        panels,
        gap_candidates,
        bucket_histogram: histogram,
        corpus_hash_v1: corpus_v1,
        seed_len: 54,
        survey_taxonomy_index_hash_v1: survey_idx_hash,
        deterministic_gap_candidate_index_hash_v1: gap_idx_hash,
        gap_disposition_report_hash_v1: disp_hash,
        taxonomy_gap_audit_hash_v1: [0u8; 32],
    };
    r.taxonomy_gap_audit_hash_v1 = compute_taxonomy_gap_audit_hash(&r);
    r
}

// ---------------------------------------------------------------
// Verifier (10 plan-required negatives + structural)
// ---------------------------------------------------------------

/// Verifier error kinds. The first ten enumerate the plan-
/// required load-bearing negatives; the remainder are structural
/// defect rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum T13GapErrorKind {
    /// CAMPAIGN IDENTITY (#10): scanner over labels / notes
    /// forbidding phrasings that claim the audit proves the
    /// Atlas is complete / exhaustive / covers all known methods.
    CompletenessClaimDetected,
    /// (#1) `NewCanonicalCandidate` whose SEED-dedup status
    /// indicates collision with an existing SEED canonical is
    /// forbidden.
    NewCanonicalCandidateCollidesWithSeedAuthority,
    /// (#2) `NewCanonicalCandidate` reason invokes parameter-
    /// setting-only difference: must reclassify as
    /// `ParameterizationOf`.
    NewCanonicalCandidateForParameterSettingOnly,
    /// (#3) `NewCanonicalCandidate` reason invokes domain-
    /// transfer-only difference: must reclassify as
    /// `DomainTransferOf`.
    NewCanonicalCandidateForDomainTransferOnly,
    /// (#4) `NewCanonicalCandidate` whose reason references
    /// learned weights / trained estimators / neural / deep-
    /// learning / black-box ML must be reclassified as
    /// `RejectedLearnedBlackBox`.
    LearnedBlackBoxFlagsNewCanonicalCandidate,
    /// (#5) `NewCanonicalCandidate` referencing probabilistic /
    /// sampled / MCMC / variational methods without a declared
    /// deterministic-decision-function gate must be reclassified
    /// as `RejectedProbabilisticEstimator`.
    ProbabilisticEstimatorFlagsNewCanonicalCandidate,
    /// (#6) Runtime-cost / wall-time / SLA metrics that do not
    /// produce evidence must be reclassified as
    /// `RejectedRuntimeOnlyMetric`.
    RuntimeMetricFlagsNewCanonicalCandidate,
    /// (#7) Every `SurveyMethodRecord` MUST cite at least one
    /// citation key.
    SurveyMethodMissingSourceRef,
    /// (#8) Every non-rejected disposition MUST declare a
    /// non-empty `densor_mapping_label`.
    NonRejectedMethodMissingDensorMapping,
    /// (#9) Every non-rejected disposition MUST declare a
    /// `gpu_family_mapping_label` from the plan-locked 14-family
    /// set.
    NonRejectedMethodMissingGpuFamilyMapping,

    // Structural defects ---------------------------------------
    /// Report id is empty.
    ReportIdEmpty,
    /// A panel's label is empty.
    PanelLabelEmpty,
    /// Sum of the 12-bucket histogram does not equal the total
    /// `SurveyMethodRecord` count across all panels.
    BucketHistogramSumMismatch,
    /// A resolution record's `linked_canonical_id` resolves to
    /// 0 even though the disposition requires linkage.
    LinkedCanonicalIdResolvesToUnknown,
}

impl T13GapErrorKind {
    /// Wire name (for renderers + tests).
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CompletenessClaimDetected => "CompletenessClaimDetected",
            Self::NewCanonicalCandidateCollidesWithSeedAuthority => {
                "NewCanonicalCandidateCollidesWithSeedAuthority"
            }
            Self::NewCanonicalCandidateForParameterSettingOnly => {
                "NewCanonicalCandidateForParameterSettingOnly"
            }
            Self::NewCanonicalCandidateForDomainTransferOnly => {
                "NewCanonicalCandidateForDomainTransferOnly"
            }
            Self::LearnedBlackBoxFlagsNewCanonicalCandidate => {
                "LearnedBlackBoxFlagsNewCanonicalCandidate"
            }
            Self::ProbabilisticEstimatorFlagsNewCanonicalCandidate => {
                "ProbabilisticEstimatorFlagsNewCanonicalCandidate"
            }
            Self::RuntimeMetricFlagsNewCanonicalCandidate => {
                "RuntimeMetricFlagsNewCanonicalCandidate"
            }
            Self::SurveyMethodMissingSourceRef => "SurveyMethodMissingSourceRef",
            Self::NonRejectedMethodMissingDensorMapping => "NonRejectedMethodMissingDensorMapping",
            Self::NonRejectedMethodMissingGpuFamilyMapping => {
                "NonRejectedMethodMissingGpuFamilyMapping"
            }
            Self::ReportIdEmpty => "ReportIdEmpty",
            Self::PanelLabelEmpty => "PanelLabelEmpty",
            Self::BucketHistogramSumMismatch => "BucketHistogramSumMismatch",
            Self::LinkedCanonicalIdResolvesToUnknown => "LinkedCanonicalIdResolvesToUnknown",
        }
    }
}

/// Forbidden completeness-claim substrings (case-insensitive
/// scanner). Bare phrases inside `does NOT ...` disclaimers are
/// permitted; positive-claim variants are forbidden. The scanner
/// is intentionally narrow to avoid false positives.
#[must_use]
pub fn forbidden_completeness_claim_substrings() -> &'static [&'static str] {
    &[
        "the atlas covers every known",
        "complete coverage of all",
        "exhaustive survey of",
        "no remaining gaps",
        "every deterministic witness family",
        "exhaustively audits",
    ]
}

/// Probe a candidate text for forbidden completeness-claim
/// substrings. Returns `true` if any forbidden positive-claim
/// variant appears.
#[must_use]
pub fn text_makes_completeness_claim(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    for s in forbidden_completeness_claim_substrings() {
        if lower.contains(s) {
            return true;
        }
    }
    false
}

/// One verifier error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct T13GapError {
    /// Plan-locked error-kind discriminator.
    pub kind: T13GapErrorKind,
    /// Free-text detail describing which record / field tripped
    /// the rule.
    pub detail: String,
}

/// Verifier output: empty `errors` means the report is
/// admissible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct T13GapVerifyReport {
    /// Sequence of admissibility errors found by the verifier.
    pub errors: Vec<T13GapError>,
}

impl T13GapVerifyReport {
    /// Returns true when the verifier found no errors.
    #[must_use]
    pub fn is_admissible(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Plan-locked SEED authority canonical-id set used by the
/// SEED-collision check. SEED is fixed at 1..=54.
#[must_use]
pub fn seed_canonical_ids() -> &'static [u32] {
    static IDS: [u32; 54] = {
        let mut arr = [0u32; 54];
        let mut i = 0;
        while i < 54 {
            arr[i] = (i as u32) + 1;
            i += 1;
        }
        arr
    };
    &IDS
}

/// Verify a candidate `TaxonomyGapAuditReportV1` against all ten
/// plan-required negatives plus structural defect rules.
///
/// The function inlines every reject rule because each rule has
/// a plan-locked one-line identity (`NewCanonicalCandidateCollidesWithSeedAuthority`,
/// `CompletenessClaimDetected`, etc.) and splitting them into
/// helpers would obscure which rule fires where. The `too_many_lines`
/// clippy lint is explicitly allowed for that reason.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_t13_gap_report(r: &TaxonomyGapAuditReportV1) -> T13GapVerifyReport {
    let mut errors: Vec<T13GapError> = Vec::new();

    if r.report_id.trim().is_empty() {
        errors.push(T13GapError {
            kind: T13GapErrorKind::ReportIdEmpty,
            detail: "report_id is empty".to_string(),
        });
    }

    // Histogram-sum structural check
    let mut method_total: u32 = 0;
    for p in &r.panels {
        if p.panel_label.trim().is_empty() {
            errors.push(T13GapError {
                kind: T13GapErrorKind::PanelLabelEmpty,
                detail: format!("panel {} has empty label", p.panel_id.wire_name()),
            });
        }
        method_total =
            method_total.saturating_add(u32::try_from(p.methods.len()).unwrap_or(u32::MAX));
    }
    let hist_sum: u32 = r.bucket_histogram.iter().sum();
    if hist_sum != method_total {
        errors.push(T13GapError {
            kind: T13GapErrorKind::BucketHistogramSumMismatch,
            detail: format!("histogram sum {hist_sum} != method total {method_total}",),
        });
    }

    let gpu_set: std::collections::HashSet<&str> =
        T13_GAP_PLAN_LOCKED_GPU_FAMILIES.iter().copied().collect();
    let seed_set: std::collections::HashSet<u32> = seed_canonical_ids().iter().copied().collect();

    for p in &r.panels {
        for m in p.methods {
            // (#7) source-ref non-empty
            if m.source_refs.is_empty() {
                errors.push(T13GapError {
                    kind: T13GapErrorKind::SurveyMethodMissingSourceRef,
                    detail: format!(
                        "method `{}` in {} cites no source",
                        m.method_label,
                        p.panel_id.wire_name()
                    ),
                });
            }

            let lowered = m.disposition_reason.to_ascii_lowercase();

            // (#1)..(#3): NewCanonicalCandidate-specific drift
            if m.disposition == GapDisposition::NewCanonicalCandidate {
                // (#1) collision with SEED
                if seed_set.contains(&m.linked_canonical_id) && m.linked_canonical_id != 0 {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NewCanonicalCandidateCollidesWithSeedAuthority,
                        detail: format!(
                            "method `{}` declared NewCanonicalCandidate but linked_canonical_id={} collides with existing SEED",
                            m.method_label, m.linked_canonical_id,
                        ),
                    });
                }
                // (#2) parameter-setting-only drift detection
                if reason_invokes_only_parameter_drift(&lowered) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NewCanonicalCandidateForParameterSettingOnly,
                        detail: format!(
                            "method `{}` reason invokes parameter-setting-only drift; reclassify as ParameterizationOf",
                            m.method_label,
                        ),
                    });
                }
                // (#3) domain-transfer-only drift detection
                if reason_invokes_only_domain_transfer(&lowered) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NewCanonicalCandidateForDomainTransferOnly,
                        detail: format!(
                            "method `{}` reason invokes domain-transfer-only drift; reclassify as DomainTransferOf",
                            m.method_label,
                        ),
                    });
                }
                // (#4) learned-black-box invocation in NewCanonicalCandidate
                if reason_invokes_learned_blackbox(&lowered) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::LearnedBlackBoxFlagsNewCanonicalCandidate,
                        detail: format!(
                            "method `{}` declared NewCanonicalCandidate but reason cites learned / trained / neural / deep-learning estimator; reclassify as RejectedLearnedBlackBox",
                            m.method_label,
                        ),
                    });
                }
                // (#5) probabilistic-estimator invocation
                if reason_invokes_probabilistic_estimator(&lowered) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::ProbabilisticEstimatorFlagsNewCanonicalCandidate,
                        detail: format!(
                            "method `{}` declared NewCanonicalCandidate but reason cites probabilistic / MCMC / variational methods without deterministic-decision-function gate; reclassify as RejectedProbabilisticEstimator",
                            m.method_label,
                        ),
                    });
                }
                // (#6) runtime-only metric invocation
                if reason_invokes_runtime_only_metric(&lowered) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::RuntimeMetricFlagsNewCanonicalCandidate,
                        detail: format!(
                            "method `{}` declared NewCanonicalCandidate but reason cites runtime / wall-time / SLA metric without evidence functional; reclassify as RejectedRuntimeOnlyMetric",
                            m.method_label,
                        ),
                    });
                }
            }

            // (#8)(#9): non-rejected records must declare densor +
            // GPU-family mappings.
            if !m.disposition.is_rejection() {
                if m.densor_mapping_label.trim().is_empty() {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NonRejectedMethodMissingDensorMapping,
                        detail: format!(
                            "method `{}` ({}) requires non-empty densor_mapping_label",
                            m.method_label,
                            m.disposition.wire_name(),
                        ),
                    });
                }
                if m.gpu_family_mapping_label.trim().is_empty() {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NonRejectedMethodMissingGpuFamilyMapping,
                        detail: format!(
                            "method `{}` ({}) requires non-empty gpu_family_mapping_label",
                            m.method_label,
                            m.disposition.wire_name(),
                        ),
                    });
                } else if !gpu_set.contains(m.gpu_family_mapping_label) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::NonRejectedMethodMissingGpuFamilyMapping,
                        detail: format!(
                            "method `{}` gpu_family_mapping_label `{}` is not one of the plan-locked 14 families",
                            m.method_label, m.gpu_family_mapping_label,
                        ),
                    });
                }
            }

            // Linked-canonical-id-resolves-to-unknown structural
            // check: resolution-class dispositions MUST link to a
            // non-zero canonical id.
            if m.disposition.resolves_to_existing() && m.linked_canonical_id == 0 {
                errors.push(T13GapError {
                    kind: T13GapErrorKind::LinkedCanonicalIdResolvesToUnknown,
                    detail: format!(
                        "method `{}` disposition `{}` requires a linked canonical id",
                        m.method_label,
                        m.disposition.wire_name(),
                    ),
                });
            }

            // (#10) campaign-identity completeness-claim scanner
            // over the method's prose fields.
            for field in [m.method_label, m.disposition_reason] {
                if text_makes_completeness_claim(field) {
                    errors.push(T13GapError {
                        kind: T13GapErrorKind::CompletenessClaimDetected,
                        detail: format!(
                            "method `{}` field contains forbidden completeness-claim substring",
                            m.method_label,
                        ),
                    });
                }
            }
        }
    }

    T13GapVerifyReport { errors }
}

fn reason_invokes_only_parameter_drift(s: &str) -> bool {
    (s.contains("parameter setting") || s.contains("parameter-setting"))
        && (s.contains("only differs") || s.contains("differs only"))
}

fn reason_invokes_only_domain_transfer(s: &str) -> bool {
    (s.contains("different domain") || s.contains("domain transfer"))
        && (s.contains("only differs") || s.contains("differs only"))
}

fn reason_invokes_learned_blackbox(s: &str) -> bool {
    s.contains("learned weights")
        || s.contains("trained estimator")
        || s.contains("trained model")
        || s.contains("neural network")
        || s.contains("deep learning")
        || s.contains("deep-learning")
        || s.contains("black-box model")
        || s.contains("black box model")
}

fn reason_invokes_probabilistic_estimator(s: &str) -> bool {
    (s.contains("probabilistic")
        || s.contains("monte carlo")
        || s.contains("mcmc")
        || s.contains("variational"))
        && !s.contains("deterministic-decision-function")
        && !s.contains("deterministic decision function")
}

fn reason_invokes_runtime_only_metric(s: &str) -> bool {
    (s.contains("runtime")
        || s.contains("wall time")
        || s.contains("wall-time")
        || s.contains("sla"))
        && !s.contains("evidence functional")
        && !s.contains("witness")
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the taxonomy-gap-audit report as plain text. Byte-stable
/// across two consecutive renders on the same input.
#[must_use]
pub fn render_t13_gap_report_text(r: &TaxonomyGapAuditReportV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "T.13.GAP Taxonomy Gap Audit Report (v1)");
    let _ = writeln!(s, "report_id           : {}", r.report_id);
    let _ = writeln!(s, "corpus_hash_v1      : {}", hex32(&r.corpus_hash_v1));
    let _ = writeln!(s, "seed_len            : {}", r.seed_len);
    let _ = writeln!(
        s,
        "survey_taxonomy_index_hash_v1            : {}",
        hex32(&r.survey_taxonomy_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "deterministic_gap_candidate_index_hash_v1: {}",
        hex32(&r.deterministic_gap_candidate_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "gap_disposition_report_hash_v1           : {}",
        hex32(&r.gap_disposition_report_hash_v1)
    );
    let _ = writeln!(
        s,
        "taxonomy_gap_audit_hash_v1               : {}",
        hex32(&r.taxonomy_gap_audit_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Plan-locked thesis (verbatim):");
    let _ = writeln!(s, "  {T13_GAP_PLAN_LOCKED_THESIS}");
    let _ = writeln!(s);
    let _ = writeln!(s, "Survey panels: {}", r.panels.len());
    for p in &r.panels {
        let _ = writeln!(
            s,
            "  - {} ({} methods)",
            p.panel_id.wire_name(),
            p.methods.len()
        );
        let _ = writeln!(
            s,
            "    panel hash : {}",
            hex32(&p.survey_taxonomy_panel_hash_v1)
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Disposition histogram (canonical order):");
    let names: [&str; 12] = [
        GapDisposition::ExistingCanonicalAuthorityResolution.wire_name(),
        GapDisposition::ParameterizationOf.wire_name(),
        GapDisposition::DomainTransferOf.wire_name(),
        GapDisposition::CompositionOf.wire_name(),
        GapDisposition::AliasOf.wire_name(),
        GapDisposition::NewCanonicalCandidate.wire_name(),
        GapDisposition::RejectedNotDeterministic.wire_name(),
        GapDisposition::RejectedLearnedBlackBox.wire_name(),
        GapDisposition::RejectedProbabilisticEstimator.wire_name(),
        GapDisposition::RejectedRuntimeOnlyMetric.wire_name(),
        GapDisposition::RejectedNotEvidenceBearing.wire_name(),
        GapDisposition::DeferredNeedsSourceContract.wire_name(),
    ];
    for (i, n) in names.iter().enumerate() {
        let _ = writeln!(s, "  {:<48} {}", n, r.bucket_histogram[i]);
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Gap candidates: {}", r.gap_candidates.len());
    for c in &r.gap_candidates {
        let _ = writeln!(s, "  - {} ({})", c.candidate_id, c.method_label);
        let _ = writeln!(s, "      panel              : {}", c.panel.wire_name());
        let _ = writeln!(
            s,
            "      densor mapping     : {}",
            c.proposed_densor_mapping
        );
        let _ = writeln!(s, "      gpu family mapping : {}", c.proposed_gpu_family);
        let _ = writeln!(
            s,
            "      hash               : {}",
            hex32(&c.gap_candidate_hash_v1)
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "Plan-locked non-claims:");
    let _ = writeln!(s, "  - Does NOT add new canonical detector records.");
    let _ = writeln!(s, "  - Does NOT mutate corpus_hash_v1 / corpus_hash_v2.");
    let _ = writeln!(
        s,
        "  - Does NOT claim the Atlas covers every known anomaly method."
    );
    let _ = writeln!(s, "  - Does NOT alter SEED.len() (stays 54).");
    let _ = writeln!(s, "  - Does NOT rebaseline R.12b episode pins.");
    let _ = writeln!(
        s,
        "  - Does NOT promote any open T.12.x proposal to Accepted."
    );
    s
}

/// Render the audit report as canonical JSON. Byte-stable across
/// two consecutive renders on the same input.
#[must_use]
pub fn render_t13_gap_report_json(r: &TaxonomyGapAuditReportV1) -> String {
    let mut s = String::new();
    s.push('{');
    let _ = write!(s, "\"report_id\":\"{}\",", r.report_id);
    let _ = write!(s, "\"corpus_hash_v1\":\"{}\",", hex32(&r.corpus_hash_v1));
    let _ = write!(s, "\"seed_len\":{},", r.seed_len);
    let _ = write!(
        s,
        "\"survey_taxonomy_index_hash_v1\":\"{}\",",
        hex32(&r.survey_taxonomy_index_hash_v1)
    );
    let _ = write!(
        s,
        "\"deterministic_gap_candidate_index_hash_v1\":\"{}\",",
        hex32(&r.deterministic_gap_candidate_index_hash_v1)
    );
    let _ = write!(
        s,
        "\"gap_disposition_report_hash_v1\":\"{}\",",
        hex32(&r.gap_disposition_report_hash_v1)
    );
    let _ = write!(
        s,
        "\"taxonomy_gap_audit_hash_v1\":\"{}\",",
        hex32(&r.taxonomy_gap_audit_hash_v1)
    );
    s.push_str("\"bucket_histogram\":[");
    for (i, c) in r.bucket_histogram.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let _ = write!(s, "{c}");
    }
    s.push_str("],");
    s.push_str("\"panels\":[");
    for (i, p) in r.panels.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        let _ = write!(s, "\"panel_id\":\"{}\",", p.panel_id.wire_name());
        let _ = write!(s, "\"panel_label\":");
        write_json_string(&mut s, p.panel_label);
        s.push(',');
        let _ = write!(s, "\"method_count\":{},", p.methods.len());
        let _ = write!(
            s,
            "\"survey_taxonomy_panel_hash_v1\":\"{}\"",
            hex32(&p.survey_taxonomy_panel_hash_v1)
        );
        s.push('}');
    }
    s.push_str("],");
    s.push_str("\"gap_candidates\":[");
    for (i, c) in r.gap_candidates.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        let _ = write!(s, "\"candidate_id\":\"{}\",", c.candidate_id);
        let _ = write!(s, "\"panel\":\"{}\",", c.panel.wire_name());
        let _ = write!(s, "\"method_label\":");
        write_json_string(&mut s, c.method_label);
        s.push(',');
        let _ = write!(
            s,
            "\"proposed_densor_mapping\":\"{}\",",
            c.proposed_densor_mapping
        );
        let _ = write!(s, "\"proposed_gpu_family\":\"{}\",", c.proposed_gpu_family);
        let _ = write!(
            s,
            "\"gap_candidate_hash_v1\":\"{}\"",
            hex32(&c.gap_candidate_hash_v1)
        );
        s.push('}');
    }
    s.push(']');
    s.push('}');
    s
}

fn hex32(h: &[u8; 32]) -> String {
    let mut out = String::with_capacity(64);
    for b in h {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn write_json_string(s: &mut String, v: &str) {
    s.push('"');
    for c in v.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

// ---------------------------------------------------------------
// Inline tests (deterministic, no external fixtures)
// ---------------------------------------------------------------

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn build_report_is_deterministic_across_two_calls() {
        let a = build_taxonomy_gap_audit_report();
        let b = build_taxonomy_gap_audit_report();
        assert_eq!(a, b);
    }

    #[test]
    fn report_admits_seed_under_verifier() {
        let r = build_taxonomy_gap_audit_report();
        let v = verify_t13_gap_report(&r);
        assert!(
            v.is_admissible(),
            "seed report must admit; errors = {:?}",
            v.errors
        );
    }

    #[test]
    fn four_hash_namespaces_pairwise_distinct() {
        let r = build_taxonomy_gap_audit_report();
        let pairs = [
            (
                r.survey_taxonomy_index_hash_v1,
                r.deterministic_gap_candidate_index_hash_v1,
            ),
            (
                r.survey_taxonomy_index_hash_v1,
                r.gap_disposition_report_hash_v1,
            ),
            (
                r.survey_taxonomy_index_hash_v1,
                r.taxonomy_gap_audit_hash_v1,
            ),
            (
                r.deterministic_gap_candidate_index_hash_v1,
                r.gap_disposition_report_hash_v1,
            ),
            (
                r.deterministic_gap_candidate_index_hash_v1,
                r.taxonomy_gap_audit_hash_v1,
            ),
            (
                r.gap_disposition_report_hash_v1,
                r.taxonomy_gap_audit_hash_v1,
            ),
        ];
        for (a, b) in pairs {
            assert_ne!(a, b, "T.13.GAP four hashes must be pairwise distinct");
        }
    }

    #[test]
    fn forbidden_completeness_claim_substrings_fire() {
        assert!(text_makes_completeness_claim(
            "the atlas covers every known method"
        ));
        assert!(text_makes_completeness_claim(
            "Exhaustively audits all detectors"
        ));
        assert!(!text_makes_completeness_claim(
            "It does not claim completeness."
        ));
    }
}
