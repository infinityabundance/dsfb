use std::time::{Duration, Instant};

use nalgebra::{DMatrix, DVector};

use crate::sim::diagnostics::DiagnosticModel;
use crate::sim::state::BenchConfig;

pub mod cov_inflate;
pub mod dsfb;
pub mod equal;
pub mod irls_huber;
pub mod nis_gating;

pub const METHOD_ORDER: [&str; 6] = [
    "equal",
    "cov_inflate",
    "irls_huber",
    "nis_hard",
    "nis_soft",
    "dsfb",
];

#[derive(Debug, Clone)]
pub struct MethodStepResult {
    pub x_hat: DVector<f64>,
    pub group_weights: Option<Vec<f64>>,
    pub solve_time: Duration,
    pub total_time: Duration,
}

pub trait ReconstructionMethod {
    fn name(&self) -> &'static str;
    fn reset(&mut self, _cfg: &BenchConfig, _model: &DiagnosticModel) {}
    fn has_weights(&self) -> bool;
    fn estimate(&mut self, model: &DiagnosticModel, y_groups: &[DVector<f64>]) -> MethodStepResult;
}

fn solve_normal_equation(normal: DMatrix<f64>, rhs: DVector<f64>) -> DVector<f64> {
    if let Some(chol) = normal.clone().cholesky() {
        return chol.solve(&rhs);
    }
    if let Some(sol) = normal.lu().solve(&rhs) {
        return sol;
    }
    DVector::<f64>::zeros(rhs.nrows())
}

pub fn solve_group_weighted_wls(
    model: &DiagnosticModel,
    y_groups: &[DVector<f64>],
    group_weights: &[f64],
) -> (DVector<f64>, Duration) {
    let t0 = Instant::now();
    let n = model.n;

    let mut normal = DMatrix::<f64>::identity(n, n) * 1e-9;
    let mut rhs = DVector::<f64>::zeros(n);

    for (k, group) in model.groups.iter().enumerate() {
        let gw = group_weights[k].max(0.0);
        if gw <= 0.0 {
            continue;
        }

        let y = &y_groups[k];
        for i in 0..group.dim() {
            let var = group.r_diag[i].max(1e-12);
            let inv_var = gw / var;
            let row = group.h.row(i);
            let yi = y[i];

            for a in 0..n {
                let ha = row[a];
                rhs[a] += inv_var * ha * yi;
                for b in 0..n {
                    normal[(a, b)] += inv_var * ha * row[b];
                }
            }
        }
    }

    let x = solve_normal_equation(normal, rhs);
    (x, t0.elapsed())
}

pub fn solve_measurement_weighted_wls(
    model: &DiagnosticModel,
    y_groups: &[DVector<f64>],
    measurement_weights: &[Vec<f64>],
) -> (DVector<f64>, Duration) {
    let t0 = Instant::now();
    let n = model.n;

    let mut normal = DMatrix::<f64>::identity(n, n) * 1e-9;
    let mut rhs = DVector::<f64>::zeros(n);

    for (k, group) in model.groups.iter().enumerate() {
        let y = &y_groups[k];
        for i in 0..group.dim() {
            let mw = measurement_weights[k][i].max(0.0);
            if mw <= 0.0 {
                continue;
            }

            let var = group.r_diag[i].max(1e-12);
            let inv_var = mw / var;
            let row = group.h.row(i);
            let yi = y[i];

            for a in 0..n {
                let ha = row[a];
                rhs[a] += inv_var * ha * yi;
                for b in 0..n {
                    normal[(a, b)] += inv_var * ha * row[b];
                }
            }
        }
    }

    let x = solve_normal_equation(normal, rhs);
    (x, t0.elapsed())
}

pub fn compute_group_nis(
    model: &DiagnosticModel,
    y_groups: &[DVector<f64>],
    x_hat: &DVector<f64>,
) -> Vec<f64> {
    let mut nis = Vec::with_capacity(model.groups.len());

    for (k, group) in model.groups.iter().enumerate() {
        let residual = &y_groups[k] - &group.h * x_hat;
        let mut sum = 0.0;
        for i in 0..group.dim() {
            let var = group.r_diag[i].max(1e-12);
            sum += residual[i] * residual[i] / var;
        }
        nis.push(sum / group.dim() as f64);
    }

    nis
}

pub fn canonical_method_list(raw: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for name in METHOD_ORDER {
        if raw.iter().any(|m| m == name) {
            out.push(name.to_string());
        }
    }
    out
}
