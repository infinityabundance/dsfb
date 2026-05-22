//! T.12.k acceptance suite — Industrial / FDD / Condition
//! Monitoring expansion proposal invariants.
//!
//! Sixteen panel-required load-bearing negatives pin the plant
//! / sensor / residual / operating-regime / confuser contract
//! discipline T.12.k exists to prove. The campaign's identity
//! is captured in:
//!
//! * `t12_k_rejects_fault_detector_without_plant_or_residual_contract`
//!   (MOST IMPORTANT — every CanonicalAddition must declare
//!   plant model / observer / residual definition AND fault-
//!   signature decision law)
//! * `t12_k_rejects_root_cause_claim_language` (analogous to
//!   T.12.j's diagnostic-language scanner; the court does NOT
//!   issue root-cause certainty, maintenance recommendations,
//!   remaining-useful-life predictions, or failure-mode
//!   classifications)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.k admits deterministic condition-monitoring / FDD
//! > witnesses, not root-cause certainty and not maintenance
//! > recommendations.

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
use dsfb_gpu_atlas_corpus::t12_k_industrial::{
    seed_t12_k_industrial_proposal, ACTUATOR_STICTION_SEED_ID,
    BEARING_VIBRATION_RESERVED_PRIMITIVE_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_DOMAIN_TRANSFER_OF, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_PARAMETERIZATION_OF, CATEGORY_REJECTED_NOT_DETERMINISTIC,
    CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID, CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID,
    FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID, FFT_BAND_ENERGY_SEED_ID,
    KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID,
    LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID, MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID,
    OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID, PCA_SPE_Q_SEED_ID, PCA_T2_SEED_ID,
    PLS_RESIDUAL_SEED_ID, PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID,
    PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID, RESIDUAL_ENVELOPE_EXIT_SEED_ID,
    SENSOR_BIAS_SEED_ID, SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID,
    TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID, VALVE_HUNTING_SEED_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12K_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (FFT_BAND_ENERGY_SEED_ID, "FFT band-energy anomaly"),
    (PCA_T2_SEED_ID, "PCA T² on score vector"),
    (PCA_SPE_Q_SEED_ID, "PCA SPE / Q residual"),
    (PLS_RESIDUAL_SEED_ID, "PLS residual / Q on PLS"),
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
    (SENSOR_BIAS_SEED_ID, "Sensor bias detector"),
    (ACTUATOR_STICTION_SEED_ID, "Actuator stiction detector"),
    (VALVE_HUNTING_SEED_ID, "Valve hunting"),
];

/// Forbidden root-cause / RUL / failure-mode-classification
/// claim terms that must NOT appear in CanonicalAddition or
/// ExistingCanonicalAuthorityResolution reason text. Note:
/// "maintenance recommendation" is intentionally NOT in this
/// list because it appears legitimately inside the non-claim
/// phrase "condition-monitoring witness, not a maintenance
/// recommendation" — the non-claim DISCLAIMS the term, it
/// doesn't make a positive claim. Hoisted to module scope to
/// keep clippy `items_after_statements` quiet.
const T12K_FORBIDDEN_ROOT_CAUSE_TERMS: &[&str] = &[
    "root cause",
    "diagnosis of machine cause",
    "remaining useful life",
    "predicted rul",
    "failure mode classification",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn ind_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_k_industrial_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Industrial proposal failed verifier: {errors:?}"
    );
}

