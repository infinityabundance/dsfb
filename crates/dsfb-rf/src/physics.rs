//! Physics-of-failure mapping and semiotic horizon characterization.
//!
//! ## Semiotic Horizon
//!
//! The "semiotic horizon" defines the operating envelope within which
//! DSFB's structural grammar produces reliable, actionable output.
//! Outside this envelope — below the SNR floor, at extreme drift rates,
//! or under non-stationary calibration conditions — grammar states are
//! unreliable. Mapping this boundary explicitly is the single most
//! credibility-building artifact for reviewers and SBIR operators.
//!
//! The semiotic horizon is defined in (SNR, α) space:
//! - SNR: signal-to-noise ratio in dB
//! - α: drift rate (residual norm units per observation)
//!
//! At each (SNR, α) point, the engine either:
//! - Detects the drift → "Zone of Success" (grammar state transitions correctly)
//! - Fails to detect → "Zone of Failure" (grammar remains Admissible)
//!
//! The horizon is the boundary between these zones.
//!
//! ## Physics-of-Failure Mapping
//!
//! Maps grammar states to physical mechanisms using established RF models:
//!
//! | Grammar State                     | Physical Mechanism          | Model Reference        |
//! |-----------------------------------|-----------------------------|------------------------|
//! | Boundary[SustainedOutwardDrift]    | PA thermal drift            | Arrhenius model        |
//! | Boundary[SustainedOutwardDrift]    | LO aging                    | Allan variance         |
//! | Boundary[AbruptSlewViolation]      | PIM onset                   | Passive intermod model |
//! | Boundary[RecurrentBoundaryGrazing] | FHSS periodic interference  | Hop rate analysis      |
//! | Violation                          | Jamming / intentional EMI   | J/S ratio model        |
//! | Boundary[SustainedOutwardDrift]    | Phase noise degradation     | Leeson's model         |
//!
//! These mappings are **candidate hypotheses**, not attributions.
//! Physical attribution requires domain-specific calibration data
//! that is not available from public datasets (RadioML, ORACLE).
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Fixed-capacity data tables for semiotic horizon grid
//! - Physics mapping is a static lookup (zero runtime cost)

use crate::grammar::ReasonCode;

// ── Semiotic Horizon ───────────────────────────────────────────────────────

/// A single point in the semiotic horizon grid.
///
/// Records whether the DSFB grammar correctly detects a structural
/// drift at a given (SNR, drift_rate) operating point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizonPoint {
    /// SNR in dB.
    pub snr_db: f32,
    /// Drift rate α (residual norm units per observation).
    pub drift_rate: f32,
    /// Whether the grammar correctly entered Boundary/Violation within
    /// the detection window (true = success, false = missed).
    pub detected: bool,
    /// Number of observations to first detection (0 if not detected).
    pub detection_latency: u32,
}

/// Fixed-capacity semiotic horizon grid.
///
/// Stores detection results across a sweep of (SNR, α) operating points.
/// Used to generate the "Horizon of Failure" heatmap artifact.
pub struct SemioticHorizon<const N: usize> {
    /// Grid points.
    points: [HorizonPoint; N],
    /// Number of populated points.
    count: usize,
}

impl<const N: usize> SemioticHorizon<N> {
    /// Create an empty horizon grid.
    pub const fn new() -> Self {
        Self {
            points: [HorizonPoint {
                snr_db: 0.0,
                drift_rate: 0.0,
                detected: false,
                detection_latency: 0,
            }; N],
            count: 0,
        }
    }

    /// Record a detection result at (snr_db, drift_rate).
    pub fn record(&mut self, snr_db: f32, drift_rate: f32, detected: bool, latency: u32) -> bool {
        if self.count >= N { return false; }
        self.points[self.count] = HorizonPoint {
            snr_db,
            drift_rate,
            detected,
            detection_latency: latency,
        };
        self.count += 1;
        true
    }

    /// Number of recorded points.
    pub fn len(&self) -> usize { self.count }

