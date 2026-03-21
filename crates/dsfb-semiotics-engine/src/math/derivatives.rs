use crate::engine::types::{
    DriftSample, DriftTrajectory, ResidualTrajectory, SlewSample, SlewTrajectory,
};
use crate::math::metrics::{euclidean_norm, scalar_derivative};

// TRACE:DEFINITION:DEF-DRIFT:Finite-difference drift:Implements channel-wise first derivative of the residual trajectory.
pub fn compute_drift_trajectory(
    residual: &ResidualTrajectory,
    _dt: f64,
    scenario_id: &str,
) -> DriftTrajectory {
    let count = residual.samples.len();
    let dims = residual
        .samples
        .first()
        .map(|sample| sample.values.len())
        .unwrap_or_default();
    let times = residual
        .samples
        .iter()
        .map(|sample| sample.time)
        .collect::<Vec<_>>();
    let channel_derivatives = (0..dims)
        .map(|dimension| {
            let values = residual
                .samples
                .iter()
                .map(|sample| sample.values[dimension])
                .collect::<Vec<_>>();
            scalar_derivative(&values, &times)
        })
        .collect::<Vec<_>>();

    let mut samples = Vec::with_capacity(count);
    for (index, sample) in residual.samples.iter().enumerate() {
        let values = (0..dims)
            .map(|dimension| channel_derivatives[dimension][index])
            .collect::<Vec<_>>();
        samples.push(DriftSample {
            step: sample.step,
            time: sample.time,
            values: values.clone(),
            norm: euclidean_norm(&values),
        });
    }

    DriftTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: residual.channel_names.clone(),
        samples,
    }
}

// TRACE:DEFINITION:DEF-SLEW:Nonuniform finite-difference slew:Implements channel-wise second derivative over nonuniform sampled times.
pub fn compute_slew_trajectory(
    residual: &ResidualTrajectory,
    _dt: f64,
    scenario_id: &str,
) -> SlewTrajectory {
    let count = residual.samples.len();
    let dims = residual
        .samples
        .first()
        .map(|sample| sample.values.len())
        .unwrap_or_default();
    let times = residual
        .samples
        .iter()
        .map(|sample| sample.time)
        .collect::<Vec<_>>();
    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let values = if count < 3 {
            vec![0.0; dims]
        } else {
            let (left, center, right) = if index == 0 {
                (0, 1, 2)
            } else if index + 1 == count {
                (count - 3, count - 2, count - 1)
            } else {
                (index - 1, index, index + 1)
            };
            (0..dims)
                .map(|dimension| {
                    second_derivative_nonuniform(
                        residual.samples[left].values[dimension],
                        times[left],
                        residual.samples[center].values[dimension],
                        times[center],
                        residual.samples[right].values[dimension],
                        times[right],
                    )
                })
                .collect::<Vec<_>>()
        };
        samples.push(SlewSample {
            step: residual.samples[index].step,
            time: residual.samples[index].time,
            values: values.clone(),
            norm: euclidean_norm(&values),
        });
    }

    SlewTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: residual.channel_names.clone(),
        samples,
    }
}

// TRACE:ALGORITHM:ALG-NONUNIFORM-SECOND-DERIVATIVE:Nonuniform three-point curvature estimate:Used by slew construction near boundaries and interior samples.
fn second_derivative_nonuniform(
    left_value: f64,
    left_time: f64,
    center_value: f64,
    center_time: f64,
    right_value: f64,
    right_time: f64,
) -> f64 {
    let left_term = (left_time - center_time) * (left_time - right_time);
    let center_term = (center_time - left_time) * (center_time - right_time);
    let right_term = (right_time - left_time) * (right_time - center_time);
    if left_term.abs() <= 1.0e-12 || center_term.abs() <= 1.0e-12 || right_term.abs() <= 1.0e-12 {
        0.0
    } else {
        2.0 * (left_value / left_term + center_value / center_term + right_value / right_term)
    }
}
