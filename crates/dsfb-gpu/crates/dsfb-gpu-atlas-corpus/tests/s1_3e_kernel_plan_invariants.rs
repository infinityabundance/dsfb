//! S1.3e acceptance suite — kernel plan + family schedule +
//! parameter table invariants.
//!
//! Eight panel-required load-bearing negatives pin the
//! discipline S1.3e exists to prove:
//!
//! * `s13e_rejects_kernel_plan_using_budget_disabled_detector`
//! * `s13e_rejects_kernel_plan_using_ff3_rejected_record`
//! * `s13e_rejects_kernel_plan_without_gpu_family_mapping`
//! * `s13e_rejects_parameter_table_without_stable_order`
//! * `s13e_rejects_family_schedule_without_declared_cost_model`
//! * `s13e_rejects_kernel_plan_that_mutates_activation_or_budget_hash`
//! * `s13e_rejects_cuda_execution_claim_inside_kernel_plan`
//! * `s13e_rejects_nondeterministic_tie_break_in_family_order`
//!
//! Panel-locked one-line verdict (verbatim):
//!
//! > S1.3d says who survives budgeted deployment; S1.3e says
//! > how the survivors are packed into deterministic GPU-
//! > family execution lanes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use dsfb_gpu_atlas_corpus::consolidate::build_consolidation_report;
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::ff1_passport_materialisation::build_ff1_passport_index_from;
use dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::{
    build_budgeted_activation_summary, build_budgeted_activation_summary_with,
    default_redundancy_clusters, default_task_budget,
};
use dsfb_gpu_atlas_corpus::s1_3e_kernel_plan::{
    build_kernel_family_schedule_v1, build_kernel_family_schedule_v1_from,
    build_kernel_parameter_table_v1, build_kernel_parameter_table_v1_from, build_kernel_plan_v1,
    build_kernel_plan_v1_from, forbidden_execution_claim_substrings,
    render_kernel_family_schedule_json, render_kernel_family_schedule_text,
    render_kernel_parameter_table_json, render_kernel_parameter_table_text,
    render_kernel_plan_json, render_kernel_plan_text, resolve_gpu_family_wire_name, verify_s1_3e,
    FamilyLane, KernelFamilyScheduleV1, KernelPlanV1, ParameterTableRow, S13eVerifyErrorKind,
    S13E_KERNEL_FAMILY_SCHEDULE_DOMAIN_V1, S13E_KERNEL_FAMILY_SCHEDULE_SCHEMA_V1,
    S13E_KERNEL_PARAMETER_TABLE_DOMAIN_V1, S13E_KERNEL_PARAMETER_TABLE_SCHEMA_V1,
    S13E_KERNEL_PLAN_DOMAIN_V1, S13E_KERNEL_PLAN_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::GpuFamilyKernel;

// ---------------------------------------------------------------
// Helpers — built once per test under the panel-locked default
// state so every assertion talks about the same anchors.
// ---------------------------------------------------------------

fn fresh_plan() -> KernelPlanV1 {
    build_kernel_plan_v1()
}

fn fresh_schedule() -> KernelFamilyScheduleV1 {
    build_kernel_family_schedule_v1()
}

fn fresh_summary() -> dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::BudgetedActivationSummary {
    build_budgeted_activation_summary()
}

fn verify_default() -> Vec<dsfb_gpu_atlas_corpus::s1_3e_kernel_plan::S13eVerifyError> {
    let plan = fresh_plan();
    let schedule = build_kernel_family_schedule_v1();
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index)
}

// ---------------------------------------------------------------
// Baseline state + pinned anchors
// ---------------------------------------------------------------

#[test]
fn s13e_default_plan_has_152_active_across_14_lanes() {
    let p = fresh_plan();
    assert_eq!(p.total_active_count, 152);
    assert_eq!(p.lane_count, 14);
}

#[test]
fn s13e_default_plan_passes_verifier() {
    let errors = verify_default();
    assert!(
        errors.is_empty(),
        "expected zero verifier errors at S1.3e baseline; got {errors:?}"
    );
}

#[test]
fn s13e_default_schedule_has_14_lanes_summing_to_152() {
    let s = fresh_schedule();
    assert_eq!(s.lanes.len(), 14);
    let total: u32 = s.lanes.iter().map(|l| l.active_detector_count).sum();
    assert_eq!(total, 152);
    assert_eq!(s.total_active_count, 152);
}

#[test]
fn s13e_seed_len_pinned_at_54() {
    assert_eq!(SEED.len(), 54);
}

