use crate::engine::types::{
    ObservedTrajectory, PredictedTrajectory, ResidualSample, ResidualTrajectory,
};
use crate::math::metrics::euclidean_norm;

// TRACE:DEFINITION:DEF-RESIDUAL:Residual construction:Implements sample-wise observed minus predicted residual formation.
pub fn compute_residual_trajectory(
    observed: &ObservedTrajectory,
    predicted: &PredictedTrajectory,
    scenario_id: &str,
) -> ResidualTrajectory {
    let samples = observed
        .samples
        .iter()
        .zip(&predicted.samples)
        .map(|(obs, pred)| {
            let values = obs
                .values
                .iter()
                .zip(&pred.values)
                .map(|(y, y_hat)| y - y_hat)
                .collect::<Vec<_>>();
            ResidualSample {
                step: obs.step,
                time: obs.time,
                values: values.clone(),
                norm: euclidean_norm(&values),
            }
        })
        .collect();

    ResidualTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: observed.channel_names.clone(),
        samples,
    }
}
