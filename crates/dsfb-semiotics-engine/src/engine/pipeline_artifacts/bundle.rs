//! Bundle packaging helpers for deterministic artifact export.
//!
//! This module owns the final markdown/PDF/archive materialization steps so the top-level
//! orchestration layer can stay focused on export sequencing rather than file-format details.

use std::path::Path;

use anyhow::{Context, Result};

use crate::engine::types::{EngineOutputBundle, FigureArtifact, ReportManifest};
use crate::evaluation::types::{ArtifactCompletenessCheck, FigureIntegrityCheck};
use crate::io::output::OutputLayout;
use crate::io::zip::zip_directory;
use crate::report::artifact_report::build_markdown_report;
use crate::report::pdf::write_artifact_pdf;

use super::report::collect_pdf_text_artifacts;

pub(crate) struct ReportBundlePaths<'a> {
    pub report_markdown_path: &'a Path,
    pub report_pdf_path: &'a Path,
    pub manifest_path: &'a Path,
}

pub(crate) fn write_report_bundle(
    bundle: &EngineOutputBundle,
    figure_artifacts: &[FigureArtifact],
    layout: &OutputLayout,
    manifest: &ReportManifest,
    completeness: Option<&ArtifactCompletenessCheck>,
    figure_integrity_checks: Option<&[FigureIntegrityCheck]>,
    paths: ReportBundlePaths<'_>,
) -> Result<()> {
    let markdown = build_markdown_report(
        bundle,
        figure_artifacts,
        manifest,
        completeness,
        figure_integrity_checks,
    );
    std::fs::write(paths.report_markdown_path, &markdown)
        .with_context(|| format!("failed to write {}", paths.report_markdown_path.display()))?;

    let text_artifacts =
        collect_pdf_text_artifacts(layout, manifest, &markdown, paths.manifest_path)?;
    write_artifact_pdf(
        paths.report_pdf_path,
        "dsfb-semiotics-engine report",
        &markdown
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        figure_artifacts,
        manifest,
        &text_artifacts,
    )?;
    Ok(())
}

pub(crate) fn write_zip_bundle(run_dir: &Path, zip_path: &Path) -> Result<()> {
    zip_directory(run_dir, zip_path)
}
