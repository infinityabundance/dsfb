//! Residual sign computation: the primary inferential object.
//!
//! A [`ResidualSign`] encodes the instantaneous structural state of a residual
//! trajectory at a single observation point. It captures not just the magnitude
//! of deviation (which scalar thresholds already use) but the **direction**
//! (drift), **curvature** (slew), and **temporal persistence** of that deviation.
//!
//! This is the core insight of DSFB: residuals carry structured temporal
//! information that scalar alarm methods discard.

/// Source type for a residual measurement.
///
/// Each variant represents a distinct telemetry channel from which residuals
/// are derived. The observer computes residual signs independently per source,
/// then correlates across sources in the grammar layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResidualSource {
    /// Request/response latency (e.g., p50, p95, p99 in nanoseconds).
    Latency,
    /// Operations per second or bytes per second throughput.
    Throughput,
    /// Error rate (errors per total requests in a window).
    ErrorRate,
    /// Queue depth for bounded channels or task queues.
    QueueDepth,
    /// Consensus heartbeat round-trip time (e.g., Raft leader→follower).
    HeartbeatRtt,
    /// Async runtime task poll duration.
    PollDuration,
    /// Resident set size or heap allocation rate.
    MemoryUsage,
    /// Serialization/deserialization throughput or latency.
    SerdeLatency,
    /// gRPC/HTTP2 flow control window utilization.
    FlowControlWindow,
    /// DNS resolution latency.
    DnsLatency,
    /// Custom user-defined source with a static label.
    Custom(&'static str),
}

/// A single residual measurement sample from the observed system.
///
/// This is the **input** to the DSFB observer. It is accepted as an immutable
/// reference — no mutation of the sample or its source is possible through
/// this type.
#[derive(Debug, Clone, Copy)]
pub struct ResidualSample {
    /// The measured value (e.g., latency in nanoseconds).
    pub value: f64,
    /// The expected/baseline value for this source under current regime.
    pub baseline: f64,
    /// Monotonic timestamp in nanoseconds (or cycle index).
    pub timestamp_ns: u64,
    /// Which telemetry channel produced this sample.
    pub source: ResidualSource,
}

impl ResidualSample {
    /// Compute the raw residual: measured minus baseline.
    #[inline]
    pub fn residual(&self) -> f64 {
        self.value - self.baseline
    }
}

/// The residual sign: DSFB's primary inferential object.
///
/// Captures the instantaneous structural state of a residual trajectory:
/// - `residual` (r): raw deviation from baseline
/// - `drift` (ω): first derivative — direction and rate of change
/// - `slew` (α): second derivative — curvature / acceleration of change
///
/// These three quantities, together with the admissibility envelope and
/// grammar state, fully determine the structural interpretation.
#[derive(Debug, Clone, Copy)]
pub struct ResidualSign {
    /// Raw residual: r(k) = measured(k) - baseline(k).
    pub residual: f64,
    /// Drift: estimated first derivative of the residual trajectory.
    /// Positive drift = residual growing (degradation direction).
    pub drift: f64,
    /// Slew: estimated second derivative of the residual trajectory.
    /// Positive slew = drift accelerating (worsening curvature).
    pub slew: f64,
    /// Monotonic timestamp (nanoseconds) of this observation.
    pub timestamp_ns: u64,
    /// Source channel that produced this residual.
    pub source: ResidualSource,
}

/// Windowed residual sign estimator.
///
/// Computes drift and slew from a sliding window of residual samples
/// using finite differences. The window size `P` controls the persistence
/// requirement: drift must be sustained over P samples to register.
///
/// ## Failure Mode FM-02: Insufficient Persistence Window
///
/// If P is too small, transient noise triggers drift detection.
/// If P is too large, early degradation is filtered out.
/// The recommended range is P ∈ [20, 100] for typical distributed
/// system telemetry at 1-second sampling intervals.
pub struct ResidualEstimator {
    /// Ring buffer of recent residuals (fixed-size, stack-allocated).
    window: [f64; 128],
    /// Ring buffer of timestamps.
    timestamps: [u64; 128],
    /// Current write position in the ring buffer.
    head: usize,
    /// Number of samples received (saturates at window size).
    count: usize,
    /// Persistence window size (how many samples for drift/slew estimation).
    persistence_window: usize,
    /// Source channel this estimator tracks.
    source: ResidualSource,
}

