//! T.11b — `CourtPrecedent`: deterministic jurisprudence over
//! dedup, witness-law, L-band, usefulness, and corpus-binding
//! rules.
//!
//! The panel framing:
//!
//! > The Atlas court is not merely a set of current
//! > classifications; it carries deterministic precedents
//! > explaining why each alias, composition, witness role,
//! > implementation claim, and usefulness claim is admitted,
//! > rejected, or deferred.
//!
//! T.11b is **derived jurisprudence**: every precedent is
//! projected from an already-frozen surface (T.4 court records,
//! T.6 fusion / witness law, T.7 L-band law, T.8 usefulness law,
//! T.10 corpus hash freeze, S1.2 registry binding, T.11a passport
//! coverage). The collector invents no new judgments; it only
//! turns existing rules into a canonical receipt.
//!
//! **Hash posture (panel-locked)**:
//!
//! - `corpus_hash_v1` stays frozen.
//! - `precedent_hash_v1` is an additional court-layer receipt
//!   computed by [`compute_precedent_hash_v1`]. It does NOT
//!   alter `corpus_hash_v1`; it is a separate cumulative hash.
//!
//! **Panel-locked non-claims (T.11b)**:
//!
//! - Does NOT emit CaseFileV2 episode-transcript bodies (T.11d).
//! - Does NOT implement UnitSemantics / SamplingLaw receipts
//!   (T.11e).
//! - Does NOT implement external provenance export (DSFB-PROV /
//!   OpenLineage / NIST AI RMF / RO-Crate).
//! - Does NOT change `corpus_hash_v1`, `registry_hash_v2`, or
//!   any D16 / D64 / D128 / D205 GPU behavior.
//! - The precedent hash is DSFB-native; no in-toto / SLSA / SPDX
//!   / CycloneDX compatibility claim.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::claims::CLAIMS;
use crate::court::classify_all;
use crate::seed::SEED;
use crate::types::{
    CanonicalisationDecision, DedupReason, DedupSubject, DetectorAliasId, DetectorCanonicalId,
};

/// Domain separator prefix for `precedent_hash_v1`.
/// **Panel-locked**; changing it changes every precedent hash.
pub const COURT_PRECEDENT_DOMAIN: &str = "DSFB-GPU-ATLAS:COURT-PRECEDENT:v1\0";

/// Schema identifier carried inside the precedent hash material.
pub const COURT_PRECEDENT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:COURT-PRECEDENT:v1";

/// Stable handle for one [`CourtPrecedent`].
///
/// IDs are assigned **after** the canonical sort over the
/// (kind, binding, reason) tuple, so two builds against the same
/// corpus produce the same id for each precedent. Adding a new
/// kind / binding combination in a future commit may shift later
/// ids; this is intentional — the (kind, binding, reason) tuple
/// is the stable key, not the numeric id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrecedentId(pub u32);

/// Kind of court precedent. Panel-locked at 13 variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrecedentKind {
    /// A canonical SEED record is judged Canonical (the
    /// representative of its equivalence class).
    DedupCanonical,
    /// An alias claim collapses into a canonical record.
    AliasCollapse,
    /// A canonical record is judged a composition over named
    /// canonical parents (e.g. Western Electric = Shewhart +
    /// rule-set composition).
    CompositionJudgment,
    /// A canonical record is judged a parameterisation of another
    /// canonical (same formula, different parameters).
    ParameterizationJudgment,
    /// A canonical record is admitted as the deterministic-seed
    /// reduction of a stochastic literature original (semantic
    /// role is preserved; the stochastic original is NOT
    /// canonical here).
    SemanticRoleSeparation,
    /// Global witness-law (T.6 fusion-layer law that every
    /// admitted witness must obey).
    WitnessLaw,
    /// Global negative-witness law (T.6 confuser / boundary law).
    NegativeWitnessLaw,
    /// Global L-band honesty law (T.7 implementation-status
    /// ladder rule).
    LBandHonestyLaw,
    /// Global usefulness-ledger honesty law (T.8 evidence-level
    /// rule).
    UsefulnessHonestyLaw,
    /// Global corpus-hash law (T.10 freeze rule).
    CorpusHashLaw,
    /// Global registry-binding law (S1.2 `HashFrozenT10` +
    /// `source_corpus_hash` cross-field rule).
    RegistryBindingLaw,
    /// Global constitution-coverage law (T.11a passport coverage
    /// + ConstitutionFlags requirement).
    ConstitutionLaw,
    /// Deferred-gate law (a panel-locked future commitment that
    /// is documented but not yet enforced; severity is `Deferred`).
    DeferredGateLaw,
}

impl PrecedentKind {
    /// Canonical wire name for hashing + rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DedupCanonical => "DedupCanonical",
            Self::AliasCollapse => "AliasCollapse",
            Self::CompositionJudgment => "CompositionJudgment",
            Self::ParameterizationJudgment => "ParameterizationJudgment",
            Self::SemanticRoleSeparation => "SemanticRoleSeparation",
            Self::WitnessLaw => "WitnessLaw",
            Self::NegativeWitnessLaw => "NegativeWitnessLaw",
            Self::LBandHonestyLaw => "LBandHonestyLaw",
            Self::UsefulnessHonestyLaw => "UsefulnessHonestyLaw",
            Self::CorpusHashLaw => "CorpusHashLaw",
            Self::RegistryBindingLaw => "RegistryBindingLaw",
            Self::ConstitutionLaw => "ConstitutionLaw",
            Self::DeferredGateLaw => "DeferredGateLaw",
        }
    }
}

