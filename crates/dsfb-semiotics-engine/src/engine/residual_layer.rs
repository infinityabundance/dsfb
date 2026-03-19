use crate::engine::types::{ObservedTrajectory, PredictedTrajectory, ResidualTrajectory};
use crate::math::residual::compute_residual_trajectory;

pub fn extract_residuals(
    observed: &ObservedTrajectory,
    predicted: &PredictedTrajectory,
    scenario_id: &str,
) -> ResidualTrajectory {
    compute_residual_trajectory(observed, predicted, scenario_id)
}
