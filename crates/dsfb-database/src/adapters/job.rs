//! JOB adapter (Join Order Benchmark, Leis et al., VLDB 2015).
//!
//! Real subset CSV columns (after running JOB through DuckDB / PostgreSQL
//! with `EXPLAIN ANALYZE` and exporting via `scripts/fetch_ceb.sh`-style
//! tooling — JOB itself is just the 113 SQL files; you generate the trace):
//!   * `query_id` (e.g. `1a`, `33c`)
//!   * `iteration` (replay number)
//!   * `est_rows`, `actual_rows` per top-level result
//!   * `latency_ms`
//!   * `plan_hash` (SHA-1 of the EXPLAIN tree, for plan-change detection)
//!
//! What we extract:
//!   * `Cardinality` — `log10(actual / est)` per query (top-level).
//!   * `PlanRegression` — latency residual per query class + plan-hash
//!     transition events.
//!
//! What we cannot extract:
//!   * `Contention`, `CacheIo`, `WorkloadPhase` (single-tenant replay; no
//!     phase changes in a 113-query benchmark).

use super::DatasetAdapter;
use crate::residual::{cardinality, plan_regression, ResidualStream};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// Upper bound on the number of JOB rows loaded from a CSV. The
/// original Join Order Benchmark has 113 queries; even with repeated
/// iterations and plan-variant rows this cap catches runaway inputs
/// without truncating realistic traces.
const MAX_JOB_ROWS: usize = 100_000_000;

pub struct Job;

#[derive(Debug, serde::Deserialize)]
struct Row {
    query_id: String,
    iteration: u64,
    est_rows: f64,
    actual_rows: f64,
    latency_ms: f64,
    #[serde(default)]
    plan_hash: String,
}

impl DatasetAdapter for Job {
    fn name(&self) -> &'static str {
        "job"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let mut rdr = csv::Reader::from_path(path)
            .with_context(|| format!("opening job csv at {}", path.display()))?;
        let mut rows: Vec<Row> = rdr
            .deserialize()
            .filter_map(Result::ok)
            .take(MAX_JOB_ROWS)
            .collect();
        debug_assert!(rows.len() <= MAX_JOB_ROWS, "iterator bound enforced");
        rows.sort_by(|a, b| {
            (a.iteration, a.query_id.clone()).cmp(&(b.iteration, b.query_id.clone()))
        });
        let mut stream = ResidualStream::new(format!(
            "job@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));
        let mut last_hash: HashMap<String, String> = HashMap::new();
        let mut baselines: HashMap<String, VecDeque<f64>> = HashMap::new();
        const WIN: usize = 8;
        let mut t: f64 = 0.0;
        for r in &rows {
            cardinality::push(&mut stream, t, &r.query_id, r.est_rows, r.actual_rows);
            let q = baselines.entry(r.query_id.clone()).or_default();
            let baseline = if q.is_empty() {
                r.latency_ms
            } else {
                q.iter().sum::<f64>() / q.len() as f64
            };
            plan_regression::push_latency(&mut stream, t, &r.query_id, r.latency_ms, baseline);
            q.push_back(r.latency_ms);
            if q.len() > WIN {
                q.pop_front();
            }
            if !r.plan_hash.is_empty() {
                let prev = last_hash.get(&r.query_id).cloned().unwrap_or_default();
                if !prev.is_empty() && prev != r.plan_hash {
                    plan_regression::push_plan_change(&mut stream, t, &r.query_id);
                }
                last_hash.insert(r.query_id.clone(), r.plan_hash.clone());
            }
            t += 1.0;
        }
        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("job-exemplar-seed{seed}"));
        // 33 queries × 5 iterations; query 17 develops a plan regression at
        // iteration 3 (latency jumps 8x and est/actual ratio jumps to 25x).
        let mut t = 0.0;
        for it in 0..5 {
            for q in 1..=33 {
                let qid = format!("q{:02}", q);
                let true_rows: f64 = 1000.0_f64 * (1.0 + rng.gen_range(0.0..0.5));
                let est_rows = if q == 17 && it >= 3 {
                    true_rows / 25.0
                } else {
                    true_rows * (1.0 + rng.gen_range(-0.05..0.05))
                };
                cardinality::push(&mut stream, t, &qid, est_rows, true_rows);
                let baseline = 100.0;
                let latency = if q == 17 && it >= 3 {
                    800.0 + rng.gen_range(-30.0..30.0)
                } else {
                    100.0 + rng.gen_range(-5.0..5.0)
                };
                plan_regression::push_latency(&mut stream, t, &qid, latency, baseline);
                if q == 17 && it == 3 {
                    plan_regression::push_plan_change(&mut stream, t, &qid);
                }
                t += 1.0;
            }
        }
        stream.sort();
        stream
    }
}
