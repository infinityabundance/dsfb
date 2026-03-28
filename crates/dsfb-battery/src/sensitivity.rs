// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing sensitivity analysis helper.

use crate::detection::{
    assign_reason_code, build_dsfb_detection, build_threshold_detection, evaluate_grammar_state,
    verify_theorem1,
};
use crate::engineer_plots::generate_sensitivity_overview_figure;
use crate::export::Stage2Results;
use crate::integration::{
    build_validity_token, compute_tactical_margin_summary, TacticalMarginSummary, ValidityToken,
};
use crate::math::{compute_all_drifts, compute_all_residuals, compute_all_slews, compute_envelope};
use crate::types::{BatteryResidual, EnvelopeParams, GrammarState, PipelineConfig, SignTuple};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

const SENSITIVITY_JSON_NAME: &str = "sensitivity_summary.json";
const SENSITIVITY_CSV_NAME: &str = "sensitivity_summary.csv";
const SENSITIVITY_FIGURE_NAME: &str = "sensitivity_overview.svg";
const SENSITIVITY_SUMMARY_NAME: &str = "implementation_summary.txt";

#[derive(Debug, Clone, Serialize)]
pub struct SensitivityScenarioResult {
    pub scenario_id: String,
    pub parameter_name: String,
    pub parameter_value: String,
    pub healthy_window: usize,
    pub drift_window: usize,
    pub drift_persistence: usize,
    pub slew_persistence: usize,
    pub envelope_sigma_multiplier: f64,
    pub first_boundary_cycle: Option<usize>,
    pub first_violation_cycle: Option<usize>,
    pub dsfb_alarm_cycle: Option<usize>,
    pub threshold_85pct_cycle: Option<usize>,
    pub lead_time_vs_threshold_baseline: Option<i64>,
    pub theorem_t_star: usize,
    pub tactical_margin: TacticalMarginSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensitivityArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub cell_id: String,
    pub source_csv: String,
    pub validity_token: Option<ValidityToken>,
    pub scenarios: Vec<SensitivityScenarioResult>,
    pub generated_figures: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run_sensitivity_workflow(
    cell_id: &str,
    source_csv: &str,
    raw_data: &[(usize, f64)],
    base_config: &PipelineConfig,
    drift_window_values: &[usize],
    drift_persistence_values: &[usize],
    slew_persistence_values: &[usize],
    sigma_multipliers: &[f64],
    tactical_margin_fraction: f64,
    output_dir: &Path,
) -> Result<SensitivityArtifact, Box<dyn std::error::Error>> {
    let capacities: Vec<f64> = raw_data.iter().map(|(_, value)| *value).collect();
    let mut scenarios = Vec::new();

    let defaults = vec![(
        "baseline".to_string(),
        "config".to_string(),
        base_config.clone(),
        3.0,
    )];

    let mut sweep_configs = Vec::new();
    for value in drift_window_values {
        let mut config = base_config.clone();
        config.drift_window = *value;
        sweep_configs.push((
            format!("drift_window_{value}"),
            "drift_window".to_string(),
            config,
            3.0,
        ));
    }
    for value in drift_persistence_values {
        let mut config = base_config.clone();
        config.drift_persistence = *value;
        sweep_configs.push((
            format!("drift_persistence_{value}"),
            "drift_persistence".to_string(),
            config,
            3.0,
        ));
    }
    for value in slew_persistence_values {
        let mut config = base_config.clone();
        config.slew_persistence = *value;
        sweep_configs.push((
            format!("slew_persistence_{value}"),
            "slew_persistence".to_string(),
            config,
            3.0,
        ));
    }
    for value in sigma_multipliers {
        sweep_configs.push((
            format!("sigma_multiplier_{value:.2}"),
            "envelope_sigma_multiplier".to_string(),
            base_config.clone(),
            *value,
        ));
    }

    for (scenario_id, parameter_name, config, sigma_multiplier) in
        defaults.into_iter().chain(sweep_configs.into_iter())
    {
        let (results, trajectory) =
            evaluate_series_with_sigma_multiplier(&capacities, &config, sigma_multiplier)?;
        let tactical_margin =
            compute_tactical_margin_summary(&capacities, &trajectory, tactical_margin_fraction);
        let parameter_value = match parameter_name.as_str() {
            "drift_window" => config.drift_window.to_string(),
            "drift_persistence" => config.drift_persistence.to_string(),
            "slew_persistence" => config.slew_persistence.to_string(),
            "envelope_sigma_multiplier" => format!("{sigma_multiplier:.2}"),
            _ => "default".to_string(),
        };
        let first_boundary_cycle = trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Boundary)
            .map(|sample| sample.cycle);
        let first_violation_cycle = trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Violation)
            .map(|sample| sample.cycle);

