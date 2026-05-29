//! `FaultPropagationWitnessV1` + `CausalNonClaimGraphV1` (P64).
//!
//! When a fault appears in an upstream unit and then in a downstream unit a residence-time later, that
//! temporal+topological co-occurrence is *evidence of propagation* — but DSFB asserts **no root cause**,
//! so it must be recorded as a candidate, never a causal proof. These two objects make that discipline
//! explicit and hash-sealed:
//!
//! - [`FaultPropagationWitnessV1`] — records an upstream onset, a downstream onset, the observed lag,
//!   the declared residence lag, whether they are consistent (within tolerance), and a **mandatory
//!   non-causal disclaimer**. A consistent lag raises a propagation *candidate*, nothing more.
//! - [`CausalNonClaimGraphV1`] — a graph whose edges carry temporal precedence + topological adjacency
//!   and whose every rendering states, in the graph itself, that **no causal claim is made**. It is the
//!   anti-overclaim object: it shows what *precedes* and what is *upstream*, and refuses to call that
//!   causation.
//!
//! Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The disclaimer attached to every propagation/causal-non-claim object. Single source of truth.
pub const NON_CAUSAL_DISCLAIMER: &str =
    "Temporal precedence + topological adjacency only. This is a propagation CANDIDATE, not a causal \
     proof: DSFB asserts no root cause. Confounders (common-cause upstream disturbance, shared sensor \
     drift, coincidental timing) are not excluded.";

/// A hash-sealed fault-propagation witness (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultPropagationWitnessV1 {
    pub upstream_unit: String,
    pub downstream_unit: String,
    /// Sample index where the fault onset was observed upstream.
    pub upstream_onset: usize,
    /// Sample index where it was observed downstream.
    pub downstream_onset: usize,
    /// `downstream_onset − upstream_onset` (0 if downstream is not later).
    pub observed_lag: usize,
    /// Residence lag in samples expected from the declared residence time.
    pub declared_residence_lag: usize,
    /// `|observed − declared| ≤ tolerance` — the lag is consistent with the declared residence time.
    pub lag_consistent: bool,
    /// Tolerance (in samples) used for the consistency test.
    pub tolerance_samples: usize,
    /// Mandatory non-causal disclaimer (always [`NON_CAUSAL_DISCLAIMER`]).
    pub non_causal_disclaimer: String,
    pub witness_hash: String,
}

impl FaultPropagationWitnessV1 {
    #[allow(clippy::too_many_arguments)]
    fn seal(
        up: &str,
        down: &str,
        uo: usize,
        do_: usize,
        obs: usize,
        decl: usize,
        consistent: bool,
        tol: usize,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"fault_propagation_witness_v1");
        h.field("upstream_unit", up.as_bytes());
        h.field("downstream_unit", down.as_bytes());
        h.u64("upstream_onset", uo as u64);
        h.u64("downstream_onset", do_ as u64);
        h.u64("observed_lag", obs as u64);
        h.u64("declared_residence_lag", decl as u64);
        h.u64("lag_consistent", consistent as u64);
        h.u64("tolerance_samples", tol as u64);
        h.field("disclaimer", NON_CAUSAL_DISCLAIMER.as_bytes());
        h.finalize_hex()
    }

    /// Build a propagation witness from the two onsets + the declared residence lag + a tolerance.
    /// `observed_lag` is `downstream_onset − upstream_onset` (0 if downstream is not strictly later).
    pub fn build(
        upstream_unit: impl Into<String>,
        downstream_unit: impl Into<String>,
        upstream_onset: usize,
        downstream_onset: usize,
        declared_residence_lag: usize,
        tolerance_samples: usize,
    ) -> Self {
        let observed_lag = downstream_onset.saturating_sub(upstream_onset);
        let lag_consistent = downstream_onset >= upstream_onset
            && observed_lag.abs_diff(declared_residence_lag) <= tolerance_samples;
        let up = upstream_unit.into();
        let down = downstream_unit.into();
        let witness_hash = Self::seal(
            &up,
            &down,
            upstream_onset,
            downstream_onset,
            observed_lag,
            declared_residence_lag,
            lag_consistent,
            tolerance_samples,
        );
        FaultPropagationWitnessV1 {
            upstream_unit: up,
            downstream_unit: down,
            upstream_onset,
            downstream_onset,
            observed_lag,
            declared_residence_lag,
            lag_consistent,
            tolerance_samples,
            non_causal_disclaimer: NON_CAUSAL_DISCLAIMER.to_string(),
            witness_hash,
        }
    }

    pub fn verify(&self) -> bool {
        self.non_causal_disclaimer == NON_CAUSAL_DISCLAIMER
            && Self::seal(
                &self.upstream_unit,
                &self.downstream_unit,
                self.upstream_onset,
                self.downstream_onset,
                self.observed_lag,
                self.declared_residence_lag,
                self.lag_consistent,
                self.tolerance_samples,
            ) == self.witness_hash
    }
}

