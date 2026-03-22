#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueHint};
use serde::Serialize;

use dsfb_semiotics_engine::engine::residual_layer::extract_residuals;
use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingSettings};
use dsfb_semiotics_engine::live::{numeric_mode_label, to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::sim::generators::synthesize;
use dsfb_semiotics_engine::sim::scenarios::{all_scenarios, ScenarioDefinition};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate bounded live-path deployment evidence for the current numeric backend"
)]
struct Args {
    #[arg(long, value_hint = ValueHint::FilePath)]
    output_json: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
struct ScenarioEvidence {
    scenario_id: String,
    channel_count: usize,
    steps_processed: usize,
    final_syntax_label: String,
    final_grammar_state: String,
    final_grammar_reason_code: String,
    final_semantic_disposition: String,
    final_semantic_disposition_code: u8,
    selected_heuristic_ids: Vec<String>,
    min_trust_scalar: f64,
    final_trust_scalar: f64,
    max_residual_norm: f64,
    note: String,
}

#[derive(Clone, Debug, Serialize)]
struct FixedPointDeploymentEvidence {
    schema_version: String,
    numeric_mode: String,
    supported_scope: Vec<String>,
    unsupported_scope: Vec<String>,
    precision_note: String,
    scenario_summaries: Vec<ScenarioEvidence>,
    note: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output_json = args.output_json.unwrap_or_else(default_output_path);
    let report = build_report()?;
    if let Some(parent) = output_json.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_json, serde_json::to_vec_pretty(&report)?)?;
    println!("fixed_point_evidence={}", output_json.display());
    Ok(())
}

fn default_output_path() -> PathBuf {
    PathBuf::from(format!(
        "docs/generated/fixed_point_deployment_evidence_{}.json",
        numeric_mode_label().replace('-', "_")
    ))
}

fn build_report() -> Result<FixedPointDeploymentEvidence> {
    Ok(FixedPointDeploymentEvidence {
        schema_version: "dsfb-semiotics-fixed-point-deployment-evidence/v1".to_string(),
        numeric_mode: numeric_mode_label().to_string(),
        supported_scope: vec![
            "bounded live OnlineStructuralEngine path".to_string(),
            "scalar push_residual_sample and batch push_residual_sample_batch".to_string(),
            "syntax / grammar / semantic status emitted by the live engine".to_string(),
        ],
        unsupported_scope: vec![
            "full offline artifact pipeline".to_string(),
            "paper/report generation under numeric-fixed".to_string(),
            "whole-crate no_std or hardware qualification claims".to_string(),
        ],
        precision_note: "Equivalence is assessed conservatively at the live-status level: syntax label, grammar state/reason, semantic disposition, selected heuristic IDs, and trust-scalar drift within a small numeric tolerance. The current evidence is intentionally scoped to the bounded live path.".to_string(),
        scenario_summaries: vec![
            run_live_scenario("imu_thermal_drift_gps_denied")?,
            run_live_scenario("regime_switch")?,
            run_live_scenario("abrupt_event")?,
        ],
        note: "This report is generated separately for each numeric backend. Deployment evidence is limited to the tested bounded live path and does not broaden into full-crate embedded readiness claims.".to_string(),
    })
}

fn constrained_live_settings(step_count: usize) -> EngineSettings {
    let mut settings = EngineSettings::default();
    settings.online.history_buffer_capacity = step_count.max(8);
    settings.online.offline_history_enabled = false;
    settings.smoothing = SmoothingSettings::safety_first();
    settings
}

fn run_live_scenario(scenario_id: &str) -> Result<ScenarioEvidence> {
    let definition = scenario_definition(scenario_id)?;
    let step_count = 96;
    let synthesis = synthesize(&definition, step_count, 1.0, 17);
    let residual = extract_residuals(&synthesis.observed, &synthesis.predicted, scenario_id);
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        scenario_id.to_string(),
        definition.channels.clone(),
        1.0,
        definition.envelope_spec.clone(),
        constrained_live_settings(step_count),
    )?;
    let mut min_trust_scalar = 1.0_f64;
    let mut max_residual_norm = 0.0_f64;
    let mut final_status = None;
    for sample in &residual.samples {
        let values = sample
            .values
            .iter()
            .copied()
            .map(to_real)
            .collect::<Vec<_>>();
        let status = engine.push_residual_sample(sample.time, &values)?;
        min_trust_scalar = min_trust_scalar.min(status.trust_scalar);
        max_residual_norm = max_residual_norm.max(status.residual_norm);
        final_status = Some(status);
    }
    let final_status =
        final_status.with_context(|| format!("missing live status for {scenario_id}"))?;
    Ok(ScenarioEvidence {
        scenario_id: scenario_id.to_string(),
        channel_count: definition.channels.len(),
        steps_processed: residual.samples.len(),
        final_syntax_label: final_status.syntax_label,
        final_grammar_state: format!("{:?}", final_status.grammar_state),
        final_grammar_reason_code: format!("{:?}", final_status.grammar_reason_code),
        final_semantic_disposition: final_status.semantic_disposition,
        final_semantic_disposition_code: final_status.semantic_disposition_code,
        selected_heuristic_ids: final_status.selected_heuristic_ids,
        min_trust_scalar,
        final_trust_scalar: final_status.trust_scalar,
        max_residual_norm,
        note: "Generated from the bounded live path using the scenario residual sequence replayed sample-by-sample under the current numeric backend.".to_string(),
    })
}

fn scenario_definition(scenario_id: &str) -> Result<ScenarioDefinition> {
    all_scenarios()
        .into_iter()
        .find(|scenario| scenario.record.id == scenario_id)
        .ok_or_else(|| anyhow!("unknown scenario `{scenario_id}`"))
}
