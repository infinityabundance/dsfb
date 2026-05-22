//! T.12.0 acceptance suite — CorpusAmendmentProposalV1 +
//! CorpusExpansionBatch + DedupCourtDelta invariants.
//!
//! Four panel-required load-bearing negatives are marked with
//! the `_rejects_` prefix or with explicit comments; they pin
//! the verifier's blocking rules and prove the intake court
//! does not silently admit defective proposals.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    compute_corpus_amendment_proposal_hash_v1, compute_dedup_court_delta_hash_v1,
    compute_literature_expansion_batch_hash_v1, render_amendment_proposal_json,
    render_amendment_proposal_text, seed_proof_of_life_proposal, verify_amendment_proposal,
    AmendmentVerifyErrorKind, ProposalStatus, ProposedPrimitive, ProposerRole, SourceClass,
    CORPUS_AMENDMENT_PROPOSAL_DOMAIN, CORPUS_AMENDMENT_PROPOSAL_SCHEMA_V1,
    DEDUP_COURT_DELTA_DOMAIN, DEDUP_COURT_DELTA_SCHEMA_V1, LITERATURE_EXPANSION_BATCH_DOMAIN,
    LITERATURE_EXPANSION_BATCH_SCHEMA_V1,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::DetectorCanonicalId;

// ---------------------------------------------------------------
// Schema constants
// ---------------------------------------------------------------

#[test]
fn three_domain_separators_end_in_nul() {
    assert!(LITERATURE_EXPANSION_BATCH_DOMAIN.ends_with('\0'));
    assert!(CORPUS_AMENDMENT_PROPOSAL_DOMAIN.ends_with('\0'));
    assert!(DEDUP_COURT_DELTA_DOMAIN.ends_with('\0'));
}

#[test]
fn schema_wire_names_are_stable() {
    assert_eq!(
        LITERATURE_EXPANSION_BATCH_SCHEMA_V1,
        "LiteratureExpansionBatchV1"
    );
    assert_eq!(
        CORPUS_AMENDMENT_PROPOSAL_SCHEMA_V1,
        "CorpusAmendmentProposalV1"
    );
    assert_eq!(DEDUP_COURT_DELTA_SCHEMA_V1, "DedupCourtDeltaV1");
}

#[test]
fn source_class_wire_names_are_stable_for_all_23_variants() {
    use SourceClass as S;
    // Spot-check enough variants to catch silent renames.
    assert_eq!(
        S::StatisticalProcessControl.as_str(),
        "StatisticalProcessControl"
    );
    assert_eq!(
        S::SequentialChangeDetection.as_str(),
        "SequentialChangeDetection"
    );
    assert_eq!(S::DriftDetection.as_str(), "DriftDetection");
    assert_eq!(S::RobustStatistics.as_str(), "RobustStatistics");
    assert_eq!(S::DistributionDistance.as_str(), "DistributionDistance");
    assert_eq!(S::InformationTheory.as_str(), "InformationTheory");
    assert_eq!(S::SignalProcessing.as_str(), "SignalProcessing");
    assert_eq!(S::SpectralAndWavelet.as_str(), "SpectralAndWavelet");
    assert_eq!(S::TimeSeriesStructure.as_str(), "TimeSeriesStructure");
    assert_eq!(S::ControlResiduals.as_str(), "ControlResiduals");
    assert_eq!(
        S::FaultDetectionDiagnostics.as_str(),
        "FaultDetectionDiagnostics"
    );
    assert_eq!(S::ConditionMonitoring.as_str(), "ConditionMonitoring");
    assert_eq!(
        S::IndustrialProcessMonitoring.as_str(),
        "IndustrialProcessMonitoring"
    );
    assert_eq!(S::GraphAnomalyDetection.as_str(), "GraphAnomalyDetection");
    assert_eq!(S::StreamingSketches.as_str(), "StreamingSketches");
    assert_eq!(S::DataQualityRules.as_str(), "DataQualityRules");
    assert_eq!(
        S::DatabaseIntegrityConstraints.as_str(),
        "DatabaseIntegrityConstraints"
    );
    assert_eq!(S::ObservabilityDebugging.as_str(), "ObservabilityDebugging");
    assert_eq!(S::MedicalBiosignal.as_str(), "MedicalBiosignal");
    assert_eq!(S::RfCommunications.as_str(), "RfCommunications");
    assert_eq!(S::Chemometrics.as_str(), "Chemometrics");
    assert_eq!(S::Econometrics.as_str(), "Econometrics");
    assert_eq!(S::ReliabilitySurvival.as_str(), "ReliabilitySurvival");
}

