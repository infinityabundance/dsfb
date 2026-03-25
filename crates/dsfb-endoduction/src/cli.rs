//! CLI argument definitions and orchestration.

use crate::types::Config;
use clap::Parser;
use std::path::PathBuf;

/// DSFB Endoduction: Structural residual analysis on NASA IMS bearings.
///
/// Evaluates the Thermodynamic Precursor Visibility Principle by testing
/// whether DSFB-style structural residual analysis detects precursor
/// behaviour earlier than conventional scalar diagnostics on real
/// run-to-failure bearing data.
#[derive(Debug, Parser, Clone)]
#[command(name = "dsfb-endoduction", version, about)]
pub struct Cli {
    /// Subcommand.
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Parser, Clone)]
pub enum Command {
    /// Run the full analysis pipeline.
    Run(RunArgs),
}

#[derive(Debug, Parser, Clone)]
pub struct RunArgs {
    /// Path to the root directory containing the IMS dataset.
    /// Expected structure: <data-root>/IMS/1st_test/, etc.
    #[arg(long, default_value = "data")]
    pub data_root: PathBuf,

    /// Bearing set to analyse (1, 2, or 3).
    #[arg(long, default_value_t = 1)]
    pub bearing_set: u32,

    /// Primary channel index (0-based).
    #[arg(long, default_value_t = 0)]
    pub channel: usize,

    /// Fraction of run used as nominal baseline (0.0..1.0).
    #[arg(long, default_value_t = 0.15)]
    pub nominal_fraction: f64,

    /// Quantile level for admissibility envelope.
    #[arg(long, default_value_t = 0.99)]
    pub envelope_quantile: f64,

    /// Consecutive windows required for sustained detection.
    #[arg(long, default_value_t = 5)]
    pub sustained: usize,

    /// Trust score threshold for precursor detection.
    #[arg(long, default_value_t = 0.5)]
    pub trust_threshold: f64,

    /// Output directory root.
    #[arg(long, default_value = "output-dsfb-endoduction")]
    pub output_dir: PathBuf,

    /// Download dataset if not present.
    #[arg(long)]
    pub download: bool,
}

impl RunArgs {
    /// Convert CLI args into a Config.
    pub fn to_config(&self) -> Config {
        Config {
            bearing_set: self.bearing_set,
            primary_channel: self.channel,
            window_size: 20480,
            nominal_fraction: self.nominal_fraction,
            envelope_quantile: self.envelope_quantile,
            sustained_count: self.sustained,
            trust_threshold: self.trust_threshold,
            output_dir: self.output_dir.clone(),
            seed: 42,
        }
    }
}
