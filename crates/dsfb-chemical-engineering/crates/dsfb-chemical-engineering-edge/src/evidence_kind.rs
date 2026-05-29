//! `EvidenceKind` (panel P82) — a typed vocabulary of *what kind of evidence* a statement rests on, linked
//! to the two strength ladders the framework already defines.
//!
//! The project already has two orthogonal "how strong" axes:
//!   - [`crate::claim_strength::ClaimStrength`] — *how strongly a claim may be made* (the paper's Tier-1/2/3
//!     hierarchy + an explicit `NonClaim`), assigned from a free-text `evidence_kind` string.
//!   - [`crate::witness_strength::PhysicalWitnessStrength`] — *how physically grounded* a witness is (the
//!     6-rung ladder from precedent-similarity up to first-principles balance closure).
//!
//! What was missing was a **typed enumeration of the evidence kinds themselves** — so a generated statement
//! can carry a single `EvidenceKind` value and *derive* both its claim tier and (where applicable) its
//! physical-witness rung deterministically, instead of passing a stringly-typed `evidence_kind` around.
//!
//! Design: [`EvidenceKind::claim_strength`] **delegates to `ClaimStrength::for_evidence_kind`** via
//! [`EvidenceKind::as_evidence_key`] — the existing string map stays the single source of truth, so this
//! enum can never disagree with the tier a free-text statement would get (a unit test pins that). The
//! physical-witness mapping follows the chemical-evidence ladder (balance/topology are physics; detector/
//! heuristic/precedent are statistical or advisory). Additive, off the replay path, no overclaiming.

use serde::{Deserialize, Serialize};

use crate::claim_strength::ClaimStrength;
use crate::witness_strength::PhysicalWitnessStrength;

/// The kind of evidence a generated statement or episode finding rests on. Twelve kinds spanning the
/// chemical evidence surface: first-principles physics, instrumentation/data health, process context,
/// chemometric detection, heuristic/precedent reasoning, and the human/derived layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    /// A mass/energy/species balance's closure residual (first-principles physics).
    PhysicalBalance,
    /// A first-principles equation residual (Arrhenius / Henry / heat-transfer / pump-curve, …).
    FirstPrinciplesEquation,
    /// Instrumentation/sensor-health evidence (calibration, frozen-tag, sensor-trust degradation).
    InstrumentationHealth,
    /// Controller / loop context (PV/MV/SP, mode, NE 107 status) corroborating an episode.
    ControllerContext,
    /// Process-topology / residence-time / propagation structure across units.
    ProcessTopology,
    /// Data-quality evidence (missingness, duplicate timestamps, out-of-range, clock skew).
    DatasetQuality,
    /// A historian-import receipt (file digest, row/tag counts, time range) — a sealed import fact.
    HistorianImport,
    /// A chemometric detector firing (the executed detector bank's per-sample exceedance witness).
    ChemometricDetector,
    /// A single process-heuristic pattern match (the H1–H6 bank), with no independent corroboration.
    HeuristicPattern,
    /// Structural similarity to a precedent episode (advisory retrieval hint only).
    PrecedentSimilarity,
    /// A human operator annotation attached to the case file (context, not a measurement).
    OperatorAnnotation,
    /// A deterministically-compiled narrative summary sentence (derived prose, evidence-anchored).
    NarrativeSummary,
}

impl EvidenceKind {
    /// All twelve kinds in declaration order (for enumeration / the navigation index).
    pub const ALL: [EvidenceKind; 12] = [
        EvidenceKind::PhysicalBalance,
        EvidenceKind::FirstPrinciplesEquation,
        EvidenceKind::InstrumentationHealth,
        EvidenceKind::ControllerContext,
        EvidenceKind::ProcessTopology,
        EvidenceKind::DatasetQuality,
        EvidenceKind::HistorianImport,
        EvidenceKind::ChemometricDetector,
        EvidenceKind::HeuristicPattern,
        EvidenceKind::PrecedentSimilarity,
        EvidenceKind::OperatorAnnotation,
        EvidenceKind::NarrativeSummary,
    ];

