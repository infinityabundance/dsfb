//! T.6 — witness-role and fusion-axis semantics.
//!
//! T.1a/T.1b ship enum-typed `WitnessRole`, `NegativeWitnessKind`,
//! and `AxisBindingSet` on every detector. T.6 adds the LAW layer:
//! what role each detector is *allowed* to play, which fusion
//! plane(s) those roles map to, and which roles suppress or
//! corroborate which other roles. T.6 does NOT implement the
//! fusion engine itself (Section S Phase 1+) — it formalises the
//! evidence-court vocabulary the engine will consume.
//!
//! **Eight fusion planes** (panel-locked, Section S parent
//! taxonomy). Each plane groups a related family of fusion axes
//! and admissibility gates:
//!
//! - `ProvenanceAdmissibility` — input + schema + missingness +
//!   sample-size gates.
//! - `NumericStrength` — residual magnitude, slew shock, robust
//!   margins.
//! - `TemporalStructure` — persistence, drift, periodicity,
//!   recovery shape.
//! - `CrossSignalStructure` — entity locality, causal adjacency,
//!   topology support.
//! - `DistributionStructure` — distribution distance, divergence,
//!   distributional drift.
//! - `SemanticBankStructure` — bank-motif fit, witness-role
//!   coherence (CPU-only authority).
//! - `ReliabilityConfuserControl` — confuser suppression,
//!   clean-window support, transient rejection.
//! - `TaskUtility` — operator readability, dashboard relevance,
//!   delta contribution.
//!
//! **Existing v1 9-axis fusion** ([`crate::types::AxisBindingSet`])
//! maps onto these planes; this module makes the mapping explicit
//! and deterministic.
//!
//! T.6 explicitly does NOT do (panel-locked):
//!
//! - No 8-plane fusion engine implementation (Section S Phase 1+).
//! - No `CaseFileV2` integration (T.11).
//! - No `corpus_hash_v1` (T.10).
//! - No empirical usefulness claims (T.8).
//! - No free-text roles — everything stays enum-driven.

extern crate alloc;
use alloc::vec::Vec;

use crate::types::{AxisBindingSet, NegativeWitnessKind, WitnessRole};

/// The eight panel-locked fusion planes. Wire names match the Rust
/// variant names verbatim so the report + (future) paper render
/// the same vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FusionPlane {
    /// Input / schema / missingness / sample-size admissibility.
    ProvenanceAdmissibility,
    /// Residual magnitude, robust margins, slew-shock magnitude.
    NumericStrength,
    /// Persistence, drift, periodicity, recovery shape, temporal
    /// locality.
    TemporalStructure,
    /// Entity locality, causal adjacency, topology support.
    CrossSignalStructure,
    /// Distribution distance, divergence, distributional drift,
    /// motif consensus.
    DistributionStructure,
    /// Bank-motif fit, witness-role coherence (CPU-only authority
    /// per the Semantic Non-Bypass Axiom).
    SemanticBankStructure,
    /// Confuser suppression, clean-window support, transient
    /// rejection, anti-hallucination.
    ReliabilityConfuserControl,
    /// Operator readability, dashboard relevance, delta
    /// contribution (T.8 ledger contributes here).
    TaskUtility,
}

impl FusionPlane {
    /// Canonical wire name (matches the Rust variant name).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProvenanceAdmissibility => "ProvenanceAdmissibility",
            Self::NumericStrength => "NumericStrength",
            Self::TemporalStructure => "TemporalStructure",
            Self::CrossSignalStructure => "CrossSignalStructure",
            Self::DistributionStructure => "DistributionStructure",
            Self::SemanticBankStructure => "SemanticBankStructure",
            Self::ReliabilityConfuserControl => "ReliabilityConfuserControl",
            Self::TaskUtility => "TaskUtility",
        }
    }

    /// Parse a wire name back into the enum.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        Some(match s {
            "ProvenanceAdmissibility" => Self::ProvenanceAdmissibility,
            "NumericStrength" => Self::NumericStrength,
            "TemporalStructure" => Self::TemporalStructure,
            "CrossSignalStructure" => Self::CrossSignalStructure,
            "DistributionStructure" => Self::DistributionStructure,
            "SemanticBankStructure" => Self::SemanticBankStructure,
            "ReliabilityConfuserControl" => Self::ReliabilityConfuserControl,
            "TaskUtility" => Self::TaskUtility,
            _ => return None,
        })
    }

    /// All eight planes in canonical order (used by the report's
    /// histogram + role-axis matrix).
    #[must_use]
    pub const fn all() -> &'static [FusionPlane] {
        &[
            Self::ProvenanceAdmissibility,
            Self::NumericStrength,
            Self::TemporalStructure,
            Self::CrossSignalStructure,
            Self::DistributionStructure,
            Self::SemanticBankStructure,
            Self::ReliabilityConfuserControl,
            Self::TaskUtility,
        ]
    }
}

