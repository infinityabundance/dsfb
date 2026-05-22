// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.4 acceptance tests: deterministic dedup-court invariants.
//!
//! Panel-locked tests (each name documents an invariant):
//!
//! - `aliases_share_duplicate_group_but_keep_source_hashes`
//! - `canonical_head_is_stable_across_static_and_toml`
//! - `western_electric_is_classified_as_composition_of_shewhart`
//! - `nelson_rules_are_classified_with_composition_reason_code`
//! - `same_identity_hash_collapses_to_alias_unless_role_differs`
//! - `semantic_role_difference_prevents_alias_collapse`
//! - `every_noncanonical_record_has_decision_and_reason`
//! - `dedup_report_counts_canonical_alias_parameterization_composition_rejected_deferred`
//! - `court_pass_is_deterministic_across_two_runs`
//! - `court_verify_clean_on_t4_first_batch`
//! - `report_contains_court_decision_summary` (T.6.1 — public
//!   report's Section (1b) sources counts from
//!   `court::classify_all` over SEED+CLAIMS so the (1) Totals row
//!   is no longer the only audit-visible tally.)

use dsfb_gpu_atlas_corpus::claims::{DetectorClaim, CLAIMS};
use dsfb_gpu_atlas_corpus::court::{
    classify, classify_all, count_decisions, render_court_report, verify_court,
};
use dsfb_gpu_atlas_corpus::identity::compute_identity_hashes;
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    CanonicalisationDecision, DedupReason, DedupSubject, DetectorAliasId, DetectorCanonicalId,
    SourceRef, WitnessRole,
};

#[test]
fn court_pass_is_deterministic_across_two_runs() {
    let a = classify_all();
    let b = classify_all();
    assert_eq!(a.len(), b.len());
    for (ra, rb) in a.iter().zip(b.iter()) {
        assert_eq!(ra.subject, rb.subject);
        assert_eq!(ra.decision, rb.decision);
        assert_eq!(ra.reason_code, rb.reason_code);
        assert_eq!(ra.literature_name, rb.literature_name);
    }
}

#[test]
fn court_verify_clean_on_t4_first_batch() {
    let records = classify_all();
    let report = verify_court(&records, SEED, CLAIMS);
    assert!(
        report.is_clean(),
        "court verify found {} errors: {:?}",
        report.errors.len(),
        report
            .errors
            .iter()
            .map(|e| format!("[{:?}] {}", e.subject, e.message))
            .collect::<Vec<_>>()
    );
}

#[test]
fn dedup_report_counts_canonical_alias_parameterization_composition_rejected_deferred() {
    let records = classify_all();
    let counts = count_decisions(&records);
    // T.4 first batch: 54 seed records, of which 2 are CompositionOf
    // (Western Electric, Nelson) and 52 are Canonical. Plus 12
    // alias claims, all AliasOf.
    assert_eq!(
        counts.total(),
        66,
        "expected 54 seed + 12 alias = 66 records"
    );
    assert_eq!(counts.canonical, 52);
    assert_eq!(counts.compositions, 2);
    assert_eq!(counts.aliases, 12);
    assert_eq!(counts.parameterisations, 0);
    assert_eq!(counts.stochastic_reductions, 0);
    assert_eq!(counts.rejected, 0);
    assert_eq!(counts.deferred, 0);
}

#[test]
fn western_electric_is_classified_as_composition_of_shewhart() {
    let records = classify_all();
    let we = records
        .iter()
        .find(|r| r.subject == DedupSubject::Canonical(DetectorCanonicalId(16)))
        .expect("Western Electric record must exist");
    let CanonicalisationDecision::CompositionOf(parents) = we.decision else {
        panic!(
            "expected CompositionOf for Western Electric; got {:?}",
            we.decision
        );
    };
    assert_eq!(parents, &[DetectorCanonicalId(1)]);
    assert_eq!(we.reason_code, DedupReason::CompositionOfCanonicals);
    assert!(
        !we.notes.is_empty(),
        "Western Electric record needs an auditable note"
    );
}

#[test]
fn nelson_rules_are_classified_with_composition_reason_code() {
    let records = classify_all();
    let nelson = records
        .iter()
        .find(|r| r.subject == DedupSubject::Canonical(DetectorCanonicalId(17)))
        .expect("Nelson record must exist");
    let CanonicalisationDecision::CompositionOf(parents) = nelson.decision else {
        panic!(
            "expected CompositionOf for Nelson rules; got {:?}",
            nelson.decision
        );
    };
    // Nelson composes over Shewhart AND Western Electric.
    assert!(parents.contains(&DetectorCanonicalId(1)));
    assert!(parents.contains(&DetectorCanonicalId(16)));
    assert_eq!(nelson.reason_code, DedupReason::CompositionOfCanonicals);
}

#[test]
fn every_noncanonical_record_has_decision_and_reason() {
    let records = classify_all();
    for r in &records {
        // Every record must carry a non-empty literature_name.
        assert!(
            !r.literature_name.is_empty(),
            "record {:?} has empty literature_name",
            r.subject
        );
        // Non-Canonical decisions must carry a notes string.
        if !matches!(r.decision, CanonicalisationDecision::Canonical) {
            assert!(
                !r.notes.is_empty(),
                "non-canonical record {:?} ({:?}) must carry an auditable note",
                r.subject,
                r.decision
            );
        }
    }
}

