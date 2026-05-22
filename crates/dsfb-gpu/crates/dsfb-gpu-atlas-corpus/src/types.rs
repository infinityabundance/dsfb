//! Structural schema for the literature detector corpus and the
//! Detector Canonicalisation Court.
//!
//! This file is the load-bearing public surface of Section T.1: every
//! type the panel verdict named lives here so future T.2..T.9 commits
//! can populate them without restructuring. The schema is intentionally
//! verbose — each enum variant and bitset flag is a deliberate
//! deduplication-vs-distinct signal in the court.
//!
//! Why structs and `&'static` slices rather than `Vec` and `String`:
//! the seed corpus lives in a `const` table in [`crate::seed`] so two
//! builds on different machines produce byte-identical bytes. Anything
//! that owns a `String` would defeat that property.
//!
//! Why the 32-bit `DetectorCanonicalId` newtype lives here while the
//! five-hash identity lives in [`crate::identity`] (T.3): the schema
//! layer needs a stable handle every seed entry can reference,
//! independently of how the formal `detector_identity_hash` is
//! composed. Identity-hash bytes never round-trip through schema
//! data, so the two layers can evolve without touching seed-table
//! bytes.

/// Stable identifier for a canonical literature detector record.
///
/// Two builds on different machines produce the same set of
/// `DetectorCanonicalId` values for the same seed table, because the
/// IDs are derived from a stable hash over the canonical-form bytes,
/// not from insertion order. T.3's five-hash identity
/// (`source_hash` / `formula_hash` / `parameter_hash` /
/// `implementation_hash` / `semantic_role_hash` composed into
/// `detector_identity_hash`) lives in [`crate::identity`]; the 32-bit
/// `DetectorCanonicalId` remains the schema-level handle used by every
/// court / genealogy / ledger consumer. Equality is by handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectorCanonicalId(pub u32);

/// Identifier of a duplicate-equivalence class in the dedup court.
///
/// Every literature claim is assigned a `DuplicateGroupId`: aliases
/// of the same canonical primitive share one group, and the canonical
/// representative carries the same group ID as its aliases. The
/// dedup court machinery lives in [`crate::court`] (T.4); this type
/// is the schema-level handle every court consumer reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DuplicateGroupId(pub u32);

/// Alias-side identifier used by the dedup court to track per-alias
/// records distinct from their canonical representative's ID.
///
/// Most aliases collapse into their canonical (via [`DedupRecord`])
/// and never need a fresh canonical ID; the alias ID is the handle
/// used inside court records to refer to the alias without conflating
/// it with the canonical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetectorAliasId(pub u32);

/// Citation / provenance record for a literature primitive.
///
/// Every detector entry MUST carry at least one `SourceRef` (or an
/// explicit engineering-practice note in `notes`). The verifier (see
/// [`crate::verify`]) enforces this; the constraint is what
/// distinguishes a provenance-bearing court from a detector zoo.
#[derive(Debug, Clone, Copy)]
pub struct SourceRef {
    /// Short citation key used by paper bibliographies (e.g. `shewhart1924`).
    pub citation_key: &'static str,
    /// Human-readable title (paper title, book chapter, standard, etc.).
    pub title: &'static str,
    /// Authors as a single canonical-form string (e.g. `"de Beer, Riaan"`
    /// or `"Roberts, S. W."`). Multi-author citations use `"; "` as the
    /// separator so canonical ordering is unambiguous.
    pub authors: &'static str,
    /// Year of publication. `0` indicates "engineering practice with no
    /// dated primary source" and triggers the `notes` field requirement
    /// in the verifier.
    pub year: u16,
    /// Venue, journal, conference, or general-source descriptor
    /// (e.g. `"Bell System Technical Journal"`, `"NIST FIPS 180-4"`,
    /// `"engineering practice"`).
    pub venue_or_source: &'static str,
    /// Optional DOI or stable URL (omit if not yet deposited).
    pub doi_or_url: Option<&'static str>,
    /// Free-form provenance note; mandatory when `year == 0`.
    pub notes: &'static str,
}

/// Coarse primitive-family classification.
///
/// This is a small, stable taxonomy: every literature primitive must
/// fit one of these families. New families are added by commit, not
/// by mutation. The variants are ordered for canonical sorting (T.9
/// reports + future T.10 `corpus_hash_v1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrimitiveFamily {
    /// Static or near-static thresholds on a scalar feature
    /// (Shewhart-style control charts, Tukey fences, z-thresholds).
    ScalarThreshold,
    /// Sliding-window statistics over a fixed-width window
    /// (rolling mean, rolling median, rolling MAD, Hampel filter).
    WindowStatistic,
    /// Sequential recurrence over time-ordered observations
    /// (EWMA, CUSUM, Page-Hinkley, GLR change detection).
    SequentialRecurrence,
    /// Distribution-distance / divergence measures
    /// (KS, KL, JS, MMD, Wasserstein, energy distance, PSI).
    DistributionDistance,
    /// Rank or order statistics
    /// (Mann-Kendall, Pettitt, Theil-Sen slope, robust-rank tests).
    RankStatistic,
    /// Spectral / frequency-domain detectors
    /// (FFT band energy, spectral centroid shift, spectral entropy).
    Spectral,
    /// Wavelet-domain detectors
    /// (DWT coefficient burst, wavelet packet energy).
    Wavelet,
    /// Graph-local detectors (per-node anomalies on a graph topology).
    GraphLocal,
    /// Graph-global detectors (whole-graph structural anomalies).
    GraphGlobal,
    /// Tabular-constraint detectors
    /// (functional-dependency violation, uniqueness violation).
    TabularConstraint,
    /// Categorical-histogram detectors
    /// (cardinality drift, category emergence, category collapse).
    CategoricalHistogram,
    /// Missingness-pattern detectors
    /// (missingness spike, missingness coupling, null-run anomaly).
    Missingness,
    /// Residual / observer / parity-space detectors from FDD literature.
    ResidualObserver,
    /// PCA / PLS / latent-projection residual detectors (SPE, T2).
    ProjectionResidual,
    /// Multivariate hypothesis-testing detectors (Hotelling T2, MEWMA).
    MultivariateHypothesis,
    /// Information-theoretic detectors (entropy break, mutual-info shift).
    InformationTheory,
    /// Operability-derived detectors (oscillation, valve hunting,
    /// stiction). These are FDD-adjacent but distinct enough in
    /// the literature to warrant their own family.
    OperabilityDiagnostic,
    /// Debug / observability primitives that originated outside the
    /// classical detector literature but operate over the same
    /// residual-evidence shape (latency ramp, fanout cascade).
    DebugObservability,
    /// "Negative" / anti-hallucination witnesses that fire to say
    /// "do not trust this episode" (single-window spike confuser,
    /// missingness-artifact confuser, schema-change confuser).
    NegativeWitness,
}

