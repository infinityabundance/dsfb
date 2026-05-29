//! `ResidualProvenanceGraphV1` — a first-class, hash-sealed provenance graph (P58).
//!
//! The pipeline already writes a flat `residual_provenance.csv` ledger (which residuals fed which
//! detector). This promotes that lineage to an explicit **graph** of the full chain —
//! `raw channel → residual stream → detector → episode → operator label → court root` — emittable as
//! **Graphviz DOT** and **JSON**, and sealed with a `graph_hash`. It makes the "where did this label
//! come from?" question answerable as a walkable artifact: every label traces back through its episode
//! and detectors to the raw channels, and forward to the Court Record root that sealed it.
//!
//! Additive: it re-expresses lineage the ledger already records; it changes no sealed pipeline artifact.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The kind of a provenance node (the layer of the chain it belongs to).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    /// A raw input channel / process variable.
    Raw,
    /// A residual stream derived from a raw channel (observed − baseline).
    Residual,
    /// A detector that consumed residual streams.
    Detector,
    /// A fused structural episode.
    Episode,
    /// An operator-facing heuristic label.
    Label,
    /// The Court Record root that sealed the case (terminal node).
    CourtRoot,
}

impl NodeKind {
    /// Stable tag used in the hash preimage and the DOT rendering (decoupled from `Debug`).
    fn tag(self) -> &'static str {
        match self {
            NodeKind::Raw => "raw",
            NodeKind::Residual => "residual",
            NodeKind::Detector => "detector",
            NodeKind::Episode => "episode",
            NodeKind::Label => "label",
            NodeKind::CourtRoot => "court_root",
        }
    }
    /// Graphviz node shape per layer (purely cosmetic).
    fn shape(self) -> &'static str {
        match self {
            NodeKind::Raw => "box",
            NodeKind::Residual => "ellipse",
            NodeKind::Detector => "component",
            NodeKind::Episode => "folder",
            NodeKind::Label => "note",
            NodeKind::CourtRoot => "doubleoctagon",
        }
    }
}

/// A provenance node: a stable `id` (unique within the graph), its `kind`, and a human-readable `label`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
}

/// A directed provenance edge `from → to` (both must be node ids in the graph).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvEdge {
    pub from: String,
    pub to: String,
}

/// A hash-sealed residual-provenance graph (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualProvenanceGraphV1 {
    pub dataset: String,
    pub nodes: Vec<ProvNode>,
    pub edges: Vec<ProvEdge>,
    /// SHA-256 (via [`CanonicalHasher`]) over the dataset + nodes + edges, in insertion order.
    pub graph_hash: String,
}

/// Incremental builder for a provenance graph. Add nodes by layer, link them, then `seal()`.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceGraphBuilder {
    dataset: String,
    nodes: Vec<ProvNode>,
    edges: Vec<ProvEdge>,
}

impl ProvenanceGraphBuilder {
    pub fn new(dataset: impl Into<String>) -> Self {
        ProvenanceGraphBuilder {
            dataset: dataset.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node and return its id (so callers can chain links). Ids are caller-supplied + unique.
    pub fn node(
        &mut self,
        id: impl Into<String>,
        kind: NodeKind,
        label: impl Into<String>,
    ) -> String {
        let id = id.into();
        self.nodes.push(ProvNode {
            id: id.clone(),
            kind,
            label: label.into(),
        });
        id
    }

    /// Link `from → to` (a directed provenance edge). Both ids should already be nodes.
    pub fn link(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push(ProvEdge {
            from: from.into(),
            to: to.into(),
        });
    }

    /// Finalise: compute the `graph_hash` and produce the sealed graph.
    pub fn seal(self) -> ResidualProvenanceGraphV1 {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"residual_provenance_graph_v1");
        h.field("dataset", self.dataset.as_bytes());
        for n in &self.nodes {
            h.field("node_id", n.id.as_bytes());
            h.field("node_kind", n.kind.tag().as_bytes());
            h.field("node_label", n.label.as_bytes());
        }
        for e in &self.edges {
            h.field("edge_from", e.from.as_bytes());
            h.field("edge_to", e.to.as_bytes());
        }
        let graph_hash = h.finalize_hex();
        ResidualProvenanceGraphV1 {
            dataset: self.dataset,
            nodes: self.nodes,
            edges: self.edges,
            graph_hash,
        }
    }
}

impl ResidualProvenanceGraphV1 {
    /// Re-derive the graph hash from the nodes + edges and check it matches `graph_hash`.
    pub fn verify(&self) -> bool {
        let mut b = ProvenanceGraphBuilder::new(self.dataset.clone());
        b.nodes = self.nodes.clone();
        b.edges = self.edges.clone();
        b.seal().graph_hash == self.graph_hash
    }

    /// Render as a Graphviz DOT digraph (nodes shaped + labelled by layer, edges directed).
    pub fn to_dot(&self) -> String {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        let mut out = String::new();
        out.push_str(&format!(
            "digraph residual_provenance {{\n  label=\"{}\";\n  rankdir=LR;\n",
            esc(&self.dataset)
        ));
        for n in &self.nodes {
            out.push_str(&format!(
                "  \"{}\" [shape={}, label=\"{}\"];\n",
                esc(&n.id),
                n.kind.shape(),
                esc(&n.label)
            ));
        }
        for e in &self.edges {
            out.push_str(&format!("  \"{}\" -> \"{}\";\n", esc(&e.from), esc(&e.to)));
        }
        out.push_str("}\n");
        out
    }

    /// Render as pretty JSON (the full sealed graph).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a small raw→residual→detector→episode→label→court chain.
    fn demo() -> ResidualProvenanceGraphV1 {
        let mut b = ProvenanceGraphBuilder::new("cstr_reactor");
        b.node("raw:T_reactor", NodeKind::Raw, "reactor temperature");
        b.node("res:T_reactor", NodeKind::Residual, "T_reactor − baseline");
        b.node("det:ewma_spe", NodeKind::Detector, "EWMA(SPE)");
        b.node("ep:0", NodeKind::Episode, "episode #0 (EnvViolation)");
        b.node(
            "lbl:H3",
            NodeKind::Label,
            "reactor thermal excursion candidate",
        );
        b.node("court:root", NodeKind::CourtRoot, "evidence_root abc123…");
        b.link("raw:T_reactor", "res:T_reactor");
        b.link("res:T_reactor", "det:ewma_spe");
        b.link("det:ewma_spe", "ep:0");
        b.link("ep:0", "lbl:H3");
        b.link("lbl:H3", "court:root");
        b.seal()
    }

    #[test]
    fn graph_is_deterministic_and_self_verifies() {
        let a = demo();
        let b = demo();
        assert_eq!(
            a.graph_hash, b.graph_hash,
            "graph hash must be deterministic"
        );
        assert_eq!(a.graph_hash.len(), 64);
        assert!(a.verify());
        assert_eq!(a.nodes.len(), 6);
        assert_eq!(a.edges.len(), 5);
    }

    #[test]
    fn dot_and_json_render_the_full_chain() {
        let g = demo();
        let dot = g.to_dot();
        assert!(dot.starts_with("digraph residual_provenance {"));
        assert!(dot.contains("\"raw:T_reactor\" -> \"res:T_reactor\";"));
        assert!(dot.contains("doubleoctagon")); // the court-root shape is present
        let json = g.to_json();
        assert!(json.contains("\"graph_hash\""));
        assert!(json.contains("\"court:root\""));
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut g = demo();
        assert!(g.verify());
        g.nodes[0].label = "tampered".into();
        assert!(!g.verify());
    }
}
