//! Phase-C1: OpenTelemetry DB-spans ingestor (scaffold).
//!
//! This adapter consumes a JSON-array file of **database-operation
//! spans** — the span subset a production OTel-collector emits when an
//! application instruments `pg`, `mysql`, `mssql`, or similar drivers
//! with the standard OTel DB semantic conventions (`db.system`,
//! `db.statement`, `db.statement_hash`, etc.). The full OTLP/JSON
//! envelope is verbose (nested `resourceSpans → scopeSpans → spans`);
//! a production collector pipeline typically flattens DB spans with a
//! `transform` processor before export, and that flattened shape is
//! what this adapter expects.
//!
//! ## Expected JSON shape
//!
//! ```json
//! [
//!   {
//!     "t_start_ns": 1700000000000000000,
//!     "t_end_ns":   1700000000012345678,
//!     "statement_hash": "a1b2c3d4",
//!     "db_system": "postgresql"
//!   },
//!   ...
//! ]
//! ```
//!
//! Fields:
//!
//!   * `t_start_ns` / `t_end_ns` — span boundaries in Unix nanoseconds
//!     (OTel standard). The adapter uses the delta as the per-span
//!     duration and the start as the residual timestamp.
//!   * `statement_hash` — per-statement identifier; maps to the motif
//!     grammar's `channel` discriminator. Use `db.statement_hash` from
//!     OTel, or any stable hash you compute at instrumentation time.
//!   * `db_system` — optional; included in the stream source label so
//!     downstream reports can segment by engine.
//!
//! The mapping from spans → residuals mirrors the `pg_stat_statements`
//! adapter: per-`statement_hash` rolling baseline over the first
//! `BASELINE_WINDOW` spans, then every subsequent span pushes a
//! `plan_regression` residual through
//! [`plan_regression::push_latency`](crate::residual::plan_regression::push_latency).
//! We do **not** emit cardinality / contention / cache residuals: the
//! DB-span semantic conventions expose none of those signals
//! directly, and inferring them would violate the adapter's no-
//! inference invariant documented in [`super`].
//!
//! ## What this scaffold is NOT
//!
//! This is an *offline file-reading adapter*, not a live OTLP gRPC
//! server. A production deployment would run `otel-collector` with a
//! file exporter, write rotated JSON batches to disk, and point this
//! adapter (or its streaming variant) at the batches. Wiring up
//! live OTLP ingest requires `tonic` + `opentelemetry-proto` and is
//! deferred to the pilot-deployment phase (see paper limitation 32).

use crate::residual::{plan_regression, ResidualStream};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Number of leading spans per `statement_hash` used to establish the
/// per-statement latency baseline. Matches
/// [`crate::adapters::postgres::load_pg_stat_statements`]'s
/// `BASELINE_WINDOW = 3`.
const BASELINE_WINDOW: usize = 3;

/// Upper bound on the number of statement_hashes the adapter will
/// hold. Matches the Postgres adapter's `MAX_QIDS`.
const MAX_STATEMENT_HASHES: usize = 1_000_000;

/// Upper bound on the number of spans the adapter will read.
const MAX_SPANS: usize = 100_000_000;

/// Minimum positive duration, in milliseconds, below which a span is
/// ignored. Protects against clock-skew-induced negative or zero
/// durations.
const MIN_DURATION_MS: f64 = 1e-6;

#[derive(Debug, Deserialize)]
pub struct DbSpan {
    pub t_start_ns: i128,
    pub t_end_ns: i128,
    pub statement_hash: String,
    #[serde(default)]
    pub db_system: Option<String>,
}

