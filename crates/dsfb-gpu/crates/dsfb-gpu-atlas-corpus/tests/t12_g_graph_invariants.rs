//! T.12.g acceptance suite — Graph / Topology Anomaly
//! expansion proposal invariants.
//!
//! Ten panel-required load-bearing negatives pin the graph-
//! model + baseline + update-law + metric-law + decision-law
//! discipline T.12.g exists to prove ("a graph metric is not a
//! detector until the baseline, update law, metric law, and
//! decision law are declared").
//!
//! T.12.g is the FIRST T.12.x proposal to exercise TWO
//! RejectedNotDeterministic records in one commit (community
//! boundary shift + random-walk embedding anomaly).

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
use dsfb_gpu_atlas_corpus::t12_g_graph::{
    seed_t12_g_graph_proposal, BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID,
    BRIDGE_NODE_RESERVED_CANONICAL_ID, CASCADE_PRECURSOR_RESERVED_CANONICAL_ID,
    CATEGORY_CANONICAL_ADDITION, CATEGORY_DOMAIN_TRANSFER_OF,
    CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION, CATEGORY_PARAMETERIZATION_OF,
    CATEGORY_REJECTED_NOT_DETERMINISTIC, CLUSTERING_SHIFT_RESERVED_CANONICAL_ID,
    COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID, DEGREE_SPIKE_RESERVED_CANONICAL_ID,
    DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID, EDGE_CUT_RESERVED_CANONICAL_ID,
    FANOUT_CASCADE_SEED_ID, K_HOP_FANOUT_RESERVED_PRIMITIVE_ID, MOTIF_COUNT_RESERVED_CANONICAL_ID,
    PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID, RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID,
    WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID,
};
use dsfb_gpu_atlas_corpus::types::{DetectorAliasId, DetectorCanonicalId};

// ---------------------------------------------------------------
// Shape + admissibility
// ---------------------------------------------------------------

#[test]
fn graph_proposal_is_admissible_under_t12_0_verifier() {
    let p = seed_t12_g_graph_proposal();
    let errors = verify_amendment_proposal(&p);
    assert!(
        errors.is_empty(),
        "Graph proposal failed verifier: {errors:?}"
    );
}

#[test]
fn graph_proposal_has_open_status() {
    let p = seed_t12_g_graph_proposal();
    assert!(matches!(p.status, ProposalStatus::Open));
}

#[test]
fn graph_proposal_targets_graph_anomaly_detection() {
    let p = seed_t12_g_graph_proposal();
    assert!(matches!(
        p.target_source_class,
        SourceClass::GraphAnomalyDetection
    ));
}

/// Load-bearing negative #1 (panel-required).
#[test]
fn t12_g_does_not_mutate_seed_len() {
    let _ = seed_t12_g_graph_proposal();
    assert_eq!(SEED.len(), 54);
}

#[test]
fn graph_proposal_proposes_thirteen_primitives() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.body.proposed_primitives.len(), 13);
}

#[test]
fn graph_proposal_proposes_zero_aliases() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.body.proposed_aliases.len(), 0);
}

#[test]
fn graph_proposal_proposes_fifteen_dedup_records() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.body.proposed_dedup_records.len(), 15);
}

#[test]
fn graph_proposal_proposes_four_genealogy_edges() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.body.proposed_genealogy_edges.len(), 4);
}

#[test]
fn graph_proposal_proposes_ten_source_refs() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.body.proposed_source_refs.len(), 10);
}

#[test]
fn graph_delta_has_eight_new_canonical_records() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(p.dedup_court_delta.new_canonical_records.len(), 8);
}

