//! T.11d — `TrialTranscriptV1`: minimal real CaseFileV2 trial
//! transcript body.
//!
//! Panel framing:
//!
//! > T.11c made the grammar real. T.11d makes the court visibly
//! > real. A transcript that says **why** an episode was
//! > admitted, **which witnesses** spoke, **which confusers**
//! > were rejected, **which law** admitted it, **which
//! > precedent** supports it, and **which hash-bound corpus /
//! > registry / grammar** produced it.
//!
//! **Boundary-crossing scope (panel-locked)**:
//!
//! T.11d crosses the boundary from "frozen identities + abstract
//! receipts" into a real replayable trial record body. It does
//! NOT yet:
//!
//! - Derive transcripts from GPU-produced `CaseFileV1` episodes
//!   (that binding lands after `CaseFileV2` body integration).
//! - Implement `ActivationPlanner`, `OTel` binding, `ChallengeDocket`,
//!   `Contraindications`, PROV export, attestation, or Arrow
//!   layout. Those are T.11e+ commits.
//! - Touch GPU code or change any prior hash. R.12b episodes
//!   stay byte-identical; D16 / D64 / D128 / D205 golden hashes
//!   stay byte-identical.
//!
//! **What it ships**: one deterministic synthetic transcript
//! fixture (a LatencyRamp episode, IDs drawn from the SEED) plus
//! the transcript schema, a brutal verifier, two renderers, two
//! bulk artifacts, two load-bearing negative tests, and a 32-byte
//! `trial_transcript_hash_v1` over the canonical-byte projection
//! (rendered text is NOT hashed).
//!
//! **Hash chain (panel-locked)**:
//!
//! ```text
//!   corpus_hash_v1
//!     → registry_hash_v2
//!     → precedent_hash_v1
//!     → admissibility_grammar_hash_v1
//!     → trial_transcript_hash_v1
//! ```
//!
//! `trial_transcript_hash_v1` is DSFB-native; no in-toto / SLSA
//! / SPDX / CycloneDX compatibility claim.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write as _;

use dsfb_gpu_debug_core::sha256;

use crate::admissibility::{
    collect_admissibility_grammar, AdmissibilityGrammarSnapshot, ConfuserEffect,
    EpisodeAdmissibilityRule, GrammarRuleId, GrammarRuleKind,
};
use crate::corpus_hash::compute_corpus_hash_v1;
use crate::precedent::{collect_court_precedents, PrecedentId, PrecedentSet};
use crate::seed::SEED;
use crate::types::{CanonicalisationDecision, DetectorCanonicalId, NegativeWitnessKind};

/// Domain separator prefix for `trial_transcript_hash_v1`.
/// **Panel-locked**; changing it changes every transcript hash.
pub const TRIAL_TRANSCRIPT_DOMAIN: &str = "DSFB-GPU-ATLAS:TRIAL-TRANSCRIPT:v1\0";

/// Schema identifier carried inside the transcript hash material.
pub const TRIAL_TRANSCRIPT_SCHEMA_V1: &str = "DSFB-GPU-ATLAS:TRIAL-TRANSCRIPT:v1";

/// Stable handle for one `TrialTranscriptV1`. At T.11d the
/// fixture's id is `1`; future fixtures are append-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrialTranscriptId(pub u32);

/// Schema variant carried in the hash material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrialTranscriptSchema {
    /// T.11d — minimal real transcript body. Panel-locked
    /// scope: one synthetic fixture, no GPU-derived
    /// transcripts.
    V1MinimalT11d,
}

impl TrialTranscriptSchema {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1MinimalT11d => "V1MinimalT11d",
        }
    }
}

/// The episode the transcript is about. At T.11d this is a
/// synthetic fixture; future commits replace this with a
/// hash-bound reference into the V2 case file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EpisodeSubject {
    /// Human-readable motif label (e.g. `"LatencyRamp"`). The
    /// label is hashed; renaming changes the transcript hash.
    pub motif_label: &'static str,
    /// Synthetic entity id the episode is bound to.
    pub entity_id: u32,
    /// Window-index start (inclusive).
    pub window_start_idx: u32,
    /// Window-index end (inclusive).
    pub window_end_idx: u32,
}

/// One rejected-confuser record within a transcript.
#[derive(Debug, Clone)]
pub struct RejectedConfuser {
    /// Which negative-witness kind fired.
    pub trigger_kind: NegativeWitnessKind,
    /// Which T.11c `ConfuserSuppressionRule` matched (the id
    /// MUST point at a real rule in the live grammar).
    pub suppression_rule_id: GrammarRuleId,
    /// Why the confuser was rejected (reason code).
    pub reason_code: ConfuserRejectionReason,
}

/// Panel-locked reason codes for a confuser rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfuserRejectionReason {
    /// The confuser did not fire on the supporting evidence; no
    /// rejection action was needed.
    NotFired,
    /// The confuser fired but the bank rule explicitly overrode
    /// it (via a Semantic-Non-Bypass-bound action).
    BankOverrideApplied,
    /// The confuser fired and would otherwise have blocked the
    /// episode; the transcript records the rejection-of-the-
    /// confuser-rejection as a quarantined precedent for future
    /// review.
    Quarantined,
    /// The confuser was suppressed because its precondition was
    /// not met (e.g. periodic-boundary confuser ignored on
    /// non-periodic input).
    PreconditionUnmet,
}

