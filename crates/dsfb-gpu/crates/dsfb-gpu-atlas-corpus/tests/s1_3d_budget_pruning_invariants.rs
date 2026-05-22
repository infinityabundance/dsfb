//! S1.3d acceptance suite — budget pruning + redundancy
//! suppression invariants.
//!
//! Eight panel-required load-bearing negatives pin the
//! discipline S1.3d exists to prove:
//!
//! * `s13d_rejects_budget_plan_that_uses_ff3_rejected_record`
//! * `s13d_rejects_silent_detector_drop_without_suppression_reason`
//! * `s13d_rejects_redundancy_suppression_without_surviving_representative`
//! * `s13d_rejects_budget_overrun_without_reason_coded_pruning`
//! * `s13d_rejects_nondeterministic_tie_break_between_equal_priority_detectors`
//! * `s13d_rejects_gpu_family_budget_without_declared_cost_model`
//! * `s13d_rejects_pruning_that_mutates_corpus_hash_v1_or_v2`
//! * `s13d_rejects_schema_upgrade_side_effect_inside_budget_pruning`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > Eligibility is not activation; activation is not budget
//! > admission.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::ff2_activation_ratification_gate::{
    build_ff2_activation_ratification_gate_from, default_candidate_ids,
};
use dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::build_ff3_registry_generation_gate;
use dsfb_gpu_atlas_corpus::ff4_readme_authority_boundary::build_ff4_readme_authority_boundary_policy;
use dsfb_gpu_atlas_corpus::proposal_schema_policy::build_proposal_schema_upgrade_policy;
use dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, build_budgeted_activation_summary_from,
    build_budgeted_activation_summary_with, default_redundancy_clusters, default_task_budget,
    render_s13d_plan_json, render_s13d_plan_text, render_s13d_redundancy_json,
    render_s13d_redundancy_text, render_s13d_summary_json, render_s13d_summary_text, verify_s1_3d,
    GpuFamilyBudget, RedundancyCluster, S13dBudgetDecision, S13dBudgetDisableReason,
    S13dBudgetRetainReason, S13dOutcome, S13dVerifyErrorKind,
    S13D_BUDGETED_ACTIVATION_SUMMARY_DOMAIN_V1, S13D_BUDGETED_ACTIVATION_SUMMARY_SCHEMA_V1,
    S13D_BUDGET_PRUNING_PLAN_DOMAIN_V1, S13D_BUDGET_PRUNING_PLAN_SCHEMA_V1,
    S13D_REDUNDANCY_SUPPRESSION_DOMAIN_V1, S13D_REDUNDANCY_SUPPRESSION_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

fn live_ff5_policy_hash() -> [u8; 32] {
    build_proposal_schema_upgrade_policy().proposal_schema_upgrade_policy_hash_v1
}

fn fresh_summary() -> dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::BudgetedActivationSummary {
    build_budgeted_activation_summary()
}

fn verify_default() -> Vec<dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::S13dVerifyError> {
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let ff3_gate = build_ff3_registry_generation_gate();
    verify_s1_3d(&summary, &report, &ff3_gate, live_ff5_policy_hash())
}

// Static cluster declarations are hoisted to module scope because
// `RedundancyCluster::member_canonical_ids` carries a `&'static [u32]`
// slice; declaring the slice inside the test body trips clippy's
// `items_after_statements` lint. The hoist is mechanical and does
// not change test semantics.

static S13D_TEST_MEMBERS_1_2_3: &[u32] = &[1, 2, 3];
static S13D_TEST_CLUSTERS_PLAN_HASH_SHIFT: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "test_cluster_seed_1_2_3",
    member_canonical_ids: S13D_TEST_MEMBERS_1_2_3,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_1_2: &[u32] = &[1, 2];
static S13D_TEST_CLUSTERS_REDUNDANCY_HASH_SHIFT: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "test_cluster_pair",
    member_canonical_ids: S13D_TEST_MEMBERS_1_2,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_SHEWHART: &[u32] = &[1, 2, 3];
static S13D_TEST_CLUSTERS_SHEWHART: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "shewhart_aliases",
    member_canonical_ids: S13D_TEST_MEMBERS_SHEWHART,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_10_20_30: &[u32] = &[10, 20, 30];
static S13D_TEST_CLUSTERS_LOWEST_PICKS_10: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "test_lowest_picks_10",
    member_canonical_ids: S13D_TEST_MEMBERS_10_20_30,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_REP_PAIR: &[u32] = &[1, 2];
static S13D_TEST_CLUSTERS_REP_TEST: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "rep_test",
    member_canonical_ids: S13D_TEST_MEMBERS_REP_PAIR,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_EMPTY: &[u32] = &[];
