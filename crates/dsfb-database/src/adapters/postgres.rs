//! PostgreSQL `pg_stat_statements` adapter — real engine bridge.
//!
//! This is the only adapter in the crate that targets a *production engine*
//! rather than a public benchmark dataset. It is intentionally minimal: it
//! reads a CSV exported from `pg_stat_statements` snapshots and emits the
//! two residual classes that view alone supports — **plan-regression** and
//! **workload-phase**. It does **not** emit cardinality, contention, or
//! cache-I/O residuals; those require additional views (`EXPLAIN ANALYZE`,
//! `pg_stat_activity`, `pg_stat_io`) and are deferred to per-view adapters
//! we have not yet shipped. The honest deployability matrix in §11 of the
//! paper records these gaps.
//!
//! ## Expected CSV schema (PostgreSQL 14+)
//!
//! Export with:
//!
//! ```sql
//! \copy (
//!   SELECT
//!     extract(epoch from now())::float8 AS snapshot_t,
//!     md5(queryid::text)                AS query_id,
//!     calls                             AS calls,
//!     total_exec_time                   AS total_exec_time_ms
//!   FROM pg_stat_statements
//! ) TO '~/pgss_snapshot.csv' WITH CSV HEADER
//! ```
//!
//! at a regular interval (e.g. every 60 seconds), appending each snapshot
//! to the same file. The adapter expects exactly these columns; older
//! PostgreSQL releases (≤ 13) named the column `total_time` rather than
//! `total_exec_time` — pre-process those exports with `s/total_time/total_exec_time/`.
//! `query_id` is hashed with `md5()` so the export contains no query text.
//!
//! ## What the adapter computes
//!
//! For each `query_id`, snapshots are sorted by `snapshot_t` and consecutive
//! pairs produce one *mean-time-per-call* sample:
//!
//! ```text
//! Δexec  = total_exec_time_ms[t] − total_exec_time_ms[t-1]
//! Δcalls = calls[t] − calls[t-1]
//! mean   = Δexec / max(Δcalls, 1)
//! ```
//!
//! A per-`query_id` baseline is the mean of the first `BASELINE_WINDOW`
//! intervals; once the baseline is set, subsequent intervals push a
//! plan-regression residual via [`crate::residual::plan_regression::push_latency`].
//!
//! Workload-phase residuals are pushed once per snapshot timestamp: the
//! Shannon entropy of the per-snapshot call-share distribution across
//! `query_id`s, normalised to `[0, 1]` by dividing by `log(n_active_qids)`.
//! A drop in entropy (the workload concentrates on fewer query classes) is
//! the workload-phase signal documented in fact #16 of the concordance.

use crate::residual::{plan_regression, workload_phase, ResidualStream};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Number of intervals at the start of each query's history used to
/// establish its latency baseline. Picked to match the per-motif
/// `min_dwell_seconds=5.0` of `plan_regression_onset` on a 60-second
/// snapshot cadence: 3 intervals × 60 s = 180 s, enough to absorb the
/// initial warm-up without masking a real regression that begins later.
const BASELINE_WINDOW: usize = 3;

/// Upper bound on the number of distinct `query_id`s the adapter will hold.
/// `pg_stat_statements.max` defaults to 5000; this bound is a ~200×
/// safety headroom that still prevents unbounded-HashMap blow-up on a
/// corrupted snapshot.
const MAX_QIDS: usize = 1_000_000;

/// Upper bound on the number of CSV rows the adapter reads. At a
/// 60-second snapshot cadence and 5000 distinct `query_id`s this is ~2
/// days of continuous collection — well beyond the ~hour-long analysis
/// windows the crate is evaluated on.
const MAX_PGSS_ROWS: usize = 100_000_000;

#[derive(Debug, serde::Deserialize)]
struct Row {
    snapshot_t: f64,
    query_id: String,
    calls: u64,
    total_exec_time_ms: f64,
}

