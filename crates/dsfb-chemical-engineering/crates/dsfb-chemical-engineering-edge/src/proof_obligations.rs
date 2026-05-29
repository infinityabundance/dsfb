//! `ProofObligationLedgerV1` (Wave-7 formalism) — a sealed ledger of the DSFB grammar/fusion **proof
//! obligations**, each tagged with how it is currently discharged (a Kani harness today) and exported in a
//! form a Lean4 / Coq formalisation can pick up next.
//!
//! The Kani harnesses machine-check three grammar invariants (totality, the interior/sensor-fault and
//! beyond-bound/interior separations) on a bounded domain; the Lean 4 development in
//! `formal/lean/DsfbGrammar.lean` re-proves those three **unbounded** (over `Int`) and additionally proves
//! **quorum soundness** and **episode-compression monotonicity** (previously open). An independent **Coq/Rocq**
//! development (`formal/coq/DsfbGrammar.v`, using only the standard-library modules `List` / `ZArith` / `Lia`,
//! no third-party Coq deps) cross-checks the same theorems in a second prover kernel. Only **replay
//! determinism** remains empirical (the `verify-replay` gate + golden hashes). This
//! object makes the full obligation set explicit and sealed, tagging each with its discharge mechanism
//! (`KaniVerified` / `Lean4Verified` / `OpenForFormalProof`; the `Lean4Verified` ones are also Coq-verified).
//!
//! Bounded (non-claims): a `KaniVerified` obligation is machine-checked **on the finite/bounded domain of its
//! harness**, not a universal proof; an `OpenForFormalProof` obligation is *stated*, not proven — listing it
//! is honesty about what remains, not a claim that it holds. Additive + off the replay path; deterministic,
//! hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// How an obligation is currently discharged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofStatus {
    /// Machine-checked by a Kani harness (on the harness's bounded domain).
    KaniVerified,
    /// Machine-checked by a Lean 4 proof in `formal/lean/DsfbGrammar.lean` (unbounded over `Int`).
    Lean4Verified,
    /// Stated for a Lean4/Coq formalisation; not yet machine-proven.
    OpenForFormalProof,
}

impl ProofStatus {
    pub fn tag(self) -> &'static str {
        match self {
            ProofStatus::KaniVerified => "kani_verified",
            ProofStatus::Lean4Verified => "lean4_verified",
            ProofStatus::OpenForFormalProof => "open_for_formal_proof",
        }
    }
    /// True iff the obligation is discharged by *some* machine checker (Kani or Lean 4).
    pub fn is_machine_checked(self) -> bool {
        matches!(self, ProofStatus::KaniVerified | ProofStatus::Lean4Verified)
    }
}

/// One proof obligation: a formal statement + its discharge mechanism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofObligation {
    pub id: String,
    pub statement: String,
    /// The current mechanism (e.g. a Kani harness name, or `"lean4/coq (open)"`).
    pub mechanism: String,
    pub status: String,
}

/// A hash-sealed proof-obligation ledger (schema v1) — the formal-methods handoff checklist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofObligationLedgerV1 {
    pub obligations: Vec<ProofObligation>,
    pub n_machine_checked: usize,
    pub n_open: usize,
    pub non_claim: String,
    pub ledger_hash: String,
}

impl ProofObligationLedgerV1 {
    const NON_CLAIM: &'static str =
        "KaniVerified = machine-checked on the harness's bounded domain (not a universal proof); OpenForFormalProof = stated for Lean4/Coq, not yet proven";

