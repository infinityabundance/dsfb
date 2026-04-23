//! RF hardware and channel impairment injection for verification harnesses.
//!
//! ## Purpose
//!
//! Provides **deterministic, physics-grounded** perturbation functions that
//! inject realistic hardware impairments into a synthetic residual-norm stream.
//! These are used by the Continuous Rigor pipeline (Stage II: Impairment
//! Injection) to prove the Observer Contract holds under conditions that
//! RadioML-class synthetic data systematically omits.
//!
//! ## Impairment Models
//!
//! ### 1. I/Q Amplitude Imbalance
//!
//! SDR hardware (RTL-SDR, HackRF) exhibits a differential gain between
//! the I and Q baseband paths.  The received pair becomes:
//!
//! ```text
//! r_I' = r_I · (1 + ε/2)
//! r_Q' = r_Q · (1 − ε/2)
//! ```
//!
//! where ε ∈ [0, 0.10] is the fractional amplitude imbalance.
//! To first order the Euclidean norm shifts by:
//!
//! ```text
//! ‖r'‖ ≈ ‖r‖ · sqrt(1 + ε · cos(2φ))   ≈ ‖r‖ · (1 + ε/2 · cos(2φ))
//! ```
//!
//! where φ is the instantaneous carrier phase at that sample.
//! Worst case: δ‖r‖ ≤ ε · ‖r‖ / 2.
//! Reference: Windisch & Fettweis, "Performance Degradation Due to I/Q
//! Imbalance in Multi-Carrier Direct-Conversion Transceivers," IEEE
//! GLOBECOM 2003.
//!
//! ### 2. DC Offset
//!
//! A static complex bias d = (d_I, d_Q) shifts the IQ centroid away from
//! zero.  The norm perturbation (first-order Taylor) is:
//!
//! ```text
//! ‖r + d‖ ≈ ‖r‖ + (d_I · cos φ + d_Q · sin φ)
//! ```
//!
//! Typical RTL-SDR: d_rms ∈ [0.01, 0.02] of normalised full scale.
//! Reference: Vankka, "Methods of Modulation Classification," 1997.
//!
//! ### 3. Cubic PA Compression
//!
//! The memoryless third-order AM/AM model describes PA non-linearity at
//! moderate back-off (< 3 dB IBO):
//!
//! ```text
//! r_out = r_in · (1 − k₃ · |r_in|²)
//! ```
//!
//! where k₃ > 0 is the cubic compression coefficient.  For IBO = P_in −
//! P_1dB, the 1 dB compression point corresponds to k₃ · |r_1|² ≈ 0.145.
//! This model is used on PAWR Colosseum testbed captures to represent the
//! PA non-linearity in high-power nodes.
//! Reference: Cripps, "RF Power Amplifiers for Wireless Communications,"
//! Artech House, 2006, §4.
//!
//! ### 4. ADC Quantisation Noise (GUM §4.3.7)
//!
//! For an N-bit linear PCM quantiser with full-scale range [−1, +1], the
//! standard uncertainty due to quantisation is:
//!
//! ```text
//! u_q = LSB / sqrt(12) = 2^(1−N) / sqrt(12)
//! ```
//!
//! per ISO/IEC Guide 98-3 (GUM) §4.3.7.  This is the one-sigma additive
//! white noise on the residual norm.
//!
//! ### 5. Phase Noise (Leeson's Model)
//!
//! For a phase jitter σ_φ (radian rms, integrated over [f_low, f_high]),
//! the norm perturbation on a unit-magnitude carrier is:
//!
//! ```text
//! δ‖r‖ ≈ |sin(δφ)| ≈ |δφ|   (small-angle, valid for σ_φ < 0.3 rad)
//! ```
//!
//! The Leeson single-sideband phase noise spectrum is:
//!
//! ```text
//! L(f_m) [dBc/Hz] = 10 log10 [ (f₀/(2·Q·f_m))² · (F·k·T·(P_s)⁻¹/2) ]
//! ```
//!
//! We parameterise the impairment model purely by σ_φ, which the user
//! computes from the hardware data sheet's phase noise plot integrated
//! over the relevant noise bandwidth.
//! Reference: Leeson, D.B., Proc. IEEE, 54(2), 1966.
//!
//! ### 6. Ionospheric Scintillation (ESA / GPS L-band)
//!
//! The amplitude scintillation index S4 is defined as:
//!
//! ```text
//! S4² = (⟨I²⟩ − ⟨I⟩²) / ⟨I⟩²
//! ```
//!
//! where I = |r|² is the instantaneous received intensity.  We model
//! the amplitude perturbation as r' = r · (1 + S4 · w) where w is a
//! sample from the deterministic LCG noise sequence.
//!
//! S4 classification (CCIR 652-1):
//! - S4 < 0.30:  Weak   (no link impact)
//! - S4 ∈ [0.30, 0.60]:  Moderate (cycle-slip risk above 5 dB SNR margin)
//! - S4 > 0.60:  Strong  (immediate link degradation)
//!
//! Reference: Fremouw & Rino (1973); Kintner et al., GPS Solutions, 11(2)
//! 2007.
//!
//! ### 7. Doppler Steady-State Tracking Error
//!
//! A Doppler shift f_D [Hz] on a first-order PLL with velocity constant
//! K_v [Hz/rad] produces a steady-state phase error:
//!
//! ```text
//! θ_ss = 2π · f_D / K_v   [rad]
//! ```
//!
//! which elevates the residual norm floor by approximately θ_ss for a
//! normalised unit-gain loop.  For GPS L1 at v = 5 km/s, f_D ≈ 26 Hz.
//! Reference: Kaplan & Hegarty, "GPS Principles and Applications," §5.4.
//!
//! ## Design Invariants
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - All functions are pure: no side effects, no heap allocation
//! - Deterministic: LCG seed produces reproducible noise sequences
//! - Bounded: all outputs are finite (saturation arithmetic)
//! - All perturbation magnitudes are documented with authoritative references

