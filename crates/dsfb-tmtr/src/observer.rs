use serde::Serialize;

use crate::scenario::ScenarioDefinition;
use crate::trust::{trust_from_envelope, update_envelope};

#[derive(Debug, Clone)]
pub struct ObserverSpec {
    pub level: usize,
    pub name: String,
    pub gain: f64,
    pub trust_ceiling: f64,
    pub trust_beta: f64,
    pub envelope_decay: f64,
    pub drift_scale: f64,
    pub measurement_bias_scale: f64,
    pub availability_penalty: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObserverSeries {
    pub level: usize,
    pub name: String,
    pub prediction: Vec<f64>,
    pub estimate: Vec<f64>,
    pub measurement: Vec<Option<f64>>,
    pub innovation: Vec<f64>,
    pub residual: Vec<f64>,
    pub trust: Vec<f64>,
    pub envelope: Vec<f64>,
    pub available: Vec<bool>,
}

impl ObserverSeries {
    pub fn correction_driver(&self, step: usize) -> f64 {
        self.innovation[step] + 0.35 * self.residual[step]
    }

    pub fn recompute_after_estimate_update(&mut self, truth: &[f64], spec: &ObserverSpec) {
        let mut previous_envelope = 0.0;
        for step in 0..self.estimate.len() {
            self.residual[step] = truth[step] - self.estimate[step];
            let penalty = if self.available[step] {
                0.0
            } else {
                spec.availability_penalty
            };
            let next_envelope = update_envelope(
                previous_envelope,
                self.residual[step],
                penalty,
                spec.envelope_decay,
            );
            self.envelope[step] = next_envelope;
            let mut trust = trust_from_envelope(next_envelope, spec.trust_beta, spec.trust_ceiling);
            if spec.level == 3 && !self.available[step] {
                trust = trust.max(spec.trust_ceiling * 0.72);
            }
            self.trust[step] = trust;
            previous_envelope = next_envelope;
        }
    }
}

pub fn build_specs(definition: &ScenarioDefinition) -> Vec<ObserverSpec> {
    (0..3)
        .map(|index| ObserverSpec {
            level: index + 1,
            name: ScenarioDefinition::level_name(index + 1).to_string(),
            gain: definition.level_gains[index],
            trust_ceiling: definition.trust_ceilings[index],
            trust_beta: definition.trust_betas[index],
            envelope_decay: definition.envelope_decay[index],
            drift_scale: definition.drift_scales[index],
            measurement_bias_scale: definition.measurement_bias_scales[index],
            availability_penalty: definition.availability_penalties[index],
        })
        .collect()
}

pub fn simulate_observers(
    definition: &ScenarioDefinition,
    truth: &[f64],
) -> (Vec<ObserverSpec>, Vec<ObserverSeries>) {
    let specs = build_specs(definition);
    let series = specs
        .iter()
        .map(|spec| simulate_single_observer(spec, definition, truth))
        .collect();
    (specs, series)
}

fn simulate_single_observer(
    spec: &ObserverSpec,
    definition: &ScenarioDefinition,
    truth: &[f64],
) -> ObserverSeries {
    let n = truth.len();
    let mut prediction = vec![0.0; n];
    let mut estimate = vec![0.0; n];
    let mut measurement = vec![None; n];
    let mut innovation = vec![0.0; n];
    let mut residual = vec![0.0; n];
    let mut trust = vec![0.0; n];
    let mut envelope = vec![0.0; n];
    let mut available = vec![false; n];
    let mut previous_envelope = 0.0;

    for step in 0..n {
        let u = step as f64;
        let degrade = definition.degradation_factor(step);
        let is_available = if spec.level == 3 {
            definition.l3_available(step)
        } else {
            true
        };
        available[step] = is_available;

        let base_measurement_bias = spec.measurement_bias_scale
            * ((0.031 * u + spec.level as f64 * 0.6).sin()
                + 0.45 * (0.089 * u + spec.level as f64).cos());
        let degrade_bias = match spec.level {
            1 => degrade * (0.28 + spec.drift_scale * 2.6 + 0.06 * (0.15 * u).sin()),
            2 => degrade * (0.12 + spec.drift_scale * 1.8 + 0.04 * (0.11 * u).cos()),
            _ => 0.0,
        };
        let measured_value = truth[step] + base_measurement_bias + degrade_bias;
        if is_available {
            measurement[step] = Some(measured_value);
        }

        prediction[step] = if step == 0 {
            truth[0] + spec.level as f64 * 0.04
        } else {
            let previous_velocity = if step > 1 {
                estimate[step - 1] - estimate[step - 2]
            } else {
                0.0
            };
            let drift_term = spec.drift_scale
                * (0.014 + 0.005 * (0.071 * u + spec.level as f64).sin())
                * (1.0 + 1.9 * degrade);
            estimate[step - 1] + 0.89 * previous_velocity + drift_term
        };

        innovation[step] = if is_available {
            measured_value - prediction[step]
        } else {
            0.0
        };
        estimate[step] = if is_available {
            prediction[step] + spec.gain * innovation[step]
        } else {
            prediction[step]
        };
        residual[step] = truth[step] - estimate[step];

        let penalty = if is_available {
            0.0
        } else {
            spec.availability_penalty
        };
        let next_envelope = update_envelope(
            previous_envelope,
            residual[step],
            penalty,
            spec.envelope_decay,
        );
        envelope[step] = next_envelope;
        let mut trust_value =
            trust_from_envelope(next_envelope, spec.trust_beta, spec.trust_ceiling);
        if spec.level == 3 && !is_available {
            trust_value = trust_value.max(spec.trust_ceiling * 0.72);
        }
        trust[step] = trust_value;
        previous_envelope = next_envelope;
    }

    ObserverSeries {
        level: spec.level,
        name: spec.name.clone(),
        prediction,
        estimate,
        measurement,
        innovation,
        residual,
        trust,
        envelope,
        available,
    }
}
