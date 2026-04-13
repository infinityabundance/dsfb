/// DSFB Oil & Gas — Drift and Slew Computation
///
/// Implements Equations (2) and (3) from the paper.
/// All functions are pure: no I/O, no side-effects, deterministic.

#[cfg(feature = "alloc")]
use crate::types::ResidualTriple;
#[cfg(feature = "alloc")]
use crate::types::ResidualSample;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Drift estimator (causal sliding window mean)
// ─────────────────────────────────────────────────────────────────────────────

/// Causal, sliding-window drift estimator.
///
/// Maintains a ring buffer of the most recent `window` residual values.
/// δ_k^(w) = (1/w) Σ_{j=k−w+1}^{k} r_j  (Equation 2, paper).
///
/// For k < w−1 the effective window is k+1 (initialisation period).
#[cfg(feature = "alloc")]
pub struct DriftEstimator {
    window: usize,
    buffer: Vec<f64>,
    head: usize,
    filled: usize,
    running_sum: f64,
}

#[cfg(feature = "alloc")]
impl DriftEstimator {
    pub fn new(window: usize) -> Self {
        assert!(window > 0, "drift window must be positive");
        DriftEstimator {
            window,
            buffer: {
                let mut v = Vec::with_capacity(window);
                v.resize(window, 0.0);
                v
            },
            head: 0,
            filled: 0,
            running_sum: 0.0,
        }
    }

    /// Push one residual value and return the current drift estimate.
    pub fn push(&mut self, r: f64) -> f64 {
        // Subtract the value being evicted from the running sum.
        if self.filled == self.window {
            self.running_sum -= self.buffer[self.head];
        }
        self.buffer[self.head] = r;
        self.running_sum += r;
        self.head = (self.head + 1) % self.window;
        if self.filled < self.window {
            self.filled += 1;
        }
        self.running_sum / self.filled as f64
    }

    pub fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|v| *v = 0.0);
        self.head = 0;
        self.filled = 0;
        self.running_sum = 0.0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Slew estimator (first-order finite difference)
// ─────────────────────────────────────────────────────────────────────────────

/// Causal first-order finite-difference slew estimator.
///
/// σ_k = (r_k − r_{k−1}) / Δt  (Equation 3, paper).
/// For k=0, σ_0 = 0 by convention.
pub struct SlewEstimator {
    prev_r: Option<f64>,
}

impl SlewEstimator {
    pub fn new() -> Self {
        SlewEstimator { prev_r: None }
    }

    /// Push one residual value and return the slew.
    /// `dt` is the elapsed time since the previous sample (seconds).
    pub fn push(&mut self, r: f64, dt: f64) -> f64 {
        let sigma = match self.prev_r {
            None => 0.0,
            Some(prev) => {
                if dt > 0.0 { (r - prev) / dt } else { 0.0 }
            }
        };
        self.prev_r = Some(r);
        sigma
    }

    pub fn reset(&mut self) {
        self.prev_r = None;
    }
}

impl Default for SlewEstimator {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Residual processor: produces ResidualTriple from a stream of ResidualSample
// ─────────────────────────────────────────────────────────────────────────────

/// Stateful processor that converts a stream of [`ResidualSample`] into a
/// stream of [`ResidualTriple`].
///
/// Holds the drift and slew estimators.  Not thread-safe; use one per channel.
#[cfg(feature = "alloc")]
pub struct ResidualProcessor {
    drift: DriftEstimator,
    slew: SlewEstimator,
    prev_ts: Option<f64>,
}

#[cfg(feature = "alloc")]
impl ResidualProcessor {
    pub fn new(drift_window: usize) -> Self {
        ResidualProcessor {
            drift: DriftEstimator::new(drift_window),
            slew: SlewEstimator::new(),
            prev_ts: None,
        }
    }

    /// Process one sample and return the corresponding triple.
    pub fn process(&mut self, sample: &ResidualSample) -> ResidualTriple {
        let r = sample.residual();

        // ── OOB guard ─────────────────────────────────────────────────────────
        // If the sample is non-finite (NaN, ±∞), return a sentinel triple
        // WITHOUT updating the drift ring buffer, the slew estimator, or
        // prev_ts.  This prevents IEEE 754 NaN poisoning of the ring-buffer
        // running_sum, preserving state continuity for the next finite sample.
        // The grammar automaton's OOB check will emit GrammarState::SensorFault.
        if !r.is_finite() {
            return ResidualTriple {
                r,
                delta: f64::NAN,
                sigma: f64::NAN,
                timestamp: sample.timestamp,
            };
        }

        let dt = match self.prev_ts {
            None => 1.0, // first step: dt undefined; slew = 0 by convention
            Some(prev_ts) => {
                let d = sample.timestamp - prev_ts;
                if d <= 0.0 { 1.0 } else { d }
            }
        };
        self.prev_ts = Some(sample.timestamp);

        let delta = self.drift.push(r);
        let sigma = self.slew.push(r, dt);

        ResidualTriple {
            r,
            delta,
            sigma,
            timestamp: sample.timestamp,
        }
    }

    pub fn reset(&mut self) {
        self.drift.reset();
        self.slew.reset();
        self.prev_ts = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn drift_window_1_equals_residual() {
        let mut d = DriftEstimator::new(1);
        for v in [1.0f64, 2.0, 3.0, -1.0] {
            let out = d.push(v);
            // With window=1, drift == residual itself
            assert!((out - v).abs() < 1e-12, "drift w=1 mismatch");
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn drift_constant_input_converges() {
        let mut d = DriftEstimator::new(10);
        for _ in 0..20 {
            let v = d.push(5.0);
            // After fill, mean of constant = constant
            if d.filled == d.window { assert!((v - 5.0).abs() < 1e-12); }
        }
    }

    #[test]
    fn slew_first_step_is_zero() {
        let mut s = SlewEstimator::new();
        assert_eq!(s.push(10.0, 1.0), 0.0);
    }

    #[test]
    fn slew_correct_rate() {
        let mut s = SlewEstimator::new();
        s.push(0.0, 1.0);
        let sigma = s.push(5.0, 0.5); // 5/0.5 = 10
        assert!((sigma - 10.0).abs() < 1e-12);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn processor_no_nan() {
        let mut proc = ResidualProcessor::new(5);
        for i in 0..20 {
            let s = ResidualSample::new(i as f64, i as f64 * 1.1, i as f64, "test");
            let t = proc.process(&s);
            assert!(!t.r.is_nan());
            assert!(!t.delta.is_nan());
            assert!(!t.sigma.is_nan());
        }
    }
}
