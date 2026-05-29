//! `ClaimStrengthV1` (Wave-2 P73) — makes the paper's Tier-1/2/3 certainty hierarchy *executable*.
//!
//! The paper states a certainty hierarchy (Tier 1 proven by sealed artifacts; Tier 2 evidence-motivated
//! interpretation; Tier 3 newly-demonstrated bounded) and every episode wears a claim-boundary badge. This
//! module turns that prose discipline into a typed tag that any generated statement can carry, so a reader
//! (or an audit) can see, per sentence/row, *how strong the claim is* — and so the framework can never
//! silently present an interpretation as a fact.
//!
//! Bounded by construction: the lowest level is [`ClaimStrength::NonClaim`] — the things DSFB explicitly does
//! NOT assert (root cause, causality, accuracy superiority, regulatory compliance). Additive + off the
//! replay path. The mapping from an evidence-object *kind* to a strength is deterministic, so a tagged
//! statement ledger is hash-sealed + self-verifying like every other evidence object.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// How strongly a generated statement is claimed — the executable form of the Tier-1/2/3 hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStrength {
    /// Tier 1 — byte-exact, replay-verified by a sealed artifact (evidence_root, episode counts, hashes).
    SealedFact,
    /// Tier 2 — evidence-motivated interpretation: a candidate label / structural motif (advisory).
    EvidenceInterpretation,
    /// Tier 3 — a newly-demonstrated, *bounded* implication (e.g. a regime-conditioned improvement).
    SpeculativeImplication,
    /// Explicitly NOT asserted: root cause, causality, accuracy superiority, regulatory compliance, control.
    NonClaim,
}

impl ClaimStrength {
    /// The paper-tier number (1/2/3); `0` for an explicit non-claim.
    pub fn tier(self) -> u8 {
        match self {
            ClaimStrength::SealedFact => 1,
            ClaimStrength::EvidenceInterpretation => 2,
            ClaimStrength::SpeculativeImplication => 3,
            ClaimStrength::NonClaim => 0,
        }
    }

    /// Stable tag fed to the hasher / rendered (never localise or reorder — part of the seal).
    pub fn tag(self) -> &'static str {
        match self {
            ClaimStrength::SealedFact => "SealedFact",
            ClaimStrength::EvidenceInterpretation => "EvidenceInterpretation",
            ClaimStrength::SpeculativeImplication => "SpeculativeImplication",
            ClaimStrength::NonClaim => "NonClaim",
        }
    }

    /// Deterministically map an evidence-object *kind* (the `evidence_kind` a statement is derived from) to
    /// its claim strength. Unknown kinds default to the conservative `EvidenceInterpretation` — never higher.
    pub fn for_evidence_kind(kind: &str) -> ClaimStrength {
        match kind {
            // Tier 1 — sealed, replay-verifiable facts.
            "evidence_root" | "bundle_root" | "replay_hash" | "replay" | "atlas_hash"
            | "corpus_hash" | "sha256" | "digest" | "manifest" => ClaimStrength::SealedFact,
            // Tier 2 — advisory interpretations.
            "episode" | "label" | "heuristic" | "motif" | "witness" | "confuser" | "ruled_out"
            | "balance" | "ne107" | "disagreement" => ClaimStrength::EvidenceInterpretation,
            // Tier 3 — bounded newly-demonstrated effects.
            "regime_improvement"
            | "regime"
            | "propagation_candidate"
            | "precedent"
            | "similarity" => ClaimStrength::SpeculativeImplication,
            // Explicit non-claims.
            "root_cause"
            | "causality"
            | "accuracy_superiority"
            | "compliance"
            | "control_action" => ClaimStrength::NonClaim,
            _ => ClaimStrength::EvidenceInterpretation,
        }
    }
}

/// One claim-tagged statement: its text, the evidence kind it derives from, and its assigned strength.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaggedStatement {
    pub text: String,
    pub evidence_kind: String,
    pub strength_tag: String,
    pub tier: u8,
}

/// A hash-sealed ledger of claim-tagged statements (schema v1): the executable record that every statement
/// in an artifact carries a claim strength, with the tier distribution summarised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimStrengthLedgerV1 {
    pub statements: Vec<TaggedStatement>,
    /// Count per tier: `[non_claim, tier1, tier2, tier3]`.
    pub tier_counts: [usize; 4],
    pub ledger_hash: String,
}

