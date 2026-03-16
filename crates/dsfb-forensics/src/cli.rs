//! Command-line surface for `dsfb-forensics`.
//!
//! References: `CORE-10` for deterministic composition of the full audit stack and
//! `DSFB-06` for replayability of deterministic reconstruction from the same trace.

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Serialize;

/// Toggle for the EKF baseline observer used to detect silent failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
pub enum BaselineComparison {
    /// Run the EKF baseline and report silent failures.
    On,
    /// Disable the EKF baseline and skip silent-failure reporting.
    Off,
}

impl BaselineComparison {
    /// References: `CORE-08` and `CORE-10`.
    pub fn enabled(self) -> bool {
        matches!(self, Self::On)
    }
}

/// Controls whether the run emits only markdown or markdown plus a JSON report.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ReportFormat {
    /// Emit the required `forensic_report.md`.
    Markdown,
    /// Emit `forensic_report.md` and `forensic_report.json`.
    Json,
    /// Emit both human-readable and machine-readable report views.
    Both,
}

impl ReportFormat {
    /// References: `CORE-10` and `DSFB-06`.
    pub fn writes_json(self) -> bool {
        matches!(self, Self::Json | Self::Both)
    }
}

/// Robust CLI for the forensic audit engine.
#[derive(Debug, Parser)]
#[command(
    name = "dsfb-forensics",
    version,
    about = "Reference specification and forensic audit layer for the DSFB stack"
)]
pub struct Cli {
    /// Path to a CSV or JSON trace file.
    #[arg(long, value_name = "PATH")]
    pub input_trace: PathBuf,

    /// Slew envelope threshold used by the shatter detector.
    #[arg(long, default_value_t = 6.0)]
    pub slew_threshold: f64,

    /// Trust floor below which an update is treated as structurally inconsistent.
    #[arg(long, default_value_t = 0.20)]
    pub trust_alpha: f64,

    /// Enable or disable the EKF baseline comparison.
    #[arg(long, value_enum, default_value_t = BaselineComparison::On)]
    pub baseline_comparison: BaselineComparison,

    /// Report view selection. Markdown is always written; JSON is optional.
    #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
    pub report_format: ReportFormat,
}
