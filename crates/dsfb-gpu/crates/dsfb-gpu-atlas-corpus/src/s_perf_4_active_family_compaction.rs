//! S-PERF.4 --- active-detector family compaction
//! benchmark schema.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S-PERF.4 defines how the 152 active detectors are
//! > compacted into GPU-family benchmark lanes for Layer-A
//! > measurement. It does not run the benchmark, claim
//! > saturation, change CUDA kernels, or alter activation /
//! > corpus authority. It defines the benchmark schema,
//! > family grouping, parameter-table shape, and compaction
//! > accounting that S-PERF.5 will measure.**
//!
//! Core rule (panel-locked):
//!
//! > Detector count is not kernel count. Active witnesses
//! > must be family-compacted before performance claims are
//! > made.
//!
//! ## Why
//!
//! S-PERF.1 supplied the byte-accounting receipt (measurement
//! law). S-PERF.2 isolated the Layer-A evidence-factory path
//! (measurement boundary). S-PERF.3 pinned the public-data
//! workload (measurement input). S-PERF.4 supplies the
//! *benchmarkable shape* of the active detector set: the 152
//! S1.3d-Active detectors grouped into the 14 GPU-family
//! lanes from S1.3e KernelPlan, each lane's parameter-table
//! offsets pinned, and the entire compaction plan bound by
//! hash to the upstream S1.3d / S1.3e / FF.1 / S-PERF.2 /
//! S-PERF.3 anchors. Without this binding, a future
//! "saturation" claim could silently re-shuffle the active
//! set or substitute one parameter-table layout for another
//! and the number would not be comparable.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes (none folded upstream):
//!
//! - `active_family_compaction_plan_hash_v1` under
//!   `DSFB-GPU-ATLAS:ACTIVE-FAMILY-COMPACTION-PLAN:v1\0`.
//!   Pins the bytes of the per-family compaction plan (one
//!   entry per family lane: family wire name, active
//!   canonical ids sorted ascending, active detector count,
//!   parameter-table offset, expected kernel name,
//!   aggregate cost estimate).
//! - `compacted_parameter_table_receipt_hash_v1` under
//!   `DSFB-GPU-ATLAS:COMPACTED-PARAMETER-TABLE-RECEIPT:v1\0`.
//!   Pins the per-family parameter-table byte-layout shape
//!   (per-family byte size, total byte size, sort-order
//!   wire name).
//! - `family_compaction_benchmark_schema_hash_v1` under
//!   `DSFB-GPU-ATLAS:FAMILY-COMPACTION-BENCHMARK-SCHEMA:v1\0`.
//!   Top-level META-hash binding the plan + the parameter-
//!   table receipt + the S-PERF.2 Layer-A pipeline + traffic
//!   receipt hashes + the S-PERF.3 public-data bundle hash.
//!
//! ## Panel-locked non-claims
//!
//! S-PERF.4 does NOT:
//!
//! - run any benchmark;
//! - claim memory-bandwidth saturation;
//! - emit any timing receipt;
//! - change any CUDA kernel;
//! - change any court decision (S1.3a / FF.2 / FF.3 /
//!   S1.3d / S1.3e / S1.3f / S1.3g);
//! - alter activation outcomes (the active set is read
//!   verbatim from S1.3d's `BudgetedActivationSummary`);
//! - mutate any upstream hash anchor (`corpus_hash_v1`,
//!   `corpus_hash_v2`, every T.11.* / T.12.* / FF.* /
//!   S1.3.* / T.12.PROV / S-PERF.1 / S-PERF.2 / S-PERF.3
//!   hash byte-identical);
//! - alter `SEED.len()` (stays at 54);
//! - emit detector outputs, witness records, fusion
//!   tensors, candidate intervals, or episodes;
//! - decide contraindications or challenges;
//! - modify the registry crate;
//! - download or fetch any dataset bytes.
//!
//! S-PERF.4 ships ONLY the compaction plan + parameter-table
//! receipt + benchmark schema + verifier + builder + baseline
//! derived deterministically from the live S1.3d / S1.3e /
//! FF.1 / S-PERF.2 / S-PERF.3 modules + renderers.
//!
//! ## Panel-locked one-line verdict
//!
//! > S-PERF.3 gives the evidence factory public data;
//! > S-PERF.4 packs the active court witnesses into
//! > benchmarkable GPU-family lanes.

use core::fmt::Write;
use std::collections::{BTreeMap, BTreeSet};

use dsfb_gpu_debug_core::sha256;

use crate::ff1_passport_materialisation::build_ff1_passport_index;
use crate::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, BudgetedActivationSummary, S13dOutcome,
};
use crate::s1_3e_kernel_plan::{build_kernel_plan_v1, FamilyLane, KernelPlanV1};
use crate::s_perf_2_layer_a_resident_pipeline::seed_baseline_layer_a_traffic_receipt;
use crate::s_perf_3_public_data_saturation_bundle::seed_baseline_public_data_saturation_bundle;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for
/// `active_family_compaction_plan_hash_v1`.
pub const ACTIVE_FAMILY_COMPACTION_PLAN_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:ACTIVE-FAMILY-COMPACTION-PLAN:v1\0";

/// Schema identifier for
/// `active_family_compaction_plan_hash_v1`.
pub const ACTIVE_FAMILY_COMPACTION_PLAN_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:ACTIVE-FAMILY-COMPACTION-PLAN:v1";

/// Domain separator for
/// `compacted_parameter_table_receipt_hash_v1`.
pub const COMPACTED_PARAMETER_TABLE_RECEIPT_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:COMPACTED-PARAMETER-TABLE-RECEIPT:v1\0";

/// Schema identifier for
/// `compacted_parameter_table_receipt_hash_v1`.
pub const COMPACTED_PARAMETER_TABLE_RECEIPT_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:COMPACTED-PARAMETER-TABLE-RECEIPT:v1";

/// Domain separator for
/// `family_compaction_benchmark_schema_hash_v1`.
pub const FAMILY_COMPACTION_BENCHMARK_SCHEMA_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:FAMILY-COMPACTION-BENCHMARK-SCHEMA:v1\0";

/// Schema identifier for
/// `family_compaction_benchmark_schema_hash_v1`.
pub const FAMILY_COMPACTION_BENCHMARK_SCHEMA_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:FAMILY-COMPACTION-BENCHMARK-SCHEMA:v1";

