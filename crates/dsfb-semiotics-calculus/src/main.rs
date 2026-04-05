//! # `dsfb-semiotics-calculus` — CLI binary
//!
//! Generates the ten canonical DSSC figures (`fig01…fig10`), a JSON summary,
//! a Markdown report, and a ZIP bundle — all placed in `./dsfb-sc-artifacts/`
//! (or a directory specified with `--output`).
//!
//! ## Usage
//!
//! ```text
//! dsfb-semiotics-calculus [--output <dir>]
//! ```
//!
//! Exit codes:
//! - 0: all artifacts written successfully
//! - 1: I/O error (details printed to stderr)

use std::path::PathBuf;
use std::process;

use dsfb_semiotics_calculus::figures::write_all_artifacts;

fn main() {
    let out_dir = parse_output_arg().unwrap_or_else(|| PathBuf::from("dsfb-sc-artifacts"));

    eprintln!(
        "dsfb-semiotics-calculus v{} — generating artifacts in {:?}",
        env!("CARGO_PKG_VERSION"),
        out_dir
    );

    match write_all_artifacts(&out_dir) {
        Ok(files) => {
            eprintln!("\nArtifacts written ({} files):", files.len());
            for f in &files {
                eprintln!("  {}", f.display());
            }
            eprintln!("\nDone. ZIP bundle: {}/dsfb-semiotics-calculus-artifacts.zip", out_dir.display());
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error writing artifacts: {}", e);
            process::exit(1);
        }
    }
}

fn parse_output_arg() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--output" || args[i] == "-o" {
            if i + 1 < args.len() {
                return Some(PathBuf::from(&args[i + 1]));
            }
        } else if let Some(path) = args[i].strip_prefix("--output=") {
            return Some(PathBuf::from(path));
        }
        i += 1;
    }
    None
}
