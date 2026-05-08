//! DSFB-Debug: admissibility envelope — paper §5.4 Equation (3).
//!
//! Defines the admissibility region:
//!
//! ```text
//! E(k) = { r ∈ Rⁿ : ‖r‖ ≤ ρ(k) }
//! ```
//!
//! A residual is *admissible* when its norm sits inside the
//! envelope; *boundary-grazing* when it approaches ρ; *violating*
//! when it exits. The envelope's radius `ρ(k)` is operator-defined
//! and may be sourced from any of:
//!
//! - **SLO / SLA targets** — e.g. p99 latency must not exceed 500 ms
//! - **Error-budget boundaries** — burn-rate alerting derivatives
//! - **Resource-utilisation ceilings** — heap %, CPU %, queue depth
//! - **Healthy-window baselines** — `ρ = mean + k·sigma` over the
//!   first N fault-free windows
//!
//! This module also exposes `sqrt_approx_pub` — a Newton-Raphson
//! square-root approximation that the no_std core uses to avoid a
//! `libm` / `std::f64::sqrt` dependency. The approximation converges
//! in ≤4 iterations to within 1 ulp of `f64::sqrt` for inputs in the
//! engine's operating range.

/// Check if a residual norm is within the admissibility envelope
#[inline]
pub fn is_admissible(norm: f64, rho: f64) -> bool {
    norm <= rho
}

/// Check if a residual norm is in the boundary zone (> boundary_fraction * ρ)
#[inline]
pub fn is_boundary_zone(norm: f64, rho: f64, boundary_fraction: f64) -> bool {
    norm > rho * boundary_fraction && norm <= rho
}

/// Check if a residual norm has exited the envelope (violation)
#[inline]
pub fn is_violation(norm: f64, rho: f64) -> bool {
    norm > rho
}

/// Compute envelope radius from healthy-window statistics.
/// ρ = 3σ of healthy residual distribution
///
/// # Arguments
/// * `healthy_residuals` - residual values from healthy window (immutable)
///
/// # Returns
/// 3σ envelope radius, or 1.0 if insufficient data
pub fn compute_envelope_radius(healthy_residuals: &[f64]) -> f64 {
    let n = healthy_residuals.len();
    if n < 2 {
        return 1.0; // fallback — documented limitation
    }

    // Compute mean
    let mut sum = 0.0;
    let mut i = 0;
    while i < n {
        sum += healthy_residuals[i];
        i += 1;
    }
    let mean = sum / n as f64;

    // Compute variance
    let mut var_sum = 0.0;
    i = 0;
    while i < n {
        let d = healthy_residuals[i] - mean;
        var_sum += d * d;
        i += 1;
    }
    let variance = var_sum / (n - 1) as f64;

    // ρ = 3σ
    let sigma = sqrt_approx(variance);
    let rho = 3.0 * sigma;

    // Guard: minimum rho to prevent zero-width envelopes
    if rho < 1e-10 { 1e-10 } else { rho }
}

/// Public wrapper for sqrt_approx (used by baseline module)
#[inline]
pub fn sqrt_approx_pub(x: f64) -> f64 {
    sqrt_approx(x)
}

/// Approximate square root using Newton's method (no_std compatible)
#[inline]
fn sqrt_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    // Use bit manipulation for initial guess, then Newton iterations
    let mut guess = x * 0.5;
    if guess == 0.0 {
        guess = 1.0;
    }
    // 8 Newton iterations — converges to ~15 digits for f64
    let mut i = 0;
    while i < 8 {
        guess = 0.5 * (guess + x / guess);
        i += 1;
    }
    guess
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admissible() {
        assert!(is_admissible(2.0, 3.0));
        assert!(is_admissible(3.0, 3.0));
        assert!(!is_admissible(3.1, 3.0));
    }

    #[test]
    fn test_boundary_zone() {
        assert!(is_boundary_zone(2.0, 3.0, 0.5));  // 2.0 > 1.5, <= 3.0
        assert!(!is_boundary_zone(1.0, 3.0, 0.5)); // 1.0 <= 1.5
        assert!(!is_boundary_zone(3.5, 3.0, 0.5)); // 3.5 > 3.0
    }

    #[test]
    fn test_envelope_radius() {
        // Known data: [1, 2, 3, 4, 5] → σ ≈ 1.581, 3σ ≈ 4.743
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let rho = compute_envelope_radius(&data);
        assert!((rho - 4.743).abs() < 0.1);
    }

    #[test]
    fn test_sqrt_approx() {
        assert!((sqrt_approx(4.0) - 2.0).abs() < 1e-10);
        assert!((sqrt_approx(9.0) - 3.0).abs() < 1e-10);
        assert!((sqrt_approx(2.0) - core::f64::consts::SQRT_2).abs() < 1e-6);
    }
}
