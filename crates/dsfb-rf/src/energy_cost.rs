//! Thermodynamic Spectrum Auditing via Landauer's Principle.
//!
//! ## Motivation
//!
//! Every bit of unmodeled residual structure represents information that the
//! receiver's DSP stack must process without being able to bound it in time.
//! Landauer's Principle (Bennett 1982, following Landauer 1961) states that
//! erasing a single bit of entropy requires a minimum energy dissipation:
//!
//! $$E_{\min} = k_B \cdot T \cdot \ln 2$$
//!
//! where $k_B = 1.380649 \times 10^{-23}\ \mathrm{J\,K^{-1}}$ is the
//! Boltzmann constant and $T$ is the thermodynamic temperature.
//!
//! DSFB-RF uses this to convert the *structural information excess* of a
//! `Boundary` or `Violation` grammar state into physical units:
//! **Joules per observation** and **Watts** of entropy-processing burden.
//! This yields the "Structural Energy Waste" metric — a thermodynamic
//! quantity that is entirely absent from scalar SNR-based detectors.
//!
//! ## Design
//!
//! The residual stream carries (at minimum) thermal kT noise. Any residual
//! entropy **above** the thermal floor is *structural entropy* — information
//! induced by an exogenous emitter that the receiver's processor must carry
//! without being able to classify it as desired signal.
//!
//! We quantify this per-observation as:
//!
//! $$E_{\mathrm{struct}} = k_B \cdot T \cdot H_{\mathrm{excess}} \cdot \ln 2$$
//!
//! where $H_{\mathrm{excess}} = H_{\mathrm{obs}} - H_{\mathrm{thermal}}$ is
//! the excess differential entropy above the Johnson-Nyquist floor.
//!
//! For a Gaussian residual of variance $\sigma^2$ the differential entropy is:
//!
//! $$H = \tfrac{1}{2}\ln(2\pi e\,\sigma^2)$$
//!
//! At the thermal floor: $\sigma_{\mathrm{th}}^2 = k_B T B$ (Johnson-Nyquist
//! power in bandwidth $B$, 1-Ω impedance).
//!
//! ## Non-Claims
//!
//! The Landauer bound is the *thermodynamic minimum* — real DSP stacks
//! dissipate orders of magnitude more energy per bit. This module reports
//! the **fundamental lower bound** as a physically meaningful comparative
//! metric, not the actual processor power draw. The value is a relative
//! tool for comparing structural entropy burden across spectrum conditions.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! All arithmetic is closed-form; uses `crate::math::{ln_f32, sqrt_f32}`.
//! No heap allocation. Crate-wide `#![forbid(unsafe_code)]` applies.
//!
//! ## References
//!
//! - Landauer (1961), "Irreversibility and Heat Generation in the Computing
//!   Process", IBM J. Res. Dev. 5(3):183-191.
//! - Bennett (1982), "The thermodynamics of computation", IJTP 21(12).
//! - Brillouin (1956), Science and Information Theory, Academic Press.
//! - CODATA 2018: $k_B = 1.380649 \times 10^{-23}\ \mathrm{J\,K^{-1}}$ (exact).

// ── Physical Constants ─────────────────────────────────────────────────────

/// Boltzmann constant $k_B$ (J / K), CODATA 2018 exact.
pub const K_BOLTZMANN: f64 = 1.380_649e-23_f64;

/// Natural logarithm of 2 (used in Landauer's formula).
pub const LN2: f64 = core::f64::consts::LN_2;

/// Room temperature reference (T₀ = 290 K, IEEE 802 noise figure reference).
pub const T_ROOM_K: f32 = 290.0;

/// Landauer energy per bit erasure at room temperature T₀ = 290 K (J).
///
/// $E_L = k_B \cdot T_0 \cdot \ln 2 \approx 2.77 \times 10^{-21}\ \mathrm{J}$
pub const LANDAUER_ROOM_J: f64 = K_BOLTZMANN * 290.0 * LN_2_F64;

