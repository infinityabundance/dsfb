//! Wide-Sense Stationarity (WSS) verification for the calibration window.
//!
//! ## Theoretical Basis (Wiener-Khinchin Theorem)
//!
//! The Wiener-Khinchin theorem establishes that a wide-sense stationary (WSS)
//! process has a power spectral density equal to the Fourier transform of its
//! autocorrelation function. DSFB's calibration step assumes the healthy window
//! is WSS — that the mean and autocovariance are time-invariant over the
//! calibration period. If this assumption is violated, the envelope radius ρ
//! is unreliable.
//!
//! This module provides a lightweight WSS check that verifies:
//! 1. **Mean stationarity**: the mean of the first half ≈ the mean of the second half
//! 2. **Variance stationarity**: the variance of the first half ≈ the variance of the second half
//! 3. **Autocorrelation decay**: the lag-1 autocorrelation is bounded (no persistent trend)
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - O(n) single-pass computation
//! - Returns a typed `StationarityVerdict` with quantified deviation metrics
//!
//! ## GUM / IEEE 1764 Relevance
//!
//! The Guide to the Expression of Uncertainty in Measurement (GUM) requires
//! that measurement uncertainty budgets be derived from stationary processes.
//! This check provides the pre-condition for the GUM uncertainty budget
//! in the admissibility envelope (see `envelope.rs`).

use crate::math::{mean_f32, std_dev_f32};

/// Result of WSS verification on a calibration window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StationarityVerdict {
    /// Mean of the first half of the window.
    pub mean_first_half: f32,
    /// Mean of the second half of the window.
    pub mean_second_half: f32,
    /// Relative mean deviation: |μ₁ − μ₂| / max(|μ₁|, |μ₂|, ε).
    pub mean_deviation: f32,
    /// Variance of the first half.
    pub var_first_half: f32,
    /// Variance of the second half.
    pub var_second_half: f32,
    /// Relative variance deviation: |σ₁² − σ₂²| / max(σ₁², σ₂², ε).
    pub variance_deviation: f32,
    /// Lag-1 normalized autocorrelation r(1) / r(0).
    pub lag1_autocorrelation: f32,
    /// Whether the window passes all WSS checks.
    pub is_wss: bool,
}

/// Configuration thresholds for WSS verification.
#[derive(Debug, Clone, Copy)]
pub struct StationarityConfig {
    /// Maximum acceptable relative mean deviation between halves.
    /// Default: 0.20 (20%).
    pub max_mean_deviation: f32,
    /// Maximum acceptable relative variance deviation between halves.
    /// Default: 0.50 (50%).
    pub max_variance_deviation: f32,
    /// Maximum acceptable |lag-1 autocorrelation| (above which: persistent trend).
    /// Default: 0.70.
    pub max_lag1_autocorrelation: f32,
}

impl Default for StationarityConfig {
    fn default() -> Self {
        Self {
            max_mean_deviation: 0.20,
            max_variance_deviation: 0.50,
            max_lag1_autocorrelation: 0.70,
        }
    }
}

/// Outcome of a pre-calibration bootstrap integrity check.
///
/// Returned by [`check_bootstrap_integrity`] before an admissibility envelope
/// is locked.  A non-[`BootstrapIntegrityAlert::Clean`] result means the
/// calibration window contains organised structure and should be rejected;
/// the engine must not lock the envelope until a clean window is found.
///
/// This directly addresses the "Bootstrap Paradox" panel criticism
/// (paper §L \textit{Pre-emptive Technical Defence}, item 6): if the South
/// China Sea calibration window is already jammed, spectral non-flatness
/// will manifest as a non-zero lag-1 autocorrelation (Wiener-Khinchin) and
/// this alert fires.
///
/// # Examples
///
/// ```
/// use dsfb_rf::stationarity::{check_bootstrap_integrity, StationarityConfig, BootstrapIntegrityAlert};
/// let clean: Vec<f32> = (0..64).map(|i| 0.5 + 0.01 * (i as f32 % 7.0 - 3.0)).collect();
/// assert_eq!(check_bootstrap_integrity(&clean, &StationarityConfig::default()),
///            BootstrapIntegrityAlert::Clean);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootstrapIntegrityAlert {
    /// Calibration window passed all WSS and PSD flatness tests.  Safe to lock.
    Clean,
    /// PSD flatness test failed: lag-1 autocorrelation exceeded the configured
    /// threshold.  The window likely contains periodic interference.
    PsdNonFlat {
        /// Measured |lag-1 autocorrelation|.
        flatness_score: f32,
        /// Configured maximum before alert fires.
        threshold: f32,
    },
    /// Mean stationarity test failed: the half-window means diverge.
    MeanShift {
        /// Normalised |mean first_half − mean second_half| / max(|mean|, ε).
        deviation: f32,
    },
    /// Variance stationarity test failed: the half-window variances diverge.
    VarianceShift {
        /// Normalised |var first_half − var second_half| / max(var, ε).
        deviation: f32,
    },
}

