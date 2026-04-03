// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{
    load_capacity_csv, resolve_helper_output_dir, run_resource_trace_workflow, PipelineConfig,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in resource integrity and audit helper")]
struct Cli {
    /// Path to a NASA capacity CSV file. Defaults to the B0005 capacity series.
    #[arg(long)]
    data: Option<PathBuf>,

    /// Explicit output directory override.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Number of repeated host measurements used for timing averages.
    #[arg(long, default_value_t = 5)]
    timing_repeats: usize,

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

    /// End-of-life fraction of initial capacity.
    #[arg(long)]
    eol_fraction: Option<f64>,

    /// Boundary fraction for grammar classification.
    #[arg(long)]
    boundary_fraction: Option<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_path = cli
        .data
        .unwrap_or_else(|| crate_dir.join("data").join("nasa_b0005_capacity.csv"));

    let _ = load_capacity_csv(&data_path)?;

    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "resource_trace",
        "dsfb_battery_resource_trace",
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

    let artifacts = run_resource_trace_workflow(
        &crate_dir,
        &data_path,
        &output_dir,
        &config,
        cli.timing_repeats,
    )?;

    println!(
        "Resource trace helper completed for {}",
        artifacts.resource_trace.run_summary.cell_id
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