// ---------------------------------------------------------------
// Panel-locked parameter-table sort order
// ---------------------------------------------------------------

/// The panel-locked sort order every S-PERF.4 parameter table
/// MUST declare. The verifier rejects any other value via
/// `ParameterTableWithoutStableSortOrder`.
///
/// Wire name: `"CanonicalIdAscendingWithinFamily"`. Means the
/// parameter table is grouped by GPU family wire name
/// ascending, and within each family group the rows are
/// sorted by canonical id ascending. This matches S1.3e's
/// `KernelParameterTableV1` sort order exactly so the two
/// receipts agree by construction.
pub const S_PERF_4_PANEL_LOCKED_PARAMETER_TABLE_SORT_ORDER: &str =
    "CanonicalIdAscendingWithinFamily";

// ---------------------------------------------------------------
// Forbidden benchmark-claim substrings
// ---------------------------------------------------------------

/// Substrings that MUST NOT appear in any S-PERF.4 free-text
/// field (`schema_id`, `plan_id`, `family_wire_name`,
/// `expected_kernel_name`). The S-PERF.4 schema DEFINES the
/// benchmark shape; saturation / throughput / peak-percentage
/// CLAIMS belong to S-PERF.5+ measurement commits, never the
/// schema definition.
///
/// The scanner is case-insensitive so phrasing variants like
/// "PEAK%" or "SaTuRaTeS" are all caught. The forbidden set
/// mirrors the S-PERF.3 set verbatim so the two commits
/// enforce the same discipline.
const S_PERF_4_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS: &[&str] = &[
    "achieves saturation",
    "saturates the bandwidth",
    "saturates peak",
    "% of peak",
    "percent of peak",
    "outperforms",
    "beats the baseline",
    "world record",
    "fastest gpu",
    "production-ready performance",
    "petaflops",
    "memory-bandwidth saturation",
];

// ---------------------------------------------------------------
// FamilyLaneCompactionEntry
// ---------------------------------------------------------------

/// One per-family lane compaction entry. Pins which active
/// detector canonical ids belong to which GPU family lane,
/// what the lane's parameter-table offset is, and the
/// expected kernel name + aggregate cost estimate inherited
/// from S1.3e's [`FamilyLane`].
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `active_family_compaction_plan_hash_v1`.
#[derive(Debug, Clone)]
pub struct FamilyLaneCompactionEntry {
    /// GPU family wire name (e.g.
    /// `"WindowStatisticFamily"`). Non-empty (panel-required
    /// negative #3).
    pub gpu_family_wire_name: &'static str,
    /// Active detector canonical ids assigned to this lane,
    /// sorted ascending. The verifier rejects unsorted lists
    /// via the structural rule
    /// `LaneCanonicalIdsNotSortedAscending`.
    pub active_canonical_ids: Vec<u32>,
    /// Mirrors `active_canonical_ids.len()` (redundant
    /// cross-check the verifier enforces via
    /// `LaneActiveDetectorCountMismatch`).
    pub active_detector_count: u32,
    /// Stable offset within the global parameter table.
    /// Lanes are sorted by `gpu_family_wire_name` ascending;
    /// offsets are cumulative.
    pub parameter_table_offset: u32,
    /// Canonical expected kernel name (mirrors S1.3e's
    /// [`FamilyLane::expected_kernel_name`]; non-empty).
    pub expected_kernel_name: &'static str,
    /// Aggregate cost estimate for this lane in microseconds
    /// (mirrors S1.3e's [`FamilyLane::aggregate_cost_us`]).
    pub aggregate_cost_us: u64,
}

// ---------------------------------------------------------------
// ActiveFamilyCompactionPlanV1
// ---------------------------------------------------------------