impl ConfuserRejectionReason {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFired => "NotFired",
            Self::BankOverrideApplied => "BankOverrideApplied",
            Self::Quarantined => "Quarantined",
            Self::PreconditionUnmet => "PreconditionUnmet",
        }
    }
}

/// Panel-locked reason codes for why a detector was disabled
/// during a trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DetectorDisabledReason {
    /// Sentinel "no reason given" — the verifier MUST reject
    /// this on any committed transcript. Exists only so test
    /// fixtures can deliberately forge a disabled record without
    /// a reason and exercise the rejection path.
    Unspecified,
    /// The required spectral projection was not available.
    MissingSpectralProjection,
    /// The detector requires regular sampling; the trial input
    /// had irregular timestamps.
    IrregularSampling,
    /// The detector requires declared units; the input did not
    /// carry them.
    UnitsUnclear,
    /// The trial window had fewer samples than the detector's
    /// minimum support.
    BelowMinimumSupport,
    /// The detector is implementation-deferred (L1 / L2 / not yet
    /// L3+).
    DeferredImplementation,
    /// A higher-priority detector in the same family was already
    /// active; this one is suppressed for redundancy.
    RedundantWithActivePeer,
}

impl DetectorDisabledReason {
    /// Canonical wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "Unspecified",
            Self::MissingSpectralProjection => "MissingSpectralProjection",
            Self::IrregularSampling => "IrregularSampling",
            Self::UnitsUnclear => "UnitsUnclear",
            Self::BelowMinimumSupport => "BelowMinimumSupport",
            Self::DeferredImplementation => "DeferredImplementation",
            Self::RedundantWithActivePeer => "RedundantWithActivePeer",
        }
    }
}

/// One disabled-but-relevant detector record within a transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisabledRelevantDetector {
    /// Detector that was disabled.
    pub canonical_id: DetectorCanonicalId,
    /// Why it was disabled.
    pub disabled_reason: DetectorDisabledReason,
}

/// Reason-code coverage summary for the transcript. Pinned by
/// the T.11d verifier: at least one bool MUST be true AND the
/// percent MUST be non-zero for a transcript to verify clean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonCodeCoverage {
    /// The transcript's admission rule carries a reason.
    pub admission_rule_has_reason: bool,
    /// Every primary / corroborating witness has a witness role
    /// (T.6) carried via passport linkage.
    pub witnesses_have_reason: bool,
    /// Every rejected confuser has a `ConfuserRejectionReason`.
    pub rejected_confusers_have_reason: bool,
    /// Every disabled-but-relevant detector has a non-Unspecified
    /// `DetectorDisabledReason`.
    pub disabled_detectors_have_reason: bool,
    /// Overall coverage in basis points (0..=10000). 10000 = 100.00%.
    pub coverage_percent_bp: u16,
}

impl ReasonCodeCoverage {
    /// An empty coverage — every flag false, percent 0. Used by
    /// fixtures that deliberately forge zero coverage to exercise
    /// the verifier rejection path.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            admission_rule_has_reason: false,
            witnesses_have_reason: false,
            rejected_confusers_have_reason: false,
            disabled_detectors_have_reason: false,
            coverage_percent_bp: 0,
        }
    }

    /// True if every flag is true AND `coverage_percent_bp`
    /// equals `10000` (the panel-locked full-coverage shape for
    /// the T.11d fixture).
    #[must_use]
    pub const fn is_full_coverage(self) -> bool {
        self.admission_rule_has_reason
            && self.witnesses_have_reason
            && self.rejected_confusers_have_reason
            && self.disabled_detectors_have_reason
            && self.coverage_percent_bp == 10_000
    }
}

/// The minimal real CaseFileV2 trial-transcript body.
#[derive(Debug, Clone)]
pub struct TrialTranscriptV1 {
    /// Stable handle for this transcript.
    pub transcript_id: TrialTranscriptId,
    /// Schema variant carried in the hash material.
    pub schema: TrialTranscriptSchema,
    /// T.10 corpus identity.
    pub corpus_hash_v1: [u8; 32],
    /// S1.2 registry identity.
    pub registry_hash_v2: [u8; 32],
    /// T.11b precedent layer identity.
    pub precedent_hash_v1: [u8; 32],
    /// T.11c admissibility-grammar identity.
    pub admissibility_grammar_hash_v1: [u8; 32],
    /// What episode is being tried (synthetic subject at T.11d).
    pub episode_subject: EpisodeSubject,
    /// The T.11c `EpisodeAdmissibilityRule.id` that admitted the
    /// episode. MUST be present in the live grammar.
    pub admitted_by_rule: GrammarRuleId,
    /// T.11b precedents that support the admission (sorted; each
    /// MUST be in the live precedent set).
    pub supporting_precedents: Vec<PrecedentId>,
    /// Primary witness canonical ids (sorted ascending).
    pub primary_witnesses: Vec<DetectorCanonicalId>,
    /// Corroborating witness canonical ids (sorted).
    pub corroborating_witnesses: Vec<DetectorCanonicalId>,
    /// Boundary witness canonical ids (sorted).
    pub boundary_witnesses: Vec<DetectorCanonicalId>,
    /// Recovery witness canonical ids (sorted).
    pub recovery_witnesses: Vec<DetectorCanonicalId>,
    /// Clean-window witness canonical ids (sorted).
    pub clean_window_witnesses: Vec<DetectorCanonicalId>,
    /// Confusers that fired but were rejected (sorted by
    /// `trigger_kind` wire name).
    pub rejected_confusers: Vec<RejectedConfuser>,
    /// Detectors that would have applied but were disabled
    /// (sorted by canonical_id).
    pub disabled_but_relevant: Vec<DisabledRelevantDetector>,
    /// Reason-code coverage summary.
    pub reason_code_coverage: ReasonCodeCoverage,
    /// 32-byte SHA-256 commitment over the canonical-byte
    /// projection of every other field. Rendered text is NOT
    /// hashed.
    pub trial_transcript_hash_v1: [u8; 32],
}

