//! Snowset adapter (Vuppalapati et al., NSDI 2020).
//!
//! Real subset: CSV files distributed at
//! [github.com/resource-disaggregation/snowset](https://github.com/resource-disaggregation/snowset),
//! columns (subset used here):
//!   * `queryId`, `warehouseId`, `databaseId`
//!   * `createdTime`, `endTime` (epoch microseconds)
//!   * `compilationTime`, `executionTime`, `queueTime` (microseconds)
//!   * `bytesScannedFromCache`, `bytesScannedFromStorage`
//!   * `usrCpuTime`, `sysCpuTime`
//!
//! What we extract:
//!   * `PlanRegression` — `executionTime − rolling_baseline(executionTime)`
//!     per `(warehouseId, queryId)` pair (proxy for query class — Snowset
//!     anonymises SQL text per fact #16).
//!   * `WorkloadPhase` — JS divergence over the per-warehouse query-class
//!     histogram in 5-minute buckets.
//!   * `CacheIo` — `bytesScannedFromStorage / (bytes…Cache + bytes…Storage)`
//!     drift (cache-miss-rate residual).
//!
//! What we cannot extract (paper says so explicitly):
//!   * `Cardinality` — Snowset does not publish `est_rows`/`actual_rows`.
//!   * `Contention` — no lock-wait stream.

use super::DatasetAdapter;
use crate::residual::{
    cache_io, plan_regression, workload_phase, ResidualStream,
};
use anyhow::{Context, Result};
use rand::SeedableRng;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::path::Path;

pub struct Snowset;

#[derive(Debug, serde::Deserialize)]
struct Row {
    #[serde(rename = "queryId")]
    query_id: String,
    #[serde(rename = "warehouseId")]
    warehouse_id: String,
    #[serde(rename = "createdTime")]
    created_time_us: f64,
    #[serde(rename = "executionTime")]
    execution_time_us: f64,
    #[serde(default, rename = "bytesScannedFromCache")]
    bytes_cache: f64,
    #[serde(default, rename = "bytesScannedFromStorage")]
    bytes_storage: f64,
}

impl DatasetAdapter for Snowset {
    fn name(&self) -> &'static str {
        "snowset"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let mut rdr = csv::Reader::from_path(path)
            .with_context(|| format!("opening snowset subset at {}", path.display()))?;
        let mut rows: Vec<Row> = Vec::new();
        for r in rdr.deserialize() {
            let row: Row = match r {
                Ok(r) => r,
                Err(_) => continue, // tolerate malformed rows
            };
            if !row.execution_time_us.is_finite() || !row.created_time_us.is_finite() {
                continue;
            }
            rows.push(row);
        }
        rows.sort_by(|a, b| {
            a.created_time_us
                .partial_cmp(&b.created_time_us)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut stream = ResidualStream::new(format!(
            "snowset@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));
        let t0 = rows.first().map(|r| r.created_time_us).unwrap_or(0.0);

        // rolling baselines per (warehouse, query) and per-warehouse cache ratio
        let mut baselines: HashMap<(String, String), VecDeque<f64>> = HashMap::new();
        let mut cache_baseline: HashMap<String, VecDeque<f64>> = HashMap::new();
        const WIN: usize = 64;

        for r in &rows {
            let t = (r.created_time_us - t0) / 1e6;
            let key = (r.warehouse_id.clone(), r.query_id.clone());
            let q = baselines.entry(key.clone()).or_default();
            let baseline = if q.is_empty() {
                r.execution_time_us
            } else {
                q.iter().sum::<f64>() / q.len() as f64
            };
            // milliseconds
            plan_regression::push_latency(
                &mut stream,
                t,
                &format!("{}/{}", r.warehouse_id, r.query_id),
                r.execution_time_us / 1e3,
                baseline / 1e3,
            );
            q.push_back(r.execution_time_us);
            if q.len() > WIN {
                q.pop_front();
            }

            let total = r.bytes_cache + r.bytes_storage;
            if total > 0.0 {
                let cache_ratio = r.bytes_cache / total;
                let cb = cache_baseline.entry(r.warehouse_id.clone()).or_default();
                let expected = if cb.is_empty() {
                    cache_ratio
                } else {
                    cb.iter().sum::<f64>() / cb.len() as f64
                };
                cache_io::push_hit_ratio(
                    &mut stream,
                    t,
                    &r.warehouse_id,
                    expected,
                    cache_ratio,
                );
                cb.push_back(cache_ratio);
                if cb.len() > WIN {
                    cb.pop_front();
                }
            }
        }

        // Workload-phase residual via JS divergence of the per-warehouse
        // query-class histogram over 5-minute buckets.
        let bucket_seconds = 300.0;
        let mut histos: HashMap<String, HashMap<String, u64>> = HashMap::new();
        let mut prev_histos: HashMap<String, HashMap<String, u64>> = HashMap::new();
        let mut current_bucket = 0_i64;
        for r in &rows {
            let t = (r.created_time_us - t0) / 1e6;
            let bucket = (t / bucket_seconds) as i64;
            if bucket != current_bucket {
                for (wh, h) in &histos {
                    if let Some(prev) = prev_histos.get(wh) {
                        let d = workload_phase::js_divergence(prev, h);
                        workload_phase::push_jsd(
                            &mut stream,
                            current_bucket as f64 * bucket_seconds,
                            wh,
                            d,
                        );
                    }
                }
                prev_histos = std::mem::take(&mut histos);
                current_bucket = bucket;
            }
            *histos
                .entry(r.warehouse_id.clone())
                .or_default()
                .entry(r.query_id.clone())
                .or_insert(0) += 1;
        }
        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("snowset-exemplar-seed{seed}"));
        let warehouses = ["wh_a", "wh_b", "wh_c"];
        let queries = ["q1", "q2", "q3", "q4", "q5"];
        // Stable phase: 3000 s of low-residual traffic.
        for i in 0..3000 {
            let t = i as f64;
            let w = warehouses[(i / 200) % warehouses.len()];
            let q = queries[(i / 13) % queries.len()];
            let base = 50.0;
            let jitter: f64 = rng.gen_range(-3.0..3.0);
            plan_regression::push_latency(
                &mut stream,
                t,
                &format!("{w}/{q}"),
                base + jitter,
                base,
            );
            cache_io::push_hit_ratio(&mut stream, t, w, 0.92, 0.92 + rng.gen_range(-0.01..0.01));
        }
        // Phase shift: warehouse `wh_b` adopts a heavier query mix; phase JSD
        // crosses threshold around t=3300.
        for i in 3000..6000 {
            let t = i as f64;
            let w = "wh_b";
            let q = if rng.gen_bool(0.7) { "q_heavy" } else { "q5" };
            let base = 80.0;
            let jitter: f64 = rng.gen_range(-5.0..15.0);
            plan_regression::push_latency(
                &mut stream,
                t,
                &format!("{w}/{q}"),
                base + jitter,
                base,
            );
            cache_io::push_hit_ratio(
                &mut stream,
                t,
                w,
                0.92,
                0.55 + rng.gen_range(-0.05..0.05),
            );
        }
        // Synthetic JSD residual rising at the phase boundary
        for k in 0..30 {
            let t = 3000.0 + 50.0 * k as f64;
            let d = if (10..20).contains(&k) {
                0.4 + rng.gen_range(-0.05..0.05)
            } else {
                0.05 + rng.gen_range(0.0..0.03)
            };
            workload_phase::push_jsd(&mut stream, t, "wh_b", d);
        }
        stream.sort();
        stream
    }
}