/// The active-family compaction plan. Pins which active
/// detectors are in which family lane, plus the four upstream
/// anchor hashes the plan depends on
/// (`source_budget_summary_hash`, `source_kernel_plan_hash`,
/// `source_passport_index_hash`, plus the panel-locked
/// `corpus_hash_v1`).
///
/// Field order is the canonical hash order; do not reorder
/// without rebaselining
/// `active_family_compaction_plan_hash_v1`.
#[derive(Debug, Clone)]
pub struct ActiveFamilyCompactionPlanV1 {
    /// Human-readable plan identifier (non-empty).
    pub plan_id: &'static str,
    /// `corpus_hash_v1` anchor (must equal live
    /// `compute_corpus_hash_v1`).
    pub corpus_hash_v1: [u8; 32],
    /// S1.3d's `budgeted_activation_summary_hash_v1` (must
    /// equal live `build_budgeted_activation_summary`'s
    /// hash).
    pub source_budget_summary_hash: [u8; 32],
    /// S1.3e's `kernel_plan_hash_v1` (must equal live
    /// `build_kernel_plan_v1`'s hash). Non-zero (panel-
    /// required negative #1).
    pub source_kernel_plan_hash: [u8; 32],
    /// FF.1's `ff1_passport_index_hash_v1` (must equal live
    /// `build_ff1_passport_index`'s hash).
    pub source_passport_index_hash: [u8; 32],
    /// Per-family lane entries, sorted ascending by
    /// `gpu_family_wire_name`.
    pub family_lanes: Vec<FamilyLaneCompactionEntry>,
    /// Mirrors `family_lanes.iter().map(|l|
    /// l.active_detector_count).sum()` (redundant cross-
    /// check).
    pub total_active_detector_count: u32,
    /// Mirrors `family_lanes.len()` (redundant cross-check).
    pub total_family_lane_count: u32,
    /// `active_family_compaction_plan_hash_v1`.
    pub active_family_compaction_plan_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// CompactedParameterTableReceiptV1
// ---------------------------------------------------------------

/// Pins the per-family parameter-table shape (per-family byte
/// size, total byte size, panel-locked sort-order wire name).
/// References the compaction plan by hash so the verifier can
/// cross-check the lane set matches.
#[derive(Debug, Clone)]
pub struct CompactedParameterTableReceiptV1 {
    /// Hash of the compaction plan this receipt was emitted
    /// against.
    pub plan_hash: [u8; 32],
    /// Per-family byte size (sorted ascending by family wire
    /// name). One entry per family lane.
    pub per_family_byte_size: Vec<(&'static str, u64)>,
    /// Total parameter-table bytes (sum of per-family
    /// values).
    pub total_parameter_table_bytes: u64,
    /// Sort-order wire name (panel-locked: must equal
    /// [`S_PERF_4_PANEL_LOCKED_PARAMETER_TABLE_SORT_ORDER`];
    /// panel-required negative #4 fires otherwise).
    pub sort_order_wire_name: &'static str,
    /// `compacted_parameter_table_receipt_hash_v1`.
    pub compacted_parameter_table_receipt_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// FamilyCompactionBenchmarkSchemaV1
// ---------------------------------------------------------------

/// The top-level S-PERF.4 benchmark schema. Binds the
/// compaction plan + the parameter-table receipt + the
/// S-PERF.2 Layer-A pipeline + traffic receipt anchors +
/// the S-PERF.3 public-data bundle anchor.
#[derive(Debug, Clone)]
pub struct FamilyCompactionBenchmarkSchemaV1 {
    /// Human-readable schema identifier (non-empty).
    pub schema_id: &'static str,
    /// Wrapped compaction plan.
    pub compaction_plan: ActiveFamilyCompactionPlanV1,
    /// Wrapped parameter-table receipt.
    pub parameter_table_receipt: CompactedParameterTableReceiptV1,
    /// S-PERF.2 Layer-A resident pipeline hash. Must equal
    /// the live baseline pipeline hash (panel-required
    /// negative #8).
    pub layer_a_pipeline_hash: [u8; 32],
    /// S-PERF.2 Layer-A traffic receipt hash. Carried so the
    /// schema cites the full S-PERF.2 receipt chain, not
    /// just the pipeline shape.
    pub layer_a_traffic_receipt_hash: [u8; 32],
    /// S-PERF.3 public-data saturation bundle hash. Must
    /// equal the live baseline bundle hash (panel-required
    /// negative #7).
    pub public_data_bundle_hash: [u8; 32],
    /// `family_compaction_benchmark_schema_hash_v1`.
    pub family_compaction_benchmark_schema_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S-PERF.4 rejected a plan, parameter-table receipt, or
/// benchmark schema. Eight panel-required load-bearing
/// negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SPerf4VerifyErrorKind {
    /// Panel-required negative #1. The compaction plan has
    /// `source_kernel_plan_hash == [0; 32]` (the benchmark
    /// schema does not cite an S1.3e KernelPlan).
    BenchmarkSchemaWithoutKernelPlanHash,
    /// Panel-required negative #2. A lane references a
    /// canonical id that is NOT in S1.3d's Active set.
    DetectorNotActiveInBudgetSummary {
        /// The defective canonical id.
        canonical_id: u32,
        /// The family lane that referenced it.
        family_wire_name: &'static str,
    },
    /// Panel-required negative #3. A family lane has empty
    /// `gpu_family_wire_name`.
    FamilyLaneWithoutGpuFamilyMapping,
    /// Panel-required negative #4. The parameter-table
    /// receipt declares a `sort_order_wire_name` other than
    /// [`S_PERF_4_PANEL_LOCKED_PARAMETER_TABLE_SORT_ORDER`].
    ParameterTableWithoutStableSortOrder {
        /// The observed sort-order wire name.
        observed_sort_order_wire_name: &'static str,
    },
    /// Panel-required negative #5. The same canonical id
    /// appears in more than one family lane (the plan
    /// would count a detector variant as a separate
    /// canonical).
    CompactionThatCountsDetectorVariantsAsNewCanonicals {
        /// The duplicated canonical id.
        canonical_id: u32,
    },
    /// Panel-required negative #6. A free-text field
    /// (`schema_id`, `plan_id`, `family_wire_name`,
    /// `expected_kernel_name`) contains a forbidden
    /// benchmark-claim substring (case-insensitive scan
    /// over [`S_PERF_4_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS`]).
    BenchmarkClaimInsideSchema {
        /// Schema field where the violation appeared.
        location: &'static str,
        /// The forbidden substring observed.
        forbidden_substring: &'static str,
    },
    /// Panel-required negative #7. The schema's
    /// `public_data_bundle_hash` does not equal the live
    /// S-PERF.3 baseline bundle hash.
    DatasetBundleHashMismatch {
        /// Hash the schema claimed.
        claimed: [u8; 32],
        /// Hash the live S-PERF.3 baseline returns.
        actual: [u8; 32],
    },
    /// Panel-required negative #8. The schema's
    /// `layer_a_pipeline_hash` does not equal the live
    /// S-PERF.2 baseline pipeline hash.
    LayerAPipelineHashMismatch {
        /// Hash the schema claimed.
        claimed: [u8; 32],
        /// Hash the live S-PERF.2 baseline returns.
        actual: [u8; 32],
    },
    /// Structural defect: `plan_id` is empty.
    PlanIdEmpty,
    /// Structural defect: `schema_id` is empty.
    SchemaIdEmpty,
    /// Structural defect: `family_lanes` is empty (the
    /// compaction plan has nothing to compact).
    FamilyLanesEmpty,
    /// Structural defect: per-lane `active_canonical_ids` is
    /// not sorted ascending.
    LaneCanonicalIdsNotSortedAscending {
        /// The family lane that broke the sort invariant.
        family_wire_name: &'static str,
    },
    /// Structural defect: per-lane `active_detector_count`
    /// does not equal `active_canonical_ids.len()`.
    LaneActiveDetectorCountMismatch {
        /// The defective family wire name.
        family_wire_name: &'static str,
        /// What the lane claimed.
        claimed: u32,
        /// What `active_canonical_ids.len()` actually is.
        actual: u32,
    },
    /// Structural defect: lanes are not sorted ascending by
    /// `gpu_family_wire_name`.
    FamilyLanesNotSortedAscendingByGpuFamilyWireName,
    /// Structural defect: plan's
    /// `total_active_detector_count` does not equal the sum
    /// of per-lane counts.
    TotalActiveDetectorCountMismatch {
        /// What the plan claimed.
        claimed: u32,
        /// What the per-lane sum is.
        actual: u32,
    },
    /// Structural defect: plan's `total_family_lane_count`
    /// does not equal `family_lanes.len()`.
    TotalFamilyLaneCountMismatch {
        /// What the plan claimed.
        claimed: u32,
        /// What `family_lanes.len()` actually is.
        actual: u32,
    },
    /// Structural defect: parameter-table receipt's
    /// `plan_hash` does not equal the compaction plan's
    /// hash.
    ParameterTableReceiptPlanHashMismatch {
        /// What the receipt cited.
        claimed: [u8; 32],
        /// What the plan's hash actually is.
        actual: [u8; 32],
    },
    /// Structural defect: parameter-table receipt's
    /// `total_parameter_table_bytes` does not equal the sum
    /// of per-family bytes.
    ParameterTableTotalBytesMismatch {
        /// What the receipt claimed.
        claimed: u64,
        /// What the per-family sum is.
        actual_sum: u64,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPerf4VerifyError {
    /// Error kind (see [`SPerf4VerifyErrorKind`]).
    pub kind: SPerf4VerifyErrorKind,
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build an [`ActiveFamilyCompactionPlanV1`] and populate
/// `active_family_compaction_plan_hash_v1`. The builder
/// sorts the lanes defensively so the canonical hash is
/// identical regardless of caller order.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn build_active_family_compaction_plan(
    plan_id: &'static str,
    corpus_hash_v1: [u8; 32],
    source_budget_summary_hash: [u8; 32],
    source_kernel_plan_hash: [u8; 32],
    source_passport_index_hash: [u8; 32],
    mut family_lanes: Vec<FamilyLaneCompactionEntry>,
) -> ActiveFamilyCompactionPlanV1 {
    family_lanes.sort_by_key(|l| l.gpu_family_wire_name);
    let total_active_detector_count: u32 = family_lanes
        .iter()
        .map(|l| l.active_detector_count)
        .fold(0u32, u32::saturating_add);
    let total_family_lane_count = u32::try_from(family_lanes.len()).unwrap_or(u32::MAX);
    let mut p = ActiveFamilyCompactionPlanV1 {
        plan_id,
        corpus_hash_v1,
        source_budget_summary_hash,
        source_kernel_plan_hash,
        source_passport_index_hash,
        family_lanes,
        total_active_detector_count,
        total_family_lane_count,
        active_family_compaction_plan_hash_v1: [0u8; 32],
    };
    p.active_family_compaction_plan_hash_v1 = compute_active_family_compaction_plan_hash(&p);
    p
}

/// Build a [`CompactedParameterTableReceiptV1`] and populate
/// `compacted_parameter_table_receipt_hash_v1`. The builder
/// sorts `per_family_byte_size` defensively.
#[must_use]
pub fn build_compacted_parameter_table_receipt(
    plan_hash: [u8; 32],
    mut per_family_byte_size: Vec<(&'static str, u64)>,
    sort_order_wire_name: &'static str,
) -> CompactedParameterTableReceiptV1 {
    per_family_byte_size.sort_by_key(|(name, _)| *name);
    let total_parameter_table_bytes: u64 = per_family_byte_size
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    let mut r = CompactedParameterTableReceiptV1 {
        plan_hash,
        per_family_byte_size,
        total_parameter_table_bytes,
        sort_order_wire_name,
        compacted_parameter_table_receipt_hash_v1: [0u8; 32],
    };
    r.compacted_parameter_table_receipt_hash_v1 =
        compute_compacted_parameter_table_receipt_hash(&r);
    r
}

/// Build a [`FamilyCompactionBenchmarkSchemaV1`] and populate
/// `family_compaction_benchmark_schema_hash_v1`.
#[must_use]
pub fn build_family_compaction_benchmark_schema(
    schema_id: &'static str,
    compaction_plan: ActiveFamilyCompactionPlanV1,
    parameter_table_receipt: CompactedParameterTableReceiptV1,
    layer_a_pipeline_hash: [u8; 32],
    layer_a_traffic_receipt_hash: [u8; 32],
    public_data_bundle_hash: [u8; 32],
) -> FamilyCompactionBenchmarkSchemaV1 {
    let mut s = FamilyCompactionBenchmarkSchemaV1 {
        schema_id,
        compaction_plan,
        parameter_table_receipt,
        layer_a_pipeline_hash,
        layer_a_traffic_receipt_hash,
        public_data_bundle_hash,
        family_compaction_benchmark_schema_hash_v1: [0u8; 32],
    };
    s.family_compaction_benchmark_schema_hash_v1 =
        compute_family_compaction_benchmark_schema_hash(&s);
    s
}

// ---------------------------------------------------------------
// Seed (panel-locked baseline; derived from live upstream
// modules so the baseline cannot drift from the production
// court state)
// ---------------------------------------------------------------

/// Build the panel-locked baseline S-PERF.4 compaction plan.
/// Derived deterministically from the live S1.3d
/// `BudgetedActivationSummary` + S1.3e `KernelPlanV1` + FF.1
/// passport index, so the baseline cannot drift from the
/// production court state.
///
/// The S1.3e kernel plan already contains the 14 family
/// lanes with their active canonical-id sets; S-PERF.4
/// translates each [`FamilyLane`] into a
/// [`FamilyLaneCompactionEntry`] and computes the cumulative
/// parameter-table offset.
#[must_use]
pub fn seed_baseline_active_family_compaction_plan() -> ActiveFamilyCompactionPlanV1 {
    let summary = build_budgeted_activation_summary();
    let kernel_plan = build_kernel_plan_v1();
    let passport_index = build_ff1_passport_index();
    build_active_family_compaction_plan_from(
        "s_perf_4_baseline_compaction_plan_v1",
        &summary,
        &kernel_plan,
        passport_index.ff1_passport_index_hash_v1,
    )
}

/// Build the active-family compaction plan from injected
/// upstream artifacts (used by tests to exercise the
/// verifier's negative paths).
#[must_use]
pub fn build_active_family_compaction_plan_from(
    plan_id: &'static str,
    summary: &BudgetedActivationSummary,
    kernel_plan: &KernelPlanV1,
    passport_index_hash: [u8; 32],
) -> ActiveFamilyCompactionPlanV1 {
    // Rebuild the S1.3e family schedule so we can walk the
    // 14 lanes. The schedule is cheap to rebuild from the
    // summary; the rebuild guarantees we never read stale
    // state.
    let schedule = crate::s1_3e_kernel_plan::build_kernel_family_schedule_v1_from(summary);
    let mut offset: u32 = 0;
    let lanes: Vec<FamilyLaneCompactionEntry> = schedule
        .lanes
        .iter()
        .map(|l: &FamilyLane| {
            let entry = FamilyLaneCompactionEntry {
                gpu_family_wire_name: l.gpu_family_wire_name,
                active_canonical_ids: l.active_canonical_ids.clone(),
                active_detector_count: l.active_detector_count,
                parameter_table_offset: offset,
                expected_kernel_name: l.expected_kernel_name,
                aggregate_cost_us: l.aggregate_cost_us,
            };
            offset = offset.saturating_add(l.active_detector_count);
            entry
        })
        .collect();
    build_active_family_compaction_plan(
        plan_id,
        summary.plan.corpus_hash_v1,
        summary.budgeted_activation_summary_hash_v1,
        kernel_plan.kernel_plan_hash_v1,
        passport_index_hash,
        lanes,
    )
}

/// Build the panel-locked baseline parameter-table receipt.
/// Per-family byte size is the lane's `active_detector_count`
/// multiplied by the panel-locked
/// `S_PERF_4_BYTES_PER_PARAMETER_ROW` constant.
///
/// The byte size is a *receipt* declaration, not a kernel
/// runtime measurement; S-PERF.5+ commits replace this
/// placeholder calculation with the actual measured bytes
/// when the parameter table is laid out in CUDA `__constant__`
/// or device global memory.
#[must_use]
pub fn seed_baseline_compacted_parameter_table_receipt() -> CompactedParameterTableReceiptV1 {
    let plan = seed_baseline_active_family_compaction_plan();
    build_compacted_parameter_table_receipt_from_plan(&plan)
}

/// Build the parameter-table receipt from an injected plan.
#[must_use]
pub fn build_compacted_parameter_table_receipt_from_plan(
    plan: &ActiveFamilyCompactionPlanV1,
) -> CompactedParameterTableReceiptV1 {
    let per_family_byte_size: Vec<(&'static str, u64)> = plan
        .family_lanes
        .iter()
        .map(|l| {
            let bytes = u64::from(l.active_detector_count) * S_PERF_4_BYTES_PER_PARAMETER_ROW;
            (l.gpu_family_wire_name, bytes)
        })
        .collect();
    build_compacted_parameter_table_receipt(
        plan.active_family_compaction_plan_hash_v1,
        per_family_byte_size,
        S_PERF_4_PANEL_LOCKED_PARAMETER_TABLE_SORT_ORDER,
    )
}

/// Panel-locked per-row byte size for the parameter table at
/// S-PERF.4 baseline. 32 bytes per row covers the row's
/// canonical id (u32) + family wire-name offset (u32) +
/// 6 panel-locked parameter slots (u32 each). Future
/// S-PERF.* commits may switch this to a wider per-row layout
/// once the CUDA `__constant__`-memory layout is pinned;
/// changing it here rebaselines the parameter-table receipt
/// hash but does not change the compaction plan hash.
pub const S_PERF_4_BYTES_PER_PARAMETER_ROW: u64 = 32;

/// Build the panel-locked baseline S-PERF.4 benchmark schema.
/// Composes the baseline plan + parameter-table receipt + the
/// S-PERF.2 baseline pipeline + traffic receipt hashes + the
/// S-PERF.3 baseline bundle hash.
#[must_use]
pub fn seed_baseline_family_compaction_benchmark_schema() -> FamilyCompactionBenchmarkSchemaV1 {
    let plan = seed_baseline_active_family_compaction_plan();
    let table_receipt = build_compacted_parameter_table_receipt_from_plan(&plan);
    let traffic = seed_baseline_layer_a_traffic_receipt();
    let bundle = seed_baseline_public_data_saturation_bundle();
    build_family_compaction_benchmark_schema(
        "s_perf_4_baseline_benchmark_schema_v1",
        plan,
        table_receipt,
        traffic.pipeline.layer_a_resident_pipeline_hash_v1,
        traffic.layer_a_traffic_receipt_hash_v1,
        bundle.public_data_saturation_bundle_hash_v1,
    )
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_active_family_compaction_plan_hash(p: &ActiveFamilyCompactionPlanV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(ACTIVE_FAMILY_COMPACTION_PLAN_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(ACTIVE_FAMILY_COMPACTION_PLAN_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, p.plan_id.as_bytes());
    buf.extend_from_slice(&p.corpus_hash_v1);
    buf.extend_from_slice(&p.source_budget_summary_hash);
    buf.extend_from_slice(&p.source_kernel_plan_hash);
    buf.extend_from_slice(&p.source_passport_index_hash);
    let n = u32::try_from(p.family_lanes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for l in &p.family_lanes {
        push_len_prefixed(&mut buf, l.gpu_family_wire_name.as_bytes());
        let m = u32::try_from(l.active_canonical_ids.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&m.to_be_bytes());
        for id in &l.active_canonical_ids {
            buf.extend_from_slice(&id.to_be_bytes());
        }
        buf.extend_from_slice(&l.active_detector_count.to_be_bytes());
        buf.extend_from_slice(&l.parameter_table_offset.to_be_bytes());
        push_len_prefixed(&mut buf, l.expected_kernel_name.as_bytes());
        buf.extend_from_slice(&l.aggregate_cost_us.to_be_bytes());
    }
    buf.extend_from_slice(&p.total_active_detector_count.to_be_bytes());
    buf.extend_from_slice(&p.total_family_lane_count.to_be_bytes());
    sha256(&buf)
}

fn compute_compacted_parameter_table_receipt_hash(
    r: &CompactedParameterTableReceiptV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(COMPACTED_PARAMETER_TABLE_RECEIPT_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(COMPACTED_PARAMETER_TABLE_RECEIPT_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&r.plan_hash);
    let n = u32::try_from(r.per_family_byte_size.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&n.to_be_bytes());
    for (name, bytes) in &r.per_family_byte_size {
        push_len_prefixed(&mut buf, name.as_bytes());
        buf.extend_from_slice(&bytes.to_be_bytes());
    }
    buf.extend_from_slice(&r.total_parameter_table_bytes.to_be_bytes());
    push_len_prefixed(&mut buf, r.sort_order_wire_name.as_bytes());
    sha256(&buf)
}

fn compute_family_compaction_benchmark_schema_hash(
    s: &FamilyCompactionBenchmarkSchemaV1,
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(FAMILY_COMPACTION_BENCHMARK_SCHEMA_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(FAMILY_COMPACTION_BENCHMARK_SCHEMA_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    push_len_prefixed(&mut buf, s.schema_id.as_bytes());
    buf.extend_from_slice(&s.compaction_plan.active_family_compaction_plan_hash_v1);
    buf.extend_from_slice(
        &s.parameter_table_receipt
            .compacted_parameter_table_receipt_hash_v1,
    );
    buf.extend_from_slice(&s.layer_a_pipeline_hash);
    buf.extend_from_slice(&s.layer_a_traffic_receipt_hash);
    buf.extend_from_slice(&s.public_data_bundle_hash);
    sha256(&buf)
}

fn push_len_prefixed(buf: &mut Vec<u8>, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(bytes);
}

// ---------------------------------------------------------------
// Verifier --- compaction plan
// ---------------------------------------------------------------

/// Verify a compaction plan against the panel-locked rules
/// (negatives #1, #2, #3, #5, and the per-lane part of #6)
/// plus structural defects (empty plan_id, empty lanes,
/// unsorted per-lane canonical ids, count mismatch).
#[must_use]
#[allow(clippy::too_many_lines)] // 8 panel-required negatives + structural
pub fn verify_active_family_compaction_plan(
    plan: &ActiveFamilyCompactionPlanV1,
) -> Vec<SPerf4VerifyError> {
    let mut errors: Vec<SPerf4VerifyError> = Vec::new();

    // Panel-required negative #1.
    if plan.source_kernel_plan_hash == [0u8; 32] {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::BenchmarkSchemaWithoutKernelPlanHash,
        });
    }

    // Panel-required negative #2: every lane id must be in
    // the live S1.3d Active set.
    let live_summary = build_budgeted_activation_summary();
    let live_active_ids: BTreeSet<u32> = live_summary
        .plan
        .decisions
        .iter()
        .filter(|d| matches!(d.outcome, S13dOutcome::Active))
        .map(|d| d.canonical_id)
        .collect();
    for l in &plan.family_lanes {
        for id in &l.active_canonical_ids {
            if !live_active_ids.contains(id) {
                errors.push(SPerf4VerifyError {
                    kind: SPerf4VerifyErrorKind::DetectorNotActiveInBudgetSummary {
                        canonical_id: *id,
                        family_wire_name: l.gpu_family_wire_name,
                    },
                });
            }
        }
    }

    // Panel-required negative #3.
    for l in &plan.family_lanes {
        if l.gpu_family_wire_name.is_empty() {
            errors.push(SPerf4VerifyError {
                kind: SPerf4VerifyErrorKind::FamilyLaneWithoutGpuFamilyMapping,
            });
        }
    }

    // Panel-required negative #5: no duplicate canonical id
    // across lanes.
    let mut seen: BTreeMap<u32, &'static str> = BTreeMap::new();
    for l in &plan.family_lanes {
        for id in &l.active_canonical_ids {
            if seen.insert(*id, l.gpu_family_wire_name).is_some() {
                errors.push(SPerf4VerifyError {
                    kind:
                        SPerf4VerifyErrorKind::CompactionThatCountsDetectorVariantsAsNewCanonicals {
                            canonical_id: *id,
                        },
                });
            }
        }
    }

    // Panel-required negative #6 (per-plan scan).
    scan_for_forbidden_substring(plan.plan_id, "plan_id", &mut errors);
    for l in &plan.family_lanes {
        scan_for_forbidden_substring(l.gpu_family_wire_name, "family_wire_name", &mut errors);
        scan_for_forbidden_substring(l.expected_kernel_name, "expected_kernel_name", &mut errors);
    }

    // Structural defects.
    if plan.plan_id.is_empty() {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::PlanIdEmpty,
        });
    }
    if plan.family_lanes.is_empty() {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::FamilyLanesEmpty,
        });
    }
    // Per-lane sorted-ascending invariant.
    for l in &plan.family_lanes {
        for w in l.active_canonical_ids.windows(2) {
            if w[0] >= w[1] {
                errors.push(SPerf4VerifyError {
                    kind: SPerf4VerifyErrorKind::LaneCanonicalIdsNotSortedAscending {
                        family_wire_name: l.gpu_family_wire_name,
                    },
                });
                break;
            }
        }
        let actual_count = u32::try_from(l.active_canonical_ids.len()).unwrap_or(u32::MAX);
        if l.active_detector_count != actual_count {
            errors.push(SPerf4VerifyError {
                kind: SPerf4VerifyErrorKind::LaneActiveDetectorCountMismatch {
                    family_wire_name: l.gpu_family_wire_name,
                    claimed: l.active_detector_count,
                    actual: actual_count,
                },
            });
        }
    }
    // Lanes sorted ascending by family wire name.
    for w in plan.family_lanes.windows(2) {
        if w[0].gpu_family_wire_name >= w[1].gpu_family_wire_name {
            errors.push(SPerf4VerifyError {
                kind: SPerf4VerifyErrorKind::FamilyLanesNotSortedAscendingByGpuFamilyWireName,
            });
            break;
        }
    }
    let actual_total_count: u32 = plan
        .family_lanes
        .iter()
        .map(|l| l.active_detector_count)
        .fold(0u32, u32::saturating_add);
    if plan.total_active_detector_count != actual_total_count {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::TotalActiveDetectorCountMismatch {
                claimed: plan.total_active_detector_count,
                actual: actual_total_count,
            },
        });
    }
    let actual_lane_count = u32::try_from(plan.family_lanes.len()).unwrap_or(u32::MAX);
    if plan.total_family_lane_count != actual_lane_count {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::TotalFamilyLaneCountMismatch {
                claimed: plan.total_family_lane_count,
                actual: actual_lane_count,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- parameter-table receipt
// ---------------------------------------------------------------

/// Verify a parameter-table receipt against the compaction
/// plan it was emitted against. Surfaces panel-required
/// negative #4 (sort order) plus structural defects
/// (plan-hash mismatch, total-bytes mismatch).
#[must_use]
pub fn verify_compacted_parameter_table_receipt(
    receipt: &CompactedParameterTableReceiptV1,
    plan: &ActiveFamilyCompactionPlanV1,
) -> Vec<SPerf4VerifyError> {
    let mut errors: Vec<SPerf4VerifyError> = Vec::new();

    // Panel-required negative #4.
    if receipt.sort_order_wire_name != S_PERF_4_PANEL_LOCKED_PARAMETER_TABLE_SORT_ORDER {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::ParameterTableWithoutStableSortOrder {
                observed_sort_order_wire_name: receipt.sort_order_wire_name,
            },
        });
    }

    // Structural: plan-hash mismatch.
    if receipt.plan_hash != plan.active_family_compaction_plan_hash_v1 {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::ParameterTableReceiptPlanHashMismatch {
                claimed: receipt.plan_hash,
                actual: plan.active_family_compaction_plan_hash_v1,
            },
        });
    }

    // Structural: total bytes mismatch.
    let actual_sum: u64 = receipt
        .per_family_byte_size
        .iter()
        .map(|(_, v)| *v)
        .fold(0u64, u64::saturating_add);
    if actual_sum != receipt.total_parameter_table_bytes {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::ParameterTableTotalBytesMismatch {
                claimed: receipt.total_parameter_table_bytes,
                actual_sum,
            },
        });
    }

    errors
}

// ---------------------------------------------------------------
// Verifier --- benchmark schema
// ---------------------------------------------------------------

/// Verify a complete benchmark schema. Walks the compaction
/// plan + the parameter-table receipt through their own
/// verifiers, then enforces panel-required negatives #7 +
/// #8 + the schema-level part of #6 plus structural defects
/// (empty schema_id).
#[must_use]
pub fn verify_family_compaction_benchmark_schema(
    schema: &FamilyCompactionBenchmarkSchemaV1,
) -> Vec<SPerf4VerifyError> {
    let mut errors: Vec<SPerf4VerifyError> = Vec::new();

    // Re-run the sub-verifiers.
    errors.extend(verify_active_family_compaction_plan(
        &schema.compaction_plan,
    ));
    errors.extend(verify_compacted_parameter_table_receipt(
        &schema.parameter_table_receipt,
        &schema.compaction_plan,
    ));

    // Panel-required negative #6 (schema-level scan).
    scan_for_forbidden_substring(schema.schema_id, "schema_id", &mut errors);

    // Panel-required negative #7.
    let live_bundle = seed_baseline_public_data_saturation_bundle();
    if schema.public_data_bundle_hash != live_bundle.public_data_saturation_bundle_hash_v1 {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::DatasetBundleHashMismatch {
                claimed: schema.public_data_bundle_hash,
                actual: live_bundle.public_data_saturation_bundle_hash_v1,
            },
        });
    }

