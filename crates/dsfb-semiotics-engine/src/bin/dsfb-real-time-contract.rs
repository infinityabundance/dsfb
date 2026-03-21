#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};
use dsfb_semiotics_engine::build_real_time_contract_summary;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Generate a machine-readable summary of the bounded live-path real-time contract"
)]
struct Args {
    #[arg(
        long,
        value_hint = ValueHint::FilePath,
        default_value = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/docs/generated/real_time_contract_summary.json"
        )
    )]
    output_json: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let summary = build_real_time_contract_summary();
    if let Some(parent) = args.output_json.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.output_json, serde_json::to_vec_pretty(&summary)?)?;
    println!("real_time_contract_summary={}", args.output_json.display());
    Ok(())
}
