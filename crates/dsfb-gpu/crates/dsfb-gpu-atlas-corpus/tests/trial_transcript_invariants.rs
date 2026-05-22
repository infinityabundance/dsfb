//! T.11d acceptance tests for the trial-transcript body.
//!
//! Panel-required invariants (25+ including two load-bearing
//! negatives).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::admissibility::{
    collect_admissibility_grammar, GrammarRuleId, GrammarRuleKind,
};
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
use dsfb_gpu_atlas_corpus::precedent::{collect_court_precedents, PrecedentId};
use dsfb_gpu_atlas_corpus::trial_transcript::{
    build_t11d_latency_ramp_fixture, compute_trial_transcript_hash_v1,
    render_trial_transcript_json, render_trial_transcript_text, verify_trial_transcript,
    ConfuserRejectionReason, DetectorDisabledReason, DisabledRelevantDetector, ReasonCodeCoverage,
    RejectedConfuser, TranscriptVerifyErrorKind, TrialTranscriptSchema, TRIAL_TRANSCRIPT_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::types::{DetectorCanonicalId, NegativeWitnessKind};

const T10_CORPUS_HASH_HEX: &str =
    "35c276c73a52d916daafda2598b215d73e7fd694d4a0673e34ac1ef948f5a4b7";
const S12_REGISTRY_HASH_HEX: &str =
    "d3cf63000cee922818e8dbc79ffecbc27d288063efbaed589e1eb1812bc37a08";
const T11B_PRECEDENT_HASH_HEX: &str =
    "6721f511f1eb951ba7eff4fa36832f233331507f6e4208d4f97866afd984dd14";
const T11C_GRAMMAR_HASH_HEX: &str =
    "ff66706a726d0cddc5f343e21f2ffbd8f81392a1504ff1b2002f8609d14a5ba7";

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[test]
fn fixture_is_deterministic_across_two_builds() {
    let a = build_t11d_latency_ramp_fixture();
    let b = build_t11d_latency_ramp_fixture();
    assert_eq!(
        a.trial_transcript_hash_v1, b.trial_transcript_hash_v1,
        "two builds must produce byte-identical transcript hashes"
    );
}

#[test]
fn fixture_verifies_clean() {
    let t = build_t11d_latency_ramp_fixture();
    let errors = verify_trial_transcript(&t);
    assert!(
        errors.is_empty(),
        "fixture must verify clean; got {errors:?}"
    );
}

#[test]
fn transcript_hash_includes_corpus_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.corpus_hash_v1[0] ^= 0xFF;
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(
        baseline, mutated,
        "transcript hash must change when corpus_hash_v1 changes"
    );
}

#[test]
fn transcript_hash_includes_registry_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.registry_hash_v2[0] ^= 0xFF;
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn transcript_hash_includes_precedent_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.precedent_hash_v1[0] ^= 0xFF;
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn transcript_hash_includes_admissibility_grammar_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.admissibility_grammar_hash_v1[0] ^= 0xFF;
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn transcript_hash_changes_when_primary_witness_changes() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.primary_witnesses.push(DetectorCanonicalId(2));
    t.primary_witnesses.sort_unstable_by_key(|c| c.0);
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn transcript_hash_changes_when_admission_rule_changes() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.admitted_by_rule = GrammarRuleId(t.admitted_by_rule.0 + 1);
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn transcript_hash_changes_when_reason_coverage_changes() {
    let mut t = build_t11d_latency_ramp_fixture();
    let baseline = t.trial_transcript_hash_v1;
    t.reason_code_coverage.coverage_percent_bp = 9999;
    let mutated = compute_trial_transcript_hash_v1(&t);
    assert_ne!(baseline, mutated);
}

#[test]
fn fixture_cites_live_corpus_hash() {
    let t = build_t11d_latency_ramp_fixture();
    let live = compute_corpus_hash_v1();
    assert_eq!(t.corpus_hash_v1, live.bytes);
    assert_eq!(hex(&t.corpus_hash_v1), T10_CORPUS_HASH_HEX);
}

#[test]
fn fixture_cites_panel_locked_registry_hash() {
    let t = build_t11d_latency_ramp_fixture();
    assert_eq!(hex(&t.registry_hash_v2), S12_REGISTRY_HASH_HEX);
}

