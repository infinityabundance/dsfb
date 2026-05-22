//! T.5 — deterministic detector genealogy graph.
//!
//! The graph is a court transcript of detector ancestry: every edge
//! is either (a) a `GenealogyEdges` declaration from the
//! T.1a/T.1b static seed or (b) a derivation from a T.4 court
//! judgment (`AliasOf` → `AliasCollapsedInto`, `CompositionOf` →
//! `Composes`). T.5 introduces NO new classifications — it surfaces
//! relationships the seed and the court already encode.
//!
//! Hard discipline (panel-locked):
//!
//! - Every edge must be honest. The seven edge kinds reflect what
//!   the corpus actually knows (`DerivedFrom`, `Generalizes`,
//!   `SpecialCaseOf`, `ParameterVariantOf`, `DomainTransferOf`,
//!   `Composes`, `AliasCollapsedInto`). When the relationship is
//!   only "shares a primitive family" we emit nothing — a missing
//!   edge is better than a wrong edge.
//! - The graph is a DAG over the strict-ancestry edges
//!   (`DerivedFrom`, `Generalizes`, `SpecialCaseOf`,
//!   `ParameterVariantOf`, `Composes`). The DAG check rejects
//!   cycles. `AliasCollapsedInto` and `DomainTransferOf` are
//!   informational and bypass the cycle check.
//! - Both DOT and JSON exports are byte-deterministic. Two runs
//!   over the same inputs produce identical output.
//!
//! T.5 explicitly does NOT do (panel-locked):
//!
//! - No `corpus_hash_v1` (that is T.10).
//! - No GPU-family mapping (T.11).
//! - No new equivalence judgments (T.4's province).
//! - No fuzzy "looks-like" edges. Every edge has a deterministic
//!   source in the seed or the court.

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use crate::claims::{DetectorClaim, CLAIMS};
use crate::court::{classify, classify_all};
use crate::seed::SEED;
use crate::types::{
    CanonicalisationDecision, DedupRecord, DedupSubject, DetectorAliasId, DetectorCanonicalId,
    LiteratureDetector,
};

/// Schema version string used in the JSON export. Bumps require
/// a backwards-compatible migration path (and a v2 domain
/// separator if T.10's `corpus_hash_v1` is consuming this).
pub const GENEALOGY_SCHEMA: &str = "DSFB-GPU-ATLAS:GENEALOGY:v1";

/// One node in the genealogy graph.
///
/// Canonical seed records and alias claims both appear as nodes.
/// The discriminator is the `id` field's variant ([`DedupSubject`]).
#[derive(Debug, Clone, Copy)]
pub struct GenealogyNode {
    /// Unique identifier (either Canonical(id) or AliasClaim(id)).
    pub id: DedupSubject,
    /// Human-readable label echoing the seed display_name or the
    /// claim literature_name.
    pub label: &'static str,
    /// True if the node is a canonical record with `is_origin=true`
    /// in its `GenealogyEdges` (no recorded ancestors). Alias
    /// claims are never origin nodes; they are always
    /// `AliasCollapsedInto` a canonical.
    pub is_origin: bool,
}

/// One directed edge in the genealogy graph.
///
/// Direction convention: `source` is the ancestor / parent /
/// component; `target` is the descendant / child / composer /
/// alias. Reading an edge left-to-right is "parent → child":
///
/// - "Shewhart → Robust z (`DerivedFrom`)" reads as "Robust z
///   derives from Shewhart".
/// - "Shewhart → Western Electric (`Composes`)" reads as "Shewhart
///   is composed into Western Electric" (Western Electric uses
///   Shewhart as a component).
/// - "Robust z → robust z-score (`AliasCollapsedInto`)" reads as
///   "robust z-score collapses into Robust z".
#[derive(Debug, Clone, Copy)]
pub struct GenealogyEdge {
    /// The parent / ancestor / component / canonical.
    pub source: DedupSubject,
    /// The child / descendant / composer / alias.
    pub target: DedupSubject,
    /// The kind of relationship.
    pub kind: GenealogyEdgeKind,
}

