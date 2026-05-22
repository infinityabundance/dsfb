//! T.11c — `AdmissibilityGrammarSnapshot`: versioned grammar of
//! admissible episode forms.
//!
//! The panel framing:
//!
//! > These are the only episode-admission shapes the court
//! > recognises. The admissibility grammar defines which witness
//! > configurations may become episodes; detector firings alone
//! > are never episodes until the bank-governed grammar admits
//! > them.
//!
//! T.11c is a **derived law layer** on top of T.11b precedents.
//! Every grammar rule cites one or more T.11b
//! [`crate::precedent::CourtPrecedent`] records. The collector
//! invents no new judgments — it turns the T.6 / T.7 witness and
//! confuser laws into a small, panel-locked set of episode-
//! admission rules that the future T.11d CaseFileV2 trial
//! transcript will cite.
//!
//! **Hash posture (panel-locked)**:
//!
//! - `corpus_hash_v1` stays frozen (T.10).
//! - `registry_hash_v2` stays frozen (S1.2).
//! - `precedent_hash_v1` stays frozen (T.11b).
//! - `admissibility_grammar_hash_v1` is a NEW receipt computed
//!   by [`compute_admissibility_grammar_hash_v1`].
//!
//! The future T.11d body will carry the full legal chain:
//!
//! ```text
//!   corpus_hash_v1
//!     → registry_hash_v2
//!     → precedent_hash_v1
//!     → admissibility_grammar_hash_v1
//!     → casefile_v2_body_hash
//! ```
//!
//! **Panel-locked non-claims (T.11c)**:
//!
//! - Does NOT emit CaseFileV2 episode-transcript bodies (T.11d).
//! - Does NOT implement UnitSemantics / SamplingLaw receipts
//!   (T.11e).
//! - Does NOT implement external provenance export (DSFB-PROV /
//!   OpenLineage / NIST AI RMF / RO-Crate).
//! - Does NOT change any prior hash; the grammar is a separate
//!   cumulative receipt.
//! - Does NOT include grammar rule ids on the passport hash;
//!   passport_hash bytes are unchanged. The passport-grammar
//!   crosswalk is a separate artifact.
//! - The admissibility-grammar hash is DSFB-native; no in-toto /
//!   SLSA / SPDX / CycloneDX compatibility claim.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::passport::all_passports;
use crate::precedent::{
    collect_court_precedents, CourtPrecedent, PrecedentId, PrecedentKind, PrecedentReason,
    PrecedentSet,
};
use crate::types::{DetectorCanonicalId, NegativeWitnessKind};

/// Domain separator prefix for `admissibility_grammar_hash_v1`.
/// **Panel-locked**; changing it changes the grammar hash.
pub const ADMISSIBILITY_GRAMMAR_DOMAIN: &str = "DSFB-GPU-ATLAS:ADMISSIBILITY-GRAMMAR:v1\0";

/// Schema identifier carried inside the grammar hash material.
pub const ADMISSIBILITY_GRAMMAR_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:ADMISSIBILITY-GRAMMAR:v1";

/// Stable handle for one grammar rule (admission or confuser-
/// suppression). IDs are assigned **after** canonical sort so two
/// builds against the same precedent set produce the same id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammarRuleId(pub u32);

/// Kind of grammar rule. Panel-locked at 9 variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrammarRuleKind {
    /// A rule that declares the admissible shape of an episode.
    EpisodeAdmission,
    /// A rule that suppresses or downgrades admission when a
    /// negative-witness firing is present.
    ConfuserSuppression,
    /// A rule restricting how clean-window witnesses may
    /// contribute (never admit alone).
    CleanWindowSupport,
    /// A rule restricting how recovery-edge witnesses may
    /// contribute (close, never originate alone).
    RecoveryClosure,
    /// A rule restricting how boundary witnesses may contribute
    /// (define start/end, never classify alone).
    BoundaryCondition,
    /// A rule that explicitly blocks negative-witness-only
    /// admission.
    NegativeWitnessRejection,
    /// A rule that names the minimum evidence threshold for any
    /// admission.
    MinimumEvidence,
    /// A rule binding admission to the bank-governed
    /// `BankAdmissionToken` (Semantic Non-Bypass Axiom).
    SemanticNonBypass,
    /// A rule that admits `Unknown` / `Deferred` outcomes
    /// explicitly rather than as silent failures.
    DeferredUnknown,
}

impl GrammarRuleKind {
    /// Canonical wire name for hashing + rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EpisodeAdmission => "EpisodeAdmission",
            Self::ConfuserSuppression => "ConfuserSuppression",
            Self::CleanWindowSupport => "CleanWindowSupport",
            Self::RecoveryClosure => "RecoveryClosure",
            Self::BoundaryCondition => "BoundaryCondition",
            Self::NegativeWitnessRejection => "NegativeWitnessRejection",
            Self::MinimumEvidence => "MinimumEvidence",
            Self::SemanticNonBypass => "SemanticNonBypass",
            Self::DeferredUnknown => "DeferredUnknown",
        }
    }
}

/// Severity of a grammar rule. Hard rules MUST be obeyed for
/// admission to occur; Soft rules are documented expectations;
/// Deferred rules name future commitments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrammarRuleSeverity {
    /// The future T.11d body verifier rejects admissions that
    /// violate this rule.
    Hard,
    /// Documented expectation; auditor surface only.
    Soft,
    /// Future commitment; the grammar declares it but no body
    /// yet enforces it.
    Deferred,
}

