//! `dsfb-forensics` is the reference specification and forensic audit layer for the
//! Drift-Slew Fusion Bootstrap stack.
//!
//! The crate operationalizes theorem-bank ideas from the DSFB technical series:
//! `CORE-04` for trust-gated causal graphs, `CORE-08` for anomaly soundness,
//! `CORE-10` for deterministic stack composition, `DSFB-07` and `DSFB-08` for
//! residual consistency, `DSCD-05` and `DSCD-07` for admissible graph pruning,
//! and `TMTR-01` and `TMTR-04` for monotone trust descent and stabilization.

pub mod auditor;
pub mod benchmark;
pub mod cli;
pub mod complexity;
pub mod ekf;
pub mod fs;
pub mod graph;
pub mod input;
pub mod report;

pub use auditor::{
    infer_initial_state, AuditRun, ForensicAuditor, ForensicConfig, ForensicRunSummary,
};
pub use benchmark::{
    generate_trace as generate_benchmark_trace, write_trace_csv as write_benchmark_trace_csv,
    BenchmarkConfig, BenchmarkMetadata, BenchmarkScenario, BenchmarkWriteTrace,
};
pub use cli::{BaselineComparison, Cli, ReportFormat};
pub use fs::{create_run_directory, create_run_directory_at, RunDirectory};
pub use input::{load_trace, TraceDocument, TraceStep, TruthState};
pub use report::{award_seal, render_markdown_report, SealLevel};
