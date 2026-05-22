//! S1.3f acceptance suite --- CaseFileV2 activation
//! integration invariants.
//!
//! Ten panel-required load-bearing negatives pin the
//! discipline S1.3f exists to prove:
//!
//! * `s13f_rejects_casefile_without_activation_plan_hash`
//! * `s13f_rejects_casefile_without_activation_context_hash`
//! * `s13f_rejects_casefile_without_budget_summary_hash`
//! * `s13f_rejects_casefile_without_kernel_plan_hash`
//! * `s13f_rejects_casefile_with_kernel_plan_not_matching_budgeted_activation`
//! * `s13f_rejects_casefile_with_detector_result_not_in_kernel_plan`
//! * `s13f_rejects_casefile_with_suppressed_detector_result_as_active`
//! * `s13f_rejects_casefile_without_ff2_or_ff3_gate_hash`
//! * `s13f_rejects_casefile_without_challenge_or_contraindication_linkage`
//! * `s13f_rejects_casefile_authority_chain_mutating_upstream_hashes`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > S1.3f makes CaseFileV2 carry the whole activation-to-
//! > kernel authority chain, so evidence output cannot be
//! > detached from the court decisions that allowed it to
//! > exist.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::challenge_docket::{
    collect_challenge_docket, compute_challenge_docket_hash_v1,
};
use dsfb_gpu_atlas_corpus::contraindication::{
    collect_contraindications, compute_contraindication_hash_v1,
};
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::coverage_holes::{
    collect_coverage_holes, compute_coverage_hole_hash_v1,
};
use dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, build_budgeted_activation_summary_with,
    default_redundancy_clusters, default_task_budget,
};
use dsfb_gpu_atlas_corpus::s1_3e_kernel_plan::{
    build_kernel_family_schedule_v1_from, build_kernel_parameter_table_v1_from,
    build_kernel_plan_v1_from,
};
use dsfb_gpu_atlas_corpus::s1_3f_casefile_v2_activation::{
    build_activation_binding, build_casefile_v2_authority_chain, build_default_activation_context,
    build_kernel_plan_binding, build_live_ff2_ff3_gates, build_live_plan_audit,
    canonical_gpu_family_wire_names, render_activation_binding_json,
    render_activation_binding_text, render_authority_chain_json, render_authority_chain_text,
    render_kernel_plan_binding_json, render_kernel_plan_binding_text, verify_s1_3f,
    CaseFileBodyDetectorResultClaim, CaseFileV2AuthorityChain, LaneMembershipRow,
    S13fVerifyErrorKind, CASEFILE_V2_ACTIVATION_BINDING_DOMAIN_V1,
    CASEFILE_V2_ACTIVATION_BINDING_SCHEMA_V1, CASEFILE_V2_AUTHORITY_CHAIN_DOMAIN_V1,
    CASEFILE_V2_AUTHORITY_CHAIN_SCHEMA_V1, CASEFILE_V2_KERNEL_PLAN_BINDING_DOMAIN_V1,
    CASEFILE_V2_KERNEL_PLAN_BINDING_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn fresh_chain() -> CaseFileV2AuthorityChain {
    build_casefile_v2_authority_chain()
}

fn verify_with_no_body_claims(
    chain: &CaseFileV2AuthorityChain,
) -> Vec<dsfb_gpu_atlas_corpus::s1_3f_casefile_v2_activation::S13fVerifyError> {
    let summary = build_budgeted_activation_summary();
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let cf_hash = compute_contraindication_hash_v1(&collect_contraindications());
    let ch_hash = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let coverage_hash = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    verify_s1_3f(
        chain,
        &[],
        &summary,
        &audit,
        &context,
        &plan,
        cf_hash,
        ch_hash,
        coverage_hash,
    )
}

// ---------------------------------------------------------------
// Baseline state + pinned anchors
// ---------------------------------------------------------------

#[test]
fn s13f_default_chain_passes_verifier_with_empty_body() {
    let chain = fresh_chain();
    let errors = verify_with_no_body_claims(&chain);
    assert!(
        errors.is_empty(),
        "expected zero verifier errors at S1.3f baseline; got {errors:?}"
    );
}

#[test]
fn s13f_default_chain_activation_binding_has_54_decisions_152_active_0_disabled() {
    let chain = fresh_chain();
    assert_eq!(chain.activation_binding.activation_decision_count, 54);
    assert_eq!(chain.activation_binding.budget_active_count, 152);
    assert_eq!(chain.activation_binding.budget_disabled_count, 0);
}

#[test]
fn s13f_default_chain_kernel_plan_binding_has_14_lanes_152_active() {
    let chain = fresh_chain();
    assert_eq!(chain.kernel_plan_binding.lane_count, 14);
    assert_eq!(chain.kernel_plan_binding.total_active_count, 152);
    assert_eq!(chain.kernel_plan_binding.lane_membership_index.len(), 152);
}

#[test]
fn s13f_seed_len_pinned_at_54() {
    assert_eq!(SEED.len(), 54);
}

#[test]
fn s13f_chain_pins_corpus_hash_v1_live_value() {
    let chain = fresh_chain();
    assert_eq!(chain.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn s13f_chain_pins_ff2_gate_hash() {
    let chain = fresh_chain();
    let (ff2, _ff3) = build_live_ff2_ff3_gates();
    assert_eq!(chain.ff2_activation_ratification_gate_hash_v1, ff2);
}

#[test]
fn s13f_chain_pins_ff3_gate_hash() {
    let chain = fresh_chain();
    let (_ff2, ff3) = build_live_ff2_ff3_gates();
    assert_eq!(chain.ff3_registry_generation_gate_hash_v1, ff3);
}

#[test]
fn s13f_chain_pins_contraindication_hash() {
    let chain = fresh_chain();
    let live = compute_contraindication_hash_v1(&collect_contraindications());
    assert_eq!(chain.detector_contraindication_hash_v1, live);
}

#[test]
fn s13f_chain_pins_challenge_docket_hash() {
    let chain = fresh_chain();
    let live = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    assert_eq!(chain.challenge_docket_hash_v1, live);
}

#[test]
fn s13f_chain_pins_coverage_hole_hash() {
    let chain = fresh_chain();
    let live = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    assert_eq!(chain.coverage_hole_hash_v1, live);
}

// ---------------------------------------------------------------
// Determinism + sensitivity invariants
// ---------------------------------------------------------------

#[test]
fn s13f_authority_chain_hash_is_deterministic_across_two_builds() {
    let a = build_casefile_v2_authority_chain().casefile_v2_authority_chain_hash_v1;
    let b = build_casefile_v2_authority_chain().casefile_v2_authority_chain_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13f_activation_binding_hash_is_deterministic_across_two_builds() {
    let a = build_casefile_v2_authority_chain()
        .activation_binding
        .casefile_v2_activation_binding_hash_v1;
    let b = build_casefile_v2_authority_chain()
        .activation_binding
        .casefile_v2_activation_binding_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13f_kernel_plan_binding_hash_is_deterministic_across_two_builds() {
    let a = build_casefile_v2_authority_chain()
        .kernel_plan_binding
        .casefile_v2_kernel_plan_binding_hash_v1;
    let b = build_casefile_v2_authority_chain()
        .kernel_plan_binding
        .casefile_v2_kernel_plan_binding_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13f_authority_chain_text_is_byte_stable_across_two_renders() {
    let a = render_authority_chain_text(&fresh_chain());
    let b = render_authority_chain_text(&fresh_chain());
    assert_eq!(a, b);
}

#[test]
fn s13f_authority_chain_json_is_byte_stable_across_two_renders() {
    let a = render_authority_chain_json(&fresh_chain());
    let b = render_authority_chain_json(&fresh_chain());
    assert_eq!(a, b);
}

#[test]
fn s13f_activation_binding_text_is_byte_stable_across_two_renders() {
    let a = render_activation_binding_text(&fresh_chain().activation_binding);
    let b = render_activation_binding_text(&fresh_chain().activation_binding);
    assert_eq!(a, b);
}

#[test]
fn s13f_kernel_plan_binding_text_is_byte_stable_across_two_renders() {
    let a = render_kernel_plan_binding_text(&fresh_chain().kernel_plan_binding);
    let b = render_kernel_plan_binding_text(&fresh_chain().kernel_plan_binding);
    assert_eq!(a, b);
}

#[test]
fn s13f_activation_binding_json_is_byte_stable_across_two_renders() {
    let a = render_activation_binding_json(&fresh_chain().activation_binding);
    let b = render_activation_binding_json(&fresh_chain().activation_binding);
    assert_eq!(a, b);
}

#[test]
fn s13f_kernel_plan_binding_json_is_byte_stable_across_two_renders() {
    let a = render_kernel_plan_binding_json(&fresh_chain().kernel_plan_binding);
    let b = render_kernel_plan_binding_json(&fresh_chain().kernel_plan_binding);
    assert_eq!(a, b);
}

#[test]
fn s13f_authority_chain_hash_changes_when_budget_pressure_changes_active_set() {
    let base = build_casefile_v2_authority_chain().casefile_v2_authority_chain_hash_v1;
    // A tighter budget changes S1.3d's active set → cascades
    // into S1.3e family schedule → S1.3f kernel-plan binding
    // → top-level authority chain hash.
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let schedule = build_kernel_family_schedule_v1_from(&summary);
    let table = build_kernel_parameter_table_v1_from(&summary);
    let activation_binding = build_activation_binding(&summary, &context, &audit);
    let kernel_plan_binding = build_kernel_plan_binding(&plan, &schedule, &table);
    let pressed_kp_binding_hash = kernel_plan_binding.casefile_v2_kernel_plan_binding_hash_v1;
    let pressed_act_binding_hash = activation_binding.casefile_v2_activation_binding_hash_v1;
    let pressed_kp_only = pressed_kp_binding_hash
        != fresh_chain()
            .kernel_plan_binding
            .casefile_v2_kernel_plan_binding_hash_v1;
    let pressed_act_only = pressed_act_binding_hash
        != fresh_chain()
            .activation_binding
            .casefile_v2_activation_binding_hash_v1;
    assert!(
        pressed_kp_only || pressed_act_only,
        "budget pressure should propagate to at least one binding hash; \
         base authority hash = {base:?}, kp_binding shift = {pressed_kp_only}, \
         act_binding shift = {pressed_act_only}"
    );
}

// ---------------------------------------------------------------
// Domain-separator + schema-id pins
// ---------------------------------------------------------------

#[test]
fn s13f_activation_binding_domain_is_pinned() {
    assert_eq!(
        CASEFILE_V2_ACTIVATION_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1\0"
    );
}

#[test]
fn s13f_activation_binding_schema_is_pinned() {
    assert_eq!(
        CASEFILE_V2_ACTIVATION_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-ACTIVATION-BINDING:v1"
    );
}

#[test]
fn s13f_kernel_plan_binding_domain_is_pinned() {
    assert_eq!(
        CASEFILE_V2_KERNEL_PLAN_BINDING_DOMAIN_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1\0"
    );
}

#[test]
fn s13f_kernel_plan_binding_schema_is_pinned() {
    assert_eq!(
        CASEFILE_V2_KERNEL_PLAN_BINDING_SCHEMA_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-KERNEL-PLAN-BINDING:v1"
    );
}

#[test]
fn s13f_authority_chain_domain_is_pinned() {
    assert_eq!(
        CASEFILE_V2_AUTHORITY_CHAIN_DOMAIN_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1\0"
    );
}

#[test]
fn s13f_authority_chain_schema_is_pinned() {
    assert_eq!(
        CASEFILE_V2_AUTHORITY_CHAIN_SCHEMA_V1,
        "DSFB-GPU-ATLAS:CASEFILE-V2-AUTHORITY-CHAIN:v1"
    );
}

#[test]
fn s13f_three_hash_namespaces_are_distinct() {
    let chain = fresh_chain();
    let a = chain
        .activation_binding
        .casefile_v2_activation_binding_hash_v1;
    let k = chain
        .kernel_plan_binding
        .casefile_v2_kernel_plan_binding_hash_v1;
    let t = chain.casefile_v2_authority_chain_hash_v1;
    assert_ne!(a, k);
    assert_ne!(a, t);
    assert_ne!(k, t);
}

// ---------------------------------------------------------------
// Structural invariants
// ---------------------------------------------------------------

#[test]
fn s13f_lane_membership_index_is_sorted_ascending() {
    let chain = fresh_chain();
    for w in chain.kernel_plan_binding.lane_membership_index.windows(2) {
        assert!(w[0].canonical_id < w[1].canonical_id);
    }
}

#[test]
fn s13f_lane_membership_index_size_matches_active_count() {
    let chain = fresh_chain();
    assert_eq!(
        chain.kernel_plan_binding.lane_membership_index.len(),
        chain.kernel_plan_binding.total_active_count as usize
    );
}

#[test]
fn s13f_every_lane_membership_row_carries_a_canonical_family_wire_name() {
    let chain = fresh_chain();
    let canonical = canonical_gpu_family_wire_names();
    for row in &chain.kernel_plan_binding.lane_membership_index {
        assert!(
            canonical.contains(row.gpu_family_wire_name),
            "row carried non-canonical family wire name `{}`",
            row.gpu_family_wire_name
        );
    }
}

#[test]
fn s13f_lane_membership_index_covers_every_active_canonical_id() {
    let chain = fresh_chain();
    let summary = build_budgeted_activation_summary();
    let active_ids: std::collections::BTreeSet<u32> = summary
        .plan
        .decisions
        .iter()
        .filter(|d| d.outcome == dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::S13dOutcome::Active)
        .map(|d| d.canonical_id)
        .collect();
    let membership_ids: std::collections::BTreeSet<u32> = chain
        .kernel_plan_binding
        .lane_membership_index
        .iter()
        .map(|r| r.canonical_id)
        .collect();
    assert_eq!(active_ids, membership_ids);
}

// ---------------------------------------------------------------
// Panel-required load-bearing negatives (10)
// ---------------------------------------------------------------

#[test]
fn s13f_rejects_casefile_without_activation_plan_hash() {
    let mut chain = fresh_chain();
    chain.activation_binding.activation_plan_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutActivationPlanHash
    )));
}

#[test]
fn s13f_rejects_casefile_without_activation_context_hash() {
    let mut chain = fresh_chain();
    chain.activation_binding.activation_context_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutActivationContextHash
    )));
}