const LN_2_F64: f64 = core::f64::consts::LN_2;

// ── Thermal Reference ─────────────────────────────────────────────────────

/// Johnson-Nyquist noise variance at temperature T and bandwidth B (Ω = 1).
///
/// $\sigma_{\mathrm{th}}^2 = k_B T B$ in SI units.
/// Returns the noise power in Watts (at 1 Ω) as `f32`.
#[inline]
pub fn thermal_noise_power(temp_k: f32, bandwidth_hz: f32) -> f32 {
    (K_BOLTZMANN as f32) * temp_k * bandwidth_hz
}

/// Differential entropy (nats) of a zero-mean Gaussian with variance σ².
///
/// $H = \tfrac{1}{2}\ln(2\pi e\,\sigma^2)$
///
/// Returns entropy in nats. Multiply by $1/\ln 2$ to convert to bits.
#[inline]
pub fn gaussian_entropy_nats(sigma_sq: f32) -> f32 {
    use crate::math::ln_f32;
    // ln(2πe σ²) / 2
    let two_pi_e: f32 = 2.0 * core::f32::consts::PI * core::f32::consts::E;
    0.5 * ln_f32(two_pi_e * sigma_sq.max(1e-30))
}

/// Excess structural entropy (nats) above the Johnson-Nyquist floor.
///
/// $H_{\mathrm{excess}} = H_{\mathrm{obs}} - H_{\mathrm{thermal}}$
///
/// A negative result (residual below thermal floor) is clamped to zero —
/// the structural entropy burden is non-negative by definition.
#[inline]
pub fn excess_entropy_nats(obs_sigma_sq: f32, thermal_sigma_sq: f32) -> f32 {
    let h_obs = gaussian_entropy_nats(obs_sigma_sq);
    let h_th  = gaussian_entropy_nats(thermal_sigma_sq.max(1e-30));
    (h_obs - h_th).max(0.0)
}

// ── Landauer Structural Energy ─────────────────────────────────────────────

/// Structural energy burden per observation (Joules).
///
/// This is the Landauer minimum energy required to process the excess
/// structural entropy in one observation interval at temperature `temp_k`.
///
/// $$E_{\mathrm{struct}} = k_B \cdot T \cdot H_{\mathrm{excess}}$$
///
/// (The `ln 2` factor appears when converting bits to nats; since
/// `gaussian_entropy_nats` already returns nats, we use k_B·T directly.)
#[inline]
pub fn structural_energy_joules(
    obs_sigma_sq:     f32,
    thermal_sigma_sq: f32,
    temp_k:           f32,
) -> f32 {
    let h_excess = excess_entropy_nats(obs_sigma_sq, thermal_sigma_sq);
    (K_BOLTZMANN as f32) * temp_k * h_excess
}

/// Structural power burden (Watts) given sample rate `fs_hz`.
///
/// $P_{\mathrm{struct}} = E_{\mathrm{struct}} \times f_s$
///
/// Interpretation: the irreducible thermodynamic power that an ideal
/// receiver-side processor must dissipate to process the excess structural
/// entropy per second. Even a computationally perfect Brownian engine
/// cannot do better than this minimum.
#[inline]
pub fn structural_power_watts(
    obs_sigma_sq:     f32,
    thermal_sigma_sq: f32,
    temp_k:           f32,
    fs_hz:            f32,
) -> f32 {
    structural_energy_joules(obs_sigma_sq, thermal_sigma_sq, temp_k) * fs_hz
}

// ── Grammar-State Energy Audit ─────────────────────────────────────────────

/// Landauer energy classification of a grammar state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LandauerClass {
    /// Residual below thermal floor — structurally empty. Zero burden.
    SubThermal,
    /// Residual indistinguishable from Johnson-Nyquist floor.
    /// Minimal structural burden ($H_{\text{excess}} < 0.1$ nat).
    Thermal,
    /// Residual contains mild structural excess.
    /// Typical for `Boundary` grammar state during onset.
    MildBurden,
    /// Residual contains significant structural entropy.
    /// Typical for `Violation` grammar state under sustained interference.
    ModerateBurden,
    /// Residual entropy well above thermal floor.
    /// Jammer or deliberate emitter saturating the receiver's observation
    /// capacity. Maximum Landauer burden.
    SevereBurden,
}

