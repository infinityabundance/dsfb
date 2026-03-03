//! Simulation harness for DSFB
//!
//! Generates synthetic data and runs comparison between different observers

use crate::observer::DsfbObserver;
use crate::params::DsfbParams;
use crate::state::DsfbState;
use crate::trust::TrustStats;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

/// True system dynamics state
#[derive(Debug, Clone)]
pub struct TrueState {
    pub phi: f64,
    pub omega: f64,
    pub alpha: f64,
}

impl TrueState {
    pub fn new(phi: f64, omega: f64, alpha: f64) -> Self {
        Self { phi, omega, alpha }
    }
}

/// Frequency-only observer (baseline without alpha state)
pub struct FreqOnlyObserver {
    phi: f64,
    omega: f64,
    k_phi: f64,
    k_omega: f64,
}

impl FreqOnlyObserver {
    pub fn new(k_phi: f64, k_omega: f64) -> Self {
        Self {
            phi: 0.0,
            omega: 0.0,
            k_phi,
            k_omega,
        }
    }

    pub fn step(&mut self, measurements: &[f64], dt: f64) -> f64 {
        // Predict (no alpha term)
        let phi_pred = self.phi + self.omega * dt;

        // Mean measurement
        let mean_meas: f64 = measurements.iter().sum::<f64>() / measurements.len() as f64;

        // Residual
        let residual = mean_meas - phi_pred;

        // Correct
        self.phi = phi_pred + self.k_phi * residual;
        self.omega += self.k_omega * residual;

        self.phi
    }
}

/// Simulation configuration
#[derive(Clone)]
pub struct SimConfig {
    pub dt: f64,
    pub steps: usize,
    pub sigma_noise: f64,
    pub sigma_alpha: f64,
    pub drift_beta: f64,
    pub impulse_start: usize,
    pub impulse_duration: usize,
    pub impulse_amplitude: f64,
    pub seed: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            dt: 0.01,
            steps: 1000,
            sigma_noise: 0.05,
            sigma_alpha: 0.01,
            drift_beta: 0.1,
            impulse_start: 300,
            impulse_duration: 100,
            impulse_amplitude: 1.0,
            seed: 42,
        }
    }
}

/// Simulation results for one time step
#[derive(Debug, Clone)]
pub struct SimStep {
    pub t: f64,
    pub phi_true: f64,
    pub y1: f64,
    pub y2: f64,
    pub phi_mean: f64,
    pub phi_freqonly: f64,
    pub phi_dsfb: f64,
    pub err_mean: f64,
    pub err_freqonly: f64,
    pub err_dsfb: f64,
    pub w2: f64,
    pub s2: f64,
}

/// Rich DSFB simulation trace for downstream consumers.
#[derive(Debug, Clone)]
pub struct SimulationTraceStep {
    pub step: usize,
    pub t: f64,
    pub phi_true: f64,
    pub measurements: Vec<f64>,
    pub phi_mean: f64,
    pub phi_freqonly: f64,
    pub dsfb_state: DsfbState,
    pub err_mean: f64,
    pub err_freqonly: f64,
    pub err_dsfb: f64,
    pub trust_stats: Vec<TrustStats>,
    pub residuals: Vec<f64>,
    pub aggregate_residual: f64,
}

/// Run the drift-impulse simulation
pub fn run_simulation(config: SimConfig, dsfb_params: DsfbParams) -> Vec<SimStep> {
    run_simulation_trace(config, dsfb_params)
        .into_iter()
        .map(|step| SimStep {
            t: step.t,
            phi_true: step.phi_true,
            y1: step.measurements.first().copied().unwrap_or_default(),
            y2: step.measurements.get(1).copied().unwrap_or_default(),
            phi_mean: step.phi_mean,
            phi_freqonly: step.phi_freqonly,
            phi_dsfb: step.dsfb_state.phi,
            err_mean: step.err_mean,
            err_freqonly: step.err_freqonly,
            err_dsfb: step.err_dsfb,
            w2: step
                .trust_stats
                .get(1)
                .map(|stats| stats.weight)
                .unwrap_or_default(),
            s2: step
                .trust_stats
                .get(1)
                .map(|stats| stats.residual_ema)
                .unwrap_or_default(),
        })
        .collect()
}

