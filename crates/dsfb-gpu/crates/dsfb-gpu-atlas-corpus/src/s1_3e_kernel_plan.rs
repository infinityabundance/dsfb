//! S1.3e --- `KernelPlanV1`: deterministic GPU-family
//! execution plan above S1.3d.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S1.3e converts the budgeted activation surface into a
//! > deterministic GPU-family execution plan. It does not
//! > execute kernels, generate CUDA code, mutate corpus
//! > authority, or change activation / budget decisions. It
//! > maps retained witnesses into family-compacted kernel
//! > lanes, parameter-table ranges, memory estimates, and
//! > execution-plan receipts. Core rule: budget admission is
//! > not execution; `KernelPlanV1` is a deterministic plan,
//! > not a GPU run.**
//!
//! ## Why
//!
//! After S1.3a (activation), FF.2 (ratification gate), FF.3
//! (registry-generation gate), and S1.3d (budget pruning +
//! redundancy suppression), the court has classified every
//! ratified candidate into Active or Disabled with a reason
//! code. What is missing is the bridge from "this set of 152
//! Active detectors should run" to "this is the deterministic
//! GPU-family schedule the executor will dispatch." Without
//! S1.3e the operator cannot replay or audit the
//! family-compaction decision; without a panel-locked
//! reason-code separation, "kernel launch" would silently
//! collapse into "activation" (false equivalence the panel
//! has explicitly forbidden across the S1.3a / FF.2 / FF.3 /
//! S1.3d axes).
//!
//! S1.3e ships:
//!
//! 1. A panel-locked deterministic family-compaction policy
//!    that groups every S1.3d-Active detector by its
//!    [`crate::types::GpuFamilyKernel`] (SEED ids carry their
//!    own family tag in
//!    [`crate::types::LiteratureDetector::gpu_family`];
//!    T12-ratified ids carry the family wire name in the
//!    [`crate::ff1_passport_materialisation::T12RatifiedPassport::gpu_family_wire_name`]
//!    field).
//! 2. A [`KernelFamilyScheduleV1`] artifact: per-family lane
//!    entries sorted by family wire name ascending, each
//!    carrying the lane's active detector ids, declared cost
//!    model, expected kernel name, and aggregate cost
//!    estimate.
//! 3. A [`KernelParameterTableV1`] artifact: a stable-ordered
//!    parameter-table over the active surface (sorted by
//!    `(family_wire_name, canonical_id)` ascending), each row
//!    carrying the canonical id, the family wire name, a
//!    parameter-table offset, and the per-detector cost
//!    quote.
//! 4. A top-level [`KernelPlanV1`] artifact: META-hash binding
//!    the schedule + parameter table + nine pinned upstream
//!    anchor hashes (the seven S1.3d pinned anchors plus the
//!    two S1.3d artifact hashes themselves).
//!
//! ## Panel-locked non-claims
//!
//! S1.3e does NOT:
//!
//! - execute kernels;
//! - emit CUDA source, PTX, SASS, or any cubin bytes;
//! - mutate any upstream hash anchor (corpus / passport /
//!   FF.x / S1.3d hashes are all read-only);
//! - alter [`crate::seed::SEED`] (`SEED.len()` stays at 54);
//! - change S1.3a SEED activation decisions or FF.2
//!   ratification decisions or FF.3 registry eligibility or
//!   S1.3d budget admission;
//! - itself emit a `CaseFileV2Header` --- that integration is
//!   S1.3f's job;
//! - decide contraindications or challenges;
//! - modify the registry crate.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes:
//!
//! - `kernel_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0`. META-hashes the
//!   nine pinned upstream anchors plus the schedule hash plus
//!   the parameter-table hash plus per-family lane counts.
//! - `kernel_family_schedule_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0`.
//!   META-hashes the sorted list of [`FamilyLane`] entries.
//! - `kernel_parameter_table_hash_v1` under
//!   `DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0`.
//!   META-hashes the sorted list of [`ParameterTableRow`]
//!   entries.
//!
//! Every prior anchor stays byte-identical. The plan is
//! built deterministically and two builds produce byte-equal
//! hashes (the determinism gate the acceptance suite pins).
//!
//! ## Panel-locked verdict (one line)
//!
//! > S1.3d says who survives budgeted deployment; S1.3e says
//! > how the survivors are packed into deterministic GPU-
//! > family execution lanes.

use core::fmt::Write;
use std::collections::{BTreeMap, BTreeSet};

use dsfb_gpu_debug_core::sha256;

use crate::corpus_hash::compute_corpus_hash_v1;
use crate::ff1_passport_materialisation::{build_ff1_passport_index_from, Ff1PassportIndex};
use crate::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, BudgetedActivationSummary, S13dOutcome,
};
use crate::seed::SEED;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// SHA-256 domain separator for `kernel_plan_hash_v1`.
pub const S13E_KERNEL_PLAN_DOMAIN_V1: &str = "DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0";

/// Schema identifier for `kernel_plan_hash_v1`.
pub const S13E_KERNEL_PLAN_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1";

/// SHA-256 domain separator for
/// `kernel_family_schedule_hash_v1`.
pub const S13E_KERNEL_FAMILY_SCHEDULE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0";

/// Schema identifier for `kernel_family_schedule_hash_v1`.
pub const S13E_KERNEL_FAMILY_SCHEDULE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1";

/// SHA-256 domain separator for
/// `kernel_parameter_table_hash_v1`.
pub const S13E_KERNEL_PARAMETER_TABLE_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0";

/// Schema identifier for `kernel_parameter_table_hash_v1`.
pub const S13E_KERNEL_PARAMETER_TABLE_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1";

// ---------------------------------------------------------------
// Per-family lane record
// ---------------------------------------------------------------