static S13D_TEST_CLUSTERS_EMPTY_MEMBERS: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "empty_cluster",
    member_canonical_ids: S13D_TEST_MEMBERS_EMPTY,
    selection_rule: "lowest_canonical_id",
}];
static S13D_TEST_MEMBERS_EMPTY_RULE_PAIR: &[u32] = &[1, 2];
static S13D_TEST_CLUSTERS_EMPTY_RULE: &[RedundancyCluster] = &[RedundancyCluster {
    cluster_id: "empty_rule_cluster",
    member_canonical_ids: S13D_TEST_MEMBERS_EMPTY_RULE_PAIR,
    selection_rule: "",
}];

// ---------------------------------------------------------------
// Baseline state + pinned anchors
// ---------------------------------------------------------------

#[test]
fn s13d_default_summary_has_152_active_zero_disabled() {
    let s = fresh_summary();
    assert_eq!(s.plan.active_count, 152);
    assert_eq!(s.plan.disabled_count, 0);
    assert_eq!(s.plan.decisions.len(), 152);
}

#[test]
fn s13d_default_summary_passes_verifier() {
    let errors = verify_default();
    assert!(
        errors.is_empty(),
        "expected zero verifier errors at S1.3d baseline; got {errors:?}"
    );
}

#[test]
fn s13d_seed_len_pinned_at_54() {
    assert_eq!(SEED.len(), 54);
}

