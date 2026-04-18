//! TPC-DS adapter.
//!
//! TPC-DS is a benchmark *control* environment, not a real workload — we
//! use it for the perturbation harness (see `crate::perturbation`) so that
//! every motif class can be evaluated against an injected, known-window
//! ground truth. The crate ships a fully-deterministic exemplar that
//! mimics the structure of a TPC-DS scale-1 trace under each perturbation
//! class. To run on real TPC-DS data, install `duckdb` with the `tpcds`
//! extension and call `scripts/build_tpcds.sh` which writes a CSV in the
//! same format as the exemplar.
//!
//! Trace CSV columns:
//!   * `query_id` (one of `q1`..`q99`)
//!   * `t_seconds` (wall-clock since trace start)
//!   * `latency_ms`, `est_rows`, `actual_rows`
//!   * optional `wait_event`, `wait_seconds`, `cache_hit_ratio`

use super::DatasetAdapter;
use crate::residual::{
    cache_io, cardinality, contention, plan_regression, workload_phase, ResidualStream,
};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// Upper bound on TPC-DS rows loaded from a CSV. TPC-DS scale-1 on a
/// 99-query workload produces ~10k-50k rows depending on iterations;
/// 100M is a cap that still accepts scale-1000 long runs.
const MAX_TPCDS_ROWS: usize = 100_000_000;

/// Rolling-baseline window for per-query latency.
const TPCDS_BASELINE_WIN: usize = 16;

/// Workload-phase histogram bucket width in seconds.
const TPCDS_BUCKET_SECONDS: f64 = 30.0;

/// Target cache-hit ratio treated as the reference point for
/// cache-I/O residuals (`cache_io::push_hit_ratio`). Matches the §7
/// default for the TPC-DS exemplar.
const TPCDS_CACHE_TARGET_RATIO: f64 = 0.95;

pub struct TpcDs;

#[derive(Debug, serde::Deserialize)]
struct Row {
    query_id: String,
    t_seconds: f64,
    latency_ms: f64,
    est_rows: f64,
    actual_rows: f64,
    #[serde(default)]
    wait_event: String,
    #[serde(default)]
    wait_seconds: f64,
    #[serde(default)]
    cache_hit_ratio: f64,
}

fn load_tpcds_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening tpcds csv at {}", path.display()))?;
    let mut rows: Vec<Row> = rdr
        .deserialize()
        .filter_map(Result::ok)
        .take(MAX_TPCDS_ROWS)
        .collect();
    debug_assert!(rows.len() <= MAX_TPCDS_ROWS, "iterator bound enforced");
    rows.sort_by(|a, b| {
        a.t_seconds
            .partial_cmp(&b.t_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(rows)
}

fn emit_tpcds_residuals(stream: &mut ResidualStream, rows: &[Row]) {
    let mut baselines: HashMap<String, VecDeque<f64>> = HashMap::new();
    let mut histos: HashMap<String, u64> = HashMap::new();
    let mut prev_histos: HashMap<String, u64> = HashMap::new();
    let mut current_bucket: i64 = 0;

    for r in rows.iter() {
        cardinality::push(stream, r.t_seconds, &r.query_id, r.est_rows, r.actual_rows);
        emit_tpcds_plan_regression(stream, &mut baselines, r);
        if !r.wait_event.is_empty() && r.wait_seconds > 0.0 {
            contention::push_wait(stream, r.t_seconds, &r.wait_event, r.wait_seconds);
        }
        if r.cache_hit_ratio > 0.0 {
            cache_io::push_hit_ratio(
                stream,
                r.t_seconds,
                "tpcds",
                TPCDS_CACHE_TARGET_RATIO,
                r.cache_hit_ratio,
            );
        }
        let bucket = (r.t_seconds / TPCDS_BUCKET_SECONDS) as i64;
        if bucket != current_bucket {
            let d = workload_phase::js_divergence(&prev_histos, &histos);
            debug_assert!((0.0..=1.0).contains(&d), "JSD is in [0,1]");
            workload_phase::push_jsd(
                stream,
                current_bucket as f64 * TPCDS_BUCKET_SECONDS,
                "tpcds",
                d,
            );
            prev_histos = std::mem::take(&mut histos);
            current_bucket = bucket;
        }
        *histos.entry(r.query_id.clone()).or_insert(0) += 1;
    }
}

fn emit_tpcds_plan_regression(
    stream: &mut ResidualStream,
    baselines: &mut HashMap<String, VecDeque<f64>>,
    r: &Row,
) {
    debug_assert!(r.latency_ms.is_finite(), "latency must be finite");
    debug_assert!(r.t_seconds.is_finite(), "t_seconds must be finite");
    let q = baselines.entry(r.query_id.clone()).or_default();
    let baseline = if q.is_empty() {
        r.latency_ms
    } else {
        q.iter().sum::<f64>() / q.len() as f64
    };
    plan_regression::push_latency(stream, r.t_seconds, &r.query_id, r.latency_ms, baseline);
    q.push_back(r.latency_ms);
    if q.len() > TPCDS_BASELINE_WIN {
        q.pop_front();
    }
    debug_assert!(
        q.len() <= TPCDS_BASELINE_WIN,
        "rolling window bound enforced"
    );
}

impl DatasetAdapter for TpcDs {
    fn name(&self) -> &'static str {
        "tpcds"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let rows = load_tpcds_rows(path)?;
        debug_assert!(rows.len() <= MAX_TPCDS_ROWS, "row-count bound enforced");
        let mut stream = ResidualStream::new(format!(
            "tpcds@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));
        emit_tpcds_residuals(&mut stream, &rows);
        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        // The exemplar is built by the perturbation harness; see
        // `crate::perturbation::tpcds_with_perturbations`. This function
        // returns a clean baseline (no perturbations) for unit tests.
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("tpcds-exemplar-seed{seed}"));
        for q in 1..=99 {
            let qid = format!("q{}", q);
            for it in 0..30 {
                let t = q as f64 * 30.0 + it as f64;
                let true_rows: f64 = 5000.0 * (1.0 + rng.gen_range(0.0..0.4));
                let est_rows = true_rows * (1.0 + rng.gen_range(-0.08..0.08));
                cardinality::push(&mut stream, t, &qid, est_rows, true_rows);
                let base = 50.0_f64;
                plan_regression::push_latency(
                    &mut stream,
                    t,
                    &qid,
                    base + rng.gen_range(-2.0..2.0),
                    base,
                );
                cache_io::push_hit_ratio(
                    &mut stream,
                    t,
                    "tpcds",
                    0.95,
                    0.95 + rng.gen_range(-0.005..0.005),
                );
            }
        }
        stream.sort();
        stream
    }
}
