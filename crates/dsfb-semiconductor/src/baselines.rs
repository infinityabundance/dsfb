use crate::config::PipelineConfig;
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EwmaFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub ewma: Vec<f64>,
    pub healthy_mean: f64,
    pub healthy_std: f64,
    pub threshold: f64,
    pub alarm: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaselineSet {
    pub ewma: Vec<EwmaFeatureTrace>,
}

pub fn compute_baselines(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    config: &PipelineConfig,
) -> BaselineSet {
    let ewma = residuals
        .traces
        .iter()
        .zip(&nominal.features)
        .map(|(trace, feature)| {
            let ewma = ewma_series(&trace.norms, config.ewma_alpha);
            let healthy_ewma = dataset
                .healthy_pass_indices
                .iter()
                .filter_map(|&idx| ewma.get(idx).copied())
                .collect::<Vec<_>>();
            let healthy_mean = mean(&healthy_ewma).unwrap_or(0.0);
            let healthy_std = sample_std(&healthy_ewma, healthy_mean).unwrap_or(0.0);
            let threshold = if feature.analyzable {
                healthy_mean + config.ewma_sigma_multiplier * healthy_std.max(config.epsilon)
            } else {
                0.0
            };
            let alarm = ewma
                .iter()
                .map(|value| feature.analyzable && *value > threshold)
                .collect::<Vec<_>>();

            EwmaFeatureTrace {
                feature_index: trace.feature_index,
                feature_name: trace.feature_name.clone(),
                ewma,
                healthy_mean,
                healthy_std,
                threshold,
                alarm,
            }
        })
        .collect::<Vec<_>>();

    BaselineSet { ewma }
}

pub fn ewma_series(values: &[f64], alpha: f64) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(values.len());
    let mut state = values[0];
    out.push(state);
    for value in &values[1..] {
        state = alpha * *value + (1.0 - alpha) * state;
        out.push(state);
    }
    out
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
    fn ewma_series_matches_recursive_definition() {
        let ewma = ewma_series(&[1.0, 3.0, 5.0], 0.5);
        assert_eq!(ewma, vec![1.0, 2.0, 3.5]);
    }
}