#[test]
fn s13e_plan_pins_corpus_hash_v1_live_value() {
    let p = fresh_plan();
    assert_eq!(p.corpus_hash_v1, compute_corpus_hash_v1().bytes);
}

#[test]
fn s13e_plan_pins_corpus_hash_v2_from_consolidation_report() {
    let p = fresh_plan();
    let report = build_consolidation_report();
    assert_eq!(p.corpus_hash_v2, report.corpus_hash_v2);
}

#[test]
fn s13e_plan_pins_s1_3d_budget_pruning_plan_hash() {
    let p = fresh_plan();
    let summary = fresh_summary();
    assert_eq!(
        p.budget_pruning_plan_hash_v1,
        summary.plan.budget_pruning_plan_hash_v1
    );
}

#[test]
fn s13e_plan_pins_s1_3d_budgeted_activation_summary_hash() {
    let p = fresh_plan();
    let summary = fresh_summary();
    assert_eq!(
        p.budgeted_activation_summary_hash_v1,
        summary.budgeted_activation_summary_hash_v1
    );
}

#[test]
fn s13e_plan_pins_ff2_gate_hash() {
    let p = fresh_plan();
    let summary = fresh_summary();
    assert_eq!(
        p.ff2_activation_ratification_gate_hash_v1,
        summary.plan.ff2_activation_ratification_gate_hash_v1
    );
}

#[test]
fn s13e_plan_pins_ff3_gate_hash() {
    let p = fresh_plan();
    let summary = fresh_summary();
    assert_eq!(
        p.ff3_registry_generation_gate_hash_v1,
        summary.plan.ff3_registry_generation_gate_hash_v1
    );
}

#[test]
fn s13e_plan_pins_seed_len_54() {
    assert_eq!(fresh_plan().seed_len, 54);
}

// ---------------------------------------------------------------
// Determinism + sensitivity invariants
// ---------------------------------------------------------------

