use anyhow::{bail, Context, Result};
use nalgebra::{DMatrix, DVector};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::sim::diagnostics::{generate_measurements, DiagnosticModel, MeasurementFrame};
use crate::sim::faults::apply_impulse_corruption;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchConfig {
    pub schema_version: String,
    pub steps: usize,
    pub dt: f64,
    pub n: usize,
    pub group_dims: Vec<usize>,
    pub noise_std: Vec<f64>,
    pub process_noise_std: f64,
    pub bandwidth_groups: Vec<usize>,
    pub bandwidth_tau: f64,
    pub corruption_group: usize,
    pub corruption_channel: usize,
    pub corruption_start: usize,
    pub corruption_duration: usize,
    pub corruption_amplitude: f64,
    pub cov_inflate_factor: f64,
    pub nis_threshold: f64,
    pub nis_soft_scale: f64,
    pub irls_delta: f64,
    pub irls_max_iter: usize,
    pub irls_tol: f64,
    pub dsfb_alpha: f64,
    pub dsfb_beta: f64,
    pub dsfb_w_min: f64,
    pub matrix_seed: u64,
    pub seeds: Vec<u64>,
    pub methods: Vec<String>,
    pub alpha_values: Option<Vec<f64>>,
    pub beta_values: Option<Vec<f64>>,
}

impl BenchConfig {
    pub fn from_toml_file(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let cfg: BenchConfig = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML config: {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.steps == 0 {
            bail!("steps must be > 0");
        }
        if self.n == 0 {
            bail!("n must be > 0");
        }
        if self.dt <= 0.0 {
            bail!("dt must be > 0");
        }
        if self.group_dims.is_empty() {
            bail!("group_dims must be non-empty");
        }
        if self.group_dims.iter().any(|&m| m == 0) {
            bail!("all entries in group_dims must be > 0");
        }
        if self.noise_std.len() != self.group_dims.len() {
            bail!("noise_std length must equal group_dims length");
        }
        if self.noise_std.iter().any(|&s| s <= 0.0) {
            bail!("all noise_std entries must be > 0");
        }
        if self.corruption_group >= self.group_dims.len() {
            bail!("corruption_group index out of range");
        }
        if self.corruption_channel >= self.group_dims[self.corruption_group] {
            bail!("corruption_channel index out of range for corruption_group");
        }
        if self.corruption_start >= self.steps {
            bail!("corruption_start must be < steps");
        }
        if self.corruption_duration == 0 {
            bail!("corruption_duration must be > 0");
        }
        if self.irls_max_iter == 0 {
            bail!("irls_max_iter must be > 0");
        }
        if !(0.0..=1.0).contains(&self.dsfb_w_min) {
            bail!("dsfb_w_min must be in [0, 1]");
        }
        if self.dsfb_beta <= 0.0 || self.dsfb_beta > 1.0 {
            bail!("dsfb_beta must be in (0, 1]");
        }
        if self.bandwidth_tau < 0.0 {
            bail!("bandwidth_tau must be >= 0");
        }
        if self.seeds.is_empty() {
            bail!("seeds must be non-empty");
        }
        Ok(())
    }

    pub fn total_measurements(&self) -> usize {
        self.group_dims.iter().sum()
    }

    pub fn group_count(&self) -> usize {
        self.group_dims.len()
    }
}

#[derive(Debug, Clone)]
pub struct SimulationData {
    pub t: Vec<f64>,
    pub x_true: Vec<DVector<f64>>,
    pub measurements: Vec<MeasurementFrame>,
    pub corruption_active: Vec<bool>,
}

fn build_dynamics_matrix(n: usize, dt: f64) -> DMatrix<f64> {
    let mut a = DMatrix::<f64>::identity(n, n);
    for i in 0..n {
        let coupling = 0.015 * dt;
        a[(i, i)] = 1.0 - 0.002 * dt;
        if i + 1 < n {
            a[(i, i + 1)] = coupling;
        }
        if i > 0 {
            a[(i, i - 1)] = -0.5 * coupling;
        }
    }
    a
}

fn deterministic_drive(n: usize, t: f64, dt: f64) -> DVector<f64> {
    let mut u = DVector::<f64>::zeros(n);
    for i in 0..n {
        let f1 = 0.07 * (i as f64 + 1.0);
        let f2 = 0.03 * (i as f64 + 2.0);
        u[i] = dt * (0.05 * (f1 * t).sin() + 0.03 * (f2 * t).cos());
    }
    u
}

pub fn generate_simulation_data(
    cfg: &BenchConfig,
    model: &DiagnosticModel,
    seed: u64,
) -> Result<SimulationData> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let process_noise = Normal::new(0.0, cfg.process_noise_std)
        .context("failed to create process noise distribution")?;

    let a = build_dynamics_matrix(cfg.n, cfg.dt);
    let mut x = DVector::<f64>::zeros(cfg.n);
    let mut low_pass_state: Vec<Option<DVector<f64>>> = vec![None; cfg.group_count()];

    let mut t_vec = Vec::with_capacity(cfg.steps);
    let mut x_true = Vec::with_capacity(cfg.steps);
    let mut frames = Vec::with_capacity(cfg.steps);
    let mut corruption_flags = Vec::with_capacity(cfg.steps);

    for step in 0..cfg.steps {
        let t = step as f64 * cfg.dt;

        let mut frame = generate_measurements(cfg, model, &x, step, &mut low_pass_state, &mut rng)?;
        let corrupted = apply_impulse_corruption(cfg, &mut frame, step);

        t_vec.push(t);
        x_true.push(x.clone());
        frames.push(frame);
        corruption_flags.push(corrupted);

        let mut next_x = &a * &x + deterministic_drive(cfg.n, t, cfg.dt);
        for i in 0..cfg.n {
            next_x[i] += process_noise.sample(&mut rng);
        }
        x = next_x;
    }

    Ok(SimulationData {
        t: t_vec,
        x_true,
        measurements: frames,
        corruption_active: corruption_flags,
    })
}
