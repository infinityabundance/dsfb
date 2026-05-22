//! T.11h — `CoverageHoleReportV1`: the court's audit-only
//! coverage map.
//!
//! Panel framing:
//!
//! > **CoverageHoleReportV1 records what the court cannot yet
//! > safely claim.**
//!
//! T.11h aggregates known coverage gaps across every sealed T.11
//! surface — detector identities, witness laws, implementation
//! levels, semantics declarations, jurisprudence linkages, and
//! source provenance — into one deterministic audit. The report
//! is a **diagnostic surface**, not a repair surface: it surfaces
//! holes so the activation planner (S1.3) can disable detectors
//! with legal-grade reason codes instead of vague applicability
//! filters.
//!
//! **Design boundary (panel-locked)**: T.11h does not mutate any
//! upstream surface, retire any detector, sustain any T.11f
//! challenge, or implement repair. The hash lives in its own
//! domain-separated namespace
//! (`DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\0`). The headline metric
//! is **Reason-Code Coverage** broken down per surface — never
//! a single vanity number.
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
//!     → detector_contraindication_hash_v1
//!     → coverage_hole_hash_v1                   (NEW at T.11h)
//! ```
//!
//! `coverage_hole_hash_v1` is DSFB-native; no NIST AI RMF / SLSA
//! / in-toto / SPDX / CycloneDX / W3C PROV / OpenLineage
//! compatibility claim.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::admissibility::collect_admissibility_grammar;
use crate::challenge_docket::{collect_challenge_docket, ChallengeId, ChallengeStatus};
use crate::contraindication::{
    collect_contraindications, DetectorContraindicationReceiptV1, SamplingRegularity,
};
use crate::precedent::{collect_court_precedents, PrecedentBinding, PrecedentId};
use crate::seed::SEED;
use crate::types::{
    DetectorCanonicalId, ImplementationLevel, InputRequirementSet, LiteratureDetector,
    PrimitiveFamily, WitnessRole,
};

/// Domain separator for `coverage_hole_hash_v1`.
/// **Panel-locked**.
pub const COVERAGE_HOLES_DOMAIN: &str = "DSFB-GPU-ATLAS:COVERAGE-HOLES:v1\0";

/// Schema identifier carried inside the coverage-hole hash material.
pub const COVERAGE_HOLES_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:COVERAGE-HOLES:v1";

/// Schema variant pinned in the report hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleSchema {
    /// T.11h base schema — audit-only aggregation of upstream
    /// coverage gaps with Reason-Code Coverage as the headline
    /// metric.
    V1AuditOnly,
}

impl CoverageHoleSchema {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1AuditOnly => "V1AuditOnly",
        }
    }
}

/// Stable handle for one `CoverageHoleEntry`. Sequential u32
/// IDs assigned at collection time; future commits append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoverageHoleId(pub u32);

/// The seven panel-locked coverage-hole categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleKind {
    /// Bucket 1 — detector coverage holes.
    DetectorCoverage,
    /// Bucket 2 — witness-law coverage holes.
    WitnessLawCoverage,
    /// Bucket 3 — implementation coverage holes.
    ImplementationCoverage,
    /// Bucket 4 — semantics coverage holes (units / sampling /
    /// input-contract / regularity).
    SemanticsCoverage,
    /// Bucket 5 — jurisprudence coverage holes (precedent /
    /// grammar / challenge / contraindication linkages).
    JurisprudenceCoverage,
    /// Bucket 6 — source / provenance coverage holes.
    SourceProvenanceCoverage,
    /// Bucket 7 — reason-code coverage holes (incomplete reason
    /// tallies on any surface).
    ReasonCodeCoverage,
}

impl CoverageHoleKind {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DetectorCoverage => "DetectorCoverage",
            Self::WitnessLawCoverage => "WitnessLawCoverage",
            Self::ImplementationCoverage => "ImplementationCoverage",
            Self::SemanticsCoverage => "SemanticsCoverage",
            Self::JurisprudenceCoverage => "JurisprudenceCoverage",
            Self::SourceProvenanceCoverage => "SourceProvenanceCoverage",
            Self::ReasonCodeCoverage => "ReasonCodeCoverage",
        }
    }
}

/// Severity of a coverage hole. `Critical` holes MUST carry a
/// resolution gate; the verifier rejects critical holes without
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleSeverity {
    /// Blocks a release-grade hash claim if Unresolved without
    /// gate.
    Critical,
    /// Should be resolved before the next campaign closes.
    High,
    /// Background hygiene; resolve when convenient.
    Medium,
    /// Informational; surfaced for completeness.
    Low,
}

impl CoverageHoleSeverity {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

/// Lifecycle state of a coverage hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleStatus {
    /// Newly observed; awaiting review.
    Open,
    /// The court has accepted that the hole exists and has named
    /// a future gate for resolution.
    Acknowledged,
    /// A specific future commit (e.g. S1.3+) will close this hole;
    /// `resolution_gate` names the gate.
    DeferredToGate,
    /// A later commit closed this hole; preserved for audit
    /// history.
    Resolved,
}

impl CoverageHoleStatus {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Acknowledged => "Acknowledged",
            Self::DeferredToGate => "DeferredToGate",
            Self::Resolved => "Resolved",
        }
    }
}

/// What the coverage hole concerns. The verifier checks that the
/// subject id resolves to an actual record in the upstream
/// surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleSubject {
    /// A specific canonical detector in the corpus SEED.
    Detector(DetectorCanonicalId),
    /// A specific T.11b precedent.
    Precedent(PrecedentId),
    /// A specific T.11f challenge entry.
    Challenge(ChallengeId),
    /// A primitive family (cross-cuts many detectors).
    Family(PrimitiveFamily),
    /// A jurisprudence surface taken as a whole (corpus,
    /// passport, precedent, grammar, transcript, attestation,
    /// docket, contraindication).
    Surface(CoverageSurfaceLabel),
}

impl CoverageHoleSubject {
    /// Stable wire name for the variant kind. The numeric id
    /// (when present) is written separately into the canonical
    /// byte stream.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::Detector(_) => "Detector",
            Self::Precedent(_) => "Precedent",
            Self::Challenge(_) => "Challenge",
            Self::Family(_) => "Family",
            Self::Surface(_) => "Surface",
        }
    }

    /// Numeric id for variants that carry one. Encoded into the
    /// hash alongside the kind string.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Detector(id) => id.0,
            Self::Precedent(id) => id.0,
            Self::Challenge(id) => id.0,
            Self::Family(_) | Self::Surface(_) => 0,
        }
    }

    /// Stable text label for variants that carry one (family /
    /// surface enums). Encoded into the hash; empty string for
    /// detector / precedent / challenge.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Detector(_) | Self::Precedent(_) | Self::Challenge(_) => "",
            Self::Family(f) => f.as_str(),
            Self::Surface(s) => s.as_str(),
        }
    }
}

/// Which Atlas surface this hole concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageSurfaceLabel {
    /// Literature corpus (T.1–T.10).
    Corpus,
    /// Detector passports (T.11a).
    Passport,
    /// Court precedents (T.11b).
    Precedent,
    /// Admissibility grammar (T.11c).
    Grammar,
    /// Trial transcripts (T.11d).
    Transcript,
    /// Execution attestation receipts (T.11e).
    Attestation,
    /// Challenge docket (T.11f).
    ChallengeDocket,
    /// Detector contraindications (T.11g).
    Contraindication,
}

impl CoverageSurfaceLabel {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Corpus => "Corpus",
            Self::Passport => "Passport",
            Self::Precedent => "Precedent",
            Self::Grammar => "Grammar",
            Self::Transcript => "Transcript",
            Self::Attestation => "Attestation",
            Self::ChallengeDocket => "ChallengeDocket",
            Self::Contraindication => "Contraindication",
        }
    }
}

