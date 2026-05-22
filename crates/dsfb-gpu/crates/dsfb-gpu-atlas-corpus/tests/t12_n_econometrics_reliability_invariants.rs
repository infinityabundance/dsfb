//! T.12.n acceptance suite — Econometrics + Reliability /
//! Survival expansion proposal invariants.
//!
//! Six panel-required load-bearing negatives pin the
//! market-prediction / investment-or-credit-decision / RUL-or-
//! failure-time-certainty / survival-without-censoring-or-time-
//! origin / econometric-without-stationarity-or-window /
//! black-box-forecaster-without-formula contract discipline
//! T.12.n exists to prove. The campaign's identity is captured
//! in:
//!
//! * `t12_n_rejects_market_prediction_claim_language`
//!   (MOST IMPORTANT — forbidden-term scanner; market /
//!   investment / trading-signal terms appear ONLY in
//!   `RejectedNotDeterministic`)
//! * `t12_n_rejects_investment_or_credit_decision_claim_language`
//!   (forbidden-term scanner; credit-decision / actuarial-
//!   pricing / fiduciary-recommendation terms)
//! * `t12_n_rejects_rul_or_failure_time_certainty_claim_language`
//!   (forbidden-term scanner; RUL / failure-time / warranty
//!   / maintenance-recommendation terms)
//! * `t12_n_rejects_survival_witness_without_censoring_or_time_origin_contract`
//!   (every survival CanonicalAddition must declare censoring
//!   law + time-origin law)
//! * `t12_n_rejects_econometric_witness_without_stationarity_or_window_contract`
//!   (every econometric CanonicalAddition must declare
//!   stationarity contract + window contract)
//! * `t12_n_rejects_black_box_forecaster_without_formula`
//!   (6413 + 6414 rejections must demand deterministic
//!   formula + training-data anchor + tie-break + numeric
//!   mode + no market / no RUL claim)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.n admits deterministic econometric, reliability,
//! > survival, and degradation witnesses. It does not admit
//! > market prediction, investment advice, credit-decision
//! > authority, actuarial pricing authority, causal economic
//! > certainty, RUL certainty, maintenance recommendations,
//! > or failure-time prediction.

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
use dsfb_gpu_atlas_corpus::t12_n_econometrics_reliability::{
    seed_t12_n_econometrics_reliability_proposal, BAI_PERRON_RESERVED_CANONICAL_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, COINTEGRATION_BREAK_RESERVED_CANONICAL_ID,
    COX_SCHOENFELD_RESERVED_CANONICAL_ID, CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID,
    CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID, CUSUM_SEED_ID,
    GARCH_RESIDUAL_RESERVED_CANONICAL_ID, HAUSMAN_RESERVED_CANONICAL_ID,
    HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID, KM_SURVIVAL_RESERVED_CANONICAL_ID,
    LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID, LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID,
    MANN_KENDALL_SEED_ID, PAGE_HINKLEY_SEED_ID, PARIS_ERDOGAN_RESERVED_CANONICAL_ID,
    QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID, RESIDUAL_ENVELOPE_EXIT_SEED_ID,
    WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12N_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (CUSUM_SEED_ID, "CUSUM (cumulative sum) chart"),
    (PAGE_HINKLEY_SEED_ID, "Page-Hinkley test"),
    (MANN_KENDALL_SEED_ID, "Mann-Kendall trend test"),
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
];

/// Econometric CanonicalAddition records whose reasons must
/// declare a stationarity contract + window contract.
const T12N_ECONOMETRIC_CANONICAL_IDS: &[u32] = &[
    GARCH_RESIDUAL_RESERVED_CANONICAL_ID,
    COINTEGRATION_BREAK_RESERVED_CANONICAL_ID,
    HAUSMAN_RESERVED_CANONICAL_ID,
    BAI_PERRON_RESERVED_CANONICAL_ID,
];

/// Survival CanonicalAddition records whose reasons must
/// declare a censoring law + time-origin law.
const T12N_SURVIVAL_CANONICAL_IDS: &[u32] = &[
    KM_SURVIVAL_RESERVED_CANONICAL_ID,
    COX_SCHOENFELD_RESERVED_CANONICAL_ID,
    WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID,
];

