//! Static seed corpus of literature detector primitives.
//!
//! The panel verdict locked T.1a (the scaffolding commit) as
//! "modest and sharp" — 15 primitives that exercised the schema
//! shape. T.1b (this commit) expands the seed to 54 primitives
//! across the panel-enumerated families:
//!
//! - **SPC / process monitoring**: Shewhart, EWMA, CUSUM, Page-
//!   Hinkley, Western Electric rules, Nelson rules, Tukey fences,
//!   Hotelling T2, PCA T2, PCA SPE / Q residual, PLS residual,
//!   residual envelope exit, sensor bias, actuator stiction,
//!   valve hunting.
//! - **Robust statistics**: robust z (MAD), Hampel filter, Tukey
//!   fences (above).
//! - **Distribution / distance**: KS, Anderson-Darling, Cramer-
//!   von Mises, KL, JS, Hellinger, MMD, Wasserstein, energy
//!   distance, total variation, Population Stability Index.
//! - **Sequential / change-point**: CUSUM (above), Page-Hinkley
//!   (above), Pettitt, SNHT, MOSUM, Buishand range, Mann-Kendall.
//! - **Spectral / signal**: FFT band-energy, spectral entropy,
//!   wavelet energy, autocorrelation break.
//! - **Data quality**: missingness spike, missingness coupling,
//!   schema drift, cardinality drift, uniqueness violation,
//!   functional-dependency violation.
//! - **Debug / observability**: latency ramp, error burst, slew
//!   shock, fanout cascade.
//! - **Medical / biosignal**: R-peak interval anomaly, HRV time-
//!   domain shift, QRS width anomaly, ST-segment deviation proxy.
//! - **RF / communications**: carrier-frequency-offset residual,
//!   EVM anomaly.
//! - **Negative-witness lane**: single-window spike confuser.
//!
//! T.2 will mirror this seed into per-domain TOML files under
//! `corpus/` for the formal source-ingestion format; T.1b keeps
//! the static Rust seed so the schema breadth can be debugged in
//! one place before storage layout changes.
//!
//! Every entry carries:
//!
//! * At least one [`SourceRef`] OR an explicit engineering-practice
//!   note (per the verifier's `declared_provenance` constraint).
//! * All eight [`ConstitutionFlags`] set to `true`.
//! * A non-empty [`DomainTagSet`] and [`InputRequirementSet`].
//! * `DuplicateGroupId` equal to `canonical_id` (every T.1a entry
//!   is canonical; T.1b+ introduces aliases that will collapse
//!   into these canonicals via the dedup court).
//!
//! Canonical-IDs are stable, dense, and start at 1; the value 0 is
//! reserved as a future "null / not-applicable" sentinel.

use crate::types::{
    AxisBindingSet, ConfuserProfile, ConstitutionFlags, DecisionFunctional, DetectorCanonicalId,
    DeterministicStatus, DomainTagSet, DuplicateGroupId, GenealogyEdges, GpuFamilyKernel,
    ImplementationLevel, InputRequirementSet, LifecycleState, LiteratureDetector, MathFormId,
    NegativeWitnessKind, ParameterBounds, PrimitiveFamily, SourceRef, UsefulnessLedgerSnapshot,
    WitnessKind, WitnessRole,
};

/// Constitution flags with every required attestation set to `true`.
/// Every seed entry uses this; the verifier checks each field
/// independently so omissions are caught at build time.
const ALL_TRUE: ConstitutionFlags = ConstitutionFlags {
    declared_input_contract: true,
    declared_output_type: true,
    declared_deterministic_form: true,
    declared_provenance: true,
    declared_equivalence_status: true,
    declared_witness_role: true,
    declared_activation_conditions: true,
    declared_failure_confuser_modes: true,
};

/// The unmeasured ledger row used by every T.1a seed entry. T.8
/// populates real measurements per (task × dataset).
const UNMEASURED: UsefulnessLedgerSnapshot = UsefulnessLedgerSnapshot::unmeasured();

/// Origin-of-genealogy edges used by foundational primitives.
const ORIGIN: GenealogyEdges = GenealogyEdges::origin();

