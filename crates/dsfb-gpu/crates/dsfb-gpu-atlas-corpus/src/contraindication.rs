//! T.11g — `DetectorContraindicationReceiptV1`: the court's
//! datasheet / model-card / safety-label layer.
//!
//! Panel framing:
//!
//! > A detector is not fully admissible until the court knows
//! > **when it should not be trusted**.
//!
//! T.11g answers nine questions per detector — works-best-when,
//! fails-when, known-confusers, required-sampling-law,
//! required-units, minimum-support, do-not-use-for,
//! closest-aliases, closest-non-aliases — plus an adversarial-
//! twin layer that names the detectors most likely to be
//! confused with it. Every receipt is DSFB-native and
//! verifier-enforced.
//!
//! **Design boundary (panel-locked)**: T.11g does **not** mutate
//! any prior hash surface. It does not mutate `DetectorPassport`
//! hashes (T.11a); instead it emits a separate
//! **passport-contraindication crosswalk** artifact — the same
//! pattern T.11c used to bind passports to admissibility
//! grammar rules without churning passport hashes.
//!
//! **Hash chain (panel-locked)**:
//!
//! ```text
//!   corpus_hash_v1
//!     → registry_hash_v2
//!     → precedent_hash_v1
//!     → admissibility_grammar_hash_v1
//!     → trial_transcript_hash_v1
//!     → execution_attestation receipt_hash_v1
//!     → challenge_docket_hash_v1
//!     → detector_contraindication_hash_v1   (NEW at T.11g)
//! ```
//!
//! `detector_contraindication_hash_v1` is DSFB-native; no
//! datasheet / model-card / NIST AI RMF / SLSA / in-toto / SPDX
//! / CycloneDX compatibility claim.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::seed::SEED;
use crate::types::{
    DetectorCanonicalId, ImplementationLevel, InputRequirementSet, NegativeWitnessKind,
    PrimitiveFamily, WitnessRole,
};

/// Domain separator for `detector_contraindication_hash_v1`.
/// **Panel-locked**; changing it changes every contraindication
/// receipt's namespace.
pub const DETECTOR_CONTRAINDICATION_DOMAIN: &str = "DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1\0";

/// Schema identifier carried inside the contraindication hash
/// material.
pub const DETECTOR_CONTRAINDICATION_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:DETECTOR-CONTRAINDICATION:v1";

/// Schema variant pinned in the receipt hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContraindicationSchema {
    /// T.11g base schema — datasheet-/model-card-/safety-label-
    /// inspired but DSFB-native.
    V1DatasheetLike,
}

impl ContraindicationSchema {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1DatasheetLike => "V1DatasheetLike",
        }
    }
}

/// Free-form "works best when" categorical hint. Enum-only so the
/// verifier never has to grep prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorksBestWhenCondition {
    /// Detector benefits from a stable baseline window.
    StableBaselineWindow,
    /// Detector designed for persistent residual elevation, not
    /// single-window spikes.
    PersistentResidualElevation,
    /// Detector expects a regularly-sampled input series.
    RegularSampling,
    /// Detector requires unit-resolved inputs.
    KnownUnits,
    /// Detector accuracy improves with sufficient sample size.
    SufficientSampleSize,
    /// Detector needs ordered-time input (sequential law).
    TimeOrderedInput,
    /// Detector consumes categorical or binned features.
    BinnedOrCategorical,
    /// Detector consumes numeric features.
    NumericInput,
    /// Detector consumes graph topology (nodes + edges).
    GraphTopologyPresent,
    /// Detector handles non-stationary inputs gracefully.
    NonStationaryInputs,
    /// Detector benefits from known seasonality period.
    SeasonalityKnown,
    /// Detector requires a baseline / reference distribution.
    BaselineReferenceAvailable,
    /// Detector benefits from an explicit missingness mask.
    MissingnessMaskPresent,
}

impl WorksBestWhenCondition {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StableBaselineWindow => "StableBaselineWindow",
            Self::PersistentResidualElevation => "PersistentResidualElevation",
            Self::RegularSampling => "RegularSampling",
            Self::KnownUnits => "KnownUnits",
            Self::SufficientSampleSize => "SufficientSampleSize",
            Self::TimeOrderedInput => "TimeOrderedInput",
            Self::BinnedOrCategorical => "BinnedOrCategorical",
            Self::NumericInput => "NumericInput",
            Self::GraphTopologyPresent => "GraphTopologyPresent",
            Self::NonStationaryInputs => "NonStationaryInputs",
            Self::SeasonalityKnown => "SeasonalityKnown",
            Self::BaselineReferenceAvailable => "BaselineReferenceAvailable",
            Self::MissingnessMaskPresent => "MissingnessMaskPresent",
        }
    }
}

/// "Fails when" categorical hint. Enumerable so the verifier can
/// reason about which failure modes are declared without prose
/// parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailsWhenCondition {
    /// Below-threshold sample size.
    SmallSampleSize,
    /// Baseline shifts during the window.
    NonStationaryBaseline,
    /// Sampling rate is not regular.
    IrregularSampling,
    /// Unit semantics ambiguous (ms vs µs, °C vs °F, ...).
    UnitsAmbiguous,
    /// Schema flips mid-window (column rename, type change).
    SchemaUnstable,
    /// High missingness contaminates the feature.
    HighMissingness,
    /// Detector fires on periodic-boundary effects.
    PeriodicBoundaryEffects,
    /// Single-observation spike instead of a real episode.
    SingleObservationSpike,
    /// Batch boundary causes artifactual firing.
    BatchBoundaryArtifact,
    /// Clock skew between sources contaminates the timing axis.
    ClockSkew,
    /// Deployment / build marker causes artifactual firing.
    DeploymentMarkerArtifact,
    /// Input contract is undeclared / unknown.
    UnknownInputContract,
}

impl FailsWhenCondition {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SmallSampleSize => "SmallSampleSize",
            Self::NonStationaryBaseline => "NonStationaryBaseline",
            Self::IrregularSampling => "IrregularSampling",
            Self::UnitsAmbiguous => "UnitsAmbiguous",
            Self::SchemaUnstable => "SchemaUnstable",
            Self::HighMissingness => "HighMissingness",
            Self::PeriodicBoundaryEffects => "PeriodicBoundaryEffects",
            Self::SingleObservationSpike => "SingleObservationSpike",
            Self::BatchBoundaryArtifact => "BatchBoundaryArtifact",
            Self::ClockSkew => "ClockSkew",
            Self::DeploymentMarkerArtifact => "DeploymentMarkerArtifact",
            Self::UnknownInputContract => "UnknownInputContract",
        }
    }
}

/// Known confuser binding — a direct cross-link to a
/// `NegativeWitnessKind` variant. The verifier ensures every
/// Primary-witness detector binds at least one confuser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnownConfuserBinding {
    /// Which negative witness variant blocks this detector.
    pub confuser: NegativeWitnessKind,
}

