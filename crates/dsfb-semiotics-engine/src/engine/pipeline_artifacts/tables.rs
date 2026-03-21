//! CSV/JSON tabular export helpers for deterministic artifact bundles.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

use crate::engine::event_timeline::build_scenario_event_timeline;
use crate::engine::pipeline_artifacts::figures::write_summary_figure_source_tables;
use crate::engine::types::{EngineOutputBundle, FigureArtifact};
use crate::evaluation::types::{FigureIntegrityCheck, SweepPointResult, SweepRunSummary};
use crate::figures::source::FigureSourceTable;
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;
use crate::io::output::OutputLayout;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::math::metrics::format_metric;

/// Summary of tabular export work performed for one completed run.
#[derive(Clone, Debug, Default)]
pub(crate) struct TabularArtifactsSummary {
    pub figure_integrity_checks: Vec<FigureIntegrityCheck>,
}

pub(crate) fn write_tabular_artifacts(
    bundle: &EngineOutputBundle,
    figure_source_tables: &[FigureSourceTable],
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    count_like_integer_tolerance: f64,
) -> Result<TabularArtifactsSummary> {
    let scenario_catalog = bundle
        .scenario_outputs
        .iter()
        .map(|scenario| scenario.record.clone())
        .collect::<Vec<_>>();
    write_rows(
        layout.csv_dir.join("scenario_catalog.csv").as_path(),
        scenario_catalog.clone(),
    )?;
    write_rows(
        layout.csv_dir.join("detectability_bounds.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.detectability.clone()),
    )?;
    write_rows(
        layout.csv_dir.join("semantic_matches.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| semantic_csv_row(&scenario.semantics)),
    )?;
    write_rows(
        layout.csv_dir.join("reproducibility_check.csv").as_path(),
        bundle
            .reproducibility_checks
            .iter()
            .map(|check| reproducibility_csv_row(bundle, check)),
    )?;
    write_rows(
        layout.csv_dir.join("reproducibility_summary.csv").as_path(),
        std::iter::once(bundle.reproducibility_summary.clone()),
    )?;
    write_rows(
        layout.csv_dir.join("evaluation_summary.csv").as_path(),
        std::iter::once(evaluation_summary_csv_row(&bundle.evaluation.summary)),
    )?;
    write_rows(
        layout.csv_dir.join("scenario_evaluations.csv").as_path(),
        bundle
            .evaluation
            .scenario_evaluations
            .iter()
            .map(scenario_evaluation_csv_row),
    )?;
    write_rows(
        layout.csv_dir.join("baseline_comparators.csv").as_path(),
        bundle.evaluation.baseline_results.clone(),
    )?;
    if !bundle.evaluation.smoothing_comparison_report.is_empty() {
        write_rows(
            layout
                .csv_dir
                .join("smoothing_comparison_report.csv")
                .as_path(),
            bundle.evaluation.smoothing_comparison_report.clone(),
        )?;
    }
    if !bundle.evaluation.retrieval_latency_report.is_empty() {
        write_rows(
            layout
                .csv_dir
                .join("retrieval_latency_report.csv")
                .as_path(),
            bundle.evaluation.retrieval_latency_report.clone(),
        )?;
    }
    write_rows(
        layout.csv_dir.join("comparator_results.csv").as_path(),
        bundle
            .evaluation
            .baseline_results
            .iter()
            .map(|result| comparator_results_csv_row(bundle, result)),
    )?;
    write_rows(
        layout
            .csv_dir
            .join("heuristic_bank_validation.csv")
            .as_path(),
        std::iter::once(bank_validation_csv_row(&bundle.evaluation.bank_validation)),
    )?;
    write_rows(
        layout.csv_dir.join("bank_validation_report.csv").as_path(),
        std::iter::once(bank_validation_csv_row(&bundle.evaluation.bank_validation)),
    )?;
    if !bundle.evaluation.sweep_results.is_empty() {
        write_rows(
            layout.csv_dir.join("sweep_results.csv").as_path(),
            bundle
                .evaluation
                .sweep_results
                .iter()
                .map(sweep_point_csv_row),
        )?;
    }
    if let Some(summary) = &bundle.evaluation.sweep_summary {
        write_rows(
            layout.csv_dir.join("sweep_summary.csv").as_path(),
            std::iter::once(sweep_summary_csv_row(summary)),
        )?;
    }

    let figure_integrity_checks = write_summary_figure_source_tables(
        bundle,
        figure_source_tables,
        figure_artifacts,
        layout,
        count_like_integer_tolerance,
    )?;

    let grammar_rows = bundle
        .scenario_outputs
        .iter()
        .flat_map(|scenario| scenario.grammar.clone())
        .collect::<Vec<_>>();
    write_rows(
        layout.csv_dir.join("grammar_events.csv").as_path(),
        grammar_rows,
    )?;

    write_rows(
        layout.csv_dir.join("pipeline_summary.csv").as_path(),
        bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.syntax.clone()),
    )?;

    for scenario in &bundle.scenario_outputs {
        let event_timeline = build_scenario_event_timeline(bundle, scenario)?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_event_timeline.csv", scenario.record.id))
                .as_path(),
            event_timeline.clone(),
        )?;
        write_pretty(
            layout
                .json_dir
                .join(format!("{}_event_timeline.json", scenario.record.id))
                .as_path(),
            &event_timeline,
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_timeseries.csv", scenario.record.id))
                .as_path(),
            scenario
                .observed
                .samples
                .iter()
                .zip(&scenario.predicted.samples)
                .map(|(observed, predicted)| {
                    time_series_row(&scenario.record.id, observed, predicted)
                }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_residual.csv", scenario.record.id))
                .as_path(),
            scenario.residual.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_drift.csv", scenario.record.id))
                .as_path(),
            scenario.drift.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_slew.csv", scenario.record.id))
                .as_path(),
            scenario.slew.samples.iter().map(|sample| {
                vector_norm_row(
                    &scenario.record.id,
                    sample.step,
                    sample.time,
                    &sample.values,
                    sample.norm,
                )
            }),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_sign.csv", scenario.record.id))
                .as_path(),
            scenario
                .sign
                .samples
                .iter()
                .map(|sample| sign_csv_row(&scenario.record.id, sample)),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_envelope.csv", scenario.record.id))
                .as_path(),
            scenario.envelope.samples.clone(),
        )?;
        write_rows(
            layout
                .csv_dir
                .join(format!("{}_grammar.csv", scenario.record.id))
                .as_path(),
            scenario.grammar.clone(),
        )?;
        if let Some(coordinated) = &scenario.coordinated {
            write_rows(
                layout
                    .csv_dir
                    .join(format!("{}_coordinated.csv", scenario.record.id))
                    .as_path(),
                coordinated.points.clone(),
            )?;
        }
    }

    write_pretty(
        layout.json_dir.join("run_metadata.json").as_path(),
        &bundle.run_metadata,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("loaded_heuristic_bank_descriptor.json")
            .as_path(),
        &bundle.run_metadata.bank,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_catalog.json").as_path(),
        &scenario_catalog,
    )?;
    write_pretty(
        layout.json_dir.join("reproducibility_check.json").as_path(),
        &bundle.reproducibility_check,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("reproducibility_checks.json")
            .as_path(),
        &bundle.reproducibility_checks,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("reproducibility_summary.json")
            .as_path(),
        &bundle.reproducibility_summary,
    )?;
    write_pretty(
        layout.json_dir.join("evaluation_summary.json").as_path(),
        &bundle.evaluation.summary,
    )?;
    write_pretty(
        layout.json_dir.join("scenario_evaluations.json").as_path(),
        &bundle.evaluation.scenario_evaluations,
    )?;
    write_pretty(
        layout.json_dir.join("baseline_comparators.json").as_path(),
        &bundle.evaluation.baseline_results,
    )?;
    if !bundle.evaluation.smoothing_comparison_report.is_empty() {
        write_pretty(
            layout
                .json_dir
                .join("smoothing_comparison_report.json")
                .as_path(),
            &bundle.evaluation.smoothing_comparison_report,
        )?;
    }
    if !bundle.evaluation.retrieval_latency_report.is_empty() {
        write_pretty(
            layout
                .json_dir
                .join("retrieval_latency_report.json")
                .as_path(),
            &bundle.evaluation.retrieval_latency_report,
        )?;
    }
    write_pretty(
        layout.json_dir.join("comparator_results.json").as_path(),
        &bundle
            .evaluation
            .baseline_results
            .iter()
            .map(|result| comparator_results_csv_row(bundle, result))
            .collect::<Vec<_>>(),
    )?;
    write_pretty(
        layout.json_dir.join("semantic_matches.json").as_path(),
        &bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.semantics.clone())
            .collect::<Vec<_>>(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("heuristic_bank_validation.json")
            .as_path(),
        &bundle.evaluation.bank_validation,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("bank_validation_report.json")
            .as_path(),
        &bundle.evaluation.bank_validation,
    )?;
    if !bundle.evaluation.sweep_results.is_empty() {
        write_pretty(
            layout.json_dir.join("sweep_results.json").as_path(),
            &bundle.evaluation.sweep_results,
        )?;
    }
    if let Some(summary) = &bundle.evaluation.sweep_summary {
        write_pretty(
            layout.json_dir.join("sweep_summary.json").as_path(),
            summary,
        )?;
    }
    write_pretty(
        layout.json_dir.join("scenario_outputs.json").as_path(),
        &bundle.scenario_outputs,
    )?;

    Ok(TabularArtifactsSummary {
        figure_integrity_checks,
    })
}

#[derive(Clone, Debug, Serialize)]
struct TimeSeriesCsvRow {
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
struct VectorNormCsvRow {
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
struct SignCsvRow {
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
struct SemanticMatchCsvRow {
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
struct ReproducibilityCsvRow {
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
struct EvaluationSummaryCsvRow {
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
struct ScenarioEvaluationCsvRow {
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
struct BankValidationCsvRow {
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
struct SweepPointCsvRow {
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
struct SweepSummaryCsvRow {
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
struct ComparatorResultsCsvRow {
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

fn time_series_row(
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

fn vector_norm_row(
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

fn sign_csv_row(scenario_id: &str, sample: &crate::engine::types::SignSample) -> SignCsvRow {
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

fn semantic_csv_row(result: &crate::engine::types::SemanticMatchResult) -> SemanticMatchCsvRow {
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

fn reproducibility_csv_row(
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

fn evaluation_summary_csv_row(
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

fn scenario_evaluation_csv_row(
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

fn bank_validation_csv_row(
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

fn sweep_point_csv_row(point: &SweepPointResult) -> SweepPointCsvRow {
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

fn sweep_summary_csv_row(summary: &SweepRunSummary) -> SweepSummaryCsvRow {
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

fn comparator_results_csv_row(
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
