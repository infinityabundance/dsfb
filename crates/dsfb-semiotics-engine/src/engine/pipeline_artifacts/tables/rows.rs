//! Row-type and row-construction helpers for tabular artifact exports.

use super::*;

#[derive(Clone, Debug, Serialize)]
pub(super) struct TimeSeriesCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    observed_ch1: Option<f64>,
    observed_ch2: Option<f64>,
    observed_ch3: Option<f64>,
    observed_ch4: Option<f64>,
    predicted_ch1: Option<f64>,
    predicted_ch2: Option<f64>,
    predicted_ch3: Option<f64>,
    predicted_ch4: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct VectorNormCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    ch1: Option<f64>,
    ch2: Option<f64>,
    ch3: Option<f64>,
    ch4: Option<f64>,
    norm: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct SignCsvRow {
    scenario_id: String,
    step: usize,
    time: f64,
    residual_ch1: Option<f64>,
    residual_ch2: Option<f64>,
    residual_ch3: Option<f64>,
    residual_ch4: Option<f64>,
    drift_ch1: Option<f64>,
    drift_ch2: Option<f64>,
    drift_ch3: Option<f64>,
    drift_ch4: Option<f64>,
    slew_ch1: Option<f64>,
    slew_ch2: Option<f64>,
    slew_ch3: Option<f64>,
    slew_ch4: Option<f64>,
    residual_norm: f64,
    drift_norm: f64,
    slew_norm: f64,
    projection_1: f64,
    projection_2: f64,
    projection_3: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct SemanticMatchCsvRow {
    scenario_id: String,
    disposition: String,
    motif_summary: String,
    heuristic_bank_entry_count: usize,
    heuristic_candidates_post_admissibility: usize,
    heuristic_candidates_post_regime: usize,
    heuristic_candidates_pre_scope: usize,
    heuristic_candidates_post_scope: usize,
    heuristics_rejected_by_admissibility: usize,
    heuristics_rejected_by_regime: usize,
    heuristics_rejected_by_scope: usize,
    heuristics_selected_final: usize,
    retrieval_path: String,
    prefilter_candidate_count: usize,
    prefilter_candidate_ids: String,
    index_buckets_considered: usize,
    candidate_ids_post_admissibility: String,
    candidate_ids_post_regime: String,
    candidate_ids_post_scope: String,
    rejected_by_admissibility_ids: String,
    rejected_by_regime_ids: String,
    rejected_by_scope_ids: String,
    selected_labels: String,
    selected_heuristic_ids: String,
    resolution_basis: String,
    unknown_reason_class: String,
    unknown_reason_detail: String,
    candidate_labels: String,
    candidate_regimes: String,
    candidate_regime_explanations: String,
    candidate_admissibility: String,
    candidate_scope: String,
    candidate_metric_highlights: String,
    candidate_applicability_notes: String,
    candidate_provenance_notes: String,
    candidate_rationales: String,
    compatibility_note: String,
    compatibility_reasons: String,
    conflict_notes: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ReproducibilityCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    scenario_id: String,
    first_hash: String,
    second_hash: String,
    identical: bool,
    materialized_components: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct EvaluationSummaryCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    evaluation_version: String,
    input_mode: String,
    scenario_count: usize,
    semantic_disposition_counts: String,
    syntax_label_counts: String,
    boundary_interaction_count: usize,
    violation_count: usize,
    comparator_trigger_counts: String,
    reproducible_scenario_count: usize,
    all_reproducible: bool,
    minimum_trust_scalar: f64,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ScenarioEvaluationCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    scenario_id: String,
    input_mode: String,
    syntax_label: String,
    semantic_disposition: String,
    selected_heuristic_ids: String,
    grammar_reason_code: String,
    grammar_reason_text: String,
    trust_scalar: f64,
    heuristic_bank_entry_count: usize,
    heuristic_candidates_post_admissibility: usize,
    heuristic_candidates_post_regime: usize,
    heuristic_candidates_pre_scope: usize,
    heuristic_candidates_post_scope: usize,
    heuristics_rejected_by_admissibility: usize,
    heuristics_rejected_by_regime: usize,
    heuristics_rejected_by_scope: usize,
    heuristics_selected_final: usize,
    boundary_sample_count: usize,
    violation_sample_count: usize,
    first_boundary_time: Option<f64>,
    first_violation_time: Option<f64>,
    reproducible: bool,
    triggered_baseline_count: usize,
    unknown_reason_class: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct BankValidationCsvRow {
    schema_version: String,
    engine_version: String,
    bank_schema_version: String,
    bank_version: String,
    bank_source_kind: String,
    bank_source_path: String,
    bank_content_hash: String,
    strict_validation: bool,
    validation_mode: String,
    entry_count: usize,
    valid: bool,
    duplicate_ids: String,
    self_link_notes: String,
    compatibility_conflicts: String,
    missing_compatibility_links: String,
    missing_incompatibility_links: String,
    strict_validation_errors: String,
    unknown_link_targets: String,
    provenance_gaps: String,
    regime_tag_notes: String,
    retrieval_priority_notes: String,
    scope_sanity_notes: String,
    violations: String,
    warnings: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct SweepPointCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    sweep_family: String,
    scenario_id: String,
    parameter_name: String,
    parameter_value: f64,
    secondary_parameter_name: String,
    secondary_parameter_value: Option<f64>,
    syntax_label: String,
    semantic_disposition: String,
    selected_heuristic_ids: String,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct SweepSummaryCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    sweep_family: String,
    member_count: usize,
    unique_syntax_labels: String,
    unique_semantic_dispositions: String,
    unknown_count: usize,
    ambiguous_count: usize,
    disposition_flip_count: usize,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct ComparatorResultsCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    scenario_id: String,
    comparator_id: String,
    comparator_name: String,
    comparator_label: String,
    alarm: bool,
    first_alarm_step: Option<usize>,
    first_alarm_time: Option<f64>,
    config_reference: String,
    comparator_summary: String,
    distinction_note: String,
}

pub(super) fn time_series_row(
    scenario_id: &str,
    observed: &crate::engine::types::VectorSample,
    predicted: &crate::engine::types::VectorSample,
) -> TimeSeriesCsvRow {
    TimeSeriesCsvRow {
        scenario_id: scenario_id.to_string(),
        step: observed.step,
        time: observed.time,
        observed_ch1: value_at(&observed.values, 0),
        observed_ch2: value_at(&observed.values, 1),
        observed_ch3: value_at(&observed.values, 2),
        observed_ch4: value_at(&observed.values, 3),
        predicted_ch1: value_at(&predicted.values, 0),
        predicted_ch2: value_at(&predicted.values, 1),
        predicted_ch3: value_at(&predicted.values, 2),
        predicted_ch4: value_at(&predicted.values, 3),
    }
}

pub(super) fn vector_norm_row(
    scenario_id: &str,
    step: usize,
    time: f64,
    values: &[f64],
    norm: f64,
) -> VectorNormCsvRow {
    VectorNormCsvRow {
        scenario_id: scenario_id.to_string(),
        step,
        time,
        ch1: value_at(values, 0),
        ch2: value_at(values, 1),
        ch3: value_at(values, 2),
        ch4: value_at(values, 3),
        norm,
    }
}

pub(super) fn sign_csv_row(
    scenario_id: &str,
    sample: &crate::engine::types::SignSample,
) -> SignCsvRow {
    SignCsvRow {
        scenario_id: scenario_id.to_string(),
        step: sample.step,
        time: sample.time,
        residual_ch1: value_at(&sample.residual, 0),
        residual_ch2: value_at(&sample.residual, 1),
        residual_ch3: value_at(&sample.residual, 2),
        residual_ch4: value_at(&sample.residual, 3),
        drift_ch1: value_at(&sample.drift, 0),
        drift_ch2: value_at(&sample.drift, 1),
        drift_ch3: value_at(&sample.drift, 2),
        drift_ch4: value_at(&sample.drift, 3),
        slew_ch1: value_at(&sample.slew, 0),
        slew_ch2: value_at(&sample.slew, 1),
        slew_ch3: value_at(&sample.slew, 2),
        slew_ch4: value_at(&sample.slew, 3),
        residual_norm: sample.residual_norm,
        drift_norm: sample.drift_norm,
        slew_norm: sample.slew_norm,
        projection_1: sample.projection[0],
        projection_2: sample.projection[1],
        projection_3: sample.projection[2],
    }
}

pub(super) fn semantic_csv_row(
    result: &crate::engine::types::SemanticMatchResult,
) -> SemanticMatchCsvRow {
    SemanticMatchCsvRow {
        scenario_id: result.scenario_id.clone(),
        disposition: format!("{:?}", result.disposition),
        motif_summary: result.motif_summary.clone(),
        heuristic_bank_entry_count: result.retrieval_audit.heuristic_bank_entry_count,
        heuristic_candidates_post_admissibility: result
            .retrieval_audit
            .heuristic_candidates_post_admissibility,
        heuristic_candidates_post_regime: result.retrieval_audit.heuristic_candidates_post_regime,
        heuristic_candidates_pre_scope: result.retrieval_audit.heuristic_candidates_pre_scope,
        heuristic_candidates_post_scope: result.retrieval_audit.heuristic_candidates_post_scope,
        heuristics_rejected_by_admissibility: result
            .retrieval_audit
            .heuristics_rejected_by_admissibility,
        heuristics_rejected_by_regime: result.retrieval_audit.heuristics_rejected_by_regime,
        heuristics_rejected_by_scope: result.retrieval_audit.heuristics_rejected_by_scope,
        heuristics_selected_final: result.retrieval_audit.heuristics_selected_final,
        retrieval_path: result.retrieval_audit.retrieval_path.clone(),
        prefilter_candidate_count: result.retrieval_audit.prefilter_candidate_count,
        prefilter_candidate_ids: result.retrieval_audit.prefilter_candidate_ids.join(" | "),
        index_buckets_considered: result.retrieval_audit.index_buckets_considered,
        candidate_ids_post_admissibility: result
            .retrieval_audit
            .candidate_ids_post_admissibility
            .join(" | "),
        candidate_ids_post_regime: result.retrieval_audit.candidate_ids_post_regime.join(" | "),
        candidate_ids_post_scope: result.retrieval_audit.candidate_ids_post_scope.join(" | "),
        rejected_by_admissibility_ids: result
            .retrieval_audit
            .rejected_by_admissibility_ids
            .join(" | "),
        rejected_by_regime_ids: result.retrieval_audit.rejected_by_regime_ids.join(" | "),
        rejected_by_scope_ids: result.retrieval_audit.rejected_by_scope_ids.join(" | "),
        selected_labels: result.selected_labels.join(" | "),
        selected_heuristic_ids: result.selected_heuristic_ids.join(" | "),
        resolution_basis: result.resolution_basis.clone(),
        unknown_reason_class: result.unknown_reason_class.clone().unwrap_or_default(),
        unknown_reason_detail: result.unknown_reason_detail.clone().unwrap_or_default(),
        candidate_labels: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.motif_label,
                    format_metric(candidate.score)
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
        candidate_regimes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id,
                    if candidate.matched_regimes.is_empty() {
                        "none".to_string()
                    } else {
                        candidate.matched_regimes.join("|")
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(" | "),
        candidate_regime_explanations: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.regime_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_admissibility: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.admissibility_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_scope: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.scope_explanation
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_metric_highlights: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id,
                    candidate.metric_highlights.join("; ")
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_applicability_notes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.entry.applicability_note
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_provenance_notes: result
            .candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}",
                    candidate.entry.heuristic_id, candidate.entry.provenance.note
                )
            })
            .collect::<Vec<_>>()
            .join(" || "),
        candidate_rationales: result
            .candidates
            .iter()
            .map(|candidate| format!("{}:{}", candidate.entry.heuristic_id, candidate.rationale))
            .collect::<Vec<_>>()
            .join(" || "),
        compatibility_note: result.compatibility_note.clone(),
        compatibility_reasons: result.compatibility_reasons.join(" | "),
        conflict_notes: result.conflict_notes.join(" | "),
        note: result.note.clone(),
    }
}

