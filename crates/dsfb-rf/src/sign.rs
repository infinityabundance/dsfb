//! Residual sign tuple: (‖r‖, ṙ, r̈)
//!
//! The semiotic manifold coordinate. This is the primary inferential object
//! from which all higher-level DSFB states derive.
//!
//! ## Mathematical Definition (paper §B.2)
//!
//! σ(k) = (‖r(k)‖, ṙ(k), r̈(k))
//!
//! ṙ(k) = (1/W) Σ_{j=k-W+1}^{k} (‖r(j)‖ - ‖r(j-1)‖)
//! r̈(k) = ṙ(k) - ṙ(k-1)
//!
//! Sub-threshold observations (SNR < SNR_floor) contribute zero to
//! drift and slew sums (missingness-aware signal validity).
//!
//! ## Key distinction from Luenberger/Kalman (paper §I-C, Table I)
//!
//! A linear observer gain matrix L maps r(k) → L·r(k), collapsing the
//! manifold to a scalar. The sign tuple preserves all three coordinates:
//! magnitude, drift direction, and trajectory curvature.

use crate::platform::SnrFloor;

/// The residual sign tuple σ(k) = (‖r‖, ṙ, r̈).
///
/// This is the coordinate on the semiotic manifold M_sem ⊂ ℝ³.
/// All DSFB grammar states, motif classifications, and DSA scores
/// are derived from this object alone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignTuple {
    /// ‖r(k)‖ — instantaneous residual norm. What threshold detectors see.
    pub norm: f32,
    /// ṙ(k) — finite-difference drift rate. What threshold detectors discard.
    pub drift: f32,
    /// r̈(k) — slew (trajectory curvature). Abrupt regime change signal.
    pub slew: f32,
}

impl SignTuple {
    /// Construct a sign tuple directly from components.
    #[inline]
    pub const fn new(norm: f32, drift: f32, slew: f32) -> Self {
        Self { norm, drift, slew }
    }

    /// Zero sign tuple — admissible baseline.
    #[inline]
    pub const fn zero() -> Self {
        Self { norm: 0.0, drift: 0.0, slew: 0.0 }
    }

    /// Returns true if drift is persistently outward (drift > 0).
    #[inline]
    pub fn is_outward_drift(&self) -> bool {
        self.drift > 0.0
    }

    /// Returns true if slew magnitude exceeds threshold δ_s.
    #[inline]
    pub fn is_abrupt_slew(&self, delta_s: f32) -> bool {
        self.slew.abs() > delta_s
    }
}

/// Fixed-capacity sliding window for computing sign tuples.
///
/// Generic parameter `W` is the window width. All storage is stack-allocated.
/// No heap allocation, no std, no unsafe.
pub struct SignWindow<const W: usize> {
    /// Circular buffer of recent residual norms.
    norms: [f32; W],
    /// Previous drift estimate (for slew computation).
    prev_drift: f32,
    /// Write position in the circular buffer.
    head: usize,
    /// Number of valid observations inserted so far (saturates at W).
    count: usize,
}

impl<const W: usize> SignWindow<W> {
    /// Create a new empty sign window.
    pub const fn new() -> Self {
        Self {
            norms: [0.0; W],
            prev_drift: 0.0,
            head: 0,
            count: 0,
        }
    }

    /// Push a new residual norm observation and return the current sign tuple.
    ///
    /// If `sub_threshold` is true (SNR below floor), the norm is stored as-is
    /// but drift and slew are forced to zero per the missingness-aware signal
    /// validity rule (paper §IX-C, §B.2).
    pub fn push(&mut self, norm: f32, sub_threshold: bool, snr_floor: SnrFloor) -> SignTuple {
        // snr_floor is consumed for type-safety at the API boundary; the
        // sub-threshold decision has already been applied by the caller and
        // is passed via the `sub_threshold` flag.
        core::hint::black_box(snr_floor);

        // Write norm into circular buffer
        self.norms[self.head] = norm;
        self.head = (self.head + 1) % W;
        if self.count < W {
            self.count += 1;
        }

        if sub_threshold || self.count < 2 {
            // Sub-threshold: zero drift and slew per paper §B.2
            self.prev_drift = 0.0;
            return SignTuple::new(norm, 0.0, 0.0);
        }

        // Compute mean first-difference over filled portion of window
        let filled = self.count.min(W);
        let mut sum_diff = 0.0_f32;
        let mut n_diffs = 0usize;

        for i in 1..filled {
            let cur_idx = (self.head + W - 1 - (i - 1)) % W;
            let prev_idx = (self.head + W - 1 - i) % W;
            sum_diff += self.norms[cur_idx] - self.norms[prev_idx];
            n_diffs += 1;
        }

        let drift = if n_diffs > 0 {
            sum_diff / n_diffs as f32
        } else {
            0.0
        };
        let slew = drift - self.prev_drift;
        self.prev_drift = drift;

        SignTuple::new(norm, drift, slew)
    }

    /// Reset the window (e.g., after a waveform transition suppression period).
    pub fn reset(&mut self) {
        self.norms = [0.0; W];
        self.prev_drift = 0.0;
        self.head = 0;
        self.count = 0;
    }
}

impl<const W: usize> Default for SignWindow<W> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SnrFloor;

    #[test]
    fn sign_tuple_zero_is_admissible() {
        let s = SignTuple::zero();
        assert_eq!(s.norm, 0.0);
        assert!(!s.is_outward_drift());
        assert!(!s.is_abrupt_slew(0.01));
    }

    #[test]
    fn window_drift_monotone_increase() {
        let mut w = SignWindow::<5>::new();
        let floor = SnrFloor::default();
        // Feed monotonically increasing norms — drift should be positive
        let mut last_drift = -f32::INFINITY;
        for i in 0..8u32 {
            let norm = i as f32 * 0.01;
            let sig = w.push(norm, false, floor);
            if i >= 2 {
                assert!(sig.drift >= 0.0, "drift should be non-negative for increasing norms");
                let _ = last_drift; // suppress unused warning
                last_drift = sig.drift;
            }
        }
        let _ = last_drift;
    }

    #[test]
    fn window_sub_threshold_forces_zero_drift() {
        let mut w = SignWindow::<5>::new();
        let floor = SnrFloor::default();
        // Sub-threshold observations must not contribute drift
        for i in 0..5u32 {
            let norm = i as f32 * 0.05;
            let sig = w.push(norm, true, floor);
            assert_eq!(sig.drift, 0.0);
            assert_eq!(sig.slew, 0.0);
        }
    }

    #[test]
    fn window_reset_clears_state() {
        let mut w = SignWindow::<5>::new();
        let floor = SnrFloor::default();
        for i in 0..5u32 {
            w.push(i as f32 * 0.1, false, floor);
        }
        w.reset();
        let sig = w.push(0.05, false, floor);
        // After reset, count=1, so drift/slew should be zero
        assert_eq!(sig.drift, 0.0);
    }
}