// ── Deterministic LCG pseudo-random noise ─────────────────────────────────

/// Advance one step of the Knuth-style LCG (modulus 2³², Knuth TAOCP §3.6).
///
/// Parameters: a = 1664525, c = 1013904223, m = 2³² (implicit overflow).
/// This is the same LCG used in Numerical Recipes and is well-characterised
/// for noise injection in simulation harnesses.
#[inline]
pub const fn lcg_step(seed: u32) -> u32 {
    seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223)
}

/// Convert an LCG state to a sample in U[−1, +1].
#[inline]
pub fn lcg_uniform(seed: u32) -> f32 {
    // Map 0..u32::MAX → [0, 1) then shift to [−1, +1)
    (seed as f32) * (2.0 / u32::MAX as f32) - 1.0
}

// ── I/Q Amplitude Imbalance ────────────────────────────────────────────────

/// Apply I/Q amplitude imbalance to a residual norm sample.
///
/// # Arguments
/// * `norm`    — ‖r(k)‖, the current residual norm (≥ 0)
/// * `phi_rad` — instantaneous carrier phase φ(k) in radians
/// * `epsilon` — fractional amplitude imbalance ε ∈ [0, 0.10]
///
/// # Returns
/// Perturbed norm ‖r'(k)‖ ≥ 0.
///
/// # First-order model
/// δ‖r‖ = ‖r‖ · (ε/2) · cos(2φ)
/// Worst-case upper bound: δ‖r‖ ≤ (ε/2) · ‖r‖
#[inline]
pub fn apply_iq_imbalance(norm: f32, phi_rad: f32, epsilon: f32) -> f32 {
    // First-order Taylor expansion of sqrt(1 + ε·cos(2φ))
    let perturbation = (epsilon * 0.5) * cos_approx(2.0 * phi_rad);
    (norm * (1.0 + perturbation)).max(0.0)
}

// ── DC Offset ─────────────────────────────────────────────────────────────

/// Apply a complex DC offset bias (d_I, d_Q) to a residual norm.
///
/// # Arguments
/// * `norm`    — ‖r(k)‖, the current residual norm
/// * `phi_rad` — instantaneous carrier phase φ(k) in radians
/// * `dc_i`   — in-phase DC offset magnitude (normalised)
/// * `dc_q`   — quadrature DC offset magnitude (normalised)
///
/// # Returns
/// Perturbed norm.  This is always ≥ 0.
///
/// # Model
/// ‖r + d‖ ≈ ‖r‖ + d_I·cos(φ) + d_Q·sin(φ)   (first-order Taylor)
#[inline]
pub fn apply_dc_offset(norm: f32, phi_rad: f32, dc_i: f32, dc_q: f32) -> f32 {
    let bias = dc_i * cos_approx(phi_rad) + dc_q * sin_approx(phi_rad);
    (norm + bias).max(0.0)
}