#[test]
fn s13f_rejects_casefile_without_budget_summary_hash() {
    let mut chain = fresh_chain();
    chain.activation_binding.budgeted_activation_summary_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutBudgetSummaryHash
    )));
}

#[test]
fn s13f_rejects_casefile_without_kernel_plan_hash() {
    let mut chain = fresh_chain();
    chain.kernel_plan_binding.kernel_plan_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13fVerifyErrorKind::CasefileWithoutKernelPlanHash)));
}

#[test]
fn s13f_rejects_casefile_with_kernel_plan_not_matching_budgeted_activation() {
    let mut chain = fresh_chain();
    // Mutate the activation binding's budgeted-summary anchor
    // to a non-zero, non-matching value. The verifier
    // observes the kernel plan was built against the live
    // S1.3d summary, but the activation binding claims a
    // different one → R.5 fires.
    chain.activation_binding.budgeted_activation_summary_hash_v1 = [0x42u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithKernelPlanNotMatchingBudgetedActivation { .. }
    )));
}

#[test]
fn s13f_rejects_casefile_with_detector_result_not_in_kernel_plan() {
    let chain = fresh_chain();
    let summary = build_budgeted_activation_summary();
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let cf_hash = compute_contraindication_hash_v1(&collect_contraindications());
    let ch_hash = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let coverage_hash = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let claims = vec![CaseFileBodyDetectorResultClaim {
        canonical_id: 99_999,
        outcome_wire_name: "Active",
    }];
    let errors = verify_s1_3f(
        &chain,
        &claims,
        &summary,
        &audit,
        &context,
        &plan,
        cf_hash,
        ch_hash,
        coverage_hash,
    );
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithDetectorResultNotInKernelPlan { canonical_id }
        if canonical_id == 99_999
    )));
}