/// Coarse mathematical-form classifier used for dedup against the
/// `formula_hash` lane of the (future) five-hash identity.
///
/// Two detectors with the same `MathFormId` and the same parameter
/// grid are candidates for dedup; two with different `MathFormId`
/// values are NOT dedup'd even when their domain framing looks
/// similar. The court's `CanonicalisationDecision` is what actually
/// pins the call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MathFormId {
    /// `x > threshold` or `|x| > threshold`.
    Threshold,
    /// `(x - mu) / sigma > threshold` (z-score family).
    StandardisedDeviation,
    /// Robust z: `(x - median) / (k * MAD) > threshold`.
    RobustStandardisedDeviation,
    /// EWMA recurrence with control limit.
    ExponentialMovingAverage,
    /// CUSUM / Page-Hinkley one-sided sequential test.
    CumulativeSum,
    /// Generalised likelihood ratio over a window.
    GeneralisedLikelihoodRatio,
    /// Hotelling T2 quadratic form on the score vector.
    HotellingTSquared,
    /// PCA squared-prediction-error / Q statistic.
    SquaredPredictionError,
    /// Kolmogorov-Smirnov D statistic (supremum of CDF gap).
    KolmogorovSmirnov,
    /// Kullback-Leibler divergence.
    KullbackLeibler,
    /// Jensen-Shannon divergence.
    JensenShannon,
    /// Maximum mean discrepancy (kernel embedding distance).
    MaximumMeanDiscrepancy,
    /// Mann-Kendall rank trend statistic.
    MannKendallRank,
    /// Wavelet coefficient magnitude over a band.
    WaveletCoefficientMagnitude,
    /// FFT band-energy ratio or anomaly.
    FftBandEnergy,
    /// Boolean missingness-mask aggregator over a window.
    MissingnessAggregate,
    /// Boolean schema-violation predicate over a row.
    SchemaPredicate,
    /// Cumulative residual over a sliding window
    /// (latency ramp, error burst).
    WindowedResidualAccumulator,
    /// Boolean aggregate over a rule set (Western Electric / Nelson
    /// SPC rule families). The statistic is the count of individual
    /// rule predicates that fire over the window.
    BooleanRuleAggregate,
    /// Quantile-fence threshold: `|x - q_median| > k * IQR` (Tukey
    /// fences). Robust against non-normal tails; distinct from the
    /// MAD-based RobustStandardisedDeviation form.
    QuantileFence,
    /// Anderson-Darling weighted-CDF statistic (heavier tail weight
    /// than KS).
    AndersonDarling,
    /// Cramer-von Mises integrated-squared-CDF-gap statistic.
    CramerVonMises,
    /// Wasserstein / earth-mover distance.
    Wasserstein,
    /// Energy distance (Szekely-Rizzo).
    EnergyDistance,
    /// Hellinger distance (bounded distribution-distance metric).
    HellingerDistance,
    /// Total variation distance.
    TotalVariation,
    /// Population Stability Index — categorical/binned KL-like drift.
    PopulationStabilityIndex,
    /// Pettitt rank-based change-point statistic.
    RankChangePoint,
    /// Homogeneity-test statistic (SNHT, Buishand range, structural
    /// homogeneity over the series).
    HomogeneityTest,
    /// Moving-sum-of-residuals statistic (MOSUM).
    MovingSumOfResiduals,
    /// Spectral entropy of the power spectrum.
    SpectralEntropy,
    /// Autocorrelation-coefficient break at a known lag.
    AutocorrelationBreak,
    /// Oscillation-signature aggregate (mean-crossing rate, period
    /// regularity, amplitude envelope). Used by stiction / valve-
    /// hunting detectors.
    OscillationSignature,
    /// Categorical-cardinality drift over a window.
    CategoricalCardinality,
    /// Residual-envelope exit: `residual outside [lo, hi] envelope`.
    EnvelopeExit,
}

/// Decision-functional shape: what the detector actually computes to
/// emit a boolean firing decision.
///
/// `MathFormId` says *what* the test statistic is; `DecisionFunctional`
/// says *how* it's compared to produce a firing bit. Two detectors
/// with identical `MathFormId` but different decision functionals
/// (e.g. one-sided vs two-sided) are NOT dedup'd; the difference
/// changes which evidence the witness produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecisionFunctional {
    /// `statistic > threshold` (upper-tail firing).
    OneSidedUpper,
    /// `statistic < threshold` (lower-tail firing).
    OneSidedLower,
    /// `|statistic| > threshold` (two-sided).
    TwoSided,
    /// Sequential-stopping rule: fire when cumulative statistic
    /// crosses the control limit.
    SequentialStopping,
    /// Persistence-gated: fire only after the underlying predicate
    /// has held for N consecutive windows.
    PersistenceGated,
    /// Boundary-edge: fire on the transition from inactive to active
    /// (recovery-edge witness style).
    BoundaryEdge,
    /// Aggregate over a window then threshold (count, sum, max).
    AggregateThreshold,
    /// Fire if any one of a fixed rule set fires (Western Electric /
    /// Nelson rule aggregation; OR of N independent rule predicates).
    RuleSetAny,
}

/// Bitset of input-contract requirements.
///
/// Each flag is a structurally distinct dedup signal: two detectors
/// that share a formula but require different inputs (one needs
/// ordered time, the other doesn't) are NOT dedup'd, because the
/// input contract is part of the witness's identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InputRequirementSet(pub u32);

impl InputRequirementSet {
    /// Bit position for `requires_ordered_time`.
    pub const ORDERED_TIME: u32 = 1 << 0;
    /// Bit position for `requires_regular_sampling`.
    pub const REGULAR_SAMPLING: u32 = 1 << 1;
    /// Bit position for `requires_numeric`.
    pub const NUMERIC: u32 = 1 << 2;
    /// Bit position for `requires_categorical`.
    pub const CATEGORICAL: u32 = 1 << 3;
    /// Bit position for `requires_graph`.
    pub const GRAPH: u32 = 1 << 4;
    /// Bit position for `requires_baseline_window`.
    pub const BASELINE_WINDOW: u32 = 1 << 5;
    /// Bit position for `requires_reference_distribution`.
    pub const REFERENCE_DISTRIBUTION: u32 = 1 << 6;
    /// Bit position for `requires_labels`.
    pub const LABELS: u32 = 1 << 7;
    /// Bit position for `requires_seasonality_period`.
    pub const SEASONALITY_PERIOD: u32 = 1 << 8;
    /// Bit position for `requires_topology`.
    pub const TOPOLOGY: u32 = 1 << 9;
    /// Bit position for `requires_units`.
    pub const UNITS: u32 = 1 << 10;
    /// Bit position for `requires_missingness_mask`.
    pub const MISSINGNESS_MASK: u32 = 1 << 11;

    /// True if the set has at least one requirement declared. The
    /// verifier rejects detectors with `InputRequirementSet(0)`
    /// because every literature primitive operates over *some*
    /// input contract; the empty set is a schema error, not a
    /// "no requirements" claim.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Output-witness type — what evidence the detector emits when it fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessKind {
    /// Boolean firing bit at the cell granularity (per entity per window).
    BooleanCell,
    /// Boolean firing bit at the row granularity (per row / sample).
    BooleanRow,
    /// Q16.16 scalar margin (signed signed-margin output, used by
    /// the v0 detector cells).
    ScalarMargin,
    /// Interval / span over time or rows (start-end pair).
    Interval,
    /// Subgraph or vertex set (graph-local detectors).
    SubgraphSelection,
    /// Categorical label assignment.
    CategoryLabel,
    /// Histogram delta (categorical or numeric).
    HistogramDelta,
}

/// Witness role in the fusion court.
///
/// Multiple detectors that emit the same `WitnessKind` may play
/// different roles: one fires as a `Primary` witness to *support*
/// an episode, while another fires as a `Confuser` to *block* the
/// same episode under the same evidence. The fusion layer (Section
/// S) uses these roles to produce human-legible verdicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessRole {
    /// Direct primary witness for an episode motif.
    Primary,
    /// Corroborating evidence for a primary witness.
    Corroborating,
    /// Negative witness: fires to BLOCK admission of an episode.
    Confuser,
    /// Boundary witness: fires at episode start or end transitions.
    Boundary,
    /// Clean-window stability witness: fires when there is NO episode.
    CleanWindow,
    /// Recovery witness: fires on episode closure.
    Recovery,
    /// Timing-only witness: fires on temporal alignment, not magnitude.
    Timing,
    /// Distribution-shift witness (no per-cell semantics).
    Distribution,
    /// Topology witness (graph-structural).
    Topology,
    /// Causality / lead-lag proxy witness.
    CausalityProxy,
}

