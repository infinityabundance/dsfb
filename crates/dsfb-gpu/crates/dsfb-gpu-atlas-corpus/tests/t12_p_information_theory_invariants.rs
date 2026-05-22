//! T.12.p acceptance suite — Information Theory catch-up
//! expansion proposal invariants.
//!
//! Six panel-required load-bearing negatives pin the contract
//! discipline T.12.p exists to prove:
//!
//! * `t12_p_rejects_information_witness_without_estimator_or_binning_contract`
//!   (every CanonicalAddition declares estimator + binning OR
//!   partition law; every ExistingCanonicalAuthorityResolution
//!   declares estimator + bin / partition / FFT-window contract)
//! * `t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law`
//!   (every CanonicalAddition AND every authority resolution that
//!   carries an entropy-style functional declares log base AND
//!   empty-bin law AND smoothing rule)
//! * `t12_p_rejects_mutual_information_without_joint_distribution_contract`
//!   (every MI / conditional-entropy CanonicalAddition AND every
//!   MI-parameterization declares an explicit joint-distribution
//!   contract over (X, Y))
//! * `t12_p_rejects_causal_information_flow_claim_language`
//!   (forbidden-term scanner for causal flow / Granger-style
//!   causal / intervention-truth claim language across every
//!   non-rejection record)
//! * `t12_p_rejects_privacy_or_security_claim_language`
//!   (forbidden-term scanner for privacy-leakage certainty /
//!   cryptographic-security / information-theoretically-secure-
//!   encryption verdict language across every non-rejection
//!   record)
//! * `t12_p_rejects_learned_embedding_information_score_without_formula`
//!   (6610 + 6611 rejections require deterministic formula +
//!   training-data anchor / declared binning / declared log /
//!   smoothing / sample-correction law + tie-break + numeric mode)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.p admits deterministic information-theoretic witnesses:
//! > entropy, divergence, mutual-information, coding-length,
//! > compression, surprise, and dependence-structure evidence
//! > with declared estimator, binning, smoothing, sample-support,
//! > and numeric laws. It does not admit semantic meaning, causal
//! > information flow certainty, privacy leakage certainty,
//! > cryptographic security claims, or learned representation
//! > claims.

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
use dsfb_gpu_atlas_corpus::t12_o_streaming_sketches::seed_t12_o_streaming_sketches_proposal;
use dsfb_gpu_atlas_corpus::t12_p_information_theory::{
    seed_t12_p_information_theory_proposal, BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID,
    CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID, CROSS_ENTROPY_RESERVED_CANONICAL_ID, JS_SEED_ID,
    KL_SEED_ID, LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID, MDL_RESERVED_CANONICAL_ID,
    MUTUAL_INFORMATION_RESERVED_CANONICAL_ID, NORMALIZED_MI_RESERVED_PRIMITIVE_ID,
    RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID, SHANNON_ENTROPY_RESERVED_CANONICAL_ID,
    SPECTRAL_ENTROPY_SEED_ID, TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12P_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (KL_SEED_ID, "Kullback-Leibler divergence"),
    (JS_SEED_ID, "Jensen-Shannon divergence"),
    (SPECTRAL_ENTROPY_SEED_ID, "Spectral entropy"),
];

/// CanonicalAddition ids that carry entropy-style functionals
/// (Shannon, Conditional entropy, Cross-entropy, MDL). Each must
/// declare log base + empty-bin law + smoothing.
const T12P_ENTROPY_STYLE_CANONICAL_IDS: &[u32] = &[
    SHANNON_ENTROPY_RESERVED_CANONICAL_ID,
    CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID,
    CROSS_ENTROPY_RESERVED_CANONICAL_ID,
];

/// CanonicalAddition ids that are joint-distribution functionals
/// (Conditional entropy, Mutual information). Each must declare
/// a joint-distribution contract over (X, Y).
const T12P_JOINT_DISTRIBUTION_CANONICAL_IDS: &[u32] = &[
    CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID,
    MUTUAL_INFORMATION_RESERVED_CANONICAL_ID,
];