        scenarios.push(SensitivityScenarioResult {
            scenario_id,
            parameter_name,
            parameter_value,
            healthy_window: config.healthy_window,
            drift_window: config.drift_window,
            drift_persistence: config.drift_persistence,
            slew_persistence: config.slew_persistence,
            envelope_sigma_multiplier: sigma_multiplier,
            first_boundary_cycle,
            first_violation_cycle,
            dsfb_alarm_cycle: results.dsfb_detection.alarm_cycle,
            threshold_85pct_cycle: results.threshold_detection.alarm_cycle,
            lead_time_vs_threshold_baseline: first_boundary_cycle
                .or(first_violation_cycle)
                .zip(results.threshold_detection.alarm_cycle)
                .map(|(signal, threshold)| threshold as i64 - signal as i64),
            theorem_t_star: results.theorem1.t_star,
            tactical_margin,
        });
    }

    std::fs::create_dir_all(output_dir)?;
    let artifact = SensitivityArtifact {
        artifact_type: "dsfb_battery_sensitivity_analysis".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_sensitivity_helper".to_string(),
        cell_id: cell_id.to_string(),
        source_csv: source_csv.to_string(),
        validity_token: scenarios
            .last()
            .and_then(|scenario| scenario.dsfb_alarm_cycle)
            .map(|cycle| build_validity_token(cycle, true, 60)),
        scenarios: scenarios.clone(),
        generated_figures: vec![SENSITIVITY_FIGURE_NAME.to_string()],
        notes: vec![
            "This workflow sweeps one parameter at a time around the current production configuration.".to_string(),
            "Envelope scaling is helper-only here; the production path remains fixed at 3σ.".to_string(),
        ],
    };

    write_pretty_json(&artifact, &output_dir.join(SENSITIVITY_JSON_NAME))?;
    write_sensitivity_csv(&scenarios, &output_dir.join(SENSITIVITY_CSV_NAME))?;
    generate_sensitivity_overview_figure(&scenarios, &output_dir.join(SENSITIVITY_FIGURE_NAME))?;
    write_summary_text(&artifact, &output_dir.join(SENSITIVITY_SUMMARY_NAME))?;
    Ok(artifact)
}

fn evaluate_series_with_sigma_multiplier(
    capacities: &[f64],
    config: &PipelineConfig,
    sigma_multiplier: f64,
) -> Result<(Stage2Results, Vec<BatteryResidual>), Box<dyn std::error::Error>> {
    let healthy_data = &capacities[..config.healthy_window];
    let mut envelope = compute_envelope(healthy_data)?;
    envelope.rho = envelope.sigma * sigma_multiplier;

    let residuals = compute_all_residuals(capacities, envelope.mu);
    let drifts = compute_all_drifts(&residuals, config.drift_window);
    let slews = compute_all_slews(&drifts, config.drift_window);

    let mut trajectory = Vec::with_capacity(capacities.len());
    let mut drift_persist_count = 0usize;
    let mut slew_persist_count = 0usize;

    for (index, capacity) in capacities.iter().enumerate() {
        if drifts[index] < -config.drift_threshold {
            drift_persist_count += 1;
        } else {
            drift_persist_count = 0;
        }
        if slews[index] < -config.slew_threshold {
            slew_persist_count += 1;
        } else {
            slew_persist_count = 0;
        }

        let sign = SignTuple {
            r: residuals[index],
            d: drifts[index],
            s: slews[index],
        };
        let grammar_state = evaluate_grammar_state(
            residuals[index],
            &envelope,
            drifts[index],
            slews[index],
            drift_persist_count,
            slew_persist_count,
            config,
        );
        let reason_code = assign_reason_code(
            &sign,
            grammar_state,
            drift_persist_count,
            slew_persist_count,
            config,
        );
        trajectory.push(BatteryResidual {
            cycle: index + 1,
            capacity_ah: *capacity,
            sign,
            grammar_state,
            reason_code,
        });
    }

    let eol_capacity = config.eol_fraction * capacities[0];
    let dsfb_detection = build_dsfb_detection(&trajectory, capacities, eol_capacity);
    let threshold_detection = build_threshold_detection(capacities, 0.85, eol_capacity);
    let theorem1 = verify_theorem1(&envelope, &trajectory, config);
    let results = Stage2Results {
        data_provenance: "Engineer sensitivity helper".to_string(),
        config: config.clone(),
        envelope: EnvelopeParams {
            mu: envelope.mu,
            sigma: envelope.sigma,
            rho: envelope.rho,
        },
        dsfb_detection,
        threshold_detection,
        theorem1,
    };
    Ok((results, trajectory))
}