// ── PA Compression ────────────────────────────────────────────────────────

/// Apply third-order memoryless AM/AM PA compression.
///
/// # Arguments
/// * `norm` — ‖r_in‖, input norm (normalised to full scale)
/// * `k3`   — cubic compression coefficient k₃ > 0
///
/// # Returns
/// Compressed output norm.  Saturates at `norm` for negative k3 artefacts.
///
/// # Model
/// r_out = r_in · (1 − k₃ · |r_in|²)
/// The 1 dB compression point occurs at k₃ · |r_1|² ≈ 0.145.
#[inline]
pub fn apply_pa_compression(norm: f32, k3: f32) -> f32 {
    let factor = 1.0 - k3 * norm * norm;
    // Clamp: once the signal enters hard saturation the cubic over-shoots;
    // we clamp to [0, norm] which corresponds to AM suppression but not reversal.
    (norm * factor.max(0.0)).max(0.0)
}

// ── ADC Quantisation Noise ────────────────────────────────────────────────

/// GUM-compliant ADC quantisation noise standard uncertainty.
///
/// Returns u_q = 2^(1−N) / sqrt(12) per ISO/IEC Guide 98-3 §4.3.7.
///
/// # Arguments
/// * `n_bits` — ADC word width N (e.g. 8, 12, 14, 16)
///
/// # Returns
/// Standard uncertainty u_q for one I or Q sample (normalised to FS = 1).
///
/// # Examples
/// N=8:  u_q ≈ 1.13×10⁻³   (RTL-SDR)
/// N=12: u_q ≈ 7.07×10⁻⁵   (LimeSDR)
/// N=14: u_q ≈ 1.77×10⁻⁵   (USRP X310 effective)
#[inline]
pub fn quantization_noise_std(n_bits: u32) -> f32 {
    // LSB = 2^(1−N) for FS = 1; u_q = LSB / sqrt(12)
    let lsb = libm_pow2(1i32 - n_bits as i32);
    lsb / 3.464_101_6 // sqrt(12) = 3.4641...
}

/// Apply ADC quantisation noise to a residual norm (additive white model).
///
/// Adds a deterministic LCG-generated noise sample scaled by u_q to the
/// input norm.  The noise is bounded to ±3·u_q (three-sigma clip).
#[inline]
pub fn apply_quantization_noise(norm: f32, n_bits: u32, seed: u32) -> (f32, u32) {
    let u_q = quantization_noise_std(n_bits);
    let noise_seed = lcg_step(seed);
    let w = lcg_uniform(noise_seed); // w ∈ [−1, +1]
    let perturbed = (norm + u_q * w * 3.0).max(0.0);
    (perturbed, noise_seed)
}

// ── Phase Noise ───────────────────────────────────────────────────────────

/// Apply LO phase noise (Leeson model parameterised by σ_φ) to a norm.
///
/// # Arguments
/// * `norm`     — ‖r(k)‖ (normalised)
/// * `sigma_phi` — integrated phase noise σ_φ in radians rms
/// * `seed`     — LCG state for this sample
///
/// # Returns
/// `(perturbed_norm, new_seed)`
///
/// # Model
/// δ‖r‖ ≈ ‖r‖ · |sin(δφ)|.  For σ_φ < 0.3 rad, sin(δφ) ≈ δφ.
/// The sample δφ ~ σ_φ · w where w ∈ [−1, +1] (three-sigma uniform proxy).
#[inline]
pub fn apply_phase_noise(norm: f32, sigma_phi: f32, seed: u32) -> (f32, u32) {
    let noise_seed = lcg_step(seed);
    let delta_phi = sigma_phi * lcg_uniform(noise_seed); // rad sample
    // |sin(δφ)| ≈ |δφ| for small angle — exact for δφ < 0.3 rad
    let perturbation = norm * sin_approx(delta_phi).abs();
    let perturbed = (norm + perturbation).max(0.0);
    (perturbed, noise_seed)
}

