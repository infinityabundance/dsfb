//! Detector family enum — the head of the algebra grammar.
//!
//! `DetectorFamily` names a deterministic mathematical primitive
//! ([Shewhart chart, EWMA, CUSUM, ...]) without committing to
//! specific parameters. The full `DetectorSpec` glues a family
//! to a (transform, window, statistic, comparator, gate,
//! persistence) tuple via the panel-locked algebra.
//!
//! **The list at S1.1 is panel-recommended, not exhaustive.**
//! 43 families are seeded; later commits may add more. The order
//! is pinned by [`DetectorFamily::all`] and the acceptance test
//! `detector_family_order_is_stable`. Changing the order would
//! invalidate every [`crate::FamilyId`] assigned to a family, so
//! the order is treated as part of the algebra's stable surface.
//!
//! No detector kernel runs at S1.1. The family enum is purely a
//! type-level tag; the family-to-GPU-kernel mapping is S1.4+
//! work.

/// One named detector family. Each variant is the head of the
/// algebra's grammar: pair a family with a (transform, window,
/// statistic, comparator, gate, persistence) parameter set and a
/// canonical name results.
///
/// The canonical wire name (used in
/// [`crate::CanonicalDetectorName`]) is the variant identifier
/// rendered as uppercase snake-case, e.g. `RobustZMad` →
/// `ROBUST_Z_MAD`. The mapping lives in
/// [`DetectorFamily::canonical_wire_name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DetectorFamily {
    // --- Statistical process control ---
    /// Shewhart control chart.
    Shewhart,
    /// Western Electric rules (composition of Shewhart with
    /// run-length predicates).
    WesternElectric,
    /// Nelson rules (extended Shewhart with additional
    /// run-length / trend predicates).
    NelsonRules,
    /// Exponentially-weighted moving-average control chart.
    Ewma,
    /// Cumulative-sum control chart.
    Cusum,
    /// Page-Hinkley sequential change detector.
    PageHinkley,
    /// Hotelling T² multivariate control chart.
    HotellingT2,
    /// PCA squared prediction error (Q statistic / SPE).
    PcaSpeQ,

    // --- Robust statistics ---
    /// Robust z-score using median absolute deviation.
    RobustZMad,
    /// Hampel filter (median + scaled MAD threshold).
    Hampel,
    /// Tukey-fence outlier detector (k × IQR rule).
    TukeyFence,

    // --- Distribution distance / divergence ---
    /// Kolmogorov-Smirnov statistic.
    KolmogorovSmirnov,
    /// Anderson-Darling test.
    AndersonDarling,
    /// Cramér-von Mises test.
    CramerVonMises,
    /// Wasserstein (earth-mover's) distance.
    Wasserstein,
    /// Energy distance (Székely).
    EnergyDistance,
    /// Hellinger distance.
    Hellinger,
    /// Kullback-Leibler divergence.
    KullbackLeibler,
    /// Jensen-Shannon divergence.
    JensenShannon,
    /// Population Stability Index.
    PopulationStabilityIndex,

    // --- Trend / change-point ---
    /// Mann-Kendall trend test.
    MannKendall,
    /// Pettitt change-point test.
    Pettitt,
    /// Standard Normal Homogeneity Test.
    Snht,
    /// MOSUM (moving-sum) test.
    Mosum,

    // --- Spectral / signal ---
    /// FFT band-energy detector.
    FftBandEnergy,
    /// Spectral entropy.
    SpectralEntropy,
    /// Wavelet-coefficient energy.
    WaveletEnergy,
    /// Autocorrelation break detector.
    AutocorrelationBreak,

    // --- Data quality ---
    /// Sudden spike in missingness rate.
    MissingnessSpike,
    /// Coupling between missingness patterns across columns.
    MissingnessCoupling,
    /// Drift in column cardinality.
    CardinalityDrift,
    /// Uniqueness-constraint violation.
    UniquenessViolation,
    /// Functional-dependency violation.
    FunctionalDependencyViolation,

    // --- Debug / observability (mirrors dsfb-gpu-debug bank) ---
    /// Latency-ramp anomaly (the canonical R.9 GPU-implemented motif).
    LatencyRamp,
    /// Error-rate burst.
    ErrorBurst,
    /// Slew-shock recovery edge.
    SlewShock,
    /// Fan-out cascade precursor.
    FanoutCascade,

    // --- Industrial / FDD ---
    /// Residual envelope exit (innovation-bound breach).
    ResidualEnvelopeExit,
    /// Sensor bias detector.
    SensorBias,
    /// Actuator stiction detector.
    ActuatorStiction,
    /// Valve-hunting (limit-cycle) detector.
    ValveHunting,

    // --- RF / communications ---
    /// Carrier-frequency offset residual.
    CarrierFrequencyOffset,
    /// EVM (error-vector-magnitude) anomaly.
    EvmAnomaly,
}

