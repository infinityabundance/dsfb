// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{resolve_helper_output_dir, run_sota_comparison_workflow, PipelineConfig};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in engineer-facing comparison helper")]
struct Cli {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = 0.88)]
    tactical_margin_fraction: f64,
    #[arg(long, default_value_t = 20)]
    rul_alarm_horizon_cycles: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = cli.data_dir.unwrap_or_else(|| crate_dir.join("data"));
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/sota_comparison",
        "dsfb_battery_sota_comparison",
        cli.output,
    );

    let artifact = run_sota_comparison_workflow(
        &data_dir,
        &output_dir,
        &PipelineConfig::default(),
        cli.tactical_margin_fraction,
        cli.rul_alarm_horizon_cycles,
    )?;

    println!(
        "SOTA comparison helper completed for cells: {}",
        artifact.cells_included.join(", ")
    );
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
