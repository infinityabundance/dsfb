use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use chrono::Utc;
use dsfb::DsfbParams;

#[derive(Debug, Clone)]
pub struct DscdSweepConfig {
    pub num_events: usize,
    pub tau_min: f64,
    pub tau_max: f64,
    pub tau_steps: usize,
    pub max_depth: Option<usize>,
    pub dsfb_params: DsfbParams,
}

impl Default for DscdSweepConfig {
    fn default() -> Self {
        Self {
            num_events: 1_024,
            tau_min: 0.0,
            tau_max: 1.0,
            tau_steps: 101,
            max_depth: None,
            dsfb_params: DsfbParams::default(),
        }
    }
}

impl DscdSweepConfig {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.num_events > 0, "num_events must be greater than zero");
        ensure!(self.tau_steps > 0, "tau_steps must be greater than zero");
        ensure!(self.tau_min.is_finite(), "tau_min must be finite");
        ensure!(self.tau_max.is_finite(), "tau_max must be finite");
        ensure!(
            self.tau_max >= self.tau_min,
            "tau_max must be greater than or equal to tau_min"
        );
        Ok(())
    }

    pub fn tau_grid(&self) -> Vec<f64> {
        if self.tau_steps == 1 {
            return vec![self.tau_min];
        }

        let span = self.tau_max - self.tau_min;
        let denom = (self.tau_steps - 1) as f64;
        (0..self.tau_steps)
            .map(|idx| self.tau_min + span * idx as f64 / denom)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct OutputPaths {
    pub root: PathBuf,
    pub run_dir: PathBuf,
}

pub fn workspace_root_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

pub fn create_timestamped_output_dir() -> Result<OutputPaths> {
    let root = workspace_root_dir().join("output-dsfb-dscd");
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create output root {}", root.display()))?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let mut run_dir = root.join(&timestamp);
    let mut suffix = 1_u32;

    while run_dir.exists() {
        run_dir = root.join(format!("{timestamp}_{suffix:02}"));
        suffix += 1;
    }

    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create run directory {}", run_dir.display()))?;

    Ok(OutputPaths { root, run_dir })
}
