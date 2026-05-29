//! Chemometric detector records — the curated authority schema for what a detector *is* and *means*.
//!
//! Each [`ChemometricDetectorRecordV1`] is static authority metadata: the mathematical primitive, its
//! decision functional, the positive/negative witness it emits, the fusion axes it contributes to, its
//! known confusers, and its source basis. This crate ships the **18 executed** detectors as the strict
//! authority set (the ones the edge pipeline actually runs); the wider prior-art surface stays
//! catalogued in the edge corpus TOML. No detector executes here — these are records, not code.

/// Coarse statistical primitive a detector is built on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PrimitiveFamily {
    ScalarThreshold,
    ProjectionResidual,
    WindowStatistic,
    SequentialRecurrence,
    RankStatistic,
    DistributionDistance,
    Spectral,
    ResidualObserver,
}

/// The positive witness a detector emits when it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum OutputWitnessKind {
    ScalarExceedance,
    LatentDistance,
    ResidualNorm,
    TrendStatistic,
    Changepoint,
    DistributionShift,
    DensityDistance,
    SpectralShift,
    CoMonotoneCount,
    StructuralIsolation,
}

/// What a detector's *quiet* (non-firing) state asserts — the negative witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NegativeWitnessKind {
    WithinControlLimit,
    NoTrend,
    NoDistributionShift,
    NoSpectralShift,
    NoStructuralBreak,
    NoIsolatedDeviation,
}

/// Role a detector plays in fused evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum WitnessRole {
    Primary,
    Supporting,
}

/// DSFB fusion axis a detector contributes evidence to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FusionAxis {
    ResidualMagnitude,
    LatentMagnitude,
    ResidualEnergy,
    TemporalDrift,
    Changepoint,
    DistributionShift,
    Density,
    Spectral,
    StructureCoherence,
    Attribution,
}

/// Whether a detector is deterministic-by-construction or carries stochastic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DeterministicStatus {
    DeterministicNative,
    Stochastic,
}

/// Whether a detector is executed by the edge pipeline or catalogued as prior-art surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ImplementationStatus {
    Executed,
    Catalogued,
}

/// One curated chemometric detector record (schema v1).
///
/// `canonical_id` is the stable integer key (matches the edge corpus TOML); `detector_id` is the
/// snake_case identifier the edge pipeline uses as `Detector::id()` and is the key the edge subset
/// gate checks membership against. All fields are `&'static` so the record set is `const` and
/// `no_std`-friendly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChemometricDetectorRecordV1 {
    /// Stable integer key (matches the edge corpus TOML); the canonical identity/sort key folded into the hash.
    pub canonical_id: u32,
    /// snake_case identifier the edge pipeline uses as `Detector::id()`; the key the executed-subset gate checks.
    pub detector_id: &'static str,
    /// Human-readable detector name for reports and figures.
    pub display_name: &'static str,
    /// Other names the method is known by in the literature — a search/dedup aid, not load-bearing.
    pub aliases: &'static [&'static str],
    /// Coarse method-family label (e.g. `"ClassicalMspc"`, `"ChangePoint"`, `"DistributionalDrift"`).
    pub family: &'static str,
    /// The structural primitive this detector reduces to (`PrimitiveFamily`); governs fusion compatibility.
    pub primitive_family: PrimitiveFamily,
    /// The statistic computed, as an inspectable string (e.g. `"StandardisedDeviation"`, `"CUSUM"`) — documentation of the form, not executable code.
    pub mathematical_form: &'static str,
    /// How the statistic becomes a decision (e.g. `"TwoSided"`, `"OneSidedUpper"`) — inspectable string.
    pub decision_functional: &'static str,
    /// What the detector needs to run (e.g. `"per-variable residuals"`, `"baseline window"`) — its input contract.
    pub input_requirements: &'static [&'static str],
    /// Domains the method originated in — a provenance/breadth tag (e.g. `"INDUSTRIAL"`, `"TABULAR"`).
    pub origin_domains: &'static [&'static str],
    /// The kind of *positive* evidence the detector emits when it fires (`OutputWitnessKind`).
    pub output_witness: OutputWitnessKind,
    /// Whether this detector is a primary witness or a corroborator within fusion (`WitnessRole`).
    pub witness_role: WitnessRole,
    /// The *negative* ("did NOT fire, and here is why that is informative") evidence it contributes (`NegativeWitnessKind`).
    pub negative_witness_kind: NegativeWitnessKind,
    /// Which fusion axes this detector votes on — it contributes only to its declared axes, never others.
    pub fusion_axes: &'static [FusionAxis],
    /// Known confusers: conditions that can make this detector misfire — documented honestly and surfaced in confuser dockets.
    pub confuser_profile: &'static [&'static str],
    /// Whether the detector is deterministic-native or requires determinisation (`DeterministicStatus`).
    pub deterministic_status: DeterministicStatus,
    /// `Executed` by the edge pipeline vs `Catalogued` prior-art surface (`ImplementationStatus`).
    pub implementation_status: ImplementationStatus,
    /// Literature / standard references grounding the detector (e.g. `"Shewhart control chart; 3-sigma rule"`).
    pub source_refs: &'static [&'static str],
}

