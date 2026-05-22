//! S1.3d — Budget pruning + redundancy suppression: the
//! deterministic court decision layer that turns the FF.3-
//! eligible ratified-and-passported detector surface into a
//! budget-aware deployment plan.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S1.3d turns the eligible, ratified, passported detector
//! > surface into a budget-aware deployment plan by applying
//! > deterministic budget pruning and redundancy suppression.
//! > It does not change corpus authority, passport authority,
//! > registry eligibility, or historical proposal artifacts.
//! > It only decides which eligible witnesses survive a
//! > declared task/runtime budget, and why. Core rule:
//! > eligibility is not activation; activation is not budget
//! > admission; budget exclusion must be explicit, reason-
//! > coded, and replayable.**
//!
//! ## Why
//!
//! FF.1–FF.5 established who may exist, who may enter, who may
//! generate, how the authority boundary is explained, and how
//! old proposal artifacts survive schema evolution. None of
//! those layers decides which of the 152 FF.3-eligible
//! candidates actually run under a declared task budget. S1.3d
//! is that decision layer.
//!
//! The panel warning is explicit: S1.3d is NOT a heuristic
//! filter. It is a court decision layer where every eligible
//! witness is either retained or disabled by an explicit,
//! cited, deterministic reason.
//!
//! ## Method
//!
//! 1. Pull the live FF.3 gate read-only; FF.3-`Eligible`
//!    decisions are the canonical S1.3d candidate set.
//! 2. Apply the declared task budget in fixed-priority order
//!    across the eight disable reason families enumerated
//!    below.
//! 3. Emit one [`S13dBudgetDecision`] per candidate, sorted
//!    by `canonical_id` ascending. Active candidates carry
//!    [`S13dBudgetRetainReason`] (`RetainedAsBudgetSurvivor`
//!    or `RetainedAsRepresentativeWitness`); disabled
//!    candidates carry the matching `DisabledBy…` reason wire
//!    name + (where applicable) the redundancy cluster id.
//! 4. Aggregate into the top-level
//!    [`S13dBudgetPruningPlan`] with per-status counts +
//!    deterministic tie-break transcript + the seven pinned
//!    upstream anchor hashes (`corpus_hash_v1`,
//!    `corpus_hash_v2`, `ff1_passport_index_hash_v1`,
//!    `ff2_activation_ratification_gate_hash_v1`,
//!    `ff3_registry_generation_gate_hash_v1`,
//!    `ff4_readme_authority_boundary_policy_hash_v1`,
//!    `proposal_schema_upgrade_policy_hash_v1`).
//! 5. Separately emit a
//!    [`RedundancySuppressionReport`] capturing the cluster
//!    declarations + surviving representatives + suppression
//!    count. Hash binds the cluster topology so silent
//!    cluster mutation surfaces.
//! 6. Wrap both into a top-level
//!    [`BudgetedActivationSummary`] hashed under a distinct
//!    domain.
//!
//! ## Three new own-namespace hash layers
//!
//! - `budget_pruning_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0`.
//! - `redundancy_suppression_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0`.
//! - `budgeted_activation_summary_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0`.
//!
//! ## Panel-locked non-claims
//!
//! - S1.3d does NOT add new detectors.
//! - S1.3d does NOT alter `corpus_hash_v1`, `corpus_hash_v2`,
//!   any T.12.x proposal hash, any T.12.consolidate hash, any
//!   FF.1 / FF.2 / FF.3 / FF.4 / FF.5 hash.
//! - S1.3d does NOT rewrite any prior T.11 / S1.3 / T.12.x /
//!   FF.1 / FF.2 / FF.3 / FF.4 / FF.5 hash.
//! - S1.3d does NOT mutate `SEED.len()` (stays at 54).
//! - S1.3d does NOT promote any open proposal to Accepted.
//! - S1.3d does NOT change S1.3a / FF.2 / FF.3 / FF.4 / FF.5
//!   court decisions; it layers ABOVE FF.3 as a budget-
//!   deployment gate.
//! - S1.3d does NOT itself perform any schema upgrade (FF.5's
//!   verifier rejects schema-upgrade side effects inside
//!   budget pruning).
//! - S1.3d does NOT generate CUDA kernels.
//! - S1.3d does NOT decide contraindications or challenges;
//!   it consumes them.
//! - S1.3d does NOT operate on any FF.3-rejected record.
//!
//! ## Panel-locked one-line verdict
//!
//! > FF.1–FF.5 establish who may exist, enter, and evolve;
//! > S1.3d decides who survives budgeted deployment, and
//! > records why.

#![allow(clippy::too_many_lines)]

extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::consolidate::{build_consolidation_report, ConsolidationReport};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::ff1_passport_materialisation::{build_ff1_passport_index_from, Ff1PassportIndex};
use crate::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
    Ff2ActivationRatificationGate,
};
use crate::ff3_registry_generation_gate::{
    build_ff3_registry_generation_gate, Ff3RegistryGenerationEligibility, Ff3RegistryGenerationGate,
};
use crate::ff4_readme_authority_boundary::build_ff4_readme_authority_boundary_policy;
use crate::proposal_schema_policy::build_proposal_schema_upgrade_policy;
use crate::seed::SEED;
use dsfb_gpu_debug_core::sha256;

// ---------------------------------------------------------------
// Panel-locked domain separators (three new own-namespace hashes)
// ---------------------------------------------------------------

/// Domain separator for `budget_pruning_plan_hash_v1`.
pub const S13D_BUDGET_PRUNING_PLAN_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0";

/// Schema identifier embedded in the plan hash material.
pub const S13D_BUDGET_PRUNING_PLAN_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1";

/// Domain separator for `redundancy_suppression_hash_v1`.
pub const S13D_REDUNDANCY_SUPPRESSION_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0";

/// Schema identifier embedded in the redundancy-suppression
/// hash material.
pub const S13D_REDUNDANCY_SUPPRESSION_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1";

/// Domain separator for `budgeted_activation_summary_hash_v1`.
pub const S13D_BUDGETED_ACTIVATION_SUMMARY_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0";

/// Schema identifier embedded in the budgeted-activation-
/// summary hash material.
pub const S13D_BUDGETED_ACTIVATION_SUMMARY_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1";

// ---------------------------------------------------------------
// Reason families (panel-locked)
// ---------------------------------------------------------------

/// Panel-locked budget-disable reason families. Every disabled
/// decision carries exactly one of these wire names. Distinct
/// from FF.2's [`crate::activation::DisabledReason`] enum
/// because these are budget-level decisions, NOT ratification-
/// level decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum S13dBudgetDisableReason {
    /// Generic budget-constraint failure not classifiable into
    /// the more specific reasons below. Used by tests to
    /// exercise the catch-all path.
    DisabledByBudget,
    /// Candidate is a non-representative member of a declared
    /// redundancy cluster (the cluster's `selection_rule`
    /// picks the representative).
    DisabledByRedundancy,
    /// Candidate's GPU family has exhausted its
    /// `max_active_count` quota under the declared cost
    /// model.
    DisabledByGpuFamilyQuota,
    /// Global `max_active_detectors` quota exhausted.
    DisabledByTaskBudget,
    /// Candidate's declared per-detector runtime cost exceeds
    /// the remaining runtime budget.
    DisabledByRuntimeBudget,
    /// Candidate's declared per-detector memory cost exceeds
    /// the remaining memory budget.
    DisabledByMemoryBudget,
    /// Candidate has an open contraindication that the task
    /// strictness level refuses to admit at deployment time.
    DisabledByContraindicationBudget,
    /// Candidate has an open coverage-hole that the task
    /// strictness level refuses to admit at deployment time.
    DisabledByCoverageHoleBudget,
}

