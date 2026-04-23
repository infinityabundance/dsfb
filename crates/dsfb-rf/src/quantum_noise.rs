//! Quantum-Limited Observation: Standard Quantum Limit noise model.
//!
//! ## Vision
//!
//! The next generation of RF receivers will use **Rydberg atom quantum
//! sensors** (Shaffer et al. 2018; Simons et al. 2021) instead of
//! traditional silicon ADC chains. These sensors approach the
//! **Standard Quantum Limit** (SQL) — the irreducible noise floor imposed by
//! the Heisenberg Uncertainty Principle on simultaneous amplitude-phase
//! measurement.
//!
//! DSFB-RF is the first structural semiotic engine calibrated to the SQL.
//! This module provides:
//!
//! 1. **Heisenberg Uncertainty boundary** for joint amplitude-phase observation
//!    at a given carrier frequency and photon occupation number.
//! 2. **Shot noise variance** for a quantum receiver at signal power P.
//! 3. **Thermal squeezing factor** — the ratio of thermal Johnson-Nyquist noise
//!    to the SQL, which collapses to 1 only at cryogenic temperatures.
//! 4. **`QuantumNoiseTwin`** — a digital twin that models the residual noise
//!    floor of a Rydberg/quantum receiver so that the DSFB admissibility
//!    envelope can be calibrated to the true physical observability limit.
//!
//! ## Standard Quantum Limit Background
//!
//! For a coherent-state (classical-quantum) receiver measuring a field
//! of mean photon number $\bar{n}$, the SQL on amplitude estimation is:
//!
//! $$\sigma_{SQL}^2 = \frac{\hbar \omega}{2}$$
//!
//! (half a photon of energy per quadrature). For a signal power P at
//! frequency ω:
//!
//! $$\bar{n} = \frac{P}{\hbar \omega B}$$
//!
//! where B is bandwidth. **Shot noise power** (the irreducible quantum
//! noise at the SQL):
//!
//! $$P_{shot} = \hbar \omega B$$
//!
//! **Thermal noise** (Johnson-Nyquist) at temperature T:
//!
//! $$P_{th} = k_B T B$$
//!
//! The **Quantum-to-Thermal ratio** determines whether a receiver is in the
//! quantum or classical regime:
//!
//! $$R_{QT} = \frac{\hbar\omega}{k_B T}$$
//!
//! At room temperature and 10 GHz: $R_{QT} \approx 1.6 \times 10^{-3}$
//! (classical regime). At 10 mK (dilution refrigerator): $R_{QT} \approx 48$
//! (deep quantum regime).
//!
//! ## Non-Claims
//!
//! 1. Current deployed RF receivers (silicon ADC chains, SDRs) operate
//!    **far above** the SQL. The SQL noise floor is a future architectural
//!    reference point, not a current measurement claim.
//! 2. Rydberg atom receivers are at TRL 3-4 for narrowband laboratory
//!    operation (Simons et al. 2021). Broadband deployment remains research.
//! 3. This module provides the calibration framework; actual quantum sensor
//!    integration requires Phase II hardware access.
//!
//! ## no_std / no_alloc / zero-unsafe
//!
//! All arithmetic is closed-form `f32`.  Uses `crate::math::sqrt_f32`.
//!
//! ## References
//!
//! - Caves (1981), "Quantum-mechanical noise in an interferometer", PRA.
//! - Shaffer, Pfau & Löw (2018), "Light-atom interfaces in atomic ensembles".
//! - Simons et al. (2021), "Rydberg atom-based field sensing", IEEE AP-S.
//! - Gardiner & Zoller (2004), Quantum Noise, Springer.

// ── Physical Constants ─────────────────────────────────────────────────────

/// Reduced Planck constant ħ (J·s).
pub const H_BAR: f32 = 1.054_571_817e-34_f32;

/// Boltzmann constant k_B (J/K).
pub const K_B: f32 = 1.380_649e-23_f32;

/// 2π
const TWO_PI: f32 = 2.0 * core::f32::consts::PI;

