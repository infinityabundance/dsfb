//! T.12.f acceptance suite — Time-Series Structure / Control
//! Residuals expansion proposal invariants.
//!
//! Ten panel-required load-bearing negatives pin the residual-
//! and-decision-law discipline T.12.f exists to prove ("a model
//! is not a detector until the residual and decision law are
//! declared"). Plus per-SEED-record collision tests for every
//! existing T.12.f-relevant canonical and per-CanonicalAddition
//! contract-declaration tests.

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
use dsfb_gpu_atlas_corpus::t12_f_timeseries::{
    seed_t12_f_timeseries_proposal, ACTUATOR_STICTION_SEED_ID,
    ARIMA_RESIDUAL_RESERVED_CANONICAL_ID, AR_RESIDUAL_RESERVED_CANONICAL_ID,
    AUTOCORRELATION_BREAK_SEED_ID, BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, ERROR_BURST_SEED_ID,
    INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID, LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID,
    OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID, PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID,
    PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID, RESIDUAL_ENVELOPE_EXIT_SEED_ID,
    RUN_LENGTH_RESERVED_CANONICAL_ID, SENSOR_BIAS_SEED_ID, STL_RESIDUAL_RESERVED_CANONICAL_ID,
    UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID, VALVE_HUNTING_SEED_ID,
    VARIANCE_RATIO_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

/// Every existing SEED canonical id T.12.f recognises via an
/// `ExistingCanonicalAuthorityResolution` record.
const T12F_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
    (SENSOR_BIAS_SEED_ID, "Sensor bias detector"),
    (ACTUATOR_STICTION_SEED_ID, "Actuator stiction detector"),
    (VALVE_HUNTING_SEED_ID, "Valve hunting detector"),
    (
        AUTOCORRELATION_BREAK_SEED_ID,
        "Autocorrelation-coefficient break",
    ),
    (ERROR_BURST_SEED_ID, "Error burst"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn timeseries_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_f_timeseries_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Time-series proposal failed verifier: {errors:?}"
    );
}

#[test]
fn timeseries_proposal_has_open_status() {
    let p = seed_t12_f_timeseries_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn timeseries_proposal_targets_time_series_structure() {
    let p = seed_t12_f_timeseries_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::TimeSeriesStructure
    ));
}

/// Load-bearing negative #1 (panel-required).
#[test]
fn t12_f_does_not_mutate_seed_len() {
    let _ = seed_t12_f_timeseries_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn timeseries_proposal_proposes_twelve_primitives() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 12);
}

#[test]
fn timeseries_proposal_proposes_zero_aliases() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn timeseries_proposal_proposes_nineteen_dedup_records() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 19);
}

#[test]
fn timeseries_proposal_proposes_ten_genealogy_edges() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 10);
}

#[test]
fn timeseries_proposal_proposes_eight_source_refs() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 8);
}

#[test]
fn timeseries_delta_has_eight_new_canonical_records() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn timeseries_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_f_timeseries_proposal();
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

#[test]
fn timeseries_proposal_court_delta_category_counts() {
    let p = seed_t12_f_timeseries_proposal();
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
    assert_eq!(canonical, 8);
    assert_eq!(existing, 6);
    assert_eq!(transfer, 1);
    assert_eq!(paramof, 3);
    assert_eq!(rejected, 1);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn timeseries_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_f_timeseries_proposal();
    let b = seed_t12_f_timeseries_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_timeseries_proposal_hash_matches_stored() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn timeseries_proposal_hash_is_distinct_from_t12_0_a_b_c_d_e() {
    let ts = seed_t12_f_timeseries_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        pol.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        spc.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        scd.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        drift.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        robust.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        ts.corpus_amendment_proposal_hash_v1,
        spectral.corpus_amendment_proposal_hash_v1
    );
}

