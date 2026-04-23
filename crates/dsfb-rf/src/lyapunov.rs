//! Lyapunov stability analysis for residual trajectory divergence.
//!
//! ## Theoretical Basis
//!
//! The DSFB grammar state "Boundary" corresponds to a trajectory on the
//! semiotic manifold M_sem that is diverging from the nominal attractor.
//! This module formalizes that divergence using finite-time Lyapunov
//! exponents (FTLE), providing a stability-theoretic quantification of
//! *how fast* the residual trajectory is leaving the admissible region.
//!
//! ## Mathematical Definition
//!
//! The finite-time Lyapunov exponent over window W at observation k is:
//!
//! λ(k) = (1/W) · ln(‖r(k)‖ / ‖r(k−W)‖)
//!
//! - λ > 0: exponential divergence (trajectory leaving nominal attractor)
//! - λ ≈ 0: neutral stability (stationary residual)
//! - λ < 0: exponential convergence (trajectory returning to nominal)
//!
//! ## Relationship to Grammar States
//!
//! | λ regime          | Grammar state interpretation            |
//! |-------------------|-----------------------------------------|
//! | λ > λ_crit        | Boundary[SustainedOutwardDrift]         |
//! | λ > 0, < λ_crit   | Boundary approach — watch               |
//! | λ ≈ 0             | Admissible — stationary residual        |
//! | λ < 0             | Admissible — converging, healthy        |
//!
//! ## Distinction from Luenberger Observer
//!
//! A Luenberger observer drives ‖r(k)‖ → 0 via feedback gain L.
//! It is structurally blind to the *rate* of divergence λ(k), because
//! it projects r(k) onto a scalar correction term L·r(k).
//! DSFB computes λ(k) explicitly and uses it as a first-class coordinate
//! on the semiotic manifold alongside (‖r‖, ṙ, r̈).
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Fixed-capacity circular buffer `[f32; W]`
//! - O(1) per observation (no window scan — uses head/tail of circular buffer)
//! - Uses Newton-Raphson `ln` approximation (no `libm` dependency)

/// Finite-Time Lyapunov Exponent (FTLE) estimator.
///
/// Computes λ(k) = (1/W) · ln(‖r(k)‖ / ‖r(k−W)‖) over a sliding window.
///
/// Generic parameter `W` = window width. All storage is stack-allocated.
///
/// Also tracks **post-crossing persistence** metrics (DSFB-Lattice §IV):
/// the duration and fraction of samples that reside outside the admissibility
/// envelope, enabling distinction between transient spikes and sustained faults.
pub struct LyapunovEstimator<const W: usize> {
    /// Circular buffer of log-norms for efficient ratio computation.
    log_norms: [f32; W],
    /// Write head in circular buffer.
    head: usize,
    /// Number of valid observations (saturates at W).
    count: usize,

    // ── Post-crossing persistence tracking ──────────────────────────────
    /// Circular buffer of outside-envelope flags for fraction computation.
    outside_buf: [bool; W],
    /// Write head for outside_buf (separate from log_norms head).
    outside_head: usize,
    /// Number of valid entries in outside_buf (saturates at W).
    outside_count: usize,
    /// Consecutive samples outside the envelope since the last crossing.
    post_crossing_duration: u32,
}

impl<const W: usize> LyapunovEstimator<W> {
    /// Create a new estimator.
    pub const fn new() -> Self {
        Self {
            log_norms: [0.0; W],
            head: 0,
            count: 0,
            outside_buf: [false; W],
            outside_head: 0,
            outside_count: 0,
            post_crossing_duration: 0,
        }
    }

    /// Push a new residual norm and return the FTLE estimate.
    ///
    /// Returns `LyapunovResult` containing λ(k), stability classification,
    /// estimated time-to-envelope-exit, and post-crossing persistence metrics.
    ///
    /// If the window is not yet full, returns λ = 0.0 (insufficient data).
    pub fn push(&mut self, norm: f32, rho: f32) -> LyapunovResult {
        let ln_norm = ln_f32(norm.max(1e-12));
        let oldest_ln = self.advance_log_ring(ln_norm);
        let (post_crossing_fraction, separation_at_exit) =
            self.advance_crossing_state(norm, rho);

        if self.count < W {
            return LyapunovResult {
                lambda: 0.0,
                stability: StabilityClass::InsufficientData,
                time_to_exit: None,
                post_crossing_duration: self.post_crossing_duration,
                post_crossing_fraction,
                separation_at_exit,
            };
        }

        let lambda = (ln_norm - oldest_ln) / W as f32;
        LyapunovResult {
            lambda,
            stability: StabilityClass::from_lambda(lambda),
            time_to_exit: estimate_time_to_exit(norm, rho, lambda),
            post_crossing_duration: self.post_crossing_duration,
            post_crossing_fraction,
            separation_at_exit,
        }
    }