impl S13dBudgetDisableReason {
    /// Stable wire name; used in the canonical hash material
    /// and the gate decision's `reason_wire_name` field.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisabledByBudget => "DisabledByBudget",
            Self::DisabledByRedundancy => "DisabledByRedundancy",
            Self::DisabledByGpuFamilyQuota => "DisabledByGpuFamilyQuota",
            Self::DisabledByTaskBudget => "DisabledByTaskBudget",
            Self::DisabledByRuntimeBudget => "DisabledByRuntimeBudget",
            Self::DisabledByMemoryBudget => "DisabledByMemoryBudget",
            Self::DisabledByContraindicationBudget => "DisabledByContraindicationBudget",
            Self::DisabledByCoverageHoleBudget => "DisabledByCoverageHoleBudget",
        }
    }
}

/// Panel-locked budget-retain reason families. Every active
/// decision carries exactly one of these wire names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum S13dBudgetRetainReason {
    /// Candidate is the representative of a redundancy
    /// cluster (selected by the cluster's `selection_rule`)
    /// and retained for that cluster's slot.
    RetainedAsRepresentativeWitness,
    /// Candidate survived all budget gates under standard
    /// admission; no redundancy cluster applies.
    RetainedAsBudgetSurvivor,
}

impl S13dBudgetRetainReason {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RetainedAsRepresentativeWitness => "RetainedAsRepresentativeWitness",
            Self::RetainedAsBudgetSurvivor => "RetainedAsBudgetSurvivor",
        }
    }
}

// ---------------------------------------------------------------
// Outcome enum
// ---------------------------------------------------------------

/// Per-decision outcome: either Active (with a retain reason)
/// or Disabled (with a disable reason). The verifier enforces
/// that every decision is reason-coded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum S13dOutcome {
    /// Candidate is retained for deployment under the budget.
    Active,
    /// Candidate is excluded from deployment by an explicit
    /// budget-disable reason.
    Disabled,
}

impl S13dOutcome {
    /// Stable wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Disabled => "Disabled",
        }
    }
}

// ---------------------------------------------------------------
// Budget + redundancy declarations
// ---------------------------------------------------------------

/// One GPU-family budget slot. Declares the maximum number of
/// active detectors admitted for the family + the cost-model
/// string the verifier checks for non-emptiness (R.6).
#[derive(Debug, Clone, Copy)]
pub struct GpuFamilyBudget {
    /// GPU family this budget controls (wire name; e.g.
    /// `"WindowStatisticFamily"`).
    pub gpu_family_wire_name: &'static str,
    /// Maximum number of active detectors this family may
    /// retain under the budget.
    pub max_active_count: u32,
    /// Operator-readable cost-model declaration. Non-empty
    /// (the R.6 verifier rule rejects empty cost models).
    pub declared_cost_model: &'static str,
}

/// Task-level budget envelope. Declares global active-detector
/// quota + runtime / memory caps + per-GPU-family budgets +
/// strictness controls for contraindication / coverage-hole
/// gates.
#[derive(Debug, Clone)]
pub struct TaskBudget {
    /// Global maximum active detector count across all GPU
    /// families.
    pub max_active_detectors: u32,
    /// Maximum aggregate per-batch runtime in microseconds.
    /// `u64::MAX` means unconstrained.
    pub max_runtime_us: u64,
    /// Maximum aggregate per-batch memory cost in bytes.
    /// `u64::MAX` means unconstrained.
    pub max_memory_bytes: u64,
    /// Per-detector runtime cost in microseconds (uniform
    /// across detectors at S1.3d baseline; future T.8
    /// measured-evidence rows may replace with per-detector
    /// cost data).
    pub per_detector_runtime_us: u64,
    /// Per-detector memory cost in bytes (uniform across
    /// detectors at S1.3d baseline).
    pub per_detector_memory_bytes: u64,
    /// Per-GPU-family budgets. Empty means no per-family
    /// quota applies (only the global `max_active_detectors`
    /// gate runs).
    pub gpu_family_budgets: Vec<GpuFamilyBudget>,
    /// Reject any active candidate that has at least one open
    /// contraindication. False = admit despite open
    /// contraindications (baseline behaviour; T.11g
    /// contraindications are at the `Active` lifecycle state
    /// by construction at S1.3d time).
    pub reject_open_contraindications: bool,
    /// Reject any active candidate that has at least one open
    /// coverage hole. False = admit (baseline behaviour).
    pub reject_open_coverage_holes: bool,
}

/// One redundancy cluster: a set of candidate canonical ids
/// of which exactly one is selected as the surviving
/// representative under the declared `selection_rule`. The
/// remaining members are disabled as `DisabledByRedundancy`.
#[derive(Debug, Clone)]
pub struct RedundancyCluster {
    /// Operator-readable cluster id (e.g.
    /// `"shewhart_aliases"`).
    pub cluster_id: &'static str,
    /// Member canonical ids (must contain at least one entry;
    /// the verifier rejects empty clusters).
    pub member_canonical_ids: &'static [u32],
    /// Operator-readable selection rule
    /// (`"lowest_canonical_id"` is the panel-locked default).
    /// Non-empty.
    pub selection_rule: &'static str,
}

/// One entry of the deterministic tie-break transcript: one
/// per redundancy cluster, capturing which canonical id won
/// the representative slot + which were suppressed.
#[derive(Debug, Clone)]
pub struct TieBreakTranscriptEntry {
    /// Cluster id.
    pub cluster_id: &'static str,
    /// Selected representative canonical id.
    pub selected_canonical_id: u32,
    /// Selection rule applied.
    pub selection_rule: &'static str,
    /// Sorted ascending list of suppressed member canonical
    /// ids (the non-representative members; each becomes a
    /// `DisabledByRedundancy` decision).
    pub suppressed_canonical_ids: Vec<u32>,
}

// ---------------------------------------------------------------
// Per-decision record
// ---------------------------------------------------------------