#[test]
fn s13e_plan_hash_is_deterministic_across_two_builds() {
    let a = build_kernel_plan_v1().kernel_plan_hash_v1;
    let b = build_kernel_plan_v1().kernel_plan_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13e_schedule_hash_is_deterministic_across_two_builds() {
    let a = build_kernel_family_schedule_v1().kernel_family_schedule_hash_v1;
    let b = build_kernel_family_schedule_v1().kernel_family_schedule_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13e_parameter_table_hash_is_deterministic_across_two_builds() {
    let a = build_kernel_parameter_table_v1().kernel_parameter_table_hash_v1;
    let b = build_kernel_parameter_table_v1().kernel_parameter_table_hash_v1;
    assert_eq!(a, b);
}

#[test]
fn s13e_plan_text_is_byte_stable_across_two_renders() {
    let a = render_kernel_plan_text(&fresh_plan());
    let b = render_kernel_plan_text(&fresh_plan());
    assert_eq!(a, b);
}

#[test]
fn s13e_plan_json_is_byte_stable_across_two_renders() {
    let a = render_kernel_plan_json(&fresh_plan());
    let b = render_kernel_plan_json(&fresh_plan());
    assert_eq!(a, b);
}

#[test]
fn s13e_schedule_text_is_byte_stable_across_two_renders() {
    let a = render_kernel_family_schedule_text(&fresh_schedule());
    let b = render_kernel_family_schedule_text(&fresh_schedule());
    assert_eq!(a, b);
}

#[test]
fn s13e_schedule_json_is_byte_stable_across_two_renders() {
    let a = render_kernel_family_schedule_json(&fresh_schedule());
    let b = render_kernel_family_schedule_json(&fresh_schedule());
    assert_eq!(a, b);
}

#[test]
fn s13e_parameter_table_text_is_byte_stable_across_two_renders() {
    let a = render_kernel_parameter_table_text(&build_kernel_parameter_table_v1());
    let b = render_kernel_parameter_table_text(&build_kernel_parameter_table_v1());
    assert_eq!(a, b);
}

#[test]
fn s13e_parameter_table_json_is_byte_stable_across_two_renders() {
    let a = render_kernel_parameter_table_json(&build_kernel_parameter_table_v1());
    let b = render_kernel_parameter_table_json(&build_kernel_parameter_table_v1());
    assert_eq!(a, b);
}

#[test]
fn s13e_plan_hash_changes_when_budget_pressure_changes_active_set() {
    let base = build_kernel_plan_v1().kernel_plan_hash_v1;
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let pressed = build_kernel_plan_v1_from(&summary).kernel_plan_hash_v1;
    assert_ne!(base, pressed);
}

#[test]
fn s13e_schedule_hash_changes_when_budget_pressure_changes_active_set() {
    let base = build_kernel_family_schedule_v1().kernel_family_schedule_hash_v1;
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let pressed = build_kernel_family_schedule_v1_from(&summary).kernel_family_schedule_hash_v1;
    assert_ne!(base, pressed);
}

#[test]
fn s13e_parameter_table_hash_changes_when_budget_pressure_changes_active_set() {
    let base = build_kernel_parameter_table_v1().kernel_parameter_table_hash_v1;
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let pressed = build_kernel_parameter_table_v1_from(&summary).kernel_parameter_table_hash_v1;
    assert_ne!(base, pressed);
}

// ---------------------------------------------------------------
// Domain-separator + schema-id pins
// ---------------------------------------------------------------

#[test]
fn s13e_plan_domain_separator_is_pinned() {
    assert_eq!(
        S13E_KERNEL_PLAN_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1\0"
    );
}

#[test]
fn s13e_plan_schema_id_is_pinned() {
    assert_eq!(
        S13E_KERNEL_PLAN_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-PLAN:v1"
    );
}

#[test]
fn s13e_schedule_domain_separator_is_pinned() {
    assert_eq!(
        S13E_KERNEL_FAMILY_SCHEDULE_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1\0"
    );
}

#[test]
fn s13e_schedule_schema_id_is_pinned() {
    assert_eq!(
        S13E_KERNEL_FAMILY_SCHEDULE_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-FAMILY-SCHEDULE:v1"
    );
}

#[test]
fn s13e_parameter_table_domain_separator_is_pinned() {
    assert_eq!(
        S13E_KERNEL_PARAMETER_TABLE_DOMAIN_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1\0"
    );
}

#[test]
fn s13e_parameter_table_schema_id_is_pinned() {
    assert_eq!(
        S13E_KERNEL_PARAMETER_TABLE_SCHEMA_V1,
        "DSFB-GPU-ATLAS:S13E-KERNEL-PARAMETER-TABLE:v1"
    );
}

#[test]
fn s13e_three_hash_namespaces_are_distinct() {
    let p = fresh_plan();
    assert_ne!(p.kernel_plan_hash_v1, p.kernel_family_schedule_hash_v1);
    assert_ne!(p.kernel_plan_hash_v1, p.kernel_parameter_table_hash_v1);
    assert_ne!(
        p.kernel_family_schedule_hash_v1,
        p.kernel_parameter_table_hash_v1
    );
}

// ---------------------------------------------------------------
// Structural invariants (schedule sorted, lanes sorted, etc.)
// ---------------------------------------------------------------

#[test]
fn s13e_schedule_lanes_are_sorted_ascending_by_family_wire_name() {
    let s = fresh_schedule();
    for w in s.lanes.windows(2) {
        assert!(
            w[0].gpu_family_wire_name <= w[1].gpu_family_wire_name,
            "lanes must be sorted ascending; saw `{}` then `{}`",
            w[0].gpu_family_wire_name,
            w[1].gpu_family_wire_name
        );
    }
}

#[test]
fn s13e_every_lane_has_non_empty_cost_model_and_kernel_name() {
    let s = fresh_schedule();
    for lane in &s.lanes {
        assert!(!lane.declared_cost_model.is_empty());
        assert!(!lane.expected_kernel_name.is_empty());
    }
}

#[test]
fn s13e_every_lane_carries_canonical_ids_sorted_ascending() {
    let s = fresh_schedule();
    for lane in &s.lanes {
        for w in lane.active_canonical_ids.windows(2) {
            assert!(
                w[0] < w[1],
                "duplicate or unsorted ids in lane {}",
                lane.gpu_family_wire_name
            );
        }
    }
}

#[test]
fn s13e_parameter_table_rows_match_schedule_lane_total() {
    let s = fresh_schedule();
    let total: usize = s.lanes.iter().map(|l| l.active_canonical_ids.len()).sum();
    let table = build_kernel_parameter_table_v1();
    assert_eq!(table.rows.len(), total);
}

#[test]
fn s13e_parameter_table_rows_sorted_by_family_then_canonical_id() {
    let table = build_kernel_parameter_table_v1();
    for w in table.rows.windows(2) {
        let a = (w[0].gpu_family_wire_name, w[0].canonical_id);
        let b = (w[1].gpu_family_wire_name, w[1].canonical_id);
        assert!(a < b, "rows must be sorted ascending");
    }
}

#[test]
fn s13e_aggregate_cost_us_matches_count_times_per_detector_cost() {
    let s = fresh_schedule();
    let summary = fresh_summary();
    for lane in &s.lanes {
        let expected = u64::from(lane.active_detector_count)
            * summary.plan.task_budget.per_detector_runtime_us;
        assert_eq!(lane.aggregate_cost_us, expected);
    }
}

#[test]
fn s13e_plan_total_aggregate_cost_matches_sum_of_lane_costs() {
    let p = fresh_plan();
    let s = fresh_schedule();
    let sum: u64 = s.lanes.iter().map(|l| l.aggregate_cost_us).sum();
    assert_eq!(p.total_aggregate_cost_us, sum);
}

// ---------------------------------------------------------------
// GPU-family-mapping invariants
// ---------------------------------------------------------------

#[test]
fn s13e_resolves_gpu_family_for_every_active_seed_id() {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    for id in 1..=54_u32 {
        assert!(
            resolve_gpu_family_wire_name(id, &passport_index).is_some(),
            "SEED id {id} has no GPU family mapping"
        );
    }
}

#[test]
fn s13e_resolves_gpu_family_for_every_active_ratified_id() {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    for p in &passport_index.passports {
        assert!(
            resolve_gpu_family_wire_name(p.canonical_id, &passport_index).is_some(),
            "ratified id {} has no GPU family mapping",
            p.canonical_id
        );
    }
}

#[test]
fn s13e_unknown_canonical_id_resolves_to_none() {
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    assert!(resolve_gpu_family_wire_name(99_999, &passport_index).is_none());
}

#[test]
fn s13e_baseline_lanes_contain_no_unknown_family_strings() {
    let s = fresh_schedule();
    let canonical: std::collections::BTreeSet<&'static str> = [
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
    .collect();
    for lane in &s.lanes {
        assert!(
            canonical.contains(lane.gpu_family_wire_name),
            "lane wire name `{}` is not a canonical GpuFamilyKernel wire name",
            lane.gpu_family_wire_name
        );
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negatives (8)
// ---------------------------------------------------------------

#[test]
fn s13e_rejects_kernel_plan_using_budget_disabled_detector() {
    // Inject a lane that names a canonical id S1.3d disabled
    // (we use a tight budget so id 60 is task-budget-disabled,
    // then add it to a lane in the schedule).
    let mut tight = default_task_budget();
    tight.max_active_detectors = 30;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let plan = build_kernel_plan_v1_from(&summary);
    let mut schedule = build_kernel_family_schedule_v1_from(&summary);
    // Find a disabled id (any candidate beyond the 30-active
    // cutoff is disabled).
    let disabled_id: u32 = summary
        .plan
        .decisions
        .iter()
        .find(|d| d.outcome == dsfb_gpu_atlas_corpus::s1_3d_budget_pruning::S13dOutcome::Disabled)
        .map(|d| d.canonical_id)
        .expect("expected at least one disabled decision");
    // Append it to the first lane (any lane works).
    schedule.lanes[0].active_canonical_ids.push(disabled_id);
    let table = build_kernel_parameter_table_v1_from(&summary);
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::KernelPlanUsingBudgetDisabledDetector { canonical_id }
        if canonical_id == disabled_id
    )));
}

#[test]
fn s13e_rejects_kernel_plan_using_ff3_rejected_record() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    let summary = fresh_summary();
    // 99_999 is outside the FF.3-eligible set.
    let rogue_id: u32 = 99_999;
    schedule.lanes[0].active_canonical_ids.push(rogue_id);
    let table = build_kernel_parameter_table_v1();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::KernelPlanUsingFf3RejectedRecord { canonical_id } if canonical_id == rogue_id
    )));
}

