//! Figure-source export helpers for deterministic publication-style artifacts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::engine::pipeline_artifacts::integrity::{
    build_figure_integrity_check, figure_integrity_csv_row,
};
use crate::engine::types::{EngineOutputBundle, FigureArtifact};
use crate::evaluation::types::FigureIntegrityCheck;
use crate::figures::source::{
    baseline_comparator_source_rows, detectability_source_rows, semantic_retrieval_source_rows,
    sweep_summary_source_rows, FigureSourceTable,
};
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;
use crate::io::output::OutputLayout;

pub(crate) fn write_summary_figure_source_tables(
    bundle: &EngineOutputBundle,
    figure_source_tables: &[FigureSourceTable],
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    count_like_integer_tolerance: f64,
) -> Result<Vec<FigureIntegrityCheck>> {
    let mut checks = Vec::new();
    let figure_lookup = figure_artifacts
        .iter()
        .map(|artifact| (artifact.figure_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();

    for table in figure_source_tables {
        let source_csv = layout
            .csv_dir
            .join(format!("{}_source.csv", table.figure_id))
            .display()
            .to_string();
        let source_json = layout
            .json_dir
            .join(format!("{}_source.json", table.figure_id))
            .display()
            .to_string();
        write_rows(Path::new(&source_csv), table.rows.clone())?;
        write_pretty(Path::new(&source_json), table)?;
        let artifact = figure_lookup.get(&table.figure_id);
        checks.push(build_figure_integrity_check(
            &bundle.run_metadata,
            table,
            artifact.copied(),
            &source_csv,
            &source_json,
            count_like_integer_tolerance,
            Path::new(&source_csv).is_file(),
            Path::new(&source_json).is_file(),
        ));
    }

    write_legacy_summary_figure_sources(bundle, layout)?;

    write_rows(
        layout.csv_dir.join("figure_integrity_checks.csv").as_path(),
        checks.iter().map(figure_integrity_csv_row),
    )?;
    write_rows(
        layout.csv_dir.join("figure_integrity_report.csv").as_path(),
        checks.iter().map(figure_integrity_csv_row),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_integrity_checks.json")
            .as_path(),
        &checks,
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_integrity_report.json")
            .as_path(),
        &checks,
    )?;

    Ok(checks)
}

pub(crate) fn write_legacy_summary_figure_sources(
    bundle: &EngineOutputBundle,
    layout: &OutputLayout,
) -> Result<()> {
    let detectability_rows = detectability_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_09_detectability_source.csv")
            .as_path(),
        detectability_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_09_detectability_source.json")
            .as_path(),
        &detectability_rows,
    )?;

    let semantic_rows = semantic_retrieval_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_12_semantic_retrieval_source.csv")
            .as_path(),
        semantic_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_12_semantic_retrieval_source.json")
            .as_path(),
        &semantic_rows,
    )?;

    let baseline_rows = baseline_comparator_source_rows(bundle);
    write_rows(
        layout
            .csv_dir
            .join("figure_13_internal_baseline_comparators_source.csv")
            .as_path(),
        baseline_rows.clone(),
    )?;
    write_pretty(
        layout
            .json_dir
            .join("figure_13_internal_baseline_comparators_source.json")
            .as_path(),
        &baseline_rows,
    )?;

    if !bundle.evaluation.sweep_results.is_empty() {
        let sweep_rows = sweep_summary_source_rows(bundle);
        #[derive(Serialize)]
        struct SweepLegacySourceCsvRow {
            schema_version: String,
            figure_id: String,
            sweep_family: String,
            scenario_id: String,
            parameter_name: String,
            parameter_value: f64,
            semantic_disposition: String,
            disposition_code: i32,
            selected_heuristic_ids: String,
            note: String,
        }
        write_rows(
            layout
                .csv_dir
                .join("figure_14_sweep_stability_source.csv")
                .as_path(),
            sweep_rows.iter().map(|row| SweepLegacySourceCsvRow {
                schema_version: row.schema_version.clone(),
                figure_id: row.figure_id.clone(),
                sweep_family: row.sweep_family.clone(),
                scenario_id: row.scenario_id.clone(),
                parameter_name: row.parameter_name.clone(),
                parameter_value: row.parameter_value,
                semantic_disposition: row.semantic_disposition.clone(),
                disposition_code: row.disposition_code,
                selected_heuristic_ids: row.selected_heuristic_ids.join(" | "),
                note: row.note.clone(),
            }),
        )?;
        write_pretty(
            layout
                .json_dir
                .join("figure_14_sweep_stability_source.json")
                .as_path(),
            &sweep_rows,
        )?;
    }

    Ok(())
}
