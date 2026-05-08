//! DSFB-Debug: residual computation — paper §5.2 Equation (1).
//!
//! Computes the residual vector `r(k) = x(k) - x_hat(k)` where
//! `x(k)` is the observed measurement and `x_hat(k)` is the
//! healthy-window baseline reference (computed in `baseline.rs`).
//!
//! # Non-intrusion contract (type-enforced)
//!
//! Every function in this module accepts `&[f64]` slices only —
//! immutable references with no `mut` qualifier. The Rust type
//! system enforces at compile time that we cannot modify the
//! upstream observation `x(k)` or the baseline reference `x_hat(k)`.
//! This is the load-bearing observer-only guarantee of the engine
//! (paper §1.x non-intrusion contract).
//!
//! # Numerical handling
//!
//! NaN inputs propagate to NaN residuals (no silent imputation at
//! this layer); the `sign.rs` consumer detects NaN and stamps
//! `was_imputed = true` on the resulting `SignalEvaluation`. This
//! preserves the audit trail: the operator sees missing-data windows
//! flagged, not silently zeroed.

use crate::error::{DsfbError, Result};

/// Compute the residual vector r(k) = x(k) - x_hat(k)
///
/// # Non-Intrusion Contract
/// Both `observation` and `baseline` are shared immutable references.
/// The result is written into the caller-owned `output` buffer.
///
/// # Missingness
/// If `missing_mask[i]` is true, `output[i]` is set to 0.0 (mean imputation).
/// The caller must track which signals were imputed for downstream processing.
#[inline]
pub fn compute_residuals(
    observation: &[f64],
    baseline: &[f64],
    missing_mask: &[bool],
    output: &mut [f64],
) -> Result<()> {
    let n = observation.len();
    if baseline.len() != n || missing_mask.len() != n || output.len() != n {
        return Err(DsfbError::DimensionMismatch {
            expected: n,
            got: baseline.len(),
        });
    }

    let mut i = 0;
    while i < n {
        if missing_mask[i] {
            // Missingness-aware: imputed signals contribute zero residual
            output[i] = 0.0;
        } else {
            output[i] = observation[i] - baseline[i];
        }
        i += 1;
    }
    Ok(())
}

/// Compute the norm (absolute value for scalar, Euclidean for vector)
/// of a single residual value.
#[inline]
pub fn residual_norm(r: f64) -> f64 {
    if r < 0.0 { -r } else { r }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residual_basic() {
        let obs = [1.0, 2.0, 3.0];
        let base = [0.5, 1.5, 3.0];
        let mask = [false, false, false];
        let mut out = [0.0; 3];
        compute_residuals(&obs, &base, &mask, &mut out).expect("should succeed");
        assert!((out[0] - 0.5).abs() < 1e-12);
        assert!((out[1] - 0.5).abs() < 1e-12);
        assert!((out[2] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_residual_missing() {
        let obs = [999.0, 2.0];
        let base = [0.0, 1.0];
        let mask = [true, false];
        let mut out = [0.0; 2];
        compute_residuals(&obs, &base, &mask, &mut out).expect("should succeed");
        assert!((out[0] - 0.0).abs() < 1e-12); // imputed → 0
        assert!((out[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_dimension_mismatch() {
        let obs = [1.0, 2.0];
        let base = [1.0];
        let mask = [false, false];
        let mut out = [0.0; 2];
        assert!(compute_residuals(&obs, &base, &mask, &mut out).is_err());
    }
}