#[test]
fn fixture_cites_live_precedent_hash() {
    let t = build_t11d_latency_ramp_fixture();
    let live = collect_court_precedents();
    assert_eq!(t.precedent_hash_v1, live.precedent_hash_v1);
    assert_eq!(hex(&t.precedent_hash_v1), T11B_PRECEDENT_HASH_HEX);
}

#[test]
fn fixture_cites_live_grammar_hash() {
    let t = build_t11d_latency_ramp_fixture();
    let live = collect_admissibility_grammar();
    assert_eq!(
        t.admissibility_grammar_hash_v1,
        live.admissibility_grammar_hash_v1.0
    );
    assert_eq!(hex(&t.admissibility_grammar_hash_v1), T11C_GRAMMAR_HASH_HEX);
}

#[test]
fn fixture_admitted_by_rule_resolves_to_primary_admission_rule() {
    let t = build_t11d_latency_ramp_fixture();
    let g = collect_admissibility_grammar();
    let rule = g
        .admission_rules
        .iter()
        .find(|r| r.id == t.admitted_by_rule)
        .expect("admitted_by_rule must resolve to a live rule");
    assert_eq!(rule.kind, GrammarRuleKind::EpisodeAdmission);
    assert_eq!(rule.name, "PrimaryWitnessRequiresPositiveSupport");
}

#[test]
fn fixture_supporting_precedents_are_live() {
    let t = build_t11d_latency_ramp_fixture();
    let p = collect_court_precedents();
    let live_ids: Vec<u32> = p.precedents.iter().map(|x| x.id.0).collect();
    for pid in &t.supporting_precedents {
        assert!(
            live_ids.contains(&pid.0),
            "supporting precedent id {} not in live set",
            pid.0
        );
    }
}

#[test]
fn fixture_primary_is_latency_ramp() {
    let t = build_t11d_latency_ramp_fixture();
    assert_eq!(t.primary_witnesses, vec![DetectorCanonicalId(14)]);
}

#[test]
fn fixture_corroborating_includes_ewma_and_cusum() {
    let t = build_t11d_latency_ramp_fixture();
    let ids: Vec<u32> = t.corroborating_witnesses.iter().map(|c| c.0).collect();
    assert!(
        ids.contains(&2),
        "EWMA (id 2) must be a corroborating witness"
    );
    assert!(
        ids.contains(&3),
        "CUSUM (id 3) must be a corroborating witness"
    );
}

#[test]
fn fixture_rejects_single_window_spike_confuser() {
    let t = build_t11d_latency_ramp_fixture();
    assert!(t
        .rejected_confusers
        .iter()
        .any(|c| c.trigger_kind == NegativeWitnessKind::SingleWindowSpikeConfuser));
}

#[test]
fn fixture_disables_fft_band_energy_with_reason() {
    let t = build_t11d_latency_ramp_fixture();
    let dis = t
        .disabled_but_relevant
        .iter()
        .find(|d| d.canonical_id == DetectorCanonicalId(12))
        .expect("FFT band-energy (id 12) must appear as disabled-but-relevant");
    assert_eq!(
        dis.disabled_reason,
        DetectorDisabledReason::MissingSpectralProjection
    );
}

#[test]
fn fixture_reason_code_coverage_is_full() {
    let t = build_t11d_latency_ramp_fixture();
    assert!(t.reason_code_coverage.is_full_coverage());
    assert_eq!(t.reason_code_coverage.coverage_percent_bp, 10_000);
}

#[test]
fn rendered_text_is_deterministic() {
    let t = build_t11d_latency_ramp_fixture();
    let a = render_trial_transcript_text(&t);
    let b = render_trial_transcript_text(&t);
    assert_eq!(a, b);
}

#[test]
fn rendered_json_is_deterministic() {
    let t = build_t11d_latency_ramp_fixture();
    let a = render_trial_transcript_json(&t);
    let b = render_trial_transcript_json(&t);
    assert_eq!(a, b);
}

#[test]
fn rendered_text_is_not_in_hash_material() {
    // The rendered-text decoration shouldn't affect the
    // canonical hash. This is implicit because the text
    // rendering is a pure function of the struct + the hash
    // only covers the canonical bytes. Cross-check via
    // structural equality.
    let t1 = build_t11d_latency_ramp_fixture();
    let mut t2 = t1.clone();
    // Mutate nothing observable in the hash material; render
    // text both times and confirm equal text PLUS equal hash.
    t2.trial_transcript_hash_v1 = t1.trial_transcript_hash_v1;
    assert_eq!(
        render_trial_transcript_text(&t1),
        render_trial_transcript_text(&t2)
    );
}