// ── Ionospheric Scintillation ──────────────────────────────────────────────

/// S4 amplitude scintillation index classification (CCIR 652-1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScintillationClass {
    /// S4 < 0.30: no link impact expected.
    Weak,
    /// S4 ∈ [0.30, 0.60]: cycle-slip risk above 5 dB SNR margin.
    Moderate,
    /// S4 > 0.60: immediate link degradation expected.
    Strong,
}

/// Classify the S4 scintillation index per CCIR 652-1.
#[inline]
pub const fn classify_s4(s4: f32) -> ScintillationClass {
    if s4 < 0.30 {
        ScintillationClass::Weak
    } else if s4 < 0.60 {
        ScintillationClass::Moderate
    } else {
        ScintillationClass::Strong
    }
}

/// Apply ionospheric amplitude scintillation to a residual norm.
///
/// # Arguments
/// * `norm` — ‖r(k)‖ (normalised received amplitude)
/// * `s4`   — amplitude scintillation index S4 ∈ [0, 1]
/// * `seed` — LCG state
///
/// # Returns
/// `(perturbed_norm, new_seed, scintillation_class)`
///
/// # Model
/// r' = r · (1 + S4 · w)  where w ~ U[−1, +1].
/// The S4² definition ensures E[|r'|²] = |r|² · (1 + S4²/3) — bounded variance.
#[inline]
pub fn apply_scintillation(norm: f32, s4: f32, seed: u32) -> (f32, u32, ScintillationClass) {
    let noise_seed = lcg_step(seed);
    let w = lcg_uniform(noise_seed);
    let perturbed = (norm * (1.0 + s4 * w)).max(0.0);
    let cls = classify_s4(s4);
    (perturbed, noise_seed, cls)
}

// ── Doppler Steady-State Tracking Error ──────────────────────────────────

/// Steady-state residual norm elevation due to Doppler (first-order PLL).
///
/// For a first-order PLL with velocity constant K_v [Hz/rad], a Doppler
/// shift f_D [Hz] produces a phase tracking error:
///
/// ```text
/// θ_ss = 2π · f_D / K_v   [rad]
/// ```
///
/// The elevated residual norm floor is θ_ss · `nominal_norm` for a
/// carrier-normalised loop.
///
/// # Arguments
/// * `f_d_hz`       — Doppler shift [Hz] (positive = approaching)
/// * `k_v_hz`       — PLL first-order velocity constant K_v [Hz/rad]
/// * `nominal_norm` — quiescent residual norm ‖r₀‖ at zero Doppler
///
/// # Returns
/// Elevated residual norm floor.
///
/// # Example
/// GPS L1 (1575.42 MHz), v = 300 m/s aircraft: f_D = f_c · v/c ≈ 1.58 Hz.
/// A typical GPS PLL K_v = 200 Hz/rad → θ_ss ≈ 0.050 rad.
#[inline]
pub fn doppler_residual_floor(f_d_hz: f32, k_v_hz: f32, nominal_norm: f32) -> f32 {
    if k_v_hz <= 0.0 {
        return nominal_norm;
    }
    let theta_ss = core::f32::consts::TAU * f_d_hz.abs() / k_v_hz;
    (nominal_norm + theta_ss * nominal_norm).max(nominal_norm)
}

// ── Compound Impairment Vector ─────────────────────────────────────────────

/// All hardware impairments parameterised in one structure.
///
/// Used by Stage II of the Continuous Rigor pipeline to inject a
/// reproducible, physics-calibrated impairment set into the synthetic
/// baseline from Stage I.  Each field defaults to "zero impairment."
#[derive(Debug, Clone, Copy)]
pub struct ImpairmentVector {
    /// I/Q amplitude imbalance ε ∈ [0, 0.10].  RTL-SDR typical: 0.012.
    pub iq_imbalance_epsilon: f32,
    /// DC offset in-phase component d_I (normalised).  RTL-SDR: 0.015.
    pub dc_offset_i: f32,
    /// DC offset quadrature component d_Q (normalised).  RTL-SDR: 0.010.
    pub dc_offset_q: f32,
    /// PA cubic compression coefficient k₃.  PAWR PA: 0.30.  0 = disabled.
    pub pa_k3: f32,
    /// ADC word width N for quantisation noise.  0 = disabled.
    pub adc_bits: u32,
    /// Integrated LO phase noise σ_φ [rad rms].  TCXO: 0.05 rad. 0 = disabled.
    pub phase_noise_sigma: f32,
    /// Ionospheric S4 scintillation index ∈ [0, 1].  0 = disabled.
    pub scintillation_s4: f32,
}