#[test]
fn s13f_rejects_casefile_with_suppressed_detector_result_as_active() {
    // Tighten the budget so id-1 might be suppressed; we
    // need a budgeted-Disabled id and then claim it Active.
    let mut tight = default_task_budget();
    tight.max_active_detectors = 10;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let schedule = build_kernel_family_schedule_v1_from(&summary);
    let table = build_kernel_parameter_table_v1_from(&summary);
    let act_binding = build_activation_binding(&summary, &context, &audit);
    let kp_binding = build_kernel_plan_binding(&plan, &schedule, &table);
    // Pick a Disabled id whose lane membership is in the
    // schedule (we artificially add it to the membership
    // index for this test, so R.6 doesn't also fire).
    let disabled_id: u32 = summary
        .plan
        .decisions
        .iter()
        .find(|d| d.outcome == dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::S13dOutcome::Disabled)
        .map(|d| d.canonical_id)
        .expect("expected at least one disabled id under tight budget");
    let mut kp_binding_mut = kp_binding.clone();
    kp_binding_mut
        .lane_membership_index
        .push(LaneMembershipRow {
            canonical_id: disabled_id,
            gpu_family_wire_name: "WindowStatisticFamily",
            lane_offset: 0,
        });
    kp_binding_mut
        .lane_membership_index
        .sort_by_key(|r| r.canonical_id);
    let chain = CaseFileV2AuthorityChain {
        activation_binding: act_binding,
        kernel_plan_binding: kp_binding_mut,
        ff2_activation_ratification_gate_hash_v1: summary
            .plan
            .ff2_activation_ratification_gate_hash_v1,
        ff3_registry_generation_gate_hash_v1: summary.plan.ff3_registry_generation_gate_hash_v1,
        detector_contraindication_hash_v1: compute_contraindication_hash_v1(
            &collect_contraindications(),
        ),
        challenge_docket_hash_v1: compute_challenge_docket_hash_v1(&collect_challenge_docket()),
        coverage_hole_hash_v1: compute_coverage_hole_hash_v1(&collect_coverage_holes()),
        corpus_hash_v1: summary.plan.corpus_hash_v1,
        corpus_hash_v2: summary.plan.corpus_hash_v2,
        casefile_v2_authority_chain_hash_v1: [0u8; 32],
    };
    let claims = vec![CaseFileBodyDetectorResultClaim {
        canonical_id: disabled_id,
        outcome_wire_name: "Active",
    }];
    let cf_hash = compute_contraindication_hash_v1(&collect_contraindications());
    let ch_hash = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let coverage_hash = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let errors = verify_s1_3f(
        &chain,
        &claims,
        &summary,
        &audit,
        &context,
        &plan,
        cf_hash,
        ch_hash,
        coverage_hash,
    );
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithSuppressedDetectorResultAsActive { canonical_id }
        if canonical_id == disabled_id
    )));
}

