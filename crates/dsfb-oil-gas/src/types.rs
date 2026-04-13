/// DSFB Oil & Gas — Core Types
///
/// All types in this module are pure data; no I/O and no side-effects.
/// The framework is read-only: no type here writes back to any upstream
/// SCADA register, DCS tag, alarm limit, or control variable.
///
/// # no_std layout
/// Types available unconditionally (no heap):
///   `GrammarState`, `AdmissibilityEnvelope`, `ResidualTriple`, `ReasonCode`
///
/// Types gated behind `feature = "alloc"` (require heap):
///   `ResidualSample`, `AnnotatedStep`, `Episode`, `EpisodeSummary`,
///   `DsfbDomainFrame`

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

// ─────────────────────────────────────────────────────────────────────────────
// Residual sample  (alloc: contains String channel)
// ─────────────────────────────────────────────────────────────────────────────

/// A single residual observation: r_k = observed_k − expected_k.
///
/// The field `channel` identifies the physical signal (e.g., "tubing_pressure",
/// "torque_kNm"). It is purely informational; the arithmetic is channel-agnostic.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq)]
pub struct ResidualSample {
    /// Time index (seconds since epoch or arbitrary monotone counter).
    pub timestamp: f64,
    /// Observed sensor value (engineering units, caller-normalised).
    pub observed: f64,
    /// Expected value from upstream estimator (model, filter, baseline).
    pub expected: f64,
    /// Human-readable channel identifier.
    pub channel: String,
}

#[cfg(feature = "alloc")]
impl ResidualSample {
    /// Scalar residual r_k = observed − expected.
    #[inline]
    pub fn residual(&self) -> f64 {
        self.observed - self.expected
    }