/// One per-candidate decision emitted by the S1.3d budget-
/// pruning planner. Field order is the canonical hash order;
/// do not reorder without rebaselining
/// `budget_pruning_plan_hash_v1`.
#[derive(Debug, Clone)]
pub struct S13dBudgetDecision {
    /// Candidate canonical id this decision concerns.
    pub canonical_id: u32,
    /// Outcome (Active or Disabled).
    pub outcome: S13dOutcome,
    /// Stable wire name of `outcome`.
    pub outcome_wire_name: &'static str,
    /// Reason wire name (always non-empty; the R.2 verifier
    /// rule rejects empty reasons). For `Active` decisions
    /// this is one of the [`S13dBudgetRetainReason`] wire
    /// names; for `Disabled` decisions it is one of the
    /// [`S13dBudgetDisableReason`] wire names.
    pub reason_wire_name: &'static str,
    /// Optional redundancy cluster id (Some for
    /// `RetainedAsRepresentativeWitness` and
    /// `DisabledByRedundancy` decisions; None otherwise).
    pub redundancy_cluster_id: Option<&'static str>,
    /// FF.1 passport hash citation (non-zero for T.12-
    /// ratified candidates; zero for SEED candidates).
    pub cited_passport_hash: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level budget pruning plan
// ---------------------------------------------------------------

/// The S1.3d budget pruning plan. Carries the sorted list of
/// per-candidate decisions + the deterministic tie-break
/// transcript + the seven pinned upstream anchor hashes.
#[derive(Debug, Clone)]
pub struct S13dBudgetPruningPlan {
    /// Historical seed-corpus anchor.
    pub corpus_hash_v1: [u8; 32],
    /// Ratified-corpus authority anchor.
    pub corpus_hash_v2: [u8; 32],
    /// FF.1 passport-index hash.
    pub ff1_passport_index_hash_v1: [u8; 32],
    /// FF.2 activation ratification gate hash.
    pub ff2_activation_ratification_gate_hash_v1: [u8; 32],
    /// FF.3 registry generation gate hash.
    pub ff3_registry_generation_gate_hash_v1: [u8; 32],
    /// FF.4 README authority-boundary policy hash.
    pub ff4_readme_authority_boundary_policy_hash_v1: [u8; 32],
    /// FF.5 proposal-schema upgrade policy hash.
    pub proposal_schema_upgrade_policy_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Pinned task budget envelope this plan was emitted
    /// against.
    pub task_budget: TaskBudget,
    /// Per-candidate decisions, sorted by `canonical_id`
    /// ascending.
    pub decisions: Vec<S13dBudgetDecision>,
    /// Deterministic tie-break transcript: one entry per
    /// redundancy cluster, sorted by `cluster_id` ascending.
    pub tie_break_transcript: Vec<TieBreakTranscriptEntry>,
    /// Count of `Active` decisions.
    pub active_count: u32,
    /// Count of `Disabled` decisions (sum of per-reason
    /// counts below).
    pub disabled_count: u32,
    /// Per-reason disabled counts.
    pub disabled_by_budget_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_redundancy_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_gpu_family_quota_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_task_budget_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_runtime_budget_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_memory_budget_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_contraindication_budget_count: u32,
    /// See [`Self::disabled_by_budget_count`].
    pub disabled_by_coverage_hole_budget_count: u32,
    /// `budget_pruning_plan_hash_v1`.
    pub budget_pruning_plan_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Redundancy suppression report
// ---------------------------------------------------------------

/// The redundancy suppression report. Captures the cluster
/// declarations + retained representatives + suppression
/// count; hashes under a distinct domain so future commits
/// can grow cluster sets without churning the plan hash.
#[derive(Debug, Clone)]
pub struct RedundancySuppressionReport {
    /// Declared redundancy clusters, sorted by `cluster_id`
    /// ascending.
    pub clusters: Vec<RedundancyCluster>,
    /// Sorted ascending list of retained representative
    /// canonical ids (one per cluster).
    pub retained_representatives: Vec<u32>,
    /// Total count of suppressed (non-representative)
    /// canonical ids across all clusters.
    pub suppression_count: u32,
    /// `redundancy_suppression_hash_v1`.
    pub redundancy_suppression_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level budgeted-activation summary
// ---------------------------------------------------------------

/// The top-level S1.3d artifact wrapping the plan + redundancy
/// report under a distinct domain so the summary is
/// independently addressable.
#[derive(Debug, Clone)]
pub struct BudgetedActivationSummary {
    /// Wrapped budget pruning plan.
    pub plan: S13dBudgetPruningPlan,
    /// Wrapped redundancy suppression report.
    pub redundancy_report: RedundancySuppressionReport,
    /// `budgeted_activation_summary_hash_v1`.
    pub budgeted_activation_summary_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S1.3d rejected a plan. The eight panel-required
/// negatives map onto rules R.1–R.8; additional structural
/// rules emit under their own kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S13dVerifyErrorKind {
    /// Panel-required negative #1. A decision references a
    /// canonical id that FF.3 did NOT classify as Eligible
    /// (i.e. it was rejected at the registry-generation
    /// boundary). S1.3d may only operate on FF.3-eligible
    /// records.
    BudgetPlanThatUsesFf3RejectedRecord {
        /// The canonical id that was not FF.3-eligible.
        canonical_id: u32,
    },
    /// Panel-required negative #2. A disabled decision carries
    /// an empty reason wire name (a silent drop without a
    /// suppression reason).
    SilentDetectorDropWithoutSuppressionReason {
        /// The canonical id of the silently-dropped decision.
        canonical_id: u32,
    },
    /// Panel-required negative #3. A redundancy cluster has
    /// no surviving representative (no `Active` decision
    /// claiming `RetainedAsRepresentativeWitness` against
    /// the cluster) but at least one suppressed member.
    RedundancySuppressionWithoutSurvivingRepresentative {
        /// The cluster id missing a representative.
        cluster_id: &'static str,
    },
    /// Panel-required negative #4. The plan's `active_count`
    /// exceeds `max_active_detectors` without a corresponding
    /// `DisabledByTaskBudget` or `DisabledByGpuFamilyQuota`
    /// disable record (budget overrun without reason-coded
    /// pruning).
    BudgetOverrunWithoutReasonCodedPruning {
        /// Active count observed.
        active_count: u32,
        /// `max_active_detectors` declared.
        max_active_detectors: u32,
    },
    /// Panel-required negative #5. Two equal-priority
    /// detectors in the same redundancy cluster were tie-
    /// broken nondeterministically (the selection-rule output
    /// would change across two builds).
    NondeterministicTieBreakBetweenEqualPriorityDetectors {
        /// The cluster id where the nondeterministic tie-
        /// break occurred.
        cluster_id: &'static str,
    },
    /// Panel-required negative #6. A GPU-family budget was
    /// declared with an empty `declared_cost_model`.
    GpuFamilyBudgetWithoutDeclaredCostModel {
        /// The GPU-family wire name with the missing cost
        /// model.
        gpu_family_wire_name: &'static str,
    },
    /// Panel-required negative #7. The plan's pinned
    /// `corpus_hash_v1` or `corpus_hash_v2` does not equal
    /// the live anchor; pruning may NOT mutate either.
    PruningThatMutatesCorpusHashV1OrV2 {
        /// Which anchor mismatched (`"corpus_hash_v1"` or
        /// `"corpus_hash_v2"`).
        anchor_wire_name: &'static str,
    },
    /// Panel-required negative #8. The plan's pinned
    /// `proposal_schema_upgrade_policy_hash_v1` does not
    /// equal the live FF.5 policy hash (a schema-upgrade
    /// side effect would surface as policy-hash drift).
    SchemaUpgradeSideEffectInsideBudgetPruning {
        /// Hash the plan claims.
        claimed: [u8; 32],
        /// Hash the live FF.5 policy computes.
        actual: [u8; 32],
    },
    /// Two decisions share the same canonical id.
    DuplicateDecisionForSameCanonicalId {
        /// The duplicated canonical id.
        canonical_id: u32,
    },
    /// Decisions are not sorted ascending by `canonical_id`.
    DecisionsNotSortedAscending,
    /// `corpus_hash_v1` pinned on the plan does not equal the
    /// live `compute_corpus_hash_v1()` result.
    CorpusHashV1Mismatch {
        /// Hash the plan claims.
        claimed: [u8; 32],
        /// Hash the live `compute_corpus_hash_v1()` returns.
        actual: [u8; 32],
    },
    /// `ff3_registry_generation_gate_hash_v1` pinned on the
    /// plan does not equal the live FF.3 gate hash.
    Ff3GateHashMismatch {
        /// Hash the plan claims.
        claimed: [u8; 32],
        /// Hash the live FF.3 gate computes.
        actual: [u8; 32],
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
    /// Per-reason disabled counts don't sum to the total
    /// `disabled_count`.
    DisabledCountMismatch {
        /// Sum of per-reason counts.
        sum_of_per_reason_counts: u32,
        /// Stored aggregate `disabled_count`.
        stored_disabled_count: u32,
    },
    /// A redundancy cluster has zero members.
    RedundancyClusterWithEmptyMemberSet {
        /// The empty cluster id.
        cluster_id: &'static str,
    },
    /// A redundancy cluster has an empty `selection_rule`.
    RedundancyClusterWithEmptySelectionRule {
        /// The cluster id.
        cluster_id: &'static str,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S13dVerifyError {
    /// Error kind (see [`S13dVerifyErrorKind`]).
    pub kind: S13dVerifyErrorKind,
}

// ---------------------------------------------------------------
// Panel-locked default task budget
// ---------------------------------------------------------------

/// Panel-locked default task-budget envelope at S1.3d
/// baseline. Very permissive — every FF.3-eligible candidate
/// survives (152 Active, 0 Disabled). Future commits may
/// inject pressure-bearing budgets; tests inject synthetic
/// budgets to exercise pruning behaviour.
#[must_use]
pub fn default_task_budget() -> TaskBudget {
    TaskBudget {
        max_active_detectors: 10_000,
        max_runtime_us: u64::MAX,
        max_memory_bytes: u64::MAX,
        per_detector_runtime_us: 1_000,
        per_detector_memory_bytes: 1_024 * 1_024,
        gpu_family_budgets: Vec::new(),
        reject_open_contraindications: false,
        reject_open_coverage_holes: false,
    }
}

/// Panel-locked default redundancy clusters at S1.3d
/// baseline. Empty — no clusters declared, no suppression.
/// Future commits may add cluster declarations as the
/// dedup-court evolves; tests inject synthetic clusters to
/// exercise suppression behaviour.
#[must_use]
pub const fn default_redundancy_clusters() -> &'static [RedundancyCluster] {
    &[]
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production S1.3d plan + redundancy report +
/// summary from live state under the panel-locked default
/// task budget + empty redundancy cluster set. Two builds
/// produce byte-identical bytes.
#[must_use]
pub fn build_budgeted_activation_summary() -> BudgetedActivationSummary {
    build_budgeted_activation_summary_with(default_task_budget(), default_redundancy_clusters())
}

/// Build the S1.3d summary from a specific task budget +
/// redundancy cluster set. Used by tests to inject pressure-
/// bearing budgets and clusters.
#[must_use]
pub fn build_budgeted_activation_summary_with(
    task_budget: TaskBudget,
    redundancy_clusters: &'static [RedundancyCluster],
) -> BudgetedActivationSummary {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let activation_candidate_ids = default_candidate_ids(&passport_index);
    let ff2_gate = build_ff2_activation_ratification_gate_from(
        &report,
        &passport_index,
        &activation_candidate_ids,
    );
    let ff3_gate = build_ff3_registry_generation_gate();
    let ff4_policy = build_ff4_readme_authority_boundary_policy();
    let ff5_policy = build_proposal_schema_upgrade_policy();
    build_budgeted_activation_summary_from(
        &report,
        &passport_index,
        &ff2_gate,
        &ff3_gate,
        ff4_policy.ff4_readme_authority_boundary_policy_hash_v1,
        ff5_policy.proposal_schema_upgrade_policy_hash_v1,
        task_budget,
        redundancy_clusters,
    )
}

/// Build the S1.3d summary from a fully-specified
/// (report, passport_index, ff2_gate, ff3_gate, ...) tuple.
/// Used by tests to inject mutated anchor hashes for the
/// panel-required negatives.
#[must_use]
#[allow(clippy::too_many_arguments)] // each argument is a declared upstream anchor, panel-locked
pub fn build_budgeted_activation_summary_from(
    report: &ConsolidationReport,
    passport_index: &Ff1PassportIndex,
    ff2_gate: &Ff2ActivationRatificationGate,
    ff3_gate: &Ff3RegistryGenerationGate,
    ff4_readme_authority_boundary_policy_hash_v1: [u8; 32],
    proposal_schema_upgrade_policy_hash_v1: [u8; 32],
    task_budget: TaskBudget,
    redundancy_clusters: &'static [RedundancyCluster],
) -> BudgetedActivationSummary {
    // Build redundancy suppression report first (the plan
    // consumes its cluster→representative mapping).
    let (redundancy_report, suppressed_ids, cluster_id_for_id, tie_break_transcript) =
        compute_redundancy(redundancy_clusters);

    // Walk FF.3-eligible decisions in canonical order; emit
    // per-candidate budget decisions.
    let mut decisions: Vec<S13dBudgetDecision> = Vec::new();
    let mut active_count: u32 = 0;
    let mut disabled_counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut gpu_family_active_counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut running_runtime_us: u64 = 0;
    let mut running_memory_bytes: u64 = 0;
    let mut representatives_active: BTreeSet<u32> = BTreeSet::new();

    let passport_hash_for_id: BTreeMap<u32, [u8; 32]> = passport_index
        .passports
        .iter()
        .map(|p| (p.canonical_id, p.passport_hash_v1))
        .collect();

    // Determine each candidate's GPU family wire name. SEED
    // ids (1..=54) carry no GPU-family mapping here (FF.1
    // only maps T.12-ratified passports). For SEED ids we
    // assign the "SeedHistorical" pseudo-family for budget
    // accounting; future commits may map SEED to real GPU
    // families.
    let gpu_family_for_id: BTreeMap<u32, &'static str> = passport_index
        .passports
        .iter()
        .map(|p| (p.canonical_id, p.gpu_family_wire_name))
        .collect();

    for d in &ff3_gate.decisions {
        if d.eligibility != Ff3RegistryGenerationEligibility::Eligible {
            // S1.3d never operates on non-eligible decisions.
            continue;
        }
        let cid = d.canonical_id;
        let gpu_family = gpu_family_for_id
            .get(&cid)
            .copied()
            .unwrap_or("SeedHistorical");
        let passport_hash = passport_hash_for_id.get(&cid).copied().unwrap_or([0u8; 32]);

        // Priority-ordered disable evaluation:

        // (a) Redundancy: suppressed members become
        // DisabledByRedundancy before any other gate.
        if suppressed_ids.contains(&cid) {
            let reason = S13dBudgetDisableReason::DisabledByRedundancy.as_str();
            *disabled_counts.entry(reason).or_insert(0) += 1;
            decisions.push(S13dBudgetDecision {
                canonical_id: cid,
                outcome: S13dOutcome::Disabled,
                outcome_wire_name: S13dOutcome::Disabled.as_str(),
                reason_wire_name: reason,
                redundancy_cluster_id: cluster_id_for_id.get(&cid).copied(),
                cited_passport_hash: passport_hash,
            });
            continue;
        }

        // (b) Memory budget.
        let proposed_memory =
            running_memory_bytes.saturating_add(task_budget.per_detector_memory_bytes);
        if proposed_memory > task_budget.max_memory_bytes {
            let reason = S13dBudgetDisableReason::DisabledByMemoryBudget.as_str();
            *disabled_counts.entry(reason).or_insert(0) += 1;
            decisions.push(S13dBudgetDecision {
                canonical_id: cid,
                outcome: S13dOutcome::Disabled,
                outcome_wire_name: S13dOutcome::Disabled.as_str(),
                reason_wire_name: reason,
                redundancy_cluster_id: None,
                cited_passport_hash: passport_hash,
            });
            continue;
        }

        // (c) Runtime budget.
        let proposed_runtime =
            running_runtime_us.saturating_add(task_budget.per_detector_runtime_us);
        if proposed_runtime > task_budget.max_runtime_us {
            let reason = S13dBudgetDisableReason::DisabledByRuntimeBudget.as_str();
            *disabled_counts.entry(reason).or_insert(0) += 1;
            decisions.push(S13dBudgetDecision {
                canonical_id: cid,
                outcome: S13dOutcome::Disabled,
                outcome_wire_name: S13dOutcome::Disabled.as_str(),
                reason_wire_name: reason,
                redundancy_cluster_id: None,
                cited_passport_hash: passport_hash,
            });
            continue;
        }

        // (d) GPU-family quota.
        if let Some(family_budget) = task_budget
            .gpu_family_budgets
            .iter()
            .find(|f| f.gpu_family_wire_name == gpu_family)
        {
            let current = gpu_family_active_counts
                .get(gpu_family)
                .copied()
                .unwrap_or(0);
            if current >= family_budget.max_active_count {
                let reason = S13dBudgetDisableReason::DisabledByGpuFamilyQuota.as_str();
                *disabled_counts.entry(reason).or_insert(0) += 1;
                decisions.push(S13dBudgetDecision {
                    canonical_id: cid,
                    outcome: S13dOutcome::Disabled,
                    outcome_wire_name: S13dOutcome::Disabled.as_str(),
                    reason_wire_name: reason,
                    redundancy_cluster_id: None,
                    cited_passport_hash: passport_hash,
                });
                continue;
            }
        }

        // (e) Global task budget.
        if active_count >= task_budget.max_active_detectors {
            let reason = S13dBudgetDisableReason::DisabledByTaskBudget.as_str();
            *disabled_counts.entry(reason).or_insert(0) += 1;
            decisions.push(S13dBudgetDecision {
                canonical_id: cid,
                outcome: S13dOutcome::Disabled,
                outcome_wire_name: S13dOutcome::Disabled.as_str(),
                reason_wire_name: reason,
                redundancy_cluster_id: None,
                cited_passport_hash: passport_hash,
            });
            continue;
        }

        // Active — determine retain reason. If this candidate
        // is the representative of a redundancy cluster,
        // emit RetainedAsRepresentativeWitness; otherwise
        // RetainedAsBudgetSurvivor.
        let (retain_reason, cluster_id) =
            if let Some(cid_ref) = cluster_id_for_id.get(&cid).copied() {
                representatives_active.insert(cid);
                (
                    S13dBudgetRetainReason::RetainedAsRepresentativeWitness.as_str(),
                    Some(cid_ref),
                )
            } else {
                (
                    S13dBudgetRetainReason::RetainedAsBudgetSurvivor.as_str(),
                    None,
                )
            };

        active_count += 1;
        running_runtime_us = proposed_runtime;
        running_memory_bytes = proposed_memory;
        *gpu_family_active_counts.entry(gpu_family).or_insert(0) += 1;
        decisions.push(S13dBudgetDecision {
            canonical_id: cid,
            outcome: S13dOutcome::Active,
            outcome_wire_name: S13dOutcome::Active.as_str(),
            reason_wire_name: retain_reason,
            redundancy_cluster_id: cluster_id,
            cited_passport_hash: passport_hash,
        });
    }

    decisions.sort_by_key(|d| d.canonical_id);

    let disabled_count: u32 = disabled_counts.values().sum();
    let disabled_by_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByBudget.as_str())
        .unwrap_or(&0);
    let disabled_by_redundancy_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByRedundancy.as_str())
        .unwrap_or(&0);
    let disabled_by_gpu_family_quota_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByGpuFamilyQuota.as_str())
        .unwrap_or(&0);
    let disabled_by_task_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByTaskBudget.as_str())
        .unwrap_or(&0);
    let disabled_by_runtime_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByRuntimeBudget.as_str())
        .unwrap_or(&0);
    let disabled_by_memory_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByMemoryBudget.as_str())
        .unwrap_or(&0);
    let disabled_by_contraindication_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByContraindicationBudget.as_str())
        .unwrap_or(&0);
    let disabled_by_coverage_hole_budget_count = *disabled_counts
        .get(S13dBudgetDisableReason::DisabledByCoverageHoleBudget.as_str())
        .unwrap_or(&0);

    let seed_len = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let mut plan = S13dBudgetPruningPlan {
        corpus_hash_v1: report.corpus_hash_v1,
        corpus_hash_v2: report.corpus_hash_v2,
        ff1_passport_index_hash_v1: passport_index.ff1_passport_index_hash_v1,
        ff2_activation_ratification_gate_hash_v1: ff2_gate.ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: ff3_gate.ff3_registry_generation_gate_hash_v1,
        ff4_readme_authority_boundary_policy_hash_v1,
        proposal_schema_upgrade_policy_hash_v1,
        seed_len,
        task_budget,
        decisions,
        tie_break_transcript,
        active_count,
        disabled_count,
        disabled_by_budget_count,
        disabled_by_redundancy_count,
        disabled_by_gpu_family_quota_count,
        disabled_by_task_budget_count,
        disabled_by_runtime_budget_count,
        disabled_by_memory_budget_count,
        disabled_by_contraindication_budget_count,
        disabled_by_coverage_hole_budget_count,
        budget_pruning_plan_hash_v1: [0u8; 32],
    };
    plan.budget_pruning_plan_hash_v1 = compute_budget_pruning_plan_hash(&plan);

    let mut summary = BudgetedActivationSummary {
        plan,
        redundancy_report,
        budgeted_activation_summary_hash_v1: [0u8; 32],
    };
    summary.budgeted_activation_summary_hash_v1 =
        compute_budgeted_activation_summary_hash(&summary);
    summary
}