#[test]
fn s13f_rejects_casefile_without_ff2_gate_hash() {
    let mut chain = fresh_chain();
    chain.ff2_activation_ratification_gate_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutFf2OrFf3GateHash { gate_wire_name }
        if gate_wire_name == "ff2"
    )));
}

#[test]
fn s13f_rejects_casefile_without_ff3_gate_hash() {
    let mut chain = fresh_chain();
    chain.ff3_registry_generation_gate_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutFf2OrFf3GateHash { gate_wire_name }
        if gate_wire_name == "ff3"
    )));
}

#[test]
fn s13f_rejects_casefile_without_challenge_linkage() {
    let mut chain = fresh_chain();
    chain.challenge_docket_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage { linkage_wire_name }
        if linkage_wire_name == "challenge"
    )));
}

#[test]
fn s13f_rejects_casefile_without_contraindication_linkage() {
    let mut chain = fresh_chain();
    chain.detector_contraindication_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage { linkage_wire_name }
        if linkage_wire_name == "contraindication"
    )));
}

#[test]
fn s13f_rejects_casefile_without_coverage_hole_linkage() {
    let mut chain = fresh_chain();
    chain.coverage_hole_hash_v1 = [0u8; 32];
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileWithoutChallengeOrContraindicationLinkage { linkage_wire_name }
        if linkage_wire_name == "coverage_hole"
    )));
}