#[test]
fn s13d_plan_pins_corpus_hash_v1_live_value() {
    let s = fresh_summary();
    assert_eq!(s.plan.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn s13d_plan_pins_corpus_hash_v2_from_consolidation_report() {
    let s = fresh_summary();
    let report = build_consolidation_report();
    assert_eq!(s.plan.corpus_hash_v2, report.corpus_hash_v2);
}

#[test]
fn s13d_plan_pins_ff1_passport_index_hash() {
    let s = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    assert_eq!(
        s.plan.ff1_passport_index_hash_v1,
        passport_index.ff1_passport_index_hash_v1
    );
}

#[test]
fn s13d_plan_pins_ff2_gate_hash() {
    let s = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let ids = default_candidate_ids(&passport_index);
    let ff2 = build_ff2_activation_ratification_gate_from(&report, &passport_index, &ids);
    assert_eq!(
        s.plan.ff2_activation_ratification_gate_hash_v1,
        ff2.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn s13d_plan_pins_ff3_gate_hash() {
    let s = fresh_summary();
    let ff3 = build_ff3_registry_generation_gate();
    assert_eq!(
        s.plan.ff3_registry_generation_gate_hash_v1,
        ff3.ff3_registry_generation_gate_hash_v1
    );
}

#[test]
fn s13d_plan_pins_ff4_policy_hash() {
    let s = fresh_summary();
    let ff4 = build_ff4_readme_authority_boundary_policy();
    assert_eq!(
        s.plan.ff4_readme_authority_boundary_policy_hash_v1,
        ff4.ff4_readme_authority_boundary_policy_hash_v1
    );
}

#[test]
fn s13d_plan_pins_ff5_policy_hash() {
    let s = fresh_summary();
    assert_eq!(
        s.plan.proposal_schema_upgrade_policy_hash_v1,
        live_ff5_policy_hash()
    );
}

#[test]
fn s13d_plan_pins_seed_len_54() {
    let s = fresh_summary();
    assert_eq!(s.plan.seed_len, 54);
}

// ---------------------------------------------------------------
// Determinism + sensitivity invariants
// ---------------------------------------------------------------

#[test]
fn s13d_plan_hash_is_deterministic_across_two_builds() {
    let a = build_budgeted_activation_summary()
        .plan
        .budget_pruning_plan_hash_v1;
    let b = build_budgeted_activation_summary()
        .plan
        .budget_pruning_plan_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13d_redundancy_hash_is_deterministic_across_two_builds() {
    let a = build_budgeted_activation_summary()
        .redundancy_report
        .redundancy_suppression_hash_v1;
    let b = build_budgeted_activation_summary()
        .redundancy_report
        .redundancy_suppression_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13d_summary_hash_is_deterministic_across_two_builds() {
    let a = build_budgeted_activation_summary().budgeted_activation_summary_hash_v1;
    let b = build_budgeted_activation_summary().budgeted_activation_summary_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13d_plan_text_is_byte_stable_across_two_renders() {
    let a = render_s13d_plan_text(&fresh_summary().plan);
    let b = render_s13d_plan_text(&fresh_summary().plan);
    assert_eq!(a, b);
}

#[test]
fn s13d_plan_json_is_byte_stable_across_two_renders() {
    let a = render_s13d_plan_json(&fresh_summary().plan);
    let b = render_s13d_plan_json(&fresh_summary().plan);
    assert_eq!(a, b);
}

#[test]
fn s13d_redundancy_text_is_byte_stable_across_two_renders() {
    let a = render_s13d_redundancy_text(&fresh_summary().redundancy_report);
    let b = render_s13d_redundancy_text(&fresh_summary().redundancy_report);
    assert_eq!(a, b);
}

#[test]
fn s13d_redundancy_json_is_byte_stable_across_two_renders() {
    let a = render_s13d_redundancy_json(&fresh_summary().redundancy_report);
    let b = render_s13d_redundancy_json(&fresh_summary().redundancy_report);
    assert_eq!(a, b);
}

#[test]
fn s13d_summary_text_is_byte_stable_across_two_renders() {
    let a = render_s13d_summary_text(&fresh_summary());
    let b = render_s13d_summary_text(&fresh_summary());
    assert_eq!(a, b);
}

#[test]
fn s13d_summary_json_is_byte_stable_across_two_renders() {
    let a = render_s13d_summary_json(&fresh_summary());
    let b = render_s13d_summary_json(&fresh_summary());
    assert_eq!(a, b);
}

#[test]
fn s13d_plan_hash_changes_when_max_active_detectors_changes() {
    let base = build_budgeted_activation_summary_with(
        default_task_budget(),
        default_redundancy_clusters(),
    )
    .plan
    .budget_pruning_plan_hash_v1;
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50; // forces pruning
    let pressed = build_budgeted_activation_summary_with(tight, default_redundancy_clusters())
        .plan
        .budget_pruning_plan_hash_v1;
    assert_ne!(base, pressed);
}

#[test]
fn s13d_plan_hash_changes_when_redundancy_cluster_added() {
    let base = build_budgeted_activation_summary_with(
        default_task_budget(),
        default_redundancy_clusters(),
    )
    .plan
    .budget_pruning_plan_hash_v1;
    let injected = build_budgeted_activation_summary_with(
        default_task_budget(),
        S13D_TEST_CLUSTERS_PLAN_HASH_SHIFT,
    )
    .plan
    .budget_pruning_plan_hash_v1;
    assert_ne!(base, injected);
}

#[test]
fn s13d_redundancy_hash_changes_when_cluster_added() {
    let base = build_budgeted_activation_summary_with(
        default_task_budget(),
        default_redundancy_clusters(),
    )
    .redundancy_report
    .redundancy_suppression_hash_v1;
    let injected = build_budgeted_activation_summary_with(
        default_task_budget(),
        S13D_TEST_CLUSTERS_REDUNDANCY_HASH_SHIFT,
    )
    .redundancy_report
    .redundancy_suppression_hash_v1;
    assert_ne!(base, injected);
}

#[test]
fn s13d_summary_hash_includes_plan_and_redundancy() {
    // If the plan hash changes the summary hash must change.
    let base = build_budgeted_activation_summary().budgeted_activation_summary_hash_v1;
    let mut tight = default_task_budget();
    tight.max_active_detectors = 30;
    let altered = build_budgeted_activation_summary_with(tight, default_redundancy_clusters())
        .budgeted_activation_summary_hash_v1;
    assert_ne!(base, altered);
}

// ---------------------------------------------------------------
// Domain-separator + schema-id pins
// ---------------------------------------------------------------

#[test]
fn s13d_plan_domain_separator_is_pinned() {
    assert_eq!(
        S13D_BUDGET_PRUNING_PLAN_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1\0"
    );
}

#[test]
fn s13d_plan_schema_id_is_pinned() {
    assert_eq!(
        S13D_BUDGET_PRUNING_PLAN_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13D-BUDGET-PRUNING-PLAN:v1"
    );
}

#[test]
fn s13d_redundancy_domain_separator_is_pinned() {
    assert_eq!(
        S13D_REDUNDANCY_SUPPRESSION_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1\0"
    );
}

#[test]
fn s13d_redundancy_schema_id_is_pinned() {
    assert_eq!(
        S13D_REDUNDANCY_SUPPRESSION_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13D-REDUNDANCY-SUPPRESSION:v1"
    );
}

#[test]
fn s13d_summary_domain_separator_is_pinned() {
    assert_eq!(
        S13D_BUDGETED_ACTIVATION_SUMMARY_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1\0"
    );
}

#[test]
fn s13d_summary_schema_id_is_pinned() {
    assert_eq!(
        S13D_BUDGETED_ACTIVATION_SUMMARY_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13D-BUDGETED-ACTIVATION-SUMMARY:v1"
    );
}

#[test]
fn s13d_three_hash_namespaces_are_distinct() {
    let s = fresh_summary();
    assert_ne!(
        s.plan.budget_pruning_plan_hash_v1,
        s.redundancy_report.redundancy_suppression_hash_v1
    );
    assert_ne!(
        s.plan.budget_pruning_plan_hash_v1,
        s.budgeted_activation_summary_hash_v1
    );
    assert_ne!(
        s.redundancy_report.redundancy_suppression_hash_v1,
        s.budgeted_activation_summary_hash_v1
    );
}

// ---------------------------------------------------------------
// Structural invariants (decisions sorted, IDs unique, reason-
// coded)
// ---------------------------------------------------------------

#[test]
fn s13d_decisions_are_sorted_ascending_by_canonical_id() {
    let s = fresh_summary();
    for w in s.plan.decisions.windows(2) {
        assert!(
            w[0].canonical_id <= w[1].canonical_id,
            "decision list must be sorted ascending; saw {} then {}",
            w[0].canonical_id,
            w[1].canonical_id
        );
    }
}

#[test]
fn s13d_every_decision_has_non_empty_reason_wire_name() {
    let s = fresh_summary();
    for d in &s.plan.decisions {
        assert!(
            !d.reason_wire_name.is_empty(),
            "decision for canonical_id={} carries empty reason wire name",
            d.canonical_id
        );
    }
}

#[test]
fn s13d_every_active_decision_carries_a_retain_reason() {
    let s = fresh_summary();
    let valid = [
        S13dBudgetRetainReason::RetainedAsBudgetSurvivor.as_str(),
        S13dBudgetRetainReason::RetainedAsRepresentativeWitness.as_str(),
    ];
    for d in &s.plan.decisions {
        if d.outcome == S13dOutcome::Active {
            assert!(
                valid.contains(&d.reason_wire_name),
                "active decision for canonical_id={} carries non-retain wire name `{}`",
                d.canonical_id,
                d.reason_wire_name
            );
        }
    }
}

#[test]
fn s13d_every_disabled_decision_carries_a_disable_reason() {
    let s = fresh_summary();
    let valid = [
        S13dBudgetDisableReason::DisabledByBudget.as_str(),
        S13dBudgetDisableReason::DisabledByRedundancy.as_str(),
        S13dBudgetDisableReason::DisabledByGpuFamilyQuota.as_str(),
        S13dBudgetDisableReason::DisabledByTaskBudget.as_str(),
        S13dBudgetDisableReason::DisabledByRuntimeBudget.as_str(),
        S13dBudgetDisableReason::DisabledByMemoryBudget.as_str(),
        S13dBudgetDisableReason::DisabledByContraindicationBudget.as_str(),
        S13dBudgetDisableReason::DisabledByCoverageHoleBudget.as_str(),
    ];
    for d in &s.plan.decisions {
        if d.outcome == S13dOutcome::Disabled {
            assert!(
                valid.contains(&d.reason_wire_name),
                "disabled decision for canonical_id={} carries non-disable wire name `{}`",
                d.canonical_id,
                d.reason_wire_name
            );
        }
    }
}

#[test]
fn s13d_canonical_ids_unique_across_decisions() {
    let s = fresh_summary();
    let mut ids: Vec<u32> = s.plan.decisions.iter().map(|d| d.canonical_id).collect();
    ids.sort_unstable();
    let original_len = ids.len();
    ids.dedup();
    assert_eq!(
        ids.len(),
        original_len,
        "duplicate canonical_id in decision list"
    );
}

#[test]
fn s13d_baseline_uses_only_ff3_eligible_ids() {
    let s = fresh_summary();
    let ff3 = build_ff3_registry_generation_gate();
    let eligible: std::collections::BTreeSet<u32> = ff3
        .decisions
        .iter()
        .filter(|d| {
            d.eligibility
                == dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::Ff3RegistryGenerationEligibility::Eligible
        })
        .map(|d| d.canonical_id)
        .collect();
    for d in &s.plan.decisions {
        assert!(
            eligible.contains(&d.canonical_id),
            "decision for canonical_id={} is not FF.3-eligible",
            d.canonical_id
        );
    }
}

#[test]
fn s13d_baseline_decision_count_equals_ff3_eligible_count() {
    let s = fresh_summary();
    let ff3 = build_ff3_registry_generation_gate();
    let eligible_count = ff3
        .decisions
        .iter()
        .filter(|d| {
            d.eligibility
                == dsfb_gpu_atlas_corpus::ff3_registry_generation_gate::Ff3RegistryGenerationEligibility::Eligible
        })
        .count();
    assert_eq!(s.plan.decisions.len(), eligible_count);
}

#[test]
fn s13d_baseline_decisions_all_retained_as_budget_survivor() {
    let s = fresh_summary();
    for d in &s.plan.decisions {
        assert_eq!(d.outcome, S13dOutcome::Active);
        assert_eq!(
            d.reason_wire_name,
            S13dBudgetRetainReason::RetainedAsBudgetSurvivor.as_str()
        );
    }
}

#[test]
fn s13d_baseline_seed_ids_carry_zero_passport_hash() {
    let s = fresh_summary();
    for d in &s.plan.decisions {
        if d.canonical_id <= 54 {
            assert_eq!(
                d.cited_passport_hash, [0u8; 32],
                "SEED id {} should carry zero passport hash",
                d.canonical_id
            );
        }
    }
}

#[test]
fn s13d_baseline_ratified_ids_carry_non_zero_passport_hash() {
    let s = fresh_summary();
    let mut found_nonzero = false;
    for d in &s.plan.decisions {
        if d.canonical_id > 54 {
            assert_ne!(
                d.cited_passport_hash, [0u8; 32],
                "ratified id {} should carry non-zero passport hash",
                d.canonical_id
            );
            found_nonzero = true;
        }
    }
    assert!(found_nonzero, "expected at least one ratified candidate");
}

#[test]
fn s13d_baseline_tie_break_transcript_is_empty() {
    let s = fresh_summary();
    assert!(s.plan.tie_break_transcript.is_empty());
}

#[test]
fn s13d_baseline_per_reason_counts_sum_to_zero() {
    let s = fresh_summary();
    assert_eq!(s.plan.disabled_by_budget_count, 0);
    assert_eq!(s.plan.disabled_by_redundancy_count, 0);
    assert_eq!(s.plan.disabled_by_gpu_family_quota_count, 0);
    assert_eq!(s.plan.disabled_by_task_budget_count, 0);
    assert_eq!(s.plan.disabled_by_runtime_budget_count, 0);
    assert_eq!(s.plan.disabled_by_memory_budget_count, 0);
    assert_eq!(s.plan.disabled_by_contraindication_budget_count, 0);
    assert_eq!(s.plan.disabled_by_coverage_hole_budget_count, 0);
}

#[test]
fn s13d_baseline_redundancy_report_empty() {
    let s = fresh_summary();
    assert!(s.redundancy_report.clusters.is_empty());
    assert!(s.redundancy_report.retained_representatives.is_empty());
    assert_eq!(s.redundancy_report.suppression_count, 0);
}

// ---------------------------------------------------------------
// Pressure-bearing budget scenarios (exercise the disable
// reasons)
// ---------------------------------------------------------------

#[test]
fn s13d_max_active_detectors_50_produces_task_budget_disables() {
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let s = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    assert_eq!(s.plan.active_count, 50);
    assert_eq!(s.plan.disabled_count, 152 - 50);
    assert_eq!(s.plan.disabled_by_task_budget_count, 102);
}

#[test]
fn s13d_max_runtime_us_forces_runtime_disables() {
    let mut budget = default_task_budget();
    // 50 * per_detector_runtime_us(1000) = 50_000 us; this
    // admits exactly 50 detectors.
    budget.max_runtime_us = 50_000;
    let s = build_budgeted_activation_summary_with(budget, default_redundancy_clusters());
    assert!(s.plan.disabled_by_runtime_budget_count > 0);
    assert_eq!(
        s.plan.active_count as u64,
        s.plan
            .task_budget
            .max_runtime_us
            .saturating_div(s.plan.task_budget.per_detector_runtime_us)
    );
}

#[test]
fn s13d_max_memory_bytes_forces_memory_disables() {
    let mut budget = default_task_budget();
    // 50 * per_detector_memory_bytes(1 MiB) = 50 MiB
    budget.max_memory_bytes = 50 * 1024 * 1024;
    let s = build_budgeted_activation_summary_with(budget, default_redundancy_clusters());
    assert!(s.plan.disabled_by_memory_budget_count > 0);
}

#[test]
fn s13d_redundancy_cluster_suppresses_non_representatives() {
    let s =
        build_budgeted_activation_summary_with(default_task_budget(), S13D_TEST_CLUSTERS_SHEWHART);
    // 2 + 3 are suppressed; 1 is retained as representative.
    assert_eq!(s.plan.disabled_by_redundancy_count, 2);
    assert_eq!(s.redundancy_report.suppression_count, 2);
    assert_eq!(s.redundancy_report.retained_representatives, vec![1]);
    // Active count drops by the 2 suppressed.
    assert_eq!(s.plan.active_count, 152 - 2);
}

#[test]
fn s13d_redundancy_tie_break_picks_lowest_canonical_id() {
    let s = build_budgeted_activation_summary_with(
        default_task_budget(),
        S13D_TEST_CLUSTERS_LOWEST_PICKS_10,
    );
    assert_eq!(s.plan.tie_break_transcript.len(), 1);
    let entry = &s.plan.tie_break_transcript[0];
    assert_eq!(entry.selected_canonical_id, 10);
    assert_eq!(entry.suppressed_canonical_ids, vec![20, 30]);
}

#[test]
fn s13d_redundancy_representative_carries_representative_witness_reason() {
    let s =
        build_budgeted_activation_summary_with(default_task_budget(), S13D_TEST_CLUSTERS_REP_TEST);
    let rep = s
        .plan
        .decisions
        .iter()
        .find(|d| d.canonical_id == 1)
        .expect("decision 1 should be present");
    assert_eq!(rep.outcome, S13dOutcome::Active);
    assert_eq!(
        rep.reason_wire_name,
        S13dBudgetRetainReason::RetainedAsRepresentativeWitness.as_str()
    );
    assert_eq!(rep.redundancy_cluster_id, Some("rep_test"));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negatives (8)
// ---------------------------------------------------------------

#[test]
fn s13d_rejects_budget_plan_that_uses_ff3_rejected_record() {
    // Inject a decision for a canonical id NOT in the FF.3-
    // eligible set (e.g., 9999, which is outside the SEED range
    // and outside the ratified expansion index).
    let mut s = fresh_summary();
    let rogue_id: u32 = 9999;
    s.plan.decisions.push(S13dBudgetDecision {
        canonical_id: rogue_id,
        outcome: S13dOutcome::Active,
        outcome_wire_name: "Active",
        reason_wire_name: S13dBudgetRetainReason::RetainedAsBudgetSurvivor.as_str(),
        redundancy_cluster_id: None,
        cited_passport_hash: [0u8; 32],
    });
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::BudgetPlanThatUsesFf3RejectedRecord { canonical_id } if canonical_id == rogue_id
    )));
}

#[test]
fn s13d_rejects_silent_detector_drop_without_suppression_reason() {
    let mut s = fresh_summary();
    if let Some(d) = s.plan.decisions.first_mut() {
        d.outcome = S13dOutcome::Disabled;
        d.outcome_wire_name = "Disabled";
        d.reason_wire_name = ""; // silent drop
    }
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::SilentDetectorDropWithoutSuppressionReason { .. }
    )));
}

#[test]
fn s13d_rejects_redundancy_suppression_without_surviving_representative() {
    // Build a real plan, then mutate the transcript to add a
    // suppressed-member entry whose cluster_id has no Active
    // RepresentativeWitness decision in the list.
    let mut s = fresh_summary();
    s.plan.tie_break_transcript.push(
        dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::TieBreakTranscriptEntry {
            cluster_id: "orphan_cluster",
            selected_canonical_id: 1, // arbitrary low canonical id
            selection_rule: "lowest_canonical_id",
            suppressed_canonical_ids: vec![100, 200],
        },
    );
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::RedundancySuppressionWithoutSurvivingRepresentative { cluster_id }
        if cluster_id == "orphan_cluster"
    )));
}

