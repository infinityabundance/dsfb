//! Residual construction: r(t) = x_obs(t) - x_model(t).
//!
//! The nominal model prediction is the per-sample mean waveform estimated
//! from the nominal baseline window. The residual captures departure from
//! that baseline behavior.

use crate::baseline::NominalBaseline;

/// Compute the residual signal for one window.
///
/// r(t) = x_obs(t) - x_model(t)
///
/// If the observation is longer than the model waveform, excess samples
/// use the overall mean as the model value (conservative extension).
pub fn compute_residual(observation: &[f64], baseline: &NominalBaseline) -> Vec<f64> {
    let model = &baseline.mean_waveform;
    let overall_mean = if model.is_empty() {
        0.0
    } else {
        crate::baseline::mean(model)
    };

    observation
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let m = if i < model.len() {
                model[i]
            } else {
                overall_mean
            };
            x - m
        })
        .collect()
}
