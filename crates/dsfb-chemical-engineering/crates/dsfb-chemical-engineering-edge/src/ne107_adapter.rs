//! `NamurNe107AdapterV1` — an *executable* DSFB-state → NAMUR NE 107 status adapter (panel #8).
//!
//! NAMUR NE 107 condenses device/diagnostic state into a small set of operator-facing categories (Good /
//! Function Check / Out of Specification / Maintenance Required / Failure). The report already prints a
//! per-sample NE 107 status string ([`crate::report::ne107_status`]); this object turns that mapping into a
//! **first-class, hash-sealed adapter** that additionally carries the *evidence kind*, the *physical-witness
//! strength*, and an explicit *confidence boundary* — so the mapped status is never read as a hard device
//! verdict. The typed [`Ne107Status`] is pinned to agree byte-for-byte with `report::ne107_status` by a test
//! (single source of truth; no second mapping to drift).
//!
//! **NON-CLAIM:** a status-adapter MAPPING for operator legibility — NOT certified device diagnostics, not a
//! compliance claim, and not an alarm. Even a `Failure` mapping is emitted only as a *candidate*, qualified by
//! the witness strength behind it. Self-sealed; not part of any frozen authority hash. Additive, read-only.

use serde::{Deserialize, Serialize};

use crate::dsfb_core::GrammarState;
use crate::evidence_kind::EvidenceKind;
use crate::hashing::CanonicalHasher;
use crate::witness_strength::PhysicalWitnessStrength;

/// The five condensed NAMUR NE 107 status categories. `tag()` returns exactly the strings
/// [`crate::report::ne107_status`] emits (pinned equal by a test), so the typed and string paths never diverge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ne107Status {
    /// Good / no diagnostic — `"OK"`.
    Ok,
    /// Maintenance required — a watch state (slow drift, grazing, recovery).
    MaintenanceRequired,
    /// Out of specification — a sharp transient excursion (slew spike).
    OutOfSpecification,
    /// Failure — a gross violation or compound disturbance (candidate only — see the adapter's confidence boundary).
    Failure,
    /// Function check — the reading itself is suspect (sensor fault).
    FunctionCheck,
}

impl Ne107Status {
    /// The NE 107 status string — identical to [`crate::report::ne107_status`]'s output (test-pinned).
    pub fn tag(self) -> &'static str {
        match self {
            Ne107Status::Ok => "OK",
            Ne107Status::MaintenanceRequired => "Maintenance required",
            Ne107Status::OutOfSpecification => "Out of specification",
            Ne107Status::Failure => "Failure",
            Ne107Status::FunctionCheck => "Function check",
        }
    }

    /// Map a DSFB grammar state to its NE 107 category — mirrors [`crate::report::ne107_status`] exactly.
    pub fn from_grammar_state(state: GrammarState) -> Ne107Status {
        match state {
            GrammarState::Nominal => Ne107Status::Ok,
            GrammarState::DriftAccum | GrammarState::BoundaryGrazing | GrammarState::Recovery => {
                Ne107Status::MaintenanceRequired
            }
            GrammarState::SlewSpike => Ne107Status::OutOfSpecification,
            GrammarState::EnvViolation | GrammarState::Compound => Ne107Status::Failure,
            GrammarState::SensorFault => Ne107Status::FunctionCheck,
        }
    }
}

/// A sealed DSFB-state → NE 107 adapter record (schema v1) — the mapped status plus the evidence context that
/// bounds how strongly it may be read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamurNe107AdapterV1 {
    /// The DSFB grammar-state token (e.g. `"EV"`, `"CP"`, `"SF"`).
    pub dsfb_state: String,
    /// The evidence kind backing the episode (from the witness rung).
    pub evidence_kind: String,
    /// The physical-witness rung the episode sits at.
    pub witness_strength: String,
    /// The mapped NE 107 category.
    pub mapped_status: Ne107Status,
    /// How strongly the status may be read, given the witness — always advisory; a `Failure` is a *candidate*.
    pub confidence_boundary: String,
    /// The load-bearing non-claim.
    pub non_claim: String,
    /// SHA-256 (via [`CanonicalHasher`]) sealing the adapter.
    pub adapter_hash: String,
}

impl NamurNe107AdapterV1 {
    /// The standing non-claim, part of the seal preimage.
    pub const NON_CLAIM: &'static str = "A status-adapter mapping for operator legibility (NAMUR NE 107 \
        categories) — NOT certified device diagnostics, not a compliance claim, and not an alarm; a Failure \
        mapping is emitted only as a witness-qualified candidate.";