#[test]
fn s13d_rejects_budget_overrun_without_reason_coded_pruning() {
    // Mutate: keep active_count high but lower the budget
    // ceiling so active_count > max; no DisabledByTaskBudget /
    // DisabledByGpuFamilyQuota present.
    let mut s = fresh_summary();
    s.plan.task_budget.max_active_detectors = 10;
    // active_count is 152 from baseline; no disable rows.
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::BudgetOverrunWithoutReasonCodedPruning { .. }
    )));
}

#[test]
fn s13d_rejects_nondeterministic_tie_break_between_equal_priority_detectors() {
    let mut s = fresh_summary();
    // Selected id is not the minimum of the cluster: classic
    // nondeterministic tie-break signature.
    s.plan.tie_break_transcript.push(
        dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::TieBreakTranscriptEntry {
            cluster_id: "bad_tie_break",
            selected_canonical_id: 30, // higher than suppressed
            selection_rule: "lowest_canonical_id",
            suppressed_canonical_ids: vec![10, 20],
        },
    );
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::NondeterministicTieBreakBetweenEqualPriorityDetectors { cluster_id }
        if cluster_id == "bad_tie_break"
    )));
}

#[test]
fn s13d_rejects_gpu_family_budget_without_declared_cost_model() {
    let mut budget = default_task_budget();
    budget.gpu_family_budgets = vec![GpuFamilyBudget {
        gpu_family_wire_name: "WindowStatisticFamily",
        max_active_count: 100,
        declared_cost_model: "", // empty cost model = R.6 fail
    }];
    let s = build_budgeted_activation_summary_with(budget, default_redundancy_clusters());
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::GpuFamilyBudgetWithoutDeclaredCostModel { gpu_family_wire_name }
        if gpu_family_wire_name == "WindowStatisticFamily"
    )));
}

