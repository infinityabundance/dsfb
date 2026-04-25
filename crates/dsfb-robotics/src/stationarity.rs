//! Wide-sense-stationarity (WSS) check for calibration windows.
//!
//! Before an admissibility envelope is committed, the healthy-window
//! slice should be approximately wide-sense stationary. If it is not
//! — mean or variance drifts visibly across the window — then the
//! `ρ = μ + 3σ` rule is extracting an envelope from an already-drifting
//! signal, and the envelope will be biased.
//!
//! The check here is deliberately simple: split the window in halves,
//! compute each half's mean and variance, and report whether the two
//! halves are "close enough" under caller-supplied tolerances. Phase 2
//! provides the binary-decision form; Phase 6 Kani harnesses prove
//! monotonicity of the tolerance gate.

use crate::math;

/// Outcome of a WSS check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WssCheck {
    /// Window passed the stationarity check.
    Stationary,
    /// Mean drift between halves exceeds `mean_tol`.
    MeanDrift {
        /// Sample mean of the first half of the window.
        first_mean: f64,
        /// Sample mean of the second half of the window.
        second_mean: f64,
    },
    /// Variance drift between halves exceeds `var_tol`.
    VarianceDrift {
        /// Sample variance of the first half of the window.
        first_var: f64,
        /// Sample variance of the second half of the window.
        second_var: f64,
    },
    /// Window is too short to split meaningfully (`len < 2`).
    TooShort,
    /// Window contains no finite samples in one or both halves.
    NonFinite,
}

/// Two-half WSS check with caller-supplied tolerances.
///
/// The default Phase 2 rule: the half-mean difference must satisfy
/// `|μ₁ − μ₂| ≤ mean_tol` *and* the half-variance ratio must satisfy
/// `max(v₁, v₂) / max(min(v₁, v₂), ε) ≤ var_tol`, where `ε` is a tiny
/// floor to avoid divide-by-zero on constant-variance windows.
#[must_use]
pub fn check(xs: &[f64], mean_tol: f64, var_tol: f64) -> WssCheck {
    debug_assert!(mean_tol >= 0.0, "mean_tol must be non-negative");
    debug_assert!(var_tol >= 1.0, "var_tol is a ratio ≥ 1.0");
    if xs.len() < 2 {
        return WssCheck::TooShort;
    }
    let mid = xs.len() / 2;
    let (first, second) = xs.split_at(mid);

    let Some(m1) = math::finite_mean(first) else { return WssCheck::NonFinite; };
    let Some(m2) = math::finite_mean(second) else { return WssCheck::NonFinite; };
    let Some(v1) = math::finite_variance(first) else { return WssCheck::NonFinite; };
    let Some(v2) = math::finite_variance(second) else { return WssCheck::NonFinite; };

    if math::abs_f64(m1 - m2) > mean_tol {
        return WssCheck::MeanDrift { first_mean: m1, second_mean: m2 };
    }

    let eps = 1e-15_f64;
    let vmax = v1.max(v2);
    let vmin = v1.min(v2).max(eps);
    if vmax / vmin > var_tol {
        return WssCheck::VarianceDrift { first_var: v1, second_var: v2 };
    }

    WssCheck::Stationary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_window_is_stationary() {
        let xs = [1.0_f64; 16];
        assert_eq!(check(&xs, 1e-3, 2.0), WssCheck::Stationary);
    }

    #[test]
    fn detects_mean_drift() {
        let mut xs = [0.0_f64; 16];
        for (i, v) in xs.iter_mut().enumerate() {
            *v = if i < 8 { 0.0 } else { 1.0 };
        }
        let r = check(&xs, 0.01, 2.0);
        assert!(matches!(r, WssCheck::MeanDrift { .. }), "got {r:?}");
    }

    #[test]
    fn detects_variance_drift() {
        let mut xs = [0.0_f64; 16];
        for (i, v) in xs.iter_mut().enumerate() {
            *v = if i < 8 { 0.001 * (i as f64) } else { 1.0 * (i as f64 - 7.0) };
        }
        let r = check(&xs, 100.0, 2.0);
        assert!(matches!(r, WssCheck::VarianceDrift { .. }), "got {r:?}");
    }

    #[test]
    fn too_short_window_reports_too_short() {
        assert_eq!(check(&[], 0.1, 2.0), WssCheck::TooShort);
        assert_eq!(check(&[1.0], 0.1, 2.0), WssCheck::TooShort);
    }

    #[test]
    fn all_nan_window_is_non_finite() {
        let xs = [f64::NAN; 8];
        assert_eq!(check(&xs, 0.1, 2.0), WssCheck::NonFinite);
    }
}
