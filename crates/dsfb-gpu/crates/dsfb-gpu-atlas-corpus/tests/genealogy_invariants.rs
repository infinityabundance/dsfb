// Tests legitimately panic on parse / load failures so the test
// output names the assertion location; the workspace's pedantic
// lints would otherwise flag every .expect() / .unwrap().
#![allow(clippy::expect_used, clippy::unwrap_used)]

//! T.5 acceptance tests: detector genealogy graph invariants.
//!
//! Panel-locked tests (each name documents an invariant):
//!
//! - `genealogy_is_dag` — no cycle through strict-ancestry edges.
//! - `genealogy_export_dot_is_stable` — two runs produce identical
//!   DOT bytes.
//! - `genealogy_export_json_is_stable` — same for JSON.
//! - `every_alias_decision_has_alias_collapsed_edge` — T.4 court
//!   decisions imply T.5 edges (the most important test; turns
//!   the graph into an audit surface, not decoration).
//! - `every_composition_decision_has_composes_edge`
//! - `missing_genealogy_target_is_rejected` — verify rejects
//!   dangling-target edges.
//! - `cycle_is_rejected` — synthesised seed with a cycle fails
//!   verify.
//! - `canonical_origin_nodes_may_have_no_parents`
//! - `non_origin_noncanonical_nodes_have_at_least_one_edge`

use dsfb_gpu_atlas_corpus::claims::{DetectorClaim, CLAIMS};
use dsfb_gpu_atlas_corpus::court::classify_all;
use dsfb_gpu_atlas_corpus::genealogy::{
    build_from, build_from_pair, build_genealogy, export_dot, export_json, verify_genealogy,
    GenealogyEdge, GenealogyEdgeKind, GenealogyNode, GENEALOGY_SCHEMA,
};
use dsfb_gpu_atlas_corpus::seed::SEED;
use dsfb_gpu_atlas_corpus::types::{
    CanonicalisationDecision, DedupReason, DedupRecord, DedupSubject, DetectorAliasId,
    DetectorCanonicalId,
};