/// Sub-classifier for "negative witnesses" — detectors whose ONLY
/// purpose is to suppress an admission that would otherwise look
/// plausible.
///
/// The panel calls these the "anti-hallucination" witnesses. They
/// are listed separately from the `WitnessRole::Confuser` lane so
/// the court can record *why* a confuser fired, not just *that* it
/// fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NegativeWitnessKind {
    /// Sample size below the threshold the primitive needs.
    SmallSampleConfuser,
    /// A single-window spike that doesn't persist long enough to
    /// admit a Real episode.
    SingleWindowSpikeConfuser,
    /// Firing aligned with a known periodic boundary (e.g. exact
    /// hour-boundary spikes in trace catalogs).
    PeriodicBoundaryConfuser,
    /// Firing caused by a missingness artifact, not by real signal.
    MissingnessArtifactConfuser,
    /// Firing caused by a schema change (column type flipped, new
    /// category emerged) rather than by anomaly.
    SchemaChangeConfuser,
    /// Firing caused by a unit-scale change (Q16.16 saturation,
    /// f64 NaN, ms→µs unit flip).
    UnitScaleChangeConfuser,
    /// Firing aligned with a deployment / build marker.
    DeploymentMarkerConfuser,
    /// Firing caused by clock skew between sources.
    ClockSkewConfuser,
    /// Firing aligned with batch-boundary effects (BLOB ingest,
    /// hourly partition rollover).
    BatchBoundaryConfuser,
    /// Sentinel used by detectors that are NOT negative witnesses;
    /// distinguishes "no negative role" from "missing field".
    NotANegativeWitness,
}

/// Bitset of fusion-axis bindings.
///
/// Each detector contributes evidence to one or more fusion axes.
/// The declarative axis-to-plane mapping lives in
/// [`crate::fusion::axes_to_planes`] (T.6); runtime fusion-engine
/// semantics belong to Section S. The bit positions match the v1
/// 9-axis fusion labels (axes 1..9) so the existing dsfb-gpu-debug
/// bank can be
/// mapped onto the corpus without renumbering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct AxisBindingSet(pub u16);

impl AxisBindingSet {
    /// Axis 1: residual magnitude.
    pub const AXIS_1_RESIDUAL_MAGNITUDE: u16 = 1 << 0;
    /// Axis 2: drift persistence.
    pub const AXIS_2_DRIFT_PERSISTENCE: u16 = 1 << 1;
    /// Axis 3: slew shock.
    pub const AXIS_3_SLEW_SHOCK: u16 = 1 << 2;
    /// Axis 4: temporal locality.
    pub const AXIS_4_TEMPORAL_LOCALITY: u16 = 1 << 3;
    /// Axis 5: entity locality.
    pub const AXIS_5_ENTITY_LOCALITY: u16 = 1 << 4;
    /// Axis 6: causal / topological adjacency.
    pub const AXIS_6_CAUSAL_ADJACENCY: u16 = 1 << 5;
    /// Axis 7: detector motif consensus.
    pub const AXIS_7_MOTIF_CONSENSUS: u16 = 1 << 6;
    /// Axis 8: bank semantic admissibility (CPU-only).
    pub const AXIS_8_BANK_ADMISSIBILITY: u16 = 1 << 7;
    /// Axis 9: confuser suppression (CPU-only).
    pub const AXIS_9_CONFUSER_SUPPRESSION: u16 = 1 << 8;

    /// True if at least one axis is bound. Verifier rejects empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Bitset of domain tags — where the primitive originated and where
/// it is reasonable to activate it.
///
/// A detector with `Tabular | TimeSeries` set is applicable to either
/// domain; a detector with only `Industrial` set is the natural
/// fit only for FDD / industrial telemetry. The activation planner
/// (Section S) uses this bitset to gate detectors against the input
/// manifest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DomainTagSet(pub u16);

impl DomainTagSet {
    /// Debug / trace catalogs (the dsfb-gpu-debug fixture domain).
    pub const DEBUG: u16 = 1 << 0;
    /// Distributed-system telemetry (OTel spans, Prometheus metrics).
    pub const TELEMETRY: u16 = 1 << 1;
    /// Generic tabular data (rows × columns).
    pub const TABULAR: u16 = 1 << 2;
    /// Univariate or multivariate time-series.
    pub const TIME_SERIES: u16 = 1 << 3;
    /// Graph / topology data.
    pub const GRAPH: u16 = 1 << 4;
    /// Industrial process monitoring / FDD.
    pub const INDUSTRIAL: u16 = 1 << 5;
    /// Categorical-only data (no useful numeric features).
    pub const CATEGORICAL: u16 = 1 << 6;
    /// Pure missingness mask (the categorical/numeric data is absent
    /// and only the missingness pattern is interrogated).
    pub const MISSINGNESS: u16 = 1 << 7;
    /// Continuous event stream (vs windowed batch).
    pub const EVENT_STREAM: u16 = 1 << 8;
    /// Medical / biosignal domain.
    pub const MEDICAL: u16 = 1 << 9;
    /// RF / communications signals.
    pub const RF_COMMS: u16 = 1 << 10;
    /// Chemometrics / lab data.
    pub const CHEMOMETRICS: u16 = 1 << 11;
    /// Database / data-quality constraints.
    pub const DATABASE: u16 = 1 << 12;

    /// True if at least one domain is declared. Verifier rejects empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Coarse confuser-profile classification for fusion-time suppression.
///
/// The classification lives on every corpus record; T.6 wired the
/// declarative compatibility rules around it (see
/// [`crate::fusion::COMPATIBILITY_RULES`]). Runtime fusion-time
/// suppression itself remains a Section S concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfuserProfile {
    /// No confuser-suppression hooks needed.
    None,
    /// Suppresses based on small-sample regime.
    SmallSample,
    /// Suppresses based on single-window-spike regime.
    SingleWindowSpike,
    /// Suppresses based on missingness-artifact regime.
    MissingnessArtifact,
    /// Suppresses based on schema-change regime.
    SchemaChange,
    /// Suppresses based on batch-boundary regime.
    BatchBoundary,
    /// Suppresses based on a generic "multiple known-confuser regimes".
    Multiple,
}

/// Coarse deterministic-status classification.
///
/// The literature contains primitives that are inherently
/// probabilistic (random projections, MCMC sampling, learned
/// thresholds). The corpus accepts a *deterministic reduction* of
/// such primitives — pinned seeds, fixed budgets, point-estimate
/// reductions — but the original-probabilistic origin must be
/// declared so the paper can be honest about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeterministicStatus {
    /// Literature primitive is deterministic by construction.
    DeterministicNative,
    /// Literature primitive is probabilistic; the corpus implements
    /// a deterministic-seed reduction (e.g. fixed-seed Isolation
    /// Forest, point-estimate BOCPD).
    DeterministicReductionOfStochastic,
    /// Literature primitive is probabilistic and the corpus does
    /// NOT yet ship a deterministic reduction (Cited-only L0).
    StochasticOnly,
    /// The primitive's determinism is contingent on a parameter
    /// (e.g. KS is deterministic only when the reference CDF is
    /// fixed; otherwise it's a bootstrap procedure).
    DeterministicConditional,
}

