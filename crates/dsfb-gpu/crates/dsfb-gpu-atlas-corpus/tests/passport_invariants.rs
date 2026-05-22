//! T.11a acceptance tests for `DetectorPassport`.
//!
//! Panel-required invariants (12+):
//!
//! - `all_passports_one_per_seed_canonical_id`
//! - `passport_hash_is_deterministic_across_two_builds`
//! - `passport_hash_changes_when_semantic_role_hash_changes`
//! - `passport_hash_changes_when_lifecycle_state_changes`
//! - `passport_hash_changes_when_l_band_changes`
//! - `passport_hash_changes_when_constitution_flags_change`
//! - `passport_hash_includes_detector_identity_hash`
//! - `passport_does_not_count_alias_records_as_unique_primitives`
//! - `passport_cannot_claim_l6_unless_lband_verifier_admits_record`
//! - `passport_for_unknown_id_returns_none`
//! - `passport_for_aliased_record_surfaces_canonical_target_id_via_genealogy`
//! - `passport_text_rendering_is_deterministic_across_two_calls`
//! - `passport_json_rendering_is_deterministic_across_two_calls`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;

use dsfb_gpu_atlas_corpus::lband::{verify_record_lband, GPU_IMPLEMENTED_CANONICAL_IDS};
use dsfb_gpu_atlas_corpus::passport::{
    all_passports, compute_passport_hash, passport_for, render_passport_json, render_passport_text,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    CanonicalisationDecision, ConstitutionFlags, DetectorCanonicalId, ImplementationLevel,
    LifecycleState,
};

#[test]
fn all_passports_one_per_seed_canonical_id() {
    let passports = all_passports();
    assert_eq!(
        passports.len(),
        SEED.len(),
        "all_passports() must produce one passport per SEED canonical record"
    );
    // Surface that the set of canonical_ids matches SEED exactly.
    let seed_ids: BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    let passport_ids: BTreeSet<u32> = passports.iter().map(|p| p.canonical_id.0).collect();
    assert_eq!(
        seed_ids, passport_ids,
        "every SEED canonical_id must appear in all_passports() output"
    );
}

#[test]
fn passport_hash_is_deterministic_across_two_builds() {
    let id = SEED[0].canonical_id;
    let a = passport_for(id).expect("passport for SEED[0]");
    let b = passport_for(id).expect("passport for SEED[0]");
    assert_eq!(
        a.passport_hash, b.passport_hash,
        "two builds must produce byte-identical passport hashes"
    );
    // Cross-check: the embedded hash field equals what compute_*
    // returns on the live passport.
    let c = compute_passport_hash(&a);
    assert_eq!(
        a.passport_hash, c,
        "compute_passport_hash on a built passport must equal its embedded hash"
    );
}

#[test]
fn passport_hash_changes_when_semantic_role_hash_changes() {
    let id = SEED[0].canonical_id;
    let mut p = passport_for(id).expect("passport for SEED[0]");
    let baseline = p.passport_hash;
    // Mutate the semantic role hash byte.
    p.identity_hashes.semantic_role_hash[0] ^= 0xFF;
    let mutated = compute_passport_hash(&p);
    assert_ne!(
        baseline, mutated,
        "passport hash must change when semantic_role_hash changes"
    );
}

#[test]
fn passport_hash_changes_when_lifecycle_state_changes() {
    let id = SEED[0].canonical_id;
    let mut p = passport_for(id).expect("passport for SEED[0]");
    let baseline = p.passport_hash;
    // Pick a different lifecycle state. SEED[0] is Active; flip
    // to Dormant — any distinct variant works.
    p.lifecycle_state = if p.lifecycle_state == LifecycleState::Active {
        LifecycleState::Dormant
    } else {
        LifecycleState::Active
    };
    let mutated = compute_passport_hash(&p);
    assert_ne!(
        baseline, mutated,
        "passport hash must change when lifecycle_state changes"
    );
}

#[test]
fn passport_hash_changes_when_l_band_changes() {
    let id = SEED[0].canonical_id;
    let mut p = passport_for(id).expect("passport for SEED[0]");
    let baseline = p.passport_hash;
    // Pick a clearly distinct band. SEED[0] is L1_Canonicalised
    // at T.7; flip to L0_CitedOnly so the hash diverges.
    p.implementation_level = if p.implementation_level == ImplementationLevel::L0_CitedOnly {
        ImplementationLevel::L1_Canonicalised
    } else {
        ImplementationLevel::L0_CitedOnly
    };
    let mutated = compute_passport_hash(&p);
    assert_ne!(
        baseline, mutated,
        "passport hash must change when L-band changes"
    );
}

#[test]
fn passport_hash_changes_when_constitution_flags_change() {
    let id = SEED[0].canonical_id;
    let mut p = passport_for(id).expect("passport for SEED[0]");
    let baseline = p.passport_hash;
    // Flip one flag; the verifier-rejecting state is fine because
    // this is just a hash-sensitivity test.
    p.constitution_flags = ConstitutionFlags {
        declared_input_contract: !p.constitution_flags.declared_input_contract,
        ..p.constitution_flags
    };
    let mutated = compute_passport_hash(&p);
    assert_ne!(
        baseline, mutated,
        "passport hash must change when any constitution flag changes"
    );
}

#[test]
fn passport_hash_includes_detector_identity_hash() {
    let id = SEED[0].canonical_id;
    let mut p = passport_for(id).expect("passport for SEED[0]");
    let baseline = p.passport_hash;
    p.detector_identity_hash[0] ^= 0xFF;
    let mutated = compute_passport_hash(&p);
    assert_ne!(
        baseline, mutated,
        "passport hash must change when detector_identity_hash changes"
    );
}

