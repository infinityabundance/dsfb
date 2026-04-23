//! GUM-compliant uncertainty budget for admissibility envelopes.
//!
//! ## Theoretical Basis: Guide to the Expression of Uncertainty in Measurement (GUM)
//!
//! GUM (JCGM 100:2008 / ISO/IEC Guide 98-3) and IEEE 1764 require that
//! measurement uncertainty budgets decompose the total uncertainty into
//! Type A (statistical) and Type B (systematic) contributions.
//!
//! DSFB applies this framework to the admissibility envelope radius ρ:
//!
//! **Type A (statistical):** Standard uncertainty of the healthy-window
//! residual norm, derived from N independent observations:
//!   u_A = σ_healthy / √N
//!
//! **Type B (systematic):** Known systematic contributors that affect
//! the residual but are not captured in the calibration window:
//! - Receiver noise figure uncertainty (typically ±0.5 dB → mapped to norm)
//! - ADC quantization noise (Q / √12 for Q = LSB in residual norm units)
//! - Temperature-dependent gain variation (manufacturer specification)
//! - Clock/LO phase noise floor contribution
//!
//! **Combined standard uncertainty:**
//!   u_c = √(u_A² + Σᵢ u_B,i²)
//!
//! **Expanded uncertainty (coverage k=3 for 99.7% confidence):**
//!   U = k · u_c = 3 · u_c
//!
//! The expanded uncertainty U is the principled basis for setting ρ.
//! Using ρ = μ + U (with k=3) provides a GUM-traceable envelope rather
//! than an ad-hoc "3σ rule."
//!
//! ## Pre-condition: WSS Verification
//!
//! GUM requires that Type A uncertainty be derived from a stationary process.
//! The stationarity module (`stationarity.rs`) provides the WSS pre-condition.
//! If the calibration window fails WSS verification, the GUM uncertainty
//! budget is flagged as unreliable.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - O(n) single-pass computation from healthy window norms

use crate::math::{mean_f32, std_dev_f32, sqrt_f32};

/// A single Type B systematic uncertainty contributor.
#[derive(Debug, Clone, Copy)]
pub struct TypeBContributor {
    /// Human-readable name of the contributor.
    pub name: &'static str,
    /// Standard uncertainty in residual norm units.
    pub u_b: f32,
    /// Source description (manufacturer spec, calibration certificate, etc.).
    pub source: &'static str,
}

/// Complete GUM uncertainty budget for the admissibility envelope.
#[derive(Debug, Clone, Copy)]
pub struct UncertaintyBudget {
    /// Number of observations in the calibration window.
    pub n_observations: usize,
    /// Healthy window mean μ.
    pub mean: f32,
    /// Healthy window standard deviation σ.
    pub std_dev: f32,
    /// Type A standard uncertainty: u_A = σ / √N.
    pub u_a: f32,
    /// Combined Type B standard uncertainty: √(Σ u_B,i²).
    pub u_b_combined: f32,
    /// Combined standard uncertainty: u_c = √(u_A² + u_B²).
    pub u_c: f32,
    /// Coverage factor k (default: 3.0 for 99.7% confidence).
    pub coverage_factor: f32,
    /// Expanded uncertainty: U = k · u_c.
    pub expanded_uncertainty: f32,
    /// GUM-derived envelope radius: ρ = μ + U.
    pub rho_gum: f32,
    /// Whether the WSS pre-condition was satisfied.
    pub wss_verified: bool,
}

/// Configuration for the GUM uncertainty budget.
#[derive(Debug, Clone)]
pub struct UncertaintyConfig {
    /// Coverage factor k. Default 3.0 (99.7% confidence interval).
    pub coverage_factor: f32,
    /// Type B systematic uncertainty contributors.
    /// Fixed-capacity array for no_alloc compatibility.
    pub type_b: [Option<TypeBContributor>; 8],
    /// Number of populated Type B entries.
    pub type_b_count: usize,
}

impl Default for UncertaintyConfig {
    fn default() -> Self {
        Self {
            coverage_factor: 3.0,
            type_b: [None; 8],
            type_b_count: 0,
        }
    }
}

