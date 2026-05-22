//! T.12.i acceptance suite — Observability / Debugging
//! expansion proposal invariants.
//!
//! Eleven panel-required load-bearing negatives pin the
//! telemetry-field + aggregation-law + topology-scope +
//! baseline + decision-law + confuser-semantics discipline
//! T.12.i exists to prove ("an observability symptom is not
//! a detector until the telemetry field, aggregation law,
//! baseline, topology scope, and confuser semantics are
//! declared").
//!
//! Most-important load-bearing negative:
//! `t12_i_rejects_vendor_apm_score_without_deterministic_formula`
//! — vendor APM products expose anomaly scores without stable
//! public decision functionals. The court must NOT launder
//! those as canonical witnesses.

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
use dsfb_gpu_atlas_corpus::t12_i_observability::{
    seed_t12_i_observability_proposal, BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, COLD_START_TRANSIENT_RESERVED_CANONICAL_ID,
    ERROR_BURST_SEED_ID, FANOUT_CASCADE_SEED_ID, GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID,
    HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID, K_HOP_FANOUT_RESERVED_PRIMITIVE_ID, LATENCY_RAMP_SEED_ID,
    LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID, QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID,
    QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID, RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID,
    RETRY_STORM_RESERVED_CANONICAL_ID, SATURATION_PRECURSOR_RESERVED_CANONICAL_ID,
    SINGLE_WINDOW_SPIKE_CONFUSER_SEED_ID, SLEW_SHOCK_SEED_ID,
    THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID, TIMEOUT_BURST_RESERVED_CANONICAL_ID,
    VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

const T12I_AUTHORITY_RESOLVED_SEED_IDS: &[(u32, &str)] = &[
    (LATENCY_RAMP_SEED_ID, "Latency ramp"),
    (
        SINGLE_WINDOW_SPIKE_CONFUSER_SEED_ID,
        "Single-window spike confuser",
    ),
    (ERROR_BURST_SEED_ID, "Error burst"),
    (SLEW_SHOCK_SEED_ID, "Slew shock"),
    (FANOUT_CASCADE_SEED_ID, "Fanout cascade"),
];

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn obs_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_i_observability_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Observability proposal failed verifier: {errors:?}"
    );
}

#[test]
fn obs_proposal_has_open_status() {
    let p = seed_t12_i_observability_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn obs_proposal_targets_observability_debugging() {
    let p = seed_t12_i_observability_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::ObservabilityDebugging
    ));
}

/// Load-bearing negative #1 (panel-required):
/// `t12_i_does_not_mutate_seed_len`.
#[test]
fn t12_i_does_not_mutate_seed_len() {
    let _ = seed_t12_i_observability_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn obs_proposal_proposes_fourteen_primitives() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 14);
}

#[test]
fn obs_proposal_proposes_zero_aliases() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn obs_proposal_proposes_twentyone_dedup_records() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 21);
}

#[test]
fn obs_proposal_proposes_twelve_genealogy_edges() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 12);
}

#[test]
fn obs_proposal_proposes_nine_source_refs() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 9);
}

#[test]
fn obs_delta_has_eight_new_canonical_records() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn obs_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_i_observability_proposal();
    let categories: std::collections::BTreeSet<&str> = p
        .body
        .proposed_dedup_records
        .iter()
        .map(|r| r.decision_wire_name)
        .collect();
    assert_eq!(categories.len(), 5);
}

#[test]
fn obs_proposal_carries_two_rejection_records() {
    let p = seed_t12_i_observability_proposal();
    let count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        count, 2,
        "T.12.i must carry TWO RejectedNotDeterministic records \
        (third T.12.x with two, following T.12.g and T.12.h)"
    );
}

