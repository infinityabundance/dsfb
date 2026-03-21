#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use dsfb_semiotics_engine::live::{to_real, OnlineStructuralEngine};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Replay exactly one bounded live-engine transition from a serialized state snapshot"
)]
struct ReplayArgs {
    #[arg(long, value_hint = ValueHint::FilePath)]
    snapshot_in: PathBuf,

    #[arg(long)]
    sample_time: f64,

    #[arg(
        long,
        help = "Comma-separated residual values in channel order, for example `0.10,0.02,-0.01`"
    )]
    sample_values: String,

    #[arg(long, value_hint = ValueHint::FilePath)]
    snapshot_out: Option<PathBuf>,
}

fn parse_values(csv: &str) -> Result<Vec<f64>> {
    csv.split(',')
        .map(|token| {
            token
                .trim()
                .parse::<f64>()
                .map_err(|error| anyhow!("failed to parse sample value `{token}`: {error}"))
        })
        .collect()
}

fn main() -> Result<()> {
    let args = ReplayArgs::parse();
    let snapshot_bytes = fs::read(&args.snapshot_in)?;
    let mut engine = OnlineStructuralEngine::from_snapshot_binary(&snapshot_bytes)?;
    let values = parse_values(&args.sample_values)?;
    let real_values = values.into_iter().map(to_real).collect::<Vec<_>>();
    let status = engine.push_residual_sample(args.sample_time, &real_values)?;

    println!(
        "snapshot_schema={}",
        dsfb_semiotics_engine::live::LIVE_ENGINE_SNAPSHOT_SCHEMA_VERSION
    );
    println!("step={}", status.step);
    println!("time={}", status.time);
    println!("syntax={}", status.syntax_label);
    println!("grammar_reason={}", status.grammar_reason_text);
    println!("semantic_disposition={}", status.semantic_disposition);
    println!("trust_scalar={:.6}", status.trust_scalar);
    println!("numeric_mode={}", status.numeric_mode);

    if let Some(snapshot_out) = args.snapshot_out {
        fs::write(snapshot_out, engine.snapshot_binary()?)?;
    }

    Ok(())
}