/// Sampling-law-kind: which class of sampling the detector
/// requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplingLawKind {
    /// Strict fixed-rate sampling (spectral, wavelet).
    RegularFixedRate,
    /// Time-ordered but irregular sampling acceptable.
    OrderedNonRegular,
    /// Unordered row set (tabular, graph).
    UnorderedRowSet,
    /// Graph-adjacency law (graph-local / graph-global detectors).
    GraphAdjacency,
}

impl SamplingLawKind {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegularFixedRate => "RegularFixedRate",
            Self::OrderedNonRegular => "OrderedNonRegular",
            Self::UnorderedRowSet => "UnorderedRowSet",
            Self::GraphAdjacency => "GraphAdjacency",
        }
    }
}

/// Sampling regularity tolerance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplingRegularity {
    /// Strictly regular (spectral / wavelet require this).
    StrictlyRegular,
    /// Small jitter tolerated within a declared envelope.
    JitterTolerated,
    /// Irregular sampling is admissible (rank statistics, etc.).
    IrregularAdmissible,
}

impl SamplingRegularity {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StrictlyRegular => "StrictlyRegular",
            Self::JitterTolerated => "JitterTolerated",
            Self::IrregularAdmissible => "IrregularAdmissible",
        }
    }
}

/// Required-sampling-law section: the panel-locked
/// "this detector NEEDS this sampling shape" declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequiredSamplingLaw {
    /// Which sampling-law class the detector requires.
    pub kind: SamplingLawKind,
    /// Minimum number of observations the detector needs to
    /// produce a verdict.
    pub min_observations: u32,
    /// How regular the sampling must be.
    pub regularity: SamplingRegularity,
}

/// Required-unit-semantics section: the detector's relationship to
/// physical units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitSemanticsKind {
    /// Detector requires explicit physical units (Hz, °C, ms,
    /// dB, ...).
    PhysicalUnitsRequired,
    /// Detector is a dimensionless ratio (works on normalised
    /// values).
    DimensionlessRatio,
    /// Detector operates on counts / cardinality (integer).
    CountOrCardinality,
    /// Detector operates on categorical labels (no units).
    CategoricalLabels,
    /// Detector operates on boolean state.
    BooleanState,
    /// Unit semantics not applicable (graph topology, schema
    /// constraints).
    None,
}

impl UnitSemanticsKind {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PhysicalUnitsRequired => "PhysicalUnitsRequired",
            Self::DimensionlessRatio => "DimensionlessRatio",
            Self::CountOrCardinality => "CountOrCardinality",
            Self::CategoricalLabels => "CategoricalLabels",
            Self::BooleanState => "BooleanState",
            Self::None => "None",
        }
    }
}

/// How precisely the unit must be declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitResolution {
    /// Exactly declared at ingestion.
    ExactDeclared,
    /// Inferred from baseline window distribution.
    InferredFromBaseline,
    /// Not applicable for this detector class.
    NotApplicable,
}

impl UnitResolution {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactDeclared => "ExactDeclared",
            Self::InferredFromBaseline => "InferredFromBaseline",
            Self::NotApplicable => "NotApplicable",
        }
    }
}

/// Required-unit-semantics composite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequiredUnitSemantics {
    /// Which unit-semantics class the detector requires.
    pub kind: UnitSemanticsKind,
    /// How precisely the unit must be declared at ingestion.
    pub min_unit_resolution: UnitResolution,
}

/// Minimum-support pre-conditions before the detector can fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MinimumSupport {
    /// Minimum baseline observations before the detector can fire.
    pub min_baseline_observations: u32,
    /// Minimum active observations to admit a firing.
    pub min_active_observations: u32,
    /// Minimum distinct entities (for cross-entity detectors).
    pub min_distinct_entities: u32,
}

/// Categorical "do not use for" disqualifier. Enum-only so the
/// verifier can enforce that Active detectors declare at least
/// one disqualifier (rule 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DoNotUseForReason {
    /// Streaming inputs without replay buffer.
    StreamingWithoutReplay,
    /// Unbounded history (detector windows must be finite).
    UnboundedHistory,
    /// Inputs without a declared sampling law.
    DataWithoutSampling,
    /// Inputs without a declared input contract.
    InputsWithoutDeclaredContract,
    /// Adversarial-evasion scenarios where the input is shaped to
    /// avoid the detector.
    AdversarialEvasionScenarios,
    /// Safety-critical decisions without human review.
    SafetyCriticalWithoutHumanReview,
    /// Probabilistic decision-making (the Atlas is deterministic
    /// only).
    ProbabilisticDecisionMaking,
    /// Black-box retrieval-augmentation where the LLM is the
    /// finder of facts (the Atlas does the deterministic pre-pass
    /// first; the LLM narrates).
    BlackBoxRetrievalAugmentation,
}

impl DoNotUseForReason {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StreamingWithoutReplay => "StreamingWithoutReplay",
            Self::UnboundedHistory => "UnboundedHistory",
            Self::DataWithoutSampling => "DataWithoutSampling",
            Self::InputsWithoutDeclaredContract => "InputsWithoutDeclaredContract",
            Self::AdversarialEvasionScenarios => "AdversarialEvasionScenarios",
            Self::SafetyCriticalWithoutHumanReview => "SafetyCriticalWithoutHumanReview",
            Self::ProbabilisticDecisionMaking => "ProbabilisticDecisionMaking",
            Self::BlackBoxRetrievalAugmentation => "BlackBoxRetrievalAugmentation",
        }
    }
}

/// Closest-alias binding: a detector this one is **legitimately**
/// closest to (because T.4 dedup court collapsed them under the
/// same canonical or one is a parameter variant of the other).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClosestAliasBinding {
    /// Canonical id of the closest legitimate alias.
    pub canonical_id: DetectorCanonicalId,
    /// Why this binding is considered an alias.
    pub similarity_reason: AliasSimilarityReason,
}

/// Why a closest-alias binding is legitimate (per T.4 dedup
/// court).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AliasSimilarityReason {
    /// Identical formula hash.
    IdenticalFormula,
    /// Identical semantic-role hash.
    IdenticalSemanticRole,
    /// Already declared alias in the T.4 dedup court.
    AliasInDedupCourt,
    /// Parameter variant only — same primitive, different
    /// parameter grid.
    ParameterVariantOnly,
}

impl AliasSimilarityReason {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IdenticalFormula => "IdenticalFormula",
            Self::IdenticalSemanticRole => "IdenticalSemanticRole",
            Self::AliasInDedupCourt => "AliasInDedupCourt",
            Self::ParameterVariantOnly => "ParameterVariantOnly",
        }
    }
}

/// Closest-non-alias binding: a detector that **looks** like an
/// alias on the surface but is in fact semantically distinct. This
/// is the "do not collapse us" claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClosestNonAliasBinding {
    /// Canonical id of the surface-similar but semantically
    /// distinct detector.
    pub canonical_id: DetectorCanonicalId,
    /// Why the two should NOT be collapsed.
    pub distinction_reason: NonAliasDistinctionReason,
}

