//! Report-manifest and bundle-completeness helpers for deterministic artifact export.

use std::path::Path;

use anyhow::{Context, Result};

use crate::engine::types::{EngineOutputBundle, FigureArtifact, ReportManifest};
use crate::evaluation::types::ArtifactCompletenessCheck;
use crate::io::output::OutputLayout;
use crate::io::schema::ARTIFACT_SCHEMA_VERSION;
use crate::report::pdf::PdfTextArtifact;

pub(crate) fn build_report_manifest(
    bundle: &EngineOutputBundle,
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    report_markdown_path: &Path,
    report_pdf_path: &Path,
    zip_path: &Path,
    completeness: Option<&ArtifactCompletenessCheck>,
) -> Result<ReportManifest> {
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
    Ok(ReportManifest {
        schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
        engine_version: bundle.run_metadata.crate_version.clone(),
        bank_version: bundle.run_metadata.bank.bank_version.clone(),
        run_configuration_hash: bundle.run_metadata.run_configuration_hash.clone(),
        crate_name: bundle.run_metadata.crate_name.clone(),
        crate_version: bundle.run_metadata.crate_version.clone(),
        timestamp: bundle.run_metadata.timestamp.clone(),
        input_mode: bundle.run_metadata.input_mode.clone(),
        online_history_buffer_capacity: bundle.run_metadata.online_history_buffer_capacity,
        numeric_mode: bundle.run_metadata.numeric_mode.clone(),
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

pub(crate) fn build_artifact_completeness(
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

pub(crate) fn collect_pdf_text_artifacts(
    layout: &OutputLayout,
    manifest: &ReportManifest,
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
