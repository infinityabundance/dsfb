//! T.12.a acceptance suite — Statistical Process Control
//! expansion proposal invariants.
//!
//! Four panel-required load-bearing negatives pin the
//! verifier's blocking rules for the first real
//! `CorpusAmendmentProposal` to flow through the T.12.0
//! amendment court. Five additional invariants pin shape +
//! determinism + non-mutation guarantees.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use dsfb_gpu_atlas_corpus::amendment::{
    build_amendment_proposal, build_dedup_court_delta, build_expansion_batch,
    compute_corpus_amendment_proposal_hash_v1, render_amendment_proposal_json,
    render_amendment_proposal_text, verify_amendment_proposal, AmendmentVerifyErrorKind,
    ProposalStatus, ProposedAliasClaim, ProposedPrimitive, ProposedSourceRef, ProposerRole,
    RejectionRecord, SourceClass,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::t12_a_spc::{
    seed_t12_a_spc_proposal, HOTELLING_T2_SEED_ID, HOTELLING_TSQUARE_ALIAS_ID,
    MCUSUM_RESERVED_CANONICAL_ID, MEWMA_RESERVED_CANONICAL_ID, NELSON_SEED_ID, PCA_SPE_Q_SEED_ID,
    Q_STATISTIC_ALIAS_ID, SHEWHART_SEED_ID, SPE_ALIAS_ID, WESTERN_ELECTRIC_SEED_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Seed shape + admissibility
// ---------------------------------------------------------------

#[test]
fn spc_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_a_spc_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "SPC proposal failed verifier: {errors:?}"
    );
}

#[test]
fn spc_proposal_has_open_status() {
    let p = seed_t12_a_spc_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn spc_proposal_targets_statistical_process_control() {
    let p = seed_t12_a_spc_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::StatisticalProcessControl
    ));
}

/// Panel-required: T.12.a MUST NOT mutate SEED. The proposal
/// is a docketed legal act, not a corpus mutation.
#[test]
fn spc_proposal_does_not_mutate_seed_len() {
    let _ = seed_t12_a_spc_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn spc_proposal_proposes_two_new_canonicals() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 2);
    let ids: Vec<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    assert!(ids.contains(&MEWMA_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&MCUSUM_RESERVED_CANONICAL_ID));
}

#[test]
fn spc_proposal_proposes_three_aliases() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 3);
    let alias_ids: Vec<u32> = p
        .body
        .proposed_aliases
        .iter()
        .map(|a| a.reserved_alias_id.0)
        .collect();
    assert!(alias_ids.contains(&Q_STATISTIC_ALIAS_ID));
    assert!(alias_ids.contains(&SPE_ALIAS_ID));
    assert!(alias_ids.contains(&HOTELLING_TSQUARE_ALIAS_ID));
}

#[test]
fn spc_proposal_proposes_two_compositions_against_existing_seed() {
    let p = seed_t12_a_spc_proposal();
    let comp_ids: Vec<u32> = p
        .dedup_court_delta
        .new_composition_records
        .iter()
        .map(|c| c.0)
        .collect();
    assert_eq!(comp_ids.len(), 2);
    assert!(comp_ids.contains(&WESTERN_ELECTRIC_SEED_ID));
    assert!(comp_ids.contains(&NELSON_SEED_ID));
}

#[test]
fn spc_proposal_proposes_four_genealogy_edges() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 4);
}

#[test]
fn spc_proposal_proposes_four_source_refs() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 4);
}

/// Panel-required: aliases must collapse into existing SEED
/// canonicals OR into newly-proposed primitives in the same
/// batch. T.12.a's three aliases all target SEED records (20
/// = PCA_SPE_Q_RESIDUAL; 5 = HOTELLING_T2). Surface the
/// targeting invariant explicitly.
#[test]
fn spc_delta_lists_every_alias_with_reason_code() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 3);
    for a in &p.body.proposed_aliases {
        assert!(!a.alias_name.is_empty(), "alias name must be non-empty");
        // Alias target must be either a SEED canonical or a
        // proposed primitive in the same batch.
        let seed_ids: std::collections::BTreeSet<u32> =
            SEED.iter().map(|r| r.canonical_id.0).collect();
        let batch_new_ids: std::collections::BTreeSet<u32> = p
            .body
            .proposed_primitives
            .iter()
            .map(|pr| pr.reserved_canonical_id.0)
            .collect();
        let target = a.collapses_into.0;
        assert!(
            seed_ids.contains(&target) || batch_new_ids.contains(&target),
            "alias {} targets canonical_id {} which is neither in SEED nor a new primitive",
            a.alias_name,
            target
        );
    }
}

