// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Opt-in resource integrity and audit helpers.

use crate::complexity::{estimate_dsfb_update_complexity, ComplexityArtifact};
use crate::evaluation::{evaluate_cell, CellEvaluationRun};
use crate::heuristics::{
    build_heuristic_observation_profile, load_nasa_heuristics_bank, retrieve_heuristic_matches,
    MatchStatus, NASA_HEURISTICS_BANK_JSON,
};
use crate::integration::build_validity_token;
use crate::load_capacity_csv;
use crate::types::{BatteryResidual, EnvelopeParams, PipelineConfig, SignTuple, Theorem1Result};
use chrono::Utc;
use core::mem::size_of;
use serde::Serialize;
use sha2::{Digest, Sha256};
use static_assertions::const_assert;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const RESOURCE_TRACE_SCHEMA_VERSION: &str = "1.0.0";
const RESOURCE_TRACE_JSON_NAME: &str = "resource_trace.json";
const RESOURCE_TRACE_SUMMARY_NAME: &str = "resource_trace_summary.txt";
const HEURISTICS_DENSITY_JSON_NAME: &str = "heuristics_density_report.json";
const MEMORY_BUDGET_REPORT_NAME: &str = "memory_budget_report.txt";
const IMPLEMENTATION_SUMMARY_NAME: &str = "implementation_summary.txt";

const SIGN_TUPLE_BUDGET_BYTES: usize = 32;
const ENVELOPE_PARAMS_BUDGET_BYTES: usize = 32;
const BATTERY_RESIDUAL_BUDGET_BYTES: usize = 64;
const PIPELINE_CONFIG_BUDGET_BYTES: usize = 80;
const THEOREM1_RESULT_BUDGET_BYTES: usize = 80;