pub(super) fn reproducibility_csv_row(
    bundle: &EngineOutputBundle,
    check: &crate::engine::types::ReproducibilityCheck,
) -> ReproducibilityCsvRow {
    ReproducibilityCsvRow {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        scenario_id: check.scenario_id.clone(),
        first_hash: check.first_hash.clone(),
        second_hash: check.second_hash.clone(),
        identical: check.identical,
        materialized_components: check.materialized_components.join(" | "),
        note: check.note.clone(),
    }
}

pub(super) fn evaluation_summary_csv_row(
    summary: &crate::evaluation::types::RunEvaluationSummary,
) -> EvaluationSummaryCsvRow {
    EvaluationSummaryCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        evaluation_version: summary.evaluation_version.clone(),
        input_mode: summary.input_mode.clone(),
        scenario_count: summary.scenario_count,
        semantic_disposition_counts: join_count_map(&summary.semantic_disposition_counts),
        syntax_label_counts: join_count_map(&summary.syntax_label_counts),
        boundary_interaction_count: summary.boundary_interaction_count,
        violation_count: summary.violation_count,
        comparator_trigger_counts: join_count_map(&summary.comparator_trigger_counts),
        reproducible_scenario_count: summary.reproducible_scenario_count,
        all_reproducible: summary.all_reproducible,
        minimum_trust_scalar: summary.minimum_trust_scalar,
        note: summary.note.clone(),
    }
}

