// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{resolve_helper_output_dir, verify_heuristics_bank};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Opt-in heuristics-bank integrity verifier")]
struct Cli {
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cli = Cli::parse();
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/heuristics",
        "dsfb_battery_heuristics_verification",
        cli.output,
    );
    std::fs::create_dir_all(&output_dir)?;

    let verification = verify_heuristics_bank(&crate_dir)?;
    std::fs::write(
        output_dir.join("heuristics_bank_verification.json"),
        serde_json::to_string_pretty(&verification)?,
    )?;

    println!("Heuristics bank verified: {}", verification.verified);
    println!("Artifacts written to: {}", output_dir.display());
    Ok(())
}
