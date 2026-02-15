use std::time::Instant;

use nalgebra::DVector;

use crate::methods::{solve_group_weighted_wls, MethodStepResult, ReconstructionMethod};
use crate::sim::diagnostics::DiagnosticModel;
use crate::sim::state::BenchConfig;

pub struct CovInflateMethod {
    weights: Vec<f64>,
}

impl CovInflateMethod {
    pub fn new() -> Self {
        Self {
            weights: Vec::new(),
        }
    }
}

impl ReconstructionMethod for CovInflateMethod {
    fn name(&self) -> &'static str {
        "cov_inflate"
    }

    fn reset(&mut self, cfg: &BenchConfig, model: &DiagnosticModel) {
        self.weights = vec![1.0; model.groups.len()];
        let w = (1.0 / cfg.cov_inflate_factor.max(1e-9)).clamp(0.0, 1.0);
        if cfg.corruption_group < self.weights.len() {
            self.weights[cfg.corruption_group] = w;
        }
    }

    fn has_weights(&self) -> bool {
        true
    }

    fn estimate(&mut self, model: &DiagnosticModel, y_groups: &[DVector<f64>]) -> MethodStepResult {
        let total_t0 = Instant::now();
        let (x_hat, solve_time) = solve_group_weighted_wls(model, y_groups, &self.weights);
        MethodStepResult {
            x_hat,
            group_weights: Some(self.weights.clone()),
            solve_time,
            total_time: total_t0.elapsed(),
        }
    }
}