/// The 15-primitive T.1a seed corpus.
///
/// Each entry's `canonical_id` is the row index + 1; `duplicate_group`
/// equals `canonical_id` because every T.1a entry is canonical.
pub static SEED: &[LiteratureDetector] = &[
    // 1. Shewhart control chart — the foundational SPC primitive.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(1),
        display_name: "Shewhart control chart",
        aliases: &["3-sigma rule", "x-bar chart"],
        source_refs: &[SourceRef {
            citation_key: "shewhart1924",
            title: "Economic Control of Quality of Manufactured Product",
            authors: "Shewhart, Walter A.",
            year: 1931,
            venue_or_source: "Van Nostrand (book); Bell Labs memo 1924",
            doi_or_url: None,
            notes: "Foundational SPC chart. Quality control / out-of-control rule.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::StandardisedDeviation,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "k-sigma control limit (typically 3); baseline window length.",
        },
        duplicate_group: DuplicateGroupId(1),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 2. EWMA chart.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(2),
        display_name: "EWMA control chart",
        aliases: &[
            "exponentially weighted moving average chart",
            "EWMA residual chart",
            "Roberts EWMA",
        ],
        source_refs: &[SourceRef {
            citation_key: "roberts1959",
            title: "Control Chart Tests Based on Geometric Moving Averages",
            authors: "Roberts, S. W.",
            year: 1959,
            venue_or_source: "Technometrics 1(3): 239-250",
            doi_or_url: Some("https://doi.org/10.1080/00401706.1959.10489860"),
            notes: "First EWMA control-chart formulation.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::ExponentialMovingAverage,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE,
        ),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "lambda smoothing constant; control limit; warmup window.",
        },
        duplicate_group: DuplicateGroupId(2),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 3. CUSUM.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(3),
        display_name: "CUSUM (cumulative sum) chart",
        aliases: &["Page CUSUM", "cumulative sum control chart"],
        source_refs: &[SourceRef {
            citation_key: "page1954",
            title: "Continuous Inspection Schemes",
            authors: "Page, E. S.",
            year: 1954,
            venue_or_source: "Biometrika 41(1/2): 100-115",
            doi_or_url: Some("https://doi.org/10.1093/biomet/41.1-2.100"),
            notes: "Foundational sequential change-detection chart.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::CumulativeSum,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "reference value k; decision threshold h; one- vs two-sided.",
        },
        duplicate_group: DuplicateGroupId(3),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 4. Page-Hinkley test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(4),
        display_name: "Page-Hinkley test",
        aliases: &["Hinkley test", "cumulative mean-shift detector"],
        source_refs: &[SourceRef {
            citation_key: "hinkley1971",
            title: "Inference about the Change-Point from Cumulative Sum Tests",
            authors: "Hinkley, David V.",
            year: 1971,
            venue_or_source: "Biometrika 58(3): 509-523",
            doi_or_url: Some("https://doi.org/10.1093/biomet/58.3.509"),
            notes: "Sequential change-point inference building on Page CUSUM.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::CumulativeSum,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "drift magnitude delta; alarm threshold lambda.",
        },
        duplicate_group: DuplicateGroupId(4),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(3)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 5. Hotelling T-squared.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(5),
        display_name: "Hotelling T-squared",
        aliases: &["Hotelling's T2", "multivariate T-square", "T2 statistic"],
        source_refs: &[SourceRef {
            citation_key: "hotelling1931",
            title: "The Generalization of Student's Ratio",
            authors: "Hotelling, Harold",
            year: 1931,
            venue_or_source: "Annals of Mathematical Statistics 2(3): 360-378",
            doi_or_url: Some("https://doi.org/10.1214/aoms/1177732979"),
            notes: "Multivariate analogue of Student's t; foundation of MVA SPC.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::CHEMOMETRICS | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::MultivariateHypothesis,
        mathematical_form: MathFormId::HotellingTSquared,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::BASELINE_WINDOW
                | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ProjectionResidualFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "feature dimensionality p; control limit at chosen alpha.",
        },
        duplicate_group: DuplicateGroupId(5),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 6. Robust z / MAD.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(6),
        display_name: "Robust z-score (median / MAD)",
        aliases: &[
            "MAD outlier detector",
            "median-MAD z",
            "robust z-score",
            "MAD-z",
        ],
        source_refs: &[SourceRef {
            citation_key: "iglewicz1993",
            title: "How to Detect and Handle Outliers",
            authors: "Iglewicz, Boris; Hoaglin, David C.",
            year: 1993,
            venue_or_source: "ASQC Quality Press (book)",
            doi_or_url: None,
            notes: "Modified-z robust outlier method using median + MAD.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR
                | DomainTagSet::TIME_SERIES
                | DomainTagSet::INDUSTRIAL
                | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::RobustStandardisedDeviation,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "window length; modified-z threshold (often 3.5).",
        },
        duplicate_group: DuplicateGroupId(6),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(1)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 7. Hampel filter.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(7),
        display_name: "Hampel filter",
        aliases: &["Hampel outlier filter", "median-MAD rolling filter"],
        source_refs: &[SourceRef {
            citation_key: "hampel1974",
            title: "The Influence Curve and Its Role in Robust Estimation",
            authors: "Hampel, Frank R.",
            year: 1974,
            venue_or_source: "Journal of the American Statistical Association 69(346): 383-393",
            doi_or_url: Some("https://doi.org/10.1080/01621459.1974.10482962"),
            notes: "Foundational paper on robust influence functions; the Hampel-filter rolling form is the practical detector.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL | DomainTagSet::MEDICAL,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::RobustStandardisedDeviation,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "rolling window half-width; sigma multiplier (typically 3).",
        },
        duplicate_group: DuplicateGroupId(7),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(6)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 8. Kolmogorov-Smirnov two-sample test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(8),
        display_name: "Kolmogorov-Smirnov two-sample test",
        aliases: &["KS test", "two-sample KS", "Smirnov test"],
        source_refs: &[
            SourceRef {
                citation_key: "kolmogorov1933",
                title: "Sulla Determinazione Empirica di una Legge di Distribuzione",
                authors: "Kolmogorov, Andrey N.",
                year: 1933,
                venue_or_source: "Giornale dell'Istituto Italiano degli Attuari 4: 83-91",
                doi_or_url: None,
                notes: "Foundational empirical-distribution-function statistic.",
            },
            SourceRef {
                citation_key: "smirnov1948",
                title: "Table for Estimating the Goodness of Fit of Empirical Distributions",
                authors: "Smirnov, Nikolai V.",
                year: 1948,
                venue_or_source: "Annals of Mathematical Statistics 19(2): 279-281",
                doi_or_url: Some("https://doi.org/10.1214/aoms/1177730256"),
                notes: "Two-sample extension and critical-value tables.",
            },
        ],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::KolmogorovSmirnov,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicConditional,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference CDF; critical-value alpha.",
        },
        duplicate_group: DuplicateGroupId(8),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 9. KL divergence.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(9),
        display_name: "Kullback-Leibler divergence",
        aliases: &["relative entropy", "KL divergence", "I-divergence"],
        source_refs: &[SourceRef {
            citation_key: "kullback1951",
            title: "On Information and Sufficiency",
            authors: "Kullback, Solomon; Leibler, Richard A.",
            year: 1951,
            venue_or_source: "Annals of Mathematical Statistics 22(1): 79-86",
            doi_or_url: Some("https://doi.org/10.1214/aoms/1177729694"),
            notes: "Foundational definition of relative entropy / KL divergence.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES | DomainTagSet::CATEGORICAL,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::KullbackLeibler,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference distribution; numeric epsilon for log(0) handling.",
        },
        duplicate_group: DuplicateGroupId(9),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 10. Maximum Mean Discrepancy.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(10),
        display_name: "Maximum Mean Discrepancy (MMD)",
        aliases: &["MMD", "kernel two-sample test", "RKHS distance"],
        source_refs: &[SourceRef {
            citation_key: "gretton2012",
            title: "A Kernel Two-Sample Test",
            authors: "Gretton, Arthur; Borgwardt, Karsten M.; Rasch, Malte J.; Schoelkopf, Bernhard; Smola, Alexander",
            year: 2012,
            venue_or_source: "Journal of Machine Learning Research 13: 723-773",
            doi_or_url: None,
            notes: "Modern unbiased empirical-MMD formulation with kernel embedding.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::MaximumMeanDiscrepancy,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicConditional,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "kernel choice (RBF / linear / etc.); kernel bandwidth; reference sample.",
        },
        duplicate_group: DuplicateGroupId(10),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 11. Mann-Kendall trend test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(11),
        display_name: "Mann-Kendall trend test",
        aliases: &["Mann-Kendall tau", "MK trend test", "MK statistic"],
        source_refs: &[
            SourceRef {
                citation_key: "mann1945",
                title: "Nonparametric Tests Against Trend",
                authors: "Mann, Henry B.",
                year: 1945,
                venue_or_source: "Econometrica 13(3): 245-259",
                doi_or_url: Some("https://doi.org/10.2307/1907187"),
                notes: "Foundational rank-based trend test.",
            },
            SourceRef {
                citation_key: "kendall1948",
                title: "Rank Correlation Methods",
                authors: "Kendall, Maurice G.",
                year: 1948,
                venue_or_source: "Charles Griffin & Co. (book)",
                doi_or_url: None,
                notes: "Extension and standardisation of the rank-trend statistic.",
            },
        ],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL | DomainTagSet::CHEMOMETRICS,
        ),
        primitive_family: PrimitiveFamily::RankStatistic,
        mathematical_form: MathFormId::MannKendallRank,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::RankStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "series length; significance threshold.",
        },
        duplicate_group: DuplicateGroupId(11),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 12. FFT band-energy anomaly.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(12),
        display_name: "FFT band-energy anomaly",
        aliases: &["band-power anomaly", "frequency-band energy detector"],
        source_refs: &[SourceRef {
            citation_key: "cooley1965",
            title: "An Algorithm for the Machine Calculation of Complex Fourier Series",
            authors: "Cooley, James W.; Tukey, John W.",
            year: 1965,
            venue_or_source: "Mathematics of Computation 19(90): 297-301",
            doi_or_url: Some("https://doi.org/10.2307/2003354"),
            notes: "FFT algorithm; the band-energy detector is the practical signal-processing application.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL | DomainTagSet::RF_COMMS,
        ),
        primitive_family: PrimitiveFamily::Spectral,
        mathematical_form: MathFormId::FftBandEnergy,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_3_SLEW_SHOCK),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SpectralFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "FFT window size; frequency band [f_lo, f_hi]; energy threshold.",
        },
        duplicate_group: DuplicateGroupId(12),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 13. Missingness spike.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(13),
        display_name: "Missingness spike",
        aliases: &["null-rate spike", "NULL burst detector"],
        source_refs: &[SourceRef {
            citation_key: "rubin1976",
            title: "Inference and Missing Data",
            authors: "Rubin, Donald B.",
            year: 1976,
            venue_or_source: "Biometrika 63(3): 581-592",
            doi_or_url: Some("https://doi.org/10.1093/biomet/63.3.581"),
            notes: "Foundational missingness-mechanism theory; the spike detector is engineering practice over MAR/MNAR patterns.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::MISSINGNESS | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::Missingness,
        mathematical_form: MathFormId::MissingnessAggregate,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::MISSINGNESS_MASK | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SchemaChange,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::MissingnessFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "rolling-window null-rate baseline; spike multiplier.",
        },
        duplicate_group: DuplicateGroupId(13),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 14. Latency ramp (debug / observability primitive).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(14),
        display_name: "Latency ramp",
        aliases: &["sustained-latency-elevation detector", "latency creep"],
        source_refs: &[SourceRef {
            citation_key: "dsfb_gpu_debug",
            title: "DSFB-GPU-Debug v0 detector taxonomy",
            authors: "de Beer, Riaan",
            year: 2026,
            venue_or_source: "engineering practice (DSFB-GPU-Debug crate)",
            doi_or_url: None,
            notes: "Debug / observability primitive: a sustained-latency-elevation witness over a windowed residual accumulator. Bank-motif anchor for trace catalogs.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::EVENT_STREAM,
        ),
        primitive_family: PrimitiveFamily::DebugObservability,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::PersistenceGated,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE
                | AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE
                | AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::SingleWindowSpike,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L6_CpuGpuByteEquivalent,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "baseline window; latency threshold (Q16.16); persistence count.",
        },
        duplicate_group: DuplicateGroupId(14),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(1), DetectorCanonicalId(2)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 15. Single-window spike confuser (negative witness).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(15),
        display_name: "Single-window spike confuser",
        aliases: &["one-window spike rejector", "transient-spike confuser"],
        source_refs: &[SourceRef {
            citation_key: "dsfb_gpu_debug_confusers",
            title: "DSFB-GPU-Debug v0 confuser taxonomy",
            authors: "de Beer, Riaan",
            year: 2026,
            venue_or_source: "engineering practice (DSFB-GPU-Debug heuristics bank)",
            doi_or_url: None,
            notes: "Anti-hallucination witness: fires to BLOCK an admission when an episode candidate is supported by only a single window of elevated residual. Maps to the Phase 5.6+ confuser axis of the v0 bank.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::NegativeWitness,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::PersistenceGated,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Confuser,
        negative_witness_kind: NegativeWitnessKind::SingleWindowSpikeConfuser,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_9_CONFUSER_SUPPRESSION),
        confuser_profile: ConfuserProfile::SingleWindowSpike,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L6_CpuGpuByteEquivalent,
        gpu_family: GpuFamilyKernel::NegativeWitnessFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "minimum-persistence count (typically >= 2); spike-elevation threshold.",
        },
        duplicate_group: DuplicateGroupId(15),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: SPC / process monitoring (16-25) =====
    // 16. Western Electric rules.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(16),
        display_name: "Western Electric SPC rules",
        aliases: &["WECO rules", "Western Electric run rules", "WE rules"],
        source_refs: &[SourceRef {
            citation_key: "westernelectric1956",
            title: "Statistical Quality Control Handbook",
            authors: "Western Electric Company",
            year: 1956,
            venue_or_source: "Western Electric Co., Indianapolis (book)",
            doi_or_url: None,
            notes: "Foundational set of 4 run rules for Shewhart charts (any single point > 3 sigma, 2-of-3 > 2 sigma, 4-of-5 > 1 sigma, 8-in-a-row on one side).",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::BooleanRuleAggregate,
        decision_functional: DecisionFunctional::RuleSetAny,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_7_MOTIF_CONSENSUS,
        ),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "control-limit sigma (typically 3); rule subset enabled (1..4).",
        },
        duplicate_group: DuplicateGroupId(16),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(1)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 17. Nelson rules.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(17),
        display_name: "Nelson SPC rules",
        aliases: &["Nelson run rules", "Nelson 8 rules"],
        source_refs: &[SourceRef {
            citation_key: "nelson1984",
            title: "The Shewhart Control Chart -- Tests for Special Causes",
            authors: "Nelson, Lloyd S.",
            year: 1984,
            venue_or_source: "Journal of Quality Technology 16(4): 237-239",
            doi_or_url: Some("https://doi.org/10.1080/00224065.1984.11978921"),
            notes: "Refined 8-rule extension of the Western Electric rule set.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::BooleanRuleAggregate,
        decision_functional: DecisionFunctional::RuleSetAny,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_7_MOTIF_CONSENSUS,
        ),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "control-limit sigma; rule subset enabled (1..8).",
        },
        duplicate_group: DuplicateGroupId(17),
        // Nelson is the historical successor of Western Electric:
        // it incorporates WE's rules and adds four more. We
        // record that as `derived_from = [Shewhart, WE]` plus the
        // T.4 court's `CompositionOf([Shewhart, WE])` judgment.
        // We deliberately do NOT also encode `generalizes = [WE]`:
        // that set-theoretic reading is true but adds a back-edge
        // (Nelson -> WE) that contradicts the historical-descent
        // edge (WE -> Nelson) and would create a cycle in the
        // genealogy DAG. Per panel discipline (T.5), a missing
        // edge is better than a wrong edge.
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(1), DetectorCanonicalId(16)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 18. Tukey fences.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(18),
        display_name: "Tukey fences",
        aliases: &["IQR fences", "Tukey inner/outer fences", "boxplot outlier rule"],
        source_refs: &[SourceRef {
            citation_key: "tukey1977",
            title: "Exploratory Data Analysis",
            authors: "Tukey, John W.",
            year: 1977,
            venue_or_source: "Addison-Wesley (book)",
            doi_or_url: None,
            notes: "Foundational EDA text; inner fence at +/- 1.5 IQR, outer at +/- 3 IQR.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::QuantileFence,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "baseline window length; fence multiplier k (1.5 inner, 3 outer).",
        },
        duplicate_group: DuplicateGroupId(18),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 19. PCA T-squared on score vector.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(19),
        display_name: "PCA T-squared (on score vector)",
        aliases: &["PCA T2", "Hotelling T2 on PCA scores", "principal-component T2"],
        source_refs: &[SourceRef {
            citation_key: "jackson1991",
            title: "A User's Guide to Principal Components",
            authors: "Jackson, J. Edward",
            year: 1991,
            venue_or_source: "Wiley (book)",
            doi_or_url: None,
            notes: "Standard MVA-SPC reference; combines Hotelling T2 with PCA score-space monitoring.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::CHEMOMETRICS | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::ProjectionResidual,
        mathematical_form: MathFormId::HotellingTSquared,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::BASELINE_WINDOW
                | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ProjectionResidualFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "n_components; reference covariance; control-limit alpha.",
        },
        duplicate_group: DuplicateGroupId(19),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(5)],
            generalizes: &[],
            special_case_of: &[DetectorCanonicalId(5)],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 20. PCA SPE / Q residual.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(20),
        display_name: "PCA SPE / Q residual",
        aliases: &[
            "squared prediction error",
            "Q statistic",
            "PCA residual norm",
            "SPE Q",
        ],
        source_refs: &[SourceRef {
            citation_key: "jackson1979",
            title: "Control Procedures for Residuals Associated with Principal Component Analysis",
            authors: "Jackson, J. Edward; Mudholkar, Govind S.",
            year: 1979,
            venue_or_source: "Technometrics 21(3): 341-349",
            doi_or_url: Some("https://doi.org/10.1080/00401706.1979.10489779"),
            notes: "Foundational paper on the Q / SPE residual statistic for PCA-based process monitoring.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::CHEMOMETRICS | DomainTagSet::TABULAR,
        ),
        primitive_family: PrimitiveFamily::ProjectionResidual,
        mathematical_form: MathFormId::SquaredPredictionError,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::BASELINE_WINDOW
                | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ProjectionResidualFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "n_components; reference covariance; control limit (Jackson-Mudholkar).",
        },
        duplicate_group: DuplicateGroupId(20),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 21. PLS residual.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(21),
        display_name: "PLS residual / Q on PLS",
        aliases: &["PLS Q residual", "PLS-based SPE", "PLS contribution residual"],
        source_refs: &[SourceRef {
            citation_key: "macgregor1995",
            title: "Statistical Process Control of Multivariate Processes",
            authors: "MacGregor, John F.; Kourti, Theodora",
            year: 1995,
            venue_or_source: "Control Engineering Practice 3(3): 403-414",
            doi_or_url: Some("https://doi.org/10.1016/0967-0661(95)00014-L"),
            notes: "MVA-SPC overview that crystallises the PCA/PLS Q-statistic detector for industrial processes.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::CHEMOMETRICS,
        ),
        primitive_family: PrimitiveFamily::ProjectionResidual,
        mathematical_form: MathFormId::SquaredPredictionError,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::BASELINE_WINDOW
                | InputRequirementSet::REFERENCE_DISTRIBUTION
                | InputRequirementSet::LABELS,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ProjectionResidualFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "n_latent_variables; reference X/Y; control limit.",
        },
        duplicate_group: DuplicateGroupId(21),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(20)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 22. Residual envelope exit (FDD).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(22),
        display_name: "Residual envelope exit",
        aliases: &["residual band exit", "innovation envelope detector"],
        source_refs: &[SourceRef {
            citation_key: "isermann2006",
            title: "Fault-Diagnosis Systems: An Introduction from Fault Detection to Fault Tolerance",
            authors: "Isermann, Rolf",
            year: 2006,
            venue_or_source: "Springer (book)",
            doi_or_url: Some("https://doi.org/10.1007/3-540-30368-5"),
            notes: "FDD textbook; residual-envelope and parity-space residual detectors are core observer-based methods.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::ResidualObserver,
        mathematical_form: MathFormId::EnvelopeExit,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "lower and upper envelope bounds.",
        },
        duplicate_group: DuplicateGroupId(22),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 23. Sensor bias detector.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(23),
        display_name: "Sensor bias detector",
        aliases: &["sensor offset detector", "calibration-drift detector"],
        source_refs: &[SourceRef {
            citation_key: "venkatasubramanian2003",
            title: "A Review of Process Fault Detection and Diagnosis Part III: Process History Based Methods",
            authors: "Venkatasubramanian, Venkat; Rengaswamy, Raghunathan; Yin, Kewen; Kavuri, Surya N.",
            year: 2003,
            venue_or_source: "Computers and Chemical Engineering 27(3): 327-346",
            doi_or_url: Some("https://doi.org/10.1016/S0098-1354(02)00162-X"),
            notes: "Comprehensive FDD review; sensor-bias detection covered as a slow-drift residual on an observer.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::ResidualObserver,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::PersistenceGated,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE | AxisBindingSet::AXIS_5_ENTITY_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "drift accumulation window; bias threshold; persistence count.",
        },
        duplicate_group: DuplicateGroupId(23),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 24. Actuator stiction detector.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(24),
        display_name: "Actuator stiction detector",
        aliases: &["stiction detector", "control-valve stiction signature"],
        source_refs: &[SourceRef {
            citation_key: "horch1999",
            title: "A Simple Method for Detection of Stiction in Control Valves",
            authors: "Horch, Alexander",
            year: 1999,
            venue_or_source: "Control Engineering Practice 7(10): 1221-1231",
            doi_or_url: Some("https://doi.org/10.1016/S0967-0661(99)00099-5"),
            notes: "Classical stiction-detection method based on cross-correlation between controller output and process variable.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::OperabilityDiagnostic,
        mathematical_form: MathFormId::OscillationSignature,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_3_SLEW_SHOCK | AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "cross-correlation lag window; signature threshold; minimum oscillation periods.",
        },
        duplicate_group: DuplicateGroupId(24),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 25. Valve hunting (control-loop oscillation) detector.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(25),
        display_name: "Valve hunting (control-loop oscillation) detector",
        aliases: &["loop-oscillation detector", "valve hunting signature"],
        source_refs: &[SourceRef {
            citation_key: "haegglund1995",
            title: "A Control-Loop Performance Monitor",
            authors: "Haegglund, Tore",
            year: 1995,
            venue_or_source: "Control Engineering Practice 3(11): 1543-1551",
            doi_or_url: Some("https://doi.org/10.1016/0967-0661(95)00164-P"),
            notes: "Foundational control-loop performance-monitoring method; oscillation-period regularity used to flag valve hunting.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::INDUSTRIAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::OperabilityDiagnostic,
        mathematical_form: MathFormId::OscillationSignature,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "zero-crossing-rate window; regularity threshold; min period count.",
        },
        duplicate_group: DuplicateGroupId(25),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(24)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: distribution / distance (26-33) =====
    // 26. Anderson-Darling test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(26),
        display_name: "Anderson-Darling test",
        aliases: &["A-D test", "AD statistic"],
        source_refs: &[SourceRef {
            citation_key: "anderson1952",
            title: "Asymptotic Theory of Certain Goodness-of-Fit Criteria Based on Stochastic Processes",
            authors: "Anderson, Theodore W.; Darling, Donald A.",
            year: 1952,
            venue_or_source: "Annals of Mathematical Statistics 23(2): 193-212",
            doi_or_url: Some("https://doi.org/10.1214/aoms/1177729437"),
            notes: "Weighted Cramer-von Mises statistic; heavier tail weighting than KS.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::AndersonDarling,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicConditional,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference CDF; critical-value alpha.",
        },
        duplicate_group: DuplicateGroupId(26),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(8)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 27. Cramer-von Mises test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(27),
        display_name: "Cramer-von Mises test",
        aliases: &["CvM test", "Cramer-von Mises omega-squared statistic"],
        source_refs: &[SourceRef {
            citation_key: "anderson1962",
            title: "On the Distribution of the Two-Sample Cramer-von Mises Criterion",
            authors: "Anderson, Theodore W.",
            year: 1962,
            venue_or_source: "Annals of Mathematical Statistics 33(3): 1148-1159",
            doi_or_url: Some("https://doi.org/10.1214/aoms/1177704477"),
            notes: "Two-sample integrated-squared-CDF-gap statistic; complements KS and Anderson-Darling.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::CramerVonMises,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicConditional,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference CDF; critical-value alpha.",
        },
        duplicate_group: DuplicateGroupId(27),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(8)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 28. Wasserstein distance.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(28),
        display_name: "Wasserstein / earth-mover distance",
        aliases: &["Earth Mover's Distance", "EMD", "Kantorovich-Rubinstein distance"],
        source_refs: &[SourceRef {
            citation_key: "villani2008",
            title: "Optimal Transport: Old and New",
            authors: "Villani, Cedric",
            year: 2008,
            venue_or_source: "Grundlehren der mathematischen Wissenschaften vol. 338, Springer",
            doi_or_url: Some("https://doi.org/10.1007/978-3-540-71050-9"),
            notes: "Foundational optimal-transport reference; the Wasserstein distance is the canonical OT metric.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::Wasserstein,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference distribution; transport metric (p-Wasserstein order).",
        },
        duplicate_group: DuplicateGroupId(28),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 29. Energy distance.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(29),
        display_name: "Energy distance",
        aliases: &["Szekely-Rizzo energy distance", "energy statistic"],
        source_refs: &[SourceRef {
            citation_key: "szekely2013",
            title: "Energy Statistics: A Class of Statistics Based on Distances",
            authors: "Szekely, Gabor J.; Rizzo, Maria L.",
            year: 2013,
            venue_or_source: "Journal of Statistical Planning and Inference 143(8): 1249-1272",
            doi_or_url: Some("https://doi.org/10.1016/j.jspi.2013.03.018"),
            notes: "Foundational energy-statistics paper; energy distance is a rotation-invariant distribution distance.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::EnergyDistance,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "reference sample.",
        },
        duplicate_group: DuplicateGroupId(29),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 30. Hellinger distance.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(30),
        display_name: "Hellinger distance",
        aliases: &["Bhattacharyya-Hellinger distance"],
        source_refs: &[SourceRef {
            citation_key: "hellinger1909",
            title: "Neue Begruendung der Theorie quadratischer Formen von unendlichvielen Veraenderlichen",
            authors: "Hellinger, Ernst",
            year: 1909,
            venue_or_source: "Journal fuer die reine und angewandte Mathematik 136: 210-271",
            doi_or_url: None,
            notes: "Foundational paper introducing what is now called Hellinger distance; bounded metric on probability distributions.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::CATEGORICAL | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::HellingerDistance,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "reference distribution; binning if applied to continuous data.",
        },
        duplicate_group: DuplicateGroupId(30),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 31. Population Stability Index.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(31),
        display_name: "Population Stability Index (PSI)",
        aliases: &["PSI", "population stability score", "binned KL-style drift"],
        source_refs: &[SourceRef {
            citation_key: "wu2010psi",
            title: "Population Stability Index: A Predictive Modelling Tool",
            authors: "Wu, Du; Olson, David L.",
            year: 2010,
            venue_or_source: "Service Science 2(1-2): 124-131",
            doi_or_url: Some("https://doi.org/10.1287/serv.2.1_2.124"),
            notes: "Practitioner-oriented introduction of the binned KL-style drift score widely used in credit risk and model monitoring.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::CATEGORICAL | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::PopulationStabilityIndex,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SchemaChange,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "binning strategy; PSI threshold (commonly 0.1 minor, 0.25 major).",
        },
        duplicate_group: DuplicateGroupId(31),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(9)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 32. Jensen-Shannon divergence.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(32),
        display_name: "Jensen-Shannon divergence",
        aliases: &["JS divergence", "JSD", "Jensen difference"],
        source_refs: &[SourceRef {
            citation_key: "lin1991",
            title: "Divergence Measures Based on the Shannon Entropy",
            authors: "Lin, Jianhua",
            year: 1991,
            venue_or_source: "IEEE Transactions on Information Theory 37(1): 145-151",
            doi_or_url: Some("https://doi.org/10.1109/18.61115"),
            notes: "Foundational paper defining the symmetric, bounded JS divergence.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::CATEGORICAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::JensenShannon,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "reference distribution; numeric epsilon for log(0) handling.",
        },
        duplicate_group: DuplicateGroupId(32),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(9)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 33. Total variation distance.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(33),
        display_name: "Total variation distance",
        aliases: &["TV distance", "statistical distance", "L1 distribution distance"],
        source_refs: &[SourceRef {
            citation_key: "levin2017",
            title: "Markov Chains and Mixing Times (Second Edition)",
            authors: "Levin, David A.; Peres, Yuval",
            year: 2017,
            venue_or_source: "American Mathematical Society (book)",
            doi_or_url: Some("https://doi.org/10.1090/mbk/107"),
            notes: "Modern reference treatment of total-variation distance; widely used as the canonical L1 metric on probability measures.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::CATEGORICAL,
        ),
        primitive_family: PrimitiveFamily::DistributionDistance,
        mathematical_form: MathFormId::TotalVariation,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::DistributionDistanceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "reference distribution; bin layout if continuous.",
        },
        duplicate_group: DuplicateGroupId(33),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: sequential / change-point (34-37) =====
    // 34. Pettitt test.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(34),
        display_name: "Pettitt change-point test",
        aliases: &["Pettitt non-parametric change-point test"],
        source_refs: &[SourceRef {
            citation_key: "pettitt1979",
            title: "A Non-Parametric Approach to the Change-Point Problem",
            authors: "Pettitt, A. N.",
            year: 1979,
            venue_or_source: "Journal of the Royal Statistical Society Series C 28(2): 126-135",
            doi_or_url: Some("https://doi.org/10.2307/2346729"),
            notes: "Rank-based non-parametric change-point statistic; complements parametric CUSUM-class methods.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::RankStatistic,
        mathematical_form: MathFormId::RankChangePoint,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::RankStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "series length; significance threshold.",
        },
        duplicate_group: DuplicateGroupId(34),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 35. Standard Normal Homogeneity Test (SNHT).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(35),
        display_name: "Standard Normal Homogeneity Test (SNHT)",
        aliases: &["SNHT", "Alexandersson SNHT", "Alexandersson homogeneity test"],
        source_refs: &[SourceRef {
            citation_key: "alexandersson1986",
            title: "A Homogeneity Test Applied to Precipitation Data",
            authors: "Alexandersson, Hans",
            year: 1986,
            venue_or_source: "Journal of Climatology 6(6): 661-675",
            doi_or_url: Some("https://doi.org/10.1002/joc.3370060607"),
            notes: "Foundational standard-normal homogeneity test for climatology / time-series change-point.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::HomogeneityTest,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "test-window length; critical-value threshold.",
        },
        duplicate_group: DuplicateGroupId(35),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 36. MOSUM.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(36),
        display_name: "MOSUM (moving-sum-of-residuals) test",
        aliases: &["MOSUM", "moving sum process", "Chu-Hornik-Kuan MOSUM"],
        source_refs: &[SourceRef {
            citation_key: "chu1995",
            title: "MOSUM Tests for Parameter Constancy",
            authors: "Chu, Chia-Shang J.; Hornik, Kurt; Kuan, Chung-Ming",
            year: 1995,
            venue_or_source: "Biometrika 82(3): 603-617",
            doi_or_url: Some("https://doi.org/10.1093/biomet/82.3.603"),
            notes: "Foundational MOSUM-test paper for change-point in regression / time-series parameter constancy.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL | DomainTagSet::TELEMETRY,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::MovingSumOfResiduals,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "moving window length h; significance threshold.",
        },
        duplicate_group: DuplicateGroupId(36),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(3)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 37. Buishand range.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(37),
        display_name: "Buishand range test",
        aliases: &["Buishand range statistic", "Buishand U-statistic"],
        source_refs: &[SourceRef {
            citation_key: "buishand1982",
            title: "Some Methods for Testing the Homogeneity of Rainfall Records",
            authors: "Buishand, T. A.",
            year: 1982,
            venue_or_source: "Journal of Hydrology 58(1-2): 11-27",
            doi_or_url: Some("https://doi.org/10.1016/0022-1694(82)90066-X"),
            notes: "Foundational homogeneity-range test for series with single change-point.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::INDUSTRIAL,
        ),
        primitive_family: PrimitiveFamily::SequentialRecurrence,
        mathematical_form: MathFormId::HomogeneityTest,
        decision_functional: DecisionFunctional::SequentialStopping,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SequentialRecurrenceFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "series length; critical-value threshold.",
        },
        duplicate_group: DuplicateGroupId(37),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(35)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: signal / spectral (38-40) =====
    // 38. Spectral entropy.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(38),
        display_name: "Spectral entropy",
        aliases: &["spectral Shannon entropy", "power-spectrum entropy"],
        source_refs: &[SourceRef {
            citation_key: "inouye1991",
            title: "Quantification of EEG Irregularity by Use of the Entropy of the Power Spectrum",
            authors: "Inouye, Tsuyoshi; et al.",
            year: 1991,
            venue_or_source: "Electroencephalography and Clinical Neurophysiology 79(3): 204-210",
            doi_or_url: Some("https://doi.org/10.1016/0013-4694(91)90138-T"),
            notes: "Foundational application of Shannon entropy on the power spectrum as an anomaly / irregularity metric.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES
                | DomainTagSet::MEDICAL
                | DomainTagSet::RF_COMMS
                | DomainTagSet::INDUSTRIAL,
        ),
        primitive_family: PrimitiveFamily::Spectral,
        mathematical_form: MathFormId::SpectralEntropy,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_3_SLEW_SHOCK),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SpectralFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "FFT window size; entropy-threshold baseline.",
        },
        duplicate_group: DuplicateGroupId(38),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(12)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 39. Wavelet coefficient energy.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(39),
        display_name: "Wavelet coefficient energy",
        aliases: &["wavelet energy detector", "DWT band-energy anomaly"],
        source_refs: &[SourceRef {
            citation_key: "mallat1999",
            title: "A Wavelet Tour of Signal Processing (Second Edition)",
            authors: "Mallat, Stephane",
            year: 1999,
            venue_or_source: "Academic Press (book)",
            doi_or_url: None,
            notes: "Standard reference on wavelet decomposition; the energy-per-band detector is a direct application.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES
                | DomainTagSet::MEDICAL
                | DomainTagSet::INDUSTRIAL
                | DomainTagSet::RF_COMMS,
        ),
        primitive_family: PrimitiveFamily::Wavelet,
        mathematical_form: MathFormId::WaveletCoefficientMagnitude,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_3_SLEW_SHOCK),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WaveletFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "wavelet basis; decomposition depth; band-energy threshold.",
        },
        duplicate_group: DuplicateGroupId(39),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 40. Autocorrelation break.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(40),
        display_name: "Autocorrelation-coefficient break",
        aliases: &["ACF break detector", "autocorrelation discontinuity", "ACF lag-k shift"],
        source_refs: &[SourceRef {
            citation_key: "box2015",
            title: "Time Series Analysis: Forecasting and Control (Fifth Edition)",
            authors: "Box, George E. P.; Jenkins, Gwilym M.; Reinsel, Gregory C.; Ljung, Greta M.",
            year: 2015,
            venue_or_source: "Wiley (book)",
            doi_or_url: None,
            notes: "Standard reference on time-series autocorrelation; ACF-break detection is a direct application for change in serial dependence.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TIME_SERIES | DomainTagSet::TELEMETRY | DomainTagSet::INDUSTRIAL,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::AutocorrelationBreak,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "lag k; rolling window; ACF-shift threshold.",
        },
        duplicate_group: DuplicateGroupId(40),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: debug / observability / data quality (41-48) =====
    // 41. Error burst (debug bank motif).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(41),
        display_name: "Error burst",
        aliases: &["error-rate burst", "5xx-rate burst", "error-density spike"],
        source_refs: &[SourceRef {
            citation_key: "dsfb_gpu_debug",
            title: "DSFB-GPU-Debug v0 detector taxonomy",
            authors: "de Beer, Riaan",
            year: 2026,
            venue_or_source: "engineering practice (DSFB-GPU-Debug crate)",
            doi_or_url: None,
            notes: "Debug bank motif: aggregated error-rate over a window crossing a threshold. Structural witness; not a classical academic primitive.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::EVENT_STREAM,
        ),
        primitive_family: PrimitiveFamily::DebugObservability,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::SingleWindowSpike,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L6_CpuGpuByteEquivalent,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "burst window; error-rate threshold; persistence count.",
        },
        duplicate_group: DuplicateGroupId(41),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(14)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 42. Slew shock (debug bank motif).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(42),
        display_name: "Slew shock",
        aliases: &["slew shock + recovery edge", "step-shock detector"],
        source_refs: &[SourceRef {
            citation_key: "dsfb_gpu_debug",
            title: "DSFB-GPU-Debug v0 detector taxonomy",
            authors: "de Beer, Riaan",
            year: 2026,
            venue_or_source: "engineering practice (DSFB-GPU-Debug crate)",
            doi_or_url: None,
            notes: "Debug bank motif: rapid residual step followed by recovery; structural witness for transient anomalies. Not a classical academic primitive.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::EVENT_STREAM,
        ),
        primitive_family: PrimitiveFamily::DebugObservability,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::BoundaryEdge,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::Interval,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_3_SLEW_SHOCK | AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::SingleWindowSpike,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L6_CpuGpuByteEquivalent,
        gpu_family: GpuFamilyKernel::ResidualObserverFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "step magnitude threshold; recovery window; recovery-edge tolerance.",
        },
        duplicate_group: DuplicateGroupId(42),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(14)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 43. Fanout cascade (debug bank motif).
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(43),
        display_name: "Fanout cascade",
        aliases: &["fanout precursor", "cascade-precursor detector", "service-fanout burst"],
        source_refs: &[SourceRef {
            citation_key: "dsfb_gpu_debug",
            title: "DSFB-GPU-Debug v0 detector taxonomy",
            authors: "de Beer, Riaan",
            year: 2026,
            venue_or_source: "engineering practice (DSFB-GPU-Debug crate)",
            doi_or_url: None,
            notes: "Debug bank motif: simultaneous elevated residuals across multiple entities. Structural witness for cascade-style failures.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::DEBUG | DomainTagSet::TELEMETRY | DomainTagSet::GRAPH,
        ),
        primitive_family: PrimitiveFamily::DebugObservability,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::TOPOLOGY,
        ),
        output_witness: WitnessKind::SubgraphSelection,
        witness_role: WitnessRole::Topology,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_5_ENTITY_LOCALITY | AxisBindingSet::AXIS_6_CAUSAL_ADJACENCY,
        ),
        confuser_profile: ConfuserProfile::Multiple,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L6_CpuGpuByteEquivalent,
        gpu_family: GpuFamilyKernel::GraphLocalFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "co-firing window; min-entity-count threshold; topology hop distance.",
        },
        duplicate_group: DuplicateGroupId(43),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(14)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 44. Missingness coupling.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(44),
        display_name: "Missingness coupling",
        aliases: &["paired-NULL coupling", "co-missingness detector"],
        source_refs: &[SourceRef {
            citation_key: "vanbuuren2018",
            title: "Flexible Imputation of Missing Data (Second Edition)",
            authors: "van Buuren, Stef",
            year: 2018,
            venue_or_source: "Chapman and Hall/CRC (book)",
            doi_or_url: Some("https://doi.org/10.1201/9780429492259"),
            notes: "Modern reference on missing-data mechanisms; coupling between columns flags MNAR-style patterns.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::MISSINGNESS | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::Missingness,
        mathematical_form: MathFormId::MissingnessAggregate,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::MISSINGNESS_MASK | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(
            AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_5_ENTITY_LOCALITY,
        ),
        confuser_profile: ConfuserProfile::SchemaChange,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::MissingnessFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 3,
            description: "column-pair set; coupling threshold; baseline window.",
        },
        duplicate_group: DuplicateGroupId(44),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(13)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 45. Schema drift.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(45),
        display_name: "Schema drift",
        aliases: &["schema-change detector", "column-type drift"],
        source_refs: &[SourceRef {
            citation_key: "redyuk2021",
            title: "Automating Large-Scale Data Quality Verification",
            authors: "Schelter, Sebastian; Lange, Dustin; Schmidt, Philipp; et al.",
            year: 2018,
            venue_or_source: "PVLDB 11(12): 1781-1794",
            doi_or_url: Some("https://doi.org/10.14778/3229863.3229867"),
            notes: "Reference architecture for declarative data-quality constraints (Deequ); schema-drift detection is a structural rule over column types and value domains.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::TabularConstraint,
        mathematical_form: MathFormId::SchemaPredicate,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::CATEGORICAL | InputRequirementSet::REFERENCE_DISTRIBUTION,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_5_ENTITY_LOCALITY),
        confuser_profile: ConfuserProfile::SchemaChange,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::TabularConstraintFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "reference schema; violation-rate threshold.",
        },
        duplicate_group: DuplicateGroupId(45),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 46. Cardinality drift.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(46),
        display_name: "Cardinality drift",
        aliases: &["category-cardinality drift", "distinct-value-count drift"],
        source_refs: &[SourceRef {
            citation_key: "schelter2018",
            title: "Automating Large-Scale Data Quality Verification",
            authors: "Schelter, Sebastian; Lange, Dustin; Schmidt, Philipp; et al.",
            year: 2018,
            venue_or_source: "PVLDB 11(12): 1781-1794",
            doi_or_url: Some("https://doi.org/10.14778/3229863.3229867"),
            notes: "Same Deequ reference. Cardinality drift is one of the canonical declarative-data-quality constraints over categorical columns.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::CATEGORICAL | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::CategoricalHistogram,
        mathematical_form: MathFormId::CategoricalCardinality,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::CATEGORICAL | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Distribution,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_7_MOTIF_CONSENSUS),
        confuser_profile: ConfuserProfile::SchemaChange,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::CategoricalHistogramFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "rolling-window distinct-count baseline; drift factor.",
        },
        duplicate_group: DuplicateGroupId(46),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 47. Uniqueness violation.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(47),
        display_name: "Uniqueness violation",
        aliases: &["primary-key violation", "duplicate-row detector"],
        source_refs: &[SourceRef {
            citation_key: "abedjan2015",
            title: "Profiling Relational Data: A Survey",
            authors: "Abedjan, Ziawasch; Golab, Lukasz; Naumann, Felix",
            year: 2015,
            venue_or_source: "VLDB Journal 24(4): 557-581",
            doi_or_url: Some("https://doi.org/10.1007/s00778-015-0389-y"),
            notes: "Comprehensive survey of data-profiling primitives; uniqueness-constraint violation is a foundational structural rule.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::TabularConstraint,
        mathematical_form: MathFormId::SchemaPredicate,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::CATEGORICAL | InputRequirementSet::NUMERIC,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_5_ENTITY_LOCALITY),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::TabularConstraintFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "key column(s).",
        },
        duplicate_group: DuplicateGroupId(47),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 48. Functional-dependency violation.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(48),
        display_name: "Functional-dependency violation",
        aliases: &["FD violation", "FD-rule break"],
        source_refs: &[SourceRef {
            citation_key: "abedjan2015fd",
            title: "Profiling Relational Data: A Survey",
            authors: "Abedjan, Ziawasch; Golab, Lukasz; Naumann, Felix",
            year: 2015,
            venue_or_source: "VLDB Journal 24(4): 557-581",
            doi_or_url: Some("https://doi.org/10.1007/s00778-015-0389-y"),
            notes: "Functional-dependency violation: rows that share LHS values but differ on RHS values.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::TABULAR | DomainTagSet::DATABASE,
        ),
        primitive_family: PrimitiveFamily::TabularConstraint,
        mathematical_form: MathFormId::SchemaPredicate,
        decision_functional: DecisionFunctional::AggregateThreshold,
        input_requirements: InputRequirementSet(
            InputRequirementSet::CATEGORICAL | InputRequirementSet::NUMERIC,
        ),
        output_witness: WitnessKind::BooleanRow,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_5_ENTITY_LOCALITY),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::TabularConstraintFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "FD rule (LHS columns -> RHS columns).",
        },
        duplicate_group: DuplicateGroupId(48),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(47)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // ===== T.1b expansion: medical / RF (49-54) =====
    // 49. R-peak interval anomaly.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(49),
        display_name: "R-peak interval anomaly (RR-interval)",
        aliases: &["RR-interval anomaly", "heart-rate-interval anomaly"],
        source_refs: &[SourceRef {
            citation_key: "pan1985",
            title: "A Real-Time QRS Detection Algorithm",
            authors: "Pan, Jiapu; Tompkins, Willis J.",
            year: 1985,
            venue_or_source: "IEEE Transactions on Biomedical Engineering BME-32(3): 230-236",
            doi_or_url: Some("https://doi.org/10.1109/TBME.1985.325532"),
            notes: "Foundational QRS / R-peak detection algorithm; RR-interval anomaly is a direct downstream detector.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::MEDICAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "rolling-window RR baseline; deviation threshold.",
        },
        duplicate_group: DuplicateGroupId(49),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 50. HRV time-domain shift.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(50),
        display_name: "HRV time-domain shift",
        aliases: &["heart-rate-variability time-domain detector", "SDNN/RMSSD shift"],
        source_refs: &[SourceRef {
            citation_key: "taskforce1996",
            title: "Heart Rate Variability: Standards of Measurement, Physiological Interpretation, and Clinical Use",
            authors: "Task Force of the European Society of Cardiology and the North American Society of Pacing and Electrophysiology",
            year: 1996,
            venue_or_source: "Circulation 93(5): 1043-1065",
            doi_or_url: Some("https://doi.org/10.1161/01.CIR.93.5.1043"),
            notes: "Definitive HRV standards document; time-domain indices SDNN/RMSSD and their shifts are clinically grounded detectors.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::MEDICAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::WindowStatistic,
        mathematical_form: MathFormId::WindowedResidualAccumulator,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE),
        confuser_profile: ConfuserProfile::SmallSample,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::WindowStatisticFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "epoch length (e.g. 5-min); SDNN/RMSSD shift threshold.",
        },
        duplicate_group: DuplicateGroupId(50),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(49)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 51. QRS width anomaly.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(51),
        display_name: "QRS width anomaly",
        aliases: &["QRS duration anomaly", "wide-QRS detector"],
        source_refs: &[SourceRef {
            citation_key: "pan1985_2",
            title: "A Real-Time QRS Detection Algorithm",
            authors: "Pan, Jiapu; Tompkins, Willis J.",
            year: 1985,
            venue_or_source: "IEEE Transactions on Biomedical Engineering BME-32(3): 230-236",
            doi_or_url: Some("https://doi.org/10.1109/TBME.1985.325532"),
            notes: "QRS-width measurement is a direct output of the Pan-Tompkins detector; anomaly is a scalar threshold on the width relative to a baseline.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::MEDICAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::Threshold,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 1,
            description: "QRS-width threshold (typically >= 120 ms).",
        },
        duplicate_group: DuplicateGroupId(51),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(49)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 52. ST-segment deviation proxy.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(52),
        display_name: "ST-segment deviation proxy",
        aliases: &["ST-segment elevation / depression detector"],
        source_refs: &[SourceRef {
            citation_key: "ihbk2012",
            title: "Detection of ECG ST-Segment Episodes Using Spline-Based Methods",
            authors: "Garcia, Joaquin; Sornmo, Leif; Olmos, Salvador; Laguna, Pablo",
            year: 2000,
            venue_or_source: "Physiological Measurement 21(2): 343-356",
            doi_or_url: Some("https://doi.org/10.1088/0967-3334/21/2/308"),
            notes: "ST-segment deviation detection; literature-grounded ECG screening proxy. Diagnostic interpretation is out of scope.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::MEDICAL | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::Threshold,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC | InputRequirementSet::ORDERED_TIME,
        ),
        output_witness: WitnessKind::BooleanCell,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "ST-baseline reference; deviation threshold (e.g. 0.1 mV).",
        },
        duplicate_group: DuplicateGroupId(52),
        genealogy: GenealogyEdges {
            derived_from: &[DetectorCanonicalId(49)],
            generalizes: &[],
            special_case_of: &[],
            is_origin: false,
        },
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 53. Carrier-frequency-offset residual.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(53),
        display_name: "Carrier-frequency-offset residual",
        aliases: &["CFO residual", "carrier-offset detector"],
        source_refs: &[SourceRef {
            citation_key: "morelli1999",
            title: "An Improved Frequency Offset Estimator for OFDM Applications",
            authors: "Morelli, Michele; Mengali, Umberto",
            year: 1999,
            venue_or_source: "IEEE Communications Letters 3(3): 75-77",
            doi_or_url: Some("https://doi.org/10.1109/4234.752907"),
            notes: "Reference CFO estimator for OFDM; residual after compensation is the natural anomaly detector.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::RF_COMMS | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::Spectral,
        mathematical_form: MathFormId::Threshold,
        decision_functional: DecisionFunctional::TwoSided,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::REGULAR_SAMPLING,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_3_SLEW_SHOCK),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::SpectralFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "offset-estimator window; residual threshold (Hz).",
        },
        duplicate_group: DuplicateGroupId(53),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
    // 54. EVM anomaly.
    LiteratureDetector {
        canonical_id: DetectorCanonicalId(54),
        display_name: "Error Vector Magnitude (EVM) anomaly",
        aliases: &["EVM detector", "constellation-error-vector anomaly"],
        source_refs: &[SourceRef {
            citation_key: "shafik2006",
            title: "On the Extended Relationships Among EVM, BER and SNR as Performance Metrics",
            authors: "Shafik, Rishad A.; Rahman, Md. Shahriar; Islam, AHM Razibul",
            year: 2006,
            venue_or_source: "International Conference on Electrical and Computer Engineering: 408-411",
            doi_or_url: Some("https://doi.org/10.1109/ICECE.2006.355657"),
            notes: "Standard treatment of EVM as a constellation-error-vector magnitude metric; anomaly is a threshold over rolling EVM.",
        }],
        origin_domains: DomainTagSet(
            DomainTagSet::RF_COMMS | DomainTagSet::TIME_SERIES,
        ),
        primitive_family: PrimitiveFamily::ScalarThreshold,
        mathematical_form: MathFormId::Threshold,
        decision_functional: DecisionFunctional::OneSidedUpper,
        input_requirements: InputRequirementSet(
            InputRequirementSet::NUMERIC
                | InputRequirementSet::ORDERED_TIME
                | InputRequirementSet::BASELINE_WINDOW,
        ),
        output_witness: WitnessKind::ScalarMargin,
        witness_role: WitnessRole::Primary,
        negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
        fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
        confuser_profile: ConfuserProfile::None,
        deterministic_status: DeterministicStatus::DeterministicNative,
        implementation_status: ImplementationLevel::L1_Canonicalised,
        gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
        parameter_bounds: ParameterBounds {
            axis_count: 2,
            description: "EVM-window length; threshold (% RMS).",
        },
        duplicate_group: DuplicateGroupId(54),
        genealogy: ORIGIN,
        usefulness: UNMEASURED,
        lifecycle_state: LifecycleState::Active,
        constitution_compliance: ALL_TRUE,
    },
];
