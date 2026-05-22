//! T.12.o acceptance suite — Streaming Sketches expansion
//! proposal invariants.
//!
//! Six panel-required load-bearing negatives pin the contract
//! discipline T.12.o exists to prove:
//!
//! * `t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract`
//!   (every hash-based sketch CanonicalAddition declares hash
//!   family + width / bucket count + seed; every deterministic
//!   sketch declares width / k + update rule + merge law)
//! * `t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty`
//!   (forbidden-term scanner for deterministic-accuracy-bound
//!   language; sketch outputs are PROBABILISTIC estimates under
//!   a declared error-bound contract, never deterministic
//!   certainty)
//! * `t12_o_rejects_approximate_query_truth_claim_language`
//!   (forbidden-term scanner for approximate-query-truth /
//!   database-correctness-verdict / sketch-query-verdict
//!   language)
//! * `t12_o_rejects_privacy_or_anonymization_claim_language`
//!   (forbidden-term scanner for differential-privacy /
//!   k-anonymous-output / anonymization-authority language)
//! * `t12_o_rejects_mergeable_sketch_without_merge_law`
//!   (every CanonicalAddition that claims "mergeable" must
//!   declare a merge law)
//! * `t12_o_rejects_black_box_streaming_anomaly_score_without_formula`
//!   (6513 + 6514 rejections require deterministic formula +
//!   training-data anchor / declared hash / width / depth /
//!   seed / merge contract + tie-break + numeric mode)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.o admits deterministic streaming-sketch witnesses:
//! > bounded-memory, mergeable or update-order-declared
//! > summaries for frequency, cardinality, quantile, heavy-
//! > hitter, membership, and moment / variance evidence. It
//! > does not admit probabilistic accuracy as certainty,
//! > randomized sketch behavior without seed / width / depth /
//! > hash-family declaration, privacy claims, database
//! > correctness authority, or approximate-query truth.

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
use dsfb_gpu_atlas_corpus::t12_d_robust::seed_t12_d_robust_proposal;
use dsfb_gpu_atlas_corpus::t12_e_spectral::seed_t12_e_spectral_proposal;
use dsfb_gpu_atlas_corpus::t12_f_timeseries::seed_t12_f_timeseries_proposal;
use dsfb_gpu_atlas_corpus::t12_g_graph::seed_t12_g_graph_proposal;
use dsfb_gpu_atlas_corpus::t12_h_dataquality::seed_t12_h_dataquality_proposal;
use dsfb_gpu_atlas_corpus::t12_i_observability::seed_t12_i_observability_proposal;
use dsfb_gpu_atlas_corpus::t12_j_biosignal::seed_t12_j_biosignal_proposal;
use dsfb_gpu_atlas_corpus::t12_k_industrial::seed_t12_k_industrial_proposal;
use dsfb_gpu_atlas_corpus::t12_l_chemometrics::seed_t12_l_chemometrics_proposal;
use dsfb_gpu_atlas_corpus::t12_m_rf::seed_t12_m_rf_proposal;
use dsfb_gpu_atlas_corpus::t12_n_econometrics_reliability::seed_t12_n_econometrics_reliability_proposal;
use dsfb_gpu_atlas_corpus::t12_o_streaming_sketches::{
    seed_t12_o_streaming_sketches_proposal, AMS_RESERVED_CANONICAL_ID,
    BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID,
    BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID, BLOOM_RESERVED_CANONICAL_ID,
    CARDINALITY_DRIFT_SEED_ID, CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, CMS_RESIDUAL_RESERVED_CANONICAL_ID, ERROR_BURST_SEED_ID,
    FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID, GREENWALD_KHANNA_RESERVED_CANONICAL_ID,
    HLL_RESERVED_CANONICAL_ID, KS_SEED_ID, LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID,
    MISRA_GRIES_RESERVED_CANONICAL_ID, MISSINGNESS_SPIKE_SEED_ID,
    SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID, SPACE_SAVING_RESERVED_CANONICAL_ID,
    STREAMING_KS_RESERVED_PRIMITIVE_ID, T_DIGEST_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12O_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (KS_SEED_ID, "Kolmogorov-Smirnov two-sample test"),
    (MISSINGNESS_SPIKE_SEED_ID, "Missingness spike"),
    (ERROR_BURST_SEED_ID, "Error burst"),
    (CARDINALITY_DRIFT_SEED_ID, "Cardinality drift"),
];

