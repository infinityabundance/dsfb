//! T.12.b acceptance suite — Sequential Change Detection
//! expansion proposal invariants.
//!
//! Fourteen panel-required load-bearing negatives pin the
//! cross-class dedup authority discipline that T.12.b exists to
//! prove. Additional invariants pin shape, determinism,
//! rendering byte-stability, and SEED non-mutation.
//!
//! The headline is **cross-class dedup authority**, not
//! detector quantity (panel-locked): seven existing SEED records
//! (CUSUM, Page-Hinkley, Mann-Kendall, Pettitt, SNHT, MOSUM,
//! Buishand range) must not be re-canonicalised under reserved
//! 5xxx ids; BOCPD must not be silently admitted to
//! `new_canonical_records`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    compute_corpus_amendment_proposal_hash_v1, render_amendment_proposal_json,
    render_amendment_proposal_text, seed_proof_of_life_proposal, verify_amendment_proposal,
    AmendmentVerifyErrorKind, ProposalStatus, ProposedDedupRecord, ProposedPrimitive, ProposerRole,
    RejectionRecord, SourceClass,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::t12_a_spc::seed_t12_a_spc_proposal;
use dsfb_gpu_atlas_corpus::t12_b_scd::{
    seed_t12_b_scd_proposal, BINARY_SEGMENTATION_RESERVED_CANONICAL_ID,
    BOCPD_RESERVED_PRIMITIVE_ID, BUISHAND_SEED_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_DOMAIN_TRANSFER_OF, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, CUSUM_SEED_ID, GLR_RESERVED_CANONICAL_ID,
    MANN_KENDALL_SEED_ID, MOSUM_SEED_ID, PAGE_HINKLEY_SEED_ID, PELT_RESERVED_CANONICAL_ID,
    PETTITT_SEED_ID, SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID, SNHT_SEED_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn scd_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_b_scd_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "SCD proposal failed verifier: {errors:?}"
    );
}