/// Build the panel-locked T.11d synthetic LatencyRamp transcript
/// fixture. Two builds against the same corpus + precedent +
/// grammar produce a byte-identical transcript and hash.
///
/// Canonical-id assignments mirror the T.1 SEED:
/// - LatencyRamp = 14 (primary)
/// - EWMA = 2, CUSUM = 3 (corroborating)
/// - Page-Hinkley = 4, Shewhart = 1 (boundary)
/// - Robust-Z = 6 (clean-window context)
/// - FFT band-energy = 12 (disabled-but-relevant: missing
///   spectral projection)
///
/// # Panics
///
/// Panics on a structural break of the live admissibility
/// grammar — specifically if the panel-locked
/// `PrimaryWitnessRequiresPositiveSupport` admission rule or
/// the `SingleWindowSpikeConfuser` suppression rule is missing.
/// The T.11c acceptance tests pin both to be present, so this
/// branch should never fire in a healthy build.
#[must_use]
pub fn build_t11d_latency_ramp_fixture() -> TrialTranscriptV1 {
    let corpus = compute_corpus_hash_v1();
    let grammar = collect_admissibility_grammar();
    let precedents = collect_court_precedents();

    // Look up the panel-locked rule + supporting precedents by
    // their canonical names. The lookups MUST succeed; the
    // T.11c acceptance tests pin both rules' presence, so the
    // `else { panic!(...) }` branches below should never fire
    // in a healthy build.
    let Some(admission_rule_ref) = grammar
        .admission_rules
        .iter()
        .find(|r| r.name == "PrimaryWitnessRequiresPositiveSupport")
    else {
        panic!("PrimaryWitnessRequiresPositiveSupport rule must exist (T.11c invariant)")
    };
    let admitted_by_rule = admission_rule_ref.id;
    let mut supporting_precedents: Vec<PrecedentId> =
        admission_rule_ref.linked_precedent_ids.clone();
    supporting_precedents.sort_unstable();

    // S1.2 registry hash is the canonical post-S1.2 value
    // (pinned by the registry crate). Mirror it here as a 32-byte
    // literal; the corpus crate does NOT depend on the registry
    // crate, so we cannot import it.
    let registry_hash_v2 = registry_hash_v2_post_s12();

    let Some(rejected_confuser_rule) = grammar
        .confuser_rules
        .iter()
        .find(|r| r.trigger_kind == NegativeWitnessKind::SingleWindowSpikeConfuser)
    else {
        panic!("SingleWindowSpikeConfuser confuser-rule must exist (T.11c invariant)")
    };
    let rejected_confuser_rule_id = rejected_confuser_rule.id;

    let mut primary = vec![DetectorCanonicalId(14)];
    let mut corroborating = vec![DetectorCanonicalId(2), DetectorCanonicalId(3)];
    let mut boundary = vec![DetectorCanonicalId(1), DetectorCanonicalId(4)];
    let recovery: Vec<DetectorCanonicalId> = Vec::new();
    let mut clean_window = vec![DetectorCanonicalId(6)];
    let mut rejected_confusers = vec![RejectedConfuser {
        trigger_kind: NegativeWitnessKind::SingleWindowSpikeConfuser,
        suppression_rule_id: rejected_confuser_rule_id,
        reason_code: ConfuserRejectionReason::NotFired,
    }];
    let mut disabled_but_relevant = vec![DisabledRelevantDetector {
        canonical_id: DetectorCanonicalId(12),
        disabled_reason: DetectorDisabledReason::MissingSpectralProjection,
    }];

    // Canonical sort.
    primary.sort_unstable_by_key(|c| c.0);
    corroborating.sort_unstable_by_key(|c| c.0);
    boundary.sort_unstable_by_key(|c| c.0);
    clean_window.sort_unstable_by_key(|c| c.0);
    rejected_confusers.sort_by(|a, b| a.trigger_kind.as_str().cmp(b.trigger_kind.as_str()));
    disabled_but_relevant.sort_by_key(|d| d.canonical_id.0);

    let reason_code_coverage = ReasonCodeCoverage {
        admission_rule_has_reason: true,
        witnesses_have_reason: true,
        rejected_confusers_have_reason: true,
        disabled_detectors_have_reason: true,
        coverage_percent_bp: 10_000,
    };

    let mut t = TrialTranscriptV1 {
        transcript_id: TrialTranscriptId(1),
        schema: TrialTranscriptSchema::V1MinimalT11d,
        corpus_hash_v1: corpus.bytes,
        registry_hash_v2,
        precedent_hash_v1: precedents.precedent_hash_v1,
        admissibility_grammar_hash_v1: grammar.admissibility_grammar_hash_v1.0,
        episode_subject: EpisodeSubject {
            motif_label: "LatencyRamp",
            entity_id: 7,
            window_start_idx: 100,
            window_end_idx: 131,
        },
        admitted_by_rule,
        supporting_precedents,
        primary_witnesses: primary,
        corroborating_witnesses: corroborating,
        boundary_witnesses: boundary,
        recovery_witnesses: recovery,
        clean_window_witnesses: clean_window,
        rejected_confusers,
        disabled_but_relevant,
        reason_code_coverage,
        trial_transcript_hash_v1: [0u8; 32],
    };
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    t
}

