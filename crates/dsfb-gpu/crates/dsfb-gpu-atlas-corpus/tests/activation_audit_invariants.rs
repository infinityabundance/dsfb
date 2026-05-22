//! S1.3b acceptance suite — transcript + diff invariants.
//!
//! Every test states the WHY in a leading comment.
//! Four panel-required load-bearing negatives are marked with
//! the `_rejects_` prefix; they pin the verifier's blocking
//! rules and prove the explanation layer cannot silently admit
//! incoherent transcripts or diffs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::activation::{
    collect_activation_plan, ActivationStatus, KNOWN_S12_REGISTRY_HASH_V2,
};
use dsfb_gpu_atlas_corpus::activation_audit::{
    build_diff, build_plan_audit, build_transcript_for, compute_diff_hash_v1,
    compute_transcript_hash_v1, render_audit_json, render_audit_text, render_diff_json,
    render_diff_text, render_transcript_json, render_transcript_text, verify_diff,
    verify_transcript, ActivationDiffV1, ActivationPlanAuditV1, ArtifactKind, DiffVerifyErrorKind,
    FactRole, TranscriptVerifyErrorKind, ACTIVATION_DIFF_DOMAIN, ACTIVATION_DIFF_SCHEMA_V1,
    ACTIVATION_TRANSCRIPT_DOMAIN, ACTIVATION_TRANSCRIPT_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;

// ---------------------------------------------------------------
// Schema constants
// ---------------------------------------------------------------

/// Panel-locked: both domain separators end in NUL.
#[test]
fn activation_audit_domain_separators_end_in_nul() {
    assert!(ACTIVATION_TRANSCRIPT_DOMAIN.ends_with('\0'));
    assert!(ACTIVATION_DIFF_DOMAIN.ends_with('\0'));
}

/// Panel-locked: schema wire names are stable.
#[test]
fn activation_audit_schema_wire_names_are_stable() {
    assert_eq!(
        ACTIVATION_TRANSCRIPT_SCHEMA_V1,
        "ActivationDecisionTranscriptV1"
    );
    assert_eq!(ACTIVATION_DIFF_SCHEMA_V1, "ActivationDiffV1");
}

// ---------------------------------------------------------------
// Transcript shape
// ---------------------------------------------------------------

/// build_transcript_for returns Some for every SEED record.
#[test]
fn transcript_exists_for_every_seed_record() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id);
        assert!(
            t.is_some(),
            "no transcript for canonical_id {}",
            record.canonical_id.0
        );
    }
}

/// Transcript final_reason wire name matches the source plan's
/// reason wire name.
#[test]
fn transcript_final_reason_matches_plan_decision() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    for d in &plan.decisions {
        let t = build_transcript_for(d.canonical_id).unwrap();
        let plan_wire = match (d.enabled_reason, d.disabled_reason) {
            (Some(r), _) => r.as_str(),
            (_, Some(r)) => r.as_str(),
            _ => "",
        };
        assert_eq!(t.final_reason.wire_name, plan_wire);
    }
}

/// Every transcript carries at least one ContributingFact
/// (Passport + LBand + 2 anchors at minimum).
#[test]
fn transcript_always_carries_at_least_one_fact() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id).unwrap();
        assert!(!t.contributing_facts.is_empty());
    }
}

/// Contributing facts are sorted deterministically.
#[test]
fn transcript_facts_are_sorted_deterministically() {
    let t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    for win in t.contributing_facts.windows(2) {
        let (a, b) = (&win[0], &win[1]);
        let key_a = (a.artifact_kind, a.artifact_id, a.role);
        let key_b = (b.artifact_kind, b.artifact_id, b.role);
        assert!(key_a <= key_b);
    }
}

/// Every Disabled / Deferred transcript carries a non-empty
/// blocking_chain (mirrors verifier load-bearing #3).
#[test]
fn disabled_transcript_carries_blocking_chain() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id).unwrap();
        if matches!(
            t.activation_status,
            ActivationStatus::Disabled | ActivationStatus::Deferred
        ) {
            assert!(
                !t.blocking_chain.is_empty(),
                "transcript for {} (Disabled) has empty blocking_chain",
                record.canonical_id.0
            );
        }
    }
}