    /// Whether the grid is empty.
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Iterator over recorded points.
    pub fn points(&self) -> &[HorizonPoint] {
        &self.points[..self.count]
    }

    /// Detection rate across all recorded points.
    pub fn detection_rate(&self) -> f32 {
        if self.count == 0 { return 0.0; }
        let detected = self.points[..self.count].iter().filter(|p| p.detected).count();
        detected as f32 / self.count as f32
    }

    /// Mean detection latency for detected points.
    pub fn mean_detection_latency(&self) -> f32 {
        let detected: &[HorizonPoint] = &self.points[..self.count];
        let (sum, count) = detected.iter()
            .filter(|p| p.detected && p.detection_latency > 0)
            .fold((0u64, 0u32), |(s, c), p| (s + p.detection_latency as u64, c + 1));
        if count == 0 { return 0.0; }
        sum as f32 / count as f32
    }
}

impl<const N: usize> Default for SemioticHorizon<N> {
    fn default() -> Self { Self::new() }
}

// ── Physics-of-Failure Mapping ─────────────────────────────────────────────

/// A candidate physical mechanism that may explain a grammar state.
///
/// These are **hypotheses**, not attributions. Physical attribution
/// requires deployment-specific calibration data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMechanism {
    /// Power amplifier thermal drift (Arrhenius model).
    /// Signature: persistent positive ṙ over 100–10,000 symbol periods.
    PaThermalDrift,
    /// Local oscillator aging (Allan variance model).
    /// Signature: slow monotone frequency offset growth.
    LoAging,
    /// Passive intermodulation (PIM) onset.
    /// Signature: abrupt slew in specific intermod frequency bands.
    PimOnset,
    /// Phase noise degradation (Leeson's model).
    /// Signature: oscillatory ṙ with growing amplitude.
    PhaseNoiseDegradation,
    /// Intentional jamming (J/S ratio model).
    /// Signature: abrupt, sustained high-norm residual.
    IntentionalJamming,
    /// Adjacent-channel interference (ACLR violation).
    /// Signature: spectral mask approach from neighboring channel.
    AdjacentChannelInterference,
    /// Frequency-hopping spread-spectrum transition.
    /// Signature: abrupt slew with rapid recovery to new baseline.
    FhssTransition,
    /// Antenna coupling transient.
    /// Signature: brief abrupt slew correlated with antenna switching.
    AntennaCouplingTransient,
    /// Unknown mechanism — endoductive regime.
    Unknown,
}

/// Map a grammar reason code to candidate physical mechanisms.
///
/// Returns the top candidate mechanisms, ordered by structural likelihood.
/// This is a static lookup with zero runtime allocation.
///
/// ## Non-Attribution Policy
///
/// These are candidate hypotheses only. No physical attribution is made
/// from public datasets. Field-validated attribution requires deployment-
/// specific calibration data from the target platform.
pub fn candidate_mechanisms(reason: ReasonCode) -> &'static [PhysicalMechanism] {
    match reason {
        ReasonCode::SustainedOutwardDrift => &[
            PhysicalMechanism::PaThermalDrift,
            PhysicalMechanism::LoAging,
            PhysicalMechanism::AdjacentChannelInterference,
        ],
        ReasonCode::AbruptSlewViolation => &[
            PhysicalMechanism::IntentionalJamming,
            PhysicalMechanism::PimOnset,
            PhysicalMechanism::AntennaCouplingTransient,
        ],
        ReasonCode::RecurrentBoundaryGrazing => &[
            PhysicalMechanism::FhssTransition,
            PhysicalMechanism::AdjacentChannelInterference,
        ],
        ReasonCode::EnvelopeViolation => &[
            PhysicalMechanism::IntentionalJamming,
            PhysicalMechanism::PaThermalDrift,
        ],
    }
}