impl UncertaintyConfig {
    /// Create a config with typical RF receiver Type B contributors.
    ///
    /// Uses conservative estimates for common SDR receivers (USRP B200 class):
    /// - Noise figure uncertainty: ±0.005 normalized norm units (~0.5 dB)
    /// - ADC quantization: ±0.001 (14-bit ADC → Q ≈ 6e-5, Q/√12 ≈ 1.7e-5)
    /// - Thermal gain drift: ±0.003 (0.02 dB/°C over ±10°C range)
    pub fn typical_sdr() -> Self {
        let mut cfg = Self::default();
        cfg.add_type_b(TypeBContributor {
            name: "receiver_noise_figure",
            u_b: 0.005,
            source: "manufacturer_specification_±0.5dB",
        });
        cfg.add_type_b(TypeBContributor {
            name: "adc_quantization",
            u_b: 0.001,
            source: "14bit_ADC_Q_div_sqrt12",
        });
        cfg.add_type_b(TypeBContributor {
            name: "thermal_gain_drift",
            u_b: 0.003,
            source: "0.02dB_per_C_over_10C_range",
        });
        cfg
    }

    /// Add a Type B contributor. Returns false if the array is full.
    pub fn add_type_b(&mut self, contrib: TypeBContributor) -> bool {
        if self.type_b_count >= 8 { return false; }
        self.type_b[self.type_b_count] = Some(contrib);
        self.type_b_count += 1;
        true
    }
}

/// Compute the GUM uncertainty budget from healthy-window norms.
///
/// Returns `None` if the window is empty.
pub fn compute_budget(
    healthy_norms: &[f32],
    config: &UncertaintyConfig,
    wss_verified: bool,
) -> Option<UncertaintyBudget> {
    if healthy_norms.is_empty() {
        return None;
    }

    let n = healthy_norms.len();
    let mean = mean_f32(healthy_norms);
    let std_dev = std_dev_f32(healthy_norms);

    // Type A: u_A = σ / √N
    let u_a = std_dev / sqrt_f32(n as f32);

    // Type B: u_B = √(Σ u_B,i²)
    let mut u_b_sq = 0.0_f32;
    for i in 0..config.type_b_count {
        if let Some(ref c) = config.type_b[i] {
            u_b_sq += c.u_b * c.u_b;
        }
    }
    let u_b_combined = sqrt_f32(u_b_sq);

    // Combined: u_c = √(u_A² + u_B²)
    let u_c = sqrt_f32(u_a * u_a + u_b_combined * u_b_combined);

    // Expanded: U = k · u_c
    let expanded = config.coverage_factor * u_c;

    // GUM-derived ρ: μ + U
    let rho_gum = mean + expanded;

    Some(UncertaintyBudget {
        n_observations: n,
        mean,
        std_dev,
        u_a,
        u_b_combined,
        u_c,
        coverage_factor: config.coverage_factor,
        expanded_uncertainty: expanded,
        rho_gum,
        wss_verified,
    })
}

// ── CRLB Floor ─────────────────────────────────────────────────────────────
//
// Cramér-Rao Lower Bound for phase and frequency estimation.
//
// For a single complex tone in AWGN with N observations at linear SNR γ:
//   CRLB_phase = 1 / (N · γ)           [rad²]          (Kay 1993, §3.7)
//   CRLB_freq  = 6 / (N³ · γ · (2π)²) [normalized Hz²] (Rife & Boorstyn 1974)
//
// The physics-noise floor for the admissibility radius is:
//   ρ_floor = 1 / √γ
//
// If the current ρ is less than MARGIN_FACTOR × ρ_floor, the envelope
// is operating dangerously close to the theoretical noise floor.
// Margins < 3× constitute a CRLB alert (the envelope cannot reliably
// distinguish admissible drift from measurement noise).
//
// References:
//   Kay, S.M. (1993) "Fundamentals of Statistical Signal Processing:
//       Estimation Theory," Prentice Hall, §3.7.  ISBN 0-13-345711-7.
//   Rife, D.C. and Boorstyn, R.R. (1974) "Single-tone parameter
//       estimation from discrete-time observations," IEEE Trans.
//       Inf. Theory, 20(5):591–598. doi:10.1109/TIT.1974.1055282.
//   Van Trees, H.L. (1968) "Detection, Estimation, Modulation Theory,"
//       Part I. Wiley. §2.4.

/// Minimum margin factor to avoid operating in the noise-floor regime.
pub const CRLB_MARGIN_THRESHOLD: f32 = 3.0;

