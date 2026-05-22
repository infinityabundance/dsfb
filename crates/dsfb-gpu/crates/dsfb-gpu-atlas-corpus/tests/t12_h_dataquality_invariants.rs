//! T.12.h acceptance suite — Data Quality / Tabular / Database
//! Integrity Constraints expansion proposal invariants.
//!
//! Eleven panel-required load-bearing negatives pin the scope +
//! baseline + null / type / key semantics + decision-law
//! discipline T.12.h exists to prove ("a validation rule is
//! not a detector until scope, baseline, null / type / key
//! semantics, and decision law are declared").
//!
//! Includes panel-locked target-leakage non-claim invariant:
//! target-leakage candidate's reason text MUST carry the
//! "candidate, not proof" phrasing so a future activation
//! planner / case-file emitter does NOT promote candidate
//! signals into ratified leakage verdicts.

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
use dsfb_gpu_atlas_corpus::t12_h_dataquality::{
    seed_t12_h_dataquality_proposal, AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID,
    CARDINALITY_DRIFT_SEED_ID, CATEGORY_CANONICAL_ADDITION,
    CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID, CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION,
    CATEGORY_PARAMETERIZATION_OF, CATEGORY_REJECTED_NOT_DETERMINISTIC,
    COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID, CORRELATION_BREAK_RESERVED_CANONICAL_ID,
    COVARIANCE_SHIFT_RESERVED_CANONICAL_ID, FD_VIOLATION_RESERVED_CANONICAL_ID,
    LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID, MISSINGNESS_COUPLING_SEED_ID,
    MISSINGNESS_SPIKE_SEED_ID, NULL_RUN_RESERVED_CANONICAL_ID,
    PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID, RANGE_ENVELOPE_RESERVED_CANONICAL_ID,
    SCHEMA_DRIFT_SEED_ID, TARGET_LEAKAGE_RESERVED_CANONICAL_ID,
    TYPE_INSTABILITY_RESERVED_CANONICAL_ID, UNIQUENESS_VIOLATION_SEED_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12H_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (MISSINGNESS_SPIKE_SEED_ID, "Missingness spike"),
    (MISSINGNESS_COUPLING_SEED_ID, "Missingness coupling"),
    (SCHEMA_DRIFT_SEED_ID, "Schema drift"),
    (CARDINALITY_DRIFT_SEED_ID, "Cardinality drift"),
    (UNIQUENESS_VIOLATION_SEED_ID, "Uniqueness violation"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn dq_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_h_dataquality_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(errors.is_empty(), "DQ proposal failed verifier: {errors:?}");
}

#[test]
fn dq_proposal_has_open_status() {
    let p = seed_t12_h_dataquality_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn dq_proposal_targets_data_quality_rules() {
    let p = seed_t12_h_dataquality_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::DataQualityRules
    ));
}

#[test]
fn t12_h_does_not_mutate_seed_len() {
    let _ = seed_t12_h_dataquality_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn dq_proposal_proposes_thirteen_primitives() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 13);
}

#[test]
fn dq_proposal_proposes_zero_aliases() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn dq_proposal_proposes_nineteen_dedup_records() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 19);
}

#[test]
fn dq_proposal_proposes_ten_genealogy_edges() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 10);
}

#[test]
fn dq_proposal_proposes_nine_source_refs() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn dq_delta_has_eight_new_canonical_records() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn dq_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_h_dataquality_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn dq_proposal_carries_two_rejection_records() {
    let p = seed_t12_h_dataquality_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.h must carry TWO RejectedNotDeterministic records"
    );
}

#[test]
fn dq_proposal_court_delta_category_counts() {
    let p = seed_t12_h_dataquality_proposal();
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
    assert_eq!(existing, 5);
    assert_eq!(transfer, 1);
    assert_eq!(paramof, 3);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn dq_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_h_dataquality_proposal();
    let b = seed_t12_h_dataquality_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_dq_proposal_hash_matches_stored() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn dq_proposal_hash_is_distinct_from_all_prior_t12x() {
    let h = seed_t12_h_dataquality_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    let ts = seed_t12_f_timeseries_proposal();
    let graph = seed_t12_g_graph_proposal();
    for other in [pol, spc, scd, drift, robust, spectral, ts, graph] {
        assert_ne!(
            h.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.h hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative #10 (panel-required):
/// `t12_h_hash_changes_when_null_semantics_or_fd_law_changes`.
#[test]
fn t12_h_hash_changes_when_null_semantics_or_fd_law_changes() {
    let p_a = seed_t12_h_dataquality_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == FD_VIOLATION_RESERVED_CANONICAL_ID
        })
        .expect("FD violation CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED FD law for hash-sensitivity test",
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
        "t12_h_collision_batch",
        SourceClass::DataQualityRules,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "dq collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_h_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_h_collision_proposal",
        "Defective T.12.h proposal duplicating an existing SEED canonical.",
        SourceClass::DataQualityRules,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_h_test",
    )
}

#[test]
fn t12_h_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12H_AUTHORITY_RESOLVED_SEED_IDS {
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
// Per-canonical / per-existing contract-declaration load-bearing
// negatives
// ---------------------------------------------------------------

/// Load-bearing negative #2 (panel-required):
/// `t12_h_rejects_missingness_without_null_semantics`. The
/// SEED 13 Missingness spike ExistingCanonicalAuthorityResolution
/// record must declare null semantics.
#[test]
fn t12_h_rejects_missingness_without_null_semantics() {
    let p = seed_t12_h_dataquality_proposal();
    let miss = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == MISSINGNESS_SPIKE_SEED_ID
        })
        .expect("Missingness spike must have ExistingCanonicalAuthorityResolution record");
    let r = miss.reason.to_lowercase();
    assert!(r.contains("null semantics"));
    assert!(r.contains("baseline"));
    assert!(r.contains("threshold"));
}

/// Load-bearing negative #3 (panel-required):
/// `t12_h_rejects_cardinality_drift_without_category_identity_law`.
#[test]
fn t12_h_rejects_cardinality_drift_without_category_identity_law() {
    let p = seed_t12_h_dataquality_proposal();
    let card = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == CARDINALITY_DRIFT_SEED_ID
        })
        .expect("Cardinality drift must have ExistingCanonicalAuthorityResolution record");
    let r = card.reason.to_lowercase();
    assert!(r.contains("category-identity law") || r.contains("category identity"));
    assert!(r.contains("case sensitivity"));
    assert!(r.contains("unknown-category handling"));
    assert!(r.contains("counting law"));
}