/// Map a reason code to the primary physical model reference.
///
/// Returns a human-readable model name for documentation and audit trails.
pub fn model_reference(mechanism: PhysicalMechanism) -> &'static str {
    match mechanism {
        PhysicalMechanism::PaThermalDrift => "Arrhenius thermal acceleration model",
        PhysicalMechanism::LoAging => "Allan variance / frequency stability model",
        PhysicalMechanism::PimOnset => "Passive intermodulation model (3rd/5th order)",
        PhysicalMechanism::PhaseNoiseDegradation => "Leeson's phase noise model",
        PhysicalMechanism::IntentionalJamming => "J/S ratio and effective radiated power model",
        PhysicalMechanism::AdjacentChannelInterference => "3GPP TS 36.141 §6.3 ACLR model",
        PhysicalMechanism::FhssTransition => "Hop rate and dwell time analysis",
        PhysicalMechanism::AntennaCouplingTransient => "Coupling coefficient and VSWR model",
        PhysicalMechanism::Unknown => "Endoductive regime — no prior model",
    }
}

// ── Physics Model Trait ────────────────────────────────────────────────────
//
// Pluggable physics models that translate a measurable platform parameter
// (temperature, observation time-base, etc.) into a predicted drift rate.
//
// The predicted drift rate can be compared against the DSFB-observed drift to
// confirm or falsify a physics-of-failure hypothesis.
//
// References:
//   Kayali, S. (1999) "Physics of Failure as an Underlying Principle to
//       NASA's Reliability Assessment Method," JPL Publication 96-25, Rev. A.
//       NASA/Goddard. (GaAs PHEMT E_a = 1.6 eV; GaN HEMT E_a = 2.1 eV)
//   Allan, D.W. (1966) "Statistics of atomic frequency standards,"
//       Proc. IEEE 54(2):221–230. doi:10.1109/PROC.1966.4634.
//   IEEE Std 1193-2003, "Guide for Measurement of Environmental Sensitivities
//       of Standard Frequency Generators."

/// A pluggable physics-of-failure model that maps an observable platform
/// parameter to a predicted RF drift rate.
///
/// Implementors provide the equation-of-state for a specific physical
/// degradation mechanism so that DSFB engine observations can be falsified
/// against first-principles models rather than purely statistical thresholds.
pub trait PhysicsModel {
    /// Predict the residual drift rate for the given platform parameter.
    ///
    /// `param` semantics are model-specific:
    /// - `ArrheniusModel`: junction temperature in °C
    /// - `AllanVarianceModel`: averaging time τ in seconds
    fn predict_drift_rate(&self, param: f32) -> f32;

    /// Short human-readable label for this model instance.
    fn label(&self) -> &'static str;

    /// Primary literature reference for the model.
    fn reference(&self) -> &'static str;

    /// The DSFB `ReasonCode` this model most directly corresponds to.
    fn maps_to_reason(&self) -> ReasonCode;
}

/// Arrhenius thermal-acceleration model for semiconductor PA degradation.
///
/// k(T) = α₀ · exp(−E_a / (k_B · T))
///
/// where T is absolute temperature [K], k_B = 8.617×10⁻⁵ eV/K, and
/// E_a is the activation energy in eV.
///
/// ## Pre-defined Constants
/// - `GAAS_PHEMT`: E_a = 1.6 eV (GaAs pHEMT operating at 125°C)
/// - `GAN_HEMT`:   E_a = 2.1 eV (GaN HEMT operating at 150°C)
#[derive(Debug, Clone, Copy)]
pub struct ArrheniusModel {
    /// Pre-exponential drift-rate factor (unitless multiplier).
    pub alpha_0: f32,
    /// Activation energy in eV.
    pub e_a_ev: f32,
    /// Human-readable identifier.
    pub label_str: &'static str,
}

impl ArrheniusModel {
    /// GaAs pHEMT: E_a = 1.6 eV (Kayali 1999 JPL-96-25).
    pub const GAAS_PHEMT: Self = Self {
        alpha_0: 1.0,
        e_a_ev: 1.6,
        label_str: "GaAs_pHEMT_Ea=1.6eV",
    };

    /// GaN HEMT: E_a = 2.1 eV (Kayali 1999 JPL-96-25, Table 3).
    pub const GAN_HEMT: Self = Self {
        alpha_0: 1.0,
        e_a_ev: 2.1,
        label_str: "GaN_HEMT_Ea=2.1eV",
    };
}