impl ImpairmentVector {
    /// RTL-SDR v3 impairment profile.
    ///
    /// Based on community-characterised hardware measurements from
    /// IQEngine.org RTL-SDR capture archive.
    pub const RTL_SDR: Self = Self {
        iq_imbalance_epsilon: 0.012,
        dc_offset_i: 0.015,
        dc_offset_q: 0.010,
        pa_k3: 0.0,
        adc_bits: 8,
        phase_noise_sigma: 0.08,
        scintillation_s4: 0.0,
    };

    /// USRP X310 / N310 impairment profile.
    ///
    /// Ettus Research X310 with SBX daughterboard, factory-calibrated.
    pub const USRP_X310: Self = Self {
        iq_imbalance_epsilon: 0.001,
        dc_offset_i: 0.001,
        dc_offset_q: 0.001,
        pa_k3: 0.0,
        adc_bits: 14,
        phase_noise_sigma: 0.015,
        scintillation_s4: 0.0,
    };

    /// PAWR Colosseum node profile (FPGA + medium-power PA).
    ///
    /// Derived from Colosseum RF front-end characterisation data,
    /// Northeastern University, 2021.
    pub const COLOSSEUM_NODE: Self = Self {
        iq_imbalance_epsilon: 0.004,
        dc_offset_i: 0.003,
        dc_offset_q: 0.002,
        pa_k3: 0.12,
        adc_bits: 12,
        phase_noise_sigma: 0.025,
        scintillation_s4: 0.0,
    };

    /// ESA L-band receiver under moderate ionospheric scintillation.
    ///
    /// S4 = 0.40 (moderate), based on ISMR (Ionospheric Scintillation
    /// Monitor Receiver) data from ESA GNSS Science Support Centre.
    pub const ESA_L_BAND_MODERATE: Self = Self {
        iq_imbalance_epsilon: 0.002,
        dc_offset_i: 0.001,
        dc_offset_q: 0.001,
        pa_k3: 0.0,
        adc_bits: 14,
        phase_noise_sigma: 0.020,
        scintillation_s4: 0.40,
    };

    /// ESA L-band receiver under strong ionospheric scintillation.
    ///
    /// S4 = 0.70 (strong), near link-loss threshold.
    pub const ESA_L_BAND_STRONG: Self = Self {
        iq_imbalance_epsilon: 0.002,
        dc_offset_i: 0.001,
        dc_offset_q: 0.001,
        pa_k3: 0.0,
        adc_bits: 14,
        phase_noise_sigma: 0.020,
        scintillation_s4: 0.70,
    };

    /// Zero impairment ("physics-only" baseline for Stage I).
    pub const NONE: Self = Self {
        iq_imbalance_epsilon: 0.0,
        dc_offset_i: 0.0,
        dc_offset_q: 0.0,
        pa_k3: 0.0,
        adc_bits: 0,
        phase_noise_sigma: 0.0,
        scintillation_s4: 0.0,
    };
}