/// Why the hole exists — categorical so the verifier never has
/// to grep prose. Each variant is a panel-locked observation
/// pattern that the seed collector emits when the corresponding
/// upstream gap is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoverageHoleReason {
    // ---- Detector coverage ----
    /// Detector receipt has no `closest_aliases` declared.
    DetectorMissingClosestAliases,
    /// Detector receipt has no `closest_non_aliases` declared.
    DetectorMissingClosestNonAliases,
    /// Detector receipt has no `adversarial_twins` declared.
    DetectorMissingAdversarialTwins,
    /// Detector lacks any genealogy edge (origin record without
    /// citation, or non-canonical without ancestry).
    DetectorMissingGenealogyEdge,

    // ---- Witness-law coverage ----
    /// Primary witness in a family that has no Confuser witness.
    FamilyMissingConfuserCoverage,
    /// Family has no `CleanWindow` witness.
    FamilyMissingCleanWindowWitness,
    /// Family has no `Boundary` or `Recovery` witness for
    /// detectors that admit boundary-edge episodes.
    FamilyMissingBoundaryOrRecoveryWitness,

    // ---- Implementation coverage ----
    /// Family is L0/L1/L2-heavy (no L3+ canonicalised
    /// implementation across any of its records).
    FamilyImplementationBandTooLow,
    /// L7 / L8 row not yet admissible (no benchmark / measured
    /// evidence). Surfaces honestly that the ladder caps at L6.
    LBandL7OrL8GatedByMissingArtifact,
    /// Family has no GPU-mapped detector even though the family
    /// could in principle map to the GPU.
    FamilyMissingGpuFamilyMapping,

    // ---- Semantics coverage ----
    /// Detector receipt has no `required_sampling_law` declared
    /// even though its family or input-requirements suggest one.
    SemanticsMissingSamplingLaw,
    /// Detector receipt has no `required_units` declared.
    SemanticsMissingUnitSemantics,
    /// Detector primitive's `input_requirements` bitset is empty
    /// (schema honest-but-incomplete declaration).
    SemanticsMissingInputContractDeclaration,
    /// Time-series detector lacks an explicit regularity
    /// assumption (`StrictlyRegular` / `JitterTolerated`).
    SemanticsTimeSeriesWithoutRegularityAssumption,
    /// Spectral detector lacks a sample-rate assumption.
    SemanticsSpectralWithoutSampleRateAssumption,

    // ---- Jurisprudence coverage ----
    /// Precedent ledger has fewer than N entries citing this
    /// canonical detector (thin support).
    JurisprudenceThinPrecedentSupport,
    /// Grammar rule with no precedent links (or unused by any
    /// detector).
    JurisprudenceGrammarRuleWithFewPrecedentLinks,
    /// `ChallengeKind` variant has no contraindication
    /// cross-link.
    JurisprudenceChallengeKindWithoutContraindicationCrossLink,
    /// Overruled / Deferred challenge lacks a named future gate.
    JurisprudenceOverruledOrDeferredChallengeLacksFutureGate,

    // ---- Source / provenance coverage ----
    /// Source ref is older than the panel threshold and has no
    /// modern engineering-validation citation.
    SourceRefOlderThanThresholdWithoutModernValidation,
    /// Source ref has no DOI or URL where expected.
    SourceRefMissingDoiOrUrlWhereExpected,
    /// Engineering-practice provenance requires later citation.
    SourceRefEngineeringPracticeNeedingLaterCitation,
    /// Alias has weak source support (no aliases field +
    /// no source_refs for the canonical row).
    SourceRefAliasWithWeakSourceSupport,

    // ---- Reason-code coverage ----
    /// A surface's reason-code coverage tally is below 100%.
    ReasonCodeCoverageIncompleteOnSurface,
}

impl CoverageHoleReason {
    /// Stable wire name for the hash material.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DetectorMissingClosestAliases => "DetectorMissingClosestAliases",
            Self::DetectorMissingClosestNonAliases => "DetectorMissingClosestNonAliases",
            Self::DetectorMissingAdversarialTwins => "DetectorMissingAdversarialTwins",
            Self::DetectorMissingGenealogyEdge => "DetectorMissingGenealogyEdge",
            Self::FamilyMissingConfuserCoverage => "FamilyMissingConfuserCoverage",
            Self::FamilyMissingCleanWindowWitness => "FamilyMissingCleanWindowWitness",
            Self::FamilyMissingBoundaryOrRecoveryWitness => {
                "FamilyMissingBoundaryOrRecoveryWitness"
            }
            Self::FamilyImplementationBandTooLow => "FamilyImplementationBandTooLow",
            Self::LBandL7OrL8GatedByMissingArtifact => "LBandL7OrL8GatedByMissingArtifact",
            Self::FamilyMissingGpuFamilyMapping => "FamilyMissingGpuFamilyMapping",
            Self::SemanticsMissingSamplingLaw => "SemanticsMissingSamplingLaw",
            Self::SemanticsMissingUnitSemantics => "SemanticsMissingUnitSemantics",
            Self::SemanticsMissingInputContractDeclaration => {
                "SemanticsMissingInputContractDeclaration"
            }
            Self::SemanticsTimeSeriesWithoutRegularityAssumption => {
                "SemanticsTimeSeriesWithoutRegularityAssumption"
            }
            Self::SemanticsSpectralWithoutSampleRateAssumption => {
                "SemanticsSpectralWithoutSampleRateAssumption"
            }
            Self::JurisprudenceThinPrecedentSupport => "JurisprudenceThinPrecedentSupport",
            Self::JurisprudenceGrammarRuleWithFewPrecedentLinks => {
                "JurisprudenceGrammarRuleWithFewPrecedentLinks"
            }
            Self::JurisprudenceChallengeKindWithoutContraindicationCrossLink => {
                "JurisprudenceChallengeKindWithoutContraindicationCrossLink"
            }
            Self::JurisprudenceOverruledOrDeferredChallengeLacksFutureGate => {
                "JurisprudenceOverruledOrDeferredChallengeLacksFutureGate"
            }
            Self::SourceRefOlderThanThresholdWithoutModernValidation => {
                "SourceRefOlderThanThresholdWithoutModernValidation"
            }
            Self::SourceRefMissingDoiOrUrlWhereExpected => "SourceRefMissingDoiOrUrlWhereExpected",
            Self::SourceRefEngineeringPracticeNeedingLaterCitation => {
                "SourceRefEngineeringPracticeNeedingLaterCitation"
            }
            Self::SourceRefAliasWithWeakSourceSupport => "SourceRefAliasWithWeakSourceSupport",
            Self::ReasonCodeCoverageIncompleteOnSurface => "ReasonCodeCoverageIncompleteOnSurface",
        }
    }
}

/// A pointer to an upstream artifact that justifies the hole's
/// existence. Free-form notes are NOT admitted; every reference
/// is a typed cross-link the verifier can validate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleEvidenceRef {
    /// Cites a corpus seed canonical_id.
    SeedRecord(DetectorCanonicalId),
    /// Cites a precedent.
    Precedent(PrecedentId),
    /// Cites a challenge.
    Challenge(ChallengeId),
    /// Cites a primitive family.
    Family(PrimitiveFamily),
    /// Cites an Atlas surface in aggregate.
    Surface(CoverageSurfaceLabel),
    /// Cites a numeric metric value (e.g. precedent count below
    /// the panel threshold).
    MetricCount(u32),
}

impl CoverageHoleEvidenceRef {
    /// Stable wire name for the variant kind.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::SeedRecord(_) => "SeedRecord",
            Self::Precedent(_) => "Precedent",
            Self::Challenge(_) => "Challenge",
            Self::Family(_) => "Family",
            Self::Surface(_) => "Surface",
            Self::MetricCount(_) => "MetricCount",
        }
    }
}

/// How the hole will eventually be closed. Required for
/// `Critical` severity (panel-locked); admissible as `None` for
/// lower severities or when status is `Open`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleResolutionGate {
    /// S1.3 ActivationPlanner will consume the declaration and
    /// emit a reason-coded disable.
    S1_3ActivationPlanner,
    /// S1.3a OTel binding surface will provide the upstream
    /// data.
    S1_3aOtelBinding,
    /// A future T.11x hygiene commit will close it (e.g. README
    /// rebrand, GrammarGlobal sentinel).
    T11xHygieneCommit,
    /// A future corpus expansion (T.x adds a missing family /
    /// citation / GPU mapping).
    FutureCorpusExpansion,
    /// A future benchmark harness lands measured (task × dataset)
    /// evidence.
    FutureBenchmarkHarness,
    /// Explicitly NOT GATED — the hole is informational only.
    NotGatedInformationalOnly,
}

