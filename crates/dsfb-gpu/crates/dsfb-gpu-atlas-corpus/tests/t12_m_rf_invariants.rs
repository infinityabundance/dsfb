//! T.12.m acceptance suite — RF / Communications expansion
//! proposal invariants.
//!
//! Eight panel-required load-bearing negatives pin the
//! signal-representation / sampling / carrier-or-channel /
//! synchronization / window-or-transform / decision-functional
//! contract discipline T.12.m exists to prove. The campaign's
//! identity is captured in:
//!
//! * `t12_m_rejects_rf_detector_without_signal_or_sampling_contract`
//!   (MOST IMPORTANT — every CanonicalAddition must declare
//!   signal representation + sampling law + carrier or channel
//!   assumption + window-or-transform law AND decision
//!   functional)
//! * `t12_m_rejects_emitter_identification_claim_language`
//!   (MOST IMPORTANT — forbidden-term scanner; emitter
//!   attribution / transmitter identification / transmitter
//!   fingerprint terms appear ONLY in
//!   `RejectedNotDeterministic`)
//! * `t12_m_rejects_geolocation_or_attribution_claim_language`
//!   (forbidden-term scanner)
//! * `t12_m_rejects_spectrum_enforcement_claim_language`
//!   (forbidden-term scanner)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.m admits deterministic RF / communications signal
//! > witnesses, not emitter attribution, transmitter
//! > identification, geolocation, spectrum-enforcement
//! > authority, military classification, or communications-
//! > intelligence conclusions.

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
use dsfb_gpu_atlas_corpus::t12_m_rf::{
    seed_t12_m_rf_proposal, AUTOCORRELATION_BREAK_SEED_ID,
    BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID,
    BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_DOMAIN_TRANSFER_OF, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_PARAMETERIZATION_OF, CATEGORY_REJECTED_NOT_DETERMINISTIC, CFO_RESIDUAL_SEED_ID,
    CIR_DRIFT_RESERVED_CANONICAL_ID, CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID,
    CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID, ERROR_BURST_SEED_ID, EVM_ANOMALY_SEED_ID,
    FFT_BAND_ENERGY_SEED_ID, FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID,
    IQ_IMBALANCE_RESERVED_CANONICAL_ID, LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID,
    PHASE_NOISE_RESERVED_CANONICAL_ID, RESIDUAL_ENVELOPE_EXIT_SEED_ID,
    SNR_DROP_RESERVED_PRIMITIVE_ID, SPECTRAL_ENTROPY_SEED_ID,
    SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID, SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12M_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (FFT_BAND_ENERGY_SEED_ID, "FFT band-energy anomaly"),
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
    (SPECTRAL_ENTROPY_SEED_ID, "Spectral entropy"),
    (AUTOCORRELATION_BREAK_SEED_ID, "Autocorrelation break"),
    (CFO_RESIDUAL_SEED_ID, "Carrier-frequency-offset residual"),
    (EVM_ANOMALY_SEED_ID, "Error Vector Magnitude (EVM) anomaly"),
];

/// Forbidden emitter-identification / transmitter-identification
/// terms that must NOT appear in CanonicalAddition or
/// ExistingCanonicalAuthorityResolution reason text. The
/// non-claim phrase "RF signal witness, not emitter
/// attribution or spectrum enforcement" disclaims the words
/// "emitter attribution" and "spectrum enforcement"
/// legitimately — so only POSITIVE-claim variants are forbidden
/// here. The rejection records (6313 / 6314) intentionally use
/// "emitter attribution" / "transmitter identification" /
/// "transmitter fingerprint" inside disclaiming "does NOT
/// issue" sentences.
const T12M_FORBIDDEN_EMITTER_ID_TERMS: &[&str] = &[
    "emitter identified as",
    "transmitter identified as",
    "transmitter identity verified",
    "device identity verified",
    "transmitter fingerprint matched",
];

/// Forbidden geolocation / attribution claim terms. The
/// non-claim disclaims "geolocation" legitimately, so only
/// the certainty / verdict variants are forbidden.
const T12M_FORBIDDEN_GEOLOCATION_TERMS: &[&str] = &[
    "geolocation certainty",
    "geolocates the",
    "transmitter location verified",
    "transmitter located at",
    "geolocated to",
];