/// Forbidden market-prediction claim terms. Disclaimer phrasing
/// like "not market prediction" or "not investment advice" is
/// legitimately allowed and does NOT contain any of these
/// CERTAINTY-claim variants.
const T12N_FORBIDDEN_MARKET_TERMS: &[&str] = &[
    "stock price prediction",
    "market return forecast",
    "trading signal",
    "buy signal",
    "sell signal",
    "alpha generation",
    "predicts the market",
];

/// Forbidden investment / credit-decision / actuarial-pricing
/// claim terms.
const T12N_FORBIDDEN_INVESTMENT_OR_CREDIT_TERMS: &[&str] = &[
    "credit approval verdict",
    "credit denial verdict",
    "loan approval verdict",
    "investment advice verdict",
    "fiduciary recommendation verdict",
    "investment recommendation verdict",
    "issues credit decisions",
    "issues investment advice",
];

/// Forbidden RUL / failure-time / maintenance-recommendation
/// claim terms.
const T12N_FORBIDDEN_RUL_OR_FAILURE_TIME_TERMS: &[&str] = &[
    "rul prediction",
    "remaining useful life certainty",
    "failure time prediction",
    "predicted failure date",
    "guaranteed lifetime",
    "warranty extension recommendation",
    "issues maintenance recommendations",
    "issues rul predictions",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn n_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "T.12.n proposal failed verifier: {errors:?}"
    );
}

#[test]
fn n_proposal_has_open_status() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn n_proposal_targets_econometrics() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert!(matches!(p.target_source_class, SourceClass::Econometrics));
}

/// Load-bearing negative #1 (panel-required).
#[test]
fn t12_n_does_not_mutate_seed_len() {
    let _ = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn n_proposal_proposes_fourteen_primitives() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 14);
}

#[test]
fn n_proposal_proposes_zero_aliases() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn n_proposal_proposes_twenty_dedup_records() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 20);
}

#[test]
fn n_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn n_proposal_proposes_eleven_source_refs() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 11);
}

#[test]
fn n_delta_has_eight_new_canonical_records() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn n_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn n_proposal_carries_two_rejection_records() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.n must carry TWO RejectedNotDeterministic records \
        (eighth T.12.x with two, following T.12.g / h / i / j / k / l / m)"
    );
}

#[test]
fn n_proposal_court_delta_category_counts() {
    let p = seed_t12_n_econometrics_reliability_proposal();
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
fn n_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_n_econometrics_reliability_proposal();
    let b = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_n_proposal_hash_matches_stored() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn n_proposal_hash_is_distinct_from_all_prior_t12x() {
    let n = seed_t12_n_econometrics_reliability_proposal();
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
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio, ind, chem, rf,
    ] {
        assert_ne!(
            n.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.n hash must differ from every prior T.12.x"
        );
    }
}

#[test]
fn t12_n_hash_changes_when_econometric_or_survival_law_changes() {
    let p_a = seed_t12_n_econometrics_reliability_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == GARCH_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("GARCH CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED GARCH conditional-variance / decision law for \
            hash-sensitivity test",
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
        "t12_n_collision_batch",
        SourceClass::Econometrics,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "econometrics collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_n_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_n_collision_proposal",
        "Defective T.12.n proposal duplicating an existing SEED canonical.",
        SourceClass::Econometrics,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_n_test",
    )
}

#[test]
fn t12_n_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12N_AUTHORITY_RESOLVED_SEED_IDS {
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
// MOST IMPORTANT load-bearing negative: market-prediction
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_market_prediction_claim_language() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12N_FORBIDDEN_MARKET_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden market-prediction term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative: investment / credit-
// decision claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_investment_or_credit_decision_claim_language() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12N_FORBIDDEN_INVESTMENT_OR_CREDIT_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden investment / credit-decision / actuarial-pricing term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative: RUL / failure-time
// certainty claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_rul_or_failure_time_certainty_claim_language() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12N_FORBIDDEN_RUL_OR_FAILURE_TIME_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden RUL / failure-time / maintenance-recommendation term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative: every survival /
// reliability CanonicalAddition must declare censoring law
// (where applicable) + time-origin law (where applicable).
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_survival_witness_without_censoring_or_time_origin_contract() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for canonical_id in T12N_SURVIVAL_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Survival / reliability CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        assert!(
            lower.contains("censoring law")
                || lower.contains("right-censoring")
                || lower.contains("right censoring"),
            "Survival / reliability canonical_id={} must declare censoring law: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("time-origin law") || lower.contains("time origin"),
            "Survival / reliability canonical_id={} must declare time-origin law: {:?}",
            canonical_id,
            rec.reason
        );
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative: every econometric
// CanonicalAddition must declare stationarity contract + window
// contract.
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_econometric_witness_without_stationarity_or_window_contract() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for canonical_id in T12N_ECONOMETRIC_CANONICAL_IDS {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == *canonical_id
            })
            .expect("Econometric CanonicalAddition record must exist");
        let lower = rec.reason.to_lowercase();
        assert!(
            lower.contains("stationarity contract"),
            "Econometric canonical_id={} must declare stationarity contract: {:?}",
            canonical_id,
            rec.reason
        );
        assert!(
            lower.contains("window contract"),
            "Econometric canonical_id={} must declare window contract: {:?}",
            canonical_id,
            rec.reason
        );
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative: black-box forecasters
// (market + RUL) require deterministic formula + training-data
// anchor + tie-break + numeric mode + no market / no RUL claim.
// ---------------------------------------------------------------