/// One per-family kernel execution lane. Carries the active
/// detector ids assigned to this GPU family, the declared
/// cost model (R.5 verifier rejects empty strings), the
/// expected kernel name, and the aggregate cost estimate.
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining `kernel_family_schedule_hash_v1`.
#[derive(Debug, Clone)]
pub struct FamilyLane {
    /// Stable wire name of the GPU family (matches
    /// [`crate::types::GpuFamilyKernel::as_str`] for SEED
    /// candidates; matches the FF.1 passport's
    /// `gpu_family_wire_name` for T12-ratified candidates).
    pub gpu_family_wire_name: &'static str,
    /// Active detector canonical ids assigned to this lane,
    /// sorted ascending. Determinism gate: the verifier
    /// rejects unsorted lanes via
    /// `LaneDetectorIdsNotSortedAscending`.
    pub active_canonical_ids: Vec<u32>,
    /// Number of active detectors in this lane (mirrors
    /// `active_canonical_ids.len()` as a redundant cross-
    /// check the verifier enforces).
    pub active_detector_count: u32,
    /// Operator-readable cost model declaration (e.g.
    /// `"O(window) per detector evaluation"`). Non-empty
    /// (R.5 verifier rule rejects empty cost models, mirror
    /// of S1.3d's `GpuFamilyBudgetWithoutDeclaredCostModel`).
    pub declared_cost_model: &'static str,
    /// Canonical expected kernel name (e.g.
    /// `"dsfb_gpu_window_statistic_family_kernel"`). Non-
    /// empty.
    pub expected_kernel_name: &'static str,
    /// Aggregate cost estimate for this lane in microseconds.
    /// Computed deterministically as `active_detector_count *
    /// per_detector_runtime_us` from the inherited S1.3d
    /// `TaskBudget` envelope.
    pub aggregate_cost_us: u64,
}

// ---------------------------------------------------------------
// Per-detector parameter-table row record
// ---------------------------------------------------------------

/// One row in the canonical parameter table. Carries the
/// canonical id, the GPU family wire name, the row's
/// position within the lane (offset), and the per-detector
/// cost quote.
///
/// Rows are sorted by `(gpu_family_wire_name, canonical_id)`
/// ascending; the verifier rejects unsorted tables via
/// `ParameterTableNotSorted`.
#[derive(Debug, Clone)]
pub struct ParameterTableRow {
    /// Detector canonical id.
    pub canonical_id: u32,
    /// GPU family wire name (matches the family lane the row
    /// belongs to).
    pub gpu_family_wire_name: &'static str,
    /// Stable offset within the family lane (0-based,
    /// ascending by `canonical_id` within the lane).
    pub lane_offset: u32,
    /// Per-detector cost quote in microseconds (inherited
    /// from the S1.3d task budget's
    /// `per_detector_runtime_us` value).
    pub per_detector_runtime_us: u64,
}

// ---------------------------------------------------------------
// Top-level family schedule
// ---------------------------------------------------------------

