// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{resolve_sbir_demo_output_dir, run_sbir_demo_workflow, SbirDemoOptions};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Reviewer-facing SBIR demo bundle over the existing dsfb-battery helper workflows"
)]
struct Cli {
    /// Primary NASA PCoE cell used for the audit trace bundle.
    #[arg(long, default_value = "B0005")]
    cell: String,

    /// Include the additive multi-cell comparison helper outputs.
    #[arg(long, default_value_t = false)]
    multicell: bool,

    /// Include the additive host-side resource trace helper outputs.
    #[arg(long, default_value_t = false)]
    trace_resources: bool,

    /// Directory containing NASA PCoE capacity CSV files.
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Output directory or bundle name. A single relative name is created under outputs/sbir_demo/.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Timing repeats passed to the resource trace helper when enabled.
    #[arg(long, default_value_t = 5)]
    timing_repeats: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = cli.data_dir.unwrap_or_else(|| crate_dir.join("data"));
    let output_dir = resolve_sbir_demo_output_dir(&crate_dir, cli.output);

    let result = run_sbir_demo_workflow(&SbirDemoOptions {
        crate_dir,
        data_dir,
        output_dir,
        primary_cell_id: cli.cell,
        include_multicell: cli.multicell,
        trace_resources: cli.trace_resources,
        timing_repeats: cli.timing_repeats,
    })?;

    println!(
        "SBIR demo bundle written to: {}",
        result.output_root.display()
    );
    println!(
        "Reviewer summary: {}",
        result.reviewer_summary_path.display()
    );
    println!("Manifest: {}", result.manifest_path.display());
    println!(
        "Primary audit trace: {}",
        result.primary_audit_trace_path.display()
    );
    Ok(())
}
