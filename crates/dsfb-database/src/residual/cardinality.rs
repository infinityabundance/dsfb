//! Cardinality-mismatch residuals.
//!
//! Per the paperstack (fact #1: *estimated rows vs actual rows is the canonical
//! SQL residual*), this is the residual class with the strongest claim to
//! existing as a deterministic, almost-noise-free signal — the optimiser
//! itself produced both numbers from the same query.
//!
//! We store `log10(actual_rows / max(estimated_rows, 1))`, which puts a
//! 10× under-estimate at +1.0 and a 10× over-estimate at −1.0. This is the
//! "q-error in log-space" used throughout the cardinality-estimation
//! literature (Leis et al. 2015, *How Good Are Query Optimizers, Really?*).
//!
//! Available natively in:
//!   * SQL Server `sys.dm_exec_query_plan` (`EstimateRows` vs `ActualRows`)
//!   * Oracle `V$SQL_PLAN_STATISTICS_ALL` (`OUTPUT_ROWS` vs plan estimate)
//!   * PostgreSQL `EXPLAIN (ANALYZE, BUFFERS)` row estimates vs actual
//!   * CEB: ground-truth cardinalities + PostgreSQL estimates
//!   * MySQL: only via slow-log + `EXPLAIN FORMAT=TREE` post-hoc

use super::{ResidualClass, ResidualSample, ResidualStream};

/// Push a cardinality residual. `qclass` is the query / subplan identifier.
pub fn push(
    stream: &mut ResidualStream,
    t: f64,
    qclass: &str,
    estimated_rows: f64,
    actual_rows: f64,
) {
    let est = estimated_rows.max(1.0);
    let act = actual_rows.max(1.0);
    let q = (act / est).log10();
    stream.push(
        ResidualSample::new(t, ResidualClass::Cardinality, q).with_channel(qclass),
    );
}
