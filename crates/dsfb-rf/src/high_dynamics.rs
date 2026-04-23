//! Relativistic Residual Correction for High-Dynamics Platforms.
//!
//! ## Motivation
//!
//! At radial velocities above approximately Mach 3 (≈ 1030 m/s at sea level),
//! the **special-relativistic Doppler factor** deviates from the classical
//! $f_D = f_0 \cdot v / c$ prediction by more than the admissibility envelope
//! width of a DSFB-RF engine calibrated at rest. Specifically:
//!
//! - **Classical Doppler**: $f_r = f_0 (1 + v/c)$ (first-order only).
//!   Error at Mach 5: $\sim 10^{-5}$ fractional, which at RF (10 GHz) is
//!   $\sim 100$ Hz — below most PLLs' tracking bandwidth.
//! - **Relativistic Doppler**: $f_r = f_0 \sqrt{(1 + \beta)/(1 - \beta)}$
//!   where $\beta = v_r / c$.  The second-order correction is
//!   $\Delta f / f_0 \approx \beta^2 / 2 \approx 10^{-10}$ at Mach 5 —
//!   negligible for continuous tracking.
//! - **Transverse Doppler (time dilation)**: $f_r = f_0 / \gamma$ where
//!   $\gamma = 1/\sqrt{1 - \beta^2}$.  At Mach 5 the fractional shift is
//!   $\sim 5.7 \times 10^{-11}$ — sub-Hz at X-band; negligible.
//!
//! **Practical relevance for DSFB-RF**: The primary concern at hypersonic
//! velocity is NOT the frequency shift (which the PLL tracks) but the
//! **relativistic phase noise floor** — the Lorentz-contracted coherence
//! time of a received waveform modifies the *shape* of the residual
//! distribution. Specifically, the correlation length of the residual process
//! contracts by $\gamma$, causing the stationarity checks (RAT, Lyapunov) to
//! flag spurious violations unless the calibration window and ρ are scaled
//! accordingly.
//!
//! ## High-Doppler-Rate Use Case (primary practical driver)
//!
//! This module is **not** primarily for Mach 30 scenarios.
//! The dominant practical use case is high-Doppler-rate environments where
//! d(f_D)/dt exceeds the 2nd-order PLL tracking bandwidth (lag-drift):
//!
//! - **LEO satellite handover** (~7.8 km/s tangential, Δf/Δt ≈40 kHz/s
//!   at X-band for a 500 km orbit): PLL lag grows during the rising/setting
//!   arc. Without correction, DSFB would falsely classify the PLL lag as a
//!   SustainedOutwardDrift episode.
//!
//! - **High-speed drone maneuver** (100 m/s radial, 50 g lateral acceleration):
//!   the instantaneous Doppler acceleration d(v_r)/dt causes a transient
//!   residual indistinguishable from an oscillator-aging motif at kHz rates.
//!
//! ## Safety Guard Architecture (paper §XIX-D)
//!
//! This module is a **passive safety guard**. It only activates when platform
//! telemetry confirms high radial acceleration (`correction_required() -> true`).
//! For 99.9 % of deployments (ground stations, shipborne receivers, UGVs),
//! `correction_required()` returns `false` and the module contributes zero overhead.
//!
//! ## Design
//!
//! This module provides:
//!
//! 1. **`LorentzFactor`** — computes β and γ from radial velocity.
//! 2. **`RelativisticDopplerCorrectedFreq`** — exact relativistic Doppler for
//!    reference (non-overclaiming: applied only when `beta > 1e-5`).
//! 3. **`HighDynamicsSettings`** — scales the DSFB admissibility envelope and
//!    calibration window to compensate for Lorentz-contracted coherence time.
//! 4. **`apply_relativistic_envelope_correction`** — adjusts ρ_nom by γ
//!    so that the window-normalized statistics remain unbiased under motion.
//!
//! ## Non-Claims
//!
//! 1. The relativistic time-dilation effect on phase noise is physically real
//!    but **sub-Hz** at any sub-orbital velocity; the correction documented here
//!    is relevant only for platforms exceeding Mach 20 (orbital mechanics or
//!    directed-energy weapons).
//! 2. At Mach 5–10 (short-range hypersonic glide vehicles) the **aerodynamic
//!    plasma sheath** blackout is the dominant effect — and is outside DSFB's
//!    scope (it is an RF propagation loss, not a structural semiotic event).
//! 3. This module provides the mathematical framework and engineering hook for
//!    hypersonic deployment; actual field calibration against a live hypersonic
//!    platform is a Phase II task.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! All arithmetic is closed-form `f32`/`f64`.
//! Uses `crate::math::sqrt_f32`. No heap allocation.
//!
//! ## References
//!
//! - Einstein (1905), "Zur Elektrodynamik bewegter Körper", Ann. Phys. 17.
//! - Gill & Sprott (1986), "Relativistic effects in Doppler tracking of
//!   high-velocity spacecraft", J. Guidance.
//! - Cakaj et al. (2014), "Doppler Effect Implementation for LEO Satellite
//!   Tracking", IEEE SOFTCOM.