    fn seal(
        dsfb_state: &str,
        evidence_kind: &str,
        witness_strength: &str,
        mapped: Ne107Status,
        confidence_boundary: &str,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"namur_ne107_adapter_v1");
        h.field("dsfb_state", dsfb_state.as_bytes());
        h.field("evidence_kind", evidence_kind.as_bytes());
        h.field("witness_strength", witness_strength.as_bytes());
        h.field("mapped_status", mapped.tag().as_bytes());
        h.field("confidence_boundary", confidence_boundary.as_bytes());
        h.field("non_claim", Self::NON_CLAIM.as_bytes());
        h.finalize_hex()
    }

    /// Build the adapter from a grammar state + the episode's physical-witness rung. The witness strength
    /// refines the confidence boundary: a `Failure` corroborated by a first-principles (balance/topology)
    /// witness is a stronger *candidate* than one resting on a statistical quorum alone.
    pub fn build(state: GrammarState, witness: PhysicalWitnessStrength) -> Self {
        let mapped_status = Ne107Status::from_grammar_state(state);
        let evidence_kind = EvidenceKind::from_witness_strength(witness)
            .tag()
            .to_string();
        let witness_strength = witness.tag().to_string();
        let confidence_boundary = match mapped_status {
            Ne107Status::Failure => {
                if witness.is_physical() {
                    "Failure CANDIDATE — physically corroborated (balance/topology witness); still advisory, not a certified device failure".to_string()
                } else {
                    "Failure CANDIDATE — statistical/structural witness only; corroborate with a physical (balance/topology) witness before escalation".to_string()
                }
            }
            _ => format!(
                "{} — advisory NE 107 mapping at witness rung '{}'; not a device verdict",
                mapped_status.tag(),
                witness.tag()
            ),
        };
        let dsfb_state = state.token().to_string();
        let adapter_hash = Self::seal(
            &dsfb_state,
            &evidence_kind,
            &witness_strength,
            mapped_status,
            &confidence_boundary,
        );
        NamurNe107AdapterV1 {
            dsfb_state,
            evidence_kind,
            witness_strength,
            mapped_status,
            confidence_boundary,
            non_claim: Self::NON_CLAIM.to_string(),
            adapter_hash,
        }
    }

    /// Re-derive the seal and confirm it matches (tamper-evident).
    pub fn verify(&self) -> bool {
        self.non_claim == Self::NON_CLAIM
            && self.adapter_hash
                == Self::seal(
                    &self.dsfb_state,
                    &self.evidence_kind,
                    &self.witness_strength,
                    self.mapped_status,
                    &self.confidence_boundary,
                )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_STATES: [GrammarState; 8] = [
        GrammarState::Nominal,
        GrammarState::DriftAccum,
        GrammarState::SlewSpike,
        GrammarState::EnvViolation,
        GrammarState::BoundaryGrazing,
        GrammarState::Recovery,
        GrammarState::Compound,
        GrammarState::SensorFault,
    ];

    /// Single source of truth: the typed mapping MUST equal the report's string mapping for every state.
    #[test]
    fn typed_status_agrees_with_report_string_mapping() {
        for s in ALL_STATES {
            assert_eq!(
                Ne107Status::from_grammar_state(s).tag(),
                crate::report::ne107_status(s),
                "NE107 typed/string mappings diverged for {:?}",
                s
            );
        }
    }

    #[test]
    fn sensor_fault_is_function_check_and_self_verifies() {
        let a = NamurNe107AdapterV1::build(
            GrammarState::SensorFault,
            PhysicalWitnessStrength::HeuristicPatternOnly,
        );
        assert_eq!(a.mapped_status, Ne107Status::FunctionCheck);
        assert!(a.verify());
    }

    #[test]
    fn failure_confidence_boundary_depends_on_physical_witness() {
        let phys = NamurNe107AdapterV1::build(
            GrammarState::Compound,
            PhysicalWitnessStrength::BalanceClosure,
        );
        let stat = NamurNe107AdapterV1::build(
            GrammarState::Compound,
            PhysicalWitnessStrength::DetectorFamilyQuorum,
        );
        assert_eq!(phys.mapped_status, Ne107Status::Failure);
        assert!(phys.confidence_boundary.contains("physically corroborated"));
        assert!(stat.confidence_boundary.contains("statistical"));
        // Both are CANDIDATES, never a hard failure assertion.
        assert!(
            phys.confidence_boundary.contains("CANDIDATE")
                && stat.confidence_boundary.contains("CANDIDATE")
        );
        assert_ne!(phys.adapter_hash, stat.adapter_hash);
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut a = NamurNe107AdapterV1::build(
            GrammarState::EnvViolation,
            PhysicalWitnessStrength::BalanceClosure,
        );
        a.mapped_status = Ne107Status::Ok;
        assert!(!a.verify());
    }
}