/// Forbidden causal-information-flow / Granger-style causal /
/// intervention-truth claim terms. CanonicalAddition and
/// ExistingCanonicalAuthorityResolution records legitimately
/// disclaim "causal information flow" / "intervention truth" /
/// "causation" inside "does NOT admit ..." sentences, so the
/// bare phrases are NOT forbidden; only positive-claim variants
/// are.
const T12P_FORBIDDEN_CAUSAL_FLOW: &[&str] = &[
    "granger-style causal verdict",
    "issues causal verdicts",
    "claims causal flow",
    "claims causal information flow",
    "admits causal information flow",
    "guarantees causal information flow",
    "proves causal information flow",
    "establishes causal flow",
    "claims intervention truth",
    "issues intervention-truth verdicts",
    "admits causation certainty",
    "claims causal influence certainty",
];

/// Forbidden privacy / security claim terms. CanonicalAddition
/// and ExistingCanonicalAuthorityResolution records legitimately
/// disclaim "cryptographic security" inside "does NOT admit ..."
/// sentences, so the bare phrase is NOT forbidden; only
/// positive-claim variants are.
const T12P_FORBIDDEN_PRIVACY_SECURITY: &[&str] = &[
    "side-channel-secure",
    "information-theoretically secure encryption",
    "issues privacy verdicts",
    "claims cryptographic security",
    "claims information-theoretic security",
    "claims privacy-leakage certainty",
    "admits cryptographic security",
    "guarantees privacy",
    "proves privacy",
    "issues cryptographic security verdicts",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn p_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_p_information_theory_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "T.12.p proposal failed verifier: {errors:?}"
    );
}

#[test]
fn p_proposal_has_open_status() {
    let p = seed_t12_p_information_theory_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn p_proposal_targets_information_theory() {
    let p = seed_t12_p_information_theory_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::InformationTheory
    ));
}

/// Load-bearing structural negative.
#[test]
fn t12_p_does_not_mutate_seed_len() {
    let _ = seed_t12_p_information_theory_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn p_proposal_proposes_eleven_primitives() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 11);
}

#[test]
fn p_proposal_proposes_zero_aliases() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn p_proposal_proposes_sixteen_dedup_records() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 16);
}

#[test]
fn p_proposal_proposes_nine_genealogy_edges() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 9);
}

#[test]
fn p_proposal_proposes_nine_source_refs() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn p_delta_has_five_new_canonical_records() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 5);
}

#[test]
fn p_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_p_information_theory_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn p_proposal_carries_two_rejection_records() {
    let p = seed_t12_p_information_theory_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.p must carry TWO RejectedNotDeterministic records \
        (tenth T.12.x with two, following T.12.g / h / i / j / k / l / m / n / o)"
    );
}

#[test]
fn p_proposal_court_delta_category_counts() {
    let p = seed_t12_p_information_theory_proposal();
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
    assert_eq!(canonical, 5);
    assert_eq!(existing, 3);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn p_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_p_information_theory_proposal();
    let b = seed_t12_p_information_theory_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_p_proposal_hash_matches_stored() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn p_proposal_hash_is_distinct_from_all_prior_t12x() {
    let p_p = seed_t12_p_information_theory_proposal();
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
    let stream = seed_t12_o_streaming_sketches_proposal();
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio, ind, chem, rf, econ,
        stream,
    ] {
        assert_ne!(
            p_p.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.p hash must differ from every prior T.12.x"
        );
    }
}

#[test]
fn t12_p_hash_changes_when_estimator_contract_changes() {
    let p_a = seed_t12_p_information_theory_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SHANNON_ENTROPY_RESERVED_CANONICAL_ID
        })
        .expect("Shannon entropy CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED Shannon entropy estimator / binning contract for hash-sensitivity test",
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
        "t12_p_collision_batch",
        SourceClass::InformationTheory,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "information-theory collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_p_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_p_collision_proposal",
        "Defective T.12.p proposal duplicating an existing SEED canonical.",
        SourceClass::InformationTheory,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_p_test",
    )
}