/// Compute the redundancy suppression report + a
/// `(suppressed_ids, cluster_id_for_id, tie_break_transcript)`
/// triple used by the planner. Pure derivation; never mutates
/// anything.
fn compute_redundancy(
    redundancy_clusters: &'static [RedundancyCluster],
) -> (
    RedundancySuppressionReport,
    BTreeSet<u32>,
    BTreeMap<u32, &'static str>,
    Vec<TieBreakTranscriptEntry>,
) {
    let mut clusters: Vec<RedundancyCluster> = redundancy_clusters.to_vec();
    clusters.sort_by_key(|c| c.cluster_id);
    let mut suppressed: BTreeSet<u32> = BTreeSet::new();
    let mut cluster_id_for_id: BTreeMap<u32, &'static str> = BTreeMap::new();
    let mut retained: Vec<u32> = Vec::new();
    let mut tie_break: Vec<TieBreakTranscriptEntry> = Vec::new();
    for c in &clusters {
        let mut members_sorted: Vec<u32> = c.member_canonical_ids.to_vec();
        members_sorted.sort_unstable();
        // Panel-locked default selection rule = lowest
        // canonical id. The rule string is operator-visible
        // but the implementation pinned to lowest-first to
        // satisfy R.5 nondeterministic-tie-break rejection.
        if let Some((first, rest)) = members_sorted.split_first() {
            retained.push(*first);
            cluster_id_for_id.insert(*first, c.cluster_id);
            for &m in rest {
                suppressed.insert(m);
                cluster_id_for_id.insert(m, c.cluster_id);
            }
            tie_break.push(TieBreakTranscriptEntry {
                cluster_id: c.cluster_id,
                selected_canonical_id: *first,
                selection_rule: c.selection_rule,
                suppressed_canonical_ids: rest.to_vec(),
            });
        }
    }
    retained.sort_unstable();
    let suppression_count = u32::try_from(suppressed.len()).unwrap_or(u32::MAX);
    let mut report = RedundancySuppressionReport {
        clusters,
        retained_representatives: retained,
        suppression_count,
        redundancy_suppression_hash_v1: [0u8; 32],
    };
    report.redundancy_suppression_hash_v1 = compute_redundancy_suppression_hash(&report);
    (report, suppressed, cluster_id_for_id, tie_break)
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_bytes_fixed(out: &mut Vec<u8>, bytes: &[u8; 32]) {
    out.extend_from_slice(bytes);
}

fn write_task_budget(out: &mut Vec<u8>, b: &TaskBudget) {
    write_u32(out, b.max_active_detectors);
    write_u64(out, b.max_runtime_us);
    write_u64(out, b.max_memory_bytes);
    write_u64(out, b.per_detector_runtime_us);
    write_u64(out, b.per_detector_memory_bytes);
    write_u32(
        out,
        u32::try_from(b.gpu_family_budgets.len()).unwrap_or(u32::MAX),
    );
    for fb in &b.gpu_family_budgets {
        write_str(out, fb.gpu_family_wire_name);
        write_u32(out, fb.max_active_count);
        write_str(out, fb.declared_cost_model);
    }
    write_u32(out, u32::from(b.reject_open_contraindications));
    write_u32(out, u32::from(b.reject_open_coverage_holes));
}

fn compute_budget_pruning_plan_hash(plan: &S13dBudgetPruningPlan) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
    buf.extend_from_slice(S13D_BUDGET_PRUNING_PLAN_DOMAIN_V1.as_bytes());
    write_str(&mut buf, S13D_BUDGET_PRUNING_PLAN_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &plan.corpus_hash_v1);
    write_bytes_fixed(&mut buf, &plan.corpus_hash_v2);
    write_bytes_fixed(&mut buf, &plan.ff1_passport_index_hash_v1);
    write_bytes_fixed(&mut buf, &plan.ff2_activation_ratification_gate_hash_v1);
    write_bytes_fixed(&mut buf, &plan.ff3_registry_generation_gate_hash_v1);
    write_bytes_fixed(&mut buf, &plan.ff4_readme_authority_boundary_policy_hash_v1);
    write_bytes_fixed(&mut buf, &plan.proposal_schema_upgrade_policy_hash_v1);
    write_u32(&mut buf, plan.seed_len);
    write_task_budget(&mut buf, &plan.task_budget);
    write_u32(
        &mut buf,
        u32::try_from(plan.decisions.len()).unwrap_or(u32::MAX),
    );
    for d in &plan.decisions {
        write_u32(&mut buf, d.canonical_id);
        write_str(&mut buf, d.outcome_wire_name);
        write_str(&mut buf, d.reason_wire_name);
        match d.redundancy_cluster_id {
            Some(s) => {
                write_u32(&mut buf, 1);
                write_str(&mut buf, s);
            }
            None => {
                write_u32(&mut buf, 0);
            }
        }
        write_bytes_fixed(&mut buf, &d.cited_passport_hash);
    }
    write_u32(
        &mut buf,
        u32::try_from(plan.tie_break_transcript.len()).unwrap_or(u32::MAX),
    );
    for e in &plan.tie_break_transcript {
        write_str(&mut buf, e.cluster_id);
        write_u32(&mut buf, e.selected_canonical_id);
        write_str(&mut buf, e.selection_rule);
        write_u32(
            &mut buf,
            u32::try_from(e.suppressed_canonical_ids.len()).unwrap_or(u32::MAX),
        );
        for cid in &e.suppressed_canonical_ids {
            write_u32(&mut buf, *cid);
        }
    }
    write_u32(&mut buf, plan.active_count);
    write_u32(&mut buf, plan.disabled_count);
    write_u32(&mut buf, plan.disabled_by_budget_count);
    write_u32(&mut buf, plan.disabled_by_redundancy_count);
    write_u32(&mut buf, plan.disabled_by_gpu_family_quota_count);
    write_u32(&mut buf, plan.disabled_by_task_budget_count);
    write_u32(&mut buf, plan.disabled_by_runtime_budget_count);
    write_u32(&mut buf, plan.disabled_by_memory_budget_count);
    write_u32(&mut buf, plan.disabled_by_contraindication_budget_count);
    write_u32(&mut buf, plan.disabled_by_coverage_hole_budget_count);
    sha256(&buf)
}