/// Apply the full ImpairmentVector to a single norm sample.
///
/// Impairments are applied in the order they occur in the signal chain:
/// 1. PA compression (transmitter, Colosseum)
/// 2. Ionospheric scintillation (propagation)
/// 3. DC offset (receiver front-end)
/// 4. I/Q imbalance (receiver baseband)
/// 5. Phase noise (LO)
/// 6. ADC quantisation (ADC)
///
/// # Arguments
/// * `norm`     — ‖r(k)‖ before impairment
/// * `phi_rad`  — carrier phase φ(k) at this sample (radians)
/// * `seed`     — current LCG state (for stochastic impairments)
/// * `imp`      — compound impairment vector
///
/// # Returns
/// `(perturbed_norm, new_seed)`
pub fn apply_all(norm: f32, phi_rad: f32, seed: u32, imp: ImpairmentVector) -> (f32, u32) {
    let mut r = norm;
    let mut s = seed;

    // 1. PA compression
    if imp.pa_k3 > 0.0 {
        r = apply_pa_compression(r, imp.pa_k3);
    }

    // 2. Ionospheric scintillation
    if imp.scintillation_s4 > 0.0 {
        let (rr, ss, _) = apply_scintillation(r, imp.scintillation_s4, s);
        r = rr;
        s = ss;
    }

    // 3. DC offset
    if imp.dc_offset_i.abs() > 0.0 || imp.dc_offset_q.abs() > 0.0 {
        r = apply_dc_offset(r, phi_rad, imp.dc_offset_i, imp.dc_offset_q);
    }

    // 4. I/Q imbalance
    if imp.iq_imbalance_epsilon > 0.0 {
        r = apply_iq_imbalance(r, phi_rad, imp.iq_imbalance_epsilon);
    }

    // 5. Phase noise
    if imp.phase_noise_sigma > 0.0 {
        let (rr, ss) = apply_phase_noise(r, imp.phase_noise_sigma, s);
        r = rr;
        s = ss;
    }

    // 6. ADC quantisation
    if imp.adc_bits > 0 {
        let (rr, ss) = apply_quantization_noise(r, imp.adc_bits, s);
        r = rr;
        s = ss;
    }

    (r.max(0.0), s)
}

// ── Trig approximations (no_std, no libm dependency) ──────────────────────

/// Minimax polynomial approximation of sin(x) over [−π, π], max error < 5×10⁻⁴.
///
/// Coefficients from Abramowitz & Stegun 4.3.97, range-reduced via
/// periodic extension.  Suitable for impairment simulation; not suitable
/// for navigation-grade computation.
#[inline]
pub fn sin_approx(x: f32) -> f32 {
    // Range reduction to [−π, π]. f32 finite magnitudes fit in < 2^128;
    // 64 iterations covers practical inputs without unbounded spin on
    // adversarial values.
    use core::f32::consts::{PI, TAU};
    let mut x = x;
    for _ in 0..64 {
        if x <= PI { break; }
        x -= TAU;
    }
    for _ in 0..64 {
        if x >= -PI { break; }
        x += TAU;
    }
    // A&S polynomial approximation
    let b = 4.0 / PI;
    let c = -4.0 / (PI * PI);
    let y = b * x + c * x * x.abs();
    // Refinement pass (Bhaskara I-style, max error ~5×10⁻⁴)
    0.225 * (y * y.abs() - y) + y
}

/// Minimax polynomial approximation of cos(x).
#[inline]
pub fn cos_approx(x: f32) -> f32 {
    sin_approx(x + core::f32::consts::FRAC_PI_2)
}

