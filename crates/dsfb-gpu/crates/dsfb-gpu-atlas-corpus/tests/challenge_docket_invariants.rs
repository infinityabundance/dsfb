//! T.11f — `ChallengeDocketV1` acceptance tests.
//!
//! Every test mutates one field of the docket fixture and asserts
//! a single verifier reject kind fires, OR asserts a determinism /
//! rendering / hash-pin invariant. Two panel-required load-bearing
//! negatives:
//!
//!   - `challenge_docket_rejects_sustained_challenge_without_resolution`
//!   - `challenge_docket_rejects_bad_source_without_source_evidence`
//!
//! The docket itself is an adversarial overlay; the tests treat it
//! exactly that way.

use dsfb_gpu_atlas_corpus::admissibility::GrammarRuleId;
use dsfb_gpu_atlas_corpus::challenge_docket::{
    collect_challenge_docket, compute_challenge_docket_hash_v1, render_challenge_docket_json,
    render_challenge_docket_text, verify_challenge_docket, AffectedHashSet, ChallengeDocketEntry,
    ChallengeDocketSchema, ChallengeDocketSnapshot, ChallengeEvidenceRef, ChallengeId,
    ChallengeKind, ChallengeSeverity, ChallengeStatus, ChallengeTarget, ChallengerRole,
    CourtResponse, DocketVerifyErrorKind, ProposedResolution, CHALLENGES, CHALLENGE_DOCKET_DOMAIN,
    CHALLENGE_DOCKET_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::execution_attestation::ExecutionReceiptId;
use dsfb_gpu_atlas_corpus::precedent::PrecedentId;
use dsfb_gpu_atlas_corpus::trial_transcript::TrialTranscriptId;
use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;

// ---------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------

fn synth_entry() -> ChallengeDocketEntry {
    ChallengeDocketEntry {
        challenge_id: ChallengeId(999),
        target: ChallengeTarget::CorpusGlobal,
        challenge_kind: ChallengeKind::DomainMisapplied,
        severity: ChallengeSeverity::Low,
        status: ChallengeStatus::Overruled,
        challenger_role: ChallengerRole::SelfAudit,
        claim: "synthetic",
        evidence_refs: &[ChallengeEvidenceRef::Note("synthetic")],
        affected_hashes: AffectedHashSet::default(),
        proposed_resolution: ProposedResolution::NoChangeRequired,
        court_response: CourtResponse::OverruledReason("synthetic overruled"),
        created_in_stage: "T.11f-test",
    }
}

fn singleton(e: ChallengeDocketEntry) -> ChallengeDocketSnapshot {
    ChallengeDocketSnapshot {
        schema: ChallengeDocketSchema::V1AdversarialOverlay,
        challenges: vec![e],
    }
}

// ---------------------------------------------------------------
// Schema + constants
// ---------------------------------------------------------------

#[test]
fn challenge_docket_domain_separator_is_panel_locked() {
    assert_eq!(
        CHALLENGE_DOCKET_DOMAIN,
        "DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1\0"
    );
}

#[test]
fn challenge_docket_schema_id_is_panel_locked() {
    assert_eq!(
        CHALLENGE_DOCKET_SCHEMA_V1,
        "DSFB-GPU-ATLAS:CHALLENGE-DOCKET:v1"
    );
}

#[test]
fn collected_snapshot_matches_static_seed_length() {
    let s = collect_challenge_docket();
    assert_eq!(s.challenges.len(), CHALLENGES.len());
    assert_eq!(s.challenges.len(), 10);
}

#[test]
fn collected_snapshot_is_sorted_by_challenge_id_ascending() {
    let s = collect_challenge_docket();
    for w in s.challenges.windows(2) {
        assert!(w[0].challenge_id.0 < w[1].challenge_id.0);
    }
}

#[test]
fn collected_snapshot_schema_is_v1() {
    let s = collect_challenge_docket();
    assert_eq!(s.schema.as_str(), "V1AdversarialOverlay");
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn docket_hash_is_deterministic_across_two_builds() {
    let a = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let b = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    assert_eq!(a, b);
}

#[test]
fn docket_hash_changes_when_entry_added() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges.push(synth_entry());
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_claim_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].claim = "mutated";
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_status_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].status = ChallengeStatus::Open;
    s.challenges[0].court_response = CourtResponse::NotYetResponded;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_kind_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].challenge_kind = ChallengeKind::DomainMisapplied;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_target_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].target = ChallengeTarget::CorpusGlobal;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_severity_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].severity = ChallengeSeverity::Critical;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_evidence_added() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let extra_evidence: &'static [ChallengeEvidenceRef] = &[ChallengeEvidenceRef::Note("extra")];
    let mut s = collect_challenge_docket();
    s.challenges[0].evidence_refs = extra_evidence;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_court_response_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].court_response = CourtResponse::OverruledReason("mutated reason");
    s.challenges[0].status = ChallengeStatus::Overruled;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_proposed_resolution_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].proposed_resolution = ProposedResolution::PromoteAliasToCanonical;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_changes_when_affected_hash_set_changes() {
    let base = compute_challenge_docket_hash_v1(&collect_challenge_docket());
    let mut s = collect_challenge_docket();
    s.challenges[0].affected_hashes.corpus_hash = true;
    let h = compute_challenge_docket_hash_v1(&s);
    assert_ne!(h, base);
}