/// Run the drift-impulse simulation and capture DSFB diagnostics for every step.
pub fn run_simulation_trace(
    config: SimConfig,
    dsfb_params: DsfbParams,
) -> Vec<SimulationTraceStep> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);
    let noise_dist = Normal::new(0.0, config.sigma_noise).unwrap();
    let alpha_dist = Normal::new(0.0, config.sigma_alpha).unwrap();

    // Initialize true state
    let mut true_state = TrueState::new(0.0, 0.5, 0.0);

    // Initialize observers
    let mut dsfb = DsfbObserver::new(dsfb_params, 2);
    dsfb.init(DsfbState::new(0.0, 0.5, 0.0));

    let mut freqonly = FreqOnlyObserver::new(0.5, 0.1);

    let mut trace = Vec::with_capacity(config.steps);

    for step in 0..config.steps {
        let t = step as f64 * config.dt;

        // Generate measurements
        let noise1 = noise_dist.sample(&mut rng);
        let noise2 = noise_dist.sample(&mut rng);

        let y1 = true_state.phi + noise1;

        // Channel 2 has drift
        let mut y2 = true_state.phi + config.drift_beta * t + noise2;

        // Add impulse
        if step >= config.impulse_start && step < config.impulse_start + config.impulse_duration {
            y2 += config.impulse_amplitude;
        }

        // Mean fusion
        let phi_mean = (y1 + y2) / 2.0;

        // Frequency-only observer
        let phi_freqonly = freqonly.step(&[y1, y2], config.dt);

        // DSFB observer
        let diagnostics = dsfb.step_with_diagnostics(&[y1, y2], config.dt);
        let dsfb_state = diagnostics.state;
        let phi_dsfb = dsfb_state.phi;

        // Errors
        let err_mean = (phi_mean - true_state.phi).abs();
        let err_freqonly = (phi_freqonly - true_state.phi).abs();
        let err_dsfb = (phi_dsfb - true_state.phi).abs();

        trace.push(SimulationTraceStep {
            step,
            t,
            phi_true: true_state.phi,
            measurements: vec![y1, y2],
            phi_mean,
            phi_freqonly,
            dsfb_state,
            err_mean,
            err_freqonly,
            err_dsfb,
            trust_stats: diagnostics.trust_stats,
            residuals: diagnostics.residuals,
            aggregate_residual: diagnostics.aggregate_residual,
        });

        // Update true dynamics
        true_state.phi += true_state.omega * config.dt;
        true_state.omega += true_state.alpha * config.dt;
        true_state.alpha += alpha_dist.sample(&mut rng);
    }

    trace
}

/// Calculate RMS error
pub fn rms_error(errors: &[f64]) -> f64 {
    let sum_sq: f64 = errors.iter().map(|&e| e * e).sum();
    (sum_sq / errors.len() as f64).sqrt()
}

/// Calculate peak error during impulse
pub fn peak_error_during_impulse(
    results: &[SimStep],
    impulse_start: usize,
    impulse_duration: usize,
    get_error: impl Fn(&SimStep) -> f64,
) -> f64 {
    results[impulse_start..impulse_start + impulse_duration]
        .iter()
        .map(get_error)
        .fold(0.0f64, f64::max)
}

/// Calculate recovery time (steps after impulse to reach threshold)
pub fn recovery_time(
    results: &[SimStep],
    impulse_end: usize,
    threshold: f64,
    get_error: impl Fn(&SimStep) -> f64,
) -> usize {
    for (i, step) in results[impulse_end..].iter().enumerate() {
        if get_error(step) < threshold {
            return i;
        }
    }
    results.len() - impulse_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_runs() {
        let config = SimConfig {
            steps: 100,
            ..Default::default()
        };
        let params = DsfbParams::default();
        let results = run_simulation(config, params);
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_simulation_trace_runs() {
        let config = SimConfig {
            steps: 16,
            ..Default::default()
        };
        let params = DsfbParams::default();
        let trace = run_simulation_trace(config, params);
        assert_eq!(trace.len(), 16);
        assert_eq!(trace[0].trust_stats.len(), 2);
        assert_eq!(trace[0].residuals.len(), 2);
    }

    #[test]
    fn test_rms_error() {
        let errors = vec![0.1, 0.2, 0.3];
        let rms = rms_error(&errors);
        let expected = ((0.01_f64 + 0.04 + 0.09) / 3.0).sqrt();
        assert!((rms - expected).abs() < 1e-10);
    }
}
