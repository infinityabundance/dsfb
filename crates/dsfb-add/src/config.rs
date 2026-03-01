use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};

use crate::AddError;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationConfig {
    #[serde_as(as = "DefaultOnNull")]
    pub num_lambda: usize,
    #[serde_as(as = "DefaultOnNull")]
    pub lambda_min: f64,
    #[serde_as(as = "DefaultOnNull")]
    pub lambda_max: f64,
    #[serde_as(as = "DefaultOnNull")]
    pub steps_per_run: usize,
    #[serde(default)]
    pub multi_steps_per_run: Vec<usize>,
    #[serde_as(as = "DefaultOnNull")]
    pub random_seed: u64,
    #[serde_as(as = "DefaultOnNull")]
    pub enable_aet: bool,
    #[serde_as(as = "DefaultOnNull")]
    pub enable_tcp: bool,
    #[serde_as(as = "DefaultOnNull")]
    pub enable_rlt: bool,
    #[serde_as(as = "DefaultOnNull")]
    pub enable_iwlt: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_lambda: 360,
            lambda_min: 0.0,
            lambda_max: 1.0,
            steps_per_run: 512,
            multi_steps_per_run: Vec::new(),
            random_seed: 0xADD2_0260_0001_u64,
            enable_aet: true,
            enable_tcp: true,
            enable_rlt: true,
            enable_iwlt: true,
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), AddError> {
        if self.num_lambda == 0 {
            return Err(AddError::InvalidConfig(
                "num_lambda must be greater than zero".to_string(),
            ));
        }

        if self.steps_per_run == 0 {
            return Err(AddError::InvalidConfig(
                "steps_per_run must be greater than zero".to_string(),
            ));
        }

        if self.multi_steps_per_run.iter().any(|&steps| steps == 0) {
            return Err(AddError::InvalidConfig(
                "multi_steps_per_run must contain only values greater than zero".to_string(),
            ));
        }

        if !self.lambda_min.is_finite() || !self.lambda_max.is_finite() {
            return Err(AddError::InvalidConfig(
                "lambda_min and lambda_max must be finite".to_string(),
            ));
        }

        if self.lambda_max < self.lambda_min {
            return Err(AddError::InvalidConfig(
                "lambda_max must be greater than or equal to lambda_min".to_string(),
            ));
        }

        if !(self.enable_aet || self.enable_tcp || self.enable_rlt || self.enable_iwlt) {
            return Err(AddError::InvalidConfig(
                "at least one sub-theory must be enabled".to_string(),
            ));
        }

        Ok(())
    }

    pub fn lambda_grid(&self) -> Vec<f64> {
        if self.num_lambda == 1 {
            return vec![self.lambda_min];
        }

        let span = self.lambda_max - self.lambda_min;
        let denom = (self.num_lambda - 1) as f64;

        (0..self.num_lambda)
            .map(|idx| self.lambda_min + span * idx as f64 / denom)
            .collect()
    }

    pub fn normalized_lambda(&self, lambda: f64) -> f64 {
        let span = self.lambda_max - self.lambda_min;
        if span.abs() < f64::EPSILON {
            return 0.5;
        }

        ((lambda - self.lambda_min) / span).clamp(0.0, 1.0)
    }

    pub fn sweep_steps(&self) -> Vec<usize> {
        if self.multi_steps_per_run.is_empty() {
            vec![self.steps_per_run]
        } else {
            self.multi_steps_per_run.clone()
        }
    }
}
