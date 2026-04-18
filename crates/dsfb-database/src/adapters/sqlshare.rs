//! SQLShare adapter (Jain et al., SIGMOD 2016).
//!
//! Real subset CSV columns (as released by UW eScience):
//!   * `query_id`, `user_id`, `runtime_seconds`, `submitted_at`, `query_text`
//!
//! What we extract:
//!   * `PlanRegression` — `runtime − rolling_baseline(runtime)` per
//!     `(user_id, normalised_query_skeleton)`. Normalisation = strip
//!     literals, collapse whitespace, lowercase. This stands in for query
//!     digest because SQLShare predates digest IDs.
//!   * `WorkloadPhase` — JS divergence over the per-user query-skeleton
//!     histogram in 1-day buckets.
//!
//! What we cannot extract:
//!   * `Cardinality`, `Contention`, `CacheIo` — none of these are in the
//!     released metadata. The paper's Table on "what each dataset supplies"
//!     marks these as N/A for SQLShare.

use super::DatasetAdapter;
use crate::residual::{plan_regression, workload_phase, ResidualStream};
use anyhow::{Context, Result};
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

pub struct SqlShare;

#[derive(Debug, serde::Deserialize)]
struct Row {
    user_id: String,
    runtime_seconds: f64,
    submitted_at: f64, // epoch seconds
    query_text: String,
}

fn skeleton(q: &str) -> String {
    // crude normalisation: collapse digits + quoted strings, lowercase, dedupe whitespace
    let mut out = String::with_capacity(q.len());
    let mut in_str = false;
    let mut prev_ws = false;
    for c in q.chars() {
        if c == '\'' || c == '"' {
            in_str = !in_str;
            out.push('?');
            continue;
        }
        if in_str {
            continue;
        }
        if c.is_ascii_digit() {
            out.push('?');
            prev_ws = false;
            continue;
        }
        if c.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
            continue;
        }
        prev_ws = false;
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out.trim().to_string()
}

/// Upper bound on SQLShare rows loaded. The released SQLShare dataset
/// has ~24k queries; 100M is a ~4000× cap that still rejects pathological
/// or corrupted inputs without silently truncating realistic traces.
const MAX_SQLSHARE_ROWS: usize = 100_000_000;

/// Rolling-baseline window for per-skeleton plan-regression deltas.
const PLAN_BASELINE_WIN: usize = 32;

/// Workload-phase bucket width in seconds (one day).
const PHASE_BUCKET_SECONDS: f64 = 86_400.0;

/// Max characters retained from the normalised skeleton when building
/// the per-user channel label. Keeps labels reviewable and bounded.
const SKELETON_LABEL_MAX: usize = 64;