#[test]
fn s13f_rejects_casefile_authority_chain_mutating_upstream_hashes_corpus_v1() {
    let mut chain = fresh_chain();
    chain.corpus_hash_v1[0] ^= 0xFF;
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes { anchor_wire_name }
        if anchor_wire_name == "corpus_hash_v1"
    )));
}

#[test]
fn s13f_rejects_casefile_authority_chain_mutating_upstream_hashes_kernel_plan() {
    let mut chain = fresh_chain();
    chain.kernel_plan_binding.kernel_plan_hash_v1[1] ^= 0xAA;
    let errors = verify_with_no_body_claims(&chain);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::CasefileAuthorityChainMutatingUpstreamHashes { anchor_wire_name }
        if anchor_wire_name == "kernel_plan_hash_v1"
    )));
}

// ---------------------------------------------------------------
// Body-claim outcome handling
// ---------------------------------------------------------------

#[test]
fn s13f_admits_body_claim_for_active_canonical_id() {
    let chain = fresh_chain();
    let claims = vec![CaseFileBodyDetectorResultClaim {
        canonical_id: 1,
        outcome_wire_name: "Active",
    }];
    let summary = build_budgeted_activation_summary();
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let cf_hash = compute_contraindication_hash_v1(&collect_contraindications());
    let ch_hash = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let coverage_hash = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let errors = verify_s1_3f(
        &chain,
        &claims,
        &summary,
        &audit,
        &context,
        &plan,
        cf_hash,
        ch_hash,
        coverage_hash,
    );
    assert!(
        errors.is_empty(),
        "expected zero errors for a legitimate Active body claim; got {errors:?}"
    );
}

