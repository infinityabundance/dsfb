// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Engineer-facing synthetic noise robustness helper.

use crate::engineer_plots::generate_noise_robustness_figure;
use crate::evaluation::evaluate_cell;
use crate::integration::{
    build_validity_token, compute_tactical_margin_summary, TacticalMarginSummary, ValidityToken,
};
use crate::load_capacity_csv;
use crate::nasa::{default_nasa_cell_csv_path, supported_nasa_pcoe_cells};
use crate::types::{GrammarState, PipelineConfig};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

const NOISE_JSON_NAME: &str = "noise_robustness_summary.json";
const NOISE_CSV_NAME: &str = "noise_robustness_summary.csv";
const NOISE_FIGURE_NAME: &str = "noise_robustness_overview.svg";
const NOISE_SUMMARY_NAME: &str = "implementation_summary.txt";

#[derive(Debug, Clone, Serialize)]
pub struct NoiseRobustnessRecord {
    pub cell_id: String,
    pub source_csv: String,
    pub noise_std_fraction: f64,
    pub seed_label: String,
    pub clean_dsfb_alarm_cycle: Option<usize>,
    pub noisy_dsfb_alarm_cycle: Option<usize>,
    pub noisy_threshold_85pct_cycle: Option<usize>,
    pub lead_time_vs_threshold_baseline: Option<i64>,
    pub trigger_stable_to_end: Option<bool>,
    pub trigger_unchanged_vs_clean: bool,
    pub trigger_cycle_shift_vs_clean: Option<i64>,
    pub tactical_margin: TacticalMarginSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoiseRobustnessArtifact {
    pub artifact_type: String,
    pub generated_at_utc: String,
    pub output_contract: String,
    pub perturbation_model: String,
    pub validity_token: Option<ValidityToken>,
    pub cells_included: Vec<String>,
    pub noise_levels: Vec<f64>,
    pub records: Vec<NoiseRobustnessRecord>,
    pub generated_figures: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run_noise_robustness_workflow(
    data_dir: &Path,
    output_dir: &Path,
    config: &PipelineConfig,
    noise_levels: &[f64],
    tactical_margin_fraction: f64,
) -> Result<NoiseRobustnessArtifact, Box<dyn std::error::Error>> {
    let mut records = Vec::new();
    let mut cells_included = Vec::new();

    for cell in supported_nasa_pcoe_cells() {
        let path = default_nasa_cell_csv_path(data_dir, cell);
        if !path.exists() {
            continue;
        }
        let raw_data = load_capacity_csv(&path)?;
        let clean = evaluate_cell(
            cell.cell_id,
            path.to_string_lossy().as_ref(),
            &raw_data,
            config,
        )?;
        cells_included.push(cell.cell_id.to_string());
        let initial_capacity = clean.capacities[0];

        for noise_level in noise_levels {
            let seed_label = format!("{}:{:.4}", cell.cell_id, noise_level);
            let noisy_capacities = inject_gaussian_noise(
                &clean.capacities,
                initial_capacity * *noise_level,
                &seed_label,
            );
            let noisy_raw_data: Vec<(usize, f64)> = noisy_capacities
                .iter()
                .enumerate()
                .map(|(index, value)| (index + 1, *value))
                .collect();
            let noisy = evaluate_cell(
                cell.cell_id,
                path.to_string_lossy().as_ref(),
                &noisy_raw_data,
                config,
            )?;
            let trigger_stable_to_end = noisy.summary.dsfb_alarm_cycle.map(|cycle| {
                noisy
                    .trajectory
                    .iter()
                    .skip(cycle - 1)
                    .all(|sample| sample.grammar_state != GrammarState::Admissible)
            });

            records.push(NoiseRobustnessRecord {
                cell_id: cell.cell_id.to_string(),
                source_csv: path.to_string_lossy().to_string(),
                noise_std_fraction: *noise_level,
                seed_label,
                clean_dsfb_alarm_cycle: clean.summary.dsfb_alarm_cycle,
                noisy_dsfb_alarm_cycle: noisy.summary.dsfb_alarm_cycle,
                noisy_threshold_85pct_cycle: noisy.summary.threshold_85pct_cycle,
                lead_time_vs_threshold_baseline: noisy.summary.lead_time_vs_threshold_baseline,
                trigger_stable_to_end,
                trigger_unchanged_vs_clean: noisy.summary.dsfb_alarm_cycle
                    == clean.summary.dsfb_alarm_cycle,
                trigger_cycle_shift_vs_clean: noisy
                    .summary
                    .dsfb_alarm_cycle
                    .zip(clean.summary.dsfb_alarm_cycle)
                    .map(|(noisy_cycle, clean_cycle)| noisy_cycle as i64 - clean_cycle as i64),
                tactical_margin: compute_tactical_margin_summary(
                    &noisy.capacities,
                    &noisy.trajectory,
                    tactical_margin_fraction,
                ),
            });
        }
    }

    std::fs::create_dir_all(output_dir)?;
    let artifact = NoiseRobustnessArtifact {
        artifact_type: "dsfb_battery_noise_robustness".to_string(),
        generated_at_utc: Utc::now().to_rfc3339(),
        output_contract: "engineer_facing_noise_helper".to_string(),
        perturbation_model:
            "Deterministic additive Gaussian noise on capacity only; std is reported as a fraction of the initial capacity.".to_string(),
        validity_token: records
            .last()
            .and_then(|record| record.noisy_dsfb_alarm_cycle)
            .map(|cycle| build_validity_token(cycle, true, 60)),
        cells_included,
        noise_levels: noise_levels.to_vec(),
        records: records.clone(),
        generated_figures: vec![NOISE_FIGURE_NAME.to_string()],
        notes: vec![
            "The current production path is capacity-only, so this helper perturbs capacity only.".to_string(),
            "No robustness claim is made beyond the observed trigger movement reported here.".to_string(),
        ],
    };

    write_pretty_json(&artifact, &output_dir.join(NOISE_JSON_NAME))?;
    write_noise_csv(&records, &output_dir.join(NOISE_CSV_NAME))?;
    generate_noise_robustness_figure(&records, &output_dir.join(NOISE_FIGURE_NAME))?;
    write_summary_text(&artifact, &output_dir.join(NOISE_SUMMARY_NAME))?;
    Ok(artifact)
}

fn inject_gaussian_noise(capacities: &[f64], sigma_abs: f64, seed_label: &str) -> Vec<f64> {
    let mut rng = DeterministicNormalRng::from_label(seed_label);
    capacities
        .iter()
        .map(|value| value + rng.next_gaussian() * sigma_abs)
        .collect()
}

#[derive(Debug, Clone)]
struct DeterministicNormalRng {
    state: u64,
}

impl DeterministicNormalRng {
    fn from_label(label: &str) -> Self {
        let digest = Sha256::digest(label.as_bytes());
        let mut seed_bytes = [0u8; 8];
        seed_bytes.copy_from_slice(&digest[..8]);
        let seed = u64::from_be_bytes(seed_bytes).max(1);
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state = self.state.wrapping_mul(0x2545F4914F6CDD1D);
        self.state
    }

    fn next_unit_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        (value as f64 + 1.0) / ((1u64 << 53) as f64 + 2.0)
    }

    fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_unit_f64();
        let u2 = self.next_unit_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
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

fn write_noise_csv(
    records: &[NoiseRobustnessRecord],
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "cell_id",
        "noise_std_fraction",
        "clean_dsfb_alarm_cycle",
        "noisy_dsfb_alarm_cycle",
        "noisy_threshold_85pct_cycle",
        "lead_time_vs_threshold_baseline",
        "trigger_stable_to_end",
        "trigger_unchanged_vs_clean",
        "trigger_cycle_shift_vs_clean",
        "tactical_margin_fraction",
        "tactical_margin_cycle",
        "lead_time_vs_tactical_margin_cycles",
    ])?;
    for record in records {
        writer.write_record(vec![
            record.cell_id.clone(),
            format!("{:.4}", record.noise_std_fraction),
            record
                .clean_dsfb_alarm_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record
                .noisy_dsfb_alarm_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record
                .noisy_threshold_85pct_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record
                .lead_time_vs_threshold_baseline
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record
                .trigger_stable_to_end
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record.trigger_unchanged_vs_clean.to_string(),
            record
                .trigger_cycle_shift_vs_clean
                .map(|value| value.to_string())
                .unwrap_or_default(),
            format!("{:.2}", record.tactical_margin.threshold_fraction),
            record
                .tactical_margin
                .threshold_cycle
                .map(|value| value.to_string())
                .unwrap_or_default(),
            record
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
    artifact: &NoiseRobustnessArtifact,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    lines.push("Noise robustness workflow completion summary".to_string());
    lines.push(format!(
        "Cells included: {}",
        artifact.cells_included.join(", ")
    ));
    lines.push(format!(
        "Noise levels: {}",
        artifact
            .noise_levels
            .iter()
            .map(|value| format!("{:.0}%", value * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    lines.push(format!(
        "Perturbation model: {}",
        artifact.perturbation_model
    ));
    lines.push("Generated artifacts:".to_string());
    lines.push(format!("- {}", NOISE_JSON_NAME));
    lines.push(format!("- {}", NOISE_CSV_NAME));
    lines.push(format!("- {}", NOISE_FIGURE_NAME));
    lines.push(format!("- {}", NOISE_SUMMARY_NAME));
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
    fn noise_workflow_writes_only_to_its_output_directory() {
        let data_dir = unique_temp_dir("dsfb-battery-noise-data");
        let output_dir = unique_temp_dir("dsfb-battery-noise-output");
        fs::create_dir_all(&data_dir).unwrap();
        for (cell_id, capacities) in [
            ("B0005", vec![2.0, 1.99, 1.98, 1.93, 1.86, 1.78, 1.70, 1.58]),
            ("B0006", vec![2.1, 2.08, 2.05, 1.97, 1.89, 1.80, 1.68, 1.55]),
        ] {
            write_cell_csv(&data_dir, cell_id, &capacities);
        }

        let artifact = run_noise_robustness_workflow(
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
            &[0.01, 0.02, 0.05],
            0.88,
        )
        .unwrap();

        assert!(!artifact.records.is_empty());
        assert!(output_dir.join(NOISE_JSON_NAME).exists());
        assert!(output_dir.join(NOISE_CSV_NAME).exists());
        assert!(output_dir.join(NOISE_FIGURE_NAME).exists());
        assert!(output_dir.join(NOISE_SUMMARY_NAME).exists());
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
