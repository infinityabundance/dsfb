//! `PhysicalWitnessStrengthV1` (Wave-2 P74) — a deterministic ladder that ranks *physical* evidence above
//! *advisory structure*.
//!
//! Not every witness is equally grounded. A closed mass/energy balance is first-principles physics; a
//! detector-family quorum is statistical agreement; a precedent-similarity hit is only a retrieval hint. A
//! chemical engineer reading a case file needs to see that ordering at a glance, and the framework needs it
//! so a strong physical witness is never out-ranked by a weak structural one. This module assigns each
//! witness/episode a rung on a fixed, totally-ordered ladder and seals the assignment.
//!
//! Bounded: a rung is *evidence strength*, never a root-cause or causal claim — even `BalanceClosure` (the
//! top rung) only attests that a conservation law's closure broke, not *why*. Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The physical-witness strength ladder, strongest first. `Ord` follows the ladder: a higher rung compares
/// `>` a lower one (so the strongest witness for an episode is `max`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhysicalWitnessStrength {
    /// Weakest — only resembles a past episode's structural shape (advisory retrieval hint).
    PrecedentSimilarityOnly,
    /// A single heuristic's pattern matched, with no independent corroboration.
    HeuristicPatternOnly,
    /// A quorum of independent detector *families* fired (statistical agreement, not physics).
    DetectorFamilyQuorum,
    /// Consistent with the recorded control action (PV/MV/SP context corroborates the structure).
    ControlActionConsistent,
    /// Upstream→downstream onset aligned with declared residence time (topology-consistent timing).
    TopologyResidenceAligned,
    /// Strongest — a closed mass/energy/species balance's closure residual broke (first-principles physics).
    BalanceClosure,
}

impl PhysicalWitnessStrength {
    /// Rung number 1..=6 (1 = weakest precedent-similarity, 6 = strongest balance closure).
    pub fn rung(self) -> u8 {
        match self {
            PhysicalWitnessStrength::PrecedentSimilarityOnly => 1,
            PhysicalWitnessStrength::HeuristicPatternOnly => 2,
            PhysicalWitnessStrength::DetectorFamilyQuorum => 3,
            PhysicalWitnessStrength::ControlActionConsistent => 4,
            PhysicalWitnessStrength::TopologyResidenceAligned => 5,
            PhysicalWitnessStrength::BalanceClosure => 6,
        }
    }

    /// Stable tag fed to the hasher / rendered.
    pub fn tag(self) -> &'static str {
        match self {
            PhysicalWitnessStrength::PrecedentSimilarityOnly => "PrecedentSimilarityOnly",
            PhysicalWitnessStrength::HeuristicPatternOnly => "HeuristicPatternOnly",
            PhysicalWitnessStrength::DetectorFamilyQuorum => "DetectorFamilyQuorum",
            PhysicalWitnessStrength::ControlActionConsistent => "ControlActionConsistent",
            PhysicalWitnessStrength::TopologyResidenceAligned => "TopologyResidenceAligned",
            PhysicalWitnessStrength::BalanceClosure => "BalanceClosure",
        }
    }

    /// True iff this rung is grounded in first-principles physics (balance closure or residence-time
    /// topology) rather than purely statistical/structural agreement — the line a chemical engineer cares about.
    pub fn is_physical(self) -> bool {
        matches!(
            self,
            PhysicalWitnessStrength::BalanceClosure
                | PhysicalWitnessStrength::TopologyResidenceAligned
        )
    }
}

/// A hash-sealed per-episode witness-strength assessment (schema v1): the set of witness rungs that support
/// an episode and the resulting *dominant* (strongest) rung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessStrengthAssessmentV1 {
    pub episode_ref: String,
    /// The supporting witness rungs (tags), in the order supplied.
    pub witnesses: Vec<String>,
    /// The strongest rung present (the one an operator should weigh most), as a tag.
    pub dominant: String,
    /// True iff the dominant rung is physics-grounded.
    pub dominant_is_physical: bool,
    pub assessment_hash: String,
}