/// Enabled / WarnOnly transcripts have empty blocking_chain.
#[test]
fn enabled_transcript_has_empty_blocking_chain() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id).unwrap();
        if matches!(
            t.activation_status,
            ActivationStatus::Enabled | ActivationStatus::WarnOnly
        ) {
            assert!(
                t.blocking_chain.is_empty(),
                "transcript for {} ({:?}) has non-empty blocking_chain",
                record.canonical_id.0,
                t.activation_status
            );
        }
    }
}

/// Disabled transcripts carry at least one CounterfactualStep;
/// Enabled / WarnOnly carry zero.
#[test]
fn counterfactual_path_matches_status() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id).unwrap();
        match t.activation_status {
            ActivationStatus::Disabled | ActivationStatus::Deferred => {
                assert!(
                    !t.counterfactual_path_to_enabled.is_empty(),
                    "Disabled transcript for {} has empty counterfactual path",
                    record.canonical_id.0
                );
            }
            ActivationStatus::Enabled | ActivationStatus::WarnOnly => {
                assert!(
                    t.counterfactual_path_to_enabled.is_empty(),
                    "Enabled transcript for {} has non-empty counterfactual path",
                    record.canonical_id.0
                );
            }
        }
    }
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn transcript_hash_is_deterministic_across_two_builds() {
    let a = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    let b = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    assert_eq!(a.transcript_hash_v1, b.transcript_hash_v1);
}

#[test]
fn recomputed_transcript_hash_matches_stored_hash() {
    let t = build_transcript_for(DetectorCanonicalId(14)).unwrap();
    let recomputed = compute_transcript_hash_v1(&t);
    assert_eq!(recomputed, t.transcript_hash_v1);
}

/// Load-bearing negative #4 (panel-required): changing one
/// ContributingFact changes the transcript hash.
#[test]
fn transcript_hash_changes_when_one_contributing_fact_changes() {
    let mut t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    let original = t.transcript_hash_v1;
    // Mutate the first fact's reason_code.
    t.contributing_facts[0].reason_code = "MutatedForTest";
    let recomputed = compute_transcript_hash_v1(&t);
    assert_ne!(recomputed, original);
}

/// Different canonical_ids produce different transcript hashes.
#[test]
fn different_detectors_produce_different_transcript_hashes() {
    let a = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    let b = build_transcript_for(DetectorCanonicalId(2)).unwrap();
    assert_ne!(a.transcript_hash_v1, b.transcript_hash_v1);
}

// ---------------------------------------------------------------
// Transcript verifier
// ---------------------------------------------------------------

/// Seed transcripts pass the verifier clean.
#[test]
fn verifier_admits_seed_transcripts() {
    for record in SEED {
        let t = build_transcript_for(record.canonical_id).unwrap();
        let errors = verify_transcript(&t);
        assert!(
            errors.is_empty(),
            "seed transcript for {} produced errors: {errors:?}",
            record.canonical_id.0
        );
    }
}

/// Load-bearing negative #3 (panel-required): a Disabled
/// transcript with no Blocking fact MUST be rejected.
#[test]
fn transcript_disabled_decision_must_cite_at_least_one_artifact() {
    let mut t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    assert!(matches!(t.activation_status, ActivationStatus::Disabled));
    // Strip every Blocking role.
    for f in &mut t.contributing_facts {
        if matches!(f.role, FactRole::Blocking) {
            f.role = FactRole::Informational;
        }
    }
    t.blocking_chain.clear();
    let errors = verify_transcript(&t);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        TranscriptVerifyErrorKind::DisabledTranscriptWithoutBlockingFact
    )));
}

/// Load-bearing negative #1 (panel-required): explain MUST
/// refuse an unknown canonical_id (returns None at the
/// build_transcript_for layer; verifier rule would surface the
/// defect if a malformed transcript were constructed).
#[test]
fn explain_rejects_unknown_canonical_id() {
    let result = build_transcript_for(DetectorCanonicalId(99_999));
    assert!(result.is_none());
}