/// Which corpus layer the precedent was projected from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrecedentSource {
    /// T.4 dedup court (CanonicalisationDecision + DedupReason).
    T4DedupCourt,
    /// T.5 genealogy graph edges.
    T5Genealogy,
    /// T.6 witness-role and fusion-plane compatibility law.
    T6FusionLaw,
    /// T.7 implementation-status ladder verifier rules.
    T7LBandLaw,
    /// T.8 usefulness-ledger verifier rules.
    T8UsefulnessLaw,
    /// T.9 internal-audit deferred gate (a non-claim recorded as
    /// a precedent for forward compatibility).
    T9DeferredGate,
    /// T.10 corpus-hash freeze rule.
    T10CorpusHashFreeze,
    /// S1.2 registry-binding cross-field rule.
    S12RegistryBinding,
    /// T.11a passport coverage law.
    T11aPassportCoverage,
}

impl PrecedentSource {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::T4DedupCourt => "T4DedupCourt",
            Self::T5Genealogy => "T5Genealogy",
            Self::T6FusionLaw => "T6FusionLaw",
            Self::T7LBandLaw => "T7LBandLaw",
            Self::T8UsefulnessLaw => "T8UsefulnessLaw",
            Self::T9DeferredGate => "T9DeferredGate",
            Self::T10CorpusHashFreeze => "T10CorpusHashFreeze",
            Self::S12RegistryBinding => "S12RegistryBinding",
            Self::T11aPassportCoverage => "T11aPassportCoverage",
        }
    }
}

/// What the precedent binds to.
///
/// The five variants cover every projection T.11b needs:
///
/// - `Global` — cross-cutting law that applies to every record
///   (e.g. "Primary witness cannot be negative-only").
/// - `SingleCanonical` — a single SEED canonical record (e.g. a
///   `DedupCanonical` precedent on canonical_id 1).
/// - `Composition` — a canonical record judged as a composition
///   of parents (binding lists subject first, then parents).
/// - `AliasToCanonical` — an alias claim (alias-id space 1000+)
///   collapsing into a SEED canonical.
/// - `CanonicalToCanonical` — a canonical record judged as a
///   parameterisation / stochastic-reduction of another canonical.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrecedentBinding {
    /// Cross-cutting law with no specific subject.
    Global,
    /// A single SEED canonical record.
    SingleCanonical(DetectorCanonicalId),
    /// A subject canonical + its parent canonicals (first entry
    /// in the Vec is the subject).
    Composition {
        /// The composing record's canonical id.
        subject: DetectorCanonicalId,
        /// The parent canonical ids the subject composes over.
        parents: Vec<DetectorCanonicalId>,
    },
    /// An alias-side claim collapsing into a canonical.
    AliasToCanonical {
        /// Alias-side id (1000+ space).
        alias_id: DetectorAliasId,
        /// Canonical target id.
        canonical_id: DetectorCanonicalId,
    },
    /// A canonical record bound to another canonical (e.g.
    /// parameterisation, stochastic-reduction).
    CanonicalToCanonical {
        /// Subject canonical id.
        from: DetectorCanonicalId,
        /// Target canonical id.
        to: DetectorCanonicalId,
    },
}

/// Reason the precedent was admitted. Includes every T.4
/// `DedupReason` plus the panel-locked extra reasons that cover
/// T.6 / T.7 / T.8 / T.10 / S1.2 / T.11a laws and T.9 deferred
/// gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrecedentReason {
    // T.4 / T.3 dedup reasons (mirror DedupReason)
    /// Mirrors `DedupReason::SameFormulaSameParametersSameContract`.
    SameFormulaSameParametersSameContract,
    /// Mirrors `DedupReason::SameFormulaDifferentParameters`.
    SameFormulaDifferentParameters,
    /// Mirrors `DedupReason::DifferentFormulaSameDomain`.
    DifferentFormulaSameDomain,
    /// Mirrors `DedupReason::SameFormulaDifferentInputContract`.
    SameFormulaDifferentInputContract,
    /// Mirrors `DedupReason::SameFormulaDifferentWitnessRole`.
    SameFormulaDifferentWitnessRole,
    /// Mirrors `DedupReason::DifferentDecisionFunctional`.
    DifferentDecisionFunctional,
    /// Mirrors `DedupReason::DeterministicReductionOfStochastic`.
    DeterministicReductionOfStochastic,
    /// Mirrors `DedupReason::CompositionOfCanonicals`.
    CompositionOfCanonicals,
    /// Mirrors `DedupReason::OriginRecord`.
    OriginRecord,

    // T.6 witness laws
    /// A primary witness MUST NOT be a negative-witness-only
    /// detector. The fusion layer rejects negative witnesses
    /// from emitting affirmative admissions.
    PrimaryWitnessCannotBeNegativeOnly,
    /// A clean-window witness MUST NOT admit an episode by
    /// itself. It corroborates but never carries admission.
    CleanWindowWitnessCannotAdmitAlone,
    /// A negative witness firing vetoes any admission the bank
    /// would otherwise grant on the same evidence triple.
    NegativeWitnessVetoesAdmission,

    // T.7 L-band laws
    /// L5 or L6 status requires the record's canonical_id to
    /// appear in `lband::GPU_IMPLEMENTED_CANONICAL_IDS`.
    L5L6RequiresGpuWhitelist,
    /// L7 is forbidden until a pinned benchmark artifact exists
    /// for the record's (canonical_id, dataset_id) triple.
    L7ForbiddenUntilBenchmarkArtifact,
    /// L8 is forbidden until a measured ledger row admits the
    /// record at the required usefulness evidence level.
    L8ForbiddenUntilMeasuredLedger,

    // T.8 usefulness laws
    /// A `NotScored` usefulness row MUST carry zero empirical
    /// fields. The verifier rejects fabricated numbers on
    /// unmeasured rows.
    NotScoredRequiresZeroEmpiricals,
    /// A `Retired*` lifecycle state requires measured negative
    /// evidence at `SyntheticFixtureMeasured` or higher.
    RetiredRequiresMeasuredNegativeEvidence,

    // T.10 / S1.2 cross-field laws
    /// `corpus_hash_v1` is frozen post-T.10; any byte change in
    /// SEED, CLAIMS, court decisions, genealogy, witness-role,
    /// L-band, or usefulness ledger changes the hash.
    CorpusHashV1IsFrozen,
    /// `registry_hash_v2` binds to the frozen `corpus_hash_v1`
    /// via every spec's `source_corpus_hash` field.
    RegistryHashV2BindsToFrozenCorpusHash,
    /// A spec with `corpus_binding_status = HashFrozenT10` MUST
    /// carry `source_corpus_hash != [0; 32]`; the reverse holds
    /// too.
    HashFrozenT10RequiresNonZeroSourceCorpusHash,

    // T.11a passport coverage
    /// Every SEED canonical record produces exactly one passport
    /// via `all_passports()`. The count cannot be inflated by
    /// alias claims.
    EveryCanonicalRequiresPassport,
    /// Every passport's `constitution_flags` must declare all
    /// eight flags `true`; the corpus verifier rejects otherwise.
    EveryPassportRequiresAllConstitutionFlags,

    // T.9 deferred gates (Soft / Deferred severity)
    /// CaseFileV2 episode-transcript body is deferred to T.11d.
    CaseFileV2BodyDeferredToT11d,
    /// UnitSemanticsReceipt is deferred to T.11e.
    UnitSemanticsDeferredToT11e,
    /// SamplingLawReceipt is deferred to T.11e.
    SamplingLawDeferredToT11e,
    /// External provenance export (DSFB-PROV / OpenLineage /
    /// NIST AI RMF / RO-Crate) is deferred to post-S1.3.
    ExternalProvenanceExportDeferredPostS13,
}

