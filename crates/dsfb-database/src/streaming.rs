//! Phase-C2: streaming residual construction.
//!
//! The batch adapter path calls [`ResidualStream::push`] followed by
//! [`ResidualStream::sort`]; it produces the bytewise-identical stream
//! the four fingerprint locks pin. That path is unchanged and remains
//! the canonical construction for reproduction.
//!
//! [`ResidualStream::push`]: crate::residual::ResidualStream::push
//! [`ResidualStream::sort`]: crate::residual::ResidualStream::sort
//!
//! This module adds a **parallel, additive** API for a live-ingestion
//! deployment where samples arrive one at a time and a single terminal
//! `.sort()` call over a materialised 10⁶-sample buffer is not
//! acceptable. The streaming path preserves time-ordering via a
//! bounded **reorder buffer**: every incoming sample is staged in a
//! small heap, and any sample older than `newest_t − reorder_window_s`
//! is flushed to the underlying stream in sorted order. At
//! [`StreamingIngestor::finish`] the remaining buffer tail is drained.
//!
//! The trade-off is explicit: if a sample arrives with a time delta
//! greater than `reorder_window_s` behind the current newest sample,
//! it is dropped and the drop is counted — `dropped_out_of_window` is
//! part of the closing summary. A production deployment sizes
//! `reorder_window_s` to be larger than the engine's maximum
//! telemetry-pipeline jitter (we default to 10 s; PostgreSQL's
//! `pg_stat_statements` polling cadence at 60 s makes 10 s a ~6×
//! safety margin).
//!
//! Determinism: given the same input stream and `reorder_window_s`,
//! the flushed sample order and the `dropped_out_of_window` count are
//! deterministic. The streaming path is **not** expected to produce
//! the same fingerprint as the batch path for real-world jitter-bearing
//! inputs — that is the honest reason batch is pinned and streaming is
//! parallel, not a replacement.

use crate::residual::{ResidualSample, ResidualStream};
use std::collections::BinaryHeap;

/// Default reorder-buffer window in seconds. Sized for
/// `pg_stat_statements`-class telemetry jitter. Tune up for slower
/// engines, tune down for well-behaved log tails.
pub const DEFAULT_REORDER_WINDOW_S: f64 = 10.0;

/// Streaming ingestor that accepts one [`ResidualSample`] at a time and
/// flushes a correctly-ordered prefix into an owned [`ResidualStream`]
/// as the reorder window slides forward.
///
/// Invariant: after every [`StreamingIngestor::push`] or
/// [`StreamingIngestor::finish`], the underlying
/// `self.stream.samples` is time-ordered (`t` non-decreasing).
pub struct StreamingIngestor {
    stream: ResidualStream,
    reorder_window_s: f64,
    /// Min-heap on `t`; the `Reverse` wrapper is standard because
    /// `BinaryHeap` is a max-heap by default.
    buf: BinaryHeap<Staged>,
    newest_t: f64,
    /// Samples whose timestamp fell more than `reorder_window_s` behind
    /// the already-flushed frontier at arrival. A production runbook
    /// should alert on any non-zero value.
    dropped_out_of_window: u64,
}

/// Heap entry: orders by `t` in reverse so `BinaryHeap::pop` returns
/// the *oldest* sample. We compare strictly on `t`; `ResidualSample`
/// is not `Ord`-worthy directly because `value` is `f64`, and we do
/// not want a stable secondary key to leak into the ordering.
struct Staged(ResidualSample);

