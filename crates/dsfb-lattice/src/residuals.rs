use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use crate::utils::DeterministicRng;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub steps: usize,
    pub dt: f64,
    pub damping: f64,
    pub observed_modes: usize,
}

#[derive(Clone, Debug)]
pub struct SimulationOutput {
    pub states: Vec<DVector<f64>>,
    pub observations: Vec<DVector<f64>>,
}

#[derive(Clone, Debug)]
pub struct TimeSeriesBundle {
    pub predicted: Vec<DVector<f64>>,
    pub measured: Vec<DVector<f64>>,
    pub residuals: Vec<DVector<f64>>,
    pub drifts: Vec<DVector<f64>>,
    pub slews: Vec<DVector<f64>>,
    pub predicted_norms: Vec<f64>,
    pub measured_norms: Vec<f64>,
    pub residual_norms: Vec<f64>,
    pub normalized_residual_norms: Vec<f64>,
    pub drift_norms: Vec<f64>,
    pub slew_norms: Vec<f64>,
    pub residual_energy_ratio: f64,
}

pub fn simulate_response(
    dynamical: &DMatrix<f64>,
    nominal_modes: &DMatrix<f64>,
    config: &SimulationConfig,
    forcing_variant: usize,
) -> SimulationOutput {
    let sites = dynamical.nrows();
    let observed = config.observed_modes.min(sites);
    let mode_basis = nominal_modes.columns(0, observed).into_owned();

    let mut displacement = DVector::<f64>::from_fn(sites, |row, _| {
        0.03 * ((row as f64 + 1.0) * 0.37 + forcing_variant as f64 * 0.02).sin()
    });
    let mut velocity = DVector::<f64>::from_fn(sites, |row, _| {
        0.015 * ((row as f64 + 1.0) * 0.21 + forcing_variant as f64 * 0.01).cos()
    });

    let mut states = Vec::with_capacity(config.steps);
    let mut observations = Vec::with_capacity(config.steps);
    for step in 0..config.steps {
        states.push(displacement.clone());
        observations.push(mode_basis.transpose() * &displacement);

        let force = forcing_vector(step, config.dt, sites, forcing_variant);
        let acceleration = force - dynamical * &displacement - config.damping * &velocity;
        velocity += config.dt * acceleration;
        displacement += config.dt * &velocity;
    }

    SimulationOutput { states, observations }
}

pub fn build_time_series(
    predicted: &[DVector<f64>],
    measured: &[DVector<f64>],
    normalization_epsilon: f64,
) -> TimeSeriesBundle {
    let mut residuals = Vec::with_capacity(predicted.len());
    let mut drifts = Vec::with_capacity(predicted.len());
    let mut slews = Vec::with_capacity(predicted.len());
    let mut predicted_norms = Vec::with_capacity(predicted.len());
    let mut measured_norms = Vec::with_capacity(predicted.len());
    let mut residual_norms = Vec::with_capacity(predicted.len());
    let mut normalized_residual_norms = Vec::with_capacity(predicted.len());
    let mut drift_norms = Vec::with_capacity(predicted.len());
    let mut slew_norms = Vec::with_capacity(predicted.len());
    let mut predicted_energy = 0.0;
    let mut residual_energy = 0.0;

    let channels = predicted.first().map(|vector| vector.len()).unwrap_or(0);
    let zero = DVector::<f64>::zeros(channels);
    let mut previous_residual = zero.clone();
    let mut previous_drift = zero.clone();

    for (index, (predicted_step, measured_step)) in predicted.iter().zip(measured.iter()).enumerate() {
        let residual = measured_step - predicted_step;
        let drift = if index == 0 {
            zero.clone()
        } else {
            &residual - &previous_residual
        };
        let slew = if index <= 1 {
            zero.clone()
        } else {
            &drift - &previous_drift
        };

        let predicted_norm = predicted_step.norm();
        let measured_norm = measured_step.norm();
        let residual_norm = residual.norm();
        let normalized_residual_norm = residual_norm / (predicted_norm + normalization_epsilon);
        predicted_norms.push(predicted_norm);
        measured_norms.push(measured_norm);
        residual_norms.push(residual_norm);
        normalized_residual_norms.push(normalized_residual_norm);
        drift_norms.push(drift.norm());
        slew_norms.push(slew.norm());
        predicted_energy += predicted_norm.powi(2);
        residual_energy += residual_norm.powi(2);

        residuals.push(residual.clone());
        drifts.push(drift.clone());
        slews.push(slew.clone());
        previous_residual = residual;
        previous_drift = drift;
    }

    let residual_energy_ratio = residual_energy / (predicted_energy + normalization_epsilon);

    TimeSeriesBundle {
        predicted: predicted.to_vec(),
        measured: measured.to_vec(),
        residuals,
        drifts,
        slews,
        predicted_norms,
        measured_norms,
        residual_norms,
        normalized_residual_norms,
        drift_norms,
        slew_norms,
        residual_energy_ratio,
    }
}

pub fn add_observation_noise(
    observations: &[DVector<f64>],
    noise_std: f64,
    seed: u64,
) -> Vec<DVector<f64>> {
    if noise_std <= 0.0 {
        return observations.to_vec();
    }

    let mut rng = DeterministicRng::new(seed);
    observations
        .iter()
        .map(|observation| {
            let mut noisy = observation.clone();
            for value in noisy.iter_mut() {
                *value += noise_std * rng.next_gaussian();
            }
            noisy
        })
        .collect()
}

pub fn covariance_matrix(samples: &[DVector<f64>]) -> DMatrix<f64> {
    if samples.is_empty() {
        return DMatrix::<f64>::zeros(0, 0);
    }

    let channels = samples[0].len();
    let count = samples.len() as f64;
    let mut mean = DVector::<f64>::zeros(channels);
    for sample in samples {
        mean += sample;
    }
    mean /= count.max(1.0);

    let mut covariance = DMatrix::<f64>::zeros(channels, channels);
    for sample in samples {
        let centered = sample - &mean;
        covariance += &centered * centered.transpose();
    }

    if samples.len() > 1 {
        covariance / (samples.len() as f64 - 1.0)
    } else {
        covariance
    }
}

fn forcing_vector(step: usize, dt: f64, sites: usize, forcing_variant: usize) -> DVector<f64> {
    let time = step as f64 * dt;
    let center = (sites as f64 - 1.0) / 2.0;
    let amplitude_scale = 1.0 + forcing_variant as f64 * 0.0015;
    let phase_shift = forcing_variant as f64 * 0.015;

    DVector::<f64>::from_fn(sites, |row, _| {
        let local = row as f64 - center;
        let envelope = (-0.5 * (local / 2.3).powi(2)).exp();
        amplitude_scale
            * (0.42 * (0.62 * time + 0.18 * row as f64 + phase_shift).sin()
                + 0.18 * (0.17 * time * (row as f64 + 1.0) + 0.3 * phase_shift).cos())
            + 0.22 * envelope * (1.1 * time + 0.015 * forcing_variant as f64).sin()
    })
}
