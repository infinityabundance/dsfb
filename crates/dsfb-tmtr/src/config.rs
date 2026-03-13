use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioSelection {
    All,
    DisturbanceRecovery,
    ForwardPrediction,
    HierarchyConsistency,
    AerospaceNavigation,
    RoboticsSensorOcclusion,
    IndustrialFaultRefinement,
    NeuralMultimodalDelay,
}

impl Default for ScenarioSelection {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum KernelKind {
    Uniform,
    Exponential,
    ResonanceGated,
}

impl Default for KernelKind {
    fn default() -> Self {
        Self::ResonanceGated
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub scenario: Option<ScenarioSelection>,
    #[arg(long)]
    pub output_root: Option<PathBuf>,
    #[arg(long)]
    pub n_steps: Option<usize>,
    #[arg(long)]
    pub delta: Option<usize>,
    #[arg(long)]
    pub prediction_horizon: Option<usize>,
    #[arg(long)]
    pub max_iterations: Option<usize>,
    #[arg(long)]
    pub max_recursion_depth: Option<usize>,
    #[arg(long)]
    pub convergence_tolerance: Option<f64>,
    #[arg(long)]
    pub trust_threshold: Option<f64>,
    #[arg(long)]
    pub min_trust_gap: Option<f64>,
    #[arg(long, value_enum)]
    pub kernel: Option<KernelKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub scenario: ScenarioSelection,
    pub output_root: String,
    pub n_steps: usize,
    pub delta: usize,
    pub prediction_horizon: usize,
    pub max_iterations: usize,
    pub max_recursion_depth: usize,
    pub convergence_tolerance: f64,
    pub trust_threshold: f64,
    pub min_trust_gap: f64,
    pub kernel: KernelKind,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            scenario: ScenarioSelection::All,
            output_root: default_output_root().display().to_string(),
            n_steps: 1000,
            delta: 48,
            prediction_horizon: 32,
            max_iterations: 6,
            max_recursion_depth: 2,
            convergence_tolerance: 1e-3,
            trust_threshold: 0.62,
            min_trust_gap: 0.04,
            kernel: KernelKind::ResonanceGated,
        }
    }
}

impl SimulationConfig {
    pub fn from_cli(cli: Cli) -> Result<Self> {
        let mut config = if let Some(path) = cli.config.as_ref() {
            Self::from_json_file(path)?
        } else {
            Self::default()
        };

        if let Some(scenario) = cli.scenario {
            config.scenario = scenario;
        }
        if let Some(path) = cli.output_root {
            config.output_root = path.display().to_string();
        }
        if let Some(n_steps) = cli.n_steps {
            config.n_steps = n_steps.max(64);
        }
        if let Some(delta) = cli.delta {
            config.delta = delta.max(4);
        }
        if let Some(prediction_horizon) = cli.prediction_horizon {
            config.prediction_horizon = prediction_horizon.max(4);
        }
        if let Some(max_iterations) = cli.max_iterations {
            config.max_iterations = max_iterations.max(1);
        }
        if let Some(max_recursion_depth) = cli.max_recursion_depth {
            config.max_recursion_depth = max_recursion_depth.max(1);
        }
        if let Some(convergence_tolerance) = cli.convergence_tolerance {
            config.convergence_tolerance = convergence_tolerance.max(1e-9);
        }
        if let Some(trust_threshold) = cli.trust_threshold {
            config.trust_threshold = trust_threshold.clamp(0.0, 1.0);
        }
        if let Some(min_trust_gap) = cli.min_trust_gap {
            config.min_trust_gap = min_trust_gap.max(0.0);
        }
        if let Some(kernel) = cli.kernel {
            config.kernel = kernel;
        }
        config.prediction_horizon = config
            .prediction_horizon
            .min(config.n_steps.saturating_sub(1));
        config.delta = config.delta.min(config.n_steps.saturating_sub(2));
        Ok(config)
    }

    pub fn from_json_file(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let mut config: Self = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse config file {}", path.display()))?;
        if config.output_root.is_empty() {
            config.output_root = default_output_root().display().to_string();
        }
        Ok(config)
    }

    pub fn output_root_path(&self) -> PathBuf {
        PathBuf::from(&self.output_root)
    }

    pub fn stable_hash(&self) -> Result<String> {
        let json = serde_json::to_string(self).context("failed to serialize config for hashing")?;
        Ok(stable_hash_bytes(json.as_bytes()))
    }
}

pub fn default_output_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("output-dsfb-tmtr")
}

pub fn stable_hash_bytes(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