// ── SQL Noise Computation ──────────────────────────────────────────────────

/// Shot noise power at the Standard Quantum Limit (W).
///
/// $P_{shot} = \hbar \omega B = h f B$
///
/// This is the irreducible quantum noise floor for a coherent-state receiver
/// in bandwidth B at carrier frequency f_hz.
#[inline]
pub fn shot_noise_power_w(f_hz: f32, bandwidth_hz: f32) -> f32 {
    H_BAR * TWO_PI * f_hz * bandwidth_hz
}

/// Shot noise variance (amplitude quadrature) at the SQL.
///
/// For a unit-impedance receiver: $\sigma_{shot}^2 = P_{shot}$.
#[inline]
pub fn shot_noise_variance(f_hz: f32, bandwidth_hz: f32) -> f32 {
    shot_noise_power_w(f_hz, bandwidth_hz)
}

/// Johnson-Nyquist thermal noise power (W).
///
/// $P_{th} = k_B T B$
#[inline]
pub fn thermal_noise_power_w(temp_k: f32, bandwidth_hz: f32) -> f32 {
    K_B * temp_k * bandwidth_hz
}

/// Quantum-to-Thermal ratio $R_{QT} = \hbar\omega / (k_B T)$.
///
/// - $R_{QT} \ll 1$: classical thermal regime (room temperature, GHz)
/// - $R_{QT} \gg 1$: quantum shot-noise regime (cryogenic, optical)
#[inline]
pub fn quantum_to_thermal_ratio(f_hz: f32, temp_k: f32) -> f32 {
    (H_BAR * TWO_PI * f_hz) / (K_B * temp_k.max(1e-3))
}

/// Mean photon occupation number at thermal equilibrium (Bose-Einstein).
///
/// $\bar{n}_{th} = 1 / (\exp(R_{QT}) - 1) \approx k_B T / \hbar\omega$
/// in the classical limit.
pub fn thermal_photon_number(f_hz: f32, temp_k: f32) -> f32 {
    let r = quantum_to_thermal_ratio(f_hz, temp_k);
    if r > 50.0 {
        // Deep quantum: photon number ≈ 0
        crate::math::exp_f32(-r)
    } else if r < 0.01 {
        // Classical: n_th ≈ 1/r = kT/(ħω)
        1.0 / r
    } else {
        // Bose-Einstein exact approximation
        1.0 / (crate::math::exp_f32(r) - 1.0).max(1e-9)
    }
}

// ── SQL Admissibility Calibration ─────────────────────────────────────────

/// The SQL noise floor expressed as a fraction of the thermal noise floor.
///
/// $F_{SQL} = R_{QT}$ — when this is close to 1, the receiver is near the
/// SQL. When $F_{SQL} \ll 1$ the receiver is in the deep thermal regime
/// and the SQL is irrelevant.
#[inline]
pub fn sql_fraction_of_thermal(f_hz: f32, temp_k: f32) -> f32 {
    quantum_to_thermal_ratio(f_hz, temp_k)
}

// ── Quantum Noise Digital Twin ─────────────────────────────────────────────

/// Parameterisation of a quantum-receiver noise floor.
///
/// Used to calibrate the DSFB admissibility envelope to the physical
/// observability limit of a Rydberg or cryogenic quantum sensor.
#[derive(Debug, Clone, Copy)]
pub struct QuantumNoiseTwin {
    /// Carrier frequency (Hz).
    pub carrier_hz:        f32,
    /// Measurement bandwidth (Hz).
    pub bandwidth_hz:      f32,
    /// Physical temperature of the receiver front-end (K).
    pub temp_k:            f32,
    /// Squeezing parameter r_sq (0 = coherent state SQL; > 0 = squeezed).
    /// Quadrature squeezing reduces shot noise by e^{-2r} below SQL.
    pub squeezing_r:       f32,
    /// Shot noise power at the SQL (W).
    pub shot_noise_w:      f32,
    /// Thermal noise power (W).
    pub thermal_noise_w:   f32,
    /// Quantum-to-Thermal ratio R_QT.
    pub r_qt:              f32,
    /// Effective noise floor of this receiver (W): min of thermal and
    /// potentially squeezed shot noise.
    pub effective_floor_w: f32,
    /// Receiver regime classification.
    pub regime:            ReceiverRegime,
}