/// Wire-name stability for ArtifactKind / FactRole.
#[test]
fn artifact_kind_wire_names_are_stable() {
    assert_eq!(ArtifactKind::Passport.as_str(), "Passport");
    assert_eq!(ArtifactKind::CoverageHole.as_str(), "CoverageHole");
    assert_eq!(ArtifactKind::Contraindication.as_str(), "Contraindication");
    assert_eq!(ArtifactKind::Challenge.as_str(), "Challenge");
    assert_eq!(ArtifactKind::LBand.as_str(), "LBand");
    assert_eq!(ArtifactKind::RegistryHash.as_str(), "RegistryHash");
    assert_eq!(ArtifactKind::CorpusHash.as_str(), "CorpusHash");
}

#[test]
fn fact_role_wire_names_are_stable() {
    assert_eq!(FactRole::Blocking.as_str(), "Blocking");
    assert_eq!(FactRole::Warning.as_str(), "Warning");
    assert_eq!(FactRole::Supporting.as_str(), "Supporting");
    assert_eq!(FactRole::Informational.as_str(), "Informational");
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn render_transcript_text_is_byte_stable() {
    let t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    assert_eq!(render_transcript_text(&t), render_transcript_text(&t));
}

#[test]
fn render_transcript_json_is_byte_stable() {
    let t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    assert_eq!(render_transcript_json(&t), render_transcript_json(&t));
}

#[test]
fn render_transcript_text_lists_blocking_chain_section() {
    let t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    let text = render_transcript_text(&t);
    if matches!(
        t.activation_status,
        ActivationStatus::Disabled | ActivationStatus::Deferred
    ) {
        assert!(text.contains("Blocking chain"));
    }
}

#[test]
fn render_transcript_text_lists_counterfactual_section() {
    let t = build_transcript_for(DetectorCanonicalId(1)).unwrap();
    let text = render_transcript_text(&t);
    if !t.counterfactual_path_to_enabled.is_empty() {
        assert!(text.contains("Counterfactual path"));
    }
}

// ---------------------------------------------------------------
// Audit wrapper
// ---------------------------------------------------------------

#[test]
fn audit_carries_one_transcript_per_seed_record() {
    let audit: ActivationPlanAuditV1 = build_plan_audit();
    assert_eq!(audit.transcripts.len(), SEED.len());
}

#[test]
fn audit_text_is_byte_stable() {
    let audit = build_plan_audit();
    assert_eq!(render_audit_text(&audit), render_audit_text(&audit));
}

#[test]
fn audit_json_is_byte_stable() {
    let audit = build_plan_audit();
    assert_eq!(render_audit_json(&audit), render_audit_json(&audit));
}

// ---------------------------------------------------------------
// Diff
// ---------------------------------------------------------------

#[test]
fn diff_of_plan_against_itself_is_empty() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let diff = build_diff(&plan, &plan);
    assert_eq!(diff.rows.len(), 0);
    assert_eq!(diff.decisions_added, 0);
    assert_eq!(diff.decisions_removed, 0);
    assert_eq!(diff.decisions_status_changed, 0);
    assert_eq!(diff.decisions_reason_changed, 0);
    assert_eq!(diff.decisions_citation_changed, 0);
}

#[test]
fn diff_hash_is_deterministic_across_two_builds() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let a = build_diff(&plan, &plan);
    let b = build_diff(&plan, &plan);
    assert_eq!(a.activation_diff_hash_v1, b.activation_diff_hash_v1);
}

#[test]
fn recomputed_diff_hash_matches_stored_hash() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let d = build_diff(&plan, &plan);
    let recomputed = compute_diff_hash_v1(&d);
    assert_eq!(recomputed, d.activation_diff_hash_v1);
}