/// The seven edge kinds the corpus uses today.
///
/// `RelatedFamily` is intentionally NOT included: a relationship
/// the corpus cannot honestly classify becomes a *missing* edge.
/// Future T.5.* expansions may introduce labelled but-still-honest
/// edge kinds (e.g. `SharesPrimitiveFamilyWith`) without breaking
/// the DAG invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenealogyEdgeKind {
    /// `target` derives from `source`. Source-of-truth: seed's
    /// `genealogy.derived_from` field.
    DerivedFrom,
    /// `source` generalises `target`: the target is a more
    /// specific case of the source's primitive. Source-of-truth:
    /// seed's `genealogy.generalizes` field on the SOURCE record
    /// (we invert direction when materialising the edge).
    Generalizes,
    /// `target` is a special case of `source`. Source-of-truth:
    /// seed's `genealogy.special_case_of` field on the TARGET
    /// record.
    SpecialCaseOf,
    /// `target` is a parameter variant of `source`. Source-of-
    /// truth: seed's `genealogy.parameter_variant_of` field.
    ParameterVariantOf,
    /// `target` is a domain transfer of `source` (same formula
    /// in a different domain). Source-of-truth: seed's
    /// `genealogy.domain_transfer_of` field.
    DomainTransferOf,
    /// `target` is a composition that includes `source` as a
    /// component. Source-of-truth: T.4 court record with
    /// `CanonicalisationDecision::CompositionOf(parents)`.
    Composes,
    /// `target` is an alias that collapses into the canonical
    /// `source`. Source-of-truth: T.4 court record with
    /// `CanonicalisationDecision::AliasOf(target)`.
    AliasCollapsedInto,
}

impl GenealogyEdgeKind {
    /// True for the kinds that participate in the strict-ancestry
    /// DAG (rejected if they form a cycle).
    #[must_use]
    pub const fn is_strict_ancestry(self) -> bool {
        matches!(
            self,
            Self::DerivedFrom
                | Self::Generalizes
                | Self::SpecialCaseOf
                | Self::ParameterVariantOf
                | Self::Composes
        )
    }

    /// Canonical wire name used in DOT / JSON exports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DerivedFrom => "DerivedFrom",
            Self::Generalizes => "Generalizes",
            Self::SpecialCaseOf => "SpecialCaseOf",
            Self::ParameterVariantOf => "ParameterVariantOf",
            Self::DomainTransferOf => "DomainTransferOf",
            Self::Composes => "Composes",
            Self::AliasCollapsedInto => "AliasCollapsedInto",
        }
    }
}

/// The genealogy graph: a deterministic list of nodes + edges.
///
/// Nodes and edges are sorted by stable identifiers so two builds
/// over the same seed + claims produce byte-identical exports.
#[derive(Debug, Clone, Default)]
pub struct GenealogyGraph {
    /// All nodes in canonical order (canonical_id ascending,
    /// then alias_id ascending).
    pub nodes: Vec<GenealogyNode>,
    /// All edges sorted by (source_key, target_key, kind).
    pub edges: Vec<GenealogyEdge>,
}

/// Build the canonical genealogy over the static seed + the T.4
/// CLAIMS + the T.4 court output.
#[must_use]
pub fn build_genealogy() -> GenealogyGraph {
    build_from(SEED, CLAIMS, &classify_all())
}

/// Build the genealogy over an arbitrary seed + claims + court
/// record set. Used by tests to exercise adversarial inputs.
#[must_use]
pub fn build_from(
    seed: &[LiteratureDetector],
    claims: &[DetectorClaim],
    court_records: &[DedupRecord],
) -> GenealogyGraph {
    let mut nodes: Vec<GenealogyNode> = Vec::new();
    for r in seed {
        nodes.push(GenealogyNode {
            id: DedupSubject::Canonical(r.canonical_id),
            label: r.display_name,
            is_origin: r.genealogy.is_origin,
        });
    }
    for c in claims {
        nodes.push(GenealogyNode {
            id: DedupSubject::AliasClaim(c.alias_id),
            label: c.literature_name,
            is_origin: false,
        });
    }
    nodes.sort_by_key(|n| node_sort_key(n.id));

    let mut edges: Vec<GenealogyEdge> = Vec::new();
    push_seed_edges(seed, &mut edges);
    push_court_edges(court_records, &mut edges);
    edges.sort_by_key(edge_sort_key);

    GenealogyGraph { nodes, edges }
}

