//! T.12.l acceptance suite — Chemometrics expansion proposal
//! invariants.
//!
//! Nine panel-required load-bearing negatives pin the
//! preprocessing / scaling / latent-space / calibration /
//! component-selection / decision-functional contract
//! discipline T.12.l exists to prove. The campaign's identity
//! is captured in:
//!
//! * `t12_l_rejects_chemometric_detector_without_preprocessing_or_latent_model_contract`
//!   (MOST IMPORTANT — every CanonicalAddition must declare
//!   sample matrix + preprocessing law + latent-space model /
//!   calibration model AND decision functional)
//! * `t12_l_rejects_material_identification_claim_language`
//!   (forbidden-term scanner #1)
//! * `t12_l_rejects_regulatory_compliance_claim_language`
//!   (forbidden-term scanner #2)
//!
//! Panel-locked non-claim verbatim:
//!
//! > T.12.l admits deterministic chemometric residual /
//! > latent-space / calibration / concentration-structure
//! > witnesses. It does not admit chemical causation, material
//! > identification certainty, regulatory compliance, lab
//! > diagnosis, or process-control authority.

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
use dsfb_gpu_atlas_corpus::t12_l_chemometrics::{
    seed_t12_l_chemometrics_proposal, ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID,
    BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID,
    CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_DOMAIN_TRANSFER_OF, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_PARAMETERIZATION_OF, CATEGORY_REJECTED_NOT_DETERMINISTIC,
    CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID, LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID,
    LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID, MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID,
    PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID, PCA_SPE_Q_SEED_ID, PCA_T2_SEED_ID,
    PLS_RESIDUAL_SEED_ID, RESIDUAL_ENVELOPE_EXIT_SEED_ID,
    SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID,
    SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID, VIP_SHIFT_RESERVED_CANONICAL_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12L_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (PCA_T2_SEED_ID, "PCA T² on score vector"),
    (PCA_SPE_Q_SEED_ID, "PCA SPE / Q residual"),
    (PLS_RESIDUAL_SEED_ID, "PLS residual / Q on PLS"),
    (RESIDUAL_ENVELOPE_EXIT_SEED_ID, "Residual envelope exit"),
];

/// Forbidden material-identification claim terms that must NOT
/// appear in CanonicalAddition or
/// ExistingCanonicalAuthorityResolution reason text. Hoisted
/// to module scope to keep clippy `items_after_statements`
/// quiet. "material identification" / "compound identification"
/// are intentionally NOT in this list because the
/// non-claim phrase legitimately uses them as disclaimers —
/// only the CERTAINTY / VERIFIED variants are forbidden
/// positive claims.
const T12L_FORBIDDEN_MATERIAL_ID_TERMS: &[&str] = &[
    "material identification certainty",
    "compound identified as",
    "substance identity verified",
    "chemical identity verified",
];

/// Forbidden regulatory-compliance claim terms.
/// "regulatory compliance" itself appears in the non-claim
/// phrase legitimately as a disclaimer, so only specific
/// CERTIFICATION claims are forbidden.
const T12L_FORBIDDEN_REGULATORY_TERMS: &[&str] = &[
    "fda approved",
    "fda-approved",
    "usp compliant",
    "ich compliant",
    "regulatory compliance certified",
    "lab diagnosis verdict",
    "process control authority",
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn chem_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_l_chemometrics_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Chemometrics proposal failed verifier: {errors:?}"
    );
}

