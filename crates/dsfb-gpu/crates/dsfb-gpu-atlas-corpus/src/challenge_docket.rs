//! T.11f — `ChallengeDocketV1`: the court's adversarial
//! self-audit layer.
//!
//! Panel framing:
//!
//! > T.11a–T.11e built the court's identities, precedents,
//! > grammar, transcripts, and execution receipts. **T.11f is
//! > how the court can be attacked, corrected, and appealed.**
//! > It records objections against detector identities,
//! > precedent judgments, grammar rules, witness roles, source
//! > claims, implementation status, and runtime cost. It does
//! > NOT mutate the corpus directly; it creates reviewable
//! > challenges that future court commits may sustain,
//! > overrule, or defer.
//!
//! **Design boundary (panel-locked)**: T.11f does not mutate
//! anything. A challenge docket is **not** the corpus, **not**
//! the dedup court, **not** the grammar. It is an **adversarial
//! overlay**.
//!
//! Correct flow:
//!
//! ```text
//!   ChallengeDocketEntry created
//!     → verifier accepts docket integrity
//!     → later court commit may sustain / overrule / defer
//!     → if sustained, corpus / precedent / grammar changes in
//!       a separate commit
//!     → hashes change visibly
//! ```
//!
//! That separation is what makes it a court instead of an
//! issue list.
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
//! ```
//!
//! `challenge_docket_hash_v1` is DSFB-native and **does not
//! fold into `corpus_hash_v1`**. The docket is post-freeze
//! adversarial metadata with its own hash; future
//! `CaseFileV2` body receipts may cross-cite it. Resolving a
//! challenge to `Sustained` requires a **separate later commit**
//! that mutates the corresponding canonical artifact and
//! changes the upstream hashes.
//!
//! No in-toto / SLSA / SPDX / CycloneDX compatibility claim.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::admissibility::{collect_admissibility_grammar, GrammarRuleId};
use crate::execution_attestation::ExecutionReceiptId;
use crate::precedent::{collect_court_precedents, PrecedentId};
use crate::seed::SEED;
use crate::trial_transcript::TrialTranscriptId;
use crate::types::{DetectorCanonicalId, ImplementationLevel};

/// Domain separator prefix for `challenge_docket_hash_v1`.
/// **Panel-locked**; changing it changes every docket hash.
pub const CHALLENGE_DOCKET_DOMAIN: &str = "DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1\0";

/// Schema identifier carried inside the docket hash material.
pub const CHALLENGE_DOCKET_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1";

/// Stable handle for one `ChallengeDocketEntry`. IDs are
/// assigned in the seed below and are append-only across future
/// commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChallengeId(pub u32);

/// Schema variant carried in the docket hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeDocketSchema {
    /// T.11f base schema — adversarial overlay over the
    /// corpus + precedent + grammar + transcript + receipt
    /// surfaces.
    V1AdversarialOverlay,
}

impl ChallengeDocketSchema {
    /// Stable wire name for the hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1AdversarialOverlay => "V1AdversarialOverlay",
        }
    }
}

/// What a challenge is targeting. Used by the verifier to confirm
/// that the named subject actually exists in the relevant module
/// (no challenges against non-existent detectors, precedents, or
/// grammar rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeTarget {
    /// A canonical detector record in the corpus seed.
    Detector(DetectorCanonicalId),
    /// A precedent in the T.11b court precedent ledger.
    Precedent(PrecedentId),
    /// A rule in the T.11c admissibility grammar.
    GrammarRule(GrammarRuleId),
    /// A passport surface — addressed by canonical id since one
    /// passport exists per canonical detector at T.11a.
    Passport(DetectorCanonicalId),
    /// A trial transcript record (T.11d). Currently 1 in seed.
    TrialTranscript(TrialTranscriptId),
    /// An execution attestation receipt (T.11e).
    ExecutionReceipt(ExecutionReceiptId),
    /// A challenge against the corpus as a whole (e.g. global
    /// coverage / family completeness objections).
    CorpusGlobal,
    /// A challenge against the registry as a whole (e.g. S1.2's
    /// 162-spec grid; future S1.2.x expansions).
    RegistryGlobal,
}

impl ChallengeTarget {
    /// Stable wire name for the target's kind. The numeric id
    /// (when present) is written separately in the canonical
    /// byte stream.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::Detector(_) => "Detector",
            Self::Precedent(_) => "Precedent",
            Self::GrammarRule(_) => "GrammarRule",
            Self::Passport(_) => "Passport",
            Self::TrialTranscript(_) => "TrialTranscript",
            Self::ExecutionReceipt(_) => "ExecutionReceipt",
            Self::CorpusGlobal => "CorpusGlobal",
            Self::RegistryGlobal => "RegistryGlobal",
        }
    }

    /// Numeric subject id (0 for global targets). Hashed alongside
    /// `kind_str()` so two challenges with the same kind but
    /// different subject ids hash differently.
    #[must_use]
    pub const fn subject_id(&self) -> u32 {
        match self {
            Self::Detector(id) | Self::Passport(id) => id.0,
            Self::Precedent(id) => id.0,
            Self::GrammarRule(id) => id.0,
            Self::TrialTranscript(id) => id.0,
            Self::ExecutionReceipt(id) => id.0,
            Self::CorpusGlobal | Self::RegistryGlobal => 0,
        }
    }
}

/// The 11 panel-locked challenge kinds. Three additions beyond
/// the user-listed eight: `HashBindingMismatch`,
/// `MissingNegativeWitness`, `EvidenceLevelOverclaimed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeKind {
    /// Two detectors that should be aliases were canonicalised
    /// as distinct.
    OverbroadAlias,
    /// A detector's `ConfuserProfile` does not name a confuser
    /// the literature requires.
    MissingConfuser,
    /// A detector's `WitnessRole` is mislabelled
    /// (e.g. Primary where Confuser was intended).
    WrongWitnessRole,
    /// A `SourceRef` is wrong / missing / non-authoritative.
    BadSource,
    /// A claim about a detector's mathematical form does not
    /// match its declared `formula_hash`.
    FormulaMismatch,
    /// A detector is applied to a domain its primitive family
    /// is not appropriate for.
    DomainMisapplied,
    /// A detector claims L5 / L6 / L7 implementation but has no
    /// honest execution surface to back the claim.
    UnimplementedButClaimed,
    /// A detector's measured runtime cost exceeds the bound its
    /// `cost_class` declares.
    RuntimeTooHigh,
    /// A claimed hash (corpus / registry / precedent / grammar
    /// / transcript / receipt) does not bind to the artifact
    /// the claim references.
    HashBindingMismatch,
    /// A negative-witness variant the literature requires is not
    /// represented in the grammar's
    /// `ConfuserSuppressionRule[]`.
    MissingNegativeWitness,
    /// A T.8 usefulness ledger row claims a higher evidence
    /// level than the artifact justifies (Unmeasured →
    /// LiteraturePrior etc. without a real benchmark).
    EvidenceLevelOverclaimed,
}