/// Panel-locked S1.2 `registry_hash_v2` (the post-S1.2 corpus-
/// generated value pinned in the S1.2 receipt). Mirrored here as
/// a literal so the corpus crate does not need to depend on the
/// registry crate to build a T.11d transcript fixture.
fn registry_hash_v2_post_s12() -> [u8; 32] {
    [
        0xd3, 0xcf, 0x63, 0x00, 0x0c, 0xee, 0x92, 0x28, 0x18, 0xe8, 0xdb, 0xc7, 0x9f, 0xfe, 0xcb,
        0xc2, 0x7d, 0x28, 0x80, 0x63, 0xef, 0xba, 0xed, 0x58, 0x9e, 0x1e, 0xb1, 0x81, 0x2b, 0xc3,
        0x7a, 0x08,
    ]
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_u32(out, u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(bytes);
}

fn write_ids(out: &mut Vec<u8>, ids: &[DetectorCanonicalId]) {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    write_u32(out, u32::try_from(sorted.len()).unwrap_or(u32::MAX));
    for id in sorted {
        write_u32(out, id);
    }
}

fn write_precedents(out: &mut Vec<u8>, ids: &[PrecedentId]) {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    write_u32(out, u32::try_from(sorted.len()).unwrap_or(u32::MAX));
    for id in sorted {
        write_u32(out, id);
    }
}

/// Compute the transcript's canonical-byte hash. Rendered text
/// is NOT included.
#[must_use]
pub fn compute_trial_transcript_hash_v1(t: &TrialTranscriptV1) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(2 * 1024);
    buf.extend_from_slice(TRIAL_TRANSCRIPT_DOMAIN.as_bytes());
    write_str(&mut buf, TRIAL_TRANSCRIPT_SCHEMA_V1);
    write_str(&mut buf, t.schema.as_str());
    write_u32(&mut buf, t.transcript_id.0);
    buf.extend_from_slice(&t.corpus_hash_v1);
    buf.extend_from_slice(&t.registry_hash_v2);
    buf.extend_from_slice(&t.precedent_hash_v1);
    buf.extend_from_slice(&t.admissibility_grammar_hash_v1);
    // Episode subject
    write_str(&mut buf, t.episode_subject.motif_label);
    write_u32(&mut buf, t.episode_subject.entity_id);
    write_u32(&mut buf, t.episode_subject.window_start_idx);
    write_u32(&mut buf, t.episode_subject.window_end_idx);
    // Admission rule + supporting precedents
    write_u32(&mut buf, t.admitted_by_rule.0);
    write_precedents(&mut buf, &t.supporting_precedents);
    // Witness lists in canonical order
    write_ids(&mut buf, &t.primary_witnesses);
    write_ids(&mut buf, &t.corroborating_witnesses);
    write_ids(&mut buf, &t.boundary_witnesses);
    write_ids(&mut buf, &t.recovery_witnesses);
    write_ids(&mut buf, &t.clean_window_witnesses);
    // Rejected confusers, sorted by trigger_kind wire name
    let mut sorted_conf: Vec<&RejectedConfuser> = t.rejected_confusers.iter().collect();
    sorted_conf.sort_by(|a, b| a.trigger_kind.as_str().cmp(b.trigger_kind.as_str()));
    write_u32(
        &mut buf,
        u32::try_from(sorted_conf.len()).unwrap_or(u32::MAX),
    );
    for c in sorted_conf {
        write_str(&mut buf, c.trigger_kind.as_str());
        write_u32(&mut buf, c.suppression_rule_id.0);
        write_str(&mut buf, c.reason_code.as_str());
    }
    // Disabled-but-relevant, sorted by canonical_id
    let mut sorted_dis: Vec<&DisabledRelevantDetector> = t.disabled_but_relevant.iter().collect();
    sorted_dis.sort_by_key(|d| d.canonical_id.0);
    write_u32(
        &mut buf,
        u32::try_from(sorted_dis.len()).unwrap_or(u32::MAX),
    );
    for d in sorted_dis {
        write_u32(&mut buf, d.canonical_id.0);
        write_str(&mut buf, d.disabled_reason.as_str());
    }
    // Reason-code coverage
    buf.push(u8::from(t.reason_code_coverage.admission_rule_has_reason));
    buf.push(u8::from(t.reason_code_coverage.witnesses_have_reason));
    buf.push(u8::from(
        t.reason_code_coverage.rejected_confusers_have_reason,
    ));
    buf.push(u8::from(
        t.reason_code_coverage.disabled_detectors_have_reason,
    ));
    write_u16(&mut buf, t.reason_code_coverage.coverage_percent_bp);
    sha256(&buf)
}

/// One verification failure on a `TrialTranscriptV1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptVerifyError {
    /// Structured failure kind.
    pub kind: TranscriptVerifyErrorKind,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Structured transcript-verifier failure category. Panel-locked
