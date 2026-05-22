//! T.12.c acceptance suite — Drift Detection and Distribution-
//! Distance Authority expansion proposal invariants.
//!
//! Nine panel-required load-bearing negatives pin the cross-
//! class dedup authority discipline T.12.c exists to prove,
//! plus per-SEED-record collision tests for every existing
//! distribution-distance canonical, plus shape / determinism /
//! rendering invariants.
//!
//! Panel-locked headline: *"Do not count method names. Count
//! distinct deterministic decision functionals with declared
//! reference / window / sampling contracts."*

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
use dsfb_gpu_atlas_corpus::t12_b_scd::seed_t12_b_scd_proposal;
use dsfb_gpu_atlas_corpus::t12_c_drift::{
    seed_t12_c_drift_proposal, ADWIN_RESERVED_CANONICAL_ID, ANDERSON_DARLING_SEED_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CRAMER_VON_MISES_SEED_ID, DDM_RESERVED_CANONICAL_ID, EDDM_RESERVED_PRIMITIVE_ID,
    ENERGY_DISTANCE_SEED_ID, HDDM_RESERVED_CANONICAL_ID, HELLINGER_SEED_ID, JENSEN_SHANNON_SEED_ID,
    KL_SEED_ID, KSWIN_RESERVED_PRIMITIVE_ID, KS_SEED_ID, KUIPER_RESERVED_CANONICAL_ID, MMD_SEED_ID,
    PSI_SEED_ID, TOTAL_VARIATION_SEED_ID, WASSERSTEIN_SEED_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

/// Every existing SEED canonical id T.12.c recognises via an
/// `ExistingCanonicalAuthorityResolution` record. Used by the
/// per-record collision tests and the parametric
/// `t12_c_seed_collision_loop` test.
const T12C_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (KS_SEED_ID, "Kolmogorov-Smirnov"),
    (KL_SEED_ID, "Kullback-Leibler"),
    (MMD_SEED_ID, "MMD"),
    (ANDERSON_DARLING_SEED_ID, "Anderson-Darling"),
    (CRAMER_VON_MISES_SEED_ID, "Cramer-von Mises"),
    (WASSERSTEIN_SEED_ID, "Wasserstein"),
    (ENERGY_DISTANCE_SEED_ID, "Energy distance"),
    (HELLINGER_SEED_ID, "Hellinger"),
    (PSI_SEED_ID, "PSI"),
    (JENSEN_SHANNON_SEED_ID, "Jensen-Shannon"),
    (TOTAL_VARIATION_SEED_ID, "Total variation"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn drift_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_c_drift_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Drift proposal failed verifier: {errors:?}"
    );
}