/// Load-bearing negative #9 (panel-required):
/// `t12_f_hash_changes_when_residual_definition_changes`.
#[test]
fn t12_f_hash_changes_when_residual_definition_changes() {
    let p_a = seed_t12_f_timeseries_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("Observer residual CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED residual-definition for hash-sensitivity test",
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
// SEED collision load-bearing negative + per-id tests
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
    test_label: &'static str,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_f_collision_batch",
        SourceClass::TimeSeriesStructure,
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
        "t12_f_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_f_collision_proposal",
        "Defective T.12.f-style proposal duplicating an existing SEED canonical.",
        SourceClass::TimeSeriesStructure,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_f_test",
    )
}

/// Load-bearing negative (panel-required):
/// `t12_f_existing_seed_collision_requires_authority_resolution`.
#[test]
fn t12_f_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12F_AUTHORITY_RESOLVED_SEED_IDS {
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

#[test]
fn t12_f_rejects_duplicate_sensor_bias() {
    let p = build_defective_collision_proposal(SENSOR_BIAS_SEED_ID, "Sensor bias");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == SENSOR_BIAS_SEED_ID
    )));
}

#[test]
fn t12_f_rejects_duplicate_valve_hunting() {
    let p = build_defective_collision_proposal(VALVE_HUNTING_SEED_ID, "Valve hunting");
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == VALVE_HUNTING_SEED_ID
    )));
}

#[test]
fn t12_f_rejects_duplicate_residual_envelope_exit() {
    let p = build_defective_collision_proposal(
        RESIDUAL_ENVELOPE_EXIT_SEED_ID,
        "Residual envelope exit",
    );
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == RESIDUAL_ENVELOPE_EXIT_SEED_ID
    )));
}

// ---------------------------------------------------------------
// Residual-and-decision-law load-bearing negatives (panel-required)
// ---------------------------------------------------------------

/// Load-bearing negative #2 (panel-required):
/// `t12_f_rejects_arima_residual_without_model_order_and_fit_law`.
#[test]
fn t12_f_rejects_arima_residual_without_model_order_and_fit_law() {
    let p = seed_t12_f_timeseries_proposal();
    let arima = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == ARIMA_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("ARIMA residual must have CanonicalAddition record");
    let r = arima.reason.to_lowercase();
    assert!(
        r.contains("arima order") || r.contains("order (p"),
        "ARIMA residual must declare ARIMA order: {}",
        arima.reason
    );
    assert!(
        r.contains("fit law"),
        "ARIMA residual must declare fit law: {}",
        arima.reason
    );
    assert!(
        r.contains("residual definition"),
        "ARIMA residual must declare residual definition: {}",
        arima.reason
    );
}

/// Load-bearing negative #3 (panel-required):
/// `t12_f_rejects_stl_residual_without_seasonality_and_decomposition_law`.
#[test]
fn t12_f_rejects_stl_residual_without_seasonality_and_decomposition_law() {
    let p = seed_t12_f_timeseries_proposal();
    let stl = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == STL_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("STL residual must have CanonicalAddition record");
    let r = stl.reason.to_lowercase();
    assert!(
        r.contains("seasonality period"),
        "STL residual must declare seasonality period: {}",
        stl.reason
    );
    assert!(
        r.contains("loess") || r.contains("decomposition-method"),
        "STL residual must declare decomposition method: {}",
        stl.reason
    );
    assert!(
        r.contains("residual definition"),
        "STL residual must declare residual definition: {}",
        stl.reason
    );
}

/// Load-bearing negative #4 (panel-required):
/// `t12_f_rejects_innovation_sequence_without_observer_model`.
#[test]
fn t12_f_rejects_innovation_sequence_without_observer_model() {
    let p = seed_t12_f_timeseries_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Innovation sequence must NOT be in new_canonical_records"
    );
    let innov = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == INNOVATION_SEQUENCE_RESERVED_PRIMITIVE_ID
        })
        .expect("Innovation sequence must have ParameterizationOf record");
    let r = innov.reason.to_lowercase();
    assert!(
        r.contains("observer residual"),
        "Innovation sequence must reference Observer residual parent: {}",
        innov.reason
    );
    assert!(
        r.contains("kalman"),
        "Innovation sequence must declare Kalman parameterization: {}",
        innov.reason
    );
    assert!(
        r.contains("covariances") || r.contains("covariance"),
        "Innovation sequence must declare Q/R covariance contract: {}",
        innov.reason
    );
}

