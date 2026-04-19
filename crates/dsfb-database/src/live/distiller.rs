//! Counter → residual distillation for the live PostgreSQL adapter.
//!
//! This module owns the pure function that maps cumulative-counter
//! snapshots (the output of one [`super::scraper::Scraper::next_snapshot`]
//! cycle) into residual samples on the five typed channels defined by
//! [`crate::residual::ResidualClass`]. It is called on the live path
//! from the emitter loop and is shared by the batch CSV path in
//! [`crate::adapters::postgres`] for the `plan_regression` channel
//! (both paths invoke [`PgssQidState::push_snapshot`] to preserve the
//! byte-for-byte residual value).
//!
//! ## Which classes this distiller emits (and which it refuses)
//!
//! | Class            | Source view          | Emitted here? |
//! |------------------|----------------------|---------------|
//! | PlanRegression   | pg_stat_statements   | yes           |
//! | Contention       | pg_stat_activity     | yes           |
//! | CacheIo          | pg_stat_io /         | yes           |
//! |                  | pg_stat_database     |               |
//! | WorkloadPhase    | pg_stat_statements   | yes           |
//! | Cardinality      | (not exposed by PG)  | **refused**   |
//!
//! The refused cell matches the \pmark\ in paper Table 10 for
//! PostgreSQL × Cardinality: `pg_stat_statements` does not expose
//! estimated-vs-actual row counts, so the live adapter cannot
//! construct a cardinality residual. Operators who need the
//! cardinality channel on PostgreSQL must use `auto_explain` + JSON
//! parsing, which is out of scope for this adapter.
//!
//! ## Honest divergence from the batch path
//!
//! The batch CSV path normalises workload-phase entropy against the
//! *global* maximum over the entire snapshot sequence (this is the
//! `max_entropy_seen` loop in [`crate::adapters::postgres`]). The
//! live path cannot know the global maximum in advance; it
//! normalises against the *running* maximum observed so far. Early
//! episodes on a live tape are therefore slightly under-shrunk
//! relative to a batch replay. This is disclosed in paper §Live
//! (the `tape → episodes` determinism clause is unchanged; only the
//! semantic calibration differs). A tape replay chooses the batch
//! global-max semantics explicitly via [`ReplayMode::GlobalMax`] so
//! that an operator who wants batch parity can get it from a
//! finalised tape.

use crate::residual::{cache_io, contention, plan_regression, workload_phase, ResidualSample, ResidualStream};
use std::collections::HashMap;

/// One polling tick's output from the scraper, tagged with the
/// wall-clock snapshot timestamp (seconds since the Unix epoch).
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub t: f64,
    pub pgss: Vec<PgssRow>,
    pub activity: Vec<ActivityRow>,
    pub stat_io: Vec<StatIoRow>,
    pub stat_database: Vec<StatDatabaseRow>,
}