/// Implementation-status ladder (panel-locked).
///
/// Every literature primitive carries an L-band so the paper cannot
/// overclaim coverage. The bands are ordered: a primitive at L8 has
/// also been L7, L6, L5, … etc.
///
/// The variant identifiers use the `LN_` prefix verbatim from the
/// panel verdict's documentation rather than UpperCamelCase
/// (`L0_CitedOnly`, not `L0CitedOnly`) because the band number is a
/// load-bearing semantic marker and is read in code, in reports, and
/// in the paper as a labelled scale. UpperCamelCase would lose the
/// readable separator between the band number and the band name.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImplementationLevel {
    /// L0: cited in the corpus; no implementation yet.
    L0_CitedOnly,
    /// L1: canonicalised into a unique court entry.
    L1_Canonicalised,
    /// L2: deterministic formula declared (no code yet).
    L2_DeterministicFormula,
    /// L3: CPU implementation exists.
    L3_CpuImplemented,
    /// L4: CPU implementation passes its own verification tests.
    L4_CpuVerified,
    /// L5: GPU implementation exists.
    L5_GpuImplemented,
    /// L6: CPU and GPU implementations are byte-equivalent.
    L6_CpuGpuByteEquivalent,
    /// L7: implementation is benchmark-characterised (timed +
    /// stage-profiled at some scale).
    L7_BenchmarkCharacterised,
    /// L8: usefulness ledger has at least one (task × dataset)
    /// row of empirical-delta data.
    L8_LedgerCharacterised,
}

/// Lifecycle state for a detector record.
///
/// Active by default; the dedup court can retire a detector when
/// it is redundant or harmful, and resurrect it for a specific
/// domain when the manifest proves relevance. Lifecycle transitions
/// are themselves auditable events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LifecycleState {
    /// Active in the corpus and eligible for activation by the
    /// planner (subject to manifest fit).
    Active,
    /// Dormant: the court keeps the record but the activation
    /// planner will not select it by default.
    Dormant,
    /// Retired because another canonical primitive subsumes it.
    RetiredRedundant,
    /// Retired because empirical false-positive cost was unacceptable.
    RetiredHighFalsePositive,
    /// Retired because it was too expensive to compute at scale.
    RetiredTooExpensive,
    /// Quarantined: implementation is unstable / has a known bug,
    /// scheduled for fix.
    QuarantinedUnstable,
    /// Resurrected from retirement for a specific domain because the
    /// manifest proved relevance.
    ResurrectedForDomain,
}

/// The eight constitution flags — every detector MUST declare each
/// of these before it can be admitted into the corpus.
///
/// The verifier (see [`crate::verify`]) rejects any detector with
/// any `false` field. This is the gate that keeps the Atlas from
/// becoming a junk drawer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstitutionFlags {
    /// `input_requirements` is populated and non-empty.
    pub declared_input_contract: bool,
    /// `output_witness` is set.
    pub declared_output_type: bool,
    /// `deterministic_status` is set (a stochastic primitive must
    /// say so explicitly; it cannot omit the field).
    pub declared_deterministic_form: bool,
    /// `source_refs` has at least one entry OR `notes` carries an
    /// explicit engineering-practice provenance.
    pub declared_provenance: bool,
    /// `duplicate_group` is set (canonical or alias).
    pub declared_equivalence_status: bool,
    /// `witness_role` is set.
    pub declared_witness_role: bool,
    /// `origin_domains` is non-empty (activation conditions exist).
    pub declared_activation_conditions: bool,
    /// `confuser_profile` is set (the detector states which
    /// confusers can mimic its firing, even if the value is `None`).
    pub declared_failure_confuser_modes: bool,
}

/// GPU execution-family classification.
///
/// 1000 literature primitives map to ~14 GPU execution families,
/// not 1000 kernels. This enum is the schema-side tag every record
/// carries; the actual kernel dispatch lives in `dsfb-gpu-atlas-cuda`
/// (Section S Phase 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuFamilyKernel {
    /// Scalar threshold over a single feature.
    ScalarThresholdFamily,
    /// Window-statistic family.
    WindowStatisticFamily,
    /// Sequential-recurrence family (EWMA / CUSUM / Page-Hinkley).
    SequentialRecurrenceFamily,
    /// Distribution-distance family.
    DistributionDistanceFamily,
    /// Rank-statistic family.
    RankStatisticFamily,
    /// Spectral / FFT family.
    SpectralFamily,
    /// Wavelet family.
    WaveletFamily,
    /// Graph-local family.
    GraphLocalFamily,
    /// Graph-global family.
    GraphGlobalFamily,
    /// Tabular-constraint family.
    TabularConstraintFamily,
    /// Categorical-histogram family.
    CategoricalHistogramFamily,
    /// Missingness family.
    MissingnessFamily,
    /// Residual / observer family.
    ResidualObserverFamily,
    /// Projection-residual (PCA SPE/T2) family.
    ProjectionResidualFamily,
    /// Negative-witness family (for confuser-suppression detectors
    /// that need their own kernel surface).
    NegativeWitnessFamily,
}

/// Coarse parameter-bound descriptor.
///
/// Stores a textual description of the parameter grid the primitive
/// admits. The formal `parameter_hash` (see [`crate::identity`])
/// is computed over the canonical parameter tuple, not over this
/// descriptor.
#[derive(Debug, Clone, Copy)]
pub struct ParameterBounds {
    /// Number of independent parameter axes (e.g. window, threshold,
    /// persistence) the primitive exposes.
    pub axis_count: u8,
    /// Free-form description of the bounds in human-readable form.
    /// Used by the report generator and by paper §16 deferred-work
    /// listings; not used by hash identity.
    pub description: &'static str,
}

/// Genealogy edges — how detectors descend across the literature.
///
/// Each record carries its own ancestry / generalisation /
/// specialisation / parameter-variant / domain-transfer edges as
/// `&'static` slices. The DAG construction, verifier, DOT export,
/// and JSON export live in [`crate::genealogy`] (T.5) and operate
/// over the union of these per-record edge sets plus alias edges
/// derived from the dedup court's [`CanonicalisationDecision::AliasOf`]
/// records.
#[derive(Debug, Clone, Copy)]
pub struct GenealogyEdges {
    /// Direct ancestors that this primitive is derived from.
    pub derived_from: &'static [DetectorCanonicalId],
    /// Primitives that this one generalises (it is a superset).
    pub generalizes: &'static [DetectorCanonicalId],
    /// Primitives that this one is a special case of (it is a subset).
    pub special_case_of: &'static [DetectorCanonicalId],
    /// True if this primitive is an origin point (no ancestors).
    /// Origin records must carry at least one `SourceRef`.
    pub is_origin: bool,
}

impl GenealogyEdges {
    /// Empty edges, marked as "origin" — used by seed entries that
    /// represent foundational primitives.
    #[must_use]
    pub const fn origin() -> Self {
        Self {
            derived_from: &[],
            generalizes: &[],
            special_case_of: &[],
            is_origin: true,
        }
    }
}