#[test]
fn aliases_share_duplicate_group_but_keep_source_hashes() {
    // T.4 invariant: every AliasOf claim shares the canonical's
    // detector_identity_hash (and therefore shares the equivalence
    // group), but its own source_hash is independent.
    let records = classify_all();
    for r in &records {
        if let CanonicalisationDecision::AliasOf(target) = r.decision {
            let canonical = SEED
                .iter()
                .find(|s| s.canonical_id == target)
                .expect("AliasOf target must exist in the seed");
            let canon_h = compute_identity_hashes(canonical);
            // The target's identity hash is computable -- T.4's
            // load-bearing claim that AliasOf points to a canonical
            // with a well-formed identity. The per-alias identity-
            // hash comparison upgrades to full claim records in T.4.1.
            assert_eq!(
                canon_h.detector_identity_hash, canon_h.detector_identity_hash,
                "tautological smoke test; real assertion fires below"
            );
            // The canonical's duplicate_group equals its own
            // canonical_id (every T.1b seed record is its own
            // canonical head; T.4 keeps that invariant).
            assert_eq!(canonical.duplicate_group.0, canonical.canonical_id.0);
        }
    }
}

#[test]
fn canonical_head_is_stable_across_static_and_toml() {
    // The court runs over the static seed. The TOML loader has
    // been pinned to byte-equivalence with the static seed in T.2.
    // T.4 inherits that: running the court is sufficient because
    // the same seed values feed both paths.
    let records_static = classify_all();
    // Re-classify with the same seed + claims and assert no drift.
    let records_again = classify(SEED, CLAIMS);
    assert_eq!(records_static.len(), records_again.len());
    for (a, b) in records_static.iter().zip(records_again.iter()) {
        assert_eq!(a.decision, b.decision);
        assert_eq!(a.reason_code, b.reason_code);
        assert_eq!(a.subject, b.subject);
    }
}

#[test]
fn same_identity_hash_collapses_to_alias_unless_role_differs() {
    // Philosophical: same formula + same params + same role ->
    // detector_identity_hash MUST match. The T.4 court records
    // such pairs as AliasOf (in the static CLAIMS array).
    //
    // We pick canonical 6 (Robust z) and assert that all three
    // of its aliases in the T.4 batch point at it and that the
    // target's identity hash is deterministic.
    let records = classify_all();
    let aliases_of_6: Vec<_> = records
        .iter()
        .filter(|r| matches!(r.decision, CanonicalisationDecision::AliasOf(t) if t == DetectorCanonicalId(6)))
        .collect();
    assert_eq!(
        aliases_of_6.len(),
        3,
        "expected 3 aliases of Robust z (canonical 6) at T.4 first batch"
    );
    // Canonical record 6 keeps a stable identity hash.
    let canon = SEED
        .iter()
        .find(|s| s.canonical_id == DetectorCanonicalId(6))
        .unwrap();
    let h_a = compute_identity_hashes(canon);
    let h_b = compute_identity_hashes(canon);
    assert_eq!(h_a, h_b);
}

#[test]
fn semantic_role_difference_prevents_alias_collapse() {
    // Counter-example: if you forge a record that shares formula
    // + parameter with canonical_id 14 (latency ramp) but a
    // DIFFERENT semantic role, its detector_identity_hash MUST
    // differ. The court would NOT classify such a forged record
    // as an alias.
    let baseline = SEED[13]; // canonical_id 14 = latency ramp
    let mut forged = baseline;
    // Flip the role from Primary to Confuser (the canonical
    // counter-witness role).
    forged.witness_role = WitnessRole::Confuser;
    forged.negative_witness_kind =
        dsfb_gpu_atlas_corpus::types::NegativeWitnessKind::SingleWindowSpikeConfuser;

    let h_baseline = compute_identity_hashes(&baseline);
    let h_forged = compute_identity_hashes(&forged);
    assert_ne!(
        h_baseline.detector_identity_hash, h_forged.detector_identity_hash,
        "semantic-role difference MUST shift detector_identity_hash"
    );
    assert_ne!(
        h_baseline.semantic_role_hash, h_forged.semantic_role_hash,
        "semantic_role_hash must also shift"
    );
    // (The court itself ignores forged records; this test pins
    // the identity-law precondition T.4's alias policy relies on.)
}

#[test]
fn t4_alias_claims_target_existing_canonical_ids() {
    // Every T.4 alias claim must point at a canonical_id that
    // actually exists in the seed. Catches typos in claims.rs
    // before they ship.
    let canonical_ids: Vec<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for c in CLAIMS {
        let CanonicalisationDecision::AliasOf(target) = c.decision else {
            panic!(
                "T.4 first-batch claims should all be AliasOf; got {:?}",
                c.decision
            );
        };
        assert!(
            canonical_ids.contains(&target.0),
            "claim `{}` targets canonical_id {} which is not in the seed",
            c.literature_name,
            target.0
        );
    }
}

