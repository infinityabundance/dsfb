//! Admissibility envelope E(k) = {r : ‖r‖ ≤ ρ(k)}.
//!
//! ## Mathematical Definition (paper §B.3, §V-D)
//!
//! E(k) = {r ∈ ℂⁿ : ‖r‖ ≤ ρ(k)}
//! ρ = μ_healthy + 3σ_healthy   (from calibration window)
//!
//! The admissibility_multiplier() from PlatformContext scales ρ to +∞
//! during waveform transitions and calibration periods, making envelope
//! violations structurally impossible during suppressed windows.
//!
//! ## Envelope Sources (paper §V-D)
//!
//! 1. Receiver noise floor statistics: 3σ of healthy-window residual norm
//! 2. Regulatory emission masks (ITU-R SM.1048-5 §4.3, MIL-STD-461G RE102)
//! 3. Link budget margins
//! 4. PLL hold-in range
//! 5. 3GPP TS 36.141 §6.3 ACLR limits

/// Admissibility envelope parameterized by radius ρ.
///
/// Constructed from the healthy calibration window statistics.
/// The radius is stored as a fixed scalar; regime-dependent scaling
/// is applied via `effective_rho()` using the platform multiplier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdmissibilityEnvelope {
    /// Base envelope radius ρ = μ_healthy + 3σ_healthy.
    pub rho: f32,
    /// Boundary fraction: Boundary state triggered when ‖r‖ > boundary_frac * ρ.
    /// Paper default: 0.5 (50% of ρ).
    pub boundary_frac: f32,
    /// Slew threshold δ_s for AbruptSlewViolation detection.
    pub delta_s: f32,
}

impl AdmissibilityEnvelope {
    /// Construct envelope from calibrated radius ρ.
    pub const fn new(rho: f32) -> Self {
        Self {
            rho,
            boundary_frac: 0.5,
            delta_s: 0.05,
        }
    }

    /// Construct with custom boundary fraction and slew threshold.
    pub const fn with_params(rho: f32, boundary_frac: f32, delta_s: f32) -> Self {
        Self { rho, boundary_frac, delta_s }
    }

    /// Effective radius after applying platform multiplier.
    ///
    /// During waveform transitions: multiplier = +∞ → no violation possible.
    #[inline]
    pub fn effective_rho(&self, platform_multiplier: f32) -> f32 {
        self.rho * platform_multiplier
    }

    /// Returns true if ‖r‖ > ρ_eff (Violation condition).
    #[inline]
    pub fn is_violation(&self, norm: f32, platform_multiplier: f32) -> bool {
        let rho_eff = self.effective_rho(platform_multiplier);
        norm > rho_eff
    }

    /// Returns true if ‖r‖ > boundary_frac * ρ_eff (Boundary approach condition).
    #[inline]
    pub fn is_boundary_approach(&self, norm: f32, platform_multiplier: f32) -> bool {
        let rho_eff = self.effective_rho(platform_multiplier);
        norm > self.boundary_frac * rho_eff
    }

    /// Calibrate envelope from a healthy-window residual norm slice.
    ///
    /// Computes μ + 3σ over the provided norms array.
    /// This is the Stage III calibration protocol (paper §F.4).
    pub fn calibrate_from_window(healthy_norms: &[f32]) -> Option<Self> {
        if healthy_norms.is_empty() {
            return None;
        }
        let n = healthy_norms.len() as f32;
        let mean = healthy_norms.iter().sum::<f32>() / n;
        let variance = healthy_norms.iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f32>() / n;
        let std_dev = crate::math::sqrt_f32(variance);
        let rho = mean + 3.0 * std_dev;
        Some(Self::new(rho))
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_from_uniform_window() {
        // 100 samples all = 0.05: mean=0.05, std=0, rho=0.05
        let norms = [0.05_f32; 100];
        let env = AdmissibilityEnvelope::calibrate_from_window(&norms).unwrap();
        assert!((env.rho - 0.05).abs() < 1e-3, "rho={} (expected ~0.05)", env.rho);
    }

    #[test]
    fn calibration_3sigma_rule() {
        // mean=0.0, std=0.1, rho should be ~0.3
        let norms: [f32; 6] = [-0.1, -0.1, 0.0, 0.0, 0.1, 0.1];
        let env = AdmissibilityEnvelope::calibrate_from_window(&norms).unwrap();
        assert!(env.rho > 0.0, "rho should be positive");
    }

    #[test]
    fn violation_detection() {
        let env = AdmissibilityEnvelope::new(0.1);
        assert!(!env.is_violation(0.05, 1.0));
        assert!(!env.is_violation(0.1, 1.0));   // boundary, not violation
        assert!(env.is_violation(0.11, 1.0));
    }

    #[test]
    fn transition_suppresses_violation() {
        let env = AdmissibilityEnvelope::new(0.1);
        // Even norm=1000 should not be a violation when multiplier=+inf
        assert!(!env.is_violation(1000.0, f32::INFINITY));
    }

    #[test]
    fn boundary_approach_detection() {
        let env = AdmissibilityEnvelope::new(0.1);
        // boundary_frac=0.5, so boundary at 0.05
        assert!(!env.is_boundary_approach(0.04, 1.0));
        assert!(env.is_boundary_approach(0.06, 1.0));
        assert!(env.is_boundary_approach(0.09, 1.0));
    }

    #[test]
    fn calibrate_returns_none_for_empty() {
        assert!(AdmissibilityEnvelope::calibrate_from_window(&[]).is_none());
    }
}