#[test]
fn s13d_rejects_pruning_that_mutates_corpus_hash_v1_or_v2() {
    let mut s = fresh_summary();
    let mut bogus = s.plan.corpus_hash_v1;
    bogus[0] ^= 0xFF;
    s.plan.corpus_hash_v1 = bogus;
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::PruningThatMutatesCorpusHashV1OrV2 { anchor_wire_name }
        if anchor_wire_name == "corpus_hash_v1"
    )));
}

#[test]
fn s13d_rejects_pruning_that_mutates_corpus_hash_v2() {
    let mut s = fresh_summary();
    let mut bogus = s.plan.corpus_hash_v2;
    bogus[5] ^= 0xAA;
    s.plan.corpus_hash_v2 = bogus;
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::PruningThatMutatesCorpusHashV1OrV2 { anchor_wire_name }
        if anchor_wire_name == "corpus_hash_v2"
    )));
}

#[test]
fn s13d_rejects_schema_upgrade_side_effect_inside_budget_pruning() {
    let mut s = fresh_summary();
    s.plan.proposal_schema_upgrade_policy_hash_v1 = [0u8; 32]; // pretend FF.5 hash drifted
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::SchemaUpgradeSideEffectInsideBudgetPruning { .. }
    )));
}

// ---------------------------------------------------------------
// Structural defect rules (additional sensitivity)
// ---------------------------------------------------------------

