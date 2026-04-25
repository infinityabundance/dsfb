//! Residual sign tuple σ(k) = (‖r(k)‖, ṙ(k), r̈(k)) and its sliding-window
//! estimator.
//!
//! The sign tuple is the coordinate of the **semiotic manifold** M_sem
//! ⊂ ℝ³ — DSFB's primary inferential object. Incumbent robotics
//! observers (Luenberger, Kalman, inverse-dynamics identification)
//! collapse residuals to a scalar or a covariance-shaped likelihood and
//! discard the trajectory. DSFB retains all three coordinates:
//! magnitude (what threshold alarms see), drift (what they discard
//! between alarms), and slew (the curvature signal for abrupt-onset
//! regime changes such as collisions or payload steps).
//!
//! ## Definitions
//!
//! σ(k) = (‖r(k)‖, ṙ(k), r̈(k))
//!
//! ṙ(k) = (1/W) Σ_{j=k-W+1}^{k} (‖r(j)‖ − ‖r(j-1)‖)
//!
//! r̈(k) = ṙ(k) − ṙ(k-1)
//!
//! Below-nominal-floor samples (e.g. residual magnitude below the
//! known sensor noise floor) contribute **zero** drift and slew, so
//! DSFB does not attribute structural meaning to pure-noise windows.

/// A single residual sign tuple.
///
/// All DSFB grammar states, motif classifications, and policy
/// decisions derive from this object alone. The field names deliberately
/// mirror dsfb-rf (`norm`, `drift`, `slew`) so cross-crate tooling can
/// consume sign tuples uniformly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignTuple {
    /// ‖r(k)‖ — instantaneous residual norm. What threshold detectors see.
    pub norm: f64,
    /// ṙ(k) — mean first-difference over the drift window.
    pub drift: f64,
    /// r̈(k) — slew (drift curvature).
    pub slew: f64,
}

impl SignTuple {
    /// Construct a sign tuple from explicit components.
    #[inline]
    #[must_use]
    pub const fn new(norm: f64, drift: f64, slew: f64) -> Self {
        debug_assert!(norm.is_finite() || norm.is_nan(), "norm must be finite or NaN");
        Self { norm, drift, slew }
    }

    /// The zero sign tuple — residual at rest, no drift, no curvature.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self { norm: 0.0, drift: 0.0, slew: 0.0 }
    }

    /// Returns `true` if drift is positive (outward motion relative to nominal).
    #[inline]
    #[must_use]
    pub fn is_outward_drift(&self) -> bool {
        self.drift > 0.0
    }

    /// Returns `true` if the slew magnitude exceeds the abrupt-slew
    /// threshold `delta_s`.
    #[inline]
    #[must_use]
    pub fn is_abrupt_slew(&self, delta_s: f64) -> bool {
        debug_assert!(delta_s >= 0.0, "delta_s must be non-negative");
        crate::math::abs_f64(self.slew) > delta_s
    }
}

impl Default for SignTuple {
    fn default() -> Self {
        Self::zero()
    }
}

/// Fixed-capacity sliding window for computing sign tuples from a
/// streaming residual sequence.
///
/// Generic parameter `W` is the drift-window width. All storage is
/// stack-allocated: no heap, no `unsafe`, no `std`.
pub struct SignWindow<const W: usize> {
    norms: [f64; W],
    prev_drift: f64,
    head: usize,
    /// Saturates at `W` — we never need a larger count.
    count: usize,
}

impl<const W: usize> SignWindow<W> {
    /// Create an empty sliding window.
    #[must_use]
    pub const fn new() -> Self {
        Self { norms: [0.0; W], prev_drift: 0.0, head: 0, count: 0 }
    }