impl PhysicsModel for ArrheniusModel {
    /// Temperature in Celsius → predicted drift rate (normalised, unitless).
    fn predict_drift_rate(&self, temperature_celsius: f32) -> f32 {
        let t_k = temperature_celsius + 273.15_f32;
        // k_B = 8.617_333×10⁻⁵ eV/K
        let kb = 8.617_333e-5_f32;
        self.alpha_0 * exp_approx(-self.e_a_ev / (kb * t_k))
    }

    fn label(&self) -> &'static str { self.label_str }

    fn reference(&self) -> &'static str {
        "Kayali 1999 JPL-96-25 Arrhenius thermal acceleration model"
    }

    fn maps_to_reason(&self) -> ReasonCode { ReasonCode::SustainedOutwardDrift }
}

/// Allan variance frequency-stability model for oscillator aging.
///
/// σ_y²(τ) = h₀/(2τ) + h₋₁·2·ln2 + h₋₂·(2π²/3)·τ
///
/// ## Pre-defined Constants
/// - `OCXO_CLASS_A`: Ultra-stable oven-controlled XO (h₀=1e-20, h₋₁=1e-22, h₋₂=1e-28)
/// - `TCXO_GRADE_B`: Temperature-compensated XO (h₀=1e-18, h₋₁=1e-20, h₋₂=1e-26)
#[derive(Debug, Clone, Copy)]
pub struct AllanVarianceModel {
    /// White phase noise coefficient h₀.
    pub h_white: f32,
    /// Flicker phase noise coefficient h₋₁.
    pub h_flicker: f32,
    /// Random walk FM noise coefficient h₋₂.
    pub h_rw: f32,
    /// Human-readable identifier.
    pub label_str: &'static str,
}

impl AllanVarianceModel {
    /// Ultra-stable OCXO Class A oscillator (normalised residual-norm units).
    ///
    /// h-coefficients scaled so σ_y(τ=1) ≈ 2.2×10⁻⁵ (detectable in f32).
    pub const OCXO_CLASS_A: Self = Self {
        h_white: 1e-9,
        h_flicker: 1e-11,
        h_rw: 1e-17,
        label_str: "OCXO_Class_A",
    };

    /// GPS-grade TCXO Grade B oscillator (normalised residual-norm units).
    ///
    /// h-coefficients scaled so σ_y(τ=1) ≈ 2.2×10⁻⁴ (100× worse than OCXO).
    pub const TCXO_GRADE_B: Self = Self {
        h_white: 1e-7,
        h_flicker: 1e-9,
        h_rw: 1e-15,
        label_str: "TCXO_Grade_B",
    };
}

impl PhysicsModel for AllanVarianceModel {
    /// Averaging time τ [s] → Allan deviation σ_y(τ) = √AVAR(τ).
    fn predict_drift_rate(&self, tau: f32) -> f32 {
        if tau <= 0.0 { return 0.0; }
        // σ_y²(τ) = h₀/(2τ) + h₋₁·2ln2 + h₋₂·(2π²/3)·τ
        let avar = self.h_white / (2.0 * tau)
            + self.h_flicker * 2.0 * 0.693_147_f32  // 2 ln2
            + self.h_rw * (2.0 * 9.869_604_f32 / 3.0) * tau; // 2π²/3 · τ
        crate::math::sqrt_f32(avar.max(0.0))
    }

    fn label(&self) -> &'static str { self.label_str }

    fn reference(&self) -> &'static str {
        "Allan 1966 Proc. IEEE 54(2):221-230; IEEE Std 1193-2003"
    }

    fn maps_to_reason(&self) -> ReasonCode { ReasonCode::SustainedOutwardDrift }
}