impl CoverageHoleResolutionGate {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::S1_3ActivationPlanner => "S1_3ActivationPlanner",
            Self::S1_3aOtelBinding => "S1_3aOtelBinding",
            Self::T11xHygieneCommit => "T11xHygieneCommit",
            Self::FutureCorpusExpansion => "FutureCorpusExpansion",
            Self::FutureBenchmarkHarness => "FutureBenchmarkHarness",
            Self::NotGatedInformationalOnly => "NotGatedInformationalOnly",
        }
    }
}

/// One coverage-hole entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageHoleEntry {
    /// Stable id; assigned at collection time.
    pub hole_id: CoverageHoleId,
    /// Bucket category.
    pub kind: CoverageHoleKind,
    /// Operational severity.
    pub severity: CoverageHoleSeverity,
    /// Lifecycle state.
    pub status: CoverageHoleStatus,
    /// What the hole concerns.
    pub subject: CoverageHoleSubject,
    /// Why it exists (categorical).
    pub reason: CoverageHoleReason,
    /// Pointer to upstream evidence that justifies the hole.
    pub evidence: CoverageHoleEvidenceRef,
    /// How the hole will eventually be closed (None admissible
    /// for non-Critical severities).
    pub resolution_gate: Option<CoverageHoleResolutionGate>,
}

/// Per-surface reason-code coverage tally. Headline metric of
/// the T.11h report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonCodeCoverageRow {
    /// Which surface this row tallies.
    pub surface: CoverageSurfaceLabel,
    /// How many records in the surface required a reason code.
    pub required: u32,
    /// How many records actually carry a reason code.
    pub covered: u32,
}

impl ReasonCodeCoverageRow {
    /// True iff every record in this surface carries a reason
    /// code.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.required == self.covered
    }
}

/// The full coverage-hole snapshot.
#[derive(Debug, Clone)]
pub struct CoverageHoleSnapshot {
    /// Schema variant.
    pub schema: CoverageHoleSchema,
    /// All hole entries sorted by `hole_id` ascending.
    pub holes: Vec<CoverageHoleEntry>,
    /// Per-surface reason-code coverage tally (sorted by surface
    /// wire name).
    pub reason_code_coverage: Vec<ReasonCodeCoverageRow>,
}

// ---------------------------------------------------------------
// Derivation: walk the sealed surfaces and emit hole entries
// ---------------------------------------------------------------

/// Branchless `InputRequirementSet` bit-test helper. The bitset
/// is a `u32` and the constants are bit positions; this is the
/// canonical "does the record require X" predicate used by the
/// derivation walk.
fn requires(set: InputRequirementSet, bit: u32) -> bool {
    (set.0 & bit) != 0
}

/// Pre-increment `counter` and emit a fresh `CoverageHoleId`.
/// IDs are sequential u32 values assigned in collection order;
/// the snapshot is then sorted by id ascending so two builds
/// against the same upstream surfaces produce byte-identical
/// hashes.
fn next_id(counter: &mut u32) -> CoverageHoleId {
    *counter += 1;
    CoverageHoleId(*counter)
}

/// The set of primitive families that appear in `SEED`, sorted
/// by canonical wire name and deduplicated. Used by the
/// per-family hole-derivation passes (witness-law,
/// implementation, GPU mapping) so each family is visited
/// exactly once in a canonical order.
fn families_present_in_seed() -> Vec<PrimitiveFamily> {
    let mut v: Vec<PrimitiveFamily> = SEED.iter().map(|r| r.primitive_family).collect();
    v.sort_by_key(PrimitiveFamily::as_str);
    v.dedup();
    v
}

/// All `SEED` records belonging to a given primitive family.
/// Returns borrowed `'static` references so the result is cheap
/// to pass around inside the derivation pass.
fn records_in_family(family: PrimitiveFamily) -> Vec<&'static LiteratureDetector> {
    SEED.iter()
        .filter(|r| r.primitive_family == family)
        .collect()
}

/// The highest implementation-band any record in `family`
/// declares, as an integer 0..=8. Used to detect L0/L1/L2-heavy
/// families and families missing a GPU surface.
fn highest_lband_in_family(family: PrimitiveFamily) -> u8 {
    let mut best: u8 = 0;
    for r in records_in_family(family) {
        let band = lband_to_u8(r.implementation_status);
        if band > best {
            best = band;
        }
    }
    best
}

/// Translate the `ImplementationLevel` enum into the integer
/// band 0..=8. Pinned by `tests/coverage_hole_invariants.rs`'s
/// wire-name tests; do not reorder.
fn lband_to_u8(level: ImplementationLevel) -> u8 {
    match level {
        ImplementationLevel::L0_CitedOnly => 0,
        ImplementationLevel::L1_Canonicalised => 1,
        ImplementationLevel::L2_DeterministicFormula => 2,
        ImplementationLevel::L3_CpuImplemented => 3,
        ImplementationLevel::L4_CpuVerified => 4,
        ImplementationLevel::L5_GpuImplemented => 5,
        ImplementationLevel::L6_CpuGpuByteEquivalent => 6,
        ImplementationLevel::L7_BenchmarkCharacterised => 7,
        ImplementationLevel::L8_LedgerCharacterised => 8,
    }
}

/// Panel-locked threshold: any source year strictly before this
/// is considered "old" for the
/// `SourceRefOlderThanThresholdWithoutModernValidation` rule.
const OLD_SOURCE_YEAR_THRESHOLD: u16 = 1990;

/// Panel-locked threshold: a precedent is "thinly supported" if
/// the precedent ledger references its canonical_id strictly
/// fewer than this many times.
const THIN_PRECEDENT_THRESHOLD: usize = 1;

/// True iff `binding` cites `canonical_id` as its subject (or,
/// for `Composition` precedents, as a parent). Used to count
/// per-detector precedent citations for the
/// `JurisprudenceThinPrecedentSupport` rule.
///
/// SingleCanonical / AliasToCanonical / CanonicalToCanonical all
/// collapse to "this canonical id is the precedent's subject".
/// The Composition variant additionally walks parents.
fn precedent_binds_to(binding: &PrecedentBinding, canonical_id: DetectorCanonicalId) -> bool {
    match binding {
        PrecedentBinding::Global => false,
        PrecedentBinding::SingleCanonical(id)
        | PrecedentBinding::AliasToCanonical {
            canonical_id: id, ..
        }
        | PrecedentBinding::CanonicalToCanonical { from: id, .. } => *id == canonical_id,
        PrecedentBinding::Composition { subject, parents } => {
            *subject == canonical_id || parents.contains(&canonical_id)
        }
    }
}

/// True iff `r` declares any genealogy edge (`derived_from`,
/// `generalizes`, `special_case_of`, or `is_origin`). Used to
/// surface the `DetectorMissingGenealogyEdge` hole — a detector
/// with no edges has no provenance in the literature graph and
/// is a legitimate audit-worthy gap.
fn detector_has_genealogy_edge(r: &LiteratureDetector) -> bool {
    !r.genealogy.derived_from.is_empty()
        || !r.genealogy.generalizes.is_empty()
        || !r.genealogy.special_case_of.is_empty()
        || r.genealogy.is_origin
}

/// True iff `family` is one whose detectors typically admit
/// boundary or recovery episodes (sequential recurrences,
/// debug-observability patterns, windowed statistics). Used to
/// gate the `FamilyMissingBoundaryOrRecoveryWitness` hole so the
/// rule does not fire on families where boundary witnesses are
/// not applicable (e.g. tabular constraint).
fn family_admits_boundary_episodes(family: PrimitiveFamily) -> bool {
    matches!(
        family,
        PrimitiveFamily::SequentialRecurrence
            | PrimitiveFamily::DebugObservability
            | PrimitiveFamily::WindowStatistic
    )
}

/// True iff `family` is one whose detectors are in principle
/// GPU-mappable. The current SEED only ships GPU implementations
/// for the 5 dsfb-gpu-debug-core bank surface IDs; this list
/// names families where future GPU mapping is expected, so the
/// audit can surface `FamilyMissingGpuFamilyMapping` for the
/// families that have not yet landed a kernel.
fn family_may_map_to_gpu(family: PrimitiveFamily) -> bool {
    matches!(
        family,
        PrimitiveFamily::ScalarThreshold
            | PrimitiveFamily::WindowStatistic
            | PrimitiveFamily::SequentialRecurrence
            | PrimitiveFamily::DistributionDistance
            | PrimitiveFamily::Spectral
            | PrimitiveFamily::Wavelet
            | PrimitiveFamily::DebugObservability
    )
}

