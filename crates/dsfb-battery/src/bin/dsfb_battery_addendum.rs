// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{resolve_addendum_output_dir, run_addendum_workflow};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate isolated engineer-facing addendum artifacts for dsfb-battery"
)]
struct Args {
    /// Path to the NASA B0005 capacity CSV file
    #[arg(long)]
    data: Option<PathBuf>,

    /// Optional explicit output directory override
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_path = args
        .data
        .unwrap_or_else(|| crate_dir.join("data").join("nasa_b0005_capacity.csv"));
    let output_dir = resolve_addendum_output_dir(&crate_dir, args.output);

    let summary = run_addendum_workflow(&crate_dir, &data_path, &output_dir)?;

    println!("Addendum artifacts written to {}", output_dir.display());
    println!(
        "Implementation summary: {}",
        output_dir.join("implementation_summary.txt").display()
    );
    println!(
        "Production-path statement: {}",
        summary.no_production_modification_statement
    );
    Ok(())
}