#[test]
fn t12_p_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12P_AUTHORITY_RESOLVED_SEED_IDS {
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
// declares estimator + binning OR partition law contract; every
// ExistingCanonicalAuthorityResolution declares estimator + bin /
// partition / FFT-window contract.
// ---------------------------------------------------------------

#[test]
fn t12_p_rejects_information_witness_without_estimator_or_binning_contract() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION
            && r.decision_wire_name != CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
        {
            continue;
        }
        let lower = r.reason.to_lowercase();
        // Estimator: any of plug-in / Miller-Madow / James-Stein
        // / kernel / harmonic-mean / declared estimator. MDL
        // (6605) uses a "code-length functional" instead of an
        // estimator — accepted as the estimator-equivalent.
        assert!(
            lower.contains("estimator") || lower.contains("code-length functional"),
            "Record decision={} canonical_id={} must declare estimator OR code-length \
             functional: {:?}",
            r.decision_wire_name,
            r.canonical_id.0,
            r.reason
        );
        // Input-contract: any of binning / partition law /
        // spectral-bin contract / FFT-window contract (for
        // histogram-based functionals); fixed model distribution
        // / model class (for cross-entropy / MDL); declared
        // reference distribution / declared distributions (for
        // KL / JS divergence which work on declared distributions
        // directly).
        assert!(
            lower.contains("binning")
                || lower.contains("partition")
                || lower.contains("spectral-bin contract")
                || lower.contains("fft window size")
                || lower.contains("fixed model")
                || lower.contains("model class")
                || lower.contains("declared reference distribution")
                || lower.contains("declared distributions")
                || lower.contains("symmetric mixture"),
            "Record decision={} canonical_id={} must declare binning OR partition OR \
             spectral-bin contract OR fixed model distribution OR declared model class OR \
             declared reference distribution OR symmetric mixture: {:?}",
            r.decision_wire_name,
            r.canonical_id.0,
            r.reason
        );
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #2: every entropy-style
// canonical declares log base + empty-bin law + smoothing rule.
// ---------------------------------------------------------------

#[test]
fn t12_p_rejects_entropy_detector_without_base_smoothing_and_empty_bin_law() {
    let p = seed_t12_p_information_theory_proposal();
    for canonical_id in T12P_ENTROPY_STYLE_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Entropy-style CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        assert!(
            lower.contains("log base"),
            "Entropy-style canonical_id={} must declare log base: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("empty-bin law"),
            "Entropy-style canonical_id={} must declare empty-bin law: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("smoothing"),
            "Entropy-style canonical_id={} must declare smoothing rule: {:?}",
            canonical_id,
            rec.reason
        );
    }
    // KL authority resolution also declares log base + empty-bin
    // law + epsilon-for-log(0) smoothing.
    let kl = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == KL_SEED_ID
        })
        .expect("KL authority-resolution record must exist");
    let lower = kl.reason.to_lowercase();
    assert!(lower.contains("log base"));
    assert!(lower.contains("empty-bin law"));
    assert!(lower.contains("epsilon for log(0)"));
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #3: every MI / conditional-
// entropy CanonicalAddition declares an explicit joint-
// distribution contract over (X, Y).
// ---------------------------------------------------------------

#[test]
fn t12_p_rejects_mutual_information_without_joint_distribution_contract() {
    let p = seed_t12_p_information_theory_proposal();
    for canonical_id in T12P_JOINT_DISTRIBUTION_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Joint-distribution CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        assert!(
            lower.contains("joint-distribution contract"),
            "Joint-distribution canonical_id={} must declare joint-distribution contract: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("(x, y)"),
            "Joint-distribution canonical_id={} must declare contract is over (X, Y): {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("joint binning")
                || lower.contains("joint binning or partition")
                || lower.contains("kernel-")
                    && lower.contains("density-")
                    && lower.contains("estimator")
                || lower.contains("joint cells")
                || lower.contains("partition over the product space"),
            "Joint-distribution canonical_id={} must declare joint-binning or kernel-\
             density-estimator law: {:?}",
            canonical_id,
            rec.reason
        );
    }
    // Transfer entropy proxy (6607) is a MI parameterization;
    // its reason must declare a lagged-joint contract.
    let te = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID
        })
        .expect("Transfer entropy parameterization record must exist");
    let lower = te.reason.to_lowercase();
    assert!(lower.contains("lagged-joint contract"));
    assert!(lower.contains("embedding orders k and l"));
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #4: causal-information-
// flow claim-language scanner. Forbidden terms must appear NOWHERE
// in the proposal across non-rejection records.
// ---------------------------------------------------------------

