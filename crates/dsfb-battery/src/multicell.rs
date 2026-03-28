// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Additive multi-cell workflow for repeated cell-level evaluation.

use crate::engineer_plots::{
    generate_multicell_lead_time_figure, generate_multicell_residual_state_overview,
    generate_multicell_trigger_cycle_figure,
};
use crate::evaluation::{evaluate_cell, CellEvaluationRun, CellEvaluationSummary};
use crate::export::ExportError;
use crate::load_capacity_csv;
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::types::PipelineConfig;
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

const MULTICELL_ARTIFACT_TYPE: &str = "dsfb_battery_multicell_summary";
const MULTICELL_JSON_NAME: &str = "multicell_summary.json";
const MULTICELL_CSV_NAME: &str = "multicell_summary.csv";
const MULTICELL_SUMMARY_NAME: &str = "implementation_summary.txt";
const LEAD_FIGURE_NAME: &str = "multicell_lead_time_comparison.svg";
const TRIGGER_FIGURE_NAME: &str = "multicell_trigger_cycle_overview.svg";
const OVERVIEW_FIGURE_NAME: &str = "multicell_residual_state_overview.svg";

#[derive(Debug, Clone, Serialize)]
pub struct MultiCellArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub cells_included: Vec<String>,
    pub unavailable_cells: Vec<String>,
    pub cell_summaries: Vec<CellEvaluationSummary>,
    pub generated_figures: Vec<String>,
}

pub fn run_multicell_workflow(
    data_dir: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
) -> Result<MultiCellArtifact, Box<dyn std::error::Error>> {
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
            "no NASA PCoE cell CSVs found in {} for multi-cell workflow",
            data_dir.display()
        )
        .into());
    }

    std::fs::create_dir_all(output_dir)?;

    let artifact = MultiCellArtifact {
        artifact_type: MULTICELL_ARTIFACT_TYPE.to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_multicell_helper".to_string(),
        cells_included: runs.iter().map(|run| run.summary.cell_id.clone()).collect(),
        unavailable_cells,
        cell_summaries: runs.iter().map(|run| run.summary.clone()).collect(),
        generated_figures: vec![
            LEAD_FIGURE_NAME.to_string(),
            TRIGGER_FIGURE_NAME.to_string(),
            OVERVIEW_FIGURE_NAME.to_string(),
        ],
    };

    write_pretty_json(&artifact, &output_dir.join(MULTICELL_JSON_NAME))?;
    write_multicell_csv(&artifact.cell_summaries, &output_dir.join(MULTICELL_CSV_NAME))?;
    generate_multicell_lead_time_figure(&artifact.cell_summaries, &output_dir.join(LEAD_FIGURE_NAME))?;
    generate_multicell_trigger_cycle_figure(&artifact.cell_summaries, &output_dir.join(TRIGGER_FIGURE_NAME))?;
    generate_multicell_residual_state_overview(&runs, &output_dir.join(OVERVIEW_FIGURE_NAME))?;
    write_summary_text(&artifact, &output_dir.join(MULTICELL_SUMMARY_NAME), output_dir)?;

    Ok(artifact)
}

