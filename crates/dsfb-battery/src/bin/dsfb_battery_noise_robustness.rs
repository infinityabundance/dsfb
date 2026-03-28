// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{resolve_helper_output_dir, run_noise_robustness_workflow, PipelineConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in engineer-facing noise robustness helper")]
struct Cli {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    noise_levels: Vec<f64>,
    #[arg(long, default_value_t = 0.88)]
    tactical_margin_fraction: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = cli.data_dir.unwrap_or_else(|| crate_dir.join("data"));
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/noise_robustness",
        "dsfb_battery_noise_robustness",
        cli.output,
    );

    let artifact = run_noise_robustness_workflow(
        &data_dir,
        &output_dir,
        &PipelineConfig::default(),
        if cli.noise_levels.is_empty() {
            &[0.01, 0.02, 0.05]
        } else {
            &cli.noise_levels
        },
        cli.tactical_margin_fraction,
    )?;

    println!(
        "Noise robustness helper completed for cells: {}",
        artifact.cells_included.join(", ")
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
