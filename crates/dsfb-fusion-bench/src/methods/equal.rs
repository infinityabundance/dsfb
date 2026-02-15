use std::time::Instant;

use nalgebra::DVector;

use crate::methods::{solve_group_weighted_wls, MethodStepResult, ReconstructionMethod};
use crate::sim::diagnostics::DiagnosticModel;

#[derive(Default)]
pub struct EqualMethod;

impl ReconstructionMethod for EqualMethod {
    fn name(&self) -> &'static str {
        "equal"
    }

    fn has_weights(&self) -> bool {
        false
    }

    fn estimate(&mut self, model: &DiagnosticModel, y_groups: &[DVector<f64>]) -> MethodStepResult {
        let total_t0 = Instant::now();
        let weights = vec![1.0; model.groups.len()];
        let (x_hat, solve_time) = solve_group_weighted_wls(model, y_groups, &weights);
        MethodStepResult {
            x_hat,
            group_weights: None,
            solve_time,
            total_time: total_t0.elapsed(),
        }
    }
}
