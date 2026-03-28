// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Additive helper binary for multi-cell NASA PCoE evaluation.

use clap::Parser;
use dsfb_battery::{resolve_helper_output_dir, run_multicell_workflow, PipelineConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in multi-cell NASA PCoE evaluation helper")]
struct Cli {
    /// Directory containing NASA cell CSV files such as nasa_b0005_capacity.csv
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Output directory for additive multi-cell artifacts
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Healthy window size in cycles (N_h)
    #[arg(long)]
    healthy_window: Option<usize>,

    /// Drift averaging window (W)
    #[arg(long)]
    drift_window: Option<usize>,

    /// Drift persistence length (L_d)
    #[arg(long)]
    drift_persistence: Option<usize>,

    /// Slew persistence length (L_s)
    #[arg(long)]
    slew_persistence: Option<usize>,

    /// Drift threshold (θ_d)
    #[arg(long)]
    drift_threshold: Option<f64>,

    /// Slew threshold (θ_s)
    #[arg(long)]
    slew_threshold: Option<f64>,

    /// End-of-life fraction of initial capacity
    #[arg(long)]
    eol_fraction: Option<f64>,

    /// Boundary fraction for grammar classification
    #[arg(long)]
    boundary_fraction: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = cli.data_dir.unwrap_or_else(|| crate_dir.join("data"));
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "multicell",
        "dsfb_battery_multicell",
        cli.output,
    );

    let mut config = PipelineConfig::default();
    if let Some(value) = cli.healthy_window {
        config.healthy_window = value;
    }
    if let Some(value) = cli.drift_window {
        config.drift_window = value;
    }
    if let Some(value) = cli.drift_persistence {
        config.drift_persistence = value;
    }
    if let Some(value) = cli.slew_persistence {
        config.slew_persistence = value;
    }
    if let Some(value) = cli.drift_threshold {
        config.drift_threshold = value;
    }
    if let Some(value) = cli.slew_threshold {
        config.slew_threshold = value;
    }
    if let Some(value) = cli.eol_fraction {
        config.eol_fraction = value;
    }
    if let Some(value) = cli.boundary_fraction {
        config.boundary_fraction = value;
    }

    let artifact = run_multicell_workflow(&data_dir, &output_dir, &config)?;
    println!(
        "Multi-cell helper completed for cells: {}",
        artifact.cells_included.join(", ")
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