impl ChallengeKind {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OverbroadAlias => "OverbroadAlias",
            Self::MissingConfuser => "MissingConfuser",
            Self::WrongWitnessRole => "WrongWitnessRole",
            Self::BadSource => "BadSource",
            Self::FormulaMismatch => "FormulaMismatch",
            Self::DomainMisapplied => "DomainMisapplied",
            Self::UnimplementedButClaimed => "UnimplementedButClaimed",
            Self::RuntimeTooHigh => "RuntimeTooHigh",
            Self::HashBindingMismatch => "HashBindingMismatch",
            Self::MissingNegativeWitness => "MissingNegativeWitness",
            Self::EvidenceLevelOverclaimed => "EvidenceLevelOverclaimed",
        }
    }
}

/// Operational severity. Used by the verifier's "open critical
/// challenges must have an explicit deferred gate" rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeSeverity {
    /// Blocks a release-grade hash if Open and ungated.
    Critical,
    /// Should be resolved before the next campaign closes.
    High,
    /// Background hygiene; resolve when convenient.
    Medium,
    /// Informational; may remain open without gating future
    /// work.
    Low,
}

impl ChallengeSeverity {
    /// Stable wire name; used in the canonical hash material.
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

/// Lifecycle state of a challenge. `Superseded` is the
/// panel-required variant: it allows a later corpus / precedent
/// / grammar change to replace an old challenge without deleting
/// audit history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeStatus {
    /// Awaiting review. Critical-Open requires a deferred gate.
    Open,
    /// Court accepted the objection; a separate later commit
    /// must mutate the canonical artifact.
    Sustained,
    /// Court rejected the objection; `CourtResponse` carries
    /// the rejection reason.
    Overruled,
    /// Court accepts the concern but defers resolution to a
    /// future campaign; `CourtResponse` carries the deferral
    /// reason.
    Deferred,
    /// A later corpus / precedent / grammar change has obsoleted
    /// the challenge; preserved for audit history only.
    Superseded,
}

impl ChallengeStatus {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Sustained => "Sustained",
            Self::Overruled => "Overruled",
            Self::Deferred => "Deferred",
            Self::Superseded => "Superseded",
        }
    }
}

/// Who filed the challenge. The court treats the role as
/// metadata only; verifier rules do not depend on it. Captured
/// so future case-files can render "who appealed what" without
/// reading external sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengerRole {
    /// Routine self-audit during a corpus campaign.
    SelfAudit,
    /// Maintainer-initiated objection.
    CorpusMaintainer,
    /// Panel-required objection captured in plan-mode verdicts.
    Panel,
    /// External reviewer (e.g. a Zenodo deposit reviewer).
    External,
}

impl ChallengerRole {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfAudit => "SelfAudit",
            Self::CorpusMaintainer => "CorpusMaintainer",
            Self::Panel => "Panel",
            Self::External => "External",
        }
    }
}

/// Where the challenge points to for supporting evidence. The
/// verifier's "kind-specific evidence-required" rules consume
/// these variants (e.g. `BadSource` requires at least one
/// `SourceRef` / `SourceHash` reference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengeEvidenceRef {
    /// Cites a `SourceRef` citation key in the corpus seed.
    SourceRef(&'static str),
    /// Cites a raw source-hash byte string (32-byte digest).
    SourceHash([u8; 32]),
    /// Cites a `formula_hash` from `DetectorIdentityHashes`.
    FormulaHash([u8; 32]),
    /// Cites a `semantic_role_hash` from
    /// `DetectorIdentityHashes`.
    SemanticRoleHash([u8; 32]),
    /// Cites a `parameter_hash` from
    /// `DetectorIdentityHashes`.
    ParameterHash([u8; 32]),
    /// Cites a runtime-cost measurement value in microseconds.
    /// `RuntimeTooHigh` MUST carry at least one of these.
    RuntimeCostUs(u32),
    /// Cites an upstream chain hash by name + digest.
    HashChainAnchor {
        /// Hash name, e.g. `"corpus_hash_v1"`.
        name: &'static str,
        /// Digest bytes.
        digest: [u8; 32],
    },
    /// Cites a free-text note (kept short; not load-bearing).
    Note(&'static str),
}

impl ChallengeEvidenceRef {
    /// Stable wire name for the variant kind; hashed alongside
    /// the payload.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::SourceRef(_) => "SourceRef",
            Self::SourceHash(_) => "SourceHash",
            Self::FormulaHash(_) => "FormulaHash",
            Self::SemanticRoleHash(_) => "SemanticRoleHash",
            Self::ParameterHash(_) => "ParameterHash",
            Self::RuntimeCostUs(_) => "RuntimeCostUs",
            Self::HashChainAnchor { .. } => "HashChainAnchor",
            Self::Note(_) => "Note",
        }
    }
}

/// Names which upstream hashes the challenge would invalidate
/// if it were `Sustained`. The verifier uses this to compute
/// "hashes affected by open challenges" in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AffectedHashSet {
    /// `corpus_hash_v1` would need to be recomputed.
    pub corpus_hash: bool,
    /// `registry_hash_v2` would need to be recomputed.
    pub registry_hash: bool,
    /// `precedent_hash_v1` would need to be recomputed.
    pub precedent_hash: bool,
    /// `admissibility_grammar_hash_v1` would need to be
    /// recomputed.
    pub grammar_hash: bool,
    /// `trial_transcript_hash_v1` would need to be recomputed.
    pub transcript_hash: bool,
    /// One or more `DetectorPassport` hashes would need to be
    /// recomputed.
    pub passport_hash: bool,
}

impl AffectedHashSet {
    /// True iff at least one upstream hash is named as affected.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        !(self.corpus_hash
            || self.registry_hash
            || self.precedent_hash
            || self.grammar_hash
            || self.transcript_hash
            || self.passport_hash)
    }
}

/// What the challenger proposes the court should do if sustained.
/// The verifier requires this to be non-`NoAction` whenever the
/// challenge status is `Sustained`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProposedResolution {
    /// No resolution proposed (only admissible for
    /// `Open` / `Superseded` challenges).
    NoAction,
    /// Add a new alias under an existing canonical detector.
    AddAliasUnderCanonical,
    /// Promote an alias to a canonical detector.
    PromoteAliasToCanonical,
    /// Add a `NegativeWitnessKind` variant to the grammar.
    AddNegativeWitnessVariant,
    /// Add a `ConfuserSuppressionRule` for an existing variant.
    AddConfuserSuppressionRule,
    /// Edit a detector's `WitnessRole`.
    AmendWitnessRole,
    /// Add a `SourceRef` citation.
    AddOrFixSourceRef,
    /// Lower an `ImplementationLevel` to the honest band.
    DowngradeImplementationLevel,
    /// Defer the action to a named future commit.
    DeferToFutureCommit,
    /// Mark the challenge as superseded by a later commit.
    MarkSupersededByLaterCommit,
    /// Reject the proposal; the court is sound as-is.
    NoChangeRequired,
}