#[test]
fn docket_hash_is_independent_of_entry_order() {
    let s_sorted = collect_challenge_docket();
    let mut s_rev = collect_challenge_docket();
    s_rev.challenges.reverse();
    assert_eq!(
        compute_challenge_docket_hash_v1(&s_sorted),
        compute_challenge_docket_hash_v1(&s_rev),
    );
}

// ---------------------------------------------------------------
// Positive admission paths
// ---------------------------------------------------------------

#[test]
fn verifier_admits_clean_seed_docket() {
    let s = collect_challenge_docket();
    let errors = verify_challenge_docket(&s);
    assert!(
        errors.is_empty(),
        "expected zero verifier errors on clean seed, got: {errors:?}",
    );
}

#[test]
fn verifier_admits_singleton_overruled_overruled_with_reason() {
    let s = singleton(synth_entry());
    assert!(verify_challenge_docket(&s).is_empty());
}

// ---------------------------------------------------------------
// Verifier reject paths — one test per kind
// ---------------------------------------------------------------

fn assert_only_error(s: &ChallengeDocketSnapshot, kind: DocketVerifyErrorKind) {
    let errors = verify_challenge_docket(s);
    assert!(
        errors.iter().any(|e| e.kind == kind),
        "expected {kind:?}; got {errors:?}",
    );
}

#[test]
fn verifier_rejects_challenge_against_missing_detector() {
    let mut e = synth_entry();
    e.target = ChallengeTarget::Detector(DetectorCanonicalId(99_999));
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::ChallengeAgainstMissingDetector);
}

#[test]
fn verifier_rejects_challenge_against_missing_passport() {
    let mut e = synth_entry();
    e.target = ChallengeTarget::Passport(DetectorCanonicalId(99_999));
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::ChallengeAgainstMissingDetector);
}

#[test]
fn verifier_rejects_challenge_against_missing_precedent() {
    let mut e = synth_entry();
    e.target = ChallengeTarget::Precedent(PrecedentId(99_999));
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::ChallengeAgainstMissingPrecedent);
}

#[test]
fn verifier_rejects_challenge_against_missing_grammar_rule() {
    let mut e = synth_entry();
    e.target = ChallengeTarget::GrammarRule(GrammarRuleId(99_999));
    let s = singleton(e);
    assert_only_error(
        &s,
        DocketVerifyErrorKind::ChallengeAgainstMissingGrammarRule,
    );
}

#[test]
fn verifier_rejects_empty_claim() {
    let mut e = synth_entry();
    e.claim = "";
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::EmptyClaim);
}