/// Compact bitset over the eight fusion planes (8 bits, one per
/// plane in declaration order).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FusionPlaneSet(pub u8);

impl FusionPlaneSet {
    /// Bit position for `ProvenanceAdmissibility`.
    pub const PROVENANCE_ADMISSIBILITY: u8 = 1 << 0;
    /// Bit position for `NumericStrength`.
    pub const NUMERIC_STRENGTH: u8 = 1 << 1;
    /// Bit position for `TemporalStructure`.
    pub const TEMPORAL_STRUCTURE: u8 = 1 << 2;
    /// Bit position for `CrossSignalStructure`.
    pub const CROSS_SIGNAL_STRUCTURE: u8 = 1 << 3;
    /// Bit position for `DistributionStructure`.
    pub const DISTRIBUTION_STRUCTURE: u8 = 1 << 4;
    /// Bit position for `SemanticBankStructure`.
    pub const SEMANTIC_BANK_STRUCTURE: u8 = 1 << 5;
    /// Bit position for `ReliabilityConfuserControl`.
    pub const RELIABILITY_CONFUSER_CONTROL: u8 = 1 << 6;
    /// Bit position for `TaskUtility`.
    pub const TASK_UTILITY: u8 = 1 << 7;

    /// True if no planes are bound.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Test whether a single plane is set.
    #[must_use]
    pub const fn contains(self, plane: FusionPlane) -> bool {
        self.0 & plane_bit(plane) != 0
    }

    /// Enumerate the planes set, in canonical order.
    #[must_use]
    pub fn planes(self) -> Vec<FusionPlane> {
        FusionPlane::all()
            .iter()
            .copied()
            .filter(|p| self.contains(*p))
            .collect()
    }
}

const fn plane_bit(plane: FusionPlane) -> u8 {
    match plane {
        FusionPlane::ProvenanceAdmissibility => FusionPlaneSet::PROVENANCE_ADMISSIBILITY,
        FusionPlane::NumericStrength => FusionPlaneSet::NUMERIC_STRENGTH,
        FusionPlane::TemporalStructure => FusionPlaneSet::TEMPORAL_STRUCTURE,
        FusionPlane::CrossSignalStructure => FusionPlaneSet::CROSS_SIGNAL_STRUCTURE,
        FusionPlane::DistributionStructure => FusionPlaneSet::DISTRIBUTION_STRUCTURE,
        FusionPlane::SemanticBankStructure => FusionPlaneSet::SEMANTIC_BANK_STRUCTURE,
        FusionPlane::ReliabilityConfuserControl => FusionPlaneSet::RELIABILITY_CONFUSER_CONTROL,
        FusionPlane::TaskUtility => FusionPlaneSet::TASK_UTILITY,
    }
}

