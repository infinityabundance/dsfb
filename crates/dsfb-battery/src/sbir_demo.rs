// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Reviewer-facing SBIR demo bundle orchestration.

use crate::audit::{build_stage2_audit_trace, AuditTraceBuildContext};
use crate::compliance::run_compliance_workflow;
use crate::evaluation::evaluate_cell;
use crate::export::{export_audit_trace_json, export_trajectory_csv, ExportError};
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::{
    load_capacity_csv, run_addendum_workflow, run_multicell_workflow, run_resource_trace_workflow,
    PipelineConfig,
};
use chrono::{Local, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const SBIR_BUNDLE_ARTIFACT_TYPE: &str = "dsfb_battery_sbir_demo_bundle";
const SBIR_BUNDLE_SCHEMA_VERSION: &str = "1.0.0";
const SBIR_OUTPUT_ROOT: &str = "sbir_demo";
const SBIR_OUTPUT_PREFIX: &str = "dsfb_battery_sbir_demo";
const AUDIT_TRACES_DIR: &str = "audit_traces";
const ADDENDUM_DIR: &str = "addendum";
const COMPLIANCE_DIR: &str = "compliance";
const MULTICELL_DIR: &str = "multicell";
const RESOURCE_TRACE_DIR: &str = "resource_trace";
const MULTICELL_JSON_NAME: &str = "multicell_summary.json";
const MULTICELL_CSV_NAME: &str = "multicell_summary.csv";
const RESOURCE_TRACE_JSON_NAME: &str = "resource_trace.json";
const RESOURCE_TRACE_SUMMARY_NAME: &str = "resource_trace_summary.txt";
const REVIEWER_SUMMARY_NAME: &str = "reviewer_summary.md";
const IMPLEMENTATION_SUMMARY_NAME: &str = "implementation_summary.txt";
const MANIFEST_JSON_NAME: &str = "manifest.json";
const MANIFEST_SHA256_NAME: &str = "manifest.sha256";
const PRIMARY_AUDIT_JSON_NAME: &str = "stage2_detection_results.json";
const PRIMARY_TRAJECTORY_NAME: &str = "semiotic_trajectory.csv";
#[cfg(any(test, feature = "cert-trace"))]
const CERT_TRACE_JSON_NAME: &str = "cert_trace.json";

#[derive(Debug, Error)]
pub enum SbirDemoError {
    #[error("output directory already exists and is not empty: {0}")]
    NonEmptyOutputDirectory(PathBuf),
    #[error("unsupported primary cell `{cell_id}`; supported cells: {supported:?}")]
    UnsupportedPrimaryCell {
        cell_id: String,
        supported: Vec<String>,
    },
    #[error("required input CSV was not found: {0}")]
    MissingInputCsv(PathBuf),
    #[error("expected bundle artifact was not produced: {0}")]
    MissingBundleArtifact(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("export error: {0}")]
    Export(#[from] ExportError),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct SbirDemoOptions {
    pub crate_dir: PathBuf,
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub primary_cell_id: String,
    pub include_multicell: bool,
    pub trace_resources: bool,
    pub timing_repeats: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SbirDemoInputRecord {
    pub cell_id: String,
    pub role: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SbirDemoRuntimeOptions {
    pub include_multicell: bool,
    pub trace_resources: bool,
    pub include_addendum: bool,
    pub include_compliance: bool,
    pub timing_repeats: usize,
    pub production_figures_regenerated: bool,
    pub production_figures_copied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SbirDemoArtifactRecord {
    pub path: String,
    pub sha256: String,
    pub artifact_role: String,
    pub provenance: String,
    pub description: String,
}

#[cfg(feature = "cert-trace")]
#[derive(Debug, Clone, Serialize)]
pub struct SbirCertTraceEvidenceRecord {
    pub tag: String,
    pub artifact: String,
    pub note: String,
}

#[cfg(feature = "cert-trace")]
#[derive(Debug, Clone, Serialize)]
pub struct SbirCertTraceArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub configuration_identifier: String,
    pub primary_cell_id: String,
    pub evidence_records: Vec<SbirCertTraceEvidenceRecord>,
    pub interface_contract_notes: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SbirDemoBundleManifest {
    pub artifact_type: String,
    pub schema_version: String,
    pub generated_at_utc: String,
    pub bundle_root: String,
    pub crate_name: String,
    pub crate_version: String,
    pub git_commit: Option<String>,
    pub worktree_dirty: Option<bool>,
    pub primary_cell_id: String,
    pub data_dir: String,
    pub config: PipelineConfig,
    pub config_hash: String,
    pub inputs: Vec<SbirDemoInputRecord>,
    pub options: SbirDemoRuntimeOptions,
    pub artifacts: Vec<SbirDemoArtifactRecord>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SbirDemoResult {
    pub output_root: PathBuf,
    pub reviewer_summary_path: PathBuf,
    pub implementation_summary_path: PathBuf,
    pub manifest_path: PathBuf,
    pub sha256_manifest_path: PathBuf,
    pub primary_audit_trace_path: PathBuf,
    pub bundle_manifest: SbirDemoBundleManifest,
}

pub fn resolve_sbir_demo_output_dir(crate_dir: &Path, explicit_output: Option<PathBuf>) -> PathBuf {
    match explicit_output {
        Some(output) if output.is_absolute() => output,
        Some(output) if output.components().count() == 1 => crate_dir
            .join("outputs")
            .join(SBIR_OUTPUT_ROOT)
            .join(output),
        Some(output) => crate_dir.join(output),
        None => {
            let root = crate_dir.join("outputs").join(SBIR_OUTPUT_ROOT);
            let stem = format!(
                "{}_{}",
                SBIR_OUTPUT_PREFIX,
                Local::now().format("%Y%m%d_%H%M%S")
            );
            unique_named_output_dir(&root, &stem)
        }
    }
}

pub fn run_sbir_demo_workflow(options: &SbirDemoOptions) -> Result<SbirDemoResult, SbirDemoError> {
    ensure_output_directory_is_safe(&options.output_dir)?;
    fs::create_dir_all(&options.output_dir)?;

    let config = PipelineConfig::default();
    let primary_cell = resolve_primary_cell(&options.primary_cell_id)?;
    let primary_data_path = default_nasa_cell_csv_path(&options.data_dir, primary_cell);
    if !primary_data_path.exists() {
        return Err(SbirDemoError::MissingInputCsv(primary_data_path));
    }

    let audit_dir = options
        .output_dir
        .join(AUDIT_TRACES_DIR)
        .join(primary_cell.cell_id);
    fs::create_dir_all(&audit_dir)?;

    let raw_input = load_capacity_csv(&primary_data_path)
        .map_err(|error| SbirDemoError::Message(error.to_string()))?;
    let primary_run = evaluate_cell(
        primary_cell.cell_id,
        primary_data_path.to_string_lossy().as_ref(),
        &raw_input,
        &config,
    )
    .map_err(|error| SbirDemoError::Message(error.to_string()))?;

    let trajectory_path = audit_dir.join(PRIMARY_TRAJECTORY_NAME);
    export_trajectory_csv(&primary_run.trajectory, &trajectory_path)?;
    let supporting_tables = vec![PRIMARY_TRAJECTORY_NAME.to_string()];
    let benchmark_id = format!(
        "reviewer-bundle-{}-capacity",
        primary_cell.cell_id.to_ascii_lowercase()
    );
    let regime_tag = format!(
        "nasa_pcoe_{}_capacity_only",
        primary_cell.cell_id.to_ascii_lowercase()
    );
    let primary_audit_trace = build_stage2_audit_trace(AuditTraceBuildContext {
        results: &primary_run.stage2_results,
        raw_input: &primary_run.raw_data,
        trajectory: &primary_run.trajectory,
        source_artifact: Some(&primary_data_path),
        supporting_figures: &[],
        supporting_tables: &supporting_tables,
        dataset_name: Some("NASA PCoE Battery Dataset"),
        cell_id: Some(primary_cell.cell_id),
        benchmark_id: Some(&benchmark_id),
        regime_tag: Some(&regime_tag),
    })
    .map_err(|error| SbirDemoError::Message(error.to_string()))?;
    let primary_audit_trace_path = audit_dir.join(PRIMARY_AUDIT_JSON_NAME);
    export_audit_trace_json(&primary_audit_trace, &primary_audit_trace_path)?;

    let compliance_output_dir = options.output_dir.join(COMPLIANCE_DIR);
    run_compliance_workflow(
        &options.crate_dir,
        &primary_data_path,
        &compliance_output_dir,
    )
    .map_err(|error| SbirDemoError::Message(error.to_string()))?;

    let addendum_output_dir = options.output_dir.join(ADDENDUM_DIR);
    run_addendum_workflow(&options.crate_dir, &primary_data_path, &addendum_output_dir)
        .map_err(|error| SbirDemoError::Message(error.to_string()))?;

    if options.include_multicell {
        let multicell_output_dir = options.output_dir.join(MULTICELL_DIR);
        run_multicell_workflow(&options.data_dir, &multicell_output_dir, &config)
            .map_err(|error| SbirDemoError::Message(error.to_string()))?;
        copy_bundle_artifact(
            &multicell_output_dir.join(MULTICELL_JSON_NAME),
            &options.output_dir.join(MULTICELL_JSON_NAME),
        )?;
        copy_bundle_artifact(
            &multicell_output_dir.join(MULTICELL_CSV_NAME),
            &options.output_dir.join(MULTICELL_CSV_NAME),
        )?;
    }

    if options.trace_resources {
        let resource_output_dir = options.output_dir.join(RESOURCE_TRACE_DIR);
        run_resource_trace_workflow(
            &options.crate_dir,
            &primary_data_path,
            &resource_output_dir,
            &config,
            options.timing_repeats,
        )
        .map_err(|error| SbirDemoError::Message(error.to_string()))?;
        copy_bundle_artifact(
            &resource_output_dir.join(RESOURCE_TRACE_JSON_NAME),
            &options.output_dir.join(RESOURCE_TRACE_JSON_NAME),
        )?;
        copy_bundle_artifact(
            &resource_output_dir.join(RESOURCE_TRACE_SUMMARY_NAME),
            &options.output_dir.join(RESOURCE_TRACE_SUMMARY_NAME),
        )?;
    }

    #[cfg(feature = "cert-trace")]
    let cert_trace_path = {
        let path = options.output_dir.join(CERT_TRACE_JSON_NAME);
        let artifact = build_cert_trace_artifact(options, primary_cell.cell_id, &config)?;
        write_pretty_json(&artifact, &path)?;
        Some(path)
    };

    let reviewer_summary_path = options.output_dir.join(REVIEWER_SUMMARY_NAME);
    fs::write(
        &reviewer_summary_path,
        build_reviewer_summary(options, primary_cell.cell_id, &primary_audit_trace_path),
    )?;

    let implementation_summary_path = options.output_dir.join(IMPLEMENTATION_SUMMARY_NAME);
    fs::write(
        &implementation_summary_path,
        build_implementation_summary(options),
    )?;

    let input_records = collect_input_records(
        &options.data_dir,
        primary_cell.cell_id,
        options.include_multicell,
    )?;
    let config_hash = hash_bytes_prefixed(&serde_json::to_vec(&config)?);
    let artifact_records = collect_artifact_records(&options.output_dir)?;
    let bundle_manifest = SbirDemoBundleManifest {
        artifact_type: SBIR_BUNDLE_ARTIFACT_TYPE.to_string(),
        schema_version: SBIR_BUNDLE_SCHEMA_VERSION.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        bundle_root: options.output_dir.display().to_string(),
        crate_name: env!("CARGO_PKG_NAME").to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: git_commit(&options.crate_dir),
        worktree_dirty: git_worktree_dirty(&options.crate_dir),
        primary_cell_id: primary_cell.cell_id.to_string(),
        data_dir: options.data_dir.display().to_string(),
        config,
        config_hash,
        inputs: input_records,
        options: SbirDemoRuntimeOptions {
            include_multicell: options.include_multicell,
            trace_resources: options.trace_resources,
            include_addendum: true,
            include_compliance: true,
            timing_repeats: options.timing_repeats.max(1),
            production_figures_regenerated: false,
            production_figures_copied: false,
        },
        artifacts: artifact_records,
        notes: vec![
            "This bundle is advisory-only and reviewer-facing.".to_string(),
            "Existing mono-cell production figures were not regenerated or copied by sbir-demo."
                .to_string(),
            "Compliance and addendum directories reuse existing crate-local helper workflows."
                .to_string(),
            "If resource_trace.json is present, its timing values are measured on the current host only.".to_string(),
            "Markdown summaries were generated in preference to PDF tooling to avoid invasive or fragile report dependencies.".to_string(),
        ],
    };

    let manifest_path = options.output_dir.join(MANIFEST_JSON_NAME);
    write_pretty_json(&bundle_manifest, &manifest_path)?;

    let sha256_manifest_path = options.output_dir.join(MANIFEST_SHA256_NAME);
    write_sha256_manifest(&options.output_dir, &sha256_manifest_path)?;

    #[cfg(feature = "cert-trace")]
    let _ = cert_trace_path;

    Ok(SbirDemoResult {
        output_root: options.output_dir.clone(),
        reviewer_summary_path,
        implementation_summary_path,
        manifest_path,
        sha256_manifest_path,
        primary_audit_trace_path,
        bundle_manifest,
    })
}

#[cfg(feature = "cert-trace")]
fn build_cert_trace_artifact(
    options: &SbirDemoOptions,
    primary_cell_id: &str,
    config: &PipelineConfig,
) -> Result<SbirCertTraceArtifact, SbirDemoError> {
    let config_identifier = hash_bytes_prefixed(&serde_json::to_vec(config)?);
    let mut evidence_records = vec![
        SbirCertTraceEvidenceRecord {
            tag: "REQ-advisory-interface".to_string(),
            artifact: format!("{AUDIT_TRACES_DIR}/{primary_cell_id}/{PRIMARY_AUDIT_JSON_NAME}"),
            note: "Read-only, advisory-only, and fail-silent interface fields are exposed in the primary audit trace."
                .to_string(),
        },
        SbirCertTraceEvidenceRecord {
            tag: "EVID-traceability".to_string(),
            artifact: format!("{COMPLIANCE_DIR}/stc_traceability_support.json"),
            note: "Configuration, input, and deterministic summary hashes are emitted by the compliance helper."
                .to_string(),
        },
        SbirCertTraceEvidenceRecord {
            tag: "EVID-integrity".to_string(),
            artifact: format!("{ADDENDUM_DIR}/integrity/tamper_evident_trace.json"),
            note: "Tamper-evident residual chaining remains an addendum helper and is not a certification claim."
                .to_string(),
        },
        SbirCertTraceEvidenceRecord {
            tag: "EVID-ffi-boundary".to_string(),
            artifact: "include/dsfb_battery_ffi.h".to_string(),
            note: "The narrow staticlib-facing C ABI remains the integration boundary.".to_string(),
        },
    ];

    if options.trace_resources {
        evidence_records.push(SbirCertTraceEvidenceRecord {
            tag: "EVID-resource-trace".to_string(),
            artifact: RESOURCE_TRACE_JSON_NAME.to_string(),
            note: "Host-side timing and size fields are reported with explicit measured or estimated modes.".to_string(),
        });
    }

    Ok(SbirCertTraceArtifact {
        artifact_type: "dsfb_battery_cert_trace".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "certification_facing_mapping_only".to_string(),
        configuration_identifier: config_identifier,
        primary_cell_id: primary_cell_id.to_string(),
        evidence_records,
        interface_contract_notes: vec![
            "This feature adds traceability-oriented fields and evidence pointers for reviewer-facing output only.".to_string(),
            "No compliance, certification, approval, or qualification claim is made.".to_string(),
        ],
        notes: vec![
            "cert-trace is disabled by default and does not change the production mono-cell path.".to_string(),
            "The evidence list maps existing artifacts into a certification-facing discussion scaffold.".to_string(),
        ],
    })
}

fn resolve_primary_cell(
    cell_id: &str,
) -> Result<&'static crate::nasa::NasaPcoeCellSpec, SbirDemoError> {
    supported_nasa_pcoe_cells()
        .iter()
        .find(|cell| cell.cell_id.eq_ignore_ascii_case(cell_id))
        .ok_or_else(|| SbirDemoError::UnsupportedPrimaryCell {
            cell_id: cell_id.to_string(),
            supported: supported_nasa_pcoe_cells()
                .iter()
                .map(|cell| cell.cell_id.to_string())
                .collect(),
        })
}

fn unique_named_output_dir(root: &Path, stem: &str) -> PathBuf {
    let candidate = root.join(stem);
    if !candidate.exists() {
        return candidate;
    }
    for suffix in 1.. {
        let retry = root.join(format!("{stem}_r{suffix}"));
        if !retry.exists() {
            return retry;
        }
    }
    unreachable!("unbounded retry loop for sbir demo output directory")
}

fn ensure_output_directory_is_safe(path: &Path) -> Result<(), SbirDemoError> {
    if path.exists() {
        let mut entries = fs::read_dir(path)?;
        if entries.next().transpose()?.is_some() {
            return Err(SbirDemoError::NonEmptyOutputDirectory(path.to_path_buf()));
        }
    }
    Ok(())
}

fn copy_bundle_artifact(source: &Path, destination: &Path) -> Result<(), SbirDemoError> {
    if !source.exists() {
        return Err(SbirDemoError::MissingBundleArtifact(
            source.display().to_string(),
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn collect_input_records(
    data_dir: &Path,
    primary_cell_id: &str,
    include_multicell: bool,
) -> Result<Vec<SbirDemoInputRecord>, SbirDemoError> {
    let mut cells: Vec<&str> = if include_multicell {
        supported_nasa_pcoe_cells()
            .iter()
            .map(|cell| cell.cell_id)
            .collect()
    } else {
        vec![primary_cell_id]
    };
    cells.sort_unstable();
    cells.dedup();

    let mut records = Vec::new();
    for cell_id in cells {
        let Some(cell_spec) = supported_nasa_pcoe_cells()
            .iter()
            .find(|cell| cell.cell_id.eq_ignore_ascii_case(cell_id))
        else {
            continue;
        };
        let path = default_nasa_cell_csv_path(data_dir, cell_spec);
        if !path.exists() {
            continue;
        }
        records.push(SbirDemoInputRecord {
            cell_id: cell_spec.cell_id.to_string(),
            role: if cell_spec.cell_id.eq_ignore_ascii_case(primary_cell_id) {
                "primary_cell".to_string()
            } else {
                "multicell_reference".to_string()
            },
            path: path.display().to_string(),
            sha256: hash_file_prefixed(&path)?,
        });
    }

    Ok(records)
}

fn collect_artifact_records(
    output_root: &Path,
) -> Result<Vec<SbirDemoArtifactRecord>, SbirDemoError> {
    let mut relative_paths = Vec::new();
    collect_relative_file_paths(output_root, output_root, &mut relative_paths)?;
    relative_paths.sort();

    let mut artifacts = Vec::new();
    for relative_path in relative_paths {
        if relative_path == MANIFEST_JSON_NAME || relative_path == MANIFEST_SHA256_NAME {
            continue;
        }
        let artifact_path = output_root.join(&relative_path);
        artifacts.push(SbirDemoArtifactRecord {
            path: relative_path.clone(),
            sha256: hash_file_prefixed(&artifact_path)?,
            artifact_role: artifact_role_for_path(&relative_path),
            provenance: provenance_for_path(&relative_path),
            description: description_for_path(&relative_path),
        });
    }

    Ok(artifacts)
}

fn collect_relative_file_paths(
    root: &Path,
    dir: &Path,
    files: &mut Vec<String>,
) -> Result<(), SbirDemoError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_file_paths(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            files.push(relative);
        }
    }
    Ok(())
}

fn write_sha256_manifest(output_root: &Path, manifest_path: &Path) -> Result<(), SbirDemoError> {
    let mut relative_paths = Vec::new();
    collect_relative_file_paths(output_root, output_root, &mut relative_paths)?;
    relative_paths.sort();

    let mut lines = Vec::new();
    for relative_path in relative_paths {
        if relative_path == MANIFEST_SHA256_NAME {
            continue;
        }
        let file_path = output_root.join(&relative_path);
        let digest = hash_file_hex(&file_path)?;
        lines.push(format!("{digest}  {relative_path}"));
    }

    fs::write(manifest_path, lines.join("\n"))?;
    Ok(())
}

fn artifact_role_for_path(relative_path: &str) -> String {
    if relative_path == REVIEWER_SUMMARY_NAME {
        return "reviewer_summary".to_string();
    }
    if relative_path == IMPLEMENTATION_SUMMARY_NAME {
        return "implementation_summary".to_string();
    }
    if relative_path == MULTICELL_JSON_NAME || relative_path == MULTICELL_CSV_NAME {
        return "multicell_index".to_string();
    }
    if relative_path == RESOURCE_TRACE_JSON_NAME || relative_path == RESOURCE_TRACE_SUMMARY_NAME {
        return "resource_trace_index".to_string();
    }
    if relative_path.ends_with(PRIMARY_AUDIT_JSON_NAME) {
        return "audit_trace".to_string();
    }
    if relative_path.ends_with(PRIMARY_TRAJECTORY_NAME) {
        return "audit_trajectory".to_string();
    }
    if relative_path.starts_with(COMPLIANCE_DIR) {
        return "compliance_support".to_string();
    }
    if relative_path.starts_with(ADDENDUM_DIR) {
        return "addendum_support".to_string();
    }
    if relative_path.starts_with(MULTICELL_DIR) {
        return "multicell_support".to_string();
    }
    if relative_path.starts_with(RESOURCE_TRACE_DIR) {
        return "resource_trace_support".to_string();
    }
    #[cfg(feature = "cert-trace")]
    if relative_path == CERT_TRACE_JSON_NAME {
        return "cert_trace".to_string();
    }
    "bundle_support".to_string()
}

fn provenance_for_path(relative_path: &str) -> String {
    if relative_path == MULTICELL_JSON_NAME || relative_path == MULTICELL_CSV_NAME {
        return format!("copied from {MULTICELL_DIR}/...");
    }
    if relative_path == RESOURCE_TRACE_JSON_NAME || relative_path == RESOURCE_TRACE_SUMMARY_NAME {
        return format!("copied from {RESOURCE_TRACE_DIR}/...");
    }
    if relative_path.starts_with(COMPLIANCE_DIR) {
        return "generated by run_compliance_workflow".to_string();
    }
    if relative_path.starts_with(ADDENDUM_DIR) {
        return "generated by run_addendum_workflow".to_string();
    }
    if relative_path.starts_with(MULTICELL_DIR) {
        return "generated by run_multicell_workflow".to_string();
    }
    if relative_path.starts_with(RESOURCE_TRACE_DIR) {
        return "generated by run_resource_trace_workflow".to_string();
    }
    if relative_path.starts_with(AUDIT_TRACES_DIR) {
        return "generated from evaluate_cell plus build_stage2_audit_trace".to_string();
    }
    if relative_path == REVIEWER_SUMMARY_NAME || relative_path == IMPLEMENTATION_SUMMARY_NAME {
        return "generated by sbir-demo bundle orchestration".to_string();
    }
    #[cfg(feature = "cert-trace")]
    if relative_path == CERT_TRACE_JSON_NAME {
        return "generated by cert-trace bundle feature".to_string();
    }
    "generated by sbir-demo bundle orchestration".to_string()
}

fn description_for_path(relative_path: &str) -> String {
    if relative_path == MULTICELL_JSON_NAME {
        return "Convenience copy of the multi-cell summary JSON.".to_string();
    }
    if relative_path == MULTICELL_CSV_NAME {
        return "Convenience copy of the multi-cell summary CSV.".to_string();
    }
    if relative_path == RESOURCE_TRACE_JSON_NAME {
        return "Convenience copy of the host-side resource trace JSON.".to_string();
    }
    if relative_path == RESOURCE_TRACE_SUMMARY_NAME {
        return "Convenience copy of the host-side resource trace text summary.".to_string();
    }
    if relative_path == REVIEWER_SUMMARY_NAME {
        return "Reviewer-facing markdown overview of the bundle contents and scope.".to_string();
    }
    if relative_path == IMPLEMENTATION_SUMMARY_NAME {
        return "Run-local implementation summary and scope statement.".to_string();
    }
    if relative_path.ends_with(PRIMARY_AUDIT_JSON_NAME) {
        return "Primary cell advisory audit trace contract.".to_string();
    }
    if relative_path.ends_with(PRIMARY_TRAJECTORY_NAME) {
        return "Primary cell semiotic trajectory CSV.".to_string();
    }
    #[cfg(feature = "cert-trace")]
    if relative_path == CERT_TRACE_JSON_NAME {
        return "Optional certification-facing traceability mapping scaffold.".to_string();
    }
    "Supporting bundle artifact.".to_string()
}

fn build_reviewer_summary(
    options: &SbirDemoOptions,
    primary_cell_id: &str,
    primary_audit_trace_path: &Path,
) -> String {
    let mut lines = Vec::new();
    lines.push("# SBIR Reviewer Summary".to_string());
    lines.push(String::new());
    lines.push("This bundle is a conservative orchestration layer over the existing `dsfb-battery` helper workflows. It remains read-only, advisory-only, and reviewer-facing.".to_string());
    lines.push(String::new());
    lines.push("## Run".to_string());
    lines.push(format!("- Primary cell: `{primary_cell_id}`"));
    lines.push(format!(
        "- Multi-cell comparison: {}",
        if options.include_multicell {
            "included"
        } else {
            "not requested"
        }
    ));
    lines.push(format!(
        "- Resource trace: {}",
        if options.trace_resources {
            "included"
        } else {
            "not requested"
        }
    ));
    lines.push("- Compliance support: included".to_string());
    lines.push("- Addendum support: included".to_string());
    lines.push("- Production figures: not regenerated and not copied into this bundle".to_string());
    lines.push(String::new());
    lines.push("## Key Artifacts".to_string());
    lines.push(format!(
        "- `{}`: primary cell audit trace",
        relative_display(&options.output_dir, primary_audit_trace_path)
    ));
    lines.push(format!(
        "- `{AUDIT_TRACES_DIR}/{primary_cell_id}/{PRIMARY_TRAJECTORY_NAME}`: primary cell semiotic trajectory"
    ));
    if options.include_multicell {
        lines.push(format!(
            "- `{MULTICELL_JSON_NAME}` and `{MULTICELL_CSV_NAME}`: reviewer-facing multi-cell comparison copies"
        ));
    }
    if options.trace_resources {
        lines.push(format!(
            "- `{RESOURCE_TRACE_JSON_NAME}` and `{RESOURCE_TRACE_SUMMARY_NAME}`: current-host resource trace copies"
        ));
    }
    lines.push(format!(
        "- `{COMPLIANCE_DIR}/implementation_summary.txt`: compliance/support mapping summary"
    ));
    lines.push(format!(
        "- `{ADDENDUM_DIR}/implementation_summary.txt`: addendum/integrity support summary"
    ));
    lines.push(format!(
        "- `{MANIFEST_JSON_NAME}` and `{MANIFEST_SHA256_NAME}`: reproducibility manifest and file hashes"
    ));
    #[cfg(feature = "cert-trace")]
    lines.push(format!(
        "- `{CERT_TRACE_JSON_NAME}`: optional certification-facing traceability scaffold"
    ));
    lines.push(String::new());
    lines.push("## Evidence Scope".to_string());
    lines.push("- Measured: resource-trace timing and exact file-size values are only reported when `--trace-resources` is enabled, and they remain host-environment measurements.".to_string());
    lines.push("- Estimated: hot-loop state bytes, loaded-bank bytes, and some SWaP-C notes are estimates rather than target-hardware measurements.".to_string());
    lines.push("- Documented or mapped only: compliance tables, commercial templates, and embedded-integration notes support review and planning; they are not approvals or certification artifacts.".to_string());
    lines.push(String::new());
    lines.push("## Protection Gates".to_string());
    lines.push("- The existing `dsfb-battery-demo` binary was left untouched.".to_string());
    lines.push(
        "- The current mono-cell production figure basenames and output path were not reused."
            .to_string(),
    );
    lines.push("- This bundle writes only inside its own output directory.".to_string());
    lines.push("- No PDF report was forced into the pipeline; markdown summaries are used instead of fragile report tooling.".to_string());
    lines.join("\n")
}

fn build_implementation_summary(options: &SbirDemoOptions) -> String {
    let mut lines = Vec::new();
    lines.push("SBIR demo implementation summary".to_string());
    lines.push("Added in this crate revision:".to_string());
    lines.push(
        "- reviewer-facing `sbir-demo` orchestration binary and supporting library workflow"
            .to_string(),
    );
    lines.push(
        "- reproducibility manifest and SHA-256 manifest generation for the reviewer bundle"
            .to_string(),
    );
    lines
        .push("- crate-local embedded integration guide and commercial template files".to_string());
    lines.push(
        "- additional C wrapper example under wrappers/c using the existing staticlib boundary"
            .to_string(),
    );
    lines.push("What sbir-demo does:".to_string());
    lines.push(
        "- evaluates a selected NASA PCoE cell with the existing audit-trace path".to_string(),
    );
    lines.push("- runs the existing compliance and addendum helper workflows into bundle-local subdirectories".to_string());
    lines.push("- optionally runs the existing multicell and resource-trace helpers".to_string());
    lines.push("- writes reviewer-facing markdown plus reproducibility manifests in a new isolated output root".to_string());
    lines.push("Artifacts emitted by this run may include:".to_string());
    lines.push("- audit_traces/<cell>/stage2_detection_results.json".to_string());
    lines.push("- audit_traces/<cell>/semiotic_trajectory.csv".to_string());
    lines.push("- compliance/...".to_string());
    lines.push("- addendum/...".to_string());
    if options.include_multicell {
        lines.push("- multicell_summary.json".to_string());
        lines.push("- multicell_summary.csv".to_string());
        lines.push("- multicell/...".to_string());
    }
    if options.trace_resources {
        lines.push("- resource_trace.json".to_string());
        lines.push("- resource_trace_summary.txt".to_string());
        lines.push("- resource_trace/...".to_string());
    }
    lines.push("- reviewer_summary.md".to_string());
    lines.push("- manifest.json".to_string());
    lines.push("- manifest.sha256".to_string());
    lines.push("Measured vs estimated vs documented:".to_string());
    lines.push(
        "- measured values remain limited to current-host resource tracing when enabled"
            .to_string(),
    );
    lines.push(
        "- estimated values remain limited to existing resource and SWaP-C helper outputs"
            .to_string(),
    );
    lines.push(
        "- documented or mapped items remain explicitly advisory-only and support-oriented"
            .to_string(),
    );
    lines.push("FFI/staticlib status: existing staticlib support was retained and a small wrapper example was added; no broad FFI refactor was performed.".to_string());
    lines.push(format!(
        "cert-trace feature: {}",
        if cfg!(feature = "cert-trace") {
            "added and enabled in this build"
        } else {
            "added and disabled by default in this build"
        }
    ));
    lines.push("Top-level repository README / CI: not updated, to keep the change set crate-local and avoid repo-wide churn.".to_string());
    lines.push(
        "Confirmation: the mono-cell production figure generation path was not modified."
            .to_string(),
    );
    lines.join("\n")
}

fn git_commit(crate_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(crate_dir)
        .arg("rev-parse")
        .arg("--verify")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn git_worktree_dirty(crate_dir: &Path) -> Option<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(crate_dir)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg(".")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!output.stdout.is_empty())
}

fn write_pretty_json<T: Serialize>(value: &T, path: &Path) -> Result<(), SbirDemoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn hash_file_prefixed(path: &Path) -> Result<String, SbirDemoError> {
    Ok(format!("sha256:{}", hash_file_hex(path)?))
}

fn hash_file_hex(path: &Path) -> Result<String, SbirDemoError> {
    let bytes = fs::read(path)?;
    Ok(hash_hex(&bytes))
}

fn hash_bytes_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", hash_hex(bytes))
}

fn hash_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{unique}"))
    }

    fn crate_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn write_cell_csv(dir: &Path, cell_id: &str, base: f64, slope: f64) {
        let path = dir.join(format!("nasa_{}_capacity.csv", cell_id.to_lowercase()));
        let mut writer = csv::Writer::from_path(path).unwrap();
        writer
            .write_record(["cycle", "capacity_ah", "type"])
            .unwrap();
        for cycle in 1..=60 {
            let healthy_noise = 0.001 * ((cycle as f64) * 0.37).sin();
            let degradation = if cycle > 25 {
                slope * (cycle - 25) as f64
            } else {
                0.0
            };
            let capacity = base + healthy_noise - degradation;
            writer
                .write_record([
                    cycle.to_string(),
                    format!("{capacity:.6}"),
                    "discharge".to_string(),
                ])
                .unwrap();
        }
        writer.flush().unwrap();
    }

    fn make_test_data_dir() -> PathBuf {
        let dir = unique_temp_dir("dsfb-battery-sbir-data");
        fs::create_dir_all(&dir).unwrap();
        write_cell_csv(&dir, "B0005", 2.000, 0.010);
        write_cell_csv(&dir, "B0006", 2.050, 0.011);
        write_cell_csv(&dir, "B0007", 1.950, 0.009);
        write_cell_csv(&dir, "B0018", 1.900, 0.012);
        dir
    }

    fn default_options(data_dir: &Path, output_dir: &Path) -> SbirDemoOptions {
        SbirDemoOptions {
            crate_dir: crate_dir(),
            data_dir: data_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            primary_cell_id: "B0005".to_string(),
            include_multicell: false,
            trace_resources: false,
            timing_repeats: 1,
        }
    }

    fn collect_output_names(root: &Path) -> Vec<String> {
        let mut paths = Vec::new();
        collect_relative_file_paths(root, root, &mut paths).unwrap();
        paths
    }

    #[test]
    fn sbir_output_defaults_to_isolated_root() {
        let resolved = resolve_sbir_demo_output_dir(&crate_dir(), None);
        assert!(resolved
            .to_string_lossy()
            .contains("outputs/sbir_demo/dsfb_battery_sbir_demo_"));
    }

    #[test]
    fn sbir_output_treats_single_component_override_as_bundle_name() {
        let resolved =
            resolve_sbir_demo_output_dir(&crate_dir(), Some(PathBuf::from("reviewer-bundle")));
        assert!(resolved
            .to_string_lossy()
            .ends_with("outputs/sbir_demo/reviewer-bundle"));
    }

    #[test]
    fn sbir_demo_refuses_to_overwrite_non_empty_directory() {
        let data_dir = make_test_data_dir();
        let output_dir = unique_temp_dir("dsfb-battery-sbir-output");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(output_dir.join("keep.txt"), "existing").unwrap();
        let options = default_options(&data_dir, &output_dir);

        let error = run_sbir_demo_workflow(&options).unwrap_err();
        assert!(matches!(error, SbirDemoError::NonEmptyOutputDirectory(_)));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn sbir_demo_default_bundle_keeps_optional_outputs_opt_in() {
        let data_dir = make_test_data_dir();
        let output_dir = unique_temp_dir("dsfb-battery-sbir-output");
        let options = default_options(&data_dir, &output_dir);

        let result = run_sbir_demo_workflow(&options).unwrap();

        assert!(result
            .primary_audit_trace_path
            .ends_with(Path::new(&format!(
                "{AUDIT_TRACES_DIR}/B0005/{PRIMARY_AUDIT_JSON_NAME}"
            ))));
        assert!(output_dir.join(REVIEWER_SUMMARY_NAME).exists());
        assert!(output_dir.join(IMPLEMENTATION_SUMMARY_NAME).exists());
        assert!(output_dir.join(MANIFEST_JSON_NAME).exists());
        assert!(output_dir.join(MANIFEST_SHA256_NAME).exists());
        assert!(output_dir.join(COMPLIANCE_DIR).exists());
        assert!(output_dir.join(ADDENDUM_DIR).exists());
        assert!(!output_dir.join(MULTICELL_JSON_NAME).exists());
        assert!(!output_dir.join(RESOURCE_TRACE_JSON_NAME).exists());
        assert!(!output_dir.join(RESOURCE_TRACE_SUMMARY_NAME).exists());
        #[cfg(not(feature = "cert-trace"))]
        assert!(!output_dir.join(CERT_TRACE_JSON_NAME).exists());

        let output_names = collect_output_names(&output_dir);
        assert!(!output_names
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));

        let reviewer_summary = fs::read_to_string(output_dir.join(REVIEWER_SUMMARY_NAME)).unwrap();
        assert!(reviewer_summary.contains("advisory-only"));
        assert!(reviewer_summary.contains("Production figures: not regenerated"));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn sbir_demo_optional_bundle_generates_multicell_and_resource_trace_outputs() {
        let data_dir = make_test_data_dir();
        let output_dir = unique_temp_dir("dsfb-battery-sbir-output");
        let mut options = default_options(&data_dir, &output_dir);
        options.primary_cell_id = "B0006".to_string();
        options.include_multicell = true;
        options.trace_resources = true;

        let result = run_sbir_demo_workflow(&options).unwrap();

        assert!(output_dir.join(MULTICELL_JSON_NAME).exists());
        assert!(output_dir.join(MULTICELL_CSV_NAME).exists());
        assert!(output_dir.join(RESOURCE_TRACE_JSON_NAME).exists());
        assert!(output_dir.join(RESOURCE_TRACE_SUMMARY_NAME).exists());
        assert!(output_dir
            .join(AUDIT_TRACES_DIR)
            .join("B0006")
            .join(PRIMARY_AUDIT_JSON_NAME)
            .exists());

        let manifest = fs::read_to_string(&result.manifest_path).unwrap();
        assert!(manifest.contains("\"primary_cell_id\": \"B0006\""));
        assert!(manifest.contains(MULTICELL_JSON_NAME));
        assert!(manifest.contains(RESOURCE_TRACE_JSON_NAME));

        let sha_manifest = fs::read_to_string(&result.sha256_manifest_path).unwrap();
        assert!(sha_manifest.contains(MANIFEST_JSON_NAME));
        assert!(sha_manifest.contains(MULTICELL_JSON_NAME));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn sha256_manifest_covers_manifest_and_reviewer_summary() {
        let data_dir = make_test_data_dir();
        let output_dir = unique_temp_dir("dsfb-battery-sbir-output");
        let options = default_options(&data_dir, &output_dir);

        let result = run_sbir_demo_workflow(&options).unwrap();
        let sha_manifest = fs::read_to_string(&result.sha256_manifest_path).unwrap();
        let lines: Vec<&str> = sha_manifest.lines().collect();
        assert!(lines.iter().any(|line| line.ends_with(MANIFEST_JSON_NAME)));
        assert!(lines
            .iter()
            .any(|line| line.ends_with(REVIEWER_SUMMARY_NAME)));

        let manifest_line = lines
            .iter()
            .find(|line| line.ends_with(MANIFEST_JSON_NAME))
            .unwrap();
        let expected = hash_file_hex(&result.manifest_path).unwrap();
        assert!(manifest_line.starts_with(&expected));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn docs_and_readme_keep_conservative_language() {
        let readme = fs::read_to_string(crate_dir().join("README.md")).unwrap();
        let embedded =
            fs::read_to_string(crate_dir().join("docs/embedded-integration.md")).unwrap();
        let commercial = fs::read_to_string(crate_dir().join("LICENSE-COMMERCIAL.md")).unwrap();

        for text in [&readme, &embedded, &commercial] {
            assert!(!text.contains("flight-ready"));
            assert!(!text.contains("instant deployment"));
            assert!(!text.contains("FAA approval"));
            assert!(!text.contains("EASA approval"));
            assert!(!text.contains("guarantees"));
        }
        assert!(readme.contains("For SBIR Operators"));
        assert!(embedded.contains("advisory"));
        assert!(commercial.contains("Apache 2.0"));
    }

    #[test]
    fn cargo_manifest_retains_existing_bins_and_staticlib() {
        let cargo_toml = fs::read_to_string(crate_dir().join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("crate-type = [\"rlib\", \"staticlib\"]"));
        assert!(cargo_toml.contains("name = \"dsfb-battery-demo\""));
        assert!(cargo_toml.contains("name = \"dsfb-battery-multicell\""));
        assert!(cargo_toml.contains("name = \"dsfb-battery-resource-trace\""));
        assert!(cargo_toml.contains("name = \"sbir-demo\""));
    }

    #[cfg(feature = "cert-trace")]
    #[test]
    fn cert_trace_output_is_feature_gated() {
        let data_dir = make_test_data_dir();
        let output_dir = unique_temp_dir("dsfb-battery-sbir-output");
        let options = default_options(&data_dir, &output_dir);

        run_sbir_demo_workflow(&options).unwrap();
        assert!(output_dir.join(CERT_TRACE_JSON_NAME).exists());

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }
}