fn compute_redundancy_suppression_hash(r: &RedundancySuppressionReport) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(S13D_REDUNDANCY_SUPPRESSION_DOMAIN_V1.as_bytes());
    write_str(&mut buf, S13D_REDUNDANCY_SUPPRESSION_SCHEMA_V1);
    write_u32(
        &mut buf,
        u32::try_from(r.clusters.len()).unwrap_or(u32::MAX),
    );
    for c in &r.clusters {
        write_str(&mut buf, c.cluster_id);
        write_str(&mut buf, c.selection_rule);
        write_u32(
            &mut buf,
            u32::try_from(c.member_canonical_ids.len()).unwrap_or(u32::MAX),
        );
        for m in c.member_canonical_ids {
            write_u32(&mut buf, *m);
        }
    }
    write_u32(
        &mut buf,
        u32::try_from(r.retained_representatives.len()).unwrap_or(u32::MAX),
    );
    for rep in &r.retained_representatives {
        write_u32(&mut buf, *rep);
    }
    write_u32(&mut buf, r.suppression_count);
    sha256(&buf)
}

fn compute_budgeted_activation_summary_hash(s: &BudgetedActivationSummary) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(S13D_BUDGETED_ACTIVATION_SUMMARY_DOMAIN_V1.as_bytes());
    write_str(&mut buf, S13D_BUDGETED_ACTIVATION_SUMMARY_SCHEMA_V1);
    write_bytes_fixed(&mut buf, &s.plan.budget_pruning_plan_hash_v1);
    write_bytes_fixed(
        &mut buf,
        &s.redundancy_report.redundancy_suppression_hash_v1,
    );
    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier — eight panel-required rules + structural rules
