use nalgebra::DMatrix;

use crate::config::PredictorKind;

#[derive(Debug, Clone)]
pub struct PredictorState {
    kind: PredictorKind,
    smoothing: f64,
    previous: Option<Vec<f64>>,
    previous_previous: Option<Vec<f64>>,
    previous_vectors: Option<DMatrix<f64>>,
}

impl PredictorState {
    pub fn new(kind: PredictorKind) -> Self {
        Self {
            kind,
            smoothing: 0.82,
            previous: None,
            previous_previous: None,
            previous_vectors: None,
        }
    }

    pub fn predict_values(&self, monitored_values: usize) -> Vec<f64> {
        let values = match (&self.previous_previous, &self.previous, self.kind) {
            (_, Some(previous), PredictorKind::ZeroOrderHold) => previous.clone(),
            (Some(previous_previous), Some(previous), PredictorKind::FirstOrder) => previous
                .iter()
                .zip(previous_previous.iter())
                .map(|(curr, prev)| curr + (curr - prev))
                .collect(),
            (Some(previous_previous), Some(previous), PredictorKind::SmoothCorrective) => previous
                .iter()
                .zip(previous_previous.iter())
                .enumerate()
                .map(|(index, (current, previous_value))| {
                    let previous_previous_value = previous_previous[index];
                    let delta = current - previous_value;
                    let previous_delta = previous_value - previous_previous_value;
                    smooth_corrective_prediction(
                        *current,
                        *previous_value,
                        previous_delta,
                        delta,
                        self.smoothing,
                    )
                })
                .collect(),
            (_, Some(previous), PredictorKind::FirstOrder | PredictorKind::SmoothCorrective) => {
                previous.clone()
            }
            _ => vec![0.0; monitored_values],
        };
        enforce_spectral_order(values, monitored_values)
    }

    pub fn previous_vectors(&self) -> Option<&DMatrix<f64>> {
        self.previous_vectors.as_ref()
    }

    pub fn update(&mut self, observed: &[f64], vectors: DMatrix<f64>) {
        self.previous_previous = self.previous.take();
        self.previous = Some(observed.to_vec());
        self.previous_vectors = Some(vectors);
    }
}

fn enforce_spectral_order(mut values: Vec<f64>, monitored_values: usize) -> Vec<f64> {
    values.resize(monitored_values, 0.0);
    for value in &mut values {
        if !value.is_finite() || *value < 0.0 {
            *value = 0.0;
        }
    }
    for index in 1..values.len() {
        if values[index] < values[index - 1] {
            values[index] = values[index - 1];
        }
    }
    values
}

fn smooth_corrective_prediction(
    current: f64,
    previous: f64,
    previous_delta: f64,
    delta: f64,
    smoothing: f64,
) -> f64 {
    let local_scale = current.abs().max(previous.abs()).max(0.05);
    let reversal = previous_delta * delta < 0.0;
    let acceleration = delta - previous_delta;
    let capped_delta = delta.clamp(-0.35 * local_scale, 0.35 * local_scale);
    let trend_step = if reversal {
        0.0
    } else if acceleration.abs() > 0.30 * local_scale {
        0.45 * capped_delta
    } else if delta.abs() > 0.25 * local_scale {
        0.60 * capped_delta
    } else {
        capped_delta
    };
    let first_order = current + trend_step;
    smoothing * current + (1.0 - smoothing) * first_order
}