#[test]
fn t12_n_rejects_black_box_forecaster_without_formula() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    // Market forecaster rejection record (6413).
    let market = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned market predictor rejection record must exist");
    let r = market.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == LEARNED_MARKET_PREDICTOR_RESERVED_PRIMITIVE_ID),
        "Learned market predictor must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "Market predictor rejection must require 'deterministic feature-extraction law': {}",
        market.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("declared training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    let vendors = ["bloomberg", "alphasense", "kavout", "goldman", "jp morgan"];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Market predictor rejection must name at least one vendor"
    );
    assert!(r.contains("no market-") && r.contains("prediction claim"));
    assert!(r.contains("no investment-") && r.contains("recommendation claim"));
    assert!(r.contains("no credit-") && r.contains("decision claim"));
    assert!(r.contains("no actuarial-") && r.contains("pricing claim"));

    // RUL classifier rejection record (6414).
    let rul = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned RUL classifier rejection record must exist");
    let r = rul.reason.to_lowercase();
    assert!(
        !p.dedup_court_delta
            .new_canonical_records
            .iter()
            .any(|c| c.0 == LEARNED_RUL_CLASSIFIER_RESERVED_PRIMITIVE_ID),
        "Learned RUL classifier must NOT be in new_canonical_records"
    );
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "RUL classifier rejection must require 'deterministic feature-extraction law': {}",
        rul.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("declared training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    let vendors = [
        "uptake ai",
        "c3.ai",
        "senseye",
        "ibm maximo",
        "siemens mindsphere",
    ];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "RUL classifier rejection must name at least one vendor"
    );
    assert!(r.contains("no rul-") && r.contains("certainty claim"));
    assert!(r.contains("no failure-") && r.contains("time-") && r.contains("prediction claim"));
    assert!(r.contains("no maintenance-") && r.contains("recommendation claim"));
}

// ---------------------------------------------------------------
// Per-canonical structural-distinctness assertions
// ---------------------------------------------------------------

#[test]
fn t12_n_garch_is_distinct_from_any_seed_level_model() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == GARCH_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("GARCH must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("bollerslev 1986"));
    assert!(r.contains("conditional variance model"));
    assert!(r.contains("not a level model"));
}

#[test]
fn t12_n_cointegration_distinct_from_raw_cusum() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == COINTEGRATION_BREAK_RESERVED_CANONICAL_ID
        })
        .expect("Cointegration must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("hansen 1992"));
    assert!(r.contains("cointegration regression"));
    assert!(r.contains("squared residuals"));
    assert!(r.contains("not a raw signal"));
}

#[test]
fn t12_n_hausman_distinct_from_any_residual_sequence() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == HAUSMAN_RESERVED_CANONICAL_ID
        })
        .expect("Hausman must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("hausman 1978"));
    assert!(r.contains("parameter-") && r.contains("difference"));
    assert!(r.contains("chi-squared"));
    assert!(r.contains("not on a residual sequence"));
}

#[test]
fn t12_n_bai_perron_admits_multiple_breaks_distinct_from_page_hinkley() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BAI_PERRON_RESERVED_CANONICAL_ID
        })
        .expect("Bai-Perron must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("bai-perron 1998"));
    assert!(r.contains("multiple") && r.contains("breaks"));
    assert!(r.contains("information") && r.contains("criterion"));
    assert!(
        r.contains("distinct from seed 4 page-hinkley")
            || r.contains("structurally distinct from seed 4")
    );
}