/// Classification of receiver noise regime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverRegime {
    /// Thermal noise dominates: R_QT < 0.01. Classical silicon ADC or SDR.
    DeepThermal,
    /// Thermal and quantum noise comparable: 0.01 ≤ R_QT < 1.0.
    /// Cryogenic HEMT amplifier, ultra-low-noise LNA.
    TransitionRegime,
    /// Quantum (shot) noise dominates: R_QT ≥ 1.0.
    /// Rydberg atom sensor, dilution refrigerator frontend.
    QuantumLimited,
    /// Squeezed-state receiver: below the SQL by squeezing factor e^{-2r}.
    /// Research-grade only. TRL ≤ 2 for RF applications as of 2026.
    BelowSQL,
}

#[inline]
fn squeeze_factor(squeezing_r: f32) -> f32 {
    if squeezing_r > 0.0 {
        crate::math::exp_f32(-2.0 * squeezing_r)
    } else {
        1.0
    }
}

#[inline]
fn effective_floor_w(squeezing_r: f32, squeezed_shot: f32, thermal_noise_w: f32, r_qt: f32) -> f32 {
    if squeezing_r > 0.0 && squeezed_shot < thermal_noise_w {
        squeezed_shot
    } else if r_qt >= 0.1 {
        squeezed_shot
    } else {
        thermal_noise_w
    }
}

#[inline]
fn classify_regime(squeezing_r: f32, squeezed_shot: f32, shot_noise_w: f32, r_qt: f32) -> ReceiverRegime {
    if squeezing_r > 0.0 && squeezed_shot < shot_noise_w * 0.9 {
        ReceiverRegime::BelowSQL
    } else if r_qt >= 1.0 {
        ReceiverRegime::QuantumLimited
    } else if r_qt >= 0.01 {
        ReceiverRegime::TransitionRegime
    } else {
        ReceiverRegime::DeepThermal
    }
}

impl QuantumNoiseTwin {
    /// Construct a quantum noise digital twin for the given receiver parameters.
    ///
    /// # Arguments
    /// - `carrier_hz`   — carrier frequency (Hz)
    /// - `bandwidth_hz` — measurement bandwidth (Hz)
    /// - `temp_k`       — front-end physical temperature (K)
    /// - `squeezing_r`  — quadrature squeezing parameter r
    ///   (0.0 for coherent state / SQL; use 0.0 for all current deployments)
    pub fn new(
        carrier_hz:   f32,
        bandwidth_hz: f32,
        temp_k:       f32,
        squeezing_r:  f32,
    ) -> Self {
        debug_assert!(carrier_hz > 0.0, "carrier_hz must be positive");
        debug_assert!(bandwidth_hz > 0.0, "bandwidth_hz must be positive");
        debug_assert!(temp_k > 0.0, "temp_k must be positive (Kelvin)");
        debug_assert!(squeezing_r >= 0.0, "squeezing_r must be non-negative");
        let shot_noise_w    = shot_noise_power_w(carrier_hz, bandwidth_hz);
        let thermal_noise_w = thermal_noise_power_w(temp_k, bandwidth_hz);
        let r_qt            = quantum_to_thermal_ratio(carrier_hz, temp_k);
        let squeeze_factor  = squeeze_factor(squeezing_r);
        let squeezed_shot   = shot_noise_w * squeeze_factor;
        let effective_floor_w = effective_floor_w(squeezing_r, squeezed_shot, thermal_noise_w, r_qt);
        let regime = classify_regime(squeezing_r, squeezed_shot, shot_noise_w, r_qt);
        Self {
            carrier_hz, bandwidth_hz, temp_k, squeezing_r,
            shot_noise_w, thermal_noise_w, r_qt,
            effective_floor_w, regime,
        }
    }