impl PrecedentReason {
    /// Canonical wire name. Hand-pinned so a future variant
    /// rename cannot silently shift `precedent_hash_v1`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameFormulaSameParametersSameContract => "SameFormulaSameParametersSameContract",
            Self::SameFormulaDifferentParameters => "SameFormulaDifferentParameters",
            Self::DifferentFormulaSameDomain => "DifferentFormulaSameDomain",
            Self::SameFormulaDifferentInputContract => "SameFormulaDifferentInputContract",
            Self::SameFormulaDifferentWitnessRole => "SameFormulaDifferentWitnessRole",
            Self::DifferentDecisionFunctional => "DifferentDecisionFunctional",
            Self::DeterministicReductionOfStochastic => "DeterministicReductionOfStochastic",
            Self::CompositionOfCanonicals => "CompositionOfCanonicals",
            Self::OriginRecord => "OriginRecord",
            Self::PrimaryWitnessCannotBeNegativeOnly => "PrimaryWitnessCannotBeNegativeOnly",
            Self::CleanWindowWitnessCannotAdmitAlone => "CleanWindowWitnessCannotAdmitAlone",
            Self::NegativeWitnessVetoesAdmission => "NegativeWitnessVetoesAdmission",
            Self::L5L6RequiresGpuWhitelist => "L5L6RequiresGpuWhitelist",
            Self::L7ForbiddenUntilBenchmarkArtifact => "L7ForbiddenUntilBenchmarkArtifact",
            Self::L8ForbiddenUntilMeasuredLedger => "L8ForbiddenUntilMeasuredLedger",
            Self::NotScoredRequiresZeroEmpiricals => "NotScoredRequiresZeroEmpiricals",
            Self::RetiredRequiresMeasuredNegativeEvidence => {
                "RetiredRequiresMeasuredNegativeEvidence"
            }
            Self::CorpusHashV1IsFrozen => "CorpusHashV1IsFrozen",
            Self::RegistryHashV2BindsToFrozenCorpusHash => "RegistryHashV2BindsToFrozenCorpusHash",
            Self::HashFrozenT10RequiresNonZeroSourceCorpusHash => {
                "HashFrozenT10RequiresNonZeroSourceCorpusHash"
            }
            Self::EveryCanonicalRequiresPassport => "EveryCanonicalRequiresPassport",
            Self::EveryPassportRequiresAllConstitutionFlags => {
                "EveryPassportRequiresAllConstitutionFlags"
            }
            Self::CaseFileV2BodyDeferredToT11d => "CaseFileV2BodyDeferredToT11d",
            Self::UnitSemanticsDeferredToT11e => "UnitSemanticsDeferredToT11e",
            Self::SamplingLawDeferredToT11e => "SamplingLawDeferredToT11e",
            Self::ExternalProvenanceExportDeferredPostS13 => {
                "ExternalProvenanceExportDeferredPostS13"
            }
        }
    }

    /// Map a T.4 `DedupReason` to the matching `PrecedentReason`.
    /// The two enums share the dedup-side names verbatim.
    #[must_use]
    pub const fn from_dedup_reason(r: DedupReason) -> Self {
        match r {
            DedupReason::SameFormulaSameParametersSameContract => {
                Self::SameFormulaSameParametersSameContract
            }
            DedupReason::SameFormulaDifferentParameters => Self::SameFormulaDifferentParameters,
            DedupReason::DifferentFormulaSameDomain => Self::DifferentFormulaSameDomain,
            DedupReason::SameFormulaDifferentInputContract => {
                Self::SameFormulaDifferentInputContract
            }
            DedupReason::SameFormulaDifferentWitnessRole => Self::SameFormulaDifferentWitnessRole,
            DedupReason::DifferentDecisionFunctional => Self::DifferentDecisionFunctional,
            DedupReason::DeterministicReductionOfStochastic => {
                Self::DeterministicReductionOfStochastic
            }
            DedupReason::CompositionOfCanonicals => Self::CompositionOfCanonicals,
            DedupReason::OriginRecord => Self::OriginRecord,
        }
    }
}