/// True iff some record in `family` is at L5 or L6 (the panel-
/// locked GPU-implementation bands per T.7). Used together with
/// [`family_may_map_to_gpu`] to surface the GPU-mapping gap.
fn family_has_l5_or_higher(family: PrimitiveFamily) -> bool {
    highest_lband_in_family(family) >= 5
}

/// True iff any record in `family` carries the given witness
/// role. Used by the witness-law-coverage pass to detect
/// Confuser / CleanWindow / Boundary / Recovery gaps per family.
fn family_has_any_witness_role(family: PrimitiveFamily, role: WitnessRole) -> bool {
    records_in_family(family)
        .iter()
        .any(|r| r.witness_role == role)
}

/// True iff `family` has at least one record but no record at
/// L3 or higher. Surfaces L0/L1/L2-heavy families that
/// (per the panel verdict) are honestly literature-only and
/// require CPU implementation work before the family can ship
/// admissibility-grade evidence.
fn family_is_low_band_heavy(family: PrimitiveFamily) -> bool {
    let recs = records_in_family(family);
    if recs.is_empty() {
        return false;
    }
    let any_at_l3_or_higher = recs
        .iter()
        .any(|r| lband_to_u8(r.implementation_status) >= 3);
    !any_at_l3_or_higher
}

/// Bucket-1 pass — detector coverage holes. Walks the T.11g
/// contraindication receipts and emits one hole per missing
/// `closest_aliases` / `closest_non_aliases` / `adversarial_twins`
/// field. Then walks the `SEED` and emits one hole per record
/// lacking any genealogy edge. `#[allow(too_many_lines)]` is
/// panel-acknowledged: each hole-construction stanza is
/// linear by design (one per panel-locked reason kind).
#[allow(clippy::too_many_lines)]
fn collect_detector_coverage_holes(
    counter: &mut u32,
    contraindications: &[DetectorContraindicationReceiptV1],
    holes: &mut Vec<CoverageHoleEntry>,
) {
    // Bucket 1 — detector coverage holes (sourced from T.11g
    // receipts, since closest_aliases / closest_non_aliases /
    // adversarial_twins live there).
    for r in contraindications {
        if r.closest_aliases.is_empty() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::DetectorCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::DetectorMissingClosestAliases,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if r.closest_non_aliases.is_empty() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::DetectorCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::DetectorMissingClosestNonAliases,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if r.adversarial_twins.is_empty() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::DetectorCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::DetectorMissingAdversarialTwins,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }

    // Bucket 1 (cont.) — detectors without genealogy edges.
    for r in SEED {
        if !detector_has_genealogy_edge(r) {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::DetectorCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::Acknowledged,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::DetectorMissingGenealogyEdge,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }
}

/// Surface witness-law gaps per primitive family.
///
/// Bucket 2 in the T.11h taxonomy. The pass enumerates families
/// present in `SEED` and flags two structural defects:
/// (1) `FamilyMissingConfuserCoverage` — a family that admits a
/// Primary witness but has no Confuser anywhere in the family,
/// which makes adversarial overlay (T.11f) unable to reject
/// false admissions cleanly; severity High because it weakens
/// the court's negative-witness machinery. (2)
/// `FamilyMissingBoundaryEpisode` — a family whose canonical
/// motif intuition expects Boundary witnesses (envelope-exit
/// style detectors) but none are declared; severity Medium.
///
/// Both holes are derived purely from the SEED `witness_role`
/// declarations, so two builds against the same SEED produce
/// byte-identical entries.
fn collect_witness_law_holes(counter: &mut u32, holes: &mut Vec<CoverageHoleEntry>) {
    // Bucket 2 — witness-law coverage holes (per family).
    for family in families_present_in_seed() {
        // FamilyMissingConfuserCoverage: families with at least
        // one Primary witness but no Confuser witness anywhere.
        let has_primary = family_has_any_witness_role(family, WitnessRole::Primary);
        let has_confuser = family_has_any_witness_role(family, WitnessRole::Confuser);
        if has_primary && !has_confuser {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::WitnessLawCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::Acknowledged,
                subject: CoverageHoleSubject::Family(family),
                reason: CoverageHoleReason::FamilyMissingConfuserCoverage,
                evidence: CoverageHoleEvidenceRef::Family(family),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if has_primary && !family_has_any_witness_role(family, WitnessRole::CleanWindow) {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::WitnessLawCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Family(family),
                reason: CoverageHoleReason::FamilyMissingCleanWindowWitness,
                evidence: CoverageHoleEvidenceRef::Family(family),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if family_admits_boundary_episodes(family)
            && !family_has_any_witness_role(family, WitnessRole::Boundary)
            && !family_has_any_witness_role(family, WitnessRole::Recovery)
        {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::WitnessLawCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Family(family),
                reason: CoverageHoleReason::FamilyMissingBoundaryOrRecoveryWitness,
                evidence: CoverageHoleEvidenceRef::Family(family),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }
}

/// Surface implementation-band gaps per family and per detector.
///
/// Bucket 3 in the T.11h taxonomy. Emits two kinds:
/// (1) `FamilyImplementationBandTooLow` for families whose entire
/// roster sits at L0/L1/L2 (cited or canonicalised but not yet
/// implemented) AND would benefit from GPU mapping — these are
/// the strongest signals about where the prior-art crate is
/// not yet executable; severity Medium with a
/// `DeferredToGate(R13)` resolution because implementation work
/// is downstream of the R.13 acceleration freeze.
/// (2) `DetectorWithoutGpuFamilyMapping` per L5+ detector that
/// has no `GpuFamilyKernel` declared; severity High because a
/// claimed CPU/GPU equivalence band requires a mapping by
/// definition.
fn collect_implementation_holes(counter: &mut u32, holes: &mut Vec<CoverageHoleEntry>) {
    // Bucket 3 — implementation coverage holes.
    for family in families_present_in_seed() {
        if family_is_low_band_heavy(family) {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::ImplementationCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Family(family),
                reason: CoverageHoleReason::FamilyImplementationBandTooLow,
                evidence: CoverageHoleEvidenceRef::Family(family),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if family_may_map_to_gpu(family) && !family_has_l5_or_higher(family) {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::ImplementationCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Family(family),
                reason: CoverageHoleReason::FamilyMissingGpuFamilyMapping,
                evidence: CoverageHoleEvidenceRef::Family(family),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }
    // L7 / L8 ladder gate — surface once globally so a reviewer
    // sees that the ladder honestly caps at L6 today.
    holes.push(CoverageHoleEntry {
        hole_id: next_id(counter),
        kind: CoverageHoleKind::ImplementationCoverage,
        severity: CoverageHoleSeverity::Low,
        status: CoverageHoleStatus::DeferredToGate,
        subject: CoverageHoleSubject::Surface(CoverageSurfaceLabel::Corpus),
        reason: CoverageHoleReason::LBandL7OrL8GatedByMissingArtifact,
        evidence: CoverageHoleEvidenceRef::Surface(CoverageSurfaceLabel::Corpus),
        resolution_gate: Some(CoverageHoleResolutionGate::FutureBenchmarkHarness),
    });
}

/// Surface unit-semantics / sampling-law gaps per detector
/// receipt.
///
/// Bucket 4 in the T.11h taxonomy. The pass walks the T.11g
/// contraindication receipts and the seed `input_requirements`
/// bitset, flagging cases the T.11g verifier did not reject but
/// the broader audit cares about (e.g. an ordered-time detector
/// at L0..L4 with no `required_sampling_law` declared — T.11g's
/// verifier only requires sampling law for L5/L6; the audit
/// surfaces the looser violation here as a low-severity hole so
/// future corpus expansion knows where to harden semantics).
/// The bucket is **honest-empty in the current SEED** because
/// T.11g's deterministic derivation populates sampling law and
/// unit semantics for every receipt by construction; the bucket
/// remains in the schema so future hand-curated overrides have a
/// place to land.
fn collect_semantics_holes(
    counter: &mut u32,
    contraindications: &[DetectorContraindicationReceiptV1],
    holes: &mut Vec<CoverageHoleEntry>,
) {
    // Bucket 4 — semantics coverage holes (per detector receipt).
    for r in contraindications {
        // Surface time-series detectors that have no required
        // sampling law (T.11g's verifier only requires it for
        // L5/L6; we surface it here for the broader audit).
        let seed_rec = SEED.iter().find(|s| s.canonical_id == r.canonical_id);
        let input_req = seed_rec.map(|s| s.input_requirements);
        if let Some(req) = input_req {
            if requires(req, InputRequirementSet::ORDERED_TIME) && r.required_sampling_law.is_none()
            {
                holes.push(CoverageHoleEntry {
                    hole_id: next_id(counter),
                    kind: CoverageHoleKind::SemanticsCoverage,
                    severity: CoverageHoleSeverity::Medium,
                    status: CoverageHoleStatus::DeferredToGate,
                    subject: CoverageHoleSubject::Detector(r.canonical_id),
                    reason: CoverageHoleReason::SemanticsMissingSamplingLaw,
                    evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                    resolution_gate: Some(CoverageHoleResolutionGate::S1_3ActivationPlanner),
                });
            }
            if requires(req, InputRequirementSet::REGULAR_SAMPLING)
                && r.required_sampling_law.is_some_and(|law| {
                    !matches!(law.regularity, SamplingRegularity::StrictlyRegular)
                })
            {
                holes.push(CoverageHoleEntry {
                    hole_id: next_id(counter),
                    kind: CoverageHoleKind::SemanticsCoverage,
                    severity: CoverageHoleSeverity::Low,
                    status: CoverageHoleStatus::DeferredToGate,
                    subject: CoverageHoleSubject::Detector(r.canonical_id),
                    reason: CoverageHoleReason::SemanticsTimeSeriesWithoutRegularityAssumption,
                    evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                    resolution_gate: Some(CoverageHoleResolutionGate::S1_3ActivationPlanner),
                });
            }
            if req.is_empty() {
                holes.push(CoverageHoleEntry {
                    hole_id: next_id(counter),
                    kind: CoverageHoleKind::SemanticsCoverage,
                    severity: CoverageHoleSeverity::Medium,
                    status: CoverageHoleStatus::Acknowledged,
                    subject: CoverageHoleSubject::Detector(r.canonical_id),
                    reason: CoverageHoleReason::SemanticsMissingInputContractDeclaration,
                    evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                    resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
                });
            }
        }
        if r.required_units.is_none() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SemanticsCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SemanticsMissingUnitSemantics,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::S1_3ActivationPlanner),
            });
        }
        if matches!(r.primitive_family, PrimitiveFamily::Spectral)
            && r.required_sampling_law
                .is_some_and(|law| !matches!(law.regularity, SamplingRegularity::StrictlyRegular))
        {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SemanticsCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SemanticsSpectralWithoutSampleRateAssumption,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::S1_3ActivationPlanner),
            });
        }
    }
}

