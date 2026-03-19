use crate::engine::types::{
    DriftTrajectory, ResidualTrajectory, SignSample, SignTrajectory, SlewTrajectory,
};
use crate::math::metrics::project_sign;

pub fn construct_signs(
    residual: &ResidualTrajectory,
    drift: &DriftTrajectory,
    slew: &SlewTrajectory,
) -> SignTrajectory {
    let samples = residual
        .samples
        .iter()
        .zip(&drift.samples)
        .zip(&slew.samples)
        .map(|((residual_sample, drift_sample), slew_sample)| {
            let projection = project_sign(
                &residual_sample.values,
                &drift_sample.values,
                &slew_sample.values,
            );
            SignSample {
                step: residual_sample.step,
                time: residual_sample.time,
                residual: residual_sample.values.clone(),
                drift: drift_sample.values.clone(),
                slew: slew_sample.values.clone(),
                residual_norm: residual_sample.norm,
                drift_norm: drift_sample.norm,
                slew_norm: slew_sample.norm,
                projection,
            }
        })
        .collect();

    SignTrajectory {
        scenario_id: residual.scenario_id.clone(),
        channel_names: residual.channel_names.clone(),
        samples,
    }
}