/// Precedent severity. Hard precedents reject on violation; Soft
/// precedents are documented expectations; Deferred precedents
/// are panel-locked future commitments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrecedentSeverity {
    /// The verifier rejects on violation.
    Hard,
    /// Documented expectation; auditor surface only.
    Soft,
    /// Panel-locked future commitment; not yet enforced.
    Deferred,
}

impl PrecedentSeverity {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hard => "Hard",
            Self::Soft => "Soft",
            Self::Deferred => "Deferred",
        }
    }
}

/// One court precedent record.
#[derive(Debug, Clone)]
pub struct CourtPrecedent {
    /// Stable handle within this `PrecedentSet`.
    pub id: PrecedentId,
    /// What kind of judgment this records.
    pub kind: PrecedentKind,
    /// Which corpus layer projected this precedent.
    pub source: PrecedentSource,
    /// What the precedent binds to.
    pub binding: PrecedentBinding,
    /// Why the precedent applies.
    pub reason: PrecedentReason,
    /// How strictly the precedent is enforced.
    pub severity: PrecedentSeverity,
    /// Free-form note for human readers (the public report
    /// shows this verbatim).
    pub notes: &'static str,
}

/// The full collected precedent ledger.
#[derive(Debug, Clone)]
pub struct PrecedentSet {
    /// Precedents in canonical order (sorted by
    /// `(kind, binding_subject, binding_target, reason)`).
    /// IDs are assigned after the sort so two builds produce
    /// the same `PrecedentId` for each precedent.
    pub precedents: Vec<CourtPrecedent>,
    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of every precedent. Two builds against the
    /// same corpus produce the same hash.
    pub precedent_hash_v1: [u8; 32],
}

/// Collect every court precedent from the live corpus surfaces
/// (T.4 / T.6 / T.7 / T.8 / T.10 / S1.2 / T.11a / T.9 deferred
/// gates). Deterministic across two builds.
#[must_use]
pub fn collect_court_precedents() -> PrecedentSet {
    let mut out: Vec<CourtPrecedent> = Vec::with_capacity(128);

    project_t4_dedup(&mut out);
    project_t6_witness_law(&mut out);
    project_t7_lband_law(&mut out);
    project_t8_usefulness_law(&mut out);
    project_t10_corpus_hash_law(&mut out);
    project_s12_registry_binding_law(&mut out);
    project_t11a_passport_coverage_law(&mut out);
    project_t9_deferred_gates(&mut out);

    // Canonical sort. The (kind, binding subject, binding
    // target, reason wire-name) tuple is the stable key.
    out.sort_by(|a, b| canonical_sort_key(a).cmp(&canonical_sort_key(b)));

    // Assign deterministic ids AFTER sort so two builds produce
    // the same id for each precedent. Ids start at 1.
    for (i, p) in out.iter_mut().enumerate() {
        p.id = PrecedentId(u32::try_from(i + 1).unwrap_or(u32::MAX));
    }

    let precedent_hash_v1 = compute_precedent_hash_v1_raw(&out);
    PrecedentSet {
        precedents: out,
        precedent_hash_v1,
    }
}

fn canonical_sort_key(p: &CourtPrecedent) -> (&'static str, u32, u32, &'static str) {
    let (subject, target) = binding_subject_target(&p.binding);
    (p.kind.as_str(), subject, target, p.reason.as_str())
}

fn binding_subject_target(b: &PrecedentBinding) -> (u32, u32) {
    match b {
        PrecedentBinding::Global => (0, 0),
        PrecedentBinding::SingleCanonical(id) => (id.0, 0),
        PrecedentBinding::Composition { subject, parents } => {
            let smallest_parent = parents.iter().map(|p| p.0).min().unwrap_or(0);
            (subject.0, smallest_parent)
        }
        PrecedentBinding::AliasToCanonical {
            alias_id,
            canonical_id,
        } => (alias_id.0, canonical_id.0),
        PrecedentBinding::CanonicalToCanonical { from, to } => (from.0, to.0),
    }
}