/// Map a v1 9-axis bitset onto the 8 fusion planes.
///
/// The mapping is panel-locked and deterministic. It does NOT
/// compute fusion outputs — it labels which planes a detector
/// touches so the report and the future fusion engine know which
/// plane gates the detector must satisfy.
///
/// | Axis (v1)                              | Plane                          |
/// |----------------------------------------|--------------------------------|
/// | AXIS_1_RESIDUAL_MAGNITUDE              | NumericStrength                |
/// | AXIS_2_DRIFT_PERSISTENCE               | TemporalStructure              |
/// | AXIS_3_SLEW_SHOCK                      | NumericStrength                |
/// | AXIS_4_TEMPORAL_LOCALITY               | TemporalStructure              |
/// | AXIS_5_ENTITY_LOCALITY                 | CrossSignalStructure           |
/// | AXIS_6_CAUSAL_ADJACENCY                | CrossSignalStructure           |
/// | AXIS_7_MOTIF_CONSENSUS                 | DistributionStructure          |
/// | AXIS_8_BANK_ADMISSIBILITY              | SemanticBankStructure          |
/// | AXIS_9_CONFUSER_SUPPRESSION            | ReliabilityConfuserControl     |
///
/// Planes with no v1 axis (`ProvenanceAdmissibility`,
/// `TaskUtility`) are reserved for future Section S layers; T.6
/// never sets them from `AxisBindingSet` alone.
#[must_use]
pub fn axes_to_planes(axes: AxisBindingSet) -> FusionPlaneSet {
    let mut bits = 0u8;
    if axes.0 & (AxisBindingSet::AXIS_1_RESIDUAL_MAGNITUDE | AxisBindingSet::AXIS_3_SLEW_SHOCK) != 0
    {
        bits |= FusionPlaneSet::NUMERIC_STRENGTH;
    }
    if axes.0
        & (AxisBindingSet::AXIS_2_DRIFT_PERSISTENCE | AxisBindingSet::AXIS_4_TEMPORAL_LOCALITY)
        != 0
    {
        bits |= FusionPlaneSet::TEMPORAL_STRUCTURE;
    }
    if axes.0 & (AxisBindingSet::AXIS_5_ENTITY_LOCALITY | AxisBindingSet::AXIS_6_CAUSAL_ADJACENCY)
        != 0
    {
        bits |= FusionPlaneSet::CROSS_SIGNAL_STRUCTURE;
    }
    if axes.0 & AxisBindingSet::AXIS_7_MOTIF_CONSENSUS != 0 {
        bits |= FusionPlaneSet::DISTRIBUTION_STRUCTURE;
    }
    if axes.0 & AxisBindingSet::AXIS_8_BANK_ADMISSIBILITY != 0 {
        bits |= FusionPlaneSet::SEMANTIC_BANK_STRUCTURE;
    }
    if axes.0 & AxisBindingSet::AXIS_9_CONFUSER_SUPPRESSION != 0 {
        bits |= FusionPlaneSet::RELIABILITY_CONFUSER_CONTROL;
    }
    FusionPlaneSet(bits)
}

/// Verdict on whether two witness roles may co-fire on the same
/// candidate evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompatibilityVerdict {
    /// The two roles may co-fire and corroborate each other.
    Compatible,
    /// The roles may co-fire but neither corroborates the other
    /// (independent evidence streams).
    Independent,
    /// `role_a` suppresses `role_b` when both fire (`Confuser`
    /// suppresses `Primary`, etc.). The suppression direction is
    /// non-commutative.
    Suppresses,
    /// The two roles cannot co-fire on the same admitted episode
    /// without breaking semantics (e.g. `CleanWindow` +
    /// `Primary` is a contradiction).
    Incompatible,
}

/// One row in the compatibility rule table.
///
/// Read as: `role_a` and `role_b` (in that order) carry the named
/// `verdict`. Suppression is non-commutative: `(Confuser, Primary,
/// Suppresses)` means a Confuser suppresses a co-firing Primary,
/// not the other way around.
#[derive(Debug, Clone, Copy)]
pub struct WitnessCompatibilityRule {
    /// First role in the pair.
    pub role_a: WitnessRole,
    /// Second role in the pair.
    pub role_b: WitnessRole,
    /// What the rule says about co-firing.
    pub verdict: CompatibilityVerdict,
}