/// Hash-based sketch CanonicalAddition records with PER-ROW or
/// PER-SKETCH SEED arrays (CMS / Bloom / AMS). Each MUST
/// declare a hash family + width / bucket count / bit-array
/// size + seed array.
const T12O_HASH_BASED_SEEDED_CANONICAL_IDS: &[u32] = &[
    CMS_RESIDUAL_RESERVED_CANONICAL_ID,
    BLOOM_RESERVED_CANONICAL_ID,
    AMS_RESERVED_CANONICAL_ID,
];

/// HyperLogLog uses a SINGLE hash function with a declared
/// precision parameter rather than a per-row seed array.
/// Treated as hash-based but its contract is hash family +
/// bucket count m = 2^precision (no separate "seed array").
const T12O_HLL_CANONICAL_ID: u32 = HLL_RESERVED_CANONICAL_ID;

/// Deterministic-counts / deterministic-tuple sketch
/// CanonicalAddition records (Misra-Gries / Space-Saving /
/// Greenwald-Khanna / t-digest). Each MUST declare a width (k
/// counters or epsilon bound or compression delta) + update
/// rule + merge law.
const T12O_DETERMINISTIC_CANONICAL_IDS: &[u32] = &[
    MISRA_GRIES_RESERVED_CANONICAL_ID,
    SPACE_SAVING_RESERVED_CANONICAL_ID,
    GREENWALD_KHANNA_RESERVED_CANONICAL_ID,
    T_DIGEST_RESERVED_CANONICAL_ID,
];

/// Forbidden terms turning probabilistic error bounds into
/// deterministic certainty claims.
const T12O_FORBIDDEN_PROBABILISTIC_AS_CERTAINTY: &[&str] = &[
    "deterministic accuracy bound",
    "deterministic count certainty",
    "exact within probabilistic error",
    "sketch estimate is exact",
    "approximate count is exact",
    "guaranteed accuracy",
];

/// Forbidden approximate-query-truth / database-correctness-
/// verdict terms.
const T12O_FORBIDDEN_APPROXIMATE_QUERY_TRUTH: &[&str] = &[
    "approximate query truth",
    "sketch query verdict",
    "database correctness verdict",
    "approx_query_is_truth",
    "issues approximate query truth",
];

/// Forbidden privacy / anonymization claim terms. The Bloom-
/// filter CanonicalAddition record legitimately disclaims
/// "anonymization authority" inside a "does NOT admit ..."
/// sentence, so the bare phrase "anonymization authority" is
/// NOT forbidden; only positive-claim variants are.
const T12O_FORBIDDEN_PRIVACY_OR_ANONYMIZATION: &[&str] = &[
    "differential privacy guarantee",
    "privacy-preserving certainty",
    "k-anonymous output",
    "issues privacy verdicts",
    "issues anonymization",
    "claims anonymization authority",
    "claims differential privacy",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn o_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "T.12.o proposal failed verifier: {errors:?}"
    );
}

#[test]
fn o_proposal_has_open_status() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn o_proposal_targets_streaming_sketches() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::StreamingSketches
    ));
}

/// Load-bearing negative #1 (panel-required).
#[test]
fn t12_o_does_not_mutate_seed_len() {
    let _ = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn o_proposal_proposes_fourteen_primitives() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 14);
}

#[test]
fn o_proposal_proposes_zero_aliases() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn o_proposal_proposes_twenty_dedup_records() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 20);
}

#[test]
fn o_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn o_proposal_proposes_ten_source_refs() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 10);
}

#[test]
fn o_delta_has_eight_new_canonical_records() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn o_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn o_proposal_carries_two_rejection_records() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.o must carry TWO RejectedNotDeterministic records \
        (ninth T.12.x with two, following T.12.g / h / i / j / k / l / m / n)"
    );
}