impl DetectorFamily {
    /// Canonical ordering of every `DetectorFamily` variant.
    /// **Order is panel-locked** — see module-level docstring.
    /// Two builds on different machines see the same order.
    #[must_use]
    pub const fn all() -> &'static [DetectorFamily] {
        &[
            DetectorFamily::Shewhart,
            DetectorFamily::WesternElectric,
            DetectorFamily::NelsonRules,
            DetectorFamily::Ewma,
            DetectorFamily::Cusum,
            DetectorFamily::PageHinkley,
            DetectorFamily::HotellingT2,
            DetectorFamily::PcaSpeQ,
            DetectorFamily::RobustZMad,
            DetectorFamily::Hampel,
            DetectorFamily::TukeyFence,
            DetectorFamily::KolmogorovSmirnov,
            DetectorFamily::AndersonDarling,
            DetectorFamily::CramerVonMises,
            DetectorFamily::Wasserstein,
            DetectorFamily::EnergyDistance,
            DetectorFamily::Hellinger,
            DetectorFamily::KullbackLeibler,
            DetectorFamily::JensenShannon,
            DetectorFamily::PopulationStabilityIndex,
            DetectorFamily::MannKendall,
            DetectorFamily::Pettitt,
            DetectorFamily::Snht,
            DetectorFamily::Mosum,
            DetectorFamily::FftBandEnergy,
            DetectorFamily::SpectralEntropy,
            DetectorFamily::WaveletEnergy,
            DetectorFamily::AutocorrelationBreak,
            DetectorFamily::MissingnessSpike,
            DetectorFamily::MissingnessCoupling,
            DetectorFamily::CardinalityDrift,
            DetectorFamily::UniquenessViolation,
            DetectorFamily::FunctionalDependencyViolation,
            DetectorFamily::LatencyRamp,
            DetectorFamily::ErrorBurst,
            DetectorFamily::SlewShock,
            DetectorFamily::FanoutCascade,
            DetectorFamily::ResidualEnvelopeExit,
            DetectorFamily::SensorBias,
            DetectorFamily::ActuatorStiction,
            DetectorFamily::ValveHunting,
            DetectorFamily::CarrierFrequencyOffset,
            DetectorFamily::EvmAnomaly,
        ]
    }

    /// Stable canonical wire name for the family. Uppercase
    /// snake-case. Used as the leading token in
    /// [`crate::CanonicalDetectorName`]. The mapping is
    /// hand-pinned (rather than auto-derived from `Debug`) so a
    /// future Rust-version rename of the variant cannot silently
    /// shift the wire format.
    #[must_use]
    pub const fn canonical_wire_name(self) -> &'static str {
        match self {
            Self::Shewhart => "SHEWHART",
            Self::WesternElectric => "WESTERN_ELECTRIC",
            Self::NelsonRules => "NELSON_RULES",
            Self::Ewma => "EWMA",
            Self::Cusum => "CUSUM",
            Self::PageHinkley => "PAGE_HINKLEY",
            Self::HotellingT2 => "HOTELLING_T2",
            Self::PcaSpeQ => "PCA_SPE_Q",
            Self::RobustZMad => "ROBUST_Z_MAD",
            Self::Hampel => "HAMPEL",
            Self::TukeyFence => "TUKEY_FENCE",
            Self::KolmogorovSmirnov => "KOLMOGOROV_SMIRNOV",
            Self::AndersonDarling => "ANDERSON_DARLING",
            Self::CramerVonMises => "CRAMER_VON_MISES",
            Self::Wasserstein => "WASSERSTEIN",
            Self::EnergyDistance => "ENERGY_DISTANCE",
            Self::Hellinger => "HELLINGER",
            Self::KullbackLeibler => "KULLBACK_LEIBLER",
            Self::JensenShannon => "JENSEN_SHANNON",
            Self::PopulationStabilityIndex => "POPULATION_STABILITY_INDEX",
            Self::MannKendall => "MANN_KENDALL",
            Self::Pettitt => "PETTITT",
            Self::Snht => "SNHT",
            Self::Mosum => "MOSUM",
            Self::FftBandEnergy => "FFT_BAND_ENERGY",
            Self::SpectralEntropy => "SPECTRAL_ENTROPY",
            Self::WaveletEnergy => "WAVELET_ENERGY",
            Self::AutocorrelationBreak => "AUTOCORRELATION_BREAK",
            Self::MissingnessSpike => "MISSINGNESS_SPIKE",
            Self::MissingnessCoupling => "MISSINGNESS_COUPLING",
            Self::CardinalityDrift => "CARDINALITY_DRIFT",
            Self::UniquenessViolation => "UNIQUENESS_VIOLATION",
            Self::FunctionalDependencyViolation => "FUNCTIONAL_DEPENDENCY_VIOLATION",
            Self::LatencyRamp => "LATENCY_RAMP",
            Self::ErrorBurst => "ERROR_BURST",
            Self::SlewShock => "SLEW_SHOCK",
            Self::FanoutCascade => "FANOUT_CASCADE",
            Self::ResidualEnvelopeExit => "RESIDUAL_ENVELOPE_EXIT",
            Self::SensorBias => "SENSOR_BIAS",
            Self::ActuatorStiction => "ACTUATOR_STICTION",
            Self::ValveHunting => "VALVE_HUNTING",
            Self::CarrierFrequencyOffset => "CARRIER_FREQUENCY_OFFSET",
            Self::EvmAnomaly => "EVM_ANOMALY",
        }
    }

    /// `FamilyId` of this variant, derived from its position in
    /// [`DetectorFamily::all`]. Two builds produce the same id
    /// for the same variant because `all()` is panel-locked.
    #[must_use]
    pub fn family_id(self) -> crate::ids::FamilyId {
        let all = Self::all();
        for (idx, f) in all.iter().enumerate() {
            if *f == self {
                return crate::ids::FamilyId(idx as u32);
            }
        }
        // Unreachable: every variant is in `all()`. Acceptance
        // test `detector_family_order_is_stable` covers this.
        crate::ids::FamilyId(u32::MAX)
    }
}
