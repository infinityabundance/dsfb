//! Deterministic low-latency smoothing helpers used before finite differencing.
//!
//! The smoother is intentionally simple and auditable. It attenuates high-frequency jitter before
//! drift and slew estimation without changing the raw residual export path.

use crate::engine::settings::{SmoothingMode, SmoothingSettings};
use crate::engine::types::{ResidualSample, ResidualTrajectory};
use crate::math::metrics::euclidean_norm;

/// Applies the configured deterministic preconditioning path to a scalar sequence.
#[must_use]
pub fn smooth_scalar_series(values: &[f64], settings: &SmoothingSettings) -> Vec<f64> {
    match settings.mode {
        SmoothingMode::Disabled => values.to_vec(),
        SmoothingMode::ExponentialMovingAverage => {
            if values.is_empty() {
                return Vec::new();
            }
            let alpha = settings.exponential_alpha.clamp(1.0e-6, 1.0);
            let mut smoothed = Vec::with_capacity(values.len());
            let mut state = values[0];
            smoothed.push(state);
            for value in values.iter().skip(1) {
                state = alpha * *value + (1.0 - alpha) * state;
                smoothed.push(state);
            }
            smoothed
        }
    }
}

/// Applies the configured deterministic smoother channel-wise while preserving the original
/// timing, steps, and raw residual export path.
#[must_use]
pub fn smooth_residual_trajectory(
    residual: &ResidualTrajectory,
    settings: &SmoothingSettings,
) -> ResidualTrajectory {
    if !settings.enabled() || residual.samples.is_empty() {
        return residual.clone();
    }

    let dims = residual
        .samples
        .first()
        .map(|sample| sample.values.len())
        .unwrap_or_default();
    let smoothed_channels = (0..dims)
        .map(|dimension| {
            let values = residual
                .samples
                .iter()
                .map(|sample| sample.values[dimension])
                .collect::<Vec<_>>();
            smooth_scalar_series(&values, settings)
        })
        .collect::<Vec<_>>();

    let samples = residual
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let values = (0..dims)
                .map(|dimension| smoothed_channels[dimension][index])
                .collect::<Vec<_>>();
            ResidualSample {
                step: sample.step,
                time: sample.time,
                norm: euclidean_norm(&values),
                values,
            }
        })
        .collect::<Vec<_>>();

    ResidualTrajectory {
        scenario_id: residual.scenario_id.clone(),
        channel_names: residual.channel_names.clone(),
        samples,
    }
}

#[cfg(test)]
mod tests {
    use super::{smooth_residual_trajectory, smooth_scalar_series};
    use crate::engine::settings::{SmoothingMode, SmoothingSettings};
    use crate::engine::types::{ResidualSample, ResidualTrajectory};

    #[test]
    fn disabled_smoothing_returns_input() {
        let values = vec![0.0, 1.0, 0.0];
        assert_eq!(
            smooth_scalar_series(
                &values,
                &SmoothingSettings {
                    mode: SmoothingMode::Disabled,
                    exponential_alpha: 0.25,
                }
            ),
            values
        );
    }

    #[test]
    fn exponential_smoothing_reduces_alternating_jitter() {
        let raw = vec![0.0, 1.0, -1.0, 1.0, -1.0];
        let smoothed = smooth_scalar_series(
            &raw,
            &SmoothingSettings {
                mode: SmoothingMode::ExponentialMovingAverage,
                exponential_alpha: 0.25,
            },
        );
        let raw_variation = raw
            .windows(2)
            .map(|window| (window[1] - window[0]).abs())
            .sum::<f64>();
        let smoothed_variation = smoothed
            .windows(2)
            .map(|window| (window[1] - window[0]).abs())
            .sum::<f64>();
        assert!(smoothed_variation < raw_variation);
    }

    #[test]
    fn smoothing_preserves_trajectory_shape() {
        let residual = ResidualTrajectory {
            scenario_id: "shape".to_string(),
            channel_names: vec!["x".to_string()],
            samples: vec![
                ResidualSample {
                    step: 0,
                    time: 0.0,
                    values: vec![0.0],
                    norm: 0.0,
                },
                ResidualSample {
                    step: 1,
                    time: 1.0,
                    values: vec![1.0],
                    norm: 1.0,
                },
            ],
        };
        let smoothed = smooth_residual_trajectory(
            &residual,
            &SmoothingSettings {
                mode: SmoothingMode::ExponentialMovingAverage,
                exponential_alpha: 0.5,
            },
        );
        assert_eq!(smoothed.samples.len(), residual.samples.len());
        assert_eq!(smoothed.samples[0].time, 0.0);
        assert_eq!(smoothed.samples[1].step, 1);
    }
}