#[test]
fn s13e_rejects_kernel_plan_without_gpu_family_mapping() {
    // Mutate the passport index by emptying it; every
    // ratified id then resolves to None for the resolver.
    let plan = fresh_plan();
    let schedule = fresh_schedule();
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let mut passport_index = build_ff1_passport_index_from(&report);
    passport_index.passports = Vec::new();
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::KernelPlanWithoutGpuFamilyMapping { .. }
    )));
}

#[test]
fn s13e_rejects_parameter_table_without_stable_order() {
    let plan = fresh_plan();
    let schedule = fresh_schedule();
    let mut table = build_kernel_parameter_table_v1();
    // Swap two adjacent rows to violate sort order.
    if table.rows.len() >= 2 {
        table.rows.swap(0, 1);
    }
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::ParameterTableWithoutStableOrder
    )));
}

#[test]
fn s13e_rejects_family_schedule_without_declared_cost_model() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    schedule.lanes[0].declared_cost_model = "";
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::FamilyScheduleWithoutDeclaredCostModel { .. }
    )));
}

#[test]
fn s13e_rejects_kernel_plan_that_mutates_activation_or_budget_hash() {
    let mut plan = fresh_plan();
    plan.budget_pruning_plan_hash_v1[0] ^= 0xFF;
    let schedule = fresh_schedule();
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::KernelPlanThatMutatesActivationOrBudgetHash { anchor_wire_name }
        if anchor_wire_name == "budget_pruning_plan_hash_v1"
    )));
}