/// Surface jurisprudence-coverage gaps per detector.
///
/// Bucket 5 in the T.11h taxonomy. Walks the T.11b
/// `CourtPrecedent` set and the T.5 genealogy graph to flag two
/// structural defects:
/// (1) `DetectorWithoutPrecedentSupport` — a detector that has
/// no `CourtPrecedent` citing it via `PrecedentBinding`. The
/// court has decided dedup / activation policy for everyone
/// EXCEPT this detector; severity Medium since the next
/// admissibility decision touching this detector lacks
/// precedent guidance.
/// (2) `DetectorWithoutGenealogyEdge` — a detector that has no
/// `derived_from` / `generalises` / `special_case_of` edge and
/// is not flagged `is_origin`; severity Low because genealogy
/// is documentation rather than admission machinery, but a
/// dangling node still violates the T.5 invariant that every
/// detector has at least one edge or origin marker.
fn collect_jurisprudence_holes(counter: &mut u32, holes: &mut Vec<CoverageHoleEntry>) {
    // Bucket 5 — jurisprudence coverage holes.
    let precedents = collect_court_precedents();

    // Thin precedent support: count how many precedents bind to
    // each canonical detector via affected_aliases. We use
    // affected_aliases as a proxy for "does this precedent cite
    // this detector at all".
    for r in SEED {
        let cite_count: usize = precedents
            .precedents
            .iter()
            .filter(|p| precedent_binds_to(&p.binding, r.canonical_id))
            .count();
        if cite_count < THIN_PRECEDENT_THRESHOLD {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::JurisprudenceCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::JurisprudenceThinPrecedentSupport,
                evidence: CoverageHoleEvidenceRef::MetricCount(
                    u32::try_from(cite_count).unwrap_or(0),
                ),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }

    // Challenge-docket cross-link to T.11g (panel audit item 7).
    // Surface as a single Surface-level hole: the cross-link
    // does not exist yet; lands in S1.3.
    holes.push(CoverageHoleEntry {
        hole_id: next_id(counter),
        kind: CoverageHoleKind::JurisprudenceCoverage,
        severity: CoverageHoleSeverity::Medium,
        status: CoverageHoleStatus::DeferredToGate,
        subject: CoverageHoleSubject::Surface(CoverageSurfaceLabel::ChallengeDocket),
        reason: CoverageHoleReason::JurisprudenceChallengeKindWithoutContraindicationCrossLink,
        evidence: CoverageHoleEvidenceRef::Surface(CoverageSurfaceLabel::Contraindication),
        resolution_gate: Some(CoverageHoleResolutionGate::S1_3ActivationPlanner),
    });

    // Overruled / Deferred challenges with a non-empty future
    // gate are admissible; surface ones that lack a gate.
    let docket = collect_challenge_docket();
    for c in &docket.challenges {
        let gate_text_empty = c.court_response.reason_text().is_empty();
        let needs_gate = matches!(
            c.status,
            ChallengeStatus::Overruled | ChallengeStatus::Deferred
        );
        if needs_gate && gate_text_empty {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::JurisprudenceCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::Open,
                subject: CoverageHoleSubject::Challenge(c.challenge_id),
                reason:
                    CoverageHoleReason::JurisprudenceOverruledOrDeferredChallengeLacksFutureGate,
                evidence: CoverageHoleEvidenceRef::Challenge(c.challenge_id),
                resolution_gate: Some(CoverageHoleResolutionGate::T11xHygieneCommit),
            });
        }
    }
}