/// Load-bearing negative #5 (panel-required):
/// `t12_f_rejects_periodicity_break_without_lag_and_peak_selection_law`.
#[test]
fn t12_f_rejects_periodicity_break_without_lag_and_peak_selection_law() {
    let p = seed_t12_f_timeseries_proposal();
    let periodicity = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == PERIODICITY_BREAK_RESERVED_PRIMITIVE_ID
        })
        .expect("Periodicity break must have ParameterizationOf record");
    let r = periodicity.reason.to_lowercase();
    assert!(
        r.contains("lag-correlation break")
            || r.contains("lag-range")
            || r.contains("candidate-lag range"),
        "Periodicity break must declare lag-range / lag-correlation parent: {}",
        periodicity.reason
    );
    assert!(
        r.contains("peak-selection"),
        "Periodicity break must declare peak-selection law: {}",
        periodicity.reason
    );
}

/// Load-bearing negative #6 (panel-required):
/// `t12_f_rejects_variance_ratio_without_window_pair_law`.
#[test]
fn t12_f_rejects_variance_ratio_without_window_pair_law() {
    let p = seed_t12_f_timeseries_proposal();
    let vr = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == VARIANCE_RATIO_RESERVED_CANONICAL_ID
        })
        .expect("Variance-ratio shift must have CanonicalAddition record");
    let r = vr.reason.to_lowercase();
    assert!(
        r.contains("window-pair law"),
        "Variance-ratio must declare window-pair law: {}",
        vr.reason
    );
    assert!(
        r.contains("short window") && r.contains("long window"),
        "Variance-ratio must declare short + long window sizes: {}",
        vr.reason
    );
    assert!(
        r.contains("ratio definition"),
        "Variance-ratio must declare ratio definition: {}",
        vr.reason
    );
}

/// Load-bearing negative #7 (panel-required):
/// `t12_f_rejects_run_length_detector_without_event_definition`.
#[test]
fn t12_f_rejects_run_length_detector_without_event_definition() {
    let p = seed_t12_f_timeseries_proposal();
    let run_length = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == RUN_LENGTH_RESERVED_CANONICAL_ID
        })
        .expect("Run-length anomaly must have CanonicalAddition record");
    let r = run_length.reason.to_lowercase();
    assert!(
        r.contains("event definition"),
        "Run-length must declare event definition: {}",
        run_length.reason
    );
    assert!(
        r.contains("run-length law"),
        "Run-length must declare run-length law: {}",
        run_length.reason
    );
}

/// Load-bearing negative #8 (panel-required, MOST IMPORTANT):
/// `t12_f_rejects_control_residual_without_plant_or_observer_contract`.
/// Both observer residual + parity-space residual canonicals
/// must declare a plant-or-observer contract.
#[test]
fn t12_f_rejects_control_residual_without_plant_or_observer_contract() {
    let p = seed_t12_f_timeseries_proposal();
    for canonical_id in [
        OBSERVER_RESIDUAL_RESERVED_CANONICAL_ID,
        PARITY_SPACE_RESIDUAL_RESERVED_CANONICAL_ID,
    ] {
        let record = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                    && r.canonical_id.0 == canonical_id
            })
            .unwrap_or_else(|| {
                panic!("control-residual canonical id {canonical_id} must have CanonicalAddition record")
            });
        let r = record.reason.to_lowercase();
        assert!(
            r.contains("plant or observer contract")
                || r.contains("state model")
                || r.contains("parity equations"),
            "control-residual record at id {canonical_id} must declare plant-or-observer contract: {}",
            record.reason
        );
        assert!(
            r.contains("residual definition"),
            "control-residual record at id {canonical_id} must declare residual definition: {}",
            record.reason
        );
        assert!(
            r.contains("envelope law"),
            "control-residual record at id {canonical_id} must declare envelope law: {}",
            record.reason
        );
        assert!(
            r.contains("threshold"),
            "control-residual record at id {canonical_id} must declare threshold: {}",
            record.reason
        );
    }
    // Also: each control-residual ExistingCanonicalAuthorityResolution
    // (sensor bias, actuator stiction, valve hunting) declares its
    // plant-or-observer contract.
    for seed_id in [
        SENSOR_BIAS_SEED_ID,
        ACTUATOR_STICTION_SEED_ID,
        VALVE_HUNTING_SEED_ID,
    ] {
        let record = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                    && r.canonical_id.0 == seed_id
            })
            .unwrap_or_else(|| {
                panic!("control-residual SEED id {seed_id} must have ExistingCanonicalAuthorityResolution record")
            });
        let r = record.reason.to_lowercase();
        assert!(
            r.contains("plant or observer contract"),
            "control-residual SEED id {seed_id} record must declare plant-or-observer contract: {}",
            record.reason
        );
    }
}

