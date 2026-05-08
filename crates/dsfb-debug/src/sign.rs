//! DSFB-Debug: sign-tuple computation — paper §5.3 Equation (2).
//!
//! Computes the per-(window, signal) residual signature
//! σ(k) = (‖r(k)‖, ṙ(k), r̈(k)) where:
//!
//! - **‖r(k)‖** — instantaneous deviation magnitude (provided by
//!   `residual::residual_norm`)
//! - **ṙ(k)** — finite-difference drift rate over a fixed-width
//!   window W (window-to-window slope)
//! - **r̈(k)** — first difference of the drift (curvature of the
//!   trajectory; "slew")
//!
//! σ(k) is the structural signature consumed by the grammar
//! evaluator; it captures both *where* the residual is (norm) and
//! *where it is going* (drift, slew). This second-order information
//! is the structural-detectability advantage over flat thresholding,
//! which sees only the norm.
//!
//! The computation is a deterministic two-pass over the residual
//! window: pass 1 computes the regression slope (drift), pass 2
//! takes the first difference of consecutive drift values (slew).
//! No allocation; works in-place on the residual slice.

use crate::residual::residual_norm;
use crate::types::SignTuple;

/// Compute the sign tuple for a single signal at window k.
///
/// # Arguments
/// * `norms` - slice of residual norms for the last W+1 windows (immutable)
/// * `k` - current index into `norms` (must be >= 1)
///
/// # Returns
/// SignTuple with norm, drift, and slew
#[inline]
pub fn compute_sign_tuple(norms: &[f64], k: usize) -> SignTuple {
    if k == 0 || norms.is_empty() {
        return SignTuple::ZERO;
    }

    let norm = if k < norms.len() { residual_norm(norms[k]) } else { 0.0 };

    // Drift: finite difference ṙ(k) = ‖r(k)‖ - ‖r(k-1)‖
    let drift = if k >= 1 && k < norms.len() {
        norms[k] - norms[k - 1]
    } else {
        0.0
    };

    // Slew: second difference r̈(k) = ṙ(k) - ṙ(k-1)
    let slew = if k >= 2 && k < norms.len() {
        let drift_prev = norms[k - 1] - norms[k - 2];
        drift - drift_prev
    } else {
        0.0
    };

    SignTuple { norm, drift, slew }
}

/// Compute rolling drift persistence: fraction of last W windows with drift > 0
#[inline]
pub fn drift_persistence(norms: &[f64], k: usize, window: usize) -> f64 {
    if k < 1 || window == 0 {
        return 0.0;
    }
    let start = k.saturating_sub(window);
    let mut positive_count: u32 = 0;
    let mut total: u32 = 0;
    let mut i = start + 1;
    while i <= k && i < norms.len() {
        let drift = norms[i] - norms[i - 1];
        if drift > 0.0 {
            positive_count += 1;
        }
        total += 1;
        i += 1;
    }
    if total == 0 { 0.0 } else { positive_count as f64 / total as f64 }
}

/// Compute rolling boundary density: fraction of last W windows in Boundary state.
/// `states` is a slice of 0=Admissible, 1=Boundary, 2=Violation
#[inline]
pub fn boundary_density(states: &[u8], k: usize, window: usize) -> f64 {
    if window == 0 || k == 0 {
        return 0.0;
    }
    let start = k.saturating_sub(window);
    let mut boundary_count: u32 = 0;
    let mut total: u32 = 0;
    let mut i = start;
    while i <= k && i < states.len() {
        if states[i] == 1 {
            boundary_count += 1;
        }
        total += 1;
        i += 1;
    }
    if total == 0 { 0.0 } else { boundary_count as f64 / total as f64 }
}

/// Compute rolling slew density: fraction of last W windows with |slew| > δ_s
#[inline]
pub fn slew_density(norms: &[f64], k: usize, window: usize, delta_s: f64) -> f64 {
    if k < 2 || window == 0 {
        return 0.0;
    }
    let start = if k > window { k - window } else { 2 };
    let start = if start < 2 { 2 } else { start };
    let mut slew_count: u32 = 0;
    let mut total: u32 = 0;
    let mut i = start;
    while i <= k && i < norms.len() {
        let drift_cur = norms[i] - norms[i - 1];
        let drift_prev = norms[i - 1] - norms[i - 2];
        let slew = drift_cur - drift_prev;
        if slew > delta_s || slew < -delta_s {
            slew_count += 1;
        }
        total += 1;
        i += 1;
    }
    if total == 0 { 0.0 } else { slew_count as f64 / total as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_tuple_basic() {
        let norms = [0.0, 1.0, 3.0, 2.0];
        let s = compute_sign_tuple(&norms, 2);
        assert!((s.norm - 3.0).abs() < 1e-12);
        assert!((s.drift - 2.0).abs() < 1e-12); // 3.0 - 1.0
        let drift_prev = 1.0; // 1.0 - 0.0
        assert!((s.slew - (2.0 - drift_prev)).abs() < 1e-12); // 2.0 - 1.0 = 1.0
    }

    #[test]
    fn test_sign_tuple_zero_index() {
        let norms = [1.0, 2.0];
        let s = compute_sign_tuple(&norms, 0);
        assert_eq!(s, SignTuple::ZERO);
    }

    #[test]
    fn test_drift_persistence() {
        // All increasing → persistence = 1.0
        let norms = [0.0, 1.0, 2.0, 3.0, 4.0];
        assert!((drift_persistence(&norms, 4, 4) - 1.0).abs() < 1e-12);
    }
}
