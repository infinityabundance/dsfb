//! Deterministic pipeline artifact assembly, manifest construction, figure-source exports, and tabular serialization.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::engine::types::{EngineOutputBundle, FigureArtifact};
use crate::evaluation::types::{ArtifactCompletenessCheck, FigureIntegrityCheck};
use crate::figures::plots::render_all_figures;
use crate::figures::source::{
    baseline_comparator_source_rows, detectability_source_rows,
    prepare_publication_figure_source_tables, semantic_retrieval_source_rows,
    sweep_summary_source_rows, FigureSourceTable,
};
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;
use crate::io::output::{prepare_clean_export_layout, OutputLayout};
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::io::zip::zip_directory;
use crate::math::metrics::format_metric;
use crate::report::artifact_report::build_markdown_report;
use crate::report::pdf::{write_artifact_pdf, PdfTextArtifact};

/// Filesystem artifact inventory written for one completed deterministic run.
#[derive(Clone, Debug)]
pub struct ExportedArtifacts {
    pub run_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub report_markdown: PathBuf,
    pub report_pdf: PathBuf,
    pub zip_path: PathBuf,
    pub figure_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default)]
struct TabularArtifactsSummary {
    figure_integrity_checks: Vec<FigureIntegrityCheck>,
}

/// Writes figures, CSV, JSON, markdown, PDF, and zip artifacts for one completed bundle.
pub fn export_artifacts(bundle: &EngineOutputBundle) -> Result<ExportedArtifacts> {
    let layout = OutputLayout {
        timestamp: bundle.run_metadata.timestamp.clone(),
        run_dir: bundle.run_dir.clone(),
        figures_dir: bundle.run_dir.join("figures"),
        csv_dir: bundle.run_dir.join("csv"),
        json_dir: bundle.run_dir.join("json"),
        report_dir: bundle.run_dir.join("report"),
    };
    prepare_clean_export_layout(&layout)?;

    let figure_source_tables = prepare_publication_figure_source_tables(bundle)?;
    let figure_artifacts = render_all_figures(&figure_source_tables, &layout.figures_dir)?;
    let tabular_summary = write_tabular_artifacts(
        bundle,
        &figure_source_tables,
        &figure_artifacts,
        &layout,
        bundle
            .run_metadata
            .engine_settings
            .plotting
            .count_like_integer_tolerance,
    )?;

    let manifest_path = layout.run_dir.join("manifest.json");
    let report_markdown_path = layout.report_dir.join("dsfb_semiotics_engine_report.md");
    let report_pdf_path = layout.report_dir.join("dsfb_semiotics_engine_report.pdf");
    let zip_path = layout.run_dir.join(format!(
        "dsfb-semiotics-engine-{}.zip",
        bundle.run_metadata.timestamp
    ));

    let initial_manifest = build_report_manifest(
        bundle,
        &figure_artifacts,
        &layout,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        None,
    )?;
    let initial_markdown = build_markdown_report(
        bundle,
        &figure_artifacts,
        &initial_manifest,
        None,
        Some(&tabular_summary.figure_integrity_checks),
    );
    std::fs::write(&report_markdown_path, &initial_markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_pretty(&manifest_path, &initial_manifest)?;
    let text_artifacts = collect_pdf_text_artifacts(
        &layout,
        &initial_manifest,
        &initial_markdown,
        &manifest_path,
    )?;
    write_artifact_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &initial_markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        &figure_artifacts,
        &initial_manifest,
        &text_artifacts,
    )?;
    zip_directory(&layout.run_dir, &zip_path)?;
    let completeness = build_artifact_completeness(
        bundle,
        &layout,
        &figure_artifacts,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        &manifest_path,
    )?;
    write_rows(
        layout.csv_dir.join("artifact_completeness.csv").as_path(),
        std::iter::once(completeness.clone()),
    )?;
    write_pretty(
        layout.json_dir.join("artifact_completeness.json").as_path(),
        &completeness,
    )?;
    let report_manifest = build_report_manifest(
        bundle,
        &figure_artifacts,
        &layout,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        Some(&completeness),
    )?;
    let final_markdown = build_markdown_report(
        bundle,
        &figure_artifacts,
        &report_manifest,
        Some(&completeness),
        Some(&tabular_summary.figure_integrity_checks),
    );
    std::fs::write(&report_markdown_path, &final_markdown)
        .with_context(|| format!("failed to write {}", report_markdown_path.display()))?;
    write_pretty(&manifest_path, &report_manifest)?;
    let final_text_artifacts =
        collect_pdf_text_artifacts(&layout, &report_manifest, &final_markdown, &manifest_path)?;
    write_artifact_pdf(
        &report_pdf_path,
        "dsfb-semiotics-engine report",
        &final_markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        &figure_artifacts,
        &report_manifest,
        &final_text_artifacts,
    )?;
    zip_directory(&layout.run_dir, &zip_path)?;

    Ok(ExportedArtifacts {
        run_dir: layout.run_dir,
        manifest_path,
        report_markdown: report_markdown_path,
        report_pdf: report_pdf_path,
        zip_path,
        figure_paths: figure_artifacts
            .iter()
            .flat_map(|figure| {
                [
                    PathBuf::from(&figure.png_path),
                    PathBuf::from(&figure.svg_path),
                ]
            })
            .collect(),
    })
}
fn write_tabular_artifacts(
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

fn write_summary_figure_source_tables(
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

fn write_legacy_summary_figure_sources(
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

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-integrity helpers keep the export contract explicit without hiding artifact fields behind ad hoc tuples."
)]
fn build_figure_integrity_check(
    run_metadata: &crate::engine::types::RunMetadata,
    table: &FigureSourceTable,
    artifact: Option<&FigureArtifact>,
    source_csv: &str,
    source_json: &str,
    count_like_integer_tolerance: f64,
    source_csv_present: bool,
    source_json_present: bool,
) -> FigureIntegrityCheck {
    let observed_panels = ordered_panel_ids(table);
    let panel_labels = observed_panels
        .iter()
        .map(|panel_id| panel_title(table, panel_id))
        .collect::<Vec<_>>();
    let series_lengths = observed_panels
        .iter()
        .map(|panel_id| {
            table
                .rows
                .iter()
                .filter(|row| row.panel_id == *panel_id)
                .count()
        })
        .collect::<Vec<_>>();
    let source_row_count = table.rows.len();
    let nonempty_series = !table.rows.is_empty() && series_lengths.iter().all(|length| *length > 0);
    let nonzero_values_present = table.rows.iter().any(|row| {
        row.y_value.abs() > 1.0e-12
            || row
                .secondary_y_value
                .map(|value| value.abs() > 1.0e-12)
                .unwrap_or(false)
    });
    let count_like_panels_integerlike = table.count_like_panel_ids.iter().all(|panel_id| {
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == *panel_id)
            .all(|row| (row.y_value - row.y_value.round()).abs() <= count_like_integer_tolerance)
    });
    let png_path = artifact
        .map(|artifact| artifact.png_path.clone())
        .unwrap_or_default();
    let svg_path = artifact
        .map(|artifact| artifact.svg_path.clone())
        .unwrap_or_default();
    let png_present = !png_path.is_empty() && Path::new(&png_path).is_file();
    let svg_present = !svg_path.is_empty() && Path::new(&svg_path).is_file();
    let expected_panels = if table.expected_panel_ids.is_empty() {
        (0..table.expected_panel_count)
            .map(|index| format!("panel_{}", index + 1))
            .collect::<Vec<_>>()
    } else {
        table.expected_panel_ids.clone()
    };
    let source_table_present = source_csv_present && source_json_present;
    let failures = figure_integrity_failures(
        table,
        &expected_panels,
        &observed_panels,
        source_table_present,
        nonempty_series,
        count_like_panels_integerlike,
        png_present,
        svg_present,
    );
    let integrity_passed = failures.is_empty();

    FigureIntegrityCheck {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: run_metadata.crate_version.clone(),
        bank_version: run_metadata.bank.bank_version.clone(),
        figure_id: table.figure_id.clone(),
        expected_panel_count: table.expected_panel_count,
        observed_panel_count: observed_panels.len(),
        expected_panels,
        observed_panels,
        panel_labels,
        series_lengths,
        source_row_count,
        source_table_present,
        nonempty_series,
        nonzero_values_present,
        count_like_panels_integerlike,
        consistent_with_source: integrity_passed,
        integrity_passed,
        failures,
        source_csv: source_csv.to_string(),
        source_json: source_json.to_string(),
        png_path,
        svg_path,
        png_present,
        svg_present,
        note: "Figure rendered from the exported figure-source table; integrity check covers panel identity, source rows, count-like panels, and emitted PNG/SVG presence."
            .to_string(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Figure-integrity failures are assembled from explicit typed checks so each exported failure surface stays auditable."
)]
fn figure_integrity_failures(
    table: &FigureSourceTable,
    expected_panels: &[String],
    observed_panels: &[String],
    source_table_present: bool,
    nonempty_series: bool,
    count_like_panels_integerlike: bool,
    png_present: bool,
    svg_present: bool,
) -> Vec<String> {
    let mut failures = Vec::new();
    if !source_table_present {
        failures.push("source table missing".to_string());
    }
    if observed_panels.len() != table.expected_panel_count {
        failures.push(format!(
            "expected {} panels but observed {}",
            table.expected_panel_count,
            observed_panels.len()
        ));
    }
    if expected_panels != observed_panels {
        failures.push(format!(
            "expected panels [{}] but observed [{}]",
            expected_panels.join(", "),
            observed_panels.join(", ")
        ));
    }
    if !nonempty_series {
        failures.push("one or more observed panels has an empty series".to_string());
    }
    if !count_like_panels_integerlike {
        failures.push("count-like panel used non-integer-like values".to_string());
    }
    if !png_present {
        failures.push("png render missing".to_string());
    }
    if !svg_present {
        failures.push("svg render missing".to_string());
    }
    failures
}

fn ordered_panel_ids(table: &FigureSourceTable) -> Vec<String> {
    let mut panel_ids = Vec::new();
    for row in &table.rows {
        if !panel_ids.iter().any(|panel_id| panel_id == &row.panel_id) {
            panel_ids.push(row.panel_id.clone());
        }
    }
    panel_ids
}

fn panel_title(table: &FigureSourceTable, panel_id: &str) -> String {
    table
        .rows
        .iter()
        .find(|row| row.panel_id == panel_id)
        .map(|row| row.panel_title.clone())
        .unwrap_or_else(|| panel_id.to_string())
}

fn collect_relative_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().display().to_string())
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn build_report_manifest(
    bundle: &EngineOutputBundle,
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    report_markdown_path: &Path,
    report_pdf_path: &Path,
    zip_path: &Path,
    completeness: Option<&ArtifactCompletenessCheck>,
) -> Result<crate::engine::types::ReportManifest> {
    let mut notes = vec![
        "Synthetic and CSV-driven runs share the same deterministic engine layers.".to_string(),
        "Semantic outputs are constrained heuristic retrieval results, not unique-cause claims.".to_string(),
        "Evaluation outputs summarize the deterministic engine with internal deterministic comparators only.".to_string(),
        format!(
            "Heuristic bank source=`{}`, version=`{}`, hash=`{}`.",
            bundle.run_metadata.bank.source_kind.as_label(),
            bundle.run_metadata.bank.bank_version,
            bundle.run_metadata.bank.content_hash
        ),
    ];
    if let Some(completeness) = completeness {
        notes.push(format!(
            "Artifact completeness: complete=`{}` with {} figures, {} CSV files, and {} JSON files.",
            completeness.complete,
            completeness.figure_count,
            completeness.csv_count,
            completeness.json_count
        ));
    }
    Ok(crate::engine::types::ReportManifest {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        run_configuration_hash: bundle.run_metadata.run_configuration_hash.clone(),
        crate_name: bundle.run_metadata.crate_name.clone(),
        crate_version: bundle.run_metadata.crate_version.clone(),
        timestamp: bundle.run_metadata.timestamp.clone(),
        input_mode: bundle.run_metadata.input_mode.clone(),
        bank: bundle.run_metadata.bank.clone(),
        run_dir: layout.run_dir.display().to_string(),
        report_markdown: report_markdown_path.display().to_string(),
        report_pdf: report_pdf_path.display().to_string(),
        zip_archive: zip_path.display().to_string(),
        figure_paths: figure_artifacts
            .iter()
            .flat_map(|figure| [figure.png_path.clone(), figure.svg_path.clone()])
            .collect(),
        csv_paths: collect_relative_files(&layout.csv_dir)?,
        json_paths: collect_relative_files(&layout.json_dir)?,
        scenario_ids: bundle
            .scenario_outputs
            .iter()
            .map(|scenario| scenario.record.id.clone())
            .collect(),
        notes,
    })
}

