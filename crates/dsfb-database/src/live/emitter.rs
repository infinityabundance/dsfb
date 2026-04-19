//! Buffered-rescan episode emitter for the live path.
//!
//! On every polling tick, the live loop ingests fresh residual samples
//! via [`LiveEmitter::push_samples`], re-runs the unmodified batch
//! [`MotifEngine::run`] on the growing in-memory buffer, and emits
//! only those episodes that were not seen in the previous rescan.
//!
//! The crucial property: **the motif runner is untouched**. The same
//! state machines, the same grammar parameters, the same episode
//! close logic as the batch path. This is not a refactor of the
//! grammar — it is a *call* to the grammar on a growing buffer. The
//! batch fingerprint tests remain valid by construction because the
//! grammar's byte-level output depends only on the input stream.
//!
//! Memory is bounded by `retention_window_s`: after every ingest,
//! samples older than the oldest currently-open episode are trimmed.
//! An *open* episode is one that appears in the rescan but has not
//! yet been emitted — its `t_end` is still in the future. This is a
//! conservative bound; a long-running contention regime that never
//! closes will keep its samples alive. Operators can cap the buffer
//! explicitly via `--retention-window-sec`; when the cap is hit, the
//! emitter logs a warning to stderr (the motif may be truncated).

use crate::grammar::{Episode, MotifClass, MotifEngine, MotifGrammar};
use crate::residual::{ResidualSample, ResidualStream};
use std::collections::HashSet;

/// Key identifying a distinct episode for deduplication across
/// rescans. `t_start` is discretised to milliseconds so that
/// floating-point noise across rescans (there shouldn't be any —
/// `MotifEngine::run` is deterministic — but defensive) cannot cause
/// a duplicate emission.
type EpisodeKey = (MotifClass, Option<String>, i64);

fn episode_key(ep: &Episode) -> EpisodeKey {
    (ep.motif, ep.channel.clone(), (ep.t_start * 1000.0) as i64)
}

pub struct LiveEmitter {
    buffer: ResidualStream,
    engine: MotifEngine,
    emitted: HashSet<EpisodeKey>,
    retention_window_s: f64,
    max_samples: usize,
}

impl LiveEmitter {
    /// Construct a fresh emitter with the given motif grammar. The
    /// internal `ResidualStream` starts empty.
    pub fn new(grammar: MotifGrammar, retention_window_s: f64, max_samples: usize) -> Self {
        Self {
            buffer: ResidualStream::new("live-postgres"),
            engine: MotifEngine::new(grammar),
            emitted: HashSet::new(),
            retention_window_s,
            max_samples,
        }
    }

    /// Push this polling-tick's residuals into the buffer and return
    /// any newly-closed episodes. The returned list is time-ordered.
    pub fn push_samples(&mut self, samples: Vec<ResidualSample>) -> Vec<Episode> {
        if samples.is_empty() {
            return Vec::new();
        }
        for s in samples {
            self.buffer.push(s);
        }
        self.buffer.sort();
        let all = self.engine.run(&self.buffer);
        let mut fresh = Vec::new();
        for ep in all.iter() {
            let key = episode_key(ep);
            if self.emitted.insert(key) {
                fresh.push(ep.clone());
            }
        }
        self.trim(&all);
        fresh
    }

    /// Trim the in-memory buffer. Drops samples older than the
    /// earliest open-episode `t_start` (any episode that is still
    /// being tracked by the motif state machine), and falls back to
    /// the retention window if no episodes are open. Also enforces
    /// the absolute `max_samples` ceiling.
    fn trim(&mut self, all_episodes: &[Episode]) {
        let open_t: Option<f64> = all_episodes
            .iter()
            .filter(|e| !self.emitted.contains(&episode_key(e)))
            .map(|e| e.t_start)
            .reduce(f64::min);
        let last_t = self.buffer.samples.last().map(|s| s.t).unwrap_or(0.0);
        let retention_cutoff = last_t - self.retention_window_s;
        let cutoff = match open_t {
            Some(t) => t.min(retention_cutoff),
            None => retention_cutoff,
        };
        self.buffer.samples.retain(|s| s.t >= cutoff);
        if self.buffer.samples.len() > self.max_samples {
            let excess = self.buffer.samples.len() - self.max_samples;
            eprintln!(
                "warning: live buffer exceeded max_samples ({}); dropping {} oldest samples — open episodes may be truncated",
                self.max_samples, excess
            );
            self.buffer.samples.drain(0..excess);
        }
    }

    /// Total samples currently resident in the buffer. Useful for
    /// the poll-log telemetry row.
    pub fn buffer_len(&self) -> usize {
        self.buffer.samples.len()
    }

    /// Complete list of emitted episode keys so far. Used by the
    /// shutdown path to write a final episodes CSV with the full
    /// episode set.
    pub fn emitted_count(&self) -> usize {
        self.emitted.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::{plan_regression, ResidualClass};

    fn grammar() -> MotifGrammar {
        MotifGrammar::default()
    }

    #[test]
    fn emits_episode_once_across_rescans() {
        let mut em = LiveEmitter::new(grammar(), 3600.0, 1_000_000);
        // Generate a clean plan-regression onset: steady baseline
        // then sustained 3x latency for long enough to pass the 5s
        // min_dwell.
        let mut total_emitted = 0;
        for i in 0..60 {
            let t = i as f64;
            let baseline = 10.0;
            let latency = if i >= 20 { 30.0 } else { baseline };
            let mut tmp = ResidualStream::new("");
            plan_regression::push_latency(&mut tmp, t, "qA", latency, baseline);
            let new = em.push_samples(tmp.samples);
            total_emitted += new.len();
        }
        // At least one plan_regression_onset must have closed.
        assert!(total_emitted >= 1);
    }

    #[test]
    fn emitter_is_idempotent_on_empty_push() {
        let mut em = LiveEmitter::new(grammar(), 3600.0, 1_000_000);
        let out = em.push_samples(Vec::new());
        assert!(out.is_empty());
    }

    #[test]
    fn trim_respects_retention_window() {
        let mut em = LiveEmitter::new(grammar(), 10.0, 1_000_000);
        for i in 0..50 {
            let mut tmp = ResidualStream::new("");
            plan_regression::push_latency(&mut tmp, i as f64, "qA", 10.0, 10.0);
            em.push_samples(tmp.samples);
        }
        // Retention window = 10 s, no open episodes (steady baseline),
        // so the buffer should be roughly bounded to the last 10 s.
        assert!(em.buffer_len() <= 15, "buffer should be trimmed to ~10 s: {}", em.buffer_len());
    }

    #[test]
    fn residual_class_of_emitted_matches_motif_class() {
        // Sanity: the emitter's episodes reference their own motif class,
        // which maps to the residual class the samples were pushed on.
        let mut em = LiveEmitter::new(grammar(), 3600.0, 1_000_000);
        for i in 0..80 {
            let t = i as f64;
            let latency = if i >= 20 { 40.0 } else { 10.0 };
            let mut tmp = ResidualStream::new("");
            plan_regression::push_latency(&mut tmp, t, "qA", latency, 10.0);
            for ep in em.push_samples(tmp.samples) {
                assert_eq!(ep.motif.residual_class(), ResidualClass::PlanRegression);
            }
        }
    }
}
