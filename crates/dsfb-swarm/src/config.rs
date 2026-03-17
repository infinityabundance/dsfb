use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::cli::{BenchmarkArgs, ReportArgs, RunArgs, ScenarioArgs};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioKind {
    Nominal,
    GradualEdgeDegradation,
    AdversarialAgent,
    CommunicationLoss,
    All,
}

impl ScenarioKind {
    pub fn executable_scenarios(self) -> Vec<Self> {
        match self {
            Self::All => vec![
                Self::Nominal,
                Self::GradualEdgeDegradation,
                Self::AdversarialAgent,
                Self::CommunicationLoss,
            ],
            other => vec![other],
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::GradualEdgeDegradation => "gradual_edge_degradation",
            Self::AdversarialAgent => "adversarial_agent",
            Self::CommunicationLoss => "communication_loss",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PredictorKind {
    ZeroOrderHold,
    FirstOrder,
    SmoothCorrective,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum TrustGateMode {
    BinaryEnvelope,
    SmoothDecay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub scenario: ScenarioKind,
    pub steps: usize,
    pub agents: usize,
    pub dt: f64,
    pub interaction_radius: f64,
    pub k_neighbors: usize,
    pub base_gain: f64,
    pub noise_level: f64,
    pub warmup_steps: usize,
    pub multi_mode: bool,
    pub monitored_modes: usize,
    pub mode_shapes: bool,
    pub predictor: PredictorKind,
    pub trust_mode: TrustGateMode,
    pub output_root: PathBuf,
    pub report_pdf: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            scenario: ScenarioKind::All,
            steps: 220,
            agents: 36,
            dt: 0.08,
            interaction_radius: 1.45,
            k_neighbors: 4,
            base_gain: 1.0,
            noise_level: 0.03,
            warmup_steps: 36,
            multi_mode: true,
            monitored_modes: 4,
            mode_shapes: true,
            predictor: PredictorKind::SmoothCorrective,
            trust_mode: TrustGateMode::SmoothDecay,
            output_root: default_output_root(),
            report_pdf: true,
        }
    }
}

impl RunConfig {
    pub fn default_quickstart() -> Self {
        Self {
            scenario: ScenarioKind::All,
            steps: 120,
            agents: 24,
            warmup_steps: 24,
            noise_level: 0.02,
            ..Self::default()
        }
    }

    pub fn resolve_with_patch(config_path: Option<&Path>, patch: RunArgsPatch) -> Result<Self> {
        let mut config = Self::default();
        if let Some(path) = config_path {
            let file_config = FileConfig::load(path)?;
            if let Some(run) = file_config.run {
                config.apply_patch(run);
            }
        }
        config.apply_patch(patch);
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.steps < 2 {
            bail!("steps must be at least 2");
        }
        if self.agents < 4 {
            bail!("agents must be at least 4");
        }
        if self.dt <= 0.0 {
            bail!("dt must be positive");
        }
        if self.interaction_radius <= 0.0 {
            bail!("interaction_radius must be positive");
        }
        if self.k_neighbors == 0 {
            bail!("k_neighbors must be greater than zero");
        }
        if self.monitored_modes == 0 {
            bail!("monitored_modes must be greater than zero");
        }
        if self.warmup_steps >= self.steps {
            bail!("warmup_steps must be smaller than steps");
        }
        Ok(())
    }

    pub fn apply_patch(&mut self, patch: RunArgsPatch) {
        if let Some(value) = patch.scenario {
            self.scenario = value;
        }
        if let Some(value) = patch.steps {
            self.steps = value;
        }
        if let Some(value) = patch.agents {
            self.agents = value;
        }
        if let Some(value) = patch.dt {
            self.dt = value;
        }
        if let Some(value) = patch.interaction_radius {
            self.interaction_radius = value;
        }
        if let Some(value) = patch.k_neighbors {
            self.k_neighbors = value;
        }
        if let Some(value) = patch.base_gain {
            self.base_gain = value;
        }
        if let Some(value) = patch.noise_level {
            self.noise_level = value;
        }
        if let Some(value) = patch.warmup_steps {
            self.warmup_steps = value;
        }
        if let Some(value) = patch.multi_mode {
            self.multi_mode = value;
        }
        if let Some(value) = patch.monitored_modes {
            self.monitored_modes = value;
        }
        if let Some(value) = patch.mode_shapes {
            self.mode_shapes = value;
        }
        if let Some(value) = patch.predictor {
            self.predictor = value;
        }
        if let Some(value) = patch.trust_mode {
            self.trust_mode = value;
        }
        if let Some(value) = patch.output_root {
            self.output_root = value;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub steps: usize,
    pub sizes: Vec<usize>,
    pub noise_levels: Vec<f64>,
    pub scenarios: Vec<ScenarioKind>,
    pub multi_mode: bool,
    pub monitored_modes: usize,
    pub mode_shapes: bool,
    pub predictor: PredictorKind,
    pub trust_mode: TrustGateMode,
    pub output_root: PathBuf,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            steps: 140,
            sizes: vec![20, 50, 100, 200],
            noise_levels: vec![0.01, 0.05, 0.10, 0.20],
            scenarios: vec![
                ScenarioKind::Nominal,
                ScenarioKind::GradualEdgeDegradation,
                ScenarioKind::AdversarialAgent,
                ScenarioKind::CommunicationLoss,
            ],
            multi_mode: true,
            monitored_modes: 4,
            mode_shapes: true,
            predictor: PredictorKind::SmoothCorrective,
            trust_mode: TrustGateMode::SmoothDecay,
            output_root: default_output_root(),
        }
    }
}

impl BenchmarkConfig {
    pub fn resolve_with_patch(config_path: Option<&Path>, patch: BenchmarkArgsPatch) -> Result<Self> {
        let mut config = Self::default();
        if let Some(path) = config_path {
            let file_config = FileConfig::load(path)?;
            if let Some(bench) = file_config.benchmark {
                config.apply_patch(bench)?;
            }
        }
        config.apply_patch(patch)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.steps < 2 {
            bail!("benchmark steps must be at least 2");
        }
        if self.sizes.is_empty() || self.sizes.iter().any(|value| *value < 4) {
            bail!("benchmark sizes must be non-empty and all at least 4");
        }
        if self.noise_levels.is_empty() || self.noise_levels.iter().any(|value| *value < 0.0) {
            bail!("benchmark noise levels must be non-empty and non-negative");
        }
        if self.scenarios.is_empty() {
            bail!("benchmark scenarios must be non-empty");
        }
        Ok(())
    }

    pub fn apply_patch(&mut self, patch: BenchmarkArgsPatch) -> Result<()> {
        if let Some(value) = patch.steps {
            self.steps = value;
        }
        if let Some(value) = patch.sizes {
            self.sizes = value;
        }
        if let Some(value) = patch.noise_levels {
            self.noise_levels = value;
        }
        if let Some(value) = patch.scenarios {
            self.scenarios = value;
        }
        if let Some(value) = patch.multi_mode {
            self.multi_mode = value;
        }
        if let Some(value) = patch.monitored_modes {
            self.monitored_modes = value;
        }
        if let Some(value) = patch.mode_shapes {
            self.mode_shapes = value;
        }
        if let Some(value) = patch.predictor {
            self.predictor = value;
        }
        if let Some(value) = patch.trust_mode {
            self.trust_mode = value;
        }
        if let Some(value) = patch.output_root {
            self.output_root = value;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ReportArgsPatch {
    pub run_dir: Option<PathBuf>,
    pub output_root: PathBuf,
}

impl From<ReportArgs> for ReportArgsPatch {
    fn from(value: ReportArgs) -> Self {
        Self {
            run_dir: value.run_dir,
            output_root: value.output_root.unwrap_or_else(default_output_root),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedCommand {
    Run(RunConfig),
    Quickstart(RunConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunArgsPatch {
    pub scenario: Option<ScenarioKind>,
    pub steps: Option<usize>,
    pub agents: Option<usize>,
    pub dt: Option<f64>,
    pub interaction_radius: Option<f64>,
    pub k_neighbors: Option<usize>,
    pub base_gain: Option<f64>,
    pub noise_level: Option<f64>,
    pub warmup_steps: Option<usize>,
    pub multi_mode: Option<bool>,
    pub monitored_modes: Option<usize>,
    pub mode_shapes: Option<bool>,
    pub predictor: Option<PredictorKind>,
    pub trust_mode: Option<TrustGateMode>,
    pub output_root: Option<PathBuf>,
}

impl From<RunArgs> for RunArgsPatch {
    fn from(value: RunArgs) -> Self {
        Self {
            scenario: value.scenario,
            steps: value.steps,
            agents: value.agents,
            dt: value.dt,
            interaction_radius: value.interaction_radius,
            k_neighbors: value.k_neighbors,
            base_gain: value.base_gain,
            noise_level: value.noise,
            warmup_steps: value.warmup_steps,
            multi_mode: value.multi_mode.then_some(true),
            monitored_modes: value.modes,
            mode_shapes: value.mode_shapes.then_some(true),
            predictor: value.predictor,
            trust_mode: value.trust_mode,
            output_root: value.output_root,
        }
    }
}

impl From<ScenarioArgs> for RunArgsPatch {
    fn from(value: ScenarioArgs) -> Self {
        RunArgsPatch::from(value.inner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkArgsPatch {
    pub steps: Option<usize>,
    pub sizes: Option<Vec<usize>>,
    pub noise_levels: Option<Vec<f64>>,
    pub scenarios: Option<Vec<ScenarioKind>>,
    pub multi_mode: Option<bool>,
    pub monitored_modes: Option<usize>,
    pub mode_shapes: Option<bool>,
    pub predictor: Option<PredictorKind>,
    pub trust_mode: Option<TrustGateMode>,
    pub output_root: Option<PathBuf>,
}

impl BenchmarkArgsPatch {
    pub fn try_from_args(value: BenchmarkArgs) -> Result<Self> {
        Ok(Self {
            steps: value.steps,
            sizes: value.sizes.as_deref().map(parse_usize_list).transpose()?,
            noise_levels: value.noise.as_deref().map(parse_f64_list).transpose()?,
            scenarios: parse_scenarios_argument(value.scenarios.as_deref(), value.all_scenarios)?,
            multi_mode: value.multi_mode.then_some(true),
            monitored_modes: value.modes,
            mode_shapes: value.mode_shapes.then_some(true),
            predictor: value.predictor,
            trust_mode: value.trust_mode,
            output_root: value.output_root,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileConfig {
    pub run: Option<RunArgsPatch>,
    pub benchmark: Option<BenchmarkArgsPatch>,
}

impl FileConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();
        match extension {
            "json" => serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse JSON config {}", path.display())),
            "toml" => toml::from_str(&raw)
                .with_context(|| format!("failed to parse TOML config {}", path.display())),
            _ => bail!("config file must use .json or .toml extension"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RunDirectory {
    pub output_root: PathBuf,
    pub timestamp: String,
    pub run_dir: PathBuf,
}

pub fn default_output_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-swarm")
}

pub fn create_timestamped_run_directory(root: &Path) -> Result<RunDirectory> {
    fs::create_dir_all(root)
        .with_context(|| format!("failed to create output root {}", root.display()))?;
    loop {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let run_dir = root.join(&timestamp);
        if !run_dir.exists() {
            fs::create_dir_all(&run_dir)
                .with_context(|| format!("failed to create run directory {}", run_dir.display()))?;
            return Ok(RunDirectory {
                output_root: root.to_path_buf(),
                timestamp,
                run_dir,
            });
        }
        thread::sleep(Duration::from_millis(1100));
    }
}

fn parse_usize_list(input: &str) -> Result<Vec<usize>> {
    input
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .trim()
                .parse::<usize>()
                .with_context(|| format!("failed to parse usize list item '{value}'"))
        })
        .collect()
}

fn parse_f64_list(input: &str) -> Result<Vec<f64>> {
    input
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .trim()
                .parse::<f64>()
                .with_context(|| format!("failed to parse f64 list item '{value}'"))
        })
        .collect()
}

fn parse_scenarios_argument(input: Option<&str>, all_scenarios: bool) -> Result<Option<Vec<ScenarioKind>>> {
    if all_scenarios {
        return Ok(Some(vec![
            ScenarioKind::Nominal,
            ScenarioKind::GradualEdgeDegradation,
            ScenarioKind::AdversarialAgent,
            ScenarioKind::CommunicationLoss,
        ]));
    }
    input
        .map(|value| {
            let mut scenarios = Vec::new();
            for item in value.split(',').filter(|item| !item.trim().is_empty()) {
                let parsed = match item.trim() {
                    "nominal" => ScenarioKind::Nominal,
                    "gradual_edge_degradation" | "gradual-edge-degradation" => {
                        ScenarioKind::GradualEdgeDegradation
                    }
                    "adversarial_agent" | "adversarial-agent" => ScenarioKind::AdversarialAgent,
                    "communication_loss" | "communication-loss" => ScenarioKind::CommunicationLoss,
                    "all" => ScenarioKind::All,
                    other => bail!("unrecognized scenario list item '{other}'"),
                };
                scenarios.extend(parsed.executable_scenarios());
            }
            Ok(scenarios)
        })
        .transpose()
}
