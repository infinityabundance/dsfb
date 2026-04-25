//! Healthy-window calibration.
//!
//! The admissibility envelope is calibrated from a slice of
//! *known-good* residual norms (the healthy window). Stage III of the
//! companion paper fixes the protocol: `ρ = μ_healthy + 3 × σ_healthy`.
//! This module provides a thin, deterministic wrapper around
//! [`AdmissibilityEnvelope::calibrate_from_window`] plus a validity
//! check for the window itself (non-empty, mostly finite, and not
//! itself already violating).

use crate::envelope::AdmissibilityEnvelope;
use crate::math;

/// Result of a calibration attempt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibrationOutcome {
    /// Calibration succeeded; the envelope is ready for use.
    Ok(AdmissibilityEnvelope),
    /// Window contains no finite samples at all.
    EmptyOrAllNonFinite,
    /// Window has a non-finite mean or variance (numerical pathology).
    NonFiniteStatistics,
    /// Window's spread is suspiciously large — calibrating from this
    /// slice would produce an uninformative envelope.
    ///
    /// Tunable via [`calibrate_with_gate`]; [`calibrate`] uses a
    /// conservative default (`cv > 10.0` ⇒ reject) so a caller cannot
    /// accidentally calibrate from a fault-contaminated window.
    WindowTooNoisy {
        /// Sample mean of the healthy window.
        mean: f64,
        /// Sample standard deviation of the healthy window.
        std_dev: f64,
    },
}

/// Calibrate with the default gate (coefficient-of-variation ≤ 10).
///
/// Most deployments should use this. Research scenarios that need
/// looser gating can use [`calibrate_with_gate`].
#[must_use]
pub fn calibrate(healthy_norms: &[f64]) -> CalibrationOutcome {
    calibrate_with_gate(healthy_norms, 10.0)
}

/// Calibrate with a custom coefficient-of-variation gate.
///
/// `max_cv` is the largest permitted `σ / |μ|` (or, if `μ ≈ 0`, the
/// largest permitted `σ`). Calibration windows noisier than this are
/// rejected without producing an envelope, so the caller must
/// re-sample a cleaner window before proceeding. Passing `f64::INFINITY`
/// disables the gate entirely (not recommended for production).
#[must_use]
pub fn calibrate_with_gate(healthy_norms: &[f64], max_cv: f64) -> CalibrationOutcome {
    debug_assert!(max_cv >= 0.0, "max_cv must be non-negative");

    let Some(mean) = math::finite_mean(healthy_norms) else {
        return CalibrationOutcome::EmptyOrAllNonFinite;
    };
    let Some(var) = math::finite_variance(healthy_norms) else {
        return CalibrationOutcome::EmptyOrAllNonFinite;
    };
    let Some(std_dev) = math::sqrt_f64(var) else {
        return CalibrationOutcome::NonFiniteStatistics;
    };
    if !mean.is_finite() || !std_dev.is_finite() {
        return CalibrationOutcome::NonFiniteStatistics;
    }

    // Coefficient-of-variation gate: σ / |μ| if μ not ≈ 0, else σ
    // alone (treating small-mean windows as an absolute-spread check).
    let abs_mean = math::abs_f64(mean);
    let cv = if abs_mean > 1e-9 { std_dev / abs_mean } else { std_dev };
    if cv > max_cv {
        return CalibrationOutcome::WindowTooNoisy { mean, std_dev };
    }

    let rho = mean + 3.0 * std_dev;
    debug_assert!(rho.is_finite(), "calibrated rho must be finite");
    CalibrationOutcome::Ok(AdmissibilityEnvelope::new(rho))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_window_rejected() {
        assert!(matches!(calibrate(&[]), CalibrationOutcome::EmptyOrAllNonFinite));
    }

    #[test]
    fn all_nan_rejected() {
        let xs = [f64::NAN; 10];
        assert!(matches!(calibrate(&xs), CalibrationOutcome::EmptyOrAllNonFinite));
    }

    #[test]
    fn clean_window_produces_envelope() {
        let xs = [0.05_f64; 100];
        match calibrate(&xs) {
            CalibrationOutcome::Ok(env) => assert!((env.rho - 0.05).abs() < 1e-12),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn very_noisy_window_rejected_by_default_gate() {
        // mean ≈ 0.01, std dev ≈ 0.5 → cv ≈ 50 (above the default 10.0).
        let xs = [0.01, -0.5, 0.5, -0.5, 0.5, -0.5, 0.5];
        match calibrate(&xs) {
            CalibrationOutcome::WindowTooNoisy { .. } => {}
            other => panic!("expected WindowTooNoisy, got {other:?}"),
        }
    }

    #[test]
    fn noisy_window_accepted_with_relaxed_gate() {
        let xs = [0.01, -0.5, 0.5, -0.5, 0.5, -0.5, 0.5];
        match calibrate_with_gate(&xs, f64::INFINITY) {
            CalibrationOutcome::Ok(env) => assert!(env.rho.is_finite()),
            other => panic!("expected Ok with relaxed gate, got {other:?}"),
        }
    }
}