#[test]
fn verifier_rejects_empty_created_in_stage() {
    let mut e = synth_entry();
    e.created_in_stage = "";
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::EmptyChallenger);
}

#[test]
fn verifier_rejects_duplicate_challenge_id() {
    let mut a = synth_entry();
    a.challenge_id = ChallengeId(42);
    let mut b = synth_entry();
    b.challenge_id = ChallengeId(42);
    let s = ChallengeDocketSnapshot {
        schema: ChallengeDocketSchema::V1AdversarialOverlay,
        challenges: vec![a, b],
    };
    assert_only_error(&s, DocketVerifyErrorKind::DuplicateChallengeId);
}

// PANEL-REQUIRED LOAD-BEARING NEGATIVE #1
#[test]
fn challenge_docket_rejects_sustained_challenge_without_resolution() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Sustained;
    e.proposed_resolution = ProposedResolution::NoAction;
    e.court_response = CourtResponse::SustainedAwaitingMutation;
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::SustainedWithoutResolution);
}

#[test]
fn verifier_rejects_overruled_with_not_yet_responded() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Overruled;
    e.court_response = CourtResponse::NotYetResponded;
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::OverruledWithoutCourtResponse);
}

#[test]
fn verifier_rejects_overruled_with_empty_reason_text() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Overruled;
    e.court_response = CourtResponse::OverruledReason("");
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::OverruledWithoutCourtResponse);
}

#[test]
fn verifier_rejects_deferred_with_not_yet_responded() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Deferred;
    e.court_response = CourtResponse::NotYetResponded;
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::DeferredWithoutDeferralReason);
}

#[test]
fn verifier_rejects_deferred_with_empty_gate_text() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Deferred;
    e.court_response = CourtResponse::DeferredToGate("");
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::DeferredWithoutDeferralReason);
}

#[test]
fn verifier_rejects_superseded_with_not_yet_responded() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Superseded;
    e.court_response = CourtResponse::NotYetResponded;
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::SupersededWithoutCommitReference);
}

#[test]
fn verifier_rejects_superseded_with_empty_commit_reference() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Superseded;
    e.court_response = CourtResponse::SupersededByCommit("");
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::SupersededWithoutCommitReference);
}

#[test]
fn verifier_rejects_open_with_court_response() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Open;
    e.court_response = CourtResponse::OverruledReason("court spoke too early");
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::StatusResponseInconsistent);
}

#[test]
fn verifier_rejects_runtime_too_high_without_runtime_evidence() {
    let mut e = synth_entry();
    e.challenge_kind = ChallengeKind::RuntimeTooHigh;
    e.evidence_refs = &[ChallengeEvidenceRef::Note("no runtime data")];
    let s = singleton(e);
    assert_only_error(
        &s,
        DocketVerifyErrorKind::RuntimeTooHighWithoutRuntimeEvidence,
    );
}

#[test]
fn verifier_rejects_formula_mismatch_without_formula_hash() {
    let mut e = synth_entry();
    e.challenge_kind = ChallengeKind::FormulaMismatch;
    e.evidence_refs = &[ChallengeEvidenceRef::Note("no formula hash")];
    let s = singleton(e);
    assert_only_error(
        &s,
        DocketVerifyErrorKind::FormulaMismatchWithoutFormulaHashReference,
    );
}

// PANEL-REQUIRED LOAD-BEARING NEGATIVE #2
#[test]
fn challenge_docket_rejects_bad_source_without_source_evidence() {
    let mut e = synth_entry();
    e.challenge_kind = ChallengeKind::BadSource;
    e.evidence_refs = &[ChallengeEvidenceRef::Note("no source ref")];
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::BadSourceWithoutSourceEvidence);
}