fn write_pretty_json<T: Serialize>(
    value: &T,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn write_sensitivity_csv(
    scenarios: &[SensitivityScenarioResult],
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "scenario_id",
        "parameter_name",
        "parameter_value",
        "drift_window",
        "drift_persistence",
        "slew_persistence",
        "envelope_sigma_multiplier",
        "first_boundary_cycle",
        "first_violation_cycle",
        "dsfb_alarm_cycle",
        "threshold_85pct_cycle",
        "lead_time_vs_threshold_baseline",
        "theorem_t_star",
        "tactical_margin_fraction",
        "tactical_margin_cycle",
        "lead_time_vs_tactical_margin_cycles",
    ])?;
    for scenario in scenarios {
        writer.write_record(vec![
            scenario.scenario_id.clone(),
            scenario.parameter_name.clone(),
            scenario.parameter_value.clone(),
            scenario.drift_window.to_string(),
            scenario.drift_persistence.to_string(),
            scenario.slew_persistence.to_string(),
            format!("{:.2}", scenario.envelope_sigma_multiplier),
            scenario
                .first_boundary_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario
                .first_violation_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario
                .dsfb_alarm_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario
                .threshold_85pct_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario
                .lead_time_vs_threshold_baseline
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario.theorem_t_star.to_string(),
            format!("{:.2}", scenario.tactical_margin.threshold_fraction),
            scenario
                .tactical_margin
                .threshold_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            scenario
                .tactical_margin
                .lead_time_vs_margin_cycles
                .map(|value| value.to_string())
                .unwrap_or_default(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_summary_text(
    artifact: &SensitivityArtifact,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    lines.push("Sensitivity workflow completion summary".to_string());
    lines.push(format!("Cell: {}", artifact.cell_id));
    lines.push(format!("Source CSV: {}", artifact.source_csv));
    lines.push(format!("Scenario count: {}", artifact.scenarios.len()));
    lines.push("Generated artifacts:".to_string());
    lines.push(format!("- {}", SENSITIVITY_JSON_NAME));
    lines.push(format!("- {}", SENSITIVITY_CSV_NAME));
    lines.push(format!("- {}", SENSITIVITY_FIGURE_NAME));
    lines.push(format!("- {}", SENSITIVITY_SUMMARY_NAME));
    for note in &artifact.notes {
        lines.push(format!("- {}", note));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", stem, unique))
    }

    #[test]
    fn sensitivity_workflow_writes_only_to_its_output_directory() {
        let raw_data = vec![
            (1, 2.000),
            (2, 1.999),
            (3, 2.001),
            (4, 1.970),
            (5, 1.940),
            (6, 1.900),
            (7, 1.860),
            (8, 1.820),
        ];
        let output_dir = unique_temp_dir("dsfb-battery-sensitivity");
        let artifact = run_sensitivity_workflow(
            "B0005",
            "synthetic.csv",
            &raw_data,
            &PipelineConfig {
                healthy_window: 3,
                drift_window: 1,
                drift_persistence: 1,
                slew_persistence: 1,
                drift_threshold: 0.002,
                slew_threshold: 0.001,
                eol_fraction: 0.80,
                boundary_fraction: 0.80,
            },
            &[1, 2],
            &[1, 2],
            &[1, 2],
            &[2.5, 3.0],
            0.88,
            &output_dir,
        )
        .unwrap();

        assert!(!artifact.scenarios.is_empty());
        assert!(output_dir.join(SENSITIVITY_JSON_NAME).exists());
        assert!(output_dir.join(SENSITIVITY_CSV_NAME).exists());
        assert!(output_dir.join(SENSITIVITY_FIGURE_NAME).exists());
        assert!(output_dir.join(SENSITIVITY_SUMMARY_NAME).exists());
        assert!(!output_dir.join("stage2_detection_results.json").exists());
        let entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));
        let _ = fs::remove_dir_all(output_dir);
    }
}