    /// `σ²_floor` for DSFB calibration: effective noise floor variance.
    ///
    /// Use this as the `expected_sigma` parameter in
    /// `calibration::check_calibration_window` for quantum receivers.
    #[inline]
    pub fn sigma_sq_floor(&self) -> f32 {
        self.effective_floor_w
    }

    /// How many times the floor exceeds the pure SQL (1.0 = exactly at SQL).
    ///
    /// Values > 1.0 mean the receiver is above the SQL (typical for thermal).
    #[inline]
    pub fn sql_margin(&self) -> f32 {
        self.effective_floor_w / self.shot_noise_w.max(1e-30)
    }

    /// Non-claim text for operator display.
    pub const DISCLAIMER: &'static str =
        "Quantum noise model is a physical reference bound. \
         All current SDR/FPGA receivers operate at the DeepThermal regime \
         (R_QT << 1). QuantumLimited regime requires cryogenic hardware \
         not available in Phase I. Provided as calibration framework.";
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_noise_increases_with_frequency() {
        let p_uhf = shot_noise_power_w(435e6, 1e6);
        let p_xband = shot_noise_power_w(10e9, 1e6);
        assert!(p_xband > p_uhf, "higher carrier must give higher shot noise");
    }

    #[test]
    fn thermal_noise_room_temp_1mhz() {
        // kT·B at 290 K, 1 MHz ≈ 4.0e-15 W
        let p = thermal_noise_power_w(290.0, 1e6);
        assert!(p > 3.5e-15 && p < 4.5e-15, "thermal: {:.3e}", p);
    }

    #[test]
    fn rqt_deep_thermal_at_room_temp_ghz() {
        let r = quantum_to_thermal_ratio(10e9, 290.0);
        assert!(r < 0.01, "10 GHz at 290K is deep thermal: R_QT = {:.2e}", r);
    }

    #[test]
    fn rqt_quantum_limited_at_cryo() {
        // At 10 mK and 10 GHz, R_QT >> 1
        let r = quantum_to_thermal_ratio(10e9, 0.010);
        assert!(r > 1.0, "10 GHz at 10 mK must be quantum-limited: R_QT = {:.2}", r);
    }

    #[test]
    fn quantum_noise_twin_deep_thermal_regime() {
        let twin = QuantumNoiseTwin::new(10e9, 1e6, 290.0, 0.0);
        assert_eq!(twin.regime, ReceiverRegime::DeepThermal,
            "10 GHz / 290 K must be DeepThermal");
        assert!(twin.sql_margin() > 100.0,
            "room-temp receiver is far above SQL: {:.2}", twin.sql_margin());
    }

    #[test]
    fn quantum_noise_twin_quantum_limited_at_cryo() {
        let twin = QuantumNoiseTwin::new(10e9, 1e6, 0.01, 0.0);
        assert!(
            matches!(twin.regime, ReceiverRegime::QuantumLimited | ReceiverRegime::TransitionRegime),
            "10 GHz / 10 mK: {:?}", twin.regime,
        );
    }

    #[test]
    fn squeezing_reduces_shot_noise() {
        let twin_no_sq = QuantumNoiseTwin::new(10e9, 1e6, 0.01, 0.0);
        let twin_sq    = QuantumNoiseTwin::new(10e9, 1e6, 0.01, 2.0); // 4x squeezing
        // Squeezed shot noise = shot * exp(-4) ≈ 0.018 × shot
        assert!(twin_sq.effective_floor_w < twin_no_sq.effective_floor_w,
            "squeezing must reduce noise floor");
    }

    #[test]
    fn thermal_photon_number_classical_limit() {
        // At room temperature, 10 GHz: n_th ≈ kT/(hf) ≈ 600
        let n = thermal_photon_number(10e9, 290.0);
        assert!(n > 100.0 && n < 1e5,
            "10 GHz at 290K: n_th = {:.1}", n);
    }
}