/// Usefulness-ledger row stub.
///
/// At T.1a-T.7 every record carries zeros and a sample_count of 0;
/// this embedded row is a per-detector prior-summary view, not a
/// trial record. T.8 renames this type to `UsefulnessLedgerSnapshot`
/// and introduces a richer ledger keyed by
/// (canonical_id, task_id, domain, dataset_id) in a separate module
/// with an `UsefulnessEvidenceLevel` honesty ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UsefulnessLedgerSnapshot {
    /// Number of unique admissions this detector contributed
    /// across the recorded sample.
    pub unique_episode_gain: i64,
    /// Estimated false-positive cost on clean windows.
    pub clean_window_false_positive_cost: i64,
    /// Estimated confuser-suppression contribution.
    pub confuser_reduction_gain: i64,
    /// p50 runtime cost in microseconds.
    pub runtime_cost_us_p50: u32,
    /// Memory cost in bytes (peak per-cell working set).
    pub memory_cost_bytes: u64,
    /// Operator-readability score, hand-rated in 0..=10.
    pub operator_readability_score: u8,
    /// Number of (task × dataset) runs aggregated into this row.
    pub sample_count: u64,
}

impl UsefulnessLedgerSnapshot {
    /// Empty / unmeasured ledger row. Seed entries use this until
    /// T.8 populates real data.
    #[must_use]
    pub const fn unmeasured() -> Self {
        Self {
            unique_episode_gain: 0,
            clean_window_false_positive_cost: 0,
            confuser_reduction_gain: 0,
            runtime_cost_us_p50: 0,
            memory_cost_bytes: 0,
            operator_readability_score: 0,
            sample_count: 0,
        }
    }
}

/// One literature-detector record in the corpus.
///
/// Every field is mandatory. The verifier in [`crate::verify`]
/// walks the seed and rejects any record that omits a field the
/// schema requires (empty domain set, all-`false` constitution
/// flags, no source ref / engineering-practice note, etc.).
///
/// The five-hash identity (`DetectorIdentityHashes`) is NOT
/// stored on the record at T.1a; it is computed by [`crate::verify`]
/// from the record bytes when needed. Future T.3 work makes the
/// hashes load-bearing for cross-corpus dedup; T.1a keeps the
/// schema stable so that change does not touch seed data.
#[derive(Debug, Clone, Copy)]
pub struct LiteratureDetector {
    /// Canonical handle for this record.
    pub canonical_id: DetectorCanonicalId,
    /// Human-readable display name (e.g. `"Shewhart control chart"`).
    pub display_name: &'static str,
    /// Other names this primitive goes by in the literature.
    pub aliases: &'static [&'static str],
    /// Provenance: at least one `SourceRef` MUST be present unless
    /// the primitive is engineering-practice with explicit notes.
    pub source_refs: &'static [SourceRef],
    /// Where this primitive was originally developed / where it's
    /// natively applicable.
    pub origin_domains: DomainTagSet,
    /// Primitive-family classification.
    pub primitive_family: PrimitiveFamily,
    /// Coarse mathematical-form classification.
    pub mathematical_form: MathFormId,
    /// Decision-functional shape.
    pub decision_functional: DecisionFunctional,
    /// Input-contract bitset.
    pub input_requirements: InputRequirementSet,
    /// What this detector emits when it fires.
    pub output_witness: WitnessKind,
    /// Court role in the fusion layer.
    pub witness_role: WitnessRole,
    /// Negative-witness sub-classifier (use `NotANegativeWitness`
    /// for positive witnesses).
    pub negative_witness_kind: NegativeWitnessKind,
    /// Fusion-axis bitset.
    pub fusion_axes: AxisBindingSet,
    /// Coarse confuser-profile classification.
    pub confuser_profile: ConfuserProfile,
    /// Deterministic-status classification.
    pub deterministic_status: DeterministicStatus,
    /// Implementation-status band (L0..L8).
    pub implementation_status: ImplementationLevel,
    /// GPU execution family.
    pub gpu_family: GpuFamilyKernel,
    /// Parameter-bound descriptor.
    pub parameter_bounds: ParameterBounds,
    /// Duplicate-equivalence class membership.
    pub duplicate_group: DuplicateGroupId,
    /// Genealogy edges.
    pub genealogy: GenealogyEdges,
    /// Usefulness-ledger row (zeros until T.8 populates).
    pub usefulness: UsefulnessLedgerSnapshot,
    /// Lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// Constitution flags — every field must be `true` to pass
    /// the verifier.
    pub constitution_compliance: ConstitutionFlags,
}

/// The dedup-court decision for a single literature claim.
///
/// The enum lives here at the schema layer; the court machinery
/// that produces decisions from seed claims lives in
/// [`crate::court`] (T.4) with reason codes per decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalisationDecision {
    /// This claim is the canonical representative of its
    /// equivalence class.
    Canonical,
    /// This claim is an alias of the named canonical primitive.
    AliasOf(DetectorCanonicalId),
    /// This claim is a parameterisation of the named canonical
    /// primitive (different parameters, same formula).
    ParameterisationOf(DetectorCanonicalId),
    /// This claim is a composition of the named canonical
    /// primitives (a meta-detector built out of others).
    CompositionOf(&'static [DetectorCanonicalId]),
    /// This claim was originally stochastic and is admitted as
    /// a deterministic-seed reduction of the named canonical
    /// (which itself stays in `StochasticOnly` status).
    StochasticOriginalDeterministicReduction(DetectorCanonicalId),
    /// This claim is rejected because it is not deterministic and
    /// the corpus has no deterministic-reduction story for it.
    RejectedNotDeterministic,
    /// This claim is rejected because it isn't actually a detector
    /// in the corpus sense (e.g. a dimensionality-reduction step,
    /// a feature transform without a firing decision).
    RejectedNotDetector,
    /// The court has not yet made a final decision; the claim is
    /// pending review.
    DeferredNeedsReview,
}

/// Reason code attached to a `CanonicalisationDecision` for the
/// public dedup report.
///
/// The reason codes are exhaustive: every dedup decision must
/// produce a reason. The court ([`crate::court`], T.4) emits one
/// per subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DedupReason {
    /// Same formula, same parameter grid, same input contract,
    /// different name in the literature.
    SameFormulaSameParametersSameContract,
    /// Same formula, different parameter values (window length,
    /// persistence, threshold scale).
    SameFormulaDifferentParameters,
    /// Different math, same domain, comparable outputs — keep both.
    DifferentFormulaSameDomain,
    /// Same formula, different input contract — keep both as
    /// structurally distinct witnesses.
    SameFormulaDifferentInputContract,
    /// Same formula, different witness role (one is a primary
    /// witness, the other is a confuser) — keep both.
    SameFormulaDifferentWitnessRole,
    /// Different decision functional (one-sided vs two-sided,
    /// persistence-gated vs not) — keep both.
    DifferentDecisionFunctional,
    /// Probabilistic original; the canonical entry is the
    /// deterministic-seed reduction.
    DeterministicReductionOfStochastic,
    /// Composition of multiple canonical primitives.
    CompositionOfCanonicals,
    /// Origin / no ancestors recorded; canonical by default.
    OriginRecord,
}

/// What the dedup court is judging — either an existing canonical
/// seed record or a separate literature-name claim.
///
/// The two-variant subject is the panel-locked T.4 design: the
/// canonical seed records keep their identity, and alias claims
/// (literature names for the same mathematical witness) get their
/// own court records pointing at the canonical they collapse into.
/// This preserves an audit trail — every alias has its own
/// reason code and notes string in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DedupSubject {
    /// One of the existing canonical seed records.
    Canonical(DetectorCanonicalId),
    /// A separate literature-name alias claim (entry in
    /// `crate::claims::CLAIMS`).
    AliasClaim(DetectorAliasId),
}