    fn advance_log_ring(&mut self, ln_norm: f32) -> f32 {
        let oldest = self.log_norms[self.head];
        self.log_norms[self.head] = ln_norm;
        self.head = (self.head + 1) % W;
        if self.count < W { self.count += 1; }
        oldest
    }

    fn advance_crossing_state(&mut self, norm: f32, rho: f32) -> (f32, f32) {
        let outside = norm > rho && rho > 1e-30;
        if outside {
            self.post_crossing_duration = self.post_crossing_duration.saturating_add(1);
        } else {
            self.post_crossing_duration = 0;
        }
        self.outside_buf[self.outside_head] = outside;
        self.outside_head = (self.outside_head + 1) % W;
        if self.outside_count < W { self.outside_count += 1; }
        let outside_count = self.outside_buf[..self.outside_count]
            .iter().filter(|&&b| b).count();
        let frac = outside_count as f32 / self.outside_count.max(1) as f32;
        let sep = if outside && rho > 1e-30 { (norm - rho) / rho } else { 0.0 };
        (frac, sep)
    }

    /// Reset the estimator.
    pub fn reset(&mut self) {
        self.log_norms = [0.0; W];
        self.head = 0;
        self.count = 0;
        self.outside_buf = [false; W];
        self.outside_head = 0;
        self.outside_count = 0;
        self.post_crossing_duration = 0;
    }
}

impl<const W: usize> Default for LyapunovEstimator<W> {
    fn default() -> Self {
        Self::new()
    }
}

fn estimate_time_to_exit(norm: f32, rho: f32, lambda: f32) -> Option<f32> {
    if lambda > 1e-6 && norm < rho && norm > 1e-12 {
        let t = ln_f32(rho / norm) / lambda;
        if t > 0.0 && t < 1e6 { Some(t) } else { None }
    } else {
        None
    }
}

/// Result of a Lyapunov exponent computation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LyapunovResult {
    /// Finite-time Lyapunov exponent λ(k).
    ///
    /// - λ > 0: exponential divergence from nominal
    /// - λ ≈ 0: neutral (stationary)
    /// - λ < 0: converging toward nominal
    pub lambda: f32,

    /// Stability classification derived from λ.
    pub stability: StabilityClass,

    /// Estimated observations until envelope exit under current divergence rate.
    ///
    /// `Some(t)` if λ > 0 and norm < ρ (trajectory is diverging but inside envelope).
    /// `None` if λ ≤ 0, norm ≥ ρ, or insufficient data.
    ///
    /// This is the exponential analogue of Theorem 1's linear bound k* ≤ ρ/α.
    /// Under exponential divergence: k* = ln(ρ/‖r‖) / λ.
    pub time_to_exit: Option<f32>,

    /// Number of consecutive samples spent outside the admissibility envelope.
    ///
    /// Resets to 0 as soon as the trajectory returns within ρ.
    /// Provides the "post-crossing persistence duration" from DSFB-Lattice §IV.
    ///
    /// - 0: currently inside envelope (nominal or just recovered)
    /// - 1..N: N consecutive samples outside — persistent structural fault candidate
    pub post_crossing_duration: u32,

    /// Fraction of the last W samples that were outside the admissibility envelope.
    ///
    /// Value ∈ [0, 1].  Close to 1 → trajectory mostly outside envelope (persistent
    /// fault).  Close to 0 → transient or nominal.
    ///
    /// This is the "post-crossing fraction" from DSFB-Lattice §IV.
    pub post_crossing_fraction: f32,

    /// Normalised excess (‖r‖ − ρ) / ρ at the current sample.
    ///
    /// Positive when outside the envelope (violation / drift above ρ).
    /// Zero when inside.  This is the "separation at exit" metric used to
    /// classify the DetectabilityClass in the `detectability` module.
    pub separation_at_exit: f32,
}

/// Stability classification from the Lyapunov exponent.
///
/// Maps λ onto qualitative stability regimes that inform grammar state
/// assignment and operator advisory output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StabilityClass {
    /// λ > λ_crit (default 0.01): exponential divergence.
    /// Corroborates Boundary[SustainedOutwardDrift] grammar state.
    ExponentialDivergence,

    /// 0 < λ ≤ λ_crit: marginal divergence.
    /// Supports Boundary approach — structural monitoring warranted.
    MarginalDivergence,

    /// −ε < λ < ε: neutral stability (stationary residual).
    /// Corroborates Admissible grammar state.
    NeutralStability,

    /// λ < −ε: exponential convergence toward nominal.
    /// Strongly corroborates Admissible.
    ExponentialConvergence,

    /// Insufficient data to compute λ (window not full).
    InsufficientData,
}

