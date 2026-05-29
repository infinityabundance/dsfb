//! `MultiPhysicsCrossWitnessV1` (Wave-7 physics) — a joint grammar over **multiple physics domains**
//! (chemical, mechanical/vibration, thermal, acoustic) that recognises **cross-domain motifs**: a fault that
//! shows up coherently across domains (e.g. an acoustic spike → flow drop → composition shift = cavitation)
//! is far stronger evidence than any single-domain firing.
//!
//! Each domain contributes a firing (a labelled residual breach). A `CrossMotif` declares which domains must
//! co-fire to constitute a known multi-physics signature; the witness records which motifs matched and how
//! many domains lit up.
//!
//! Bounded (non-claims): a matched motif is a **co-occurrence pattern across domains, not a proven causal
//! chain** — "acoustic → flow → composition" is a candidate cavitation signature to investigate, never proof
//! of the mechanism or its direction. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// A physics domain a residual family belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsDomain {
    Chemical,
    Mechanical,
    Thermal,
    Acoustic,
}

impl PhysicsDomain {
    pub fn tag(self) -> &'static str {
        match self {
            PhysicsDomain::Chemical => "chemical",
            PhysicsDomain::Mechanical => "mechanical",
            PhysicsDomain::Thermal => "thermal",
            PhysicsDomain::Acoustic => "acoustic",
        }
    }
}

/// One domain's firing (a labelled residual breach in that domain).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainFiring {
    pub domain: String,
    pub label: String,
    pub fired: bool,
}

impl DomainFiring {
    pub fn new(domain: PhysicsDomain, label: impl Into<String>, fired: bool) -> Self {
        DomainFiring {
            domain: domain.tag().into(),
            label: label.into(),
            fired,
        }
    }
}

/// A declared cross-domain motif: the set of domains that must co-fire, and its (candidate) interpretation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossMotif {
    pub name: String,
    pub required_domains: Vec<PhysicsDomain>,
    pub interpretation: String,
}

/// A hash-sealed multi-physics cross-witness (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPhysicsCrossWitnessV1 {
    pub firings: Vec<DomainFiring>,
    /// Names of the motifs whose every required domain fired.
    pub matched_motifs: Vec<String>,
    /// Distinct domains that fired.
    pub n_domains_fired: usize,
    pub non_claim: String,
    pub witness_hash: String,
}

impl MultiPhysicsCrossWitnessV1 {
    const NON_CLAIM: &'static str =
        "a matched motif is a cross-domain co-occurrence pattern, NOT a proven causal chain or its direction";

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"multi_physics_cross_witness_v1");
        for f in &self.firings {
            h.field("domain", f.domain.as_bytes());
            h.field("label", f.label.as_bytes());
            h.u64("fired", f.fired as u64);
        }
        for m in &self.matched_motifs {
            h.field("motif", m.as_bytes());
        }
        h.u64("n_domains_fired", self.n_domains_fired as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// True iff every required domain of `motif` has a fired [`DomainFiring`] in `firings`.
    fn motif_matches(motif: &CrossMotif, firings: &[DomainFiring]) -> bool {
        motif
            .required_domains
            .iter()
            .all(|d| firings.iter().any(|f| f.domain == d.tag() && f.fired))
    }

    /// Build from per-domain firings + the declared motif catalogue.
    pub fn build(firings: Vec<DomainFiring>, motifs: &[CrossMotif]) -> Self {
        let matched_motifs: Vec<String> = motifs
            .iter()
            .filter(|m| Self::motif_matches(m, &firings))
            .map(|m| m.name.clone())
            .collect();
        let mut fired_domains: Vec<&str> = firings
            .iter()
            .filter(|f| f.fired)
            .map(|f| f.domain.as_str())
            .collect();
        fired_domains.sort();
        fired_domains.dedup();
        let n_domains_fired = fired_domains.len();
        let mut w = MultiPhysicsCrossWitnessV1 {
            firings,
            matched_motifs,
            n_domains_fired,
            non_claim: Self::NON_CLAIM.into(),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// True iff at least one cross-domain motif matched (multi-physics corroboration).
    pub fn has_cross_domain_evidence(&self) -> bool {
        !self.matched_motifs.is_empty()
    }

    pub fn verify(&self) -> bool {
        self.non_claim == Self::NON_CLAIM && self.seal() == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PhysicsDomain::*;

    fn cavitation_motif() -> CrossMotif {
        CrossMotif {
            name: "cavitation_signature".into(),
            required_domains: vec![Acoustic, Hydraulic_as_chemical_flow(), Chemical],
            interpretation: "acoustic spike co-occurring with a flow drop and a composition shift"
                .into(),
        }
    }
    // (flow lives in the chemical/process domain here; the helper keeps the test readable.)
    #[allow(non_snake_case)]
    fn Hydraulic_as_chemical_flow() -> PhysicsDomain {
        PhysicsDomain::Mechanical
    }

    #[test]
    fn cross_motif_matches_only_when_all_domains_fire() {
        let motifs = [cavitation_motif()];
        // All three domains fire → motif matches.
        let firings = vec![
            DomainFiring::new(Acoustic, "broadband_spike", true),
            DomainFiring::new(Mechanical, "flow_drop", true),
            DomainFiring::new(Chemical, "composition_shift", true),
            DomainFiring::new(Thermal, "temp_ok", false),
        ];
        let w = MultiPhysicsCrossWitnessV1::build(firings, &motifs);
        assert!(w.has_cross_domain_evidence());
        assert_eq!(w.matched_motifs, vec!["cavitation_signature".to_string()]);
        assert_eq!(w.n_domains_fired, 3);
        assert!(w.non_claim.contains("NOT a proven causal chain"));
        assert!(w.witness_hash.len() == 64 && w.verify());
    }

    #[test]
    fn partial_firing_does_not_match_the_motif() {
        let motifs = [cavitation_motif()];
        // Only acoustic fires → no cross-domain corroboration.
        let firings = vec![
            DomainFiring::new(Acoustic, "broadband_spike", true),
            DomainFiring::new(Mechanical, "flow_ok", false),
            DomainFiring::new(Chemical, "comp_ok", false),
        ];
        let w = MultiPhysicsCrossWitnessV1::build(firings, &motifs);
        assert!(!w.has_cross_domain_evidence() && w.matched_motifs.is_empty());
        assert_eq!(w.n_domains_fired, 1);
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let firings = vec![DomainFiring::new(Acoustic, "x", true)];
        let mut w = MultiPhysicsCrossWitnessV1::build(firings, &[]);
        assert!(w.verify());
        w.matched_motifs.push("forged".into());
        assert!(!w.verify());
    }
}
