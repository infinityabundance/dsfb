use nalgebra::DMatrix;
use serde::Serialize;

use crate::math::spectrum::sign_ambiguous_distance;

#[derive(Debug, Clone, Serialize)]
pub struct ResidualStack {
    pub predicted: Vec<f64>,
    pub observed: Vec<f64>,
    pub residuals: Vec<f64>,
    pub drifts: Vec<f64>,
    pub slews: Vec<f64>,
    pub mode_shape_residuals: Vec<f64>,
    pub scalar_predicted: f64,
    pub scalar_observed: f64,
    pub scalar_residual: f64,
    pub scalar_drift: f64,
    pub scalar_slew: f64,
    pub stack_norm: f64,
    pub mode_shape_norm: f64,
    pub combined_score: f64,
}

impl ResidualStack {
    pub fn empty(monitored_modes: usize) -> Self {
        Self {
            predicted: vec![0.0; monitored_modes],
            observed: vec![0.0; monitored_modes],
            residuals: vec![0.0; monitored_modes],
            drifts: vec![0.0; monitored_modes],
            slews: vec![0.0; monitored_modes],
            mode_shape_residuals: vec![0.0; monitored_modes],
            scalar_predicted: 0.0,
            scalar_observed: 0.0,
            scalar_residual: 0.0,
            scalar_drift: 0.0,
            scalar_slew: 0.0,
            stack_norm: 0.0,
            mode_shape_norm: 0.0,
            combined_score: 0.0,
        }
    }
}

pub fn compute_residual_stack(
    observed: &[f64],
    predicted: &[f64],
    previous_residuals: Option<&[f64]>,
    previous_drifts: Option<&[f64]>,
    current_vectors: &DMatrix<f64>,
    previous_vectors: Option<&DMatrix<f64>>,
    dt: f64,
    include_mode_shapes: bool,
) -> ResidualStack {
    let residuals = observed
        .iter()
        .zip(predicted.iter())
        .map(|(obs, pred)| obs - pred)
        .collect::<Vec<_>>();
    let drifts = residuals
        .iter()
        .enumerate()
        .map(|(index, residual)| {
            let previous = previous_residuals
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or(*residual);
            (residual - previous) / dt
        })
        .collect::<Vec<_>>();
    let slews = drifts
        .iter()
        .enumerate()
        .map(|(index, drift)| {
            let previous = previous_drifts
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or(*drift);
            (drift - previous) / dt
        })
        .collect::<Vec<_>>();
    let mode_shape_residuals = if include_mode_shapes {
        compute_mode_shape_residuals(current_vectors, previous_vectors, observed.len())
    } else {
        vec![0.0; observed.len()]
    };
    let stack_norm = euclidean_norm(&residuals);
    let mode_shape_norm = euclidean_norm(&mode_shape_residuals);
    let combined_score = stack_norm + 0.5 * mode_shape_norm;

    ResidualStack::from_components(
        predicted,
        observed,
        residuals,
        drifts,
        slews,
        mode_shape_residuals,
        stack_norm,
        mode_shape_norm,
        combined_score,
    )
}

impl ResidualStack {
    fn from_components(
        predicted: &[f64],
        observed: &[f64],
        residuals: Vec<f64>,
        drifts: Vec<f64>,
        slews: Vec<f64>,
        mode_shape_residuals: Vec<f64>,
        stack_norm: f64,
        mode_shape_norm: f64,
        combined_score: f64,
    ) -> Self {
        Self {
            predicted: predicted.to_vec(),
            observed: observed.to_vec(),
            scalar_predicted: predicted.first().copied().unwrap_or(0.0),
            scalar_observed: observed.first().copied().unwrap_or(0.0),
            scalar_residual: residuals.first().copied().unwrap_or(0.0),
            scalar_drift: drifts.first().copied().unwrap_or(0.0),
            scalar_slew: slews.first().copied().unwrap_or(0.0),
            residuals,
            drifts,
            slews,
            mode_shape_residuals,
            stack_norm,
            mode_shape_norm,
            combined_score,
        }
    }
}

fn compute_mode_shape_residuals(
    current_vectors: &DMatrix<f64>,
    previous_vectors: Option<&DMatrix<f64>>,
    monitored_modes: usize,
) -> Vec<f64> {
    (0..monitored_modes)
        .map(|offset| match previous_vectors {
            Some(previous)
                if current_vectors.ncols() > offset + 1 && previous.ncols() > offset + 1 =>
            {
                sign_ambiguous_distance(
                    &current_vectors.column(offset + 1).into_owned(),
                    &previous.column(offset + 1).into_owned(),
                )
            }
            _ => 0.0,
        })
        .collect()
}

fn euclidean_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}