#[test]
fn t12_n_km_survival_declares_kaplan_meier() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == KM_SURVIVAL_RESERVED_CANONICAL_ID
        })
        .expect("KM must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("kaplan") && r.contains("meier 1958"));
    assert!(r.contains("product-limit"));
}

#[test]
fn t12_n_cox_schoenfeld_declares_proportional_hazards() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == COX_SCHOENFELD_RESERVED_CANONICAL_ID
        })
        .expect("Cox/Schoenfeld must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("cox 1972"));
    assert!(r.contains("schoenfeld 1982"));
    assert!(r.contains("proportional-hazards"));
    assert!(r.contains("partial-likelihood"));
}

#[test]
fn t12_n_weibull_declares_shape_and_scale() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == WEIBULL_FAILURE_RATE_RESERVED_CANONICAL_ID
        })
        .expect("Weibull must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("weibull 1951"));
    assert!(r.contains("shape") && r.contains("scale"));
    assert!(r.contains("mle estimation"));
}

#[test]
fn t12_n_paris_erdogan_declares_stress_intensity() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == PARIS_ERDOGAN_RESERVED_CANONICAL_ID
        })
        .expect("Paris-Erdogan must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("paris") && r.contains("erdogan 1963"));
    assert!(r.contains("stress-intensity-range"));
    assert!(r.contains("c and m"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_n_cusum_recursive_residuals_is_parameterizationof_cusum() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == CUSUM_RECURSIVE_RESIDUALS_RESERVED_PRIMITIVE_ID
        })
        .expect("CUSUM-of-recursive-residuals must have ParameterizationOf record");
    assert!(rec.reason.contains("CUSUM"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("brown") && r.contains("durbin") && r.contains("evans 1975"));
}

#[test]
fn t12_n_quandt_andrews_is_parameterizationof_page_hinkley() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == QUANDT_ANDREWS_RESERVED_PRIMITIVE_ID
        })
        .expect("Quandt-Andrews must have ParameterizationOf record");
    assert!(rec.reason.contains("Page-Hinkley"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("f-statistic"));
    assert!(r.contains("quandt 1960") || r.contains("chow 1960") || r.contains("andrews 1993"));
}

#[test]
fn t12_n_hazard_rate_change_is_parameterizationof_residual_envelope() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == HAZARD_RATE_CHANGE_RESERVED_PRIMITIVE_ID
        })
        .expect("Hazard-rate change must have ParameterizationOf record");
    assert!(rec.reason.contains("Residual envelope exit"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("piecewise-constant hazard"));
}

#[test]
fn t12_n_cumulative_damage_is_parameterizationof_cusum() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == CUMULATIVE_DAMAGE_RESERVED_PRIMITIVE_ID
        })
        .expect("Cumulative damage must have ParameterizationOf record");
    assert!(rec.reason.contains("CUSUM"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("palmgren 1924") || r.contains("miner 1945"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_n_cusum_domain_transfer_to_econometrics_and_reliability_exists() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF && r.canonical_id.0 == CUSUM_SEED_ID
        })
        .expect("CUSUM must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared structural-") && r.contains("change ancestor"));
    assert!(r.contains("econometrics") && r.contains("reliabilitysurvival"));
}

#[test]
fn t12_n_residual_envelope_domain_transfer_to_reliability_exists() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == RESIDUAL_ENVELOPE_EXIT_SEED_ID
        })
        .expect("Residual envelope exit must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared") && r.contains("envelope-") && r.contains("boundary ancestor"));
    assert!(r.contains("reliabilitysurvival"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_n_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_n_econometrics_reliability_proposal();
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
fn t12_n_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_n_econometrics_reliability_proposal();
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
fn t12_n_authority_resolution_covers_all_t12n_seed_ids() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12N_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.n"
        );
    }
}

#[test]
fn t12_n_every_dedup_record_has_reason() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_n_canonical_addition_ids_are_in_6401_to_6408_range() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6401..=6408).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.n reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_n_parameterization_ids_are_in_6409_to_6412_range() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6409..=6412).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.n reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_n_rejection_ids_are_in_6413_to_6414_range() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6413..=6414).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.n reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn n_proposal_text_rendering_byte_stable() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn n_proposal_json_rendering_byte_stable() {
    let p = seed_t12_n_econometrics_reliability_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
