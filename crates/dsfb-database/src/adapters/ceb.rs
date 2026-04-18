//! CEB adapter (Cardinality Estimation Benchmark, Negi et al.).
//!
//! Real subset CSV columns (after `scripts/fetch_ceb.sh` exports the
//! pickle to CSV):
//!   * `query_id`, `subplan_id`
//!   * `true_rows` (ground truth from PostgreSQL `EXPLAIN ANALYZE`)
//!   * `est_rows` (PostgreSQL optimiser estimate)
//!   * optional: `query_class`, `template_id`
//!
//! What we extract:
//!   * `Cardinality` — `log10(true_rows / est_rows)` per `(query_id, subplan_id)`.
//!     This is the only public dataset in our stack with **ground-truth
//!     cardinalities**, which is why we treat its results as the
//!     cardinality-mismatch motif's primary empirical evidence.
//!
//! What we cannot extract:
//!   * Latency, plan changes over time, contention, cache I/O — CEB is a
//!     batch benchmark, not a temporal trace; we synthesise time as
//!     `t = subplan_index_within_query + query_index * 1.0` so the trace is
//!     well-defined.

use super::DatasetAdapter;
use crate::residual::{cardinality, ResidualStream};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::path::Path;

/// Upper bound on the number of CEB rows loaded from a CSV. The
/// published CEB release is ~13k rows; this ~7000× headroom catches
/// runaway input without silently truncating realistic datasets.
const MAX_CEB_ROWS: usize = 100_000_000;

pub struct Ceb;

#[derive(Debug, serde::Deserialize)]
struct Row {
    query_id: String,
    subplan_id: String,
    true_rows: f64,
    est_rows: f64,
}

impl DatasetAdapter for Ceb {
    fn name(&self) -> &'static str {
        "ceb"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let mut rdr = csv::Reader::from_path(path)
            .with_context(|| format!("opening ceb csv at {}", path.display()))?;
        let mut rows: Vec<Row> = rdr
            .deserialize()
            .filter_map(Result::ok)
            .take(MAX_CEB_ROWS)
            .collect();
        debug_assert!(rows.len() <= MAX_CEB_ROWS, "iterator bound enforced");
        rows.sort_by(|a, b| a.query_id.cmp(&b.query_id));
        let mut stream = ResidualStream::new(format!(
            "ceb@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));
        // Synthesise a time index: each query gets one second; subplans
        // within a query are spaced 0.01 s apart.
        let mut q_index: HashMap<String, usize> = HashMap::new();
        let mut sp_index: HashMap<String, usize> = HashMap::new();
        for r in &rows {
            let next_q = q_index.len();
            let qi = *q_index.entry(r.query_id.clone()).or_insert(next_q);
            let next_sp = sp_index.len();
            let sp = *sp_index
                .entry(format!("{}#{}", r.query_id, r.subplan_id))
                .or_insert(next_sp);
            let t = qi as f64 + (sp % 100) as f64 * 0.01;
            cardinality::push(
                &mut stream,
                t,
                &format!("{}#{}", r.query_id, r.subplan_id),
                r.est_rows,
                r.true_rows,
            );
        }
        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("ceb-exemplar-seed{seed}"));
        // 200 queries; subplans 1..=10. The first 100 queries have
        // well-calibrated estimates; queries 100..200 develop a 30x
        // under-estimate on subplan 7 (a join with stale stats).
        //
        // Channels in the exemplar are *per-subplan-template* (`sp{n}`),
        // not per-(query, subplan), because the operator-meaningful
        // grouping for cardinality drift is "the same logical join
        // across queries" — and a per-(query, subplan) channel would
        // contain a single sample, which the EMA cannot smooth.
        // Real-data CEB load (`load(path)`) keeps the
        // per-(query, subplan) channel because each subplan in CEB is a
        // distinct statement; the exemplar collapses for demo clarity.
        for q in 0..200 {
            for sp in 1..=10 {
                let t = q as f64 + (sp as f64) * 0.01;
                let true_rows: f64 = 1000.0_f64 * (1.0 + rng.gen_range(0.0..2.0));
                let est_rows: f64 = if q >= 100 && sp == 7 {
                    true_rows / 30.0
                } else {
                    true_rows * (1.0 + rng.gen_range(-0.1..0.1))
                };
                cardinality::push(&mut stream, t, &format!("sp{sp}"), est_rows, true_rows);
            }
        }
        stream.sort();
        stream
    }
}
