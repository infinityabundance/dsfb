//! T.12.d acceptance suite — Robust Statistics expansion
//! proposal invariants.
//!
//! Nine panel-required load-bearing negatives pin the
//! alias-heavy dedup discipline T.12.d exists to prove, plus
//! per-SEED-record collision tests for every existing
//! robust-statistics canonical, plus tests that each
//! CanonicalAddition's reason text declares its specific
//! estimator law (pair-selection / quartile law / trim
//! fraction / winsor limit / windowed-local-median /
//! deterministic seed for RANSAC).

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
use dsfb_gpu_atlas_corpus::t12_c_drift::seed_t12_c_drift_proposal;
use dsfb_gpu_atlas_corpus::t12_d_robust::{
    seed_t12_d_robust_proposal, BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, HAMPEL_SEED_ID, K_IQR_FENCE_RESERVED_PRIMITIVE_ID,
    MODIFIED_Z_RESERVED_PRIMITIVE_ID, RANSAC_RESERVED_PRIMITIVE_ID, ROBUST_Z_SEED_ID,
    ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID, THEIL_SEN_RESERVED_CANONICAL_ID,
    TRIMMED_MEAN_RESERVED_CANONICAL_ID, TUKEY_FENCES_SEED_ID,
    WINSORIZED_MEAN_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

/// Every existing SEED canonical id T.12.d recognises via an
/// `ExistingCanonicalAuthorityResolution` record. Used by the
/// parametric collision-loop test.
const T12D_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (ROBUST_Z_SEED_ID, "Robust z-score (median / MAD)"),
    (HAMPEL_SEED_ID, "Hampel filter"),
    (TUKEY_FENCES_SEED_ID, "Tukey fences"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn robust_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_d_robust_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Robust proposal failed verifier: {errors:?}"
    );
}

