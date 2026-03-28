// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Additive ablation workflow for engineer-facing comparisons.

use crate::engineer_plots::generate_ablation_comparison_figure;
use crate::evaluation::{evaluate_cell, CellEvaluationRun};
use crate::export::ExportError;
use crate::load_capacity_csv;
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::types::{BatteryResidual, DetectionResult, EnvelopeParams, PipelineConfig};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

const ABLATION_ARTIFACT_TYPE: &str = "dsfb_battery_ablation_summary";
const ABLATION_FIGURE_NAME: &str = "ablation_method_comparison.svg";
const ABLATION_JSON_NAME: &str = "ablation_summary.json";
const ABLATION_CSV_NAME: &str = "ablation_summary.csv";
const ABLATION_SUMMARY_NAME: &str = "implementation_summary.txt";

#[derive(Debug, Clone, Serialize)]
pub struct AblationMethodSummary {
    pub method: String,
    pub trigger_cycle: Option<usize>,
    pub lead_vs_threshold_baseline: Option<i64>,
    pub trigger_stable_to_end: Option<bool>,
    pub triggered_samples: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationCellSummary {
    pub cell_id: String,
    pub threshold_baseline: AblationMethodSummary,
    pub cumulative_residual: AblationMethodSummary,
    pub dsfb: AblationMethodSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub cells_included: Vec<String>,
    pub unavailable_cells: Vec<String>,
    pub methods: Vec<String>,
    pub cell_results: Vec<AblationCellSummary>,
    pub generated_figures: Vec<String>,
}

pub fn build_cumulative_residual_detection(
    trajectory: &[BatteryResidual],
    envelope: &EnvelopeParams,
    capacities: &[f64],
    eol_capacity: f64,
    config: &PipelineConfig,
) -> DetectionResult {
    let alarm_cycle = detect_cumulative_residual_alarm(trajectory, envelope, config);
    let eol_cycle = capacities
        .iter()
        .enumerate()
        .find_map(|(index, capacity)| {
            if *capacity < eol_capacity {
                Some(index + 1)
            } else {
                None
            }
        });
    let lead_time_cycles = alarm_cycle
        .zip(eol_cycle)
        .map(|(alarm, eol)| eol as i64 - alarm as i64);

    DetectionResult {
        method: "Cumulative Residual Change".to_string(),
        alarm_cycle,
        eol_cycle,
        lead_time_cycles,
    }
}

pub fn run_ablation_workflow(
    data_dir: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
) -> Result<AblationArtifact, Box<dyn std::error::Error>> {
    let mut runs: Vec<CellEvaluationRun> = Vec::new();
    let mut unavailable_cells = Vec::new();

    for cell in supported_nasa_pcoe_cells() {
        let path = default_nasa_cell_csv_path(data_dir, cell);
        if !path.exists() {
            unavailable_cells.push(cell.cell_id.to_string());
            continue;
        }

        let raw_data = load_capacity_csv(&path)?;
        let run = evaluate_cell(cell.cell_id, path.to_string_lossy().as_ref(), &raw_data, config)?;
        runs.push(run);
    }

    if runs.is_empty() {
        return Err(format!(
            "no NASA PCoE cell CSVs found in {} for ablation workflow",
            data_dir.display()
        )
        .into());
    }

    std::fs::create_dir_all(output_dir)?;

    let mut cell_results = Vec::new();
    for run in &runs {
        let threshold_alarm = run.threshold_detection.alarm_cycle;
        let threshold_baseline = AblationMethodSummary {
            method: run.threshold_detection.method.clone(),
            trigger_cycle: threshold_alarm,
            lead_vs_threshold_baseline: threshold_alarm.map(|_| 0),
            trigger_stable_to_end: threshold_stable_to_end(&run.capacities, 0.85),
            triggered_samples: threshold_alarm.map(|cycle| run.capacities.len() - cycle + 1),
        };

        let cumulative_detection = build_cumulative_residual_detection(
            &run.trajectory,
            &run.envelope,
            &run.capacities,
            run.dsfb_detection
                .eol_cycle
                .map(|cycle| run.capacities[cycle - 1])
                .unwrap_or_else(|| config.eol_fraction * run.capacities[0]),
            config,
        );
        let cumulative_trigger = cumulative_detection.alarm_cycle;
        let cumulative_residual = AblationMethodSummary {
            method: cumulative_detection.method,
            trigger_cycle: cumulative_trigger,
            lead_vs_threshold_baseline: cumulative_trigger
                .zip(threshold_alarm)
                .map(|(trigger, threshold)| threshold as i64 - trigger as i64),
            trigger_stable_to_end: cumulative_trigger.map(|_| true),
            triggered_samples: cumulative_trigger.map(|cycle| run.capacities.len() - cycle + 1),
        };

        let dsfb_trigger = run.dsfb_detection.alarm_cycle;
        let dsfb = AblationMethodSummary {
            method: run.dsfb_detection.method.clone(),
            trigger_cycle: dsfb_trigger,
            lead_vs_threshold_baseline: dsfb_trigger
                .zip(threshold_alarm)
                .map(|(trigger, threshold)| threshold as i64 - trigger as i64),
            trigger_stable_to_end: dsfb_trigger
                .map(|cycle| run.trajectory.iter().skip(cycle - 1).all(is_non_admissible)),
            triggered_samples: dsfb_trigger.map(|cycle| {
                run.trajectory
                    .iter()
                    .skip(cycle - 1)
                    .filter(|sample| sample.grammar_state != crate::types::GrammarState::Admissible)
                    .count()
            }),
        };

        cell_results.push(AblationCellSummary {
            cell_id: run.summary.cell_id.clone(),
            threshold_baseline,
            cumulative_residual,
            dsfb,
        });
    }

    let artifact = AblationArtifact {
        artifact_type: ABLATION_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_ablation_helper".to_string(),
        cells_included: runs.iter().map(|run| run.summary.cell_id.clone()).collect(),
        unavailable_cells,
        methods: vec![
            "Threshold Baseline".to_string(),
            "Cumulative Residual Change".to_string(),
            "DSFB Structural Alarm".to_string(),
        ],
        cell_results: cell_results.clone(),
        generated_figures: vec![ABLATION_FIGURE_NAME.to_string()],
    };

    write_pretty_json(&artifact, &output_dir.join(ABLATION_JSON_NAME))?;
    write_ablation_csv(&cell_results, &output_dir.join(ABLATION_CSV_NAME))?;
    generate_ablation_comparison_figure(&cell_results, &output_dir.join(ABLATION_FIGURE_NAME))?;
    write_summary_text(
        &artifact,
        &output_dir.join(ABLATION_SUMMARY_NAME),
        output_dir,
        ABLATION_JSON_NAME,
        ABLATION_CSV_NAME,
    )?;

    Ok(artifact)
}

fn detect_cumulative_residual_alarm(
    trajectory: &[BatteryResidual],
    envelope: &EnvelopeParams,
    config: &PipelineConfig,
) -> Option<usize> {
    if trajectory.len() <= config.healthy_window {
        return None;
    }

    let mut cumulative_outward_change = 0.0;
    for index in config.healthy_window.max(1)..trajectory.len() {
        let delta = trajectory[index].sign.r - trajectory[index - 1].sign.r;
        if delta < 0.0 {
            cumulative_outward_change += -delta;
        }
        if cumulative_outward_change >= envelope.rho {
            return Some(trajectory[index].cycle);
        }
    }

    None
}

fn threshold_stable_to_end(capacities: &[f64], threshold_fraction: f64) -> Option<bool> {
    if capacities.is_empty() {
        return None;
    }
    let threshold = capacities[0] * threshold_fraction;
    let alarm_cycle = capacities.iter().enumerate().find_map(|(index, capacity)| {
        if *capacity < threshold {
            Some(index + 1)
        } else {
            None
        }
    })?;
    Some(capacities.iter().skip(alarm_cycle - 1).all(|capacity| *capacity < threshold))
}

fn is_non_admissible(sample: &BatteryResidual) -> bool {
    sample.grammar_state != crate::types::GrammarState::Admissible
}

fn write_pretty_json<T: Serialize>(value: &T, path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn write_ablation_csv(
    cell_results: &[AblationCellSummary],
    path: &Path,
) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "cell_id",
        "method",
        "trigger_cycle",
        "lead_vs_threshold_baseline",
        "trigger_stable_to_end",
        "triggered_samples",
    ])?;

    for cell in cell_results {
        for method in [
            &cell.threshold_baseline,
            &cell.cumulative_residual,
            &cell.dsfb,
        ] {
            writer.write_record(vec![
                cell.cell_id.clone(),
                method.method.clone(),
                method
                    .trigger_cycle
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                method
                    .lead_vs_threshold_baseline
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                method
                    .trigger_stable_to_end
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                method
                    .triggered_samples
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ])?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn write_summary_text(
    artifact: &AblationArtifact,
    path: &Path,
    output_dir: &Path,
    json_name: &str,
    csv_name: &str,
) -> Result<(), ExportError> {
    let mut lines = Vec::new();
    lines.push("Ablation workflow completion summary".to_string());
    lines.push(format!("Cells included: {}", artifact.cells_included.join(", ")));
    lines.push(format!("Unavailable cells: {}", unavailable_label(&artifact.unavailable_cells)));
    lines.push(format!("Methods run: {}", artifact.methods.join(", ")));
    lines.push("Generated artifacts:".to_string());
    lines.push(format!("- {}", json_name));
    lines.push(format!("- {}", csv_name));
    lines.push(format!("- {}", ABLATION_FIGURE_NAME));
    lines.push(format!("- {}", ABLATION_SUMMARY_NAME));
    lines.push(format!("Written to: {}", output_dir.display()));
    lines.push("Gates protecting production outputs:".to_string());
    lines.push("- Existing dsfb-battery-demo binary was left unchanged.".to_string());
    lines.push("- This workflow writes only into its own output directory.".to_string());
    lines.push("- Production figure filenames and stage-II artifact paths were not reused.".to_string());
    lines.push("Data availability limitations:".to_string());
    lines.push("- The workflow evaluates only cell CSVs present in the provided data directory.".to_string());
    lines.push("Confirmation: existing mono-cell production figure paths were not modified by this workflow.".to_string());

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, lines.join("\n"))?;
    Ok(())
}

fn unavailable_label(unavailable_cells: &[String]) -> String {
    if unavailable_cells.is_empty() {
        "none".to_string()
    } else {
        unavailable_cells.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::production_figure_filenames;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(stem: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}-{}", stem, unique))
    }

    fn sample_config() -> PipelineConfig {
        PipelineConfig {
            healthy_window: 3,
            drift_window: 1,
            drift_persistence: 1,
            slew_persistence: 1,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        }
    }

    fn write_cell_csv(dir: &Path, cell_id: &str, capacities: &[f64]) {
        let path = dir.join(format!("nasa_{}_capacity.csv", cell_id.to_lowercase()));
        let mut writer = csv::Writer::from_path(path).unwrap();
        writer.write_record(["cycle", "capacity_ah", "type"]).unwrap();
        for (index, capacity) in capacities.iter().enumerate() {
            writer
                .write_record([
                    (index + 1).to_string(),
                    format!("{capacity:.6}"),
                    "discharge".to_string(),
                ])
                .unwrap();
        }
        writer.flush().unwrap();
    }

    #[test]
    fn ablation_workflow_writes_only_to_its_output_directory() {
        let data_dir = unique_temp_dir("dsfb-battery-ablation-data");
        let output_dir = unique_temp_dir("dsfb-battery-ablation-output");
        fs::create_dir_all(&data_dir).unwrap();
        let cells = [
            ("B0005", vec![2.0, 1.99, 1.98, 1.93, 1.86, 1.78, 1.70, 1.58]),
            ("B0006", vec![2.1, 2.08, 2.05, 1.97, 1.89, 1.80, 1.68, 1.55]),
            ("B0007", vec![1.9, 1.89, 1.88, 1.84, 1.79, 1.71, 1.62, 1.50]),
            ("B0018", vec![1.85, 1.84, 1.83, 1.78, 1.72, 1.63, 1.54, 1.45]),
        ];
        for (cell_id, capacities) in cells {
            write_cell_csv(&data_dir, cell_id, &capacities);
        }

        let artifact = run_ablation_workflow(&data_dir, &output_dir, &sample_config()).unwrap();

        assert_eq!(artifact.cells_included.len(), 4);
        assert!(output_dir.join(ABLATION_JSON_NAME).exists());
        assert!(output_dir.join(ABLATION_CSV_NAME).exists());
        assert!(output_dir.join(ABLATION_FIGURE_NAME).exists());
        assert!(output_dir.join(ABLATION_SUMMARY_NAME).exists());

        let entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries.iter().any(|entry| production_figure_filenames().contains(&entry.as_str())));
        assert!(!entries.iter().any(|entry| entry == "stage2_detection_results.json"));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }
}