/// The S1.3e family schedule. The list of [`FamilyLane`]
/// entries sorted by `gpu_family_wire_name` ascending.
#[derive(Debug, Clone)]
pub struct KernelFamilyScheduleV1 {
    /// Sorted ascending list of per-family lanes.
    pub lanes: Vec<FamilyLane>,
    /// Total active detector count across all lanes (mirrors
    /// `lanes.iter().map(|l| l.active_detector_count).sum()`
    /// as a redundant cross-check).
    pub total_active_count: u32,
    /// `kernel_family_schedule_hash_v1`.
    pub kernel_family_schedule_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level parameter table
// ---------------------------------------------------------------

/// The S1.3e parameter table. Sorted by
/// `(gpu_family_wire_name, canonical_id)` ascending.
#[derive(Debug, Clone)]
pub struct KernelParameterTableV1 {
    /// Sorted rows.
    pub rows: Vec<ParameterTableRow>,
    /// `kernel_parameter_table_hash_v1`.
    pub kernel_parameter_table_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Top-level plan
// ---------------------------------------------------------------

/// The top-level S1.3e kernel plan. Binds the family
/// schedule + parameter table + nine pinned upstream anchors
/// (the seven S1.3d pinned anchors plus the two S1.3d
/// artifact hashes).
#[derive(Debug, Clone)]
pub struct KernelPlanV1 {
    /// Historical seed-corpus anchor (unchanged across the
    /// entire post-T.12.consolidate arc).
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
    /// S1.3d budget pruning plan hash.
    pub budget_pruning_plan_hash_v1: [u8; 32],
    /// S1.3d budgeted activation summary hash.
    pub budgeted_activation_summary_hash_v1: [u8; 32],
    /// SEED record count (pinned at 54).
    pub seed_len: u32,
    /// Per-family lane count.
    pub lane_count: u32,
    /// Total active detector count.
    pub total_active_count: u32,
    /// Total aggregate runtime cost estimate across all lanes
    /// (microseconds).
    pub total_aggregate_cost_us: u64,
    /// `kernel_family_schedule_hash_v1` (mirrored from the
    /// schedule for hash convenience).
    pub kernel_family_schedule_hash_v1: [u8; 32],
    /// `kernel_parameter_table_hash_v1` (mirrored from the
    /// parameter table for hash convenience).
    pub kernel_parameter_table_hash_v1: [u8; 32],
    /// `kernel_plan_hash_v1`.
    pub kernel_plan_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S1.3e rejected a kernel plan. Eight panel-required
/// load-bearing negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S13eVerifyErrorKind {
    /// Panel-required negative #1. A lane references a
    /// canonical id S1.3d disabled (or otherwise not Active).
    /// Mirrors S1.3d's `BudgetPlanThatUsesFf3RejectedRecord`
    /// pattern one layer down.
    KernelPlanUsingBudgetDisabledDetector {
        /// The canonical id incorrectly placed in a lane.
        canonical_id: u32,
    },
    /// Panel-required negative #2. A lane references a
    /// canonical id FF.3 did not classify as Eligible (i.e.
    /// it never reached S1.3d at all).
    KernelPlanUsingFf3RejectedRecord {
        /// The canonical id.
        canonical_id: u32,
    },
    /// Panel-required negative #3. An active candidate has
    /// no GPU family mapping resolvable from either the SEED
    /// surface (id ≤ 54) or the FF.1 passport index
    /// (id > 54).
    KernelPlanWithoutGpuFamilyMapping {
        /// The unresolvable canonical id.
        canonical_id: u32,
    },
    /// Panel-required negative #4. The parameter table is
    /// not sorted by `(gpu_family_wire_name, canonical_id)`
    /// ascending.
    ParameterTableWithoutStableOrder,
    /// Panel-required negative #5. A family lane has an
    /// empty `declared_cost_model`.
    FamilyScheduleWithoutDeclaredCostModel {
        /// The family wire name with the missing cost model.
        gpu_family_wire_name: &'static str,
    },
    /// Panel-required negative #6. The plan's pinned
    /// activation / budget anchor hash does not equal the
    /// live anchor (a kernel-plan-emission side effect would
    /// surface as anchor-hash drift).
    KernelPlanThatMutatesActivationOrBudgetHash {
        /// Which anchor mismatched (wire name; e.g.
        /// `"budget_pruning_plan_hash_v1"`).
        anchor_wire_name: &'static str,
    },
    /// Panel-required negative #7. The plan body claims
    /// kernel execution (CUDA invocation, PTX emission,
    /// kernel-launch wording) inside an S1.3e expected-kernel
    /// or cost-model string. Scanner-style: matches a small
    /// set of forbidden execution-claim substrings.
    CudaExecutionClaimInsideKernelPlan {
        /// The lane wire name carrying the forbidden text.
        gpu_family_wire_name: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #8. The family schedule is
    /// not sorted ascending by `gpu_family_wire_name` (a
    /// nondeterministic tie-break between equal-priority
    /// families would surface here).
    NondeterministicTieBreakInFamilyOrder,
    /// Two lanes share the same `gpu_family_wire_name`.
    DuplicateFamilyLane {
        /// The duplicated family wire name.
        gpu_family_wire_name: &'static str,
    },
    /// A lane's `active_detector_count` does not equal
    /// `active_canonical_ids.len()`.
    LaneActiveCountMismatch {
        /// The lane wire name.
        gpu_family_wire_name: &'static str,
        /// Stored `active_detector_count`.
        stored: u32,
        /// Actual `active_canonical_ids.len()`.
        actual: u32,
    },
    /// A lane's `active_canonical_ids` list is not sorted
    /// ascending.
    LaneDetectorIdsNotSortedAscending {
        /// The lane wire name.
        gpu_family_wire_name: &'static str,
    },
    /// The plan's `total_active_count` does not match the
    /// schedule's sum of lane counts.
    TotalActiveCountMismatch {
        /// The plan's claimed total.
        claimed: u32,
        /// The recomputed total.
        actual: u32,
    },
    /// `corpus_hash_v1` pinned on the plan does not equal
    /// the live `compute_corpus_hash_v1()` result.
    CorpusHashV1Mismatch {
        /// Hash the plan claims.
        claimed: [u8; 32],
        /// Hash the live `compute_corpus_hash_v1()` returns.
        actual: [u8; 32],
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
    /// A parameter-table row references a canonical id not
    /// present in any family lane.
    ParameterTableRowReferencesUnknownCanonicalId {
        /// The orphan canonical id.
        canonical_id: u32,
    },
    /// A parameter-table row's `gpu_family_wire_name` does
    /// not match the lane that owns the canonical id.
    ParameterTableRowFamilyMismatch {
        /// The row's canonical id.
        canonical_id: u32,
        /// The row's claimed family.
        row_family_wire_name: &'static str,
        /// The lane the canonical id actually belongs to.
        actual_family_wire_name: &'static str,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S13eVerifyError {
    /// Error kind (see [`S13eVerifyErrorKind`]).
    pub kind: S13eVerifyErrorKind,
}

// ---------------------------------------------------------------
// Forbidden-execution-claim substring set (panel-locked)
// ---------------------------------------------------------------

/// The panel-locked forbidden-substring set the R.7 scanner
/// uses to catch any kernel-execution claim accidentally
/// embedded in an S1.3e cost-model or expected-kernel field.
/// S1.3e is a PLAN, not a RUN; phrasings that claim execution
/// (e.g. "kernel launch", "ptx emission") must NOT appear in
/// the rendered surface.
const S13E_FORBIDDEN_EXECUTION_CLAIM_SUBSTRINGS: &[&str] = &[
    "kernel launch",
    "kernel launched",
    "cuda execution",
    "cuda invocation",
    "ptx emission",
    "ptx emitted",
    "sass emission",
    "cubin emission",
    "kernel dispatched",
    "kernel ran",
    "kernel executed",
    "device executed",
];

// ---------------------------------------------------------------
// Family-mapping helpers
// ---------------------------------------------------------------

/// Resolve a canonical id to its GPU family wire name.
/// SEED ids (1..=54) read [`crate::types::LiteratureDetector::gpu_family`]
/// directly; ratified T12 ids (> 54) read the FF.1 passport
/// index's `gpu_family_wire_name` field. Returns `None` if the
/// canonical id is not present in either surface.
#[must_use]
pub fn resolve_gpu_family_wire_name(
    canonical_id: u32,
    passport_index: &Ff1PassportIndex,
) -> Option<&'static str> {
    if let Some(record) = SEED.iter().find(|r| r.canonical_id.0 == canonical_id) {
        return Some(record.gpu_family.as_str());
    }
    passport_index
        .passports
        .iter()
        .find(|p| p.canonical_id == canonical_id)
        .map(|p| p.gpu_family_wire_name)
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production S1.3e kernel plan from live state.
/// Pulls S1.3d (which itself pulls FF.2 + FF.3 + FF.4 + FF.5)
/// read-only; resolves GPU families; emits the family
/// schedule + parameter table + top-level plan; produces
/// byte-identical bytes across two builds.
#[must_use]
pub fn build_kernel_plan_v1() -> KernelPlanV1 {
    let summary = build_budgeted_activation_summary();
    build_kernel_plan_v1_from(&summary)
}

/// Build the S1.3e kernel plan from a fully-specified
/// [`BudgetedActivationSummary`]. Used by tests to inject
/// mutated S1.3d summaries (e.g. with budget-disabled
/// detectors) and observe how S1.3e responds.
#[must_use]
pub fn build_kernel_plan_v1_from(summary: &BudgetedActivationSummary) -> KernelPlanV1 {
    let report = crate::consolidate::build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let per_detector_runtime_us = summary.plan.task_budget.per_detector_runtime_us;

    // Walk S1.3d decisions; gather Active canonical ids.
    let mut active_ids: Vec<u32> = summary
        .plan
        .decisions
        .iter()
        .filter(|d| d.outcome == S13dOutcome::Active)
        .map(|d| d.canonical_id)
        .collect();
    active_ids.sort_unstable();

    // Group active ids by GPU family. Unresolved ids surface
    // via the verifier's R.3
    // `KernelPlanWithoutGpuFamilyMapping` rule; the builder
    // omits them from the schedule so the artifact remains
    // byte-deterministic even if the surface is malformed.
    let mut family_to_ids: BTreeMap<&'static str, Vec<u32>> = BTreeMap::new();
    for &id in &active_ids {
        if let Some(family) = resolve_gpu_family_wire_name(id, &passport_index) {
            family_to_ids.entry(family).or_default().push(id);
        }
    }

    // Build family lanes sorted ascending by family wire name
    // (BTreeMap iteration order is sorted by key).
    let mut lanes: Vec<FamilyLane> = Vec::new();
    for (family_wire_name, mut ids) in family_to_ids {
        ids.sort_unstable();
        let active_detector_count = u32::try_from(ids.len()).unwrap_or(u32::MAX);
        lanes.push(FamilyLane {
            gpu_family_wire_name: family_wire_name,
            active_canonical_ids: ids,
            active_detector_count,
            declared_cost_model: declared_cost_model_for_family(family_wire_name),
            expected_kernel_name: expected_kernel_name_for_family(family_wire_name),
            aggregate_cost_us: u64::from(active_detector_count) * per_detector_runtime_us,
        });
    }

    // Compute schedule hash.
    let total_active_count: u32 = lanes.iter().map(|l| l.active_detector_count).sum();
    let kernel_family_schedule_hash_v1 = compute_schedule_hash(&lanes, total_active_count);
    let schedule = KernelFamilyScheduleV1 {
        lanes: lanes.clone(),
        total_active_count,
        kernel_family_schedule_hash_v1,
    };

    // Build parameter table rows sorted by
    // (family_wire_name, canonical_id).
    let mut rows: Vec<ParameterTableRow> = Vec::new();
    for lane in &schedule.lanes {
        for (offset, &canonical_id) in lane.active_canonical_ids.iter().enumerate() {
            rows.push(ParameterTableRow {
                canonical_id,
                gpu_family_wire_name: lane.gpu_family_wire_name,
                lane_offset: u32::try_from(offset).unwrap_or(u32::MAX),
                per_detector_runtime_us,
            });
        }
    }
    let kernel_parameter_table_hash_v1 = compute_parameter_table_hash(&rows);
    let _parameter_table = KernelParameterTableV1 {
        rows: rows.clone(),
        kernel_parameter_table_hash_v1,
    };

    // Top-level plan.
    let total_aggregate_cost_us: u64 = lanes.iter().map(|l| l.aggregate_cost_us).sum();
    let lane_count = u32::try_from(schedule.lanes.len()).unwrap_or(u32::MAX);
    let mut plan = KernelPlanV1 {
        corpus_hash_v1: summary.plan.corpus_hash_v1,
        corpus_hash_v2: summary.plan.corpus_hash_v2,
        ff1_passport_index_hash_v1: summary.plan.ff1_passport_index_hash_v1,
        ff2_activation_ratification_gate_hash_v1: summary
            .plan
            .ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: summary.plan.ff3_registry_generation_gate_hash_v1,
        ff4_readme_authority_boundary_policy_hash_v1: summary
            .plan
            .ff4_readme_authority_boundary_policy_hash_v1,
        proposal_schema_upgrade_policy_hash_v1: summary.plan.proposal_schema_upgrade_policy_hash_v1,
        budget_pruning_plan_hash_v1: summary.plan.budget_pruning_plan_hash_v1,
        budgeted_activation_summary_hash_v1: summary.budgeted_activation_summary_hash_v1,
        seed_len: u32::try_from(SEED.len()).unwrap_or(u32::MAX),
        lane_count,
        total_active_count,
        total_aggregate_cost_us,
        kernel_family_schedule_hash_v1,
        kernel_parameter_table_hash_v1,
        kernel_plan_hash_v1: [0u8; 32], // placeholder; filled below
    };
    plan.kernel_plan_hash_v1 = compute_kernel_plan_hash(&plan);
    plan
}

/// Build the standalone schedule artifact from the plan's
/// inputs (test convenience).
#[must_use]
pub fn build_kernel_family_schedule_v1() -> KernelFamilyScheduleV1 {
    let summary = build_budgeted_activation_summary();
    build_kernel_family_schedule_v1_from(&summary)
}

/// Build the schedule artifact from an injected S1.3d summary
/// (test convenience).
#[must_use]
pub fn build_kernel_family_schedule_v1_from(
    summary: &BudgetedActivationSummary,
) -> KernelFamilyScheduleV1 {
    let plan = build_kernel_plan_v1_from(summary);
    KernelFamilyScheduleV1 {
        lanes: extract_lanes_from_summary(summary),
        total_active_count: plan.total_active_count,
        kernel_family_schedule_hash_v1: plan.kernel_family_schedule_hash_v1,
    }
}

/// Build the standalone parameter-table artifact.
#[must_use]
pub fn build_kernel_parameter_table_v1() -> KernelParameterTableV1 {
    let summary = build_budgeted_activation_summary();
    build_kernel_parameter_table_v1_from(&summary)
}

/// Build the parameter table from an injected S1.3d summary.
#[must_use]
pub fn build_kernel_parameter_table_v1_from(
    summary: &BudgetedActivationSummary,
) -> KernelParameterTableV1 {
    let lanes = extract_lanes_from_summary(summary);
    let mut rows: Vec<ParameterTableRow> = Vec::new();
    for lane in &lanes {
        for (offset, &canonical_id) in lane.active_canonical_ids.iter().enumerate() {
            rows.push(ParameterTableRow {
                canonical_id,
                gpu_family_wire_name: lane.gpu_family_wire_name,
                lane_offset: u32::try_from(offset).unwrap_or(u32::MAX),
                per_detector_runtime_us: summary.plan.task_budget.per_detector_runtime_us,
            });
        }
    }
    let kernel_parameter_table_hash_v1 = compute_parameter_table_hash(&rows);
    KernelParameterTableV1 {
        rows,
        kernel_parameter_table_hash_v1,
    }
}

/// Internal helper: walk the S1.3d summary and emit the per-
/// family lane list (sorted ascending by family wire name).
fn extract_lanes_from_summary(summary: &BudgetedActivationSummary) -> Vec<FamilyLane> {
    let report = crate::consolidate::build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let per_detector_runtime_us = summary.plan.task_budget.per_detector_runtime_us;

    let mut family_to_ids: BTreeMap<&'static str, Vec<u32>> = BTreeMap::new();
    for d in &summary.plan.decisions {
        if d.outcome != S13dOutcome::Active {
            continue;
        }
        if let Some(family) = resolve_gpu_family_wire_name(d.canonical_id, &passport_index) {
            family_to_ids
                .entry(family)
                .or_default()
                .push(d.canonical_id);
        }
    }

    let mut lanes: Vec<FamilyLane> = Vec::new();
    for (family_wire_name, mut ids) in family_to_ids {
        ids.sort_unstable();
        let active_detector_count = u32::try_from(ids.len()).unwrap_or(u32::MAX);
        lanes.push(FamilyLane {
            gpu_family_wire_name: family_wire_name,
            active_canonical_ids: ids,
            active_detector_count,
            declared_cost_model: declared_cost_model_for_family(family_wire_name),
            expected_kernel_name: expected_kernel_name_for_family(family_wire_name),
            aggregate_cost_us: u64::from(active_detector_count) * per_detector_runtime_us,
        });
    }
    lanes
}

// ---------------------------------------------------------------
// Per-family cost-model + expected-kernel-name policy
// (panel-locked, lookup-only)
// ---------------------------------------------------------------

/// Panel-locked declared cost-model wire name per GPU family.
/// Lookup-only; every entry intentionally avoids any of the
/// `S13E_FORBIDDEN_EXECUTION_CLAIM_SUBSTRINGS` (the R.7
/// scanner enforces the boundary). Updating this table is a
/// rebaselining event; the verifier crash-confirms that the
/// table never carries a forbidden substring via the
/// `s13e_cost_model_table_carries_no_forbidden_substring`
/// acceptance test.
const fn declared_cost_model_for_family(family_wire_name: &'static str) -> &'static str {
    match family_wire_name.as_bytes() {
        b"ScalarThresholdFamily" => "O(1) per cell scalar comparison",
        b"WindowStatisticFamily" => "O(window) per cell sliding statistic",
        b"SequentialRecurrenceFamily" => "O(1) per cell entity-serial recurrence",
        b"DistributionDistanceFamily" => "O(buckets) per pair of compared empirical distributions",
        b"RankStatisticFamily" => "O(window log window) per cell rank statistic",
        b"SpectralFamily" => "O(N log N) per window FFT band-energy",
        b"WaveletFamily" => "O(N) per window wavelet coefficient bank",
        b"GraphLocalFamily" => "O(edges) per neighbourhood update",
        b"GraphGlobalFamily" => "O(V + E) per global graph functional",
        b"TabularConstraintFamily" => "O(rows) per declared constraint",
        b"CategoricalHistogramFamily" => "O(rows) per categorical histogram update",
        b"MissingnessFamily" => "O(cells) per missingness mask traversal",
        b"ResidualObserverFamily" => "O(1) per cell observer residual",
        b"ProjectionResidualFamily" => "O(latent dim) per projection-residual evaluation",
        b"NegativeWitnessFamily" => "O(1) per cell suppression check",
        _ => "unmapped",
    }
}

/// Panel-locked expected kernel name per GPU family.
const fn expected_kernel_name_for_family(family_wire_name: &'static str) -> &'static str {
    match family_wire_name.as_bytes() {
        b"ScalarThresholdFamily" => "dsfb_gpu_scalar_threshold_family_kernel",
        b"WindowStatisticFamily" => "dsfb_gpu_window_statistic_family_kernel",
        b"SequentialRecurrenceFamily" => "dsfb_gpu_sequential_recurrence_family_kernel",
        b"DistributionDistanceFamily" => "dsfb_gpu_distribution_distance_family_kernel",
        b"RankStatisticFamily" => "dsfb_gpu_rank_statistic_family_kernel",
        b"SpectralFamily" => "dsfb_gpu_spectral_family_kernel",
        b"WaveletFamily" => "dsfb_gpu_wavelet_family_kernel",
        b"GraphLocalFamily" => "dsfb_gpu_graph_local_family_kernel",
        b"GraphGlobalFamily" => "dsfb_gpu_graph_global_family_kernel",
        b"TabularConstraintFamily" => "dsfb_gpu_tabular_constraint_family_kernel",
        b"CategoricalHistogramFamily" => "dsfb_gpu_categorical_histogram_family_kernel",
        b"MissingnessFamily" => "dsfb_gpu_missingness_family_kernel",
        b"ResidualObserverFamily" => "dsfb_gpu_residual_observer_family_kernel",
        b"ProjectionResidualFamily" => "dsfb_gpu_projection_residual_family_kernel",
        b"NegativeWitnessFamily" => "dsfb_gpu_negative_witness_family_kernel",
        _ => "unmapped",
    }
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_schedule_hash(lanes: &[FamilyLane], total_active_count: u32) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S13E_KERNEL_FAMILY_SCHEDULE_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S13E_KERNEL_FAMILY_SCHEDULE_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&total_active_count.to_be_bytes());
    buf.extend_from_slice(&u32::try_from(lanes.len()).unwrap_or(u32::MAX).to_be_bytes());
    for lane in lanes {
        buf.push(0x1e);
        push_len_prefixed(&mut buf, lane.gpu_family_wire_name.as_bytes());
        buf.extend_from_slice(&lane.active_detector_count.to_be_bytes());
        push_len_prefixed(&mut buf, lane.declared_cost_model.as_bytes());
        push_len_prefixed(&mut buf, lane.expected_kernel_name.as_bytes());
        buf.extend_from_slice(&lane.aggregate_cost_us.to_be_bytes());
        for &id in &lane.active_canonical_ids {
            buf.extend_from_slice(&id.to_be_bytes());
        }
    }
    sha256(&buf)
}

fn compute_parameter_table_hash(rows: &[ParameterTableRow]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S13E_KERNEL_PARAMETER_TABLE_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S13E_KERNEL_PARAMETER_TABLE_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&u32::try_from(rows.len()).unwrap_or(u32::MAX).to_be_bytes());
    for row in rows {
        buf.push(0x1e);
        buf.extend_from_slice(&row.canonical_id.to_be_bytes());
        push_len_prefixed(&mut buf, row.gpu_family_wire_name.as_bytes());
        buf.extend_from_slice(&row.lane_offset.to_be_bytes());
        buf.extend_from_slice(&row.per_detector_runtime_us.to_be_bytes());
    }
    sha256(&buf)
}

fn compute_kernel_plan_hash(plan: &KernelPlanV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(S13E_KERNEL_PLAN_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(S13E_KERNEL_PLAN_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&plan.corpus_hash_v1);
    buf.extend_from_slice(&plan.corpus_hash_v2);
    buf.extend_from_slice(&plan.ff1_passport_index_hash_v1);
    buf.extend_from_slice(&plan.ff2_activation_ratification_gate_hash_v1);
    buf.extend_from_slice(&plan.ff3_registry_generation_gate_hash_v1);
    buf.extend_from_slice(&plan.ff4_readme_authority_boundary_policy_hash_v1);
    buf.extend_from_slice(&plan.proposal_schema_upgrade_policy_hash_v1);
    buf.extend_from_slice(&plan.budget_pruning_plan_hash_v1);
    buf.extend_from_slice(&plan.budgeted_activation_summary_hash_v1);
    buf.extend_from_slice(&plan.seed_len.to_be_bytes());
    buf.extend_from_slice(&plan.lane_count.to_be_bytes());
    buf.extend_from_slice(&plan.total_active_count.to_be_bytes());
    buf.extend_from_slice(&plan.total_aggregate_cost_us.to_be_bytes());
    buf.extend_from_slice(&plan.kernel_family_schedule_hash_v1);
    buf.extend_from_slice(&plan.kernel_parameter_table_hash_v1);
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// Verify an S1.3e plan + schedule + parameter table against
/// the live state. Returns a vector of errors (empty when
/// the plan satisfies every panel-required + structural
/// rule).
///
/// Accepts the schedule + parameter table as separate
/// references so tests can mutate one without rebuilding the
/// other (mirror of the S1.3d / FF.5 verifier shapes).
//
// The 16 verifier rules each emit at most a couple of lines;
// in aggregate the function exceeds the workspace default
// 100-line clippy cap because the schema is wide. Splitting
// the rules into helpers would obscure the panel-locked rule
// numbering (R.1..R.8 + structural defects), so we accept
// the length deliberately.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_s1_3e(
    plan: &KernelPlanV1,
    schedule: &KernelFamilyScheduleV1,
    parameter_table: &KernelParameterTableV1,
    summary: &BudgetedActivationSummary,
    passport_index: &Ff1PassportIndex,
) -> Vec<S13eVerifyError> {
    let mut errors: Vec<S13eVerifyError> = Vec::new();

    // Build the lookup sets the rules consume.
    let ff3_eligible_ids: BTreeSet<u32> = summary
        .plan
        .decisions
        .iter()
        .map(|d| d.canonical_id)
        .collect();
    let active_ids: BTreeSet<u32> = summary
        .plan
        .decisions
        .iter()
        .filter(|d| d.outcome == S13dOutcome::Active)
        .map(|d| d.canonical_id)
        .collect();

    // R.1 KernelPlanUsingBudgetDisabledDetector + R.2
    // KernelPlanUsingFf3RejectedRecord.
    for lane in &schedule.lanes {
        for &id in &lane.active_canonical_ids {
            if !ff3_eligible_ids.contains(&id) {
                errors.push(S13eVerifyError {
                    kind: S13eVerifyErrorKind::KernelPlanUsingFf3RejectedRecord {
                        canonical_id: id,
                    },
                });
            } else if !active_ids.contains(&id) {
                errors.push(S13eVerifyError {
                    kind: S13eVerifyErrorKind::KernelPlanUsingBudgetDisabledDetector {
                        canonical_id: id,
                    },
                });
            }
        }
    }

    // R.3 KernelPlanWithoutGpuFamilyMapping: any active id
    // that resolve_gpu_family_wire_name cannot resolve.
    for &id in &active_ids {
        if resolve_gpu_family_wire_name(id, passport_index).is_none() {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::KernelPlanWithoutGpuFamilyMapping { canonical_id: id },
            });
        }
    }

    // R.4 ParameterTableWithoutStableOrder.
    for w in parameter_table.rows.windows(2) {
        let a = (w[0].gpu_family_wire_name, w[0].canonical_id);
        let b = (w[1].gpu_family_wire_name, w[1].canonical_id);
        if a > b {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::ParameterTableWithoutStableOrder,
            });
            break;
        }
    }