#[test]
fn s13d_rejects_duplicate_decision_for_same_canonical_id() {
    let mut s = fresh_summary();
    if let Some(first) = s.plan.decisions.first().cloned() {
        s.plan.decisions.push(first);
    }
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::DuplicateDecisionForSameCanonicalId { .. }
    )));
}

#[test]
fn s13d_rejects_decisions_not_sorted_ascending() {
    let mut s = fresh_summary();
    if s.plan.decisions.len() >= 2 {
        s.plan.decisions.swap(0, 1);
    }
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13dVerifyErrorKind::DecisionsNotSortedAscending)));
}

#[test]
fn s13d_rejects_disabled_count_mismatch() {
    let mut s = fresh_summary();
    s.plan.disabled_count = 5; // doesn't match the per-reason sum (0)
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13dVerifyErrorKind::DisabledCountMismatch { .. })));
}

#[test]
fn s13d_rejects_redundancy_cluster_with_empty_member_set() {
    let s = build_budgeted_activation_summary_with(
        default_task_budget(),
        S13D_TEST_CLUSTERS_EMPTY_MEMBERS,
    );
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::RedundancyClusterWithEmptyMemberSet { cluster_id }
        if cluster_id == "empty_cluster"
    )));
}

#[test]
fn s13d_rejects_redundancy_cluster_with_empty_selection_rule() {
    let s = build_budgeted_activation_summary_with(
        default_task_budget(),
        S13D_TEST_CLUSTERS_EMPTY_RULE,
    );
    let report = build_consolidation_report();
    let ff3 = build_ff3_registry_generation_gate();
    let errors = verify_s1_3d(&s, &report, &ff3, live_ff5_policy_hash());
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13dVerifyErrorKind::RedundancyClusterWithEmptySelectionRule { cluster_id }
        if cluster_id == "empty_rule_cluster"
    )));
}

