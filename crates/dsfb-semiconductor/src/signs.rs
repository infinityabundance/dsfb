use crate::config::PipelineConfig;
use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use crate::residual::ResidualSet;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureSigns {
    pub feature_index: usize,
    pub feature_name: String,
    pub drift: Vec<f64>,
    pub slew: Vec<f64>,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignSet {
    pub traces: Vec<FeatureSigns>,
}

pub fn compute_signs(
    dataset: &PreparedDataset,
    nominal: &NominalModel,
    residuals: &ResidualSet,
    config: &PipelineConfig,
) -> SignSet {
    let mut traces = Vec::with_capacity(residuals.traces.len());

    for residual_trace in &residuals.traces {
        let feature = &nominal.features[residual_trace.feature_index];
        let drift = compute_drift(&residual_trace.norms, config.drift_window);
        let slew = compute_slew(&drift);

        let healthy_drift = dataset
            .healthy_pass_indices
            .iter()
            .filter_map(|&idx| drift.get(idx).copied())
            .collect::<Vec<_>>();
        let healthy_slew = dataset
            .healthy_pass_indices
            .iter()
            .filter_map(|&idx| slew.get(idx).copied())
            .collect::<Vec<_>>();
        let drift_threshold = if feature.analyzable {
            config.drift_sigma_multiplier
                * sample_std(&healthy_drift).unwrap_or(config.epsilon).max(config.epsilon)
        } else {
            0.0
        };
        let slew_threshold = if feature.analyzable {
            config.slew_sigma_multiplier
                * sample_std(&healthy_slew).unwrap_or(config.epsilon).max(config.epsilon)
        } else {
            0.0
        };

        traces.push(FeatureSigns {
            feature_index: residual_trace.feature_index,
            feature_name: residual_trace.feature_name.clone(),
            drift,
            slew,
            drift_threshold,
            slew_threshold,
        });
    }

    SignSet { traces }
}

pub fn compute_drift(values: &[f64], window: usize) -> Vec<f64> {
    let mut drift = vec![0.0; values.len()];
    for index in window..values.len() {
        drift[index] = (values[index] - values[index - window]) / window as f64;
    }
    drift
}

pub fn compute_slew(drift: &[f64]) -> Vec<f64> {
    let mut slew = vec![0.0; drift.len()];
    for index in 1..drift.len() {
        slew[index] = drift[index] - drift[index - 1];
    }
    slew
}

fn sample_std(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
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
    fn drift_matches_window_difference() {
        let drift = compute_drift(&[0.0, 1.0, 2.0, 3.0, 4.0], 2);
        assert_eq!(drift, vec![0.0, 0.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn slew_is_difference_of_drift() {
        let slew = compute_slew(&[0.0, 0.5, 1.0, 1.0]);
        assert_eq!(slew, vec![0.0, 0.5, 0.5, 0.0]);
    }
}