    // R.5 FamilyScheduleWithoutDeclaredCostModel.
    for lane in &schedule.lanes {
        if lane.declared_cost_model.is_empty() {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::FamilyScheduleWithoutDeclaredCostModel {
                    gpu_family_wire_name: lane.gpu_family_wire_name,
                },
            });
        }
    }

    // R.6 KernelPlanThatMutatesActivationOrBudgetHash: the
    // plan must mirror the S1.3d summary's pinned anchors.
    if plan.budget_pruning_plan_hash_v1 != summary.plan.budget_pruning_plan_hash_v1 {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::KernelPlanThatMutatesActivationOrBudgetHash {
                anchor_wire_name: "budget_pruning_plan_hash_v1",
            },
        });
    }
    if plan.budgeted_activation_summary_hash_v1 != summary.budgeted_activation_summary_hash_v1 {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::KernelPlanThatMutatesActivationOrBudgetHash {
                anchor_wire_name: "budgeted_activation_summary_hash_v1",
            },
        });
    }
    if plan.ff2_activation_ratification_gate_hash_v1
        != summary.plan.ff2_activation_ratification_gate_hash_v1
    {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::KernelPlanThatMutatesActivationOrBudgetHash {
                anchor_wire_name: "ff2_activation_ratification_gate_hash_v1",
            },
        });
    }
    if plan.ff3_registry_generation_gate_hash_v1
        != summary.plan.ff3_registry_generation_gate_hash_v1
    {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::KernelPlanThatMutatesActivationOrBudgetHash {
                anchor_wire_name: "ff3_registry_generation_gate_hash_v1",
            },
        });
    }

    // R.7 CudaExecutionClaimInsideKernelPlan: scan every
    // lane's cost-model + expected-kernel-name strings.
    for lane in &schedule.lanes {
        for &forbidden in S13E_FORBIDDEN_EXECUTION_CLAIM_SUBSTRINGS {
            if contains_ascii_case_insensitive(lane.declared_cost_model, forbidden)
                || contains_ascii_case_insensitive(lane.expected_kernel_name, forbidden)
            {
                errors.push(S13eVerifyError {
                    kind: S13eVerifyErrorKind::CudaExecutionClaimInsideKernelPlan {
                        gpu_family_wire_name: lane.gpu_family_wire_name,
                        forbidden_substring: forbidden,
                    },
                });
            }
        }
    }

    // R.8 NondeterministicTieBreakInFamilyOrder.
    for w in schedule.lanes.windows(2) {
        if w[0].gpu_family_wire_name > w[1].gpu_family_wire_name {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::NondeterministicTieBreakInFamilyOrder,
            });
            break;
        }
    }

    // Structural defect rules.
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    for lane in &schedule.lanes {
        if !seen.insert(lane.gpu_family_wire_name) {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::DuplicateFamilyLane {
                    gpu_family_wire_name: lane.gpu_family_wire_name,
                },
            });
        }
        let actual = u32::try_from(lane.active_canonical_ids.len()).unwrap_or(u32::MAX);
        if actual != lane.active_detector_count {
            errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::LaneActiveCountMismatch {
                    gpu_family_wire_name: lane.gpu_family_wire_name,
                    stored: lane.active_detector_count,
                    actual,
                },
            });
        }
        for w in lane.active_canonical_ids.windows(2) {
            if w[0] > w[1] {
                errors.push(S13eVerifyError {
                    kind: S13eVerifyErrorKind::LaneDetectorIdsNotSortedAscending {
                        gpu_family_wire_name: lane.gpu_family_wire_name,
                    },
                });
                break;
            }
        }
    }

    let total: u32 = schedule.lanes.iter().map(|l| l.active_detector_count).sum();
    if total != plan.total_active_count {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::TotalActiveCountMismatch {
                claimed: plan.total_active_count,
                actual: total,
            },
        });
    }

    let live_v1 = compute_corpus_hash_v1().bytes;
    if plan.corpus_hash_v1 != live_v1 {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::CorpusHashV1Mismatch {
                claimed: plan.corpus_hash_v1,
                actual: live_v1,
            },
        });
    }

    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(S13eVerifyError {
            kind: S13eVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    // Parameter-table cross-checks.
    let mut id_to_family: BTreeMap<u32, &'static str> = BTreeMap::new();
    for lane in &schedule.lanes {
        for &id in &lane.active_canonical_ids {
            id_to_family.insert(id, lane.gpu_family_wire_name);
        }
    }
    for row in &parameter_table.rows {
        match id_to_family.get(&row.canonical_id) {
            None => errors.push(S13eVerifyError {
                kind: S13eVerifyErrorKind::ParameterTableRowReferencesUnknownCanonicalId {
                    canonical_id: row.canonical_id,
                },
            }),
            Some(&actual_family) if actual_family != row.gpu_family_wire_name => {
                errors.push(S13eVerifyError {
                    kind: S13eVerifyErrorKind::ParameterTableRowFamilyMismatch {
                        canonical_id: row.canonical_id,
                        row_family_wire_name: row.gpu_family_wire_name,
                        actual_family_wire_name: actual_family,
                    },
                });
            }
            Some(_) => {}
        }
    }

    errors
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    for window_start in 0..=h.len() - n.len() {
        let mut ok = true;
        for i in 0..n.len() {
            if !h[window_start + i].eq_ignore_ascii_case(&n[i]) {
                ok = false;
                break;
            }
        }
        if ok {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the top-level kernel plan as a deterministic text
/// report.
#[must_use]
pub fn render_kernel_plan_text(plan: &KernelPlanV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3e Kernel Plan (v1)");
    let _ = writeln!(s, "======================");
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
        "  budget_pruning_plan_hash_v1                  : {}",
        hex32(&plan.budget_pruning_plan_hash_v1)
    );
    let _ = writeln!(
        s,
        "  budgeted_activation_summary_hash_v1          : {}",
        hex32(&plan.budgeted_activation_summary_hash_v1)
    );
    let _ = writeln!(
        s,
        "  SEED.len()                                   : {}",
        plan.seed_len
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Plan shape");
    let _ = writeln!(s, "  lane_count                : {}", plan.lane_count);
    let _ = writeln!(
        s,
        "  total_active_count        : {}",
        plan.total_active_count
    );
    let _ = writeln!(
        s,
        "  total_aggregate_cost_us   : {}",
        plan.total_aggregate_cost_us
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Component hashes");
    let _ = writeln!(
        s,
        "  kernel_family_schedule_hash_v1 : {}",
        hex32(&plan.kernel_family_schedule_hash_v1)
    );
    let _ = writeln!(
        s,
        "  kernel_parameter_table_hash_v1 : {}",
        hex32(&plan.kernel_parameter_table_hash_v1)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "kernel_plan_hash_v1 : {}",
        hex32(&plan.kernel_plan_hash_v1)
    );
    s
}

/// Render the family schedule as deterministic text.
#[must_use]
pub fn render_kernel_family_schedule_text(schedule: &KernelFamilyScheduleV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3e Kernel Family Schedule (v1)");
    let _ = writeln!(s, "=================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "lanes              : {}", schedule.lanes.len());
    let _ = writeln!(s, "total_active_count : {}", schedule.total_active_count);
    let _ = writeln!(s);
    for lane in &schedule.lanes {
        let _ = writeln!(s, "  {}", lane.gpu_family_wire_name);
        let _ = writeln!(
            s,
            "    active_detector_count : {}",
            lane.active_detector_count
        );
        let _ = writeln!(
            s,
            "    declared_cost_model   : {}",
            lane.declared_cost_model
        );
        let _ = writeln!(
            s,
            "    expected_kernel_name  : {}",
            lane.expected_kernel_name
        );
        let _ = writeln!(s, "    aggregate_cost_us     : {}", lane.aggregate_cost_us);
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "kernel_family_schedule_hash_v1 : {}",
        hex32(&schedule.kernel_family_schedule_hash_v1)
    );
    s
}

/// Render the parameter table as deterministic text.
#[must_use]
pub fn render_kernel_parameter_table_text(table: &KernelParameterTableV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S1.3e Kernel Parameter Table (v1)");
    let _ = writeln!(s, "=================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "row_count : {}", table.rows.len());
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "kernel_parameter_table_hash_v1 : {}",
        hex32(&table.kernel_parameter_table_hash_v1)
    );
    s
}

/// Render the kernel plan as canonical JSON.
#[must_use]
pub fn render_kernel_plan_json(plan: &KernelPlanV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", S13E_KERNEL_PLAN_SCHEMA_V1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v1", &plan.corpus_hash_v1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v2", &plan.corpus_hash_v2);
    s.push(',');
    json_hex(
        &mut s,
        "ff1_passport_index_hash_v1",
        &plan.ff1_passport_index_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "ff2_activation_ratification_gate_hash_v1",
        &plan.ff2_activation_ratification_gate_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "ff3_registry_generation_gate_hash_v1",
        &plan.ff3_registry_generation_gate_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "ff4_readme_authority_boundary_policy_hash_v1",
        &plan.ff4_readme_authority_boundary_policy_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "proposal_schema_upgrade_policy_hash_v1",
        &plan.proposal_schema_upgrade_policy_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "budget_pruning_plan_hash_v1",
        &plan.budget_pruning_plan_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "budgeted_activation_summary_hash_v1",
        &plan.budgeted_activation_summary_hash_v1,
    );
    s.push(',');
    let _ = write!(s, "\"seed_len\":{}", plan.seed_len);
    s.push(',');
    let _ = write!(s, "\"lane_count\":{}", plan.lane_count);
    s.push(',');
    let _ = write!(s, "\"total_active_count\":{}", plan.total_active_count);
    s.push(',');
    let _ = write!(
        s,
        "\"total_aggregate_cost_us\":{}",
        plan.total_aggregate_cost_us
    );
    s.push(',');
    json_hex(
        &mut s,
        "kernel_family_schedule_hash_v1",
        &plan.kernel_family_schedule_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "kernel_parameter_table_hash_v1",
        &plan.kernel_parameter_table_hash_v1,
    );
    s.push(',');
    json_hex(&mut s, "kernel_plan_hash_v1", &plan.kernel_plan_hash_v1);
    s.push('}');
    s
}

/// Render the family schedule as canonical JSON.
#[must_use]
pub fn render_kernel_family_schedule_json(schedule: &KernelFamilyScheduleV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", S13E_KERNEL_FAMILY_SCHEDULE_SCHEMA_V1);
    s.push(',');
    let _ = write!(s, "\"total_active_count\":{}", schedule.total_active_count);
    s.push(',');
    let _ = write!(s, "\"lane_count\":{}", schedule.lanes.len());
    s.push(',');
    json_hex(
        &mut s,
        "kernel_family_schedule_hash_v1",
        &schedule.kernel_family_schedule_hash_v1,
    );
    s.push('}');
    s
}

/// Render the parameter table as canonical JSON.
#[must_use]
pub fn render_kernel_parameter_table_json(table: &KernelParameterTableV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", S13E_KERNEL_PARAMETER_TABLE_SCHEMA_V1);
    s.push(',');
    let _ = write!(s, "\"row_count\":{}", table.rows.len());
    s.push(',');
    json_hex(
        &mut s,
        "kernel_parameter_table_hash_v1",
        &table.kernel_parameter_table_hash_v1,
    );
    s.push('}');
    s
}

fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_hex(s: &mut String, key: &str, value: &[u8; 32]) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    let _ = s.write_str(&hex32(value));
    s.push('"');
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

// Forbidden-substring set is exposed read-only for the
// `s13e_cost_model_table_carries_no_forbidden_substring`
// acceptance test.
#[doc(hidden)]
#[must_use]
pub fn forbidden_execution_claim_substrings() -> &'static [&'static str] {
    S13E_FORBIDDEN_EXECUTION_CLAIM_SUBSTRINGS
}