fn project_t4_dedup(out: &mut Vec<CourtPrecedent>) {
    // T.4 court records over SEED canonicals and CLAIMS aliases.
    for rec in classify_all() {
        match rec.subject {
            DedupSubject::Canonical(subject) => {
                let (kind, binding) = match rec.decision {
                    CanonicalisationDecision::Canonical => (
                        PrecedentKind::DedupCanonical,
                        PrecedentBinding::SingleCanonical(subject),
                    ),
                    CanonicalisationDecision::AliasOf(target) => (
                        // A canonical record claiming AliasOf is
                        // unusual; bind as CanonicalToCanonical
                        // to preserve the audit trail.
                        PrecedentKind::AliasCollapse,
                        PrecedentBinding::CanonicalToCanonical {
                            from: subject,
                            to: target,
                        },
                    ),
                    CanonicalisationDecision::ParameterisationOf(target) => (
                        PrecedentKind::ParameterizationJudgment,
                        PrecedentBinding::CanonicalToCanonical {
                            from: subject,
                            to: target,
                        },
                    ),
                    CanonicalisationDecision::CompositionOf(parents) => (
                        PrecedentKind::CompositionJudgment,
                        PrecedentBinding::Composition {
                            subject,
                            parents: parents.to_vec(),
                        },
                    ),
                    CanonicalisationDecision::StochasticOriginalDeterministicReduction(target) => (
                        PrecedentKind::SemanticRoleSeparation,
                        PrecedentBinding::CanonicalToCanonical {
                            from: subject,
                            to: target,
                        },
                    ),
                    CanonicalisationDecision::RejectedNotDeterministic
                    | CanonicalisationDecision::RejectedNotDetector
                    | CanonicalisationDecision::DeferredNeedsReview => {
                        // These do not (yet) live in the SEED;
                        // the T.4 court emits them only for
                        // alias claims. Skip canonical-side
                        // rejections.
                        continue;
                    }
                };
                out.push(CourtPrecedent {
                    id: PrecedentId(0), // overwritten after sort
                    kind,
                    source: PrecedentSource::T4DedupCourt,
                    binding,
                    reason: PrecedentReason::from_dedup_reason(rec.reason_code),
                    severity: PrecedentSeverity::Hard,
                    notes: rec.notes,
                });
            }
            DedupSubject::AliasClaim(alias_id) => {
                let (kind, binding) = match rec.decision {
                    CanonicalisationDecision::AliasOf(target) => (
                        PrecedentKind::AliasCollapse,
                        PrecedentBinding::AliasToCanonical {
                            alias_id,
                            canonical_id: target,
                        },
                    ),
                    CanonicalisationDecision::ParameterisationOf(target) => (
                        PrecedentKind::ParameterizationJudgment,
                        PrecedentBinding::AliasToCanonical {
                            alias_id,
                            canonical_id: target,
                        },
                    ),
                    CanonicalisationDecision::StochasticOriginalDeterministicReduction(target) => (
                        PrecedentKind::SemanticRoleSeparation,
                        PrecedentBinding::AliasToCanonical {
                            alias_id,
                            canonical_id: target,
                        },
                    ),
                    CanonicalisationDecision::Canonical
                    | CanonicalisationDecision::CompositionOf(_)
                    | CanonicalisationDecision::RejectedNotDeterministic
                    | CanonicalisationDecision::RejectedNotDetector
                    | CanonicalisationDecision::DeferredNeedsReview => continue,
                };
                let _ = alias_id; // hush unused-binding lint
                out.push(CourtPrecedent {
                    id: PrecedentId(0),
                    kind,
                    source: PrecedentSource::T4DedupCourt,
                    binding,
                    reason: PrecedentReason::from_dedup_reason(rec.reason_code),
                    severity: PrecedentSeverity::Hard,
                    notes: rec.notes,
                });
            }
        }
    }
    // Suppress the unused-import warning when CLAIMS is only
    // referenced transitively. The classify_all() call exercises
    // the CLAIMS static; we additionally take a length here to
    // make the link explicit.
    let _ = CLAIMS.len();
}

fn project_t6_witness_law(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::PrimaryWitnessCannotBeNegativeOnly,
            "A primary witness MUST NOT be a negative-witness-only detector. The T.6 fusion layer rejects negative witnesses from emitting affirmative admissions.",
        ),
        (
            PrecedentReason::CleanWindowWitnessCannotAdmitAlone,
            "A clean-window witness MUST NOT admit an episode by itself; it corroborates but never carries admission.",
        ),
        (
            PrecedentReason::NegativeWitnessVetoesAdmission,
            "A negative witness firing vetoes any admission the bank would otherwise grant on the same evidence triple.",
        ),
    ];
    for (reason, notes) in laws {
        let kind = if matches!(reason, PrecedentReason::NegativeWitnessVetoesAdmission) {
            PrecedentKind::NegativeWitnessLaw
        } else {
            PrecedentKind::WitnessLaw
        };
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind,
            source: PrecedentSource::T6FusionLaw,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Hard,
            notes,
        });
    }
}

fn project_t7_lband_law(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::L5L6RequiresGpuWhitelist,
            "L5 or L6 status requires the record's canonical_id to appear in lband::GPU_IMPLEMENTED_CANONICAL_IDS.",
        ),
        (
            PrecedentReason::L7ForbiddenUntilBenchmarkArtifact,
            "L7_BenchmarkCharacterized is forbidden until a pinned benchmark artifact exists for the (canonical_id, dataset_id) triple.",
        ),
        (
            PrecedentReason::L8ForbiddenUntilMeasuredLedger,
            "L8_LedgerCharacterized is forbidden until a measured usefulness-ledger row admits the record at the required evidence level.",
        ),
    ];
    for (reason, notes) in laws {
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind: PrecedentKind::LBandHonestyLaw,
            source: PrecedentSource::T7LBandLaw,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Hard,
            notes,
        });
    }
}

fn project_t8_usefulness_law(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::NotScoredRequiresZeroEmpiricals,
            "A NotScored usefulness row MUST carry zero empirical fields. The T.8 verifier rejects fabricated numbers on unmeasured rows.",
        ),
        (
            PrecedentReason::RetiredRequiresMeasuredNegativeEvidence,
            "A Retired* lifecycle state requires measured negative evidence at SyntheticFixtureMeasured or higher.",
        ),
    ];
    for (reason, notes) in laws {
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind: PrecedentKind::UsefulnessHonestyLaw,
            source: PrecedentSource::T8UsefulnessLaw,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Hard,
            notes,
        });
    }
}

fn project_t10_corpus_hash_law(out: &mut Vec<CourtPrecedent>) {
    out.push(CourtPrecedent {
        id: PrecedentId(0),
        kind: PrecedentKind::CorpusHashLaw,
        source: PrecedentSource::T10CorpusHashFreeze,
        binding: PrecedentBinding::Global,
        reason: PrecedentReason::CorpusHashV1IsFrozen,
        severity: PrecedentSeverity::Hard,
        notes: "corpus_hash_v1 is frozen post-T.10. Any byte change in SEED, CLAIMS, court decisions, genealogy, witness-role, L-band, or usefulness ledger changes the hash.",
    });
}