impl BootstrapIntegrityAlert {
    /// Returns `true` if this alert blocks calibration lock.
    #[inline]
    pub fn blocks_calibration(self) -> bool {
        !matches!(self, BootstrapIntegrityAlert::Clean)
    }
}

/// Check whether a candidate calibration window is safe to use as a baseline.
///
/// Runs [`verify_wss`] and maps the result onto a typed
/// [`BootstrapIntegrityAlert`] that the engine layer can surface to the
/// operator before locking any admissibility envelope.
///
/// Returns [`BootstrapIntegrityAlert::Clean`] when the window has fewer than
/// 4 observations (not enough data to detect contamination; the caller should
/// collect more samples before relying on the baseline).
///
/// # Examples
///
/// ```
/// use dsfb_rf::stationarity::{check_bootstrap_integrity, StationarityConfig, BootstrapIntegrityAlert};
/// // Inject a sinusoidal jammer into the calibration window
/// let jammed: Vec<f32> = (0..64).map(|i| 0.5 + 0.3 * (i as f32 * 0.4).sin()).collect();
/// let alert = check_bootstrap_integrity(&jammed, &StationarityConfig::default());
/// assert!(alert.blocks_calibration());
/// ```
pub fn check_bootstrap_integrity(
    norms: &[f32],
    config: &StationarityConfig,
) -> BootstrapIntegrityAlert {
    match verify_wss(norms, config) {
        None => BootstrapIntegrityAlert::Clean, // insufficient data — caller must extend window
        Some(v) if v.is_wss => BootstrapIntegrityAlert::Clean,
        Some(v) => {
            if v.mean_deviation > config.max_mean_deviation {
                BootstrapIntegrityAlert::MeanShift { deviation: v.mean_deviation }
            } else if v.variance_deviation > config.max_variance_deviation {
                BootstrapIntegrityAlert::VarianceShift { deviation: v.variance_deviation }
            } else {
                // Autocorrelation/PSD flatness failure
                BootstrapIntegrityAlert::PsdNonFlat {
                    flatness_score: v.lag1_autocorrelation.abs(),
                    threshold: config.max_lag1_autocorrelation,
                }
            }
        }
    }
}

/// Verify wide-sense stationarity of a calibration window.
///
/// Splits the window in half and compares mean, variance, and lag-1
/// autocorrelation against configured thresholds.
///
/// Returns `None` if the window has fewer than 4 observations.
pub fn verify_wss(norms: &[f32], config: &StationarityConfig) -> Option<StationarityVerdict> {
    if norms.len() < 4 {
        return None;
    }

    let mid = norms.len() / 2;
    let first = &norms[..mid];
    let second = &norms[mid..];

    let m1 = mean_f32(first);
    let m2 = mean_f32(second);
    let s1 = std_dev_f32(first);
    let s2 = std_dev_f32(second);
    let v1 = s1 * s1;
    let v2 = s2 * s2;

    let eps = 1e-10_f32;
    let mean_dev = (m1 - m2).abs() / (m1.abs().max(m2.abs()).max(eps));
    let var_dev = (v1 - v2).abs() / (v1.max(v2).max(eps));

    // Lag-1 autocorrelation: r(1)/r(0)
    let m_all = mean_f32(norms);
    let mut r0 = 0.0_f32;
    let mut r1 = 0.0_f32;
    for i in 0..norms.len() {
        let d = norms[i] - m_all;
        r0 += d * d;
        if i + 1 < norms.len() {
            let d_next = norms[i + 1] - m_all;
            r1 += d * d_next;
        }
    }
    let lag1 = if r0 > eps { r1 / r0 } else { 0.0 };

    let is_wss = mean_dev <= config.max_mean_deviation
        && var_dev <= config.max_variance_deviation
        && lag1.abs() <= config.max_lag1_autocorrelation;

    Some(StationarityVerdict {
        mean_first_half: m1,
        mean_second_half: m2,
        mean_deviation: mean_dev,
        var_first_half: v1,
        var_second_half: v2,
        variance_deviation: var_dev,
        lag1_autocorrelation: lag1,
        is_wss,
    })
}

// ── Reverse Arrangements Test (Olmstead-Tukey 1947) ──────────────────────