#[test]
fn obs_proposal_court_delta_category_counts() {
    let p = seed_t12_i_observability_proposal();
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
    assert_eq!(transfer, 2);
    assert_eq!(paramof, 4);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn obs_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_i_observability_proposal();
    let b = seed_t12_i_observability_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_obs_proposal_hash_matches_stored() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn obs_proposal_hash_is_distinct_from_all_prior_t12x() {
    let i = seed_t12_i_observability_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    let ts = seed_t12_f_timeseries_proposal();
    let graph = seed_t12_g_graph_proposal();
    let dq = seed_t12_h_dataquality_proposal();
    for other in [pol, spc, scd, drift, robust, spectral, ts, graph, dq] {
        assert_ne!(
            i.corpus_amendment_proposal_hash_v1, other.corpus_amendment_proposal_hash_v1,
            "T.12.i hash must differ from every prior T.12.x"
        );
    }
}

/// Load-bearing negative #10 (panel-required):
/// `t12_i_hash_changes_when_telemetry_field_law_changes`.
#[test]
fn t12_i_hash_changes_when_telemetry_field_law_changes() {
    let p_a = seed_t12_i_observability_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == RETRY_STORM_RESERVED_CANONICAL_ID
        })
        .expect("Retry storm CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED telemetry-field law for hash-sensitivity test",
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
// SEED collision load-bearing negative #11
// ---------------------------------------------------------------

fn build_defective_collision_proposal(
    seed_id: u32,
) -> dsfb_gpu_atlas_corpus::amendment::CorpusAmendmentProposal {
    let bad_batch = build_expansion_batch(
        "t12_i_collision_batch",
        SourceClass::ObservabilityDebugging,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "obs collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_i_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_i_collision_proposal",
        "Defective T.12.i proposal duplicating an existing SEED canonical.",
        SourceClass::ObservabilityDebugging,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_i_test",
    )
}

/// Load-bearing negative #11 (panel-required):
/// `t12_i_existing_seed_collision_requires_authority_resolution`.
/// Most-important variant: re-adding any of the dsfb-gpu-debug
/// bank surface IDs (14, 15, 41, 42, 43) as a NEW canonical
/// must be rejected. That is what protects the L6 honesty
/// marker.
#[test]
fn t12_i_existing_seed_collision_requires_authority_resolution() {
    for (seed_id, label) in T12I_AUTHORITY_RESOLVED_SEED_IDS {
        let p = build_defective_collision_proposal(*seed_id);
        let errors = verify_amendment_proposal(&p);
        assert!(
            errors.iter().any(|e| matches!(
                e.kind,
                AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId {
                    canonical_id
                } if canonical_id.0 == *seed_id
            )),
            "expected collision rule fire for SEED id {seed_id} ({label}) — \
            re-adding the dsfb-gpu-debug bank surface as a NEW canonical must be \
            rejected"
        );
    }
}

/// Load-bearing negative #2 (panel-required):
/// `t12_i_rejects_latency_ramp_duplicate_without_existing_authority_resolution`.
/// The SEED 14 Latency ramp ExistingCanonicalAuthorityResolution
/// record must declare its telemetry-field + aggregation +
/// confuser contract.
#[test]
fn t12_i_rejects_latency_ramp_duplicate_without_existing_authority_resolution() {
    let p = seed_t12_i_observability_proposal();
    let lat = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION
                && r.canonical_id.0 == LATENCY_RAMP_SEED_ID
        })
        .expect("Latency ramp must have ExistingCanonicalAuthorityResolution record");
    let r = lat.reason.to_lowercase();
    assert!(r.contains("telemetry field"));
    assert!(r.contains("aggregation law"));
    assert!(r.contains("window pair") || r.contains("window"));
    assert!(r.contains("baseline") || r.contains("threshold"));
    assert!(r.contains("confuser") || r.contains("single-window spike"));
    // Bank-surface marker.
    assert!(r.contains("l6 dsfb-gpu-debug bank surface"));
}

/// Load-bearing negative #3 (panel-required):
/// `t12_i_rejects_retry_storm_without_retry_field_and_window_law`.
#[test]
fn t12_i_rejects_retry_storm_without_retry_field_and_window_law() {
    let p = seed_t12_i_observability_proposal();
    let rs = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == RETRY_STORM_RESERVED_CANONICAL_ID
        })
        .expect("Retry storm must have CanonicalAddition record");
    let r = rs.reason.to_lowercase();
    assert!(r.contains("retry-event field"));
    assert!(r.contains("counting law"));
    assert!(r.contains("window"));
    assert!(r.contains("threshold"));
    assert!(r.contains("confuser profile"));
}