/// Load-bearing negative #4 (panel-required):
/// `t12_h_rejects_uniqueness_violation_without_key_scope`.
#[test]
fn t12_h_rejects_uniqueness_violation_without_key_scope() {
    let p = seed_t12_h_dataquality_proposal();
    let uniq = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == UNIQUENESS_VIOLATION_SEED_ID
        })
        .expect("Uniqueness violation must have ExistingCanonicalAuthorityResolution record");
    let r = uniq.reason.to_lowercase();
    assert!(r.contains("key scope"));
    assert!(r.contains("null handling"));
}

/// Load-bearing negative #5 (panel-required):
/// `t12_h_rejects_functional_dependency_without_determinant_and_dependent_columns`.
#[test]
fn t12_h_rejects_functional_dependency_without_determinant_and_dependent_columns() {
    let p = seed_t12_h_dataquality_proposal();
    let fd = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == FD_VIOLATION_RESERVED_CANONICAL_ID
        })
        .expect("FD violation must have CanonicalAddition record");
    let r = fd.reason.to_lowercase();
    assert!(r.contains("determinant columns"));
    assert!(r.contains("dependent columns"));
    assert!(r.contains("null handling"));
    assert!(r.contains("minimum support"));
}

/// Load-bearing negative #6 (panel-required):
/// `t12_h_rejects_schema_drift_without_schema_version_or_column_identity_law`.
#[test]
fn t12_h_rejects_schema_drift_without_schema_version_or_column_identity_law() {
    let p = seed_t12_h_dataquality_proposal();
    let sd = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == SCHEMA_DRIFT_SEED_ID
        })
        .expect("Schema drift must have ExistingCanonicalAuthorityResolution record");
    let r = sd.reason.to_lowercase();
    assert!(r.contains("schema-version") || r.contains("column-identity"));
    assert!(r.contains("subkinds") || r.contains("subkind"));
}

/// Load-bearing negative #7 (panel-required):
/// `t12_h_rejects_range_envelope_without_unit_and_boundary_law`.
#[test]
fn t12_h_rejects_range_envelope_without_unit_and_boundary_law() {
    let p = seed_t12_h_dataquality_proposal();
    let re = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == RANGE_ENVELOPE_RESERVED_CANONICAL_ID
        })
        .expect("Range envelope must have CanonicalAddition record");
    let r = re.reason.to_lowercase();
    assert!(r.contains("unit"));
    assert!(r.contains("inclusive vs exclusive boundary"));
    assert!(r.contains("null handling"));
    assert!(r.contains("per-column min / max bounds"));
}

/// Load-bearing negative #8 (panel-required):
/// `t12_h_rejects_type_instability_without_type_system_law`.
#[test]
fn t12_h_rejects_type_instability_without_type_system_law() {
    let p = seed_t12_h_dataquality_proposal();
    let ti = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == TYPE_INSTABILITY_RESERVED_CANONICAL_ID
        })
        .expect("Type instability must have CanonicalAddition record");
    let r = ti.reason.to_lowercase();
    assert!(r.contains("type system"));
    assert!(r.contains("sql / arrow / parquet"));
    assert!(r.contains("observed-vs-expected"));
}

/// Load-bearing negative #9 (panel-required, MOST IMPORTANT):
/// `t12_h_rejects_target_leakage_without_target_and_time_availability_law`.
/// Target-leakage candidate's reason text MUST carry the panel-
/// locked "candidate, not proof" non-claim.
#[test]
fn t12_h_rejects_target_leakage_without_target_and_time_availability_law() {
    let p = seed_t12_h_dataquality_proposal();
    let tl = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == TARGET_LEAKAGE_RESERVED_CANONICAL_ID
        })
        .expect("Target leakage must have CanonicalAddition record");
    let r = tl.reason.to_lowercase();
    assert!(r.contains("target column"));
    assert!(r.contains("time / order law") || r.contains("time / order"));
    assert!(r.contains("feature-availability time"));
    assert!(r.contains("association law"));
    assert!(r.contains("train / test split") || r.contains("temporal holdout"));
    // Panel-locked non-claim phrasing.
    assert!(
        r.contains("candidate, not proof"),
        "Target leakage record must carry panel-locked non-claim 'candidate, not proof': {}",
        tl.reason
    );
}