#[test]
fn verifier_rejects_wrong_witness_role_without_semantic_role_hash() {
    let mut e = synth_entry();
    e.challenge_kind = ChallengeKind::WrongWitnessRole;
    e.evidence_refs = &[ChallengeEvidenceRef::Note("no semantic role hash")];
    let s = singleton(e);
    assert_only_error(
        &s,
        DocketVerifyErrorKind::WrongWitnessRoleWithoutSemanticRoleHashReference,
    );
}

#[test]
fn verifier_rejects_unimplemented_but_claimed_against_honest_l_band() {
    // SEED canonical id 1 is the first record; per the corpus it is
    // L0/L1/L2 (literature-only). Filing
    // UnimplementedButClaimed against it makes no sense.
    let mut e = synth_entry();
    e.challenge_kind = ChallengeKind::UnimplementedButClaimed;
    e.target = ChallengeTarget::Detector(DetectorCanonicalId(1));
    let s = singleton(e);
    assert_only_error(
        &s,
        DocketVerifyErrorKind::UnimplementedButClaimedAgainstHonestLBand,
    );
}

#[test]
fn verifier_rejects_open_critical_without_deferred_gate() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Open;
    e.court_response = CourtResponse::NotYetResponded;
    e.severity = ChallengeSeverity::Critical;
    e.affected_hashes.corpus_hash = true;
    e.proposed_resolution = ProposedResolution::NoChangeRequired;
    let s = singleton(e);
    assert_only_error(&s, DocketVerifyErrorKind::OpenCriticalWithoutDeferredGate);
}

#[test]
fn verifier_admits_open_critical_when_proposed_deferral_present() {
    let mut e = synth_entry();
    e.status = ChallengeStatus::Open;
    e.court_response = CourtResponse::NotYetResponded;
    e.severity = ChallengeSeverity::Critical;
    e.affected_hashes.corpus_hash = true;
    e.proposed_resolution = ProposedResolution::DeferToFutureCommit;
    let s = singleton(e);
    let errors = verify_challenge_docket(&s);
    assert!(errors.is_empty(), "expected admission, got {errors:?}");
}

// ---------------------------------------------------------------
// Target-kind coverage
// ---------------------------------------------------------------

#[test]
fn target_kind_strings_are_stable() {
    assert_eq!(
        ChallengeTarget::Detector(DetectorCanonicalId(1)).kind_str(),
        "Detector"
    );
    assert_eq!(
        ChallengeTarget::Precedent(PrecedentId(1)).kind_str(),
        "Precedent"
    );
    assert_eq!(
        ChallengeTarget::GrammarRule(GrammarRuleId(1)).kind_str(),
        "GrammarRule"
    );
    assert_eq!(
        ChallengeTarget::Passport(DetectorCanonicalId(1)).kind_str(),
        "Passport"
    );
    assert_eq!(
        ChallengeTarget::TrialTranscript(TrialTranscriptId(1)).kind_str(),
        "TrialTranscript"
    );
    assert_eq!(
        ChallengeTarget::ExecutionReceipt(ExecutionReceiptId(1)).kind_str(),
        "ExecutionReceipt"
    );
    assert_eq!(ChallengeTarget::CorpusGlobal.kind_str(), "CorpusGlobal");
    assert_eq!(ChallengeTarget::RegistryGlobal.kind_str(), "RegistryGlobal");
}

#[test]
fn challenge_kind_strings_are_stable() {
    assert_eq!(ChallengeKind::OverbroadAlias.as_str(), "OverbroadAlias");
    assert_eq!(ChallengeKind::MissingConfuser.as_str(), "MissingConfuser");
    assert_eq!(ChallengeKind::WrongWitnessRole.as_str(), "WrongWitnessRole");
    assert_eq!(ChallengeKind::BadSource.as_str(), "BadSource");
    assert_eq!(ChallengeKind::FormulaMismatch.as_str(), "FormulaMismatch");
    assert_eq!(ChallengeKind::DomainMisapplied.as_str(), "DomainMisapplied");
    assert_eq!(
        ChallengeKind::UnimplementedButClaimed.as_str(),
        "UnimplementedButClaimed",
    );
    assert_eq!(ChallengeKind::RuntimeTooHigh.as_str(), "RuntimeTooHigh");
    assert_eq!(
        ChallengeKind::HashBindingMismatch.as_str(),
        "HashBindingMismatch",
    );
    assert_eq!(
        ChallengeKind::MissingNegativeWitness.as_str(),
        "MissingNegativeWitness",
    );
    assert_eq!(
        ChallengeKind::EvidenceLevelOverclaimed.as_str(),
        "EvidenceLevelOverclaimed",
    );
}