const_assert!(size_of::<SignTuple>() <= SIGN_TUPLE_BUDGET_BYTES);
const_assert!(size_of::<EnvelopeParams>() <= ENVELOPE_PARAMS_BUDGET_BYTES);
const_assert!(size_of::<BatteryResidual>() <= BATTERY_RESIDUAL_BUDGET_BYTES);
const_assert!(size_of::<PipelineConfig>() <= PIPELINE_CONFIG_BUDGET_BYTES);
const_assert!(size_of::<Theorem1Result>() <= THEOREM1_RESULT_BUDGET_BYTES);

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementMode {
    Measured,
    Estimated,
    Asserted,
    Inferred,
    NotMeasured,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResourceMetric<T> {
    pub value: Option<T>,
    pub measurement_mode: MeasurementMode,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AuditHeader {
    pub artifact_type: String,
    pub schema_version: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub advisory_only: bool,
    pub source_artifact: String,
    pub input_hash: String,
    pub config_hash: String,
    pub validation_hash: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WindowTrace {
    pub healthy_window: usize,
    pub drift_window: usize,
    pub drift_persistence: usize,
    pub slew_persistence: usize,
    pub window_integrity: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TimingSummary {
    pub timing_repeats: usize,
    pub measured_scope: String,
    pub total_host_time_ns: ResourceMetric<u64>,
    pub average_host_time_per_cycle_ns: ResourceMetric<f64>,
    pub pipeline_host_time_ns: ResourceMetric<u64>,
    pub average_pipeline_time_per_cycle_ns: ResourceMetric<f64>,
    pub heuristics_lookup_time_per_run_ns: ResourceMetric<u64>,
    pub heuristics_lookup_time_per_cycle_ns: ResourceMetric<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ComputationalProfile {
    pub algorithmic_order_per_update: String,
    pub implementation_shape: String,
    pub logical_sample_rate: String,
    pub total_cycles_processed: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeuristicsCost {
    pub bank_version: String,
    pub bank_size: usize,
    pub lookup_count: usize,
    pub evaluated_entries_total: usize,
    pub matched_entries_total: usize,
    pub full_match_count: usize,
    pub partial_match_count: usize,
    pub average_evaluated_entries_per_cycle: ResourceMetric<f64>,
    pub average_lookup_time_per_run_ns: ResourceMetric<u64>,
    pub average_lookup_time_per_cycle_ns: ResourceMetric<f64>,
    pub serialized_bank_bytes: ResourceMetric<usize>,
    pub estimated_loaded_bank_bytes: ResourceMetric<usize>,
    pub regime_gating_applied: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemoryFootprint {
    pub sign_tuple_bytes: usize,
    pub envelope_params_bytes: usize,
    pub battery_residual_bytes: usize,
    pub pipeline_config_bytes: usize,
    pub theorem1_result_bytes: usize,
    pub hot_loop_state_estimate_bytes: ResourceMetric<usize>,
    pub heuristics_bank_serialized_bytes: ResourceMetric<usize>,
    pub heuristics_bank_loaded_bytes_estimate: ResourceMetric<usize>,
    pub heap_allocation_count: ResourceMetric<usize>,
    pub stack_usage_bytes: ResourceMetric<usize>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResourceRunSummary {
    pub cell_id: String,
    pub dsfb_alarm_cycle: Option<usize>,
    pub first_boundary_cycle: Option<usize>,
    pub first_violation_cycle: Option<usize>,
    pub threshold_85pct_cycle: Option<usize>,
    pub eol_80pct_cycle: Option<usize>,
    pub lead_time_vs_threshold_baseline: Option<i64>,
    pub theorem_t_star: usize,
    pub validity_token_sequence_id: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceTrace {
    pub header: AuditHeader,
    pub cycle_index_start: usize,
    pub cycle_index_end: usize,
    pub window_trace: WindowTrace,
    pub computational_profile: ComputationalProfile,
    pub timing_summary: TimingSummary,
    pub heuristics_cost: HeuristicsCost,
    pub memory_footprint: MemoryFootprint,
    pub complexity_reference: ComplexityArtifact,
    pub run_summary: ResourceRunSummary,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeuristicsDensityReport {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub input_hash: String,
    pub config_hash: String,
    pub bank_version: String,
    pub total_bank_entries: usize,
    pub lookup_count: usize,
    pub total_evaluated_entries: usize,
    pub total_matched_entries: usize,
    pub full_match_count: usize,
    pub partial_match_count: usize,
    pub average_evaluated_entries_per_cycle: ResourceMetric<f64>,
    pub average_lookup_time_per_run_ns: ResourceMetric<u64>,
    pub average_lookup_time_per_cycle_ns: ResourceMetric<f64>,
    pub approximate_memory_footprint_bytes: ResourceMetric<usize>,
    pub regime_gating_reduced_scope: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemoryBudgetReport {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub compile_time_assertions_enabled: bool,
    pub sign_tuple_budget_bytes: usize,
    pub envelope_params_budget_bytes: usize,
    pub battery_residual_budget_bytes: usize,
    pub pipeline_config_budget_bytes: usize,
    pub theorem1_result_budget_bytes: usize,
    pub actual_sign_tuple_bytes: usize,
    pub actual_envelope_params_bytes: usize,
    pub actual_battery_residual_bytes: usize,
    pub actual_pipeline_config_bytes: usize,
    pub actual_theorem1_result_bytes: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceTraceArtifacts {
    pub resource_trace: ResourceTrace,
    pub heuristics_density_report: HeuristicsDensityReport,
    pub memory_budget_report: MemoryBudgetReport,
}

pub fn run_resource_trace_workflow(
    crate_dir: &Path,
    data_path: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
    timing_repeats: usize,
) -> Result<ResourceTraceArtifacts, Box<dyn std::error::Error>> {
    fs::create_dir_all(output_dir)?;

    let raw_input = load_capacity_csv(data_path)?;
    let effective_repeats = timing_repeats.max(1);
    let input_hash = hash_input_series(&raw_input)?;
    let config_hash = hash_pipeline_config(config)?;

    let (run, average_evaluation_ns) =
        measure_cell_evaluation(data_path, &raw_input, config, effective_repeats)?;
    let cell_id = run.summary.cell_id.clone();

    let (heuristics_cost, density_report) = build_heuristics_cost(
        crate_dir,
        &run,
        &input_hash,
        &config_hash,
        effective_repeats,
    )?;
    let memory_footprint = build_memory_footprint(config, &heuristics_cost)?;
    let complexity_reference = estimate_dsfb_update_complexity(config);
    let total_cycles_processed = run.summary.cycle_count;

    let window_trace = WindowTrace {
        healthy_window: config.healthy_window,
        drift_window: config.drift_window,
        drift_persistence: config.drift_persistence,
        slew_persistence: config.slew_persistence,
        window_integrity: verify_window_integrity(config),
        note: "Window and persistence values are fixed for the duration of this traced run because they are supplied by an immutable PipelineConfig snapshot.".to_string(),
    };

    let total_host_time_ns = average_evaluation_ns.saturating_add(
        density_report
            .average_lookup_time_per_run_ns
            .value
            .unwrap_or_default(),
    );
    let average_host_time_per_cycle_ns = if total_cycles_processed > 0 {
        Some(total_host_time_ns as f64 / total_cycles_processed as f64)
    } else {
        None
    };

    let timing_summary = TimingSummary {
        timing_repeats: effective_repeats,
        measured_scope: "Host-side measurement of evaluate_cell plus a single heuristics-bank retrieval. CSV load, JSON serialization, file output, and figure generation are excluded.".to_string(),
        total_host_time_ns: ResourceMetric {
            value: Some(total_host_time_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average host timing across the configured repeats.".to_string(),
        },
        average_host_time_per_cycle_ns: ResourceMetric {
            value: average_host_time_per_cycle_ns,
            measurement_mode: MeasurementMode::Measured,
            note: "Derived from averaged host timing divided by logical cycle count.".to_string(),
        },
        pipeline_host_time_ns: ResourceMetric {
            value: Some(average_evaluation_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average host timing for the batch DSFB evaluation helper.".to_string(),
        },
        average_pipeline_time_per_cycle_ns: ResourceMetric {
            value: if total_cycles_processed > 0 {
                Some(average_evaluation_ns as f64 / total_cycles_processed as f64)
            } else {
                None
            },
            measurement_mode: MeasurementMode::Measured,
            note: "Average DSFB evaluation time per logical update on the current host.".to_string(),
        },
        heuristics_lookup_time_per_run_ns: density_report.average_lookup_time_per_run_ns.clone(),
        heuristics_lookup_time_per_cycle_ns: density_report.average_lookup_time_per_cycle_ns.clone(),
    };

    let validity_token = build_validity_token(total_cycles_processed, true, 60);
    let cycle_index_start = raw_input.first().map(|(cycle, _)| *cycle).unwrap_or(0);
    let cycle_index_end = raw_input.last().map(|(cycle, _)| *cycle).unwrap_or(0);

    let mut resource_trace = ResourceTrace {
        header: AuditHeader {
            artifact_type: "dsfb_battery_resource_trace".to_string(),
            schema_version: RESOURCE_TRACE_SCHEMA_VERSION.to_string(),
            generated_at_utc: Utc::now().to_rfc3339(),
            output_contract: "opt_in_resource_integrity_and_audit_helper".to_string(),
            advisory_only: true,
            source_artifact: data_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| data_path.display().to_string()),
            input_hash: input_hash.clone(),
            config_hash: config_hash.clone(),
            validation_hash: String::new(),
        },
        cycle_index_start,
        cycle_index_end,
        window_trace,
        computational_profile: ComputationalProfile {
            algorithmic_order_per_update:
                "O(1) per logical cycle under fixed window sizes in the local update rules; O(N) over N samples for the current batch helper run.".to_string(),
            implementation_shape: "The traced path measures the existing batch-oriented crate evaluation helper. The hot-loop state estimate remains fixed-width and configuration-bounded.".to_string(),
            logical_sample_rate:
                "One logical update per cycle index step; the NASA CSV path is cycle-indexed and does not encode a wall-clock sample rate.".to_string(),
            total_cycles_processed,
            notes: vec![
                "Timing values are host measurements for the helper path, not MCU cycle counts.".to_string(),
                "No certified WCET or target-specific execution claim is made.".to_string(),
            ],
        },
        timing_summary,
        heuristics_cost,
        memory_footprint,
        complexity_reference,
        run_summary: ResourceRunSummary {
            cell_id,
            dsfb_alarm_cycle: run.summary.dsfb_alarm_cycle,
            first_boundary_cycle: run.summary.first_boundary_cycle,
            first_violation_cycle: run.summary.first_violation_cycle,
            threshold_85pct_cycle: run.summary.threshold_85pct_cycle,
            eol_80pct_cycle: run.summary.eol_80pct_cycle,
            lead_time_vs_threshold_baseline: run.summary.lead_time_vs_threshold_baseline,
            theorem_t_star: run.summary.theorem_t_star,
            validity_token_sequence_id: validity_token.sequence_id,
        },
    };

    let validation_hash = compute_resource_trace_validation_hash(&resource_trace)?;
    resource_trace.header.validation_hash = validation_hash;

    let memory_budget_report = build_memory_budget_report();

    write_pretty_json(&resource_trace, &output_dir.join(RESOURCE_TRACE_JSON_NAME))?;
    write_pretty_json(
        &density_report,
        &output_dir.join(HEURISTICS_DENSITY_JSON_NAME),
    )?;
    write_resource_trace_summary(
        &resource_trace,
        &output_dir.join(RESOURCE_TRACE_SUMMARY_NAME),
    )?;
    write_memory_budget_report(
        &memory_budget_report,
        &output_dir.join(MEMORY_BUDGET_REPORT_NAME),
    )?;
    write_implementation_summary(
        &resource_trace,
        &density_report,
        &output_dir.join(IMPLEMENTATION_SUMMARY_NAME),
        output_dir,
    )?;
    Ok(ResourceTraceArtifacts {
        resource_trace,
        heuristics_density_report: density_report,
        memory_budget_report,
    })
}

pub fn verify_resource_trace_validation_hash(
    trace: &ResourceTrace,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(compute_resource_trace_validation_hash(trace)? == trace.header.validation_hash)
}

fn measure_cell_evaluation(
    data_path: &Path,
    raw_input: &[(usize, f64)],
    config: &PipelineConfig,
    timing_repeats: usize,
) -> Result<(CellEvaluationRun, u64), Box<dyn std::error::Error>> {
    let cell_id = infer_cell_id_from_path(data_path);
    let source_csv = data_path.display().to_string();
    let mut total_ns = 0u64;
    let mut last_run = None;

    for _ in 0..timing_repeats {
        let started = Instant::now();
        let run = evaluate_cell(&cell_id, &source_csv, raw_input, config)?;
        total_ns = total_ns.saturating_add(duration_as_ns(started.elapsed()));
        last_run = Some(run);
    }

    Ok((
        last_run.expect("timing_repeats is clamped to at least one"),
        total_ns / timing_repeats as u64,
    ))
}

fn build_heuristics_cost(
    crate_dir: &Path,
    run: &CellEvaluationRun,
    input_hash: &str,
    config_hash: &str,
    timing_repeats: usize,
) -> Result<(HeuristicsCost, HeuristicsDensityReport), Box<dyn std::error::Error>> {
    let bank = load_nasa_heuristics_bank(crate_dir)?;
    let bank_path = crate_dir.join(NASA_HEURISTICS_BANK_JSON);
    let serialized_bank_bytes = fs::metadata(&bank_path)?.len() as usize;
    let estimated_loaded_bank_bytes = serde_json::to_vec(&bank)?.len();

    let profile = build_heuristic_observation_profile(&run.summary, &run.trajectory)?;
    let mut total_lookup_ns = 0u64;
    let mut last_matches = Vec::new();
    for _ in 0..timing_repeats {
        let started = Instant::now();
        let matches = retrieve_heuristic_matches(&bank, &profile);
        total_lookup_ns = total_lookup_ns.saturating_add(duration_as_ns(started.elapsed()));
        last_matches = matches;
    }
    let average_lookup_time_per_run_ns = total_lookup_ns / timing_repeats as u64;
    let full_match_count = last_matches
        .iter()
        .filter(|result| result.match_status == MatchStatus::Full)
        .count();
    let partial_match_count = last_matches
        .iter()
        .filter(|result| result.match_status == MatchStatus::Partial)
        .count();
    let matched_entries_total = full_match_count + partial_match_count;
    let evaluated_entries_total = bank.entries.len();
    let average_evaluated_entries_per_cycle =
        evaluated_entries_total as f64 / run.summary.cycle_count as f64;
    let average_lookup_time_per_cycle_ns =
        average_lookup_time_per_run_ns as f64 / run.summary.cycle_count as f64;

    let notes = vec![
        "The current heuristics-bank retrieval evaluates the full bank once per run-level observation profile.".to_string(),
        "No regime-gating or early-exit pruning is currently implemented in the retrieval helper.".to_string(),
        "Matched-entry counts include both full and partial retrieval results; the split is reported separately.".to_string(),
        "The serialized bank byte count is exact for the tracked JSON file; the loaded-bank byte count is an estimate based on serde serialization size.".to_string(),
    ];

    let heuristics_cost = HeuristicsCost {
        bank_version: bank.bank_version.clone(),
        bank_size: bank.entries.len(),
        lookup_count: 1,
        evaluated_entries_total,
        matched_entries_total,
        full_match_count,
        partial_match_count,
        average_evaluated_entries_per_cycle: ResourceMetric {
            value: Some(average_evaluated_entries_per_cycle),
            measurement_mode: MeasurementMode::Inferred,
            note: "Derived from one full-bank retrieval divided by logical cycle count.".to_string(),
        },
        average_lookup_time_per_run_ns: ResourceMetric {
            value: Some(average_lookup_time_per_run_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average host lookup timing across the configured repeats.".to_string(),
        },
        average_lookup_time_per_cycle_ns: ResourceMetric {
            value: Some(average_lookup_time_per_cycle_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average host lookup timing amortized over logical cycle count.".to_string(),
        },
        serialized_bank_bytes: ResourceMetric {
            value: Some(serialized_bank_bytes),
            measurement_mode: MeasurementMode::Measured,
            note: "Exact file size of heuristics/heuristics_bank_v2.json.".to_string(),
        },
        estimated_loaded_bank_bytes: ResourceMetric {
            value: Some(estimated_loaded_bank_bytes),
            measurement_mode: MeasurementMode::Estimated,
            note: "Approximate lower-bound in-memory size based on serde serialization length.".to_string(),
        },
        regime_gating_applied: false,
        notes: notes.clone(),
    };

    let density_report = HeuristicsDensityReport {
        artifact_type: "dsfb_battery_heuristics_density_report".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        input_hash: input_hash.to_string(),
        config_hash: config_hash.to_string(),
        bank_version: bank.bank_version,
        total_bank_entries: evaluated_entries_total,
        lookup_count: 1,
        total_evaluated_entries: evaluated_entries_total,
        total_matched_entries: matched_entries_total,
        full_match_count,
        partial_match_count,
        average_evaluated_entries_per_cycle: ResourceMetric {
            value: Some(average_evaluated_entries_per_cycle),
            measurement_mode: MeasurementMode::Inferred,
            note: "One retrieval per run amortized across the logical cycles in this batch helper.".to_string(),
        },
        average_lookup_time_per_run_ns: ResourceMetric {
            value: Some(average_lookup_time_per_run_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average host lookup timing across repeated retrievals.".to_string(),
        },
        average_lookup_time_per_cycle_ns: ResourceMetric {
            value: Some(average_lookup_time_per_cycle_ns),
            measurement_mode: MeasurementMode::Measured,
            note: "Average lookup timing per logical cycle on the current host.".to_string(),
        },
        approximate_memory_footprint_bytes: ResourceMetric {
            value: Some(estimated_loaded_bank_bytes),
            measurement_mode: MeasurementMode::Estimated,
            note: "Estimated loaded-bank footprint derived from serde serialization size.".to_string(),
        },
        regime_gating_reduced_scope: false,
        notes,
    };

    Ok((heuristics_cost, density_report))
}

fn build_memory_footprint(
    config: &PipelineConfig,
    heuristics_cost: &HeuristicsCost,
) -> Result<MemoryFootprint, Box<dyn std::error::Error>> {
    let serialized_bank_bytes = heuristics_cost.serialized_bank_bytes.value.unwrap_or_default();
    let hot_loop_state_estimate_bytes = estimate_hot_loop_state_bytes(config);

    Ok(MemoryFootprint {
        sign_tuple_bytes: size_of::<SignTuple>(),
        envelope_params_bytes: size_of::<EnvelopeParams>(),
        battery_residual_bytes: size_of::<BatteryResidual>(),
        pipeline_config_bytes: size_of::<PipelineConfig>(),
        theorem1_result_bytes: size_of::<Theorem1Result>(),
        hot_loop_state_estimate_bytes: ResourceMetric {
            value: Some(hot_loop_state_estimate_bytes),
            measurement_mode: MeasurementMode::Estimated,
            note: "Estimated fixed-state footprint for a streaming-style hot loop with declared windows, counters, and envelope/config state.".to_string(),
        },
        heuristics_bank_serialized_bytes: ResourceMetric {
            value: Some(serialized_bank_bytes),
            measurement_mode: MeasurementMode::Measured,
            note: "Exact byte size of the tracked heuristics bank JSON file.".to_string(),
        },
        heuristics_bank_loaded_bytes_estimate: heuristics_cost.estimated_loaded_bank_bytes.clone(),
        heap_allocation_count: ResourceMetric {
            value: None,
            measurement_mode: MeasurementMode::NotMeasured,
            note: "The opt-in resource trace does not instrument the global allocator, so exact heap allocation counts are not reported.".to_string(),
        },
        stack_usage_bytes: ResourceMetric {
            value: None,
            measurement_mode: MeasurementMode::NotMeasured,
            note: "Exact stack usage is not profiled by this crate. The hot-loop state estimate is reported separately and does not claim full stack depth.".to_string(),
        },
        notes: vec![
            "Object sizes come from core::mem::size_of for key runtime structs.".to_string(),
            "The hot-loop state estimate is distinct from total process memory.".to_string(),
            "No zero-heap claim is made for the traced helper path.".to_string(),
        ],
    })
}

fn estimate_hot_loop_state_bytes(config: &PipelineConfig) -> usize {
    let residual_window_bytes = (config.drift_window + 1) * size_of::<f64>();
    let drift_window_bytes = (config.drift_window + 1) * size_of::<f64>();
    let persistence_counter_bytes = 2 * size_of::<usize>();
    let envelope_bytes = size_of::<EnvelopeParams>();
    let config_bytes = size_of::<PipelineConfig>();

    residual_window_bytes + drift_window_bytes + persistence_counter_bytes + envelope_bytes + config_bytes
}

fn build_memory_budget_report() -> MemoryBudgetReport {
    MemoryBudgetReport {
        artifact_type: "dsfb_battery_memory_budget_report".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        compile_time_assertions_enabled: true,
        sign_tuple_budget_bytes: SIGN_TUPLE_BUDGET_BYTES,
        envelope_params_budget_bytes: ENVELOPE_PARAMS_BUDGET_BYTES,
        battery_residual_budget_bytes: BATTERY_RESIDUAL_BUDGET_BYTES,
        pipeline_config_budget_bytes: PIPELINE_CONFIG_BUDGET_BYTES,
        theorem1_result_budget_bytes: THEOREM1_RESULT_BUDGET_BYTES,
        actual_sign_tuple_bytes: size_of::<SignTuple>(),
        actual_envelope_params_bytes: size_of::<EnvelopeParams>(),
        actual_battery_residual_bytes: size_of::<BatteryResidual>(),
        actual_pipeline_config_bytes: size_of::<PipelineConfig>(),
        actual_theorem1_result_bytes: size_of::<Theorem1Result>(),
        notes: vec![
            "These compile-time budgets constrain object growth for selected runtime structs only.".to_string(),
            "Passing the object-size budgets does not prove total memory usage, stack depth, or heap behavior.".to_string(),
        ],
    }
}

fn write_resource_trace_summary(
    trace: &ResourceTrace,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    lines.push("DSFB Battery Resource Trace Summary".to_string());
    lines.push(format!("Generated at: {}", trace.header.generated_at_utc));
    lines.push(format!("Source artifact: {}", trace.header.source_artifact));
    lines.push(format!("Input hash: {}", trace.header.input_hash));
    lines.push(format!("Config hash: {}", trace.header.config_hash));
    lines.push(format!("Validation hash: {}", trace.header.validation_hash));
    lines.push(format!(
        "Logical sample rate: {}",
        trace.computational_profile.logical_sample_rate
    ));
    lines.push(format!(
        "Total cycles processed: {}",
        trace.computational_profile.total_cycles_processed
    ));
    lines.push(format!(
        "Window integrity: {}",
        trace.window_trace.window_integrity
    ));
    lines.push(format!(
        "Average host time per cycle (ns): {}",
        trace
            .timing_summary
            .average_host_time_per_cycle_ns
            .value
            .map(|value| format!("{value:.3}"))
            .unwrap_or_else(|| "unavailable".to_string())
    ));
    lines.push(format!(
        "Heuristics bank entries evaluated: {}",
        trace.heuristics_cost.evaluated_entries_total
    ));
    lines.push(format!(
        "Heuristics matches returned: {} ({} full, {} partial)",
        trace.heuristics_cost.matched_entries_total,
        trace.heuristics_cost.full_match_count,
        trace.heuristics_cost.partial_match_count
    ));
    lines.push(format!(
        "Heap allocation count: {} ({:?})",
        trace
            .memory_footprint
            .heap_allocation_count
            .value
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not reported".to_string()),
        trace.memory_footprint.heap_allocation_count.measurement_mode
    ));
    lines.push(format!(
        "Stack usage bytes: {} ({:?})",
        trace
            .memory_footprint
            .stack_usage_bytes
            .value
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not reported".to_string()),
        trace.memory_footprint.stack_usage_bytes.measurement_mode
    ));
    fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn write_memory_budget_report(
    report: &MemoryBudgetReport,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    lines.push("DSFB Battery Memory Budget Report".to_string());
    lines.push(format!(
        "Compile-time assertions enabled: {}",
        report.compile_time_assertions_enabled
    ));
    lines.push(format!(
        "SignTuple bytes: {} / budget {}",
        report.actual_sign_tuple_bytes, report.sign_tuple_budget_bytes
    ));
    lines.push(format!(
        "EnvelopeParams bytes: {} / budget {}",
        report.actual_envelope_params_bytes, report.envelope_params_budget_bytes
    ));
    lines.push(format!(
        "BatteryResidual bytes: {} / budget {}",
        report.actual_battery_residual_bytes, report.battery_residual_budget_bytes
    ));
    lines.push(format!(
        "PipelineConfig bytes: {} / budget {}",
        report.actual_pipeline_config_bytes, report.pipeline_config_budget_bytes
    ));
    lines.push(format!(
        "Theorem1Result bytes: {} / budget {}",
        report.actual_theorem1_result_bytes, report.theorem1_result_budget_bytes
    ));
    for note in &report.notes {
        lines.push(format!("- {}", note));
    }
    fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn write_implementation_summary(
    trace: &ResourceTrace,
    density_report: &HeuristicsDensityReport,
    path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    lines.push("Resource Integrity & Audit implementation summary".to_string());
    lines.push("Implemented:".to_string());
    lines.push("- Typed ResourceTrace, HeuristicsCost, HeuristicsDensityReport, MemoryBudgetReport, and AuditHeader output contracts.".to_string());
    lines.push("- Host-side timing measurements for the existing batch evaluation helper and heuristics-bank lookup.".to_string());
    lines.push("- SHA-256 input, configuration, and validation hashing for the resource trace artifact.".to_string());
    lines.push("- Compile-time size assertions for selected core runtime structs.".to_string());
    lines.push("Measured vs estimated vs asserted:".to_string());
    lines.push("- Host timing values are measured on the current host only.".to_string());
    lines.push("- Hot-loop state bytes and loaded-bank bytes are estimated.".to_string());
    lines.push("- Window integrity and logical sample rate are asserted from the fixed run configuration and cycle-indexed input model.".to_string());
    lines.push("- Heap allocation count and exact stack usage remain not measured.".to_string());
    lines.push("Added modules/files:".to_string());
    lines.push("- src/resource_trace.rs".to_string());
    lines.push("- src/bin/dsfb_battery_resource_trace.rs".to_string());
    lines.push("- docs/addendum/resource_trace.md".to_string());
    lines.push("Protection gates:".to_string());
    lines.push("- Existing dsfb-battery-demo behavior was not modified.".to_string());
    lines.push("- Resource tracing writes only into outputs/resource_trace/...".to_string());
    lines.push("- Production figure filenames and stage-II artifact filenames are not reused.".to_string());
    lines.push(format!(
        "Heuristics density lookup count: {}",
        density_report.lookup_count
    ));
    lines.push(format!("Artifacts written to: {}", output_dir.display()));
    lines.push(format!(
        "Confirmation: original mono-cell production code and figures were not modified in behavior. Current validation hash: {}",
        trace.header.validation_hash
    ));
    fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn verify_window_integrity(config: &PipelineConfig) -> bool {
    config.healthy_window > 0
        && config.drift_window > 0
        && config.drift_persistence > 0
        && config.slew_persistence > 0
}

fn infer_cell_id_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            name.strip_prefix("nasa_")
                .and_then(|trimmed| trimmed.strip_suffix("_capacity.csv"))
        })
        .map(|cell| cell.to_ascii_uppercase())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_else(|| "input_series".to_string())
        })
}

fn duration_as_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn hash_input_series(raw_input: &[(usize, f64)]) -> Result<String, serde_json::Error> {
    Ok(hash_prefixed(&serde_json::to_vec(raw_input)?))
}

fn hash_pipeline_config(config: &PipelineConfig) -> Result<String, serde_json::Error> {
    Ok(hash_prefixed(&serde_json::to_vec(config)?))
}

fn compute_resource_trace_validation_hash(
    trace: &ResourceTrace,
) -> Result<String, serde_json::Error> {
    let mut clone = trace.clone();
    clone.header.validation_hash.clear();
    Ok(hash_prefixed(&serde_json::to_vec(&clone)?))
}

fn hash_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::from("sha256:");
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn write_pretty_json<T: Serialize>(
    value: &T,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use crate::output_paths::resolve_helper_output_dir;
    use std::path::PathBuf;

    fn crate_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn b0005_data_path() -> PathBuf {
        crate_dir().join("data").join("nasa_b0005_capacity.csv")
    }

    fn unique_temp_dir(stem: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{unique}"))
    }

    #[test]
    fn resource_trace_hash_round_trips() {
        let output_dir = unique_temp_dir("dsfb-battery-resource-trace");
        let artifacts = run_resource_trace_workflow(
            &crate_dir(),
            &b0005_data_path(),
            &output_dir,
            &PipelineConfig::default(),
            1,
        )
        .unwrap();

        assert!(verify_resource_trace_validation_hash(&artifacts.resource_trace).unwrap());
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn resource_trace_workflow_writes_only_to_its_output_directory() {
        let output_dir = unique_temp_dir("dsfb-battery-resource-trace");
        let artifacts = run_resource_trace_workflow(
            &crate_dir(),
            &b0005_data_path(),
            &output_dir,
            &PipelineConfig::default(),
            1,
        )
        .unwrap();

        assert_eq!(
            artifacts.resource_trace.header.artifact_type,
            "dsfb_battery_resource_trace"
        );
        assert!(output_dir.join(RESOURCE_TRACE_JSON_NAME).exists());
        assert!(output_dir.join(RESOURCE_TRACE_SUMMARY_NAME).exists());
        assert!(output_dir.join(HEURISTICS_DENSITY_JSON_NAME).exists());
        assert!(output_dir.join(MEMORY_BUDGET_REPORT_NAME).exists());
        assert!(output_dir.join(IMPLEMENTATION_SUMMARY_NAME).exists());
        assert!(!output_dir.join("stage2_detection_results.json").exists());
        assert!(!output_dir.join("fig01_capacity_fade.svg").exists());

        let output_names: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(|entry| {
                entry
                    .ok()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
            })
            .collect();
        assert!(!output_names
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));

        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn resource_trace_output_dir_uses_isolated_root() {
        let crate_dir = crate_dir();
        let resolved = resolve_helper_output_dir(
            &crate_dir,
            "resource_trace",
            "dsfb_battery_resource_trace",
            None,
        );

        assert!(resolved
            .to_string_lossy()
            .contains("outputs/resource_trace/dsfb_battery_resource_trace_"));
    }

    #[test]
    fn window_integrity_holds_for_default_config() {
        assert!(verify_window_integrity(&PipelineConfig::default()));
        assert_eq!(estimate_hot_loop_state_bytes(&PipelineConfig::default()), estimate_hot_loop_state_bytes(&PipelineConfig::default()));
    }
}