/// One dedup-court record.
///
/// The court emits exactly one [`DedupRecord`] per subject (each
/// canonical seed record AND each alias claim). The classification
/// is deterministic; two passes over the same input produce
/// byte-identical record sequences.
#[derive(Debug, Clone, Copy)]
pub struct DedupRecord {
    /// What the court is judging.
    pub subject: DedupSubject,
    /// Human-readable literature name for the subject (used by the
    /// report). For canonical subjects this echoes
    /// [`LiteratureDetector::display_name`]; for alias claims this
    /// is the alias's literature_name string.
    pub literature_name: &'static str,
    /// The court's decision.
    pub decision: CanonicalisationDecision,
    /// The reason code attached to the decision.
    pub reason_code: DedupReason,
    /// Free-form notes for the public report.
    pub notes: &'static str,
}

// =========================================================
// T.2 — wire-name string conversion for the TOML round trip.
// =========================================================
//
// Each enum exposes `as_str(&self) -> &'static str` and
// `from_wire(s: &str) -> Option<Self>`. The wire names match the
// Rust variant names verbatim so the TOML files are unambiguous
// when read alongside the source. The bitsets expose
// `bit_names() -> &'static [(u_, &'static str)]` so dump and load
// share one canonical bit ordering.

impl PrimitiveFamily {
    /// Canonical wire name (Rust variant name).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ScalarThreshold => "ScalarThreshold",
            Self::WindowStatistic => "WindowStatistic",
            Self::SequentialRecurrence => "SequentialRecurrence",
            Self::DistributionDistance => "DistributionDistance",
            Self::RankStatistic => "RankStatistic",
            Self::Spectral => "Spectral",
            Self::Wavelet => "Wavelet",
            Self::GraphLocal => "GraphLocal",
            Self::GraphGlobal => "GraphGlobal",
            Self::TabularConstraint => "TabularConstraint",
            Self::CategoricalHistogram => "CategoricalHistogram",
            Self::Missingness => "Missingness",
            Self::ResidualObserver => "ResidualObserver",
            Self::ProjectionResidual => "ProjectionResidual",
            Self::MultivariateHypothesis => "MultivariateHypothesis",
            Self::InformationTheory => "InformationTheory",
            Self::OperabilityDiagnostic => "OperabilityDiagnostic",
            Self::DebugObservability => "DebugObservability",
            Self::NegativeWitness => "NegativeWitness",
        }
    }
    /// Parse a wire name back into the enum.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "ScalarThreshold" => Self::ScalarThreshold,
            "WindowStatistic" => Self::WindowStatistic,
            "SequentialRecurrence" => Self::SequentialRecurrence,
            "DistributionDistance" => Self::DistributionDistance,
            "RankStatistic" => Self::RankStatistic,
            "Spectral" => Self::Spectral,
            "Wavelet" => Self::Wavelet,
            "GraphLocal" => Self::GraphLocal,
            "GraphGlobal" => Self::GraphGlobal,
            "TabularConstraint" => Self::TabularConstraint,
            "CategoricalHistogram" => Self::CategoricalHistogram,
            "Missingness" => Self::Missingness,
            "ResidualObserver" => Self::ResidualObserver,
            "ProjectionResidual" => Self::ProjectionResidual,
            "MultivariateHypothesis" => Self::MultivariateHypothesis,
            "InformationTheory" => Self::InformationTheory,
            "OperabilityDiagnostic" => Self::OperabilityDiagnostic,
            "DebugObservability" => Self::DebugObservability,
            "NegativeWitness" => Self::NegativeWitness,
            _ => return None,
        })
    }
}

impl MathFormId {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Threshold => "Threshold",
            Self::StandardisedDeviation => "StandardisedDeviation",
            Self::RobustStandardisedDeviation => "RobustStandardisedDeviation",
            Self::ExponentialMovingAverage => "ExponentialMovingAverage",
            Self::CumulativeSum => "CumulativeSum",
            Self::GeneralisedLikelihoodRatio => "GeneralisedLikelihoodRatio",
            Self::HotellingTSquared => "HotellingTSquared",
            Self::SquaredPredictionError => "SquaredPredictionError",
            Self::KolmogorovSmirnov => "KolmogorovSmirnov",
            Self::KullbackLeibler => "KullbackLeibler",
            Self::JensenShannon => "JensenShannon",
            Self::MaximumMeanDiscrepancy => "MaximumMeanDiscrepancy",
            Self::MannKendallRank => "MannKendallRank",
            Self::WaveletCoefficientMagnitude => "WaveletCoefficientMagnitude",
            Self::FftBandEnergy => "FftBandEnergy",
            Self::MissingnessAggregate => "MissingnessAggregate",
            Self::SchemaPredicate => "SchemaPredicate",
            Self::WindowedResidualAccumulator => "WindowedResidualAccumulator",
            Self::BooleanRuleAggregate => "BooleanRuleAggregate",
            Self::QuantileFence => "QuantileFence",
            Self::AndersonDarling => "AndersonDarling",
            Self::CramerVonMises => "CramerVonMises",
            Self::Wasserstein => "Wasserstein",
            Self::EnergyDistance => "EnergyDistance",
            Self::HellingerDistance => "HellingerDistance",
            Self::TotalVariation => "TotalVariation",
            Self::PopulationStabilityIndex => "PopulationStabilityIndex",
            Self::RankChangePoint => "RankChangePoint",
            Self::HomogeneityTest => "HomogeneityTest",
            Self::MovingSumOfResiduals => "MovingSumOfResiduals",
            Self::SpectralEntropy => "SpectralEntropy",
            Self::AutocorrelationBreak => "AutocorrelationBreak",
            Self::OscillationSignature => "OscillationSignature",
            Self::CategoricalCardinality => "CategoricalCardinality",
            Self::EnvelopeExit => "EnvelopeExit",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "Threshold" => Self::Threshold,
            "StandardisedDeviation" => Self::StandardisedDeviation,
            "RobustStandardisedDeviation" => Self::RobustStandardisedDeviation,
            "ExponentialMovingAverage" => Self::ExponentialMovingAverage,
            "CumulativeSum" => Self::CumulativeSum,
            "GeneralisedLikelihoodRatio" => Self::GeneralisedLikelihoodRatio,
            "HotellingTSquared" => Self::HotellingTSquared,
            "SquaredPredictionError" => Self::SquaredPredictionError,
            "KolmogorovSmirnov" => Self::KolmogorovSmirnov,
            "KullbackLeibler" => Self::KullbackLeibler,
            "JensenShannon" => Self::JensenShannon,
            "MaximumMeanDiscrepancy" => Self::MaximumMeanDiscrepancy,
            "MannKendallRank" => Self::MannKendallRank,
            "WaveletCoefficientMagnitude" => Self::WaveletCoefficientMagnitude,
            "FftBandEnergy" => Self::FftBandEnergy,
            "MissingnessAggregate" => Self::MissingnessAggregate,
            "SchemaPredicate" => Self::SchemaPredicate,
            "WindowedResidualAccumulator" => Self::WindowedResidualAccumulator,
            "BooleanRuleAggregate" => Self::BooleanRuleAggregate,
            "QuantileFence" => Self::QuantileFence,
            "AndersonDarling" => Self::AndersonDarling,
            "CramerVonMises" => Self::CramerVonMises,
            "Wasserstein" => Self::Wasserstein,
            "EnergyDistance" => Self::EnergyDistance,
            "HellingerDistance" => Self::HellingerDistance,
            "TotalVariation" => Self::TotalVariation,
            "PopulationStabilityIndex" => Self::PopulationStabilityIndex,
            "RankChangePoint" => Self::RankChangePoint,
            "HomogeneityTest" => Self::HomogeneityTest,
            "MovingSumOfResiduals" => Self::MovingSumOfResiduals,
            "SpectralEntropy" => Self::SpectralEntropy,
            "AutocorrelationBreak" => Self::AutocorrelationBreak,
            "OscillationSignature" => Self::OscillationSignature,
            "CategoricalCardinality" => Self::CategoricalCardinality,
            "EnvelopeExit" => Self::EnvelopeExit,
            _ => return None,
        })
    }
}