impl GrammarRuleSeverity {
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

/// One panel-locked evidence-requirement clause. Multiple
/// requirements compose AND-wise inside a single
/// [`EpisodeAdmissibilityRule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceRequirement {
    /// At least one primary witness must fire.
    AtLeastOnePrimaryWitness,
    /// At least one corroborating positive witness must fire
    /// (separate detector from the primary).
    AtLeastOneCorroboratingWitness,
    /// At least one boundary witness must fire.
    AtLeastOneBoundaryWitness,
    /// At least one recovery witness must fire.
    AtLeastOneRecoveryWitness,
    /// No admission may be carried by confuser witnesses alone.
    NoConfuserOnlyAdmission,
    /// No admission may be carried by clean-window witnesses
    /// alone.
    NoCleanWindowOnlyAdmission,
    /// No admission may be carried by boundary witnesses alone.
    NoBoundaryOnlyAdmission,
    /// No admission may be carried by recovery witnesses alone.
    NoRecoveryOnlyAdmission,
    /// The admission MUST carry a valid `BankAdmissionToken`
    /// (Semantic Non-Bypass Axiom).
    BankAdmissionTokenRequired,
    /// GPU detector output is evidence only; it cannot mint an
    /// admission token.
    GpuOutputIsEvidenceOnly,
    /// `Unknown` / `Deferred` outcomes MUST be explicit grammar
    /// outputs rather than silent failures.
    DeferredUnknownIsExplicit,
}

impl EvidenceRequirement {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AtLeastOnePrimaryWitness => "AtLeastOnePrimaryWitness",
            Self::AtLeastOneCorroboratingWitness => "AtLeastOneCorroboratingWitness",
            Self::AtLeastOneBoundaryWitness => "AtLeastOneBoundaryWitness",
            Self::AtLeastOneRecoveryWitness => "AtLeastOneRecoveryWitness",
            Self::NoConfuserOnlyAdmission => "NoConfuserOnlyAdmission",
            Self::NoCleanWindowOnlyAdmission => "NoCleanWindowOnlyAdmission",
            Self::NoBoundaryOnlyAdmission => "NoBoundaryOnlyAdmission",
            Self::NoRecoveryOnlyAdmission => "NoRecoveryOnlyAdmission",
            Self::BankAdmissionTokenRequired => "BankAdmissionTokenRequired",
            Self::GpuOutputIsEvidenceOnly => "GpuOutputIsEvidenceOnly",
            Self::DeferredUnknownIsExplicit => "DeferredUnknownIsExplicit",
        }
    }
}

/// Structured witness-count predicate per rule. The future T.11d
/// body verifier compares an episode's witness counts against
/// these fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WitnessRequirement {
    /// Minimum number of primary witnesses (default 0 means "no
    /// minimum imposed by this rule").
    pub min_primary: u32,
    /// Minimum number of corroborating positive witnesses.
    pub min_corroborating: u32,
    /// Minimum number of boundary witnesses.
    pub min_boundary: u32,
    /// Minimum number of recovery witnesses.
    pub min_recovery: u32,
    /// If true, the rule forbids confuser-witness-only
    /// admission. Load-bearing for SemanticNonBypass.
    pub forbids_confuser_only: bool,
    /// If true, the rule forbids clean-window-only admission.
    pub forbids_clean_window_only: bool,
    /// If true, the rule forbids boundary-only admission.
    pub forbids_boundary_only: bool,
    /// If true, the rule forbids recovery-only admission.
    pub forbids_recovery_only: bool,
    /// If true, the rule demands a valid `BankAdmissionToken`.
    pub requires_bank_admission_token: bool,
    /// If true, the rule asserts GPU output cannot mint
    /// admission (it is evidence only).
    pub gpu_output_is_evidence_only: bool,
}

impl WitnessRequirement {
    /// An empty requirement (no constraints declared). Used by
    /// the panel-required DeferredUnknown rule which carries no
    /// witness-count predicate.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            min_primary: 0,
            min_corroborating: 0,
            min_boundary: 0,
            min_recovery: 0,
            forbids_confuser_only: false,
            forbids_clean_window_only: false,
            forbids_boundary_only: false,
            forbids_recovery_only: false,
            requires_bank_admission_token: false,
            gpu_output_is_evidence_only: false,
        }
    }
}

/// One episode-admissibility rule.
#[derive(Debug, Clone)]
pub struct EpisodeAdmissibilityRule {
    /// Stable handle within the snapshot.
    pub id: GrammarRuleId,
    /// What kind of rule this is.
    pub kind: GrammarRuleKind,
    /// Panel-locked name (used as part of the hash material).
    pub name: &'static str,
    /// Witness-count predicate.
    pub witness_requirement: WitnessRequirement,
    /// Composed evidence requirements (AND of the listed
    /// requirements).
    pub evidence_requirements: Vec<EvidenceRequirement>,
    /// Court precedents this rule cites. Every rule MUST link at
    /// least one precedent; the verifier rejects an empty list.
    pub linked_precedent_ids: Vec<PrecedentId>,
    /// Severity (Hard / Soft / Deferred).
    pub severity: GrammarRuleSeverity,
    /// Auditor-facing note (rendered verbatim in the report).
    pub notes: &'static str,
}

/// Effect of a confuser-suppression rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfuserEffect {
    /// The negative-witness firing blocks any admission the bank
    /// would otherwise grant on the same evidence triple.
    BlockAdmission,
    /// The firing downgrades the admission (e.g. from Primary to
    /// Soft) but does not block outright.
    DowngradeAdmission,
    /// The firing quarantines the episode for review — the
    /// admission is held until the operator clears the confuser.
    QuarantineEpisode,
}