#[test]
fn genealogy_is_dag() {
    let graph = build_genealogy();
    let report = verify_genealogy(&graph);
    assert!(
        report.is_clean(),
        "genealogy has {} verify errors: {:?}",
        report.errors.len(),
        report
            .errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn genealogy_node_count_is_seed_plus_claims() {
    let graph = build_genealogy();
    assert_eq!(
        graph.nodes.len(),
        SEED.len() + CLAIMS.len(),
        "node count should equal seed + claims (no extras, no drops)"
    );
}

#[test]
fn genealogy_export_dot_is_stable() {
    let graph = build_genealogy();
    let a = export_dot(&graph);
    let b = export_dot(&graph);
    assert_eq!(a, b, "DOT export must be byte-deterministic");
    assert!(a.contains("digraph dsfb_detector_genealogy"));
    assert!(a.contains(GENEALOGY_SCHEMA));
}

#[test]
fn genealogy_export_json_is_stable() {
    let graph = build_genealogy();
    let a = export_json(&graph);
    let b = export_json(&graph);
    assert_eq!(a, b, "JSON export must be byte-deterministic");
    assert!(a.contains(GENEALOGY_SCHEMA));
    assert!(a.contains("\"nodes\":"));
    assert!(a.contains("\"edges\":"));
}

#[test]
fn every_alias_decision_has_alias_collapsed_edge() {
    // The load-bearing T.5 audit invariant: T.4 court decisions
    // imply T.5 graph edges. Every AliasOf record in the court's
    // output must show up as an AliasCollapsedInto edge in the
    // genealogy.
    let court = classify_all();
    let graph = build_from(SEED, CLAIMS, &court);
    for record in &court {
        let CanonicalisationDecision::AliasOf(target) = record.decision else {
            continue;
        };
        let found = graph.edges.iter().any(|e| {
            e.kind == GenealogyEdgeKind::AliasCollapsedInto
                && e.source == DedupSubject::Canonical(target)
                && e.target == record.subject
        });
        assert!(
            found,
            "court record AliasOf({}) for subject {:?} has no matching AliasCollapsedInto edge in the genealogy",
            target.0, record.subject
        );
    }
}

#[test]
fn every_composition_decision_has_composes_edge() {
    let court = classify_all();
    let graph = build_from(SEED, CLAIMS, &court);
    for record in &court {
        let CanonicalisationDecision::CompositionOf(parents) = record.decision else {
            continue;
        };
        for parent in parents {
            let found = graph.edges.iter().any(|e| {
                e.kind == GenealogyEdgeKind::Composes
                    && e.source == DedupSubject::Canonical(*parent)
                    && e.target == record.subject
            });
            assert!(
                found,
                "court record CompositionOf({}) for subject {:?} has no matching Composes edge in the genealogy",
                parent.0, record.subject
            );
        }
    }
}

#[test]
fn missing_genealogy_target_is_rejected() {
    // Synthesise a court record whose decision points at a
    // non-existent canonical_id and feed it into build_from.
    let synthesised = DedupRecord {
        subject: DedupSubject::AliasClaim(DetectorAliasId(9999)),
        literature_name: "synthesised-bad-target",
        decision: CanonicalisationDecision::AliasOf(DetectorCanonicalId(99999)),
        reason_code: DedupReason::SameFormulaSameParametersSameContract,
        notes: "fixture: target canonical does not exist",
    };
    let mut court = classify_all();
    court.push(synthesised);
    let graph = build_from(SEED, CLAIMS, &court);
    let report = verify_genealogy(&graph);
    assert!(
        !report.is_clean(),
        "genealogy verify must reject dangling-target edges"
    );
}

#[test]
fn cycle_is_rejected() {
    // Synthesise a tiny seed with a deliberate 2-node cycle and
    // make sure verify catches it. We don't mutate the global
    // SEED; we feed a custom slice to build_from_pair.
    use dsfb_gpu_atlas_corpus::types::{
        AxisBindingSet, ConfuserProfile, ConstitutionFlags, DecisionFunctional,
        DeterministicStatus, DomainTagSet, DuplicateGroupId, GenealogyEdges, GpuFamilyKernel,
        ImplementationLevel, InputRequirementSet, LifecycleState, LiteratureDetector, MathFormId,
        NegativeWitnessKind, ParameterBounds, PrimitiveFamily, SourceRef, UsefulnessLedgerSnapshot,
        WitnessKind, WitnessRole,
    };
    static AB_DERIVED: &[DetectorCanonicalId] = &[DetectorCanonicalId(2)];
    static BA_DERIVED: &[DetectorCanonicalId] = &[DetectorCanonicalId(1)];
    let all_true = ConstitutionFlags {
        declared_input_contract: true,
        declared_output_type: true,
        declared_deterministic_form: true,
        declared_provenance: true,
        declared_equivalence_status: true,
        declared_witness_role: true,
        declared_activation_conditions: true,
        declared_failure_confuser_modes: true,
    };
    let mk = |id: u32,
              derived_from: &'static [DetectorCanonicalId],
              is_origin: bool|
     -> LiteratureDetector {
        LiteratureDetector {
            canonical_id: DetectorCanonicalId(id),
            display_name: "cycle-fixture",
            aliases: &[],
            source_refs: &[SourceRef {
                citation_key: "fixture",
                title: "fixture",
                authors: "fixture",
                year: 2026,
                venue_or_source: "engineering practice (cycle fixture)",
                doi_or_url: None,
                notes: "test fixture",
            }],
            origin_domains: DomainTagSet(DomainTagSet::INDUSTRIAL),
            primitive_family: PrimitiveFamily::ScalarThreshold,
            mathematical_form: MathFormId::Threshold,
            decision_functional: DecisionFunctional::TwoSided,
            input_requirements: InputRequirementSet(InputRequirementSet::NUMERIC),
            output_witness: WitnessKind::BooleanCell,
            witness_role: WitnessRole::Primary,
            negative_witness_kind: NegativeWitnessKind::NotANegativeWitness,
            fusion_axes: AxisBindingSet(AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE),
            confuser_profile: ConfuserProfile::None,
            deterministic_status: DeterministicStatus::DeterministicNative,
            implementation_status: ImplementationLevel::L1_Canonicalised,
            gpu_family: GpuFamilyKernel::ScalarThresholdFamily,
            parameter_bounds: ParameterBounds {
                axis_count: 1,
                description: "",
            },
            duplicate_group: DuplicateGroupId(id),
            genealogy: GenealogyEdges {
                derived_from,
                generalizes: &[],
                special_case_of: &[],
                is_origin,
            },
            usefulness: UsefulnessLedgerSnapshot::unmeasured(),
            lifecycle_state: LifecycleState::Active,
            constitution_compliance: all_true,
        }
    };
    let a = mk(1, AB_DERIVED, false);
    let b = mk(2, BA_DERIVED, false);
    let seed = [a, b];
    let graph = build_from_pair(&seed, &[]);
    let report = verify_genealogy(&graph);
    assert!(
        !report.is_clean(),
        "synthesised 2-node cycle (1 derived_from 2; 2 derived_from 1) must be rejected"
    );
    let saw_cycle = report.errors.iter().any(|e| {
        e.message.to_lowercase().contains("dag") || e.message.to_lowercase().contains("cycle")
    });
    assert!(
        saw_cycle,
        "DAG-violation message expected; got {:?}",
        report.errors
    );
}

#[test]
fn canonical_origin_nodes_may_have_no_parents() {
    let graph = build_genealogy();
    // At least some seed records are origins; the verify pass
    // must not reject them for lack of incoming strict-ancestry
    // edges.
    let origin_count = graph.nodes.iter().filter(|n| n.is_origin).count();
    assert!(
        origin_count > 0,
        "seed must contain at least one origin record"
    );
    let report = verify_genealogy(&graph);
    assert!(
        report.is_clean(),
        "origin nodes must not cause verify errors"
    );
}

#[test]
fn non_origin_canonical_nodes_have_at_least_one_strict_edge() {
    let graph = build_genealogy();
    for node in &graph.nodes {
        let DedupSubject::Canonical(_) = node.id else {
            continue;
        };
        if node.is_origin {
            continue;
        }
        // For non-origin canonicals, at least one incoming
        // strict-ancestry edge must exist (DerivedFrom /
        // Generalizes / SpecialCaseOf / ParameterVariantOf /
        // Composes).
        let has_strict_in = graph
            .edges
            .iter()
            .any(|e| e.target == node.id && e.kind.is_strict_ancestry());
        assert!(
            has_strict_in,
            "non-origin canonical {:?} ('{}') has no incoming strict-ancestry edge",
            node.id, node.label
        );
    }
}

#[test]
fn alias_claims_have_exactly_one_incoming_collapse_edge() {
    let graph = build_genealogy();
    for node in &graph.nodes {
        let DedupSubject::AliasClaim(_) = node.id else {
            continue;
        };
        let count = graph
            .edges
            .iter()
            .filter(|e| e.target == node.id && e.kind == GenealogyEdgeKind::AliasCollapsedInto)
            .count();
        assert_eq!(
            count, 1,
            "alias claim {:?} must have exactly one incoming AliasCollapsedInto edge; got {count}",
            node.id
        );
    }
}

#[test]
fn dot_export_lists_every_node() {
    let graph = build_genealogy();
    let dot = export_dot(&graph);
    for node in &graph.nodes {
        let id = match node.id {
            DedupSubject::Canonical(c) => format!("C{:03}", c.0),
            DedupSubject::AliasClaim(a) => format!("A{:04}", a.0),
        };
        assert!(
            dot.contains(&format!("\"{id}\"")),
            "DOT must contain node {id}"
        );
    }
}

#[test]
fn json_export_has_canonical_schema_string() {
    let graph = build_genealogy();
    let json = export_json(&graph);
    assert!(
        json.contains("\"schema\": \"DSFB-GPU-ATLAS:GENEALOGY:v1\""),
        "JSON export must carry the versioned schema string"
    );
}

#[test]
fn edge_kind_partitions_strict_and_loose_correctly() {
    let strict = [
        GenealogyEdgeKind::DerivedFrom,
        GenealogyEdgeKind::Generalizes,
        GenealogyEdgeKind::SpecialCaseOf,
        GenealogyEdgeKind::ParameterVariantOf,
        GenealogyEdgeKind::Composes,
    ];
    for k in strict {
        assert!(
            k.is_strict_ancestry(),
            "{k:?} must be a strict-ancestry kind"
        );
    }
    let loose = [
        GenealogyEdgeKind::AliasCollapsedInto,
        GenealogyEdgeKind::DomainTransferOf,
    ];
    for k in loose {
        assert!(
            !k.is_strict_ancestry(),
            "{k:?} must NOT be a strict-ancestry kind"
        );
    }
}

#[test]
fn western_electric_has_incoming_composes_edge_from_shewhart() {
    let graph = build_genealogy();
    let edge = graph.edges.iter().find(|e| {
        e.source == DedupSubject::Canonical(DetectorCanonicalId(1))
            && e.target == DedupSubject::Canonical(DetectorCanonicalId(16))
            && e.kind == GenealogyEdgeKind::Composes
    });
    assert!(
        edge.is_some(),
        "expected Shewhart -> Western Electric `Composes` edge from T.4 court"
    );
}

#[test]
fn nelson_has_incoming_composes_edges_from_shewhart_and_we() {
    let graph = build_genealogy();
    for parent_id in [1u32, 16u32] {
        let edge = graph.edges.iter().find(|e| {
            e.source == DedupSubject::Canonical(DetectorCanonicalId(parent_id))
                && e.target == DedupSubject::Canonical(DetectorCanonicalId(17))
                && e.kind == GenealogyEdgeKind::Composes
        });
        assert!(
            edge.is_some(),
            "expected C{parent_id:03} -> Nelson `Composes` edge from T.4 court"
        );
    }
}

#[test]
fn robust_z_aliases_collapse_to_canonical_6() {
    let graph = build_genealogy();
    let canonical_6 = DedupSubject::Canonical(DetectorCanonicalId(6));
    let aliases: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.source == canonical_6 && e.kind == GenealogyEdgeKind::AliasCollapsedInto)
        .collect();
    assert_eq!(
        aliases.len(),
        3,
        "expected 3 aliases of canonical 6 (robust-z, median-MAD z, MAD outlier detector)"
    );
}

#[test]
fn build_is_deterministic_across_two_passes() {
    let a = build_genealogy();
    let b = build_genealogy();
    assert_eq!(a.nodes.len(), b.nodes.len());
    assert_eq!(a.edges.len(), b.edges.len());
    for (na, nb) in a.nodes.iter().zip(b.nodes.iter()) {
        assert_eq!(na.id, nb.id);
        assert_eq!(na.label, nb.label);
        assert_eq!(na.is_origin, nb.is_origin);
    }
    for (ea, eb) in a.edges.iter().zip(b.edges.iter()) {
        assert_eq!(ea.source, eb.source);
        assert_eq!(ea.target, eb.target);
        assert_eq!(ea.kind, eb.kind);
    }
}

// Silence unused-import warnings.
#[allow(dead_code)]
const _: Option<(GenealogyNode, GenealogyEdge, DetectorClaim, DedupRecord)> = None;