/// Load an OTel DB-spans JSON array and emit a
/// [`ResidualStream`] of `plan_regression` residuals. The stream
/// label is `"otel-db-spans@<filename>"` plus the first-seen
/// `db_system`, if any span carried one.
pub fn load_otel_db_spans(path: &Path) -> Result<ResidualStream> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading OTel DB-spans JSON at {}", path.display()))?;
    let spans: Vec<DbSpan> = serde_json::from_str(&text)
        .with_context(|| format!("parsing OTel DB-spans JSON at {}", path.display()))?;
    debug_assert!(spans.len() <= MAX_SPANS, "span-count bound enforced");
    if spans.len() > MAX_SPANS {
        anyhow::bail!(
            "OTel DB-spans JSON at {} exceeds {} span bound",
            path.display(),
            MAX_SPANS
        );
    }

    let mut by_stmt: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    let mut first_system: Option<String> = None;
    let mut t0_ns: Option<i128> = None;
    for s in &spans {
        let dur_ns = s.t_end_ns - s.t_start_ns;
        if dur_ns <= 0 {
            continue;
        }
        let dur_ms = dur_ns as f64 / 1e6;
        if !(dur_ms >= MIN_DURATION_MS && dur_ms.is_finite()) {
            continue;
        }
        let t_rel_s = match t0_ns {
            Some(t0) => (s.t_start_ns - t0) as f64 / 1e9,
            None => {
                t0_ns = Some(s.t_start_ns);
                0.0
            }
        };
        if first_system.is_none() {
            first_system.clone_from(&s.db_system);
        }
        let entries = by_stmt.entry(s.statement_hash.clone()).or_default();
        entries.push((t_rel_s, dur_ms));
        debug_assert!(by_stmt.len() <= MAX_STATEMENT_HASHES, "qid bound enforced");
    }
    if by_stmt.len() > MAX_STATEMENT_HASHES {
        anyhow::bail!(
            "OTel DB-spans JSON at {} has >{} distinct statement hashes",
            path.display(),
            MAX_STATEMENT_HASHES
        );
    }

    // Sort each per-stmt entry list by t so baseline uses the earliest
    // durations.
    for v in by_stmt.values_mut() {
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    let source = match first_system.as_deref() {
        Some(sys) => format!(
            "otel-db-spans@{}[system={}]",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
            sys
        ),
        None => format!(
            "otel-db-spans@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ),
    };
    let mut stream = ResidualStream::new(source);

    // Emit in sorted-hash order so the fingerprint is deterministic
    // across HashMap iteration orders.
    let mut hashes: Vec<&String> = by_stmt.keys().collect();
    hashes.sort();
    for stmt in hashes {
        let entries = &by_stmt[stmt];
        if entries.len() <= BASELINE_WINDOW {
            continue;
        }
        let baseline_ms: f64 = entries
            .iter()
            .take(BASELINE_WINDOW)
            .map(|(_, d)| *d)
            .sum::<f64>()
            / BASELINE_WINDOW as f64;
        debug_assert!(baseline_ms.is_finite() && baseline_ms > 0.0);
        for (i, (t_rel, dur_ms)) in entries.iter().enumerate() {
            if i < BASELINE_WINDOW {
                continue;
            }
            plan_regression::push_latency(&mut stream, *t_rel, stmt, *dur_ms, baseline_ms);
        }
    }

    stream.sort();
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(spans: &[DbSpan]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "[").unwrap();
        for (i, s) in spans.iter().enumerate() {
            let sep = if i + 1 < spans.len() { "," } else { "" };
            let sys = s
                .db_system
                .as_deref()
                .map(|x| format!(", \"db_system\":\"{}\"", x))
                .unwrap_or_default();
            writeln!(
                tmp,
                "  {{\"t_start_ns\":{},\"t_end_ns\":{},\"statement_hash\":\"{}\"{}}}{}",
                s.t_start_ns, s.t_end_ns, s.statement_hash, sys, sep
            )
            .unwrap();
        }
        writeln!(tmp, "]").unwrap();
        tmp.flush().unwrap();
        tmp
    }

    #[test]
    fn empty_span_list_produces_empty_stream() {
        let tmp = write_fixture(&[]);
        let stream = load_otel_db_spans(tmp.path()).unwrap();
        assert!(stream.samples.is_empty());
    }

    #[test]
    fn plan_regression_residuals_emit_after_baseline_window() {
        // 10 spans for one statement: first 3 at 10 ms baseline, next 7
        // at 30 ms (a clear 3× regression). Expect 7 plan_regression
        // residuals.
        let mut spans = Vec::new();
        let t0 = 1_700_000_000_000_000_000_i128;
        for i in 0..10 {
            let start = t0 + (i as i128) * 1_000_000_000;
            let dur_ns = if i < 3 { 10_000_000 } else { 30_000_000 };
            spans.push(DbSpan {
                t_start_ns: start,
                t_end_ns: start + dur_ns,
                statement_hash: "hot_stmt".to_string(),
                db_system: Some("postgresql".to_string()),
            });
        }
        let tmp = write_fixture(&spans);
        let stream = load_otel_db_spans(tmp.path()).unwrap();
        assert_eq!(stream.samples.len(), 7);
        assert!(stream.source.contains("system=postgresql"));
    }

    #[test]
    fn non_positive_duration_spans_are_dropped() {
        let t0 = 1_700_000_000_000_000_000_i128;
        let spans = vec![
            // Good baseline.
            DbSpan {
                t_start_ns: t0,
                t_end_ns: t0 + 10_000_000,
                statement_hash: "s".into(),
                db_system: None,
            },
            DbSpan {
                t_start_ns: t0 + 1_000_000_000,
                t_end_ns: t0 + 1_000_000_000 + 10_000_000,
                statement_hash: "s".into(),
                db_system: None,
            },
            DbSpan {
                t_start_ns: t0 + 2_000_000_000,
                t_end_ns: t0 + 2_000_000_000 + 10_000_000,
                statement_hash: "s".into(),
                db_system: None,
            },
            // Duration = 0 → dropped.
            DbSpan {
                t_start_ns: t0 + 3_000_000_000,
                t_end_ns: t0 + 3_000_000_000,
                statement_hash: "s".into(),
                db_system: None,
            },
            // Negative duration → dropped.
            DbSpan {
                t_start_ns: t0 + 4_000_000_000,
                t_end_ns: t0 + 3_999_999_999,
                statement_hash: "s".into(),
                db_system: None,
            },
        ];
        let tmp = write_fixture(&spans);
        let stream = load_otel_db_spans(tmp.path()).unwrap();
        // Baseline only; no residuals emitted (need > BASELINE_WINDOW
        // samples per stmt).
        assert_eq!(stream.samples.len(), 0);
    }

    #[test]
    fn per_statement_baselines_are_independent() {
        let t0 = 1_700_000_000_000_000_000_i128;
        let mut spans = Vec::new();
        // Statement "a": 4 spans at 20 ms each — one residual near zero.
        // Statement "b": 4 spans, first 3 at 10 ms, last at 100 ms — one
        // large residual.
        for i in 0..4 {
            spans.push(DbSpan {
                t_start_ns: t0 + (i as i128) * 1_000_000_000,
                t_end_ns: t0 + (i as i128) * 1_000_000_000 + 20_000_000,
                statement_hash: "a".into(),
                db_system: None,
            });
        }
        for i in 0..4 {
            let dur = if i < 3 { 10_000_000 } else { 100_000_000 };
            spans.push(DbSpan {
                t_start_ns: t0 + (i as i128) * 1_000_000_000 + 500_000_000,
                t_end_ns: t0 + (i as i128) * 1_000_000_000 + 500_000_000 + dur,
                statement_hash: "b".into(),
                db_system: None,
            });
        }
        let tmp = write_fixture(&spans);
        let stream = load_otel_db_spans(tmp.path()).unwrap();
        assert_eq!(stream.samples.len(), 2);
        let max_abs = stream
            .samples
            .iter()
            .map(|s| s.value.abs())
            .fold(0.0_f64, f64::max);
        // Statement "b" should dominate with a 10× regression.
        assert!(max_abs > 1.0, "max |residual| was {}", max_abs);
    }
}
