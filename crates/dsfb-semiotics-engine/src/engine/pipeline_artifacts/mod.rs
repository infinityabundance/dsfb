//! Deterministic artifact assembly for completed semiotics runs.
//!
//! Responsibilities are split explicitly:
//! - [`tables`] materializes machine-readable CSV/JSON exports.
//! - [`figures`] writes figure-source tables and integrity reports.
//! - [`report`] assembles the manifest, completeness record, and PDF text appendices.
//! - [`bundle`] materializes markdown, PDF, and zip bundle files.
//! - this module keeps only the top-level export orchestration.

mod bundle;
mod figures;
mod integrity;
mod report;
mod tables;

use std::path::PathBuf;

use anyhow::Result;

use crate::engine::types::EngineOutputBundle;
use crate::figures::plots::render_all_figures;
use crate::figures::source::prepare_publication_figure_source_tables;
use crate::io::csv::write_rows;
use crate::io::json::write_pretty;
use crate::io::output::{prepare_clean_export_layout, OutputLayout};

use self::bundle::{write_report_bundle, write_zip_bundle, ReportBundlePaths};
use self::report::{build_artifact_completeness, build_report_manifest};
use self::tables::write_tabular_artifacts;

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
    let archive_stem = archive_bundle_stem(bundle);
    let zip_path = layout.run_dir.join(format!("{archive_stem}.zip"));

    let initial_manifest = build_report_manifest(
        bundle,
        &figure_artifacts,
        &layout,
        &report_markdown_path,
        &report_pdf_path,
        &zip_path,
        None,
    )?;
    write_pretty(&manifest_path, &initial_manifest)?;
    write_report_bundle(
        bundle,
        &figure_artifacts,
        &layout,
        &initial_manifest,
        None,
        Some(&tabular_summary.figure_integrity_checks),
        ReportBundlePaths {
            report_markdown_path: &report_markdown_path,
            report_pdf_path: &report_pdf_path,
            manifest_path: &manifest_path,
        },
    )?;
    write_zip_bundle(&layout.run_dir, &zip_path)?;

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
    write_pretty(&manifest_path, &report_manifest)?;
    write_report_bundle(
        bundle,
        &figure_artifacts,
        &layout,
        &report_manifest,
        Some(&completeness),
        Some(&tabular_summary.figure_integrity_checks),
        ReportBundlePaths {
            report_markdown_path: &report_markdown_path,
            report_pdf_path: &report_pdf_path,
            manifest_path: &manifest_path,
        },
    )?;
    write_zip_bundle(&layout.run_dir, &zip_path)?;

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

fn archive_bundle_stem(bundle: &EngineOutputBundle) -> String {
    format!(
        "{}-dsfb-semiotics-engine-{}",
        archive_bundle_prefix(bundle),
        bundle.run_metadata.timestamp
    )
}

fn archive_bundle_prefix(bundle: &EngineOutputBundle) -> String {
    if bundle.run_metadata.input_mode.starts_with("synthetic") {
        return "synthetic".to_string();
    }
    if bundle
        .scenario_outputs
        .iter()
        .any(|scenario| scenario.record.id.starts_with("nasa_milling"))
    {
        return "nasa_milling".to_string();
    }
    if bundle
        .scenario_outputs
        .iter()
        .any(|scenario| scenario.record.id.starts_with("nasa_bearings"))
    {
        return "nasa_bearings".to_string();
    }
    if bundle.run_metadata.input_mode == "csv" {
        return "csv".to_string();
    }
    bundle
        .run_metadata
        .input_mode
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
