//! Load a residual CSV from disk, run paper-lock end-to-end, emit
//! the JSON aggregate to stdout.
//!
//! Run with:
//!     cargo run --example load_csv_emit_json --features std,paper_lock
//!
//! Demonstrates the std-side API: [`paper_lock::run_real_data_with_csv_path`]
//! consumes a CSV at an arbitrary path (any single-column-residual
//! schema), runs the canonical paper-lock pipeline, and returns a
//! [`paper_lock::PaperLockReport`] which [`paper_lock::serialize_report`]
//! pretty-prints to deterministic JSON.
//!
//! This is the building block external observer pipelines can call to
//! turn a residual stream into a structural annotation alongside their
//! own pipeline's summary statistics.
#![cfg(all(feature = "std", feature = "paper_lock"))]

use std::path::PathBuf;

use dsfb_robotics::datasets::DatasetId;
use dsfb_robotics::paper_lock::{run_real_data_with_csv_path, serialize_report};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the published-θ̂ exemplar as the input; falls back to the
    // proxy CSV if the published one is absent. Try both crate-root and
    // workspace-root relative paths so the example works regardless of
    // the caller's cwd.
    let candidates = [
        PathBuf::from("data/processed/panda_gaz_published.csv"),
        PathBuf::from("data/processed/panda_gaz.csv"),
        PathBuf::from("crates/dsfb-robotics/data/processed/panda_gaz_published.csv"),
        PathBuf::from("crates/dsfb-robotics/data/processed/panda_gaz.csv"),
    ];
    let csv = candidates
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("data/processed/panda_gaz.csv"));

    if !csv.is_file() {
        eprintln!(
            "skipping: residual CSV not found at {} (run \
             python3 scripts/preprocess_datasets.py first)",
            csv.display()
        );
        return Ok(());
    }

    let report = match run_real_data_with_csv_path(DatasetId::PandaGaz, false, &csv) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("paper-lock: {} ({})", e.dataset.slug(), e.instructions);
            return Ok(());
        }
    };
    let json = serialize_report(&report)?;
    print!("{json}");
    Ok(())
}