    // Panel-required negative #8.
    let live_traffic = seed_baseline_layer_a_traffic_receipt();
    let live_pipeline_hash = live_traffic.pipeline.layer_a_resident_pipeline_hash_v1;
    if schema.layer_a_pipeline_hash != live_pipeline_hash {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::LayerAPipelineHashMismatch {
                claimed: schema.layer_a_pipeline_hash,
                actual: live_pipeline_hash,
            },
        });
    }

    // Structural: empty schema_id.
    if schema.schema_id.is_empty() {
        errors.push(SPerf4VerifyError {
            kind: SPerf4VerifyErrorKind::SchemaIdEmpty,
        });
    }

    errors
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn scan_for_forbidden_substring(
    text: &'static str,
    location: &'static str,
    errors: &mut Vec<SPerf4VerifyError>,
) {
    for &forbidden in S_PERF_4_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS {
        if contains_ascii_case_insensitive(text, forbidden) {
            errors.push(SPerf4VerifyError {
                kind: SPerf4VerifyErrorKind::BenchmarkClaimInsideSchema {
                    location,
                    forbidden_substring: forbidden,
                },
            });
        }
    }
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
// Renderers --- text
// ---------------------------------------------------------------

/// Render the compaction plan as deterministic text.
#[must_use]
pub fn render_active_family_compaction_plan_text(p: &ActiveFamilyCompactionPlanV1) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.4 ActiveFamilyCompactionPlanV1");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Identity");
    let _ = writeln!(s, "  plan_id : {}", p.plan_id);
    let _ = writeln!(s);
    let _ = writeln!(s, "Upstream anchors");
    let _ = writeln!(
        s,
        "  corpus_hash_v1                : {}",
        hex32(&p.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  source_budget_summary_hash    : {}",
        hex32(&p.source_budget_summary_hash)
    );
    let _ = writeln!(
        s,
        "  source_kernel_plan_hash       : {}",
        hex32(&p.source_kernel_plan_hash)
    );
    let _ = writeln!(
        s,
        "  source_passport_index_hash    : {}",
        hex32(&p.source_passport_index_hash)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Family lanes ({}) — total_active_detector_count={}",
        p.total_family_lane_count, p.total_active_detector_count
    );
    for l in &p.family_lanes {
        let _ = writeln!(
            s,
            "  {} : active={} offset={} kernel={} cost_us={}",
            l.gpu_family_wire_name,
            l.active_detector_count,
            l.parameter_table_offset,
            l.expected_kernel_name,
            l.aggregate_cost_us
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "active_family_compaction_plan_hash_v1 : {}",
        hex32(&p.active_family_compaction_plan_hash_v1)
    );
    s
}

/// Render the parameter-table receipt as deterministic text.
#[must_use]
pub fn render_compacted_parameter_table_receipt_text(
    r: &CompactedParameterTableReceiptV1,
) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "S-PERF.4 CompactedParameterTableReceiptV1");
    let _ = writeln!(s, "=========================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Plan reference");
    let _ = writeln!(s, "  plan_hash : {}", hex32(&r.plan_hash));
    let _ = writeln!(s);
    let _ = writeln!(s, "Per-family byte size ({})", r.per_family_byte_size.len());
    for (name, bytes) in &r.per_family_byte_size {
        let _ = writeln!(s, "  {name} : {bytes}");
    }
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "total_parameter_table_bytes : {}",
        r.total_parameter_table_bytes
    );
    let _ = writeln!(
        s,
        "sort_order_wire_name        : {}",
        r.sort_order_wire_name
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "compacted_parameter_table_receipt_hash_v1 : {}",
        hex32(&r.compacted_parameter_table_receipt_hash_v1)
    );
    s
}