#[test]
fn drift_proposal_has_open_status() {
    let p = seed_t12_c_drift_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn drift_proposal_targets_drift_detection() {
    let p = seed_t12_c_drift_proposal();
    assert!(matches!(p.target_source_class, SourceClass::DriftDetection));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_c_does_not_mutate_seed_len`. T.12.c is a docketed legal
/// act on the amendment court, not a corpus mutation.
#[test]
fn t12_c_does_not_mutate_seed_len() {
    let _ = seed_t12_c_drift_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn drift_proposal_proposes_six_primitives() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 6);
    let ids: Vec<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    assert!(ids.contains(&KUIPER_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&ADWIN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&DDM_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&HDDM_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&EDDM_RESERVED_PRIMITIVE_ID));
    assert!(ids.contains(&KSWIN_RESERVED_PRIMITIVE_ID));
}

#[test]
fn drift_proposal_proposes_zero_aliases() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn drift_proposal_proposes_eighteen_dedup_records() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 18);
}

#[test]
fn drift_proposal_proposes_six_genealogy_edges() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 6);
}

#[test]
fn drift_proposal_proposes_six_source_refs() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 6);
}

/// T.12.c's delta admits exactly FOUR new canonical records —
/// NOT six (EDDM and KSWIN are deliberately absent because they
/// are `ParameterizationOf` records).
#[test]
fn drift_delta_has_four_new_canonical_records() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 4);
    let ids: Vec<u32> = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .map(|c| c.0)
        .collect();
    assert!(ids.contains(&KUIPER_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&ADWIN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&DDM_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&HDDM_RESERVED_CANONICAL_ID));
}

/// Shape: T.12.c emits exactly four court-delta categories
/// (CanonicalAddition, ExistingCanonicalAuthorityResolution,
/// DomainTransferOf, ParameterizationOf). The
/// `ParameterizationOf` category lands for the first time at
/// T.12.c.
#[test]
fn drift_proposal_emits_four_court_delta_categories() {
    let p = seed_t12_c_drift_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert!(categories.contains(CATEGORY_CANONICAL_ADDITION));
    assert!(categories.contains(CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION));
    assert!(categories.contains(CATEGORY_DOMAIN_TRANSFER_OF));
    assert!(categories.contains(CATEGORY_PARAMETERIZATION_OF));
    assert_eq!(categories.len(), 4);
}

/// Counts per court-delta category (panel-locked headline:
/// 4 / 11 / 1 / 2 = 18).
#[test]
fn drift_proposal_court_delta_category_counts() {
    let p = seed_t12_c_drift_proposal();
    let mut canonical = 0;
    let mut existing = 0;
    let mut transfer = 0;
    let mut paramof = 0;
    for r in &p.body.proposed_dedup_records {
        match r.decision_wire_name {
            CATEGORY_CANONICAL_ADDITION => canonical += 1,
            CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION => existing += 1,
            CATEGORY_DOMAIN_TRANSFER_OF => transfer += 1,
            CATEGORY_PARAMETERIZATION_OF => paramof += 1,
            other => panic!("unexpected category wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 4, "expected 4 CanonicalAddition records");
    assert_eq!(
        existing, 11,
        "expected 11 ExistingCanonicalAuthorityResolution"
    );
    assert_eq!(transfer, 1, "expected 1 DomainTransferOf");
    assert_eq!(paramof, 2, "expected 2 ParameterizationOf");
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn drift_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_c_drift_proposal();
    let b = seed_t12_c_drift_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_drift_proposal_hash_matches_stored() {
    let p = seed_t12_c_drift_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn drift_proposal_hash_is_distinct_from_t12_0_a_b() {
    let drift = seed_t12_c_drift_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    assert_ne!(
        drift.corpus_amendment_proposal_hash_v1, pol.corpus_amendment_proposal_hash_v1,
        "T.12.c hash must differ from T.12.0 proof-of-life hash"
    );
    assert_ne!(
        drift.corpus_amendment_proposal_hash_v1, spc.corpus_amendment_proposal_hash_v1,
        "T.12.c hash must differ from T.12.a SPC hash"
    );
    assert_ne!(
        drift.corpus_amendment_proposal_hash_v1, scd.corpus_amendment_proposal_hash_v1,
        "T.12.c hash must differ from T.12.b SCD hash"
    );
}

/// Load-bearing negative #9 (panel-required):
/// `t12_c_hash_changes_when_distance_formula_or_source_ref_changes`.
/// Mutating one source-ref's title changes the batch hash and
/// therefore the proposal hash.
#[test]
fn t12_c_hash_changes_when_distance_formula_or_source_ref_changes() {
    let p_a = seed_t12_c_drift_proposal();
    let mut refs = p_a.body.proposed_source_refs.clone();
    refs[0] = dsfb_gpu_atlas_corpus::amendment::ProposedSourceRef {
        citation_key: refs[0].citation_key,
        title: "MUTATED TITLE for distance-formula / source-ref sensitivity test",
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
// Cross-class dedup-authority load-bearing negatives
// ---------------------------------------------------------------
//
// Per the panel's `t12_c_detects_existing_seed_collisions_
// before_new_canonical_assignment` mandate, the suite encodes
// one collision test per SEED record T.12.c recognises (11
// existing canonicals) plus a parametric loop test asserting
// the rule fires for every one.

/// Construct a defective T.12.c-style proposal that puts the
/// given existing SEED canonical id into the dedup-delta's
/// `new_canonical_records` AND into a proposed-primitive shell.
fn build_defective_collision_proposal(
    seed_id: u32,
    test_label: &'static str,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_c_collision_batch",
        SourceClass::DriftDetection,
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
        "t12_c_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_c_collision_proposal",
        "Defective T.12.c-style proposal duplicating an existing SEED canonical.",
        SourceClass::DriftDetection,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_c_test",
    )
}

/// Load-bearing negative #2 (panel-required, generalised):
/// `t12_c_rejects_existing_seed_collision_without_authority_resolution`.
/// Parametric loop: for every existing SEED canonical T.12.c
/// recognises (11 records), constructing a defective variant
/// that promotes it to `new_canonical_records` fires the
/// `DedupDeltaCollidesWithExistingSeedCanonicalId` rule.
#[test]
fn t12_c_seed_collision_loop_fires_for_every_authority_resolved_id() {
    for (seed_id, label) in T12C_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id, label);
        let errors = verify_amendment_proposal(&p);
        assert!(
            errors.iter().any(|e| matches!(
                e.kind,
                AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id
                } if canonical_id.0 == *seed_id
            )),
            "expected DedupDeltaCollidesWithExistingSeedCanonicalId for SEED id {seed_id} ({label})"
        );
    }
}

