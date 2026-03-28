// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Audit-trace contract
//
// Structured, serde-serializable contract for the Stage II JSON artifact.

use crate::export::Stage2Results;
use crate::detection::classification_is_emitted;
use crate::types::{BatteryResidual, GrammarState, PipelineConfig, ReasonCode};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::Path;
use thiserror::Error;

const SCHEMA_VERSION: &str = "1.0.0";
const ARTIFACT_TYPE: &str = "dsfb_battery_audit_trace";
const PAPER_VERSION: &str = "1.0";
const BENCHMARK_ID: &str = "stage-ii-b0005-capacity";
const REGIME_TAG: &str = "nasa_pcoe_b0005_capacity_only";
const PRIMARY_JSON_NAME: &str = "stage2_detection_results.json";

#[derive(Debug, Error)]
pub enum AuditTraceError {
    #[error("raw input series is empty")]
    EmptyInput,
    #[error("trajectory is empty")]
    EmptyTrajectory,
    #[error("raw input length ({input_len}) does not match trajectory length ({trajectory_len})")]
    LengthMismatch {
        input_len: usize,
        trajectory_len: usize,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputContract {
    pub kind: String,
    pub deterministic: bool,
    pub self_documenting: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunMetadata {
    pub run_id: String,
    pub generated_at_utc: String,
    pub crate_name: String,
    pub crate_version: String,
    pub paper_version: String,
    pub benchmark_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_commit: Option<String>,
    pub config_hash: String,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetDescriptor {
    pub dataset_name: String,
    pub cell_id: String,
    pub trajectory_unit: String,
    pub channel_scope: Vec<String>,
    pub source_artifact: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceContract {
    pub deployment_mode: String,
    pub read_only: bool,
    pub protocol_independent: bool,
    pub requires_cloud_connectivity: bool,
    pub requires_model_retraining: bool,
    pub advisory_only: bool,
    pub fail_silent_on_invalid_stream: bool,
    pub fail_silent_defined: bool,
    pub fail_silent_enforced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkConfiguration {
    pub healthy_window: usize,
    pub drift_window: usize,
    pub drift_persistence: usize,
    pub slew_persistence: usize,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
    pub eol_fraction: f64,
    pub boundary_fraction: f64,
    pub threshold_baseline_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryOutcome {
    pub initial_state: GrammarState,
    pub final_state: GrammarState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_boundary_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_violation_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_85pct_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_80pct_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_85pct_cycles: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_80pct_cycles: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_star: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_elevation_confirmed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_reason_code: Option<ReasonCode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvidence {
    pub residual_value: f64,
    pub drift_value: f64,
    pub slew_value: f64,
    pub boundary_fraction: f64,
    pub inside_admissibility_envelope: bool,
    pub near_envelope: bool,
    pub persistent_outward_drift: bool,
    pub persistent_accelerating_slew: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThresholdsInForce {
    pub boundary_threshold: f64,
    pub violation_threshold: f64,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
    pub persistence_length_drift: usize,
    pub persistence_length_slew: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistenceCounters {
    pub drift_counter: usize,
    pub slew_counter: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditFields {
    pub config_hash: String,
    pub input_hash: String,
    pub stream_valid: bool,
    pub suppressed_due_to_regime_gate: bool,
    pub suppressed_due_to_invalid_stream: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassificationSampleEvent {
    pub event_id: String,
    pub cycle_index: usize,
    pub timestamp_utc: Option<String>,
    pub classification: GrammarState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<ReasonCode>,
    pub regime_tag: String,
    pub channel: String,
    pub evidence: AuditEvidence,
    pub thresholds_in_force: ThresholdsInForce,
    pub persistence_counters: PersistenceCounters,
    pub trigger_conditions_met: Vec<String>,
    pub audit_fields: AuditFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateTransitionEvent {
    pub event_id: String,
    pub cycle_index: usize,
    pub timestamp_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<GrammarState>,
    pub current_state: GrammarState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<ReasonCode>,
    pub regime_tag: String,
    pub channel: String,
    pub evidence: AuditEvidence,
    pub thresholds_in_force: ThresholdsInForce,
    pub persistence_counters: PersistenceCounters,
    pub trigger_conditions_met: Vec<String>,
    pub explanatory_text: String,
    pub audit_fields: AuditFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvalidStreamGapEvent {
    pub event_id: String,
    pub cycle_index_start: usize,
    pub cycle_index_end: usize,
    pub timestamp_utc: Option<String>,
    pub channel: String,
    pub reason_code: ReasonCode,
    pub explanatory_text: String,
    pub audit_fields: AuditFields,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunSummaryMarkerEvent {
    pub event_id: String,
    pub cycle_index: usize,
    pub timestamp_utc: Option<String>,
    pub final_classification: GrammarState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_boundary_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_violation_cycle: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_85pct_cycles: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_time_vs_80pct_cycles: Option<i64>,
    pub advisory_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AuditEvent {
    ClassificationSample(ClassificationSampleEvent),
    StateTransition(StateTransitionEvent),
    InvalidStreamGap(InvalidStreamGapEvent),
    RunSummaryMarker(RunSummaryMarkerEvent),
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureModeObservation {
    pub interpretation: ReasonCode,
    pub first_observed_cycle: usize,
    pub last_observed_cycle: usize,
    pub sample_count: usize,
    pub advisory_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactManifest {
    pub primary_json: String,
    pub supporting_figures: Vec<String>,
    pub supporting_tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stage2AuditTraceArtifact {
    pub schema_version: String,
    pub artifact_type: String,
    pub output_contract: OutputContract,
    pub run_metadata: RunMetadata,
    pub dataset: DatasetDescriptor,
    pub interface_contract: InterfaceContract,
    pub benchmark_configuration: BenchmarkConfiguration,
    pub summary_outcome: SummaryOutcome,
    pub audit_trace: Vec<AuditEvent>,
    pub failure_mode_observations: Vec<FailureModeObservation>,
    pub artifact_manifest: ArtifactManifest,
    #[serde(flatten)]
    pub legacy_summary: Stage2Results,
}

#[derive(Debug, Clone)]
pub struct AuditTraceBuildContext<'a> {
    pub results: &'a Stage2Results,
    pub raw_input: &'a [(usize, f64)],
    pub trajectory: &'a [BatteryResidual],
    pub source_artifact: Option<&'a Path>,
    pub supporting_figures: &'a [String],
    pub supporting_tables: &'a [String],
}

#[derive(Debug, Clone)]
struct CycleAuditSupport {
    evidence: AuditEvidence,
    persistence_counters: PersistenceCounters,
    trigger_conditions_met: Vec<String>,
}

pub fn build_stage2_audit_trace(
    ctx: AuditTraceBuildContext<'_>,
) -> Result<Stage2AuditTraceArtifact, AuditTraceError> {
    if ctx.raw_input.is_empty() {
        return Err(AuditTraceError::EmptyInput);
    }
    if ctx.trajectory.is_empty() {
        return Err(AuditTraceError::EmptyTrajectory);
    }
    if ctx.raw_input.len() != ctx.trajectory.len() {
        return Err(AuditTraceError::LengthMismatch {
            input_len: ctx.raw_input.len(),
            trajectory_len: ctx.trajectory.len(),
        });
    }

    let config_hash = hash_pipeline_config(&ctx.results.config);
    let input_hash = hash_input_series(ctx.raw_input);
    let run_id = hash_prefixed(
        format!("{}|{}", config_hash, input_hash).as_bytes(),
        "run",
        16,
    );
    let thresholds = thresholds_in_force(&ctx.results.config);
    let supports = build_cycle_audit_support(ctx.trajectory, ctx.results);
    let summary_outcome = build_summary_outcome(ctx.trajectory, ctx.results);
    let audit_trace = build_audit_trace(
        ctx.trajectory,
        &thresholds,
        &supports,
        &summary_outcome,
        &config_hash,
        &input_hash,
    );

    Ok(Stage2AuditTraceArtifact {
        schema_version: SCHEMA_VERSION.to_string(),
        artifact_type: ARTIFACT_TYPE.to_string(),
        output_contract: OutputContract {
            kind: "audit_trace".to_string(),
            deterministic: true,
            self_documenting: true,
        },
        run_metadata: RunMetadata {
            run_id,
            generated_at_utc: Utc::now().to_rfc3339(),
            crate_name: env!("CARGO_PKG_NAME").to_string(),
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            paper_version: PAPER_VERSION.to_string(),
            benchmark_id: BENCHMARK_ID.to_string(),
            code_commit: None,
            config_hash: config_hash.clone(),
            input_hash: input_hash.clone(),
        },
        dataset: DatasetDescriptor {
            dataset_name: "NASA PCoE Battery Dataset".to_string(),
            cell_id: "B0005".to_string(),
            trajectory_unit: "cycle".to_string(),
            channel_scope: vec!["capacity".to_string()],
            source_artifact: source_artifact_label(ctx.source_artifact),
        },
        interface_contract: InterfaceContract {
            deployment_mode: "offline".to_string(),
            read_only: true,
            protocol_independent: true,
            requires_cloud_connectivity: false,
            requires_model_retraining: false,
            advisory_only: true,
            fail_silent_on_invalid_stream: true,
            fail_silent_defined: true,
            fail_silent_enforced: true,
        },
        benchmark_configuration: BenchmarkConfiguration {
            healthy_window: ctx.results.config.healthy_window,
            drift_window: ctx.results.config.drift_window,
            drift_persistence: ctx.results.config.drift_persistence,
            slew_persistence: ctx.results.config.slew_persistence,
            drift_threshold: ctx.results.config.drift_threshold,
            slew_threshold: ctx.results.config.slew_threshold,
            eol_fraction: ctx.results.config.eol_fraction,
            boundary_fraction: ctx.results.config.boundary_fraction,
            threshold_baseline_fraction: 0.85,
        },
        summary_outcome,
        audit_trace,
        failure_mode_observations: build_failure_mode_observations(ctx.trajectory),
        artifact_manifest: ArtifactManifest {
            primary_json: PRIMARY_JSON_NAME.to_string(),
            supporting_figures: ctx.supporting_figures.to_vec(),
            supporting_tables: ctx.supporting_tables.to_vec(),
        },
        legacy_summary: ctx.results.clone(),
    })
}

fn source_artifact_label(source_artifact: Option<&Path>) -> String {
    source_artifact
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "input_series".to_string())
}

fn thresholds_in_force(config: &PipelineConfig) -> ThresholdsInForce {
    ThresholdsInForce {
        boundary_threshold: config.boundary_fraction,
        violation_threshold: 1.0,
        drift_threshold: config.drift_threshold,
        slew_threshold: config.slew_threshold,
        persistence_length_drift: config.drift_persistence,
        persistence_length_slew: config.slew_persistence,
    }
}

fn build_cycle_audit_support(
    trajectory: &[BatteryResidual],
    results: &Stage2Results,
) -> Vec<CycleAuditSupport> {
    let mut drift_counter = 0usize;
    let mut slew_counter = 0usize;

    trajectory
        .iter()
        .map(|sample| {
            if sample.sign.d < -results.config.drift_threshold {
                drift_counter += 1;
            } else {
                drift_counter = 0;
            }

            if sample.sign.s < -results.config.slew_threshold {
                slew_counter += 1;
            } else {
                slew_counter = 0;
            }

            let residual_fraction = if results.envelope.rho > 0.0 {
                sample.sign.r.abs() / results.envelope.rho
            } else {
                0.0
            };
            let inside_admissibility_envelope = sample.sign.r.abs() <= results.envelope.rho;
            let near_envelope =
                sample.sign.r.abs() > results.config.boundary_fraction * results.envelope.rho;
            let persistent_outward_drift = drift_counter >= results.config.drift_persistence;
            let persistent_accelerating_slew =
                slew_counter >= results.config.slew_persistence && persistent_outward_drift;

            let mut trigger_conditions_met = Vec::new();
            if near_envelope {
                trigger_conditions_met.push("boundary_proximity".to_string());
            }
            if sample.sign.r.abs() > results.envelope.rho {
                trigger_conditions_met.push("envelope_exit".to_string());
            }
            if persistent_outward_drift {
                trigger_conditions_met.push("persistent_outward_drift".to_string());
            }
            if persistent_accelerating_slew {
                trigger_conditions_met.push("persistent_accelerating_slew".to_string());
            }

            CycleAuditSupport {
                evidence: AuditEvidence {
                    residual_value: sample.sign.r,
                    drift_value: sample.sign.d,
                    slew_value: sample.sign.s,
                    boundary_fraction: residual_fraction,
                    inside_admissibility_envelope,
                    near_envelope,
                    persistent_outward_drift,
                    persistent_accelerating_slew,
                },
                persistence_counters: PersistenceCounters {
                    drift_counter,
                    slew_counter,
                },
                trigger_conditions_met,
            }
        })
        .collect()
}

fn build_summary_outcome(
    trajectory: &[BatteryResidual],
    results: &Stage2Results,
) -> SummaryOutcome {
    let first_boundary_cycle = trajectory
        .iter()
        .filter(|sample| classification_is_emitted(sample))
        .find(|sample| sample.grammar_state == GrammarState::Boundary)
        .map(|sample| sample.cycle);
    let first_violation_cycle = trajectory
        .iter()
        .filter(|sample| classification_is_emitted(sample))
        .find(|sample| sample.grammar_state == GrammarState::Violation)
        .map(|sample| sample.cycle);
    let first_non_admissible_cycle = first_boundary_cycle.or(first_violation_cycle);
    let capacity_85pct_cycle = results.threshold_detection.alarm_cycle;
    let capacity_80pct_cycle = results.dsfb_detection.eol_cycle;
    let lead_time_vs_85pct_cycles = first_non_admissible_cycle
        .zip(capacity_85pct_cycle)
        .map(|(a, b)| b as i64 - a as i64);
    let lead_time_vs_80pct_cycles = first_non_admissible_cycle
        .zip(capacity_80pct_cycle)
        .map(|(a, b)| b as i64 - a as i64);
    let persistent_elevation_confirmed = first_non_admissible_cycle.map(|start_cycle| {
        trajectory
            .iter()
            .skip(start_cycle.saturating_sub(1))
            .take(2)
            .all(|sample| sample.grammar_state != GrammarState::Admissible)
    });
    let primary_reason_code = first_non_admissible_cycle
        .and_then(|cycle| trajectory.iter().find(|sample| sample.cycle == cycle))
        .and_then(|sample| sample.reason_code)
        .or_else(|| {
            trajectory
                .iter()
                .filter(|sample| classification_is_emitted(sample))
                .find_map(|sample| sample.reason_code)
        });

    let initial_state = trajectory
        .iter()
        .find(|sample| classification_is_emitted(sample))
        .map(|sample| sample.grammar_state)
        .unwrap_or(GrammarState::Admissible);
    let final_state = trajectory
        .iter()
        .rev()
        .find(|sample| classification_is_emitted(sample))
        .map(|sample| sample.grammar_state)
        .unwrap_or(GrammarState::Admissible);

    SummaryOutcome {
        initial_state,
        final_state,
        first_boundary_cycle,
        first_violation_cycle,
        capacity_85pct_cycle,
        capacity_80pct_cycle,
        lead_time_vs_85pct_cycles,
        lead_time_vs_80pct_cycles,
        t_star: Some(results.theorem1.t_star),
        persistent_elevation_confirmed,
        primary_reason_code,
    }
}

fn build_audit_trace(
    trajectory: &[BatteryResidual],
    thresholds: &ThresholdsInForce,
    supports: &[CycleAuditSupport],
    summary: &SummaryOutcome,
    config_hash: &str,
    input_hash: &str,
) -> Vec<AuditEvent> {
    let mut events = Vec::with_capacity(trajectory.len() * 2 + 1);
    let mut last_emitted_state = None;
    let mut invalid_gap_start = None;
    let mut invalid_gap_end = None;

    for (index, sample) in trajectory.iter().enumerate() {
        let support = &supports[index];
        if !classification_is_emitted(sample) {
            if invalid_gap_start.is_none() {
                invalid_gap_start = Some(sample.cycle);
            }
            invalid_gap_end = Some(sample.cycle);
            continue;
        }

        if let (Some(start), Some(end)) = (invalid_gap_start.take(), invalid_gap_end.take()) {
            events.push(AuditEvent::InvalidStreamGap(InvalidStreamGapEvent {
                event_id: format!("evt-{start:06}-invalid-stream-gap"),
                cycle_index_start: start,
                cycle_index_end: end,
                timestamp_utc: None,
                channel: "capacity".to_string(),
                reason_code: ReasonCode::InvalidStreamSuppression,
                explanatory_text: "Classification suppressed because the upstream sample stream or its fixed-window residual/drift/slew terms were invalid for this interval under the fail-silent contract.".to_string(),
                audit_fields: AuditFields {
                    config_hash: config_hash.to_string(),
                    input_hash: input_hash.to_string(),
                    stream_valid: false,
                    suppressed_due_to_regime_gate: false,
                    suppressed_due_to_invalid_stream: true,
                },
            }));
        }

        let audit_fields = AuditFields {
            config_hash: config_hash.to_string(),
            input_hash: input_hash.to_string(),
            stream_valid: true,
            suppressed_due_to_regime_gate: false,
            suppressed_due_to_invalid_stream: false,
        };

        events.push(AuditEvent::ClassificationSample(
            ClassificationSampleEvent {
                event_id: format!("evt-{:06}-classification", sample.cycle),
                cycle_index: sample.cycle,
                timestamp_utc: None,
                classification: sample.grammar_state,
                interpretation: sample.reason_code,
                regime_tag: REGIME_TAG.to_string(),
                channel: "capacity".to_string(),
                evidence: support.evidence.clone(),
                thresholds_in_force: thresholds.clone(),
                persistence_counters: support.persistence_counters.clone(),
                trigger_conditions_met: support.trigger_conditions_met.clone(),
                audit_fields: audit_fields.clone(),
            },
        ));

        let previous_state = last_emitted_state;

        // Emit a transition whenever the observed per-cycle classification changes.
        // The current stage II artifact records those changes as-is; it does not
        // impose a monotone-only progression on the grammar states.
        let should_emit_transition = match previous_state {
            Some(previous) => previous != sample.grammar_state,
            None => sample.grammar_state != GrammarState::Admissible,
        };

        if should_emit_transition {
            events.push(AuditEvent::StateTransition(StateTransitionEvent {
                event_id: transition_event_id(sample.cycle, previous_state, sample.grammar_state),
                cycle_index: sample.cycle,
                timestamp_utc: None,
                previous_state,
                current_state: sample.grammar_state,
                reason_code: sample.reason_code,
                regime_tag: REGIME_TAG.to_string(),
                channel: "capacity".to_string(),
                evidence: support.evidence.clone(),
                thresholds_in_force: thresholds.clone(),
                persistence_counters: support.persistence_counters.clone(),
                trigger_conditions_met: support.trigger_conditions_met.clone(),
                explanatory_text: transition_explanation(sample, support),
                audit_fields,
            }));
        }

        last_emitted_state = Some(sample.grammar_state);
    }

    if let (Some(start), Some(end)) = (invalid_gap_start.take(), invalid_gap_end.take()) {
        events.push(AuditEvent::InvalidStreamGap(InvalidStreamGapEvent {
            event_id: format!("evt-{start:06}-invalid-stream-gap"),
            cycle_index_start: start,
            cycle_index_end: end,
            timestamp_utc: None,
            channel: "capacity".to_string(),
            reason_code: ReasonCode::InvalidStreamSuppression,
            explanatory_text: "Classification suppressed because the upstream sample stream or its fixed-window residual/drift/slew terms were invalid for this interval under the fail-silent contract.".to_string(),
            audit_fields: AuditFields {
                config_hash: config_hash.to_string(),
                input_hash: input_hash.to_string(),
                stream_valid: false,
                suppressed_due_to_regime_gate: false,
                suppressed_due_to_invalid_stream: true,
            },
        }));
    }

    let final_cycle = trajectory[trajectory.len() - 1].cycle;
    let final_classification = trajectory
        .iter()
        .rev()
        .find(|sample| classification_is_emitted(sample))
        .map(|sample| sample.grammar_state)
        .unwrap_or(GrammarState::Admissible);
    events.push(AuditEvent::RunSummaryMarker(RunSummaryMarkerEvent {
        event_id: "evt-run-summary".to_string(),
        cycle_index: final_cycle,
        timestamp_utc: None,
        final_classification,
        first_boundary_cycle: summary.first_boundary_cycle,
        first_violation_cycle: summary.first_violation_cycle,
        lead_time_vs_85pct_cycles: summary.lead_time_vs_85pct_cycles,
        lead_time_vs_80pct_cycles: summary.lead_time_vs_80pct_cycles,
        advisory_text: "Stage II advisory output completed for the capacity-only audit trace."
            .to_string(),
    }));

    events
}

fn transition_event_id(
    cycle: usize,
    previous_state: Option<GrammarState>,
    current_state: GrammarState,
) -> String {
    let previous = previous_state
        .map(grammar_state_slug)
        .unwrap_or_else(|| "start".to_string());
    let current = grammar_state_slug(current_state);
    format!("evt-{cycle:06}-{previous}-to-{current}")
}

fn grammar_state_slug(state: GrammarState) -> String {
    match state {
        GrammarState::Admissible => "admissible".to_string(),
        GrammarState::Boundary => "boundary".to_string(),
        GrammarState::Violation => "violation".to_string(),
    }
}

fn transition_explanation(sample: &BatteryResidual, support: &CycleAuditSupport) -> String {
    match sample.grammar_state {
        GrammarState::Violation => {
            "Classification moved to Violation because the residual exited the admissibility envelope under the current stage II capacity trace."
                .to_string()
        }
        GrammarState::Boundary => {
            let mut clauses = Vec::new();
            if support
                .trigger_conditions_met
                .iter()
                .any(|value| value == "persistent_outward_drift")
            {
                clauses.push("persistent outward residual drift");
            }
            if support
                .trigger_conditions_met
                .iter()
                .any(|value| value == "persistent_accelerating_slew")
            {
                clauses.push("persistent accelerating slew");
            }
            if support
                .trigger_conditions_met
                .iter()
                .any(|value| value == "boundary_proximity")
            {
                clauses.push("boundary-envelope proximity");
            }

            if clauses.is_empty() {
                "Classification moved to Boundary based on the configured stage II capacity interpretation rules."
                    .to_string()
            } else {
                format!(
                    "Classification moved to Boundary because {} was observed before envelope exit.",
                    clauses.join(" with ")
                )
            }
        }
        GrammarState::Admissible => {
            "Classification returned to Admissible because the residual was inside the admissibility envelope and no persistence-gated boundary conditions were active under the current stage II capacity interpretation rules."
                .to_string()
        }
    }
}

fn build_failure_mode_observations(trajectory: &[BatteryResidual]) -> Vec<FailureModeObservation> {
    let mut observations: Vec<FailureModeObservation> = Vec::new();

    for sample in trajectory
        .iter()
        .filter(|sample| {
            sample.reason_code.is_some()
                && sample.reason_code != Some(ReasonCode::InvalidStreamSuppression)
        })
    {
        let reason_code = sample.reason_code.unwrap();
        if let Some(existing) = observations
            .iter_mut()
            .find(|observation| observation.interpretation == reason_code)
        {
            existing.last_observed_cycle = sample.cycle;
            existing.sample_count += 1;
            continue;
        }

        observations.push(FailureModeObservation {
            interpretation: reason_code,
            first_observed_cycle: sample.cycle,
            last_observed_cycle: sample.cycle,
            sample_count: 1,
            advisory_text: format!(
                "{} was observed in the stage II capacity channel as an advisory interpretation, not a decision.",
                reason_code
            ),
        });
    }

    observations
}

fn hash_pipeline_config(config: &PipelineConfig) -> String {
    let payload = format!(
        concat!(
            "healthy_window={}\n",
            "drift_window={}\n",
            "drift_persistence={}\n",
            "slew_persistence={}\n",
            "drift_threshold={:.12}\n",
            "slew_threshold={:.12}\n",
            "eol_fraction={:.12}\n",
            "boundary_fraction={:.12}\n",
        ),
        config.healthy_window,
        config.drift_window,
        config.drift_persistence,
        config.slew_persistence,
        config.drift_threshold,
        config.slew_threshold,
        config.eol_fraction,
        config.boundary_fraction,
    );
    hash_prefixed(payload.as_bytes(), "sha256", 64)
}

fn hash_input_series(raw_input: &[(usize, f64)]) -> String {
    let mut payload = String::new();
    for (cycle, capacity) in raw_input {
        let _ = writeln!(&mut payload, "{cycle},{capacity:.12}");
    }
    hash_prefixed(payload.as_bytes(), "sha256", 64)
}

fn hash_prefixed(bytes: &[u8], prefix: &str, chars: usize) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    format!("{prefix}:{}", &hex[..chars.min(hex.len())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::Stage2Results;
    use crate::types::{
        BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, PipelineConfig, ReasonCode,
        SignTuple, Theorem1Result,
    };

    fn sample_results() -> (Stage2Results, Vec<(usize, f64)>, Vec<BatteryResidual>) {
        let config = PipelineConfig {
            healthy_window: 2,
            drift_window: 1,
            drift_persistence: 1,
            slew_persistence: 1,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        };

        let raw_input = vec![
            (1, 2.0000),
            (2, 1.9950),
            (3, 1.9200),
            (4, 1.6800),
            (5, 1.5800),
        ];

        let trajectory = vec![
            BatteryResidual {
                cycle: 1,
                capacity_ah: 2.0000,
                sign: SignTuple {
                    r: 0.0025,
                    d: 0.0,
                    s: 0.0,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 2,
                capacity_ah: 1.9950,
                sign: SignTuple {
                    r: -0.0025,
                    d: -0.0015,
                    s: -0.0005,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 3,
                capacity_ah: 1.9200,
                sign: SignTuple {
                    r: -0.0400,
                    d: -0.0030,
                    s: -0.0012,
                },
                grammar_state: GrammarState::Boundary,
                reason_code: Some(ReasonCode::SustainedCapacityFade),
            },
            BatteryResidual {
                cycle: 4,
                capacity_ah: 1.6800,
                sign: SignTuple {
                    r: -0.0850,
                    d: -0.0040,
                    s: -0.0018,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
            BatteryResidual {
                cycle: 5,
                capacity_ah: 1.5800,
                sign: SignTuple {
                    r: -0.1200,
                    d: -0.0045,
                    s: -0.0015,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
        ];

        let results = Stage2Results {
            data_provenance:
                "NASA PCoE Battery Dataset, Cell B0005 (capacity-only synthetic contract sample)"
                    .to_string(),
            config,
            envelope: EnvelopeParams {
                mu: 1.9975,
                sigma: 0.0167,
                rho: 0.0500,
            },
            dsfb_detection: DetectionResult {
                method: "DSFB Structural Alarm".to_string(),
                alarm_cycle: Some(3),
                eol_cycle: Some(5),
                lead_time_cycles: Some(2),
            },
            threshold_detection: DetectionResult {
                method: "Threshold Baseline (85% of initial)".to_string(),
                alarm_cycle: Some(4),
                eol_cycle: Some(5),
                lead_time_cycles: Some(1),
            },
            theorem1: Theorem1Result {
                rho: 0.0500,
                alpha: 0.0035,
                kappa: 0.0,
                t_star: 15,
                actual_detection_cycle: Some(3),
                bound_satisfied: Some(true),
            },
        };

        (results, raw_input, trajectory)
    }

    #[test]
    fn audit_trace_contract_serializes_expected_structure() {
        let (results, raw_input, trajectory) = sample_results();
        let figures = vec!["fig06_grammar_state_timeline.svg".to_string()];
        let tables = vec!["semiotic_trajectory.csv".to_string()];
        let artifact = build_stage2_audit_trace(AuditTraceBuildContext {
            results: &results,
            raw_input: &raw_input,
            trajectory: &trajectory,
            source_artifact: Some(Path::new("data/nasa_b0005_capacity.csv")),
            supporting_figures: &figures,
            supporting_tables: &tables,
        })
        .unwrap();

        let value = serde_json::to_value(&artifact).unwrap();
        assert_eq!(value["schema_version"], "1.0.0");
        assert_eq!(value["artifact_type"], "dsfb_battery_audit_trace");
        assert_eq!(value["run_metadata"]["crate_name"], "dsfb-battery");
        assert_eq!(value["dataset"]["cell_id"], "B0005");
        assert_eq!(value["interface_contract"]["fail_silent_on_invalid_stream"], true);
        assert_eq!(value["interface_contract"]["fail_silent_defined"], true);
        assert_eq!(value["interface_contract"]["fail_silent_enforced"], true);
        assert_eq!(
            value["audit_trace"][0]["event_type"],
            "classification_sample"
        );
        assert!(value["audit_trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "state_transition"));
        assert_eq!(value["summary_outcome"]["first_boundary_cycle"], 3);
        assert_eq!(value["summary_outcome"]["first_violation_cycle"], 4);
        assert_eq!(value["summary_outcome"]["t_star"], 15);
    }

    #[test]
    fn audit_trace_records_return_to_admissible_transitions() {
        let config = PipelineConfig {
            healthy_window: 2,
            drift_window: 1,
            drift_persistence: 2,
            slew_persistence: 2,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        };

        let raw_input = vec![(1, 2.0000), (2, 1.9000), (3, 1.9750)];
        let trajectory = vec![
            BatteryResidual {
                cycle: 1,
                capacity_ah: 2.0000,
                sign: SignTuple {
                    r: 0.0000,
                    d: 0.0,
                    s: 0.0,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 2,
                capacity_ah: 1.9000,
                sign: SignTuple {
                    r: -0.0900,
                    d: -0.0040,
                    s: -0.0015,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::SustainedCapacityFade),
            },
            BatteryResidual {
                cycle: 3,
                capacity_ah: 1.9750,
                sign: SignTuple {
                    r: -0.0100,
                    d: 0.0005,
                    s: 0.0002,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
        ];

        let results = Stage2Results {
            data_provenance: "Synthetic return-transition sample".to_string(),
            config,
            envelope: EnvelopeParams {
                mu: 1.9875,
                sigma: 0.0200,
                rho: 0.0500,
            },
            dsfb_detection: DetectionResult {
                method: "DSFB Structural Alarm".to_string(),
                alarm_cycle: Some(2),
                eol_cycle: None,
                lead_time_cycles: None,
            },
            threshold_detection: DetectionResult {
                method: "Threshold Baseline (85% of initial)".to_string(),
                alarm_cycle: None,
                eol_cycle: None,
                lead_time_cycles: None,
            },
            theorem1: Theorem1Result {
                rho: 0.0500,
                alpha: 0.0040,
                kappa: 0.0,
                t_star: 13,
                actual_detection_cycle: Some(2),
                bound_satisfied: Some(true),
            },
        };

        let artifact = build_stage2_audit_trace(AuditTraceBuildContext {
            results: &results,
            raw_input: &raw_input,
            trajectory: &trajectory,
            source_artifact: None,
            supporting_figures: &[],
            supporting_tables: &[],
        })
        .unwrap();

        let value = serde_json::to_value(&artifact).unwrap();
        let return_transition = value["audit_trace"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| {
                event["event_type"] == "state_transition"
                    && event["previous_state"] == "Violation"
                    && event["current_state"] == "Admissible"
            })
            .unwrap();

        assert_eq!(return_transition["cycle_index"], 3);
        assert_eq!(return_transition["audit_fields"]["stream_valid"], true);
        assert_eq!(
            return_transition["audit_fields"]["suppressed_due_to_invalid_stream"],
            false
        );
        assert_eq!(
            return_transition["explanatory_text"],
            "Classification returned to Admissible because the residual was inside the admissibility envelope and no persistence-gated boundary conditions were active under the current stage II capacity interpretation rules."
        );
    }

    #[test]
    fn audit_trace_emits_invalid_stream_gap_and_suppresses_classification() {
        let config = PipelineConfig {
            healthy_window: 2,
            drift_window: 1,
            drift_persistence: 1,
            slew_persistence: 1,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        };
        let capacities = [2.0, 1.99, 1.95, f64::NAN, 1.93, 1.88, 1.84, 1.80];
        let raw_input: Vec<(usize, f64)> = capacities
            .iter()
            .enumerate()
            .map(|(index, capacity)| (index + 1, *capacity))
            .collect();
        let (envelope, trajectory) = crate::detection::run_dsfb_pipeline(&capacities, &config).unwrap();
        let eol_capacity = config.eol_fraction * capacities[0];
        let dsfb_detection = crate::detection::build_dsfb_detection(&trajectory, &capacities, eol_capacity);
        let threshold_detection =
            crate::detection::build_threshold_detection(&capacities, 0.85, eol_capacity);
        let theorem1 = crate::detection::verify_theorem1(&envelope, &trajectory, &config);
        let results = Stage2Results {
            data_provenance: "Synthetic invalid-stream sample".to_string(),
            config,
            envelope,
            dsfb_detection,
            threshold_detection,
            theorem1,
        };

        let artifact = build_stage2_audit_trace(AuditTraceBuildContext {
            results: &results,
            raw_input: &raw_input,
            trajectory: &trajectory,
            source_artifact: None,
            supporting_figures: &[],
            supporting_tables: &[],
        })
        .unwrap();

        let value = serde_json::to_value(&artifact).unwrap();
        let gap_event = value["audit_trace"]
            .as_array()
            .unwrap()
            .iter()
            .find(|event| event["event_type"] == "invalid_stream_gap")
            .unwrap();
        assert_eq!(gap_event["cycle_index_start"], 4);
        assert_eq!(gap_event["cycle_index_end"], 6);
        assert_eq!(gap_event["reason_code"], "InvalidStreamSuppression");
        assert_eq!(gap_event["audit_fields"]["stream_valid"], false);
        assert_eq!(
            gap_event["audit_fields"]["suppressed_due_to_invalid_stream"],
            true
        );
        assert!(!value["audit_trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                let event_type = event["event_type"].as_str().unwrap_or_default();
                let cycle = event["cycle_index"].as_u64().unwrap_or_default();
                (event_type == "classification_sample" || event_type == "state_transition")
                    && (4..=6).contains(&(cycle as usize))
            }));
        assert!(value["audit_trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| {
                event["event_type"] == "classification_sample"
                    && event["cycle_index"] == serde_json::Value::from(7)
            }));
    }
}