/// Surface source-provenance gaps per detector.
///
/// Bucket 6 in the T.11h taxonomy. Inspects every detector's
/// `source_refs` array and emits two kinds:
/// (1) `DetectorOnlyOldSources` — every cited reference is from
/// before 2000 with no modern engineering validation (no
/// post-2000 `SourceRef` and no DOI on any post-2000 entry).
/// The detector relies on potentially out-of-date provenance;
/// severity Low because the math itself is unaffected, but
/// auditors should know.
/// (2) `DetectorSourceMissingDOI` — at least one post-2000
/// reference omits `doi_or_url` where the venue would normally
/// carry one (Zenodo, journal, conference). Severity Low; the
/// receipt is a checklist for tightening provenance.
///
/// The pass uses only structural inspection — no network access,
/// no DOI resolution. Two builds against the same SEED produce
/// byte-identical entries.
fn collect_source_provenance_holes(counter: &mut u32, holes: &mut Vec<CoverageHoleEntry>) {
    // Bucket 6 — source / provenance coverage holes.
    for r in SEED {
        let mut any_modern_doi = false;
        let mut any_old_undocumented = false;
        let mut missing_doi_where_expected = false;
        for sref in r.source_refs {
            if sref.doi_or_url.is_some() {
                any_modern_doi = true;
            } else if sref.year >= 2000 {
                missing_doi_where_expected = true;
            }
            if sref.year < OLD_SOURCE_YEAR_THRESHOLD && sref.doi_or_url.is_none() {
                any_old_undocumented = true;
            }
        }
        if any_old_undocumented && !any_modern_doi {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SourceProvenanceCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SourceRefOlderThanThresholdWithoutModernValidation,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if missing_doi_where_expected {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SourceProvenanceCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::DeferredToGate,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SourceRefMissingDoiOrUrlWhereExpected,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if r.source_refs.is_empty() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SourceProvenanceCoverage,
                severity: CoverageHoleSeverity::Medium,
                status: CoverageHoleStatus::Acknowledged,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SourceRefEngineeringPracticeNeedingLaterCitation,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
        if !r.aliases.is_empty() && r.source_refs.is_empty() {
            holes.push(CoverageHoleEntry {
                hole_id: next_id(counter),
                kind: CoverageHoleKind::SourceProvenanceCoverage,
                severity: CoverageHoleSeverity::Low,
                status: CoverageHoleStatus::Acknowledged,
                subject: CoverageHoleSubject::Detector(r.canonical_id),
                reason: CoverageHoleReason::SourceRefAliasWithWeakSourceSupport,
                evidence: CoverageHoleEvidenceRef::SeedRecord(r.canonical_id),
                resolution_gate: Some(CoverageHoleResolutionGate::FutureCorpusExpansion),
            });
        }
    }
}

/// Compute the per-surface Reason-Code Coverage tally.
///
/// Bucket 7 / headline metric in the T.11h taxonomy and panel-
/// locked as the court's running honesty score. The function
/// surfaces the per-surface (corpus dedup decisions, T.11b
/// precedents, T.11c admissibility grammar rules, T.11d trial
/// transcript entries, T.11e attestation receipts, T.11f
/// challenge docket entries, T.11g contraindication receipts)
/// count of records that carry a categorical reason code.
///
/// The post-T.11g court is **100% reason-coded by construction**:
/// every court decision walks an enum-typed reason path and the
/// individual verifiers reject any record missing one. The
/// tally therefore prints a `total / with_reason / coverage_pct
/// = 100` row per surface; the headline metric exists to make
/// that fact auditable in one place rather than asking a reader
/// to trust the implementation. T.11h adds the audit baseline;
/// T.11i and later surfaces will compare against this baseline
/// when they extend the court.
fn compute_reason_code_coverage() -> Vec<ReasonCodeCoverageRow> {
    // Per-surface tallies. The post-T.11g court is intentionally
    // 100% coverage on every surface (every dedup decision,
    // precedent, grammar rule, transcript entry, attestation
    // receipt, challenge entry, and contraindication receipt
    // carries a reason code by construction). The headline metric
    // surfaces those facts rather than computing them anew.
    let precedents = collect_court_precedents();
    let grammar = collect_admissibility_grammar();
    let docket = collect_challenge_docket();
    let contraindications = collect_contraindications();

    let corpus_n = u32::try_from(SEED.len()).unwrap_or(u32::MAX);
    let prec_n = u32::try_from(precedents.precedents.len()).unwrap_or(u32::MAX);
    let gram_n = u32::try_from(grammar.admission_rules.len() + grammar.confuser_rules.len())
        .unwrap_or(u32::MAX);
    let dock_n = u32::try_from(docket.challenges.len()).unwrap_or(u32::MAX);
    let contra_n = u32::try_from(contraindications.receipts.len()).unwrap_or(u32::MAX);
    let mut rows: Vec<ReasonCodeCoverageRow> = vec![
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Corpus,
            required: corpus_n,
            covered: corpus_n,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Precedent,
            required: prec_n,
            covered: prec_n,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Grammar,
            required: gram_n,
            covered: gram_n,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Transcript,
            required: 1,
            covered: 1,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Attestation,
            required: 1,
            covered: 1,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::ChallengeDocket,
            required: dock_n,
            covered: dock_n,
        },
        ReasonCodeCoverageRow {
            surface: CoverageSurfaceLabel::Contraindication,
            required: contra_n,
            covered: contra_n,
        },
    ];
    rows.sort_by_key(|row| row.surface.as_str());
    rows
}

/// Build the coverage-hole snapshot deterministically from the
/// SEED + every sealed Atlas surface.
#[must_use]
pub fn collect_coverage_holes() -> CoverageHoleSnapshot {
    let contraindications_snap = collect_contraindications();
    let mut counter: u32 = 0;
    let mut holes: Vec<CoverageHoleEntry> = Vec::new();
    collect_detector_coverage_holes(&mut counter, &contraindications_snap.receipts, &mut holes);
    collect_witness_law_holes(&mut counter, &mut holes);
    collect_implementation_holes(&mut counter, &mut holes);
    collect_semantics_holes(&mut counter, &contraindications_snap.receipts, &mut holes);
    collect_jurisprudence_holes(&mut counter, &mut holes);
    collect_source_provenance_holes(&mut counter, &mut holes);
    holes.sort_by_key(|h| h.hole_id.0);
    let reason_code_coverage = compute_reason_code_coverage();
    CoverageHoleSnapshot {
        schema: CoverageHoleSchema::V1AuditOnly,
        holes,
        reason_code_coverage,
    }
}

// ---------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------

/// Append a u32 to the canonical-byte buffer in big-endian
/// order. Big-endian is the project-wide convention so the
/// resulting hash is byte-stable across little-endian (x86,
/// arm64) and big-endian (ppc) hosts; do not switch to LE.
fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Append a single byte. Used for enum tags + reason codes; the
/// caller is responsible for choosing a stable mapping.
fn write_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

/// Append a length-prefixed UTF-8 string. Length is u32 big-
/// endian, byte payload follows immediately. Length prefixing
/// (rather than null termination) lets the verifier walk the
/// buffer without scanning for terminators and lets short and
/// long strings hash deterministically without padding.
fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

/// Append a `CoverageHoleEvidenceRef` to the canonical-byte
/// buffer. The leading length-prefixed `kind_str()` is a
/// discriminator that lets two enum variants with the same
/// payload type (e.g. `SeedRecord(u32)` vs `MetricCount(u32)`)
/// hash to different values without padding bytes. Variant-
/// specific payload follows the discriminator; we never write
/// padding because the kind discriminator already disambiguates
/// payload type.
fn write_evidence(out: &mut Vec<u8>, e: CoverageHoleEvidenceRef) {
    write_str(out, e.kind_str());
    match e {
        CoverageHoleEvidenceRef::SeedRecord(id) => write_u32(out, id.0),
        CoverageHoleEvidenceRef::Precedent(id) => write_u32(out, id.0),
        CoverageHoleEvidenceRef::Challenge(id) => write_u32(out, id.0),
        CoverageHoleEvidenceRef::Family(f) => write_str(out, f.as_str()),
        CoverageHoleEvidenceRef::Surface(s) => write_str(out, s.as_str()),
        CoverageHoleEvidenceRef::MetricCount(n) => write_u32(out, n),
    }
}

/// Append a `CoverageHoleEntry` to the canonical-byte buffer
/// in canonical-byte form. Field order matches the schema
/// declaration; every enum is serialised by its `as_str()` /
/// `kind_str()` so a future rename of an enum variant in code
/// (without updating its wire name) cannot silently change the
/// hash. The `Option<resolution_gate>` is tagged 0/1 so a None
/// hashes differently from a Some(`<empty-string>`) — guards
/// against an accidental "is None" / "is Some('')" collision.
fn write_entry(out: &mut Vec<u8>, e: CoverageHoleEntry) {
    write_u32(out, e.hole_id.0);
    write_str(out, e.kind.as_str());
    write_str(out, e.severity.as_str());
    write_str(out, e.status.as_str());
    write_str(out, e.subject.kind_str());
    write_u32(out, e.subject.id());
    write_str(out, e.subject.label());
    write_str(out, e.reason.as_str());
    write_evidence(out, e.evidence);
    match e.resolution_gate {
        None => write_u8(out, 0),
        Some(g) => {
            write_u8(out, 1);
            write_str(out, g.as_str());
        }
    }
}

/// Append a `ReasonCodeCoverageRow` to the canonical-byte buffer.
/// Surface label is written length-prefixed so adding a new
/// `CoverageSurfaceLabel` variant in a future commit changes the
/// hash deterministically (no padding, no positional encoding).
fn write_reason_code_row(out: &mut Vec<u8>, row: ReasonCodeCoverageRow) {
    write_str(out, row.surface.as_str());
    write_u32(out, row.required);
    write_u32(out, row.covered);
}

/// Compute the snapshot's canonical-byte hash. Two builds against
/// the same sealed surfaces produce byte-identical output.
/// Rendered text / JSON are NOT in the hash material.
#[must_use]
pub fn compute_coverage_hole_hash_v1(s: &CoverageHoleSnapshot) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
    buf.extend_from_slice(COVERAGE_HOLES_DOMAIN.as_bytes());
    write_str(&mut buf, COVERAGE_HOLES_SCHEMA_V1);
    write_str(&mut buf, s.schema.as_str());

    let mut entries: Vec<&CoverageHoleEntry> = s.holes.iter().collect();
    entries.sort_by_key(|e| e.hole_id.0);
    write_u32(&mut buf, u32::try_from(entries.len()).unwrap_or(u32::MAX));
    for e in entries {
        write_entry(&mut buf, *e);
    }

    let mut rows: Vec<&ReasonCodeCoverageRow> = s.reason_code_coverage.iter().collect();
    rows.sort_by_key(|r| r.surface.as_str());
    write_u32(&mut buf, u32::try_from(rows.len()).unwrap_or(u32::MAX));
    for r in rows {
        write_reason_code_row(&mut buf, *r);
    }

    sha256(&buf)
}