/// Why a closest-non-alias binding is legitimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NonAliasDistinctionReason {
    /// Different decision functional (one-sided vs two-sided,
    /// etc.).
    DifferentDecisionFunctional,
    /// Different input contract (categorical vs numeric, etc.).
    DifferentInputContract,
    /// Different witness role (Primary vs Confuser).
    DifferentWitnessRole,
    /// Different sampling law (regular vs irregular).
    DifferentSamplingLaw,
    /// Different domain scope (tabular vs graph, etc.).
    DifferentDomainScope,
    /// Different mathematical form (RobustZ vs Tukey fence).
    DifferentMathematicalForm,
}

impl NonAliasDistinctionReason {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DifferentDecisionFunctional => "DifferentDecisionFunctional",
            Self::DifferentInputContract => "DifferentInputContract",
            Self::DifferentWitnessRole => "DifferentWitnessRole",
            Self::DifferentSamplingLaw => "DifferentSamplingLaw",
            Self::DifferentDomainScope => "DifferentDomainScope",
            Self::DifferentMathematicalForm => "DifferentMathematicalForm",
        }
    }
}

/// Adversarial-twin relation: detectors that are likely to be
/// confused with the subject and the kind of confusion involved.
/// Sharpens the Anti-Hallucination Ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectorTwinRelation {
    /// Same formula but plays a different witness role.
    SameFormulaDifferentRole(DetectorCanonicalId),
    /// Same witness role but uses a different formula.
    SameRoleDifferentFormula(DetectorCanonicalId),
    /// Same primitive family but requires a different sampling
    /// law.
    SameFamilyDifferentSamplingLaw(DetectorCanonicalId),
    /// Looks like an alias by surface name but is semantically
    /// distinct.
    AliasLikeButSemanticallyDistinct(DetectorCanonicalId),
    /// Is the canonical confuser of this primary witness.
    ConfuserOfPrimary(DetectorCanonicalId),
}

impl DetectorTwinRelation {
    /// Stable wire name for the variant kind; the canonical id is
    /// hashed separately.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::SameFormulaDifferentRole(_) => "SameFormulaDifferentRole",
            Self::SameRoleDifferentFormula(_) => "SameRoleDifferentFormula",
            Self::SameFamilyDifferentSamplingLaw(_) => "SameFamilyDifferentSamplingLaw",
            Self::AliasLikeButSemanticallyDistinct(_) => "AliasLikeButSemanticallyDistinct",
            Self::ConfuserOfPrimary(_) => "ConfuserOfPrimary",
        }
    }

    /// The canonical id this twin relation points at.
    #[must_use]
    pub const fn target(&self) -> DetectorCanonicalId {
        match self {
            Self::SameFormulaDifferentRole(id)
            | Self::SameRoleDifferentFormula(id)
            | Self::SameFamilyDifferentSamplingLaw(id)
            | Self::AliasLikeButSemanticallyDistinct(id)
            | Self::ConfuserOfPrimary(id) => *id,
        }
    }
}

/// One per-detector contraindication receipt. Built deterministically
/// from each `LiteratureDetector` in `SEED` via
/// [`collect_contraindications`].
#[derive(Debug, Clone)]
pub struct DetectorContraindicationReceiptV1 {
    /// Canonical detector this receipt describes.
    pub canonical_id: DetectorCanonicalId,
    /// Operator-readable name, mirrored from the SEED record.
    pub display_name: &'static str,
    /// Witness role mirrored from the SEED record.
    pub witness_role: WitnessRole,
    /// Primitive family mirrored from the SEED record.
    pub primitive_family: PrimitiveFamily,
    /// Implementation level mirrored from the SEED record.
    pub implementation_level: ImplementationLevel,
    /// Categorical "works best when" conditions.
    pub works_best_when: Vec<WorksBestWhenCondition>,
    /// Categorical "fails when" conditions.
    pub fails_when: Vec<FailsWhenCondition>,
    /// Known confusers (cross-linked to T.6 `NegativeWitnessKind`).
    pub known_confusers: Vec<KnownConfuserBinding>,
    /// Required sampling law (None when not applicable).
    pub required_sampling_law: Option<RequiredSamplingLaw>,
    /// Required unit semantics (None when not applicable).
    pub required_units: Option<RequiredUnitSemantics>,
    /// Minimum support pre-conditions.
    pub minimum_support: MinimumSupport,
    /// "Do not use for" categorical disqualifiers.
    pub do_not_use_for: Vec<DoNotUseForReason>,
    /// Closest legitimate aliases (T.4 court).
    pub closest_aliases: Vec<ClosestAliasBinding>,
    /// Closest non-aliases (surface-similar but semantically
    /// distinct).
    pub closest_non_aliases: Vec<ClosestNonAliasBinding>,
    /// Adversarial-twin relations.
    pub adversarial_twins: Vec<DetectorTwinRelation>,
}

/// A collected, deterministically-sorted contraindication
/// snapshot.
#[derive(Debug, Clone)]
pub struct ContraindicationSnapshot {
    /// Schema variant.
    pub schema: ContraindicationSchema,
    /// One receipt per canonical detector in `SEED`, sorted by
    /// `canonical_id` ascending.
    pub receipts: Vec<DetectorContraindicationReceiptV1>,
}

fn requires(set: InputRequirementSet, bit: u32) -> bool {
    (set.0 & bit) != 0
}

/// Derive the default `works_best_when` for a record from its
/// primitive family + input requirements. The verifier rules below
/// guarantee that whatever this returns satisfies them.
fn derive_works_best_when(
    family: PrimitiveFamily,
    input_requirements: InputRequirementSet,
) -> Vec<WorksBestWhenCondition> {
    let mut v: Vec<WorksBestWhenCondition> = Vec::new();
    v.push(WorksBestWhenCondition::SufficientSampleSize);
    if requires(input_requirements, InputRequirementSet::BASELINE_WINDOW) {
        v.push(WorksBestWhenCondition::StableBaselineWindow);
    }
    if requires(input_requirements, InputRequirementSet::REGULAR_SAMPLING) {
        v.push(WorksBestWhenCondition::RegularSampling);
    }
    if requires(input_requirements, InputRequirementSet::ORDERED_TIME) {
        v.push(WorksBestWhenCondition::TimeOrderedInput);
    }
    if requires(input_requirements, InputRequirementSet::NUMERIC) {
        v.push(WorksBestWhenCondition::NumericInput);
    }
    if requires(input_requirements, InputRequirementSet::CATEGORICAL) {
        v.push(WorksBestWhenCondition::BinnedOrCategorical);
    }
    if requires(input_requirements, InputRequirementSet::GRAPH) {
        v.push(WorksBestWhenCondition::GraphTopologyPresent);
    }
    if requires(
        input_requirements,
        InputRequirementSet::REFERENCE_DISTRIBUTION,
    ) {
        v.push(WorksBestWhenCondition::BaselineReferenceAvailable);
    }
    if requires(input_requirements, InputRequirementSet::SEASONALITY_PERIOD) {
        v.push(WorksBestWhenCondition::SeasonalityKnown);
    }
    if requires(input_requirements, InputRequirementSet::UNITS) {
        v.push(WorksBestWhenCondition::KnownUnits);
    }
    if requires(input_requirements, InputRequirementSet::MISSINGNESS_MASK) {
        v.push(WorksBestWhenCondition::MissingnessMaskPresent);
    }
    if matches!(
        family,
        PrimitiveFamily::SequentialRecurrence | PrimitiveFamily::DebugObservability
    ) {
        v.push(WorksBestWhenCondition::PersistentResidualElevation);
    }
    v.sort();
    v.dedup();
    v
}