impl DecisionFunctional {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OneSidedUpper => "OneSidedUpper",
            Self::OneSidedLower => "OneSidedLower",
            Self::TwoSided => "TwoSided",
            Self::SequentialStopping => "SequentialStopping",
            Self::PersistenceGated => "PersistenceGated",
            Self::BoundaryEdge => "BoundaryEdge",
            Self::AggregateThreshold => "AggregateThreshold",
            Self::RuleSetAny => "RuleSetAny",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "OneSidedUpper" => Self::OneSidedUpper,
            "OneSidedLower" => Self::OneSidedLower,
            "TwoSided" => Self::TwoSided,
            "SequentialStopping" => Self::SequentialStopping,
            "PersistenceGated" => Self::PersistenceGated,
            "BoundaryEdge" => Self::BoundaryEdge,
            "AggregateThreshold" => Self::AggregateThreshold,
            "RuleSetAny" => Self::RuleSetAny,
            _ => return None,
        })
    }
}

impl WitnessKind {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::BooleanCell => "BooleanCell",
            Self::BooleanRow => "BooleanRow",
            Self::ScalarMargin => "ScalarMargin",
            Self::Interval => "Interval",
            Self::SubgraphSelection => "SubgraphSelection",
            Self::CategoryLabel => "CategoryLabel",
            Self::HistogramDelta => "HistogramDelta",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "BooleanCell" => Self::BooleanCell,
            "BooleanRow" => Self::BooleanRow,
            "ScalarMargin" => Self::ScalarMargin,
            "Interval" => Self::Interval,
            "SubgraphSelection" => Self::SubgraphSelection,
            "CategoryLabel" => Self::CategoryLabel,
            "HistogramDelta" => Self::HistogramDelta,
            _ => return None,
        })
    }
}

impl WitnessRole {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Corroborating => "Corroborating",
            Self::Confuser => "Confuser",
            Self::Boundary => "Boundary",
            Self::CleanWindow => "CleanWindow",
            Self::Recovery => "Recovery",
            Self::Timing => "Timing",
            Self::Distribution => "Distribution",
            Self::Topology => "Topology",
            Self::CausalityProxy => "CausalityProxy",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "Primary" => Self::Primary,
            "Corroborating" => Self::Corroborating,
            "Confuser" => Self::Confuser,
            "Boundary" => Self::Boundary,
            "CleanWindow" => Self::CleanWindow,
            "Recovery" => Self::Recovery,
            "Timing" => Self::Timing,
            "Distribution" => Self::Distribution,
            "Topology" => Self::Topology,
            "CausalityProxy" => Self::CausalityProxy,
            _ => return None,
        })
    }
}

impl NegativeWitnessKind {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SmallSampleConfuser => "SmallSampleConfuser",
            Self::SingleWindowSpikeConfuser => "SingleWindowSpikeConfuser",
            Self::PeriodicBoundaryConfuser => "PeriodicBoundaryConfuser",
            Self::MissingnessArtifactConfuser => "MissingnessArtifactConfuser",
            Self::SchemaChangeConfuser => "SchemaChangeConfuser",
            Self::UnitScaleChangeConfuser => "UnitScaleChangeConfuser",
            Self::DeploymentMarkerConfuser => "DeploymentMarkerConfuser",
            Self::ClockSkewConfuser => "ClockSkewConfuser",
            Self::BatchBoundaryConfuser => "BatchBoundaryConfuser",
            Self::NotANegativeWitness => "NotANegativeWitness",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "SmallSampleConfuser" => Self::SmallSampleConfuser,
            "SingleWindowSpikeConfuser" => Self::SingleWindowSpikeConfuser,
            "PeriodicBoundaryConfuser" => Self::PeriodicBoundaryConfuser,
            "MissingnessArtifactConfuser" => Self::MissingnessArtifactConfuser,
            "SchemaChangeConfuser" => Self::SchemaChangeConfuser,
            "UnitScaleChangeConfuser" => Self::UnitScaleChangeConfuser,
            "DeploymentMarkerConfuser" => Self::DeploymentMarkerConfuser,
            "ClockSkewConfuser" => Self::ClockSkewConfuser,
            "BatchBoundaryConfuser" => Self::BatchBoundaryConfuser,
            "NotANegativeWitness" => Self::NotANegativeWitness,
            _ => return None,
        })
    }
}

impl ConfuserProfile {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::SmallSample => "SmallSample",
            Self::SingleWindowSpike => "SingleWindowSpike",
            Self::MissingnessArtifact => "MissingnessArtifact",
            Self::SchemaChange => "SchemaChange",
            Self::BatchBoundary => "BatchBoundary",
            Self::Multiple => "Multiple",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "None" => Self::None,
            "SmallSample" => Self::SmallSample,
            "SingleWindowSpike" => Self::SingleWindowSpike,
            "MissingnessArtifact" => Self::MissingnessArtifact,
            "SchemaChange" => Self::SchemaChange,
            "BatchBoundary" => Self::BatchBoundary,
            "Multiple" => Self::Multiple,
            _ => return None,
        })
    }
}

impl DeterministicStatus {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DeterministicNative => "DeterministicNative",
            Self::DeterministicReductionOfStochastic => "DeterministicReductionOfStochastic",
            Self::StochasticOnly => "StochasticOnly",
            Self::DeterministicConditional => "DeterministicConditional",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "DeterministicNative" => Self::DeterministicNative,
            "DeterministicReductionOfStochastic" => Self::DeterministicReductionOfStochastic,
            "StochasticOnly" => Self::StochasticOnly,
            "DeterministicConditional" => Self::DeterministicConditional,
            _ => return None,
        })
    }
}

impl ImplementationLevel {
    /// Canonical wire name (matches the L-band identifier verbatim).
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::L0_CitedOnly => "L0_CitedOnly",
            Self::L1_Canonicalised => "L1_Canonicalised",
            Self::L2_DeterministicFormula => "L2_DeterministicFormula",
            Self::L3_CpuImplemented => "L3_CpuImplemented",
            Self::L4_CpuVerified => "L4_CpuVerified",
            Self::L5_GpuImplemented => "L5_GpuImplemented",
            Self::L6_CpuGpuByteEquivalent => "L6_CpuGpuByteEquivalent",
            Self::L7_BenchmarkCharacterised => "L7_BenchmarkCharacterised",
            Self::L8_LedgerCharacterised => "L8_LedgerCharacterised",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "L0_CitedOnly" => Self::L0_CitedOnly,
            "L1_Canonicalised" => Self::L1_Canonicalised,
            "L2_DeterministicFormula" => Self::L2_DeterministicFormula,
            "L3_CpuImplemented" => Self::L3_CpuImplemented,
            "L4_CpuVerified" => Self::L4_CpuVerified,
            "L5_GpuImplemented" => Self::L5_GpuImplemented,
            "L6_CpuGpuByteEquivalent" => Self::L6_CpuGpuByteEquivalent,
            "L7_BenchmarkCharacterised" => Self::L7_BenchmarkCharacterised,
            "L8_LedgerCharacterised" => Self::L8_LedgerCharacterised,
            _ => return None,
        })
    }
}