/// Cramér-Rao Lower Bound floor and admissibility margin.
///
/// Encapsulates the theoretical noise-floor limits for phase and frequency
/// estimation at a given SNR and observation count.
#[derive(Debug, Clone, Copy)]
pub struct CrlbFloor {
    /// SNR used for computation, in dB.
    pub snr_db: f32,
    /// Number of observations (calibration window length).
    pub n_observations: usize,
    /// CRLB for phase variance [rad²]: 1 / (N · γ).
    pub crlb_phase_var: f32,
    /// CRLB for frequency variance [normalized]: 6 / (N³ · γ · (2π)²).
    pub crlb_freq_var: f32,
    /// Physics noise floor for ρ: 1 / √γ.
    pub rho_physics_floor: f32,
    /// Whether the current ρ is above the physics floor.
    pub rho_above_physics_floor: bool,
    /// Margin factor: ρ / ρ_floor. Values < CRLB_MARGIN_THRESHOLD warrant alert.
    pub rho_margin_factor: f32,
    /// Whether a CRLB margin alert is raised (margin < threshold).
    pub crlb_alert: bool,
}

/// Compute the CRLB floor and admissibility margin for the given SNR and ρ.
///
/// - `snr_db`:        Observed SNR in dB (typically the calibration-window SNR estimate).
/// - `n_observations`: Number of observations in the calibration window.
/// - `rho`:           Current admissibility radius (from GUM budget or direct assignment).
///
/// Returns `None` if `n_observations == 0` or `snr_db` is extremely negative (< −60 dB).
pub fn compute_crlb_floor(
    snr_db: f32,
    n_observations: usize,
    rho: f32,
) -> Option<CrlbFloor> {
    if n_observations == 0 { return None; }
    if snr_db < -60.0 { return None; } // below practical floor

    // Linear SNR: γ = 10^(snr_db / 10)
    let gamma = pow10_approx(snr_db / 10.0);
    if gamma <= 0.0 { return None; }

    let n = n_observations as f32;
    let n3 = n * n * n;
    let two_pi_sq = 4.0 * 9.869_604_f32; // (2π)² = 4π²

    // CRLB phase: 1 / (N · γ)
    let crlb_phase = 1.0 / (n * gamma);
    // CRLB freq: 6 / (N³ · γ · (2π)²)
    let crlb_freq = 6.0 / (n3 * gamma * two_pi_sq);

    // Physics noise floor for ρ
    let rho_floor = 1.0 / crate::math::sqrt_f32(gamma);

    let above = rho > rho_floor;
    let margin = if rho_floor > 0.0 { rho / rho_floor } else { f32::MAX };
    let alert = margin < CRLB_MARGIN_THRESHOLD;

    Some(CrlbFloor {
        snr_db,
        n_observations,
        crlb_phase_var: crlb_phase,
        crlb_freq_var: crlb_freq,
        rho_physics_floor: rho_floor,
        rho_above_physics_floor: above,
        rho_margin_factor: margin,
        crlb_alert: alert,
    })
}

// ── Private math helpers (no libm) ─────────────────────────────────────────

/// Compute 10^x without libm by reducing to 2^(x · log₂10).
///
/// Accurate to < 0.3% for |x| ≤ 10 (covers SNR range −100 dB to +100 dB after /10).
fn pow10_approx(x: f32) -> f32 {
    // 10^x = 2^(x · log₂(10));  log₂(10) ≈ 3.321_928
    pow2_approx(x * 3.321_928_f32)
}