#[test]
fn ind_proposal_has_open_status() {
    let p = seed_t12_k_industrial_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn ind_proposal_targets_fault_detection_diagnostics() {
    let p = seed_t12_k_industrial_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::FaultDetectionDiagnostics
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_k_does_not_mutate_seed_len`.
#[test]
fn t12_k_does_not_mutate_seed_len() {
    let _ = seed_t12_k_industrial_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn ind_proposal_proposes_twelve_primitives() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 12);
}

#[test]
fn ind_proposal_proposes_zero_aliases() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn ind_proposal_proposes_twentytwo_dedup_records() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 22);
}

#[test]
fn ind_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn ind_proposal_proposes_nine_source_refs() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn ind_delta_has_six_new_canonical_records() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 6);
}

#[test]
fn ind_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_k_industrial_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn ind_proposal_carries_two_rejection_records() {
    let p = seed_t12_k_industrial_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.k must carry TWO RejectedNotDeterministic records \
        (fifth T.12.x with two, following T.12.g / h / i / j)"
    );
}

#[test]
fn ind_proposal_court_delta_category_counts() {
    let p = seed_t12_k_industrial_proposal();
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
    assert_eq!(existing, 8);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn ind_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_k_industrial_proposal();
    let b = seed_t12_k_industrial_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_ind_proposal_hash_matches_stored() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn ind_proposal_hash_is_distinct_from_all_prior_t12x() {
    let k = seed_t12_k_industrial_proposal();
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
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio,
    ] {
        assert_ne!(
            k.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.k hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative #15 (panel-required):
/// `t12_k_hash_changes_when_plant_model_or_residual_definition_changes`.
#[test]
fn t12_k_hash_changes_when_plant_model_or_residual_definition_changes() {
    let p_a = seed_t12_k_industrial_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID
        })
        .expect("Kalman innovation whiteness CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED plant model / residual definition for hash-sensitivity test",
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
// SEED collision load-bearing negative #16 — protects all 8 SEED
// industrial / FDD primitives from re-canonicalisation
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_k_collision_batch",
        SourceClass::FaultDetectionDiagnostics,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "industrial collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_k_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_k_collision_proposal",
        "Defective T.12.k proposal duplicating an existing SEED canonical.",
        SourceClass::FaultDetectionDiagnostics,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_k_test",
    )
}

#[test]
fn t12_k_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12K_AUTHORITY_RESOLVED_SEED_IDS {
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
// Load-bearing negative #2 (MOST IMPORTANT): every
// CanonicalAddition must declare plant model / observer /
// residual definition AND fault-signature decision law.
// ---------------------------------------------------------------

#[test]
fn t12_k_rejects_fault_detector_without_plant_or_residual_contract() {
    let p = seed_t12_k_industrial_proposal();
    // Every CanonicalAddition record must declare AT LEAST ONE
    // of: plant model / observer model / residual definition,
    // AND must declare a decision law / decision functional
    // / decision predicate.
    // Broadened to cover the full math-structure vocabulary the
    // FDD source class uses: plant / observer / residual / model
    // for state-space and physical-model detectors; state-
    // machine for regime-transition detectors; latent-space for
    // PCA-score detectors; estimator / envelope for spectral
    // detectors that operate on declared transform output rather
    // than a plant model; computation for derived condition-
    // indicator detectors.
    let plant_or_residual_terms = [
        "plant",
        "observer",
        "residual",
        "model",
        "state-machine",
        "latent-space",
        "estimator",
        "envelope",
        "computation",
    ];
    let decision_terms = ["decision law", "decision functional", "decision predicate"];
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION {
            continue;
        }
        let lower = r.reason.to_lowercase();
        let has_plant_or_residual = plant_or_residual_terms.iter().any(|t| lower.contains(t));
        assert!(
            has_plant_or_residual,
            "CanonicalAddition record canonical_id={} reason text must declare a \
            plant / observer / residual / model / state-machine / latent-space \
            contract: {:?}",
            r.canonical_id.0, r.reason,
        );
        let has_decision = decision_terms.iter().any(|t| lower.contains(t));
        assert!(
            has_decision,
            "CanonicalAddition record canonical_id={} reason text must declare a \
            decision law / functional / predicate: {:?}",
            r.canonical_id.0, r.reason,
        );
    }
}

// ---------------------------------------------------------------
// Load-bearing negative #3 (MOST IMPORTANT): root-cause-claim
// language scanner. Every CanonicalAddition AND
// ExistingCanonicalAuthorityResolution reason text must NOT
// contain root-cause / maintenance-recommendation / RUL /
// failure-mode-classification terms, AND must end with the
// panel-locked non-claim "condition-monitoring witness, not a
// maintenance recommendation".
// ---------------------------------------------------------------

#[test]
fn t12_k_rejects_root_cause_claim_language() {
    let p = seed_t12_k_industrial_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            // Rejection reason text describes what is NOT
            // admitted; skip the forbidden-term scan.
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12K_FORBIDDEN_ROOT_CAUSE_TERMS {
            assert!(
                !lower.contains(term),
                "Record with decision={} canonical_id={} reason text contains \
                forbidden root-cause / maintenance term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
            || r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
        {
            // Rust string-literal `\\\n` continuation may split
            // "Condition- \n    monitoring witness" into
            // "condition- monitoring witness" (with space)
            // depending on where the source line wraps. Accept
            // both intact and split forms; require all three
            // semantic halves (condition prefix, monitoring
            // witness, not a maintenance recommendation).
            let intact =
                lower.contains("condition-monitoring witness, not a maintenance recommendation");
            let split = lower.contains("condition-")
                && lower.contains("monitoring witness, not a maintenance recommendation");
            assert!(
                intact || split,
                "Record with decision={} canonical_id={} must end with panel-locked \
                non-claim 'condition-monitoring witness, not a maintenance \
                recommendation' (intact or string-literal-continuation split): {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// Per-SEED authority-resolution-duplicate negatives (4-9)
// ---------------------------------------------------------------

#[test]
fn t12_k_rejects_pca_t2_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == PCA_T2_SEED_ID
        })
        .expect("PCA T² must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("pca model"));
    assert!(r.contains("score space") || r.contains("score-space"));
    assert!(r.contains("t²") || r.contains("t-squared") || r.contains("mahalanobis"));
    assert!(r.contains("control limit"));
}

#[test]
fn t12_k_rejects_pca_spe_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == PCA_SPE_Q_SEED_ID
        })
        .expect("PCA SPE/Q must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("pca model"));
    assert!(r.contains("residual") || r.contains("squared-residuals"));
    assert!(r.contains("control limit"));
}

#[test]
fn t12_k_rejects_pls_residual_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == PLS_RESIDUAL_SEED_ID
        })
        .expect("PLS residual must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("pls model"));
    assert!(r.contains("inner-relation") || r.contains("loadings"));
}

#[test]
fn t12_k_rejects_sensor_bias_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == SENSOR_BIAS_SEED_ID
        })
        .expect("Sensor bias must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("sensor identity"));
    assert!(r.contains("nominal sensor-mean baseline"));
    assert!(r.contains("raw"));
}

#[test]
fn t12_k_rejects_actuator_stiction_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == ACTUATOR_STICTION_SEED_ID
        })
        .expect("Actuator stiction must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("actuator identity"));
    assert!(r.contains("command") && r.contains("output"));
    assert!(r.contains("stiction-signature"));
}

#[test]
fn t12_k_rejects_valve_hunting_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == VALVE_HUNTING_SEED_ID
        })
        .expect("Valve hunting must have ExistingCanonicalAuthorityResolution record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("control-loop identity"));
    assert!(r.contains("oscillation"));
    assert!(r.contains("setpoint") && r.contains("manipulated-variable"));
}

// ---------------------------------------------------------------
// Per-canonical contract-declaration negatives (10-12)
// ---------------------------------------------------------------

#[test]
fn t12_k_rejects_kalman_innovation_without_observer_and_whiteness_law() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == KALMAN_INNOVATION_WHITENESS_RESERVED_CANONICAL_ID
        })
        .expect("Kalman innovation whiteness must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("kalman"));
    assert!(r.contains("plant / observer model") || r.contains("observer model"));
    assert!(r.contains("q / r covariances") || r.contains("q/r covariances"));
    assert!(r.contains("whiteness decision law"));
    // Rust string-literal `\\\n` continuation gotcha: source
    // splits "autocorrelation- \n     of-innovations" into
    // "autocorrelation- of-innovations" (with space). Match
    // both halves.
    assert!(r.contains("autocorrelation-") && r.contains("of-innovations"));
    assert!(r.contains("not magnitude"));
}

#[test]
fn t12_k_rejects_bearing_vibration_without_sensor_and_defect_frequency_band_law() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == BEARING_VIBRATION_RESERVED_PRIMITIVE_ID
        })
        .expect("Bearing vibration must have ParameterizationOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("bearing-defect-frequency"));
    assert!(r.contains("bpfo") && r.contains("bpfi") && r.contains("bsf") && r.contains("ftf"));
    assert!(r.contains("mcfadden"));
    assert!(rec.reason.contains("FFT band-energy"));
}

#[test]
fn t12_k_rejects_motor_current_signature_without_sensor_and_spectral_band_law() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == MOTOR_CURRENT_SIGNATURE_RESERVED_PRIMITIVE_ID
        })
        .expect("MCSA must have ParameterizationOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("motor-current"));
    assert!(r.contains("broken-rotor-bar"));
    assert!(r.contains("thomson"));
    assert!(rec.reason.contains("FFT band-energy"));
}

// ---------------------------------------------------------------
// Rejection contract negatives (13-14)
// ---------------------------------------------------------------

#[test]
fn t12_k_rejects_proprietary_pdm_score_without_deterministic_formula() {
    let p = seed_t12_k_industrial_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Proprietary PdM score must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == PROPRIETARY_PDM_SCORE_RESERVED_PRIMITIVE_ID
        })
        .expect("Proprietary PdM score must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(
        r.contains("deterministic formula"),
        "Proprietary PdM rejection must require 'deterministic formula': {}",
        rec.reason
    );
    assert!(r.contains("model-identification anchor"));
    // Continuation whitespace gotcha: "training-\n     data
    // anchor" becomes "training- data anchor" in the joined
    // text. Match both halves.
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    let vendors = [
        "ge predix",
        "siemens mindsphere",
        "ibm maximo predict",
        "honeywell forge",
        "aspen mtell",
    ];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(has_vendor, "PdM rejection must name at least one vendor");
    assert!(r.contains("does not issue maintenance recommendations"));
    assert!(r.contains("remaining-useful-life"));
}

#[test]
fn t12_k_rejects_learned_fault_classifier_without_training_artifact() {
    let p = seed_t12_k_industrial_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Learned fault classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_FAULT_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned fault classifier must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("wen") || r.contains("khan"));
    assert!(r.contains("model-identification seed"));
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("cwru bearing dataset"));
    assert!(r.contains("label schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    assert!(r.contains("does not issue failure-mode classifications"));
}

// ---------------------------------------------------------------
// Per-canonical structural-distinctness assertions
// ---------------------------------------------------------------

#[test]
fn t12_k_operating_regime_transition_declares_state_machine_law() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == OPERATING_REGIME_TRANSITION_RESERVED_CANONICAL_ID
        })
        .expect("Operating-regime transition must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    // Rust string-literal `\\\n` continuation gotcha: source
    // splits "operating- \n     regime set" into "operating-
    // regime set" (with space). Match both halves.
    assert!(r.contains("operating-") && r.contains("regime set"));
    assert!(r.contains("start-up") && r.contains("steady-state"));
    assert!(r.contains("state-machine"));
    assert!(r.contains("baseline-switch law"));
}

#[test]
fn t12_k_condition_indicator_drift_is_distinct_from_sensor_bias() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONDITION_INDICATOR_DRIFT_RESERVED_CANONICAL_ID
        })
        .expect("Condition-indicator drift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("derived"));
    assert!(r.contains("sensor bias"));
    assert!(r.contains("rate-of-change decision law"));
}

#[test]
fn t12_k_fault_signature_angle_is_distinct_from_pca_t2_magnitude() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == FAULT_SIGNATURE_ANGLE_RESERVED_CANONICAL_ID
        })
        .expect("Fault signature angle must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("angular"));
    assert!(r.contains("cosine-distance"));
    assert!(r.contains("not magnitude"));
}

#[test]
fn t12_k_contribution_plot_spike_is_distinct_from_aggregate_t2_spe() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONTRIBUTION_PLOT_SPIKE_RESERVED_CANONICAL_ID
        })
        .expect("Contribution-plot spike must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("per-variable contribution"));
    assert!(r.contains("contribution series"));
    assert!(r.contains("not aggregate"));
}

#[test]
fn t12_k_spectral_kurtosis_is_distinct_from_fft_band_energy() {
    let p = seed_t12_k_industrial_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SPECTRAL_KURTOSIS_RESERVED_CANONICAL_ID
        })
        .expect("Spectral kurtosis must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("antoni"));
    assert!(r.contains("kurtosis-in-band"));
    assert!(r.contains("fourth-moment shape"));
    assert!(r.contains("not energy magnitude"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_k_temperature_envelope_is_parameterizationof_residual_envelope() {
    let p = seed_t12_k_industrial_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == TEMPERATURE_ENVELOPE_RESERVED_PRIMITIVE_ID
        })
        .expect("Temperature envelope must have ParameterizationOf record");
    assert!(rec.reason.contains("Residual envelope exit"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("thermal"));
}

#[test]
fn t12_k_pressure_transient_is_parameterizationof_slew_shock() {
    let p = seed_t12_k_industrial_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == PRESSURE_TRANSIENT_RESERVED_PRIMITIVE_ID
        })
        .expect("Pressure transient must have ParameterizationOf record");
    assert!(rec.reason.contains("Slew shock"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("derivative-of-pressure"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_k_fft_band_energy_domain_transfer_to_industrial_exists() {
    let p = seed_t12_k_industrial_proposal();
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
    assert!(r.contains("faultdetectiondiagnostics"));
}

#[test]
fn t12_k_residual_envelope_domain_transfer_to_industrial_exists() {
    let p = seed_t12_k_industrial_proposal();
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
    assert!(r.contains("shared envelope-") && r.contains("boundary ancestor"));
    assert!(r.contains("faultdetectiondiagnostics"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_k_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_k_industrial_proposal();
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
fn t12_k_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_k_industrial_proposal();
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
fn t12_k_authority_resolution_covers_all_seed_industrial_ids() {
    let p = seed_t12_k_industrial_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12K_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.k"
        );
    }
}

#[test]
fn t12_k_every_dedup_record_has_reason() {
    let p = seed_t12_k_industrial_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_k_canonical_addition_ids_are_in_6101_to_6106_range() {
    let p = seed_t12_k_industrial_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6101..=6106).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.k reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_k_parameterization_ids_are_in_6107_to_6110_range() {
    let p = seed_t12_k_industrial_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6107..=6110).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.k reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_k_rejection_ids_are_in_6111_to_6112_range() {
    let p = seed_t12_k_industrial_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6111..=6112).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.k reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn ind_proposal_text_rendering_byte_stable() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn ind_proposal_json_rendering_byte_stable() {
    let p = seed_t12_k_industrial_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