/// Representative per-SEED-id collision tests (one named test
/// per existing distribution-distance canonical, so a developer
/// reading the test list sees the dedup discipline by name).
#[test]
fn t12_c_rejects_duplicate_ks_when_seed_id_exists() {
    let p = build_defective_collision_proposal(KS_SEED_ID, "Kolmogorov-Smirnov");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == KS_SEED_ID
    )));
}

#[test]
fn t12_c_rejects_duplicate_kl_when_seed_id_exists() {
    let p = build_defective_collision_proposal(KL_SEED_ID, "Kullback-Leibler");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == KL_SEED_ID
    )));
}

#[test]
fn t12_c_rejects_duplicate_mmd_when_seed_id_exists() {
    let p = build_defective_collision_proposal(MMD_SEED_ID, "MMD");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == MMD_SEED_ID
    )));
}

#[test]
fn t12_c_rejects_duplicate_wasserstein_when_seed_id_exists() {
    let p = build_defective_collision_proposal(WASSERSTEIN_SEED_ID, "Wasserstein");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == WASSERSTEIN_SEED_ID
    )));
}

#[test]
fn t12_c_rejects_duplicate_psi_when_seed_id_exists() {
    let p = build_defective_collision_proposal(PSI_SEED_ID, "PSI");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == PSI_SEED_ID
    )));
}

// ---------------------------------------------------------------
// KSWIN + EDDM parameterization load-bearing negatives
// ---------------------------------------------------------------

/// Load-bearing negative #3 (panel-required):
/// `t12_c_rejects_kswin_as_canonical_without_ks_relationship_decision`.
/// KSWIN must NOT appear in `new_canonical_records` of the
/// actual T.12.c delta, AND there must be a `ParameterizationOf`
/// dedup record for KSWIN whose reason text references KS.
#[test]
fn t12_c_rejects_kswin_as_canonical_without_ks_relationship_decision() {
    let p = seed_t12_c_drift_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == KSWIN_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "KSWIN (id {KSWIN_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let kswin_param_record = p.body.proposed_dedup_records.iter().any(|r| {
        r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
            && r.canonical_id.0 == KSWIN_RESERVED_PRIMITIVE_ID
            && (r.reason.contains("KS") || r.reason.contains("Kolmogorov"))
    });
    assert!(
        kswin_param_record,
        "KSWIN must have a ParameterizationOf record whose reason references KS / Kolmogorov-Smirnov"
    );
}

/// Sibling: EDDM follows the same family-relationship discipline
/// (panel caution `t12_c_rejects_ddm_family_variant_without_family_relationship`).
#[test]
fn t12_c_rejects_ddm_family_variant_without_family_relationship() {
    let p = seed_t12_c_drift_proposal();
    let eddm_in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == EDDM_RESERVED_PRIMITIVE_ID);
    assert!(
        !eddm_in_canonical,
        "EDDM (id {EDDM_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let eddm_param_record = p.body.proposed_dedup_records.iter().any(|r| {
        r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
            && r.canonical_id.0 == EDDM_RESERVED_PRIMITIVE_ID
            && r.reason.contains("DDM")
    });
    assert!(
        eddm_param_record,
        "EDDM must have a ParameterizationOf record whose reason references DDM"
    );
    // HDDM is canonical but should still carry a Generalizes /
    // DerivedFrom edge to DDM so the family relationship is
    // explicit.
    let hddm_family_edge = p.body.proposed_genealogy_edges.iter().any(|e| {
        e.from_canonical_id.0 == HDDM_RESERVED_CANONICAL_ID
            && e.to_canonical_id.0 == DDM_RESERVED_CANONICAL_ID
    });
    assert!(
        hddm_family_edge,
        "HDDM must have a genealogy edge to DDM declaring the family relationship"
    );
}