impl ResidualEstimator {
    /// Create a new estimator for the given source with persistence window P.
    ///
    /// # Panics
    ///
    /// Panics if `persistence_window` is 0 or greater than 128.
    pub fn new(source: ResidualSource, persistence_window: usize) -> Self {
        assert!(
            persistence_window > 0 && persistence_window <= 128,
            "Persistence window must be in [1, 128], got {}",
            persistence_window
        );
        Self {
            window: [0.0; 128],
            timestamps: [0; 128],
            head: 0,
            count: 0,
            persistence_window,
            source,
        }
    }

    /// Ingest a new sample and compute the current residual sign.
    ///
    /// Accepts the sample as an immutable reference — the observer never
    /// modifies the sample or its source.
    pub fn observe(&mut self, sample: &ResidualSample) -> ResidualSign {
        let r = sample.residual();

        // Write into ring buffer
        self.window[self.head] = r;
        self.timestamps[self.head] = sample.timestamp_ns;
        self.head = (self.head + 1) % 128;
        if self.count < 128 {
            self.count += 1;
        }

        let drift = self.estimate_drift();
        let slew = self.estimate_slew();

        ResidualSign {
            residual: r,
            drift,
            slew,
            timestamp_ns: sample.timestamp_ns,
            source: self.source,
        }
    }