/// Complete thermodynamic audit for one observation window.
#[derive(Debug, Clone, Copy)]
pub struct LandauerAudit {
    /// Observed residual variance σ²_obs.
    pub obs_sigma_sq:     f32,
    /// Thermal floor variance σ²_th at the specified temperature and bandwidth.
    pub thermal_sigma_sq: f32,
    /// Excess structural entropy above thermal floor (nats).
    pub excess_nats:      f32,
    /// Structural energy per observation (Joules, Landauer minimum bound).
    pub energy_joules:    f32,
    /// Structural power burden (Watts, at sample rate fs).
    pub power_watts:      f32,
    /// Classification of structural burden.
    pub class:            LandauerClass,
    /// Effective "entropy multiplier" = σ²_obs / σ²_th.
    /// Values > 1 confirm a structural emitter above the noise floor.
    pub entropy_ratio:    f32,
}

impl LandauerAudit {
    /// Non-claim note for operator display.
    pub const DISCLAIMER: &'static str =
        "Landauer bound is a thermodynamic minimum; actual DSP power \
         exceeds this by ~10^20. Use as relative comparative metric only.";
}

/// Compute a full Landauer thermodynamic audit.
///
/// # Arguments
/// - `obs_sigma_sq`   — observed residual variance over the window
/// - `bandwidth_hz`   — receiver measurement bandwidth (Hz)
/// - `temp_k`         — receiver physical temperature (K); use 290.0 for room temp
/// - `fs_hz`          — sample rate (Hz), used to convert energy → power
pub fn landauer_audit(
    obs_sigma_sq: f32,
    bandwidth_hz: f32,
    temp_k:       f32,
    fs_hz:        f32,
) -> LandauerAudit {
    let thermal_sigma_sq = thermal_noise_power(temp_k, bandwidth_hz);
    let excess_nats = excess_entropy_nats(obs_sigma_sq, thermal_sigma_sq);
    let energy_joules = structural_energy_joules(obs_sigma_sq, thermal_sigma_sq, temp_k);
    let power_watts = energy_joules * fs_hz;
    let entropy_ratio = obs_sigma_sq / thermal_sigma_sq.max(1e-30);

    let class = if obs_sigma_sq <= thermal_sigma_sq {
        LandauerClass::SubThermal
    } else if excess_nats < 0.1 {
        LandauerClass::Thermal
    } else if excess_nats < 1.0 {
        LandauerClass::MildBurden
    } else if excess_nats < 5.0 {
        LandauerClass::ModerateBurden
    } else {
        LandauerClass::SevereBurden
    };

    LandauerAudit {
        obs_sigma_sq,
        thermal_sigma_sq,
        excess_nats,
        energy_joules,
        power_watts,
        class,
        entropy_ratio,
    }
}

/// Cumulative structural energy over a window of audits (Joules).
pub fn cumulative_energy(audits: &[LandauerAudit]) -> f32 {
    audits.iter().map(|a| a.energy_joules).sum()
}