/// Compute 2^y for arbitrary float y.
///
/// Splits y into integer n and fractional f parts, uses a 3-term Horner
/// polynomial for 2^f, then scales by 2^n via repeated multiply/divide.
fn pow2_approx(y: f32) -> f32 {
    // Clamp to a safe range to avoid overflow/underflow
    let y = if y > 120.0 { 120.0 } else if y < -120.0 { -120.0 } else { y };
    let n = if y >= 0.0 { y as i32 } else { y as i32 - 1 };
    let frac = y - n as f32; // ∈ [0, 1)
    // 2^frac ≈ 1 + frac·(ln2 + frac·(ln2²/2 + frac·ln2³/6)) — Taylor of exp(frac·ln2)
    let ln2 = 0.693_147_f32;
    let mantissa = 1.0 + frac * (ln2 + frac * (0.240_226_f32 + frac * 0.055_504_f32));
    // Scale by 2^n
    if n >= 0 {
        let mut acc = 1.0_f32;
        for _ in 0..n { acc *= 2.0; }
        acc * mantissa
    } else {
        let mut acc = 1.0_f32;
        for _ in 0..(-n) { acc *= 0.5; }
        acc * mantissa
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_from_constant_window() {
        let norms = [0.05_f32; 100];
        let config = UncertaintyConfig::default();
        let budget = compute_budget(&norms, &config, true).unwrap();
        // σ = 0, so u_A = 0, u_c = 0, ρ = μ = 0.05
        assert!((budget.mean - 0.05).abs() < 1e-4);
        assert!(budget.u_a < 1e-4, "u_A should be ~0 for constant window");
        assert!((budget.rho_gum - 0.05).abs() < 1e-3);
        assert!(budget.wss_verified);
    }

    #[test]
    fn budget_with_type_b_contributors() {
        let norms = [0.05_f32; 100];
        let config = UncertaintyConfig::typical_sdr();
        let budget = compute_budget(&norms, &config, true).unwrap();
        // With Type B contributors, ρ_GUM > μ
        assert!(budget.rho_gum > budget.mean,
            "ρ_GUM must exceed mean with Type B contributors");
        assert!(budget.u_b_combined > 0.0);
        assert!(budget.u_c > 0.0);
    }

    #[test]
    fn budget_type_a_decreases_with_n() {
        let norms_small: [f32; 10] = core::array::from_fn(|i| 0.05 + i as f32 * 0.001);
        let norms_large: [f32; 100] = core::array::from_fn(|i| 0.05 + (i % 10) as f32 * 0.001);
        let config = UncertaintyConfig::default();
        let b_small = compute_budget(&norms_small, &config, true).unwrap();
        let b_large = compute_budget(&norms_large, &config, true).unwrap();
        assert!(b_large.u_a < b_small.u_a,
            "u_A must decrease with more observations: {} vs {}", b_large.u_a, b_small.u_a);
    }

    #[test]
    fn budget_coverage_factor_scales() {
        let norms: [f32; 50] = core::array::from_fn(|i| 0.05 + (i as f32 * 0.001).sin() * 0.01);
        let mut cfg_k2 = UncertaintyConfig::default();
        cfg_k2.coverage_factor = 2.0;
        let mut cfg_k3 = UncertaintyConfig::default();
        cfg_k3.coverage_factor = 3.0;
        let b2 = compute_budget(&norms, &cfg_k2, true).unwrap();
        let b3 = compute_budget(&norms, &cfg_k3, true).unwrap();
        assert!(b3.expanded_uncertainty > b2.expanded_uncertainty,
            "k=3 must give larger U than k=2");
    }

    #[test]
    fn returns_none_for_empty() {
        assert!(compute_budget(&[], &UncertaintyConfig::default(), true).is_none());
    }

    // ── CRLB Floor Tests ───────────────────────────────────────────────────

    #[test]
    fn crlb_returns_none_for_zero_obs() {
        assert!(compute_crlb_floor(10.0, 0, 0.1).is_none());
    }

    #[test]
    fn crlb_returns_none_below_practical_floor() {
        assert!(compute_crlb_floor(-70.0, 100, 0.1).is_none());
    }

    #[test]
    fn crlb_high_snr_low_variance() {
        let c = compute_crlb_floor(30.0, 100, 0.2).unwrap();
        // At 30 dB SNR with 100 obs, CRLB_phase = 1/(100·1000) = 1e-5
        assert!(c.crlb_phase_var < 1e-3,
            "high-SNR CRLB_phase should be very small: {}", c.crlb_phase_var);
        assert!(c.rho_above_physics_floor);
    }

    #[test]
    fn crlb_low_snr_alert_raised() {
        // At -10 dB SNR, γ ≈ 0.1, ρ_floor = 1/√0.1 ≈ 3.16
        // ρ = 0.5 → margin ≈ 0.16 << 3.0 → alert
        let c = compute_crlb_floor(-10.0, 50, 0.5).unwrap();
        assert!(c.crlb_alert,
            "low-SNR with small ρ must raise CRLB alert: margin={}", c.rho_margin_factor);
        assert!(!c.rho_above_physics_floor);
    }

    #[test]
    fn crlb_rho_above_floor_no_alert_if_large_margin() {
        // At 20 dB SNR, γ=100, ρ_floor = 1/10 = 0.1
        // ρ = 0.5 → margin = 5.0 > 3.0 → no alert
        let c = compute_crlb_floor(20.0, 100, 0.5).unwrap();
        assert!(!c.crlb_alert,
            "large margin must not alert: margin={}", c.rho_margin_factor);
        assert!(c.rho_margin_factor > CRLB_MARGIN_THRESHOLD);
    }

    #[test]
    fn crlb_freq_var_decreases_with_more_obs() {
        let c100 = compute_crlb_floor(0.0, 100, 0.5).unwrap();
        let c200 = compute_crlb_floor(0.0, 200, 0.5).unwrap();
        assert!(c200.crlb_freq_var < c100.crlb_freq_var,
            "CRLB_freq must decrease with more observations");
    }
}
