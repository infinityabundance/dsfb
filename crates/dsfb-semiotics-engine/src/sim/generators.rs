use crate::engine::types::{ObservedTrajectory, PredictedTrajectory, ScenarioRecord, VectorSample};
use crate::sim::coordinated::grouped_residual;
use crate::sim::degradation::{
    curvature_residual, gradual_residual, inward_residual, regime_switched_residual,
};
use crate::sim::disturbances::{
    abrupt_event_residual, deterministic_noise, nominal_residual, oscillatory_residual,
};
use crate::sim::scenarios::{ScenarioDefinition, ScenarioKind};

#[derive(Clone, Debug)]
pub struct ScenarioSynthesis {
    pub record: ScenarioRecord,
    pub observed: ObservedTrajectory,
    pub predicted: PredictedTrajectory,
}

pub fn synthesize(
    definition: &ScenarioDefinition,
    steps: usize,
    dt: f64,
    seed: u64,
) -> ScenarioSynthesis {
    let samples = (0..steps)
        .map(|step| {
            let time = step as f64 * dt;
            let predicted = (0..definition.channels.len())
                .map(|channel| prediction_signal(time, channel))
                .collect::<Vec<_>>();
            let residual = (0..definition.channels.len())
                .map(|channel| residual_signal(definition.kind, step, channel, seed))
                .collect::<Vec<_>>();
            let observed = predicted
                .iter()
                .zip(&residual)
                .map(|(base, residual)| base + residual)
                .collect::<Vec<_>>();

            (
                VectorSample {
                    step,
                    time,
                    values: observed,
                },
                VectorSample {
                    step,
                    time,
                    values: predicted,
                },
            )
        })
        .collect::<Vec<_>>();

    let observed = ObservedTrajectory {
        scenario_id: definition.record.id.clone(),
        channel_names: definition.channels.clone(),
        samples: samples
            .iter()
            .map(|(observed, _)| observed.clone())
            .collect(),
    };
    let predicted = PredictedTrajectory {
        scenario_id: definition.record.id.clone(),
        channel_names: definition.channels.clone(),
        samples: samples
            .iter()
            .map(|(_, predicted)| predicted.clone())
            .collect(),
    };

    ScenarioSynthesis {
        record: definition.record.clone(),
        observed,
        predicted,
    }
}

fn prediction_signal(time: f64, channel: usize) -> f64 {
    0.6 * (0.035 * time + 0.45 * channel as f64).sin()
        + 0.22 * (0.011 * time + 0.3 * channel as f64).cos()
}

fn residual_signal(kind: ScenarioKind, step: usize, channel: usize, seed: u64) -> f64 {
    match kind {
        ScenarioKind::NominalStable => nominal_residual(step, channel),
        ScenarioKind::GradualDegradation => gradual_residual(step, channel),
        ScenarioKind::CurvatureOnset => curvature_residual(step, channel),
        ScenarioKind::AbruptEvent => abrupt_event_residual(step, channel),
        ScenarioKind::OscillatoryBounded => oscillatory_residual(step, channel),
        ScenarioKind::OutwardExitA => linear_exit_residual(step, channel, 0.15, 0.0042),
        ScenarioKind::OutwardExitB => linear_exit_residual(step, channel, 0.12, 0.0050),
        ScenarioKind::OutwardExitC => linear_exit_residual(step, channel, 0.18, 0.0038),
        ScenarioKind::InwardInvariance => inward_residual(step, channel),
        ScenarioKind::GroupedCorrelated => grouped_residual(step, channel),
        ScenarioKind::RegimeSwitch => regime_switched_residual(step, channel),
        ScenarioKind::NoisyStructured => {
            gradual_residual(step, channel) * 0.82 + deterministic_noise(step, channel, seed)
        }
        ScenarioKind::MagnitudeMatchedAdmissible => magnitude_matched_admissible(step, channel),
        ScenarioKind::MagnitudeMatchedDetectable => {
            linear_exit_residual(step, channel, 0.24, 0.0028)
        }
    }
}

fn linear_exit_residual(step: usize, channel: usize, base: f64, slope: f64) -> f64 {
    let t = step as f64;
    match channel {
        0 => base + slope * t,
        1 => 0.018 + 0.004 * (0.05 * t + 0.5).sin(),
        _ => 0.012 + 0.003 * (0.04 * t + 1.0).cos(),
    }
}

fn magnitude_matched_admissible(step: usize, channel: usize) -> f64 {
    let t = step as f64;
    match channel {
        0 => 0.24 + 0.09 * (0.08 * t).sin(),
        1 => 0.04 + 0.03 * (0.05 * t + 0.3).sin(),
        _ => 0.03 + 0.02 * (0.04 * t + 0.8).cos(),
    }
}