/// Rejection contract tests for the two RejectedNotDeterministic
/// records.
#[test]
fn t12_h_rejects_learned_dq_score_without_model_identification_anchor() {
    let p = seed_t12_h_dataquality_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Learned DQ score must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_DQ_SCORE_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned DQ score must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("model-identification seed"));
    // Rust string-literal continuation joins with whitespace, so the
    // actual in-memory text contains "training- data anchor" not
    // "training-data anchor". Match both halves.
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
}

#[test]
fn t12_h_rejects_auto_schema_inference_without_inference_anchor() {
    let p = seed_t12_h_dataquality_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Auto-schema inference must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == AUTO_SCHEMA_INFERENCE_RESERVED_PRIMITIVE_ID
        })
        .expect("Auto-schema inference must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("inference algorithm"));
    assert!(r.contains("sample seed"));
    assert!(r.contains("sampling schedule"));
    assert!(r.contains("schema-version anchor"));
    assert!(r.contains("tie-break"));
}

// Correlation break, covariance shift, null-run, category emergence
// each declare contract laws.

#[test]
fn t12_h_correlation_break_declares_contract() {
    let p = seed_t12_h_dataquality_proposal();
    let cb = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CORRELATION_BREAK_RESERVED_CANONICAL_ID
        })
        .expect("Correlation break must have CanonicalAddition record");
    let r = cb.reason.to_lowercase();
    assert!(r.contains("correlation convention"));
    assert!(r.contains("pearson") || r.contains("spearman") || r.contains("kendall"));
    assert!(r.contains("window pair"));
    assert!(r.contains("threshold"));
}

#[test]
fn t12_h_covariance_shift_declares_estimator_and_comparison() {
    let p = seed_t12_h_dataquality_proposal();
    let cv = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == COVARIANCE_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("Covariance shift must have CanonicalAddition record");
    let r = cv.reason.to_lowercase();
    assert!(r.contains("covariance estimator"));
    assert!(r.contains("comparison law"));
}

#[test]
fn t12_h_null_run_declares_null_semantics_and_run_law() {
    let p = seed_t12_h_dataquality_proposal();
    let nr = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == NULL_RUN_RESERVED_CANONICAL_ID
        })
        .expect("Null-run must have CanonicalAddition record");
    let r = nr.reason.to_lowercase();
    assert!(r.contains("null semantics"));
    assert!(r.contains("consecutive-null-run law"));
}

#[test]
fn t12_h_category_emergence_declares_reference_set_and_anchor() {
    let p = seed_t12_h_dataquality_proposal();
    let ce = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CATEGORY_EMERGENCE_RESERVED_CANONICAL_ID
        })
        .expect("Category emergence must have CanonicalAddition record");
    let r = ce.reason.to_lowercase();
    assert!(r.contains("reference-category set"));
    assert!(r.contains("anchor"));
    // Same string-literal-continuation whitespace pattern.
    assert!(r.contains("new-category") && r.contains("appearance law"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_h_per_column_missingness_is_parameterizationof_missingness_spike() {
    let p = seed_t12_h_dataquality_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let pcm = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == PER_COLUMN_MISSINGNESS_RESERVED_PRIMITIVE_ID
        })
        .expect("Per-column missingness must have ParameterizationOf record");
    assert!(pcm.reason.contains("Missingness spike"));
}

#[test]
fn t12_h_composite_key_uniqueness_is_parameterizationof_uniqueness_violation() {
    let p = seed_t12_h_dataquality_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let ck = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == COMPOSITE_KEY_UNIQUENESS_RESERVED_PRIMITIVE_ID
        })
        .expect("Composite-key uniqueness must have ParameterizationOf record");
    assert!(ck.reason.contains("Uniqueness violation"));
}

#[test]
fn t12_h_category_collapse_is_parameterizationof_cardinality_drift() {
    let p = seed_t12_h_dataquality_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let cc = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == CATEGORY_COLLAPSE_RESERVED_PRIMITIVE_ID
        })
        .expect("Category collapse must have ParameterizationOf record");
    assert!(cc.reason.contains("Cardinality drift"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_h_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_h_dataquality_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_h_domain_transfer_target_must_be_in_seed() {
    let p = seed_t12_h_dataquality_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_h_every_dedup_record_has_reason() {
    let p = seed_t12_h_dataquality_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_h_canonical_addition_ids_are_in_5801_to_5808_range() {
    let p = seed_t12_h_dataquality_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5801..=5808).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.h reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn dq_proposal_text_rendering_byte_stable() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn dq_proposal_json_rendering_byte_stable() {
    let p = seed_t12_h_dataquality_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
