// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Isolated compliance/support layer for standards mapping, static scanning,
// and operator-overlay helper outputs. This module is std-only and does not
// modify the production mono-cell workflow.

use crate::detection::{
    build_dsfb_detection, build_threshold_detection, run_dsfb_pipeline, verify_theorem1,
};
use crate::load_b0005_csv;
use crate::types::{BatteryResidual, GrammarState, PipelineConfig, ReasonCode};
use chrono::{Local, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const COMPLIANCE_ARTIFACT_TYPE: &str = "dsfb_battery_compliance_support";
const SAFE_RUST_AUDIT_TYPE: &str = "dsfb_battery_safe_rust_audit";
const DETERMINISM_CHECK_TYPE: &str = "dsfb_battery_determinism_check";
const STC_TRACEABILITY_TYPE: &str = "dsfb_battery_stc_traceability_support";
const OPERATOR_OVERLAY_TYPE: &str = "dsfb_battery_operator_overlay";
const IMPLEMENTATION_SUMMARY_STATEMENT: &str = "No production code or figures were modified";
const PRODUCTION_ARTIFACTS_UNCHANGED_NOTE: &str =
    "The compliance helper writes only to its own output directory and does not emit production figure names or stage-II production artifact names.";
const THRESHOLD_BASELINE_FRACTION: f64 = 0.85;

#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("input series is empty")]
    EmptyInput,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("detection error: {0}")]
    Detection(#[from] crate::detection::DetectionError),
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardStatus {
    Supported,
    Partial,
    NotSupported,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StandardStatusRecord {
    pub standard: String,
    pub status: StandardStatus,
    pub mapping_artifact: String,
    pub component_refs: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SafeRustFinding {
    pub path: String,
    pub line: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeRustAudit {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub classification: StandardStatus,
    pub core_engine_paths: Vec<String>,
    pub unsafe_hits_all_src: Vec<SafeRustFinding>,
    pub unsafe_hits_core_boundary: Vec<SafeRustFinding>,
    pub dynamic_allocation_hits_core: Vec<SafeRustFinding>,
    pub recursion_hits_all_src: Vec<SafeRustFinding>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeterminismCheckArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub classification: StandardStatus,
    pub summary_hash_run_1: String,
    pub summary_hash_run_2: String,
    pub repeated_run_equal: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StcTraceabilitySupport {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub classification: StandardStatus,
    pub config_hash: String,
    pub input_hash: String,
    pub summary_hash: String,
    pub reproducibility_equal: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColorLegendEntry {
    pub state: GrammarState,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorOverlayRow {
    pub cycle: usize,
    pub state: GrammarState,
    pub tri_state_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<ReasonCode>,
    pub advisory_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorOverlaySummary {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub classification: StandardStatus,
    pub final_state: GrammarState,
    pub final_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_reason_code: Option<ReasonCode>,
    pub legend: Vec<ColorLegendEntry>,
    pub rows: usize,
    pub advisory_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceImplementationSummary {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub standards_covered: Vec<StandardStatusRecord>,
    pub artifacts_generated: Vec<String>,
    pub mapped_only: Vec<String>,
    pub scaffolded: Vec<String>,
    pub output_root: String,
    pub no_production_modification_statement: String,
}

#[derive(Debug, Clone, Serialize)]
struct DeterministicSummary {
    dsfb_alarm_cycle: Option<usize>,
    threshold_alarm_cycle: Option<usize>,
    first_boundary_cycle: Option<usize>,
    first_violation_cycle: Option<usize>,
    final_state: GrammarState,
    t_star: usize,
}

pub fn resolve_compliance_output_dir(crate_dir: &Path, explicit_output: Option<PathBuf>) -> PathBuf {
    if let Some(output) = explicit_output {
        return output;
    }
    let root = crate_dir.join("outputs").join("compliance");
    let stem = format!(
        "dsfb_battery_compliance_{}",
        Local::now().format("%Y%m%d_%H%M%S")
    );
    unique_named_output_dir(&root, &stem)
}

fn unique_named_output_dir(root: &Path, stem: &str) -> PathBuf {
    let candidate = root.join(stem);
    if !candidate.exists() {
        return candidate;
    }
    for suffix in 1.. {
        let retry = root.join(format!("{}_r{}", stem, suffix));
        if !retry.exists() {
            return retry;
        }
    }
    unreachable!("unbounded retry loop for compliance output directory")
}

pub fn run_compliance_workflow(
    crate_dir: &Path,
    data_path: &Path,
    output_dir: &Path,
) -> Result<ComplianceImplementationSummary, ComplianceError> {
    let raw_input = load_b0005_csv(data_path)
        .map_err(|error| ComplianceError::Io(std::io::Error::other(error.to_string())))?;
    run_compliance_workflow_from_input(crate_dir, &raw_input, output_dir, data_path)
}

pub fn run_compliance_workflow_from_input(
    crate_dir: &Path,
    raw_input: &[(usize, f64)],
    output_dir: &Path,
    source_path: &Path,
) -> Result<ComplianceImplementationSummary, ComplianceError> {
    if raw_input.is_empty() {
        return Err(ComplianceError::EmptyInput);
    }

    fs::create_dir_all(output_dir)?;

    let capacities: Vec<f64> = raw_input.iter().map(|(_, capacity)| *capacity).collect();
    let config = PipelineConfig::default();
    let (envelope, trajectory) = run_dsfb_pipeline(&capacities, &config)?;
    let eol_capacity = config.eol_fraction * capacities[0];
    let dsfb_detection = build_dsfb_detection(&trajectory, &capacities, eol_capacity);
    let threshold_detection =
        build_threshold_detection(&capacities, THRESHOLD_BASELINE_FRACTION, eol_capacity);
    let theorem1 = verify_theorem1(&envelope, &trajectory, &config);

    let safe_rust_audit = scan_safe_rust_subset(crate_dir)?;
    let determinism_check = build_determinism_check(&capacities, &config)?;
    let stc_support = build_stc_traceability_support(
        raw_input,
        &config,
        &trajectory,
        &dsfb_detection,
        &threshold_detection,
        theorem1.t_star,
        determinism_check.repeated_run_equal,
    )?;
    let (overlay_summary, overlay_rows) = build_operator_overlay(&trajectory);
    let standards = standards_status_records();

    let operator_overlay_dir = output_dir.join("operator_overlay");
    fs::create_dir_all(&operator_overlay_dir)?;

    write_json(
        &safe_rust_audit,
        &output_dir.join("safe_rust_audit.json"),
    )?;
    write_text(
        &render_misra_equivalent_report(&safe_rust_audit),
        &output_dir.join("misra_equivalent_report.txt"),
    )?;
    write_json(
        &determinism_check,
        &output_dir.join("determinism_check.json"),
    )?;
    write_json(
        &stc_support,
        &output_dir.join("stc_traceability_support.json"),
    )?;
    write_json(
        &overlay_summary,
        &operator_overlay_dir.join("operator_overlay_summary.json"),
    )?;
    write_operator_overlay_csv(
        &overlay_rows,
        &operator_overlay_dir.join("operator_overlay_timeline.csv"),
    )?;
    write_standards_status_csv(
        &standards,
        &output_dir.join("standards_status_matrix.csv"),
    )?;

    let summary = ComplianceImplementationSummary {
        artifact_type: COMPLIANCE_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        standards_covered: standards,
        artifacts_generated: vec![
            "misra_equivalent_report.txt".to_string(),
            "safe_rust_audit.json".to_string(),
            "determinism_check.json".to_string(),
            "stc_traceability_support.json".to_string(),
            "standards_status_matrix.csv".to_string(),
            "operator_overlay/operator_overlay_summary.json".to_string(),
            "operator_overlay/operator_overlay_timeline.csv".to_string(),
        ],
        mapped_only: vec![
            "FACE UoP alignment".to_string(),
            "DO-178C DAL-C alignment".to_string(),
            "MIL-STD-882E hazard mapping".to_string(),
            "IEC 61508 compatibility mapping".to_string(),
            "IEEE 1547 cease-to-energize advisory mapping".to_string(),
            "ISO 15926, ISO/IEC 25010, and W3C SSN/SOSA semantic mappings".to_string(),
        ],
        scaffolded: vec![
            "Compliance documents provide mapping support only and do not claim certification or approval.".to_string(),
            "Structured Text output is a deterministic translation of the advisory state machine and not a deployed PLC package.".to_string(),
        ],
        output_root: output_dir.display().to_string(),
        no_production_modification_statement: IMPLEMENTATION_SUMMARY_STATEMENT.to_string(),
    };

    write_text(
        &render_implementation_summary(
            &summary,
            source_path,
            dsfb_detection.alarm_cycle,
            threshold_detection.alarm_cycle,
            theorem1.t_star,
        ),
        &output_dir.join("implementation_summary.txt"),
    )?;

    Ok(summary)
}

pub fn render_misra_equivalent_report(audit: &SafeRustAudit) -> String {
    let mut lines = vec![
        "MISRA-Equivalent Safe Rust Report".to_string(),
        "".to_string(),
        "Scope: heuristic static scan over the dsfb-battery crate source tree; this is not a formal Ferrocene qualification report or MISRA approval artifact.".to_string(),
        format!("Classification: {:?}", audit.classification).to_lowercase().replace("standardstatus::", ""),
        format!("Core engine paths: {}", audit.core_engine_paths.join(", ")),
        "".to_string(),
        format!(
            "Unsafe hits across src/: {}",
            audit.unsafe_hits_all_src.len()
        ),
        format!(
            "Unsafe hits in the core boundary set: {}",
            audit.unsafe_hits_core_boundary.len()
        ),
        format!(
            "Dynamic-allocation pattern hits in the core boundary set: {}",
            audit.dynamic_allocation_hits_core.len()
        ),
        format!(
            "Direct-recursion hits across src/: {}",
            audit.recursion_hits_all_src.len()
        ),
        "".to_string(),
        "Findings:".to_string(),
    ];

    if audit.unsafe_hits_all_src.is_empty() {
        lines.push("- No unsafe tokens were found in src/ by the current heuristic scan.".to_string());
    } else {
        for finding in &audit.unsafe_hits_all_src {
            lines.push(format!(
                "- unsafe: {}:{} -> {}",
                finding.path, finding.line, finding.snippet
            ));
        }
    }

    if audit.dynamic_allocation_hits_core.is_empty() {
        lines.push("- No dynamic-allocation patterns were found in the core boundary set.".to_string());
    } else {
        for finding in &audit.dynamic_allocation_hits_core {
            lines.push(format!(
                "- alloc-pattern: {}:{} -> {}",
                finding.path, finding.line, finding.snippet
            ));
        }
    }

    if audit.recursion_hits_all_src.is_empty() {
        lines.push("- No direct recursion was found in src/ by the current heuristic scan.".to_string());
    } else {
        for finding in &audit.recursion_hits_all_src {
            lines.push(format!(
                "- recursion: {}:{} -> {}",
                finding.path, finding.line, finding.snippet
            ));
        }
    }

    lines.push("".to_string());
    lines.push("Notes:".to_string());
    for note in &audit.notes {
        lines.push(format!("- {}", note));
    }

    lines.join("\n")
}

fn standards_status_records() -> Vec<StandardStatusRecord> {
    vec![
        StandardStatusRecord {
            standard: "FACE™ UoP".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/face_uop_mapping.md".to_string(),
            component_refs: vec![
                "src/lib.rs".to_string(),
                "src/ffi.rs".to_string(),
                "include/dsfb_battery_ffi.h".to_string(),
            ],
            notes: "Portable software-component boundaries and interface surfaces are mapped, but no formal FACE conformance claim is made.".to_string(),
        },
        StandardStatusRecord {
            standard: "DO-178C DAL advisory alignment".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/do178c_dal_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/audit.rs".to_string(),
                "src/export.rs".to_string(),
            ],
            notes: "Requirement-to-output traceability is mapped for an advisory, non-interfering role only.".to_string(),
        },
        StandardStatusRecord {
            standard: "STC support scaffold".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/stc_support.md".to_string(),
            component_refs: vec![
                "src/audit.rs".to_string(),
                "src/compliance.rs".to_string(),
            ],
            notes: "Configuration/input hashing and reproducibility support are provided as engineering evidence only.".to_string(),
        },
        StandardStatusRecord {
            standard: "MIL-STD-882E".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/mil_std_882e_mapping.md".to_string(),
            component_refs: vec![
                "src/types.rs".to_string(),
                "src/detection.rs".to_string(),
            ],
            notes: "Reason-code-to-hazard mapping is advisory and not a completed system safety assessment.".to_string(),
        },
        StandardStatusRecord {
            standard: "Ferrocene / Rust safe subset".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/rust_safety_subset.md".to_string(),
            component_refs: vec![
                "src/types.rs".to_string(),
                "src/math.rs".to_string(),
                "src/detection.rs".to_string(),
                "src/ffi.rs".to_string(),
            ],
            notes: "The core path is deterministic and bounded, but alloc-backed structures remain and unsafe is present at the FFI boundary.".to_string(),
        },
        StandardStatusRecord {
            standard: "NIST 800-171 / CMMC 2.0 Level 2".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/nist_cmmc_mapping.md".to_string(),
            component_refs: vec![
                "src/audit.rs".to_string(),
                "src/heuristics.rs".to_string(),
            ],
            notes: "Integrity and auditability mappings exist; access-control and operational controls are outside crate scope.".to_string(),
        },
        StandardStatusRecord {
            standard: "IEC 61508".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/iec_61508_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/audit.rs".to_string(),
            ],
            notes: "Deterministic advisory behavior is mapped, but no SIL claim is made.".to_string(),
        },
        StandardStatusRecord {
            standard: "IEC 61131-3".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/iec_61131_3_mapping.md".to_string(),
            component_refs: vec![
                "wrappers/plc/structured_text.st".to_string(),
                "src/detection.rs".to_string(),
            ],
            notes: "A deterministic Structured Text translation is provided as a wrapper, not as deployed control logic.".to_string(),
        },
        StandardStatusRecord {
            standard: "IEEE 1547-2018".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/ieee_1547_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/compliance.rs".to_string(),
            ],
            notes: "Violation-to-advisory escalation mapping is documented without implementing protection logic.".to_string(),
        },
        StandardStatusRecord {
            standard: "ISO 26262".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/iso_26262_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/integration.rs".to_string(),
            ],
            notes: "Diagnostic coverage is discussed as an advisory monitor only; no ASIL claim is made.".to_string(),
        },
        StandardStatusRecord {
            standard: "MISRA-equivalent safe Rust scan".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "outputs/compliance/.../misra_equivalent_report.txt".to_string(),
            component_refs: vec![
                "src/compliance.rs".to_string(),
                "src/ffi.rs".to_string(),
            ],
            notes: "The report is a heuristic scan over unsafe, alloc patterns, and direct recursion.".to_string(),
        },
        StandardStatusRecord {
            standard: "DO-311A / IEC 62619 / MIL-PRF-32565C / SAE J2929 / UL 1973".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/industry_standards_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/audit.rs".to_string(),
                "src/compliance.rs".to_string(),
            ],
            notes: "Unified advisory-monitoring mapping only; no product certification claim is made.".to_string(),
        },
        StandardStatusRecord {
            standard: "IEEE 754-2019".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/ieee_754_determinism.md".to_string(),
            component_refs: vec![
                "src/math.rs".to_string(),
                "src/detection.rs".to_string(),
                "src/compliance.rs".to_string(),
            ],
            notes: "Local repeated-run reproducibility is checked; cross-platform bitwise proof is not claimed.".to_string(),
        },
        StandardStatusRecord {
            standard: "ISO 15926".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/iso_15926_mapping.md".to_string(),
            component_refs: vec![
                "src/types.rs".to_string(),
                "src/audit.rs".to_string(),
            ],
            notes: "Lifecycle semantic mapping is provided at the information-model level only.".to_string(),
        },
        StandardStatusRecord {
            standard: "ISO/IEC 25010".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/iso_25010_mapping.md".to_string(),
            component_refs: vec![
                "src/audit.rs".to_string(),
                "src/ffi.rs".to_string(),
                "src/lib.rs".to_string(),
            ],
            notes: "Quality-attribute mapping is documented, not independently certified.".to_string(),
        },
        StandardStatusRecord {
            standard: "W3C SSN/SOSA".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/sosa_mapping.md".to_string(),
            component_refs: vec![
                "src/types.rs".to_string(),
                "src/audit.rs".to_string(),
            ],
            notes: "Signal-to-observation semantics are mapped without claiming ontology conformance.".to_string(),
        },
        StandardStatusRecord {
            standard: "ISO 21448 (SOTIF)".to_string(),
            status: StandardStatus::Partial,
            mapping_artifact: "docs/compliance/sotif_mapping.md".to_string(),
            component_refs: vec![
                "src/detection.rs".to_string(),
                "src/integration.rs".to_string(),
            ],
            notes: "Functional-inadequacy detection is mapped as an advisory monitor only.".to_string(),
        },
    ]
}

fn build_deterministic_summary(
    trajectory: &[BatteryResidual],
    dsfb_alarm_cycle: Option<usize>,
    threshold_alarm_cycle: Option<usize>,
    t_star: usize,
) -> Result<DeterministicSummary, ComplianceError> {
    let final_state = trajectory
        .last()
        .map(|sample| sample.grammar_state)
        .ok_or(ComplianceError::EmptyInput)?;
    Ok(DeterministicSummary {
        dsfb_alarm_cycle,
        threshold_alarm_cycle,
        first_boundary_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Boundary)
            .map(|sample| sample.cycle),
        first_violation_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Violation)
            .map(|sample| sample.cycle),
        final_state,
        t_star,
    })
}

fn build_determinism_check(
    capacities: &[f64],
    config: &PipelineConfig,
) -> Result<DeterminismCheckArtifact, ComplianceError> {
    let hash_run = |caps: &[f64]| -> Result<String, ComplianceError> {
        let (envelope, trajectory) = run_dsfb_pipeline(caps, config)?;
        let eol_capacity = config.eol_fraction * caps[0];
        let dsfb = build_dsfb_detection(&trajectory, caps, eol_capacity);
        let threshold = build_threshold_detection(caps, THRESHOLD_BASELINE_FRACTION, eol_capacity);
        let theorem1 = verify_theorem1(&envelope, &trajectory, config);
        let summary = build_deterministic_summary(
            &trajectory,
            dsfb.alarm_cycle,
            threshold.alarm_cycle,
            theorem1.t_star,
        )?;
        hash_bytes(&serde_json::to_vec(&summary)?)
    };

    let summary_hash_run_1 = hash_run(capacities)?;
    let summary_hash_run_2 = hash_run(capacities)?;

    Ok(DeterminismCheckArtifact {
        artifact_type: DETERMINISM_CHECK_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        classification: if summary_hash_run_1 == summary_hash_run_2 {
            StandardStatus::Supported
        } else {
            StandardStatus::Partial
        },
        repeated_run_equal: summary_hash_run_1 == summary_hash_run_2,
        summary_hash_run_1,
        summary_hash_run_2,
        notes: vec![
            "This helper checks repeated local execution on the same toolchain and input ordering.".to_string(),
            "It does not claim cross-platform IEEE 754 bit-for-bit equivalence.".to_string(),
        ],
    })
}

fn build_stc_traceability_support(
    raw_input: &[(usize, f64)],
    config: &PipelineConfig,
    trajectory: &[BatteryResidual],
    dsfb_detection: &crate::types::DetectionResult,
    threshold_detection: &crate::types::DetectionResult,
    t_star: usize,
    reproducibility_equal: bool,
) -> Result<StcTraceabilitySupport, ComplianceError> {
    let config_hash = hash_bytes(&serde_json::to_vec(config)?)?;
    let input_hash = hash_bytes(&serde_json::to_vec(raw_input)?)?;
    let summary = build_deterministic_summary(
        trajectory,
        dsfb_detection.alarm_cycle,
        threshold_detection.alarm_cycle,
        t_star,
    )?;
    let summary_hash = hash_bytes(&serde_json::to_vec(&summary)?)?;

    Ok(StcTraceabilitySupport {
        artifact_type: STC_TRACEABILITY_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        classification: StandardStatus::Partial,
        config_hash,
        input_hash,
        summary_hash,
        reproducibility_equal,
        notes: vec![
            "This scaffold provides configuration identification, input identification, and a deterministic summary hash for traceability support.".to_string(),
            "It does not constitute an STC approval package.".to_string(),
        ],
    })
}

fn build_operator_overlay(
    trajectory: &[BatteryResidual],
) -> (OperatorOverlaySummary, Vec<OperatorOverlayRow>) {
    let rows: Vec<OperatorOverlayRow> = trajectory
        .iter()
        .map(|sample| OperatorOverlayRow {
            cycle: sample.cycle,
            state: sample.grammar_state,
            tri_state_color: state_color(sample.grammar_state).to_string(),
            reason_code: sample.reason_code,
            advisory_text: operator_advisory_text(sample.grammar_state, sample.reason_code),
        })
        .collect();

    let final_sample = trajectory.last();
    let final_state = final_sample
        .map(|sample| sample.grammar_state)
        .unwrap_or(GrammarState::Admissible);
    let final_reason_code = final_sample.and_then(|sample| sample.reason_code);

    (
        OperatorOverlaySummary {
            artifact_type: OPERATOR_OVERLAY_TYPE.to_string(),
            generated_at_utc: Utc::now().to_rfc3339(),
            classification: StandardStatus::Supported,
            final_state,
            final_color: state_color(final_state).to_string(),
            final_reason_code,
            legend: vec![
                ColorLegendEntry {
                    state: GrammarState::Admissible,
                    color: "Green".to_string(),
                },
                ColorLegendEntry {
                    state: GrammarState::Boundary,
                    color: "Yellow".to_string(),
                },
                ColorLegendEntry {
                    state: GrammarState::Violation,
                    color: "Red".to_string(),
                },
            ],
            rows: rows.len(),
            advisory_text: operator_advisory_text(final_state, final_reason_code),
        },
        rows,
    )
}

fn state_color(state: GrammarState) -> &'static str {
    match state {
        GrammarState::Admissible => "Green",
        GrammarState::Boundary => "Yellow",
        GrammarState::Violation => "Red",
    }
}

fn operator_advisory_text(state: GrammarState, reason_code: Option<ReasonCode>) -> String {
    let base = match state {
        GrammarState::Admissible => {
            "Admissible: remain in advisory monitoring; the residual is within the declared envelope."
        }
        GrammarState::Boundary => {
            "Boundary: advisory-only structural deviation is approaching the admissibility envelope; review the trend and persistence counters."
        }
        GrammarState::Violation => {
            "Violation: advisory-only envelope exit is present; escalate review under host policy, but this helper does not issue control commands."
        }
    };
    match reason_code {
        Some(code) => format!("{base} Interpretation: {code}."),
        None => base.to_string(),
    }
}

fn scan_safe_rust_subset(crate_dir: &Path) -> Result<SafeRustAudit, ComplianceError> {
    let src_dir = crate_dir.join("src");
    let files = collect_rs_files(&src_dir)?;
    let core_engine_paths = vec![
        "src/types.rs".to_string(),
        "src/math.rs".to_string(),
        "src/detection.rs".to_string(),
        "src/ffi.rs".to_string(),
    ];

    let unsafe_hits_all_src = scan_for_patterns(crate_dir, &files, &["unsafe"]);
    let core_paths: Vec<PathBuf> = core_engine_paths
        .iter()
        .map(|path| crate_dir.join(path))
        .collect();
    let unsafe_hits_core_boundary = scan_for_patterns(crate_dir, &core_paths, &["unsafe"]);
    let dynamic_allocation_hits_core = scan_for_patterns(
        crate_dir,
        &core_paths,
        &["Vec<", "vec![", "String", "format!(", "Box<", "Box::", "alloc::"],
    );
    let recursion_hits_all_src = scan_for_direct_recursion(crate_dir, &files);

    Ok(SafeRustAudit {
        artifact_type: SAFE_RUST_AUDIT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        classification: if unsafe_hits_core_boundary.is_empty()
            && dynamic_allocation_hits_core.is_empty()
            && recursion_hits_all_src.is_empty()
        {
            StandardStatus::Supported
        } else {
            StandardStatus::Partial
        },
        core_engine_paths,
        unsafe_hits_all_src,
        unsafe_hits_core_boundary,
        dynamic_allocation_hits_core,
        recursion_hits_all_src,
        notes: vec![
            "The current core logic in src/types.rs, src/math.rs, and src/detection.rs is deterministic but uses alloc-backed data structures in the current implementation.".to_string(),
            "Unsafe usage is expected at the FFI boundary and is reported rather than hidden.".to_string(),
            "This is a heuristic source scan and not a certified Ferrocene audit.".to_string(),
        ],
    })
}

fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>, ComplianceError> {
    let mut files = Vec::new();
    collect_rs_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rs_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), ComplianceError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn scan_for_patterns(crate_dir: &Path, files: &[PathBuf], patterns: &[&str]) -> Vec<SafeRustFinding> {
    let mut findings = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if patterns.iter().any(|pattern| trimmed.contains(pattern)) {
                findings.push(SafeRustFinding {
                    path: relative_display(crate_dir, path),
                    line: index + 1,
                    snippet: trimmed.to_string(),
                });
            }
        }
    }
    findings
}

