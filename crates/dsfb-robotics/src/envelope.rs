//! Admissibility envelope `E(k) = { r ∈ ℝⁿ : ‖r‖ ≤ ρ(k) }`.
//!
//! The envelope radius ρ is calibrated from the healthy-window
//! statistics (mean + 3 × σ of the residual norm during a known-good
//! operating window), scaled at runtime by the
//! [`RobotContext::admissibility_multiplier`](crate::platform::RobotContext)
//! to suppress violations during commissioning, calibration, and
//! maintenance.
//!
//! A second threshold `boundary_frac × ρ` (default 0.5) defines the
//! *boundary approach* band used by the grammar FSM for the
//! `Boundary[...]` states.

use crate::math;

/// Admissibility envelope parameterised by a single radius scalar.
///
/// The envelope is constructed from the healthy-window calibration
/// slice and thereafter stays fixed. A different operating regime
/// should produce a different envelope; DSFB does **not** adapt the
/// envelope online, to keep the observer deterministic and auditable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdmissibilityEnvelope {
    /// Base radius ρ = μ_healthy + 3 × σ_healthy.
    pub rho: f64,
    /// Boundary band fraction: `Boundary` when `‖r‖ > boundary_frac × ρ_eff`.
    ///
    /// Paper default `0.5` — the outer half of the envelope is the
    /// "approach" band.
    pub boundary_frac: f64,
    /// Slew threshold δ_s for the `AbruptSlewViolation` reason code.
    pub delta_s: f64,
}

impl AdmissibilityEnvelope {
    /// Construct an envelope from an explicit radius, using the paper
    /// defaults for `boundary_frac = 0.5` and `delta_s = 0.05`.
    #[inline]
    #[must_use]
    pub const fn new(rho: f64) -> Self {
        Self { rho, boundary_frac: 0.5, delta_s: 0.05 }
    }

    /// Construct an envelope with full parameter control.
    #[inline]
    #[must_use]
    pub const fn with_params(rho: f64, boundary_frac: f64, delta_s: f64) -> Self {
        Self { rho, boundary_frac, delta_s }
    }

    /// Effective radius after applying the robot-context multiplier.
    ///
    /// When the context is `ArmCommissioning` or `Maintenance`, the
    /// multiplier is `+∞` and the effective radius is `+∞` — so no
    /// residual magnitude can be a violation.
    #[inline]
    #[must_use]
    pub fn effective_rho(&self, platform_multiplier: f64) -> f64 {
        debug_assert!(self.rho >= 0.0, "envelope radius must be non-negative");
        debug_assert!(platform_multiplier >= 0.0, "multiplier must be non-negative");
        self.rho * platform_multiplier
    }

    /// Returns `true` if `‖r‖ > ρ_eff` (envelope violation).
    #[inline]
    #[must_use]
    pub fn is_violation(&self, norm: f64, platform_multiplier: f64) -> bool {
        let rho_eff = self.effective_rho(platform_multiplier);
        norm > rho_eff
    }

    /// Returns `true` if `‖r‖` is in the boundary-approach band but not
    /// yet violating.
    #[inline]
    #[must_use]
    pub fn is_boundary_approach(&self, norm: f64, platform_multiplier: f64) -> bool {
        debug_assert!((0.0..=1.0).contains(&self.boundary_frac), "boundary_frac out of [0,1]");
        let rho_eff = self.effective_rho(platform_multiplier);
        norm > self.boundary_frac * rho_eff
    }

    /// Calibrate an envelope from a healthy-window residual norm slice.
    ///
    /// Computes `ρ = μ + 3σ` over the provided norms. Returns `None` if
    /// the slice contains no finite samples. Non-finite samples are
    /// skipped (missingness-aware). This is the Stage III calibration
    /// protocol referenced in the companion paper §F.
    #[must_use]
    pub fn calibrate_from_window(healthy_norms: &[f64]) -> Option<Self> {
        let mean = math::finite_mean(healthy_norms)?;
        let variance = math::finite_variance(healthy_norms)?;
        let std_dev = math::sqrt_f64(variance)?;
        let rho = mean + 3.0 * std_dev;
        debug_assert!(rho.is_finite(), "calibrated rho must be finite");
        debug_assert!(rho >= 0.0, "calibrated rho must be non-negative");
        Some(Self::new(rho))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_constant_window_gives_mean_rho() {
        let norms = [0.05_f64; 100];
        let env = AdmissibilityEnvelope::calibrate_from_window(&norms).expect("non-empty");
        // Constant → std=0 → rho = mean = 0.05.
        assert!((env.rho - 0.05).abs() < 1e-12);
    }

    #[test]
    fn calibration_uses_mean_plus_three_sigma() {
        // Zero-mean, variance = 0.01, so σ = 0.1 and ρ = 0 + 3·0.1 = 0.3.
        let norms: [f64; 6] = [-0.1, -0.1, 0.0, 0.0, 0.1, 0.1];
        let env = AdmissibilityEnvelope::calibrate_from_window(&norms).expect("non-empty");
        assert!(env.rho > 0.0);
        assert!(env.rho <= 0.35);
    }

    #[test]
    fn violation_boundary_are_strict_inequalities() {
        let env = AdmissibilityEnvelope::new(0.1);
        assert!(!env.is_violation(0.05, 1.0));
        assert!(!env.is_violation(0.1, 1.0), "norm == rho must not count as violation");
        assert!(env.is_violation(0.101, 1.0));
    }

    #[test]
    fn commissioning_suppresses_all_violations() {
        let env = AdmissibilityEnvelope::new(0.1);
        // `ArmCommissioning` supplies f64::INFINITY — test the math directly.
        assert!(!env.is_violation(1e9, f64::INFINITY));
        assert!(!env.is_boundary_approach(1e9, f64::INFINITY));
    }

    #[test]
    fn boundary_band_is_outer_half_by_default() {
        let env = AdmissibilityEnvelope::new(0.1);
        assert!(!env.is_boundary_approach(0.04, 1.0));
        assert!(env.is_boundary_approach(0.06, 1.0));
        assert!(env.is_boundary_approach(0.099, 1.0));
    }

    #[test]
    fn empty_calibration_returns_none() {
        assert!(AdmissibilityEnvelope::calibrate_from_window(&[]).is_none());
        assert!(AdmissibilityEnvelope::calibrate_from_window(&[f64::NAN, f64::NAN]).is_none());
    }
}