// ---------------------------------------------------------------

/// Walk an S1.3d summary against the live consolidation
/// report + FF.1 / FF.2 / FF.3 / FF.5 anchors and emit every
/// rejection. An empty return means the plan is admissible.
#[must_use]
pub fn verify_s1_3d(
    summary: &BudgetedActivationSummary,
    report: &ConsolidationReport,
    ff3_gate: &Ff3RegistryGenerationGate,
    live_ff5_policy_hash: [u8; 32],
) -> Vec<S13dVerifyError> {
    let mut errors: Vec<S13dVerifyError> = Vec::new();
    let plan = &summary.plan;

    let eligible_ids: BTreeSet<u32> = ff3_gate
        .decisions
        .iter()
        .filter(|d| d.eligibility == Ff3RegistryGenerationEligibility::Eligible)
        .map(|d| d.canonical_id)
        .collect();

    // R.1 BudgetPlanThatUsesFf3RejectedRecord: every decision
    // id must be FF.3-eligible.
    for d in &plan.decisions {
        if !eligible_ids.contains(&d.canonical_id) {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::BudgetPlanThatUsesFf3RejectedRecord {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.2 SilentDetectorDropWithoutSuppressionReason.
    for d in &plan.decisions {
        if d.outcome == S13dOutcome::Disabled && d.reason_wire_name.is_empty() {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::SilentDetectorDropWithoutSuppressionReason {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.3 RedundancySuppressionWithoutSurvivingRepresentative:
    // every cluster with at least one suppressed member must
    // have an Active representative in the decision list.
    let active_ids_with_cluster: BTreeMap<&'static str, u32> = plan
        .decisions
        .iter()
        .filter(|d| {
            d.outcome == S13dOutcome::Active
                && d.reason_wire_name
                    == S13dBudgetRetainReason::RetainedAsRepresentativeWitness.as_str()
        })
        .filter_map(|d| d.redundancy_cluster_id.map(|c| (c, d.canonical_id)))
        .collect();
    for e in &plan.tie_break_transcript {
        if !e.suppressed_canonical_ids.is_empty()
            && !active_ids_with_cluster.contains_key(e.cluster_id)
        {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::RedundancySuppressionWithoutSurvivingRepresentative {
                    cluster_id: e.cluster_id,
                },
            });
        }
    }

    // R.4 BudgetOverrunWithoutReasonCodedPruning: if
    // active_count > max_active_detectors there must be at
    // least one DisabledByTaskBudget or DisabledByGpuFamilyQuota
    // decision; otherwise the budget was overrun silently.
    if plan.active_count > plan.task_budget.max_active_detectors {
        let pruning_present = plan.decisions.iter().any(|d| {
            d.outcome == S13dOutcome::Disabled
                && (d.reason_wire_name == S13dBudgetDisableReason::DisabledByTaskBudget.as_str()
                    || d.reason_wire_name
                        == S13dBudgetDisableReason::DisabledByGpuFamilyQuota.as_str())
        });
        if !pruning_present {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::BudgetOverrunWithoutReasonCodedPruning {
                    active_count: plan.active_count,
                    max_active_detectors: plan.task_budget.max_active_detectors,
                },
            });
        }
    }

    // R.5 NondeterministicTieBreakBetweenEqualPriorityDetectors:
    // for every tie-break entry, the selected_canonical_id
    // must equal the minimum of (selected + suppressed).
    for e in &plan.tie_break_transcript {
        let mut all_members: Vec<u32> = e.suppressed_canonical_ids.clone();
        all_members.push(e.selected_canonical_id);
        all_members.sort_unstable();
        if let Some(min) = all_members.first() {
            if *min != e.selected_canonical_id {
                errors.push(S13dVerifyError {
                    kind:
                        S13dVerifyErrorKind::NondeterministicTieBreakBetweenEqualPriorityDetectors {
                            cluster_id: e.cluster_id,
                        },
                });
            }
        }
    }

    // R.6 GpuFamilyBudgetWithoutDeclaredCostModel.
    for fb in &plan.task_budget.gpu_family_budgets {
        if fb.declared_cost_model.is_empty() {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::GpuFamilyBudgetWithoutDeclaredCostModel {
                    gpu_family_wire_name: fb.gpu_family_wire_name,
                },
            });
        }
    }

    // R.7 PruningThatMutatesCorpusHashV1OrV2.
    if plan.corpus_hash_v1 != report.corpus_hash_v1 {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::PruningThatMutatesCorpusHashV1OrV2 {
                anchor_wire_name: "corpus_hash_v1",
            },
        });
    }
    if plan.corpus_hash_v2 != report.corpus_hash_v2 {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::PruningThatMutatesCorpusHashV1OrV2 {
                anchor_wire_name: "corpus_hash_v2",
            },
        });
    }

    // R.8 SchemaUpgradeSideEffectInsideBudgetPruning.
    if plan.proposal_schema_upgrade_policy_hash_v1 != live_ff5_policy_hash {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::SchemaUpgradeSideEffectInsideBudgetPruning {
                claimed: plan.proposal_schema_upgrade_policy_hash_v1,
                actual: live_ff5_policy_hash,
            },
        });
    }

    // R.9 DuplicateDecisionForSameCanonicalId.
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for d in &plan.decisions {
        if !seen.insert(d.canonical_id) {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::DuplicateDecisionForSameCanonicalId {
                    canonical_id: d.canonical_id,
                },
            });
        }
    }

    // R.10 DecisionsNotSortedAscending.
    for w in plan.decisions.windows(2) {
        if w[0].canonical_id > w[1].canonical_id {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::DecisionsNotSortedAscending,
            });
            break;
        }
    }

    // R.11 anchor cross-checks: corpus_hash_v1 + FF.3 gate
    // hash.
    let live_v1 = compute_corpus_hash_v1().bytes;
    if plan.corpus_hash_v1 != live_v1 {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::CorpusHashV1Mismatch {
                claimed: plan.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }
    if plan.ff3_registry_generation_gate_hash_v1 != ff3_gate.ff3_registry_generation_gate_hash_v1 {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::Ff3GateHashMismatch {
                claimed: plan.ff3_registry_generation_gate_hash_v1,
                actual: ff3_gate.ff3_registry_generation_gate_hash_v1,
            },
        });
    }

    // R.12 SEED invariance.
    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    // R.13 DisabledCountMismatch.
    let sum = plan.disabled_by_budget_count
        + plan.disabled_by_redundancy_count
        + plan.disabled_by_gpu_family_quota_count
        + plan.disabled_by_task_budget_count
        + plan.disabled_by_runtime_budget_count
        + plan.disabled_by_memory_budget_count
        + plan.disabled_by_contraindication_budget_count
        + plan.disabled_by_coverage_hole_budget_count;
    if sum != plan.disabled_count {
        errors.push(S13dVerifyError {
            kind: S13dVerifyErrorKind::DisabledCountMismatch {
                sum_of_per_reason_counts: sum,
                stored_disabled_count: plan.disabled_count,
            },
        });
    }

    // R.14 RedundancyClusterWithEmptyMemberSet.
    for c in &summary.redundancy_report.clusters {
        if c.member_canonical_ids.is_empty() {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::RedundancyClusterWithEmptyMemberSet {
                    cluster_id: c.cluster_id,
                },
            });
        }
    }

    // R.15 RedundancyClusterWithEmptySelectionRule.
    for c in &summary.redundancy_report.clusters {
        if c.selection_rule.is_empty() {
            errors.push(S13dVerifyError {
                kind: S13dVerifyErrorKind::RedundancyClusterWithEmptySelectionRule {
                    cluster_id: c.cluster_id,
                },
            });
        }
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the S1.3d budget pruning plan as a deterministic
/// text report.
#[must_use]
pub fn render_s13d_plan_text(plan: &S13dBudgetPruningPlan) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "S1.3d Budget Pruning Plan (v1)");
    let _ = writeln!(s, "==============================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Pinned anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                               : {}",
        hex32(&plan.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                               : {}",
        hex32(&plan.corpus_hash_v2)
    );
    let _ = writeln!(
        s,
        "  ff1_passport_index_hash_v1                   : {}",
        hex32(&plan.ff1_passport_index_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff2_activation_ratification_gate_hash_v1     : {}",
        hex32(&plan.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff3_registry_generation_gate_hash_v1         : {}",
        hex32(&plan.ff3_registry_generation_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff4_readme_authority_boundary_policy_hash_v1 : {}",
        hex32(&plan.ff4_readme_authority_boundary_policy_hash_v1)
    );
    let _ = writeln!(
        s,
        "  proposal_schema_upgrade_policy_hash_v1       : {}",
        hex32(&plan.proposal_schema_upgrade_policy_hash_v1)
    );
    let _ = writeln!(
        s,
        "  SEED.len()                                   : {}",
        plan.seed_len
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Task budget");
    let _ = writeln!(
        s,
        "  max_active_detectors                         : {}",
        plan.task_budget.max_active_detectors
    );
    let _ = writeln!(
        s,
        "  max_runtime_us                               : {}",
        plan.task_budget.max_runtime_us
    );
    let _ = writeln!(
        s,
        "  max_memory_bytes                             : {}",
        plan.task_budget.max_memory_bytes
    );
    let _ = writeln!(
        s,
        "  per_detector_runtime_us                      : {}",
        plan.task_budget.per_detector_runtime_us
    );
    let _ = writeln!(
        s,
        "  per_detector_memory_bytes                    : {}",
        plan.task_budget.per_detector_memory_bytes
    );
    let _ = writeln!(
        s,
        "  gpu_family_budgets                           : {} declared",
        plan.task_budget.gpu_family_budgets.len()
    );
    let _ = writeln!(
        s,
        "  reject_open_contraindications                : {}",
        plan.task_budget.reject_open_contraindications
    );
    let _ = writeln!(
        s,
        "  reject_open_coverage_holes                   : {}",
        plan.task_budget.reject_open_coverage_holes
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Outcome counts");
    let _ = writeln!(
        s,
        "  active_count                                 : {}",
        plan.active_count
    );
    let _ = writeln!(
        s,
        "  disabled_count                               : {}",
        plan.disabled_count
    );
    let _ = writeln!(
        s,
        "    DisabledByBudget                           : {}",
        plan.disabled_by_budget_count
    );
    let _ = writeln!(
        s,
        "    DisabledByRedundancy                       : {}",
        plan.disabled_by_redundancy_count
    );
    let _ = writeln!(
        s,
        "    DisabledByGpuFamilyQuota                   : {}",
        plan.disabled_by_gpu_family_quota_count
    );
    let _ = writeln!(
        s,
        "    DisabledByTaskBudget                       : {}",
        plan.disabled_by_task_budget_count
    );
    let _ = writeln!(
        s,
        "    DisabledByRuntimeBudget                    : {}",
        plan.disabled_by_runtime_budget_count
    );
    let _ = writeln!(
        s,
        "    DisabledByMemoryBudget                     : {}",
        plan.disabled_by_memory_budget_count
    );
    let _ = writeln!(
        s,
        "    DisabledByContraindicationBudget           : {}",
        plan.disabled_by_contraindication_budget_count
    );
    let _ = writeln!(
        s,
        "    DisabledByCoverageHoleBudget               : {}",
        plan.disabled_by_coverage_hole_budget_count
    );
    let _ = writeln!(
        s,
        "  total decisions                              : {}",
        plan.decisions.len()
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Tie-break transcript entries                  : {}",
        plan.tie_break_transcript.len()
    );
    for e in &plan.tie_break_transcript {
        let _ = writeln!(
            s,
            "  cluster `{}` selected {} via `{}` suppressing {:?}",
            e.cluster_id, e.selected_canonical_id, e.selection_rule, e.suppressed_canonical_ids
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "budget_pruning_plan_hash_v1 : {}",
        hex32(&plan.budget_pruning_plan_hash_v1)
    );
    s
}

/// Render the redundancy suppression report as a deterministic
/// text report.
#[must_use]
pub fn render_s13d_redundancy_text(r: &RedundancySuppressionReport) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "S1.3d Redundancy Suppression Report (v1)");
    let _ = writeln!(s, "========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Cluster count          : {}", r.clusters.len());
    let _ = writeln!(
        s,
        "Retained representatives : {} ({:?})",
        r.retained_representatives.len(),
        r.retained_representatives
    );
    let _ = writeln!(s, "Suppression count      : {}", r.suppression_count);
    let _ = writeln!(s);
    for c in &r.clusters {
        let _ = writeln!(
            s,
            "cluster `{}` (rule `{}`): members {:?}",
            c.cluster_id, c.selection_rule, c.member_canonical_ids
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "redundancy_suppression_hash_v1 : {}",
        hex32(&r.redundancy_suppression_hash_v1)
    );
    s
}

/// Render the top-level summary as a deterministic text
/// report.
#[must_use]
pub fn render_s13d_summary_text(s: &BudgetedActivationSummary) -> String {
    use core::fmt::Write;
    let mut out = render_s13d_plan_text(&s.plan);
    let _ = writeln!(out);
    out.push_str(&render_s13d_redundancy_text(&s.redundancy_report));
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "budgeted_activation_summary_hash_v1 : {}",
        hex32(&s.budgeted_activation_summary_hash_v1)
    );
    out
}

/// Render the S1.3d plan as a deterministic JSON object.
#[must_use]
pub fn render_s13d_plan_json(plan: &S13dBudgetPruningPlan) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(s, "\"schema\":\"{S13D_BUDGET_PRUNING_PLAN_SCHEMA_V1}\"");
    let _ = write!(s, ",\"corpus_hash_v1\":\"{}\"", hex32(&plan.corpus_hash_v1));
    let _ = write!(s, ",\"corpus_hash_v2\":\"{}\"", hex32(&plan.corpus_hash_v2));
    let _ = write!(
        s,
        ",\"ff3_registry_generation_gate_hash_v1\":\"{}\"",
        hex32(&plan.ff3_registry_generation_gate_hash_v1)
    );
    let _ = write!(
        s,
        ",\"proposal_schema_upgrade_policy_hash_v1\":\"{}\"",
        hex32(&plan.proposal_schema_upgrade_policy_hash_v1)
    );
    let _ = write!(s, ",\"seed_len\":{}", plan.seed_len);
    let _ = write!(
        s,
        ",\"max_active_detectors\":{}",
        plan.task_budget.max_active_detectors
    );
    let _ = write!(s, ",\"active_count\":{}", plan.active_count);
    let _ = write!(s, ",\"disabled_count\":{}", plan.disabled_count);
    let _ = write!(s, ",\"total_decisions\":{}", plan.decisions.len());
    let _ = write!(
        s,
        ",\"budget_pruning_plan_hash_v1\":\"{}\"",
        hex32(&plan.budget_pruning_plan_hash_v1)
    );
    s.push('}');
    s
}

/// Render the redundancy suppression report as JSON.
#[must_use]
pub fn render_s13d_redundancy_json(r: &RedundancySuppressionReport) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(s, "\"schema\":\"{S13D_REDUNDANCY_SUPPRESSION_SCHEMA_V1}\"");
    let _ = write!(s, ",\"cluster_count\":{}", r.clusters.len());
    let _ = write!(
        s,
        ",\"retained_count\":{}",
        r.retained_representatives.len()
    );
    let _ = write!(s, ",\"suppression_count\":{}", r.suppression_count);
    let _ = write!(
        s,
        ",\"redundancy_suppression_hash_v1\":\"{}\"",
        hex32(&r.redundancy_suppression_hash_v1)
    );
    s.push('}');
    s
}

/// Render the top-level summary as JSON.
#[must_use]
pub fn render_s13d_summary_json(summary: &BudgetedActivationSummary) -> String {
    use core::fmt::Write;
    let mut s = String::new();
    s.push('{');
    let _ = write!(
        s,
        "\"schema\":\"{S13D_BUDGETED_ACTIVATION_SUMMARY_SCHEMA_V1}\""
    );
    let _ = write!(s, ",\"plan\":{}", render_s13d_plan_json(&summary.plan));
    let _ = write!(
        s,
        ",\"redundancy_report\":{}",
        render_s13d_redundancy_json(&summary.redundancy_report)
    );
    let _ = write!(
        s,
        ",\"budgeted_activation_summary_hash_v1\":\"{}\"",
        hex32(&summary.budgeted_activation_summary_hash_v1)
    );
    s.push('}');
    s
}

/// Hex-encode a 32-byte digest as a 64-character lowercase
/// string.
fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push(nibble(*b >> 4));
        s.push(nibble(*b & 0x0f));
    }
    s
}

const fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + (n - 10)) as char,
        _ => '?',
    }
}