#[test]
fn robust_proposal_has_open_status() {
    let p = seed_t12_d_robust_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn robust_proposal_targets_robust_statistics() {
    let p = seed_t12_d_robust_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::RobustStatistics
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_d_does_not_mutate_seed_len`.
#[test]
fn t12_d_does_not_mutate_seed_len() {
    let _ = seed_t12_d_robust_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn robust_proposal_proposes_eight_primitives() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 8);
    let ids: Vec<u32> = p
        .body
        .proposed_primitives
        .iter()
        .map(|pr| pr.reserved_canonical_id.0)
        .collect();
    assert!(ids.contains(&THEIL_SEN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&TRIMMED_MEAN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&WINSORIZED_MEAN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&MODIFIED_Z_RESERVED_PRIMITIVE_ID));
    assert!(ids.contains(&ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID));
    assert!(ids.contains(&K_IQR_FENCE_RESERVED_PRIMITIVE_ID));
    assert!(ids.contains(&RANSAC_RESERVED_PRIMITIVE_ID));
}

#[test]
fn robust_proposal_proposes_zero_aliases() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn robust_proposal_proposes_twelve_dedup_records() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 12);
}

#[test]
fn robust_proposal_proposes_seven_genealogy_edges() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 7);
}

#[test]
fn robust_proposal_proposes_eight_source_refs() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 8);
}

#[test]
fn robust_delta_has_four_new_canonical_records() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 4);
    let ids: Vec<u32> = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .map(|c| c.0)
        .collect();
    assert!(ids.contains(&THEIL_SEN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&TRIMMED_MEAN_RESERVED_CANONICAL_ID));
    assert!(ids.contains(&WINSORIZED_MEAN_RESERVED_CANONICAL_ID));
}

/// T.12.d is the **first** proposal to exercise ALL FIVE
/// panel-locked court-delta categories. This test pins the
/// milestone.
#[test]
fn robust_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_d_robust_proposal();
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
    assert!(categories.contains(CATEGORY_REJECTED_NOT_DETERMINISTIC));
    assert_eq!(categories.len(), 5);
}

/// Counts per court-delta category (panel-locked headline:
/// 4 / 3 / 1 / 3 / 1 = 12).
#[test]
fn robust_proposal_court_delta_category_counts() {
    let p = seed_t12_d_robust_proposal();
    let mut canonical = 0;
    let mut existing = 0;
    let mut transfer = 0;
    let mut paramof = 0;
    let mut rejected = 0;
    for r in &p.body.proposed_dedup_records {
        match r.decision_wire_name {
            CATEGORY_CANONICAL_ADDITION => canonical += 1,
            CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION => existing += 1,
            CATEGORY_DOMAIN_TRANSFER_OF => transfer += 1,
            CATEGORY_PARAMETERIZATION_OF => paramof += 1,
            CATEGORY_REJECTED_NOT_DETERMINISTIC => rejected += 1,
            other => panic!("unexpected category wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 4, "expected 4 CanonicalAddition");
    assert_eq!(
        existing, 3,
        "expected 3 ExistingCanonicalAuthorityResolution"
    );
    assert_eq!(transfer, 1, "expected 1 DomainTransferOf");
    assert_eq!(paramof, 3, "expected 3 ParameterizationOf");
    assert_eq!(rejected, 1, "expected 1 RejectedNotDeterministic");
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn robust_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_d_robust_proposal();
    let b = seed_t12_d_robust_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_robust_proposal_hash_matches_stored() {
    let p = seed_t12_d_robust_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn robust_proposal_hash_is_distinct_from_t12_0_a_b_c() {
    let robust = seed_t12_d_robust_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    assert_ne!(
        robust.corpus_amendment_proposal_hash_v1,
        pol.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        robust.corpus_amendment_proposal_hash_v1,
        spc.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        robust.corpus_amendment_proposal_hash_v1,
        scd.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        robust.corpus_amendment_proposal_hash_v1,
        drift.corpus_amendment_proposal_hash_v1
    );
}

/// Load-bearing negative #9 (panel-required):
/// `t12_d_hash_changes_when_robust_statistic_law_changes`.
/// Mutating one CanonicalAddition record's reason text (the
/// estimator-law declaration) changes the proposal hash.
#[test]
fn t12_d_hash_changes_when_robust_statistic_law_changes() {
    let p_a = seed_t12_d_robust_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == THEIL_SEN_RESERVED_CANONICAL_ID
        })
        .expect("Theil-Sen CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED estimator-law declaration for hash-sensitivity test",
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

fn build_defective_collision_proposal(
    seed_id: u32,
    test_label: &'static str,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_d_collision_batch",
        SourceClass::RobustStatistics,
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
        "t12_d_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_d_collision_proposal",
        "Defective T.12.d-style proposal duplicating an existing SEED canonical.",
        SourceClass::RobustStatistics,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_d_test",
    )
}

/// Load-bearing negative #2 (panel-required, most important
/// for an alias-heavy class):
/// `t12_d_rejects_robust_z_duplicate_without_existing_authority_resolution`.
#[test]
fn t12_d_rejects_robust_z_duplicate_without_existing_authority_resolution() {
    let p = build_defective_collision_proposal(ROBUST_Z_SEED_ID, "Robust z-score (alias attempt)");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == ROBUST_Z_SEED_ID
    )));
}

/// Parametric loop for all three SEED robust-statistics records.
#[test]
fn t12_d_seed_collision_loop_fires_for_every_authority_resolved_id() {
    for (seed_id, label) in T12D_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id, label);
        let errors = verify_amendment_proposal(&p);
        assert!(
            errors.iter().any(|e| matches!(
                e.kind,
                AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id
                } if canonical_id.0 == *seed_id
            )),
            "expected collision rule fire for SEED id {seed_id} ({label})"
        );
    }
}

/// Per-named SEED collision tests.
#[test]
fn t12_d_rejects_duplicate_hampel_when_seed_id_exists() {
    let p = build_defective_collision_proposal(HAMPEL_SEED_ID, "Hampel filter");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == HAMPEL_SEED_ID
    )));
}

#[test]
fn t12_d_rejects_duplicate_tukey_when_seed_id_exists() {
    let p = build_defective_collision_proposal(TUKEY_FENCES_SEED_ID, "Tukey fences");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == TUKEY_FENCES_SEED_ID
    )));
}

// ---------------------------------------------------------------
// Estimator-law contract assertions (panel-required)
// ---------------------------------------------------------------

/// Load-bearing negative #3 (panel-required):
/// `t12_d_rejects_hampel_as_canonical_without_windowed_local_median_law`.
/// Hampel's ExistingCanonicalAuthorityResolution reason text
/// MUST declare the windowed local-median + MAD + threshold +
/// replacement / rejection law.
#[test]
fn t12_d_rejects_hampel_as_canonical_without_windowed_local_median_law() {
    let p = seed_t12_d_robust_proposal();
    let hampel = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == HAMPEL_SEED_ID
        })
        .expect("Hampel must have ExistingCanonicalAuthorityResolution record");
    let r = hampel.reason.to_lowercase();
    assert!(
        r.contains("local median") || r.contains("windowed"),
        "Hampel record must declare windowed local-median law: {}",
        hampel.reason
    );
    assert!(
        r.contains("mad"),
        "Hampel record must declare MAD component: {}",
        hampel.reason
    );
    assert!(
        r.contains("replacement") || r.contains("rejection"),
        "Hampel record must declare replacement / rejection rule: {}",
        hampel.reason
    );
}

/// Load-bearing negative #4 (panel-required):
/// `t12_d_rejects_tukey_fence_without_quartile_law`.
#[test]
fn t12_d_rejects_tukey_fence_without_quartile_law() {
    let p = seed_t12_d_robust_proposal();
    let tukey = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == TUKEY_FENCES_SEED_ID
        })
        .expect("Tukey fences must have ExistingCanonicalAuthorityResolution record");
    let r = tukey.reason.to_lowercase();
    assert!(
        r.contains("quartile"),
        "Tukey fences record must declare quartile estimator: {}",
        tukey.reason
    );
    assert!(
        r.contains("iqr") || r.contains("multiplier"),
        "Tukey fences record must declare IQR multiplier: {}",
        tukey.reason
    );
    assert!(
        r.contains("inclusive") || r.contains("exclusive") || r.contains("fence"),
        "Tukey fences record must declare fence semantics: {}",
        tukey.reason
    );
}

/// Load-bearing negative #5 (panel-required):
/// `t12_d_rejects_theil_sen_without_pair_selection_law`.
#[test]
fn t12_d_rejects_theil_sen_without_pair_selection_law() {
    let p = seed_t12_d_robust_proposal();
    let theil_sen = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == THEIL_SEN_RESERVED_CANONICAL_ID
        })
        .expect("Theil-Sen must have CanonicalAddition record");
    let r = theil_sen.reason.to_lowercase();
    assert!(
        r.contains("pair-selection") || r.contains("pair selection") || r.contains("pairwise"),
        "Theil-Sen record must declare pair-selection law: {}",
        theil_sen.reason
    );
    assert!(
        r.contains("slope-median") || r.contains("slope median") || r.contains("median"),
        "Theil-Sen record must declare slope-median law: {}",
        theil_sen.reason
    );
    assert!(
        r.contains("tie-break") || r.contains("tie break"),
        "Theil-Sen record must declare tie-break law: {}",
        theil_sen.reason
    );
}

/// Load-bearing negative #6 (panel-required):
/// `t12_d_rejects_ransac_without_deterministic_seed_and_schedule`.
/// RANSAC must NOT appear in `new_canonical_records`; AND its
/// `RejectedNotDeterministic` record must declare the four
/// reduction requirements (seed, schedule, iteration budget,
/// tie-break).
#[test]
fn t12_d_rejects_ransac_without_deterministic_seed_and_schedule() {
    let p = seed_t12_d_robust_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RANSAC_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "RANSAC (id {RANSAC_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let ransac = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == RANSAC_RESERVED_PRIMITIVE_ID
        })
        .expect("RANSAC must have RejectedNotDeterministic record");
    let r = ransac.reason.to_lowercase();
    assert!(
        r.contains("seed"),
        "RANSAC rejection must declare seed requirement: {}",
        ransac.reason
    );
    assert!(
        r.contains("schedule"),
        "RANSAC rejection must declare sample-schedule requirement: {}",
        ransac.reason
    );
    assert!(
        r.contains("iteration budget"),
        "RANSAC rejection must declare iteration-budget requirement: {}",
        ransac.reason
    );
    assert!(
        r.contains("tie-break") || r.contains("tie break"),
        "RANSAC rejection must declare tie-break requirement: {}",
        ransac.reason
    );
}

/// Load-bearing negative #7 (panel-required):
/// `t12_d_rejects_trimmed_mean_without_trim_fraction_law`.
#[test]
fn t12_d_rejects_trimmed_mean_without_trim_fraction_law() {
    let p = seed_t12_d_robust_proposal();
    let trimmed = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == TRIMMED_MEAN_RESERVED_CANONICAL_ID
        })
        .expect("Trimmed mean must have CanonicalAddition record");
    let r = trimmed.reason.to_lowercase();
    assert!(
        r.contains("trim fraction") || r.contains("trim-fraction"),
        "Trimmed mean record must declare trim-fraction law: {}",
        trimmed.reason
    );
    assert!(
        r.contains("symmetric") || r.contains("one-sided"),
        "Trimmed mean record must declare symmetric / one-sided semantics: {}",
        trimmed.reason
    );
}

/// Load-bearing negative #8 (panel-required):
/// `t12_d_rejects_winsorized_mean_without_winsor_limit_law`.
#[test]
fn t12_d_rejects_winsorized_mean_without_winsor_limit_law() {
    let p = seed_t12_d_robust_proposal();
    let winsorized = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == WINSORIZED_MEAN_RESERVED_CANONICAL_ID
        })
        .expect("Winsorized mean must have CanonicalAddition record");
    let r = winsorized.reason.to_lowercase();
    assert!(
        r.contains("winsor limit") || r.contains("winsor-limit"),
        "Winsorized mean record must declare winsor-limit law: {}",
        winsorized.reason
    );
    assert!(
        r.contains("replace") || r.contains("replacement"),
        "Winsorized mean record must declare replacement semantics: {}",
        winsorized.reason
    );
}

/// Sibling: biweight midvariance's CanonicalAddition record
/// must declare tuning-constant + convergence law.
#[test]
fn t12_d_rejects_biweight_midvariance_without_tuning_and_convergence_law() {
    let p = seed_t12_d_robust_proposal();
    let biweight = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BIWEIGHT_MIDVARIANCE_RESERVED_CANONICAL_ID
        })
        .expect("Biweight midvariance must have CanonicalAddition record");
    let r = biweight.reason.to_lowercase();
    assert!(
        r.contains("tuning constant"),
        "Biweight record must declare tuning constant: {}",
        biweight.reason
    );
    assert!(
        r.contains("convergence") || r.contains("iteration"),
        "Biweight record must declare convergence / iteration law: {}",
        biweight.reason
    );
}

// ---------------------------------------------------------------
// ParameterizationOf family-relationship invariants
// ---------------------------------------------------------------

/// Modified z-score is ParameterizationOf(robust-z).
#[test]
fn t12_d_modified_z_is_parameterizationof_robust_z() {
    let p = seed_t12_d_robust_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == MODIFIED_Z_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Modified z-score (id {MODIFIED_Z_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let modified_z = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == MODIFIED_Z_RESERVED_PRIMITIVE_ID
        })
        .expect("Modified z must have ParameterizationOf record");
    assert!(
        modified_z.reason.contains("robust-z") || modified_z.reason.contains("robust z"),
        "Modified z record must reference robust-z parent: {}",
        modified_z.reason
    );
}

/// Rolling Hampel is ParameterizationOf(Hampel).
#[test]
fn t12_d_rolling_hampel_is_parameterizationof_hampel() {
    let p = seed_t12_d_robust_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Rolling Hampel must NOT be in new_canonical_records"
    );
    let rolling_hampel = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == ROLLING_HAMPEL_RESERVED_PRIMITIVE_ID
        })
        .expect("Rolling Hampel must have ParameterizationOf record");
    assert!(
        rolling_hampel.reason.contains("Hampel"),
        "Rolling Hampel record must reference Hampel parent: {}",
        rolling_hampel.reason
    );
}

/// k×IQR fence is ParameterizationOf(Tukey fences).
#[test]
fn t12_d_k_iqr_fence_is_parameterizationof_tukey() {
    let p = seed_t12_d_robust_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == K_IQR_FENCE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "k x IQR fence must NOT be in new_canonical_records"
    );
    let k_iqr = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == K_IQR_FENCE_RESERVED_PRIMITIVE_ID
        })
        .expect("k x IQR fence must have ParameterizationOf record");
    assert!(
        k_iqr.reason.contains("Tukey fences") || k_iqr.reason.contains("Tukey"),
        "k x IQR fence record must reference Tukey fences parent: {}",
        k_iqr.reason
    );
}

/// RANSAC appears in proposed_primitives but NOT in
/// new_canonical_records (mirror of the BOCPD pattern from
/// T.12.b).
#[test]
fn t12_d_ransac_present_in_proposed_primitives_but_not_in_new_canonical_records() {
    let p = seed_t12_d_robust_proposal();
    let in_primitives = p
        .body
        .proposed_primitives
        .iter()
        .any(|pr| pr.reserved_canonical_id.0 == RANSAC_RESERVED_PRIMITIVE_ID);
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RANSAC_RESERVED_PRIMITIVE_ID);
    assert!(in_primitives);
    assert!(!in_canonical);
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_d_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_d_robust_proposal();
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

#[test]
fn t12_d_rejects_domain_transfer_without_existing_target() {
    let p = seed_t12_d_robust_proposal();
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

#[test]
fn t12_d_every_new_canonical_has_source_ref() {
    let p = seed_t12_d_robust_proposal();
    assert!(
        p.body.proposed_source_refs.len() >= p.dedup_court_delta.new_canonical_records.len(),
        "{} new canonicals vs {} source refs",
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

#[test]
fn t12_d_every_dedup_record_has_reason_code() {
    let p = seed_t12_d_robust_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(
            !r.reason.is_empty(),
            "dedup record at canonical_id {} ({}) has empty reason",
            r.canonical_id.0,
            r.decision_wire_name
        );
    }
}

#[test]
fn t12_d_canonical_addition_ids_are_in_5401_to_5404_range() {
    let p = seed_t12_d_robust_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5401..=5404).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.d reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn robust_proposal_text_rendering_byte_stable() {
    let p = seed_t12_d_robust_proposal();
    let a = render_amendment_proposal_text(&p);
    let b = render_amendment_proposal_text(&p);
    assert_eq!(a, b);
}

#[test]
fn robust_proposal_json_rendering_byte_stable() {
    let p = seed_t12_d_robust_proposal();
    let a = render_amendment_proposal_json(&p);
    let b = render_amendment_proposal_json(&p);
    assert_eq!(a, b);
}