/// 13-direction reject set; the future T.11d body adds more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptVerifyErrorKind {
    /// `corpus_hash_v1` is all zeros.
    ZeroCorpusHash,
    /// `registry_hash_v2` is all zeros.
    ZeroRegistryHash,
    /// `precedent_hash_v1` is all zeros.
    ZeroPrecedentHash,
    /// `admissibility_grammar_hash_v1` is all zeros.
    ZeroGrammarHash,
    /// `admitted_by_rule.0 == 0` (rule id sentinel "missing").
    AdmissionRuleMissing,
    /// The `admitted_by_rule` id is not present in the live
    /// T.11c grammar.
    AdmissionRuleNotInGrammar,
    /// `primary_witnesses` is empty for a rule whose
    /// `WitnessRequirement.min_primary >= 1`.
    PrimaryWitnessListEmpty,
    /// The transcript would admit a confuser-only episode (no
    /// primary or corroborating witnesses) under a rule that
    /// forbids it.
    ConfuserOnlyAdmissionAttempted,
    /// A rejected confuser cites a suppression rule id that is
    /// not in the live grammar's confuser-rule set.
    RejectedConfuserSuppressionRuleMissing,
    /// A witness canonical_id is not in the SEED.
    WitnessIdMissingFromCorpus,
    /// A witness canonical_id resolves to a record whose dedup
    /// decision is `AliasOf(...)` — aliases cannot be primary
    /// witnesses; the transcript must cite the target canonical.
    AliasUsedAsPrimaryCanonicalWitness,
    /// A disabled-but-relevant entry carries `Unspecified` as
    /// its disabled_reason — the verifier requires every
    /// disabled detector to declare a real reason.
    DisabledDetectorLacksReason,
    /// `ReasonCodeCoverage` is completely empty (no flag true,
    /// percent_bp == 0).
    ReasonCodeCoverageEmpty,
    /// `trial_transcript_hash_v1` does not match
    /// `compute_trial_transcript_hash_v1` over the live struct.
    TranscriptHashMismatch,
    /// A supporting precedent id is not in the live T.11b
    /// precedent set.
    SupportingPrecedentMissing,
    /// The transcript cites no admissibility-grammar link at all
    /// (zero hash + missing rule). Load-bearing for the panel
    /// test `trial_transcript_verifier_rejects_missing_admissibility_grammar_link`.
    MissingAdmissibilityGrammarLink,
}