#[test]
fn graph_proposal_emits_all_five_court_delta_categories() {
    let p = seed_t12_g_graph_proposal();
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

/// T.12.g is the FIRST proposal to carry TWO
/// RejectedNotDeterministic records in one commit.
#[test]
fn graph_proposal_carries_two_rejection_records() {
    let p = seed_t12_g_graph_proposal();
    let rejection_count = p
        .body
        .proposed_dedup_records
        .iter()
        .filter(|r| r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC)
        .count();
    assert_eq!(
        rejection_count, 2,
        "T.12.g must carry exactly TWO RejectedNotDeterministic records"
    );
}

#[test]
fn graph_proposal_court_delta_category_counts() {
    let p = seed_t12_g_graph_proposal();
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
    assert_eq!(existing, 1);
    assert_eq!(transfer, 1);
    assert_eq!(paramof, 3);
    assert_eq!(rejected, 2);
}

// ---------------------------------------------------------------
// Hash determinism + sensitivity
// ---------------------------------------------------------------

#[test]
fn graph_proposal_hash_is_deterministic_across_two_builds() {
    let a = seed_t12_g_graph_proposal();
    let b = seed_t12_g_graph_proposal();
    assert_eq!(
        a.corpus_amendment_proposal_hash_v1,
        b.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn recomputed_graph_proposal_hash_matches_stored() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(
        compute_corpus_amendment_proposal_hash_v1(&p),
        p.corpus_amendment_proposal_hash_v1
    );
}

#[test]
fn graph_proposal_hash_is_distinct_from_t12_0_a_b_c_d_e_f() {
    let g = seed_t12_g_graph_proposal();
    let pol = seed_proof_of_life_proposal();
    let spc = seed_t12_a_spc_proposal();
    let scd = seed_t12_b_scd_proposal();
    let drift = seed_t12_c_drift_proposal();
    let robust = seed_t12_d_robust_proposal();
    let spectral = seed_t12_e_spectral_proposal();
    let ts = seed_t12_f_timeseries_proposal();
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        pol.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        spc.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        scd.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        drift.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        robust.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        spectral.corpus_amendment_proposal_hash_v1
    );
    assert_ne!(
        g.corpus_amendment_proposal_hash_v1,
        ts.corpus_amendment_proposal_hash_v1
    );
}

/// Load-bearing negative #9 (panel-required):
/// `t12_g_hash_changes_when_graph_metric_law_changes`.
#[test]
fn t12_g_hash_changes_when_graph_metric_law_changes() {
    let p_a = seed_t12_g_graph_proposal();
    let mut records = p_a.body.proposed_dedup_records.clone();
    let idx = records
        .iter()
        .position(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("PageRank residual CanonicalAddition record must exist");
    records[idx] = ProposedDedupRecord {
        decision_wire_name: records[idx].decision_wire_name,
        canonical_id: records[idx].canonical_id,
        reason: "MUTATED graph-metric law for hash-sensitivity test",
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
        "t12_g_collision_batch",
        SourceClass::GraphAnomalyDetection,
        vec![ProposedPrimitive {
            reserved_canonical_id: DetectorCanonicalId(seed_id),
            display_name: "graph collision-test",
            motivation: "Should be rejected - id collides with existing SEED canonical.",
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let bad_delta = build_dedup_court_delta(
        "t12_g_collision_delta",
        vec![DetectorCanonicalId(seed_id)],
        Vec::new(),
        Vec::new(),
        Vec::<RejectionRecord>::new(),
        Vec::<DetectorAliasId>::new(),
    );
    build_amendment_proposal(
        "t12_g_collision_proposal",
        "Defective T.12.g-style proposal duplicating an existing SEED canonical.",
        SourceClass::GraphAnomalyDetection,
        bad_batch,
        bad_delta,
        ProposalStatus::Open,
        ProposerRole::PanelMember,
        "t12_g_test",
    )
}

/// Load-bearing negative (panel-required):
/// `t12_g_existing_seed_collision_requires_authority_resolution`.
/// Only one SEED record relevant to T.12.g (Fanout cascade 43).
#[test]
fn t12_g_existing_seed_collision_requires_authority_resolution() {
    let p = build_defective_collision_proposal(FANOUT_CASCADE_SEED_ID);
    let errors = verify_amendment_proposal(&p);
    assert!(errors.iter().any(|e| matches!(
        e.kind,
        AmendmentVerifyErrorKind::DedupDeltaCollidesWithExistingSeedCanonicalId { canonical_id }
            if canonical_id.0 == FANOUT_CASCADE_SEED_ID
    )));
}

// ---------------------------------------------------------------
// Graph-contract load-bearing negatives (panel-required)
// ---------------------------------------------------------------

/// Load-bearing negative #2 (panel-required):
/// `t12_g_rejects_degree_spike_without_baseline_and_graph_update_law`.
#[test]
fn t12_g_rejects_degree_spike_without_baseline_and_graph_update_law() {
    let p = seed_t12_g_graph_proposal();
    let degree_spike = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == DEGREE_SPIKE_RESERVED_CANONICAL_ID
        })
        .expect("Degree spike must have CanonicalAddition record");
    let r = degree_spike.reason.to_lowercase();
    assert!(r.contains("graph type"));
    assert!(r.contains("node identity law"));
    assert!(r.contains("edge identity law"));
    assert!(r.contains("baseline"));
    assert!(r.contains("threshold"));
}

/// Load-bearing negative #3 (panel-required):
/// `t12_g_rejects_betweenness_shift_without_shortest_path_and_normalization_law`.
#[test]
fn t12_g_rejects_betweenness_shift_without_shortest_path_and_normalization_law() {
    let p = seed_t12_g_graph_proposal();
    let betweenness = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BETWEENNESS_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("Betweenness shift must have CanonicalAddition record");
    let r = betweenness.reason.to_lowercase();
    assert!(r.contains("shortest-path law") || r.contains("shortest path"));
    assert!(r.contains("weighted") && r.contains("unweighted"));
    assert!(r.contains("normalization"));
    assert!(r.contains("tie-handling") || r.contains("tie handling"));
}

/// Load-bearing negative #4 (panel-required):
/// `t12_g_rejects_pagerank_residual_without_damping_and_dangling_node_law`.
#[test]
fn t12_g_rejects_pagerank_residual_without_damping_and_dangling_node_law() {
    let p = seed_t12_g_graph_proposal();
    let pagerank = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == PAGERANK_RESIDUAL_RESERVED_CANONICAL_ID
        })
        .expect("PageRank residual must have CanonicalAddition record");
    let r = pagerank.reason.to_lowercase();
    assert!(r.contains("damping factor"));
    assert!(r.contains("iteration count") || r.contains("convergence threshold"));
    assert!(r.contains("dangling-node handling") || r.contains("dangling node"));
}

/// Load-bearing negative #5 (panel-required, MOST IMPORTANT):
/// `t12_g_rejects_community_shift_without_deterministic_partition_law`.
#[test]
fn t12_g_rejects_community_shift_without_deterministic_partition_law() {
    let p = seed_t12_g_graph_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Community boundary shift must NOT be in new_canonical_records"
    );
    let community = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == COMMUNITY_BOUNDARY_RESERVED_PRIMITIVE_ID
        })
        .expect("Community boundary shift must have RejectedNotDeterministic record");
    let r = community.reason.to_lowercase();
    assert!(r.contains("algorithm"));
    assert!(r.contains("sample seed") || r.contains("seed"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("modularity rule"));
    assert!(r.contains("resolution parameter"));
    assert!(r.contains("convergence law"));
}