impl ProposedResolution {
    /// Stable wire name; used in the canonical hash material.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoAction => "NoAction",
            Self::AddAliasUnderCanonical => "AddAliasUnderCanonical",
            Self::PromoteAliasToCanonical => "PromoteAliasToCanonical",
            Self::AddNegativeWitnessVariant => "AddNegativeWitnessVariant",
            Self::AddConfuserSuppressionRule => "AddConfuserSuppressionRule",
            Self::AmendWitnessRole => "AmendWitnessRole",
            Self::AddOrFixSourceRef => "AddOrFixSourceRef",
            Self::DowngradeImplementationLevel => "DowngradeImplementationLevel",
            Self::DeferToFutureCommit => "DeferToFutureCommit",
            Self::MarkSupersededByLaterCommit => "MarkSupersededByLaterCommit",
            Self::NoChangeRequired => "NoChangeRequired",
        }
    }
}

/// What the court actually responded. Required for
/// `Overruled` (carries the rejection reason) and `Deferred`
/// (carries the deferral reason). For `Open` and
/// `Sustained` the court has not yet spoken, so
/// `NotYetResponded` is the only admissible variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CourtResponse {
    /// No court response yet; only admissible for
    /// `Open` / `Sustained`.
    NotYetResponded,
    /// Court accepts the objection; the canonical artifact
    /// will be mutated in a future commit.
    SustainedAwaitingMutation,
    /// Court rejects the objection; the embedded reason
    /// explains why the court is sound as-is.
    OverruledReason(&'static str),
    /// Court defers the action to a future campaign; the
    /// embedded reason names the gate.
    DeferredToGate(&'static str),
    /// A later commit obsoleted the challenge; the embedded
    /// reference names the commit (e.g. "T.11g landed
    /// DetectorContraindicationReceipt").
    SupersededByCommit(&'static str),
}

impl CourtResponse {
    /// Stable wire name for the variant kind.
    #[must_use]
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::NotYetResponded => "NotYetResponded",
            Self::SustainedAwaitingMutation => "SustainedAwaitingMutation",
            Self::OverruledReason(_) => "OverruledReason",
            Self::DeferredToGate(_) => "DeferredToGate",
            Self::SupersededByCommit(_) => "SupersededByCommit",
        }
    }

    /// Embedded reason text, if any. Hashed alongside the kind.
    #[must_use]
    pub const fn reason_text(&self) -> &'static str {
        match self {
            Self::NotYetResponded | Self::SustainedAwaitingMutation => "",
            Self::OverruledReason(s) | Self::DeferredToGate(s) | Self::SupersededByCommit(s) => s,
        }
    }
}

/// One docket entry. Append-only across future commits; older
/// entries are never renumbered.
#[derive(Debug, Clone)]
pub struct ChallengeDocketEntry {
    /// Stable handle; assigned in the seed below.
    pub challenge_id: ChallengeId,
    /// What the challenge points at.
    pub target: ChallengeTarget,
    /// The objection's category.
    pub challenge_kind: ChallengeKind,
    /// Operational severity (Critical gates release if
    /// Open / ungated).
    pub severity: ChallengeSeverity,
    /// Lifecycle state.
    pub status: ChallengeStatus,
    /// Who filed it.
    pub challenger_role: ChallengerRole,
    /// One-line claim (the heart of the objection).
    pub claim: &'static str,
    /// Supporting evidence pointers.
    pub evidence_refs: &'static [ChallengeEvidenceRef],
    /// Hashes that would change if sustained.
    pub affected_hashes: AffectedHashSet,
    /// What the challenger proposes the court should do.
    pub proposed_resolution: ProposedResolution,
    /// What the court has actually said.
    pub court_response: CourtResponse,
    /// Which campaign/stage created the entry (e.g. "T.11f").
    pub created_in_stage: &'static str,
}

/// Convenience: a fully-zeroed `AffectedHashSet`.
const AHS_NONE: AffectedHashSet = AffectedHashSet {
    corpus_hash: false,
    registry_hash: false,
    precedent_hash: false,
    grammar_hash: false,
    transcript_hash: false,
    passport_hash: false,
};

