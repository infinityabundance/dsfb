// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Isolated addendum layer for formal/procedural scaffolds, operator overlays,
// integrity helpers, SWaP-C reporting, and wrapper-support artifacts.

use crate::compliance::run_compliance_workflow_from_input;
use crate::complexity::{estimate_dsfb_update_complexity, ComplexityArtifact};
use crate::detection::{
    assign_reason_code, build_dsfb_detection, build_threshold_detection, evaluate_grammar_state,
    next_persistence_count, run_dsfb_pipeline, verify_theorem1,
};
use crate::integration::{
    build_knee_onset_narrative, build_validity_token, compute_tactical_margin_summary,
    KneeOnsetNarrative, TacticalMarginSummary, ValidityToken,
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

const ADDENDUM_ARTIFACT_TYPE: &str = "dsfb_battery_addendum_support";
const OVERLAY_ARTIFACT_TYPE: &str = "dsfb_battery_zero_burden_overlay";
const TAMPER_TRACE_TYPE: &str = "dsfb_battery_tamper_evident_trace";
const TAMPER_VERIFY_TYPE: &str = "dsfb_battery_tamper_evident_verification";
const BATTERY_PASSPORT_TYPE: &str = "dsfb_battery_passport_support_stub";
const SWAPC_ARTIFACT_TYPE: &str = "dsfb_battery_swapc_report";
const SEU_ARTIFACT_TYPE: &str = "dsfb_battery_seu_resilience_helper";
const THRESHOLD_BASELINE_FRACTION: f64 = 0.85;
const TACTICAL_MARGIN_FRACTION: f64 = 0.88;
const NO_PRODUCTION_MODIFICATION_STATEMENT: &str =
    "Original mono-cell production code and figures were not modified in behavior";

#[derive(Debug, Error)]
pub enum AddendumError {
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
    #[error("compliance error: {0}")]
    Compliance(#[from] crate::compliance::ComplianceError),
}

#[derive(Debug, Clone, Serialize)]
pub struct TriStateLegendEntry {
    pub state: GrammarState,
    pub color: String,
    pub operator_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddendumOverlayArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub grammar_state: GrammarState,
    pub tri_state_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<ReasonCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_threshold_cycles: Option<i64>,
    pub tactical_margin: TacticalMarginSummary,
    pub validity_token: ValidityToken,
    pub knee_onset_narrative: KneeOnsetNarrative,
    pub advisory_text: String,
    pub legend: Vec<TriStateLegendEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TamperEvidentRecord {
    pub chain_index: usize,
    pub cycle: usize,
    pub grammar_state: GrammarState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<ReasonCode>,
    pub residual_value: f64,
    pub drift_value: f64,
    pub slew_value: f64,
    pub previous_digest: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TamperEvidentTraceArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub config_hash: String,
    pub input_hash: String,
    pub record_count: usize,
    pub root_digest: String,
    pub records: Vec<TamperEvidentRecord>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TamperEvidentVerificationArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub valid: bool,
    pub verified_records: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_failed_index: Option<usize>,
    pub stored_root_digest: String,
    pub recomputed_root_digest: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatteryPassportStubArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub schema_label: String,
    pub dataset_name: String,
    pub cell_id: String,
    pub audit_trace_contract_present: bool,
    pub config_hash: String,
    pub input_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_boundary_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_violation_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_threshold_cycles: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_reason_code: Option<ReasonCode>,
    pub maintenance_relevant_annotation: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddendumSwapcArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub complexity: ComplexityArtifact,
    pub static_allocation_note: String,
    pub dynamic_allocation_note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_staticlib_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_staticlib_bytes: Option<u64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddendumSeuResilienceArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub sampled_cycles: usize,
    pub redundant_state_match: bool,
    pub redundant_reason_match: bool,
    pub invalid_state_detected: bool,
    pub status_checksum: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissionBusSignalDictionary {
    pub signal_dictionary_version: String,
    pub transport_agnostic_fields: Vec<String>,
    pub mil_std_1553_mapping: Vec<String>,
    pub arinc_429_mapping: Vec<String>,
    pub arinc_664_mapping: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddendumImplementationSummary {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub fully_implemented: Vec<String>,
    pub scaffolded: Vec<String>,
    pub wrappers_added: Vec<String>,
    pub compliance_support_matrices: Vec<String>,
    pub integrity_and_resilience_helpers: Vec<String>,
    pub output_root: String,
    pub compliance_support_output: String,
    pub no_production_modification_statement: String,
}

pub fn resolve_addendum_output_dir(crate_dir: &Path, explicit_output: Option<PathBuf>) -> PathBuf {
    if let Some(output) = explicit_output {
        return output;
    }
    let root = crate_dir.join("outputs").join("addendum");
    let stem = format!("dsfb_battery_addendum_{}", Local::now().format("%Y%m%d_%H%M%S"));
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
    unreachable!("unbounded retry loop for addendum output directory")
}

pub fn run_addendum_workflow(
    crate_dir: &Path,
    data_path: &Path,
    output_dir: &Path,
) -> Result<AddendumImplementationSummary, AddendumError> {
    let raw_input = load_b0005_csv(data_path)
        .map_err(|error| AddendumError::Io(std::io::Error::other(error.to_string())))?;
    run_addendum_workflow_from_input(crate_dir, &raw_input, data_path, output_dir)
}

pub fn run_addendum_workflow_from_input(
    crate_dir: &Path,
    raw_input: &[(usize, f64)],
    source_path: &Path,
    output_dir: &Path,
) -> Result<AddendumImplementationSummary, AddendumError> {
    if raw_input.is_empty() {
        return Err(AddendumError::EmptyInput);
    }

    fs::create_dir_all(output_dir)?;

    let capacities: Vec<f64> = raw_input.iter().map(|(_, value)| *value).collect();
    let config = PipelineConfig::default();
    let (envelope, trajectory) = run_dsfb_pipeline(&capacities, &config)?;
    let eol_capacity = config.eol_fraction * capacities[0];
    let dsfb_detection = build_dsfb_detection(&trajectory, &capacities, eol_capacity);
    let _threshold_detection =
        build_threshold_detection(&capacities, THRESHOLD_BASELINE_FRACTION, eol_capacity);
    let theorem1 = verify_theorem1(&envelope, &trajectory, &config);

    let overlay = build_zero_burden_overlay(
        &capacities,
        &trajectory,
        &config,
        dsfb_detection.lead_time_cycles,
    );
    let tamper_trace = build_tamper_evident_trace(raw_input, &trajectory, &config)?;
    let tamper_verify = tamper_evident_verification(&tamper_trace);
    let passport_stub = build_battery_passport_stub(
        raw_input,
        &trajectory,
        &config,
        dsfb_detection.lead_time_cycles,
    )?;
    let swapc_artifact = build_swapc_artifact(crate_dir, &config);
    let seu_artifact = build_seu_resilience_artifact(&trajectory, &config, envelope.rho)?;
    let mission_bus_dictionary = build_mission_bus_signal_dictionary();

    let overlay_dir = output_dir.join("operator_overlay");
    let integrity_dir = output_dir.join("integrity");
    let swapc_dir = output_dir.join("swapc");
    let passport_dir = output_dir.join("battery_passport");
    let seu_dir = output_dir.join("seu_resilience");
    let compliance_support_dir = output_dir.join("assurance_support");

    fs::create_dir_all(&overlay_dir)?;
    fs::create_dir_all(&integrity_dir)?;
    fs::create_dir_all(&swapc_dir)?;
    fs::create_dir_all(&passport_dir)?;
    fs::create_dir_all(&seu_dir)?;

    write_json(&overlay, &overlay_dir.join("decision_support_overlay.json"))?;
    write_text(
        &render_overlay_text(&overlay),
        &overlay_dir.join("decision_support_overlay.txt"),
    )?;
    write_overlay_timeline_csv(
        &trajectory,
        &overlay_dir.join("operator_overlay_timeline.csv"),
    )?;

    write_json(
        &tamper_trace,
        &integrity_dir.join("tamper_evident_trace.json"),
    )?;
    write_json(
        &tamper_verify,
        &integrity_dir.join("tamper_evident_verification.json"),
    )?;

    write_text(
        &render_swapc_report(&swapc_artifact),
        &swapc_dir.join("swapc_report.txt"),
    )?;
    write_json(&swapc_artifact, &swapc_dir.join("swapc_report.json"))?;

    write_json(
        &passport_stub,
        &passport_dir.join("battery_passport_stub.json"),
    )?;
    write_json(
        &mission_bus_dictionary,
        &output_dir.join("mission_bus_signal_dictionary.json"),
    )?;
    write_json(
        &seu_artifact,
        &seu_dir.join("seu_resilience_report.json"),
    )?;

    let compliance_summary = run_compliance_workflow_from_input(
        crate_dir,
        raw_input,
        &compliance_support_dir,
        source_path,
    )?;

    let summary = AddendumImplementationSummary {
        artifact_type: ADDENDUM_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        fully_implemented: vec![
            "Zero-burden operator overlay with tri-state color logic, reason code, validity token, and advisory narrative.".to_string(),
            "Tamper-evident residual trace with hash chaining and verification helper output.".to_string(),
            "SWaP-C report derived from the current DSFB update path and local build-artifact size checks when present.".to_string(),
            "Battery passport support stub using traceability-oriented DSFB fields.".to_string(),
            "Nested compliance support package under the addendum output root.".to_string(),
            "SEU-oriented redundant-evaluation and checksum helper artifact.".to_string(),
        ],
        scaffolded: vec![
            "Kani harnesses are provided under formal/kani but were not executed here because Kani availability is environment-dependent.".to_string(),
            "MATLAB/Simulink integration is provided as wrapper source, build notes, and interface scaffolding only.".to_string(),
            "Mission-bus, MOSA, NASA Power of 10, assurance, and passport items are mappings/support artifacts rather than certification claims.".to_string(),
        ],
        wrappers_added: vec![
            "wrappers/plc/structured_text.st".to_string(),
            "wrappers/matlab/dsfb_battery_sfun_stub.c".to_string(),
            "ffi/dsfb_battery_addendum_example.c".to_string(),
        ],
        compliance_support_matrices: vec![
            "docs/addendum/mosa_compatibility.md".to_string(),
            "docs/addendum/icd.md".to_string(),
            "docs/addendum/nasa_power_of_10_alignment.md".to_string(),
            "docs/addendum/eu_battery_passport_mapping.md".to_string(),
            "docs/addendum/assurance_mapping.md".to_string(),
            "docs/addendum/mission_bus_mapping.md".to_string(),
            "docs/addendum/seu_resilience.md".to_string(),
        ],
        integrity_and_resilience_helpers: vec![
            "outputs/addendum/.../integrity/tamper_evident_trace.json".to_string(),
            "outputs/addendum/.../integrity/tamper_evident_verification.json".to_string(),
            "outputs/addendum/.../seu_resilience/seu_resilience_report.json".to_string(),
        ],
        output_root: output_dir.display().to_string(),
        compliance_support_output: compliance_support_dir.display().to_string(),
        no_production_modification_statement: NO_PRODUCTION_MODIFICATION_STATEMENT.to_string(),
    };

    write_text(
        &render_implementation_summary(
            &summary,
            source_path,
            &overlay,
            theorem1.t_star,
            &compliance_summary.output_root,
        ),
        &output_dir.join("implementation_summary.txt"),
    )?;

    Ok(summary)
}

pub fn build_zero_burden_overlay(
    capacities: &[f64],
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
    lead_time_vs_threshold_cycles: Option<i64>,
) -> AddendumOverlayArtifact {
    let final_sample = trajectory.last();
    let final_cycle = final_sample.map(|sample| sample.cycle).unwrap_or(0);
    let grammar_state = final_sample
        .map(|sample| sample.grammar_state)
        .unwrap_or(GrammarState::Admissible);
    let reason_code = final_sample.and_then(|sample| sample.reason_code);

    AddendumOverlayArtifact {
        artifact_type: OVERLAY_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "advisory_only_decision_support_overlay".to_string(),
        grammar_state,
        tri_state_color: tri_state_color(grammar_state).to_string(),
        reason_code,
        lead_time_vs_threshold_cycles,
        tactical_margin: compute_tactical_margin_summary(capacities, trajectory, TACTICAL_MARGIN_FRACTION),
        validity_token: build_validity_token(final_cycle, true, 60),
        knee_onset_narrative: build_knee_onset_narrative(trajectory, config),
        advisory_text: operator_text(grammar_state, reason_code),
        legend: tri_state_legend(),
    }
}

pub fn build_tamper_evident_trace(
    raw_input: &[(usize, f64)],
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
) -> Result<TamperEvidentTraceArtifact, AddendumError> {
    let config_hash = hash_json(config)?;
    let input_hash = hash_json(raw_input)?;
    let mut previous_digest = format!("sha256:{}:{}", config_hash, input_hash);
    let mut records = Vec::with_capacity(trajectory.len());

    for (index, sample) in trajectory.iter().enumerate() {
        let digest = hash_string(&format!(
            "{}|{}|{}|{:?}|{:.12}|{:.12}|{:.12}",
            previous_digest,
            sample.cycle,
            sample.grammar_state,
            sample.reason_code,
            sample.sign.r,
            sample.sign.d,
            sample.sign.s
        ));
        records.push(TamperEvidentRecord {
            chain_index: index,
            cycle: sample.cycle,
            grammar_state: sample.grammar_state,
            reason_code: sample.reason_code,
            residual_value: sample.sign.r,
            drift_value: sample.sign.d,
            slew_value: sample.sign.s,
            previous_digest: previous_digest.clone(),
            digest: digest.clone(),
        });
        previous_digest = digest;
    }

    Ok(TamperEvidentTraceArtifact {
        artifact_type: TAMPER_TRACE_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        config_hash,
        input_hash,
        record_count: records.len(),
        root_digest: previous_digest,
        records,
        notes: vec![
            "This addendum trace is inspired by tamper-evident integrity goals and does not claim NIST SP 800-193 compliance.".to_string(),
            "The chain covers residual values, grammar states, reason codes, and per-record linkage.".to_string(),
        ],
    })
}

pub fn tamper_evident_verification(
    trace: &TamperEvidentTraceArtifact,
) -> TamperEvidentVerificationArtifact {
    let mut previous_digest = format!("sha256:{}:{}", trace.config_hash, trace.input_hash);
    let mut first_failed_index = None;
    let mut verified_records = 0usize;

    for record in &trace.records {
        let digest = hash_string(&format!(
            "{}|{}|{}|{:?}|{:.12}|{:.12}|{:.12}",
            previous_digest,
            record.cycle,
            record.grammar_state,
            record.reason_code,
            record.residual_value,
            record.drift_value,
            record.slew_value
        ));
        if record.previous_digest != previous_digest || record.digest != digest {
            first_failed_index = Some(record.chain_index);
            previous_digest = digest;
            break;
        }
        previous_digest = digest;
        verified_records += 1;
    }

    TamperEvidentVerificationArtifact {
        artifact_type: TAMPER_VERIFY_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        valid: first_failed_index.is_none() && previous_digest == trace.root_digest,
        verified_records,
        first_failed_index,
        stored_root_digest: trace.root_digest.clone(),
        recomputed_root_digest: previous_digest,
        notes: vec![
            "Verification recomputes the chain deterministically from stored fields.".to_string(),
            "This helper detects local record mutation in the chained trace artifact.".to_string(),
        ],
    }
}

pub fn render_swapc_report(artifact: &AddendumSwapcArtifact) -> String {
    let mut lines = vec![
        "DSFB Addendum SWaP-C Report".to_string(),
        format!("Generated at: {}", artifact.generated_at_utc),
        "".to_string(),
        format!(
            "Per-update arithmetic order: {}",
            artifact.complexity.algorithmic_order_per_update
        ),
        format!(
            "Implementation shape: {}",
            artifact.complexity.implementation_shape
        ),
        "".to_string(),
        "Warm-path operation estimate:".to_string(),
        format!(
            "- floating-point add/sub: {}",
            artifact.complexity.operation_estimate.floating_point_add_sub
        ),
        format!(
            "- floating-point mul/div: {}",
            artifact.complexity.operation_estimate.floating_point_mul_div
        ),
        format!("- abs ops: {}", artifact.complexity.operation_estimate.abs_ops),
        format!(
            "- comparisons: {}",
            artifact.complexity.operation_estimate.comparisons
        ),
        format!(
            "- integer counter updates: {}",
            artifact.complexity.operation_estimate.integer_counter_updates
        ),
        format!(
            "- rolling window reads: {}",
            artifact.complexity.operation_estimate.window_reads
        ),
        "".to_string(),
        "State footprint estimate:".to_string(),
        format!(
            "- residual window samples: {}",
            artifact.complexity.memory_footprint.residual_window_samples
        ),
        format!(
            "- drift window samples: {}",
            artifact.complexity.memory_footprint.drift_window_samples
        ),
        format!(
            "- envelope scalars: {}",
            artifact.complexity.memory_footprint.envelope_scalars
        ),
        format!(
            "- persistence counters: {}",
            artifact.complexity.memory_footprint.persistence_counters
        ),
        "".to_string(),
        format!("Static allocation note: {}", artifact.static_allocation_note),
        format!("Dynamic allocation note: {}", artifact.dynamic_allocation_note),
        match artifact.debug_staticlib_bytes {
            Some(bytes) => format!("- debug staticlib size: {} bytes", bytes),
            None => "- debug staticlib size: not measured in the current workspace state".to_string(),
        },
        match artifact.release_staticlib_bytes {
            Some(bytes) => format!("- release staticlib size: {} bytes", bytes),
            None => "- release staticlib size: not measured in the current workspace state".to_string(),
        },
        "".to_string(),
        "Notes:".to_string(),
    ];
    for note in &artifact.notes {
        lines.push(format!("- {}", note));
    }
    lines.join("\n")
}

fn build_swapc_artifact(crate_dir: &Path, config: &PipelineConfig) -> AddendumSwapcArtifact {
    let complexity = estimate_dsfb_update_complexity(config);
    AddendumSwapcArtifact {
        artifact_type: SWAPC_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        complexity,
        static_allocation_note:
            "The current crate supports a core no_std + alloc path; it is not heapless.".to_string(),
        dynamic_allocation_note:
            "Batch execution stores full vectors and therefore uses dynamic allocation in the host-side workflow.".to_string(),
        debug_staticlib_bytes: file_size(crate_dir.join("target").join("debug").join("libdsfb_battery.a")),
        release_staticlib_bytes: file_size(crate_dir.join("target").join("release").join("libdsfb_battery.a")),
        notes: vec![
            "Power/performance observations are estimate-level only; no silicon measurement is claimed.".to_string(),
            "The production mono-cell path remains batch-oriented even though the per-update rules are O(1).".to_string(),
        ],
    }
}

fn build_battery_passport_stub(
    raw_input: &[(usize, f64)],
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
    lead_time_vs_threshold_cycles: Option<i64>,
) -> Result<BatteryPassportStubArtifact, AddendumError> {
    Ok(BatteryPassportStubArtifact {
        artifact_type: BATTERY_PASSPORT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        schema_label: "engineer_facing_battery_passport_support_stub_v1".to_string(),
        dataset_name: "NASA PCoE Battery Dataset".to_string(),
        cell_id: "B0005".to_string(),
        audit_trace_contract_present: true,
        config_hash: hash_json(config)?,
        input_hash: hash_json(raw_input)?,
        first_boundary_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Boundary)
            .map(|sample| sample.cycle),
        first_violation_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Violation)
            .map(|sample| sample.cycle),
        lead_time_vs_threshold_cycles,
        primary_reason_code: trajectory.iter().find_map(|sample| sample.reason_code),
        maintenance_relevant_annotation:
            "DSFB advisory outputs can support lifecycle trend traceability and maintenance-oriented review; they are not a legal battery passport claim.".to_string(),
        notes: vec![
            "This stub maps DSFB traceability fields into passport-style lifecycle support fields.".to_string(),
            "No claim is made of CIRPASS or EU Battery Regulation compliance approval.".to_string(),
        ],
    })
}

fn build_seu_resilience_artifact(
    trajectory: &[BatteryResidual],
    config: &PipelineConfig,
    envelope_rho: f64,
) -> Result<AddendumSeuResilienceArtifact, AddendumError> {
    if trajectory.is_empty() {
        return Err(AddendumError::EmptyInput);
    }

    let mut drift_counter = 0usize;
    let mut slew_counter = 0usize;
    let mut redundant_state_match = true;
    let mut redundant_reason_match = true;
    let mut digest = Sha256::new();

    for sample in trajectory {
        drift_counter = next_persistence_count(drift_counter, sample.sign.d < -config.drift_threshold);
        slew_counter = next_persistence_count(slew_counter, sample.sign.s < -config.slew_threshold);

        let envelope = crate::types::EnvelopeParams {
            mu: 0.0,
            sigma: envelope_rho / 3.0,
            rho: envelope_rho,
        };
        let state_a = evaluate_grammar_state(
            sample.sign.r,
            &envelope,
            sample.sign.d,
            sample.sign.s,
            drift_counter,
            slew_counter,
            config,
        );
        let state_b = evaluate_grammar_state(
            sample.sign.r,
            &envelope,
            sample.sign.d,
            sample.sign.s,
            drift_counter,
            slew_counter,
            config,
        );
        if state_a != state_b || state_a != sample.grammar_state {
            redundant_state_match = false;
        }

        let reason_a = assign_reason_code(
            &sample.sign,
            state_a,
            drift_counter,
            slew_counter,
            config,
        );
        let reason_b = assign_reason_code(
            &sample.sign,
            state_b,
            drift_counter,
            slew_counter,
            config,
        );
        if reason_a != reason_b {
            redundant_reason_match = false;
        }

        digest.update(sample.cycle.to_le_bytes());
        digest.update([grammar_state_code(sample.grammar_state)]);
        digest.update(reason_code_numeric(sample.reason_code).to_le_bytes());
    }

    Ok(AddendumSeuResilienceArtifact {
        artifact_type: SEU_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        sampled_cycles: trajectory.len(),
        redundant_state_match,
        redundant_reason_match,
        invalid_state_detected: false,
        status_checksum: hash_digest(digest.finalize()),
        notes: vec![
            "This helper performs redundant evaluation and checksum generation over the advisory trajectory.".to_string(),
            "It is an SEU-oriented scaffold and does not claim rad-hard qualification.".to_string(),
        ],
    })
}

fn build_mission_bus_signal_dictionary() -> MissionBusSignalDictionary {
    MissionBusSignalDictionary {
        signal_dictionary_version: "1.0.0".to_string(),
        transport_agnostic_fields: vec![
            "grammar_state".to_string(),
            "tri_state_color".to_string(),
            "reason_code".to_string(),
            "validity_token.sequence_id".to_string(),
            "validity_token.stream_valid".to_string(),
            "lead_time_vs_threshold_cycles".to_string(),
        ],
        mil_std_1553_mapping: vec![
            "Word 1: state_code + color_code + advisory_valid".to_string(),
            "Word 2: reason_code + sequence_id".to_string(),
        ],
        arinc_429_mapping: vec![
            "Label A: state/color summary".to_string(),
            "Label B: reason code + validity flags".to_string(),
        ],
        arinc_664_mapping: vec![
            "VL payload field: JSON-like advisory summary".to_string(),
            "VL payload field: validity token and sequence metadata".to_string(),
        ],
    }
}

fn tri_state_color(state: GrammarState) -> &'static str {
    match state {
        GrammarState::Admissible => "Green",
        GrammarState::Boundary => "Yellow",
        GrammarState::Violation => "Red",
    }
}

fn tri_state_legend() -> Vec<TriStateLegendEntry> {
    vec![
        TriStateLegendEntry {
            state: GrammarState::Admissible,
            color: "Green".to_string(),
            operator_text: "normal".to_string(),
        },
        TriStateLegendEntry {
            state: GrammarState::Boundary,
            color: "Yellow".to_string(),
            operator_text: "caution / maintenance attention".to_string(),
        },
        TriStateLegendEntry {
            state: GrammarState::Violation,
            color: "Red".to_string(),
            operator_text: "structural alarm / intervention review".to_string(),
        },
    ]
}

fn operator_text(state: GrammarState, reason_code: Option<ReasonCode>) -> String {
    let state_text = match state {
        GrammarState::Admissible => {
            "Green: normal. Advisory-only monitoring indicates the current sample remains inside the admissibility envelope."
        }
        GrammarState::Boundary => {
            "Yellow: caution / maintenance attention. Advisory-only monitoring indicates structural deviation approaching the admissibility boundary."
        }
        GrammarState::Violation => {
            "Red: structural alarm / intervention review. Advisory-only monitoring indicates an envelope-exit condition under the current DSFB rules."
        }
    };
    match reason_code {
        Some(reason) => format!("{state_text} Reason code: {reason}."),
        None => state_text.to_string(),
    }
}

fn render_overlay_text(overlay: &AddendumOverlayArtifact) -> String {
    [
        "DSFB Zero-Burden Operator Overlay".to_string(),
        format!("Generated at: {}", overlay.generated_at_utc),
        format!("State: {}", overlay.grammar_state),
        format!("Color: {}", overlay.tri_state_color),
        format!("Reason code: {:?}", overlay.reason_code),
        format!(
            "Lead time vs 85% threshold: {:?}",
            overlay.lead_time_vs_threshold_cycles
        ),
        format!(
            "Lead time vs tactical margin ({:.0}%): {:?}",
            overlay.tactical_margin.threshold_fraction * 100.0,
            overlay.tactical_margin.lead_time_vs_margin_cycles
        ),
        format!(
            "Validity token sequence: {} (stream_valid={})",
            overlay.validity_token.sequence_id, overlay.validity_token.stream_valid
        ),
        format!("Advisory: {}", overlay.advisory_text),
    ]
    .join("\n")
}

fn write_overlay_timeline_csv(
    trajectory: &[BatteryResidual],
    path: &Path,
) -> Result<(), AddendumError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "cycle",
        "grammar_state",
        "tri_state_color",
        "reason_code",
        "advisory_text",
    ])?;
    for sample in trajectory {
        writer.write_record([
            sample.cycle.to_string(),
            sample.grammar_state.to_string(),
            tri_state_color(sample.grammar_state).to_string(),
            sample
                .reason_code
                .map(|reason| reason.to_string())
                .unwrap_or_default(),
            operator_text(sample.grammar_state, sample.reason_code),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn render_implementation_summary(
    summary: &AddendumImplementationSummary,
    source_path: &Path,
    overlay: &AddendumOverlayArtifact,
    t_star: usize,
    compliance_support_output: &str,
) -> String {
    let mut lines = vec![
        "DSFB Addendum Implementation Summary".to_string(),
        "".to_string(),
        format!("Artifact type: {}", summary.artifact_type),
        format!("Output root: {}", summary.output_root),
        format!("Input source: {}", source_path.display()),
        format!("Final overlay state: {}", overlay.grammar_state),
        format!("Final overlay color: {}", overlay.tri_state_color),
        format!("Theorem t_star observed in addendum helper run: {}", t_star),
        format!("Nested compliance support output: {}", compliance_support_output),
        "".to_string(),
        "Fully implemented:".to_string(),
    ];

    for item in &summary.fully_implemented {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push("Scaffolded:".to_string());
    for item in &summary.scaffolded {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push("Wrappers added:".to_string());
    for item in &summary.wrappers_added {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push("Compliance/support matrices added:".to_string());
    for item in &summary.compliance_support_matrices {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push("Integrity and resilience helpers added:".to_string());
    for item in &summary.integrity_and_resilience_helpers {
        lines.push(format!("- {}", item));
    }

    lines.push("".to_string());
    lines.push(summary.no_production_modification_statement.clone());
    lines.join("\n")
}

fn grammar_state_code(state: GrammarState) -> u8 {
    match state {
        GrammarState::Admissible => 0,
        GrammarState::Boundary => 1,
        GrammarState::Violation => 2,
    }
}

fn reason_code_numeric(reason_code: Option<ReasonCode>) -> i32 {
    match reason_code {
        None => -1,
        Some(ReasonCode::SustainedCapacityFade) => 0,
        Some(ReasonCode::AbruptResistanceSpike) => 1,
        Some(ReasonCode::RecurrentVoltageGrazing) => 2,
        Some(ReasonCode::ThermalDriftCoupling) => 3,
        Some(ReasonCode::PackImbalanceExpansion) => 4,
        Some(ReasonCode::AcceleratingFadeKnee) => 5,
        Some(ReasonCode::PossibleLithiumPlatingSignature) => 6,
        Some(ReasonCode::TransientThermalExcursionNotPersistent) => 7,
    }
}

fn file_size(path: PathBuf) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.len())
}

fn hash_json<T: Serialize + ?Sized>(value: &T) -> Result<String, AddendumError> {
    let bytes = serde_json::to_vec(value)?;
    Ok(hash_digest(Sha256::digest(bytes)))
}

fn hash_string(input: &str) -> String {
    hash_digest(Sha256::digest(input.as_bytes()))
}

fn hash_digest<D: AsRef<[u8]>>(digest: D) -> String {
    let mut output = String::from("sha256:");
    for byte in digest.as_ref() {
        let _ = write!(&mut output, "{:02x}", byte);
    }
    output
}

fn write_json<T: Serialize>(value: &T, path: &Path) -> Result<(), AddendumError> {
    write_text(&serde_json::to_string_pretty(value)?, path)
}

fn write_text(contents: &str, path: &Path) -> Result<(), AddendumError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
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
    fn resolve_addendum_output_dir_defaults_to_timestamped_directory() {
        let crate_dir = Path::new("/tmp/dsfb-battery");
        let output = resolve_addendum_output_dir(crate_dir, None);
        assert!(
            output
                .display()
                .to_string()
                .contains("outputs/addendum/dsfb_battery_addendum_")
        );
    }

    #[test]
    fn tri_state_mapping_matches_declared_colors() {
        assert_eq!(tri_state_color(GrammarState::Admissible), "Green");
        assert_eq!(tri_state_color(GrammarState::Boundary), "Yellow");
        assert_eq!(tri_state_color(GrammarState::Violation), "Red");
    }

    #[test]
    fn tamper_evident_trace_detects_mutation() {
        let raw_input = sample_raw_input();
        let capacities: Vec<f64> = raw_input.iter().map(|(_, capacity)| *capacity).collect();
        let config = PipelineConfig::default();
        let (_, trajectory) = run_dsfb_pipeline(&capacities, &config).unwrap();
        let mut trace = build_tamper_evident_trace(&raw_input, &trajectory, &config).unwrap();
        trace.records[0].residual_value += 0.1;
        let verification = tamper_evident_verification(&trace);
        assert!(!verification.valid);
        assert_eq!(verification.first_failed_index, Some(0));
    }

    #[test]
    fn addendum_workflow_writes_only_to_its_output_directory() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let output_dir = unique_temp_dir("dsfb-battery-addendum");
        let raw_input = sample_raw_input();

        let summary = run_addendum_workflow_from_input(
            crate_dir,
            &raw_input,
            Path::new("synthetic/b0005_like.csv"),
            &output_dir,
        )
        .unwrap();

        assert!(output_dir.join("implementation_summary.txt").exists());
        assert!(output_dir.join("swapc").join("swapc_report.txt").exists());
        assert!(
            output_dir
                .join("integrity")
                .join("tamper_evident_trace.json")
                .exists()
        );
        assert!(
            output_dir
                .join("operator_overlay")
                .join("decision_support_overlay.json")
                .exists()
        );
        assert!(
            output_dir
                .join("battery_passport")
                .join("battery_passport_stub.json")
                .exists()
        );
        assert!(
            output_dir
                .join("seu_resilience")
                .join("seu_resilience_report.json")
                .exists()
        );
        assert!(output_dir.join("mission_bus_signal_dictionary.json").exists());
        assert!(!output_dir.join("stage2_detection_results.json").exists());
        assert!(!output_dir.join("fig01_capacity_fade.svg").exists());
        assert_eq!(
            summary.no_production_modification_statement,
            NO_PRODUCTION_MODIFICATION_STATEMENT
        );

        let _ = fs::remove_dir_all(&output_dir);
    }
}