/// Builder: append (text, evidence_kind) statements; the strength is derived deterministically.
#[derive(Debug, Default, Clone)]
pub struct ClaimStrengthBuilder {
    statements: Vec<TaggedStatement>,
}

impl ClaimStrengthBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tag one statement by its evidence kind (strength is assigned by [`ClaimStrength::for_evidence_kind`]).
    pub fn add(&mut self, text: impl Into<String>, evidence_kind: impl Into<String>) -> &mut Self {
        let kind = evidence_kind.into();
        let s = ClaimStrength::for_evidence_kind(&kind);
        self.statements.push(TaggedStatement {
            text: text.into(),
            evidence_kind: kind,
            strength_tag: s.tag().into(),
            tier: s.tier(),
        });
        self
    }

    /// Finalise + seal the ledger.
    pub fn seal(self) -> ClaimStrengthLedgerV1 {
        let mut tier_counts = [0usize; 4];
        for st in &self.statements {
            // index: NonClaim(tier 0)→0, Tier1→1, Tier2→2, Tier3→3.
            tier_counts[st.tier as usize] += 1;
        }
        let mut h = CanonicalHasher::new();
        h.field("schema", b"claim_strength_ledger_v1");
        for st in &self.statements {
            h.field("text", st.text.as_bytes());
            h.field("evidence_kind", st.evidence_kind.as_bytes());
            h.field("strength", st.strength_tag.as_bytes());
        }
        ClaimStrengthLedgerV1 {
            statements: self.statements,
            tier_counts,
            ledger_hash: h.finalize_hex(),
        }
    }
}

impl ClaimStrengthLedgerV1 {
    /// Re-derive the seal from the record's own fields.
    pub fn verify(&self) -> bool {
        let mut b = ClaimStrengthBuilder::new();
        b.statements = self.statements.clone();
        b.seal().ledger_hash == self.ledger_hash
    }

    /// The honesty invariant: no statement claims a tier *higher* than its evidence kind permits. (A
    /// statement derived from a non-claim kind must be tagged NonClaim, etc.) Returns the offending texts.
    pub fn over_claims(&self) -> Vec<String> {
        self.statements
            .iter()
            .filter(|st| {
                let allowed = ClaimStrength::for_evidence_kind(&st.evidence_kind);
                st.strength_tag != allowed.tag()
            })
            .map(|st| st.text.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_kinds_map_to_the_right_tier() {
        assert_eq!(ClaimStrength::for_evidence_kind("evidence_root").tier(), 1);
        assert_eq!(ClaimStrength::for_evidence_kind("label").tier(), 2);
        assert_eq!(
            ClaimStrength::for_evidence_kind("regime_improvement").tier(),
            3
        );
        assert_eq!(
            ClaimStrength::for_evidence_kind("root_cause"),
            ClaimStrength::NonClaim
        );
        // Unknown kinds default conservatively to interpretation — never SealedFact.
        assert_eq!(
            ClaimStrength::for_evidence_kind("mystery"),
            ClaimStrength::EvidenceInterpretation
        );
    }

    #[test]
    fn ledger_seals_tiers_and_self_verifies() {
        let mut b = ClaimStrengthBuilder::new();
        b.add("evidence_root abc… replay-verified", "evidence_root")
            .add("candidate: reactor thermal excursion", "label")
            .add(
                "regime-conditioned envelope cut baseline FP 54%→39%",
                "regime_improvement",
            )
            .add("no physical root cause is asserted", "root_cause");
        let led = b.seal();
        // [non_claim, tier1, tier2, tier3]
        assert_eq!(led.tier_counts, [1, 1, 1, 1]);
        assert!(led.verify());
        assert!(
            led.over_claims().is_empty(),
            "builder-assigned strengths never over-claim"
        );
    }

    #[test]
    fn a_tampered_strength_is_caught_as_an_overclaim() {
        let mut b = ClaimStrengthBuilder::new();
        b.add("candidate label", "label");
        let mut led = b.seal();
        // Forge a Tier-2 interpretation into a Tier-1 SealedFact → over-claim detected (+ seal breaks).
        led.statements[0].strength_tag = ClaimStrength::SealedFact.tag().into();
        assert_eq!(led.over_claims(), vec!["candidate label".to_string()]);
        assert!(!led.verify());
    }
}