// ---------------------------------------------------------------
// Upstream anchor invariance witnesses
// ---------------------------------------------------------------

#[test]
fn s13d_does_not_alter_corpus_hash_v1() {
    let _ = build_budgeted_activation_summary();
    let v1 = compute_corpus_hash_v1().bytes;
    let expected_prefix: [u8; 4] = [0x35, 0xc2, 0x76, 0xc7];
    assert_eq!(&v1[..4], &expected_prefix);
}

#[test]
fn s13d_does_not_alter_corpus_hash_v2() {
    let _ = build_budgeted_activation_summary();
    let report = build_consolidation_report();
    let expected_prefix: [u8; 4] = [0xf1, 0xd1, 0x32, 0xeb];
    assert_eq!(&report.corpus_hash_v2[..4], &expected_prefix);
}

#[test]
fn s13d_does_not_alter_ff1_passport_index_hash() {
    let _ = build_budgeted_activation_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let expected_prefix: [u8; 4] = [0x1a, 0xd2, 0xdc, 0x2d];
    assert_eq!(
        &passport_index.ff1_passport_index_hash_v1[..4],
        &expected_prefix
    );
}

#[test]
fn s13d_does_not_alter_ff2_gate_hash() {
    let _ = build_budgeted_activation_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let ids = default_candidate_ids(&passport_index);
    let ff2 = build_ff2_activation_ratification_gate_from(&report, &passport_index, &ids);
    let expected_prefix: [u8; 4] = [0x05, 0xc1, 0xb5, 0x52];
    assert_eq!(
        &ff2.ff2_activation_ratification_gate_hash_v1[..4],
        &expected_prefix
    );
}