/// Forbidden spectrum-enforcement claim terms. "spectrum
/// enforcement" itself appears legitimately as a disclaimer
/// in the non-claim phrase, so only specific verdict
/// variants are forbidden.
const T12M_FORBIDDEN_SPECTRUM_ENFORCEMENT_TERMS: &[&str] = &[
    "regulatory enforcement verdict",
    "illegal transmission verdict",
    "unauthorized transmission verdict",
    "military classification verdict",
    "signals intelligence conclusion",
    "comint conclusion",
    "sigint verdict",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn rf_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_m_rf_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "RF / communications proposal failed verifier: {errors:?}"
    );
}

#[test]
fn rf_proposal_has_open_status() {
    let p = seed_t12_m_rf_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn rf_proposal_targets_rf_communications() {
    let p = seed_t12_m_rf_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::RfCommunications
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_m_does_not_mutate_seed_len`.
#[test]
fn t12_m_does_not_mutate_seed_len() {
    let _ = seed_t12_m_rf_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn rf_proposal_proposes_twelve_primitives() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 12);
}

#[test]
fn rf_proposal_proposes_zero_aliases() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn rf_proposal_proposes_twenty_dedup_records() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 20);
}

#[test]
fn rf_proposal_proposes_ten_genealogy_edges() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 10);
}

#[test]
fn rf_proposal_proposes_nine_source_refs() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn rf_delta_has_six_new_canonical_records() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 6);
}

#[test]
fn rf_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_m_rf_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn rf_proposal_carries_two_rejection_records() {
    let p = seed_t12_m_rf_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.m must carry TWO RejectedNotDeterministic records \
        (seventh T.12.x with two, following T.12.g / h / i / j / k / l)"
    );
}

#[test]
fn rf_proposal_court_delta_category_counts() {
    let p = seed_t12_m_rf_proposal();
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
    assert_eq!(canonical, 6);
    assert_eq!(existing, 6);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn rf_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_m_rf_proposal();
    let b = seed_t12_m_rf_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_rf_proposal_hash_matches_stored() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn rf_proposal_hash_is_distinct_from_all_prior_t12x() {
    let m = seed_t12_m_rf_proposal();
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
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio, ind, chem,
    ] {
        assert_ne!(
            m.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.m hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative: hash sensitivity.
#[test]
fn t12_m_hash_changes_when_signal_or_decision_law_changes() {
    let p_a = seed_t12_m_rf_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID
        })
        .expect("Constellation spread CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED constellation-spread signal / decision law for hash-sensitivity test",
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
        "t12_m_collision_batch",
        SourceClass::RfCommunications,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "rf collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_m_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_m_collision_proposal",
        "Defective T.12.m proposal duplicating an existing SEED canonical.",
        SourceClass::RfCommunications,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_m_test",
    )
}

#[test]
fn t12_m_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12M_AUTHORITY_RESOLVED_SEED_IDS {
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
// must declare signal representation + sampling law + carrier or
// channel assumption + window-or-transform law AND decision
// functional.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_rf_detector_without_signal_or_sampling_contract() {
    let p = seed_t12_m_rf_proposal();
    let signal_terms = [
        "signal representation",
        "baseband i/q",
        "post-equalisation i/q",
        "bandpass",
        "instantaneous-phase",
    ];
    let sampling_terms = ["sampling law"];
    let carrier_or_channel_terms = [
        "carrier assumption",
        "channel assumption",
        "declared cycle frequencies",
        "declared oscillator model",
    ];
    let transform_terms = ["window / transform law", "transform law", "estimator"];
    let decision_terms = ["decision functional", "decision law", "decision predicate"];
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION {
            continue;
        }
        let lower = r.reason.to_lowercase();
        assert!(
            signal_terms.iter().any(|t| lower.contains(t)),
            "CanonicalAddition canonical_id={} must declare signal representation: {:?}",
            r.canonical_id.0,
            r.reason
        );
        assert!(
            sampling_terms.iter().any(|t| lower.contains(t)),
            "CanonicalAddition canonical_id={} must declare sampling law: {:?}",
            r.canonical_id.0,
            r.reason
        );
        assert!(
            carrier_or_channel_terms.iter().any(|t| lower.contains(t)),
            "CanonicalAddition canonical_id={} must declare carrier or channel assumption: {:?}",
            r.canonical_id.0,
            r.reason
        );
        assert!(
            transform_terms.iter().any(|t| lower.contains(t)),
            "CanonicalAddition canonical_id={} must declare window / transform law: {:?}",
            r.canonical_id.0,
            r.reason
        );
        assert!(
            decision_terms.iter().any(|t| lower.contains(t)),
            "CanonicalAddition canonical_id={} must declare decision functional / law / predicate: {:?}",
            r.canonical_id.0,
            r.reason
        );
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #2: emitter-identification
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_emitter_identification_claim_language() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12M_FORBIDDEN_EMITTER_ID_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden emitter / transmitter identification term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
            || r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
        {
            assert!(
                lower.contains("rf signal witness")
                    || lower.contains("rf modulation-quality signal witness")
                    || lower.contains("rf channel signal witness")
                    || lower.contains("rf synchronization signal witness")
                    || lower.contains("rf spectral signal witness")
                    || lower.contains("rf envelope signal witness")
                    || lower.contains("rf correlation signal witness"),
                "Record decision={} canonical_id={} must end with panel-locked \
                non-claim 'RF signal witness, not emitter attribution or \
                spectrum enforcement': {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                r.reason,
            );
            assert!(
                lower.contains("not emitter attribution")
                    || lower.contains("not emitter attribution or geolocation")
                    || lower.contains("not emitter attribution or transmitter fingerprint")
                    || lower.contains("not emitter attribution or spectrum enforcement"),
                "Record decision={} canonical_id={} must contain 'not emitter \
                attribution ...' disclaimer: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #3: geolocation / attribution
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_geolocation_or_attribution_claim_language() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12M_FORBIDDEN_GEOLOCATION_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden geolocation / attribution term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #4: spectrum-enforcement
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_spectrum_enforcement_claim_language() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12M_FORBIDDEN_SPECTRUM_ENFORCEMENT_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden spectrum-enforcement / SIGINT term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: channel detector must
// declare channel-model contract.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_channel_detector_without_channel_model_contract() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CIR_DRIFT_RESERVED_CANONICAL_ID
        })
        .expect("CIR drift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("channel assumption"));
    assert!(r.contains("multipath") && r.contains("finite-impulse-response model"));
    assert!(r.contains("tap-delay grid"));
    assert!(r.contains("per-tap magnitude"));
    assert!(r.contains("channel-sounding waveform"));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: modulation classifier must
// declare decision functional.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_modulation_classifier_without_declared_decision_functional() {
    let p = seed_t12_m_rf_proposal();
    // Constellation spread (CanonicalAddition at 6303) must
    // declare the symbol constellation and the per-cluster
    // decision functional.
    let spread = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID
        })
        .expect("Constellation spread CanonicalAddition record must exist");
    let r = spread.reason.to_lowercase();
    assert!(
        r.contains("declared symbol constellation"),
        "Constellation spread must declare symbol constellation: {:?}",
        spread.reason
    );
    assert!(
        r.contains("decision functional"),
        "Constellation spread must explicitly declare a decision functional: {:?}",
        spread.reason
    );
    // EVM (SEED 54) is now an ExistingCanonicalAuthorityResolution
    // record. It must declare the symbol constellation AND enumerate
    // at least BPSK / QPSK / QAM constellations to nail down the
    // per-symbol error vector contract that the SEED record
    // canonicalises.
    let evm = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == EVM_ANOMALY_SEED_ID
        })
        .expect("EVM SEED 54 ExistingCanonicalAuthorityResolution record must exist");
    let r = evm.reason.to_lowercase();
    assert!(
        r.contains("declared symbol constellation"),
        "EVM SEED 54 authority record must declare symbol constellation: {:?}",
        evm.reason
    );
    assert!(
        r.contains("decision functional"),
        "EVM SEED 54 authority record must declare decision functional: {:?}",
        evm.reason
    );
    assert!(
        r.contains("bpsk") && r.contains("qpsk") && r.contains("qam"),
        "EVM SEED 54 authority record must enumerate at least BPSK / QPSK / QAM \
         constellations: {:?}",
        evm.reason
    );
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: cyclostationary witness
// must declare cycle-frequency law.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_cyclostationary_witness_without_cycle_frequency_law() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CYCLOSTATIONARY_FEATURE_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("Cyclostationary feature shift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("gardner 1987"));
    assert!(r.contains("declared cycle frequencies"));
    assert!(r.contains("symbol rate"));
    assert!(r.contains("spectral correlation function"));
    assert!(
        r.contains("declared, not implicit")
            || (r.contains("cycle-frequency law is declared") && r.contains("not implicit")),
        "Cyclostationary witness must distinguish DECLARED cycle-frequency law \
         from implicit autocorrelation: {:?}",
        rec.reason
    );
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: black-box modulation
// classifier / proprietary spectrum anomaly score requires
// deterministic feature-extraction law.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_black_box_rf_classifier_without_formula() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Black-box modulation classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == BLACK_BOX_MODULATION_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Black-box modulation classifier must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "Black-box modulation classifier rejection must require 'deterministic \
         feature-extraction law': {}",
        rec.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    let vendors = [
        "keysight",
        "rohde & schwarz",
        "ni rfic analyser ml",
        "ettus usrp",
    ];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Black-box modulation classifier rejection must name at least one vendor"
    );
    assert!(r.contains("does not issue spectrum-") && r.contains("enforcement authority"));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: learned RF fingerprinting
// classifier must be rejected; no transmitter identity claim
// allowed.
// ---------------------------------------------------------------

#[test]
fn t12_m_rejects_learned_rf_fingerprint_classifier_without_deterministic_formula() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Learned RF fingerprinting classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_RF_FINGERPRINT_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned RF fingerprinting must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("restuccia") || r.contains("sankhe") || r.contains("wang"));
    assert!(
        r.contains("deterministic feature-") && r.contains("extraction law"),
        "Learned RF fingerprinting rejection must require 'deterministic \
         feature-extraction law': {}",
        rec.reason
    );
    assert!(r.contains("declared formula"));
    assert!(r.contains("declared tie-break law"));
    assert!(r.contains("declared numeric mode"));
    assert!(r.contains("no learned opaque embedding"));
    assert!(r.contains("no transmitter identity claim"));
}

// ---------------------------------------------------------------
// Per-canonical structural-distinctness assertions
// ---------------------------------------------------------------

#[test]
fn t12_m_cfo_authority_resolves_seed_53_with_morelli_reference() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == CFO_RESIDUAL_SEED_ID
        })
        .expect("CFO residual must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("morelli") && r.contains("mengali"));
    assert!(r.contains("ofdm cfo estimator") || r.contains("mle frequency estimator"));
    assert!(r.contains("seed id 53"));
    assert!(r.contains("no duplicate admitted"));
    assert!(r.contains("reserved id 6301 stays unused"));
}

#[test]
fn t12_m_evm_authority_resolves_seed_54_with_shafik_reference() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == EVM_ANOMALY_SEED_ID
        })
        .expect("EVM anomaly must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shafik") && r.contains("rahman") && r.contains("islam"));
    assert!(r.contains("seed id 54"));
    assert!(r.contains("no duplicate admitted"));
    assert!(r.contains("reserved id 6302 stays unused"));
}

#[test]
fn t12_m_constellation_spread_is_distinct_from_evm() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONSTELLATION_SPREAD_RESERVED_CANONICAL_ID
        })
        .expect("Constellation spread must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("second-") && r.contains("moment distribution claim"));
    assert!(r.contains("not a first-") && r.contains("moment per-"));
    assert!(r.contains("seed 54 evm"));
}

#[test]
fn t12_m_cir_is_distinct_from_autocorrelation_break() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CIR_DRIFT_RESERVED_CANONICAL_ID
        })
        .expect("CIR drift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("system response to a declared impulse"));
    assert!(r.contains("not the autocorrelation"));
}

#[test]
fn t12_m_iq_imbalance_declares_gain_and_phase_balance() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == IQ_IMBALANCE_RESERVED_CANONICAL_ID
        })
        .expect("IQ imbalance must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("gain-") && r.contains("balance estimator"));
    assert!(r.contains("phase-") && r.contains("balance estimator"));
    assert!(r.contains("departs from unity") && r.contains("departs from 90"));
}

#[test]
fn t12_m_phase_noise_declares_oscillator_model() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == PHASE_NOISE_RESERVED_CANONICAL_ID
        })
        .expect("Phase-noise must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("razavi 1996"));
    assert!(r.contains("declared oscillator model"));
    assert!(r.contains("phase-locked-") && r.contains("loop"));
    assert!(r.contains("dbc/hz"));
}

#[test]
fn t12_m_symbol_timing_is_distinct_from_cfo() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SYMBOL_TIMING_OFFSET_RESERVED_CANONICAL_ID
        })
        .expect("Symbol-timing offset must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("gardner") || r.contains("early-late"));
    assert!(r.contains("symbol-") && r.contains("clock alignment claim"));
    assert!(r.contains("not a carrier-") && r.contains("phase alignment claim"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_m_spectral_mask_is_parameterizationof_fft_band_energy() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == SPECTRAL_MASK_VIOLATION_RESERVED_PRIMITIVE_ID
        })
        .expect("Spectral mask violation must have ParameterizationOf record");
    assert!(rec.reason.contains("FFT band-energy"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("itu-r sm") || r.contains("etsi en") || r.contains("fcc part 15"));
}

#[test]
fn t12_m_snr_drop_is_parameterizationof_fft_band_energy() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == SNR_DROP_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == SNR_DROP_RESERVED_PRIMITIVE_ID
        })
        .expect("SNR drop must have ParameterizationOf record");
    assert!(rec.reason.contains("FFT band-energy"));
}

#[test]
fn t12_m_burst_preamble_miss_is_parameterizationof_autocorrelation_break() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == BURST_PREAMBLE_MISS_RESERVED_PRIMITIVE_ID
        })
        .expect("Burst preamble miss must have ParameterizationOf record");
    assert!(rec.reason.contains("Autocorrelation break"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("cross-") && r.contains("correlation template"));
}

#[test]
fn t12_m_frame_error_burst_is_parameterizationof_error_burst() {
    let p = seed_t12_m_rf_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID
        })
        .expect("Frame-error burst must have ParameterizationOf record");
    assert!(rec.reason.contains("Error burst"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("ieee 802.11") || r.contains("3gpp lte") || r.contains("5g nr"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_m_fft_domain_transfer_to_rf_exists() {
    let p = seed_t12_m_rf_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == FFT_BAND_ENERGY_SEED_ID
        })
        .expect("FFT band-energy must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared spectral ancestor"));
    assert!(r.contains("rfcommunications"));
}

#[test]
fn t12_m_residual_envelope_domain_transfer_to_rf_exists() {
    let p = seed_t12_m_rf_proposal();
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
    assert!(r.contains("rfcommunications"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_m_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_m_rf_proposal();
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
fn t12_m_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_m_rf_proposal();
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
fn t12_m_authority_resolution_covers_all_seed_rf_ids() {
    let p = seed_t12_m_rf_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12M_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.m"
        );
    }
}

#[test]
fn t12_m_every_dedup_record_has_reason() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_m_canonical_addition_ids_are_in_6303_to_6308_range() {
    let p = seed_t12_m_rf_proposal();
    // Reserved ids 6301 and 6302 are deliberately unused — the CFO
    // and EVM ideas they once shadowed collapsed onto SEED 53 and
    // SEED 54 respectively under the SEED-walk-first discipline.
    // CanonicalAddition records occupy 6303..=6308 only.
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6303..=6308).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.m reserved range \
                 6303..=6308 (6301 / 6302 deliberately unused)",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_m_parameterization_ids_are_in_6309_to_6312_range() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6309..=6312).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.m reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_m_rejection_ids_are_in_6313_to_6314_range() {
    let p = seed_t12_m_rf_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6313..=6314).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.m reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_m_frame_error_burst_references_error_burst_seed() {
    let p = seed_t12_m_rf_proposal();
    // Verify the SEED 41 ancestor is reachable from the proposal
    // via the genealogy edge AND that ERROR_BURST_SEED_ID is the
    // referenced ancestor in the ParameterizationOf reason text.
    let edge = p.body.proposed_genealogy_edges.iter().find(|e| {
        e.from_canonical_id.0 == FRAME_ERROR_BURST_RESERVED_PRIMITIVE_ID
            && e.to_canonical_id.0 == ERROR_BURST_SEED_ID
            && e.edge_kind_wire_name == "ParameterVariantOf"
    });
    assert!(
        edge.is_some(),
        "Frame-error burst must have ParameterVariantOf edge to SEED {ERROR_BURST_SEED_ID}"
    );
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn rf_proposal_text_rendering_byte_stable() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn rf_proposal_json_rendering_byte_stable() {
    let p = seed_t12_m_rf_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