pub(super) fn scenario_evaluation_csv_row(
    summary: &crate::evaluation::types::ScenarioEvaluationSummary,
) -> ScenarioEvaluationCsvRow {
    ScenarioEvaluationCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        scenario_id: summary.scenario_id.clone(),
        input_mode: summary.input_mode.clone(),
        syntax_label: summary.syntax_label.clone(),
        semantic_disposition: summary.semantic_disposition.clone(),
        selected_heuristic_ids: summary.selected_heuristic_ids.join(" | "),
        grammar_reason_code: summary.grammar_reason_code.clone(),
        grammar_reason_text: summary.grammar_reason_text.clone(),
        trust_scalar: summary.trust_scalar,
        heuristic_bank_entry_count: summary.heuristic_bank_entry_count,
        heuristic_candidates_post_admissibility: summary.heuristic_candidates_post_admissibility,
        heuristic_candidates_post_regime: summary.heuristic_candidates_post_regime,
        heuristic_candidates_pre_scope: summary.heuristic_candidates_pre_scope,
        heuristic_candidates_post_scope: summary.heuristic_candidates_post_scope,
        heuristics_rejected_by_admissibility: summary.heuristics_rejected_by_admissibility,
        heuristics_rejected_by_regime: summary.heuristics_rejected_by_regime,
        heuristics_rejected_by_scope: summary.heuristics_rejected_by_scope,
        heuristics_selected_final: summary.heuristics_selected_final,
        boundary_sample_count: summary.boundary_sample_count,
        violation_sample_count: summary.violation_sample_count,
        first_boundary_time: summary.first_boundary_time,
        first_violation_time: summary.first_violation_time,
        reproducible: summary.reproducible,
        triggered_baseline_count: summary.triggered_baseline_count,
        unknown_reason_class: summary.unknown_reason_class.clone().unwrap_or_default(),
        note: summary.note.clone(),
    }
}