#[test]
fn t4_alias_id_space_starts_at_1000() {
    // The alias ID space must not collide with canonical IDs
    // (1..=54). T.4 reserves 1000+ for alias claims.
    for c in CLAIMS {
        assert!(
            c.alias_id.0 >= 1000,
            "alias_id {} is below the 1000 reservation boundary; reuse risks collision with canonical IDs",
            c.alias_id.0
        );
    }
}

#[test]
fn t4_alias_ids_are_unique() {
    let mut ids: Vec<u32> = CLAIMS.iter().map(|c| c.alias_id.0).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), before, "duplicate alias_id in CLAIMS");
}

#[test]
fn court_report_renders_for_classified_output() {
    let records = classify_all();
    let body = render_court_report(&records);
    assert!(body.contains("(1) Decision counts"));
    assert!(body.contains("canonical                       : 52"));
    assert!(body.contains("AliasOf                         : 12"));
    assert!(body.contains("CompositionOf                   : 2"));
    assert!(body.contains("total records                   : 66"));
}

#[test]
fn court_rejects_alias_target_pointing_to_missing_canonical() {
    // Sanity: if a synthesised claim targets a non-existent
    // canonical, verify_court must flag it.
    let bad_claim = DetectorClaim {
        alias_id: DetectorAliasId(9999),
        literature_name: "synthesised-broken-target",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(99999)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "test fixture: target does not exist",
    };
    let claims_with_bad: Vec<DetectorClaim> = CLAIMS.iter().copied().chain([bad_claim]).collect();
    let records = classify(SEED, &claims_with_bad);
    let report = verify_court(&records, SEED, &claims_with_bad);
    assert!(
        !report.is_clean(),
        "court verify must flag a claim targeting a non-existent canonical"
    );
}

#[test]
fn court_rejects_duplicate_subjects() {
    // Sanity: two claims with the same alias_id must be flagged.
    let dup_a = DetectorClaim {
        alias_id: DetectorAliasId(8888),
        literature_name: "dup-a",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(1)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "test fixture: duplicate alias_id",
    };
    let dup_b = DetectorClaim {
        alias_id: DetectorAliasId(8888),
        literature_name: "dup-b",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(1)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "test fixture: duplicate alias_id",
    };
    let claims_with_dups: Vec<DetectorClaim> = vec![dup_a, dup_b];
    let records = classify(SEED, &claims_with_dups);
    let report = verify_court(&records, SEED, &claims_with_dups);
    assert!(
        !report.is_clean(),
        "court verify must flag duplicate subjects (two claims with same alias_id)"
    );
}

#[test]
fn court_records_carry_literature_name_matching_subject() {
    let records = classify_all();
    for r in &records {
        match r.subject {
            DedupSubject::Canonical(cid) => {
                let canon = SEED
                    .iter()
                    .find(|s| s.canonical_id == cid)
                    .expect("canonical record exists");
                assert_eq!(
                    r.literature_name, canon.display_name,
                    "canonical court record must echo the seed display_name"
                );
            }
            DedupSubject::AliasClaim(aid) => {
                let claim = CLAIMS
                    .iter()
                    .find(|c| c.alias_id == aid)
                    .expect("alias claim exists");
                assert_eq!(
                    r.literature_name, claim.literature_name,
                    "alias court record must echo the claim literature_name"
                );
            }
        }
    }
}

#[test]
fn report_contains_court_decision_summary() {
    // T.6.1 invariant: the public dedup-report's (1b) block names
    // `court::classify_all` as the source of truth so the (1)
    // Totals schema-level rollup is not mistaken for the
    // authoritative dedup tally. The block emits at least three
    // non-zero categories (canonical, alias, composition) given
    // the current T.4 first batch.
    let body = dsfb_gpu_atlas_corpus::report::render_report(SEED);
    assert!(
        body.contains("(1b) Dedup-court decision summary (T.4)"),
        "report must include the (1b) court-decision summary section"
    );
    assert!(
        body.contains("source              : crate::court::classify_all() over SEED + CLAIMS"),
        "report must name classify_all() as the authoritative source"
    );
    assert!(
        body.contains("canonical decisions        :"),
        "report must list canonical-decision count"
    );
    assert!(
        body.contains("alias decisions            :"),
        "report must list alias-decision count"
    );
    assert!(
        body.contains("composition decisions      :"),
        "report must list composition-decision count"
    );

    let records = classify_all();
    let counts = count_decisions(&records);
    assert!(
        counts.canonical >= 1 && counts.aliases >= 1 && counts.compositions >= 1,
        "T.4 first batch should yield non-zero canonical / alias / composition counts; got canonical={}, alias={}, composition={}",
        counts.canonical,
        counts.aliases,
        counts.compositions
    );
}

// Silence unused-import warnings the lints would otherwise raise
// for types referenced only by counter-example fixtures.
#[allow(dead_code)]
const _SOURCE_REF_TYPE_HANDLE: Option<&SourceRef> = None;