#[test]
fn t12_p_rejects_causal_information_flow_claim_language() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12P_FORBIDDEN_CAUSAL_FLOW {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden causal-information-flow term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #5: privacy / security
// claim-language scanner. Forbidden terms must appear NOWHERE in
// the proposal across non-rejection records.
// ---------------------------------------------------------------

#[test]
fn t12_p_rejects_privacy_or_security_claim_language() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12P_FORBIDDEN_PRIVACY_SECURITY {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden privacy / security claim term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #6: learned MI estimator
// AND black-box vendor IT score rejections require deterministic
// formula + training-data anchor / declared binning / declared
// log / smoothing / sample-correction law + tie-break + numeric
// mode.
// ---------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)]
fn t12_p_rejects_learned_embedding_information_score_without_formula() {
    let p = seed_t12_p_information_theory_proposal();
    // Learned MI estimator (6610).
    let learned = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned MI estimator rejection record must exist");
    let r = learned.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == LEARNED_MI_ESTIMATOR_RESERVED_PRIMITIVE_ID),
        "Learned MI estimator must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "Learned MI rejection must require 'deterministic feature-extraction law': {}",
        learned.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("declared training-") && r.contains("data anchor"));
    assert!(r.contains("declared binning"));
    assert!(r.contains("declared tie-break"));
    assert!(r.contains("declared numeric mode"));
    assert!(r.contains("no learned opaque embedding"));
    let mine_refs = ["mine", "belghazi", "infovae", "cpc", "variational mi"];
    let has_ref = mine_refs.iter().any(|v| r.contains(v));
    assert!(
        has_ref,
        "Learned MI rejection must name MINE / Belghazi / InfoVAE / CPC / variational MI bound"
    );
    // Black-box vendor IT score (6611).
    let vendor = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID
        })
        .expect("Black-box vendor IT score rejection record must exist");
    let r = vendor.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == BLACK_BOX_IT_SCORE_RESERVED_PRIMITIVE_ID),
        "Vendor IT score must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("declared formula")
            && (r.contains("binning") || r.contains("partition"))
            && r.contains("smoothing")
            && r.contains("log base")
            && r.contains("numeric mode"),
        "Vendor IT score rejection must demand declared formula + binning / partition + \
         smoothing + log base + numeric mode: {}",
        vendor.reason
    );
    let vendors = [
        "aws macie",
        "ibm guardium",
        "microsoft purview",
        "symantec",
        "broadcom",
        "cisco talos",
    ];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Vendor IT score rejection must name at least one vendor"
    );
}

// ---------------------------------------------------------------
// Per-canonical structural-distinctness assertions
// ---------------------------------------------------------------

#[test]
fn t12_p_mi_distinct_from_kl() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == MUTUAL_INFORMATION_RESERVED_CANONICAL_ID
        })
        .expect("MI CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("structurally distinct from seed 9 kl divergence"));
    assert!(r.contains("joint distribution vs the product of marginals"));
    assert!(r.contains("symmetric and non-directional"));
}

#[test]
fn t12_p_cross_entropy_pins_fixed_model_distribution() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CROSS_ENTROPY_RESERVED_CANONICAL_ID
        })
        .expect("Cross-entropy CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("fixed model"));
    assert!(r.contains("parameter-pinned"));
    assert!(r.contains("frozen across the comparison window"));
    assert!(r.contains("no learned parameters at decision time"));
}

#[test]
fn t12_p_mdl_declares_two_part_code() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == MDL_RESERVED_CANONICAL_ID
        })
        .expect("MDL CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("rissanen 1978") || r.contains("rissanen 1986"));
    assert!(r.contains("two-part code"));
    assert!(r.contains("l(d | m)"));
    assert!(r.contains("l(m)"));
    assert!(r.contains("model-cost is not silently dropped"));
}

#[test]
fn t12_p_shannon_entropy_declares_partition_law_options() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SHANNON_ENTROPY_RESERVED_CANONICAL_ID
        })
        .expect("Shannon entropy CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shannon 1948"));
    assert!(
        r.contains("equal-width")
            || r.contains("equal-frequency")
            || r.contains("freedman-diaconis")
    );
    assert!(r.contains("krichevsky-trofimov") || r.contains("laplace smoothing alpha"));
}

