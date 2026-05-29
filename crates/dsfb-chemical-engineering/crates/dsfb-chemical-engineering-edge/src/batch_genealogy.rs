//! `BatchGenealogyGraphV1` (Wave-4 historian layer) — a sealed DAG of batch/campaign lineage, so a defect
//! that recurs across batches **sharing a lot, an equipment train, or a parent batch** is visible as
//! structure rather than coincidence.
//!
//! Batch plants reuse feedstock lots, equipment, and intermediates across many batches. When the same
//! anomaly appears in several batches, the engineer's first question is "what do they share?". This object
//! records the genealogy as a directed graph — a batch node carries its lot / equipment / parent links — and
//! exports it as Graphviz DOT + JSON, sealed by a `graph_hash`. A small helper finds the shared ancestry of a
//! set of flagged batches (the candidate common factor).
//!
//! Bounded (non-claims): a genealogy graph is **declared structural lineage, not a causal attribution** — a
//! shared lot among recurring episodes is a candidate common factor to investigate, never proof it is the
//! cause. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One batch and the entities it shares with others (the edges of the genealogy).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchNode {
    pub batch_id: String,
    /// Feedstock lot ids consumed by this batch.
    pub lots: Vec<String>,
    /// Equipment-train ids used by this batch.
    pub equipment: Vec<String>,
    /// Parent batch ids (e.g. a recycled intermediate or a split campaign).
    pub parents: Vec<String>,
}

/// A hash-sealed batch-genealogy graph (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchGenealogyGraphV1 {
    pub batches: Vec<BatchNode>,
    pub non_claim: String,
    pub graph_hash: String,
}

impl BatchGenealogyGraphV1 {
    const NON_CLAIM: &'static str =
        "declared structural lineage; a shared lot/equipment/parent among recurring episodes is a candidate common factor, NOT proof of causation";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"batch_genealogy_graph_v1");
        for b in &self.batches {
            h.field("batch_id", b.batch_id.as_bytes());
            for l in &b.lots {
                h.field("lot", l.as_bytes());
            }
            for e in &b.equipment {
                h.field("equipment", e.as_bytes());
            }
            for p in &b.parents {
                h.field("parent", p.as_bytes());
            }
        }
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    pub fn build(batches: Vec<BatchNode>) -> Self {
        let mut g = BatchGenealogyGraphV1 {
            batches,
            non_claim: Self::NON_CLAIM.to_string(),
            graph_hash: String::new(),
        };
        g.graph_hash = g.seal();
        g
    }

    fn node(&self, batch_id: &str) -> Option<&BatchNode> {
        self.batches.iter().find(|b| b.batch_id == batch_id)
    }

    /// The entities (lots and equipment) **shared by every** batch in `batch_ids` — the candidate common
    /// factor behind a recurring defect. Returns a sorted, de-duplicated set (deterministic). An unknown
    /// batch id, or fewer than two known batches, yields an empty set.
    pub fn shared_factors(&self, batch_ids: &[&str]) -> Vec<String> {
        let nodes: Vec<&BatchNode> = batch_ids.iter().filter_map(|id| self.node(id)).collect();
        if nodes.len() < 2 || nodes.len() != batch_ids.len() {
            return Vec::new();
        }
        let factor_set = |b: &BatchNode| -> BTreeSet<String> {
            b.lots
                .iter()
                .map(|l| format!("lot:{l}"))
                .chain(b.equipment.iter().map(|e| format!("equipment:{e}")))
                .collect()
        };
        let mut common = factor_set(nodes[0]);
        for n in &nodes[1..] {
            common = common.intersection(&factor_set(n)).cloned().collect();
        }
        common.into_iter().collect() // BTreeSet → sorted Vec
    }

    /// Render as a Graphviz DOT digraph: batch nodes, plus `lot:`/`equipment:` resource nodes they link to,
    /// and parent→child batch edges.
    pub fn to_dot(&self) -> String {
        let esc = |s: &str| s.replace('"', "'");
        let mut out = String::from("digraph batch_genealogy {\n  rankdir=LR;\n");
        for b in &self.batches {
            out.push_str(&format!("  \"{}\" [shape=box];\n", esc(&b.batch_id)));
            for l in &b.lots {
                out.push_str(&format!(
                    "  \"lot:{}\" [shape=ellipse]; \"lot:{}\" -> \"{}\";\n",
                    esc(l),
                    esc(l),
                    esc(&b.batch_id)
                ));
            }
            for e in &b.equipment {
                out.push_str(&format!(
                    "  \"equipment:{}\" [shape=hexagon]; \"equipment:{}\" -> \"{}\";\n",
                    esc(e),
                    esc(e),
                    esc(&b.batch_id)
                ));
            }
            for p in &b.parents {
                out.push_str(&format!("  \"{}\" -> \"{}\";\n", esc(p), esc(&b.batch_id)));
            }
        }
        out.push_str("}\n");
        out
    }

    pub fn verify(&self) -> bool {
        self.non_claim == Self::NON_CLAIM && self.seal() == self.graph_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> BatchGenealogyGraphV1 {
        BatchGenealogyGraphV1::build(vec![
            BatchNode {
                batch_id: "B-01".into(),
                lots: vec!["LOT-A".into()],
                equipment: vec!["R1".into()],
                parents: vec![],
            },
            BatchNode {
                batch_id: "B-02".into(),
                lots: vec!["LOT-A".into()],
                equipment: vec!["R2".into()],
                parents: vec![],
            },
            BatchNode {
                batch_id: "B-03".into(),
                lots: vec!["LOT-B".into()],
                equipment: vec!["R1".into()],
                parents: vec!["B-01".into()],
            },
        ])
    }

    #[test]
    fn shared_factors_finds_the_common_lot() {
        let g = graph();
        // B-01 and B-02 both ran LOT-A (different reactors) ⇒ the shared factor is the lot.
        assert_eq!(
            g.shared_factors(&["B-01", "B-02"]),
            vec!["lot:LOT-A".to_string()]
        );
        // B-01 and B-03 share reactor R1 (different lots) ⇒ the shared factor is the equipment.
        assert_eq!(
            g.shared_factors(&["B-01", "B-03"]),
            vec!["equipment:R1".to_string()]
        );
        // All three share nothing common to all.
        assert!(g.shared_factors(&["B-01", "B-02", "B-03"]).is_empty());
        // An unknown batch yields an empty set (no fabricated factor).
        assert!(g.shared_factors(&["B-01", "B-99"]).is_empty());
    }

    #[test]
    fn graph_seals_self_verifies_and_renders_dot() {
        let g = graph();
        assert!(g.graph_hash.len() == 64 && g.verify());
        assert_eq!(BatchGenealogyGraphV1::build(g.batches.clone()), g); // deterministic
        let dot = g.to_dot();
        assert!(
            dot.starts_with("digraph batch_genealogy {") && dot.contains("\"B-01\" -> \"B-03\"")
        );
        assert!(g.non_claim.contains("NOT proof of causation"));
        let mut t = g.clone();
        t.batches[0].lots.push("LOT-X".into());
        assert!(!t.verify());
    }
}