/// Load-bearing negative #4 (panel-required):
/// `t12_i_rejects_queue_pressure_without_queue_depth_metric_and_capacity_law`.
#[test]
fn t12_i_rejects_queue_pressure_without_queue_depth_metric_and_capacity_law() {
    let p = seed_t12_i_observability_proposal();
    let qp = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == QUEUE_DEPTH_PRESSURE_RESERVED_CANONICAL_ID
        })
        .expect("Queue-depth pressure must have CanonicalAddition record");
    let r = qp.reason.to_lowercase();
    assert!(r.contains("queue-depth metric source"));
    assert!(r.contains("capacity contract"));
    assert!(r.contains("aggregation law"));
    assert!(r.contains("threshold"));
}

/// Load-bearing negative #5 (panel-required):
/// `t12_i_rejects_saturation_precursor_without_resource_capacity_contract`.
#[test]
fn t12_i_rejects_saturation_precursor_without_resource_capacity_contract() {
    let p = seed_t12_i_observability_proposal();
    let sp = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == SATURATION_PRECURSOR_RESERVED_CANONICAL_ID
        })
        .expect("Saturation precursor must have CanonicalAddition record");
    let r = sp.reason.to_lowercase();
    assert!(r.contains("resource capacity contract"));
    assert!(r.contains("utilisation aggregation"));
    assert!(r.contains("slope or threshold law"));
    // USE method discipline must appear: utilisation / saturation / error.
    assert!(r.contains("use method"));
}

/// Load-bearing negative #6 (panel-required):
/// `t12_i_rejects_gc_pause_without_runtime_and_pause_metric_law`.
#[test]
fn t12_i_rejects_gc_pause_without_runtime_and_pause_metric_law() {
    let p = seed_t12_i_observability_proposal();
    let gc = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == GC_PAUSE_SPIKE_RESERVED_CANONICAL_ID
        })
        .expect("GC pause spike must have CanonicalAddition record");
    let r = gc.reason.to_lowercase();
    assert!(r.contains("language runtime"));
    assert!(r.contains("gc pause-duration metric"));
    assert!(r.contains("quantile or max aggregation law"));
    assert!(r.contains("threshold"));
    assert!(r.contains("full-gc vs minor-gc"));
}

/// Load-bearing negative #7 (panel-required):
/// `t12_i_rejects_fanout_variant_without_topology_scope_and_hop_law`.
/// Applied to the k-hop dependency fanout ParameterizationOf
/// record.
#[test]
fn t12_i_rejects_fanout_variant_without_topology_scope_and_hop_law() {
    let p = seed_t12_i_observability_proposal();
    let kh = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == K_HOP_FANOUT_RESERVED_PRIMITIVE_ID
        })
        .expect("k-hop dependency fanout must have ParameterizationOf record");
    let r = kh.reason.to_lowercase();
    assert!(r.contains("topology scope"));
    assert!(r.contains("hop limit"));
    assert!(r.contains("dependency graph"));
    assert!(r.contains("fanout cascade"));
}

/// Load-bearing negative #8 (panel-required):
/// `t12_i_rejects_cold_start_without_deployment_marker_or_warmup_law`.
#[test]
fn t12_i_rejects_cold_start_without_deployment_marker_or_warmup_law() {
    let p = seed_t12_i_observability_proposal();
    let cs = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == COLD_START_TRANSIENT_RESERVED_CANONICAL_ID
        })
        .expect("Cold-start transient must have CanonicalAddition record");
    let r = cs.reason.to_lowercase();
    assert!(r.contains("deployment / warm-up marker") || r.contains("warm-up marker"));
    assert!(r.contains("warmup window"));
    assert!(r.contains("suppression law"));
    assert!(r.contains("decision law"));
}