/// Derive `fails_when` from the family + input requirements.
fn derive_fails_when(
    family: PrimitiveFamily,
    input_requirements: InputRequirementSet,
) -> Vec<FailsWhenCondition> {
    let mut v: Vec<FailsWhenCondition> = Vec::new();
    v.push(FailsWhenCondition::SmallSampleSize);
    if requires(input_requirements, InputRequirementSet::REGULAR_SAMPLING) {
        v.push(FailsWhenCondition::IrregularSampling);
    }
    if requires(input_requirements, InputRequirementSet::UNITS) {
        v.push(FailsWhenCondition::UnitsAmbiguous);
    }
    if requires(input_requirements, InputRequirementSet::BASELINE_WINDOW) {
        v.push(FailsWhenCondition::NonStationaryBaseline);
    }
    if requires(input_requirements, InputRequirementSet::MISSINGNESS_MASK)
        || matches!(family, PrimitiveFamily::Missingness)
    {
        v.push(FailsWhenCondition::HighMissingness);
    }
    if matches!(family, PrimitiveFamily::TabularConstraint) {
        v.push(FailsWhenCondition::SchemaUnstable);
    }
    if matches!(family, PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet) {
        v.push(FailsWhenCondition::PeriodicBoundaryEffects);
    }
    if matches!(
        family,
        PrimitiveFamily::DebugObservability | PrimitiveFamily::SequentialRecurrence
    ) {
        v.push(FailsWhenCondition::SingleObservationSpike);
        v.push(FailsWhenCondition::BatchBoundaryArtifact);
        v.push(FailsWhenCondition::ClockSkew);
    }
    v.sort();
    v.dedup();
    v
}

/// Derive default `known_confusers` for a record. Primary
/// witnesses get at least one; Confuser witnesses themselves get
/// no entries (the confuser IS a negative witness).
fn derive_known_confusers(role: WitnessRole, family: PrimitiveFamily) -> Vec<KnownConfuserBinding> {
    if matches!(role, WitnessRole::Confuser) {
        return Vec::new();
    }
    let mut v: Vec<KnownConfuserBinding> = Vec::new();
    v.push(KnownConfuserBinding {
        confuser: NegativeWitnessKind::SmallSampleConfuser,
    });
    v.push(KnownConfuserBinding {
        confuser: NegativeWitnessKind::SingleWindowSpikeConfuser,
    });
    if matches!(family, PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet) {
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::PeriodicBoundaryConfuser,
        });
    }
    if matches!(
        family,
        PrimitiveFamily::DistributionDistance
            | PrimitiveFamily::CategoricalHistogram
            | PrimitiveFamily::TabularConstraint
    ) {
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::SchemaChangeConfuser,
        });
    }
    if matches!(family, PrimitiveFamily::Missingness) {
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::MissingnessArtifactConfuser,
        });
    }
    if matches!(
        family,
        PrimitiveFamily::DebugObservability | PrimitiveFamily::SequentialRecurrence
    ) {
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::BatchBoundaryConfuser,
        });
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::ClockSkewConfuser,
        });
        v.push(KnownConfuserBinding {
            confuser: NegativeWitnessKind::DeploymentMarkerConfuser,
        });
    }
    v
}

/// Derive `required_sampling_law` from family + input
/// requirements.
fn derive_required_sampling_law(
    family: PrimitiveFamily,
    input_requirements: InputRequirementSet,
) -> Option<RequiredSamplingLaw> {
    let kind = match family {
        PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet => SamplingLawKind::RegularFixedRate,
        PrimitiveFamily::GraphLocal | PrimitiveFamily::GraphGlobal => {
            SamplingLawKind::GraphAdjacency
        }
        PrimitiveFamily::TabularConstraint
        | PrimitiveFamily::CategoricalHistogram
        | PrimitiveFamily::Missingness => SamplingLawKind::UnorderedRowSet,
        _ if requires(input_requirements, InputRequirementSet::REGULAR_SAMPLING) => {
            SamplingLawKind::RegularFixedRate
        }
        _ if requires(input_requirements, InputRequirementSet::ORDERED_TIME) => {
            SamplingLawKind::OrderedNonRegular
        }
        _ => return None,
    };
    let regularity = match kind {
        SamplingLawKind::RegularFixedRate => SamplingRegularity::StrictlyRegular,
        SamplingLawKind::OrderedNonRegular => SamplingRegularity::JitterTolerated,
        SamplingLawKind::UnorderedRowSet | SamplingLawKind::GraphAdjacency => {
            SamplingRegularity::IrregularAdmissible
        }
    };
    let min_observations = match kind {
        SamplingLawKind::RegularFixedRate => 32,
        SamplingLawKind::OrderedNonRegular => 16,
        SamplingLawKind::UnorderedRowSet => 8,
        SamplingLawKind::GraphAdjacency => 4,
    };
    Some(RequiredSamplingLaw {
        kind,
        min_observations,
        regularity,
    })
}

/// Derive `required_units` from input requirements + family.
fn derive_required_units(
    family: PrimitiveFamily,
    input_requirements: InputRequirementSet,
) -> Option<RequiredUnitSemantics> {
    if requires(input_requirements, InputRequirementSet::UNITS) {
        return Some(RequiredUnitSemantics {
            kind: UnitSemanticsKind::PhysicalUnitsRequired,
            min_unit_resolution: UnitResolution::ExactDeclared,
        });
    }
    match family {
        PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet => Some(RequiredUnitSemantics {
            kind: UnitSemanticsKind::PhysicalUnitsRequired,
            min_unit_resolution: UnitResolution::ExactDeclared,
        }),
        PrimitiveFamily::CategoricalHistogram => Some(RequiredUnitSemantics {
            kind: UnitSemanticsKind::CategoricalLabels,
            min_unit_resolution: UnitResolution::NotApplicable,
        }),
        PrimitiveFamily::Missingness => Some(RequiredUnitSemantics {
            kind: UnitSemanticsKind::BooleanState,
            min_unit_resolution: UnitResolution::NotApplicable,
        }),
        PrimitiveFamily::GraphLocal | PrimitiveFamily::GraphGlobal => None,
        // All other primitive families: dimensionless-ratio with
        // baseline-inferred resolution. Captures the common case
        // (scalar threshold, rolling window, rank statistic,
        // sequential recurrence, distribution distance, residual
        // observer, projection residual, multivariate hypothesis,
        // information-theoretic detectors, etc.).
        _ => Some(RequiredUnitSemantics {
            kind: UnitSemanticsKind::DimensionlessRatio,
            min_unit_resolution: UnitResolution::InferredFromBaseline,
        }),
    }
}