    /// Insert the next residual norm and return the current sign tuple.
    ///
    /// When `below_floor` is `true`, the sample is stored (so future
    /// diffs see it) but drift and slew are forced to zero for this
    /// observation to avoid attributing structural meaning to
    /// noise-floor samples.
    pub fn push(&mut self, norm: f64, below_floor: bool) -> SignTuple {
        debug_assert!(W > 0, "SignWindow<0> is degenerate — W must be ≥ 1");
        debug_assert!(self.head < W.max(1), "head invariant violated");

        if W == 0 {
            // Degenerate configuration — return an all-zero tuple rather
            // than reading or writing the zero-length array. Guarded by
            // debug_assert above; release builds short-circuit safely.
            return SignTuple::zero();
        }

        self.norms[self.head] = norm;
        self.head = (self.head + 1) % W;
        if self.count < W {
            self.count += 1;
        }

        if below_floor || self.count < 2 {
            self.prev_drift = 0.0;
            return SignTuple::new(norm, 0.0, 0.0);
        }

        // Mean first-difference across the filled portion of the buffer.
        let filled = self.count.min(W);
        let mut sum_diff = 0.0_f64;
        let mut n_diffs = 0_usize;
        let mut i = 1_usize;
        while i < filled {
            let cur = (self.head + W - 1 - (i - 1)) % W;
            let prev = (self.head + W - 1 - i) % W;
            sum_diff += self.norms[cur] - self.norms[prev];
            n_diffs += 1;
            i += 1;
        }

        let drift = if n_diffs > 0 { sum_diff / n_diffs as f64 } else { 0.0 };
        let slew = drift - self.prev_drift;
        self.prev_drift = drift;
        SignTuple::new(norm, drift, slew)
    }

    /// Reset the window (e.g. after a context transition suppression
    /// period has ended).
    pub fn reset(&mut self) {
        self.norms = [0.0; W];
        self.prev_drift = 0.0;
        self.head = 0;
        self.count = 0;
    }

    /// The number of samples accumulated so far (saturates at `W`).
    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<const W: usize> Default for SignWindow<W> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_tuple_is_rest() {
        let s = SignTuple::zero();
        assert_eq!(s.norm, 0.0);
        assert!(!s.is_outward_drift());
        assert!(!s.is_abrupt_slew(0.01));
    }

    #[test]
    fn outward_drift_is_positive_drift() {
        assert!(SignTuple::new(0.1, 0.01, 0.0).is_outward_drift());
        assert!(!SignTuple::new(0.1, 0.0, 0.0).is_outward_drift());
        assert!(!SignTuple::new(0.1, -0.01, 0.0).is_outward_drift());
    }

    #[test]
    fn abrupt_slew_threshold_is_absolute() {
        assert!(SignTuple::new(0.1, 0.0, 0.1).is_abrupt_slew(0.05));
        assert!(SignTuple::new(0.1, 0.0, -0.1).is_abrupt_slew(0.05));
        assert!(!SignTuple::new(0.1, 0.0, 0.01).is_abrupt_slew(0.05));
    }

    #[test]
    fn window_sub_floor_forces_zero_drift() {
        let mut w = SignWindow::<5>::new();
        for i in 0..5u32 {
            let s = w.push(i as f64 * 0.1, true);
            assert_eq!(s.drift, 0.0);
            assert_eq!(s.slew, 0.0);
        }
    }

    #[test]
    fn window_monotone_increase_has_positive_drift() {
        let mut w = SignWindow::<5>::new();
        for i in 0..8u32 {
            let s = w.push(i as f64 * 0.01, false);
            if i >= 2 {
                assert!(s.drift > 0.0, "expected positive drift, got {}", s.drift);
            }
        }
    }

    #[test]
    fn window_constant_input_has_zero_drift() {
        let mut w = SignWindow::<5>::new();
        let mut last = None;
        for _ in 0..8 {
            last = Some(w.push(0.42, false));
        }
        let s = last.expect("pushed at least once");
        assert!(crate::math::abs_f64(s.drift) < 1e-12, "drift = {}", s.drift);
    }

    #[test]
    fn window_reset_clears_state() {
        let mut w = SignWindow::<5>::new();
        for i in 0..5u32 {
            w.push(i as f64 * 0.1, false);
        }
        w.reset();
        assert_eq!(w.count(), 0);
        let s = w.push(0.5, false);
        assert_eq!(s.drift, 0.0);
    }
}
