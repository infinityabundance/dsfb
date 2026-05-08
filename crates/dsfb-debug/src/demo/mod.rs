//! DSFB-Debug demo module — gated behind the `demo` Cargo feature.
//!
//! # What this module does
//!
//! The `demo` module is the visualisation + reporting layer for the
//! `dsfb-debug-demo` binary (`src/bin/dsfb_debug_demo.rs`). Given the
//! 12 vendored fixtures under `data/fixtures/`, it:
//!
//! 1. Reads each fixture and runs the engine via
//!    `evaluate_real_dataset` plus `run_fusion_evaluation`.
//! 2. Renders 10 figures per fixture (residual heatmap, per-signal
//!    time-series, drift / slew per-signal bars, grammar-state matrix,
//!    consensus heatmap, per-detector firing fractions, episode
//!    timeline, evidence-packet radar, replay-verification).
//! 3. Renders 3 architecture / infrastructure figures (pipeline
//!    flowchart, per-tier detector count, motif × tier affinity).
//! 4. Renders 3 cross-fixture summary figures (RSCR scatter, fusion
//!    sweep curve, FP-rate comparison).
//! 5. Assembles a single PDF report with all figures + captions.
//! 6. Zips everything into `dsfb-debug-{datetime}.zip`.
//!
//! # Output structure
//!
//! ```text
//! output-dsfb-debug/
//! └── dsfb-debug-{ISO-datetime}/
//!     ├── figures/
//!     │   ├── 00_architecture/  (3 figures)
//!     │   ├── 01_F04/  (10 figures)
//!     │   ├── ... (12 fixtures)
//!     │   └── cross_fixture/  (3 figures)
//!     ├── results/  (12 JSON metric blocks)
//!     └── report.pdf
//! dsfb-debug-{ISO-datetime}.zip
//! ```
//!
//! # Discipline
//!
//! - Every figure is rendered from real upstream-bytes fixture data;
//!   no synthetic data, no extrapolation.
//! - JSON metric blocks are verbatim from the engine's
//!   `BenchmarkMetrics` output; no rounding, no smoothing.
//! - Captions are auto-generated from fixture metadata + per-figure
//!   semantics; ≤ 80 chars per line, positioned below figure.

#![cfg(feature = "demo")]

extern crate std;
extern crate alloc;

pub mod figures;
pub mod infrastructure;
pub mod pdf_report;
pub mod runner;

pub use runner::run_demo;