#[derive(Debug, Clone)]
pub struct PgssRow {
    pub query_id: String,
    pub calls: u64,
    pub total_exec_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct ActivityRow {
    pub wait_event_type: String,
    pub wait_event: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct StatIoRow {
    pub backend_type: String,
    pub object: String,
    pub context: String,
    pub reads: u64,
    pub hits: u64,
    pub read_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct StatDatabaseRow {
    pub datname: String,
    pub blks_hit: u64,
    pub blks_read: u64,
}

/// Number of intervals at the start of a qid's history used to
/// establish its latency baseline. Mirrors
/// [`crate::adapters::postgres::BASELINE_WINDOW`] and must stay in
/// lockstep for the parity test.
pub const BASELINE_WINDOW: usize = 3;

/// Per-qid plan-regression state. Shared between the batch CSV path
/// (via [`crate::adapters::postgres`]) and the live path: both call
/// [`PgssQidState::push_snapshot`], so the residual value is computed
/// by the exact same arithmetic in both paths.
#[derive(Debug, Default, Clone)]
pub struct PgssQidState {
    last_calls: Option<u64>,
    last_total_ms: Option<f64>,
    means_for_baseline: Vec<f64>,
    baseline: Option<f64>,
}

impl PgssQidState {
    /// Ingest the next snapshot's counters for this qid. Returns the
    /// per-call mean latency and the baseline in effect for that
    /// interval, or `None` if no interval closed (no new calls, or
    /// first snapshot for this qid).
    ///
    /// The `(mean, baseline)` pair is exactly the input to
    /// [`plan_regression::push_latency`] — both the batch and the
    /// live path push with the same arguments, so the resulting
    /// residual value is byte-identical.
    pub fn push_snapshot(&mut self, calls: u64, total_exec_ms: f64) -> Option<(f64, f64)> {
        let (prev_calls, prev_total) = match (self.last_calls, self.last_total_ms) {
            (Some(c), Some(t)) => (c, t),
            _ => {
                self.last_calls = Some(calls);
                self.last_total_ms = Some(total_exec_ms);
                return None;
            }
        };
        let dc = calls.saturating_sub(prev_calls);
        let dt = total_exec_ms - prev_total;
        self.last_calls = Some(calls);
        self.last_total_ms = Some(total_exec_ms);
        if dc == 0 || dt < 0.0 {
            return None;
        }
        let mean = dt / dc as f64;
        debug_assert!(
            mean.is_finite() && mean >= 0.0,
            "dt>=0 && dc>0 => finite non-negative mean"
        );
        if self.means_for_baseline.len() < BASELINE_WINDOW {
            self.means_for_baseline.push(mean);
            if self.means_for_baseline.len() == BASELINE_WINDOW {
                let s: f64 = self.means_for_baseline.iter().sum();
                self.baseline = Some(s / BASELINE_WINDOW as f64);
            }
            return None;
        }
        let baseline = self.baseline.expect(
            "baseline must be populated once means_for_baseline.len() == BASELINE_WINDOW",
        );
        Some((mean, baseline))
    }
}

/// Per-(wait_event_type, wait_event) contention state. Tracks the
/// cumulative sample count so a delta can be pushed as a `Contention`
/// residual at each poll.
#[derive(Debug, Default, Clone)]
pub struct ContentionWaitState {
    last_count: Option<u64>,
}

/// Per-database CacheIo state (from `pg_stat_database`).
#[derive(Debug, Default, Clone)]
pub struct CacheIoDbState {
    last_hit: Option<u64>,
    last_read: Option<u64>,
}

/// Streaming distillation state. One instance per live session; the
/// replay-tape path constructs a fresh instance to replay the same
/// snapshots deterministically.
#[derive(Debug, Default)]
pub struct DistillerState {
    t0: Option<f64>,
    pgss_qids: HashMap<String, PgssQidState>,
    prev_pgss_calls: HashMap<String, u64>,
    activity_waits: HashMap<(String, String), ContentionWaitState>,
    cache_db: HashMap<String, CacheIoDbState>,
    max_entropy_seen: f64,
}

impl DistillerState {
    pub fn new() -> Self {
        Self::default()
    }

    fn t_rel(&mut self, t_abs: f64) -> f64 {
        if self.t0.is_none() {
            self.t0 = Some(t_abs);
        }
        t_abs - self.t0.unwrap()
    }

    /// Push one snapshot through the distiller. Emits every residual
    /// that became emittable at this snapshot: plan-regression
    /// residuals for qids whose baseline has warmed up, a
    /// workload-phase entropy residual for the full snapshot,
    /// contention residuals for wait-event counts that advanced, and
    /// cache-io residuals for databases that saw block traffic.
    pub fn ingest(&mut self, snap: &Snapshot) -> Vec<ResidualSample> {
        let mut out = Vec::new();
        let t_rel = self.t_rel(snap.t);
        self.ingest_pgss(snap, t_rel, &mut out);
        self.ingest_activity(snap, t_rel, &mut out);
        self.ingest_stat_io(snap, t_rel, &mut out);
        self.ingest_stat_database(snap, t_rel, &mut out);
        self.ingest_workload_phase(snap, t_rel, &mut out);
        out
    }

    fn ingest_pgss(&mut self, snap: &Snapshot, t_rel: f64, out: &mut Vec<ResidualSample>) {
        // Stable qid order for byte-deterministic output.
        let mut qids_in_snapshot: Vec<&PgssRow> = snap.pgss.iter().collect();
        qids_in_snapshot.sort_by(|a, b| a.query_id.cmp(&b.query_id));
        for row in qids_in_snapshot {
            let st = self
                .pgss_qids
                .entry(row.query_id.clone())
                .or_default();
            if let Some((mean, baseline)) = st.push_snapshot(row.calls, row.total_exec_time_ms) {
                let mut stream = ResidualStream::new("");
                plan_regression::push_latency(
                    &mut stream,
                    t_rel,
                    &row.query_id,
                    mean,
                    baseline,
                );
                out.extend(stream.samples);
            }
        }
    }

    fn ingest_activity(&mut self, snap: &Snapshot, t_rel: f64, out: &mut Vec<ResidualSample>) {
        // Aggregate sessions by (wait_event_type, wait_event). PG
        // exposes one row per session; we want one residual per
        // wait-class.
        let mut counts: HashMap<(String, String), u64> = HashMap::new();
        for row in snap.activity.iter() {
            if row.wait_event_type == "None" {
                continue;
            }
            let key = (row.wait_event_type.clone(), row.wait_event.clone());
            *counts.entry(key).or_default() += 1;
        }
        // Stable key order for deterministic output.
        let mut keys: Vec<(String, String)> = counts.keys().cloned().collect();
        keys.sort();
        for key in keys {
            let count = counts[&key];
            let st = self
                .activity_waits
                .entry(key.clone())
                .or_default();
            let prev = st.last_count.unwrap_or(0);
            st.last_count = Some(count);
            let delta = count.saturating_sub(prev);
            if delta == 0 {
                continue;
            }
            let mut stream = ResidualStream::new("");
            let wait_label = format!("{}::{}", key.0, key.1);
            contention::push_wait(&mut stream, t_rel, &wait_label, delta as f64);
            out.extend(stream.samples);
        }
    }

    fn ingest_stat_io(&mut self, snap: &Snapshot, t_rel: f64, out: &mut Vec<ResidualSample>) {
        // Collapse per-(backend_type, object, context) reads/hits into
        // one Hit-ratio residual per (object, context) bucket. We
        // ignore backend_type to keep the cardinality manageable.
        let mut buckets: HashMap<(String, String), (u64, u64)> = HashMap::new();
        for row in snap.stat_io.iter() {
            let key = (row.object.clone(), row.context.clone());
            let e = buckets.entry(key).or_default();
            e.0 += row.hits;
            e.1 += row.reads;
        }
        let mut keys: Vec<(String, String)> = buckets.keys().cloned().collect();
        keys.sort();
        for key in keys {
            let (hits, reads) = buckets[&key];
            let total = hits + reads;
            if total == 0 {
                continue;
            }
            let observed = hits as f64 / total as f64;
            // "Expected" is 1.0 — a perfectly-buffered cache hits
            // 100 % of accesses. The hit-ratio residual is therefore
            // the amount by which the actual cache falls short of a
            // perfect cache. That mirrors what
            // `cache_io::push_hit_ratio` does: `expected − observed`,
            // positive when the cache is underperforming.
            let mut stream = ResidualStream::new("");
            let bucket_label = format!("{}::{}", key.0, key.1);
            cache_io::push_hit_ratio(&mut stream, t_rel, &bucket_label, 1.0, observed);
            out.extend(stream.samples);
        }
    }

    fn ingest_stat_database(
        &mut self,
        snap: &Snapshot,
        t_rel: f64,
        out: &mut Vec<ResidualSample>,
    ) {
        // Fallback path: if pg_stat_io was empty (PG <16), fall back
        // to pg_stat_database deltas. If pg_stat_io supplied data
        // this snapshot we still record the running database-level
        // state (for continuity on engine upgrade) but skip the
        // emission.
        let emit_fallback = snap.stat_io.is_empty();
        let mut rows: Vec<&StatDatabaseRow> = snap.stat_database.iter().collect();
        rows.sort_by(|a, b| a.datname.cmp(&b.datname));
        for row in rows {
            let st = self.cache_db.entry(row.datname.clone()).or_default();
            let prev_hit = st.last_hit.unwrap_or(row.blks_hit);
            let prev_read = st.last_read.unwrap_or(row.blks_read);
            st.last_hit = Some(row.blks_hit);
            st.last_read = Some(row.blks_read);
            if !emit_fallback {
                continue;
            }
            let dh = row.blks_hit.saturating_sub(prev_hit);
            let dr = row.blks_read.saturating_sub(prev_read);
            let total = dh + dr;
            if total == 0 {
                continue;
            }
            let observed = dh as f64 / total as f64;
            let mut stream = ResidualStream::new("");
            let label = format!("db::{}", row.datname);
            cache_io::push_hit_ratio(&mut stream, t_rel, &label, 1.0, observed);
            out.extend(stream.samples);
        }
    }

    fn ingest_workload_phase(
        &mut self,
        snap: &Snapshot,
        t_rel: f64,
        out: &mut Vec<ResidualSample>,
    ) {
        // Shannon entropy of per-qid Δcalls shares at this snapshot.
        // Normalised against max-entropy-seen-so-far (running max);
        // this is the documented divergence from the batch path.
        let mut rows: Vec<&PgssRow> = snap.pgss.iter().collect();
        rows.sort_by(|a, b| a.query_id.cmp(&b.query_id));
        let mut shares: Vec<f64> = Vec::new();
        let mut total: u64 = 0;
        for row in rows.iter() {
            let prev = self
                .prev_pgss_calls
                .get(&row.query_id)
                .copied()
                .unwrap_or(0);
            let delta = row.calls.saturating_sub(prev);
            self.prev_pgss_calls.insert(row.query_id.clone(), row.calls);
            if delta > 0 {
                shares.push(delta as f64);
                total += delta;
            }
        }
        if total == 0 || shares.is_empty() {
            return;
        }
        for s in shares.iter_mut() {
            *s /= total as f64;
        }
        let entropy: f64 = shares
            .iter()
            .filter(|s| **s > 0.0)
            .map(|s| -s * s.ln())
            .sum();
        debug_assert!(entropy.is_finite() && entropy >= 0.0);
        self.max_entropy_seen = self.max_entropy_seen.max(entropy);
        if self.max_entropy_seen <= 0.0 {
            return;
        }
        let normalised = entropy / self.max_entropy_seen;
        let r = 1.0 - normalised;
        let mut stream = ResidualStream::new("");
        workload_phase::push_jsd(&mut stream, t_rel, "pgss_digest_mix", r);
        out.extend(stream.samples);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pgss(qid: &str, calls: u64, total: f64) -> PgssRow {
        PgssRow {
            query_id: qid.to_string(),
            calls,
            total_exec_time_ms: total,
        }
    }

    #[test]
    fn plan_regression_warmup_honors_baseline_window() {
        let mut st = PgssQidState::default();
        // snapshot 0: prime
        assert_eq!(st.push_snapshot(10, 100.0), None);
        // snapshot 1: first interval — still warming up
        assert_eq!(st.push_snapshot(20, 200.0), None);
        // snapshot 2: second interval — still warming up
        assert_eq!(st.push_snapshot(30, 300.0), None);
        // snapshot 3: third interval — baseline frozen, no residual yet
        assert_eq!(st.push_snapshot(40, 400.0), None);
        // snapshot 4: first emittable interval
        let out = st.push_snapshot(50, 500.0).unwrap();
        assert!(out.0.is_finite());
        assert!(out.1.is_finite());
        assert!((out.1 - 10.0).abs() < 1e-9, "baseline should be 10 ms/call");
    }

    #[test]
    fn live_and_batch_math_agree_for_plan_regression() {
        // Feed identical counters through PgssQidState (live) and
        // through the `per_query_mean_exec_time`-equivalent logic
        // used in adapters::postgres (batch). We only compare the
        // (mean, baseline) pair — the push_latency call is byte-
        // identical in both paths.
        let snaps = [(10, 100.0), (20, 220.0), (30, 360.0), (40, 520.0), (50, 700.0)];
        let mut live = PgssQidState::default();
        let mut live_out = Vec::new();
        for (c, t) in snaps.iter() {
            if let Some(pair) = live.push_snapshot(*c, *t) {
                live_out.push(pair);
            }
        }
        // Batch equivalent (mirrors per_query_mean_exec_time + first-3 baseline)
        let mut means: Vec<f64> = Vec::new();
        for w in snaps.windows(2) {
            let dt = w[1].1 - w[0].1;
            let dc = w[1].0 - w[0].0;
            if dc == 0 {
                continue;
            }
            means.push(dt / dc as f64);
        }
        assert!(means.len() > BASELINE_WINDOW);
        let baseline: f64 = means.iter().take(BASELINE_WINDOW).sum::<f64>()
            / BASELINE_WINDOW as f64;
        let batch_out: Vec<(f64, f64)> = means
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= BASELINE_WINDOW)
            .map(|(_, m)| (*m, baseline))
            .collect();
        assert_eq!(live_out, batch_out);
    }

    #[test]
    fn distiller_emits_plan_regression_after_warmup() {
        let mut d = DistillerState::new();
        for (i, (c, t)) in [(10, 100.0), (20, 220.0), (30, 360.0), (40, 520.0), (50, 700.0)]
            .iter()
            .enumerate()
        {
            let snap = Snapshot {
                t: i as f64,
                pgss: vec![pgss("q1", *c, *t)],
                ..Default::default()
            };
            let emitted = d.ingest(&snap);
            if i < 4 {
                // Only workload_phase may fire (shares entropy is
                // trivially zero with a single qid, so nothing
                // emits).
                assert!(
                    !emitted.iter().any(|s| matches!(
                        s.class,
                        crate::residual::ResidualClass::PlanRegression
                    )),
                    "plan_regression should not emit before warm-up (i={})",
                    i
                );
            } else {
                assert!(emitted.iter().any(|s| matches!(
                    s.class,
                    crate::residual::ResidualClass::PlanRegression
                )));
            }
        }
    }
}