/// Load-bearing negative #6 (panel-required):
/// `t12_g_rejects_motif_count_without_motif_enumeration_law`.
#[test]
fn t12_g_rejects_motif_count_without_motif_enumeration_law() {
    let p = seed_t12_g_graph_proposal();
    let motif = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == MOTIF_COUNT_RESERVED_CANONICAL_ID
        })
        .expect("Motif-count anomaly must have CanonicalAddition record");
    let r = motif.reason.to_lowercase();
    assert!(r.contains("motif definition"));
    assert!(r.contains("motif-enumeration law") || r.contains("enumeration law"));
    assert!(r.contains("count normalization"));
}

/// Load-bearing negative #7 (panel-required):
/// `t12_g_rejects_cascade_precursor_without_temporal_edge_order_law`.
#[test]
fn t12_g_rejects_cascade_precursor_without_temporal_edge_order_law() {
    let p = seed_t12_g_graph_proposal();
    let cascade = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CASCADE_PRECURSOR_RESERVED_CANONICAL_ID
        })
        .expect("Cascade precursor must have CanonicalAddition record");
    let r = cascade.reason.to_lowercase();
    assert!(r.contains("temporal-edge-ordering law") || r.contains("temporal-edge-ordering"));
    assert!(r.contains("causal-adjacency"));
    assert!(r.contains("hop limit"));
    assert!(r.contains("clock-skew"));
}

/// Load-bearing negative #8 (panel-required):
/// `t12_g_rejects_bridge_node_without_connectivity_definition`.
#[test]
fn t12_g_rejects_bridge_node_without_connectivity_definition() {
    let p = seed_t12_g_graph_proposal();
    let bridge = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == BRIDGE_NODE_RESERVED_CANONICAL_ID
        })
        .expect("Bridge-node emergence must have CanonicalAddition record");
    let r = bridge.reason.to_lowercase();
    assert!(r.contains("connectivity definition"));
    assert!(r.contains("dfs-based articulation"));
    assert!(r.contains("tie-break"));
}

/// Random-walk embedding rejection contract.
#[test]
fn t12_g_rejects_random_walk_embedding_without_walk_seed_and_anchor() {
    let p = seed_t12_g_graph_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID);
    assert!(
        !in_canonical,
        "Random-walk embedding anomaly must NOT be in new_canonical_records"
    );
    let rwe = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_REJECTED_NOT_DETERMINISTIC
                && r.canonical_id.0 == RANDOM_WALK_EMBEDDING_RESERVED_PRIMITIVE_ID
        })
        .expect("Random-walk embedding must have RejectedNotDeterministic record");
    let r = rwe.reason.to_lowercase();
    assert!(r.contains("walk seed"));
    assert!(r.contains("walk length"));
    assert!(r.contains("walk count"));
    assert!(r.contains("tie-break"));
    assert!(r.contains("embedding-projection matrix anchor") || r.contains("matrix anchor"));
    assert!(r.contains("numeric mode"));
}