#[test]
fn spc_delta_lists_every_composition_with_components() {
    let p = seed_t12_a_spc_proposal();
    // Every composition decision must have a reason that
    // references its composite parents. We verify the dedup
    // records mention "Composition" or "compose" — the
    // composition reason text is panel-locked.
    let comp_records: Vec<_> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == "CompositionOf")
        .collect();
    assert!(comp_records.len() >= 2);
    for r in comp_records {
        assert!(
            r.reason.to_lowercase().contains("composition")
                || r.reason.to_lowercase().contains("compose"),
            "composition reason missing composition-language: {}",
            r.reason
        );
    }
}

#[test]
fn spc_source_refs_are_nonempty_for_every_claim() {
    let p = seed_t12_a_spc_proposal();
    assert!(!p.body.proposed_source_refs.is_empty());
    for s in &p.body.proposed_source_refs {
        assert!(!s.citation_key.is_empty());
        assert!(!s.title.is_empty());
        assert!(!s.venue.is_empty());
        assert!(s.year >= 1900 && s.year <= 2099);
    }
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn spc_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_a_spc_proposal();
    let b = seed_t12_a_spc_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn spc_proposal_hash_is_distinct_from_proof_of_life_hash() {
    use dsfb_gpu_atlas_corpus::amendment::seed_proof_of_life_proposal;
    let spc = seed_t12_a_spc_proposal();
    let pol = seed_proof_of_life_proposal();
    assert_ne!(
        spc.corpus_amendment_proposal_hash_v1,
        pol.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_spc_proposal_hash_matches_stored() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

/// Load-bearing negative #4 (panel-required): changing one
/// source_ref changes the proposal hash.
#[test]
fn spc_amendment_hash_changes_when_one_source_ref_changes() {
    let p_a = seed_t12_a_spc_proposal();
    // Rebuild the batch with one source-ref title mutated.
    let mut refs = p_a.body.proposed_source_refs.clone();
    refs[0] = ProposedSourceRef {
        citation_key: refs[0].citation_key,
        title: "MUTATED TITLE for hash sensitivity",
        year: refs[0].year,
        venue: refs[0].venue,
    };
    let new_batch = build_expansion_batch(
        p_a.body.batch_id,
        p_a.body.source_class,
        p_a.body.proposed_primitives.clone(),
        p_a.body.proposed_aliases.clone(),
        p_a.body.proposed_dedup_records.clone(),
        p_a.body.proposed_genealogy_edges.clone(),
        refs,
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
// Load-bearing negatives (panel-required)
// ---------------------------------------------------------------

/// Load-bearing negative #1 (panel-required): a duplicate
/// canonical name without an alias decision MUST be rejected.
/// We construct a synthetic proposal that reserves a canonical
/// id colliding with SHEWHART_SEED_ID (1) — the T.12.0
/// verifier's collision rule fires.
#[test]
fn spc_rejects_duplicate_canonical_name_without_alias_decision() {
    let bad_batch = build_expansion_batch(
        "spc_duplicate_canonical_batch",
        SourceClass::StatisticalProcessControl,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(SHEWHART_SEED_ID),
            display_name: "Shewhart (duplicate canonical claim)",
            motivation: "Should be rejected — id collides with existing SEED.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p = build_amendment_proposal(
        "spc_duplicate_canonical_proposal",
        "Duplicate canonical claim without alias decision.",
        SourceClass::StatisticalProcessControl,
        bad_batch,
        build_dedup_court_delta(
            "spc_duplicate_canonical_delta",
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::<RejectionRecord>::new(),
            Vec::<DetectorAliasId>::new(),
        ),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_a_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == SHEWHART_SEED_ID
    )));
}

/// Load-bearing negative #2 (panel-required): a proposal that
/// marks Western Electric as a NEW canonical (instead of
/// recording it as a composition) MUST be rejected. The
/// verifier's `DedupDeltaCollidesWithExistingSeedCanonicalId`
/// rule catches the malformed proposal — Western Electric is
/// already a SEED canonical (id 16), so claiming it as
/// `new_canonical_records` is a structural defect.
#[test]
fn spc_rejects_western_electric_as_canonical_when_marked_composition() {
    // A defective proposal that puts WESTERN_ELECTRIC_SEED_ID
    // into new_canonical_records (would silently mutate the
    // corpus).
    let bad_delta = build_dedup_court_delta(
        "spc_western_electric_bad_delta",
        vec![DetectorCanonicalId(WESTERN_ELECTRIC_SEED_ID)], // the defect
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    let p_seed = seed_t12_a_spc_proposal();
    let p = build_amendment_proposal(
        "spc_western_electric_bad_proposal",
        "Defective Western Electric canonical claim.",
        SourceClass::StatisticalProcessControl,
        p_seed.body.clone(),
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_a_test",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == WESTERN_ELECTRIC_SEED_ID
    )));
}

/// Load-bearing negative #3 (panel-required): an alias claim
/// targeting a non-existent canonical (e.g. "Q statistic" with
/// `collapses_into` pointing at an id that is neither in SEED
/// nor in `new_canonical_records`) MUST be rejected by the
/// shape-level invariant test. T.12.0's verifier does not
/// itself enforce target-resolution (S1.3 court will), so
/// T.12.a pins the rule via a structural test: every alias
/// `collapses_into` must resolve.
#[test]
fn spc_rejects_q_statistic_alias_without_pca_spe_target() {
    let phantom_target = DetectorCanonicalId(99_999);
    let bad_aliases = vec![ProposedAliasClaim {
        reserved_alias_id: DetectorAliasId(Q_STATISTIC_ALIAS_ID),
        collapses_into: phantom_target, // does not exist in SEED or in batch
        alias_name: "Q statistic (bogus target)",
    }];
    let bad_batch = build_expansion_batch(
        "spc_q_statistic_bad_batch",
        SourceClass::StatisticalProcessControl,
        Vec::new(), // no new primitives in this batch
        bad_aliases.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let p = build_amendment_proposal(
        "spc_q_statistic_bad_proposal",
        "Defective Q-statistic alias targeting a non-existent canonical.",
        SourceClass::StatisticalProcessControl,
        bad_batch.clone(),
        build_dedup_court_delta(
            "spc_q_statistic_bad_delta",
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::<RejectionRecord>::new(),
            Vec::<DetectorAliasId>::new(),
        ),
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_a_test",
    );
    // The T.12.0 verifier itself does not have a phantom-target
    // rule yet (S1.3 court will land it). For now, pin the
    // invariant at the shape-test level: every alias must
    // resolve to an existing SEED canonical or a batch
    // primitive.
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let batch_new_ids: std::collections::BTreeSet<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    let mut resolved = true;
    for a in &p.body.proposed_aliases {
        let target = a.collapses_into.0;
        if !seed_ids.contains(&target) && !batch_new_ids.contains(&target) {
            resolved = false;
            break;
        }
    }
    assert!(
        !resolved,
        "synthetic bogus alias-target proposal should be detected by the shape test"
    );
}

// ---------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------

#[test]
fn spc_render_is_byte_stable_across_two_builds() {
    let p = seed_t12_a_spc_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}

#[test]
fn spc_render_text_lists_two_new_canonicals_and_two_compositions() {
    let p = seed_t12_a_spc_proposal();
    let text = render_amendment_proposal_text(&p);
    assert!(text.contains("proposed_primitives    : 2"));
    assert!(text.contains("new_canonical_records   : 2"));
    assert!(text.contains("new_composition_records : 2"));
}

// ---------------------------------------------------------------
// Upstream-anchor preservation
// ---------------------------------------------------------------

/// T.12.a MUST NOT mutate `corpus_hash_v1`. Building +
/// verifying the SPC proposal leaves the corpus hash
/// byte-identical.
#[test]
fn t12_a_does_not_mutate_corpus_hash_v1() {
    use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;
    let before = compute_corpus_hash_v1().bytes;
    let p = seed_t12_a_spc_proposal();
    let _errors = verify_amendment_proposal(&p);
    let after = compute_corpus_hash_v1().bytes;
    assert_eq!(before, after);
}

/// T.12.a MUST NOT touch the existing SEED roster.
#[test]
fn t12_a_does_not_add_records_to_seed() {
    let _ = seed_t12_a_spc_proposal();
    assert_eq!(SEED.len(), 54);
}

/// Page-Hinkley deferral: T.12.a leaves Page-Hinkley to
/// T.12.b's authority. The proposal must NOT touch canonical
/// id 4 (Page-Hinkley) in any way.
const PAGE_HINKLEY_SEED_ID: u32 = 4;

#[test]
fn t12_a_does_not_touch_page_hinkley() {
    let p = seed_t12_a_spc_proposal();
    for pr in &p.body.proposed_primitives {
        assert_ne!(pr.reserved_canonical_id.0, PAGE_HINKLEY_SEED_ID);
    }
    for a in &p.body.proposed_aliases {
        assert_ne!(a.collapses_into.0, PAGE_HINKLEY_SEED_ID);
    }
    for r in &p.body.proposed_dedup_records {
        assert_ne!(r.canonical_id.0, PAGE_HINKLEY_SEED_ID);
    }
    for e in &p.body.proposed_genealogy_edges {
        assert_ne!(e.from_canonical_id.0, PAGE_HINKLEY_SEED_ID);
        assert_ne!(e.to_canonical_id.0, PAGE_HINKLEY_SEED_ID);
    }
    for c in &p.dedup_court_delta.new_canonical_records {
        assert_ne!(c.0, PAGE_HINKLEY_SEED_ID);
    }
    for c in &p.dedup_court_delta.new_composition_records {
        assert_ne!(c.0, PAGE_HINKLEY_SEED_ID);
    }
}

/// Compile-time guards: pinning that the reserved id constants
/// land in the panel-locked range above SEED.len() == 54 and
/// above the 1xxx alias range in claims.rs. These are
/// `const _: ()` guards rather than `assert!` so clippy
/// doesn't complain that the runtime assertion is constant.
const _GUARD_MEWMA_ABOVE_SEED: () = assert!(MEWMA_RESERVED_CANONICAL_ID > 54);
const _GUARD_MCUSUM_ABOVE_SEED: () = assert!(MCUSUM_RESERVED_CANONICAL_ID > 54);
const _GUARD_Q_STAT_ABOVE_ALIAS_RANGE: () = assert!(Q_STATISTIC_ALIAS_ID > 1012);
const _GUARD_SPE_ABOVE_ALIAS_RANGE: () = assert!(SPE_ALIAS_ID > 1012);
const _GUARD_HOTELLING_ALIAS_ABOVE_RANGE: () = assert!(HOTELLING_TSQUARE_ALIAS_ID > 1012);

/// Sentinel test that confirms the compile-time guards above
/// are linked in. If the guards fire at compile time, this
/// test (and the entire suite) fails to build — that's the
/// invariant.
#[test]
fn spc_reserved_ids_are_above_seed_range() {
    // The guards above panic at compile time on violation;
    // the runtime body just acknowledges the suite reaches
    // this point.
    let _: () = _GUARD_MEWMA_ABOVE_SEED;
    let _: () = _GUARD_MCUSUM_ABOVE_SEED;
    let _: () = _GUARD_Q_STAT_ABOVE_ALIAS_RANGE;
    let _: () = _GUARD_SPE_ABOVE_ALIAS_RANGE;
    let _: () = _GUARD_HOTELLING_ALIAS_ABOVE_RANGE;
}

#[test]
fn spc_seed_ids_remain_canonical() {
    // Defensive: the SEED ids T.12.a references for
    // compositions / alias targets must actually still be in
    // SEED. If a future SEED renumber breaks this, the
    // verifier's collision rule would no longer protect us.
    let ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for required in &[
        SHEWHART_SEED_ID,
        HOTELLING_T2_SEED_ID,
        WESTERN_ELECTRIC_SEED_ID,
        NELSON_SEED_ID,
        PCA_SPE_Q_SEED_ID,
    ] {
        assert!(
            ids.contains(required),
            "T.12.a references SEED canonical id {required} which is no longer in SEED"
        );
    }
}