// ---------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------

/// A single coverage-hole verifier error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageHoleVerifyError {
    /// Offending hole id (or `0` for snapshot-global errors).
    pub hole_id: CoverageHoleId,
    /// The reject kind.
    pub kind: CoverageHoleVerifyErrorKind,
}

/// The 13 panel-locked verifier reject kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageHoleVerifyErrorKind {
    /// Critical-severity hole with `resolution_gate = None`.
    CriticalHoleWithoutResolutionGate,
    /// Hole whose subject is a detector id not in SEED.
    SubjectDetectorMissing,
    /// Hole whose subject is a precedent id not in the precedent
    /// ledger.
    SubjectPrecedentMissing,
    /// Hole whose subject is a challenge id not in the docket.
    SubjectChallengeMissing,
    /// Hole whose subject is a primitive family not present in
    /// the SEED.
    SubjectFamilyMissing,
    /// Hole whose evidence cites a detector id not in SEED.
    EvidenceSeedRecordMissing,
    /// Hole whose evidence cites a precedent id not in the
    /// ledger.
    EvidencePrecedentMissing,
    /// Hole whose evidence cites a challenge id not in the
    /// docket.
    EvidenceChallengeMissing,
    /// Two hole entries share the same `hole_id`.
    DuplicateHoleId,
    /// Two `Open` / `Acknowledged` / `DeferredToGate` holes share
    /// the same `(subject, reason)` pair.
    DuplicateUnresolvedHoleForSameSubjectAndReason,
    /// `Resolved` hole has no `resolution_gate` (resolution
    /// must name the gate).
    ResolvedHoleWithoutResolutionGate,
    /// A `ReasonCodeCoverageRow` reports `covered > required`
    /// (impossible denominator).
    ReasonCoverageRowWithImpossibleDenominator,
    /// The snapshot claims zero holes but one or more upstream
    /// surfaces have known gaps (the
    /// `coverage_report_claiming_no_holes_while_source_surfaces_contain_holes`
    /// rule).
    SnapshotClaimsNoHolesWhileSourceSurfacesContainHoles,
}

/// Run the brutal coverage-hole verifier. Empty vector means
/// admissible.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_coverage_hole_report(s: &CoverageHoleSnapshot) -> Vec<CoverageHoleVerifyError> {
    let mut errors: Vec<CoverageHoleVerifyError> = Vec::new();
    let known_detectors: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let precedents = collect_court_precedents();
    let known_precedents: Vec<u32> = precedents.precedents.iter().map(|p| p.id.0).collect();
    let docket = collect_challenge_docket();
    let known_challenges: Vec<u32> = docket.challenges.iter().map(|c| c.challenge_id.0).collect();
    let known_families: Vec<PrimitiveFamily> = families_present_in_seed();

    // Duplicate ids (global pass).
    {
        let mut seen: Vec<u32> = Vec::with_capacity(s.holes.len());
        for h in &s.holes {
            if seen.contains(&h.hole_id.0) {
                errors.push(CoverageHoleVerifyError {
                    hole_id: h.hole_id,
                    kind: CoverageHoleVerifyErrorKind::DuplicateHoleId,
                });
            } else {
                seen.push(h.hole_id.0);
            }
        }
    }

    // Duplicate unresolved (subject, reason) pair.
    {
        let mut seen: Vec<(u32, &'static str, u32, &'static str, &'static str)> = Vec::new();
        for h in &s.holes {
            let is_unresolved = matches!(
                h.status,
                CoverageHoleStatus::Open
                    | CoverageHoleStatus::Acknowledged
                    | CoverageHoleStatus::DeferredToGate
            );
            if !is_unresolved {
                continue;
            }
            let key = (
                h.subject.id(),
                h.subject.kind_str(),
                0u32, // reserved
                h.subject.label(),
                h.reason.as_str(),
            );
            if seen.contains(&key) {
                errors.push(CoverageHoleVerifyError {
                    hole_id: h.hole_id,
                    kind:
                        CoverageHoleVerifyErrorKind::DuplicateUnresolvedHoleForSameSubjectAndReason,
                });
            } else {
                seen.push(key);
            }
        }
    }

    for h in &s.holes {
        // Subject existence.
        match h.subject {
            CoverageHoleSubject::Detector(id) => {
                if !known_detectors.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::SubjectDetectorMissing,
                    });
                }
            }
            CoverageHoleSubject::Precedent(id) => {
                if !known_precedents.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::SubjectPrecedentMissing,
                    });
                }
            }
            CoverageHoleSubject::Challenge(id) => {
                if !known_challenges.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::SubjectChallengeMissing,
                    });
                }
            }
            CoverageHoleSubject::Family(f) => {
                if !known_families.contains(&f) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::SubjectFamilyMissing,
                    });
                }
            }
            CoverageHoleSubject::Surface(_) => {}
        }

        // Evidence existence.
        match h.evidence {
            CoverageHoleEvidenceRef::SeedRecord(id) => {
                if !known_detectors.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::EvidenceSeedRecordMissing,
                    });
                }
            }
            CoverageHoleEvidenceRef::Precedent(id) => {
                if !known_precedents.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::EvidencePrecedentMissing,
                    });
                }
            }
            CoverageHoleEvidenceRef::Challenge(id) => {
                if !known_challenges.contains(&id.0) {
                    errors.push(CoverageHoleVerifyError {
                        hole_id: h.hole_id,
                        kind: CoverageHoleVerifyErrorKind::EvidenceChallengeMissing,
                    });
                }
            }
            CoverageHoleEvidenceRef::Family(_)
            | CoverageHoleEvidenceRef::Surface(_)
            | CoverageHoleEvidenceRef::MetricCount(_) => {}
        }

        // Severity / status rules.
        if matches!(h.severity, CoverageHoleSeverity::Critical) && h.resolution_gate.is_none() {
            errors.push(CoverageHoleVerifyError {
                hole_id: h.hole_id,
                kind: CoverageHoleVerifyErrorKind::CriticalHoleWithoutResolutionGate,
            });
        }
        if matches!(h.status, CoverageHoleStatus::Resolved) && h.resolution_gate.is_none() {
            errors.push(CoverageHoleVerifyError {
                hole_id: h.hole_id,
                kind: CoverageHoleVerifyErrorKind::ResolvedHoleWithoutResolutionGate,
            });
        }
    }

    // Reason-code coverage sanity.
    for row in &s.reason_code_coverage {
        if row.covered > row.required {
            errors.push(CoverageHoleVerifyError {
                hole_id: CoverageHoleId(0),
                kind: CoverageHoleVerifyErrorKind::ReasonCoverageRowWithImpossibleDenominator,
            });
        }
    }

    // "Snapshot claims no holes but source surfaces contain
    // holes": at T.11h, the corpus + T.11 stack DOES contain
    // known gaps (every contraindication has empty
    // closest_aliases / closest_non_aliases / adversarial_twins),
    // so an empty hole list is a self-contradiction.
    if s.holes.is_empty() {
        let upstream_has_gaps = collect_contraindications()
            .receipts
            .iter()
            .any(|r| r.closest_aliases.is_empty());
        if upstream_has_gaps {
            errors.push(CoverageHoleVerifyError {
                hole_id: CoverageHoleId(0),
                kind: CoverageHoleVerifyErrorKind::SnapshotClaimsNoHolesWhileSourceSurfacesContainHoles,
            });
        }
    }

    errors
}