fn build_artifact_completeness(
    bundle: &EngineOutputBundle,
    layout: &OutputLayout,
    figure_artifacts: &[FigureArtifact],
    report_markdown_path: &Path,
    report_pdf_path: &Path,
    zip_path: &Path,
    manifest_path: &Path,
) -> Result<ArtifactCompletenessCheck> {
    let csv_count = collect_relative_files(&layout.csv_dir)?.len() + 1;
    let json_count = collect_relative_files(&layout.json_dir)?.len() + 1;
    let report_markdown_present = report_markdown_path.is_file();
    let report_pdf_present = report_pdf_path.is_file();
    let zip_present = zip_path.is_file();
    let manifest_present = manifest_path.is_file();
    let complete = report_markdown_present
        && report_pdf_present
        && zip_present
        && manifest_present
        && !figure_artifacts.is_empty();
    Ok(ArtifactCompletenessCheck {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        figure_count: figure_artifacts.len() * 2,
        csv_count,
        json_count,
        report_markdown_present,
        report_pdf_present,
        zip_present,
        manifest_present,
        complete,
        note: "Artifact completeness is evaluated after the deterministic export pipeline has emitted figures, tables, report files, manifest, and zip archive.".to_string(),
    })
}

fn collect_pdf_text_artifacts(
    layout: &OutputLayout,
    manifest: &crate::engine::types::ReportManifest,
    markdown: &str,
    manifest_path: &Path,
) -> Result<Vec<PdfTextArtifact>> {
    let mut artifacts = Vec::new();
    artifacts.push(PdfTextArtifact {
        title: "Report Markdown Source".to_string(),
        artifact_path: manifest.report_markdown.clone(),
        artifact_kind: "markdown".to_string(),
        content: markdown.to_string(),
    });
    artifacts.push(PdfTextArtifact {
        title: "Run Manifest".to_string(),
        artifact_path: manifest_path.display().to_string(),
        artifact_kind: "json".to_string(),
        content: serde_json::to_string_pretty(manifest)?,
    });

    for path in &manifest.csv_paths {
        artifacts.push(PdfTextArtifact {
            title: format!("CSV Artifact: {}", file_name(path)),
            artifact_path: path.clone(),
            artifact_kind: "csv".to_string(),
            content: std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {path}"))?,
        });
    }
    for path in &manifest.json_paths {
        artifacts.push(PdfTextArtifact {
            title: format!("JSON Artifact: {}", file_name(path)),
            artifact_path: path.clone(),
            artifact_kind: "json".to_string(),
            content: std::fs::read_to_string(path)
                .with_context(|| format!("failed to read {path}"))?,
        });
    }

    artifacts.push(PdfTextArtifact {
        title: "Archive Output Summary".to_string(),
        artifact_path: manifest.zip_archive.clone(),
        artifact_kind: "archive-summary".to_string(),
        content: format!(
            "Zip archive path: {}\nRun directory: {}\nFigures directory: {}\nCSV directory: {}\nJSON directory: {}\nReport directory: {}\n\nThe PDF report embeds the generated figure PNG artifacts and appends the text-based artifacts directly. The zip archive remains the machine-oriented bundle for direct file extraction.",
            manifest.zip_archive,
            manifest.run_dir,
            layout.figures_dir.display(),
            layout.csv_dir.display(),
            layout.json_dir.display(),
            layout.report_dir.display(),
        ),
    });

    Ok(artifacts)
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
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
struct FigureIntegrityCsvRow {
    schema_version: String,
    engine_version: String,
    bank_version: String,
    figure_id: String,
    expected_panel_count: usize,
    observed_panel_count: usize,
    expected_panels: String,
    observed_panels: String,
    panel_labels: String,
    series_lengths: String,
    source_row_count: usize,
    source_table_present: bool,
    nonempty_series: bool,
    nonzero_values_present: bool,
    count_like_panels_integerlike: bool,
    consistent_with_source: bool,
    integrity_passed: bool,
    failures: String,
    source_csv: String,
    source_json: String,
    png_path: String,
    svg_path: String,
    png_present: bool,
    svg_present: bool,
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

fn sweep_point_csv_row(point: &crate::evaluation::types::SweepPointResult) -> SweepPointCsvRow {
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

fn sweep_summary_csv_row(
    summary: &crate::evaluation::types::SweepRunSummary,
) -> SweepSummaryCsvRow {
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

fn figure_integrity_csv_row(check: &FigureIntegrityCheck) -> FigureIntegrityCsvRow {
    FigureIntegrityCsvRow {
        schema_version: check.schema_version.clone(),
        engine_version: check.engine_version.clone(),
        bank_version: check.bank_version.clone(),
        figure_id: check.figure_id.clone(),
        expected_panel_count: check.expected_panel_count,
        observed_panel_count: check.observed_panel_count,
        expected_panels: check.expected_panels.join(" | "),
        observed_panels: check.observed_panels.join(" | "),
        panel_labels: check.panel_labels.join(" | "),
        series_lengths: check
            .series_lengths
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" | "),
        source_row_count: check.source_row_count,
        source_table_present: check.source_table_present,
        nonempty_series: check.nonempty_series,
        nonzero_values_present: check.nonzero_values_present,
        count_like_panels_integerlike: check.count_like_panels_integerlike,
        consistent_with_source: check.consistent_with_source,
        integrity_passed: check.integrity_passed,
        failures: check.failures.join(" | "),
        source_csv: check.source_csv.clone(),
        source_json: check.source_json.clone(),
        png_path: check.png_path.clone(),
        svg_path: check.svg_path.clone(),
        png_present: check.png_present,
        svg_present: check.svg_present,
        note: check.note.clone(),
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

#[cfg(test)]
mod figure_integrity_tests {
    use super::{build_figure_integrity_check, figure_integrity_failures};
    use crate::engine::bank::{BankSourceKind, LoadedBankDescriptor};
    use crate::engine::settings::EngineSettings;
    use crate::engine::types::{FigureArtifact, RunMetadata};
    use crate::figures::source::{FigureSourceRow, FigureSourceTable};
    use crate::io::schema::ARTIFACT_SCHEMA_VERSION;

    fn run_metadata() -> RunMetadata {
        RunMetadata {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            run_configuration_hash: "config-hash".to_string(),
            crate_name: "dsfb-semiotics-engine".to_string(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: None,
            git_commit: None,
            timestamp: "2026-03-20T10:00:00Z".to_string(),
            input_mode: "synthetic".to_string(),
            seed: 123,
            steps: 16,
            dt: 1.0,
            engine_settings: EngineSettings::default(),
            bank: LoadedBankDescriptor {
                schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
                bank_schema_version: "dsfb-semiotics-engine-bank/v1".to_string(),
                bank_version: "heuristic-bank/test".to_string(),
                source_kind: BankSourceKind::Builtin,
                source_path: None,
                content_hash: "bank-hash".to_string(),
                strict_validation: true,
                validation_mode: "strict".to_string(),
                note: "test".to_string(),
            },
            cli_args: vec!["--scenario".to_string(), "nominal_stable".to_string()],
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        }
    }

    fn row(panel_id: &str, series_id: &str, y_value: f64) -> FigureSourceRow {
        FigureSourceRow {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            figure_id: "figure_test".to_string(),
            plot_title: "Figure Test".to_string(),
            panel_id: panel_id.to_string(),
            panel_title: panel_id.to_string(),
            x_label: "x".to_string(),
            y_label: "y".to_string(),
            series_id: series_id.to_string(),
            series_label: series_id.to_string(),
            series_kind: "line".to_string(),
            color_key: "blue".to_string(),
            point_order: 0,
            x_value: 0.0,
            y_value,
            secondary_x_value: None,
            secondary_y_value: None,
            x_tick_label: String::new(),
            annotation_text: String::new(),
            scenario_id: "scenario".to_string(),
            note: "test row".to_string(),
        }
    }

    fn artifact() -> FigureArtifact {
        FigureArtifact {
            figure_id: "figure_test".to_string(),
            caption: "caption".to_string(),
            png_path: "/tmp/nonexistent.png".to_string(),
            svg_path: "/tmp/nonexistent.svg".to_string(),
        }
    }

    #[test]
    fn test_count_panel_uses_count_values() {
        let table = FigureSourceTable {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            figure_id: "figure_test".to_string(),
            plot_title: "Figure Test".to_string(),
            generation_timestamp: "2026-03-20T10:00:00Z".to_string(),
            expected_panel_count: 1,
            expected_panel_ids: vec!["count_panel".to_string()],
            count_like_panel_ids: vec!["count_panel".to_string()],
            panel_ids: vec!["count_panel".to_string()],
            series_ids: vec!["count_series".to_string()],
            rows: vec![row("count_panel", "count_series", 1.5)],
        };
        let check = build_figure_integrity_check(
            &run_metadata(),
            &table,
            Some(&artifact()),
            "source.csv",
            "source.json",
            1.0e-9,
            true,
            true,
        );

        assert!(!check.count_like_panels_integerlike);
        assert!(!check.integrity_passed);
        assert!(check
            .failures
            .iter()
            .any(|failure| failure.contains("count-like panel")));
    }

    #[test]
    fn test_missing_panel_fails_integrity_check() {
        let table = FigureSourceTable {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            figure_id: "figure_test".to_string(),
            plot_title: "Figure Test".to_string(),
            generation_timestamp: "2026-03-20T10:00:00Z".to_string(),
            expected_panel_count: 2,
            expected_panel_ids: vec!["panel_a".to_string(), "panel_b".to_string()],
            count_like_panel_ids: Vec::new(),
            panel_ids: vec!["panel_a".to_string()],
            series_ids: vec!["series_a".to_string()],
            rows: vec![row("panel_a", "series_a", 1.0)],
        };
        let failures = figure_integrity_failures(
            &table,
            &table.expected_panel_ids,
            &["panel_a".to_string()],
            true,
            true,
            true,
            true,
            true,
        );

        assert!(failures
            .iter()
            .any(|failure| failure.contains("expected 2 panels")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("expected panels [panel_a, panel_b]")));
    }

    #[test]
    fn test_wrong_series_mapping_fails_integrity_check() {
        let table = FigureSourceTable {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            bank_version: "heuristic-bank/test".to_string(),
            figure_id: "figure_test".to_string(),
            plot_title: "Figure Test".to_string(),
            generation_timestamp: "2026-03-20T10:00:00Z".to_string(),
            expected_panel_count: 1,
            expected_panel_ids: vec!["expected_panel".to_string()],
            count_like_panel_ids: Vec::new(),
            panel_ids: vec!["wrong_panel".to_string()],
            series_ids: vec!["series_a".to_string()],
            rows: vec![row("wrong_panel", "series_a", 2.0)],
        };
        let failures = figure_integrity_failures(
            &table,
            &table.expected_panel_ids,
            &["wrong_panel".to_string()],
            true,
            true,
            true,
            true,
            true,
        );

        assert!(failures.iter().any(|failure| failure
            .contains("expected panels [expected_panel] but observed [wrong_panel]")));
    }
}
