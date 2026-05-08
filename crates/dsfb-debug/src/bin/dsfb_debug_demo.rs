//! `dsfb-debug-demo` — single-shot reviewer reproducibility binary.
//!
//! Runs the engine end-to-end on every vendored fixture, renders 13
//! figures per fixture (10 per-fixture + 3 architecture + 3 cross-
//! fixture), assembles a single PDF report, and zips everything for
//! easy download.
//!
//! Output structure:
//!
//! ```text
//! output-dsfb-debug/
//! └── dsfb-debug-{ISO-datetime}/
//!     ├── figures/
//!     ├── results/
//!     └── report.pdf
//! dsfb-debug-{ISO-datetime}.zip
//! ```
//!
//! Usage:
//!
//! ```text
//! cargo install --path . --features demo
//! dsfb-debug-demo
//! ```
//!
//! Or, without installing:
//!
//! ```text
//! cargo run --release --features demo --bin dsfb-debug-demo
//! ```

#![cfg(feature = "demo")]

fn main() {
    match dsfb_debug::demo::run_demo() {
        Ok(zip_path) => {
            println!("OK: {}", zip_path.display());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("ERROR: {:?}", e);
            std::process::exit(1);
        }
    }
}
