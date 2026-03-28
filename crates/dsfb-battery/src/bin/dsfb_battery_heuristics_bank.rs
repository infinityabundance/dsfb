// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{
    resolve_helper_output_dir, run_nasa_heuristics_bank_workflow, PipelineConfig,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in NASA-grounded heuristics-bank helper")]
struct Cli {
    /// Directory containing NASA PCoE capacity CSV files.
    #[arg(short, long, default_value = "data")]
    data: PathBuf,

    /// Explicit output directory override.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cli = Cli::parse();

    let data_dir = if cli.data.is_absolute() {
        cli.data
    } else {
        crate_dir.join(cli.data)
    };
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/heuristics_bank",
        "dsfb_battery_heuristics_bank",
        cli.output,
    );

    let artifact = run_nasa_heuristics_bank_workflow(
        &data_dir,
        &output_dir,
        &PipelineConfig::default(),
    )?;

    println!(
        "NASA-grounded heuristics bank helper completed for cells: {}",
        artifact.summary.cells_evaluated.join(", ")
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