/// One edge of the causal-non-claim graph: precedence + topology, explicitly NOT causation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalNonClaimEdge {
    pub from: String,
    pub to: String,
    /// `to_onset − from_onset` (temporal precedence; positive ⇒ `from` precedes `to`).
    pub precedence_lag: i64,
    /// Whether `from` is topologically upstream of `to` (from the process topology graph).
    pub topologically_upstream: bool,
}

/// A hash-sealed causal-NON-claim graph (schema v1) — the anti-overclaim object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalNonClaimGraphV1 {
    pub plant: String,
    pub edges: Vec<CausalNonClaimEdge>,
    /// Always [`NON_CAUSAL_DISCLAIMER`]; rendered into every output.
    pub disclaimer: String,
    pub graph_hash: String,
}

impl CausalNonClaimGraphV1 {
    fn seal(plant: &str, edges: &[CausalNonClaimEdge]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"causal_non_claim_graph_v1");
        h.field("plant", plant.as_bytes());
        for e in edges {
            h.field("from", e.from.as_bytes());
            h.field("to", e.to.as_bytes());
            h.u64("precedence_lag", e.precedence_lag as u64);
            h.u64("topologically_upstream", e.topologically_upstream as u64);
        }
        h.field("disclaimer", NON_CAUSAL_DISCLAIMER.as_bytes());
        h.finalize_hex()
    }

    pub fn build(plant: impl Into<String>, edges: Vec<CausalNonClaimEdge>) -> Self {
        let plant = plant.into();
        let graph_hash = Self::seal(&plant, &edges);
        CausalNonClaimGraphV1 {
            plant,
            edges,
            disclaimer: NON_CAUSAL_DISCLAIMER.to_string(),
            graph_hash,
        }
    }

    pub fn verify(&self) -> bool {
        self.disclaimer == NON_CAUSAL_DISCLAIMER
            && Self::seal(&self.plant, &self.edges) == self.graph_hash
    }

    /// Render as Graphviz DOT with **dashed** edges (to signal non-causality) and the disclaimer printed
    /// as the graph label — the non-claim is part of the artifact, not a footnote.
    pub fn to_dot(&self) -> String {
        let esc = |s: &str| s.replace('"', "'");
        let mut out = format!(
            "digraph causal_non_claim {{\n  label=\"{} — NO CAUSAL CLAIM: {}\";\n  rankdir=LR;\n",
            esc(&self.plant),
            esc(NON_CAUSAL_DISCLAIMER)
        );
        for e in &self.edges {
            out.push_str(&format!(
                "  \"{}\" -> \"{}\" [style=dashed, label=\"precedes by {}{}\"];\n",
                esc(&e.from),
                esc(&e.to),
                e.precedence_lag,
                if e.topologically_upstream {
                    ", upstream"
                } else {
                    ""
                }
            ));
        }
        out.push_str("}\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagation_witness_flags_consistent_lag_and_keeps_disclaimer() {
        // reactor onset @ 100, separator onset @ 105; declared residence lag 5 samples, tol 2 → consistent.
        let w = FaultPropagationWitnessV1::build("reactor", "separator", 100, 105, 5, 2);
        assert_eq!(w.observed_lag, 5);
        assert!(w.lag_consistent);
        assert_eq!(w.non_causal_disclaimer, NON_CAUSAL_DISCLAIMER);
        assert!(w.verify());
        // An onset lag far from the declared residence is not consistent.
        let w2 = FaultPropagationWitnessV1::build("reactor", "separator", 100, 140, 5, 2);
        assert!(!w2.lag_consistent && w2.verify());
    }

    #[test]
    fn causal_non_claim_graph_renders_the_disclaimer_and_self_verifies() {
        let edges = vec![
            CausalNonClaimEdge {
                from: "feed".into(),
                to: "reactor".into(),
                precedence_lag: 3,
                topologically_upstream: true,
            },
            CausalNonClaimEdge {
                from: "reactor".into(),
                to: "separator".into(),
                precedence_lag: 5,
                topologically_upstream: true,
            },
        ];
        let g = CausalNonClaimGraphV1::build("feed_reactor_separator", edges);
        assert!(g.verify());
        let dot = g.to_dot();
        assert!(dot.contains("NO CAUSAL CLAIM"));
        assert!(dot.contains("style=dashed")); // non-causal edges rendered dashed
                                               // Tampering with the disclaimer breaks verification (the non-claim is sealed).
        let mut t = g.clone();
        t.disclaimer = "this proves causation".into();
        assert!(!t.verify());
    }
}