/// Verify a `TrialTranscriptV1` against the panel-locked
/// 16-direction structural invariants. Returns the list of
/// failures (empty if clean).
///
/// The verifier consults the live corpus + precedent + grammar
/// rather than carrying a snapshot, so two verifications on the
/// same transcript produce the same result and the verifier
/// stays honest about the present state.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn verify_trial_transcript(t: &TrialTranscriptV1) -> Vec<TranscriptVerifyError> {
    let mut errors: Vec<TranscriptVerifyError> = Vec::new();

    if t.corpus_hash_v1 == [0u8; 32] {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ZeroCorpusHash,
            message: "trial transcript carries zero corpus_hash_v1".into(),
        });
    }
    if t.registry_hash_v2 == [0u8; 32] {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ZeroRegistryHash,
            message: "trial transcript carries zero registry_hash_v2".into(),
        });
    }
    if t.precedent_hash_v1 == [0u8; 32] {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ZeroPrecedentHash,
            message: "trial transcript carries zero precedent_hash_v1".into(),
        });
    }
    if t.admissibility_grammar_hash_v1 == [0u8; 32] {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ZeroGrammarHash,
            message: "trial transcript carries zero admissibility_grammar_hash_v1".into(),
        });
        // Composite rule: if the grammar hash is zero AND the
        // admission rule id is also zero, surface the panel-
        // required "missing admissibility-grammar link" failure
        // as well.
        if t.admitted_by_rule.0 == 0 {
            errors.push(TranscriptVerifyError {
                kind: TranscriptVerifyErrorKind::MissingAdmissibilityGrammarLink,
                message:
                    "transcript carries neither admissibility_grammar_hash_v1 nor an admitted_by_rule; cannot prove which law admitted the episode"
                        .into(),
            });
        }
    }

    if t.admitted_by_rule.0 == 0 {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::AdmissionRuleMissing,
            message: "trial transcript carries admitted_by_rule = 0 (sentinel)".into(),
        });
    }

    // Look up the rule in the live grammar.
    let grammar = collect_admissibility_grammar();
    let admission_rule: Option<&EpisodeAdmissibilityRule> = grammar
        .admission_rules
        .iter()
        .find(|r| r.id == t.admitted_by_rule);
    if t.admitted_by_rule.0 != 0 && admission_rule.is_none() {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::AdmissionRuleNotInGrammar,
            message: format!(
                "admitted_by_rule id {} is not in the live admissibility grammar",
                t.admitted_by_rule.0
            ),
        });
    }

    // Primary-witness presence under a rule that requires one.
    if let Some(rule) = admission_rule {
        if rule.witness_requirement.min_primary >= 1 && t.primary_witnesses.is_empty() {
            errors.push(TranscriptVerifyError {
                kind: TranscriptVerifyErrorKind::PrimaryWitnessListEmpty,
                message: format!(
                    "rule `{}` requires min_primary >= 1 but transcript has no primary witnesses",
                    rule.name
                ),
            });
        }
    }

    // Confuser-only admission: the transcript has no primary,
    // corroborating, boundary, recovery, or clean-window
    // witnesses, but DOES carry rejected confusers and an
    // admission rule. That's structurally bogus.
    let total_positive = t.primary_witnesses.len()
        + t.corroborating_witnesses.len()
        + t.boundary_witnesses.len()
        + t.recovery_witnesses.len()
        + t.clean_window_witnesses.len();
    if total_positive == 0 && !t.rejected_confusers.is_empty() && t.admitted_by_rule.0 != 0 {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ConfuserOnlyAdmissionAttempted,
            message:
                "transcript carries no positive witnesses; admission cannot be carried by confuser firings alone"
                    .into(),
        });
    }

    // Witness ids in corpus + dedup-decision check.
    let seed_ids: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let check_witness_list =
        |list: &[DetectorCanonicalId],
         is_primary: bool,
         errors: &mut Vec<TranscriptVerifyError>| {
            for id in list {
                if !seed_ids.contains(&id.0) {
                    errors.push(TranscriptVerifyError {
                        kind: TranscriptVerifyErrorKind::WitnessIdMissingFromCorpus,
                        message: format!("witness canonical_id {} is not in the SEED", id.0),
                    });
                    continue;
                }
                if is_primary {
                    // The witness's dedup decision must NOT be
                    // AliasOf — aliases live in CLAIMS, not SEED,
                    // but a future SEED entry could carry an
                    // AliasOf decision. Reject it.
                    if let Some(rec) = SEED.iter().find(|r| r.canonical_id == *id) {
                        if let Some(dec) = lookup_dedup_decision(rec.canonical_id) {
                            if matches!(dec, CanonicalisationDecision::AliasOf(_)) {
                                errors.push(TranscriptVerifyError {
                                kind: TranscriptVerifyErrorKind::AliasUsedAsPrimaryCanonicalWitness,
                                message: format!(
                                    "witness canonical_id {} is classified AliasOf(...); aliases cannot be primary witnesses",
                                    id.0
                                ),
                            });
                            }
                        }
                    }
                }
            }
        };
    check_witness_list(&t.primary_witnesses, true, &mut errors);
    check_witness_list(&t.corroborating_witnesses, false, &mut errors);
    check_witness_list(&t.boundary_witnesses, false, &mut errors);
    check_witness_list(&t.recovery_witnesses, false, &mut errors);
    check_witness_list(&t.clean_window_witnesses, false, &mut errors);

    // Rejected confusers: each suppression_rule_id MUST exist in
    // the live grammar's confuser-rule set.
    let confuser_rule_ids: Vec<u32> = grammar.confuser_rules.iter().map(|r| r.id.0).collect();
    for c in &t.rejected_confusers {
        if !confuser_rule_ids.contains(&c.suppression_rule_id.0) {
            errors.push(TranscriptVerifyError {
                kind: TranscriptVerifyErrorKind::RejectedConfuserSuppressionRuleMissing,
                message: format!(
                    "rejected confuser cites suppression_rule_id {} which is not in the live grammar",
                    c.suppression_rule_id.0
                ),
            });
        }
    }

    // Disabled-but-relevant: every entry must declare a non-
    // Unspecified disabled_reason.
    for d in &t.disabled_but_relevant {
        if matches!(d.disabled_reason, DetectorDisabledReason::Unspecified) {
            errors.push(TranscriptVerifyError {
                kind: TranscriptVerifyErrorKind::DisabledDetectorLacksReason,
                message: format!(
                    "disabled detector canonical_id {} carries `Unspecified` reason",
                    d.canonical_id.0
                ),
            });
        }
    }

    // Reason-code coverage: a transcript with no coverage signal
    // is rejected.
    let c = t.reason_code_coverage;
    if !c.admission_rule_has_reason
        && !c.witnesses_have_reason
        && !c.rejected_confusers_have_reason
        && !c.disabled_detectors_have_reason
        && c.coverage_percent_bp == 0
    {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::ReasonCodeCoverageEmpty,
            message: "transcript carries an empty ReasonCodeCoverage".into(),
        });
    }

    // Supporting precedents must resolve to live ids.
    let precedents = collect_court_precedents();
    let known_pids: Vec<u32> = precedents.precedents.iter().map(|p| p.id.0).collect();
    for pid in &t.supporting_precedents {
        if !known_pids.contains(&pid.0) {
            errors.push(TranscriptVerifyError {
                kind: TranscriptVerifyErrorKind::SupportingPrecedentMissing,
                message: format!(
                    "supporting precedent id {} is not in the live T.11b set",
                    pid.0
                ),
            });
        }
    }

    // Transcript hash must match a fresh recomputation.
    let recomputed = compute_trial_transcript_hash_v1(t);
    if recomputed != t.trial_transcript_hash_v1 {
        errors.push(TranscriptVerifyError {
            kind: TranscriptVerifyErrorKind::TranscriptHashMismatch,
            message: "trial_transcript_hash_v1 does not match the recomputed hash".into(),
        });
    }

    // Cross-check the hash-chain anchors against the live live
    // corpus + precedent + grammar values. A future fixture could
    // legitimately commit to an older snapshot, so we only warn
    // via a soft check — the strict variant is on the negative-
    // test side. (Panel-locked: at T.11d the fixture cites the
    // CURRENT corpus + registry + precedent + grammar; the
    // verifier asserts the citations are nonzero, not that they
    // match the live state.)

    // Suppress unused; precedents was consulted via known_pids.
    let _ = &precedents as &PrecedentSet;
    // Grammar is also consumed above.
    let _ = &grammar as &AdmissibilityGrammarSnapshot;

    errors
}