#[test]
fn scd_proposal_has_open_status() {
    let p = seed_t12_b_scd_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn scd_proposal_targets_sequential_change_detection() {
    let p = seed_t12_b_scd_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::SequentialChangeDetection
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_b_does_not_mutate_seed_len`. T.12.b is a docketed legal
/// act on the amendment court, not a corpus mutation. SEED stays
/// at 54.
#[test]
fn t12_b_does_not_mutate_seed_len() {
    let _ = seed_t12_b_scd_proposal();
    assert_eq!(SEED.len(), 54);
}

/// Shape: T.12.b proposes 5 primitive shells (4 new canonicals
/// + 1 BOCPD rejection shell).
#[test]
fn scd_proposal_proposes_five_primitives() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 5);
    let ids: Vec<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    assert!(ids.contains(&SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&GLR_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&BINARY_SEGMENTATION_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&PELT_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&BOCPD_RESERVED_PRIMITIVE_ID));
}

/// Shape: T.12.b proposes ZERO aliases. Cross-class authority is
/// encoded via `proposed_dedup_records` entries with the
/// `ExistingCanonicalAuthorityResolution` wire name, not via
/// `ProposedAliasClaim` (that pattern is T.12.a's).
#[test]
fn scd_proposal_proposes_zero_aliases() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

/// Shape: T.12.b proposes 13 dedup-court records spanning four
/// court-delta categories (4 CanonicalAddition + 7
/// ExistingCanonicalAuthorityResolution + 1 DomainTransferOf +
/// 1 RejectedNotDeterministic).
#[test]
fn scd_proposal_proposes_thirteen_dedup_records() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 13);
}

/// Shape: T.12.b proposes 6 genealogy edges (4 DerivedFrom-CUSUM
/// + 2 Generalizes among the new canonicals).
#[test]
fn scd_proposal_proposes_six_genealogy_edges() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 6);
}

/// Shape: T.12.b proposes 6 source-refs (one per new canonical
/// plus a BOCPD reference).
#[test]
fn scd_proposal_proposes_six_source_refs() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 6);
}

/// Shape: T.12.b's delta admits exactly FOUR new canonical
/// records — NOT five (BOCPD is deliberately absent).
#[test]
fn scd_delta_has_four_new_canonical_records() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 4);
    let ids: Vec<u32> = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .map(|c| c.0)
        .collect();
    assert!(ids.contains(&SHIRYAEV_ROBERTS_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&GLR_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&BINARY_SEGMENTATION_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&PELT_RESERVED_CANONICAL_ID));
}

/// Shape: T.12.b emits exactly four court-delta categories. The
/// panel-locked decision-category wire-name set is closed.
#[test]
fn scd_proposal_emits_four_court_delta_categories() {
    let p = seed_t12_b_scd_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert!(categories.contains(CATEGORY_CANONICAL_ADDITION));
    assert!(categories.contains(CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION));
    assert!(categories.contains(CATEGORY_DOMAIN_TRANSFER_OF));
    assert!(categories.contains(CATEGORY_REJECTED_NOT_DETERMINISTIC));
    assert_eq!(categories.len(), 4);
}

/// Shape: counts per court-delta category match the
/// panel-locked headline (4 / 7 / 1 / 1).
#[test]
fn scd_proposal_court_delta_category_counts() {
    let p = seed_t12_b_scd_proposal();
    let mut canonical = 0;
    let mut existing = 0;
    let mut transfer = 0;
    let mut rejected = 0;
    for r in &p.body.proposed_dedup_records {
        match r.decision_wire_name {
            CATEGORY_CANONICAL_ADDITION => canonical += 1,
            CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION => existing += 1,
            CATEGORY_DOMAIN_TRANSFER_OF => transfer += 1,
            CATEGORY_REJECTED_NOT_DETERMINISTIC => rejected += 1,
            other => panic!("unexpected category wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 4, "expected 4 CanonicalAddition records");
    assert_eq!(
        existing, 7,
        "expected 7 ExistingCanonicalAuthorityResolution"
    );
    assert_eq!(transfer, 1, "expected 1 DomainTransferOf");
    assert_eq!(rejected, 1, "expected 1 RejectedNotDeterministic");
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn scd_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_b_scd_proposal();
    let b = seed_t12_b_scd_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_scd_proposal_hash_matches_stored() {
    let p = seed_t12_b_scd_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn scd_proposal_hash_is_distinct_from_t12_0_and_t12_a() {
    let scd = seed_t12_b_scd_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    assert_ne!(
        scd.corpus_amendment_proposal_hash_v1, pol.corpus_amendment_proposal_hash_v1,
        "T.12.b hash must differ from T.12.0 proof-of-life hash"
    );
    assert_ne!(
        scd.corpus_amendment_proposal_hash_v1, spc.corpus_amendment_proposal_hash_v1,
        "T.12.b hash must differ from T.12.a SPC hash"
    );
}

/// Load-bearing negative #8 (panel-required):
/// `t12_b_hash_changes_when_cross_class_authority_changes`.
/// Mutating one cross-class authority record's reason text
/// changes the batch hash and therefore the proposal hash.
#[test]
fn t12_b_hash_changes_when_cross_class_authority_changes() {
    let p_a = seed_t12_b_scd_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    // Find the Page-Hinkley cross-class authority record and
    // mutate its reason text.
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == PAGE_HINKLEY_SEED_ID
        })
        .expect("Page-Hinkley cross-class authority record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED authority text for hash-sensitivity test",
    };
    let new_batch = build_expansion_batch(
        p_a.body.batch_id,
        p_a.body.source_class,
        p_a.body.proposed_primitives.clone(),
        p_a.body.proposed_aliases.clone(),
        records,
        p_a.body.proposed_genealogy_edges.clone(),
        p_a.body.proposed_source_refs.clone(),
    );
    let p_b = build_amendment_proposal(
        p_a.proposal_id,
        p_a.motivation,
        p_a.target_source_class,
        new_batch,
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
// Cross-class dedup-authority load-bearing negatives
// ---------------------------------------------------------------
//
// These seven tests each construct a malformed variant of the
// T.12.b proposal in which an existing SEED canonical id is
// silently re-canonicalised under reserved 5xxx ids. The T.12.0
// verifier's `DedupDeltaCollidesWithExistingSeedCanonicalId`
// rule fires for each. The tests pin the "do not duplicate"
// discipline panel-required for T.12.b's headline.

/// Construct a defective T.12.b-style proposal that puts the
/// given existing SEED canonical id into the dedup-delta's
/// `new_canonical_records` AND into a proposed-primitive shell.
/// Returns the proposal so the test can call the verifier and
/// assert the collision rule fires.
fn build_defective_cross_class_proposal_with_seed_collision(
    seed_id: u32,
    test_label: &'static str,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_b_cross_class_collision_batch",
        SourceClass::SequentialChangeDetection,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: test_label,
            motivation: "Should be rejected — id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_b_cross_class_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_b_cross_class_collision_proposal",
        "Defective T.12.b-style proposal duplicating an existing SEED canonical.",
        SourceClass::SequentialChangeDetection,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_b_test",
    )
}

/// Load-bearing negative #2 (panel-required):
/// `t12_b_rejects_duplicate_cusum_without_domain_transfer_or_existing_target`.
#[test]
fn t12_b_rejects_duplicate_cusum_without_domain_transfer_or_existing_target() {
    let p = build_defective_cross_class_proposal_with_seed_collision(CUSUM_SEED_ID, "CUSUM");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == CUSUM_SEED_ID
    )));
}

/// Load-bearing negative #3 (panel-required):
/// `t12_b_rejects_duplicate_page_hinkley_without_cross_class_resolution`.
#[test]
fn t12_b_rejects_duplicate_page_hinkley_without_cross_class_resolution() {
    let p = build_defective_cross_class_proposal_with_seed_collision(
        PAGE_HINKLEY_SEED_ID,
        "Page-Hinkley",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == PAGE_HINKLEY_SEED_ID
    )));
}

/// Load-bearing negative #4 (panel-required):
/// `t12_b_rejects_mann_kendall_as_new_canonical_when_seed_id_exists`.
/// Mann-Kendall is a TREND witness, not a generic SCD primitive
/// — the court catches the relabelling attempt.
#[test]
fn t12_b_rejects_mann_kendall_as_new_canonical_when_seed_id_exists() {
    let p = build_defective_cross_class_proposal_with_seed_collision(
        MANN_KENDALL_SEED_ID,
        "Mann-Kendall (relabelled as SCD)",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == MANN_KENDALL_SEED_ID
    )));
}

/// Load-bearing negative #5 (panel-required, new in T.12.b after
/// SEED walk): `t12_b_rejects_duplicate_pettitt_when_seed_id_exists`.
/// Pettitt is already canonical in SEED at id 34; the panel's
/// draft list would have collided here without this guard.
#[test]
fn t12_b_rejects_duplicate_pettitt_when_seed_id_exists() {
    let p = build_defective_cross_class_proposal_with_seed_collision(PETTITT_SEED_ID, "Pettitt");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == PETTITT_SEED_ID
    )));
}

/// Load-bearing negative #6 (panel-required, new in T.12.b after
/// SEED walk): `t12_b_rejects_duplicate_snht_when_seed_id_exists`.
#[test]
fn t12_b_rejects_duplicate_snht_when_seed_id_exists() {
    let p = build_defective_cross_class_proposal_with_seed_collision(SNHT_SEED_ID, "SNHT");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == SNHT_SEED_ID
    )));
}

/// Load-bearing negative #7 (panel-required, new in T.12.b after
/// SEED walk): `t12_b_rejects_duplicate_mosum_when_seed_id_exists`.
#[test]
fn t12_b_rejects_duplicate_mosum_when_seed_id_exists() {
    let p = build_defective_cross_class_proposal_with_seed_collision(MOSUM_SEED_ID, "MOSUM");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == MOSUM_SEED_ID
    )));
}

/// Load-bearing negative (new in T.12.b after SEED walk):
/// `t12_b_rejects_duplicate_buishand_when_seed_id_exists`.
#[test]
fn t12_b_rejects_duplicate_buishand_when_seed_id_exists() {
    let p = build_defective_cross_class_proposal_with_seed_collision(
        BUISHAND_SEED_ID,
        "Buishand range",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == BUISHAND_SEED_ID
    )));
}

// ---------------------------------------------------------------
// BOCPD rejection load-bearing negatives
// ---------------------------------------------------------------

/// Load-bearing negative (panel-required):
/// `t12_b_rejects_bocpd_as_canonical_without_deterministic_reduction_status`.
/// The actual T.12.b delta MUST NOT contain BOCPD's reserved id
/// in `new_canonical_records`. Encoding the panel-locked rule
/// as a static assertion against the canonical seed pins the
/// invariant for every future build.
#[test]
fn t12_b_rejects_bocpd_as_canonical_without_deterministic_reduction_status() {
    let p = seed_t12_b_scd_proposal();
    let bocpd_in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BOCPD_RESERVED_PRIMITIVE_ID);
    assert!(
        !bocpd_in_canonical,
        "BOCPD (id {BOCPD_RESERVED_PRIMITIVE_ID}) must NOT appear in new_canonical_records \
         without a declared deterministic reduction"
    );
}

/// Sibling assertion: BOCPD MUST appear in `proposed_primitives`
/// so the proposal documents the literature record explicitly
/// (silent omission would be dishonest).
#[test]
fn t12_b_bocpd_present_in_proposed_primitives_but_not_in_new_canonical_records() {
    let p = seed_t12_b_scd_proposal();
    let in_primitives = p
        .body
        .proposed_primitives
        .iter()
        .any(|pr| pr.reserved_canonical_id.0 == BOCPD_RESERVED_PRIMITIVE_ID);
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BOCPD_RESERVED_PRIMITIVE_ID);
    assert!(in_primitives, "BOCPD must appear in proposed_primitives");
    assert!(
        !in_canonical,
        "BOCPD must NOT appear in new_canonical_records"
    );
}

/// Sibling assertion: BOCPD MUST be the subject of a
/// `RejectedNotDeterministic` dedup record so the rejection is
/// explicit on the court delta.
#[test]
fn t12_b_bocpd_has_rejected_not_deterministic_dedup_record() {
    let p = seed_t12_b_scd_proposal();
    let bocpd_rejection_record = p.body.proposed_dedup_records.iter().any(|r| {
        r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
            && r.canonical_id.0 == BOCPD_RESERVED_PRIMITIVE_ID
    });
    assert!(bocpd_rejection_record);
}

// ---------------------------------------------------------------
// Other panel-required negatives
// ---------------------------------------------------------------

/// Load-bearing negative (panel-required):
/// `t12_b_rejects_alias_without_existing_target`. T.12.b
/// proposes ZERO aliases, but the invariant must still hold for
/// every future T.12.x sub-campaign that inherits this test
/// surface: every alias claim's `collapses_into` resolves to an
/// existing SEED canonical or to a primitive proposed in the
/// same batch. Vacuously true here; encodes the rule.
#[test]
fn t12_b_rejects_alias_without_existing_target() {
    let p = seed_t12_b_scd_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let batch_new_ids: std::collections::BTreeSet<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    for a in &p.body.proposed_aliases {
        let target = a.collapses_into.0;
        assert!(
            seed_ids.contains(&target) || batch_new_ids.contains(&target),
            "alias {} targets canonical_id {} which is neither in SEED nor a new primitive",
            a.alias_name,
            target
        );
    }
}

/// Load-bearing negative (panel-required):
/// `t12_b_rejects_domain_transfer_without_existing_target`.
/// Every `DomainTransferOf` dedup record MUST reference an
/// existing SEED canonical id (the domain-transfer target must
/// exist in the corpus).
#[test]
fn t12_b_rejects_domain_transfer_without_existing_target() {
    let p = seed_t12_b_scd_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "DomainTransferOf record references canonical_id {} not present in SEED",
                r.canonical_id.0
            );
        }
    }
}

/// Sibling load-bearing assertion: every
/// `ExistingCanonicalAuthorityResolution` record's `canonical_id`
/// MUST resolve to an existing SEED canonical. Pinning the
/// rule explicitly catches future T.12.x sub-campaigns that
/// accidentally use the wire name for a non-SEED id.
#[test]
fn t12_b_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_b_scd_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "ExistingCanonicalAuthorityResolution record references canonical_id {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

/// Load-bearing negative (panel-required):
/// `t12_b_every_new_canonical_has_source_ref`. Every reserved
/// canonical id in the dedup-court delta's
/// `new_canonical_records` must be supported by at least one
/// source-ref in the proposal's batch. Check via the source-ref
/// count meeting or exceeding the canonical-addition count.
#[test]
fn t12_b_every_new_canonical_has_source_ref() {
    let p = seed_t12_b_scd_proposal();
    assert!(
        p.body.proposed_source_refs.len() >= p.dedup_court_delta.new_canonical_records.len(),
        "T.12.b has {} new canonicals but only {} source refs",
        p.dedup_court_delta.new_canonical_records.len(),
        p.body.proposed_source_refs.len()
    );
    // Defence in depth: every source ref has a non-empty
    // citation_key + title.
    for s in &p.body.proposed_source_refs {
        assert!(
            !s.citation_key.is_empty(),
            "source ref must have a citation_key"
        );
        assert!(!s.title.is_empty(), "source ref must have a title");
        assert!(!s.venue.is_empty(), "source ref must have a venue");
    }
}

/// Load-bearing negative (panel-required):
/// `t12_b_every_rejection_has_reason_code`. Every
/// `RejectedNotDeterministic` dedup record MUST carry a
/// non-empty reason string.
#[test]
fn t12_b_every_rejection_has_reason_code() {
    let p = seed_t12_b_scd_proposal();
    let rejection_records: Vec<&ProposedDedupRecord> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .collect();
    assert!(
        !rejection_records.is_empty(),
        "T.12.b must carry at least one rejection record"
    );
    for r in rejection_records {
        assert!(
            !r.reason.is_empty(),
            "RejectedNotDeterministic record at canonical_id {} has empty reason",
            r.canonical_id.0
        );
    }
}

/// Sibling load-bearing assertion: every `CanonicalAddition`
/// record's `canonical_id` is in the 5201..=5208 reserved range
/// for T.12.b. Catches accidental id leakage outside the
/// panel-locked range.
#[test]
fn t12_b_canonical_addition_ids_are_in_5201_to_5208_range() {
    let p = seed_t12_b_scd_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5201..=5208).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.b reserved range 5201..=5208",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn scd_proposal_text_rendering_byte_stable() {
    let p = seed_t12_b_scd_proposal();
    let a = render_amendment_proposal_text(&p);
    let b = render_amendment_proposal_text(&p);
    assert_eq!(a, b);
}

#[test]
fn scd_proposal_json_rendering_byte_stable() {
    let p = seed_t12_b_scd_proposal();
    let a = render_amendment_proposal_json(&p);
    let b = render_amendment_proposal_json(&p);
    assert_eq!(a, b);
}