/// Load-bearing negative #9 (panel-required, MOST IMPORTANT):
/// `t12_i_rejects_vendor_apm_score_without_deterministic_formula`.
/// Vendor APM scores must NOT be admitted as canonical without
/// the full contract. The reason text must explicitly require
/// "deterministic formula" or equivalent.
#[test]
fn t12_i_rejects_vendor_apm_score_without_deterministic_formula() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Vendor APM score must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == VENDOR_APM_SCORE_RESERVED_PRIMITIVE_ID
        })
        .expect("Vendor APM score must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    // Panel-locked: must require a deterministic formula.
    assert!(
        r.contains("deterministic formula"),
        "Vendor APM score rejection must require 'deterministic formula': {}",
        rec.reason
    );
    assert!(r.contains("model-identification anchor"));
    // Rust string-literal continuation joins with whitespace, so
    // the actual in-memory text contains "training- data anchor"
    // not "training-data anchor". Match both halves.
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("feature schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    // At least one named vendor must appear so the rejection is
    // concretely grounded.
    let vendors = ["datadog", "new relic", "dynatrace", "splunk", "devops guru"];
    let has_vendor = vendors.iter().any(|v| r.contains(v));
    assert!(
        has_vendor,
        "vendor APM rejection must name at least one vendor"
    );
}

/// Rejection contract test #2: learned incident classifier.
#[test]
fn t12_i_rejects_learned_incident_classifier_without_model_identification() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Learned incident classifier must NOT be in new_canonical_records"
    );
    let rec = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == LEARNED_INCIDENT_CLASSIFIER_RESERVED_PRIMITIVE_ID
        })
        .expect("Learned incident classifier must have RejectedNotDeterministic record");
    let r = rec.reason.to_lowercase();
    assert!(r.contains("model-identification seed"));
    assert!(r.contains("training-") && r.contains("data anchor"));
    assert!(r.contains("label schema"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("numeric mode"));
    // Must name at least one vendor / product.
    let products = ["pagerduty", "splunk on-call", "servicenow"];
    let has_product = products.iter().any(|v| r.contains(v));
    assert!(
        has_product,
        "learned classifier rejection must name at least one product"
    );
}

// ---------------------------------------------------------------
// Per-canonical / additional contract assertions
// ---------------------------------------------------------------

#[test]
fn t12_i_timeout_burst_is_structurally_distinct_from_error_burst() {
    let p = seed_t12_i_observability_proposal();
    let tb = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == TIMEOUT_BURST_RESERVED_CANONICAL_ID
        })
        .expect("Timeout burst must have CanonicalAddition record");
    let r = tb.reason.to_lowercase();
    assert!(r.contains("timeout-event field"));
    assert!(r.contains("specific failure class"));
    // Must reference Error burst structurally to declare the
    // distinction.
    assert!(r.contains("error burst") || r.contains("seed 41"));
}

#[test]
fn t12_i_thread_pool_exhaustion_is_derivedfrom_saturation_precursor() {
    let p = seed_t12_i_observability_proposal();
    let tp = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == THREAD_POOL_EXHAUSTION_RESERVED_CANONICAL_ID
        })
        .expect("Thread-pool exhaustion must have CanonicalAddition record");
    let r = tp.reason.to_lowercase();
    assert!(r.contains("derivedfrom(saturation precursor"));
    assert!(r.contains("pool source"));
    assert!(r.contains("pool-capacity contract"));
}

#[test]
fn t12_i_backpressure_is_derivedfrom_fanout_cascade() {
    let p = seed_t12_i_observability_proposal();
    let bp = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BACKPRESSURE_PROPAGATION_RESERVED_CANONICAL_ID
        })
        .expect("Backpressure propagation must have CanonicalAddition record");
    let r = bp.reason.to_lowercase();
    assert!(r.contains("derivedfrom(fanout cascade"));
    assert!(r.contains("flow-control signal"));
    assert!(r.contains("propagation law"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_i_http_5xx_burst_is_parameterizationof_error_burst() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let h = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == HTTP_5XX_BURST_RESERVED_PRIMITIVE_ID
        })
        .expect("HTTP 5xx burst must have ParameterizationOf record");
    assert!(h.reason.contains("Error burst"));
    assert!(h.reason.to_lowercase().contains("5xx"));
}

