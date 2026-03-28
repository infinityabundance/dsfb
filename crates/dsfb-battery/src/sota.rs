// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing comparison helper against simple alternative baselines.

use crate::engineer_plots::generate_sota_comparison_figure;
use crate::evaluation::evaluate_cell;
use crate::export::ExportError;
use crate::integration::{
    build_validity_token, compute_tactical_margin_summary, TacticalMarginSummary, ValidityToken,
};
use crate::load_capacity_csv;
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::types::{BatteryResidual, EnvelopeParams, PipelineConfig};
use chrono::Utc;
use serde::Serialize;
use std::path::Path;

const SOTA_JSON_NAME: &str = "sota_comparison_summary.json";
const SOTA_CSV_NAME: &str = "sota_comparison_table.csv";
const SOTA_FIGURE_NAME: &str = "sota_detection_cycle_comparison.svg";
const SOTA_SUMMARY_NAME: &str = "implementation_summary.txt";

#[derive(Debug, Clone, Serialize)]
pub struct SotaMethodResult {
    pub method: String,
    pub trigger_cycle: Option<usize>,
    pub lead_vs_threshold_baseline: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SotaPerCellSummary {
    pub cell_id: String,
    pub threshold_baseline: SotaMethodResult,
    pub cusum_style: SotaMethodResult,
    pub ml_style_rul_proxy: SotaMethodResult,
    pub eis_style_proxy: SotaMethodResult,
    pub dsfb: SotaMethodResult,
    pub tactical_margin: TacticalMarginSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct SotaComparisonArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub validity_token: Option<ValidityToken>,
    pub cells_included: Vec<String>,
    pub comparison_scope: String,
    pub cell_summaries: Vec<SotaPerCellSummary>,
    pub generated_figures: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run_sota_comparison_workflow(
    data_dir: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
    tactical_margin_fraction: f64,
    rul_alarm_horizon_cycles: usize,
) -> Result<SotaComparisonArtifact, Box<dyn std::error::Error>> {
    let mut summaries = Vec::new();
    let mut cells_included = Vec::new();

    for cell in supported_nasa_pcoe_cells() {
        let path = default_nasa_cell_csv_path(data_dir, cell);
        if !path.exists() {
            continue;
        }
        let raw_data = load_capacity_csv(&path)?;
        let run = evaluate_cell(
            cell.cell_id,
            path.to_string_lossy().as_ref(),
            &raw_data,
            config,
        )?;
        let threshold_alarm = run.summary.threshold_85pct_cycle;
        let eol_capacity = config.eol_fraction * run.capacities[0];

        let threshold_baseline = SotaMethodResult {
            method: "Threshold Baseline".to_string(),
            trigger_cycle: threshold_alarm,
            lead_vs_threshold_baseline: threshold_alarm.map(|_| 0),
        };

        let cusum_trigger = detect_cusum_style_alarm(&run.trajectory, &run.envelope, config);
        let cusum_style = SotaMethodResult {
            method: "CUSUM-Style Residual Detector".to_string(),
            trigger_cycle: cusum_trigger,
            lead_vs_threshold_baseline: cusum_trigger
                .zip(threshold_alarm)
                .map(|(trigger, threshold)| threshold as i64 - trigger as i64),
        };

        let rul_trigger = detect_ml_style_rul_alarm(
            &run.capacities,
            eol_capacity,
            rul_alarm_horizon_cycles,
            config.healthy_window.max(10),
        );
        let ml_style_rul_proxy = SotaMethodResult {
            method: "ML-Style RUL Proxy (linear regression)".to_string(),
            trigger_cycle: rul_trigger,
            lead_vs_threshold_baseline: rul_trigger
                .zip(threshold_alarm)
                .map(|(trigger, threshold)| threshold as i64 - trigger as i64),
        };

        let eis_proxy_trigger =
            detect_eis_style_proxy_alarm(&run.trajectory, &run.envelope, config);
        let eis_style_proxy = SotaMethodResult {
            method: "EIS-Style Proxy Features".to_string(),
            trigger_cycle: eis_proxy_trigger,
            lead_vs_threshold_baseline: eis_proxy_trigger
                .zip(threshold_alarm)
                .map(|(trigger, threshold)| threshold as i64 - trigger as i64),
        };

        let dsfb = SotaMethodResult {
            method: "DSFB Structural Alarm".to_string(),
            trigger_cycle: run.summary.dsfb_alarm_cycle,
            lead_vs_threshold_baseline: run.summary.lead_time_vs_threshold_baseline,
        };

        cells_included.push(cell.cell_id.to_string());
        summaries.push(SotaPerCellSummary {
            cell_id: cell.cell_id.to_string(),
            threshold_baseline,
            cusum_style,
            ml_style_rul_proxy,
            eis_style_proxy,
            dsfb,
            tactical_margin: compute_tactical_margin_summary(
                &run.capacities,
                &run.trajectory,
                tactical_margin_fraction,
            ),
        });
    }

    std::fs::create_dir_all(output_dir)?;
    let artifact = SotaComparisonArtifact {
        artifact_type: "dsfb_battery_engineer_sota_comparison".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_baseline_comparison".to_string(),
        validity_token: summaries
            .last()
            .and_then(|summary| summary.dsfb.trigger_cycle)
            .map(|cycle| build_validity_token(cycle, true, 60)),
        cells_included,
        comparison_scope:
            "Threshold baseline, CUSUM-style detector, ML-style RUL proxy, EIS-style proxy, and DSFB.".to_string(),
        cell_summaries: summaries.clone(),
        generated_figures: vec![SOTA_FIGURE_NAME.to_string()],
        notes: vec![
            "The ML-style baseline is a simple linear-regression RUL proxy, not a trained external model.".to_string(),
            "The EIS-style baseline uses proxy features derived from available residual, drift, and slew data because true EIS is not present in the current workflow.".to_string(),
            "This helper does not claim SOTA dominance or external benchmark reproduction.".to_string(),
        ],
    };

    write_pretty_json(&artifact, &output_dir.join(SOTA_JSON_NAME))?;
    write_comparison_csv(&summaries, &output_dir.join(SOTA_CSV_NAME))?;
    generate_sota_comparison_figure(&summaries, &output_dir.join(SOTA_FIGURE_NAME))?;
    write_summary_text(&artifact, &output_dir.join(SOTA_SUMMARY_NAME))?;
    Ok(artifact)
}

fn detect_cusum_style_alarm(
    trajectory: &[BatteryResidual],
    envelope: &EnvelopeParams,
    config: &PipelineConfig,
) -> Option<usize> {
    let mut cusum = 0.0;
    for window in trajectory
        .windows(2)
        .skip(config.healthy_window.saturating_sub(1))
    {
        let delta = window[1].sign.r - window[0].sign.r;
        let outward_change = (-delta - config.drift_threshold).max(0.0);
        cusum = (cusum + outward_change).max(0.0);
        if cusum >= envelope.rho {
            return Some(window[1].cycle);
        }
    }
    None
}

fn detect_ml_style_rul_alarm(
    capacities: &[f64],
    eol_capacity: f64,
    alarm_horizon_cycles: usize,
    regression_window: usize,
) -> Option<usize> {
    if capacities.len() <= regression_window {
        return None;
    }

    for end in regression_window..capacities.len() {
        let start = end.saturating_sub(regression_window);
        let window = &capacities[start..=end];
        let (slope, intercept) = linear_regression(window, start + 1);
        if slope >= 0.0 {
            continue;
        }
        let predicted_eol_cycle = (eol_capacity - intercept) / slope;
        if !predicted_eol_cycle.is_finite() {
            continue;
        }
        let current_cycle = end + 1;
        let predicted_rul = predicted_eol_cycle - current_cycle as f64;
        if predicted_rul <= alarm_horizon_cycles as f64 {
            return Some(current_cycle);
        }
    }

    None
}

fn linear_regression(values: &[f64], cycle_start: usize) -> (f64, f64) {
    let n = values.len() as f64;
    let xs: Vec<f64> = (0..values.len())
        .map(|offset| (cycle_start + offset) as f64)
        .collect();
    let sum_x: f64 = xs.iter().sum();
    let sum_y: f64 = values.iter().sum();
    let sum_xy: f64 = xs.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f64 = xs.iter().map(|x| x * x).sum();
    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return (0.0, values[0]);
    }
    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;
    (slope, intercept)
}

fn detect_eis_style_proxy_alarm(
    trajectory: &[BatteryResidual],
    envelope: &EnvelopeParams,
    config: &PipelineConfig,
) -> Option<usize> {
    let proxies: Vec<f64> = trajectory
        .iter()
        .map(|sample| {
            (sample.sign.r.abs() / envelope.rho.max(f64::EPSILON))
                + (sample.sign.d.abs() / config.drift_threshold.max(f64::EPSILON))
                + (sample.sign.s.abs() / config.slew_threshold.max(f64::EPSILON))
        })
        .collect();

    let healthy = &proxies[..config.healthy_window.min(proxies.len())];
    let mean = healthy.iter().sum::<f64>() / healthy.len() as f64;
    let variance = if healthy.len() > 1 {
        healthy
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (healthy.len() - 1) as f64
    } else {
        0.0
    };
    let threshold = if variance > 0.0 {
        mean + 3.0 * variance.sqrt()
    } else {
        1.0
    };

    trajectory
        .iter()
        .zip(proxies.iter())
        .find_map(|(sample, proxy)| {
            if *proxy > threshold && sample.sign.r.abs() > config.boundary_fraction * envelope.rho {
                Some(sample.cycle)
            } else {
                None
            }
        })
}

fn write_pretty_json<T: Serialize>(value: &T, path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn write_comparison_csv(summaries: &[SotaPerCellSummary], path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "cell_id",
        "method",
        "trigger_cycle",
        "lead_vs_threshold_baseline",
    ])?;
    for summary in summaries {
        for method in [
            &summary.threshold_baseline,
            &summary.cusum_style,
            &summary.ml_style_rul_proxy,
            &summary.eis_style_proxy,
            &summary.dsfb,
        ] {
            writer.write_record(vec![
                summary.cell_id.clone(),
                method.method.clone(),
                method
                    .trigger_cycle
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                method
                    .lead_vs_threshold_baseline
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ])?;
        }
    }
    writer.flush()?;
    Ok(())
}