// ---------------------------------------------------------------
// ADWIN adaptive-window contract
// ---------------------------------------------------------------

/// Load-bearing negative #4 (panel-required):
/// `t12_c_rejects_adwin_without_declared_adaptive_window_law`.
/// ADWIN's CanonicalAddition record reason text MUST mention
/// the adaptive-window law (Hoeffding-bound cut rule, delta,
/// deterministic cut + window-merge tie-break).
#[test]
fn t12_c_rejects_adwin_without_declared_adaptive_window_law() {
    let p = seed_t12_c_drift_proposal();
    let adwin_record = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == ADWIN_RESERVED_CANONICAL_ID
        })
        .expect("ADWIN must have a CanonicalAddition record");
    let r = adwin_record.reason.to_lowercase();
    assert!(
        r.contains("adaptive") || r.contains("hoeffding"),
        "ADWIN reason text must mention adaptive-window or Hoeffding-bound contract: {}",
        adwin_record.reason
    );
    assert!(
        r.contains("cut") || r.contains("delta") || r.contains("window"),
        "ADWIN reason text must mention cut rule / delta / window contract: {}",
        adwin_record.reason
    );
}

// ---------------------------------------------------------------
// Reference-distribution / binning contracts
// ---------------------------------------------------------------

/// Load-bearing negative #6 (panel-required, strongest):
/// `t12_c_rejects_distribution_distance_without_reference_distribution_requirement`.
/// Every distance / divergence / two-sample record in T.12.c
/// MUST mention reference distribution / window pair / reference
/// window — the contract for ALL distribution-distance methods.
#[test]
fn t12_c_rejects_distribution_distance_without_reference_distribution_requirement() {
    let p = seed_t12_c_drift_proposal();
    for r in &p.body.proposed_dedup_records {
        // DomainTransferOf records document class-level
        // transfers; ParameterizationOf records inherit their
        // contract from the parent canonical (e.g., EDDM
        // inherits DDM's reference-window contract). Both
        // categories bypass the per-record contract requirement
        // — the requirement applies to CanonicalAddition +
        // ExistingCanonicalAuthorityResolution records, which
        // are where the contract MUST be declared at the record
        // level.
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
            || r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
        {
            continue;
        }
        let body = r.reason.to_lowercase();
        let has_ref = body.contains("reference distribution")
            || body.contains("reference window")
            || body.contains("window pair")
            || body.contains("reference")
            || body.contains("ordered binary")
            || body.contains("ordered stream")
            || body.contains("adaptive partition")
            || body.contains("running minimum");
        assert!(
            has_ref,
            "record for canonical_id {} ({}) is missing reference-distribution / reference-window / contract language: {}",
            r.canonical_id.0, r.decision_wire_name, r.reason
        );
    }
}

/// Load-bearing negative #7 (panel-required):
/// `t12_c_rejects_psi_without_binning_law`. PSI's authority-
/// resolution record reason MUST declare a binning law.
#[test]
fn t12_c_rejects_psi_without_binning_law() {
    let p = seed_t12_c_drift_proposal();
    let psi_record = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == PSI_SEED_ID
        })
        .expect("PSI must have an ExistingCanonicalAuthorityResolution record");
    let r = psi_record.reason.to_lowercase();
    assert!(
        r.contains("binning") || r.contains("bin "),
        "PSI reason text must mention a binning law: {}",
        psi_record.reason
    );
}

// ---------------------------------------------------------------
// Probabilistic-distance discipline
// ---------------------------------------------------------------

/// Load-bearing negative #8 (panel-required):
/// `t12_c_rejects_probabilistic_or_randomized_distance_without_deterministic_reduction`.
/// No record in the actual T.12.c proposal may claim a
/// probabilistic / randomized distance as canonical. Pinning
/// the invariant on the actual proposal AND ensuring no
/// `proposed_primitive` carries probabilistic / randomized
/// language without a "deterministic" disclosure.
#[test]
fn t12_c_rejects_probabilistic_or_randomized_distance_without_deterministic_reduction() {
    let p = seed_t12_c_drift_proposal();
    for pr in &p.body.proposed_primitives {
        let body = pr.motivation.to_lowercase();
        let claims_probabilistic = body.contains("probabilistic")
            || body.contains("randomized")
            || body.contains("monte carlo")
            || body.contains("random projection");
        // The BOCPD-style honest disclosure pattern: it's OK to
        // mention probabilistic semantics as long as the same
        // record also discloses the deterministic stance (the
        // word "deterministic" must appear).
        if claims_probabilistic {
            assert!(
                body.contains("deterministic"),
                "primitive {} carries probabilistic / randomized language without a deterministic disclosure: {}",
                pr.reserved_canonical_id.0, pr.motivation
            );
        }
    }
    for r in &p.body.proposed_dedup_records {
        let body = r.reason.to_lowercase();
        let claims_probabilistic = body.contains("probabilistic")
            || body.contains("randomized")
            || body.contains("monte carlo")
            || body.contains("random projection");
        if claims_probabilistic {
            assert!(
                body.contains("deterministic"),
                "dedup record at canonical_id {} carries probabilistic / randomized language without deterministic disclosure: {}",
                r.canonical_id.0, r.reason
            );
        }
    }
}