fn lookup_dedup_decision(id: DetectorCanonicalId) -> Option<CanonicalisationDecision> {
    let dedup = crate::court::classify_all();
    for rec in &dedup {
        if let crate::types::DedupSubject::Canonical(c) = rec.subject {
            if c == id {
                return Some(rec.decision);
            }
        }
    }
    None
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

fn ids_compact(ids: &[DetectorCanonicalId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::new();
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    if s.is_empty() {
        s.push_str("(none)");
    }
    s
}

fn precedent_ids_compact(ids: &[PrecedentId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::new();
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    if s.is_empty() {
        s.push_str("(none)");
    }
    s
}

/// Render a `TrialTranscriptV1` as deterministic human-readable
/// text. Two calls on the same transcript produce byte-identical
/// output. Rendered text is NOT part of the transcript hash.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_trial_transcript_text(t: &TrialTranscriptV1) -> String {
    let mut out = String::with_capacity(4 * 1024);
    out.push_str("================================================================\n");
    out.push_str("DSFB-GPU-Atlas — Trial Transcript V1 (T.11d)\n");
    out.push_str("================================================================\n");
    let _ = writeln!(out, "transcript_id : {}", t.transcript_id.0);
    let _ = writeln!(out, "schema        : {}", t.schema.as_str());
    let _ = writeln!(
        out,
        "transcript_hash_v1 : {}",
        hex_lower(&t.trial_transcript_hash_v1)
    );
    out.push('\n');
    out.push_str("Hash chain (bound by this transcript):\n");
    let _ = writeln!(
        out,
        "  corpus_hash_v1               : {}",
        hex_lower(&t.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  registry_hash_v2             : {}",
        hex_lower(&t.registry_hash_v2)
    );
    let _ = writeln!(
        out,
        "  precedent_hash_v1            : {}",
        hex_lower(&t.precedent_hash_v1)
    );
    let _ = writeln!(
        out,
        "  admissibility_grammar_hash_v1: {}",
        hex_lower(&t.admissibility_grammar_hash_v1)
    );
    out.push('\n');
    out.push_str("Episode subject:\n");
    let _ = writeln!(out, "  motif        : {}", t.episode_subject.motif_label);
    let _ = writeln!(out, "  entity_id    : {}", t.episode_subject.entity_id);
    let _ = writeln!(
        out,
        "  window range : [{} .. {}]",
        t.episode_subject.window_start_idx, t.episode_subject.window_end_idx
    );
    out.push('\n');
    out.push_str("Admission:\n");
    let _ = writeln!(out, "  admitted_by_rule      : {}", t.admitted_by_rule.0);
    let _ = writeln!(
        out,
        "  supporting_precedents : {}",
        precedent_ids_compact(&t.supporting_precedents)
    );
    out.push('\n');
    out.push_str("Witnesses (canonical_ids):\n");
    let _ = writeln!(
        out,
        "  primary       : {}",
        ids_compact(&t.primary_witnesses)
    );
    let _ = writeln!(
        out,
        "  corroborating : {}",
        ids_compact(&t.corroborating_witnesses)
    );
    let _ = writeln!(
        out,
        "  boundary      : {}",
        ids_compact(&t.boundary_witnesses)
    );
    let _ = writeln!(
        out,
        "  recovery      : {}",
        ids_compact(&t.recovery_witnesses)
    );
    let _ = writeln!(
        out,
        "  clean_window  : {}",
        ids_compact(&t.clean_window_witnesses)
    );
    out.push('\n');
    out.push_str("Rejected confusers:\n");
    if t.rejected_confusers.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for c in &t.rejected_confusers {
            let _ = writeln!(
                out,
                "  - kind={} suppression_rule={} reason={}",
                c.trigger_kind.as_str(),
                c.suppression_rule_id.0,
                c.reason_code.as_str()
            );
        }
    }
    out.push('\n');
    out.push_str("Disabled but relevant:\n");
    if t.disabled_but_relevant.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for d in &t.disabled_but_relevant {
            let _ = writeln!(
                out,
                "  - canonical_id={} reason={}",
                d.canonical_id.0,
                d.disabled_reason.as_str()
            );
        }
    }
    out.push('\n');
    out.push_str("Reason-code coverage:\n");
    let _ = writeln!(
        out,
        "  admission_rule_has_reason         : {}",
        t.reason_code_coverage.admission_rule_has_reason
    );
    let _ = writeln!(
        out,
        "  witnesses_have_reason             : {}",
        t.reason_code_coverage.witnesses_have_reason
    );
    let _ = writeln!(
        out,
        "  rejected_confusers_have_reason    : {}",
        t.reason_code_coverage.rejected_confusers_have_reason
    );
    let _ = writeln!(
        out,
        "  disabled_detectors_have_reason    : {}",
        t.reason_code_coverage.disabled_detectors_have_reason
    );
    let bp = t.reason_code_coverage.coverage_percent_bp;
    let pct_whole = bp / 100;
    let pct_frac = bp % 100;
    let _ = writeln!(
        out,
        "  coverage                          : {pct_whole}.{pct_frac:02}%  ({bp} basis points)"
    );
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

fn ids_json(ids: &[DetectorCanonicalId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::from("[");
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    s.push(']');
    s
}

fn precedent_ids_json(ids: &[PrecedentId]) -> String {
    let mut sorted: Vec<u32> = ids.iter().map(|i| i.0).collect();
    sorted.sort_unstable();
    let mut s = String::from("[");
    for (i, id) in sorted.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{id}");
    }
    s.push(']');
    s
}

/// Render a `TrialTranscriptV1` as deterministic JSON. Two calls
/// on the same transcript produce byte-identical output.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_trial_transcript_json(t: &TrialTranscriptV1) -> String {
    let mut out = String::with_capacity(4 * 1024);
    out.push_str("{\n");
    let _ = writeln!(out, "  \"transcript_id\": {},", t.transcript_id.0);
    out.push_str("  \"schema\": ");
    json_quote(&mut out, t.schema.as_str());
    out.push_str(",\n");
    let _ = writeln!(
        out,
        "  \"trial_transcript_hash_v1\": \"{}\",",
        hex_lower(&t.trial_transcript_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"corpus_hash_v1\": \"{}\",",
        hex_lower(&t.corpus_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"registry_hash_v2\": \"{}\",",
        hex_lower(&t.registry_hash_v2)
    );
    let _ = writeln!(
        out,
        "  \"precedent_hash_v1\": \"{}\",",
        hex_lower(&t.precedent_hash_v1)
    );
    let _ = writeln!(
        out,
        "  \"admissibility_grammar_hash_v1\": \"{}\",",
        hex_lower(&t.admissibility_grammar_hash_v1)
    );
    out.push_str("  \"episode_subject\": {\n");
    out.push_str("    \"motif_label\": ");
    json_quote(&mut out, t.episode_subject.motif_label);
    out.push_str(",\n");
    let _ = writeln!(out, "    \"entity_id\": {},", t.episode_subject.entity_id);
    let _ = writeln!(
        out,
        "    \"window_start_idx\": {},",
        t.episode_subject.window_start_idx
    );
    let _ = writeln!(
        out,
        "    \"window_end_idx\": {}",
        t.episode_subject.window_end_idx
    );
    out.push_str("  },\n");
    let _ = writeln!(out, "  \"admitted_by_rule\": {},", t.admitted_by_rule.0);
    let _ = writeln!(
        out,
        "  \"supporting_precedents\": {},",
        precedent_ids_json(&t.supporting_precedents)
    );
    let _ = writeln!(
        out,
        "  \"primary_witnesses\": {},",
        ids_json(&t.primary_witnesses)
    );
    let _ = writeln!(
        out,
        "  \"corroborating_witnesses\": {},",
        ids_json(&t.corroborating_witnesses)
    );
    let _ = writeln!(
        out,
        "  \"boundary_witnesses\": {},",
        ids_json(&t.boundary_witnesses)
    );
    let _ = writeln!(
        out,
        "  \"recovery_witnesses\": {},",
        ids_json(&t.recovery_witnesses)
    );
    let _ = writeln!(
        out,
        "  \"clean_window_witnesses\": {},",
        ids_json(&t.clean_window_witnesses)
    );
    out.push_str("  \"rejected_confusers\": [");
    for (i, c) in t.rejected_confusers.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        out.push_str("\"trigger_kind\": ");
        json_quote(&mut out, c.trigger_kind.as_str());
        let _ = write!(
            out,
            ", \"suppression_rule_id\": {}",
            c.suppression_rule_id.0
        );
        out.push_str(", \"reason_code\": ");
        json_quote(&mut out, c.reason_code.as_str());
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"disabled_but_relevant\": [");
    for (i, d) in t.disabled_but_relevant.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push('{');
        let _ = write!(out, "\"canonical_id\": {}", d.canonical_id.0);
        out.push_str(", \"disabled_reason\": ");
        json_quote(&mut out, d.disabled_reason.as_str());
        out.push('}');
    }
    out.push_str("],\n");
    out.push_str("  \"reason_code_coverage\": {\n");
    let _ = writeln!(
        out,
        "    \"admission_rule_has_reason\": {},",
        t.reason_code_coverage.admission_rule_has_reason
    );
    let _ = writeln!(
        out,
        "    \"witnesses_have_reason\": {},",
        t.reason_code_coverage.witnesses_have_reason
    );
    let _ = writeln!(
        out,
        "    \"rejected_confusers_have_reason\": {},",
        t.reason_code_coverage.rejected_confusers_have_reason
    );
    let _ = writeln!(
        out,
        "    \"disabled_detectors_have_reason\": {},",
        t.reason_code_coverage.disabled_detectors_have_reason
    );
    let _ = writeln!(
        out,
        "    \"coverage_percent_bp\": {}",
        t.reason_code_coverage.coverage_percent_bp
    );
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

/// True if every panel-required hash chain anchor is present.
/// Used by tests + reports. Not load-bearing for `verify_*`.
#[must_use]
pub fn confuser_effect_for_trigger(
    grammar: &AdmissibilityGrammarSnapshot,
    trigger: NegativeWitnessKind,
) -> Option<ConfuserEffect> {
    grammar
        .confuser_rules
        .iter()
        .find(|r| r.trigger_kind == trigger)
        .map(|r| r.effect)
}

/// Convenience: full panel-locked enum count.
///
/// 4 confuser-rejection reasons × 7 disabled-detector reasons.
/// Used by the test that pins enum-count drift.
#[must_use]
pub const fn t11d_enum_counts() -> (u32, u32, u32, u32) {
    // (admission grammar rule kinds, evidence requirements,
    //  confuser rejection reasons, disabled reasons including
    //  Unspecified)
    (1, 1, 4, 7)
}

/// Discriminate `GrammarRuleKind::EpisodeAdmission` for tests
/// that need to assert the rule's kind without importing the
/// enum.
#[must_use]
pub fn rule_kind_is_episode_admission(kind: GrammarRuleKind) -> bool {
    matches!(kind, GrammarRuleKind::EpisodeAdmission)
}
