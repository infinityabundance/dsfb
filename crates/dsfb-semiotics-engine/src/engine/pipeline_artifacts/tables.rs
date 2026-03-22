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

mod rows;

use self::rows::{
    bank_validation_csv_row, comparator_results_csv_row, evaluation_summary_csv_row,
    reproducibility_csv_row, scenario_evaluation_csv_row, semantic_csv_row, sign_csv_row,
    sweep_point_csv_row, sweep_summary_csv_row, time_series_row, vector_norm_row,
};

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