#[test]
fn s13e_rejects_cuda_execution_claim_inside_kernel_plan() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    // Inject a forbidden execution-claim substring into a lane.
    schedule.lanes[0].declared_cost_model = "kernel launch occurs every cell";
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::CudaExecutionClaimInsideKernelPlan { .. }
    )));
}

#[test]
fn s13e_rejects_nondeterministic_tie_break_in_family_order() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    schedule.lanes.swap(0, 1);
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::NondeterministicTieBreakInFamilyOrder
    )));
}

// ---------------------------------------------------------------
// Forbidden-substring scanner — boundary tests
// ---------------------------------------------------------------

#[test]
fn s13e_cost_model_table_carries_no_forbidden_substring() {
    let schedule = fresh_schedule();
    let forbidden = forbidden_execution_claim_substrings();
    for lane in &schedule.lanes {
        for &needle in forbidden {
            assert!(
                !lane
                    .declared_cost_model
                    .to_ascii_lowercase()
                    .contains(needle),
                "lane `{}` cost model carries forbidden substring `{}`",
                lane.gpu_family_wire_name,
                needle
            );
            assert!(
                !lane
                    .expected_kernel_name
                    .to_ascii_lowercase()
                    .contains(needle),
                "lane `{}` kernel name carries forbidden substring `{}`",
                lane.gpu_family_wire_name,
                needle
            );
        }
    }
}

#[test]
fn s13e_forbidden_substring_set_is_non_empty_and_lowercase() {
    let forbidden = forbidden_execution_claim_substrings();
    assert!(!forbidden.is_empty());
    for &needle in forbidden {
        assert_eq!(
            needle,
            needle.to_ascii_lowercase(),
            "forbidden substrings should be lowercase to match the case-insensitive scanner"
        );
    }
}

// ---------------------------------------------------------------
// Structural defect rules (additional sensitivity)
// ---------------------------------------------------------------

#[test]
fn s13e_rejects_duplicate_family_lane() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    let dup = schedule.lanes[0].clone();
    schedule.lanes.push(FamilyLane {
        gpu_family_wire_name: dup.gpu_family_wire_name,
        active_canonical_ids: dup.active_canonical_ids.clone(),
        active_detector_count: dup.active_detector_count,
        declared_cost_model: dup.declared_cost_model,
        expected_kernel_name: dup.expected_kernel_name,
        aggregate_cost_us: dup.aggregate_cost_us,
    });
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13eVerifyErrorKind::DuplicateFamilyLane { .. })));
}

#[test]
fn s13e_rejects_lane_active_count_mismatch() {
    let plan = fresh_plan();
    let mut schedule = fresh_schedule();
    schedule.lanes[0].active_detector_count = 999;
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13eVerifyErrorKind::LaneActiveCountMismatch { .. })));
}

#[test]
fn s13e_rejects_total_active_count_mismatch() {
    let mut plan = fresh_plan();
    plan.total_active_count = 9999;
    let schedule = fresh_schedule();
    let table = build_kernel_parameter_table_v1();
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, S13eVerifyErrorKind::TotalActiveCountMismatch { .. })));
}

#[test]
fn s13e_rejects_parameter_table_row_references_unknown_canonical_id() {
    let plan = fresh_plan();
    let schedule = fresh_schedule();
    let mut table = build_kernel_parameter_table_v1();
    let lane_family = schedule.lanes[0].gpu_family_wire_name;
    table.rows.push(ParameterTableRow {
        canonical_id: 99_999,
        gpu_family_wire_name: lane_family,
        lane_offset: 0,
        per_detector_runtime_us: 1_000,
    });
    let summary = fresh_summary();
    let report = build_consolidation_report();
    let passport_index = build_ff1_passport_index_from(&report);
    let errors = verify_s1_3e(&plan, &schedule, &table, &summary, &passport_index);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        S13eVerifyErrorKind::ParameterTableRowReferencesUnknownCanonicalId { canonical_id }
        if canonical_id == 99_999
    )));
}