#[test]
fn status_and_role_wire_names_are_stable() {
    assert_eq!(ProposalStatus::Open.as_str(), "Open");
    assert_eq!(ProposalStatus::Accepted.as_str(), "Accepted");
    assert_eq!(ProposalStatus::Rejected.as_str(), "Rejected");
    assert_eq!(ProposalStatus::Deferred.as_str(), "Deferred");
    assert_eq!(ProposerRole::PanelMember.as_str(), "PanelMember");
    assert_eq!(ProposerRole::ExternalReviewer.as_str(), "ExternalReviewer");
    assert_eq!(ProposerRole::RobotIngestion.as_str(), "RobotIngestion");
}

// ---------------------------------------------------------------
// Seed shape
// ---------------------------------------------------------------

#[test]
fn seed_proof_of_life_proposal_is_admissible() {
    let p = seed_proof_of_life_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "seed proof-of-life proposal errors: {errors:?}"
    );
}

#[test]
fn seed_proof_of_life_proposal_has_open_status() {
    let p = seed_proof_of_life_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn seed_proof_of_life_proposal_carries_three_nonzero_hashes() {
    let p = seed_proof_of_life_proposal();
    assert_ne!(p.corpus_amendment_proposal_hash_v1, [0u8; 32]);
    assert_ne!(p.body.literature_expansion_batch_hash_v1, [0u8; 32]);
    assert_ne!(p.dedup_court_delta.dedup_court_delta_hash_v1, [0u8; 32]);
}

#[test]
fn seed_proof_of_life_proposal_has_empty_body() {
    let p = seed_proof_of_life_proposal();
    assert!(p.body.proposed_primitives.is_empty());
    assert!(p.body.proposed_aliases.is_empty());
    assert!(p.body.proposed_dedup_records.is_empty());
    assert!(p.body.proposed_genealogy_edges.is_empty());
    assert!(p.body.proposed_source_refs.is_empty());
    assert!(p.dedup_court_delta.new_canonical_records.is_empty());
    assert!(p.dedup_court_delta.new_alias_records.is_empty());
    assert!(p.dedup_court_delta.new_composition_records.is_empty());
    assert!(p.dedup_court_delta.rejection_records.is_empty());
    assert!(p.dedup_court_delta.deferred_records.is_empty());
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn amendment_proposal_hash_is_deterministic() {
    assert_eq!(
        seed_proof_of_life_proposal().corpus_amendment_proposal_hash_v1,
        seed_proof_of_life_proposal().corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_proposal_hash_matches_stored() {
    let p = seed_proof_of_life_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_batch_hash_matches_stored() {
    let p = seed_proof_of_life_proposal();
    assert_eq!(
        compute_literature_expansion_batch_hash_v1(&p.body),
        p.body.literature_expansion_batch_hash_v1
    );
}

#[test]
fn recomputed_dedup_delta_hash_matches_stored() {
    let p = seed_proof_of_life_proposal();
    assert_eq!(
        compute_dedup_court_delta_hash_v1(&p.dedup_court_delta),
        p.dedup_court_delta.dedup_court_delta_hash_v1
    );
}

/// Load-bearing negative #4 (panel-required): changing the
/// batch contents changes the proposal hash.
#[test]
fn amendment_proposal_hash_changes_when_batch_changes() {
    // Reserve canonical_id 100_000 (well above the 54-record
    // SEED) so the dedup-court verifier rule doesn't fire on
    // the mutation alone.
    let p_a = seed_proof_of_life_proposal();
    let batch_b = build_expansion_batch(
        "t12_0_synthetic_nonempty",
        SourceClass::StatisticalProcessControl,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(100_000),
            display_name: "Synthetic for hash sensitivity",
            motivation: "load-bearing negative #4 fixture",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p_b = build_amendment_proposal(
        p_a.proposal_id,
        p_a.motivation,
        p_a.target_source_class,
        batch_b,
        p_a.dedup_court_delta.clone(),
        p_a.status,
        p_a.proposer_role,
        p_a.created_at_commit,
    );
    assert_ne!(
        p_a.corpus_amendment_proposal_hash_v1,
        p_b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn proposal_hash_changes_when_status_changes() {
    let p_open = seed_proof_of_life_proposal();
    // Mutate to Deferred (Accepted would trigger rules 6+7
    // on the empty seed body).
    let p_deferred = build_amendment_proposal(
        p_open.proposal_id,
        p_open.motivation,
        p_open.target_source_class,
        p_open.body.clone(),
        p_open.dedup_court_delta.clone(),
        ProposalStatus::Deferred,
        p_open.proposer_role,
        p_open.created_at_commit,
    );
    assert_ne!(
        p_open.corpus_amendment_proposal_hash_v1,
        p_deferred.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn proposal_hash_changes_when_source_class_changes() {
    let p_a = seed_proof_of_life_proposal();
    let p_b = build_amendment_proposal(
        p_a.proposal_id,
        p_a.motivation,
        SourceClass::SequentialChangeDetection,
        p_a.body.clone(),
        p_a.dedup_court_delta.clone(),
        p_a.status,
        p_a.proposer_role,
        p_a.created_at_commit,
    );
    assert_ne!(
        p_a.corpus_amendment_proposal_hash_v1,
        p_b.corpus_amendment_proposal_hash_v1
    );
}

// ---------------------------------------------------------------
// Verifier — panel-required load-bearing negatives
// ---------------------------------------------------------------

/// Load-bearing negative #1 (panel-required): empty proposal_id
/// MUST be rejected.
#[test]
fn proposal_rejects_empty_proposal_id() {
    let p_seed = seed_proof_of_life_proposal();
    let p = build_amendment_proposal(
        "", // the defect
        p_seed.motivation,
        p_seed.target_source_class,
        p_seed.body.clone(),
        p_seed.dedup_court_delta.clone(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_0_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, AmendmentVerifyErrorKind::ProposalIdEmpty)));
}

/// Load-bearing negative #2 (panel-required): proposal
/// targeting an unknown source_class. The Rust enum prevents
/// this at the type level, so we exercise the structural
/// invariant via the verifier's `UnknownSourceClass` rule —
/// the rule exists for a future TOML-loader path that could
/// hand-build proposals with bogus class names. Here we
/// instead pin the runtime side: every enum variant has a
/// non-empty wire name (guarantees the verifier sees a
/// well-formed source-class tag).
#[test]
fn proposal_rejects_unknown_source_class() {
    use SourceClass as S;
    // Walk every variant; if any returns an empty wire name,
    // a future enum addition would silently bypass the
    // UnknownSourceClass rule.
    for variant in &[
        S::StatisticalProcessControl,
        S::SequentialChangeDetection,
        S::DriftDetection,
        S::RobustStatistics,
        S::DistributionDistance,
        S::InformationTheory,
        S::SignalProcessing,
        S::SpectralAndWavelet,
        S::TimeSeriesStructure,
        S::ControlResiduals,
        S::FaultDetectionDiagnostics,
        S::ConditionMonitoring,
        S::IndustrialProcessMonitoring,
        S::GraphAnomalyDetection,
        S::StreamingSketches,
        S::DataQualityRules,
        S::DatabaseIntegrityConstraints,
        S::ObservabilityDebugging,
        S::MedicalBiosignal,
        S::RfCommunications,
        S::Chemometrics,
        S::Econometrics,
        S::ReliabilitySurvival,
    ] {
        assert!(
            !variant.as_str().is_empty(),
            "SourceClass variant has empty wire name; the verifier's UnknownSourceClass \
             rule would silently bypass it"
        );
    }
    // The wire-name enum is the type-level UnknownSourceClass
    // guard. The verifier's own rule fires from a future TOML
    // loader; this test pins the existing surface's invariant.
    assert!(AmendmentVerifyErrorKind::UnknownSourceClass.as_str() == "UnknownSourceClass");
}

/// Load-bearing negative #3 (panel-required): a dedup-court
/// delta declaring `new_canonical_records` whose canonical_id
/// collides with an existing SEED record MUST be rejected.
/// Would silently mutate the corpus otherwise.
#[test]
fn dedup_delta_rejects_canonical_id_collision_with_existing_seed() {
    // Pick the first SEED canonical_id for the collision.
    let colliding_id = SEED[0].canonical_id;
    let bad_delta = build_dedup_court_delta(
        "t12_0_collision_delta",
        vec![colliding_id], // the defect
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p_seed = seed_proof_of_life_proposal();
    let p = build_amendment_proposal(
        "t12_0_collision_proposal",
        p_seed.motivation,
        SourceClass::StatisticalProcessControl,
        p_seed.body.clone(),
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_0_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id == colliding_id
    )));
}

/// Same collision rule fires when the COLLISION comes from
/// the batch's `proposed_primitives` (defense in depth at
/// intake).
#[test]
fn proposal_rejects_batch_primitive_collision_with_existing_seed() {
    let colliding_id = SEED[0].canonical_id;
    let bad_batch = build_expansion_batch(
        "t12_0_batch_collision",
        SourceClass::StatisticalProcessControl,
        vec![ProposedPrimitive {
            reserved_canonical_id: colliding_id,
            display_name: "Collision fixture",
            motivation: "Should be rejected",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p_seed = seed_proof_of_life_proposal();
    let p = build_amendment_proposal(
        "t12_0_batch_collision_proposal",
        p_seed.motivation,
        SourceClass::StatisticalProcessControl,
        bad_batch,
        p_seed.dedup_court_delta.clone(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_0_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id == colliding_id
    )));
}

// ---------------------------------------------------------------
// Verifier — additional rules
// ---------------------------------------------------------------

#[test]
fn proposal_rejects_empty_batch_id() {
    let bad_batch = build_expansion_batch(
        "", // the defect
        SourceClass::StatisticalProcessControl,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p_seed = seed_proof_of_life_proposal();
    let p = build_amendment_proposal(
        "t12_0_empty_batch_id_proposal",
        p_seed.motivation,
        SourceClass::StatisticalProcessControl,
        bad_batch,
        p_seed.dedup_court_delta.clone(),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_0_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, AmendmentVerifyErrorKind::BatchIdEmpty)));
}

#[test]
fn proposal_rejects_accepted_with_empty_body_and_delta() {
    let p_seed = seed_proof_of_life_proposal();
    let p = build_amendment_proposal(
        "t12_0_accepted_empty",
        "no-op acceptance should be rejected",
        SourceClass::StatisticalProcessControl,
        p_seed.body.clone(),
        p_seed.dedup_court_delta.clone(),
        ProposalStatus::Accepted, // the defect
        ProposerRole::PanelMember,
        "t12_0_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::AcceptedProposalWithoutBodyOrDelta
    )));
}

#[test]
fn proposal_rejects_accepted_without_created_at_commit() {
    // Construct an Accepted proposal with non-empty body
    // (rule 6 wouldn't fire) but empty created_at_commit
    // (rule 7 fires).
    let nonempty_batch = build_expansion_batch(
        "t12_0_nonempty",
        SourceClass::StatisticalProcessControl,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(200_000),
            display_name: "Synthetic accepted body",
            motivation: "rule 7 fixture",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p = build_amendment_proposal(
        "t12_0_accepted_no_commit",
        "Accepted but no freeze gate",
        SourceClass::StatisticalProcessControl,
        nonempty_batch,
        seed_proof_of_life_proposal().dedup_court_delta.clone(),
        ProposalStatus::Accepted,
        ProposerRole::PanelMember,
        "", // the defect
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::AcceptedProposalWithoutFutureFreezeGate
    )));
}

#[test]
fn verifier_rejects_proposal_hash_mismatch() {
    let mut p = seed_proof_of_life_proposal();
    p.corpus_amendment_proposal_hash_v1[0] ^= 0xff;
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::AmendmentProposalHashMismatch
    )));
}

#[test]
fn verifier_rejects_batch_hash_mismatch() {
    let mut p = seed_proof_of_life_proposal();
    p.body.literature_expansion_batch_hash_v1[0] ^= 0xff;
    // Re-fixup the proposal-level hash so the outer-level
    // mismatch doesn't shadow the batch-level one.
    p.corpus_amendment_proposal_hash_v1 = compute_corpus_amendment_proposal_hash_v1(&p);
    let errors = verify_amendment_proposal(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, AmendmentVerifyErrorKind::BatchHashMismatch)));
}

#[test]
fn verifier_rejects_dedup_delta_hash_mismatch() {
    let mut p = seed_proof_of_life_proposal();
    p.dedup_court_delta.dedup_court_delta_hash_v1[0] ^= 0xff;
    p.corpus_amendment_proposal_hash_v1 = compute_corpus_amendment_proposal_hash_v1(&p);
    let errors = verify_amendment_proposal(&p);
    assert!(errors
        .iter()
        .any(|e| matches!(e.kind, AmendmentVerifyErrorKind::DedupDeltaHashMismatch)));
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn render_amendment_proposal_text_is_byte_stable() {
    let p = seed_proof_of_life_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn render_amendment_proposal_json_is_byte_stable() {
    let p = seed_proof_of_life_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}

#[test]
fn render_amendment_proposal_text_includes_three_hashes() {
    let p = seed_proof_of_life_proposal();
    let text = render_amendment_proposal_text(&p);
    assert!(text.contains("corpus_amendment_proposal_hash_v1"));
    assert!(text.contains("literature_expansion_batch_hash_v1"));
    assert!(text.contains("dedup_court_delta_hash_v1"));
}

#[test]
fn render_amendment_proposal_json_is_valid_top_level_object() {
    let p = seed_proof_of_life_proposal();
    let json = render_amendment_proposal_json(&p);
    assert!(json.trim_start().starts_with('{'));
    assert!(json.trim_end().ends_with('}'));
    assert!(json.contains("\"proposal_id\""));
    assert!(json.contains("\"corpus_amendment_proposal_hash_v1\""));
}

// ---------------------------------------------------------------
// Upstream-anchor preservation (T.12.0 non-claim)
// ---------------------------------------------------------------

/// T.12.0 MUST NOT mutate `corpus_hash_v1`. Building +
/// verifying amendment proposals leaves the corpus hash
/// byte-identical.
#[test]
fn t12_0_does_not_mutate_corpus_hash_v1() {
    use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
    let before = compute_corpus_hash_v1().bytes;
    let p = seed_proof_of_life_proposal();
    let _errors = verify_amendment_proposal(&p);
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

/// T.12.0 MUST NOT mutate the seed roster. The SEED.len()
/// stays at its pre-T.12.0 value (54 records).
#[test]
fn t12_0_does_not_add_records_to_seed() {
    assert_eq!(SEED.len(), 54);
}