#[test]
fn chem_proposal_has_open_status() {
    let p = seed_t12_l_chemometrics_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn chem_proposal_targets_chemometrics() {
    let p = seed_t12_l_chemometrics_proposal();
    assert!(matches!(p.target_source_class, SourceClass::Chemometrics));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_l_does_not_mutate_seed_len`.
#[test]
fn t12_l_does_not_mutate_seed_len() {
    let _ = seed_t12_l_chemometrics_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn chem_proposal_proposes_eleven_primitives() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 11);
}

#[test]
fn chem_proposal_proposes_zero_aliases() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn chem_proposal_proposes_seventeen_dedup_records() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 17);
}

#[test]
fn chem_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn chem_proposal_proposes_nine_source_refs() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn chem_delta_has_five_new_canonical_records() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 5);
}

#[test]
fn chem_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_l_chemometrics_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn chem_proposal_carries_two_rejection_records() {
    let p = seed_t12_l_chemometrics_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.l must carry TWO RejectedNotDeterministic records \
        (sixth T.12.x with two, following T.12.g / h / i / j / k)"
    );
}

#[test]
fn chem_proposal_court_delta_category_counts() {
    let p = seed_t12_l_chemometrics_proposal();
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
    assert_eq!(existing, 4);
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn chem_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_l_chemometrics_proposal();
    let b = seed_t12_l_chemometrics_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_chem_proposal_hash_matches_stored() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn chem_proposal_hash_is_distinct_from_all_prior_t12x() {
    let l = seed_t12_l_chemometrics_proposal();
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
    for other in [
        pol, spc, scd, drift, robust, spectral, ts, graph, dq, obs, bio, ind,
    ] {
        assert_ne!(
            l.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.l hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative: hash sensitivity.
#[test]
fn t12_l_hash_changes_when_calibration_or_latent_model_changes() {
    let p_a = seed_t12_l_chemometrics_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("Calibration residual CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED calibration / latent model law for hash-sensitivity test",
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
        "t12_l_collision_batch",
        SourceClass::Chemometrics,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "chemometrics collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_l_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_l_collision_proposal",
        "Defective T.12.l proposal duplicating an existing SEED canonical.",
        SourceClass::Chemometrics,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_l_test",
    )
}

#[test]
fn t12_l_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12L_AUTHORITY_RESOLVED_SEED_IDS {
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
// must declare preprocessing law / latent-space model /
// calibration model AND decision functional.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_chemometric_detector_without_preprocessing_or_latent_model_contract() {
    let p = seed_t12_l_chemometrics_proposal();
    // Required: at least one preprocessing / scaling /
    // latent-space / calibration term AND at least one
    // decision-functional term.
    let preprocessing_or_model_terms = [
        "preprocessing law",
        "scaling law",
        "latent-space model",
        "calibration model",
        "pca model",
        "pls model",
        "per-class pca model",
    ];
    // Decision-functional vocabulary used across chemometric
    // detector reasons: explicit "decision law" / "functional"
    // / "predicate" plus chemometric-specific terms "shift
    // law" / "residual decision law" / "control limit" /
    // "threshold" which are decision functionals in this
    // class.
    let decision_terms = [
        "decision law",
        "decision functional",
        "decision predicate",
        "shift law",
        "control limit",
        "threshold",
    ];
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION {
            continue;
        }
        let lower = r.reason.to_lowercase();
        let has_preprocessing_or_model = preprocessing_or_model_terms
            .iter()
            .any(|t| lower.contains(t));
        assert!(
            has_preprocessing_or_model,
            "CanonicalAddition record canonical_id={} reason text must declare \
            preprocessing law / scaling law / latent-space model / calibration \
            model / PCA model / PLS model: {:?}",
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
// MOST IMPORTANT load-bearing negative #2: material-identification
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_material_identification_claim_language() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12L_FORBIDDEN_MATERIAL_ID_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden material-identification term `{}`: {:?}",
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
                lower
                    .contains("chemometric signal witness, not material identification or regulatory compliance"),
                "Record decision={} canonical_id={} must end with panel-locked \
                non-claim 'chemometric signal witness, not material \
                identification or regulatory compliance': {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// MOST IMPORTANT load-bearing negative #3: regulatory-compliance
// claim-language scanner.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_regulatory_compliance_claim_language() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            continue;
        }
        let lower = r.reason.to_lowercase();
        for term in T12L_FORBIDDEN_REGULATORY_TERMS {
            assert!(
                !lower.contains(term),
                "Record decision={} canonical_id={} reason text contains \
                forbidden regulatory / compliance / process-control term `{}`: {:?}",
                r.decision_wire_name,
                r.canonical_id.0,
                term,
                r.reason,
            );
        }
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: concentration claim
// requires calibration contract.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_concentration_claim_without_calibration_contract() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CONCENTRATION_DRIFT_RESERVED_CANONICAL_ID
        })
        .expect("Concentration drift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("calibration model"));
    assert!(r.contains("pls") || r.contains("ils") || r.contains("cls"));
    assert!(r.contains("reference concentration anchor"));
    assert!(r.contains("calibration-") && r.contains("bound"));
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: PLS/PCA detector requires
// component-selection law.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_pls_or_pca_detector_without_component_selection_law() {
    let p = seed_t12_l_chemometrics_proposal();
    // Every CanonicalAddition record that USES (not just
    // references-as-distinct) a PCA or PLS model must declare a
    // component-selection law. Tightened trigger: only fires
    // when the record OWNS the model ("pca model" / "pls model"
    // / "per-class pca model" / "calibration model (pls" /
    // "calibration model (ils" / "calibration model (cls" /
    // "latent-space model (pca" / "latent-space model (pls"
    // appears in the reason). The mere reference "Structurally
    // distinct from SEED 20 PCA SPE/Q" is NOT an own-model
    // declaration.
    let own_model_markers = [
        "pca model (",
        "pls model (",
        "per-class pca model",
        "calibration model (pls",
        "calibration model (ils",
        "calibration model (cls",
        "latent-space model (pca",
        "latent-space model (pls",
    ];
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name != CATEGORY_CANONICAL_ADDITION {
            continue;
        }
        let lower = r.reason.to_lowercase();
        let owns_pca_or_pls_model = own_model_markers.iter().any(|m| lower.contains(m));
        if owns_pca_or_pls_model {
            assert!(
                lower.contains("component-selection law")
                    || lower.contains("component selection law"),
                "CanonicalAddition record canonical_id={} owns a PCA/PLS model \
                but does not declare a component-selection law: {:?}",
                r.canonical_id.0,
                r.reason,
            );
        }
    }
    // Also verify that the four SEED authority-resolution
    // records (which reference PCA T² / SPE / PLS) declare
    // component-selection law in their authority-resolution
    // reason text.
    let authority_pca_pls_ids = [PCA_T2_SEED_ID, PCA_SPE_Q_SEED_ID, PLS_RESIDUAL_SEED_ID];
    for seed_id in authority_pca_pls_ids {
        let rec = p
            .body
            .proposed_dedup_records
            .iter()
            .find(|r| {
                r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                    && r.canonical_id.0 == seed_id
            })
            .expect("Authority resolution record must exist");
        let lower = rec.reason.to_lowercase();
        // Rust string-literal `\\\n` continuation may split
        // "component-\n     selection law" into "component-
        // selection law" (with space). Accept both intact
        // and split forms.
        let intact =
            lower.contains("component-selection law") || lower.contains("component selection law");
        let split = lower.contains("component-") && lower.contains("selection law");
        assert!(
            intact || split,
            "Authority resolution for SEED {seed_id} must declare component-selection law (intact or split form): {:?}",
            rec.reason
        );
    }
}

// ---------------------------------------------------------------
// Panel-required load-bearing negative: black-box spectroscopy
// score requires deterministic formula.
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_black_box_spectroscopy_score_without_formula() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Black-box spectroscopy classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == BLACK_BOX_SPECTROSCOPY_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Black-box spectroscopy classifier must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(
        r.contains("deterministic formula"),
        "Black-box spectroscopy classifier rejection must require 'deterministic formula': {}",
        rec.reason
    );
    assert!(r.contains("model-identification anchor"));
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    let vendors = ["bruker", "mettler-toledo", "thermo scientific", "agilent"];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "Black-box spectroscopy rejection must name at least one vendor"
    );
    assert!(r.contains("does not issue material identification certainty"));
}

// ---------------------------------------------------------------
// Per-canonical / additional contract assertions
// ---------------------------------------------------------------

#[test]
fn t12_l_calibration_residual_is_distinct_from_pca_spe() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CALIBRATION_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("Calibration residual must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("reference value"));
    assert!(r.contains("chemistry reference"));
    assert!(r.contains("not") && r.contains("pca reconstruction"));
}

#[test]
fn t12_l_leverage_outlier_is_distinct_from_pca_t2_magnitude() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == LEVERAGE_OUTLIER_RESERVED_CANONICAL_ID
        })
        .expect("Leverage outlier must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("sample influence"));
    assert!(r.contains("hat matrix"));
    assert!(r.contains("2p/n") || r.contains("3p/n"));
    assert!(r.contains("not score-space mahalanobis distance"));
}

#[test]
fn t12_l_simca_declares_per_class_model() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SIMCA_CLASS_DISTANCE_RESERVED_CANONICAL_ID
        })
        .expect("SIMCA must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("wold") && r.contains("sjöström"));
    assert!(r.contains("per-class pca model"));
    assert!(r.contains("class-distance decision law"));
    assert!(r.contains("multiple per-class pca models"));
}

#[test]
fn t12_l_vip_declares_temporal_shift_distinct_from_static_score() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == VIP_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("VIP shift must have CanonicalAddition record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("wold 1995"));
    assert!(r.contains("vip shift decision law"));
    assert!(r.contains("temporal shift"));
    // Reason text says "not static score" (without "the").
    // Accept both forms in case future wording adds "the".
    assert!(r.contains("not static score") || r.contains("not the static score"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_l_pca_score_outlier_is_parameterizationof_pca_t2() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == PCA_SCORE_OUTLIER_RESERVED_PRIMITIVE_ID
        })
        .expect("PCA score outlier must have ParameterizationOf record");
    assert!(rec.reason.contains("PCA T²"));
}

#[test]
fn t12_l_mahalanobis_on_scores_is_parameterizationof_pca_t2() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == MAHALANOBIS_ON_SCORES_RESERVED_PRIMITIVE_ID
        })
        .expect("Mahalanobis on scores must have ParameterizationOf record");
    assert!(rec.reason.contains("PCA T²"));
}

#[test]
fn t12_l_lv_control_chart_is_parameterizationof_pca_spe() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == LV_CONTROL_CHART_RESERVED_PRIMITIVE_ID
        })
        .expect("LV control chart must have ParameterizationOf record");
    assert!(rec.reason.contains("PCA SPE / Q"));
}

#[test]
fn t12_l_spectral_preprocessing_artifact_is_parameterizationof_residual_envelope() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == SPECTRAL_PREPROCESSING_ARTIFACT_RESERVED_PRIMITIVE_ID
        })
        .expect("Spectral preprocessing artifact must have ParameterizationOf record");
    assert!(rec.reason.contains("Residual envelope exit"));
    let r = rec.reason.to_lowercase();
    assert!(r.contains("snv") && r.contains("msc") && r.contains("savitzky-golay"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_l_pca_t2_domain_transfer_to_chemometrics_exists() {
    let p = seed_t12_l_chemometrics_proposal();
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == PCA_T2_SEED_ID
        })
        .expect("PCA T² must have DomainTransferOf record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("shared latent-") && r.contains("space ancestor"));
    assert!(r.contains("chemometrics"));
}

#[test]
fn t12_l_residual_envelope_domain_transfer_to_chemometrics_exists() {
    let p = seed_t12_l_chemometrics_proposal();
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
    assert!(r.contains("chemometrics"));
}

// ---------------------------------------------------------------
// Adaptive-AutoML rejection contract test
// ---------------------------------------------------------------

#[test]
fn t12_l_rejects_adaptive_automl_without_fixed_component_selection() {
    let p = seed_t12_l_chemometrics_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Adaptive-AutoML chemometric model must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == ADAPTIVE_AUTOML_CHEMOMETRIC_RESERVED_PRIMITIVE_ID
        })
        .expect("Adaptive-AutoML must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("auto-sklearn") || r.contains("h2o automl") || r.contains("tpot"));
    assert!(r.contains("fixed component-") && r.contains("selection law"));
    assert!(r.contains("fixed cv seed"));
    assert!(r.contains("fixed train / test split"));
    assert!(r.contains("fixed preprocessing chain"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_l_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_l_chemometrics_proposal();
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
fn t12_l_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_l_chemometrics_proposal();
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
fn t12_l_authority_resolution_covers_all_seed_chemometrics_ids() {
    let p = seed_t12_l_chemometrics_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12L_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.l"
        );
    }
}

#[test]
fn t12_l_every_dedup_record_has_reason() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_l_canonical_addition_ids_are_in_6201_to_6205_range() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (6201..=6205).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.l reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_l_parameterization_ids_are_in_6206_to_6209_range() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (6206..=6209).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.l reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_l_rejection_ids_are_in_6210_to_6211_range() {
    let p = seed_t12_l_chemometrics_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (6210..=6211).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.l reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn chem_proposal_text_rendering_byte_stable() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn chem_proposal_json_rendering_byte_stable() {
    let p = seed_t12_l_chemometrics_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