    fn default_obligations() -> Vec<ProofObligation> {
        let o = |id: &str, stmt: &str, mech: &str, status: ProofStatus| ProofObligation {
            id: id.into(),
            statement: stmt.into(),
            mechanism: mech.into(),
            status: status.tag().into(),
        };
        use ProofStatus::*;
        vec![
            o("grammar_totality", "classify is total on finite inputs (every finite (r,δ,σ) maps to exactly one state)",
              "kani: proof_classify_is_total_on_finite + lean4/coq: classify_total", KaniVerified),
            o("interior_not_sensorfault", "a strictly-interior finite triple is never classified SensorFault",
              "kani: proof_interior_finite_is_not_sensor_fault + lean4/coq: valid_not_sensorFault", KaniVerified),
            o("beyond_bound_not_interior", "a triple beyond an envelope bound is never classified interior-nominal",
              "kani: proof_beyond_bound_is_not_interior + lean4/coq: outside_r_not_nominal", KaniVerified),
            o("quorum_soundness", "a fused episode requires ≥ quorum independent detector families firing in-window",
              "lean4 + coq: fused_sound (formal/lean/DsfbGrammar.lean, formal/coq/DsfbGrammar.v)", Lean4Verified),
            o("episode_compression_monotone", "episode fusion never increases the count of distinct episodes vs raw firings",
              "lean4 + coq: compression_monotone (formal/lean/DsfbGrammar.lean, formal/coq/DsfbGrammar.v)", Lean4Verified),
            o("replay_determinism", "identical inputs yield an identical canonical replay hash (no RNG/order dependence)",
              "open — gated empirically by verify-replay 6/6 + the golden hashes; a Lean4/Coq proof remains future work", OpenForFormalProof),
        ]
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"proof_obligation_ledger_v1");
        for ob in &self.obligations {
            h.field("id", ob.id.as_bytes());
            h.field("statement", ob.statement.as_bytes());
            h.field("mechanism", ob.mechanism.as_bytes());
            h.field("status", ob.status.as_bytes());
        }
        h.u64("n_machine_checked", self.n_machine_checked as u64);
        h.u64("n_open", self.n_open as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// True iff a status tag denotes a machine-checked obligation (Kani or Lean 4).
    fn tag_is_machine_checked(tag: &str) -> bool {
        tag == ProofStatus::KaniVerified.tag() || tag == ProofStatus::Lean4Verified.tag()
    }

    /// Build the default obligation ledger.
    pub fn build() -> Self {
        let obligations = Self::default_obligations();
        let n_machine_checked = obligations
            .iter()
            .filter(|o| Self::tag_is_machine_checked(&o.status))
            .count();
        let n_open = obligations.len() - n_machine_checked;
        let mut l = ProofObligationLedgerV1 {
            obligations,
            n_machine_checked,
            n_open,
            non_claim: Self::NON_CLAIM.into(),
            ledger_hash: String::new(),
        };
        l.ledger_hash = l.seal();
        l
    }

    /// Export the obligation statements (one per line) — the checklist a Lean4/Coq effort formalises.
    pub fn export_statements(&self) -> String {
        let mut s = String::from("# DSFB proof obligations (Lean4/Coq handoff)\n");
        for o in &self.obligations {
            s.push_str(&format!(
                "- [{}] {}: {} ({})\n",
                o.status, o.id, o.statement, o.mechanism
            ));
        }
        s
    }

    pub fn verify(&self) -> bool {
        let n_machine_checked = self
            .obligations
            .iter()
            .filter(|o| Self::tag_is_machine_checked(&o.status))
            .count();
        n_machine_checked == self.n_machine_checked
            && self.obligations.len() - n_machine_checked == self.n_open
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.ledger_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_separates_machine_checked_from_open() {
        let l = ProofObligationLedgerV1::build();
        // 3 Kani harnesses + 2 Lean4 proofs (quorum soundness, compression monotonicity) = 5 machine-checked.
        assert_eq!(l.n_machine_checked, 5);
        assert_eq!(l.n_open, 1); // only replay_determinism remains empirical (verify-replay gate)
        assert!(l.non_claim.contains("not a universal proof"));
        assert!(l.ledger_hash.len() == 64 && l.verify());
        // The export lists every obligation, with the now-discharged quorum obligation Lean4-tagged.
        let ex = l.export_statements();
        assert!(
            ex.contains("grammar_totality")
                && ex.contains("quorum_soundness")
                && ex.contains("lean4_verified")
        );
    }

    #[test]
    fn tampering_a_status_breaks_the_seal() {
        let mut l = ProofObligationLedgerV1::build();
        assert!(l.verify());
        // Forge the lone open obligation to "lean4_verified" without a proof.
        if let Some(o) = l
            .obligations
            .iter_mut()
            .find(|o| o.status == "open_for_formal_proof")
        {
            o.status = "lean4_verified".into();
        }
        assert!(!l.verify()); // the tally no longer matches
    }
}