/// The panel-locked T.6 compatibility rule set.
///
/// This table is declarative: the actual suppression / admission
/// logic lives in the future Section S fusion engine. T.6's
/// promise is only that every pair of roles the corpus may
/// produce has a documented verdict.
pub static COMPATIBILITY_RULES: &[WitnessCompatibilityRule] = &[
    // Confuser suppresses Primary / Corroborating / Boundary /
    // Recovery. CleanWindow is never admitted with a Primary, so
    // their pair is Incompatible, not Suppresses.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Confuser,
        role_b: WitnessRole::Primary,
        verdict: CompatibilityVerdict::Suppresses,
    },
    WitnessCompatibilityRule {
        role_a: WitnessRole::Confuser,
        role_b: WitnessRole::Corroborating,
        verdict: CompatibilityVerdict::Suppresses,
    },
    WitnessCompatibilityRule {
        role_a: WitnessRole::Confuser,
        role_b: WitnessRole::Boundary,
        verdict: CompatibilityVerdict::Suppresses,
    },
    WitnessCompatibilityRule {
        role_a: WitnessRole::Confuser,
        role_b: WitnessRole::Recovery,
        verdict: CompatibilityVerdict::Suppresses,
    },
    // CleanWindow is the explicit "no episode here" witness; it
    // cannot co-fire with Primary / Boundary / Recovery on the
    // same candidate without violating semantics.
    WitnessCompatibilityRule {
        role_a: WitnessRole::CleanWindow,
        role_b: WitnessRole::Primary,
        verdict: CompatibilityVerdict::Incompatible,
    },
    WitnessCompatibilityRule {
        role_a: WitnessRole::CleanWindow,
        role_b: WitnessRole::Boundary,
        verdict: CompatibilityVerdict::Incompatible,
    },
    WitnessCompatibilityRule {
        role_a: WitnessRole::CleanWindow,
        role_b: WitnessRole::Recovery,
        verdict: CompatibilityVerdict::Incompatible,
    },
    // Primary + Corroborating: both raise the admissibility
    // strength of the same candidate episode.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Primary,
        role_b: WitnessRole::Corroborating,
        verdict: CompatibilityVerdict::Compatible,
    },
    // Primary + Boundary: boundary refines the time-edges of a
    // primary admission. Compatible.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Primary,
        role_b: WitnessRole::Boundary,
        verdict: CompatibilityVerdict::Compatible,
    },
    // Primary + Recovery: recovery refines closure; compatible.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Primary,
        role_b: WitnessRole::Recovery,
        verdict: CompatibilityVerdict::Compatible,
    },
    // Distribution evidence is an independent stream: it may
    // co-fire with Primary but doesn't directly corroborate the
    // per-cell residual witness.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Distribution,
        role_b: WitnessRole::Primary,
        verdict: CompatibilityVerdict::Independent,
    },
    // Topology evidence is independent in the same way.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Topology,
        role_b: WitnessRole::Primary,
        verdict: CompatibilityVerdict::Independent,
    },
    // Timing evidence is independent of magnitude evidence.
    WitnessCompatibilityRule {
        role_a: WitnessRole::Timing,
        role_b: WitnessRole::Primary,
        verdict: CompatibilityVerdict::Independent,
    },
];

/// Look up the verdict for a `(role_a, role_b)` pair.
///
/// Returns `None` if the table has no entry for the pair (T.6's
/// convention: unspecified pairs are conservatively treated as
/// `Independent` — co-firing allowed without corroboration).
#[must_use]
pub fn lookup_verdict(role_a: WitnessRole, role_b: WitnessRole) -> Option<CompatibilityVerdict> {
    COMPATIBILITY_RULES
        .iter()
        .find(|r| r.role_a == role_a && r.role_b == role_b)
        .map(|r| r.verdict)
}

/// True if a record with [`WitnessRole::CleanWindow`] is allowed
/// to admit an episode on its own. T.6 hard-codes this to
/// `false`: clean-window witnesses are negative evidence about
/// the absence of an episode; they cannot mint one.
#[must_use]
pub const fn clean_window_can_admit_episode_alone() -> bool {
    false
}

/// True if a [`WitnessRole::Primary`] record may carry a
/// non-`NotANegativeWitness` `NegativeWitnessKind`. T.6 hard-codes
/// this to `false`: a Primary witness fires to ADMIT an episode;
/// admitting and refuting simultaneously is incoherent.
#[must_use]
pub const fn primary_may_carry_negative_witness_kind(kind: NegativeWitnessKind) -> bool {
    matches!(kind, NegativeWitnessKind::NotANegativeWitness)
}