/// Render the benchmark schema as deterministic text.
#[must_use]
pub fn render_family_compaction_benchmark_schema_text(
    s: &FamilyCompactionBenchmarkSchemaV1,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "S-PERF.4 FamilyCompactionBenchmarkSchemaV1");
    let _ = writeln!(out, "==========================================");
    let _ = writeln!(out);
    let _ = writeln!(out, "Identity");
    let _ = writeln!(out, "  schema_id : {}", s.schema_id);
    let _ = writeln!(out);
    let _ = writeln!(out, "Bound receipts");
    let _ = writeln!(
        out,
        "  active_family_compaction_plan_hash_v1     : {}",
        hex32(&s.compaction_plan.active_family_compaction_plan_hash_v1)
    );
    let _ = writeln!(
        out,
        "  compacted_parameter_table_receipt_hash_v1 : {}",
        hex32(
            &s.parameter_table_receipt
                .compacted_parameter_table_receipt_hash_v1
        )
    );
    let _ = writeln!(
        out,
        "  layer_a_pipeline_hash                     : {}",
        hex32(&s.layer_a_pipeline_hash)
    );
    let _ = writeln!(
        out,
        "  layer_a_traffic_receipt_hash              : {}",
        hex32(&s.layer_a_traffic_receipt_hash)
    );
    let _ = writeln!(
        out,
        "  public_data_bundle_hash                   : {}",
        hex32(&s.public_data_bundle_hash)
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "family_compaction_benchmark_schema_hash_v1 : {}",
        hex32(&s.family_compaction_benchmark_schema_hash_v1)
    );
    out
}