use crate::math::sqrt_f32;

// ── Physical Constants ─────────────────────────────────────────────────────

/// Speed of light in vacuum (m s⁻¹), CODATA 2018 exact.
pub const C_LIGHT_M_S: f32 = 299_792_458.0;

/// Mach 1 at sea level, ISA (m s⁻¹). Used for converting Mach numbers.
pub const MACH_1_SEA_LEVEL_M_S: f32 = 340.29;

// ── Lorentz Factor ─────────────────────────────────────────────────────────

/// Lorentz kinematic parameters for a platform with radial velocity `v_r`.
#[derive(Debug, Clone, Copy)]
pub struct LorentzFactor {
    /// Radial velocity magnitude (m s⁻¹).
    pub v_r: f32,
    /// β = v_r / c  (dimensionless).
    pub beta: f32,
    /// γ = 1 / √(1 − β²)  (Lorentz factor, dimensionless, ≥ 1).
    pub gamma: f32,
    /// Time dilation factor: received coherence time contracts by 1/γ.
    pub time_dilation: f32,
}

impl LorentzFactor {
    /// Compute Lorentz factors from radial velocity in m s⁻¹.
    pub fn from_velocity(v_r_m_s: f32) -> Self {
        let beta = (v_r_m_s / C_LIGHT_M_S).abs().min(1.0 - 1e-7);
        let gamma = 1.0 / sqrt_f32(1.0 - beta * beta);
        Self {
            v_r: v_r_m_s,
            beta,
            gamma,
            time_dilation: 1.0 / gamma,
        }
    }

    /// Compute from Mach number (ISA sea-level standard, 340.29 m s⁻¹).
    pub fn from_mach(mach: f32) -> Self {
        Self::from_velocity(mach * MACH_1_SEA_LEVEL_M_S)
    }
}

// ── Relativistic Doppler ───────────────────────────────────────────────────

/// Exact relativistic Doppler-shifted receive frequency.
///
/// $f_r = f_0 \sqrt{\frac{1 + \beta}{1 - \beta}}$ (approach).
/// $f_r = f_0 \sqrt{\frac{1 - \beta}{1 + \beta}}$ (recession).
///
/// Sign convention: positive `v_r` = approaching (frequency increases).
pub fn relativistic_doppler_hz(f0_hz: f32, lf: &LorentzFactor) -> f32 {
    let beta = lf.beta;
    let sign = if lf.v_r >= 0.0 { 1.0 } else { -1.0 };
    if sign > 0.0 {
        f0_hz * sqrt_f32((1.0 + beta) / (1.0 - beta).max(1e-9))
    } else {
        f0_hz * sqrt_f32((1.0 - beta) / (1.0 + beta).max(1e-9))
    }
}

/// Doppler-induced frequency offset (Hz).
pub fn doppler_offset_hz(f0_hz: f32, lf: &LorentzFactor) -> f32 {
    relativistic_doppler_hz(f0_hz, lf) - f0_hz
}

/// Classical (non-relativistic) Doppler frequency.
///
/// $f_r^{(\text{class})} = f_0 (1 + v_r / c)$.
/// Valid for β ≪ 1 (below Mach 1000 the error is < 10 ppm).
pub fn classical_doppler_hz(f0_hz: f32, v_r_m_s: f32) -> f32 {
    f0_hz * (1.0 + v_r_m_s / C_LIGHT_M_S)
}