fn project_s12_registry_binding_law(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::RegistryHashV2BindsToFrozenCorpusHash,
            "registry_hash_v2 binds to the frozen corpus_hash_v1 via every spec's source_corpus_hash field.",
        ),
        (
            PrecedentReason::HashFrozenT10RequiresNonZeroSourceCorpusHash,
            "A spec with corpus_binding_status = HashFrozenT10 MUST carry source_corpus_hash != [0; 32]; the reverse holds too (cross-field rule).",
        ),
    ];
    for (reason, notes) in laws {
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind: PrecedentKind::RegistryBindingLaw,
            source: PrecedentSource::S12RegistryBinding,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Hard,
            notes,
        });
    }
}

fn project_t11a_passport_coverage_law(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::EveryCanonicalRequiresPassport,
            "Every SEED canonical record produces exactly one passport via all_passports(); the count cannot be inflated by alias claims.",
        ),
        (
            PrecedentReason::EveryPassportRequiresAllConstitutionFlags,
            "Every passport's constitution_flags must declare all eight flags true; the corpus verifier rejects otherwise.",
        ),
    ];
    for (reason, notes) in laws {
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind: PrecedentKind::ConstitutionLaw,
            source: PrecedentSource::T11aPassportCoverage,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Hard,
            notes,
        });
    }
}