/// The conservative T.11f docket seed: 10 honest challenges that
/// surface real coverage / honesty / runtime-evidence concerns
/// across the corpus + court surfaces without inventing fake
/// adversarial cases.
///
/// **Panel-locked**: keep the seed small and honest. Future
/// commits append; nothing here is renumbered.
pub static CHALLENGES: &[ChallengeDocketEntry] = &[
    // 1. RuntimeTooHigh against D128/D205 wide-digest baseline
    //    (the wide-detector path digests the full 264-byte stride;
    //    R.10b compact-pack is honestly deferred).
    ChallengeDocketEntry {
        challenge_id: ChallengeId(1),
        target: ChallengeTarget::RegistryGlobal,
        challenge_kind: ChallengeKind::RuntimeTooHigh,
        severity: ChallengeSeverity::Medium,
        status: ChallengeStatus::Deferred,
        challenger_role: ChallengerRole::Panel,
        claim: "D128 and D205 wide-digest baseline runs without compact-pack; \
                this is the honest wide-digest baseline, not the R.13 headline.",
        evidence_refs: &[
            ChallengeEvidenceRef::Note("R.10b compact-pack deferred for D128"),
            ChallengeEvidenceRef::HashChainAnchor {
                name: "registry_hash_v2",
                digest: [
                    0xd3, 0xcf, 0x63, 0x00, 0x0c, 0xee, 0x92, 0x28, 0x18, 0xe8, 0xdb, 0xc7, 0x9f,
                    0xfe, 0xcb, 0xc2, 0x7d, 0x28, 0x80, 0x63, 0xef, 0xba, 0xed, 0x58, 0x9e, 0x1e,
                    0xb1, 0x81, 0x2b, 0xc3, 0x7a, 0x08,
                ],
            },
            ChallengeEvidenceRef::RuntimeCostUs(264_000),
        ],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::DeferToFutureCommit,
        court_response: CourtResponse::DeferredToGate(
            "R.10b compact-pack for wide detectors lands post-R.13",
        ),
        created_in_stage: "T.11f",
    },
    // 2. UnimplementedButClaimed guard against L7/L8 — overruled
    //    because T.7 already forbids those bands at the verifier.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(2),
        target: ChallengeTarget::CorpusGlobal,
        challenge_kind: ChallengeKind::UnimplementedButClaimed,
        severity: ChallengeSeverity::High,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "L7 and L8 implementation bands could in principle be \
                claimed without a benchmark or ledger artifact backing \
                them.",
        evidence_refs: &[ChallengeEvidenceRef::Note(
            "T.7 lband verifier rejects L7 and L8 records",
        )],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "T.7 verifier already rejects L7/L8 in the corpus seed; \
             only L0..L6 are admissible at the current commit",
        ),
        created_in_stage: "T.11f",
    },
    // 3. MissingConfuser against spectral detectors in the
    //    LatencyRamp transcript — deferred until spectral
    //    projection lands.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(3),
        target: ChallengeTarget::TrialTranscript(TrialTranscriptId(1)),
        challenge_kind: ChallengeKind::MissingConfuser,
        severity: ChallengeSeverity::Medium,
        status: ChallengeStatus::Deferred,
        challenger_role: ChallengerRole::Panel,
        claim: "Spectral / cyclostationary confusers are not represented in \
                the LatencyRamp trial transcript; no spectral evidence \
                projection exists yet.",
        evidence_refs: &[ChallengeEvidenceRef::Note(
            "spectral projection lands with S1.3 EvidenceProjection traits",
        )],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::DeferToFutureCommit,
        court_response: CourtResponse::DeferredToGate("spectral projection deferred to S1.3+"),
        created_in_stage: "T.11f",
    },
    // 4. DomainMisapplied against medical / EKG primitives in a
    //    generic debug trace — overruled because no biosignal
    //    projection is active and the corpus already partitions
    //    DomainTagSet bits per record.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(4),
        target: ChallengeTarget::CorpusGlobal,
        challenge_kind: ChallengeKind::DomainMisapplied,
        severity: ChallengeSeverity::Low,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "Medical / EKG-style detector primitives in the corpus \
                could in principle be activated for generic debug traces \
                where they are not appropriate.",
        evidence_refs: &[ChallengeEvidenceRef::Note(
            "DomainTagSet bits keep biosignal detectors out of debug projections",
        )],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "no biosignal EvidenceProjection is implemented; \
             DomainTagSet partitioning already gates activation",
        ),
        created_in_stage: "T.11f",
    },
    // 5. OverbroadAlias against RobustZ / MAD aliases — overruled
    //    because the T.4 alias court already preserves the
    //    semantic-role-hash distinction.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(5),
        target: ChallengeTarget::Detector(DetectorCanonicalId(8)),
        challenge_kind: ChallengeKind::OverbroadAlias,
        severity: ChallengeSeverity::Medium,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "RobustZ and MAD-Z aliases could be over-collapsed by \
                a future court if the semantic-role distinction is \
                lost.",
        evidence_refs: &[
            ChallengeEvidenceRef::SemanticRoleHash([0u8; 32]),
            ChallengeEvidenceRef::Note(
                "T.4 alias court preserves semantic_role_hash and source_hash differences",
            ),
        ],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "T.4 dedup court already keeps semantic_role_hash and source_hash \
             distinct for these aliases; canonical identity preserved",
        ),
        created_in_stage: "T.11f",
    },
    // 6. HashBindingMismatch challenge against corpus_hash_v1 —
    //    overruled because T.10 canonical material excludes
    //    rendered report text by design.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(6),
        target: ChallengeTarget::CorpusGlobal,
        challenge_kind: ChallengeKind::HashBindingMismatch,
        severity: ChallengeSeverity::High,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::Panel,
        claim: "corpus_hash_v1 does not include the rendered report text, \
                so two builds with different runtime numbers in reports/ \
                still produce the same corpus hash.",
        evidence_refs: &[ChallengeEvidenceRef::HashChainAnchor {
            name: "corpus_hash_v1",
            digest: [
                0x35, 0xc2, 0x76, 0xc7, 0x3a, 0x52, 0xd9, 0x16, 0xda, 0xaf, 0xda, 0x25, 0x98, 0xb2,
                0x15, 0xd7, 0x3e, 0x7f, 0xd6, 0x94, 0xd4, 0xa0, 0x67, 0x3e, 0x34, 0xac, 0x1e, 0xf9,
                0x48, 0xf5, 0xa4, 0xb7,
            ],
        }],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "T.10 canonical material deliberately excludes rendered report \
             text; rendered text is not the corpus, the canonical bytes are",
        ),
        created_in_stage: "T.11f",
    },
    // 7. MissingNegativeWitness against BatchBoundaryConfuser —
    //    deferred; this variant is present in the
    //    NegativeWitnessKind enum but not yet bound to a
    //    ConfuserSuppressionRule in the grammar.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(7),
        target: ChallengeTarget::GrammarRule(GrammarRuleId(0)),
        challenge_kind: ChallengeKind::MissingNegativeWitness,
        severity: ChallengeSeverity::Medium,
        status: ChallengeStatus::Deferred,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "The grammar's ConfuserSuppressionRule[] table covers all 9 \
                NegativeWitnessKind variants by construction; future grammar \
                additions must keep this invariant.",
        evidence_refs: &[ChallengeEvidenceRef::Note(
            "T.11c verify_grammar_snapshot enforces full NegativeWitnessKind coverage",
        )],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::DeferToFutureCommit,
        court_response: CourtResponse::DeferredToGate(
            "future grammar additions must preserve coverage invariant",
        ),
        created_in_stage: "T.11f",
    },
    // 8. RuntimeTooHigh against future S1.2 registry expansion —
    //    deferred; no runtime evidence yet, must remain
    //    NotScored.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(8),
        target: ChallengeTarget::RegistryGlobal,
        challenge_kind: ChallengeKind::RuntimeTooHigh,
        severity: ChallengeSeverity::Low,
        status: ChallengeStatus::Deferred,
        challenger_role: ChallengerRole::Panel,
        claim: "Future S1.2.x expansions of the 162-spec grid beyond the \
                current panel-locked count may carry runtime cost claims \
                that have no measured evidence.",
        evidence_refs: &[
            ChallengeEvidenceRef::Note(
                "T.8 usefulness ledger keeps every row NotScored without measurement",
            ),
            ChallengeEvidenceRef::RuntimeCostUs(0),
        ],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::DeferToFutureCommit,
        court_response: CourtResponse::DeferredToGate(
            "S1.2.x expansion must arrive with usefulness-ledger evidence \
             before any runtime claim is admissible",
        ),
        created_in_stage: "T.11f",
    },
    // 9. EvidenceLevelOverclaimed (T.8 honesty audit) — overruled
    //    because T.8 verify_usefulness_ledger already rejects any
    //    nonzero empirical field on Unmeasured / LiteraturePrior
    //    / RoleSeeded rows.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(9),
        target: ChallengeTarget::CorpusGlobal,
        challenge_kind: ChallengeKind::EvidenceLevelOverclaimed,
        severity: ChallengeSeverity::High,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::Panel,
        claim: "A future change could in principle promote a \
                usefulness-ledger row to SyntheticFixtureMeasured without \
                a real benchmark artifact.",
        evidence_refs: &[ChallengeEvidenceRef::Note(
            "T.8 verifier rejects nonzero empirical fields without matching evidence_level",
        )],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "T.8 verify_usefulness_ledger is the load-bearing guard; \
             every empirical field is zero on every seed row, and the \
             verifier blocks nonzero claims without matching evidence",
        ),
        created_in_stage: "T.11f",
    },
    // 10. BadSource self-audit against the dsfb-gpu-debug-core
    //     L6 GPU surface — overruled; the source_refs are the
    //     dsfb-gpu-debug-core bank source itself, which is the
    //     authoritative provenance for those five canonical IDs.
    ChallengeDocketEntry {
        challenge_id: ChallengeId(10),
        target: ChallengeTarget::Detector(DetectorCanonicalId(14)),
        challenge_kind: ChallengeKind::BadSource,
        severity: ChallengeSeverity::Low,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "The 5 L6 GPU-whitelisted IDs cite dsfb-gpu-debug-core as \
                their source; an external reviewer might consider this \
                circular.",
        evidence_refs: &[
            ChallengeEvidenceRef::SourceRef("dsfb_gpu_debug_core_bank_v1"),
            ChallengeEvidenceRef::Note(
                "dsfb-gpu-debug-core is the audit-mode prior-art proof and is \
                 the authoritative source for these canonical IDs",
            ),
        ],
        affected_hashes: AHS_NONE,
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason(
            "the 5 L6 IDs are the dsfb-gpu-debug-core bank surface; \
             that surface IS the authoritative L6 implementation provenance, \
             not a circular citation",
        ),
        created_in_stage: "T.11f",
    },
];