fn write_pretty_json<T: Serialize>(value: &T, path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn write_multicell_csv(
    summaries: &[CellEvaluationSummary],
    path: &Path,
) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "cell_id",
        "cycle_count",
        "dsfb_alarm_cycle",
        "first_boundary_cycle",
        "first_violation_cycle",
        "threshold_85pct_cycle",
        "eol_80pct_cycle",
        "lead_time_vs_threshold_baseline",
        "persistent_elevation_confirmed",
        "primary_reason_code",
        "theorem_t_star",
    ])?;

    for summary in summaries {
        writer.write_record(vec![
            summary.cell_id.clone(),
            summary.cycle_count.to_string(),
            summary
                .dsfb_alarm_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .first_boundary_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .first_violation_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .threshold_85pct_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .eol_80pct_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .lead_time_vs_threshold_baseline
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .persistent_elevation_confirmed
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary
                .primary_reason_code
                .map(|value| value.to_string())
                .unwrap_or_default(),
            summary.theorem_t_star.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_summary_text(
    artifact: &MultiCellArtifact,
    path: &Path,
    output_dir: &Path,
) -> Result<(), ExportError> {
    let mut lines = Vec::new();
    lines.push("Multi-cell workflow completion summary".to_string());
    lines.push(format!("Cells included: {}", artifact.cells_included.join(", ")));
    lines.push(format!(
        "Unavailable cells: {}",
        unavailable_label(&artifact.unavailable_cells)
    ));
    lines.push("Generated artifacts:".to_string());
    lines.push(format!("- {}", MULTICELL_JSON_NAME));
    lines.push(format!("- {}", MULTICELL_CSV_NAME));
    lines.push(format!("- {}", LEAD_FIGURE_NAME));
    lines.push(format!("- {}", TRIGGER_FIGURE_NAME));
    lines.push(format!("- {}", OVERVIEW_FIGURE_NAME));
    lines.push(format!("- {}", MULTICELL_SUMMARY_NAME));
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
    use crate::generate_all_figures;
    use crate::plotting::FigureContext;
    use crate::types::{
        BatteryResidual, DetectionResult, EnvelopeParams, GrammarState, ReasonCode, SignTuple,
        Theorem1Result,
    };
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
    fn multicell_workflow_writes_only_to_its_output_directory() {
        let data_dir = unique_temp_dir("dsfb-battery-multicell-data");
        let output_dir = unique_temp_dir("dsfb-battery-multicell-output");
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

        let artifact = run_multicell_workflow(&data_dir, &output_dir, &sample_config()).unwrap();

        assert_eq!(artifact.cells_included.len(), 4);
        assert!(output_dir.join(MULTICELL_JSON_NAME).exists());
        assert!(output_dir.join(MULTICELL_CSV_NAME).exists());
        assert!(output_dir.join(LEAD_FIGURE_NAME).exists());
        assert!(output_dir.join(TRIGGER_FIGURE_NAME).exists());
        assert!(output_dir.join(OVERVIEW_FIGURE_NAME).exists());
        assert!(output_dir.join(MULTICELL_SUMMARY_NAME).exists());

        let entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries.iter().any(|entry| production_figure_filenames().contains(&entry.as_str())));
        assert!(!entries.iter().any(|entry| entry == "stage2_detection_results.json"));

        let _ = fs::remove_dir_all(&data_dir);
        let _ = fs::remove_dir_all(&output_dir);
    }

    #[test]
    fn production_figure_filenames_remain_unchanged() {
        let output_dir = unique_temp_dir("dsfb-battery-production-figures");
        fs::create_dir_all(&output_dir).unwrap();

        let capacities = vec![2.0, 1.98, 1.95, 1.90, 1.82];
        let trajectory = vec![
            BatteryResidual {
                cycle: 1,
                capacity_ah: 2.0,
                sign: SignTuple {
                    r: 0.010,
                    d: 0.0,
                    s: 0.0,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 2,
                capacity_ah: 1.98,
                sign: SignTuple {
                    r: -0.010,
                    d: -0.001,
                    s: -0.0005,
                },
                grammar_state: GrammarState::Admissible,
                reason_code: None,
            },
            BatteryResidual {
                cycle: 3,
                capacity_ah: 1.95,
                sign: SignTuple {
                    r: -0.030,
                    d: -0.003,
                    s: -0.0012,
                },
                grammar_state: GrammarState::Boundary,
                reason_code: Some(ReasonCode::SustainedCapacityFade),
            },
            BatteryResidual {
                cycle: 4,
                capacity_ah: 1.90,
                sign: SignTuple {
                    r: -0.060,
                    d: -0.004,
                    s: -0.0014,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
            BatteryResidual {
                cycle: 5,
                capacity_ah: 1.82,
                sign: SignTuple {
                    r: -0.090,
                    d: -0.005,
                    s: -0.0011,
                },
                grammar_state: GrammarState::Violation,
                reason_code: Some(ReasonCode::AcceleratingFadeKnee),
            },
        ];

        let config = sample_config();
        let envelope = EnvelopeParams {
            mu: 1.99,
            sigma: 0.02,
            rho: 0.06,
        };
        let dsfb_detection = DetectionResult {
            method: "DSFB Structural Alarm".to_string(),
            alarm_cycle: Some(3),
            eol_cycle: Some(5),
            lead_time_cycles: Some(2),
        };
        let threshold_detection = DetectionResult {
            method: "Threshold Baseline (85% of initial)".to_string(),
            alarm_cycle: Some(4),
            eol_cycle: Some(5),
            lead_time_cycles: Some(1),
        };
        let theorem1 = Theorem1Result {
            rho: 0.06,
            alpha: 0.004,
            kappa: 0.0,
            t_star: 15,
            actual_detection_cycle: Some(3),
            bound_satisfied: Some(true),
        };
        let fig_ctx = FigureContext {
            capacities: &capacities,
            trajectory: &trajectory,
            envelope: &envelope,
            config: &config,
            dsfb_detection: &dsfb_detection,
            threshold_detection: &threshold_detection,
            theorem1: &theorem1,
            data_provenance: "synthetic production path test",
        };

        generate_all_figures(&fig_ctx, &output_dir).unwrap();

        let mut entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();

        let expected: Vec<String> = production_figure_filenames()
            .iter()
            .map(|name| name.to_string())
            .collect();
        assert_eq!(entries, expected);

        let _ = fs::remove_dir_all(&output_dir);
    }
}