    pub fn new(timestamp: f64, observed: f64, expected: f64, channel: impl Into<String>) -> Self {
        ResidualSample {
            timestamp,
            observed,
            expected,
            channel: channel.into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Drift / Slew state
// ─────────────────────────────────────────────────────────────────────────────

/// The (r, δ, σ) triple produced by the DSFB decomposition at each time step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResidualTriple {
    /// Raw residual r_k.
    pub r: f64,
    /// Signed causal drift δ_k (windowed mean of residuals).
    pub delta: f64,
    /// Slew σ_k = (r_k − r_{k−1}) / Δt.
    pub sigma: f64,
    /// Time stamp carried through for traceability.
    pub timestamp: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Envelope
// ─────────────────────────────────────────────────────────────────────────────

/// Admissibility bounds for a single DSFB channel.
///
/// The envelope is a product set [r_min, r_max] × [δ_min, δ_max] × [σ_min, σ_max].
/// All six bounds must be calibrated from domain physics and operational history.
/// Default constants are illustrative only; they must not be applied to real
/// systems without site-specific calibration.
#[derive(Debug, Clone, Copy)]
pub struct AdmissibilityEnvelope {
    pub r_min: f64,
    pub r_max: f64,
    pub delta_min: f64,
    pub delta_max: f64,
    pub sigma_min: f64,
    pub sigma_max: f64,
    /// Normalised grazing band ε_b ∈ (0,1).  A state within ε_b of any
    /// envelope face, but not outside it, is classified as BoundaryGrazing.
    pub grazing_band: f64,
}

impl AdmissibilityEnvelope {
    pub fn new(
        r_min: f64, r_max: f64,
        delta_min: f64, delta_max: f64,
        sigma_min: f64, sigma_max: f64,
        grazing_band: f64,
    ) -> Self {
        assert!(r_min < r_max, "r bounds must be ordered");
        assert!(delta_min < delta_max, "delta bounds must be ordered");
        assert!(sigma_min < sigma_max, "sigma bounds must be ordered");
        assert!((0.0..1.0).contains(&grazing_band), "grazing_band must be in (0,1)");
        AdmissibilityEnvelope { r_min, r_max, delta_min, delta_max, sigma_min, sigma_max, grazing_band }
    }

    /// Illustrative defaults for a generic pipeline residual channel.
    /// Must be recalibrated for any real deployment.
    pub fn default_pipeline() -> Self {
        Self::new(-5.0, 5.0, -3.0, 3.0, -10.0, 10.0, 0.1)
    }

    /// Illustrative defaults for a drilling torque-residual channel.
    pub fn default_drilling() -> Self {
        Self::new(-20.0, 20.0, -10.0, 10.0, -50.0, 50.0, 0.1)
    }

    /// Illustrative defaults for a rotating equipment head-residual channel.
    pub fn default_rotating() -> Self {
        Self::new(-8.0, 8.0, -4.0, 4.0, -20.0, 20.0, 0.1)
    }

    /// Illustrative defaults for a subsea actuation-pressure-residual channel.
    pub fn default_subsea() -> Self {
        Self::new(-50.0, 50.0, -20.0, 20.0, -200.0, 200.0, 0.08)
    }

    /// Defaults for a real subsea oil-well production-choke channel (Pa scale).
    ///
    /// Calibrated from Petrobras 3W Dataset v2.0.0 real WELL-* instances.
    /// P-MON-CKP residuals (30-min rolling-median baseline):
    ///   μ ≈ −5 600 Pa,  σ ≈ 545 000 Pa,  range [−3.0 MPa, +4.6 MPa].
    ///
    /// Thresholds at ≈ 0.9σ so normal steady-state remains interior while
    /// slug, hydrate, and valve-closure events breach the boundary.
    /// Must be recalibrated per well and per operating regime.
    pub fn default_oilwell() -> Self {
        Self::new(
            -500_000.0,  500_000.0,   // r   [Pa]  ≈ ±0.9σ instantaneous
            -200_000.0,  200_000.0,   // δ   [Pa]  tighter drift threshold
            -2_000_000.0, 2_000_000.0, // σ   [Pa]  wide spread threshold
            0.10,
        )
    }

    /// Defaults for the Equinor Volve well 15/9-F-15 surface-torque channel (kNm).
    ///
    /// Calibrated from WITSML depth-indexed log, 1 200 – 4 065 m MD, 5 326 rows
    /// at 0.5-m depth steps.  TQA residuals (20-sample rolling-median baseline):
    ///   μ ≈ 0.0 kNm,  σ ≈ 7.76 kNm,  range [−35, +35] kNm.
    ///
    /// r bounds set at ±8 kNm (≈1σ); δ tighter at ±3 kNm; σ wide at ±20 kNm.
    /// Must be recalibrated per well and per operating phase.
    pub fn default_volve_drilling() -> Self {
        Self::new(
            -8.0,  8.0,   // r  [kNm] ≈1σ TQA residual
            -3.0,  3.0,   // δ  [kNm] drift threshold
            -20.0, 20.0,  // σ  [kNm] slew threshold
            0.10,
        )
    }

    /// Defaults for the RPDBCS ESPset rotating equipment broadband-RMS channel.
    ///
    /// Calibrated from 6 032 vibration snapshots across 11 ESP units.
    /// rms_broadband residuals (15-sample rolling-median baseline):
    ///   mean ≈ 0.157,  σ ≈ 0.152,  max = 2.22 (fault events).
    ///
    /// r bounds at ±0.10 (≈0.66σ) to flag sustained deviations;
    /// δ tighter at ±0.05; σ wide at ±0.30 to capture abrupt fault onset.
    /// Must be recalibrated per pump unit and per operating speed.
    pub fn default_esp_rotating() -> Self {
        Self::new(
            -0.10,  0.10,   // r  [g RMS] ±0.66σ broadband residual
            -0.05,  0.05,   // δ  [g RMS] drift threshold
            -0.30,  0.30,   // σ  [g RMS] slew threshold
            0.10,
        )
    }



    /// Normalise a triple into [−1, 1]^3.  Returns (r̃, δ̃, σ̃).
    pub fn normalise(&self, t: &ResidualTriple) -> (f64, f64, f64) {
        let norm = |v: f64, lo: f64, hi: f64| {
            let mid = (hi + lo) / 2.0;
            let half = (hi - lo) / 2.0;
            (v - mid) / half
        };
        (
            norm(t.r, self.r_min, self.r_max),
            norm(t.delta, self.delta_min, self.delta_max),
            norm(t.sigma, self.sigma_min, self.sigma_max),
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Grammar states and tokens
// ─────────────────────────────────────────────────────────────────────────────

/// Typed grammar state emitted by the DSFB automaton at each time step.
///
/// These states are annotation tokens; they do not trigger any actuation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrammarState {
    /// Triple fully inside envelope.
    Nominal,
    /// Drift component exits envelope; slew within.
    DriftAccum,
    /// Slew component exits envelope; drift within.
    SlewSpike,
    /// Raw residual exits its admissibility bounds.
    EnvViolation,
    /// Any component within grazing band but not outside.
    BoundaryGrazing,
    /// First step after any non-Nominal state returns to Nominal.
    Recovery,
    /// Both drift and slew simultaneously outside envelope.
    Compound,
    /// One or more residual components is non-finite (NaN or ±∞).
    /// Emitted by the grammar automaton as an out-of-band sentinel when the
    /// sensor reports an unrepresentable value or the historian stream
    /// contains a gap encoded as IEEE 754 NaN.  The automaton's internal
    /// state is preserved so Recovery logic is not disrupted.
    SensorFault,
}

impl GrammarState {
    /// Short ASCII token for logging.
    pub fn token(&self) -> &'static str {
        match self {
            GrammarState::Nominal         => "NOM",
            GrammarState::DriftAccum      => "DA",
            GrammarState::SlewSpike       => "SS",
            GrammarState::EnvViolation    => "EV",
            GrammarState::BoundaryGrazing => "BG",
            GrammarState::Recovery        => "RC",
            GrammarState::Compound        => "CP",
            GrammarState::SensorFault     => "SF",
        }
    }

    pub fn is_nominal(&self) -> bool { *self == GrammarState::Nominal }
    pub fn is_non_nominal(&self) -> bool { !self.is_nominal() }
}

impl core::fmt::Display for GrammarState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.token())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Reason codes  (Copy enum — no heap allocation)
// ─────────────────────────────────────────────────────────────────────────────

/// Structured reason code attached to each grammar state classification.
///
/// `ReasonCode` is a `Copy` enum so it can be embedded in `no_alloc` contexts.
/// Reason codes are operator-facing annotations; they must not be confused
/// with root-cause attribution or alarm triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonCode {
    /// Triple fully inside calibrated envelope.
    Nominal,
    /// Sustained positive drift outside calibrated delta bounds.
    DriftPositive,
    /// Sustained negative drift outside calibrated delta bounds.
    DriftNegative,
    /// Rapid rising transient outside calibrated sigma bounds.
    SlewRising,
    /// Rapid falling transient outside calibrated sigma bounds.
    SlewFalling,
    /// Raw residual outside calibrated r bounds.
    Violation,
    /// State within grazing band of envelope boundary.
    Grazing,
    /// State returned to envelope interior following non-Nominal episode.
    Recovery,
    /// Simultaneous drift and slew exit from envelope.
    Compound,
    /// One or more residual components is non-finite (NaN or ±∞);
    /// the grammar automaton emits this out-of-band sentinel and
    /// preserves its internal state so Recovery logic is unaffected.
    OobSensor,
}

impl ReasonCode {
    /// Static human-readable description string.
    pub fn as_str(self) -> &'static str {
        match self {
            ReasonCode::Nominal       => "Operating within calibrated envelope",
            ReasonCode::DriftPositive => "Sustained positive drift outside calibrated delta bounds",
            ReasonCode::DriftNegative => "Sustained negative drift outside calibrated delta bounds",
            ReasonCode::SlewRising    => "Rapid rising transient outside calibrated sigma bounds",
            ReasonCode::SlewFalling   => "Rapid falling transient outside calibrated sigma bounds",
            ReasonCode::Violation     => "Raw residual outside calibrated r bounds",
            ReasonCode::Grazing       => "State within grazing band of envelope boundary",
            ReasonCode::Recovery      => "State returned to envelope interior following non-Nominal episode",
            ReasonCode::Compound      => "Simultaneous drift and slew exit from envelope",
            ReasonCode::OobSensor     => "Non-finite sensor value (NaN or \u{b1}\u{221e}); observation masked pending supervisor review",
        }
    }

    pub fn nominal()    -> Self { ReasonCode::Nominal }
    pub fn drift(sign: f64)  -> Self { if sign >= 0.0 { ReasonCode::DriftPositive } else { ReasonCode::DriftNegative } }
    pub fn slew(sign: f64)   -> Self { if sign >= 0.0 { ReasonCode::SlewRising } else { ReasonCode::SlewFalling } }
    pub fn violation()  -> Self { ReasonCode::Violation }
    pub fn grazing()    -> Self { ReasonCode::Grazing }
    pub fn recovery()   -> Self { ReasonCode::Recovery }
    pub fn compound()   -> Self { ReasonCode::Compound }
    pub fn oob_sensor() -> Self { ReasonCode::OobSensor }
}

impl core::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Annotated step  (alloc: contains String channel)
// ─────────────────────────────────────────────────────────────────────────────

/// A single annotated time step: triple + grammar state + reason code.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct AnnotatedStep {
    pub triple: ResidualTriple,
    pub state: GrammarState,
    pub reason: ReasonCode,
    pub channel: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Episode  (alloc: contains String channel)
// ─────────────────────────────────────────────────────────────────────────────

/// A maximal contiguous sequence of identical GrammarState classifications.
///
/// Episode aggregation compresses the raw step sequence into a compact event log.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct Episode {
    pub state: GrammarState,
    pub channel: String,
    pub start_ts: f64,
    pub end_ts: f64,
    /// Number of time steps in this episode.
    pub step_count: usize,
    /// Peak absolute residual within this episode.
    pub peak_r: f64,
    /// Peak absolute drift within this episode.
    pub peak_delta: f64,
    /// Peak absolute slew within this episode.
    pub peak_sigma: f64,
    /// Dominant sign of drift during this episode.
    pub drift_sign: f64,
    /// Representative reason code (from first step).
    pub reason: ReasonCode,
}