/// Derive `do_not_use_for` for a record. Every Active detector
/// declares at least one disqualifier (panel-locked rule 11).
fn derive_do_not_use_for(family: PrimitiveFamily) -> Vec<DoNotUseForReason> {
    let mut v: Vec<DoNotUseForReason> = Vec::new();
    v.push(DoNotUseForReason::InputsWithoutDeclaredContract);
    v.push(DoNotUseForReason::ProbabilisticDecisionMaking);
    v.push(DoNotUseForReason::BlackBoxRetrievalAugmentation);
    if matches!(
        family,
        PrimitiveFamily::Spectral
            | PrimitiveFamily::Wavelet
            | PrimitiveFamily::SequentialRecurrence
            | PrimitiveFamily::DebugObservability
    ) {
        v.push(DoNotUseForReason::DataWithoutSampling);
    }
    v.push(DoNotUseForReason::AdversarialEvasionScenarios);
    v.push(DoNotUseForReason::SafetyCriticalWithoutHumanReview);
    v.sort();
    v.dedup();
    v
}

/// Build the contraindication snapshot deterministically from
/// the corpus SEED.
#[must_use]
pub fn collect_contraindications() -> ContraindicationSnapshot {
    let mut receipts: Vec<DetectorContraindicationReceiptV1> = SEED
        .iter()
        .map(|r| DetectorContraindicationReceiptV1 {
            canonical_id: r.canonical_id,
            display_name: r.display_name,
            witness_role: r.witness_role,
            primitive_family: r.primitive_family,
            implementation_level: r.implementation_status,
            works_best_when: derive_works_best_when(r.primitive_family, r.input_requirements),
            fails_when: derive_fails_when(r.primitive_family, r.input_requirements),
            known_confusers: derive_known_confusers(r.witness_role, r.primitive_family),
            required_sampling_law: derive_required_sampling_law(
                r.primitive_family,
                r.input_requirements,
            ),
            required_units: derive_required_units(r.primitive_family, r.input_requirements),
            minimum_support: MinimumSupport {
                min_baseline_observations: 8,
                min_active_observations: 2,
                min_distinct_entities: 1,
            },
            do_not_use_for: derive_do_not_use_for(r.primitive_family),
            closest_aliases: Vec::new(),
            closest_non_aliases: Vec::new(),
            adversarial_twins: Vec::new(),
        })
        .collect();
    receipts.sort_by_key(|r| r.canonical_id.0);
    ContraindicationSnapshot {
        schema: ContraindicationSchema::V1DatasheetLike,
        receipts,
    }
}

// ---------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_opt_sampling(out: &mut Vec<u8>, s: Option<&RequiredSamplingLaw>) {
    match s {
        None => write_u8(out, 0),
        Some(law) => {
            write_u8(out, 1);
            write_str(out, law.kind.as_str());
            write_u32(out, law.min_observations);
            write_str(out, law.regularity.as_str());
        }
    }
}

fn write_opt_units(out: &mut Vec<u8>, s: Option<&RequiredUnitSemantics>) {
    match s {
        None => write_u8(out, 0),
        Some(u) => {
            write_u8(out, 1);
            write_str(out, u.kind.as_str());
            write_str(out, u.min_unit_resolution.as_str());
        }
    }
}

fn write_receipt(out: &mut Vec<u8>, r: &DetectorContraindicationReceiptV1) {
    write_u32(out, r.canonical_id.0);
    write_str(out, r.display_name);
    write_str(out, r.witness_role.as_str());
    write_str(out, r.primitive_family.as_str());
    write_str(out, r.implementation_level.as_str());

    // Sorted vector fields are already sorted by construction;
    // sort defensively to make hand-built fixtures hash-equal.
    let mut wbw: Vec<WorksBestWhenCondition> = r.works_best_when.clone();
    wbw.sort();
    write_u32(out, u32::try_from(wbw.len()).unwrap_or(u32::MAX));
    for w in wbw {
        write_str(out, w.as_str());
    }

    let mut fw: Vec<FailsWhenCondition> = r.fails_when.clone();
    fw.sort();
    write_u32(out, u32::try_from(fw.len()).unwrap_or(u32::MAX));
    for f in fw {
        write_str(out, f.as_str());
    }

    let mut kc: Vec<NegativeWitnessKind> = r.known_confusers.iter().map(|b| b.confuser).collect();
    kc.sort_by_key(NegativeWitnessKind::as_str);
    write_u32(out, u32::try_from(kc.len()).unwrap_or(u32::MAX));
    for c in kc {
        write_str(out, c.as_str());
    }

    write_opt_sampling(out, r.required_sampling_law.as_ref());
    write_opt_units(out, r.required_units.as_ref());

    write_u32(out, r.minimum_support.min_baseline_observations);
    write_u32(out, r.minimum_support.min_active_observations);
    write_u32(out, r.minimum_support.min_distinct_entities);

    let mut dnu: Vec<DoNotUseForReason> = r.do_not_use_for.clone();
    dnu.sort();
    write_u32(out, u32::try_from(dnu.len()).unwrap_or(u32::MAX));
    for d in dnu {
        write_str(out, d.as_str());
    }

    let mut aliases: Vec<&ClosestAliasBinding> = r.closest_aliases.iter().collect();
    aliases.sort_by_key(|a| (a.canonical_id.0, a.similarity_reason.as_str()));
    write_u32(out, u32::try_from(aliases.len()).unwrap_or(u32::MAX));
    for a in aliases {
        write_u32(out, a.canonical_id.0);
        write_str(out, a.similarity_reason.as_str());
    }

    let mut non_aliases: Vec<&ClosestNonAliasBinding> = r.closest_non_aliases.iter().collect();
    non_aliases.sort_by_key(|a| (a.canonical_id.0, a.distinction_reason.as_str()));
    write_u32(out, u32::try_from(non_aliases.len()).unwrap_or(u32::MAX));
    for a in non_aliases {
        write_u32(out, a.canonical_id.0);
        write_str(out, a.distinction_reason.as_str());
    }

    let mut twins: Vec<&DetectorTwinRelation> = r.adversarial_twins.iter().collect();
    twins.sort_by_key(|t| (t.kind_str(), t.target().0));
    write_u32(out, u32::try_from(twins.len()).unwrap_or(u32::MAX));
    for t in twins {
        write_str(out, t.kind_str());
        write_u32(out, t.target().0);
    }
}

/// Compute the canonical-byte hash of the contraindication
/// snapshot. Two calls on the same snapshot produce
/// byte-identical output. Rendered text/JSON is NOT included.
#[must_use]
pub fn compute_contraindication_hash_v1(s: &ContraindicationSnapshot) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(DETECTOR_CONTRAINDICATION_DOMAIN.as_bytes());
    write_str(&mut buf, DETECTOR_CONTRAINDICATION_SCHEMA_V1);
    write_str(&mut buf, s.schema.as_str());

    let mut receipts: Vec<&DetectorContraindicationReceiptV1> = s.receipts.iter().collect();
    receipts.sort_by_key(|r| r.canonical_id.0);
    write_u32(&mut buf, u32::try_from(receipts.len()).unwrap_or(u32::MAX));
    for r in receipts {
        write_receipt(&mut buf, r);
    }

    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// A single contraindication verifier error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContraindicationVerifyError {
    /// Which canonical detector the error applies to (or
    /// `DetectorCanonicalId(0)` for snapshot-global errors).
    pub canonical_id: DetectorCanonicalId,
    /// The reject kind.
    pub kind: ContraindicationVerifyErrorKind,
}

