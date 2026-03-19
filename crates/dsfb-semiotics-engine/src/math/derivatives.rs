use crate::engine::types::{
    DriftSample, DriftTrajectory, ResidualTrajectory, SlewSample, SlewTrajectory,
};
use crate::math::metrics::euclidean_norm;

pub fn compute_drift_trajectory(
    residual: &ResidualTrajectory,
    dt: f64,
    scenario_id: &str,
) -> DriftTrajectory {
    let count = residual.samples.len();
    let dims = residual
        .samples
        .first()
        .map(|sample| sample.values.len())
        .unwrap_or_default();

    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let values = match count {
            0 => Vec::new(),
            1 => vec![0.0; dims],
            _ if index == 0 => difference(
                &residual.samples[index + 1].values,
                &residual.samples[index].values,
                dt,
            ),
            _ if index + 1 == count => difference(
                &residual.samples[index].values,
                &residual.samples[index - 1].values,
                dt,
            ),
            _ => difference(
                &residual.samples[index + 1].values,
                &residual.samples[index - 1].values,
                2.0 * dt,
            ),
        };
        samples.push(DriftSample {
            step: residual.samples[index].step,
            time: residual.samples[index].time,
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

pub fn compute_slew_trajectory(
    residual: &ResidualTrajectory,
    dt: f64,
    scenario_id: &str,
) -> SlewTrajectory {
    let count = residual.samples.len();
    let dims = residual
        .samples
        .first()
        .map(|sample| sample.values.len())
        .unwrap_or_default();

    let mut samples = Vec::with_capacity(count);
    for index in 0..count {
        let values = if count < 3 {
            vec![0.0; dims]
        } else if index == 0 {
            second_difference(
                &residual.samples[2].values,
                &residual.samples[1].values,
                &residual.samples[0].values,
                dt,
            )
        } else if index + 1 == count {
            second_difference(
                &residual.samples[count - 1].values,
                &residual.samples[count - 2].values,
                &residual.samples[count - 3].values,
                dt,
            )
        } else {
            second_difference(
                &residual.samples[index + 1].values,
                &residual.samples[index].values,
                &residual.samples[index - 1].values,
                dt,
            )
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

fn difference(upper: &[f64], lower: &[f64], scale: f64) -> Vec<f64> {
    upper
        .iter()
        .zip(lower)
        .map(|(u, l)| (u - l) / scale)
        .collect()
}

fn second_difference(next: &[f64], current: &[f64], previous: &[f64], dt: f64) -> Vec<f64> {
    next.iter()
        .zip(current)
        .zip(previous)
        .map(|((n, c), p)| (n - 2.0 * c + p) / (dt * dt))
        .collect()
}