#[cfg(feature = "alloc")]
impl Episode {
    pub fn duration_s(&self) -> f64 {
        self.end_ts - self.start_ts
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Episode summary (report)  (alloc: contains String + BTreeMap)
// ─────────────────────────────────────────────────────────────────────────────

/// Summary statistics for a complete annotation run on one channel.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct EpisodeSummary {
    pub channel: String,
    pub total_steps: usize,
    pub total_episodes: usize,
    pub nominal_steps: usize,
    pub non_nominal_episodes: usize,
    /// Episode count collapse ratio = total_steps / total_episodes.
    pub episode_count_collapse: f64,
    /// Fraction of steps in Nominal state.
    pub event_density_reduction: f64,
    pub by_state: BTreeMap<GrammarState, usize>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain frame trait  (alloc: to_residual_sample() returns ResidualSample)
// ─────────────────────────────────────────────────────────────────────────────

/// Trait implemented by all domain-specific sensor frames.
/// Each frame provides a timestamp, an expected value, and an observed value.
/// The DSFB engine operates on the residual (observed − expected).
#[cfg(feature = "alloc")]
pub trait DsfbDomainFrame: Send + Sync {
    fn timestamp(&self) -> f64;
    fn expected(&self) -> f64;
    fn observed(&self) -> f64;
    fn channel_name(&self) -> &str { "default" }

    fn to_residual_sample(&self) -> ResidualSample {
        ResidualSample::new(
            self.timestamp(),
            self.observed(),
            self.expected(),
            self.channel_name(),
        )
    }
}
