use std::time::Instant;

use nalgebra::DVector;

use crate::methods::{
    compute_group_nis, solve_group_weighted_wls, MethodStepResult, ReconstructionMethod,
};
use crate::sim::diagnostics::DiagnosticModel;
use crate::sim::state::BenchConfig;

pub struct DsfbAdaptiveMethod {
    alpha: f64,
    beta: f64,
    w_min: f64,
    envelope: Vec<f64>,
}

impl DsfbAdaptiveMethod {
    pub fn new() -> Self {
        Self {
            alpha: 1.0,
            beta: 0.1,
            w_min: 0.1,
            envelope: Vec::new(),
        }
    }
}

impl ReconstructionMethod for DsfbAdaptiveMethod {
    fn name(&self) -> &'static str {
        "dsfb"
    }

    fn reset(&mut self, cfg: &BenchConfig, model: &DiagnosticModel) {
        self.alpha = cfg.dsfb_alpha;
        self.beta = cfg.dsfb_beta;
        self.w_min = cfg.dsfb_w_min;
        self.envelope = vec![1.0; model.groups.len()];
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
            let score = nis_k.sqrt();
            self.envelope[k] = (1.0 - self.beta) * self.envelope[k] + self.beta * score;
            let excess = (self.envelope[k] - 1.0).max(0.0);
            let trust = (-self.alpha * excess).exp();
            weights[k] = trust.clamp(self.w_min, 1.0);
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