fn write_summary_text(artifact: &SotaComparisonArtifact, path: &Path) -> Result<(), ExportError> {
    let mut lines = Vec::new();
    lines.push("SOTA comparison workflow completion summary".to_string());
    lines.push(format!(
        "Cells included: {}",
        artifact.cells_included.join(", ")
    ));
    lines.push(format!("Comparison scope: {}", artifact.comparison_scope));
    lines.push("Generated artifacts:".to_string());
    lines.push(format!("- {}", SOTA_JSON_NAME));
    lines.push(format!("- {}", SOTA_CSV_NAME));
    lines.push(format!("- {}", SOTA_FIGURE_NAME));
    lines.push(format!("- {}", SOTA_SUMMARY_NAME));
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

    fn write_cell_csv(dir: &Path, cell_id: &str, capacities: &[f64]) {
        let path = dir.join(format!("nasa_{}_capacity.csv", cell_id.to_lowercase()));
        let mut writer = csv::Writer::from_path(path).unwrap();
        writer
            .write_record(["cycle", "capacity_ah", "type"])
            .unwrap();
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
    fn sota_workflow_writes_only_to_its_output_directory() {
        let data_dir = unique_temp_dir("dsfb-battery-sota-data");
        let output_dir = unique_temp_dir("dsfb-battery-sota-output");
        fs::create_dir_all(&data_dir).unwrap();
        for (cell_id, capacities) in [
            ("B0005", vec![2.0, 1.99, 1.98, 1.93, 1.86, 1.78, 1.70, 1.58]),
            ("B0006", vec![2.1, 2.08, 2.05, 1.97, 1.89, 1.80, 1.68, 1.55]),
        ] {
            write_cell_csv(&data_dir, cell_id, &capacities);
        }

        let artifact = run_sota_comparison_workflow(
            &data_dir,
            &output_dir,
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
            0.88,
            20,
        )
        .unwrap();

        assert!(!artifact.cell_summaries.is_empty());
        assert!(output_dir.join(SOTA_JSON_NAME).exists());
        assert!(output_dir.join(SOTA_CSV_NAME).exists());
        assert!(output_dir.join(SOTA_FIGURE_NAME).exists());
        assert!(output_dir.join(SOTA_SUMMARY_NAME).exists());
        assert!(!output_dir.join("stage2_detection_results.json").exists());
        let entries: Vec<String> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(!entries
            .iter()
            .any(|entry| production_figure_filenames().contains(&entry.as_str())));
        let _ = fs::remove_dir_all(data_dir);
        let _ = fs::remove_dir_all(output_dir);
    }
}