#[test]
fn o_proposal_court_delta_category_counts() {
    let p = seed_t12_o_streaming_sketches_proposal();
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
            other => panic!("unexpected wire-name: {other}"),
        }
    }
    assert_eq!(canonical, 8);
    assert_eq!(existing, 4);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn o_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_o_streaming_sketches_proposal();
    let b = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_o_proposal_hash_matches_stored() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn o_proposal_hash_is_distinct_from_all_prior_t12x() {
    let o = seed_t12_o_streaming_sketches_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    let ts = seed_t12_f_timeseries_proposal();
    let graph = seed_t12_g_graph_proposal();
    let dq = seed_t12_h_dataquality_proposal();
    let obs = seed_t12_i_observability_proposal();
    let bio = seed_t12_j_biosignal_proposal();
    let ind = seed_t12_k_industrial_proposal();
    let chem = seed_t12_l_chemometrics_proposal();
    let rf = seed_t12_m_rf_proposal();
    let econ = seed_t12_n_econometrics_reliability_proposal();
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio, ind, chem, rf, econ,
    ] {
        assert_ne!(
            o.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.o hash must differ from every prior T.12.x"
        );
    }
}

#[test]
fn t12_o_hash_changes_when_sketch_contract_changes() {
    let p_a = seed_t12_o_streaming_sketches_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CMS_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("CMS CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED CMS hash family / width contract for hash-sensitivity test",
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
// SEED collision load-bearing negative
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_o_collision_batch",
        SourceClass::StreamingSketches,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "streaming-sketches collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_o_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_o_collision_proposal",
        "Defective T.12.o proposal duplicating an existing SEED canonical.",
        SourceClass::StreamingSketches,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_o_test",
    )
}

#[test]
fn t12_o_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12O_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id);
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

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #1: every CanonicalAddition
// declares hash family + width + depth + seed (for hash-based
// sketches) OR width / k + update rule + merge law (for
// deterministic sketches).
// ---------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)]
fn t12_o_rejects_sketch_without_hash_family_width_depth_or_seed_contract() {
    let p = seed_t12_o_streaming_sketches_proposal();
    // Hash-based seeded sketches (CMS / Bloom / AMS): must
    // declare hash family + width-or-bucket-count + per-row or
    // per-sketch seed.
    for canonical_id in T12O_HASH_BASED_SEEDED_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Hash-based seeded CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        assert!(
            lower.contains("hash family"),
            "Hash-based seeded canonical_id={} must declare hash family: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("width ")
                || lower.contains("bucket count")
                || lower.contains("bit-array size")
                || lower.contains("sketch width"),
            "Hash-based seeded canonical_id={} must declare width / bucket count / \
             bit-array size: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("seed"),
            "Hash-based seeded canonical_id={} must declare seed array: {:?}",
            canonical_id,
            rec.reason
        );
    }
    // HyperLogLog: single hash function with declared precision
    // parameter (m = 2^precision); no separate per-row seed
    // array. Contract is hash family + bucket count m = 2^
    // precision + harmonic-mean estimator.
    let hll = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == T12O_HLL_CANONICAL_ID
        })
        .expect("HLL CanonicalAddition record must exist");
    let lower = hll.reason.to_lowercase();
    assert!(
        lower.contains("hash family"),
        "HLL canonical_id={} must declare hash family: {:?}",
        T12O_HLL_CANONICAL_ID,
        hll.reason
    );
    assert!(
        lower.contains("bucket count"),
        "HLL canonical_id={} must declare bucket count m = 2^precision: {:?}",
        T12O_HLL_CANONICAL_ID,
        hll.reason
    );
    assert!(
        lower.contains("precision parameter"),
        "HLL canonical_id={} must declare precision parameter (the bucket-count basis): {:?}",
        T12O_HLL_CANONICAL_ID,
        hll.reason
    );
    assert!(
        lower.contains("harmonic-mean estimator"),
        "HLL canonical_id={} must declare harmonic-mean estimator: {:?}",
        T12O_HLL_CANONICAL_ID,
        hll.reason
    );
    // Deterministic sketches: must declare width (k counters /
    // epsilon bound / compression delta) + update rule + merge
    // law.
    for canonical_id in T12O_DETERMINISTIC_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Deterministic CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        // Width contract: "k counter slots" OR "epsilon error
        // bound" OR "compression delta".
        assert!(
            lower.contains("counter slots")
                || lower.contains("epsilon error bound")
                || lower.contains("compression delta"),
            "Deterministic canonical_id={} must declare width contract \
             (k counter slots / epsilon error bound / compression delta): {:?}",
            canonical_id,
            rec.reason
        );
        // Update rule contract: any of the deterministic-update
        // phrases.
        assert!(
            lower.contains("decrement-on-miss law")
                || lower.contains("replace-smallest-on-miss law")
                || lower.contains("tuple-insertion rule")
                || lower.contains("centroid scale function"),
            "Deterministic canonical_id={} must declare update rule: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("merge law"),
            "Deterministic canonical_id={} must declare merge law: {:?}",
            canonical_id,
            rec.reason
        );
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #2: probabilistic-bound-
// as-deterministic-certainty scanner.
// ---------------------------------------------------------------