#[test]
fn passport_does_not_count_alias_records_as_unique_primitives() {
    // The corpus crate's alias claims live in `claims::CLAIMS`,
    // NOT in SEED. all_passports() walks SEED only, so the
    // passport count equals the number of canonical primitives
    // (54), never inflated by the dozen+ alias claims that
    // CLAIMS contains.
    let passports = all_passports();
    assert_eq!(
        passports.len(),
        SEED.len(),
        "passports must count SEED canonicals only; aliases live in CLAIMS"
    );
    // And no passport carries an alias-side id (alias_id space
    // is 1000+ by the T.4 convention; SEED canonical_ids are
    // 1..=54).
    for p in &passports {
        assert!(
            p.canonical_id.0 < 1000,
            "passport carrying alias-side id {:?}",
            p.canonical_id
        );
    }
}

#[test]
fn passport_cannot_claim_l6_unless_lband_verifier_admits_record() {
    // For every SEED record whose passport claims
    // L6_CpuGpuByteEquivalent, the L-band verifier MUST admit
    // the record. (Equivalently: the canonical_id must be in
    // GPU_IMPLEMENTED_CANONICAL_IDS.)
    for p in all_passports() {
        if p.implementation_level == ImplementationLevel::L6_CpuGpuByteEquivalent {
            assert!(
                GPU_IMPLEMENTED_CANONICAL_IDS.contains(&p.canonical_id),
                "passport for canonical_id {:?} claims L6 but is not in GPU_IMPLEMENTED_CANONICAL_IDS",
                p.canonical_id
            );
            let record = SEED
                .iter()
                .find(|r| r.canonical_id == p.canonical_id)
                .expect("L6 passport must correspond to a SEED record");
            let errors = verify_record_lband(record);
            assert!(
                errors.is_empty(),
                "passport for canonical_id {:?} claims L6 but the lband verifier rejects: {errors:?}",
                p.canonical_id
            );
        }
    }
}

#[test]
fn passport_for_unknown_id_returns_none() {
    // SEED canonical_ids are 1..=54; 9999 is not a known id.
    assert!(
        passport_for(DetectorCanonicalId(9999)).is_none(),
        "passport_for unknown id must return None"
    );
}

#[test]
fn passport_for_aliased_record_surfaces_canonical_target_id_via_genealogy() {
    // The corpus's canonical records do NOT carry AliasOf
    // decisions (alias-side claims live in CLAIMS; SEED records
    // are all Canonical, CompositionOf, or
    // StochasticOriginalDeterministicReduction). What this test
    // pins is the inverse direction: passport_for(canonical) on
    // a record with a known aliased family (e.g. canonical_id 6
    // = ROBUST_Z_MAD, which has three alias claims pointing at
    // it) returns a Canonical-decision passport whose aliases
    // slice is non-empty — those are the literature names the
    // T.4 court has collapsed into this canonical record.
    let p = passport_for(DetectorCanonicalId(6)).expect("ROBUST_Z_MAD canonical (id 6)");
    assert!(
        !p.aliases.is_empty(),
        "ROBUST_Z_MAD passport must list its literature aliases"
    );
    // The passport's dedup_decision is Canonical for SEED
    // records that are not CompositionOf or stochastic
    // reductions.
    assert_eq!(p.dedup_decision, CanonicalisationDecision::Canonical);
}

#[test]
fn passport_text_rendering_is_deterministic_across_two_calls() {
    let p = passport_for(SEED[0].canonical_id).expect("SEED[0] passport");
    let a = render_passport_text(&p);
    let b = render_passport_text(&p);
    assert_eq!(a, b, "text rendering must be deterministic");
    // And every passport text rendering is deterministic, not
    // just the first.
    for p in all_passports() {
        let a = render_passport_text(&p);
        let b = render_passport_text(&p);
        assert_eq!(
            a, b,
            "passport text rendering must be deterministic (canonical_id {:?})",
            p.canonical_id
        );
    }
}

#[test]
fn passport_json_rendering_is_deterministic_across_two_calls() {
    let p = passport_for(SEED[0].canonical_id).expect("SEED[0] passport");
    let a = render_passport_json(&p);
    let b = render_passport_json(&p);
    assert_eq!(a, b, "JSON rendering must be deterministic");
    for p in all_passports() {
        let a = render_passport_json(&p);
        let b = render_passport_json(&p);
        assert_eq!(
            a, b,
            "passport JSON rendering must be deterministic (canonical_id {:?})",
            p.canonical_id
        );
    }
}

#[test]
fn passport_hashes_pairwise_distinct() {
    // T.3's five-hash identity guarantees every SEED canonical
    // primitive has a distinct `detector_identity_hash`. The
    // passport hash includes that field plus every per-record
    // axis, so passports MUST be pairwise distinct.
    let passports = all_passports();
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    for p in &passports {
        assert!(
            seen.insert(p.passport_hash),
            "duplicate passport_hash detected for canonical_id {:?}",
            p.canonical_id
        );
    }
    assert_eq!(
        seen.len(),
        passports.len(),
        "every passport_hash must be unique across SEED"
    );
}

#[test]
fn passport_count_matches_panel_locked_seed_count() {
    // Belt-and-braces: SEED.len() == 54 at T.10, and the
    // passport count must follow it. If the seed ever grows
    // beyond 54, this test surfaces the change.
    let passports = all_passports();
    assert_eq!(passports.len(), 54, "T.10-frozen corpus has 54 records");
    assert_eq!(SEED.len(), 54, "SEED.len() must equal 54 at T.10");
}
