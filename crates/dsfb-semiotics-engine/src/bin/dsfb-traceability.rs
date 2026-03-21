use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use dsfb_semiotics_engine::traceability::{
    check_traceability_matrix_fresh, default_matrix_path, write_traceability_matrix,
};

#[derive(Parser, Debug)]
#[command(
    name = "dsfb-traceability",
    about = "Generate or freshness-check the theorem-to-code traceability matrix"
)]
struct TraceabilityArgs {
    /// Check that the committed matrix matches the current source tags without rewriting it.
    #[arg(long)]
    check: bool,

    /// Override the output matrix path. Defaults to docs/THEOREM_TO_CODE_TRACEABILITY.md.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = TraceabilityArgs::parse();
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if args.check {
        check_traceability_matrix_fresh(&crate_root)?;
        println!("traceability_status=fresh");
        return Ok(());
    }

    let output = args
        .output
        .unwrap_or_else(|| default_matrix_path(&crate_root));
    write_traceability_matrix(&crate_root, &output)?;
    println!("traceability_matrix={}", output.display());
    Ok(())
}