/// Edge-cut and clustering canonicals also declare their
/// metric / cut law in reason text.
#[test]
fn t12_g_edge_cut_and_clustering_declare_contract() {
    let p = seed_t12_g_graph_proposal();
    let edge_cut = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == EDGE_CUT_RESERVED_CANONICAL_ID
        })
        .expect("Edge-cut must have CanonicalAddition record");
    let r = edge_cut.reason.to_lowercase();
    assert!(r.contains("cut algorithm"));
    assert!(r.contains("baseline cut value") || r.contains("baseline"));
    assert!(r.contains("threshold"));

    let clustering = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_CANONICAL_ADDITION
                && r.canonical_id.0 == CLUSTERING_SHIFT_RESERVED_CANONICAL_ID
        })
        .expect("Clustering-coefficient shift must have CanonicalAddition record");
    let r2 = clustering.reason.to_lowercase();
    assert!(r2.contains("triangle-count law"));
    assert!(r2.contains("normalization"));
    assert!(r2.contains("baseline"));
}

// ---------------------------------------------------------------
// ParameterizationOf family invariants
// ---------------------------------------------------------------

#[test]
fn t12_g_weighted_degree_is_parameterizationof_degree_spike() {
    let p = seed_t12_g_graph_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let wd = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == WEIGHTED_DEGREE_SPIKE_RESERVED_PRIMITIVE_ID
        })
        .expect("Weighted-degree spike must have ParameterizationOf record");
    assert!(
        wd.reason.contains("Degree spike"),
        "Weighted-degree spike must reference Degree spike parent: {}",
        wd.reason
    );
}

#[test]
fn t12_g_k_hop_fanout_is_parameterizationof_fanout_cascade() {
    let p = seed_t12_g_graph_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == K_HOP_FANOUT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let kh = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == K_HOP_FANOUT_RESERVED_PRIMITIVE_ID
        })
        .expect("k-hop fanout must have ParameterizationOf record");
    assert!(
        kh.reason.contains("Fanout cascade"),
        "k-hop fanout must reference Fanout cascade parent: {}",
        kh.reason
    );
}

#[test]
fn t12_g_directed_motif_is_parameterizationof_motif_count() {
    let p = seed_t12_g_graph_proposal();
    let in_canonical = p
        .dedup_court_delta
        .new_canonical_records
        .iter()
        .any(|c| c.0 == DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID);
    assert!(!in_canonical);
    let dm = p
        .body
        .proposed_dedup_records
        .iter()
        .find(|r| {
            r.decision_wire_name == CATEGORY_PARAMETERIZATION_OF
                && r.canonical_id.0 == DIRECTED_MOTIF_COUNT_RESERVED_PRIMITIVE_ID
        })
        .expect("Directed motif-count must have ParameterizationOf record");
    assert!(
        dm.reason.contains("Motif-count anomaly"),
        "Directed motif-count must reference Motif-count anomaly parent: {}",
        dm.reason
    );
}

// ---------------------------------------------------------------
// Other invariants
// ---------------------------------------------------------------

#[test]
fn t12_g_existing_canonical_authority_resolution_targets_must_be_in_seed() {
    let p = seed_t12_g_graph_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_EXISTING_CANONICAL_AUTHORITY_RESOLUTION {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_g_domain_transfer_target_must_be_in_seed() {
    let p = seed_t12_g_graph_proposal();
    let seed_ids: std::collections::BTreeSet<u32> = SEED.iter().map(|r| r.canonical_id.0).collect();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_DOMAIN_TRANSFER_OF {
            assert!(seed_ids.contains(&r.canonical_id.0));
        }
    }
}

#[test]
fn t12_g_every_dedup_record_has_reason() {
    let p = seed_t12_g_graph_proposal();
    for r in &p.body.proposed_dedup_records {
        assert!(!r.reason.is_empty());
    }
}

#[test]
fn t12_g_canonical_addition_ids_are_in_5701_to_5708_range() {
    let p = seed_t12_g_graph_proposal();
    for r in &p.body.proposed_dedup_records {
        if r.decision_wire_name == CATEGORY_CANONICAL_ADDITION {
            assert!(
                (5701..=5708).contains(&r.canonical_id.0),
                "CanonicalAddition record id {} outside T.12.g reserved range",
                r.canonical_id.0
            );
        }
    }
}

// ---------------------------------------------------------------
// Rendering byte-stability
// ---------------------------------------------------------------

#[test]
fn graph_proposal_text_rendering_byte_stable() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(
        render_amendment_proposal_text(&p),
        render_amendment_proposal_text(&p)
    );
}

#[test]
fn graph_proposal_json_rendering_byte_stable() {
    let p = seed_t12_g_graph_proposal();
    assert_eq!(
        render_amendment_proposal_json(&p),
        render_amendment_proposal_json(&p)
    );
}