#[test]
fn s13f_rejects_body_claim_with_unknown_outcome_wire_name() {
    let chain = fresh_chain();
    let claims = vec![CaseFileBodyDetectorResultClaim {
        canonical_id: 1,
        outcome_wire_name: "Sustained",
    }];
    let summary = build_budgeted_activation_summary();
    let audit = build_live_plan_audit();
    let context = build_default_activation_context();
    let plan = build_kernel_plan_v1_from(&summary);
    let cf_hash = compute_contraindication_hash_v1(&collect_contraindications());
    let ch_hash = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let coverage_hash = compute_coverage_hole_hash_v1(&collect_coverage_holes());
    let errors = verify_s1_3f(
        &chain,
        &claims,
        &summary,
        &audit,
        &context,
        &plan,
        cf_hash,
        ch_hash,
        coverage_hash,
    );
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13fVerifyErrorKind::BodyDetectorResultClaimUnknownOutcome { .. }
    )));
}

// ---------------------------------------------------------------
// Upstream anchor invariance witnesses
// ---------------------------------------------------------------

#[test]
fn s13f_does_not_alter_corpus_hash_v1() {
    let _ = build_casefile_v2_authority_chain();
    let v1 = compute_corpus_hash_v1().bytes;
    let prefix: [u8; 4] = [0x35, 0xc2, 0x76, 0xc7];
    assert_eq!(&v1[..4], &prefix);
}

#[test]
fn s13f_does_not_alter_s1_3d_budget_pruning_plan_hash() {
    let _ = build_casefile_v2_authority_chain();
    let s = build_budgeted_activation_summary();
    let prefix: [u8; 4] = [0x82, 0xbe, 0x22, 0x89];
    assert_eq!(&s.plan.budget_pruning_plan_hash_v1[..4], &prefix);
}

#[test]
fn s13f_does_not_alter_s1_3e_kernel_plan_hash() {
    let _ = build_casefile_v2_authority_chain();
    let summary = build_budgeted_activation_summary();
    let p = build_kernel_plan_v1_from(&summary);
    let prefix: [u8; 4] = [0xe4, 0x8c, 0x89, 0xb9];
    assert_eq!(&p.kernel_plan_hash_v1[..4], &prefix);
}

#[test]
fn s13f_does_not_alter_ff2_or_ff3_gate_hashes() {
    let _ = build_casefile_v2_authority_chain();
    let (ff2, ff3) = build_live_ff2_ff3_gates();
    let ff2_prefix: [u8; 4] = [0x05, 0xc1, 0xb5, 0x52];
    let ff3_prefix: [u8; 4] = [0x2f, 0xfd, 0x02, 0x22];
    assert_eq!(&ff2[..4], &ff2_prefix);
    assert_eq!(&ff3[..4], &ff3_prefix);
}

// ---------------------------------------------------------------
// Renderer-coverage spot checks
// ---------------------------------------------------------------

#[test]
fn s13f_authority_chain_text_contains_all_three_section_headers() {
    let s = render_authority_chain_text(&fresh_chain());
    assert!(s.contains("Activation binding"));
    assert!(s.contains("Kernel-plan binding"));
    assert!(s.contains("Linkage anchors"));
    assert!(s.contains("casefile_v2_authority_chain_hash_v1"));
}

#[test]
fn s13f_authority_chain_json_parses_as_object() {
    let s = render_authority_chain_json(&fresh_chain());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13f_activation_binding_json_parses_as_object() {
    let s = render_activation_binding_json(&fresh_chain().activation_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13f_kernel_plan_binding_json_parses_as_object() {
    let s = render_kernel_plan_binding_json(&fresh_chain().kernel_plan_binding);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}