impl LifecycleState {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Dormant => "Dormant",
            Self::RetiredRedundant => "RetiredRedundant",
            Self::RetiredHighFalsePositive => "RetiredHighFalsePositive",
            Self::RetiredTooExpensive => "RetiredTooExpensive",
            Self::QuarantinedUnstable => "QuarantinedUnstable",
            Self::ResurrectedForDomain => "ResurrectedForDomain",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "Active" => Self::Active,
            "Dormant" => Self::Dormant,
            "RetiredRedundant" => Self::RetiredRedundant,
            "RetiredHighFalsePositive" => Self::RetiredHighFalsePositive,
            "RetiredTooExpensive" => Self::RetiredTooExpensive,
            "QuarantinedUnstable" => Self::QuarantinedUnstable,
            "ResurrectedForDomain" => Self::ResurrectedForDomain,
            _ => return None,
        })
    }
}

impl GpuFamilyKernel {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ScalarThresholdFamily => "ScalarThresholdFamily",
            Self::WindowStatisticFamily => "WindowStatisticFamily",
            Self::SequentialRecurrenceFamily => "SequentialRecurrenceFamily",
            Self::DistributionDistanceFamily => "DistributionDistanceFamily",
            Self::RankStatisticFamily => "RankStatisticFamily",
            Self::SpectralFamily => "SpectralFamily",
            Self::WaveletFamily => "WaveletFamily",
            Self::GraphLocalFamily => "GraphLocalFamily",
            Self::GraphGlobalFamily => "GraphGlobalFamily",
            Self::TabularConstraintFamily => "TabularConstraintFamily",
            Self::CategoricalHistogramFamily => "CategoricalHistogramFamily",
            Self::MissingnessFamily => "MissingnessFamily",
            Self::ResidualObserverFamily => "ResidualObserverFamily",
            Self::ProjectionResidualFamily => "ProjectionResidualFamily",
            Self::NegativeWitnessFamily => "NegativeWitnessFamily",
        }
    }
    /// Parse a wire name.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "ScalarThresholdFamily" => Self::ScalarThresholdFamily,
            "WindowStatisticFamily" => Self::WindowStatisticFamily,
            "SequentialRecurrenceFamily" => Self::SequentialRecurrenceFamily,
            "DistributionDistanceFamily" => Self::DistributionDistanceFamily,
            "RankStatisticFamily" => Self::RankStatisticFamily,
            "SpectralFamily" => Self::SpectralFamily,
            "WaveletFamily" => Self::WaveletFamily,
            "GraphLocalFamily" => Self::GraphLocalFamily,
            "GraphGlobalFamily" => Self::GraphGlobalFamily,
            "TabularConstraintFamily" => Self::TabularConstraintFamily,
            "CategoricalHistogramFamily" => Self::CategoricalHistogramFamily,
            "MissingnessFamily" => Self::MissingnessFamily,
            "ResidualObserverFamily" => Self::ResidualObserverFamily,
            "ProjectionResidualFamily" => Self::ProjectionResidualFamily,
            "NegativeWitnessFamily" => Self::NegativeWitnessFamily,
            _ => return None,
        })
    }
}

impl InputRequirementSet {
    /// Canonical (bit, name) ordering for dump and report.
    #[must_use]
    pub const fn bit_names() -> &'static [(u32, &'static str)] {
        &[
            (Self::ORDERED_TIME, "ORDERED_TIME"),
            (Self::REGULAR_SAMPLING, "REGULAR_SAMPLING"),
            (Self::NUMERIC, "NUMERIC"),
            (Self::CATEGORICAL, "CATEGORICAL"),
            (Self::GRAPH, "GRAPH"),
            (Self::BASELINE_WINDOW, "BASELINE_WINDOW"),
            (Self::REFERENCE_DISTRIBUTION, "REFERENCE_DISTRIBUTION"),
            (Self::LABELS, "LABELS"),
            (Self::SEASONALITY_PERIOD, "SEASONALITY_PERIOD"),
            (Self::TOPOLOGY, "TOPOLOGY"),
            (Self::UNITS, "UNITS"),
            (Self::MISSINGNESS_MASK, "MISSINGNESS_MASK"),
        ]
    }
    /// Look up a single bit by wire name.
    #[must_use]
    pub fn bit_from_wire(s: &str) -> Option<u32> {
        for &(bit, name) in Self::bit_names() {
            if name == s {
                return Some(bit);
            }
        }
        None
    }
}

impl AxisBindingSet {
    /// Canonical (bit, name) ordering.
    #[must_use]
    pub const fn bit_names() -> &'static [(u16, &'static str)] {
        &[
            (Self::AXIS_1_RESIDUAL_MAGNITUDE, "AXIS_1_RESIDUAL_MAGNITUDE"),
            (Self::AXIS_2_DRIFT_PERSISTENCE, "AXIS_2_DRIFT_PERSISTENCE"),
            (Self::AXIS_3_SLEW_SHOCK, "AXIS_3_SLEW_SHOCK"),
            (Self::AXIS_4_TEMPORAL_LOCALITY, "AXIS_4_TEMPORAL_LOCALITY"),
            (Self::AXIS_5_ENTITY_LOCALITY, "AXIS_5_ENTITY_LOCALITY"),
            (Self::AXIS_6_CAUSAL_ADJACENCY, "AXIS_6_CAUSAL_ADJACENCY"),
            (Self::AXIS_7_MOTIF_CONSENSUS, "AXIS_7_MOTIF_CONSENSUS"),
            (Self::AXIS_8_BANK_ADMISSIBILITY, "AXIS_8_BANK_ADMISSIBILITY"),
            (
                Self::AXIS_9_CONFUSER_SUPPRESSION,
                "AXIS_9_CONFUSER_SUPPRESSION",
            ),
        ]
    }
    /// Look up a single bit by wire name.
    #[must_use]
    pub fn bit_from_wire(s: &str) -> Option<u16> {
        for &(bit, name) in Self::bit_names() {
            if name == s {
                return Some(bit);
            }
        }
        None
    }
}

impl DomainTagSet {
    /// Canonical (bit, name) ordering.
    #[must_use]
    pub const fn bit_names() -> &'static [(u16, &'static str)] {
        &[
            (Self::DEBUG, "DEBUG"),
            (Self::TELEMETRY, "TELEMETRY"),
            (Self::TABULAR, "TABULAR"),
            (Self::TIME_SERIES, "TIME_SERIES"),
            (Self::GRAPH, "GRAPH"),
            (Self::INDUSTRIAL, "INDUSTRIAL"),
            (Self::CATEGORICAL, "CATEGORICAL"),
            (Self::MISSINGNESS, "MISSINGNESS"),
            (Self::EVENT_STREAM, "EVENT_STREAM"),
            (Self::MEDICAL, "MEDICAL"),
            (Self::RF_COMMS, "RF_COMMS"),
            (Self::CHEMOMETRICS, "CHEMOMETRICS"),
            (Self::DATABASE, "DATABASE"),
        ]
    }
    /// Look up a single bit by wire name.
    #[must_use]
    pub fn bit_from_wire(s: &str) -> Option<u16> {
        for &(bit, name) in Self::bit_names() {
            if name == s {
                return Some(bit);
            }
        }
        None
    }
}