#[test]
fn status_strings_are_stable() {
    assert_eq!(ChallengeStatus::Open.as_str(), "Open");
    assert_eq!(ChallengeStatus::Sustained.as_str(), "Sustained");
    assert_eq!(ChallengeStatus::Overruled.as_str(), "Overruled");
    assert_eq!(ChallengeStatus::Deferred.as_str(), "Deferred");
    assert_eq!(ChallengeStatus::Superseded.as_str(), "Superseded");
}

#[test]
fn severity_strings_are_stable() {
    assert_eq!(ChallengeSeverity::Critical.as_str(), "Critical");
    assert_eq!(ChallengeSeverity::High.as_str(), "High");
    assert_eq!(ChallengeSeverity::Medium.as_str(), "Medium");
    assert_eq!(ChallengeSeverity::Low.as_str(), "Low");
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn render_text_is_deterministic_across_two_calls() {
    let s = collect_challenge_docket();
    assert_eq!(
        render_challenge_docket_text(&s),
        render_challenge_docket_text(&s),
    );
}

#[test]
fn render_json_is_deterministic_across_two_calls() {
    let s = collect_challenge_docket();
    assert_eq!(
        render_challenge_docket_json(&s),
        render_challenge_docket_json(&s),
    );
}

#[test]
fn render_text_carries_docket_hash() {
    use std::fmt::Write as _;
    let s = collect_challenge_docket();
    let h = compute_challenge_docket_hash_v1(&s);
    let text = render_challenge_docket_text(&s);
    assert!(
        text.contains("challenge_docket_hash_v1 : "),
        "text missing hash header",
    );
    let mut hex = String::with_capacity(64);
    for b in &h {
        let _ = write!(&mut hex, "{b:02x}");
    }
    assert!(text.contains(&hex), "text missing hex hash");
}

#[test]
fn render_json_carries_docket_hash() {
    let s = collect_challenge_docket();
    let json = render_challenge_docket_json(&s);
    assert!(json.contains("\"challenge_docket_hash_v1\":"));
    assert!(json.contains("\"schema\":\"V1AdversarialOverlay\""));
}

#[test]
fn render_text_carries_panel_locked_non_claim() {
    let s = collect_challenge_docket();
    let text = render_challenge_docket_text(&s);
    assert!(
        text.contains("NOT a corpus mutation"),
        "text missing panel-locked non-claim",
    );
}

#[test]
fn render_json_contains_every_entry() {
    let s = collect_challenge_docket();
    let json = render_challenge_docket_json(&s);
    for e in &s.challenges {
        let needle = format!("\"challenge_id\":{}", e.challenge_id.0);
        assert!(
            json.contains(&needle),
            "json missing challenge {}",
            e.challenge_id.0
        );
    }
}

#[test]
fn render_text_lists_every_overruled_reason() {
    let s = collect_challenge_docket();
    let text = render_challenge_docket_text(&s);
    for e in &s.challenges {
        if matches!(e.status, ChallengeStatus::Overruled) {
            let needle = format!("#{:<3}", e.challenge_id.0);
            assert!(
                text.contains(&needle),
                "text missing overruled #{}",
                e.challenge_id.0
            );
        }
    }
}

#[test]
fn hash_is_not_a_function_of_rendered_text_bytes() {
    let s1 = collect_challenge_docket();
    let mut s2 = collect_challenge_docket();
    // Mutate text-irrelevant ordering (the snapshot collector
    // already sorts before hashing; ordering swap must not move
    // the hash).
    s2.challenges.swap(0, 1);
    assert_eq!(
        compute_challenge_docket_hash_v1(&s1),
        compute_challenge_docket_hash_v1(&s2),
    );
}

// ---------------------------------------------------------------
// Seed-shape invariants (additional honest pins)
// ---------------------------------------------------------------

#[test]
fn seed_has_no_open_challenges() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        assert_ne!(
            e.status,
            ChallengeStatus::Open,
            "seed entry #{} must not be Open at T.11f",
            e.challenge_id.0,
        );
    }
}