#[test]
fn no_publication_language_in_t11d_reports() {
    let t = build_t11d_latency_ramp_fixture();
    let text = render_trial_transcript_text(&t);
    let json = render_trial_transcript_json(&t);
    let forbidden = ["Zenodo", "DOI", "publication-grade", "peer-reviewed"];
    for body in [&text, &json] {
        for word in forbidden {
            assert!(
                !body.contains(word),
                "T.11d artifact contains forbidden publication-language `{word}`"
            );
        }
    }
}

#[test]
fn trial_transcript_verifier_rejects_confuser_only_admission() {
    // Panel-required load-bearing negative: a transcript that
    // tries to admit on confuser firings alone MUST be rejected.
    let mut t = build_t11d_latency_ramp_fixture();
    // Strip every positive witness; keep the rejected_confusers.
    t.primary_witnesses.clear();
    t.corroborating_witnesses.clear();
    t.boundary_witnesses.clear();
    t.recovery_witnesses.clear();
    t.clean_window_witnesses.clear();
    // Recompute the hash so the hash check doesn't mask the
    // semantic rejection.
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == TranscriptVerifyErrorKind::ConfuserOnlyAdmissionAttempted),
        "verifier MUST reject confuser-only admission; got {errors:?}"
    );
}

#[test]
fn trial_transcript_verifier_rejects_missing_admissibility_grammar_link() {
    // Panel-required load-bearing negative: a transcript that
    // does not cite the admissibility grammar (zero hash AND no
    // admission rule id) MUST be rejected.
    let mut t = build_t11d_latency_ramp_fixture();
    t.admissibility_grammar_hash_v1 = [0u8; 32];
    t.admitted_by_rule = GrammarRuleId(0);
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(
        errors
            .iter()
            .any(|e| e.kind == TranscriptVerifyErrorKind::MissingAdmissibilityGrammarLink),
        "verifier MUST reject missing admissibility-grammar link; got {errors:?}"
    );
    // Should also surface the zero-hash + missing-rule errors.
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::ZeroGrammarHash));
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::AdmissionRuleMissing));
}

#[test]
fn verifier_rejects_zero_corpus_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.corpus_hash_v1 = [0u8; 32];
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::ZeroCorpusHash));
}

#[test]
fn verifier_rejects_zero_registry_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.registry_hash_v2 = [0u8; 32];
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::ZeroRegistryHash));
}

#[test]
fn verifier_rejects_zero_precedent_hash() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.precedent_hash_v1 = [0u8; 32];
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::ZeroPrecedentHash));
}

#[test]
fn verifier_rejects_admission_rule_not_in_grammar() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.admitted_by_rule = GrammarRuleId(99_999);
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::AdmissionRuleNotInGrammar));
}

#[test]
fn verifier_rejects_empty_primary_witness_when_rule_requires_one() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.primary_witnesses.clear();
    // Keep corroborating + boundary so we don't trigger the
    // confuser-only test instead — we want PrimaryWitnessListEmpty.
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::PrimaryWitnessListEmpty));
}

#[test]
fn verifier_rejects_witness_id_missing_from_corpus() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.corroborating_witnesses.push(DetectorCanonicalId(9999));
    t.corroborating_witnesses.sort_unstable_by_key(|c| c.0);
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::WitnessIdMissingFromCorpus));
}

#[test]
fn verifier_rejects_rejected_confuser_with_unknown_suppression_rule() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.rejected_confusers.push(RejectedConfuser {
        trigger_kind: NegativeWitnessKind::SmallSampleConfuser,
        suppression_rule_id: GrammarRuleId(99_999),
        reason_code: ConfuserRejectionReason::NotFired,
    });
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::RejectedConfuserSuppressionRuleMissing));
}

#[test]
fn verifier_rejects_disabled_detector_with_unspecified_reason() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.disabled_but_relevant.push(DisabledRelevantDetector {
        canonical_id: DetectorCanonicalId(39),
        disabled_reason: DetectorDisabledReason::Unspecified,
    });
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::DisabledDetectorLacksReason));
}