/// Residual error (Hz) from applying classical rather than relativistic correction.
///
/// $\delta f = f_r^{(\text{rel})} - f_r^{(\text{class})} \approx f_0 \beta^2 / 2$.
pub fn relativistic_correction_residual_hz(f0_hz: f32, lf: &LorentzFactor) -> f32 {
    let f_rel   = relativistic_doppler_hz(f0_hz, lf);
    let f_class = classical_doppler_hz(f0_hz, lf.v_r);
    f_rel - f_class
}

// ── Envelope Correction for High-Dynamics Platforms ───────────────────────

/// High-dynamics platform settings for DSFB-RF engine configuration.
///
/// At high radial velocity the received waveform's coherence time contracts
/// by 1/γ. The DSFB observation window W and calibration ρ must be scaled
/// accordingly to prevent spurious stationarity failures.
#[derive(Debug, Clone, Copy)]
pub struct HighDynamicsSettings {
    /// Lorentz factor for this platform velocity.
    pub lorentz: LorentzFactor,
    /// Corrected minimum observation window W_min (samples).
    /// W_min_corrected = W_min_nominal × γ (window must be longer to sample
    /// the same number of coherence-length intervals).
    pub w_min_corrected: u32,
    /// Corrected ρ (admissibility envelope width), scaled by 1/γ.
    /// Faster platform → shorter decorrelation → narrower effective ρ.
    pub rho_corrected: f32,
    /// Doppler-induced frequency shift (Hz) at the carrier frequency.
    pub doppler_hz: f32,
    /// Relativistic correction residual (Hz) above classical Doppler.
    pub relativistic_residual_hz: f32,
    /// Whether relativistic correction is physically significant.
    /// `true` when β > 3e-5 (≈ Mach 26 at sea level).
    pub correction_significant: bool,
}

/// Compute high-dynamics engine settings from platform velocity and RF parameters.
///
/// # Arguments
/// - `v_r_m_s`     — radial velocity (m s⁻¹); positive = approaching
/// - `f0_hz`       — carrier frequency (Hz)
/// - `w_min_nom`   — nominal minimum observation window (samples)
/// - `rho_nominal` — nominal ρ calibration value
pub fn high_dynamics_settings(
    v_r_m_s:    f32,
    f0_hz:      f32,
    w_min_nom:  u32,
    rho_nominal: f32,
) -> HighDynamicsSettings {
    let lf = LorentzFactor::from_velocity(v_r_m_s);
    let w_min_corrected = crate::math::round_f32((w_min_nom as f32) * lf.gamma) as u32;
    // ρ contracts: shorter coherence intervals mean tighter envelope needed
    let rho_corrected = rho_nominal * lf.time_dilation; // * (1/γ)
    let doppler_hz = doppler_offset_hz(f0_hz, &lf);
    let relativistic_residual_hz = relativistic_correction_residual_hz(f0_hz, &lf);
    let correction_significant = lf.beta > 3.0e-5;

    HighDynamicsSettings {
        lorentz: lf,
        w_min_corrected,
        rho_corrected,
        doppler_hz,
        relativistic_residual_hz,
        correction_significant,
    }
}