#[test]
fn t12_i_quantile_latency_ramp_is_parameterizationof_latency_ramp() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let q = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == QUANTILE_LATENCY_RAMP_RESERVED_PRIMITIVE_ID
        })
        .expect("p95 / p99 latency ramp must have ParameterizationOf record");
    assert!(q.reason.contains("Latency ramp"));
    let r = q.reason.to_lowercase();
    assert!(r.contains("p95") || r.contains("p99"));
}

#[test]
fn t12_i_k_hop_fanout_is_parameterizationof_fanout_cascade() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == K_HOP_FANOUT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let k = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == K_HOP_FANOUT_RESERVED_PRIMITIVE_ID
        })
        .expect("k-hop dependency fanout must have ParameterizationOf record");
    assert!(k.reason.contains("Fanout cascade"));
}

#[test]
fn t12_i_retry_rate_burst_is_parameterizationof_retry_storm() {
    let p = seed_t12_i_observability_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let r = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|x| {
            x.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && x.canonical_id.0 == RETRY_RATE_BURST_RESERVED_PRIMITIVE_ID
        })
        .expect("Retry-rate burst must have ParameterizationOf record");
    assert!(r.reason.contains("Retry storm"));
}

// ---------------------------------------------------------------
// DomainTransferOf invariants
// ---------------------------------------------------------------

#[test]
fn t12_i_fanout_cascade_domain_transfer_to_observability_exists() {
    let p = seed_t12_i_observability_proposal();
    let dt = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == FANOUT_CASCADE_SEED_ID
        })
        .expect("Fanout cascade must have DomainTransferOf record");
    let r = dt.reason.to_lowercase();
    assert!(r.contains("shared cascade ancestor"));
    assert!(r.contains("observabilitydebugging"));
}

#[test]
fn t12_i_error_burst_domain_transfer_to_observability_exists() {
    let p = seed_t12_i_observability_proposal();
    let dt = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF
                && r.canonical_id.0 == ERROR_BURST_SEED_ID
        })
        .expect("Error burst must have DomainTransferOf record");
    let r = dt.reason.to_lowercase();
    assert!(r.contains("shared rate-burst ancestor"));
    assert!(r.contains("observabilitydebugging"));
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_i_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_i_observability_proposal();
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
fn t12_i_domain_transfer_targets_must_be_in_seed() {
    let p = seed_t12_i_observability_proposal();
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
fn t12_i_authority_resolution_covers_all_bank_surface_ids() {
    // The dsfb-gpu-debug bank surface IDs are 14, 15, 41, 42, 43.
    // T.12.i MUST authority-resolve every one of them.
    let p = seed_t12_i_observability_proposal();
    let resolved: std::collections::BTreeSet<u32> = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION)
        .map(|r| r.canonical_id.0)
        .collect();
    for (id, label) in T12I_AUTHORITY_RESOLVED_SEED_IDS {
        assert!(
            resolved.contains(id),
            "SEED id {id} ({label}) must be authority-resolved by T.12.i"
        );
    }
}

#[test]
fn t12_i_every_dedup_record_has_reason() {
    let p = seed_t12_i_observability_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_i_canonical_addition_ids_are_in_5901_to_5908_range() {
    let p = seed_t12_i_observability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5901..=5908).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.i reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_i_parameterization_ids_are_in_5909_to_5912_range() {
    let p = seed_t12_i_observability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF {
            assert!(
                (5909..=5912).contains(&r.canonical_id.0),
                "ParameterizationOf record id {} outside T.12.i reserved range",
                r.canonical_id.0
            );
        }
    }
}

#[test]
fn t12_i_rejection_ids_are_in_5913_to_5914_range() {
    let p = seed_t12_i_observability_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC {
            assert!(
                (5913..=5914).contains(&r.canonical_id.0),
                "RejectedNotDeterministic record id {} outside T.12.i reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn obs_proposal_text_rendering_byte_stable() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn obs_proposal_json_rendering_byte_stable() {
    let p = seed_t12_i_observability_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