/// The 11 panel-locked verifier reject kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContraindicationVerifyErrorKind {
    /// Primary witness without at least one known confuser.
    PrimaryWithoutKnownConfuser,
    /// L4+ detector without any declared contraindication
    /// (the receipt is empty).
    LBandL4PlusWithoutContraindications,
    /// L5/L6 detector without a `required_sampling_law`.
    LBandL5OrL6WithoutRequiredSamplingLaw,
    /// Unit-sensitive detector without `required_units`. A
    /// detector is unit-sensitive if either:
    /// - its `InputRequirementSet::UNITS` bit is set, OR
    /// - its primitive family is `Spectral` or `Wavelet`
    ///   (intrinsically frequency-resolved → requires
    ///   physical units, no exceptions).
    UnitSensitiveWithoutUnitSemantics,
    /// Spectral detector without `required_sampling_law`.
    SpectralWithoutSamplingLaw,
    /// Time-series detector (requires_ordered_time) without an
    /// ordered-time `works_best_when` or `required_sampling_law`.
    TimeSeriesWithoutOrderedTimeDeclaration,
    /// Distribution detector without a `BaselineReferenceAvailable`
    /// works-best-when entry.
    DistributionWithoutReferenceBaseline,
    /// Closest-alias binding points at a canonical id not in SEED.
    ClosestAliasMissing,
    /// Closest-non-alias binding points at a canonical id not in
    /// SEED.
    ClosestNonAliasMissing,
    /// Receipt carries no `do_not_use_for` and no
    /// `fails_when` entries (truly empty contraindication).
    ContraindicationWithoutCrossReference,
    /// `do_not_use_for` is empty on an Active detector (Primary,
    /// Corroborating, Boundary, CleanWindow, Recovery, Timing,
    /// Distribution, Topology, CausalityProxy).
    ActiveWithoutDoNotUseFor,
    /// Adversarial twin points at a canonical id not in SEED.
    AdversarialTwinMissing,
    /// Adversarial twin points at the subject itself
    /// (self-reference).
    AdversarialTwinSelfReference,
    /// Subject's `canonical_id` is not in SEED.
    UnknownDetector,
    /// Two receipts share the same `canonical_id`.
    DuplicateReceipt,
}

/// Run the brutal contraindication verifier. Empty vector means
/// the snapshot is admissible.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_contraindications(s: &ContraindicationSnapshot) -> Vec<ContraindicationVerifyError> {
    let mut errors: Vec<ContraindicationVerifyError> = Vec::new();
    let known_ids: Vec<DetectorCanonicalId> = SEED.iter().map(|r| r.canonical_id).collect();
    let known_id_set: Vec<u32> = known_ids.iter().map(|id| id.0).collect();
    let input_req_for = |id: DetectorCanonicalId| -> Option<InputRequirementSet> {
        SEED.iter()
            .find(|r| r.canonical_id == id)
            .map(|r| r.input_requirements)
    };

    // Duplicate ids.
    {
        let mut seen: Vec<u32> = Vec::with_capacity(s.receipts.len());
        for r in &s.receipts {
            if seen.contains(&r.canonical_id.0) {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::DuplicateReceipt,
                });
            } else {
                seen.push(r.canonical_id.0);
            }
        }
    }

    for r in &s.receipts {
        // Unknown detector.
        if !known_id_set.contains(&r.canonical_id.0) {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::UnknownDetector,
            });
            continue;
        }

        let input_req = input_req_for(r.canonical_id).unwrap_or(InputRequirementSet(0));

        // Rule 1: Primary witness without at least one known confuser.
        if matches!(r.witness_role, WitnessRole::Primary) && r.known_confusers.is_empty() {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::PrimaryWithoutKnownConfuser,
            });
        }

        // Rule 2: L4+ detector without ANY contraindication.
        let is_l4_plus = matches!(
            r.implementation_level,
            ImplementationLevel::L4_CpuVerified
                | ImplementationLevel::L5_GpuImplemented
                | ImplementationLevel::L6_CpuGpuByteEquivalent
                | ImplementationLevel::L7_BenchmarkCharacterised
                | ImplementationLevel::L8_LedgerCharacterised
        );
        let empty_receipt = r.works_best_when.is_empty()
            && r.fails_when.is_empty()
            && r.known_confusers.is_empty()
            && r.required_sampling_law.is_none()
            && r.required_units.is_none()
            && r.do_not_use_for.is_empty();
        if is_l4_plus && empty_receipt {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::LBandL4PlusWithoutContraindications,
            });
        }

        // Rule 3: L5/L6 without required_sampling_law.
        let is_l5_l6 = matches!(
            r.implementation_level,
            ImplementationLevel::L5_GpuImplemented | ImplementationLevel::L6_CpuGpuByteEquivalent
        );
        if is_l5_l6 && r.required_sampling_law.is_none() {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::LBandL5OrL6WithoutRequiredSamplingLaw,
            });
        }

        // Rule 4: unit-sensitive without required_units. The
        // panel-locked definition of "unit-sensitive" covers two
        // populations: detectors with `InputRequirementSet::UNITS`
        // bit set AND spectral/wavelet primitive families
        // (frequency-resolved detectors intrinsically require
        // physical units).
        let unit_sensitive = requires(input_req, InputRequirementSet::UNITS)
            || matches!(
                r.primitive_family,
                PrimitiveFamily::Spectral | PrimitiveFamily::Wavelet
            );
        if unit_sensitive && r.required_units.is_none() {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::UnitSensitiveWithoutUnitSemantics,
            });
        }

        // Rule 5: spectral without sampling-law.
        if matches!(r.primitive_family, PrimitiveFamily::Spectral)
            && r.required_sampling_law.is_none()
        {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::SpectralWithoutSamplingLaw,
            });
        }

        // Rule 6: time-series detector without an ordered-time
        // declaration anywhere.
        if requires(input_req, InputRequirementSet::ORDERED_TIME) {
            let has_ot_in_wbw = r
                .works_best_when
                .iter()
                .any(|w| matches!(w, WorksBestWhenCondition::TimeOrderedInput));
            let has_ot_in_law = r.required_sampling_law.as_ref().is_some_and(|law| {
                matches!(
                    law.kind,
                    SamplingLawKind::RegularFixedRate | SamplingLawKind::OrderedNonRegular
                )
            });
            if !has_ot_in_wbw && !has_ot_in_law {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::TimeSeriesWithoutOrderedTimeDeclaration,
                });
            }
        }

        // Rule 7: distribution detector without baseline reference.
        if matches!(r.primitive_family, PrimitiveFamily::DistributionDistance) {
            let has_baseline = r
                .works_best_when
                .iter()
                .any(|w| matches!(w, WorksBestWhenCondition::BaselineReferenceAvailable));
            if !has_baseline {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::DistributionWithoutReferenceBaseline,
                });
            }
        }

        // Rule 8: closest_alias missing.
        for a in &r.closest_aliases {
            if !known_id_set.contains(&a.canonical_id.0) {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::ClosestAliasMissing,
                });
            }
        }

        // Rule 9: closest_non_alias missing.
        for a in &r.closest_non_aliases {
            if !known_id_set.contains(&a.canonical_id.0) {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::ClosestNonAliasMissing,
                });
            }
        }

        // Rule 10: contraindication without cross-reference.
        let no_cross_ref = r.do_not_use_for.is_empty()
            && r.fails_when.is_empty()
            && r.known_confusers.is_empty()
            && r.closest_non_aliases.is_empty();
        if no_cross_ref {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::ContraindicationWithoutCrossReference,
            });
        }

        // Rule 11: do_not_use_for empty on Active detectors.
        let is_active_role = !matches!(
            r.witness_role,
            WitnessRole::Confuser | WitnessRole::CleanWindow
        );
        if is_active_role && r.do_not_use_for.is_empty() {
            errors.push(ContraindicationVerifyError {
                canonical_id: r.canonical_id,
                kind: ContraindicationVerifyErrorKind::ActiveWithoutDoNotUseFor,
            });
        }

        // Adversarial-twin checks.
        for t in &r.adversarial_twins {
            let tid = t.target();
            if !known_id_set.contains(&tid.0) {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::AdversarialTwinMissing,
                });
            }
            if tid == r.canonical_id {
                errors.push(ContraindicationVerifyError {
                    canonical_id: r.canonical_id,
                    kind: ContraindicationVerifyErrorKind::AdversarialTwinSelfReference,
                });
            }
        }
    }

    errors
}

