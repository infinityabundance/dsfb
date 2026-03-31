use crate::precursor::DsaConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub healthy_pass_runs: usize,
    pub drift_window: usize,
    pub envelope_sigma: f64,
    pub boundary_fraction_of_rho: f64,
    pub state_confirmation_steps: usize,
    pub persistent_state_steps: usize,
    pub density_window: usize,
    pub ewma_alpha: f64,
    pub ewma_sigma_multiplier: f64,
    pub drift_sigma_multiplier: f64,
    pub slew_sigma_multiplier: f64,
    pub grazing_window: usize,
    pub grazing_min_hits: usize,
    pub pre_failure_lookback_runs: usize,
    pub minimum_healthy_observations: usize,
    pub epsilon: f64,
    pub dsa: DsaConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            healthy_pass_runs: 100,
            drift_window: 5,
            envelope_sigma: 3.0,
            boundary_fraction_of_rho: 0.5,
            state_confirmation_steps: 2,
            persistent_state_steps: 2,
            density_window: 10,
            ewma_alpha: 0.2,
            ewma_sigma_multiplier: 3.0,
            drift_sigma_multiplier: 3.0,
            slew_sigma_multiplier: 3.0,
            grazing_window: 10,
            grazing_min_hits: 3,
            pre_failure_lookback_runs: 20,
            minimum_healthy_observations: 2,
            epsilon: 1.0e-9,
            dsa: DsaConfig::default(),
        }
    }
}

impl PipelineConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.healthy_pass_runs < 2 {
            return Err("healthy_pass_runs must be at least 2".into());
        }
        if self.drift_window == 0 {
            return Err("drift_window must be positive".into());
        }
        if self.envelope_sigma <= 0.0 {
            return Err("envelope_sigma must be positive".into());
        }
        if !(0.0..=1.0).contains(&self.boundary_fraction_of_rho) {
            return Err("boundary_fraction_of_rho must be in [0, 1]".into());
        }
        if self.state_confirmation_steps == 0 {
            return Err("state_confirmation_steps must be positive".into());
        }
        if self.persistent_state_steps == 0 {
            return Err("persistent_state_steps must be positive".into());
        }
        if self.density_window == 0 {
            return Err("density_window must be positive".into());
        }
        if !(0.0..=1.0).contains(&self.ewma_alpha) || self.ewma_alpha == 0.0 {
            return Err("ewma_alpha must be in (0, 1]".into());
        }
        if self.ewma_sigma_multiplier <= 0.0 {
            return Err("ewma_sigma_multiplier must be positive".into());
        }
        if self.minimum_healthy_observations < 2 {
            return Err("minimum_healthy_observations must be at least 2".into());
        }
        self.dsa
            .validate()
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfiguration {
    pub dataset: String,
    pub config: PipelineConfig,
    pub data_root: String,
    pub output_root: String,
    pub secom_fetch_if_missing: bool,
}