fn push_seed_edges(seed: &[LiteratureDetector], out: &mut Vec<GenealogyEdge>) {
    for r in seed {
        let self_id = DedupSubject::Canonical(r.canonical_id);
        // derived_from: ancestor -> self
        for ancestor in r.genealogy.derived_from {
            out.push(GenealogyEdge {
                source: DedupSubject::Canonical(*ancestor),
                target: self_id,
                kind: GenealogyEdgeKind::DerivedFrom,
            });
        }
        // generalizes: self -> generalised child (self is the more
        // general parent). The schema stores this on the SOURCE
        // record; we materialise the edge in the natural direction.
        for child in r.genealogy.generalizes {
            out.push(GenealogyEdge {
                source: self_id,
                target: DedupSubject::Canonical(*child),
                kind: GenealogyEdgeKind::Generalizes,
            });
        }
        // special_case_of: parent -> self (self is the special
        // case of each parent).
        for parent in r.genealogy.special_case_of {
            out.push(GenealogyEdge {
                source: DedupSubject::Canonical(*parent),
                target: self_id,
                kind: GenealogyEdgeKind::SpecialCaseOf,
            });
        }
    }
}

fn push_court_edges(court_records: &[DedupRecord], out: &mut Vec<GenealogyEdge>) {
    for record in court_records {
        match record.decision {
            CanonicalisationDecision::CompositionOf(parents) => {
                let subject = record.subject;
                for p in parents {
                    out.push(GenealogyEdge {
                        source: DedupSubject::Canonical(*p),
                        target: subject,
                        kind: GenealogyEdgeKind::Composes,
                    });
                }
            }
            CanonicalisationDecision::AliasOf(target_canonical) => {
                out.push(GenealogyEdge {
                    source: DedupSubject::Canonical(target_canonical),
                    target: record.subject,
                    kind: GenealogyEdgeKind::AliasCollapsedInto,
                });
            }
            // Other decisions don't generate genealogy edges
            // directly; their semantic content is captured by the
            // court record itself.
            CanonicalisationDecision::Canonical
            | CanonicalisationDecision::ParameterisationOf(_)
            | CanonicalisationDecision::StochasticOriginalDeterministicReduction(_)
            | CanonicalisationDecision::RejectedNotDeterministic
            | CanonicalisationDecision::RejectedNotDetector
            | CanonicalisationDecision::DeferredNeedsReview => {}
        }
    }
}

fn node_sort_key(id: DedupSubject) -> (u8, u32) {
    match id {
        DedupSubject::Canonical(c) => (0, c.0),
        DedupSubject::AliasClaim(a) => (1, a.0),
    }
}

fn edge_sort_key(e: &GenealogyEdge) -> (u8, u32, u8, u32, &'static str) {
    let (src_kind, src_id) = node_sort_key(e.source);
    let (tgt_kind, tgt_id) = node_sort_key(e.target);
    (src_kind, src_id, tgt_kind, tgt_id, e.kind.as_str())
}

/// Verification report for the genealogy graph.
///
/// Reports:
///
/// 1. Cycle detection: any cycle through strict-ancestry edges.
/// 2. Dangling targets: edges whose source or target is not in
///    the node set.
/// 3. Origin / non-origin consistency: canonical records with
///    `is_origin=true` MUST have no incoming strict-ancestry edge;
///    canonical records with `is_origin=false` MUST have at least
///    one. Alias claims must each have exactly one incoming
///    `AliasCollapsedInto` edge.
#[derive(Debug, Clone, Default)]
pub struct GenealogyVerifyReport {
    /// Number of nodes inspected.
    pub nodes_inspected: usize,
    /// Number of edges inspected.
    pub edges_inspected: usize,
    /// Errors found (empty if clean).
    pub errors: Vec<GenealogyVerifyError>,
}

/// One internal-consistency failure.
#[derive(Debug, Clone)]
pub struct GenealogyVerifyError {
    /// Human-readable description of the failure.
    pub message: String,
}