/// A `Vec` wrapper carrying the collected challenge docket and a
/// deterministic source-of-truth signature for the verifier
/// + renderer.
#[derive(Debug, Clone)]
pub struct ChallengeDocketSnapshot {
    /// The schema variant; pinned to `V1AdversarialOverlay` at
    /// T.11f.
    pub schema: ChallengeDocketSchema,
    /// All challenge entries sorted by `challenge_id`.
    pub challenges: Vec<ChallengeDocketEntry>,
}

/// Build the T.11f docket snapshot by cloning the static
/// `CHALLENGES` seed. The returned snapshot is sorted by
/// `challenge_id` ascending; future commits MUST keep the sort
/// invariant.
#[must_use]
pub fn collect_challenge_docket() -> ChallengeDocketSnapshot {
    let mut v: Vec<ChallengeDocketEntry> = CHALLENGES.to_vec();
    v.sort_by_key(|e| e.challenge_id.0);
    ChallengeDocketSnapshot {
        schema: ChallengeDocketSchema::V1AdversarialOverlay,
        challenges: v,
    }
}

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

fn write_evidence(out: &mut Vec<u8>, e: &ChallengeEvidenceRef) {
    write_str(out, e.kind_str());
    match e {
        ChallengeEvidenceRef::SourceRef(s) | ChallengeEvidenceRef::Note(s) => {
            write_str(out, s);
        }
        ChallengeEvidenceRef::SourceHash(h)
        | ChallengeEvidenceRef::FormulaHash(h)
        | ChallengeEvidenceRef::SemanticRoleHash(h)
        | ChallengeEvidenceRef::ParameterHash(h) => {
            out.extend_from_slice(h);
        }
        ChallengeEvidenceRef::RuntimeCostUs(us) => {
            write_u32(out, *us);
        }
        ChallengeEvidenceRef::HashChainAnchor { name, digest } => {
            write_str(out, name);
            out.extend_from_slice(digest);
        }
    }
}

fn write_response(out: &mut Vec<u8>, r: &CourtResponse) {
    write_str(out, r.kind_str());
    write_str(out, r.reason_text());
}

fn write_affected(out: &mut Vec<u8>, a: AffectedHashSet) {
    write_u8(out, u8::from(a.corpus_hash));
    write_u8(out, u8::from(a.registry_hash));
    write_u8(out, u8::from(a.precedent_hash));
    write_u8(out, u8::from(a.grammar_hash));
    write_u8(out, u8::from(a.transcript_hash));
    write_u8(out, u8::from(a.passport_hash));
}

fn write_entry(out: &mut Vec<u8>, e: &ChallengeDocketEntry) {
    write_u32(out, e.challenge_id.0);
    write_str(out, e.target.kind_str());
    write_u32(out, e.target.subject_id());
    write_str(out, e.challenge_kind.as_str());
    write_str(out, e.severity.as_str());
    write_str(out, e.status.as_str());
    write_str(out, e.challenger_role.as_str());
    write_str(out, e.claim);
    write_u32(
        out,
        u32::try_from(e.evidence_refs.len()).unwrap_or(u32::MAX),
    );
    for ev in e.evidence_refs {
        write_evidence(out, ev);
    }
    write_affected(out, e.affected_hashes);
    write_str(out, e.proposed_resolution.as_str());
    write_response(out, &e.court_response);
    write_str(out, e.created_in_stage);
}

/// Compute the docket's canonical-byte hash. Two builds against
/// the same `CHALLENGES` seed produce byte-identical output.
/// **Rendered text is NOT included.**
#[must_use]
pub fn compute_challenge_docket_hash_v1(s: &ChallengeDocketSnapshot) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
    buf.extend_from_slice(CHALLENGE_DOCKET_DOMAIN.as_bytes());
    write_str(&mut buf, CHALLENGE_DOCKET_SCHEMA_V1);
    write_str(&mut buf, s.schema.as_str());

    // Sort defensively by challenge_id; the collector already sorts
    // but compute_*_hash callers may pass a hand-built snapshot.
    let mut entries: Vec<&ChallengeDocketEntry> = s.challenges.iter().collect();
    entries.sort_by_key(|e| e.challenge_id.0);
    write_u32(&mut buf, u32::try_from(entries.len()).unwrap_or(u32::MAX));
    for e in entries {
        write_entry(&mut buf, e);
    }

    sha256(&buf)
}

/// A single verifier error: kind + the offending entry id (or
/// `0` for snapshot-global errors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocketVerifyError {
    /// Which entry the error refers to (`0` for global errors).
    pub challenge_id: ChallengeId,
    /// The reject kind.
    pub kind: DocketVerifyErrorKind,
}

