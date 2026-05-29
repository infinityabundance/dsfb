//! The `Densor` trait — a sealed, self-verifying evidence object the runtime carries between stages.
//!
//! "Densor" is the generic DSFB term for a sealed residual-evidence object (the chemical crate's `…V1` records are
//! concrete densors). This trait is the minimal contract the runtime needs: a stable id, a kind, a 32-byte
//! evidence hash, and a `verify()` that re-derives the seal. It says nothing about chemistry — it is a mechanism.

use crate::errors::DensorError;

/// Coarse category of a densor, for routing/inventory only (never a claim about meaning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DensorKind {
    /// A residual / deviation object.
    Residual,
    /// An admissibility-envelope evaluation.
    Envelope,
    /// A grammar / classification state.
    Grammar,
    /// A fused / quorum object.
    Fusion,
    /// A physical / contextual witness.
    Witness,
    /// A sealed court / bundle record.
    Court,
    /// Anything else a downstream domain defines.
    Other,
}

impl DensorKind {
    pub fn tag(self) -> &'static str {
        match self {
            DensorKind::Residual => "residual",
            DensorKind::Envelope => "envelope",
            DensorKind::Grammar => "grammar",
            DensorKind::Fusion => "fusion",
            DensorKind::Witness => "witness",
            DensorKind::Court => "court",
            DensorKind::Other => "other",
        }
    }
}

/// A sealed, self-verifying evidence object.
pub trait Densor {
    /// A stable, non-empty identifier.
    fn densor_id(&self) -> &str;
    /// The densor's coarse kind (routing/inventory only).
    fn densor_kind(&self) -> DensorKind;
    /// The 32-byte evidence hash this densor seals to.
    fn evidence_hash(&self) -> [u8; 32];
    /// Re-derive the seal and confirm it matches (tamper-evident). Implementors must check `densor_id` is
    /// non-empty and that a freshly computed seal equals [`Densor::evidence_hash`].
    fn verify(&self) -> Result<(), DensorError>;
}