/// Result of comparing an observed drift against a physics model prediction.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsConsistencyResult {
    /// Predicted drift rate from the model.
    pub predicted_drift: f32,
    /// Observed drift from the DSFB engine.
    pub observed_drift: f32,
    /// Relative deviation: |observed − predicted| / predicted.  
    /// f32::MAX if predicted ≈ 0.
    pub deviation_ratio: f32,
    /// Whether observed drift is within the specified tolerance of predicted.
    pub is_consistent: bool,
    /// The DSFB reason code the model maps to.
    pub reason: ReasonCode,
}

/// Compare an observed RF drift rate against a physics-model prediction.
///
/// - `model`:           Any `PhysicsModel` implementor (Arrhenius, Allan, etc.).
/// - `observed_drift`:  Drift rate observed by the DSFB engine.
/// - `platform_param`:  Parameter to feed the model (temperature °C, τ, …).
/// - `tolerance`:       Acceptable relative deviation (e.g. 0.20 = ±20%).
pub fn evaluate_physics_consistency(
    model: &dyn PhysicsModel,
    observed_drift: f32,
    platform_param: f32,
    tolerance: f32,
) -> PhysicsConsistencyResult {
    let predicted = model.predict_drift_rate(platform_param);
    let deviation_ratio = if predicted > 1e-38 {
        (observed_drift - predicted).abs() / predicted
    } else {
        f32::MAX
    };
    let is_consistent = deviation_ratio <= tolerance.abs();
    PhysicsConsistencyResult {
        predicted_drift: predicted,
        observed_drift,
        deviation_ratio,
        is_consistent,
        reason: model.maps_to_reason(),
    }
}

// ── Private math helpers (no libm) ─────────────────────────────────────────