impl WitnessStrengthAssessmentV1 {
    /// Build from the episode's supporting rungs; the dominant rung is their maximum on the ladder.
    pub fn build(episode_ref: impl Into<String>, witnesses: &[PhysicalWitnessStrength]) -> Self {
        let episode_ref = episode_ref.into();
        let dominant = witnesses
            .iter()
            .copied()
            .max()
            .unwrap_or(PhysicalWitnessStrength::PrecedentSimilarityOnly);
        let tags: Vec<String> = witnesses.iter().map(|w| w.tag().to_string()).collect();
        let mut h = CanonicalHasher::new();
        h.field("schema", b"witness_strength_assessment_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        for w in witnesses {
            h.field("witness", w.tag().as_bytes());
            h.u64("rung", w.rung() as u64);
        }
        h.field("dominant", dominant.tag().as_bytes());
        WitnessStrengthAssessmentV1 {
            episode_ref,
            witnesses: tags,
            dominant: dominant.tag().into(),
            dominant_is_physical: dominant.is_physical(),
            assessment_hash: h.finalize_hex(),
        }
    }

    pub fn verify(&self) -> bool {
        // Re-derive from the stored witness tags and compare the WHOLE record — this catches tampering of
        // the derived fields (`dominant`, `dominant_is_physical`) as well as the hash, since those fields
        // are a pure function of `witnesses` and full-struct equality re-checks them.
        let rungs: Vec<PhysicalWitnessStrength> =
            self.witnesses.iter().filter_map(|t| from_tag(t)).collect();
        if rungs.len() != self.witnesses.len() {
            return false; // an unrecognised tag means the record was tampered
        }
        Self::build(self.episode_ref.clone(), &rungs) == *self
    }
}

/// Parse a rung tag back to its enum (for `verify`).
fn from_tag(t: &str) -> Option<PhysicalWitnessStrength> {
    use PhysicalWitnessStrength::*;
    Some(match t {
        "PrecedentSimilarityOnly" => PrecedentSimilarityOnly,
        "HeuristicPatternOnly" => HeuristicPatternOnly,
        "DetectorFamilyQuorum" => DetectorFamilyQuorum,
        "ControlActionConsistent" => ControlActionConsistent,
        "TopologyResidenceAligned" => TopologyResidenceAligned,
        "BalanceClosure" => BalanceClosure,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use PhysicalWitnessStrength::*;

    #[test]
    fn ladder_orders_physics_above_structure() {
        assert!(BalanceClosure > DetectorFamilyQuorum);
        assert!(TopologyResidenceAligned > HeuristicPatternOnly);
        assert!(DetectorFamilyQuorum > PrecedentSimilarityOnly);
        assert!(BalanceClosure.is_physical() && TopologyResidenceAligned.is_physical());
        assert!(!DetectorFamilyQuorum.is_physical() && !HeuristicPatternOnly.is_physical());
    }

    #[test]
    fn dominant_is_the_strongest_rung_and_self_verifies() {
        // An episode supported by a balance closure + a detector quorum: the balance closure dominates.
        let a = WitnessStrengthAssessmentV1::build(
            "idx=120..168",
            &[DetectorFamilyQuorum, BalanceClosure, HeuristicPatternOnly],
        );
        assert_eq!(a.dominant, "BalanceClosure");
        assert!(a.dominant_is_physical);
        assert!(a.verify());
        // A purely structural episode: dominant is the quorum, not physical.
        let b = WitnessStrengthAssessmentV1::build(
            "idx=10..12",
            &[PrecedentSimilarityOnly, DetectorFamilyQuorum],
        );
        assert_eq!(b.dominant, "DetectorFamilyQuorum");
        assert!(!b.dominant_is_physical && b.verify());
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut a = WitnessStrengthAssessmentV1::build("e", &[DetectorFamilyQuorum]);
        assert!(a.verify());
        a.dominant = "BalanceClosure".into(); // forge a stronger dominant than the witnesses support
        assert!(!a.verify());
    }
}
