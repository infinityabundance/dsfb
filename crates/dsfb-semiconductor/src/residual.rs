use crate::nominal::NominalModel;
use crate::preprocessing::PreparedDataset;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ResidualFeatureTrace {
    pub feature_index: usize,
    pub feature_name: String,
    pub imputed_values: Vec<f64>,
    pub residuals: Vec<f64>,
    pub norms: Vec<f64>,
    pub threshold_alarm: Vec<bool>,
    pub is_imputed: Vec<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidualSet {
    pub traces: Vec<ResidualFeatureTrace>,
}

pub fn compute_residuals(dataset: &PreparedDataset, nominal: &NominalModel) -> ResidualSet {
    let run_count = dataset.raw_values.len();
    let feature_count = dataset.feature_names.len();
    let mut traces = Vec::with_capacity(feature_count);

    for feature_index in 0..feature_count {
        let feature = &nominal.features[feature_index];
        let mut imputed_values = Vec::with_capacity(run_count);
        let mut residuals = Vec::with_capacity(run_count);
        let mut norms = Vec::with_capacity(run_count);
        let mut threshold_alarm = Vec::with_capacity(run_count);
        let mut is_imputed = Vec::with_capacity(run_count);

        for row in &dataset.raw_values {
            let is_missing = row[feature_index].is_none();
            let value = row[feature_index].unwrap_or(feature.healthy_mean);
            let residual = value - feature.healthy_mean;
            let norm = residual.abs();
            imputed_values.push(value);
            residuals.push(residual);
            norms.push(norm);
            threshold_alarm.push(feature.analyzable && norm > feature.rho);
            is_imputed.push(is_missing);
        }

        traces.push(ResidualFeatureTrace {
            feature_index,
            feature_name: feature.feature_name.clone(),
            imputed_values,
            residuals,
            norms,
            threshold_alarm,
            is_imputed,
        });
    }

    ResidualSet { traces }
}