/// Load a `pg_stat_statements` snapshot CSV and produce a residual stream
/// containing plan-regression + workload-phase samples. Errors if the file
/// is missing, the schema does not match, or fewer than two snapshots are
/// present (the adapter cannot compute a delta from a single snapshot).
pub fn load_pg_stat_statements(path: &Path) -> Result<ResidualStream> {
    let rows = read_and_filter_rows(path)?;
    debug_assert!(rows.len() >= 2, "post-condition: caller only sees ≥2 rows");

    let by_qid = group_and_sort_by_qid(rows);
    debug_assert!(
        !by_qid.is_empty(),
        "non-empty input must produce ≥1 qid group"
    );

    let snapshot_times = collect_unique_snapshot_times(&by_qid);
    debug_assert!(
        !snapshot_times.is_empty(),
        "≥2 rows must contribute ≥1 timestamp"
    );
    let t0 = *snapshot_times.first().unwrap_or(&0.0);

    let mut stream = ResidualStream::new(format!(
        "postgres-pgss@{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    ));

    // `.take(MAX_QIDS)` is an explicit finite-source bound: `qids_sorted` is
    // bounded above by `by_qid.len()`, which is ≤ number of distinct
    // `query_id`s in the CSV, which `MAX_QIDS` additionally caps.
    let mut qids_sorted: Vec<&String> = by_qid.keys().take(MAX_QIDS).collect();
    qids_sorted.sort();
    debug_assert!(qids_sorted.len() <= MAX_QIDS, "iterator bound enforced");
    debug_assert_eq!(
        qids_sorted.len(),
        by_qid.len(),
        "sorted view must cover all qids"
    );

    emit_plan_regression_residuals(&mut stream, &by_qid, &qids_sorted, t0);
    emit_workload_phase_residuals(&mut stream, &by_qid, &qids_sorted, &snapshot_times, t0);

    stream.sort();
    Ok(stream)
}

/// Read every row of the CSV, dropping rows with non-finite floats, and
/// enforce the ≥2-row precondition so downstream analysis can assume it.
fn read_and_filter_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening pg_stat_statements csv at {}", path.display()))?;
    let mut rows: Vec<Row> = Vec::new();
    for r in rdr.deserialize().take(MAX_PGSS_ROWS) {
        debug_assert!(rows.len() < MAX_PGSS_ROWS, "row-count bound enforced");
        let r: Row = r.context("parsing pg_stat_statements row")?;
        if !r.snapshot_t.is_finite() || !r.total_exec_time_ms.is_finite() {
            continue;
        }
        rows.push(r);
    }
    if rows.len() < 2 {
        anyhow::bail!(
            "pg_stat_statements csv at {} has fewer than 2 rows; need ≥2 snapshots to compute deltas",
            path.display()
        );
    }
    Ok(rows)
}

/// Group rows by `query_id` and sort each group by `snapshot_t`.
fn group_and_sort_by_qid(rows: Vec<Row>) -> HashMap<String, Vec<Row>> {
    let mut by_qid: HashMap<String, Vec<Row>> = HashMap::new();
    for r in rows.into_iter() {
        by_qid.entry(r.query_id.clone()).or_default().push(r);
    }
    for v in by_qid.values_mut() {
        debug_assert!(
            !v.is_empty(),
            "inserted groups are non-empty by construction"
        );
        v.sort_by(|a, b| {
            a.snapshot_t
                .partial_cmp(&b.snapshot_t)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    by_qid
}

/// Collect the unique (to 1e-9 tolerance) sorted snapshot timestamps
/// present across every qid group.
fn collect_unique_snapshot_times(by_qid: &HashMap<String, Vec<Row>>) -> Vec<f64> {
    let mut snapshot_times: Vec<f64> = by_qid
        .values()
        .flat_map(|v| v.iter().map(|r| r.snapshot_t))
        .collect();
    snapshot_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    snapshot_times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    snapshot_times
}

/// Plan-regression residuals: per-query mean-exec-time-per-call deltas
/// versus a per-query baseline established from the first
/// `BASELINE_WINDOW` intervals. Iterates qids in sorted order so the
/// resulting stream is bytewise identical across runs (HashMap
/// iteration order is not).
fn emit_plan_regression_residuals(
    stream: &mut ResidualStream,
    by_qid: &HashMap<String, Vec<Row>>,
    qids_sorted: &[&String],
    t0: f64,
) {
    for qid in qids_sorted.iter() {
        let qid: &String = qid;
        let snaps = &by_qid[qid];
        if snaps.len() < 2 {
            continue;
        }
        let means = per_query_mean_exec_time(snaps, t0);
        if means.len() <= BASELINE_WINDOW {
            continue;
        }
        debug_assert!(
            means.len() > BASELINE_WINDOW,
            "post-filter invariant guaranteed by the early-return above"
        );
        let baseline: f64 = means
            .iter()
            .take(BASELINE_WINDOW)
            .map(|(_, m)| *m)
            .sum::<f64>()
            / BASELINE_WINDOW as f64;
        debug_assert!(
            baseline.is_finite(),
            "baseline from filtered finite samples"
        );
        for (i, (t_rel, mean)) in means.iter().enumerate() {
            if i < BASELINE_WINDOW {
                continue;
            }
            plan_regression::push_latency(stream, *t_rel, qid, *mean, baseline);
        }
    }
}

/// Compute (`t_rel`, `mean_exec_time_per_call`) for every adjacent
/// snapshot pair. Drops pairs with zero new calls or negative delta.
fn per_query_mean_exec_time(snaps: &[Row], t0: f64) -> Vec<(f64, f64)> {
    let mut means: Vec<(f64, f64)> = Vec::with_capacity(snaps.len().saturating_sub(1));
    for w in snaps.windows(2) {
        let dt = w[1].total_exec_time_ms - w[0].total_exec_time_ms;
        let dc = w[1].calls.saturating_sub(w[0].calls);
        if dc == 0 || dt < 0.0 {
            continue;
        }
        let mean = dt / dc as f64;
        debug_assert!(
            mean.is_finite() && mean >= 0.0,
            "dt≥0 ∧ dc>0 ⇒ finite non-negative mean"
        );
        let t_rel = w[1].snapshot_t - t0;
        means.push((t_rel, mean));
    }
    means
}

/// Workload-phase residuals: per-snapshot share-distribution entropy
/// across query_ids. Pushes `1 − entropy/entropy_max` so a
/// *concentration* (entropy drops) maps to a positive residual the
/// workload-phase motif treats as drift in the same direction as
/// the TPC-DS JSD spikes.
fn emit_workload_phase_residuals(
    stream: &mut ResidualStream,
    by_qid: &HashMap<String, Vec<Row>>,
    qids_sorted: &[&String],
    snapshot_times: &[f64],
    t0: f64,
) {
    let mut prev_calls: HashMap<String, u64> = HashMap::new();
    let mut max_entropy_seen: f64 = 0.0;
    let mut snapshot_shares: Vec<(f64, f64)> = Vec::new();
    for &t in snapshot_times.iter() {
        let Some(entropy) = snapshot_entropy(by_qid, qids_sorted, t, &mut prev_calls) else {
            continue;
        };
        debug_assert!(
            entropy.is_finite() && entropy >= 0.0,
            "entropy finite non-negative"
        );
        max_entropy_seen = max_entropy_seen.max(entropy);
        snapshot_shares.push((t, entropy));
    }
    if max_entropy_seen <= 0.0 {
        return;
    }
    debug_assert!(
        max_entropy_seen.is_finite(),
        "non-zero max entropy must be finite"
    );
    for (t_abs, entropy) in snapshot_shares.into_iter() {
        let normalised = entropy / max_entropy_seen;
        let r = 1.0 - normalised;
        workload_phase::push_jsd(stream, t_abs - t0, "pgss_digest_mix", r);
    }
}

/// Entropy of the per-qid call-share distribution at timestamp `t`.
/// Returns `None` if no qid advanced a call at this snapshot (empty
/// distribution).
fn snapshot_entropy(
    by_qid: &HashMap<String, Vec<Row>>,
    qids_sorted: &[&String],
    t: f64,
    prev_calls: &mut HashMap<String, u64>,
) -> Option<f64> {
    let mut shares: Vec<f64> = Vec::new();
    let mut total: u64 = 0;
    for qid in qids_sorted.iter() {
        let snaps = &by_qid[*qid];
        if let Some(r) = snaps.iter().find(|r| (r.snapshot_t - t).abs() < 1e-9) {
            let prev = prev_calls.get(*qid).copied().unwrap_or(0);
            let delta = r.calls.saturating_sub(prev);
            prev_calls.insert((*qid).clone(), r.calls);
            if delta > 0 {
                shares.push(delta as f64);
                total += delta;
            }
        }
    }
    if total == 0 || shares.is_empty() {
        return None;
    }
    for s in shares.iter_mut() {
        *s /= total as f64;
    }
    let entropy: f64 = shares
        .iter()
        .filter(|s| **s > 0.0)
        .map(|s| -s * s.ln())
        .sum();
    Some(entropy)
}