use DeterministicStatus::DeterministicNative;
use FusionAxis as Ax;
use ImplementationStatus::Executed;
use NegativeWitnessKind as Neg;
use OutputWitnessKind as Out;
use PrimitiveFamily as Pf;
use WitnessRole::Primary;

/// The 18 executed chemometric detectors — the strict authority set the edge pipeline runs.
///
/// **To add detector #19:** append a record with the next free `canonical_id`; fill every field per the
/// `ChemometricDetectorRecordV1` doc above; set `implementation_status: Executed` **only** once the edge
/// pipeline genuinely runs it (otherwise `Catalogued`); then re-mint `atlas_hash_v1` as a *governed* re-freeze
/// (the gate test in `tests/atlas_gates.rs` pins the old hash and will fail until you update it deliberately).
pub const DETECTOR_RECORDS: &[ChemometricDetectorRecordV1] = &[
    ChemometricDetectorRecordV1 {
        canonical_id: 1,
        detector_id: "shewhart_max_z",
        display_name: "Shewhart multivariate max |z|",
        aliases: &["3-sigma chart"],
        family: "ClassicalMspc",
        primitive_family: Pf::ScalarThreshold,
        mathematical_form: "StandardisedDeviation",
        decision_functional: "TwoSided",
        input_requirements: &["per-variable residuals", "baseline window"],
        origin_domains: &["INDUSTRIAL", "TABULAR"],
        output_witness: Out::ScalarExceedance,
        witness_role: Primary,
        negative_witness_kind: Neg::WithinControlLimit,
        fusion_axes: &[Ax::ResidualMagnitude],
        confuser_profile: &[
            "autocorrelation inflates run length",
            "heavy-tailed baseline",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Shewhart control chart; 3-sigma rule"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 2,
        detector_id: "robust_z_mad",
        display_name: "Robust z-score (median/MAD)",
        aliases: &["Hampel z"],
        family: "ClassicalMspc",
        primitive_family: Pf::ScalarThreshold,
        mathematical_form: "RobustStandardisedDeviation",
        decision_functional: "TwoSided",
        input_requirements: &["per-variable residuals", "baseline window"],
        origin_domains: &["INDUSTRIAL", "TABULAR"],
        output_witness: Out::ScalarExceedance,
        witness_role: Primary,
        negative_witness_kind: Neg::WithinControlLimit,
        fusion_axes: &[Ax::ResidualMagnitude],
        confuser_profile: &[
            "bimodal baseline breaks the MAD scale",
            "<50% contamination assumption",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Hampel filter; robust location/scale"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 3,
        detector_id: "pca_t2",
        display_name: "PCA Hotelling T-squared (score space)",
        aliases: &["Hotelling T2"],
        family: "ClassicalMspc",
        primitive_family: Pf::ProjectionResidual,
        mathematical_form: "MahalanobisInScoreSpace",
        decision_functional: "OneSided",
        input_requirements: &["multivariate residual matrix", "baseline window"],
        origin_domains: &["CHEMOMETRIC", "INDUSTRIAL"],
        output_witness: Out::LatentDistance,
        witness_role: Primary,
        negative_witness_kind: Neg::WithinControlLimit,
        fusion_axes: &[Ax::LatentMagnitude],
        confuser_profile: &[
            "in-model shifts within score space are missed",
            "non-stationary mean",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["MSPC T2 monitoring index"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 4,
        detector_id: "pca_spe_q",
        display_name: "PCA SPE / Q residual",
        aliases: &["Q statistic", "SPE"],
        family: "ClassicalMspc",
        primitive_family: Pf::ProjectionResidual,
        mathematical_form: "ReconstructionResidualNorm",
        decision_functional: "OneSided",
        input_requirements: &["multivariate residual matrix", "baseline window"],
        origin_domains: &["CHEMOMETRIC", "INDUSTRIAL"],
        output_witness: Out::ResidualNorm,
        witness_role: Primary,
        negative_witness_kind: Neg::WithinControlLimit,
        fusion_axes: &[Ax::ResidualEnergy],
        confuser_profile: &["new direction vs amplified known direction ambiguity"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Squared prediction error / Q statistic"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 15,
        detector_id: "ewma_spe",
        display_name: "EWMA control chart (on SPE)",
        aliases: &["EWMA"],
        family: "DynamicTemporal",
        primitive_family: Pf::WindowStatistic,
        mathematical_form: "ExponentiallyWeightedMean",
        decision_functional: "TwoSided",
        input_requirements: &["ordered residual stream", "baseline window"],
        origin_domains: &["INDUSTRIAL", "TIME_SERIES"],
        output_witness: Out::TrendStatistic,
        witness_role: Primary,
        negative_witness_kind: Neg::NoTrend,
        fusion_axes: &[Ax::TemporalDrift],
        confuser_profile: &["lambda trades sensitivity vs lag", "ramp vs step ambiguity"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Roberts 1959 EWMA"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 16,
        detector_id: "cusum_spe",
        display_name: "CUSUM chart (on SPE)",
        aliases: &["CUSUM"],
        family: "DynamicTemporal",
        primitive_family: Pf::SequentialRecurrence,
        mathematical_form: "CumulativeSum",
        decision_functional: "TwoSided",
        input_requirements: &["ordered residual stream", "baseline window"],
        origin_domains: &["INDUSTRIAL", "TIME_SERIES"],
        output_witness: Out::TrendStatistic,
        witness_role: Primary,
        negative_witness_kind: Neg::NoTrend,
        fusion_axes: &[Ax::TemporalDrift],
        confuser_profile: &["reference/slack k mis-set masks small shifts"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Page 1954 cumulative sum"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 17,
        detector_id: "page_hinkley_spe",
        display_name: "Page-Hinkley test (on SPE)",
        aliases: &[],
        family: "DynamicTemporal",
        primitive_family: Pf::SequentialRecurrence,
        mathematical_form: "CumulativeDeviation",
        decision_functional: "OneSided",
        input_requirements: &["ordered residual stream", "baseline window"],
        origin_domains: &["TIME_SERIES"],
        output_witness: Out::Changepoint,
        witness_role: Primary,
        negative_witness_kind: Neg::NoTrend,
        fusion_axes: &[Ax::Changepoint],
        confuser_profile: &[
            "gradual drift below delta evades",
            "magnitude/threshold coupling",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Page-Hinkley change detection"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 18,
        detector_id: "mann_kendall_spe",
        display_name: "Mann-Kendall trend test (windowed)",
        aliases: &[],
        family: "DynamicTemporal",
        primitive_family: Pf::RankStatistic,
        mathematical_form: "SignConcordanceSum",
        decision_functional: "TwoSided",
        input_requirements: &["ordered residual stream", "window length"],
        origin_domains: &["TIME_SERIES", "ENVIRONMENTAL"],
        output_witness: Out::TrendStatistic,
        witness_role: Primary,
        negative_witness_kind: Neg::NoTrend,
        fusion_axes: &[Ax::TemporalDrift],
        confuser_profile: &["seasonality mimics trend", "ties reduce power"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Mann-Kendall nonparametric trend"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 28,
        detector_id: "psi_spe",
        display_name: "Population Stability Index (on SPE)",
        aliases: &["PSI"],
        family: "NonlinearDistributional",
        primitive_family: Pf::DistributionDistance,
        mathematical_form: "PopulationStabilityIndex",
        decision_functional: "OneSided",
        input_requirements: &["baseline + test window samples"],
        origin_domains: &["TABULAR", "TIME_SERIES"],
        output_witness: Out::DistributionShift,
        witness_role: Primary,
        negative_witness_kind: Neg::NoDistributionShift,
        fusion_axes: &[Ax::DistributionShift],
        confuser_profile: &["bin-choice sensitivity", "sample-size dependence"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["PSI population drift"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 29,
        detector_id: "ks_spe",
        display_name: "Kolmogorov-Smirnov two-sample (on SPE)",
        aliases: &["KS test"],
        family: "NonlinearDistributional",
        primitive_family: Pf::DistributionDistance,
        mathematical_form: "EmpiricalCdfSupremum",
        decision_functional: "OneSided",
        input_requirements: &["baseline + test window samples"],
        origin_domains: &["TABULAR", "TIME_SERIES"],
        output_witness: Out::DistributionShift,
        witness_role: Primary,
        negative_witness_kind: Neg::NoDistributionShift,
        fusion_axes: &[Ax::DistributionShift],
        confuser_profile: &["insensitive to tail differences", "discrete-data ties"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["KS two-sample test"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 30,
        detector_id: "knn_spe",
        display_name: "kNN distance (on SPE cloud)",
        aliases: &[],
        family: "NonlinearDistributional",
        primitive_family: Pf::DistributionDistance,
        mathematical_form: "KNearestNeighbourDistance",
        decision_functional: "OneSided",
        input_requirements: &["baseline reference cloud", "test sample"],
        origin_domains: &["TABULAR"],
        output_witness: Out::DensityDistance,
        witness_role: Primary,
        negative_witness_kind: Neg::NoDistributionShift,
        fusion_axes: &[Ax::Density],
        confuser_profile: &["k and curse-of-dimensionality", "global vs local density"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["kNN-based anomaly distance"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 31,
        detector_id: "spectral_entropy_spe",
        display_name: "Spectral entropy shift (on SPE)",
        aliases: &[],
        family: "NonlinearDistributional",
        primitive_family: Pf::Spectral,
        mathematical_form: "NormalisedSpectralEntropy",
        decision_functional: "TwoSided",
        input_requirements: &["ordered residual stream", "window length"],
        origin_domains: &["TIME_SERIES"],
        output_witness: Out::SpectralShift,
        witness_role: Primary,
        negative_witness_kind: Neg::NoSpectralShift,
        fusion_axes: &[Ax::Spectral],
        confuser_profile: &[
            "windowing/spectral leakage",
            "broadband noise raises entropy",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Power-spectrum Shannon entropy"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 47,
        detector_id: "co_drift",
        display_name: "Variable-group co-drift",
        aliases: &[],
        family: "ProcessStructure",
        primitive_family: Pf::ResidualObserver,
        mathematical_form: "SignedCoMonotoneCount",
        decision_functional: "TwoSided",
        input_requirements: &["multivariate residual matrix", "baseline window"],
        origin_domains: &["INDUSTRIAL", "CHEMOMETRIC"],
        output_witness: Out::CoMonotoneCount,
        witness_role: Primary,
        negative_witness_kind: Neg::NoStructuralBreak,
        fusion_axes: &[Ax::StructureCoherence],
        confuser_profile: &["common-cause load following mimics co-drift"],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Coherent multi-variable residual drift"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 48,
        detector_id: "sensor_bias",
        display_name: "Sensor-bias isolation",
        aliases: &[],
        family: "ProcessStructure",
        primitive_family: Pf::ResidualObserver,
        mathematical_form: "IsolationMargin",
        decision_functional: "OneSided",
        input_requirements: &["multivariate residual matrix", "baseline window"],
        origin_domains: &["INDUSTRIAL"],
        output_witness: Out::StructuralIsolation,
        witness_role: Primary,
        negative_witness_kind: Neg::NoIsolatedDeviation,
        fusion_axes: &[Ax::Attribution],
        confuser_profile: &[
            "a local true process change mimics isolation",
            "comms dropouts",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Single-variable contribution isolation"],
    },
    // ── Phase-C executed batch (promoted Catalogued → Executed; governed atlas_hash_v1 re-freeze) ──
    // Four prior-art detectors now run by the edge `build_bank`, with matching edge implementations
    // (`detectors::{Mewma, Dpca, Mosum, Mmd}`). canonical_ids match their edge-corpus census entries.
    ChemometricDetectorRecordV1 {
        canonical_id: 19,
        detector_id: "mewma",
        display_name: "MEWMA (multivariate EWMA)",
        aliases: &["MEWMA", "Lowry EWMA"],
        family: "DynamicTemporal",
        primitive_family: Pf::WindowStatistic,
        mathematical_form: "MultivariateEwma",
        decision_functional: "OneSided",
        input_requirements: &["standardised residual matrix", "baseline window"],
        origin_domains: &["CHEMOMETRIC", "INDUSTRIAL"],
        output_witness: Out::TrendStatistic,
        witness_role: Primary,
        negative_witness_kind: Neg::NoTrend,
        fusion_axes: &[Ax::TemporalDrift, Ax::LatentMagnitude],
        confuser_profile: &[
            "large lambda degenerates to Shewhart (loses memory)",
            "sustained baseline drift inflates the smoother",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Lowry, Woodall, Champ & Rigdon 1992 MEWMA"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 20,
        detector_id: "dpca",
        display_name: "Dynamic PCA (lag-augmented)",
        aliases: &["DPCA"],
        family: "DynamicTemporal",
        primitive_family: Pf::ProjectionResidual,
        mathematical_form: "LagAugmentedReconstruction",
        decision_functional: "OneSided",
        input_requirements: &[
            "standardised residual matrix",
            "baseline window",
            "temporal ordering",
        ],
        origin_domains: &["CHEMOMETRIC", "TIME_SERIES"],
        output_witness: Out::ResidualNorm,
        witness_role: Primary,
        negative_witness_kind: Neg::WithinControlLimit,
        fusion_axes: &[Ax::ResidualEnergy, Ax::TemporalDrift],
        confuser_profile: &[
            "fixed lag-1 misses higher-order dynamics",
            "non-stationary autocorrelation",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Ku, Storer & Georgakis 1995 dynamic PCA"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 26,
        detector_id: "mosum",
        display_name: "MOSUM (moving sum of residuals)",
        aliases: &["MOSUM"],
        family: "DynamicTemporal",
        primitive_family: Pf::WindowStatistic,
        mathematical_form: "MovingSum",
        decision_functional: "TwoSided",
        input_requirements: &["univariate residual stream (SPE)", "baseline window"],
        origin_domains: &["TIME_SERIES", "INDUSTRIAL"],
        output_witness: Out::Changepoint,
        witness_role: Primary,
        negative_witness_kind: Neg::NoStructuralBreak,
        fusion_axes: &[Ax::Changepoint, Ax::TemporalDrift],
        confuser_profile: &[
            "window width sets the detectable shift duration",
            "slow drift within the window is attenuated",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Bauer & Hackl 1978 moving-sum control"],
    },
    ChemometricDetectorRecordV1 {
        canonical_id: 39,
        detector_id: "mmd",
        display_name: "Maximum Mean Discrepancy (RBF, landmark)",
        aliases: &["MMD", "kernel two-sample"],
        family: "NonlinearDistributional",
        primitive_family: Pf::DistributionDistance,
        mathematical_form: "KernelMeanEmbeddingDistance",
        decision_functional: "OneSided",
        input_requirements: &["standardised residual matrix", "baseline window"],
        origin_domains: &["CHEMOMETRIC", "TABULAR"],
        output_witness: Out::DistributionShift,
        witness_role: Primary,
        negative_witness_kind: Neg::NoDistributionShift,
        fusion_axes: &[Ax::DistributionShift, Ax::Density],
        confuser_profile: &[
            "landmark approximation loses fine structure",
            "kernel bandwidth gamma sets the sensitivity scale",
        ],
        deterministic_status: DeterministicNative,
        implementation_status: Executed,
        source_refs: &["Gretton, Borgwardt, Rasch, Scholkopf & Smola 2012 MMD"],
    },
];