fn project_t9_deferred_gates(out: &mut Vec<CourtPrecedent>) {
    let laws = [
        (
            PrecedentReason::CaseFileV2BodyDeferredToT11d,
            "CaseFileV2 episode-transcript body is deferred to T.11d.",
        ),
        (
            PrecedentReason::UnitSemanticsDeferredToT11e,
            "UnitSemanticsReceipt is deferred to T.11e.",
        ),
        (
            PrecedentReason::SamplingLawDeferredToT11e,
            "SamplingLawReceipt is deferred to T.11e.",
        ),
        (
            PrecedentReason::ExternalProvenanceExportDeferredPostS13,
            "External provenance export (DSFB-PROV / OpenLineage / NIST AI RMF / RO-Crate) is deferred to post-S1.3.",
        ),
    ];
    for (reason, notes) in laws {
        out.push(CourtPrecedent {
            id: PrecedentId(0),
            kind: PrecedentKind::DeferredGateLaw,
            source: PrecedentSource::T9DeferredGate,
            binding: PrecedentBinding::Global,
            reason,
            severity: PrecedentSeverity::Deferred,
            notes,
        });
    }
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str_canon(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_binding(out: &mut Vec<u8>, b: &PrecedentBinding) {
    match b {
        PrecedentBinding::Global => {
            write_str_canon(out, "Global");
        }
        PrecedentBinding::SingleCanonical(id) => {
            write_str_canon(out, "SingleCanonical");
            write_u32(out, id.0);
        }
        PrecedentBinding::Composition { subject, parents } => {
            write_str_canon(out, "Composition");
            write_u32(out, subject.0);
            let mut sorted: Vec<u32> = parents.iter().map(|p| p.0).collect();
            sorted.sort_unstable();
            write_u32(out, u32::try_from(sorted.len()).unwrap_or(u32::MAX));
            for id in sorted {
                write_u32(out, id);
            }
        }
        PrecedentBinding::AliasToCanonical {
            alias_id,
            canonical_id,
        } => {
            write_str_canon(out, "AliasToCanonical");
            write_u32(out, alias_id.0);
            write_u32(out, canonical_id.0);
        }
        PrecedentBinding::CanonicalToCanonical { from, to } => {
            write_str_canon(out, "CanonicalToCanonical");
            write_u32(out, from.0);
            write_u32(out, to.0);
        }
    }
}

fn write_precedent(out: &mut Vec<u8>, p: &CourtPrecedent) {
    write_u32(out, p.id.0);
    write_str_canon(out, p.kind.as_str());
    write_str_canon(out, p.source.as_str());
    write_binding(out, &p.binding);
    write_str_canon(out, p.reason.as_str());
    write_str_canon(out, p.severity.as_str());
    // Notes are auditor-facing prose; include them so the hash
    // catches accidental note drift.
    write_str_canon(out, p.notes);
}

fn compute_precedent_hash_v1_raw(precedents: &[CourtPrecedent]) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(COURT_PRECEDENT_DOMAIN.as_bytes());
    write_str_canon(&mut buf, COURT_PRECEDENT_SCHEMA_V1);
    write_u32(
        &mut buf,
        u32::try_from(precedents.len()).unwrap_or(u32::MAX),
    );
    for p in precedents {
        write_precedent(&mut buf, p);
    }
    sha256(&buf)
}

/// Compute `precedent_hash_v1` for the given `PrecedentSet`. The
/// hash includes the schema id, precedent count, and every
/// precedent's canonical-byte projection in sorted order.
///
/// Two builds against the same corpus produce the same hash.
#[must_use]
pub fn compute_precedent_hash_v1(set: &PrecedentSet) -> [u8; 32] {
    compute_precedent_hash_v1_raw(&set.precedents)
}

/// One verification failure on a `PrecedentSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecedentVerifyError {
    /// The id of the offending precedent (or `PrecedentId(0)`
    /// if the failure is set-level rather than per-precedent).
    pub id: PrecedentId,
    /// Structured failure kind.
    pub kind: PrecedentVerifyErrorKind,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Structured precedent-verifier error category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecedentVerifyErrorKind {
    /// The precedent binding names a canonical_id that is not in
    /// SEED.
    MissingCanonicalSubject,
    /// The precedent binding names an alias_id that is not in
    /// CLAIMS.
    MissingAliasSubject,
    /// The precedent kind / reason pair is not on the panel-
    /// locked admissible list. Most notably, AliasCollapse with
    /// `SameFormulaDifferentWitnessRole` is rejected — alias
    /// collapse requires same formula AND same witness role.
    KindReasonIncompatible,
    /// The precedent's binding shape does not match its kind
    /// (e.g. a `Global` binding on a `DedupCanonical` precedent).
    KindBindingIncompatible,
    /// Two precedents in the set share an id. This should not
    /// happen because the collector reassigns ids after sort.
    DuplicatePrecedentId,
}

/// Verify a `PrecedentSet` against the T.11b structural
/// invariants. Returns the list of failures (empty if clean).
#[must_use]
pub fn verify_precedent_set(set: &PrecedentSet) -> Vec<PrecedentVerifyError> {
    let mut errors: Vec<PrecedentVerifyError> = Vec::new();
    let seed_ids: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let alias_ids: Vec<u32> = CLAIMS.iter().map(|c| c.alias_id.0).collect();
    let mut seen_ids: Vec<u32> = Vec::with_capacity(set.precedents.len());
    for p in &set.precedents {
        if seen_ids.contains(&p.id.0) {
            errors.push(PrecedentVerifyError {
                id: p.id,
                kind: PrecedentVerifyErrorKind::DuplicatePrecedentId,
                message: format!("duplicate precedent id {}", p.id.0),
            });
        }
        seen_ids.push(p.id.0);

        if let Err(err) = verify_binding_subjects(p, &seed_ids, &alias_ids) {
            errors.push(err);
        }
        if let Err(err) = verify_kind_reason(p) {
            errors.push(err);
        }
        if let Err(err) = verify_kind_binding(p) {
            errors.push(err);
        }
    }
    errors
}

fn verify_binding_subjects(
    p: &CourtPrecedent,
    seed_ids: &[u32],
    alias_ids: &[u32],
) -> Result<(), PrecedentVerifyError> {
    let check_canonical = |id: DetectorCanonicalId| -> Result<(), PrecedentVerifyError> {
        if !seed_ids.contains(&id.0) {
            return Err(PrecedentVerifyError {
                id: p.id,
                kind: PrecedentVerifyErrorKind::MissingCanonicalSubject,
                message: format!(
                    "precedent id {} binds to canonical_id {} which is not in SEED",
                    p.id.0, id.0
                ),
            });
        }
        Ok(())
    };
    match &p.binding {
        PrecedentBinding::Global => Ok(()),
        PrecedentBinding::SingleCanonical(id) => check_canonical(*id),
        PrecedentBinding::Composition { subject, parents } => {
            check_canonical(*subject)?;
            for par in parents {
                check_canonical(*par)?;
            }
            Ok(())
        }
        PrecedentBinding::AliasToCanonical {
            alias_id,
            canonical_id,
        } => {
            if !alias_ids.contains(&alias_id.0) {
                return Err(PrecedentVerifyError {
                    id: p.id,
                    kind: PrecedentVerifyErrorKind::MissingAliasSubject,
                    message: format!(
                        "precedent id {} binds to alias_id {} which is not in CLAIMS",
                        p.id.0, alias_id.0
                    ),
                });
            }
            check_canonical(*canonical_id)
        }
        PrecedentBinding::CanonicalToCanonical { from, to } => {
            check_canonical(*from)?;
            check_canonical(*to)
        }
    }
}

fn verify_kind_reason(p: &CourtPrecedent) -> Result<(), PrecedentVerifyError> {
    // AliasCollapse requires a "same role" alias reason —
    // SameFormulaDifferentWitnessRole or DifferentDecisionFunctional
    // are not valid alias reasons because they imply distinct
    // semantic roles. This preserves the T.3 invariant that
    // semantic_role_hash collapse only happens when role is
    // preserved.
    let alias_collapse_admissible = matches!(
        p.reason,
        PrecedentReason::SameFormulaSameParametersSameContract
            | PrecedentReason::SameFormulaDifferentParameters
            | PrecedentReason::SameFormulaDifferentInputContract
    );
    if matches!(p.kind, PrecedentKind::AliasCollapse) && !alias_collapse_admissible {
        return Err(PrecedentVerifyError {
            id: p.id,
            kind: PrecedentVerifyErrorKind::KindReasonIncompatible,
            message: format!(
                "precedent id {} declares AliasCollapse with reason {} which implies semantic-role drift; alias collapse requires same formula + same witness role",
                p.id.0,
                p.reason.as_str()
            ),
        });
    }
    Ok(())
}

fn verify_kind_binding(p: &CourtPrecedent) -> Result<(), PrecedentVerifyError> {
    // Global laws must use the Global binding; per-record
    // precedents must NOT use Global.
    let is_global_kind = matches!(
        p.kind,
        PrecedentKind::WitnessLaw
            | PrecedentKind::NegativeWitnessLaw
            | PrecedentKind::LBandHonestyLaw
            | PrecedentKind::UsefulnessHonestyLaw
            | PrecedentKind::CorpusHashLaw
            | PrecedentKind::RegistryBindingLaw
            | PrecedentKind::ConstitutionLaw
            | PrecedentKind::DeferredGateLaw
    );
    let is_global_binding = matches!(p.binding, PrecedentBinding::Global);
    if is_global_kind != is_global_binding {
        return Err(PrecedentVerifyError {
            id: p.id,
            kind: PrecedentVerifyErrorKind::KindBindingIncompatible,
            message: format!(
                "precedent id {} kind {} requires {} binding; got {}",
                p.id.0,
                p.kind.as_str(),
                if is_global_kind {
                    "Global"
                } else {
                    "per-record"
                },
                if is_global_binding {
                    "Global"
                } else {
                    "per-record"
                }
            ),
        });
    }
    Ok(())
}

/// Return the set of precedents that link to `canonical_id`.
/// Every passport's `linked_precedent_ids` is the result of this
/// function for its canonical id, in canonical-sorted order.
///
/// A precedent links to a canonical id if:
///
/// - the binding is `SingleCanonical(canonical_id)`, OR
/// - the binding is `Composition { subject, parents }` and the
///   id is the subject or a parent, OR
/// - the binding is `AliasToCanonical { canonical_id: cid, .. }`
///   and `cid == canonical_id`, OR
/// - the binding is `CanonicalToCanonical { from, to }` and the
///   id is either endpoint, OR
/// - the binding is `Global` (global laws apply to every record).
#[must_use]
pub fn precedents_for_canonical(
    set: &PrecedentSet,
    canonical_id: DetectorCanonicalId,
) -> Vec<PrecedentId> {
    let mut out: Vec<PrecedentId> = Vec::new();
    for p in &set.precedents {
        let touches = match &p.binding {
            PrecedentBinding::Global => true,
            PrecedentBinding::SingleCanonical(id) => *id == canonical_id,
            PrecedentBinding::Composition { subject, parents } => {
                *subject == canonical_id || parents.contains(&canonical_id)
            }
            PrecedentBinding::AliasToCanonical {
                canonical_id: cid, ..
            } => *cid == canonical_id,
            PrecedentBinding::CanonicalToCanonical { from, to } => {
                *from == canonical_id || *to == canonical_id
            }
        };
        if touches {
            out.push(p.id);
        }
    }
    out.sort_unstable();
    out
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[usize::from(b >> 4)] as char);
        s.push(HEX[usize::from(b & 0x0F)] as char);
    }
    s
}

