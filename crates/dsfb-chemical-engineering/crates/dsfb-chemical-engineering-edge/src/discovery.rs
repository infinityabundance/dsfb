//! `SignatureDiscoveryAssistantV1` (Wave-7 semiotics) — a **human-guided** assistant that mines recurring
//! `StructuralUnmapped` episodes into *candidate* new heuristic signatures, by clustering them on shape and
//! ranking the clusters by recurrence.
//!
//! When the bank emits `unknown` repeatedly for episodes of a similar shape, that recurrence is a signal that
//! a *new* signature may be worth defining. This assistant groups unmapped episodes by a shape key, keeps
//! clusters with at least `min_support` members, and ranks them by support (a counterfactual proxy: the more
//! often a shape recurred, the more value a new heuristic for it would have added). It proposes; a human
//! defines and approves the actual heuristic.
//!
//! Bounded (non-claims): the output is a **ranked list of candidate clusters for human review, NOT
//! auto-generated heuristics** — DSFB does not learn or add a heuristic on its own; nothing here changes the
//! frozen atlas. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One candidate signature cluster: a shape key, the unmapped episodes that share it, and its support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateCluster {
    /// The shape key the members share (e.g. an episode shape hash or a coarse token signature).
    pub shape_key: String,
    /// The episode refs in this cluster (sorted, deterministic).
    pub members: Vec<String>,
    pub support: usize,
}

/// A hash-sealed signature-discovery proposal (schema v1): candidate clusters ranked by recurrence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureDiscoveryAssistantV1 {
    pub min_support: usize,
    /// Candidate clusters with `support ≥ min_support`, ranked by descending support (ties by shape key).
    pub clusters: Vec<CandidateCluster>,
    /// Unmapped episodes whose shape did not recur enough to form a cluster (the long tail).
    pub n_singletons: usize,
    pub non_claim: String,
    pub proposal_hash: String,
}

impl SignatureDiscoveryAssistantV1 {
    const NON_CLAIM: &'static str =
        "candidate clusters for HUMAN review only; DSFB does not auto-generate or add heuristics, and nothing here changes the frozen atlas";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"signature_discovery_assistant_v1");
        h.u64("min_support", self.min_support as u64);
        for c in &self.clusters {
            h.field("shape_key", c.shape_key.as_bytes());
            for m in &c.members {
                h.field("member", m.as_bytes());
            }
            h.u64("support", c.support as u64);
        }
        h.u64("n_singletons", self.n_singletons as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Mine `(episode_ref, shape_key)` unmapped episodes into candidate clusters. Episodes sharing a shape key
    /// group together; clusters reaching `min_support` are proposed (ranked by support), the rest counted as
    /// singletons (the long tail not yet worth a heuristic).
    pub fn build(unmapped: &[(String, String)], min_support: usize) -> Self {
        let mut by_shape: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (episode_ref, shape_key) in unmapped {
            by_shape
                .entry(shape_key.clone())
                .or_default()
                .push(episode_ref.clone());
        }
        let min_support = min_support.max(2); // a "recurrence" needs ≥ 2
        let mut clusters = Vec::new();
        let mut n_singletons = 0usize;
        for (shape_key, mut members) in by_shape {
            members.sort();
            members.dedup();
            if members.len() >= min_support {
                let support = members.len();
                clusters.push(CandidateCluster {
                    shape_key,
                    members,
                    support,
                });
            } else {
                n_singletons += members.len();
            }
        }
        // Rank by descending support; ties by shape key ascending (deterministic).
        clusters.sort_by(|a, b| {
            b.support
                .cmp(&a.support)
                .then_with(|| a.shape_key.cmp(&b.shape_key))
        });
        let mut p = SignatureDiscoveryAssistantV1 {
            min_support,
            clusters,
            n_singletons,
            non_claim: Self::NON_CLAIM.into(),
            proposal_hash: String::new(),
        };
        p.proposal_hash = p.seal();
        p
    }

    /// The top candidate signature (most-recurring unmapped shape), if any.
    pub fn top_candidate(&self) -> Option<&CandidateCluster> {
        self.clusters.first()
    }

    pub fn verify(&self) -> bool {
        let sorted = self
            .clusters
            .windows(2)
            .all(|w| w[0].support >= w[1].support);
        let above_min = self.clusters.iter().all(|c| c.support >= self.min_support);
        sorted
            && above_min
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.proposal_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn clusters_recurring_shapes_and_ranks_by_support() {
        // Shape "X" recurs 3×, "Y" 2×, "Z" once (singleton).
        let unmapped = vec![
            (s("e1"), s("X")),
            (s("e2"), s("X")),
            (s("e3"), s("X")),
            (s("e4"), s("Y")),
            (s("e5"), s("Y")),
            (s("e6"), s("Z")),
        ];
        let p = SignatureDiscoveryAssistantV1::build(&unmapped, 2);
        assert_eq!(p.clusters.len(), 2); // X and Y; Z is a singleton
        assert_eq!(p.top_candidate().unwrap().shape_key, "X"); // highest support
        assert_eq!(p.top_candidate().unwrap().support, 3);
        assert_eq!(p.n_singletons, 1); // Z
        assert!(p.non_claim.contains("does not auto-generate"));
        assert!(p.proposal_hash.len() == 64 && p.verify());
    }

    #[test]
    fn no_recurrence_yields_no_candidates() {
        let unmapped = vec![(s("e1"), s("A")), (s("e2"), s("B")), (s("e3"), s("C"))];
        let p = SignatureDiscoveryAssistantV1::build(&unmapped, 2);
        assert!(p.clusters.is_empty() && p.n_singletons == 3);
        assert!(p.top_candidate().is_none() && p.verify());
    }

    #[test]
    fn tampering_a_cluster_breaks_the_seal() {
        let unmapped = vec![(s("e1"), s("X")), (s("e2"), s("X"))];
        let mut p = SignatureDiscoveryAssistantV1::build(&unmapped, 2);
        assert!(p.verify());
        p.clusters[0].support = 99; // forge inflated support
        assert!(!p.verify());
    }
}