#[test]
fn t12_p_conditional_entropy_declares_joint_minus_marginal_form() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONDITIONAL_ENTROPY_RESERVED_CANONICAL_ID
        })
        .expect("Conditional entropy CanonicalAddition record must exist");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("cover-thomas 2006"));
    assert!(r.contains("h(y|x) = h(x,y) - h(x)"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_p_normalized_mi_is_parameterizationof_mi() {
    let p = seed_t12_p_information_theory_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == NORMALIZED_MI_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == NORMALIZED_MI_RESERVED_PRIMITIVE_ID
        })
        .expect("Normalized MI must have ParameterizationOf record");
    assert!(rec.reason.contains("Mutual information"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("normalisation function"));
}

#[test]
fn t12_p_transfer_entropy_is_parameterizationof_mi_non_causal() {
    let p = seed_t12_p_information_theory_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == TRANSFER_ENTROPY_RESERVED_PRIMITIVE_ID
        })
        .expect("Transfer entropy must have ParameterizationOf record");
    assert!(rec.reason.contains("Mutual information"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("schreiber 2000"));
    assert!(r.contains("admitted only as a deterministic non-causal witness"));
    assert!(r.contains("the court does not admit transfer entropy as evidence of"));
}

#[test]
fn t12_p_renyi_tsallis_is_parameterizationof_shannon() {
    let p = seed_t12_p_information_theory_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == RENYI_TSALLIS_ENTROPY_RESERVED_PRIMITIVE_ID
        })
        .expect("Rényi / Tsallis must have ParameterizationOf record");
    assert!(rec.reason.contains("Shannon entropy"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("rényi 1961"));
    assert!(r.contains("tsallis 1988"));
    assert!(r.contains("order-alpha parameter law"));
    assert!(r.contains("limit-recovery"));
}

#[test]
fn t12_p_compression_ratio_is_parameterizationof_mdl() {
    let p = seed_t12_p_information_theory_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == COMPRESSION_RATIO_RESERVED_PRIMITIVE_ID
        })
        .expect("Compression-ratio must have ParameterizationOf record");
    assert!(rec.reason.contains("Minimum description length"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("ziv-lempel 1977") || r.contains("welch 1984"));
    assert!(r.contains("does not admit compression as a surrogate"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_p_kl_domain_transfer_to_information_theory_exists() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF && r.canonical_id.0 == KL_SEED_ID
        })
        .expect("KL must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared information-") && r.contains("theoretic divergence ancestor"));
    assert!(r.contains("informationtheory"));
}

#[test]
fn t12_p_spectral_entropy_domain_transfer_to_information_theory_exists() {
    let p = seed_t12_p_information_theory_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == SPECTRAL_ENTROPY_SEED_ID
        })
        .expect("Spectral entropy must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(
        r.contains("shared shannon-")
            && r.contains("entropy-")
            && r.contains("on-distribution ancestor")
    );
    assert!(r.contains("informationtheory"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_p_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_p_information_theory_proposal();
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
fn t12_p_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_p_information_theory_proposal();
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
fn t12_p_authority_resolution_covers_all_t12p_seed_ids() {
    let p = seed_t12_p_information_theory_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12P_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.p"
        );
    }
}

#[test]
fn t12_p_every_dedup_record_has_reason() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_p_canonical_addition_ids_are_in_6601_to_6605_range() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6601..=6605).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.p reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_p_parameterization_ids_are_in_6606_to_6609_range() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6606..=6609).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.p reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_p_rejection_ids_are_in_6610_to_6611_range() {
    let p = seed_t12_p_information_theory_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6610..=6611).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.p reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn p_proposal_text_rendering_byte_stable() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn p_proposal_json_rendering_byte_stable() {
    let p = seed_t12_p_information_theory_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}

#[test]
fn p_proposal_motivation_carries_panel_locked_non_claim() {
    let p = seed_t12_p_information_theory_proposal();
    let m = p.motivation;
    assert!(
        m.contains("does not admit semantic meaning, causal information flow certainty, privacy leakage certainty, cryptographic security claims, or learned representation claims"),
        "T.12.p motivation must carry the panel-locked non-claim verbatim"
    );
}