impl ConfuserEffect {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BlockAdmission => "BlockAdmission",
            Self::DowngradeAdmission => "DowngradeAdmission",
            Self::QuarantineEpisode => "QuarantineEpisode",
        }
    }
}

/// One confuser-suppression rule. Each named negative-witness
/// kind in [`NegativeWitnessKind`] (excluding `NotANegativeWitness`)
/// gets exactly one rule at T.11c.
#[derive(Debug, Clone)]
pub struct ConfuserSuppressionRule {
    /// Stable handle within the snapshot.
    pub id: GrammarRuleId,
    /// Which negative-witness kind triggers this suppression.
    pub trigger_kind: NegativeWitnessKind,
    /// What the rule does when the confuser fires.
    pub effect: ConfuserEffect,
    /// Cited T.11b precedents (must include at least one
    /// `NegativeWitnessLaw` or `WitnessLaw` precedent).
    pub linked_precedent_ids: Vec<PrecedentId>,
    /// Severity.
    pub severity: GrammarRuleSeverity,
    /// Auditor-facing note.
    pub notes: &'static str,
}

/// Newtype wrapper for the grammar snapshot hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GrammarSnapshotHash(pub [u8; 32]);

/// The full admissibility grammar.
#[derive(Debug, Clone)]
pub struct AdmissibilityGrammarSnapshot {
    /// Schema identifier (`ADMISSIBILITY_GRAMMAR_SCHEMA_V1`).
    pub schema: &'static str,
    /// Episode-admissibility rules in canonical-sorted order.
    pub admission_rules: Vec<EpisodeAdmissibilityRule>,
    /// Confuser-suppression rules in canonical-sorted order.
    pub confuser_rules: Vec<ConfuserSuppressionRule>,
    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of every rule. Two builds against the same
    /// precedent set produce the same hash.
    pub admissibility_grammar_hash_v1: GrammarSnapshotHash,
}

fn precedent_id_by_reason(
    precedents: &PrecedentSet,
    kind: PrecedentKind,
    reason: PrecedentReason,
) -> Option<PrecedentId> {
    precedents
        .precedents
        .iter()
        .find(|p| p.kind == kind && p.reason == reason)
        .map(|p| p.id)
}

/// All precedent ids of a given kind, in ascending order. Used by
/// the consumer-coverage tests in `tests/admissibility_invariants.rs`
/// to assert every `WitnessLaw` / `NegativeWitnessLaw` precedent is
/// consumed by at least one grammar rule.
#[must_use]
pub fn precedent_ids_by_kind(precedents: &PrecedentSet, kind: PrecedentKind) -> Vec<PrecedentId> {
    let mut out: Vec<PrecedentId> = precedents
        .precedents
        .iter()
        .filter(|p| p.kind == kind)
        .map(|p| p.id)
        .collect();
    out.sort_unstable();
    out
}

/// Collect the panel-locked admissibility grammar from the live
/// T.11b precedent set. Deterministic across two builds.
#[must_use]
pub fn collect_admissibility_grammar() -> AdmissibilityGrammarSnapshot {
    let precedents = collect_court_precedents();
    build_grammar(&precedents)
}