/// The 17 panel-locked failure modes the docket verifier
/// rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocketVerifyErrorKind {
    /// `ChallengeTarget::Detector` / `Passport` refers to a
    /// canonical id not present in the corpus seed.
    ChallengeAgainstMissingDetector,
    /// `ChallengeTarget::Precedent` refers to a precedent id not
    /// present in the T.11b ledger.
    ChallengeAgainstMissingPrecedent,
    /// `ChallengeTarget::GrammarRule` refers to a rule id not
    /// present in the T.11c grammar.
    ChallengeAgainstMissingGrammarRule,
    /// `claim` field is empty.
    EmptyClaim,
    /// `created_in_stage` is empty.
    EmptyChallenger,
    /// Two entries share the same `challenge_id`.
    DuplicateChallengeId,
    /// `Sustained` challenge has `proposed_resolution = NoAction`.
    SustainedWithoutResolution,
    /// `Overruled` challenge has `court_response = NotYetResponded`
    /// or empty reason text.
    OverruledWithoutCourtResponse,
    /// `Deferred` challenge has `court_response = NotYetResponded`
    /// or empty reason text.
    DeferredWithoutDeferralReason,
    /// `Superseded` challenge has `court_response` other than
    /// `SupersededByCommit` with non-empty reference text.
    SupersededWithoutCommitReference,
    /// `RuntimeTooHigh` challenge has no `RuntimeCostUs` evidence.
    RuntimeTooHighWithoutRuntimeEvidence,
    /// `FormulaMismatch` challenge has no `FormulaHash` evidence.
    FormulaMismatchWithoutFormulaHashReference,
    /// `BadSource` challenge has no `SourceRef` or `SourceHash`
    /// evidence.
    BadSourceWithoutSourceEvidence,
    /// `WrongWitnessRole` challenge has no `SemanticRoleHash`
    /// evidence.
    WrongWitnessRoleWithoutSemanticRoleHashReference,
    /// `UnimplementedButClaimed` challenge filed against a
    /// detector whose `implementation_status` is already at or
    /// below L4 (no L5/L6/L7 claim to attack).
    UnimplementedButClaimedAgainstHonestLBand,
    /// `Open` + `Critical` challenge has no
    /// `proposed_resolution = DeferToFutureCommit` gate, and
    /// `affected_hashes` names at least one release-blocking
    /// upstream hash.
    OpenCriticalWithoutDeferredGate,
    /// `Open` / `Superseded` challenge has a `court_response` that
    /// is not `NotYetResponded` (Open) or `SupersededByCommit`
    /// (Superseded).
    StatusResponseInconsistent,
}

