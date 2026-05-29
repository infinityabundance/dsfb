//! `MerkleDagAmendmentChainV1` (Wave-7 forensic) — a Git-like **append-only amendment DAG** over an immutable
//! Court Record, with lineage proofs and selective disclosure (for management-of-change compliance).
//!
//! The existing append-only amendment *chain* (`annotation.rs`) is linear. Real review is a DAG: an amendment
//! can build on several prior ones (a merge), and a node must be disclosable *with its lineage* without
//! revealing unrelated siblings. Each node seals its own content hash + its parents' node hashes, so the
//! whole history is Merkle-linked: tampering any ancestor's content changes its node hash and every
//! descendant's, breaking the chain. A `lineage_proof` is the set of ancestor node hashes that authenticate a
//! node back to the roots — shareable on its own (selective disclosure).
//!
//! Bounded (non-claims): the DAG authenticates *amendment lineage* (what was amended, on top of what), not
//! the *correctness* of any amendment's content, and the optional author/timestamp fields are recorded, not
//! cryptographically attested. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One amendment node: its stable id, the hash of its content, and the ids of the parent amendments it builds on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmendmentNode {
    pub id: String,
    pub content_hash: String,
    pub parents: Vec<String>,
    /// Merkle node hash = H(id, content_hash, parents' node_hashes) — recomputed in `verify`.
    pub node_hash: String,
}

/// A hash-sealed Merkle-DAG amendment chain (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerkleDagAmendmentChainV1 {
    pub nodes: Vec<AmendmentNode>,
    /// Hash over every node hash in order — the whole-DAG fingerprint.
    pub root_hash: String,
}

impl MerkleDagAmendmentChainV1 {
    /// H(id, content_hash, parents' node_hashes). Parents are resolved through `node_hash_of`.
    fn node_hash(
        id: &str,
        content_hash: &str,
        parents: &[String],
        node_hash_of: &BTreeMap<String, String>,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"merkle_dag_amendment_node_v1");
        h.field("id", id.as_bytes());
        h.field("content_hash", content_hash.as_bytes());
        for p in parents {
            // An unknown parent contributes its id (still binds), but a well-formed DAG resolves every parent.
            h.field(
                "parent_node_hash",
                node_hash_of
                    .get(p)
                    .map(|s| s.as_str())
                    .unwrap_or(p)
                    .as_bytes(),
            );
        }
        h.finalize_hex()
    }

    fn root_hash(nodes: &[AmendmentNode]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"merkle_dag_amendment_chain_v1");
        for n in nodes {
            h.field("node_hash", n.node_hash.as_bytes());
        }
        h.finalize_hex()
    }

    /// Build the DAG from `(id, content_hash, parent_ids)` rows in **topological order** (every parent appears
    /// before its children). Computes each node's Merkle hash and the whole-DAG `root_hash`.
    pub fn build(rows: &[(String, String, Vec<String>)]) -> Self {
        let mut node_hash_of: BTreeMap<String, String> = BTreeMap::new();
        let mut nodes = Vec::with_capacity(rows.len());
        for (id, content_hash, parents) in rows {
            let node_hash = Self::node_hash(id, content_hash, parents, &node_hash_of);
            node_hash_of.insert(id.clone(), node_hash.clone());
            nodes.push(AmendmentNode {
                id: id.clone(),
                content_hash: content_hash.clone(),
                parents: parents.clone(),
                node_hash,
            });
        }
        let root_hash = Self::root_hash(&nodes);
        MerkleDagAmendmentChainV1 { nodes, root_hash }
    }

    fn find(&self, id: &str) -> Option<&AmendmentNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Lineage proof for `id`: the transitive set of ancestor **node hashes** (including the node's own),
    /// sorted + deduplicated — the minimal evidence to authenticate the node back to the roots, shareable on
    /// its own (selective disclosure of one amendment's lineage without exposing unrelated siblings).
    pub fn lineage_proof(&self, id: &str) -> Vec<String> {
        let mut acc = Vec::new();
        let mut stack = vec![id.to_string()];
        while let Some(cur) = stack.pop() {
            if let Some(node) = self.find(&cur) {
                acc.push(node.node_hash.clone());
                for p in &node.parents {
                    stack.push(p.clone());
                }
            }
        }
        acc.sort();
        acc.dedup();
        acc
    }

    /// Re-derive every node hash (resolving parents in order) and the root hash; compare the whole record.
    /// Tampering any node's content hash changes it and every descendant's node hash → detected.
    pub fn verify(&self) -> bool {
        let rows: Vec<(String, String, Vec<String>)> = self
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n.content_hash.clone(), n.parents.clone()))
            .collect();
        Self::build(&rows) == *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    fn dag() -> MerkleDagAmendmentChainV1 {
        // a (root) → b, a → c, then d merges b + c.
        MerkleDagAmendmentChainV1::build(&[
            (s("a"), s("ha"), vec![]),
            (s("b"), s("hb"), vec![s("a")]),
            (s("c"), s("hc"), vec![s("a")]),
            (s("d"), s("hd"), vec![s("b"), s("c")]),
        ])
    }

    #[test]
    fn dag_builds_lineage_and_self_verifies() {
        let g = dag();
        assert_eq!(g.nodes.len(), 4);
        assert_eq!(g.root_hash.len(), 64);
        assert!(g.verify());
        // d's lineage includes d, b, c, a (4 distinct node hashes).
        assert_eq!(g.lineage_proof("d").len(), 4);
        // b's lineage is just b + a.
        assert_eq!(g.lineage_proof("b").len(), 2);
    }

    #[test]
    fn tampering_an_ancestor_breaks_descendants() {
        let mut g = dag();
        assert!(g.verify());
        // Forge the root 'a' content hash; 'a' and everything downstream should fail re-derivation.
        g.nodes[0].content_hash = s("FORGED");
        assert!(!g.verify());
    }

    #[test]
    fn merge_node_depends_on_both_parents() {
        let g = dag();
        // Changing either parent changes d's node hash → different DAG (proven via a rebuild with one parent edited).
        let alt = MerkleDagAmendmentChainV1::build(&[
            (s("a"), s("ha"), vec![]),
            (s("b"), s("hb_changed"), vec![s("a")]),
            (s("c"), s("hc"), vec![s("a")]),
            (s("d"), s("hd"), vec![s("b"), s("c")]),
        ]);
        let d_orig = g.find("d").unwrap().node_hash.clone();
        let d_alt = alt.find("d").unwrap().node_hash.clone();
        assert_ne!(d_orig, d_alt, "d's hash must depend on parent b's lineage");
    }
}