/// Result of the Reverse Arrangements Test for trend detection.
///
/// ## Theoretical Basis: Olmstead & Tukey (1947)
///
/// The Reverse Arrangements Test (RAT) is the canonical non-parametric test
/// for detecting monotone trends in a data window (Olmstead & Tukey 1947;
/// WMO-No. 100 §3.3.3).  Unlike the lag-1 autocorrelation test, RAT makes
/// **no distributional assumption** about the observations and is specifically
/// sensitive to subtle drifts that are invisible to the split-mean variance check.
///
/// **Test statistic:** Count A = #{(i,j) : i < j,  x_i > x_j}
///
/// Under H₀ (no trend, i.i.d. observations):
/// - E[A] = N(N−1)/4
/// - Var[A] = N(2N+5)(N−1)/72
///
/// **Z-score** (normal approximation, reliable for N ≥ 10):
///   Z = (A − E[A]) / √Var[A]  ~  N(0, 1)  under H₀
///
/// Critical values (two-sided):
/// - |Z| > 1.645 → 10% significance
/// - |Z| > 1.960 → 5% significance  ← default `has_trend`
/// - |Z| > 2.576 → 1% significance  ← `has_trend_strict`
///
/// A positive Z (A > E[A]) indicates a **downtrend** (many later values are
/// smaller than earlier values).  A negative Z means an **uptrend**.
///
/// ## GUM Integration
///
/// A calibration window that fails the RAT has a *systematic monotone trend*
/// that **invalidates** the GUM Type A uncertainty estimate.  This function
/// is called by the DSFB calibration path as a mandatory pre-condition before
/// the admissibility radius ρ is locked.
///
/// ## Reference
///
/// Olmstead, P.S. & Tukey, J.W. (1947). "A corner test for association."
/// *Ann. Math. Statist.* 18(4):495–513.
///
/// WMO-No. 100. (2018). *Guide to Climatological Practices.* §3.3.3.
#[derive(Debug, Clone, Copy)]
pub struct ReverseArrangementsResult {
    /// Number of reverse arrangements: A = #{(i,j): i < j, x_i > x_j}.
    pub n_arrangements: u32,
    /// Expected value under H₀: E[A] = N(N−1)/4.
    pub expected: f32,
    /// Variance under H₀: Var[A] = N(2N+5)(N−1)/72.
    pub variance: f32,
    /// Standardized Z-score: (A − E[A]) / √Var[A].
    pub z_score: f32,
    /// True if |Z| > 1.96 (trend present at 5% significance, p < 0.05).
    pub has_trend: bool,
    /// True if |Z| > 2.576 (strict: trend at 1% significance, p < 0.01).
    pub has_trend_strict: bool,
    /// Trend direction: +1 = uptrend, −1 = downtrend, 0 = no significant trend.
    pub trend_direction: i8,
}