// Long but flat: nine admission-rule literals + nine
// confuser-rule literals + an id-assignment loop. Splitting it
// into multiple small builders would add no clarity; each rule
// stands alone, and the linear declaration order is what a
// reviewer wants to read.
#[allow(clippy::too_many_lines, clippy::vec_init_then_push)]
fn build_grammar(precedents: &PrecedentSet) -> AdmissibilityGrammarSnapshot {
    let primary_law = precedent_id_by_reason(
        precedents,
        PrecedentKind::WitnessLaw,
        PrecedentReason::PrimaryWitnessCannotBeNegativeOnly,
    );
    let clean_window_law = precedent_id_by_reason(
        precedents,
        PrecedentKind::WitnessLaw,
        PrecedentReason::CleanWindowWitnessCannotAdmitAlone,
    );
    let neg_law = precedent_id_by_reason(
        precedents,
        PrecedentKind::NegativeWitnessLaw,
        PrecedentReason::NegativeWitnessVetoesAdmission,
    );
    let constitution_passport = precedent_id_by_reason(
        precedents,
        PrecedentKind::ConstitutionLaw,
        PrecedentReason::EveryCanonicalRequiresPassport,
    );
    let constitution_flags = precedent_id_by_reason(
        precedents,
        PrecedentKind::ConstitutionLaw,
        PrecedentReason::EveryPassportRequiresAllConstitutionFlags,
    );
    let registry_binding = precedent_id_by_reason(
        precedents,
        PrecedentKind::RegistryBindingLaw,
        PrecedentReason::RegistryHashV2BindsToFrozenCorpusHash,
    );
    let deferred_casefile = precedent_id_by_reason(
        precedents,
        PrecedentKind::DeferredGateLaw,
        PrecedentReason::CaseFileV2BodyDeferredToT11d,
    );

    let only_some = |ids: &[Option<PrecedentId>]| -> Vec<PrecedentId> {
        let mut out: Vec<PrecedentId> = ids.iter().filter_map(|x| *x).collect();
        out.sort_unstable();
        out
    };

    let mut admission_rules: Vec<EpisodeAdmissibilityRule> = Vec::new();

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::EpisodeAdmission,
        name: "PrimaryWitnessRequiresPositiveSupport",
        witness_requirement: WitnessRequirement {
            min_primary: 1,
            min_corroborating: 1,
            forbids_confuser_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![
            EvidenceRequirement::AtLeastOnePrimaryWitness,
            EvidenceRequirement::AtLeastOneCorroboratingWitness,
            EvidenceRequirement::NoConfuserOnlyAdmission,
        ],
        linked_precedent_ids: only_some(&[primary_law, neg_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "A primary witness can admit only with at least one non-negative corroborating witness. Negative-witness-only firings cannot mint admissions.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::CleanWindowSupport,
        name: "CleanWindowWitnessCannotAdmitAlone",
        witness_requirement: WitnessRequirement {
            forbids_clean_window_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::NoCleanWindowOnlyAdmission],
        linked_precedent_ids: only_some(&[clean_window_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "A clean-window witness corroborates but never carries admission by itself.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::BoundaryCondition,
        name: "BoundaryWitnessCannotClassifyAlone",
        witness_requirement: WitnessRequirement {
            forbids_boundary_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::NoBoundaryOnlyAdmission],
        linked_precedent_ids: only_some(&[primary_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "A boundary witness can define start / end markers but cannot classify the episode by itself.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::RecoveryClosure,
        name: "RecoveryWitnessCannotOriginateAlone",
        witness_requirement: WitnessRequirement {
            forbids_recovery_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::NoRecoveryOnlyAdmission],
        linked_precedent_ids: only_some(&[primary_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "A recovery witness can close an episode but cannot originate one by itself.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::NegativeWitnessRejection,
        name: "NegativeWitnessBlocksAdmissionUnlessBankOverride",
        witness_requirement: WitnessRequirement {
            forbids_confuser_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::NoConfuserOnlyAdmission],
        linked_precedent_ids: only_some(&[neg_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "Negative witnesses block admission unless explicitly overridden by a bank rule. The override path itself MUST be a SemanticNonBypass-bound action.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::MinimumEvidence,
        name: "MinimumPrimaryWitnessEvidence",
        witness_requirement: WitnessRequirement {
            min_primary: 1,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::AtLeastOnePrimaryWitness],
        linked_precedent_ids: only_some(&[primary_law]),
        severity: GrammarRuleSeverity::Hard,
        notes: "Every admission MUST include at least one primary witness firing.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::SemanticNonBypass,
        name: "BankAdmissionTokenIsTheOnlyAdmissionRoute",
        witness_requirement: WitnessRequirement {
            requires_bank_admission_token: true,
            forbids_confuser_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![
            EvidenceRequirement::BankAdmissionTokenRequired,
            EvidenceRequirement::NoConfuserOnlyAdmission,
        ],
        linked_precedent_ids: only_some(&[
            primary_law,
            neg_law,
            constitution_passport,
            constitution_flags,
        ]),
        severity: GrammarRuleSeverity::Hard,
        notes: "The bank module's `BankAdmissionToken` constructor is the only path that mints admitted episodes. The Semantic Non-Bypass Axiom binds the corpus laws to the Atlas runtime.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::SemanticNonBypass,
        name: "GpuOutputIsEvidenceOnly",
        witness_requirement: WitnessRequirement {
            gpu_output_is_evidence_only: true,
            ..WitnessRequirement::none()
        },
        evidence_requirements: alloc::vec![EvidenceRequirement::GpuOutputIsEvidenceOnly],
        linked_precedent_ids: only_some(&[registry_binding, constitution_flags]),
        severity: GrammarRuleSeverity::Hard,
        notes: "GPU detector output is evidence only — it cannot mint an admission token. The bank stage remains the only authority.",
    });

    admission_rules.push(EpisodeAdmissibilityRule {
        id: GrammarRuleId(0),
        kind: GrammarRuleKind::DeferredUnknown,
        name: "UnknownOrDeferredOutcomeIsExplicit",
        witness_requirement: WitnessRequirement::none(),
        evidence_requirements: alloc::vec![EvidenceRequirement::DeferredUnknownIsExplicit],
        linked_precedent_ids: only_some(&[deferred_casefile]),
        severity: GrammarRuleSeverity::Deferred,
        notes: "Unknown / deferred episode-admission outcomes MUST be explicit grammar outputs rather than silent failures. The body verifier (T.11d) will emit a `DeferredUnknown` verdict rather than collapse to `NotAdmitted`.",
    });

    // Canonical sort + id assignment for admission rules.
    admission_rules.sort_by(|a, b| admission_sort_key(a).cmp(&admission_sort_key(b)));

    let mut confuser_rules: Vec<ConfuserSuppressionRule> = Vec::new();
    for trigger in [
        NegativeWitnessKind::SmallSampleConfuser,
        NegativeWitnessKind::SingleWindowSpikeConfuser,
        NegativeWitnessKind::PeriodicBoundaryConfuser,
        NegativeWitnessKind::MissingnessArtifactConfuser,
        NegativeWitnessKind::SchemaChangeConfuser,
        NegativeWitnessKind::UnitScaleChangeConfuser,
        NegativeWitnessKind::DeploymentMarkerConfuser,
        NegativeWitnessKind::ClockSkewConfuser,
        NegativeWitnessKind::BatchBoundaryConfuser,
    ] {
        confuser_rules.push(ConfuserSuppressionRule {
            id: GrammarRuleId(0),
            trigger_kind: trigger,
            effect: ConfuserEffect::BlockAdmission,
            linked_precedent_ids: only_some(&[neg_law]),
            severity: GrammarRuleSeverity::Hard,
            notes:
                "Negative-witness firing of this kind blocks any admission on the same evidence triple. The bank stage may override only via an explicit SemanticNonBypass-bound action.",
        });
    }
    confuser_rules.sort_by(|a, b| confuser_sort_key(a).cmp(&confuser_sort_key(b)));

    // Assign deterministic ids: admission rules first (1..A),
    // then confuser rules (A+1..A+C). Two builds produce the
    // same id for each rule.
    let mut next_id: u32 = 1;
    for r in &mut admission_rules {
        r.id = GrammarRuleId(next_id);
        next_id += 1;
    }
    for r in &mut confuser_rules {
        r.id = GrammarRuleId(next_id);
        next_id += 1;
    }

    let hash = compute_grammar_hash_raw(&admission_rules, &confuser_rules);

    AdmissibilityGrammarSnapshot {
        schema: ADMISSIBILITY_GRAMMAR_SCHEMA_V1,
        admission_rules,
        confuser_rules,
        admissibility_grammar_hash_v1: GrammarSnapshotHash(hash),
    }
}

fn admission_sort_key(r: &EpisodeAdmissibilityRule) -> (&'static str, &'static str) {
    (r.kind.as_str(), r.name)
}

fn confuser_sort_key(r: &ConfuserSuppressionRule) -> (&'static str, &'static str) {
    (r.trigger_kind.as_str(), r.effect.as_str())
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str_canon(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_witness_requirement(out: &mut Vec<u8>, w: WitnessRequirement) {
    write_u32(out, w.min_primary);
    write_u32(out, w.min_corroborating);
    write_u32(out, w.min_boundary);
    write_u32(out, w.min_recovery);
    out.push(u8::from(w.forbids_confuser_only));
    out.push(u8::from(w.forbids_clean_window_only));
    out.push(u8::from(w.forbids_boundary_only));
    out.push(u8::from(w.forbids_recovery_only));
    out.push(u8::from(w.requires_bank_admission_token));
    out.push(u8::from(w.gpu_output_is_evidence_only));
}

fn write_admission(out: &mut Vec<u8>, r: &EpisodeAdmissibilityRule) {
    write_u32(out, r.id.0);
    write_str_canon(out, r.kind.as_str());
    write_str_canon(out, r.name);
    write_witness_requirement(out, r.witness_requirement);
    write_u32(
        out,
        u32::try_from(r.evidence_requirements.len()).unwrap_or(u32::MAX),
    );
    let mut sorted: Vec<&str> = r.evidence_requirements.iter().map(|e| e.as_str()).collect();
    sorted.sort_unstable();
    for e in sorted {
        write_str_canon(out, e);
    }
    let mut ids: Vec<u32> = r.linked_precedent_ids.iter().map(|i| i.0).collect();
    ids.sort_unstable();
    write_u32(out, u32::try_from(ids.len()).unwrap_or(u32::MAX));
    for id in ids {
        write_u32(out, id);
    }
    write_str_canon(out, r.severity.as_str());
    write_str_canon(out, r.notes);
}

fn write_confuser(out: &mut Vec<u8>, r: &ConfuserSuppressionRule) {
    write_u32(out, r.id.0);
    write_str_canon(out, r.trigger_kind.as_str());
    write_str_canon(out, r.effect.as_str());
    let mut ids: Vec<u32> = r.linked_precedent_ids.iter().map(|i| i.0).collect();
    ids.sort_unstable();
    write_u32(out, u32::try_from(ids.len()).unwrap_or(u32::MAX));
    for id in ids {
        write_u32(out, id);
    }
    write_str_canon(out, r.severity.as_str());
    write_str_canon(out, r.notes);
}

fn compute_grammar_hash_raw(
    admission_rules: &[EpisodeAdmissibilityRule],
    confuser_rules: &[ConfuserSuppressionRule],
) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    buf.extend_from_slice(ADMISSIBILITY_GRAMMAR_DOMAIN.as_bytes());
    write_str_canon(&mut buf, ADMISSIBILITY_GRAMMAR_SCHEMA_V1);
    write_u32(
        &mut buf,
        u32::try_from(admission_rules.len()).unwrap_or(u32::MAX),
    );
    for r in admission_rules {
        write_admission(&mut buf, r);
    }
    write_u32(
        &mut buf,
        u32::try_from(confuser_rules.len()).unwrap_or(u32::MAX),
    );
    for r in confuser_rules {
        write_confuser(&mut buf, r);
    }
    sha256(&buf)
}

/// Compute the snapshot hash. Two calls on the same snapshot
/// produce byte-identical output.
#[must_use]
pub fn compute_admissibility_grammar_hash_v1(
    snapshot: &AdmissibilityGrammarSnapshot,
) -> GrammarSnapshotHash {
    GrammarSnapshotHash(compute_grammar_hash_raw(
        &snapshot.admission_rules,
        &snapshot.confuser_rules,
    ))
}

/// One verification failure on an `AdmissibilityGrammarSnapshot`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarVerifyError {
    /// Failing rule id (or `GrammarRuleId(0)` if set-level).
    pub id: GrammarRuleId,
    /// Structured failure kind.
    pub kind: GrammarVerifyErrorKind,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Structured grammar-verifier error category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarVerifyErrorKind {
    /// A rule carries an empty `linked_precedent_ids`. Every
    /// grammar rule MUST cite at least one T.11b precedent —
    /// this is the panel-locked "derived law layer" invariant.
    RuleWithoutPrecedentLink,
    /// An `EpisodeAdmission` rule does NOT carry the
    /// `NoConfuserOnlyAdmission` requirement. Load-bearing for
    /// the Semantic Non-Bypass Axiom — a rule that allowed
    /// confuser-only admission would violate the DSFB identity.
    EpisodeAdmissionAllowsConfuserOnly,
    /// A linked precedent id is not present in the live T.11b
    /// precedent set.
    LinkedPrecedentMissing,
    /// Two rules share an id (the collector reassigns ids after
    /// sort; duplicates indicate corruption).
    DuplicateRuleId,
}

/// Verify an `AdmissibilityGrammarSnapshot` against the T.11c
/// structural invariants. Returns the list of failures (empty
/// if clean).
#[must_use]
pub fn verify_grammar_snapshot(
    snapshot: &AdmissibilityGrammarSnapshot,
    precedents: &PrecedentSet,
) -> Vec<GrammarVerifyError> {
    let mut errors: Vec<GrammarVerifyError> = Vec::new();
    let mut seen_ids: Vec<u32> = Vec::new();
    let known_precedents: Vec<u32> = precedents.precedents.iter().map(|p| p.id.0).collect();

    let check_links = |id: GrammarRuleId,
                       links: &[PrecedentId],
                       errors: &mut Vec<GrammarVerifyError>| {
        if links.is_empty() {
            errors.push(GrammarVerifyError {
                id,
                kind: GrammarVerifyErrorKind::RuleWithoutPrecedentLink,
                message: format!(
                    "grammar rule {} has no linked precedents; every rule MUST cite at least one T.11b precedent",
                    id.0
                ),
            });
        }
        for link in links {
            if !known_precedents.contains(&link.0) {
                errors.push(GrammarVerifyError {
                    id,
                    kind: GrammarVerifyErrorKind::LinkedPrecedentMissing,
                    message: format!(
                        "grammar rule {} cites precedent id {} which is not in the live precedent set",
                        id.0, link.0
                    ),
                });
            }
        }
    };

    let mut check_id = |id: GrammarRuleId, errors: &mut Vec<GrammarVerifyError>| {
        if seen_ids.contains(&id.0) {
            errors.push(GrammarVerifyError {
                id,
                kind: GrammarVerifyErrorKind::DuplicateRuleId,
                message: format!("duplicate grammar rule id {}", id.0),
            });
        }
        seen_ids.push(id.0);
    };

    for r in &snapshot.admission_rules {
        check_id(r.id, &mut errors);
        check_links(r.id, &r.linked_precedent_ids, &mut errors);
        // Semantic-non-bypass invariant: every EpisodeAdmission
        // rule MUST forbid confuser-only admission, either via
        // the WitnessRequirement flag or via the
        // NoConfuserOnlyAdmission evidence-requirement clause.
        if r.kind == GrammarRuleKind::EpisodeAdmission {
            let forbids_via_witness = r.witness_requirement.forbids_confuser_only;
            let forbids_via_evidence = r
                .evidence_requirements
                .contains(&EvidenceRequirement::NoConfuserOnlyAdmission);
            if !forbids_via_witness && !forbids_via_evidence {
                errors.push(GrammarVerifyError {
                    id: r.id,
                    kind: GrammarVerifyErrorKind::EpisodeAdmissionAllowsConfuserOnly,
                    message: format!(
                        "EpisodeAdmission rule {} allows confuser-only admission; this violates the Semantic Non-Bypass Axiom",
                        r.id.0
                    ),
                });
            }
        }
    }
    for r in &snapshot.confuser_rules {
        check_id(r.id, &mut errors);
        check_links(r.id, &r.linked_precedent_ids, &mut errors);
    }

    errors
}

/// Return the set of grammar rule ids that consume a given
/// precedent id. Used by the passport → precedent → grammar
/// crosswalk.
#[must_use]
pub fn grammar_rules_consuming(
    snapshot: &AdmissibilityGrammarSnapshot,
    precedent_id: PrecedentId,
) -> Vec<GrammarRuleId> {
    let mut out: Vec<GrammarRuleId> = Vec::new();
    for r in &snapshot.admission_rules {
        if r.linked_precedent_ids.contains(&precedent_id) {
            out.push(r.id);
        }
    }
    for r in &snapshot.confuser_rules {
        if r.linked_precedent_ids.contains(&precedent_id) {
            out.push(r.id);
        }
    }
    out.sort_unstable();
    out
}

/// One row of the passport-precedent-grammar crosswalk.
#[derive(Debug, Clone)]
pub struct CrosswalkRow {
    /// Canonical id of the detector this row describes.
    pub canonical_id: DetectorCanonicalId,
    /// Display name (echoed for human readability).
    pub display_name: &'static str,
    /// Precedents the passport links to (in sorted order).
    pub linked_precedent_ids: Vec<PrecedentId>,
    /// Grammar rules that consume any of those precedents,
    /// deduplicated and sorted.
    pub linked_grammar_rule_ids: Vec<GrammarRuleId>,
}

/// Build the full passport → precedent → grammar crosswalk.
#[must_use]
pub fn build_crosswalk(snapshot: &AdmissibilityGrammarSnapshot) -> Vec<CrosswalkRow> {
    let passports = all_passports();
    let mut out: Vec<CrosswalkRow> = Vec::with_capacity(passports.len());
    for p in &passports {
        let mut grammar_ids: Vec<u32> = Vec::new();
        for pid in &p.linked_precedent_ids {
            for gid in grammar_rules_consuming(snapshot, *pid) {
                if !grammar_ids.contains(&gid.0) {
                    grammar_ids.push(gid.0);
                }
            }
        }
        grammar_ids.sort_unstable();
        out.push(CrosswalkRow {
            canonical_id: p.canonical_id,
            display_name: p.display_name,
            linked_precedent_ids: p.linked_precedent_ids.clone(),
            linked_grammar_rule_ids: grammar_ids.iter().map(|g| GrammarRuleId(*g)).collect(),
        });
    }
    out.sort_by_key(|r| r.canonical_id.0);
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

/// Render the snapshot as a deterministic human-readable text.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_grammar_text(snapshot: &AdmissibilityGrammarSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas — Admissibility Grammar Snapshot (T.11c)\n");
    out.push_str("================================================================\n");
    let _ = writeln!(out, "Schema    : {}", snapshot.schema);
    let _ = writeln!(
        out,
        "Hash (v1) : {}",
        hex_lower(&snapshot.admissibility_grammar_hash_v1.0)
    );
    let _ = writeln!(
        out,
        "Admission rules    : {}",
        snapshot.admission_rules.len()
    );
    let _ = writeln!(
        out,
        "Confuser-suppression rules: {}",
        snapshot.confuser_rules.len()
    );
    out.push('\n');
    out.push_str("Episode-admissibility rules:\n");
    out.push_str("----------------------------------------------------------------\n");
    for r in &snapshot.admission_rules {
        let _ = writeln!(out, "[{}] {} ({})", r.id.0, r.name, r.kind.as_str());
        let _ = writeln!(out, "    severity : {}", r.severity.as_str());
        let w = r.witness_requirement;
        let _ = writeln!(
            out,
            "    witness  : min_primary={}, min_corroborating={}, min_boundary={}, min_recovery={}",
            w.min_primary, w.min_corroborating, w.min_boundary, w.min_recovery
        );
        let mut flags: Vec<&'static str> = Vec::new();
        if w.forbids_confuser_only {
            flags.push("forbids_confuser_only");
        }
        if w.forbids_clean_window_only {
            flags.push("forbids_clean_window_only");
        }
        if w.forbids_boundary_only {
            flags.push("forbids_boundary_only");
        }
        if w.forbids_recovery_only {
            flags.push("forbids_recovery_only");
        }
        if w.requires_bank_admission_token {
            flags.push("requires_bank_admission_token");
        }
        if w.gpu_output_is_evidence_only {
            flags.push("gpu_output_is_evidence_only");
        }
        if !flags.is_empty() {
            let _ = writeln!(out, "    flags    : {}", flags.join(" | "));
        }
        out.push_str("    evidence : ");
        let mut ev: Vec<&'static str> =
            r.evidence_requirements.iter().map(|e| e.as_str()).collect();
        ev.sort_unstable();
        out.push_str(&ev.join(", "));
        out.push('\n');
        out.push_str("    cites    : ");
        let mut ids: Vec<u32> = r.linked_precedent_ids.iter().map(|i| i.0).collect();
        ids.sort_unstable();
        let id_strings: Vec<String> = ids.iter().map(|i| format!("{i}")).collect();
        out.push_str(&id_strings.join(", "));
        out.push('\n');
        if !r.notes.is_empty() {
            let _ = writeln!(out, "    notes    : {}", r.notes);
        }
        out.push('\n');
    }
    out.push_str("Confuser-suppression rules:\n");
    out.push_str("----------------------------------------------------------------\n");
    for r in &snapshot.confuser_rules {
        let _ = writeln!(
            out,
            "[{}] {} → {}",
            r.id.0,
            r.trigger_kind.as_str(),
            r.effect.as_str()
        );
        let _ = writeln!(out, "    severity : {}", r.severity.as_str());
        out.push_str("    cites    : ");
        let mut ids: Vec<u32> = r.linked_precedent_ids.iter().map(|i| i.0).collect();
        ids.sort_unstable();
        let id_strings: Vec<String> = ids.iter().map(|i| format!("{i}")).collect();
        out.push_str(&id_strings.join(", "));
        out.push('\n');
        if !r.notes.is_empty() {
            let _ = writeln!(out, "    notes    : {}", r.notes);
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

fn ids_json(ids: &[PrecedentId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::from("[");
    for (i, v) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{v}");
    }
    s.push(']');
    s
}

fn grammar_ids_json(ids: &[GrammarRuleId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::from("[");
    for (i, v) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{v}");
    }
    s.push(']');
    s
}

/// Render the snapshot as a deterministic JSON object.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_grammar_json(snapshot: &AdmissibilityGrammarSnapshot) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("{\n");
    out.push_str("  \"schema\": ");
    json_quote(&mut out, snapshot.schema);
    out.push_str(",\n");
    let _ = writeln!(
        out,
        "  \"admissibility_grammar_hash_v1\": \"{}\",",
        hex_lower(&snapshot.admissibility_grammar_hash_v1.0)
    );
    out.push_str("  \"admission_rules\": [\n");
    for (i, r) in snapshot.admission_rules.iter().enumerate() {
        out.push_str("    {\n");
        let _ = writeln!(out, "      \"id\": {},", r.id.0);
        out.push_str("      \"kind\": ");
        json_quote(&mut out, r.kind.as_str());
        out.push_str(",\n");
        out.push_str("      \"name\": ");
        json_quote(&mut out, r.name);
        out.push_str(",\n");
        let w = r.witness_requirement;
        out.push_str("      \"witness_requirement\": {\n");
        let _ = writeln!(out, "        \"min_primary\": {},", w.min_primary);
        let _ = writeln!(
            out,
            "        \"min_corroborating\": {},",
            w.min_corroborating
        );
        let _ = writeln!(out, "        \"min_boundary\": {},", w.min_boundary);
        let _ = writeln!(out, "        \"min_recovery\": {},", w.min_recovery);
        let _ = writeln!(
            out,
            "        \"forbids_confuser_only\": {},",
            w.forbids_confuser_only
        );
        let _ = writeln!(
            out,
            "        \"forbids_clean_window_only\": {},",
            w.forbids_clean_window_only
        );
        let _ = writeln!(
            out,
            "        \"forbids_boundary_only\": {},",
            w.forbids_boundary_only
        );
        let _ = writeln!(
            out,
            "        \"forbids_recovery_only\": {},",
            w.forbids_recovery_only
        );
        let _ = writeln!(
            out,
            "        \"requires_bank_admission_token\": {},",
            w.requires_bank_admission_token
        );
        let _ = writeln!(
            out,
            "        \"gpu_output_is_evidence_only\": {}",
            w.gpu_output_is_evidence_only
        );
        out.push_str("      },\n");
        out.push_str("      \"evidence_requirements\": [");
        let mut ev: Vec<&'static str> =
            r.evidence_requirements.iter().map(|e| e.as_str()).collect();
        ev.sort_unstable();
        for (j, e) in ev.iter().enumerate() {
            if j > 0 {
                out.push_str(", ");
            }
            json_quote(&mut out, e);
        }
        out.push_str("],\n");
        let _ = writeln!(
            out,
            "      \"linked_precedent_ids\": {},",
            ids_json(&r.linked_precedent_ids)
        );
        out.push_str("      \"severity\": ");
        json_quote(&mut out, r.severity.as_str());
        out.push_str(",\n");
        out.push_str("      \"notes\": ");
        json_quote(&mut out, r.notes);
        out.push('\n');
        if i + 1 < snapshot.admission_rules.len() {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ],\n");
    out.push_str("  \"confuser_rules\": [\n");
    for (i, r) in snapshot.confuser_rules.iter().enumerate() {
        out.push_str("    {\n");
        let _ = writeln!(out, "      \"id\": {},", r.id.0);
        out.push_str("      \"trigger_kind\": ");
        json_quote(&mut out, r.trigger_kind.as_str());
        out.push_str(",\n");
        out.push_str("      \"effect\": ");
        json_quote(&mut out, r.effect.as_str());
        out.push_str(",\n");
        let _ = writeln!(
            out,
            "      \"linked_precedent_ids\": {},",
            ids_json(&r.linked_precedent_ids)
        );
        out.push_str("      \"severity\": ");
        json_quote(&mut out, r.severity.as_str());
        out.push_str(",\n");
        out.push_str("      \"notes\": ");
        json_quote(&mut out, r.notes);
        out.push('\n');
        if i + 1 < snapshot.confuser_rules.len() {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Render the passport-precedent-grammar crosswalk as a
/// deterministic human-readable text packet.
#[must_use]
pub fn render_crosswalk_text(rows: &[CrosswalkRow]) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas — Passport → Precedent → Grammar Crosswalk (T.11c)\n");
    out.push_str("================================================================\n");
    let _ = writeln!(out, "Rows: {}", rows.len());
    out.push('\n');
    for row in rows {
        let _ = writeln!(
            out,
            "[{}] {} ({} precedents, {} grammar rules)",
            row.canonical_id.0,
            row.display_name,
            row.linked_precedent_ids.len(),
            row.linked_grammar_rule_ids.len()
        );
        out.push_str("    precedents : ");
        let mut pids: Vec<u32> = row.linked_precedent_ids.iter().map(|i| i.0).collect();
        pids.sort_unstable();
        let id_strings: Vec<String> = pids.iter().map(|i| format!("{i}")).collect();
        out.push_str(&id_strings.join(", "));
        out.push('\n');
        out.push_str("    grammar    : ");
        let mut gids: Vec<u32> = row.linked_grammar_rule_ids.iter().map(|i| i.0).collect();
        gids.sort_unstable();
        let id_strings: Vec<String> = gids.iter().map(|i| format!("{i}")).collect();
        out.push_str(&id_strings.join(", "));
        out.push('\n');
        out.push('\n');
    }
    out
}

/// Render the crosswalk as a deterministic JSON array.
#[must_use]
pub fn render_crosswalk_json(rows: &[CrosswalkRow]) -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("[\n");
    for (i, row) in rows.iter().enumerate() {
        out.push_str("  {\n");
        let _ = writeln!(out, "    \"canonical_id\": {},", row.canonical_id.0);
        out.push_str("    \"display_name\": ");
        json_quote(&mut out, row.display_name);
        out.push_str(",\n");
        let _ = writeln!(
            out,
            "    \"linked_precedent_ids\": {},",
            ids_json(&row.linked_precedent_ids)
        );
        let _ = writeln!(
            out,
            "    \"linked_grammar_rule_ids\": {}",
            grammar_ids_json(&row.linked_grammar_rule_ids)
        );
        if i + 1 < rows.len() {
            out.push_str("  },\n");
        } else {
            out.push_str("  }\n");
        }
    }
    out.push_str("]\n");
    out
}

/// True if the rule's kind / source is one of the panel-locked
/// WitnessLaw / NegativeWitnessLaw kinds — used by the consumer-
/// coverage tests.
#[must_use]
pub fn is_witness_law_precedent(p: &CourtPrecedent) -> bool {
    matches!(p.kind, PrecedentKind::WitnessLaw)
}

/// True if the rule's kind is `NegativeWitnessLaw`.
#[must_use]
pub fn is_negative_witness_law_precedent(p: &CourtPrecedent) -> bool {
    matches!(p.kind, PrecedentKind::NegativeWitnessLaw)
}