pub(super) fn bank_validation_csv_row(
    report: &crate::engine::bank::HeuristicBankValidationReport,
) -> BankValidationCsvRow {
    BankValidationCsvRow {
        schema_version: report.schema_version.clone(),
        engine_version: report.engine_version.clone(),
        bank_schema_version: report.bank_schema_version.clone(),
        bank_version: report.bank_version.clone(),
        bank_source_kind: report.bank_source_kind.as_label().to_string(),
        bank_source_path: report.bank_source_path.clone().unwrap_or_default(),
        bank_content_hash: report.bank_content_hash.clone(),
        strict_validation: report.strict_validation,
        validation_mode: report.validation_mode.clone(),
        entry_count: report.entry_count,
        valid: report.valid,
        duplicate_ids: report.duplicate_ids.join(" | "),
        self_link_notes: report.self_link_notes.join(" | "),
        compatibility_conflicts: report.compatibility_conflicts.join(" | "),
        missing_compatibility_links: report.missing_compatibility_links.join(" | "),
        missing_incompatibility_links: report.missing_incompatibility_links.join(" | "),
        strict_validation_errors: report.strict_validation_errors.join(" | "),
        unknown_link_targets: report.unknown_link_targets.join(" | "),
        provenance_gaps: report.provenance_gaps.join(" | "),
        regime_tag_notes: report.regime_tag_notes.join(" | "),
        retrieval_priority_notes: report.retrieval_priority_notes.join(" | "),
        scope_sanity_notes: report.scope_sanity_notes.join(" | "),
        violations: report.violations.join(" | "),
        warnings: report.warnings.join(" | "),
        note: report.note.clone(),
    }
}