#[test]
fn diff_detects_status_change_when_one_decision_is_mutated() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let mut mutated = plan.clone();
    // Flip the first Disabled into Enabled-like.
    let idx = mutated
        .decisions
        .iter()
        .position(|d| matches!(d.activation_status, ActivationStatus::Disabled))
        .unwrap();
    mutated.decisions[idx].activation_status = ActivationStatus::Enabled;
    mutated.decisions[idx].enabled_reason =
        Some(dsfb_gpu_atlas_corpus::activation::EnabledReason::EnabledByPassportComplete);
    mutated.decisions[idx].disabled_reason = None;
    let diff = build_diff(&plan, &mutated);
    assert_eq!(diff.decisions_status_changed, 1);
    assert!(diff.rows.iter().any(|r| matches!(
        r.kind,
        dsfb_gpu_atlas_corpus::activation_audit::DiffChangeKind::StatusChanged
    )));
}

/// Load-bearing negative #2 (panel-required): diff between
/// plans with different `corpus_hash_v1` is rejected by the
/// verifier.
#[test]
fn diff_rejects_plans_with_different_corpus_hash() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let mut mutated = plan.clone();
    mutated.corpus_hash_v1[0] ^= 0xff;
    let diff = build_diff(&plan, &mutated);
    let errors = verify_diff(&diff, &plan, &mutated);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, DiffVerifyErrorKind::DiffRejectsMismatchedCorpusHash)));
}

/// Diff hash sensitivity: mutating one decision changes the
/// resulting diff hash.
#[test]
fn diff_hash_changes_when_plan_changes() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let mut mutated = plan.clone();
    let idx = mutated
        .decisions
        .iter()
        .position(|d| matches!(d.activation_status, ActivationStatus::Disabled))
        .unwrap();
    mutated.decisions[idx].activation_status = ActivationStatus::Enabled;
    mutated.decisions[idx].enabled_reason =
        Some(dsfb_gpu_atlas_corpus::activation::EnabledReason::EnabledByPassportComplete);
    mutated.decisions[idx].disabled_reason = None;
    let a = build_diff(&plan, &plan).activation_diff_hash_v1;
    let b = build_diff(&plan, &mutated).activation_diff_hash_v1;
    assert_ne!(a, b);
}

#[test]
fn render_diff_text_lists_change_summary() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let d: ActivationDiffV1 = build_diff(&plan, &plan);
    let text = render_diff_text(&d);
    assert!(text.contains("Change summary"));
    assert!(text.contains("DecisionAdded"));
    assert!(text.contains("DecisionRemoved"));
    assert!(text.contains("StatusChanged"));
    assert!(text.contains("ReasonChanged"));
    assert!(text.contains("CitationChanged"));
}

#[test]
fn render_diff_json_is_byte_stable() {
    let plan = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let d = build_diff(&plan, &plan);
    assert_eq!(render_diff_json(&d), render_diff_json(&d));
}

// ---------------------------------------------------------------
// Upstream hash anchor preservation (S1.3b non-claim guard)
// ---------------------------------------------------------------

/// S1.3b MUST NOT mutate any upstream hash anchor. The plan
/// hashes from the live plan are byte-identical before and
/// after building an audit and a diff.
#[test]
fn audit_and_diff_do_not_mutate_plan_hashes() {
    let plan_before = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    let _audit = build_plan_audit();
    let _diff = build_diff(&plan_before, &plan_before);
    let plan_after = collect_activation_plan(KNOWN_S12_REGISTRY_HASH_V2);
    assert_eq!(
        plan_before.activation_plan_hash_v1,
        plan_after.activation_plan_hash_v1
    );
    assert_eq!(plan_before.corpus_hash_v1, plan_after.corpus_hash_v1);
    assert_eq!(plan_before.registry_hash_v2, plan_after.registry_hash_v2);
    assert_eq!(
        plan_before.challenge_docket_hash_v1,
        plan_after.challenge_docket_hash_v1
    );
    assert_eq!(
        plan_before.detector_contraindication_hash_v1,
        plan_after.detector_contraindication_hash_v1
    );
    assert_eq!(
        plan_before.coverage_hole_hash_v1,
        plan_after.coverage_hole_hash_v1
    );
}