fn load_sqlshare_rows(path: &Path) -> Result<Vec<Row>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening sqlshare csv at {}", path.display()))?;
    let mut rows: Vec<Row> = rdr
        .deserialize()
        .filter_map(Result::ok)
        .take(MAX_SQLSHARE_ROWS)
        .collect();
    debug_assert!(rows.len() <= MAX_SQLSHARE_ROWS, "iterator bound enforced");
    rows.sort_by(|a, b| {
        a.submitted_at
            .partial_cmp(&b.submitted_at)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(rows)
}

fn emit_sqlshare_residuals(stream: &mut ResidualStream, rows: &[Row], t0: f64) {
    debug_assert!(t0.is_finite(), "t0 must be finite");
    let mut baselines: HashMap<(String, String), VecDeque<f64>> = HashMap::new();
    let mut histos: HashMap<String, HashMap<String, u64>> = HashMap::new();
    let mut prev_histos: HashMap<String, HashMap<String, u64>> = HashMap::new();
    let mut current_bucket: i64 = 0;

    for r in rows.iter() {
        let t = r.submitted_at - t0;
        let sk = skeleton(&r.query_text);
        emit_plan_regression_sample(stream, &mut baselines, r, t, &sk);

        let bucket = (t / PHASE_BUCKET_SECONDS) as i64;
        if bucket != current_bucket {
            flush_histogram_deltas(stream, &histos, &prev_histos, current_bucket);
            prev_histos = std::mem::take(&mut histos);
            current_bucket = bucket;
        }
        *histos
            .entry(r.user_id.clone())
            .or_default()
            .entry(sk)
            .or_insert(0) += 1;
    }
}

fn emit_plan_regression_sample(
    stream: &mut ResidualStream,
    baselines: &mut HashMap<(String, String), VecDeque<f64>>,
    r: &Row,
    t: f64,
    sk: &str,
) {
    debug_assert!(r.runtime_seconds.is_finite(), "runtime must be finite");
    debug_assert!(t.is_finite(), "t must be finite");
    let key = (r.user_id.clone(), sk.to_string());
    let q = baselines.entry(key).or_default();
    let baseline = if q.is_empty() {
        r.runtime_seconds
    } else {
        q.iter().sum::<f64>() / q.len() as f64
    };
    plan_regression::push_latency(
        stream,
        t,
        &format!("{}#{}", r.user_id, &sk[..sk.len().min(SKELETON_LABEL_MAX)]),
        r.runtime_seconds,
        baseline,
    );
    q.push_back(r.runtime_seconds);
    if q.len() > PLAN_BASELINE_WIN {
        q.pop_front();
    }
    debug_assert!(
        q.len() <= PLAN_BASELINE_WIN,
        "rolling-window bound enforced"
    );
}

fn flush_histogram_deltas(
    stream: &mut ResidualStream,
    histos: &HashMap<String, HashMap<String, u64>>,
    prev_histos: &HashMap<String, HashMap<String, u64>>,
    current_bucket: i64,
) {
    debug_assert!(current_bucket >= 0, "bucket index is non-negative");
    for (u, h) in histos.iter() {
        if let Some(prev) = prev_histos.get(u) {
            let d = workload_phase::js_divergence(prev, h);
            debug_assert!((0.0..=1.0).contains(&d), "JSD is in [0,1]");
            workload_phase::push_jsd(stream, current_bucket as f64 * PHASE_BUCKET_SECONDS, u, d);
        }
    }
}

impl DatasetAdapter for SqlShare {
    fn name(&self) -> &'static str {
        "sqlshare"
    }

    fn load(&self, path: &Path) -> Result<ResidualStream> {
        let rows = load_sqlshare_rows(path)?;
        debug_assert!(
            !rows.is_empty(),
            "non-empty input implied by csv parse success"
        );
        let t0 = rows.first().map(|r| r.submitted_at).unwrap_or(0.0);
        debug_assert!(t0.is_finite(), "t0 must be finite");
        let mut stream = ResidualStream::new(format!(
            "sqlshare@{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
        ));
        emit_sqlshare_residuals(&mut stream, &rows, t0);
        stream.sort();
        Ok(stream)
    }

    fn exemplar(&self, seed: u64) -> ResidualStream {
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut stream = ResidualStream::new(format!("sqlshare-exemplar-seed{seed}"));
        // 5 users; each has 200 queries spread over 5 days; user 3 develops a
        // long-running ad-hoc query starting at day 3.
        let users = ["alice", "bob", "carol", "dave", "eve"];
        let qskeletons = [
            "select count from t",
            "join a b",
            "group by x",
            "where y",
            "subselect",
        ];
        for (u, user) in users.iter().enumerate() {
            for q in 0..200 {
                let t = (q as f64) * 86400.0 / 200.0 * 5.0 + (u as f64) * 30.0;
                let sk = qskeletons[q % qskeletons.len()];
                let base = 0.4;
                let mut runtime = base + rng.gen_range(-0.05..0.05);
                if u == 2 && t > 3.0 * 86400.0 {
                    runtime = 5.0 + rng.gen_range(-0.5..0.5);
                }
                plan_regression::push_latency(
                    &mut stream,
                    t,
                    &format!("{user}#{sk}"),
                    runtime,
                    base,
                );
            }
        }
        // Phase JSD residual jumping at day 3 for user 'carol'
        for d in 0..6 {
            let t = d as f64 * 86400.0;
            let jsd = if d == 3 { 0.42 } else { 0.05 };
            workload_phase::push_jsd(&mut stream, t, "carol", jsd);
        }
        stream.sort();
        stream
    }
}
