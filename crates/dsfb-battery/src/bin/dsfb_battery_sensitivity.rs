// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{
    load_capacity_csv, resolve_helper_output_dir, run_sensitivity_workflow, PipelineConfig,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in engineer-facing sensitivity analysis helper")]
struct Cli {
    #[arg(long)]
    data: Option<PathBuf>,
    #[arg(long, default_value = "B0005")]
    cell_id: String,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    drift_window_values: Vec<usize>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    drift_persistence_values: Vec<usize>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    slew_persistence_values: Vec<usize>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    sigma_multipliers: Vec<f64>,
    #[arg(long, default_value_t = 0.88)]
    tactical_margin_fraction: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_path = cli.data.unwrap_or_else(|| {
        crate_dir
            .join("data")
            .join(format!("nasa_{}_capacity.csv", cli.cell_id.to_lowercase()))
    });
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/sensitivity",
        "dsfb_battery_sensitivity",
        cli.output,
    );
    let raw_data = load_capacity_csv(&data_path)?;

    let artifact = run_sensitivity_workflow(
        &cli.cell_id,
        data_path.to_string_lossy().as_ref(),
        &raw_data,
        &PipelineConfig::default(),
        if cli.drift_window_values.is_empty() {
            &[3, 5, 7]
        } else {
            &cli.drift_window_values
        },
        if cli.drift_persistence_values.is_empty() {
            &[8, 12, 16]
        } else {
            &cli.drift_persistence_values
        },
        if cli.slew_persistence_values.is_empty() {
            &[4, 8, 12]
        } else {
            &cli.slew_persistence_values
        },
        if cli.sigma_multipliers.is_empty() {
            &[2.5, 3.0, 3.5]
        } else {
            &cli.sigma_multipliers
        },
        cli.tactical_margin_fraction,
        &output_dir,
    )?;

    println!(
        "Sensitivity helper completed for {} with {} scenarios",
        artifact.cell_id,
        artifact.scenarios.len()
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