    /// Estimate drift (first derivative) via least-squares slope over the
    /// persistence window.
    ///
    /// Uses the standard formula: slope = (n·Σ(xy) - Σx·Σy) / (n·Σ(x²) - (Σx)²)
    /// where x is the sample index and y is the residual value.
    fn estimate_drift(&self) -> f64 {
        let n = self.count.min(self.persistence_window);
        if n < 2 {
            return 0.0;
        }

        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut sum_xy: f64 = 0.0;
        let mut sum_x2: f64 = 0.0;

        for i in 0..n {
            let idx = if self.head >= n {
                self.head - n + i
            } else {
                (128 + self.head - n + i) % 128
            };
            let x = i as f64;
            let y = self.window[idx];
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let nf = n as f64;
        let denom = nf * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-15 {
            return 0.0;
        }
        (nf * sum_xy - sum_x * sum_y) / denom
    }

    /// Estimate slew (second derivative) via difference of two half-window drifts.
    ///
    /// Splits the persistence window into two halves, computes drift on each,
    /// and reports the difference. This is a robust finite-difference approximation
    /// that avoids noise amplification from direct second differences.
    fn estimate_slew(&self) -> f64 {
        let n = self.count.min(self.persistence_window);
        if n < 4 {
            return 0.0;
        }

        let half = n / 2;

        // Drift of first half
        let drift_first = self.half_window_drift(0, half);
        // Drift of second half
        let drift_second = self.half_window_drift(half, n);

        // Slew is the change in drift
        drift_second - drift_first
    }

    /// Compute least-squares drift on a sub-window [start_offset, end_offset)
    /// relative to the current persistence window.
    fn half_window_drift(&self, start_offset: usize, end_offset: usize) -> f64 {
        let n = self.count.min(self.persistence_window);
        let sub_n = end_offset - start_offset;
        if sub_n < 2 {
            return 0.0;
        }

        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut sum_xy: f64 = 0.0;
        let mut sum_x2: f64 = 0.0;

        for i in 0..sub_n {
            let global_i = start_offset + i;
            let idx = if self.head >= n {
                self.head - n + global_i
            } else {
                (128 + self.head - n + global_i) % 128
            };
            let x = i as f64;
            let y = self.window[idx];
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let nf = sub_n as f64;
        let denom = nf * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-15 {
            return 0.0;
        }
        (nf * sum_xy - sum_x * sum_y) / denom
    }

    /// Reset the estimator state. Used when the observed system restarts
    /// or when a new workload phase begins.
    pub fn reset(&mut self) {
        self.window = [0.0; 128];
        self.timestamps = [0; 128];
        self.head = 0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_residual_produces_zero_drift_slew() {
        let mut est = ResidualEstimator::new(ResidualSource::Latency, 20);
        for i in 0..30 {
            let sample = ResidualSample {
                value: 100.0,
                baseline: 100.0,
                timestamp_ns: i * 1_000_000_000,
                source: ResidualSource::Latency,
            };
            let sign = est.observe(&sample);
            assert!((sign.residual).abs() < 1e-10);
        }
        // After enough samples, drift and slew should be ~0
        let sample = ResidualSample {
            value: 100.0,
            baseline: 100.0,
            timestamp_ns: 30_000_000_000,
            source: ResidualSource::Latency,
        };
        let sign = est.observe(&sample);
        assert!(sign.drift.abs() < 1e-10);
        assert!(sign.slew.abs() < 1e-10);
    }

    #[test]
    fn test_linear_drift_detected() {
        let mut est = ResidualEstimator::new(ResidualSource::Latency, 20);
        // Feed linearly increasing residuals: baseline=100, value = 100 + 0.5*i
        for i in 0..30u64 {
            let sample = ResidualSample {
                value: 100.0 + 0.5 * i as f64,
                baseline: 100.0,
                timestamp_ns: i * 1_000_000_000,
                source: ResidualSource::Latency,
            };
            est.observe(&sample);
        }
        let sample = ResidualSample {
            value: 100.0 + 15.5,
            baseline: 100.0,
            timestamp_ns: 31_000_000_000,
            source: ResidualSource::Latency,
        };
        let sign = est.observe(&sample);
        // Drift should be approximately 0.5 (the slope)
        assert!(
            (sign.drift - 0.5).abs() < 0.1,
            "Expected drift ~0.5, got {}",
            sign.drift
        );
        // Slew should be ~0 (constant drift)
        assert!(sign.slew.abs() < 0.2, "Expected slew ~0, got {}", sign.slew);
    }

    #[test]
    fn test_accelerating_drift_produces_positive_slew() {
        let mut est = ResidualEstimator::new(ResidualSource::Latency, 40);
        // Feed quadratically increasing residuals: r(k) = 0.01 * k^2
        for i in 0..50u64 {
            let sample = ResidualSample {
                value: 100.0 + 0.01 * (i as f64) * (i as f64),
                baseline: 100.0,
                timestamp_ns: i * 1_000_000_000,
                source: ResidualSource::Latency,
            };
            est.observe(&sample);
        }
        let sample = ResidualSample {
            value: 100.0 + 0.01 * 50.0 * 50.0,
            baseline: 100.0,
            timestamp_ns: 50_000_000_000,
            source: ResidualSource::Latency,
        };
        let sign = est.observe(&sample);
        // Slew should be positive (accelerating drift)
        assert!(
            sign.slew > 0.0,
            "Expected positive slew for quadratic growth, got {}",
            sign.slew
        );
    }

    #[test]
    fn test_source_preserved() {
        let mut est = ResidualEstimator::new(ResidualSource::HeartbeatRtt, 10);
        let sample = ResidualSample {
            value: 50.0,
            baseline: 40.0,
            timestamp_ns: 0,
            source: ResidualSource::HeartbeatRtt,
        };
        let sign = est.observe(&sample);
        assert_eq!(sign.source, ResidualSource::HeartbeatRtt);
    }
}