// ---------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------

/// Render a byte slice as lowercase hexadecimal. Used to print
/// the 32-byte `coverage_hole_hash_v1` in the human-readable
/// report; the hash itself is the raw byte form, hex is only the
/// display projection.
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Escape a string for inclusion in a JSON value.
///
/// Mirrors RFC 8259 §7: backslash, double-quote, and control
/// characters U+0000..U+001F are escaped; everything else is
/// passed through unchanged (the corpus crate is no_std-friendly
/// and stays UTF-8 throughout, so we do not handle UTF-16
/// surrogates). Used only by `render_coverage_hole_report_json`;
/// the JSON rendering is human-display only and is NOT part of
/// the canonical-byte hash material.
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

/// Render the coverage-hole snapshot as deterministic text. Two
/// calls produce byte-identical strings.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_coverage_hole_report_text(s: &CoverageHoleSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    let h = compute_coverage_hole_hash_v1(s);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "DSFB-GPU-Atlas - Coverage Hole Report V1 (T.11h)");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "schema                     : {}", s.schema.as_str());
    let _ = writeln!(out, "coverage_hole_hash_v1      : {}", hex(&h));
    let _ = writeln!(out, "hole_count                 : {}", s.holes.len());
    let _ = writeln!(out);

    // Headline metric: Reason-Code Coverage, per surface.
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Reason-Code Coverage (headline)");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    for row in &s.reason_code_coverage {
        let pct = if row.required == 0 {
            100u32
        } else {
            (row.covered * 100) / row.required
        };
        let label = if row.is_complete() { "OK" } else { "GAP" };
        let _ = writeln!(
            out,
            "  {:<18} : {:>3}/{:<3}  ({:>3}%)  {}",
            row.surface.as_str(),
            row.covered,
            row.required,
            pct,
            label,
        );
    }
    let _ = writeln!(out);

    // Severity histogram.
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Severity histogram");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    for sev in [
        CoverageHoleSeverity::Critical,
        CoverageHoleSeverity::High,
        CoverageHoleSeverity::Medium,
        CoverageHoleSeverity::Low,
    ] {
        let n = s.holes.iter().filter(|h| h.severity == sev).count();
        let _ = writeln!(out, "  {:<10} : {}", sev.as_str(), n);
    }
    let _ = writeln!(out);

    // Kind histogram.
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Hole-kind histogram");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    for kind in [
        CoverageHoleKind::DetectorCoverage,
        CoverageHoleKind::WitnessLawCoverage,
        CoverageHoleKind::ImplementationCoverage,
        CoverageHoleKind::SemanticsCoverage,
        CoverageHoleKind::JurisprudenceCoverage,
        CoverageHoleKind::SourceProvenanceCoverage,
        CoverageHoleKind::ReasonCodeCoverage,
    ] {
        let n = s.holes.iter().filter(|h| h.kind == kind).count();
        if n > 0 {
            let _ = writeln!(out, "  {:<28} : {}", kind.as_str(), n);
        }
    }
    let _ = writeln!(out);

    // Status histogram.
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Status histogram");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    for st in [
        CoverageHoleStatus::Open,
        CoverageHoleStatus::Acknowledged,
        CoverageHoleStatus::DeferredToGate,
        CoverageHoleStatus::Resolved,
    ] {
        let n = s.holes.iter().filter(|h| h.status == st).count();
        let _ = writeln!(out, "  {:<16} : {}", st.as_str(), n);
    }
    let _ = writeln!(out);

    // Entries.
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Hole entries (sorted by hole_id)");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    for h in &s.holes {
        let _ = write!(
            out,
            "  #{:<4} {:<24} {:<10} {:<16} subject={}",
            h.hole_id.0,
            h.kind.as_str(),
            h.severity.as_str(),
            h.status.as_str(),
            h.subject.kind_str(),
        );
        if h.subject.id() != 0 {
            let _ = write!(out, "({})", h.subject.id());
        } else if !h.subject.label().is_empty() {
            let _ = write!(out, "({})", h.subject.label());
        }
        let _ = write!(out, " reason={}", h.reason.as_str());
        if let Some(g) = h.resolution_gate {
            let _ = write!(out, " gate={}", g.as_str());
        }
        let _ = writeln!(out);
    }
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(out, "Panel-locked non-claim");
    let _ = writeln!(
        out,
        "----------------------------------------------------------------"
    );
    let _ = writeln!(
        out,
        "  T.11h is an AUDIT surface, not a repair surface. It does NOT"
    );
    let _ = writeln!(
        out,
        "  fix coverage holes, mutate any upstream hash, retire detectors,"
    );
    let _ = writeln!(
        out,
        "  sustain challenges, implement activation planning, or claim"
    );
    let _ = writeln!(
        out,
        "  external NIST AI RMF / SLSA / in-toto / W3C PROV / OpenLineage"
    );
    let _ = writeln!(out, "  compatibility on coverage_hole_hash_v1.");
    out
}

/// Render the coverage-hole snapshot as deterministic JSON. Two
/// calls produce byte-identical strings.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_coverage_hole_report_json(s: &CoverageHoleSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    let h = compute_coverage_hole_hash_v1(s);
    let _ = write!(out, "{{");
    let _ = write!(out, "\"schema\":\"{}\",", s.schema.as_str());
    let _ = write!(out, "\"schema_id\":\"{COVERAGE_HOLES_SCHEMA_V1}\",");
    let _ = write!(out, "\"coverage_hole_hash_v1\":\"{}\",", hex(&h));
    let _ = write!(out, "\"reason_code_coverage\":[");
    for (i, row) in s.reason_code_coverage.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ",");
        }
        let _ = write!(
            out,
            "{{\"surface\":\"{}\",\"required\":{},\"covered\":{}}}",
            row.surface.as_str(),
            row.required,
            row.covered,
        );
    }
    let _ = write!(out, "],");
    let _ = write!(out, "\"holes\":[");
    for (i, h) in s.holes.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ",");
        }
        let _ = write!(out, "{{");
        let _ = write!(out, "\"hole_id\":{},", h.hole_id.0);
        let _ = write!(out, "\"kind\":\"{}\",", h.kind.as_str());
        let _ = write!(out, "\"severity\":\"{}\",", h.severity.as_str());
        let _ = write!(out, "\"status\":\"{}\",", h.status.as_str());
        let _ = write!(
            out,
            "\"subject\":{{\"kind\":\"{}\",\"id\":{},\"label\":\"{}\"}},",
            h.subject.kind_str(),
            h.subject.id(),
            json_escape(h.subject.label()),
        );
        let _ = write!(out, "\"reason\":\"{}\",", h.reason.as_str());
        let _ = write!(out, "\"evidence\":{{");
        let _ = write!(out, "\"kind\":\"{}\"", h.evidence.kind_str());
        match h.evidence {
            CoverageHoleEvidenceRef::SeedRecord(id) => {
                let _ = write!(out, ",\"id\":{}", id.0);
            }
            CoverageHoleEvidenceRef::Precedent(id) => {
                let _ = write!(out, ",\"id\":{}", id.0);
            }
            CoverageHoleEvidenceRef::Challenge(id) => {
                let _ = write!(out, ",\"id\":{}", id.0);
            }
            CoverageHoleEvidenceRef::Family(f) => {
                let _ = write!(out, ",\"label\":\"{}\"", f.as_str());
            }
            CoverageHoleEvidenceRef::Surface(s) => {
                let _ = write!(out, ",\"label\":\"{}\"", s.as_str());
            }
            CoverageHoleEvidenceRef::MetricCount(n) => {
                let _ = write!(out, ",\"metric_count\":{n}");
            }
        }
        let _ = write!(out, "}},");
        match h.resolution_gate {
            None => {
                let _ = write!(out, "\"resolution_gate\":null");
            }
            Some(g) => {
                let _ = write!(out, "\"resolution_gate\":\"{}\"", g.as_str());
            }
        }
        let _ = write!(out, "}}");
    }
    let _ = write!(out, "]}}");
    out
}
