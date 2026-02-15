use anyhow::{Context, Result};
use nalgebra::{DMatrix, DVector};
use rand::distributions::{Distribution as RandDistribution, Uniform};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::Normal;

use crate::sim::state::BenchConfig;

#[derive(Debug, Clone)]
pub struct DiagnosticGroup {
    pub h: DMatrix<f64>,
    pub r_diag: DVector<f64>,
    pub bandwidth_mismatch: bool,
}

impl DiagnosticGroup {
    pub fn dim(&self) -> usize {
        self.h.nrows()
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticModel {
    pub n: usize,
    pub groups: Vec<DiagnosticGroup>,
}

#[derive(Debug, Clone)]
pub struct MeasurementFrame {
    pub y_groups: Vec<DVector<f64>>,
}

pub fn build_diagnostic_model(cfg: &BenchConfig) -> Result<DiagnosticModel> {
    let mut rng = ChaCha8Rng::seed_from_u64(cfg.matrix_seed);
    let uniform = Uniform::new(-0.45_f64, 0.45_f64);

    let mut groups = Vec::with_capacity(cfg.group_count());
    let mut running_offset = 0usize;

    for (k, &m_k) in cfg.group_dims.iter().enumerate() {
        let mut h = DMatrix::<f64>::zeros(m_k, cfg.n);
        for r in 0..m_k {
            for c in 0..cfg.n {
                h[(r, c)] = uniform.sample(&mut rng);
            }

            // Inject deterministic structure so every state dimension is observed across groups.
            let anchor_col = (running_offset + r) % cfg.n;
            h[(r, anchor_col)] += 1.0 + 0.15 * (k as f64);

            let side_col = (anchor_col + 3) % cfg.n;
            h[(r, side_col)] += 0.2 * (1.0 + (r as f64 / (m_k as f64 + 1.0)));
        }

        let sigma = cfg.noise_std[k];
        let mut r_diag = DVector::<f64>::zeros(m_k);
        for i in 0..m_k {
            r_diag[i] = sigma * sigma;
        }

        let mismatch = cfg.bandwidth_groups.contains(&k);
        groups.push(DiagnosticGroup {
            h,
            r_diag,
            bandwidth_mismatch: mismatch,
        });
        running_offset += m_k;
    }

    Ok(DiagnosticModel { n: cfg.n, groups })
}

pub fn generate_measurements(
    cfg: &BenchConfig,
    model: &DiagnosticModel,
    x_true: &DVector<f64>,
    _step: usize,
    low_pass_state: &mut [Option<DVector<f64>>],
    rng: &mut impl Rng,
) -> Result<MeasurementFrame> {
    let alpha_lp = if cfg.bandwidth_tau <= 0.0 {
        1.0
    } else {
        (cfg.dt / (cfg.bandwidth_tau + cfg.dt)).clamp(0.0, 1.0)
    };

    let mut y_groups = Vec::with_capacity(model.groups.len());

    for (k, group) in model.groups.iter().enumerate() {
        let ideal = &group.h * x_true;
        let mut base = ideal.clone();

        if group.bandwidth_mismatch {
            match &mut low_pass_state[k] {
                Some(prev) => {
                    for i in 0..group.dim() {
                        prev[i] += alpha_lp * (ideal[i] - prev[i]);
                    }
                    base = prev.clone();
                }
                None => {
                    low_pass_state[k] = Some(ideal.clone());
                    base = ideal;
                }
            }
        }

        let sigma = cfg.noise_std[k];
        let noise_dist = Normal::new(0.0, sigma)
            .with_context(|| format!("failed to create measurement noise for group {k}"))?;

        let mut y = base;
        for i in 0..group.dim() {
            y[i] += noise_dist.sample(rng);
        }
        y_groups.push(y);
    }

    Ok(MeasurementFrame { y_groups })
}