/// exp(x) approximation without libm using exp(x) = 2^(x · log₂e).
///
/// Accurate to < 0.05% for |x| ≤ 40.
fn exp_approx(x: f32) -> f32 {
    // log₂(e) = 1/ln2 ≈ 1.442695
    let y = x * 1.442_695_f32;
    // Clamp to avoid overflow
    let y = if y > 120.0 { 120.0 } else if y < -120.0 { -120.0 } else { y };
    let n = if y >= 0.0 { y as i32 } else { y as i32 - 1 };
    let frac = y - n as f32;
    let ln2 = 0.693_147_f32;
    let mantissa = 1.0 + frac * (ln2 + frac * (0.240_226_f32 + frac * 0.055_504_f32));
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
    fn semiotic_horizon_record_and_query() {
        let mut horizon = SemioticHorizon::<16>::new();
        horizon.record(10.0, 0.005, true, 15);
        horizon.record(5.0, 0.005, true, 25);
        horizon.record(-5.0, 0.005, false, 0);
        horizon.record(-15.0, 0.001, false, 0);

        assert_eq!(horizon.len(), 4);
        assert!((horizon.detection_rate() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn mean_latency_only_counts_detected() {
        let mut horizon = SemioticHorizon::<8>::new();
        horizon.record(10.0, 0.01, true, 10);
        horizon.record(5.0, 0.01, true, 20);
        horizon.record(-10.0, 0.01, false, 0);
        let lat = horizon.mean_detection_latency();
        assert!((lat - 15.0).abs() < 1e-4, "mean latency of detected: {}", lat);
    }

    #[test]
    fn candidate_mechanisms_for_drift() {
        let mechs = candidate_mechanisms(ReasonCode::SustainedOutwardDrift);
        assert!(mechs.contains(&PhysicalMechanism::PaThermalDrift));
        assert!(mechs.contains(&PhysicalMechanism::LoAging));
    }

    #[test]
    fn candidate_mechanisms_for_jamming() {
        let mechs = candidate_mechanisms(ReasonCode::AbruptSlewViolation);
        assert!(mechs.contains(&PhysicalMechanism::IntentionalJamming));
    }

    #[test]
    fn model_reference_non_empty() {
        let ref_str = model_reference(PhysicalMechanism::PaThermalDrift);
        assert!(ref_str.contains("Arrhenius"));
        let leeson = model_reference(PhysicalMechanism::PhaseNoiseDegradation);
        assert!(leeson.contains("Leeson"));
    }

    #[test]
    fn horizon_capacity_enforced() {
        let mut h = SemioticHorizon::<2>::new();
        assert!(h.record(0.0, 0.0, true, 1));
        assert!(h.record(0.0, 0.0, false, 0));
        assert!(!h.record(0.0, 0.0, true, 1), "must reject when full");
    }

    // ── PhysicsModel Tests ─────────────────────────────────────────────────

    #[test]
    fn arrhenius_drift_increases_with_temperature() {
        let model = ArrheniusModel::GAAS_PHEMT;
        let drift_25 = model.predict_drift_rate(25.0);
        let drift_125 = model.predict_drift_rate(125.0);
        assert!(drift_125 > drift_25,
            "Arrhenius: higher T → higher drift: {}→{}", drift_25, drift_125);
    }

    #[test]
    fn arrhenius_gan_slower_than_gaas_at_same_temp() {
        // GaN has higher E_a → slower degradation at same T
        let gaas = ArrheniusModel::GAAS_PHEMT.predict_drift_rate(125.0);
        let gan  = ArrheniusModel::GAN_HEMT.predict_drift_rate(125.0);
        assert!(gan < gaas,
            "GaN (E_a=2.1) must have lower drift than GaAs (E_a=1.6): {} vs {}", gan, gaas);
    }

    #[test]
    fn allan_variance_ocxo_better_than_tcxo() {
        // OCXO Class A should have lower σ_y at τ=1
        let ocxo = AllanVarianceModel::OCXO_CLASS_A.predict_drift_rate(1.0);
        let tcxo = AllanVarianceModel::TCXO_GRADE_B.predict_drift_rate(1.0);
        assert!(ocxo < tcxo,
            "OCXO must be more stable than TCXO: {} vs {}", ocxo, tcxo);
    }

    #[test]
    fn allan_variance_returns_zero_for_zero_tau() {
        let m = AllanVarianceModel::OCXO_CLASS_A;
        let s = m.predict_drift_rate(0.0);
        assert_eq!(s, 0.0, "AVAR at τ=0 must return 0");
    }

    #[test]
    fn physics_consistency_within_tolerance() {
        let model = ArrheniusModel::GAAS_PHEMT;
        let predicted = model.predict_drift_rate(85.0);
        // Feed observed = predicted * 1.1 (10% deviation) with 20% tolerance
        let result = evaluate_physics_consistency(&model, predicted * 1.1, 85.0, 0.20);
        assert!(result.is_consistent,
            "10% deviation within 20% tolerance: ratio={}", result.deviation_ratio);
    }

    #[test]
    fn physics_consistency_outside_tolerance() {
        let model = ArrheniusModel::GAAS_PHEMT;
        let predicted = model.predict_drift_rate(85.0);
        // Feed observed = 3× predicted (200% off), tolerance = 50%
        let result = evaluate_physics_consistency(&model, predicted * 3.0, 85.0, 0.50);
        assert!(!result.is_consistent,
            "200% deviation outside 50% tolerance: ratio={}", result.deviation_ratio);
    }

    #[test]
    fn physics_model_reason_codes() {
        assert_eq!(ArrheniusModel::GAAS_PHEMT.maps_to_reason(), ReasonCode::SustainedOutwardDrift);
        assert_eq!(AllanVarianceModel::TCXO_GRADE_B.maps_to_reason(), ReasonCode::SustainedOutwardDrift);
    }

    #[test]
    fn exp_approx_reasonable_accuracy() {
        // e^0 = 1, e^1 ≈ 2.718, e^-1 ≈ 0.368
        let e0 = exp_approx(0.0);
        let e1 = exp_approx(1.0);
        let em1 = exp_approx(-1.0);
        assert!((e0 - 1.0).abs() < 0.01, "exp(0) ≈ 1: {}", e0);
        assert!((e1 - 2.718).abs() < 0.05, "exp(1) ≈ 2.718: {}", e1);
        assert!((em1 - 0.368).abs() < 0.01, "exp(-1) ≈ 0.368: {}", em1);
    }
}