/// AR residual must declare model order + fit law (sibling
/// invariant to the ARIMA test).
#[test]
fn t12_f_rejects_ar_residual_without_model_order_and_fit_law() {
    let p = seed_t12_f_timeseries_proposal();
    let ar = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == AR_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("AR residual must have CanonicalAddition record");
    let r = ar.reason.to_lowercase();
    assert!(
        r.contains("ar order p"),
        "AR residual must declare AR order p: {}",
        ar.reason
    );
    assert!(
        r.contains("fit law"),
        "AR residual must declare fit law: {}",
        ar.reason
    );
}

/// Lag-correlation break must declare multi-lag scope.
#[test]
fn t12_f_lag_correlation_break_declares_lag_range_and_normalization() {
    let p = seed_t12_f_timeseries_proposal();
    let lag = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == LAG_CORRELATION_BREAK_RESERVED_CANONICAL_ID
        })
        .expect("Lag-correlation break must have CanonicalAddition record");
    let r = lag.reason.to_lowercase();
    assert!(r.contains("lag range"));
    assert!(r.contains("autocorrelation convention"));
    assert!(r.contains("normalization"));
}

// ---------------------------------------------------------------
// RANSAC-style rejection: unidentified-model anomaly
// ---------------------------------------------------------------

#[test]
fn t12_f_rejects_unidentified_model_anomaly_as_canonical() {
    let p = seed_t12_f_timeseries_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Unidentified-model anomaly (id {UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID}) must NOT be in new_canonical_records"
    );
    let rejection = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == UNIDENTIFIED_MODEL_RESERVED_PRIMITIVE_ID
        })
        .expect("Unidentified-model anomaly must have RejectedNotDeterministic record");
    let r = rejection.reason.to_lowercase();
    assert!(r.contains("model-order-search seed"));
    assert!(r.contains("identification algorithm"));
    assert!(r.contains("fit-data anchor"));
    assert!(r.contains("tie-break law"));
    assert!(r.contains("numeric mode"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_f_burstiness_index_is_parameterizationof_error_burst() {
    let p = seed_t12_f_timeseries_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let burst = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == BURSTINESS_INDEX_RESERVED_PRIMITIVE_ID
        })
        .expect("Burstiness index must have ParameterizationOf record");
    assert!(
        burst.reason.contains("Error burst"),
        "Burstiness index must reference Error burst parent: {}",
        burst.reason
    );
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_f_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_f_timeseries_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_f_domain_transfer_target_must_be_in_seed() {
    let p = seed_t12_f_timeseries_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_f_every_dedup_record_has_reason() {
    let p = seed_t12_f_timeseries_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_f_canonical_addition_ids_are_in_5601_to_5608_range() {
    let p = seed_t12_f_timeseries_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5601..=5608).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.f reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn timeseries_proposal_text_rendering_byte_stable() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn timeseries_proposal_json_rendering_byte_stable() {
    let p = seed_t12_f_timeseries_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