// ---------------------------------------------------------------
// Passport / contraindication crosswalk (separate artifact, panel-
// locked: does NOT mutate passport hashes)
// ---------------------------------------------------------------

/// One row in the passport ↔ contraindication crosswalk. The
/// crosswalk lives in its own namespace so it can evolve without
/// churning DetectorPassport hashes.
#[derive(Debug, Clone, Copy)]
pub struct PassportContraindicationCrosswalkRow {
    /// Canonical id.
    pub canonical_id: DetectorCanonicalId,
    /// Display name (mirrored from SEED for operator legibility).
    pub display_name: &'static str,
    /// Number of `known_confusers` declared.
    pub known_confuser_count: u32,
    /// Number of `do_not_use_for` reasons.
    pub do_not_use_for_count: u32,
    /// Number of `closest_aliases` entries.
    pub closest_alias_count: u32,
    /// Number of `closest_non_aliases` entries.
    pub closest_non_alias_count: u32,
    /// Number of `adversarial_twins` entries.
    pub adversarial_twin_count: u32,
}

/// Build the passport↔contraindication crosswalk from a
/// contraindication snapshot. The crosswalk is rendered separately
/// from the receipt body; the receipt hash already covers the same
/// underlying facts.
#[must_use]
pub fn build_passport_crosswalk(
    s: &ContraindicationSnapshot,
) -> Vec<PassportContraindicationCrosswalkRow> {
    let mut rows: Vec<PassportContraindicationCrosswalkRow> = s
        .receipts
        .iter()
        .map(|r| PassportContraindicationCrosswalkRow {
            canonical_id: r.canonical_id,
            display_name: r.display_name,
            known_confuser_count: u32::try_from(r.known_confusers.len()).unwrap_or(u32::MAX),
            do_not_use_for_count: u32::try_from(r.do_not_use_for.len()).unwrap_or(u32::MAX),
            closest_alias_count: u32::try_from(r.closest_aliases.len()).unwrap_or(u32::MAX),
            closest_non_alias_count: u32::try_from(r.closest_non_aliases.len()).unwrap_or(u32::MAX),
            adversarial_twin_count: u32::try_from(r.adversarial_twins.len()).unwrap_or(u32::MAX),
        })
        .collect();
    rows.sort_by_key(|r| r.canonical_id.0);
    rows
}