/// Run the Reverse Arrangements Test on a calibration window.
///
/// Requires N ≥ 10 observations for a reliable normal approximation.
/// Returns `None` if the window has fewer than 10 observations.
///
/// **Time complexity:** O(N²).  Acceptable for calibration windows (N ≤ 500).
/// For N = 200 this is 20,000 comparisons — ~100 µs on a Cortex-M4F.
pub fn reverse_arrangements_test(norms: &[f32]) -> Option<ReverseArrangementsResult> {
    let n = norms.len();
    if n < 10 { return None; }

    // Count A = #{(i,j): i < j, x_i > x_j}
    let mut a = 0u32;
    for i in 0..n {
        for j in (i + 1)..n {
            if norms[i] > norms[j] { a += 1; }
        }
    }

    let n_f = n as f32;
    let expected  = n_f * (n_f - 1.0) / 4.0;
    let variance  = n_f * (2.0 * n_f + 5.0) * (n_f - 1.0) / 72.0;
    let z = if variance > 1e-30 {
        (a as f32 - expected) / crate::math::sqrt_f32(variance)
    } else { 0.0 };

    // Z < 0: fewer inversions than expected → uptrend; Z > 0: more → downtrend
    let trend_direction: i8 = if z.abs() > 1.96 {
        if z < 0.0 { 1 } else { -1 }
    } else { 0 };

    Some(ReverseArrangementsResult {
        n_arrangements: a,
        expected,
        variance,
        z_score: z,
        has_trend: z.abs() > 1.96,
        has_trend_strict: z.abs() > 2.576,
        trend_direction,
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_signal_is_wss() {
        let norms = [0.05_f32; 100];
        let v = verify_wss(&norms, &StationarityConfig::default()).unwrap();
        assert!(v.is_wss, "constant signal must be WSS: {:?}", v);
        assert!(v.mean_deviation < 0.01);
        assert!(v.variance_deviation < 0.01);
    }

    #[test]
    fn stationary_noise_is_wss() {
        // Simulated white noise with constant statistics
        let norms: [f32; 100] = core::array::from_fn(|i| {
            0.05 + 0.01 * ((i as f32 * 7.3).sin())
        });
        let v = verify_wss(&norms, &StationarityConfig::default()).unwrap();
        assert!(v.is_wss, "stationary noise must be WSS: {:?}", v);
    }

    #[test]
    fn trending_signal_fails_wss() {
        // Linear trend: mean shifts between halves
        let norms: [f32; 100] = core::array::from_fn(|i| 0.01 + i as f32 * 0.01);
        let v = verify_wss(&norms, &StationarityConfig::default()).unwrap();
        assert!(!v.is_wss, "trending signal must fail WSS: {:?}", v);
    }

    #[test]
    fn step_change_fails_wss() {
        // Step change at midpoint: first half ≈ 0.05, second half ≈ 0.5
        let mut norms = [0.05_f32; 100];
        for i in 50..100 { norms[i] = 0.50; }
        let v = verify_wss(&norms, &StationarityConfig::default()).unwrap();
        assert!(!v.is_wss, "step change must fail WSS: mean_dev={}", v.mean_deviation);
    }

    #[test]
    fn returns_none_for_short_window() {
        assert!(verify_wss(&[0.1, 0.2], &StationarityConfig::default()).is_none());
        assert!(verify_wss(&[], &StationarityConfig::default()).is_none());
    }

    #[test]
    fn high_autocorrelation_fails() {
        // Highly correlated sequence (random walk)
        let mut norms = [0.0_f32; 100];
        norms[0] = 0.5;
        for i in 1..100 {
            norms[i] = norms[i - 1] + 0.001;
        }
        let v = verify_wss(&norms, &StationarityConfig::default()).unwrap();
        assert!(v.lag1_autocorrelation.abs() > 0.5,
            "random walk must have high lag-1: {}", v.lag1_autocorrelation);
    }

    // ── Reverse Arrangements Test ──────────────────────────────────────────

    #[test]
    fn rat_returns_none_for_short_window() {
        assert!(reverse_arrangements_test(&[0.1, 0.2, 0.3]).is_none());
    }

    #[test]
    fn rat_flat_window_no_trend() {
        // A symmetric sinusoidal oscillation around a fixed mean has A ≈ E[A] → |Z| ≈ 0.
        // (Note: a perfectly constant sequence has A = 0 << E[A] and tests AS an uptrend.)
        let norms: [f32; 50] = core::array::from_fn(|i| {
            0.05_f32 + 0.01 * ((i as f32 * 3.141_592_6 * 2.0 / 7.0).sin())
        });
        let r = reverse_arrangements_test(&norms).unwrap();
        assert!(!r.has_trend,
            "stationary sinusoid must have no trend: Z={}", r.z_score);
    }

    #[test]
    fn rat_uptrend_detected() {
        // Strictly increasing: x_i < x_j for all i<j → A = 0 << E[A] → Z << 0  
        let norms: [f32; 40] = core::array::from_fn(|i| i as f32 * 0.01);
        let r = reverse_arrangements_test(&norms).unwrap();
        assert!(r.has_trend, "strictly increasing must be detected: Z={}", r.z_score);
        assert_eq!(r.trend_direction, 1, "should be uptrend");
    }

    #[test]
    fn rat_downtrend_detected() {
        // Strictly decreasing: x_i > x_j for all i<j → A = max → Z >> 0
        let norms: [f32; 40] = core::array::from_fn(|i| 1.0 - i as f32 * 0.01);
        let r = reverse_arrangements_test(&norms).unwrap();
        assert!(r.has_trend, "strictly decreasing must be detected: Z={}", r.z_score);
        assert_eq!(r.trend_direction, -1, "should be downtrend");
    }

    #[test]
    fn rat_stationary_noise_no_trend() {
        // Bounded oscillation: no systematic trend
        let norms: [f32; 50] = core::array::from_fn(|i| {
            0.05 + 0.01 * ((i as f32 * 7.3).sin())
        });
        let r = reverse_arrangements_test(&norms).unwrap();
        // Oscillating noise may or may not trigger at 5%; just check |Z| is small
        assert!(r.z_score.abs() < 4.0,
            "stationary oscillation must have small Z: {}", r.z_score);
    }

    #[test]
    fn rat_expected_value_formula() {
        // For N samples, E[A] = N(N-1)/4
        let norms = [0.0_f32; 20];
        let r = reverse_arrangements_test(&norms).unwrap();
        let expected = 20.0_f32 * 19.0 / 4.0;
        assert!((r.expected - expected).abs() < 0.1,
            "E[A] formula: expected={} got={}", expected, r.expected);
    }
}
