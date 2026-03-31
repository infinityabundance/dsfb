use crate::config::PipelineConfig;
use crate::preprocessing::PreparedDataset;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NominalFeature {
    pub feature_index: usize,
    pub feature_name: String,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub rho: f64,
    pub healthy_observations: usize,
    pub analyzable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NominalModel {
    pub features: Vec<NominalFeature>,
}

pub fn build_nominal_model(dataset: &PreparedDataset, config: &PipelineConfig) -> NominalModel {
    let feature_count = dataset.feature_names.len();
    let mut features = Vec::with_capacity(feature_count);

    for feature_index in 0..feature_count {
        let healthy_values = dataset
            .healthy_pass_indices
            .iter()
            .filter_map(|&run_index| dataset.raw_values[run_index][feature_index])
            .collect::<Vec<_>>();
        let healthy_observations = healthy_values.len();
        let healthy_mean = mean(&healthy_values).unwrap_or(0.0);
        let healthy_std = sample_std(&healthy_values, healthy_mean).unwrap_or(0.0);
        let analyzable =
            healthy_observations >= config.minimum_healthy_observations && healthy_std > config.epsilon;
        let rho = if analyzable {
            config.envelope_sigma * healthy_std
        } else {
            0.0
        };

        features.push(NominalFeature {
            feature_index,
            feature_name: dataset.feature_names[feature_index].clone(),
            healthy_mean,
            healthy_std,
            rho,
            healthy_observations,
            analyzable,
        });
    }

    NominalModel { features }
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn sample_std(values: &[f64], mean: f64) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let variance = values
        .iter()
        .map(|value| {
            let centered = *value - mean;
            centered * centered
        })
        .sum::<f64>()
        / (values.len() as f64 - 1.0);
    Some(variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_std_is_zero_for_constant_series() {
        let std = sample_std(&[2.0, 2.0, 2.0], 2.0).unwrap();
        assert_eq!(std, 0.0);
    }
}