// ---------------------------------------------------------------
// Upstream anchor invariance witnesses
// ---------------------------------------------------------------

#[test]
fn s13e_does_not_alter_corpus_hash_v1() {
    let _ = build_kernel_plan_v1();
    let v1 = compute_corpus_hash_v1().bytes;
    let prefix: [u8; 4] = [0x35, 0xc2, 0x76, 0xc7];
    assert_eq!(&v1[..4], &prefix);
}

#[test]
fn s13e_does_not_alter_corpus_hash_v2() {
    let _ = build_kernel_plan_v1();
    let report = build_consolidation_report();
    let prefix: [u8; 4] = [0xf1, 0xd1, 0x32, 0xeb];
    assert_eq!(&report.corpus_hash_v2[..4], &prefix);
}

#[test]
fn s13e_does_not_alter_s1_3d_budget_pruning_plan_hash() {
    let _ = build_kernel_plan_v1();
    let s = build_budgeted_activation_summary();
    let prefix: [u8; 4] = [0x82, 0xbe, 0x22, 0x89];
    assert_eq!(&s.plan.budget_pruning_plan_hash_v1[..4], &prefix);
}

#[test]
fn s13e_does_not_alter_s1_3d_budgeted_activation_summary_hash() {
    let _ = build_kernel_plan_v1();
    let s = build_budgeted_activation_summary();
    let prefix: [u8; 4] = [0x5f, 0xea, 0xb2, 0x38];
    assert_eq!(&s.budgeted_activation_summary_hash_v1[..4], &prefix);
}

// ---------------------------------------------------------------
// Pressure-bearing scenarios
// ---------------------------------------------------------------

#[test]
fn s13e_max_active_detectors_50_produces_50_active_in_plan() {
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let plan = build_kernel_plan_v1_from(&summary);
    assert_eq!(plan.total_active_count, 50);
}

#[test]
fn s13e_max_active_detectors_50_lane_total_equals_50() {
    let mut tight = default_task_budget();
    tight.max_active_detectors = 50;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let schedule = build_kernel_family_schedule_v1_from(&summary);
    let total: u32 = schedule.lanes.iter().map(|l| l.active_detector_count).sum();
    assert_eq!(total, 50);
}

#[test]
fn s13e_max_active_detectors_zero_produces_empty_schedule() {
    let mut tight = default_task_budget();
    tight.max_active_detectors = 0;
    let summary = build_budgeted_activation_summary_with(tight, default_redundancy_clusters());
    let plan = build_kernel_plan_v1_from(&summary);
    assert_eq!(plan.total_active_count, 0);
    assert_eq!(plan.lane_count, 0);
}

// ---------------------------------------------------------------
// Renderer-coverage spot checks
// ---------------------------------------------------------------

#[test]
fn s13e_plan_text_contains_pinned_anchors_section() {
    let s = render_kernel_plan_text(&fresh_plan());
    assert!(s.contains("Pinned anchors"));
    assert!(s.contains("corpus_hash_v1"));
    assert!(s.contains("budget_pruning_plan_hash_v1"));
    assert!(s.contains("budgeted_activation_summary_hash_v1"));
    assert!(s.contains("kernel_plan_hash_v1"));
}

#[test]
fn s13e_schedule_text_lists_every_lane() {
    let s = render_kernel_family_schedule_text(&fresh_schedule());
    let lanes = fresh_schedule().lanes;
    for lane in &lanes {
        assert!(
            s.contains(lane.gpu_family_wire_name),
            "schedule text missing lane `{}`",
            lane.gpu_family_wire_name
        );
    }
}

#[test]
fn s13e_plan_json_parses_as_object() {
    let s = render_kernel_plan_json(&fresh_plan());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13e_schedule_json_parses_as_object() {
    let s = render_kernel_family_schedule_json(&fresh_schedule());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}

#[test]
fn s13e_parameter_table_json_parses_as_object() {
    let s = render_kernel_parameter_table_json(&build_kernel_parameter_table_v1());
    assert!(s.trim_start().starts_with('{'));
    assert!(s.trim_end().ends_with('}'));
}
