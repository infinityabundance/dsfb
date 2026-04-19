#![forbid(unsafe_code)]

//! # dsfb-database
//!
//! Deterministic, read-only structural observer over residual trajectories in
//! SQL database telemetry. Built on the [`dsfb`] core (Drift–Slew Fusion
//! Bootstrap) by R. de Beer (2026); this crate adds only the
//! database-specific residual *construction* and motif *grammar*.
//!
//! ## What this crate is
//!
//! `dsfb-database` consumes residual streams that production SQL engines
//! already emit (`pg_stat_statements`, `pg_stat_io`, `pg_stat_activity`;
//! `sys.dm_exec_query_stats`, `sys.query_store_*`; MySQL Performance Schema;
//! Oracle ASH/AWR/`V$SQL_PLAN_STATISTICS_ALL`) and structures them into a
//! small grammar of operator-legible *motif episodes*:
//!
//!   1. plan-regression onset
//!   2. cardinality-mismatch regime
//!   3. contention ramp
//!   4. cache / buffer collapse
//!   5. workload phase transition
//!
//! ## What this crate is NOT
//!
//! It does **not** optimise queries, replace the optimiser, modify execution
//! plans, change DBMS behaviour, or claim causal correctness. See
//! [`non_claims`] (re-exported below) — these strings are pinned by a
//! compile-time test (`tests/non_claim_lock.rs`).

pub mod adapters;
pub mod baselines;
pub mod grammar;
#[cfg(feature = "live-postgres")]
pub mod live;
pub mod live_mysql;
pub mod metrics;
pub mod metrics_exporter;
pub mod non_claims;
pub mod perturbation;
pub mod report;
pub mod residual;
pub mod streaming;

pub use grammar::{Episode, MotifClass, MotifEngine, MotifGrammar};
pub use residual::{ResidualClass, ResidualSample, ResidualStream};

/// The crate version as recorded in `Cargo.toml`. Embedded in every report so
/// that figures can be traced back to the exact code that produced them.
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