#[test]
fn t12_o_rejects_probabilistic_error_bound_as_deterministic_certainty() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12O_FORBIDDEN_PROBABILISTIC_AS_CERTAINTY {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden probabilistic-as-deterministic-certainty term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #3: approximate-query-
// truth claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_o_rejects_approximate_query_truth_claim_language() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12O_FORBIDDEN_APPROXIMATE_QUERY_TRUTH {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden approximate-query-truth term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #4: privacy / anonymization
// claim-language scanner. Forbidden terms must appear NOWHERE in
// the proposal, including in the RejectedNotDeterministic
// records (the campaign refuses to claim privacy authority OR
// to reject vendor sketches FOR privacy violations; sketches
// are simply sketches with declared probabilistic error bounds).
// ---------------------------------------------------------------

#[test]
fn t12_o_rejects_privacy_or_anonymization_claim_language() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        let lower = r.reason.to_lowercase();
        for term in T12O_FORBIDDEN_PRIVACY_OR_ANONYMIZATION {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden privacy / anonymization claim term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #5: every CanonicalAddition
// that claims mergeability must declare a merge law.
// ---------------------------------------------------------------

#[test]
fn t12_o_rejects_mergeable_sketch_without_merge_law() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION {
            continue;
        }
        let lower = r.reason.to_lowercase();
        let claims_mergeable = lower.contains("mergeable");
        if claims_mergeable {
            assert!(
                lower.contains("merge law"),
                "CanonicalAddition canonical_id={} claims mergeability but does not declare \
                a merge law: {:?}",
                r.canonical_id.0,
                r.reason
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #6: black-box streaming-
// anomaly score / vendor sketch rejections require deterministic
// formula + training-data anchor / declared hash / width / depth
// / seed / merge contract + tie-break + numeric mode.
// ---------------------------------------------------------------

#[test]
fn t12_o_rejects_black_box_streaming_anomaly_score_without_formula() {
    let p = seed_t12_o_streaming_sketches_proposal();
    // Learned streaming-anomaly score (6513).
    let learned = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned streaming-anomaly rejection record must exist");
    let r = learned.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == LEARNED_STREAMING_ANOMALY_RESERVED_PRIMITIVE_ID),
        "Learned streaming-anomaly score must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "Learned streaming-anomaly rejection must require 'deterministic feature-extraction law': {}",
        learned.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("declared update rule"));
    assert!(r.contains("declared training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    assert!(r.contains("no learned opaque embedding"));
    let vendors = [
        "datadog watchdog",
        "datarobot",
        "splunk stream ml",
        "aws lookout for metrics",
        "azure anomaly detector",
    ];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Learned streaming-anomaly rejection must name at least one vendor"
    );
    // Black-box vendor sketch (6514).
    let vendor = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID
        })
        .expect("Vendor sketch rejection record must exist");
    let r = vendor.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == BLACK_BOX_VENDOR_SKETCH_RESERVED_PRIMITIVE_ID),
        "Vendor sketch must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("hash family")
            && r.contains("width")
            && r.contains("depth")
            && r.contains("seed")
            && r.contains("merge law"),
        "Vendor-sketch rejection must demand declared hash family + width + depth + seed + \
         merge law: {}",
        vendor.reason
    );
    let vendors = ["snowflake", "bigquery", "druid", "clickhouse", "aws athena"];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Vendor-sketch rejection must name at least one vendor"
    );
}

// ---------------------------------------------------------------
// Per-canonical structural-distinctness assertions
// ---------------------------------------------------------------

#[test]
fn t12_o_space_saving_distinct_from_misra_gries() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SPACE_SAVING_RESERVED_CANONICAL_ID
        })
        .expect("Space-Saving CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("replace-smallest-on-miss"));
    assert!(r.contains("structurally distinct from misra-gries"));
    assert!(r.contains("decrement-all-on-miss"));
}

