use crate::config::PipelineConfig;
use crate::dataset::secom::SecomDataset;
use crate::error::{DsfbSemiconductorError, Result};
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DatasetSummary {
    pub run_count: usize,
    pub feature_count: usize,
    pub pass_count: usize,
    pub fail_count: usize,
    pub dataset_missing_fraction: f64,
    pub healthy_pass_runs_requested: usize,
    pub healthy_pass_runs_found: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparedDataset {
    pub feature_names: Vec<String>,
    pub labels: Vec<i8>,
    pub timestamps: Vec<NaiveDateTime>,
    pub raw_values: Vec<Vec<Option<f64>>>,
    pub healthy_pass_indices: Vec<usize>,
    pub per_feature_missing_fraction: Vec<f64>,
    pub summary: DatasetSummary,
}

pub fn prepare_secom(dataset: &SecomDataset, config: &PipelineConfig) -> Result<PreparedDataset> {
    let mut runs = dataset.runs.clone();
    runs.sort_by_key(|run| (run.timestamp, run.index));

    let run_count = runs.len();
    let feature_count = dataset.feature_names.len();
    let pass_count = runs.iter().filter(|run| run.label == -1).count();
    let fail_count = runs.iter().filter(|run| run.label == 1).count();

    let healthy_pass_indices = runs
        .iter()
        .enumerate()
        .filter_map(|(index, run)| (run.label == -1).then_some(index))
        .take(config.healthy_pass_runs)
        .collect::<Vec<_>>();
    let healthy_pass_runs_found = healthy_pass_indices.len();

    if healthy_pass_runs_found < config.minimum_healthy_observations {
        return Err(DsfbSemiconductorError::DatasetFormat(format!(
            "SECOM does not provide enough passing runs for a healthy window: found {}, need at least {}",
            healthy_pass_runs_found,
            config.minimum_healthy_observations
        )));
    }

    let raw_values = runs
        .iter()
        .map(|run| run.features.clone())
        .collect::<Vec<_>>();

    let timestamps = runs.iter().map(|run| run.timestamp).collect::<Vec<_>>();
    let labels = runs.iter().map(|run| run.label).collect::<Vec<_>>();

    let total_cells = (run_count * feature_count) as f64;
    let mut missing_cells = 0usize;
    let mut per_feature_missing = vec![0usize; feature_count];
    for row in &raw_values {
        for (feature_index, value) in row.iter().enumerate() {
            if value.is_none() {
                missing_cells += 1;
                per_feature_missing[feature_index] += 1;
            }
        }
    }

    let per_feature_missing_fraction = per_feature_missing
        .into_iter()
        .map(|missing| missing as f64 / run_count as f64)
        .collect::<Vec<_>>();

    Ok(PreparedDataset {
        feature_names: dataset.feature_names.clone(),
        labels,
        timestamps,
        raw_values,
        healthy_pass_indices,
        per_feature_missing_fraction,
        summary: DatasetSummary {
            run_count,
            feature_count,
            pass_count,
            fail_count,
            dataset_missing_fraction: if total_cells > 0.0 {
                missing_cells as f64 / total_cells
            } else {
                0.0
            },
            healthy_pass_runs_requested: config.healthy_pass_runs,
            healthy_pass_runs_found,
        },
    })
}