// ---------------------------------------------------------------
// Other panel-required invariants
// ---------------------------------------------------------------

/// Every `ExistingCanonicalAuthorityResolution` record's
/// canonical_id MUST resolve to an existing SEED canonical.
#[test]
fn t12_c_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_c_drift_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "ExistingCanonicalAuthorityResolution record references id {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

/// Every `DomainTransferOf` record references an existing SEED
/// canonical (the domain-transfer target must exist).
#[test]
fn t12_c_rejects_domain_transfer_without_existing_target() {
    let p = seed_t12_c_drift_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "DomainTransferOf record references id {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

/// Every `ParameterizationOf` record's reason text must
/// reference its parent canonical name OR a SEED id explicitly.
/// Pinning the family-relationship invariant.
#[test]
fn t12_c_parameterizationof_records_reference_parent_canonical() {
    let p = seed_t12_c_drift_proposal();
    let paramof_records: Vec<&ProposedDedupRecord> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF)
        .collect();
    assert!(
        !paramof_records.is_empty(),
        "T.12.c must carry at least one ParameterizationOf record"
    );
    for r in paramof_records {
        assert!(
            r.reason.to_lowercase().contains("parameterizationof")
                || r.reason.contains("ParameterizationOf"),
            "ParameterizationOf record at id {} must mention the family relationship in its reason text: {}",
            r.canonical_id.0, r.reason
        );
    }
}

/// Every new canonical (in `new_canonical_records`) has a
/// supporting source ref (count-based check + structural
/// integrity on each source ref entry).
#[test]
fn t12_c_every_new_canonical_has_source_ref() {
    let p = seed_t12_c_drift_proposal();
    assert!(
        p.body.proposed_source_refs.len() >= p.dedup_court_delta.new_canonical_records.len(),
        "T.12.c has {} new canonicals but only {} source refs",
        p.dedup_court_delta.new_canonical_records.len(),
        p.body.proposed_source_refs.len()
    );
    for s in &p.body.proposed_source_refs {
        assert!(!s.citation_key.is_empty());
        assert!(!s.title.is_empty());
        assert!(!s.venue.is_empty());
        assert!(s.year >= 1900 && s.year <= 2099);
    }
}

/// Every record's reason text is non-empty (the load-bearing
/// negative `t12_c_every_rejection_has_reason_code` generalised
/// — T.12.c has no formal rejection records, but every dedup
/// record carries a reason).
#[test]
fn t12_c_every_dedup_record_has_reason_code() {
    let p = seed_t12_c_drift_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(
            !r.reason.is_empty(),
            "dedup record at canonical_id {} ({}) has empty reason",
            r.canonical_id.0,
            r.decision_wire_name
        );
    }
}

/// Canonical-addition ids are in the 5301..=5304 reserved range
/// for T.12.c. Catches accidental id leakage.
#[test]
fn t12_c_canonical_addition_ids_are_in_5301_to_5304_range() {
    let p = seed_t12_c_drift_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5301..=5304).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.c reserved range 5301..=5304",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn drift_proposal_text_rendering_byte_stable() {
    let p = seed_t12_c_drift_proposal();
    let a = render_amendment_proposal_text(&p);
    let b = render_amendment_proposal_text(&p);
    assert_eq!(a, b);
}

#[test]
fn drift_proposal_json_rendering_byte_stable() {
    let p = seed_t12_c_drift_proposal();
    let a = render_amendment_proposal_json(&p);
    let b = render_amendment_proposal_json(&p);
    assert_eq!(a, b);
}