#[test]
fn s13d_does_not_alter_ff3_gate_hash() {
    let _ = build_budgeted_activation_summary();
    let ff3 = build_ff3_registry_generation_gate();
    let expected_prefix: [u8; 4] = [0x2f, 0xfd, 0x02, 0x22];
    assert_eq!(
        &ff3.ff3_registry_generation_gate_hash_v1[..4],
        &expected_prefix
    );
}

#[test]
fn s13d_does_not_alter_ff4_policy_hash() {
    let _ = build_budgeted_activation_summary();
    let ff4 = build_ff4_readme_authority_boundary_policy();
    let expected_prefix: [u8; 4] = [0x22, 0xb9, 0xdc, 0xb5];
    assert_eq!(
        &ff4.ff4_readme_authority_boundary_policy_hash_v1[..4],
        &expected_prefix
    );
}

#[test]
fn s13d_does_not_alter_ff5_policy_hash() {
    let _ = build_budgeted_activation_summary();
    let ff5 = build_proposal_schema_upgrade_policy();
    let expected_prefix: [u8; 4] = [0x94, 0xe0, 0x0a, 0xb1];
    assert_eq!(
        &ff5.proposal_schema_upgrade_policy_hash_v1[..4],
        &expected_prefix
    );
}

// ---------------------------------------------------------------
// Builder helpers + fully-specified surface
// ---------------------------------------------------------------

#[test]
fn s13d_build_from_full_args_matches_with_helper() {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let ids = default_candidate_ids(&passport_index);
    let ff2 = build_ff2_activation_ratification_gate_from(&report, &passport_index, &ids);
    let ff3 = build_ff3_registry_generation_gate();
    let ff4 = build_ff4_readme_authority_boundary_policy();
    let ff5 = build_proposal_schema_upgrade_policy();
    let full = build_budgeted_activation_summary_from(
        &report,
        &passport_index,
        &ff2,
        &ff3,
        ff4.ff4_readme_authority_boundary_policy_hash_v1,
        ff5.proposal_schema_upgrade_policy_hash_v1,
        default_task_budget(),
        default_redundancy_clusters(),
    );
    let helper = build_budgeted_activation_summary();
    assert_eq!(
        full.budgeted_activation_summary_hash_v1,
        helper.budgeted_activation_summary_hash_v1
    );
}

#[test]
fn s13d_default_task_budget_is_panel_permissive() {
    let b = default_task_budget();
    assert_eq!(b.max_active_detectors, 10_000);
    assert_eq!(b.max_runtime_us, u64::MAX);
    assert_eq!(b.max_memory_bytes, u64::MAX);
    assert!(b.gpu_family_budgets.is_empty());
    assert!(!b.reject_open_contraindications);
    assert!(!b.reject_open_coverage_holes);
}

#[test]
fn s13d_default_redundancy_clusters_is_empty() {
    assert!(default_redundancy_clusters().is_empty());
}

// ---------------------------------------------------------------
// Renderer-coverage spot checks
// ---------------------------------------------------------------

#[test]
fn s13d_plan_text_contains_pinned_anchors_section() {
    let s = render_s13d_plan_text(&fresh_summary().plan);
    assert!(s.contains("Pinned anchors"));
    assert!(s.contains("corpus_hash_v1"));
    assert!(s.contains("corpus_hash_v2"));
    assert!(s.contains("ff1_passport_index_hash_v1"));
    assert!(s.contains("ff2_activation_ratification_gate_hash_v1"));
    assert!(s.contains("ff3_registry_generation_gate_hash_v1"));
    assert!(s.contains("ff4_readme_authority_boundary_policy_hash_v1"));
    assert!(s.contains("proposal_schema_upgrade_policy_hash_v1"));
    assert!(s.contains("budget_pruning_plan_hash_v1"));
}

#[test]
fn s13d_summary_text_includes_redundancy_block() {
    let s = render_s13d_summary_text(&fresh_summary());
    assert!(s.contains("Redundancy"));
}

#[test]
fn s13d_summary_text_includes_active_count() {
    let s = render_s13d_summary_text(&fresh_summary());
    assert!(s.contains("active_count"));
    assert!(s.contains("152"));
}

#[test]
fn s13d_plan_json_parses_as_object() {
    let s = render_s13d_plan_json(&fresh_summary().plan);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13d_redundancy_json_parses_as_object() {
    let s = render_s13d_redundancy_json(&fresh_summary().redundancy_report);
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13d_summary_json_parses_as_object() {
    let s = render_s13d_summary_json(&fresh_summary());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}