/// Maximum instantaneous structural power in a window (Watts).
pub fn peak_power(audits: &[LandauerAudit]) -> f32 {
    audits.iter().map(|a| a.power_watts).fold(0.0_f32, f32::max)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermal_noise_power_room_temp() {
        // At T=290 K, B=1 MHz: P = kT·B = 1.380649e-23 × 290 × 1e6 ≈ 4.0e-15 W
        let p = thermal_noise_power(290.0, 1e6);
        assert!(p > 3.5e-15 && p < 4.5e-15,
            "thermal noise power at 290K/1MHz: {:.3e}", p);
    }

    #[test]
    fn gaussian_entropy_increases_with_variance() {
        let h_small = gaussian_entropy_nats(0.001);
        let h_large = gaussian_entropy_nats(0.1);
        assert!(h_large > h_small, "entropy must increase with variance");
    }

    #[test]
    fn excess_entropy_zero_at_thermal_floor() {
        // When obs == thermal, no structural excess
        let sigma_sq = 1e-12_f32;
        let excess = excess_entropy_nats(sigma_sq, sigma_sq);
        assert!(excess.abs() < 1e-5, "no excess at thermal floor: {}", excess);
    }

    #[test]
    fn excess_entropy_positive_above_floor() {
        let thermal = 1e-12_f32;
        let obs = 1e-9_f32; // 1000x above floor
        let excess = excess_entropy_nats(obs, thermal);
        assert!(excess > 3.0, "excess nats >> 0 when obs >> thermal: {}", excess);
    }

    #[test]
    fn structural_energy_increases_with_obs_variance() {
        let thermal = 1e-12_f32;
        let e_mild = structural_energy_joules(1e-11, thermal, 290.0);
        let e_severe = structural_energy_joules(1e-9, thermal, 290.0);
        assert!(e_severe > e_mild, "more obs variance → more Landauer burden");
        assert!(e_mild > 0.0, "non-zero burden above thermal floor");
    }

    #[test]
    fn sub_thermal_gives_no_energy() {
        // Observation below thermal floor → zero energy (clamped)
        let thermal = 1e-12_f32;
        let obs = 1e-13_f32; // below floor
        let e = structural_energy_joules(obs, thermal, 290.0);
        assert_eq!(e, 0.0, "sub-thermal residual must produce zero Landauer energy");
    }

    #[test]
    fn landauer_audit_class_severe_at_high_snr() {
        let audit = landauer_audit(1e-9, 1e6, 290.0, 1e6);
        assert_eq!(audit.class, LandauerClass::SevereBurden,
            "1000x above thermal must be SevereBurden: {:?}", audit.class);
        assert!(audit.entropy_ratio > 100.0,
            "entropy_ratio must be > 100: {}", audit.entropy_ratio);
    }

    #[test]
    fn landauer_audit_class_thermal_at_floor() {
        let audit = landauer_audit(
            thermal_noise_power(290.0, 1e6) * 1.01, // just above floor
            1e6, 290.0, 1e6,
        );
        assert!(
            matches!(audit.class, LandauerClass::Thermal | LandauerClass::MildBurden),
            "just above floor: {:?}", audit.class
        );
    }

    #[test]
    fn cumulative_energy_sums_correctly() {
        let a = landauer_audit(1e-10, 1e6, 290.0, 1e6);
        let b = landauer_audit(1e-9,  1e6, 290.0, 1e6);
        let total = cumulative_energy(&[a, b]);
        // f32 has ~7 significant digits; allow relative error of 1e-4
        let diff = (total - a.energy_joules - b.energy_joules).abs();
        let scale = a.energy_joules + b.energy_joules;
        assert!(diff < scale * 1e-4 + 1e-30,
            "cumulative mismatch: diff={:.3e} scale={:.3e}", diff, scale);
    }

    #[test]
    fn peak_power_selects_maximum() {
        let a = landauer_audit(1e-10, 1e6, 290.0, 2e6);
        let b = landauer_audit(1e-9,  1e6, 290.0, 2e6);
        let peak = peak_power(&[a, b]);
        assert!((peak - b.power_watts).abs() < b.power_watts * 1e-4 + 1e-30,
            "peak must be the higher-variance audit");
    }

    #[test]
    fn landauer_room_constant_reasonable() {
        // k_B * T0 * ln2 ≈ 2.804e-21 J
        assert!(LANDAUER_ROOM_J > 2.5e-21 && LANDAUER_ROOM_J < 3.2e-21,
            "Landauer room constant: {:.3e}", LANDAUER_ROOM_J);
    }
}