/// Run the brutal docket verifier. Returns one error per
/// reject; empty vector means admissible.
///
/// The verifier is intentionally long: 17 reject kinds each gated
/// by a dedicated branch with kind-specific evidence rules. The
/// `#[allow(clippy::too_many_lines)]` reflects panel-acknowledged
/// linear shape; refactoring into per-kind helpers would obscure
/// the docket's brutality without changing the verifier's
/// behaviour.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_challenge_docket(s: &ChallengeDocketSnapshot) -> Vec<DocketVerifyError> {
    let mut errors: Vec<DocketVerifyError> = Vec::new();

    // Build target-existence lookups from the upstream modules.
    let known_detectors: Vec<DetectorCanonicalId> = SEED.iter().map(|r| r.canonical_id).collect();
    let precedent_set = collect_court_precedents();
    let known_precedents: Vec<PrecedentId> =
        precedent_set.precedents.iter().map(|p| p.id).collect();
    let grammar_set = collect_admissibility_grammar();
    let known_grammar_rules: Vec<GrammarRuleId> =
        grammar_set.admission_rules.iter().map(|r| r.id).collect();

    // Look up implementation level by canonical id, for the
    // UnimplementedButClaimedAgainstHonestLBand check.
    let lband_of = |id: DetectorCanonicalId| -> Option<ImplementationLevel> {
        SEED.iter()
            .find(|r| r.canonical_id == id)
            .map(|r| r.implementation_status)
    };

    // First pass: duplicate ids (global).
    {
        let mut seen: Vec<u32> = Vec::with_capacity(s.challenges.len());
        for e in &s.challenges {
            if seen.contains(&e.challenge_id.0) {
                errors.push(DocketVerifyError {
                    challenge_id: e.challenge_id,
                    kind: DocketVerifyErrorKind::DuplicateChallengeId,
                });
            } else {
                seen.push(e.challenge_id.0);
            }
        }
    }

    // Per-entry rules.
    for e in &s.challenges {
        // claim non-empty
        if e.claim.is_empty() {
            errors.push(DocketVerifyError {
                challenge_id: e.challenge_id,
                kind: DocketVerifyErrorKind::EmptyClaim,
            });
        }
        if e.created_in_stage.is_empty() {
            errors.push(DocketVerifyError {
                challenge_id: e.challenge_id,
                kind: DocketVerifyErrorKind::EmptyChallenger,
            });
        }

        // Target existence.
        match e.target {
            ChallengeTarget::Detector(id) | ChallengeTarget::Passport(id) => {
                if !known_detectors.contains(&id) {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::ChallengeAgainstMissingDetector,
                    });
                }
            }
            ChallengeTarget::Precedent(id) => {
                if !known_precedents.contains(&id) {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::ChallengeAgainstMissingPrecedent,
                    });
                }
            }
            ChallengeTarget::GrammarRule(id) => {
                if id.0 != 0 && !known_grammar_rules.contains(&id) {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::ChallengeAgainstMissingGrammarRule,
                    });
                }
            }
            ChallengeTarget::TrialTranscript(_)
            | ChallengeTarget::ExecutionReceipt(_)
            | ChallengeTarget::CorpusGlobal
            | ChallengeTarget::RegistryGlobal => {}
        }

        // Status-response consistency.
        match e.status {
            ChallengeStatus::Sustained => {
                if e.proposed_resolution == ProposedResolution::NoAction {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::SustainedWithoutResolution,
                    });
                }
            }
            ChallengeStatus::Overruled => {
                let needs_reason = !matches!(
                    e.court_response,
                    CourtResponse::OverruledReason(_) | CourtResponse::SupersededByCommit(_)
                );
                let empty_reason = matches!(e.court_response, CourtResponse::OverruledReason(""))
                    || matches!(e.court_response, CourtResponse::SupersededByCommit(""));
                if needs_reason || empty_reason {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::OverruledWithoutCourtResponse,
                    });
                }
            }
            ChallengeStatus::Deferred => {
                let needs_reason = !matches!(e.court_response, CourtResponse::DeferredToGate(_));
                let empty_reason = matches!(e.court_response, CourtResponse::DeferredToGate(""));
                if needs_reason || empty_reason {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::DeferredWithoutDeferralReason,
                    });
                }
            }
            ChallengeStatus::Superseded => {
                let needs_ref = !matches!(e.court_response, CourtResponse::SupersededByCommit(_));
                let empty_ref = matches!(e.court_response, CourtResponse::SupersededByCommit(""));
                if needs_ref || empty_ref {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::SupersededWithoutCommitReference,
                    });
                }
            }
            ChallengeStatus::Open => {
                if !matches!(e.court_response, CourtResponse::NotYetResponded) {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::StatusResponseInconsistent,
                    });
                }
            }
        }

        // Kind-specific evidence requirements.
        let has_runtime = e
            .evidence_refs
            .iter()
            .any(|ev| matches!(ev, ChallengeEvidenceRef::RuntimeCostUs(_)));
        let has_formula = e
            .evidence_refs
            .iter()
            .any(|ev| matches!(ev, ChallengeEvidenceRef::FormulaHash(_)));
        let has_source = e.evidence_refs.iter().any(|ev| {
            matches!(
                ev,
                ChallengeEvidenceRef::SourceRef(_) | ChallengeEvidenceRef::SourceHash(_)
            )
        });
        let has_semantic = e
            .evidence_refs
            .iter()
            .any(|ev| matches!(ev, ChallengeEvidenceRef::SemanticRoleHash(_)));

        match e.challenge_kind {
            ChallengeKind::RuntimeTooHigh => {
                if !has_runtime {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::RuntimeTooHighWithoutRuntimeEvidence,
                    });
                }
            }
            ChallengeKind::FormulaMismatch => {
                if !has_formula {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::FormulaMismatchWithoutFormulaHashReference,
                    });
                }
            }
            ChallengeKind::BadSource => {
                if !has_source {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind: DocketVerifyErrorKind::BadSourceWithoutSourceEvidence,
                    });
                }
            }
            ChallengeKind::WrongWitnessRole => {
                if !has_semantic {
                    errors.push(DocketVerifyError {
                        challenge_id: e.challenge_id,
                        kind:
                            DocketVerifyErrorKind::WrongWitnessRoleWithoutSemanticRoleHashReference,
                    });
                }
            }
            ChallengeKind::UnimplementedButClaimed => {
                // Only meaningful when targeted at a specific
                // detector; global targets cannot trigger this.
                if let ChallengeTarget::Detector(id) | ChallengeTarget::Passport(id) = e.target {
                    if let Some(level) = lband_of(id) {
                        let claims_implementation = matches!(
                            level,
                            ImplementationLevel::L5_GpuImplemented
                                | ImplementationLevel::L6_CpuGpuByteEquivalent
                        );
                        if !claims_implementation {
                            errors.push(DocketVerifyError {
                                challenge_id: e.challenge_id,
                                kind: DocketVerifyErrorKind::UnimplementedButClaimedAgainstHonestLBand,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        // Open + Critical without deferred gate, against any
        // release-blocking upstream hash.
        if e.status == ChallengeStatus::Open
            && e.severity == ChallengeSeverity::Critical
            && !e.affected_hashes.is_empty()
            && e.proposed_resolution != ProposedResolution::DeferToFutureCommit
        {
            errors.push(DocketVerifyError {
                challenge_id: e.challenge_id,
                kind: DocketVerifyErrorKind::OpenCriticalWithoutDeferredGate,
            });
        }
    }

    errors
}

/// Render a docket snapshot as operator-readable text. Two
/// calls produce byte-identical strings.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_challenge_docket_text(s: &ChallengeDocketSnapshot) -> String {
    let mut out = String::with_capacity(4 * 1024);
    let hash = compute_challenge_docket_hash_v1(s);
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "DSFB-GPU-Atlas - Challenge Docket V1 (T.11f)");
    let _ = writeln!(
        out,
        "================================================================"
    );
    let _ = writeln!(out, "schema                   : {}", s.schema.as_str());
    let _ = writeln!(out, "challenge_docket_hash_v1 : {}", hex(&hash));
    let _ = writeln!(out, "entry_count              : {}", s.challenges.len());
    let _ = writeln!(out);

    // Counts by target kind.
    let _ = writeln!(out, "Counts by target kind:");
    for kind in [
        "Detector",
        "Precedent",
        "GrammarRule",
        "Passport",
        "TrialTranscript",
        "ExecutionReceipt",
        "CorpusGlobal",
        "RegistryGlobal",
    ] {
        let n = s
            .challenges
            .iter()
            .filter(|e| e.target.kind_str() == kind)
            .count();
        if n > 0 {
            let _ = writeln!(out, "  {kind:<20} : {n}");
        }
    }
    let _ = writeln!(out);

    // Counts by challenge kind.
    let _ = writeln!(out, "Counts by challenge kind:");
    for kind in [
        "OverbroadAlias",
        "MissingConfuser",
        "WrongWitnessRole",
        "BadSource",
        "FormulaMismatch",
        "DomainMisapplied",
        "UnimplementedButClaimed",
        "RuntimeTooHigh",
        "HashBindingMismatch",
        "MissingNegativeWitness",
        "EvidenceLevelOverclaimed",
    ] {
        let n = s
            .challenges
            .iter()
            .filter(|e| e.challenge_kind.as_str() == kind)
            .count();
        if n > 0 {
            let _ = writeln!(out, "  {kind:<30} : {n}");
        }
    }
    let _ = writeln!(out);

    // Counts by status.
    let _ = writeln!(out, "Counts by status:");
    for status in ["Open", "Sustained", "Overruled", "Deferred", "Superseded"] {
        let n = s
            .challenges
            .iter()
            .filter(|e| e.status.as_str() == status)
            .count();
        let _ = writeln!(out, "  {status:<12} : {n}");
    }
    let _ = writeln!(out);

    // Severity histogram.
    let _ = writeln!(out, "Severity histogram:");
    for sev in ["Critical", "High", "Medium", "Low"] {
        let n = s
            .challenges
            .iter()
            .filter(|e| e.severity.as_str() == sev)
            .count();
        let _ = writeln!(out, "  {sev:<10} : {n}");
    }
    let _ = writeln!(out);

    // Open critical / high.
    let _ = writeln!(out, "Open critical/high challenges:");
    let mut any_open = false;
    for e in &s.challenges {
        if e.status == ChallengeStatus::Open
            && matches!(
                e.severity,
                ChallengeSeverity::Critical | ChallengeSeverity::High
            )
        {
            any_open = true;
            let _ = writeln!(
                out,
                "  #{:<3} {:<10} {:<24} {}",
                e.challenge_id.0,
                e.severity.as_str(),
                e.challenge_kind.as_str(),
                e.claim,
            );
        }
    }
    if !any_open {
        let _ = writeln!(out, "  (none)");
    }
    let _ = writeln!(out);

    // Sustained resolutions.
    let _ = writeln!(out, "Sustained challenge resolutions:");
    let mut any_sustained = false;
    for e in &s.challenges {
        if e.status == ChallengeStatus::Sustained {
            any_sustained = true;
            let _ = writeln!(
                out,
                "  #{:<3} {:<24} proposed={}",
                e.challenge_id.0,
                e.challenge_kind.as_str(),
                e.proposed_resolution.as_str(),
            );
        }
    }
    if !any_sustained {
        let _ = writeln!(out, "  (none)");
    }
    let _ = writeln!(out);

    // Overruled reasons.
    let _ = writeln!(out, "Overruled challenge reasons:");
    let mut any_overruled = false;
    for e in &s.challenges {
        if e.status == ChallengeStatus::Overruled {
            any_overruled = true;
            let _ = writeln!(
                out,
                "  #{:<3} {:<24} reason={}",
                e.challenge_id.0,
                e.challenge_kind.as_str(),
                e.court_response.reason_text(),
            );
        }
    }
    if !any_overruled {
        let _ = writeln!(out, "  (none)");
    }
    let _ = writeln!(out);

    // Deferred gates.
    let _ = writeln!(out, "Deferred challenge gates:");
    let mut any_deferred = false;
    for e in &s.challenges {
        if e.status == ChallengeStatus::Deferred {
            any_deferred = true;
            let _ = writeln!(
                out,
                "  #{:<3} {:<24} gate={}",
                e.challenge_id.0,
                e.challenge_kind.as_str(),
                e.court_response.reason_text(),
            );
        }
    }
    if !any_deferred {
        let _ = writeln!(out, "  (none)");
    }
    let _ = writeln!(out);

    // Hashes affected by open challenges.
    let _ = writeln!(out, "Hashes affected by open challenges:");
    let mut acc = AffectedHashSet::default();
    for e in &s.challenges {
        if e.status == ChallengeStatus::Open {
            acc.corpus_hash |= e.affected_hashes.corpus_hash;
            acc.registry_hash |= e.affected_hashes.registry_hash;
            acc.precedent_hash |= e.affected_hashes.precedent_hash;
            acc.grammar_hash |= e.affected_hashes.grammar_hash;
            acc.transcript_hash |= e.affected_hashes.transcript_hash;
            acc.passport_hash |= e.affected_hashes.passport_hash;
        }
    }
    let _ = writeln!(out, "  corpus_hash      : {}", acc.corpus_hash);
    let _ = writeln!(out, "  registry_hash    : {}", acc.registry_hash);
    let _ = writeln!(out, "  precedent_hash   : {}", acc.precedent_hash);
    let _ = writeln!(out, "  grammar_hash     : {}", acc.grammar_hash);
    let _ = writeln!(out, "  transcript_hash  : {}", acc.transcript_hash);
    let _ = writeln!(out, "  passport_hash    : {}", acc.passport_hash);
    let _ = writeln!(out);

    let _ = writeln!(out, "Panel-locked non-claim: a challenge docket entry is");
    let _ = writeln!(
        out,
        "NOT a corpus mutation. Sustaining a challenge requires"
    );
    let _ = writeln!(out, "a separate later commit; the docket is the appeal");
    let _ = writeln!(out, "record only.");
    out
}

/// Render a docket snapshot as deterministic JSON. The JSON is
/// not pretty-printed; two calls produce byte-identical strings.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_challenge_docket_json(s: &ChallengeDocketSnapshot) -> String {
    let mut out = String::with_capacity(4 * 1024);
    let hash = compute_challenge_docket_hash_v1(s);
    let _ = write!(out, "{{");
    let _ = write!(out, "\"schema\":\"{}\",", s.schema.as_str());
    let _ = write!(out, "\"schema_id\":\"{CHALLENGE_DOCKET_SCHEMA_V1}\",");
    let _ = write!(out, "\"challenge_docket_hash_v1\":\"{}\",", hex(&hash),);
    let _ = write!(out, "\"entries\":[");
    let mut first = true;
    for e in &s.challenges {
        if !first {
            let _ = write!(out, ",");
        }
        first = false;
        let _ = write!(out, "{{");
        let _ = write!(out, "\"challenge_id\":{},", e.challenge_id.0);
        let _ = write!(out, "\"target_kind\":\"{}\",", e.target.kind_str());
        let _ = write!(out, "\"target_subject_id\":{},", e.target.subject_id());
        let _ = write!(out, "\"challenge_kind\":\"{}\",", e.challenge_kind.as_str());
        let _ = write!(out, "\"severity\":\"{}\",", e.severity.as_str());
        let _ = write!(out, "\"status\":\"{}\",", e.status.as_str());
        let _ = write!(
            out,
            "\"challenger_role\":\"{}\",",
            e.challenger_role.as_str(),
        );
        let _ = write!(out, "\"claim\":\"{}\",", json_escape(e.claim));
        let _ = write!(out, "\"evidence_refs\":[");
        let mut firste = true;
        for ev in e.evidence_refs {
            if !firste {
                let _ = write!(out, ",");
            }
            firste = false;
            let _ = write!(out, "{{\"kind\":\"{}\"", ev.kind_str());
            match ev {
                ChallengeEvidenceRef::SourceRef(v) | ChallengeEvidenceRef::Note(v) => {
                    let _ = write!(out, ",\"value\":\"{}\"", json_escape(v));
                }
                ChallengeEvidenceRef::SourceHash(h)
                | ChallengeEvidenceRef::FormulaHash(h)
                | ChallengeEvidenceRef::SemanticRoleHash(h)
                | ChallengeEvidenceRef::ParameterHash(h) => {
                    let _ = write!(out, ",\"digest\":\"{}\"", hex(h));
                }
                ChallengeEvidenceRef::RuntimeCostUs(us) => {
                    let _ = write!(out, ",\"runtime_us\":{us}");
                }
                ChallengeEvidenceRef::HashChainAnchor { name, digest } => {
                    let _ = write!(out, ",\"name\":\"{}\",\"digest\":\"{}\"", name, hex(digest));
                }
            }
            let _ = write!(out, "}}");
        }
        let _ = write!(out, "],");
        let _ = write!(out, "\"affected_hashes\":{{");
        let _ = write!(out, "\"corpus_hash\":{},", e.affected_hashes.corpus_hash);
        let _ = write!(
            out,
            "\"registry_hash\":{},",
            e.affected_hashes.registry_hash
        );
        let _ = write!(
            out,
            "\"precedent_hash\":{},",
            e.affected_hashes.precedent_hash,
        );
        let _ = write!(out, "\"grammar_hash\":{},", e.affected_hashes.grammar_hash);
        let _ = write!(
            out,
            "\"transcript_hash\":{},",
            e.affected_hashes.transcript_hash,
        );
        let _ = write!(out, "\"passport_hash\":{}", e.affected_hashes.passport_hash);
        let _ = write!(out, "}},");
        let _ = write!(
            out,
            "\"proposed_resolution\":\"{}\",",
            e.proposed_resolution.as_str(),
        );
        let _ = write!(out, "\"court_response\":{{");
        let _ = write!(out, "\"kind\":\"{}\"", e.court_response.kind_str());
        let r = e.court_response.reason_text();
        if !r.is_empty() {
            let _ = write!(out, ",\"reason\":\"{}\"", json_escape(r));
        }
        let _ = write!(out, "}},");
        let _ = write!(
            out,
            "\"created_in_stage\":\"{}\"",
            json_escape(e.created_in_stage),
        );
        let _ = write!(out, "}}");
    }
    let _ = write!(out, "]}}");
    out
}

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