// ---------------------------------------------------------------
// Renderers --- JSON
// ---------------------------------------------------------------

/// Render the compaction plan as canonical JSON.
#[must_use]
pub fn render_active_family_compaction_plan_json(p: &ActiveFamilyCompactionPlanV1) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", ACTIVE_FAMILY_COMPACTION_PLAN_SCHEMA_V1);
    s.push(',');
    json_string(&mut s, "plan_id", p.plan_id);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v1", &p.corpus_hash_v1);
    s.push(',');
    json_hex(
        &mut s,
        "source_budget_summary_hash",
        &p.source_budget_summary_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "source_kernel_plan_hash",
        &p.source_kernel_plan_hash,
    );
    s.push(',');
    json_hex(
        &mut s,
        "source_passport_index_hash",
        &p.source_passport_index_hash,
    );
    s.push(',');
    let _ = write!(
        s,
        "\"total_active_detector_count\":{}",
        p.total_active_detector_count
    );
    s.push(',');
    let _ = write!(
        s,
        "\"total_family_lane_count\":{}",
        p.total_family_lane_count
    );
    s.push(',');
    s.push_str("\"family_lanes\":[");
    for (i, l) in p.family_lanes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_string(&mut s, "gpu_family_wire_name", l.gpu_family_wire_name);
        s.push(',');
        let _ = write!(s, "\"active_detector_count\":{}", l.active_detector_count);
        s.push(',');
        let _ = write!(s, "\"parameter_table_offset\":{}", l.parameter_table_offset);
        s.push(',');
        json_string(&mut s, "expected_kernel_name", l.expected_kernel_name);
        s.push(',');
        let _ = write!(s, "\"aggregate_cost_us\":{}", l.aggregate_cost_us);
        s.push('}');
    }
    s.push(']');
    s.push(',');
    json_hex(
        &mut s,
        "active_family_compaction_plan_hash_v1",
        &p.active_family_compaction_plan_hash_v1,
    );
    s.push('}');
    s
}