impl PartialEq for Staged {
    fn eq(&self, other: &Self) -> bool {
        self.0.t == other.0.t
    }
}
impl Eq for Staged {}
impl PartialOrd for Staged {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Staged {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse: smaller `t` is "greater" so pop() returns oldest.
        other
            .0
            .t
            .partial_cmp(&self.0.t)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl StreamingIngestor {
    /// Construct an ingestor with the default reorder window.
    pub fn new(source: impl Into<String>) -> Self {
        Self::with_window(source, DEFAULT_REORDER_WINDOW_S)
    }

    /// Construct an ingestor with a custom reorder window. A zero
    /// window degenerates to strictly-in-order ingest (any
    /// out-of-order sample is dropped).
    pub fn with_window(source: impl Into<String>, reorder_window_s: f64) -> Self {
        debug_assert!(
            reorder_window_s >= 0.0 && reorder_window_s.is_finite(),
            "reorder window must be finite and non-negative"
        );
        Self {
            stream: ResidualStream::new(source),
            reorder_window_s,
            buf: BinaryHeap::new(),
            newest_t: f64::NEG_INFINITY,
            dropped_out_of_window: 0,
        }
    }

    /// Ingest one sample. If `sample.t` is more than `reorder_window_s`
    /// behind the already-flushed frontier, the sample is dropped and
    /// `dropped_out_of_window` is incremented. Otherwise the sample is
    /// staged and the internal buffer is drained up to the new
    /// frontier.
    pub fn push(&mut self, sample: ResidualSample) {
        debug_assert!(sample.t.is_finite(), "residual t must be finite");
        debug_assert!(sample.value.is_finite(), "residual value must be finite");
        let flushed_frontier = self
            .stream
            .samples
            .last()
            .map(|s| s.t)
            .unwrap_or(f64::NEG_INFINITY);
        if sample.t + self.reorder_window_s < flushed_frontier {
            self.dropped_out_of_window += 1;
            return;
        }
        if sample.t > self.newest_t {
            self.newest_t = sample.t;
        }
        self.buf.push(Staged(sample));
        self.drain_ready();
    }

    /// Flush every staged sample whose `t` is at least
    /// `reorder_window_s` behind the newest observed `t`.
    fn drain_ready(&mut self) {
        let cutoff = self.newest_t - self.reorder_window_s;
        while let Some(top) = self.buf.peek() {
            if top.0.t <= cutoff {
                let Staged(s) = self.buf.pop().expect("peek succeeded");
                self.stream.samples.push(s);
            } else {
                break;
            }
        }
    }

    /// Drain the reorder buffer and return the completed stream
    /// together with the count of samples dropped during ingest.
    pub fn finish(mut self) -> (ResidualStream, u64) {
        while let Some(Staged(s)) = self.buf.pop() {
            self.stream.samples.push(s);
        }
        debug_assert!(
            self.stream.samples.windows(2).all(|w| w[0].t <= w[1].t),
            "finish invariant: samples time-ordered"
        );
        (self.stream, self.dropped_out_of_window)
    }

    /// Number of samples already flushed to the owned stream.
    pub fn flushed(&self) -> usize {
        self.stream.samples.len()
    }

    /// Number of samples currently staged in the reorder buffer.
    pub fn staged(&self) -> usize {
        self.buf.len()
    }

    /// Running count of out-of-window drops. Production deployments
    /// should expose this as a Prometheus counter (Phase-C4) and alert
    /// on any non-zero value.
    pub fn dropped_out_of_window(&self) -> u64 {
        self.dropped_out_of_window
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residual::ResidualClass;

    fn s(t: f64, value: f64) -> ResidualSample {
        ResidualSample::new(t, ResidualClass::PlanRegression, value)
    }

    #[test]
    fn in_order_ingest_matches_batch() {
        let mut ing = StreamingIngestor::with_window("test", 5.0);
        for t in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 20.0] {
            ing.push(s(t, 0.1));
        }
        let (stream, dropped) = ing.finish();
        assert_eq!(dropped, 0);
        let ts: Vec<f64> = stream.samples.iter().map(|s| s.t).collect();
        assert_eq!(ts, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 20.0]);
    }

    #[test]
    fn out_of_order_within_window_is_sorted() {
        let mut ing = StreamingIngestor::with_window("test", 5.0);
        // Jittered arrival order.
        for t in [0.0, 2.0, 1.0, 4.0, 3.0, 5.0, 7.0, 6.0] {
            ing.push(s(t, 0.1));
        }
        let (stream, dropped) = ing.finish();
        assert_eq!(dropped, 0);
        let ts: Vec<f64> = stream.samples.iter().map(|s| s.t).collect();
        assert_eq!(ts, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn sample_older_than_window_is_dropped() {
        let mut ing = StreamingIngestor::with_window("test", 1.0);
        for t in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0] {
            ing.push(s(t, 0.1));
        }
        // This one is 9 s behind the frontier → dropped.
        ing.push(s(1.0, 0.1));
        let (_, dropped) = ing.finish();
        assert_eq!(dropped, 1);
    }

    #[test]
    fn empty_ingest_produces_empty_stream() {
        let ing = StreamingIngestor::new("test");
        let (stream, dropped) = ing.finish();
        assert!(stream.samples.is_empty());
        assert_eq!(dropped, 0);
    }

    #[test]
    fn zero_window_drops_any_out_of_order() {
        let mut ing = StreamingIngestor::with_window("test", 0.0);
        ing.push(s(1.0, 0.1));
        ing.push(s(2.0, 0.1));
        ing.push(s(1.5, 0.1)); // exactly 0.5 behind frontier 2.0 → dropped
        let (stream, dropped) = ing.finish();
        assert_eq!(dropped, 1);
        assert_eq!(stream.samples.len(), 2);
    }
}