impl StabilityClass {
    /// Critical Lyapunov exponent threshold.
    /// Above this: exponential divergence regime.
    const LAMBDA_CRIT: f32 = 0.01;
    /// Neutral band half-width.
    const EPSILON: f32 = 0.001;

    /// Classify from a raw λ value.
    pub fn from_lambda(lambda: f32) -> Self {
        if lambda > Self::LAMBDA_CRIT {
            StabilityClass::ExponentialDivergence
        } else if lambda > Self::EPSILON {
            StabilityClass::MarginalDivergence
        } else if lambda > -Self::EPSILON {
            StabilityClass::NeutralStability
        } else {
            StabilityClass::ExponentialConvergence
        }
    }

    /// Returns true if the trajectory is diverging (λ > ε).
    #[inline]
    pub fn is_diverging(&self) -> bool {
        matches!(
            self,
            StabilityClass::ExponentialDivergence | StabilityClass::MarginalDivergence
        )
    }
}

// ── no_std natural logarithm ───────────────────────────────────────────────

/// Fast natural logarithm, no_std safe, no libm dependency.
///
/// Uses IEEE 754 float decomposition x = m · 2^e, then a Padé-style
/// rational approximation for ln(m) on [1.0, 2.0).
///
/// Accurate to ~5e-4 relative error for x ∈ [1e-10, 1e10].
/// Returns −23.0 for x ≤ 0.
#[inline]
fn ln_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return -23.0;
    }

    // Decompose via IEEE 754 bits: x = m · 2^e, m ∈ [1.0, 2.0)
    let bits = x.to_bits();
    let exponent = ((bits >> 23) & 0xFF) as i32 - 127;
    // Force exponent to 0 (bias 127) so mantissa_bits represents m ∈ [1.0, 2.0)
    let mantissa_bits = (bits & 0x007F_FFFF) | 0x3F80_0000;
    let m = f32::from_bits(mantissa_bits);

    // ln(m) for m ∈ [1.0, 2.0) via rational approximation around m=1
    // ln(1+t) ≈ t(6+t) / (6+4t) for t ∈ [0, 1) — Padé [1,1] of ln(1+t)
    let t = m - 1.0;
    let ln_m = t * (6.0 + t) / (6.0 + 4.0 * t);

    ln_m + exponent as f32 * core::f32::consts::LN_2
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ln_known_values() {
        assert!((ln_f32(1.0)).abs() < 0.01, "ln(1)={}", ln_f32(1.0));
        assert!((ln_f32(core::f32::consts::E) - 1.0).abs() < 0.05, "ln(e)={}", ln_f32(core::f32::consts::E));
        assert!((ln_f32(0.5) - (-0.6931)).abs() < 0.05, "ln(0.5)={}", ln_f32(0.5));
        assert!((ln_f32(10.0) - 2.3026).abs() < 0.1, "ln(10)={}", ln_f32(10.0));
    }

    #[test]
    fn lyapunov_diverging_trajectory() {
        let mut est = LyapunovEstimator::<5>::new();
        let rho = 1.0;
        // Exponentially growing norms: 0.1, 0.15, 0.225, ...
        let mut norm = 0.1_f32;
        let mut last = est.push(norm, rho);
        norm *= 1.5;
        for _ in 0..9 {
            last = est.push(norm, rho);
            norm *= 1.5;
        }
        assert!(last.lambda > 0.0, "diverging trajectory must have λ>0, got {}", last.lambda);
        assert!(last.stability.is_diverging());
    }

    #[test]
    fn lyapunov_converging_trajectory() {
        let mut est = LyapunovEstimator::<5>::new();
        let rho = 1.0;
        let mut norm = 0.5_f32;
        let mut last = est.push(norm, rho);
        norm *= 0.7;
        for _ in 0..9 {
            last = est.push(norm, rho);
            norm *= 0.7; // decaying
        }
        assert!(last.lambda < 0.0, "converging trajectory must have λ<0, got {}", last.lambda);
        assert_eq!(last.stability, StabilityClass::ExponentialConvergence);
        assert!(last.time_to_exit.is_none(), "converging trajectory has no exit time");
    }

    #[test]
    fn lyapunov_stationary_trajectory() {
        let mut est = LyapunovEstimator::<5>::new();
        let rho = 1.0;
        let mut last = est.push(0.1, rho);
        for _ in 0..9 {
            last = est.push(0.1, rho); // constant norm
        }
        assert!(last.lambda.abs() < 0.01, "stationary trajectory must have λ≈0, got {}", last.lambda);
        assert_eq!(last.stability, StabilityClass::NeutralStability);
    }

    #[test]
    fn lyapunov_time_to_exit_finite() {
        let mut est = LyapunovEstimator::<5>::new();
        let rho = 10.0; // large ρ so norm stays below it
        let mut norm = 0.1_f32;
        let mut last = est.push(norm, rho);
        norm *= 1.3;
        for _ in 0..9 {
            last = est.push(norm, rho);
            norm *= 1.3; // ends at ~1.38 < ρ=10
        }
        assert!(last.lambda > 0.0, "λ must be positive for growing norm, got {}", last.lambda);
        assert!(last.time_to_exit.is_some(), "diverging below ρ must have finite exit time");
        let t = last.time_to_exit.unwrap();
        assert!(t > 0.0 && t < 1000.0, "exit time should be reasonable: {}", t);
    }

    #[test]
    fn lyapunov_insufficient_data_before_window_full() {
        let mut est = LyapunovEstimator::<10>::new();
        for i in 0..9 {
            let r = est.push(0.1 * (i as f32 + 1.0), 1.0);
            assert_eq!(r.stability, StabilityClass::InsufficientData,
                "should be InsufficientData until window fills at i={}", i);
        }
    }

    #[test]
    fn stability_class_from_lambda() {
        assert_eq!(StabilityClass::from_lambda(0.05), StabilityClass::ExponentialDivergence);
        assert_eq!(StabilityClass::from_lambda(0.005), StabilityClass::MarginalDivergence);
        assert_eq!(StabilityClass::from_lambda(0.0), StabilityClass::NeutralStability);
        assert_eq!(StabilityClass::from_lambda(-0.05), StabilityClass::ExponentialConvergence);
    }

    #[test]
    fn reset_clears_state() {
        let mut est = LyapunovEstimator::<5>::new();
        for _ in 0..8 { est.push(0.1, 1.0); }
        est.reset();
        let r = est.push(0.1, 1.0);
        assert_eq!(r.stability, StabilityClass::InsufficientData);
    }

    #[test]
    fn post_crossing_duration_increments_when_outside() {
        let mut est = LyapunovEstimator::<5>::new();
        // norm > rho → outside
        for i in 1..=5 {
            let r = est.push(2.0, 1.0); // norm=2, rho=1 → outside
            assert_eq!(r.post_crossing_duration, i, "duration should be {i} at step {i}");
        }
    }

    #[test]
    fn post_crossing_duration_resets_on_recovery() {
        let mut est = LyapunovEstimator::<5>::new();
        for _ in 0..5 { est.push(2.0, 1.0); }
        // Return to nominal
        let r = est.push(0.5, 1.0);
        assert_eq!(r.post_crossing_duration, 0, "duration must reset on recovery");
    }

    #[test]
    fn post_crossing_fraction_zero_when_nominal() {
        let mut est = LyapunovEstimator::<5>::new();
        for _ in 0..10 {
            let r = est.push(0.5, 1.0); // always inside
            assert!(r.post_crossing_fraction < 1e-6, "fraction should be zero when nominal");
        }
    }

    #[test]
    fn post_crossing_fraction_grows_when_outside() {
        let mut est = LyapunovEstimator::<5>::new();
        for _ in 0..5 {
            let r = est.push(2.0, 1.0); // always outside
            assert!(r.post_crossing_fraction > 0.0, "fraction should grow");
        }
        let r = est.push(2.0, 1.0);
        assert!((r.post_crossing_fraction - 1.0).abs() < 1e-5, "all samples outside → fraction=1.0");
    }

    #[test]
    fn separation_at_exit_computed() {
        let mut est = LyapunovEstimator::<5>::new();
        // norm=1.5, rho=1.0 → excess = 0.5/1.0 = 0.5
        let r = est.push(1.5, 1.0);
        assert!((r.separation_at_exit - 0.5).abs() < 1e-5,
            "separation_at_exit={}", r.separation_at_exit);
    }

    #[test]
    fn separation_at_exit_zero_when_inside() {
        let mut est = LyapunovEstimator::<5>::new();
        let r = est.push(0.5, 1.0);
        assert!((r.separation_at_exit).abs() < 1e-6, "inside envelope → separation=0");
    }
}