#[test]
fn seed_has_no_sustained_challenges() {
    // Panel-locked posture: T.11f ships conservative seed only.
    // Any Sustained entry would require a separate later commit
    // that mutates the canonical artifact.
    let s = collect_challenge_docket();
    for e in &s.challenges {
        assert_ne!(
            e.status,
            ChallengeStatus::Sustained,
            "seed entry #{} must not be Sustained at T.11f",
            e.challenge_id.0,
        );
    }
}

#[test]
fn seed_overruled_entries_carry_overruled_reason_variant() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        if matches!(e.status, ChallengeStatus::Overruled) {
            assert!(
                matches!(e.court_response, CourtResponse::OverruledReason(_)),
                "seed entry #{} Overruled but court_response has wrong variant",
                e.challenge_id.0,
            );
            assert!(
                !e.court_response.reason_text().is_empty(),
                "seed entry #{} Overruled reason empty",
                e.challenge_id.0,
            );
        }
    }
}

#[test]
fn seed_deferred_entries_carry_deferred_to_gate_variant() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        if matches!(e.status, ChallengeStatus::Deferred) {
            assert!(
                matches!(e.court_response, CourtResponse::DeferredToGate(_)),
                "seed entry #{} Deferred but court_response has wrong variant",
                e.challenge_id.0,
            );
            assert!(
                !e.court_response.reason_text().is_empty(),
                "seed entry #{} Deferred gate text empty",
                e.challenge_id.0,
            );
        }
    }
}

#[test]
fn seed_every_entry_was_created_in_t11f() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        assert_eq!(e.created_in_stage, "T.11f");
    }
}

#[test]
fn seed_runtime_too_high_entries_carry_runtime_evidence() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        if matches!(e.challenge_kind, ChallengeKind::RuntimeTooHigh) {
            let has = e
                .evidence_refs
                .iter()
                .any(|ev| matches!(ev, ChallengeEvidenceRef::RuntimeCostUs(_)));
            assert!(
                has,
                "seed entry #{} RuntimeTooHigh missing RuntimeCostUs evidence",
                e.challenge_id.0,
            );
        }
    }
}

#[test]
fn seed_bad_source_entries_carry_source_evidence() {
    let s = collect_challenge_docket();
    for e in &s.challenges {
        if matches!(e.challenge_kind, ChallengeKind::BadSource) {
            let has = e.evidence_refs.iter().any(|ev| {
                matches!(
                    ev,
                    ChallengeEvidenceRef::SourceRef(_) | ChallengeEvidenceRef::SourceHash(_)
                )
            });
            assert!(
                has,
                "seed entry #{} BadSource missing SourceRef/SourceHash",
                e.challenge_id.0,
            );
        }
    }
}

#[test]
fn affected_hash_set_default_is_all_false() {
    let a = AffectedHashSet::default();
    assert!(a.is_empty());
    assert!(!a.corpus_hash);
    assert!(!a.registry_hash);
    assert!(!a.precedent_hash);
    assert!(!a.grammar_hash);
    assert!(!a.transcript_hash);
    assert!(!a.passport_hash);
}

#[test]
fn affected_hash_set_not_empty_when_any_field_true() {
    let a = AffectedHashSet {
        corpus_hash: true,
        ..AffectedHashSet::default()
    };
    assert!(!a.is_empty());
}