// ---------------------------------------------------------------
// Renderers (text + JSON)
// ---------------------------------------------------------------

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// Render the snapshot as deterministic operator-readable text.
/// Two calls produce byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_contraindications_text(s: &ContraindicationSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    let h = compute_contraindication_hash_v1(s);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(
        out,
        "DSFB-GPU-Atlas - Detector Contraindication Receipts V1 (T.11g)"
    );
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(
        out,
        "schema                          : {}",
        s.schema.as_str()
    );
    let _ = writeln!(out, "detector_contraindication_hash_v1: {}", hex(&h));
    let _ = writeln!(
        out,
        "receipt_count                   : {}",
        s.receipts.len()
    );
    let _ = writeln!(out);

    for r in &s.receipts {
        let _ = writeln!(
            out,
            "----------------------------------------------------------------"
        );
        let _ = writeln!(
            out,
            "#{:<4} {:<48} role={} L={}",
            r.canonical_id.0,
            r.display_name,
            r.witness_role.as_str(),
            r.implementation_level.as_str(),
        );
        let _ = writeln!(out, "  family            : {}", r.primitive_family.as_str());
        if !r.works_best_when.is_empty() {
            let _ = write!(out, "  works_best_when   :");
            for w in &r.works_best_when {
                let _ = write!(out, " {}", w.as_str());
            }
            let _ = writeln!(out);
        }
        if !r.fails_when.is_empty() {
            let _ = write!(out, "  fails_when        :");
            for f in &r.fails_when {
                let _ = write!(out, " {}", f.as_str());
            }
            let _ = writeln!(out);
        }
        if !r.known_confusers.is_empty() {
            let _ = write!(out, "  known_confusers   :");
            for c in &r.known_confusers {
                let _ = write!(out, " {}", c.confuser.as_str());
            }
            let _ = writeln!(out);
        }
        if let Some(law) = &r.required_sampling_law {
            let _ = writeln!(
                out,
                "  sampling_law      : {} regularity={} min_obs={}",
                law.kind.as_str(),
                law.regularity.as_str(),
                law.min_observations,
            );
        }
        if let Some(u) = &r.required_units {
            let _ = writeln!(
                out,
                "  required_units    : {} resolution={}",
                u.kind.as_str(),
                u.min_unit_resolution.as_str(),
            );
        }
        let _ = writeln!(
            out,
            "  minimum_support   : baseline={} active={} entities={}",
            r.minimum_support.min_baseline_observations,
            r.minimum_support.min_active_observations,
            r.minimum_support.min_distinct_entities,
        );
        if !r.do_not_use_for.is_empty() {
            let _ = write!(out, "  do_not_use_for    :");
            for d in &r.do_not_use_for {
                let _ = write!(out, " {}", d.as_str());
            }
            let _ = writeln!(out);
        }
        if !r.closest_aliases.is_empty() {
            let _ = write!(out, "  closest_aliases   :");
            for a in &r.closest_aliases {
                let _ = write!(
                    out,
                    " #{}({})",
                    a.canonical_id.0,
                    a.similarity_reason.as_str(),
                );
            }
            let _ = writeln!(out);
        }
        if !r.closest_non_aliases.is_empty() {
            let _ = write!(out, "  closest_non_alias :");
            for a in &r.closest_non_aliases {
                let _ = write!(
                    out,
                    " #{}({})",
                    a.canonical_id.0,
                    a.distinction_reason.as_str(),
                );
            }
            let _ = writeln!(out);
        }
        if !r.adversarial_twins.is_empty() {
            let _ = write!(out, "  adversarial_twins :");
            for t in &r.adversarial_twins {
                let _ = write!(out, " {}#{}", t.kind_str(), t.target().0);
            }
            let _ = writeln!(out);
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Panel-locked non-claim: T.11g is a DSFB-native");
    let _ = writeln!(out, "datasheet-/model-card-/safety-label-inspired surface.");
    let _ = writeln!(out, "It does NOT mutate corpus / registry / precedent /");
    let _ = writeln!(out, "grammar / transcript / receipt / passport hashes; the");
    let _ = writeln!(out, "passport linkage lives in a separate crosswalk.");
    out
}

/// Render the snapshot as deterministic JSON. Two calls produce
/// byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_contraindications_json(s: &ContraindicationSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    let h = compute_contraindication_hash_v1(s);
    let _ = write!(out, "{{");
    let _ = write!(out, "\"schema\":\"{}\",", s.schema.as_str());
    let _ = write!(
        out,
        "\"schema_id\":\"{DETECTOR_CONTRAINDICATION_SCHEMA_V1}\",",
    );
    let _ = write!(
        out,
        "\"detector_contraindication_hash_v1\":\"{}\",",
        hex(&h),
    );
    let _ = write!(out, "\"receipts\":[");
    let mut first = true;
    for r in &s.receipts {
        if !first {
            let _ = write!(out, ",");
        }
        first = false;
        let _ = write!(out, "{{");
        let _ = write!(out, "\"canonical_id\":{},", r.canonical_id.0);
        let _ = write!(out, "\"display_name\":\"{}\",", json_escape(r.display_name),);
        let _ = write!(out, "\"witness_role\":\"{}\",", r.witness_role.as_str());
        let _ = write!(
            out,
            "\"primitive_family\":\"{}\",",
            r.primitive_family.as_str(),
        );
        let _ = write!(
            out,
            "\"implementation_level\":\"{}\",",
            r.implementation_level.as_str(),
        );
        let _ = write!(out, "\"works_best_when\":[");
        for (i, w) in r.works_best_when.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(out, "\"{}\"", w.as_str());
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"fails_when\":[");
        for (i, f) in r.fails_when.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(out, "\"{}\"", f.as_str());
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"known_confusers\":[");
        for (i, c) in r.known_confusers.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(out, "\"{}\"", c.confuser.as_str());
        }
        let _ = write!(out, "],");
        match &r.required_sampling_law {
            None => {
                let _ = write!(out, "\"required_sampling_law\":null,");
            }
            Some(law) => {
                let _ = write!(
                    out,
                    "\"required_sampling_law\":{{\"kind\":\"{}\",\"min_observations\":{},\"regularity\":\"{}\"}},",
                    law.kind.as_str(),
                    law.min_observations,
                    law.regularity.as_str(),
                );
            }
        }
        match &r.required_units {
            None => {
                let _ = write!(out, "\"required_units\":null,");
            }
            Some(u) => {
                let _ = write!(
                    out,
                    "\"required_units\":{{\"kind\":\"{}\",\"resolution\":\"{}\"}},",
                    u.kind.as_str(),
                    u.min_unit_resolution.as_str(),
                );
            }
        }
        let _ = write!(
            out,
            "\"minimum_support\":{{\"baseline\":{},\"active\":{},\"entities\":{}}},",
            r.minimum_support.min_baseline_observations,
            r.minimum_support.min_active_observations,
            r.minimum_support.min_distinct_entities,
        );
        let _ = write!(out, "\"do_not_use_for\":[");
        for (i, d) in r.do_not_use_for.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(out, "\"{}\"", d.as_str());
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"closest_aliases\":[");
        for (i, a) in r.closest_aliases.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(
                out,
                "{{\"canonical_id\":{},\"reason\":\"{}\"}}",
                a.canonical_id.0,
                a.similarity_reason.as_str(),
            );
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"closest_non_aliases\":[");
        for (i, a) in r.closest_non_aliases.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(
                out,
                "{{\"canonical_id\":{},\"reason\":\"{}\"}}",
                a.canonical_id.0,
                a.distinction_reason.as_str(),
            );
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"adversarial_twins\":[");
        for (i, t) in r.adversarial_twins.iter().enumerate() {
            if i > 0 {
                let _ = write!(out, ",");
            }
            let _ = write!(
                out,
                "{{\"kind\":\"{}\",\"target\":{}}}",
                t.kind_str(),
                t.target().0,
            );
        }
        let _ = write!(out, "]");
        let _ = write!(out, "}}");
    }
    let _ = write!(out, "]}}");
    out
}

/// Render the passport↔contraindication crosswalk as text.
#[must_use]
pub fn render_passport_crosswalk_text(s: &ContraindicationSnapshot) -> String {
    let rows = build_passport_crosswalk(s);
    let mut out = String::with_capacity(4 * 1024);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(
        out,
        "DSFB-GPU-Atlas - Passport-Contraindication Crosswalk (T.11g)"
    );
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(
        out,
        "  id  name                                       confusers do_not closest_alias closest_non twins"
    );
    for r in rows {
        let _ = writeln!(
            out,
            "  {:<3} {:<42} {:>9} {:>6} {:>13} {:>11} {:>5}",
            r.canonical_id.0,
            r.display_name,
            r.known_confuser_count,
            r.do_not_use_for_count,
            r.closest_alias_count,
            r.closest_non_alias_count,
            r.adversarial_twin_count,
        );
    }
    out
}

/// Render the passport↔contraindication crosswalk as JSON.
#[must_use]
pub fn render_passport_crosswalk_json(s: &ContraindicationSnapshot) -> String {
    let rows = build_passport_crosswalk(s);
    let mut out = String::with_capacity(4 * 1024);
    let _ = write!(out, "{{\"rows\":[");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ",");
        }
        let _ = write!(
            out,
            "{{\"canonical_id\":{},\"display_name\":\"{}\",\"known_confusers\":{},\"do_not_use_for\":{},\"closest_aliases\":{},\"closest_non_aliases\":{},\"adversarial_twins\":{}}}",
            r.canonical_id.0,
            json_escape(r.display_name),
            r.known_confuser_count,
            r.do_not_use_for_count,
            r.closest_alias_count,
            r.closest_non_alias_count,
            r.adversarial_twin_count,
        );
    }
    let _ = write!(out, "]}}");
    out
}
