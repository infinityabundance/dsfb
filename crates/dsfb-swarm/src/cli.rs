use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::{PredictorKind, ScenarioKind, TrustGateMode};

#[derive(Debug, Parser)]
#[command(
    name = "dsfb-swarm",
    version,
    about = "Deterministic spectral residual inference demonstrator for swarm interaction networks"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run(RunArgs),
    Scenario(ScenarioArgs),
    Benchmark(BenchmarkArgs),
    Quickstart(QuickstartArgs),
    Report(ReportArgs),
}

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub scenario: Option<ScenarioKind>,
    #[arg(long)]
    pub steps: Option<usize>,
    #[arg(long)]
    pub agents: Option<usize>,
    #[arg(long)]
    pub dt: Option<f64>,
    #[arg(long)]
    pub interaction_radius: Option<f64>,
    #[arg(long)]
    pub k_neighbors: Option<usize>,
    #[arg(long)]
    pub base_gain: Option<f64>,
    #[arg(long)]
    pub noise: Option<f64>,
    #[arg(long)]
    pub warmup_steps: Option<usize>,
    #[arg(long)]
    pub multi_mode: bool,
    #[arg(long)]
    pub modes: Option<usize>,
    #[arg(long)]
    pub mode_shapes: bool,
    #[arg(long, value_enum)]
    pub predictor: Option<PredictorKind>,
    #[arg(long, value_enum)]
    pub trust_mode: Option<TrustGateMode>,
    #[arg(long)]
    pub output_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ScenarioArgs {
    #[command(flatten)]
    pub inner: RunArgs,
}

#[derive(Debug, Clone, Args)]
pub struct QuickstartArgs {
    #[arg(long)]
    pub output_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct BenchmarkArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub steps: Option<usize>,
    #[arg(long)]
    pub sizes: Option<String>,
    #[arg(long)]
    pub noise: Option<String>,
    #[arg(long)]
    pub scenarios: Option<String>,
    #[arg(long)]
    pub all_scenarios: bool,
    #[arg(long)]
    pub multi_mode: bool,
    #[arg(long)]
    pub mode_shapes: bool,
    #[arg(long)]
    pub modes: Option<usize>,
    #[arg(long, value_enum)]
    pub predictor: Option<PredictorKind>,
    #[arg(long, value_enum)]
    pub trust_mode: Option<TrustGateMode>,
    #[arg(long)]
    pub output_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct ReportArgs {
    #[arg(long)]
    pub run_dir: Option<PathBuf>,
    #[arg(long)]
    pub output_root: Option<PathBuf>,
}