impl GenealogyVerifyReport {
    /// True if no errors were recorded.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Verify the graph: DAG check + dangling-target check +
/// origin / non-origin consistency check.
#[must_use]
pub fn verify_genealogy(graph: &GenealogyGraph) -> GenealogyVerifyReport {
    let mut report = GenealogyVerifyReport {
        nodes_inspected: graph.nodes.len(),
        edges_inspected: graph.edges.len(),
        ..Default::default()
    };
    check_dangling_targets(graph, &mut report);
    check_origin_consistency(graph, &mut report);
    check_dag(graph, &mut report);
    report
}

fn check_dangling_targets(graph: &GenealogyGraph, report: &mut GenealogyVerifyReport) {
    let node_set: Vec<DedupSubject> = graph.nodes.iter().map(|n| n.id).collect();
    for edge in &graph.edges {
        if !node_set.contains(&edge.source) {
            report.errors.push(GenealogyVerifyError {
                message: format!("edge source {:?} is not in the node set", edge.source),
            });
        }
        if !node_set.contains(&edge.target) {
            report.errors.push(GenealogyVerifyError {
                message: format!("edge target {:?} is not in the node set", edge.target),
            });
        }
    }
}

fn check_origin_consistency(graph: &GenealogyGraph, report: &mut GenealogyVerifyReport) {
    for node in &graph.nodes {
        let strict_in = graph
            .edges
            .iter()
            .filter(|e| e.target == node.id && e.kind.is_strict_ancestry())
            .count();
        match node.id {
            DedupSubject::Canonical(_) => {
                if node.is_origin && strict_in > 0 {
                    report.errors.push(GenealogyVerifyError {
                        message: format!(
                            "node {:?} claims is_origin=true but has {strict_in} incoming strict-ancestry edge(s)",
                            node.id
                        ),
                    });
                }
                if !node.is_origin && strict_in == 0 {
                    report.errors.push(GenealogyVerifyError {
                        message: format!(
                            "non-origin canonical node {:?} has no incoming strict-ancestry edges",
                            node.id
                        ),
                    });
                }
            }
            DedupSubject::AliasClaim(_) => {
                let alias_in = graph
                    .edges
                    .iter()
                    .filter(|e| {
                        e.target == node.id && e.kind == GenealogyEdgeKind::AliasCollapsedInto
                    })
                    .count();
                if alias_in != 1 {
                    report.errors.push(GenealogyVerifyError {
                        message: format!(
                            "alias claim {:?} must have exactly one incoming AliasCollapsedInto edge; found {alias_in}",
                            node.id
                        ),
                    });
                }
            }
        }
    }
}

fn check_dag(graph: &GenealogyGraph, report: &mut GenealogyVerifyReport) {
    // Iterative Kahn-style topological sort over strict-ancestry edges.
    let node_ids: Vec<DedupSubject> = graph.nodes.iter().map(|n| n.id).collect();
    let mut indegree: Vec<usize> = node_ids
        .iter()
        .map(|&n| {
            graph
                .edges
                .iter()
                .filter(|e| e.target == n && e.kind.is_strict_ancestry())
                .count()
        })
        .collect();
    let mut frontier: Vec<usize> = indegree
        .iter()
        .enumerate()
        .filter(|(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();
    let mut visited = 0usize;
    while let Some(idx) = frontier.pop() {
        visited += 1;
        let nid = node_ids[idx];
        for edge in &graph.edges {
            if edge.source == nid && edge.kind.is_strict_ancestry() {
                if let Some(tgt_idx) = node_ids.iter().position(|&n| n == edge.target) {
                    indegree[tgt_idx] = indegree[tgt_idx].saturating_sub(1);
                    if indegree[tgt_idx] == 0 {
                        frontier.push(tgt_idx);
                    }
                }
            }
        }
    }
    if visited < node_ids.len() {
        report.errors.push(GenealogyVerifyError {
            message: format!(
                "genealogy is not a DAG: {} of {} nodes were unreachable via topological sort (cycle present)",
                node_ids.len() - visited,
                node_ids.len()
            ),
        });
    }
}

/// Render the genealogy as a Graphviz DOT document. Deterministic.
#[must_use]
pub fn export_dot(graph: &GenealogyGraph) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "digraph dsfb_detector_genealogy {{");
    let _ = writeln!(out, "  // schema: {GENEALOGY_SCHEMA}");
    let _ = writeln!(out, "  rankdir=LR;");
    let _ = writeln!(out, "  node [shape=box, fontname=\"monospace\"];");
    for node in &graph.nodes {
        let id = format_node_id(node.id);
        let shape = match node.id {
            DedupSubject::Canonical(_) => "box",
            DedupSubject::AliasClaim(_) => "ellipse",
        };
        let _ = writeln!(
            out,
            "  \"{}\" [label=\"{}\\n{}\", shape={}];",
            id,
            id,
            escape_dot(node.label),
            shape
        );
    }
    for edge in &graph.edges {
        let _ = writeln!(
            out,
            "  \"{}\" -> \"{}\" [label=\"{}\"];",
            format_node_id(edge.source),
            format_node_id(edge.target),
            edge.kind.as_str()
        );
    }
    let _ = writeln!(out, "}}");
    out
}

/// Render the genealogy as a flat-list JSON document. Deterministic.
/// Schema: `DSFB-GPU-ATLAS:GENEALOGY:v1`.
#[must_use]
pub fn export_json(graph: &GenealogyGraph) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"schema\": \"{GENEALOGY_SCHEMA}\",");
    let _ = writeln!(out, "  \"nodes\": [");
    for (i, node) in graph.nodes.iter().enumerate() {
        let comma = if i + 1 < graph.nodes.len() { "," } else { "" };
        let id = format_node_id(node.id);
        let kind = match node.id {
            DedupSubject::Canonical(_) => "Canonical",
            DedupSubject::AliasClaim(_) => "AliasClaim",
        };
        let _ = writeln!(
            out,
            "    {{ \"id\": \"{id}\", \"kind\": \"{kind}\", \"label\": \"{label}\", \"is_origin\": {origin} }}{comma}",
            label = escape_json(node.label),
            origin = node.is_origin
        );
    }
    let _ = writeln!(out, "  ],");
    let _ = writeln!(out, "  \"edges\": [");
    for (i, edge) in graph.edges.iter().enumerate() {
        let comma = if i + 1 < graph.edges.len() { "," } else { "" };
        let _ = writeln!(
            out,
            "    {{ \"source\": \"{}\", \"target\": \"{}\", \"kind\": \"{}\" }}{}",
            format_node_id(edge.source),
            format_node_id(edge.target),
            edge.kind.as_str(),
            comma
        );
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

fn format_node_id(id: DedupSubject) -> String {
    match id {
        DedupSubject::Canonical(c) => format!("C{:03}", c.0),
        DedupSubject::AliasClaim(a) => format!("A{:04}", a.0),
    }
}

fn escape_dot(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            other => out.push(other),
        }
    }
    out
}