fn format_binding(b: &PrecedentBinding) -> String {
    match b {
        PrecedentBinding::Global => "Global".to_string(),
        PrecedentBinding::SingleCanonical(id) => format!("SingleCanonical({})", id.0),
        PrecedentBinding::Composition { subject, parents } => {
            let mut sorted: Vec<u32> = parents.iter().map(|p| p.0).collect();
            sorted.sort_unstable();
            let mut s = format!("Composition(subject={}, parents=[", subject.0);
            for (i, id) in sorted.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                let _ = write!(s, "{id}");
            }
            s.push_str("])");
            s
        }
        PrecedentBinding::AliasToCanonical {
            alias_id,
            canonical_id,
        } => format!(
            "AliasToCanonical(alias={}, canonical={})",
            alias_id.0, canonical_id.0
        ),
        PrecedentBinding::CanonicalToCanonical { from, to } => {
            format!("CanonicalToCanonical(from={}, to={})", from.0, to.0)
        }
    }
}

/// Render the full precedent set as a deterministic
/// human-readable text report. Two calls produce byte-identical
/// output.
#[must_use]
pub fn render_precedents_text(set: &PrecedentSet) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas — Court Precedents (T.11b)\n");
    out.push_str("================================================================\n");
    let _ = writeln!(out, "Schema    : {COURT_PRECEDENT_SCHEMA_V1}");
    let _ = writeln!(out, "Count     : {}", set.precedents.len());
    let _ = writeln!(out, "Hash (v1) : {}", hex_lower(&set.precedent_hash_v1));
    out.push('\n');
    for p in &set.precedents {
        let _ = writeln!(
            out,
            "[{}] {} ({})",
            p.id.0,
            p.kind.as_str(),
            p.source.as_str()
        );
        let _ = writeln!(out, "    binding  : {}", format_binding(&p.binding));
        let _ = writeln!(out, "    reason   : {}", p.reason.as_str());
        let _ = writeln!(out, "    severity : {}", p.severity.as_str());
        if !p.notes.is_empty() {
            let _ = writeln!(out, "    notes    : {}", p.notes);
        }
        out.push('\n');
    }
    out
}

fn json_quote(out: &mut String, s: &str) {
    out.push('"');
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
    out.push('"');
}

/// Render the full precedent set as a deterministic JSON object.
/// Two calls produce byte-identical output.
#[must_use]
pub fn render_precedents_json(set: &PrecedentSet) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("{\n");
    out.push_str("  \"schema\": ");
    json_quote(&mut out, COURT_PRECEDENT_SCHEMA_V1);
    out.push_str(",\n");
    let _ = writeln!(out, "  \"count\": {},", set.precedents.len());
    let _ = writeln!(
        out,
        "  \"precedent_hash_v1\": \"{}\",",
        hex_lower(&set.precedent_hash_v1)
    );
    out.push_str("  \"precedents\": [\n");
    for (i, p) in set.precedents.iter().enumerate() {
        out.push_str("    {\n");
        let _ = writeln!(out, "      \"id\": {},", p.id.0);
        out.push_str("      \"kind\": ");
        json_quote(&mut out, p.kind.as_str());
        out.push_str(",\n");
        out.push_str("      \"source\": ");
        json_quote(&mut out, p.source.as_str());
        out.push_str(",\n");
        out.push_str("      \"binding\": ");
        json_quote(&mut out, &format_binding(&p.binding));
        out.push_str(",\n");
        out.push_str("      \"reason\": ");
        json_quote(&mut out, p.reason.as_str());
        out.push_str(",\n");
        out.push_str("      \"severity\": ");
        json_quote(&mut out, p.severity.as_str());
        out.push_str(",\n");
        out.push_str("      \"notes\": ");
        json_quote(&mut out, p.notes);
        out.push('\n');
        if i + 1 < set.precedents.len() {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}