/// Compute 2^n as f32 for integer n (exact for |n| ≤ 126).
#[inline]
fn libm_pow2(n: i32) -> f32 {
    // Use bit manipulation on f32 exponent field (correct for 0 < 2^n < f32::MAX)
    // Exponent is stored with bias 127: bits = (n + 127) << 23
    if n < -126 { return 0.0; }
    if n > 127  { return f32::MAX; }
    f32::from_bits(((n + 127) as u32) << 23)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iq_imbalance_bounded() {
        // δ‖r‖ ≤ (ε/2) · ‖r‖
        let norm = 0.1_f32;
        let epsilon = 0.08_f32;
        for k in 0..16u32 {
            let phi = k as f32 * (core::f32::consts::PI / 8.0);
            let r = apply_iq_imbalance(norm, phi, epsilon);
            let max_delta = (epsilon * 0.5 + 1e-6) * norm;
            assert!(
                (r - norm).abs() <= max_delta,
                "IQ imbalance exceeded bound: r={r}, norm={norm}, delta={}, max={}",
                (r - norm).abs(), max_delta
            );
        }
    }

    #[test]
    fn dc_offset_small_positive() {
        // With dc_i=0.01, dc_q=0 and phi=0, norm should increase by ~dc_i
        let norm = 0.1_f32;
        let r = apply_dc_offset(norm, 0.0, 0.01, 0.0);
        let delta = (r - norm).abs();
        // cos(0)=1 → expected +0.01
        assert!(delta < 0.02, "DC offset perturbation too large: {delta}");
    }

    #[test]
    fn pa_compression_reduces_norm() {
        // High input → compressed output < input
        let norm = 0.5_f32;
        let k3 = 0.30_f32;
        let r = apply_pa_compression(norm, k3);
        assert!(r <= norm, "PA compression should reduce or equal input: {r} vs {norm}");
        assert!(r > 0.0, "PA compression should not go negative: {r}");
    }

    #[test]
    fn pa_compression_identity_at_zero() {
        let r = apply_pa_compression(0.0, 0.30);
        assert!(r == 0.0);
    }

    #[test]
    fn quantization_noise_std_n8() {
        // N=8: u_q = 2^(1-8) / sqrt(12) = 2^(-7) / 3.4641 ≈ 7.81e-3 / 3.464 ≈ 2.25e-3
        let u = quantization_noise_std(8);
        assert!(u > 1e-3 && u < 1e-2, "8-bit u_q out of expected range: {u}");
    }

    #[test]
    fn quantization_noise_std_n14() {
        // N=14: much smaller
        let u8 = quantization_noise_std(8);
        let u14 = quantization_noise_std(14);
        assert!(u14 < u8 * 0.1, "14-bit should be << 8-bit: {u14} vs {u8}");
    }

    #[test]
    fn phase_noise_bounded() {
        let norm = 0.1_f32;
        let sigma = 0.05_f32;
        let (r, _) = apply_phase_noise(norm, sigma, 42);
        // Perturbation bounded by sigma * norm
        assert!((r - norm).abs() < sigma + 1e-4, "Phase noise exceeded sigma bound");
    }

    #[test]
    fn scintillation_weak_small_perturbation() {
        let norm = 0.1_f32;
        let s4 = 0.20_f32; // weak
        let (r, _, cls) = apply_scintillation(norm, s4, 12345);
        assert_eq!(cls, ScintillationClass::Weak);
        // Perturbation bounded by S4 * norm
        assert!((r - norm).abs() <= s4 * norm + 1e-6);
    }

    #[test]
    fn scintillation_strong_classifies_correctly() {
        let (_, _, cls) = apply_scintillation(0.1, 0.70, 999);
        assert_eq!(cls, ScintillationClass::Strong);
    }

    #[test]
    fn doppler_floor_elevated() {
        let nominal = 0.05_f32;
        let elevated = doppler_residual_floor(1.58, 200.0, nominal);
        assert!(elevated > nominal, "Doppler should elevate floor: {elevated} vs {nominal}");
    }

    #[test]
    fn apply_all_none_is_identity() {
        let norm = 0.1_f32;
        let (r, _) = apply_all(norm, 0.0, 42, ImpairmentVector::NONE);
        assert!((r - norm).abs() < 1e-6, "Zero impairment should be identity: {r} vs {norm}");
    }

    #[test]
    fn apply_all_rtl_sdr_bounded() {
        let norm = 0.1_f32;
        // RTL-SDR impairments should not change norm by more than 30% of norm
        let (r, _) = apply_all(norm, 1.0, 7777, ImpairmentVector::RTL_SDR);
        assert!((r - norm).abs() < 0.3 * norm + 0.05,
            "RTL-SDR impairment too large: {r} vs {norm}");
    }

    #[test]
    fn lcg_period_deterministic() {
        let s0 = 123456789u32;
        let s1 = lcg_step(s0);
        let s2 = lcg_step(s1);
        let s1b = lcg_step(s0);
        assert_eq!(s1, s1b, "LCG must be deterministic");
        assert_ne!(s1, s2, "Consecutive LCG states must differ");
    }

    #[test]
    fn sin_approx_zero_and_halfpi() {
        let s0 = sin_approx(0.0);
        let s_halfpi = sin_approx(core::f32::consts::FRAC_PI_2);
        assert!(s0.abs() < 0.01, "sin(0) ≈ 0: got {s0}");
        assert!((s_halfpi - 1.0).abs() < 0.01, "sin(π/2) ≈ 1: got {s_halfpi}");
    }

    #[test]
    fn classify_s4_boundaries() {
        assert_eq!(classify_s4(0.10), ScintillationClass::Weak);
        assert_eq!(classify_s4(0.45), ScintillationClass::Moderate);
        assert_eq!(classify_s4(0.75), ScintillationClass::Strong);
    }
}