/// Render the parameter-table receipt as canonical JSON.
#[must_use]
pub fn render_compacted_parameter_table_receipt_json(
    r: &CompactedParameterTableReceiptV1,
) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        COMPACTED_PARAMETER_TABLE_RECEIPT_SCHEMA_V1,
    );
    s.push(',');
    json_hex(&mut s, "plan_hash", &r.plan_hash);
    s.push(',');
    s.push_str("\"per_family_byte_size\":[");
    for (i, (name, bytes)) in r.per_family_byte_size.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('{');
        json_string(&mut s, "gpu_family_wire_name", name);
        s.push(',');
        let _ = write!(s, "\"bytes\":{bytes}");
        s.push('}');
    }
    s.push(']');
    s.push(',');
    let _ = write!(
        s,
        "\"total_parameter_table_bytes\":{}",
        r.total_parameter_table_bytes
    );
    s.push(',');
    json_string(&mut s, "sort_order_wire_name", r.sort_order_wire_name);
    s.push(',');
    json_hex(
        &mut s,
        "compacted_parameter_table_receipt_hash_v1",
        &r.compacted_parameter_table_receipt_hash_v1,
    );
    s.push('}');
    s
}

/// Render the benchmark schema as canonical JSON.
#[must_use]
pub fn render_family_compaction_benchmark_schema_json(
    s: &FamilyCompactionBenchmarkSchemaV1,
) -> String {
    let mut out = String::new();
    out.push('{');
    json_field(
        &mut out,
        "schema_id",
        FAMILY_COMPACTION_BENCHMARK_SCHEMA_SCHEMA_V1,
    );
    out.push(',');
    json_string(&mut out, "schema_identifier", s.schema_id);
    out.push(',');
    json_hex(
        &mut out,
        "active_family_compaction_plan_hash_v1",
        &s.compaction_plan.active_family_compaction_plan_hash_v1,
    );
    out.push(',');
    json_hex(
        &mut out,
        "compacted_parameter_table_receipt_hash_v1",
        &s.parameter_table_receipt
            .compacted_parameter_table_receipt_hash_v1,
    );
    out.push(',');
    json_hex(&mut out, "layer_a_pipeline_hash", &s.layer_a_pipeline_hash);
    out.push(',');
    json_hex(
        &mut out,
        "layer_a_traffic_receipt_hash",
        &s.layer_a_traffic_receipt_hash,
    );
    out.push(',');
    json_hex(
        &mut out,
        "public_data_bundle_hash",
        &s.public_data_bundle_hash,
    );
    out.push(',');
    json_hex(
        &mut out,
        "family_compaction_benchmark_schema_hash_v1",
        &s.family_compaction_benchmark_schema_hash_v1,
    );
    out.push('}');
    out
}

fn json_field(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":\"");
    s.push_str(value);
    s.push('"');
}

fn json_string(s: &mut String, key: &str, value: &str) {
    s.push('"');
    s.push_str(key);
    s.push_str("\":");
    s.push('"');
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

/// Test-only read-only access to the forbidden benchmark-
/// claim substring set.
#[doc(hidden)]
#[must_use]
pub fn forbidden_benchmark_claim_substrings() -> &'static [&'static str] {
    S_PERF_4_FORBIDDEN_BENCHMARK_CLAIM_SUBSTRINGS
}