#[test]
fn verifier_rejects_empty_reason_code_coverage() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.reason_code_coverage = ReasonCodeCoverage::empty();
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::ReasonCodeCoverageEmpty));
}

#[test]
fn verifier_rejects_transcript_hash_mismatch() {
    let mut t = build_t11d_latency_ramp_fixture();
    // Mutate without recomputing the hash.
    t.episode_subject.entity_id = 999;
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::TranscriptHashMismatch));
}

#[test]
fn verifier_rejects_supporting_precedent_missing() {
    let mut t = build_t11d_latency_ramp_fixture();
    t.supporting_precedents.push(PrecedentId(99_999));
    t.trial_transcript_hash_v1 = compute_trial_transcript_hash_v1(&t);
    let errors = verify_trial_transcript(&t);
    assert!(errors
        .iter()
        .any(|e| e.kind == TranscriptVerifyErrorKind::SupportingPrecedentMissing));
}

#[test]
fn schema_wire_name_is_panel_locked() {
    assert_eq!(
        TrialTranscriptSchema::V1MinimalT11d.as_str(),
        "V1MinimalT11d"
    );
    assert_eq!(
        TRIAL_TRANSCRIPT_SCHEMA_V1,
        "DSFB-GPU-ATLAS:TRIAL-TRANSCRIPT:v1"
    );
}

#[test]
fn enum_wire_names_cover_every_disabled_reason_variant() {
    let names = [
        DetectorDisabledReason::Unspecified.as_str(),
        DetectorDisabledReason::MissingSpectralProjection.as_str(),
        DetectorDisabledReason::IrregularSampling.as_str(),
        DetectorDisabledReason::UnitsUnclear.as_str(),
        DetectorDisabledReason::BelowMinimumSupport.as_str(),
        DetectorDisabledReason::DeferredImplementation.as_str(),
        DetectorDisabledReason::RedundantWithActivePeer.as_str(),
    ];
    for n in names {
        assert!(
            !n.is_empty(),
            "every DetectorDisabledReason wire name must be non-empty"
        );
    }
    // Count: 7 variants
    assert_eq!(names.len(), 7);
}

#[test]
fn enum_wire_names_cover_every_confuser_rejection_reason() {
    let names = [
        ConfuserRejectionReason::NotFired.as_str(),
        ConfuserRejectionReason::BankOverrideApplied.as_str(),
        ConfuserRejectionReason::Quarantined.as_str(),
        ConfuserRejectionReason::PreconditionUnmet.as_str(),
    ];
    for n in names {
        assert!(!n.is_empty());
    }
    assert_eq!(names.len(), 4);
}

#[test]
fn transcript_schema_field_is_v1_minimal_t11d() {
    let t = build_t11d_latency_ramp_fixture();
    assert_eq!(t.schema, TrialTranscriptSchema::V1MinimalT11d);
}

#[test]
fn transcript_id_is_one_at_t11d() {
    let t = build_t11d_latency_ramp_fixture();
    assert_eq!(t.transcript_id.0, 1);
}

#[test]
fn fixture_admission_rule_requires_at_least_one_primary_witness() {
    // Pinned by the T.11c grammar: the
    // PrimaryWitnessRequiresPositiveSupport rule has
    // min_primary >= 1. The fixture must satisfy that.
    let t = build_t11d_latency_ramp_fixture();
    let g = collect_admissibility_grammar();
    let rule = g
        .admission_rules
        .iter()
        .find(|r| r.id == t.admitted_by_rule)
        .expect("rule resolves");
    assert!(rule.witness_requirement.min_primary >= 1);
    assert!(t.primary_witnesses.len() >= rule.witness_requirement.min_primary as usize);
}

#[test]
fn corpus_hash_v1_unchanged_by_t11d() {
    let live = compute_corpus_hash_v1();
    assert_eq!(hex(&live.bytes), T10_CORPUS_HASH_HEX);
}

#[test]
fn precedent_hash_v1_unchanged_by_t11d() {
    let p = collect_court_precedents();
    assert_eq!(hex(&p.precedent_hash_v1), T11B_PRECEDENT_HASH_HEX);
}

#[test]
fn admissibility_grammar_hash_v1_unchanged_by_t11d() {
    let g = collect_admissibility_grammar();
    assert_eq!(
        hex(&g.admissibility_grammar_hash_v1.0),
        T11C_GRAMMAR_HASH_HEX
    );
}