    /// Stable tag (snake_case; part of any seal — never localise or reorder).
    pub fn tag(self) -> &'static str {
        match self {
            EvidenceKind::PhysicalBalance => "physical_balance",
            EvidenceKind::FirstPrinciplesEquation => "first_principles_equation",
            EvidenceKind::InstrumentationHealth => "instrumentation_health",
            EvidenceKind::ControllerContext => "controller_context",
            EvidenceKind::ProcessTopology => "process_topology",
            EvidenceKind::DatasetQuality => "dataset_quality",
            EvidenceKind::HistorianImport => "historian_import",
            EvidenceKind::ChemometricDetector => "chemometric_detector",
            EvidenceKind::HeuristicPattern => "heuristic_pattern",
            EvidenceKind::PrecedentSimilarity => "precedent_similarity",
            EvidenceKind::OperatorAnnotation => "operator_annotation",
            EvidenceKind::NarrativeSummary => "narrative_summary",
        }
    }

    /// Parse a `tag()` back to its variant (round-trips with `tag`).
    pub fn from_tag(tag: &str) -> Option<EvidenceKind> {
        EvidenceKind::ALL.into_iter().find(|k| k.tag() == tag)
    }

    /// The canonical free-text `evidence_kind` key this kind maps onto in the existing claim-strength map
    /// (`ClaimStrength::for_evidence_kind`). Keeping this enum's tier derivation routed through that one map
    /// means the typed and stringly-typed paths can never disagree (pinned by a unit test).
    pub fn as_evidence_key(self) -> &'static str {
        match self {
            // First-principles physics is interpretation-grade (Tier 2): strong evidence, still not a sealed fact.
            EvidenceKind::PhysicalBalance | EvidenceKind::FirstPrinciplesEquation => "balance",
            // Sensor/data health + human context are advisory interpretations (Tier 2).
            EvidenceKind::InstrumentationHealth
            | EvidenceKind::DatasetQuality
            | EvidenceKind::OperatorAnnotation => "witness",
            // Controller/NE-107 context (Tier 2).
            EvidenceKind::ControllerContext => "ne107",
            // Topology/propagation is a bounded newly-demonstrated implication (Tier 3).
            EvidenceKind::ProcessTopology => "propagation_candidate",
            // A historian import receipt is a sealed file-digest fact (Tier 1).
            EvidenceKind::HistorianImport => "manifest",
            // Detector firing + heuristic match + compiled narrative are advisory interpretations (Tier 2).
            EvidenceKind::ChemometricDetector => "motif",
            EvidenceKind::HeuristicPattern => "heuristic",
            EvidenceKind::NarrativeSummary => "label",
            // Precedent similarity is a bounded retrieval-hint implication (Tier 3).
            EvidenceKind::PrecedentSimilarity => "precedent",
        }
    }

    /// The claim strength this evidence kind can support — derived from the single-source-of-truth map.
    pub fn claim_strength(self) -> ClaimStrength {
        ClaimStrength::for_evidence_kind(self.as_evidence_key())
    }

    /// The physical-witness rung this evidence kind corresponds to, when it maps onto the
    /// [`PhysicalWitnessStrength`] ladder. `None` for kinds that are not a process-witness rung
    /// (instrumentation/data health, historian import, operator annotation, narrative, raw equation residual).
    pub fn witness_strength(self) -> Option<PhysicalWitnessStrength> {
        match self {
            EvidenceKind::PhysicalBalance => Some(PhysicalWitnessStrength::BalanceClosure),
            EvidenceKind::ProcessTopology => {
                Some(PhysicalWitnessStrength::TopologyResidenceAligned)
            }
            EvidenceKind::ControllerContext => {
                Some(PhysicalWitnessStrength::ControlActionConsistent)
            }
            EvidenceKind::ChemometricDetector => {
                Some(PhysicalWitnessStrength::DetectorFamilyQuorum)
            }
            EvidenceKind::HeuristicPattern => Some(PhysicalWitnessStrength::HeuristicPatternOnly),
            EvidenceKind::PrecedentSimilarity => {
                Some(PhysicalWitnessStrength::PrecedentSimilarityOnly)
            }
            _ => None,
        }
    }

    /// The evidence kind a [`PhysicalWitnessStrength`] rung corresponds to — the inverse of
    /// [`Self::witness_strength`] over the six process-witness rungs (total: every rung maps to exactly one
    /// kind, and `witness_strength(from_witness_strength(w)) == Some(w)`, pinned by a unit test).
    pub fn from_witness_strength(w: PhysicalWitnessStrength) -> EvidenceKind {
        match w {
            PhysicalWitnessStrength::BalanceClosure => EvidenceKind::PhysicalBalance,
            PhysicalWitnessStrength::TopologyResidenceAligned => EvidenceKind::ProcessTopology,
            PhysicalWitnessStrength::ControlActionConsistent => EvidenceKind::ControllerContext,
            PhysicalWitnessStrength::DetectorFamilyQuorum => EvidenceKind::ChemometricDetector,
            PhysicalWitnessStrength::HeuristicPatternOnly => EvidenceKind::HeuristicPattern,
            PhysicalWitnessStrength::PrecedentSimilarityOnly => EvidenceKind::PrecedentSimilarity,
        }
    }

    /// True iff this kind is grounded in first-principles physics (delegates to the witness rung).
    pub fn is_physical(self) -> bool {
        self.witness_strength().is_some_and(|w| w.is_physical())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `from_witness_strength` is the exact inverse of `witness_strength` over the six process-witness rungs.
    #[test]
    fn witness_strength_round_trips() {
        use PhysicalWitnessStrength::*;
        for w in [
            BalanceClosure,
            TopologyResidenceAligned,
            ControlActionConsistent,
            DetectorFamilyQuorum,
            HeuristicPatternOnly,
            PrecedentSimilarityOnly,
        ] {
            assert_eq!(
                EvidenceKind::from_witness_strength(w).witness_strength(),
                Some(w),
                "round-trip must hold for {}",
                w.tag()
            );
        }
    }

    /// The typed enum's claim tier MUST equal what the free-text `evidence_kind` map yields — the typed and
    /// stringly-typed paths share one source of truth (`ClaimStrength::for_evidence_kind`).
    #[test]
    fn claim_strength_agrees_with_the_string_map() {
        for k in EvidenceKind::ALL {
            assert_eq!(
                k.claim_strength(),
                ClaimStrength::for_evidence_kind(k.as_evidence_key()),
                "{}: typed tier must match the string-map tier",
                k.tag()
            );
        }
    }

    /// Spot-check the intended tier assignments so a future map edit that silently changes them is caught.
    #[test]
    fn intended_tiers() {
        assert_eq!(
            EvidenceKind::HistorianImport.claim_strength(),
            ClaimStrength::SealedFact
        ); // Tier 1
        assert_eq!(
            EvidenceKind::PhysicalBalance.claim_strength(),
            ClaimStrength::EvidenceInterpretation
        ); // Tier 2
        assert_eq!(
            EvidenceKind::ProcessTopology.claim_strength(),
            ClaimStrength::SpeculativeImplication
        ); // Tier 3
        assert_eq!(
            EvidenceKind::PrecedentSimilarity.claim_strength(),
            ClaimStrength::SpeculativeImplication
        );
    }

    /// The two first-principles kinds that map onto physical witness rungs are physically grounded; the
    /// statistical/advisory kinds are witnesses but not physical; the human/derived/health kinds are None.
    #[test]
    fn physical_grounding_is_correct() {
        assert!(EvidenceKind::PhysicalBalance.is_physical());
        assert!(EvidenceKind::ProcessTopology.is_physical());
        assert!(!EvidenceKind::ChemometricDetector.is_physical()); // a witness rung, but statistical
        assert!(EvidenceKind::ChemometricDetector
            .witness_strength()
            .is_some());
        assert!(EvidenceKind::HeuristicPattern.witness_strength().is_some());
        assert!(EvidenceKind::NarrativeSummary.witness_strength().is_none());
        assert!(EvidenceKind::HistorianImport.witness_strength().is_none());
    }

    /// Tags are unique and round-trip through `from_tag`.
    #[test]
    fn tags_unique_and_round_trip() {
        let mut seen = std::collections::BTreeSet::new();
        for k in EvidenceKind::ALL {
            assert!(seen.insert(k.tag()), "duplicate tag {}", k.tag());
            assert_eq!(EvidenceKind::from_tag(k.tag()), Some(k));
        }
        assert_eq!(seen.len(), 12);
        assert_eq!(EvidenceKind::from_tag("nope"), None);
    }
}
