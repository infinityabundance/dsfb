use std::time::Instant;

use nalgebra::DVector;

use crate::methods::{
    compute_group_nis, solve_group_weighted_wls, MethodStepResult, ReconstructionMethod,
};
use crate::sim::diagnostics::DiagnosticModel;
use crate::sim::state::BenchConfig;

#[derive(Debug, Clone, Copy)]
pub enum NisMode {
    Hard,
    Soft,
}

pub struct NisGatingMethod {
    mode: NisMode,
    threshold: f64,
    soft_scale: f64,
}

impl NisGatingMethod {
    pub fn new(mode: NisMode) -> Self {
        Self {
            mode,
            threshold: 3.0,
            soft_scale: 0.5,
        }
    }
}

impl ReconstructionMethod for NisGatingMethod {
    fn name(&self) -> &'static str {
        match self.mode {
            NisMode::Hard => "nis_hard",
            NisMode::Soft => "nis_soft",
        }
    }

    fn reset(&mut self, cfg: &BenchConfig, _model: &DiagnosticModel) {
        self.threshold = cfg.nis_threshold;
        self.soft_scale = cfg.nis_soft_scale;
    }

    fn has_weights(&self) -> bool {
        true
    }

    fn estimate(&mut self, model: &DiagnosticModel, y_groups: &[DVector<f64>]) -> MethodStepResult {
        let total_t0 = Instant::now();

        let (x_eq, solve_0) =
            solve_group_weighted_wls(model, y_groups, &vec![1.0; model.groups.len()]);
        let nis = compute_group_nis(model, y_groups, &x_eq);

        let mut weights = vec![1.0; model.groups.len()];
        for (k, nis_k) in nis.iter().enumerate() {
            let w = match self.mode {
                NisMode::Hard => {
                    if *nis_k > self.threshold {
                        0.0
                    } else {
                        1.0
                    }
                }
                NisMode::Soft => {
                    let excess = (*nis_k - self.threshold).max(0.0);
                    1.0 / (1.0 + self.soft_scale * excess)
                }
            };
            weights[k] = w.clamp(0.0, 1.0);
        }

        let (x_hat, solve_1) = solve_group_weighted_wls(model, y_groups, &weights);
        MethodStepResult {
            x_hat,
            group_weights: Some(weights),
            solve_time: solve_0 + solve_1,
            total_time: total_t0.elapsed(),
        }
    }
}