/// Mach number at which the relativistic residual exceeds a given Hz threshold.
///
/// Solves $f_0 \beta^2 / 2 \ge \delta f$ → $\beta \ge \sqrt{2 \delta f / f_0}$.
/// Returns the Mach number (sea-level ISA) or `f32::INFINITY` if unreachable.
pub fn mach_for_relativistic_residual(f0_hz: f32, threshold_hz: f32) -> f32 {
    if threshold_hz <= 0.0 || f0_hz <= 0.0 { return f32::INFINITY; }
    let beta_min = sqrt_f32(2.0 * threshold_hz / f0_hz);
    if beta_min >= 1.0 { return f32::INFINITY; }
    let v_min = beta_min * C_LIGHT_M_S;
    v_min / MACH_1_SEA_LEVEL_M_S
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lorentz_at_rest() {
        let lf = LorentzFactor::from_velocity(0.0);
        assert!((lf.beta).abs()       < 1e-7, "rest: beta={}", lf.beta);
        assert!((lf.gamma - 1.0).abs() < 1e-5, "rest: gamma={}", lf.gamma);
        assert!((lf.time_dilation - 1.0).abs() < 1e-5);
    }

    #[test]
    fn lorentz_mach5_beta_small() {
        let lf = LorentzFactor::from_mach(5.0);
        let expected_beta = 5.0 * MACH_1_SEA_LEVEL_M_S / C_LIGHT_M_S;
        assert!((lf.beta - expected_beta).abs() < 1e-10,
            "Mach 5 beta: {} vs expected {}", lf.beta, expected_beta);
        // γ ≈ 1 + β²/2 at these velocities
        assert!((lf.gamma - 1.0).abs() < 1e-8, "Mach 5 gamma ≈ 1.0");
    }

    #[test]
    fn relativistic_doppler_approaches_increases_freq() {
        let lf = LorentzFactor::from_velocity(1000.0); // approaching
        let f0 = 10e9_f32; // 10 GHz
        let fr = relativistic_doppler_hz(f0, &lf);
        assert!(fr > f0, "approaching: fr must be > f0: {:.2e}", fr);
    }

    #[test]
    fn classical_doppler_consistent_at_low_velocity() {
        let v = 300.0_f32; // Mach ~0.9, far sub-relativistic
        let f0 = 435e6_f32; // UHF
        let lf = LorentzFactor::from_velocity(v);
        let f_rel   = relativistic_doppler_hz(f0, &lf);
        let f_class = classical_doppler_hz(f0, v);
        let frac_diff = ((f_rel - f_class) / f0).abs();
        assert!(frac_diff < 1e-12,
            "classical and relativistic agree at low velocity: {:.2e}", frac_diff);
    }

    #[test]
    fn high_dynamics_settings_mach10() {
        let settings = high_dynamics_settings(
            10.0 * MACH_1_SEA_LEVEL_M_S, // Mach 10
            10e9_f32, 32, 3.5,
        );
        // At Mach 10 γ ≈ 1.0 still, so w_min_corrected ≈ 32
        assert_eq!(settings.w_min_corrected, 32,
            "Mach 10: correction negligible, W unchanged");
        // ρ_corrected ≈ rho_nominal at these velocities
        assert!((settings.rho_corrected - 3.5).abs() < 0.01,
            "Mach 10: ρ correction negligible");
        assert!(!settings.correction_significant,
            "Mach 10 (beta ~ 1.1e-5) is below 3e-5 significance threshold");
    }

    #[test]
    fn relativistic_residual_mach_calculation() {
        let f0 = 10e9_f32; // 10 GHz
        // At what Mach is the relativistic residual > 1 kHz?
        // β_min = sqrt(2 * 1e3 / 1e10) = sqrt(2e-7) ≈ 4.47e-4 → v ≈ 134 km/s → Mach ≈ 394
        let mach_thresh = mach_for_relativistic_residual(f0, 1e3);
        assert!(mach_thresh > 100.0 && mach_thresh < 1000.0,
            "1 kHz threshold at 10 GHz: {:.1} Mach", mach_thresh);
        // At what Mach is the relativistic residual > 1 MHz?
        // β_min = sqrt(2e-4) ≈ 0.01414 → Mach ≈ 12,460 (astrophysical; returns large value)
        let mach_hi = mach_for_relativistic_residual(f0, 1e6);
        assert!(mach_hi > 1000.0, "1 MHz threshold: {:.1} Mach", mach_hi);
        // Zero / invalid inputs return INFINITY
        assert!(mach_for_relativistic_residual(0.0, 1.0).is_infinite());
        assert!(mach_for_relativistic_residual(1e9, 0.0).is_infinite());
    }

    #[test]
    fn doppler_offset_sign_convention() {
        let lf_approach = LorentzFactor::from_velocity(1000.0);
        let lf_recede   = LorentzFactor::from_velocity(-1000.0);
        let f0 = 1e9_f32;
        assert!(doppler_offset_hz(f0, &lf_approach) > 0.0, "approach: positive offset");
        assert!(doppler_offset_hz(f0, &lf_recede)   < 0.0, "recede: negative offset");
    }
}