fn scan_for_direct_recursion(crate_dir: &Path, files: &[PathBuf]) -> Vec<SafeRustFinding> {
    let mut findings = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        let mut functions = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if let Some(name) = parse_fn_name(line) {
                functions.push((index, name));
            }
        }

        for (idx, (start, name)) in functions.iter().enumerate() {
            let end = functions
                .get(idx + 1)
                .map(|(next_start, _)| *next_start)
                .unwrap_or(lines.len());
            let pattern = format!("{name}(");
            for (offset, line) in lines.iter().enumerate().take(end).skip(start + 1) {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains(&pattern) {
                    findings.push(SafeRustFinding {
                        path: relative_display(crate_dir, path),
                        line: offset + 1,
                        snippet: trimmed.to_string(),
                    });
                    break;
                }
            }
        }
    }
    findings
}

fn parse_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return None;
    }
    let fn_pos = trimmed.find("fn ")?;
    let after = &trimmed[fn_pos + 3..];
    let name: String = after
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn relative_display(crate_dir: &Path, path: &Path) -> String {
    path.strip_prefix(crate_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn hash_bytes(bytes: &[u8]) -> Result<String, ComplianceError> {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        write!(&mut encoded, "{:02x}", byte)
            .map_err(|error| ComplianceError::Io(std::io::Error::other(error.to_string())))?;
    }
    Ok(encoded)
}

fn write_text(contents: &str, path: &Path) -> Result<(), ComplianceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn write_json<T: Serialize>(value: &T, path: &Path) -> Result<(), ComplianceError> {
    write_text(&serde_json::to_string_pretty(value)?, path)
}

fn write_operator_overlay_csv(
    rows: &[OperatorOverlayRow],
    path: &Path,
) -> Result<(), ComplianceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record(["cycle", "state", "tri_state_color", "reason_code", "advisory_text"])?;
    for row in rows {
        writer.write_record([
            row.cycle.to_string(),
            row.state.to_string(),
            row.tri_state_color.clone(),
            row.reason_code.map(|code| code.to_string()).unwrap_or_default(),
            row.advisory_text.clone(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_standards_status_csv(
    rows: &[StandardStatusRecord],
    path: &Path,
) -> Result<(), ComplianceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "standard",
        "status",
        "mapping_artifact",
        "component_refs",
        "notes",
    ])?;
    for row in rows {
        writer.write_record([
            row.standard.clone(),
            format!("{:?}", row.status).to_lowercase(),
            row.mapping_artifact.clone(),
            row.component_refs.join("; "),
            row.notes.clone(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn render_implementation_summary(
    summary: &ComplianceImplementationSummary,
    source_path: &Path,
    dsfb_alarm_cycle: Option<usize>,
    threshold_alarm_cycle: Option<usize>,
    t_star: usize,
) -> String {
    let mut lines = vec![
        "Compliance Support Layer Summary".to_string(),
        "".to_string(),
        format!("Artifact type: {}", summary.artifact_type),
        format!("Output root: {}", summary.output_root),
        format!("Input source: {}", source_path.display()),
        format!("DSFB alarm cycle observed in helper run: {:?}", dsfb_alarm_cycle),
        format!(
            "Threshold baseline cycle observed in helper run: {:?}",
            threshold_alarm_cycle
        ),
        format!("Theorem t_star observed in helper run: {}", t_star),
        "".to_string(),
        "Standards covered:".to_string(),
    ];

    for record in &summary.standards_covered {
        lines.push(format!(
            "- {} -> {:?} ({})",
            record.standard,
            record.status,
            record.mapping_artifact
        ));
    }

    lines.push("".to_string());
    lines.push("Artifacts generated:".to_string());
    for artifact in &summary.artifacts_generated {
        lines.push(format!("- {}", artifact));
    }

    lines.push("".to_string());
    lines.push("Mapped only:".to_string());
    for item in &summary.mapped_only {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push("Scaffolded:".to_string());
    for item in &summary.scaffolded {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push(summary.no_production_modification_statement.clone());
    lines.push(PRODUCTION_ARTIFACTS_UNCHANGED_NOTE.to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", stem, unique))
    }

    fn sample_raw_input() -> Vec<(usize, f64)> {
        (1..=60)
            .map(|cycle| {
                let healthy = 2.0 + 0.001 * ((cycle as f64) * 1.3).sin();
                let degraded = if cycle > 25 {
                    healthy - 0.01 * (cycle - 25) as f64
                } else {
                    healthy
                };
                (cycle, degraded)
            })
            .collect()
    }

    #[test]
    fn resolve_compliance_output_dir_defaults_to_timestamped_directory() {
        let crate_dir = Path::new("/tmp/dsfb-battery");
        let output = resolve_compliance_output_dir(crate_dir, None);
        assert!(
            output
                .display()
                .to_string()
                .contains("outputs/compliance/dsfb_battery_compliance_")
        );
    }

    #[test]
    fn unique_named_output_dir_avoids_overwrite() {
        let root = unique_temp_dir("dsfb-battery-compliance-root");
        fs::create_dir_all(&root).unwrap();
        let first = root.join("dsfb_battery_compliance_fixed");
        fs::create_dir_all(&first).unwrap();
        let second = unique_named_output_dir(&root, "dsfb_battery_compliance_fixed");
        assert_ne!(second, first);
        assert!(second.file_name().unwrap().to_string_lossy().contains("_r1"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn compliance_workflow_writes_only_to_its_output_directory() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let output_dir = unique_temp_dir("dsfb-battery-compliance");
        let raw_input = sample_raw_input();

        let summary = run_compliance_workflow_from_input(
            crate_dir,
            &raw_input,
            &output_dir,
            Path::new("synthetic/b0005_like.csv"),
        )
        .unwrap();

        assert!(output_dir.join("misra_equivalent_report.txt").exists());
        assert!(output_dir.join("implementation_summary.txt").exists());
        assert!(output_dir.join("determinism_check.json").exists());
        assert!(output_dir.join("stc_traceability_support.json").exists());
        assert!(output_dir.join("standards_status_matrix.csv").exists());
        assert!(
            output_dir
                .join("operator_overlay")
                .join("operator_overlay_summary.json")
                .exists()
        );
        assert!(
            output_dir
                .join("operator_overlay")
                .join("operator_overlay_timeline.csv")
                .exists()
        );
        assert_eq!(
            summary.no_production_modification_statement,
            IMPLEMENTATION_SUMMARY_STATEMENT
        );
        assert!(!output_dir.join("stage2_detection_results.json").exists());
        assert!(!output_dir.join("fig01_capacity_fade.svg").exists());

        let summary_text = fs::read_to_string(output_dir.join("implementation_summary.txt")).unwrap();
        assert!(summary_text.contains("No production code or figures were modified"));

        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn misra_report_mentions_unsafe_and_alloc_findings() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let audit = scan_safe_rust_subset(crate_dir).unwrap();
        let report = render_misra_equivalent_report(&audit);
        assert!(report.contains("MISRA-Equivalent Safe Rust Report"));
        assert!(report.contains("Dynamic-allocation pattern hits"));
        assert!(report.contains("Unsafe hits"));
    }
}