pub(super) fn sweep_point_csv_row(point: &SweepPointResult) -> SweepPointCsvRow {
    SweepPointCsvRow {
        schema_version: point.schema_version.clone(),
        engine_version: point.engine_version.clone(),
        bank_version: point.bank_version.clone(),
        sweep_family: point.sweep_family.clone(),
        scenario_id: point.scenario_id.clone(),
        parameter_name: point.parameter_name.clone(),
        parameter_value: point.parameter_value,
        secondary_parameter_name: point.secondary_parameter_name.clone().unwrap_or_default(),
        secondary_parameter_value: point.secondary_parameter_value,
        syntax_label: point.syntax_label.clone(),
        semantic_disposition: point.semantic_disposition.clone(),
        selected_heuristic_ids: point.selected_heuristic_ids.join(" | "),
        note: point.note.clone(),
    }
}

pub(super) fn sweep_summary_csv_row(summary: &SweepRunSummary) -> SweepSummaryCsvRow {
    SweepSummaryCsvRow {
        schema_version: summary.schema_version.clone(),
        engine_version: summary.engine_version.clone(),
        bank_version: summary.bank_version.clone(),
        sweep_family: summary.sweep_family.clone(),
        member_count: summary.member_count,
        unique_syntax_labels: summary.unique_syntax_labels.join(" | "),
        unique_semantic_dispositions: summary.unique_semantic_dispositions.join(" | "),
        unknown_count: summary.unknown_count,
        ambiguous_count: summary.ambiguous_count,
        disposition_flip_count: summary.disposition_flip_count,
        note: summary.note.clone(),
    }
}

pub(super) fn comparator_results_csv_row(
    bundle: &EngineOutputBundle,
    result: &crate::evaluation::types::BaselineComparatorResult,
) -> ComparatorResultsCsvRow {
    ComparatorResultsCsvRow {
        schema_version: result.schema_version.clone(),
        engine_version: result.engine_version.clone(),
        bank_version: result.bank_version.clone(),
        scenario_id: result.scenario_id.clone(),
        comparator_id: result.comparator_id.clone(),
        comparator_name: result.comparator_id.clone(),
        comparator_label: result.comparator_label.clone(),
        alarm: result.triggered,
        first_alarm_step: result.first_trigger_step,
        first_alarm_time: result.first_trigger_time,
        config_reference: bundle.run_metadata.run_configuration_hash.clone(),
        comparator_summary: result.comparator_summary.clone(),
        distinction_note: result.distinction_note.clone(),
    }
}

fn join_count_map(map: &BTreeMap<String, usize>) -> String {
    map.iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn value_at(values: &[f64], index: usize) -> Option<f64> {
    values.get(index).copied()
}
