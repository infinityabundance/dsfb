use crate::baselines::compute_baselines;
use crate::config::PipelineConfig;
use crate::dataset::phm2018::support_status;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::evaluate_grammar;
use crate::heuristics::{
    expanded_semantic_policy_definitions, heuristic_policy_definition, HeuristicAlertClass,
};
use crate::nominal::build_nominal_model;
use crate::output_paths::create_timestamped_run_dir;
use crate::precursor::evaluate_dsa;
use crate::preprocessing::{DatasetSummary, PreparedDataset};
use crate::residual::compute_residuals;
use crate::semiotics::{build_semantic_layer, classify_motifs};
use crate::signs::compute_signs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

const PHM_SENSOR_COLUMN_START: usize = 7;
const PHM_MAX_AGGREGATED_POINTS: usize = 4096;
const PHM_SELECTED_DSA_WINDOW: usize = 8;
const PHM_SELECTED_DSA_PERSISTENCE: usize = 2;
const PHM_SELECTED_DSA_TAU: f64 = 1.5;
const PHM_SELECTED_DSA_M: usize = 1;
const PHM_THRESHOLD_BASELINE: &str = "run_energy_scalar_threshold";

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018RunArtifacts {
    pub run_dir: PathBuf,
    pub lead_time_metrics_path: PathBuf,
    pub early_warning_stats_path: PathBuf,
    pub structural_metrics_path: PathBuf,
    pub claim_alignment_report_path: PathBuf,
    pub manifest_path: PathBuf,
    pub zip_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018LeadTimeRow {
    pub run_id: String,
    pub dsfb_detection_time: Option<i64>,
    pub threshold_detection_time: Option<i64>,
    pub lead_time_delta: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018EarlyWarningStats {
    pub threshold_baseline: String,
    pub total_runs: usize,
    pub comparable_runs: usize,
    pub mean_lead_delta: Option<f64>,
    pub median_lead_delta: Option<f64>,
    pub percent_runs_dsfb_earlier: f64,
    pub percent_runs_equal: f64,
    pub percent_runs_later: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018StructuralMetrics {
    pub threshold_baseline: String,
    pub total_runs: usize,
    pub runs_with_structured_emergence: usize,
    pub comparable_structure_runs: usize,
    pub runs_with_structure_before_threshold: usize,
    pub percent_structure_before_threshold: f64,
    pub mean_structure_minus_threshold_delta: Option<f64>,
    pub median_structure_minus_threshold_delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimAlignmentReport {
    pub secom_supports: Vec<String>,
    pub secom_does_not_support: Vec<String>,
    pub phm2018_supports: Vec<String>,
    pub claims_not_made: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Phm2018ArtifactManifest {
    dataset: String,
    run_dir: String,
    lead_time_metrics_path: String,
    early_warning_stats_path: String,
    structural_metrics_path: String,
    support_status_path: String,
    claim_alignment_report_path: String,
    zip_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phm2018RunDetail {
    pub run_id: String,
    pub fault_time: i64,
    pub fault_index: usize,
    pub healthy_prefix_count: usize,
    pub evaluation_start_run_index: usize,
    pub dsfb_detection_run_index: Option<usize>,
    pub threshold_detection_run_index: Option<usize>,
    pub earliest_semantic_run_index: Option<usize>,
    pub earliest_structured_run_index: Option<usize>,
    pub dsfb_detection_time: Option<i64>,
    pub threshold_detection_time: Option<i64>,
    pub earliest_semantic_time: Option<i64>,
    pub earliest_structured_time: Option<i64>,
    pub lead_time_delta: Option<i64>,
    pub structure_minus_threshold_delta: Option<i64>,
}

#[derive(Debug, Clone)]
struct Phm2018RunSpec {
    run_id: String,
    sensor_path: PathBuf,
    fault_time: i64,
}

#[derive(Debug, Clone)]
struct Phm2018RunSeries {
    run_id: String,
    timestamps_raw: Vec<i64>,
    feature_names: Vec<String>,
    raw_values: Vec<Vec<Option<f64>>>,
    fault_time: i64,
    fault_index: usize,
    healthy_prefix_count: usize,
}

pub fn run_phm2018_benchmark(
    data_root: &Path,
    output_root: &Path,
    secom_run_dir: Option<&Path>,
) -> Result<Phm2018RunArtifacts> {
    let status = support_status(data_root);
    if !status.extracted_dataset_detected {
        return Err(DsfbSemiconductorError::DatasetMissing {
            dataset: "PHM 2018 ion mill etch",
            path: status.extracted_dataset_path,
        });
    }

    let run_specs = load_phm2018_train_run_specs(&status.extracted_dataset_path)?;
    if run_specs.is_empty() {
        return Err(DsfbSemiconductorError::DatasetFormat(
            "PHM 2018 extracted tree contains no train runs".into(),
        ));
    }

    fs::create_dir_all(output_root)?;
    let run_dir = create_timestamped_run_dir(output_root, "phm2018")?;
    let mut lead_time_rows = Vec::new();
    let mut run_details = Vec::new();

    for run_spec in &run_specs {
        let run = load_phm2018_train_run_series(run_spec)?;
        let config = phm_pipeline_config(run.healthy_prefix_count, run.fault_index);
        let prepared = run.as_prepared_dataset();
        let nominal = build_nominal_model(&prepared, &config);
        let residuals = compute_residuals(&prepared, &nominal);
        let signs = compute_signs(&prepared, &nominal, &residuals, &config);
        let baselines = compute_baselines(&prepared, &nominal, &residuals, &config);
        let grammar = evaluate_grammar(&residuals, &signs, &nominal, &config);
        let motifs = classify_motifs(
            &prepared,
            &nominal,
            &residuals,
            &signs,
            &grammar,
            config.pre_failure_lookback_runs,
        );
        let semantic_layer = build_semantic_layer(
            &prepared,
            &residuals,
            &signs,
            &grammar,
            &motifs,
            &nominal,
            config.pre_failure_lookback_runs,
        );
        let dsa = evaluate_dsa(
            &prepared,
            &nominal,
            &residuals,
            &signs,
            &baselines,
            &grammar,
            &config.dsa,
            config.pre_failure_lookback_runs,
        )?;

        let evaluation_start_run_index = run.healthy_prefix_count.min(run.fault_index);
        let dsfb_detection_run_index = (evaluation_start_run_index..run.fault_index)
            .find(|&run_index| dsa.run_signals.primary_run_alert[run_index]);
        let threshold_detection_run_index = (evaluation_start_run_index..run.fault_index)
            .find(|&run_index| baselines.run_energy.alarm[run_index]);
        let dsfb_detection_time =
            dsfb_detection_run_index.map(|run_index| run.timestamps_raw[run_index]);
        let threshold_detection_time =
            threshold_detection_run_index.map(|run_index| run.timestamps_raw[run_index]);
        let earliest_semantic_run_index = semantic_layer
            .ranked_candidates
            .iter()
            .filter(|row| {
                row.run_index >= evaluation_start_run_index && row.run_index < run.fault_index
            })
            .filter(|row| {
                !matches!(
                    heuristic_alert_default(row.heuristic_name.as_str()),
                    HeuristicAlertClass::Silent
                )
            })
            .map(|row| row.run_index)
            .min();
        let earliest_structured_run_index = earliest_semantic_run_index.or_else(|| {
            motifs
                .traces
                .iter()
                .flat_map(|trace| trace.labels.iter().enumerate())
                .filter(|(run_index, label)| {
                    *run_index >= evaluation_start_run_index
                        && *run_index < run.fault_index
                        && !matches!(label, crate::semiotics::DsfbMotifClass::StableAdmissible)
                })
                .map(|(run_index, _)| run_index)
                .min()
        });
        let earliest_semantic_time =
            earliest_semantic_run_index.map(|run_index| run.timestamps_raw[run_index]);
        let earliest_structured_time =
            earliest_structured_run_index.map(|run_index| run.timestamps_raw[run_index]);
        let lead_time_delta = match (dsfb_detection_time, threshold_detection_time) {
            (Some(dsfb), Some(threshold)) => Some(threshold - dsfb),
            _ => None,
        };
        let structure_minus_threshold_delta =
            match (earliest_structured_time, threshold_detection_time) {
                (Some(structure), Some(threshold)) => Some(threshold - structure),
                _ => None,
            };

        lead_time_rows.push(Phm2018LeadTimeRow {
            run_id: run.run_id.clone(),
            dsfb_detection_time,
            threshold_detection_time,
            lead_time_delta,
        });
        run_details.push(Phm2018RunDetail {
            run_id: run.run_id.clone(),
            fault_time: run.fault_time,
            fault_index: run.fault_index,
            healthy_prefix_count: run.healthy_prefix_count,
            evaluation_start_run_index,
            dsfb_detection_run_index,
            threshold_detection_run_index,
            earliest_semantic_run_index,
            earliest_structured_run_index,
            dsfb_detection_time,
            threshold_detection_time,
            earliest_semantic_time,
            earliest_structured_time,
            lead_time_delta,
            structure_minus_threshold_delta,
        });
    }

    let early_warning_stats = summarize_phm_lead_times(&lead_time_rows);
    let structural_metrics = summarize_phm_structural_metrics(&run_details);
    let secom_run_dir = resolve_secom_run_dir(secom_run_dir, output_root)?;
    let claim_alignment_report =
        build_claim_alignment_report(&secom_run_dir, &early_warning_stats, &structural_metrics)?;

    let lead_time_metrics_path = run_dir.join("phm2018_lead_time_metrics.csv");
    let early_warning_stats_path = run_dir.join("phm2018_early_warning_stats.json");
    let structural_metrics_path = run_dir.join("phm2018_structural_metrics.json");
    let claim_alignment_report_path = run_dir.join("claim_alignment_report.json");
    let manifest_path = run_dir.join("artifact_manifest.json");
    let zip_path = run_dir.join("run_bundle.zip");

    write_serialized_csv(&lead_time_metrics_path, &lead_time_rows)?;
    write_json_pretty(&early_warning_stats_path, &early_warning_stats)?;
    write_json_pretty(&structural_metrics_path, &structural_metrics)?;
    write_json_pretty(&run_dir.join("phm2018_support_status.json"), &status)?;
    write_json_pretty(&run_dir.join("phm2018_run_details.json"), &run_details)?;
    write_json_pretty(&claim_alignment_report_path, &claim_alignment_report)?;
    write_json_pretty(
        &manifest_path,
        &Phm2018ArtifactManifest {
            dataset: "PHM2018".into(),
            run_dir: run_dir.display().to_string(),
            lead_time_metrics_path: lead_time_metrics_path.display().to_string(),
            early_warning_stats_path: early_warning_stats_path.display().to_string(),
            structural_metrics_path: structural_metrics_path.display().to_string(),
            support_status_path: run_dir
                .join("phm2018_support_status.json")
                .display()
                .to_string(),
            claim_alignment_report_path: claim_alignment_report_path.display().to_string(),
            zip_path: zip_path.display().to_string(),
        },
    )?;
    zip_directory(&run_dir, &zip_path)?;

    Ok(Phm2018RunArtifacts {
        run_dir,
        lead_time_metrics_path,
        early_warning_stats_path,
        structural_metrics_path,
        claim_alignment_report_path,
        manifest_path,
        zip_path,
    })
}

fn load_phm2018_train_run_specs(extracted_root: &Path) -> Result<Vec<Phm2018RunSpec>> {
    let train_dir = extracted_root.join("train");
    let fault_dir = train_dir.join("train_faults");
    let ttf_dir = train_dir.join("train_ttf");

    let sensor_files = fs::read_dir(&train_dir)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("csv"))
        .collect::<Vec<_>>();

    let fault_times = load_fault_times(&fault_dir)?;
    let ttf_fallbacks = load_ttf_zero_times(&ttf_dir)?;
    let mut run_specs = Vec::new();

    for sensor_path in sensor_files {
        let run_id = run_id_from_sensor_path(&sensor_path)?;
        let fault_time = fault_times
            .get(&run_id)
            .copied()
            .or_else(|| ttf_fallbacks.get(&run_id).copied())
            .ok_or_else(|| {
                DsfbSemiconductorError::DatasetFormat(format!(
                    "missing fault target for PHM train run {run_id}"
                ))
            })?;
        run_specs.push(Phm2018RunSpec {
            run_id,
            fault_time,
            sensor_path,
        });
    }

    run_specs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    Ok(run_specs)
}

fn load_phm2018_train_run_series(run_spec: &Phm2018RunSpec) -> Result<Phm2018RunSeries> {
    let (timestamps_raw, feature_names, raw_values) =
        load_sensor_csv_aggregated(&run_spec.sensor_path)?;
    if timestamps_raw.is_empty() || feature_names.is_empty() || raw_values.is_empty() {
        return Err(DsfbSemiconductorError::DatasetFormat(format!(
            "empty PHM train run {} at {}",
            run_spec.run_id,
            run_spec.sensor_path.display()
        )));
    }
    let fault_index = timestamps_raw
        .iter()
        .position(|time| *time >= run_spec.fault_time)
        .unwrap_or_else(|| timestamps_raw.len().saturating_sub(1));
    let healthy_prefix_count = healthy_prefix_count(fault_index, timestamps_raw.len());

    Ok(Phm2018RunSeries {
        run_id: run_spec.run_id.clone(),
        timestamps_raw,
        feature_names,
        raw_values,
        fault_time: run_spec.fault_time,
        fault_index,
        healthy_prefix_count,
    })
}

impl Phm2018RunSeries {
    fn as_prepared_dataset(&self) -> PreparedDataset {
        let run_count = self.raw_values.len();
        let feature_count = self.feature_names.len();
        let total_cells = run_count * feature_count;
        let missing_cells = self
            .raw_values
            .iter()
            .flat_map(|row| row.iter())
            .filter(|value| value.is_none())
            .count();
        let mut per_feature_missing_fraction = vec![0.0; feature_count];
        for feature_index in 0..feature_count {
            let missing = self
                .raw_values
                .iter()
                .filter(|row| row[feature_index].is_none())
                .count();
            per_feature_missing_fraction[feature_index] = if run_count == 0 {
                0.0
            } else {
                missing as f64 / run_count as f64
            };
        }

        let mut labels = vec![-1; run_count];
        if self.fault_index < labels.len() {
            labels[self.fault_index] = 1;
        }
        let timestamps = self
            .timestamps_raw
            .iter()
            .enumerate()
            .map(|(index, value)| {
                DateTime::<Utc>::from_timestamp(*value, 0)
                    .map(|value| value.naive_utc())
                    .unwrap_or_else(|| {
                        DateTime::<Utc>::from_timestamp(index as i64, 0)
                            .expect("valid synthetic timestamp")
                            .naive_utc()
                    })
            })
            .collect::<Vec<_>>();
        let healthy_pass_indices = (0..self.healthy_prefix_count).collect::<Vec<_>>();

        PreparedDataset {
            feature_names: self.feature_names.clone(),
            labels,
            timestamps,
            raw_values: self.raw_values.clone(),
            healthy_pass_indices,
            per_feature_missing_fraction,
            summary: DatasetSummary {
                run_count,
                feature_count,
                pass_count: run_count.saturating_sub(1),
                fail_count: 1,
                dataset_missing_fraction: if total_cells == 0 {
                    0.0
                } else {
                    missing_cells as f64 / total_cells as f64
                },
                healthy_pass_runs_requested: self.healthy_prefix_count,
                healthy_pass_runs_found: self.healthy_prefix_count,
            },
        }
    }
}

fn phm_pipeline_config(healthy_prefix_count: usize, fault_index: usize) -> PipelineConfig {
    PipelineConfig {
        healthy_pass_runs: healthy_prefix_count.max(2),
        pre_failure_lookback_runs: fault_index.max(1),
        dsa: crate::precursor::DsaConfig {
            window: PHM_SELECTED_DSA_WINDOW,
            persistence_runs: PHM_SELECTED_DSA_PERSISTENCE,
            alert_tau: PHM_SELECTED_DSA_TAU,
            corroborating_feature_count_min: PHM_SELECTED_DSA_M,
        },
        ..PipelineConfig::default()
    }
}

fn healthy_prefix_count(fault_index: usize, run_len: usize) -> usize {
    let proportional = (fault_index as f64 * 0.10).round() as usize;
    proportional
        .clamp(25, 200)
        .min(fault_index.max(2))
        .min(run_len)
}

fn run_id_from_sensor_path(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| {
            DsfbSemiconductorError::DatasetFormat("invalid PHM sensor filename".into())
        })?;
    let mut parts = stem.split('_');
    let lot = parts
        .next()
        .ok_or_else(|| DsfbSemiconductorError::DatasetFormat("missing PHM lot id".into()))?;
    let tool = parts
        .next()
        .ok_or_else(|| DsfbSemiconductorError::DatasetFormat("missing PHM tool id".into()))?;
    Ok(format!("{lot}_{tool}"))
}

fn load_sensor_csv_aggregated(
    path: &Path,
) -> Result<(Vec<i64>, Vec<String>, Vec<Vec<Option<f64>>>)> {
    let total_rows = estimate_csv_data_rows(path)?;
    let bucket_size = (total_rows / PHM_MAX_AGGREGATED_POINTS).max(1);

    let mut reader = csv::ReaderBuilder::new().from_path(path)?;
    let header = reader
        .headers()?
        .iter()
        .skip(PHM_SENSOR_COLUMN_START)
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let feature_count = header.len();

    let mut timestamps = Vec::new();
    let mut raw_values = Vec::new();
    let mut bucket_time_sum = 0f64;
    let mut bucket_time_count = 0usize;
    let mut bucket_row_count = 0usize;
    let mut bucket_sums = vec![0.0; feature_count];
    let mut bucket_counts = vec![0usize; feature_count];

    for record in reader.records() {
        let record = record?;
        let timestamp = record
            .get(0)
            .ok_or_else(|| DsfbSemiconductorError::DatasetFormat("missing PHM time field".into()))?
            .parse::<i64>()
            .map_err(|err| {
                DsfbSemiconductorError::DatasetFormat(format!(
                    "invalid PHM time value in {}: {err}",
                    path.display()
                ))
            })?;
        bucket_time_sum += timestamp as f64;
        bucket_time_count += 1;
        bucket_row_count += 1;

        for (feature_index, value) in record.iter().skip(PHM_SENSOR_COLUMN_START).enumerate() {
            if let Ok(parsed) = value.parse::<f64>() {
                bucket_sums[feature_index] += parsed;
                bucket_counts[feature_index] += 1;
            }
        }

        if bucket_row_count >= bucket_size {
            finalize_aggregate_bucket(
                &mut timestamps,
                &mut raw_values,
                &mut bucket_time_sum,
                &mut bucket_time_count,
                &mut bucket_row_count,
                &mut bucket_sums,
                &mut bucket_counts,
            );
        }
    }

    finalize_aggregate_bucket(
        &mut timestamps,
        &mut raw_values,
        &mut bucket_time_sum,
        &mut bucket_time_count,
        &mut bucket_row_count,
        &mut bucket_sums,
        &mut bucket_counts,
    );

    Ok((timestamps, header, raw_values))
}

fn estimate_csv_data_rows(path: &Path) -> Result<usize> {
    let file = File::open(path)?;
    let total_bytes = file.metadata()?.len() as f64;
    let mut reader = BufReader::new(file);
    let mut sampled_lines = 0usize;
    let mut sampled_bytes = 0usize;
    let mut buffer = String::new();

    while sampled_lines < 4096 {
        buffer.clear();
        let bytes = reader.read_line(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        sampled_lines += 1;
        sampled_bytes += bytes;
    }

    if sampled_lines == 0 || sampled_bytes == 0 {
        return Ok(0);
    }

    let average_bytes_per_line = sampled_bytes as f64 / sampled_lines as f64;
    let estimated_total_lines = (total_bytes / average_bytes_per_line).round() as usize;
    Ok(estimated_total_lines.saturating_sub(1))
}

fn finalize_aggregate_bucket(
    timestamps: &mut Vec<i64>,
    raw_values: &mut Vec<Vec<Option<f64>>>,
    bucket_time_sum: &mut f64,
    bucket_time_count: &mut usize,
    bucket_row_count: &mut usize,
    bucket_sums: &mut [f64],
    bucket_counts: &mut [usize],
) {
    if *bucket_row_count == 0 {
        return;
    }

    timestamps.push((*bucket_time_sum / *bucket_time_count as f64).round() as i64);
    raw_values.push(
        bucket_sums
            .iter()
            .zip(bucket_counts.iter())
            .map(|(sum, count)| (*count > 0).then_some(*sum / *count as f64))
            .collect(),
    );

    *bucket_time_sum = 0.0;
    *bucket_time_count = 0;
    *bucket_row_count = 0;
    bucket_sums.fill(0.0);
    bucket_counts.fill(0);
}

fn load_fault_times(fault_dir: &Path) -> Result<BTreeMap<String, i64>> {
    let mut map = BTreeMap::new();
    for entry in fs::read_dir(fault_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let run_id = file_name
            .split("_train_fault_data")
            .next()
            .unwrap_or(file_name)
            .to_string();
        let mut reader = csv::ReaderBuilder::new().from_path(&path)?;
        let mut earliest: Option<i64> = None;
        for record in reader.records() {
            let record = record?;
            let time = record
                .get(0)
                .ok_or_else(|| {
                    DsfbSemiconductorError::DatasetFormat(format!(
                        "fault file {} missing time column",
                        path.display()
                    ))
                })?
                .parse::<i64>()
                .map_err(|err| {
                    DsfbSemiconductorError::DatasetFormat(format!(
                        "invalid fault time in {}: {err}",
                        path.display()
                    ))
                })?;
            earliest = Some(match earliest {
                Some(current) => current.min(time),
                None => time,
            });
        }
        if let Some(time) = earliest {
            map.insert(run_id, time);
        }
    }
    Ok(map)
}

fn load_ttf_zero_times(ttf_dir: &Path) -> Result<BTreeMap<String, i64>> {
    let mut map = BTreeMap::new();
    for entry in fs::read_dir(ttf_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
            continue;
        }
        let run_id = run_id_from_sensor_path(&path)?;
        let mut reader = csv::ReaderBuilder::new().from_path(&path)?;
        let mut earliest = None;
        for record in reader.records() {
            let record = record?;
            let time = record
                .get(0)
                .ok_or_else(|| {
                    DsfbSemiconductorError::DatasetFormat(format!(
                        "ttf file {} missing time column",
                        path.display()
                    ))
                })?
                .parse::<i64>()
                .map_err(|err| {
                    DsfbSemiconductorError::DatasetFormat(format!(
                        "invalid ttf time in {}: {err}",
                        path.display()
                    ))
                })?;
            let has_zero = record
                .iter()
                .skip(1)
                .filter_map(|value| value.parse::<f64>().ok())
                .any(|value| value <= 0.0);
            if has_zero {
                earliest = Some(time);
                break;
            }
        }
        if let Some(time) = earliest {
            map.insert(run_id, time);
        }
    }
    Ok(map)
}

fn summarize_phm_lead_times(rows: &[Phm2018LeadTimeRow]) -> Phm2018EarlyWarningStats {
    let comparable = rows
        .iter()
        .filter_map(|row| row.lead_time_delta.map(|value| value as f64))
        .collect::<Vec<_>>();
    let earlier = rows
        .iter()
        .filter(
            |row| match (row.dsfb_detection_time, row.threshold_detection_time) {
                (Some(dsfb), Some(threshold)) => dsfb < threshold,
                (Some(_), None) => true,
                _ => false,
            },
        )
        .count();
    let equal = rows
        .iter()
        .filter(
            |row| match (row.dsfb_detection_time, row.threshold_detection_time) {
                (Some(dsfb), Some(threshold)) => dsfb == threshold,
                _ => false,
            },
        )
        .count();
    let later = rows.len().saturating_sub(earlier + equal);
    let mut sorted = comparable.clone();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.is_empty() {
        None
    } else if sorted.len() % 2 == 1 {
        Some(sorted[sorted.len() / 2])
    } else {
        Some((sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0)
    };

    Phm2018EarlyWarningStats {
        threshold_baseline: PHM_THRESHOLD_BASELINE.into(),
        total_runs: rows.len(),
        comparable_runs: comparable.len(),
        mean_lead_delta: (!comparable.is_empty())
            .then_some(comparable.iter().sum::<f64>() / comparable.len() as f64),
        median_lead_delta: median,
        percent_runs_dsfb_earlier: percent(earlier, rows.len()),
        percent_runs_equal: percent(equal, rows.len()),
        percent_runs_later: percent(later, rows.len()),
    }
}

fn summarize_phm_structural_metrics(run_details: &[Phm2018RunDetail]) -> Phm2018StructuralMetrics {
    let comparable = run_details
        .iter()
        .filter_map(|detail| {
            detail
                .structure_minus_threshold_delta
                .map(|value| value as f64)
        })
        .collect::<Vec<_>>();
    let runs_with_structured_emergence = run_details
        .iter()
        .filter(|detail| detail.earliest_structured_run_index.is_some())
        .count();
    let runs_with_structure_before_threshold = run_details
        .iter()
        .filter(|detail| {
            detail
                .structure_minus_threshold_delta
                .is_some_and(|value| value > 0)
        })
        .count();
    let mut sorted = comparable.clone();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.is_empty() {
        None
    } else if sorted.len() % 2 == 1 {
        Some(sorted[sorted.len() / 2])
    } else {
        Some((sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0)
    };

    Phm2018StructuralMetrics {
        threshold_baseline: PHM_THRESHOLD_BASELINE.into(),
        total_runs: run_details.len(),
        runs_with_structured_emergence,
        comparable_structure_runs: comparable.len(),
        runs_with_structure_before_threshold,
        percent_structure_before_threshold: percent(
            runs_with_structure_before_threshold,
            run_details.len(),
        ),
        mean_structure_minus_threshold_delta: (!comparable.is_empty())
            .then_some(comparable.iter().sum::<f64>() / comparable.len() as f64),
        median_structure_minus_threshold_delta: median,
    }
}

fn build_claim_alignment_report(
    secom_run_dir: &Path,
    phm_stats: &Phm2018EarlyWarningStats,
    phm_structural: &Phm2018StructuralMetrics,
) -> Result<ClaimAlignmentReport> {
    let secom_targets =
        load_json::<serde_json::Value>(&secom_run_dir.join("dsa_operator_delta_targets.json"))?;
    let episode_precision =
        load_json::<serde_json::Value>(&secom_run_dir.join("episode_precision_metrics.json")).ok();
    let episode_precision_text = episode_precision
        .as_ref()
        .and_then(|json| {
            Some((
                json.get("dsfb_precision")?.as_f64()?,
                json.get("raw_alarm_precision")?.as_f64()?,
                json.get("precision_gain_factor")?.as_f64()?,
            ))
        })
        .map(|(dsfb_precision, raw_precision, gain)| {
            format!(
                "episode precision, with DSFB at {:.1}% versus a raw-boundary proxy of {:.2}% ({:.1}x)",
                dsfb_precision * 100.0,
                raw_precision * 100.0,
                gain,
            )
        })
        .unwrap_or_else(|| "episode precision surfaced as the primary operator metric".into());
    let delta_investigation = secom_targets
        .get("delta_investigation_load")
        .and_then(|value| value.as_f64())
        .unwrap_or_default()
        * 100.0;
    let delta_episode = secom_targets
        .get("delta_episode_count")
        .and_then(|value| value.as_f64())
        .unwrap_or_default()
        * 100.0;
    let delta_nuisance_vs_ewma = secom_targets
        .get("delta_nuisance_vs_ewma")
        .and_then(|value| value.as_f64())
        .unwrap_or_default()
        * 100.0;

    let mut phm_supports = Vec::new();
    if phm_stats.percent_runs_dsfb_earlier > phm_stats.percent_runs_later {
        phm_supports.push(format!(
            "early warning: DSFB fires earlier than the {} baseline on {:.1}% of PHM2018 runs",
            PHM_THRESHOLD_BASELINE,
            phm_stats.percent_runs_dsfb_earlier * 100.0
        ));
    } else {
        phm_supports.push(format!(
                "bounded PHM2018 comparison only: DSFB is earlier than the {} baseline on {:.1}% of runs, so a broad early-warning claim is not made",
                PHM_THRESHOLD_BASELINE,
                phm_stats.percent_runs_dsfb_earlier * 100.0
            ));
    }
    if let Some(mean_delta) = phm_stats.mean_lead_delta {
        if mean_delta > 0.0 {
            phm_supports.push(format!(
                "lead-time advantage: mean {}-minus-DSFB detection gap {:.2}",
                PHM_THRESHOLD_BASELINE, mean_delta
            ));
        } else {
            phm_supports.push(format!(
                "lead-time comparison only: mean {}-minus-DSFB detection gap {:.2}",
                PHM_THRESHOLD_BASELINE, mean_delta
            ));
        }
    }
    if let Some(mean_structure_delta) = phm_structural.mean_structure_minus_threshold_delta {
        phm_supports.push(format!(
            "structure-emergence comparison: mean {}-minus-structure-emergence gap {:.2}, with structure preceding threshold on {:.1}% of runs",
            PHM_THRESHOLD_BASELINE,
            mean_structure_delta,
            phm_structural.percent_structure_before_threshold * 100.0,
        ));
    }

    Ok(ClaimAlignmentReport {
        secom_supports: vec![
            format!(
                "episode compression: {:.1}% reduction versus the raw-boundary episode baseline",
                delta_episode
            ),
            episode_precision_text,
            format!(
                "investigation load reduction: {:.1}% versus Numeric-only DSA",
                delta_investigation
            ),
        ],
        secom_does_not_support: vec![
            format!(
                "≥40% nuisance reduction versus EWMA; the saved SECOM row achieves {:.1}%",
                delta_nuisance_vs_ewma
            ),
            "strong early-warning claims; SECOM threshold and EWMA still lead DSFB on mean lead time".into(),
        ],
        phm2018_supports: phm_supports,
        claims_not_made: vec![
            "any unsupported delta without naming its baseline".into(),
            "universal dominance over scalar baselines".into(),
            "SECOM early-warning superiority".into(),
            "PHM burden reduction without direct PHM burden metrics".into(),
        ],
    })
}

fn heuristic_alert_default(heuristic_name: &str) -> HeuristicAlertClass {
    heuristic_policy_definition(heuristic_name)
        .map(|definition| definition.alert_class_default)
        .or_else(|| {
            expanded_semantic_policy_definitions()
                .into_iter()
                .find(|definition| definition.motif_name == heuristic_name)
                .map(|definition| definition.alert_class_default)
        })
        .unwrap_or(HeuristicAlertClass::Silent)
}

fn resolve_secom_run_dir(secom_run_dir: Option<&Path>, output_root: &Path) -> Result<PathBuf> {
    if let Some(path) = secom_run_dir {
        return Ok(path.to_path_buf());
    }
    let candidates = [
        output_root.to_path_buf(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-semiconductor"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .join("output-dsfb-semiconductor"),
    ];
    for root in candidates {
        if let Some(path) = latest_run_dir(&root, "_secom")? {
            return Ok(path);
        }
    }
    Err(DsfbSemiconductorError::DatasetFormat(
        "could not resolve a SECOM run directory for claim alignment".into(),
    ))
}

fn latest_run_dir(root: &Path, suffix: &str) -> Result<Option<PathBuf>> {
    let mut dirs = fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("dsfb-semiconductor") && name.ends_with(suffix))
        })
        .collect::<Vec<_>>();
    dirs.sort();
    Ok(dirs.pop())
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, value)?;
    Ok(())
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}

fn write_serialized_csv<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn zip_directory(run_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut stack = vec![run_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path
                .strip_prefix(run_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                stack.push(path);
            } else {
                zip.start_file(relative, options)?;
                zip.write_all(&fs::read(path)?)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
