use std::time::Instant;

use nalgebra::DVector;

use crate::methods::{
    solve_group_weighted_wls, solve_measurement_weighted_wls, MethodStepResult,
    ReconstructionMethod,
};
use crate::sim::diagnostics::DiagnosticModel;
use crate::sim::state::BenchConfig;

pub struct IrlsHuberMethod {
    delta: f64,
    max_iter: usize,
    tol: f64,
}

impl IrlsHuberMethod {
    pub fn new() -> Self {
        Self {
            delta: 1.5,
            max_iter: 8,
            tol: 1e-6,
        }
    }
}

impl ReconstructionMethod for IrlsHuberMethod {
    fn name(&self) -> &'static str {
        "irls_huber"
    }

    fn reset(&mut self, cfg: &BenchConfig, _model: &DiagnosticModel) {
        self.delta = cfg.irls_delta;
        self.max_iter = cfg.irls_max_iter;
        self.tol = cfg.irls_tol;
    }

    fn has_weights(&self) -> bool {
        false
    }

    fn estimate(&mut self, model: &DiagnosticModel, y_groups: &[DVector<f64>]) -> MethodStepResult {
        let total_t0 = Instant::now();

        let (mut x_hat, mut solve_time) =
            solve_group_weighted_wls(model, y_groups, &vec![1.0; model.groups.len()]);

        for _ in 0..self.max_iter {
            let mut measurement_weights: Vec<Vec<f64>> = Vec::with_capacity(model.groups.len());

            for (k, group) in model.groups.iter().enumerate() {
                let residual = &y_groups[k] - &group.h * &x_hat;
                let mut w_k = vec![1.0; group.dim()];
                for i in 0..group.dim() {
                    let sigma = group.r_diag[i].sqrt().max(1e-12);
                    let z = residual[i] / sigma;
                    let abs_z = z.abs();
                    w_k[i] = if abs_z <= self.delta {
                        1.0
                    } else {
                        self.delta / abs_z
                    };
                }
                measurement_weights.push(w_k);
            }

            let prev = x_hat.clone();
            let (new_x, this_solve) =
                solve_measurement_weighted_wls(model, y_groups, &measurement_weights);
            solve_time += this_solve;
            x_hat = new_x;

            let dx = (&x_hat - prev).norm();
            if dx < self.tol {
                break;
            }
        }

        MethodStepResult {
            x_hat,
            group_weights: None,
            solve_time,
            total_time: total_t0.elapsed(),
        }
    }
}