#[test]
fn t12_o_hll_distinct_from_pre_hll_flajolet_martin() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == HLL_RESERVED_CANONICAL_ID
        })
        .expect("HLL CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("flajolet-fusy-gandouet-meunier 2007"));
    assert!(r.contains("harmonic-mean estimator"));
    assert!(r.contains("bias correction"));
}

#[test]
fn t12_o_t_digest_declares_deterministic_centroid_merge_law() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == T_DIGEST_RESERVED_CANONICAL_ID
        })
        .expect("t-digest CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("dunning 2019"));
    assert!(r.contains("deterministic centroid-merge law"));
    assert!(r.contains("explicitly not randomized"));
}

#[test]
fn t12_o_greenwald_khanna_declares_deterministic_epsilon_quantile() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == GREENWALD_KHANNA_RESERVED_CANONICAL_ID
        })
        .expect("Greenwald-Khanna CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("greenwald-khanna 2001"));
    assert!(r.contains("deterministic epsilon-approximate quantile guarantee"));
    assert!(r.contains("one-sided"));
}

#[test]
fn t12_o_ams_declares_4_wise_independent_hash_family() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == AMS_RESERVED_CANONICAL_ID
        })
        .expect("AMS CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("alon-matias-szegedy 1999"));
    assert!(r.contains("4-wise-independent hash family"));
    assert!(r.contains("moment order"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_o_flajolet_martin_is_parameterizationof_cardinality_drift() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == FLAJOLET_MARTIN_RESERVED_PRIMITIVE_ID
        })
        .expect("Flajolet-Martin must have ParameterizationOf record");
    assert!(rec.reason.contains("Cardinality drift"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("flajolet-martin 1985") || r.contains("durand-flajolet 2003"));
}

#[test]
fn t12_o_streaming_ks_is_parameterizationof_ks() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == STREAMING_KS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == STREAMING_KS_RESERVED_PRIMITIVE_ID
        })
        .expect("Streaming KS must have ParameterizationOf record");
    assert!(rec.reason.contains("Kolmogorov-Smirnov two-sample test"));
}

#[test]
fn t12_o_sliding_window_burst_is_parameterizationof_error_burst() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == SLIDING_WINDOW_BURST_SKETCH_RESERVED_PRIMITIVE_ID
        })
        .expect("Sliding-window burst sketch must have ParameterizationOf record");
    assert!(rec.reason.contains("Error burst"));
}

#[test]
fn t12_o_bloom_inversion_missingness_is_parameterizationof_missingness_spike() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == BLOOM_INVERSION_MISSINGNESS_RESERVED_PRIMITIVE_ID
        })
        .expect("Bloom-inversion missingness must have ParameterizationOf record");
    assert!(rec.reason.contains("Missingness spike"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_o_cardinality_drift_domain_transfer_to_streaming_sketches_exists() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == CARDINALITY_DRIFT_SEED_ID
        })
        .expect("Cardinality drift must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared cardinality ancestor"));
    assert!(r.contains("streamingsketches"));
}

#[test]
fn t12_o_ks_domain_transfer_to_streaming_sketches_exists() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF && r.canonical_id.0 == KS_SEED_ID
        })
        .expect("KS must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared distribution-") && r.contains("distance ancestor"));
    assert!(r.contains("streamingsketches"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_o_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "ExistingCanonicalAuthorityResolution target {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_o_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(
                seed_ids.contains(&r.canonical_id.0),
                "DomainTransferOf target {} not in SEED",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_o_authority_resolution_covers_all_t12o_seed_ids() {
    let p = seed_t12_o_streaming_sketches_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12O_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.o"
        );
    }
}

#[test]
fn t12_o_every_dedup_record_has_reason() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_o_canonical_addition_ids_are_in_6501_to_6508_range() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6501..=6508).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.o reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_o_parameterization_ids_are_in_6509_to_6512_range() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6509..=6512).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.o reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_o_rejection_ids_are_in_6513_to_6514_range() {
    let p = seed_t12_o_streaming_sketches_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6513..=6514).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.o reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn o_proposal_text_rendering_byte_stable() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn o_proposal_json_rendering_byte_stable() {
    let p = seed_t12_o_streaming_sketches_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
