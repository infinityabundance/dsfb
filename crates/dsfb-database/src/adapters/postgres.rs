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
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening pg_stat_statements csv at {}", path.display()))?;
    let mut rows: Vec<Row> = Vec::new();
    for r in rdr.deserialize() {
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

    // Group by query_id; within each group sort by snapshot_t.
    let mut by_qid: HashMap<String, Vec<Row>> = HashMap::new();
    for r in rows {
        by_qid.entry(r.query_id.clone()).or_default().push(r);
    }
    for v in by_qid.values_mut() {
        v.sort_by(|a, b| a.snapshot_t.partial_cmp(&b.snapshot_t).unwrap_or(std::cmp::Ordering::Equal));
    }

    let mut snapshot_times: Vec<f64> = by_qid
        .values()
        .flat_map(|v| v.iter().map(|r| r.snapshot_t))
        .collect();
    snapshot_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    snapshot_times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let t0 = *snapshot_times.first().unwrap_or(&0.0);

    let mut stream = ResidualStream::new(format!(
        "postgres-pgss@{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    ));

    // Plan-regression residuals: per-query mean-exec-time-per-call deltas
    // versus a per-query baseline established from the first BASELINE_WINDOW
    // intervals. Iterate over qids in sorted order so the residual stream
    // is bytewise identical across runs (HashMap iteration order is not).
    let mut qids_sorted: Vec<&String> = by_qid.keys().collect();
    qids_sorted.sort();
    let mut qid_means: HashMap<String, Vec<f64>> = HashMap::new();
    for qid in &qids_sorted {
        let qid: &String = qid;
        let snaps = &by_qid[qid];
        if snaps.len() < 2 {
            continue;
        }
        let mut means: Vec<(f64, f64)> = Vec::new();
        for w in snaps.windows(2) {
            let dt = w[1].total_exec_time_ms - w[0].total_exec_time_ms;
            let dc = w[1].calls.saturating_sub(w[0].calls);
            if dc == 0 || dt < 0.0 {
                continue;
            }
            let mean = dt / dc as f64;
            let t_rel = w[1].snapshot_t - t0;
            means.push((t_rel, mean));
        }
        if means.len() <= BASELINE_WINDOW {
            continue;
        }
        let baseline: f64 =
            means.iter().take(BASELINE_WINDOW).map(|(_, m)| *m).sum::<f64>()
                / BASELINE_WINDOW as f64;
        qid_means.insert(qid.clone(), means.iter().map(|(_, m)| *m).collect());
        for (i, (t_rel, mean)) in means.iter().enumerate() {
            if i < BASELINE_WINDOW {
                continue;
            }
            plan_regression::push_latency(&mut stream, *t_rel, qid, *mean, baseline);
        }
    }

    // Workload-phase residuals: per-snapshot share-distribution entropy
    // across query_ids. We push the *negative log-entropy ratio* so that
    // a *concentration* (entropy drops) maps to a positive residual the
    // workload-phase motif treats as drift in the same direction as
    // tpcds JSD spikes.
    let mut prev_calls: HashMap<String, u64> = HashMap::new();
    let mut max_entropy_seen: f64 = 0.0;
    let mut snapshot_shares: Vec<(f64, Vec<f64>)> = Vec::new();
    for &t in &snapshot_times {
        let mut shares: Vec<f64> = Vec::new();
        let mut total: u64 = 0;
        for qid in &qids_sorted {
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
            continue;
        }
        for s in shares.iter_mut() {
            *s /= total as f64;
        }
        let entropy: f64 = shares
            .iter()
            .filter(|s| **s > 0.0)
            .map(|s| -s * s.ln())
            .sum();
        max_entropy_seen = max_entropy_seen.max(entropy);
        snapshot_shares.push((t, vec![entropy]));
    }
    if max_entropy_seen > 0.0 {
        for (t_abs, v) in snapshot_shares {
            let entropy = v[0];
            let normalised = entropy / max_entropy_seen;
            // residual = 1 - normalised entropy; high = concentrated workload.
            let r = 1.0 - normalised;
            workload_phase::push_jsd(&mut stream, t_abs - t0, "pgss_digest_mix", r);
        }
    }

    stream.sort();
    let _ = qid_means;
    Ok(stream)
}