// Convenience: build then verify in one pass for tests.
#[doc(hidden)]
#[must_use]
pub fn build_and_verify() -> (GenealogyGraph, GenealogyVerifyReport) {
    let graph = build_genealogy();
    let report = verify_genealogy(&graph);
    (graph, report)
}

// Force the `classify` path to be exercised at compile time so
// callers don't need to re-import the court module just to use
// genealogy. The standalone helper is also handy for adversarial
// fixtures that supply a synthesised seed.
#[doc(hidden)]
#[must_use]
pub fn build_from_pair(seed: &[LiteratureDetector], claims: &[DetectorClaim]) -> GenealogyGraph {
    let court = classify(seed, claims);
    build_from(seed, claims, &court)
}

// Internal sanity test: every alias claim resolves to a canonical
// node when the court output is fed in. Kept as a #[doc(hidden)]
// function so the integration tests can exercise it without
// duplicating the iteration.
#[doc(hidden)]
#[must_use]
pub fn aliases_with_no_incoming_edge(graph: &GenealogyGraph) -> Vec<DetectorAliasId> {
    let mut out: Vec<DetectorAliasId> = Vec::new();
    for n in &graph.nodes {
        let DedupSubject::AliasClaim(aid) = n.id else {
            continue;
        };
        let has_edge = graph
            .edges
            .iter()
            .any(|e| e.target == n.id && e.kind == GenealogyEdgeKind::AliasCollapsedInto);
        if !has_edge {
            out.push(aid);
        }
    }
    out
}

// Allow DetectorCanonicalId to be used directly in tests that
// reference the seed by ID without importing it explicitly.
#[allow(dead_code)]
const _CANONICAL_ID_HANDLE: Option<DetectorCanonicalId> = None;
