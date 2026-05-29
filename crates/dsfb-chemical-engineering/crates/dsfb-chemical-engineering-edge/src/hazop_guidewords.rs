//! `HAZOPGuidewordMappingV1` — a residual-evidence mapping inspired by the classic HAZOP guideword set.
//!
//! Process engineers reason about deviations with a fixed guideword vocabulary (No/Not, More, Less, Reverse,
//! As well as, Part of, Other than, Early/Late). This module maps each guideword to its **DSFB residual-stream
//! analogue** — how that class of deviation tends to imprint on the residual triple `(r, δ, σ)` and the grammar —
//! and names the **existing** DSFB witnesses that would actually carry the evidence. It is a *bridge for reading
//! the residual court the way a HAZOP team reads a node*, not a new detector.
//!
//! **NON-CLAIM (load-bearing):** this is a residual-evidence *analogy* inspired by HAZOP guidewords. It is **NOT**
//! a formal process-hazard analysis (HAZOP / PHA), not a claim of hazard coverage, and not a root-cause assertion.
//! Each row is a *candidate* reading; the strongest thing it can say is bounded by the cited witness's strength.
//!
//! Self-sealed (its own `mapping_hash`); **not** folded into `atlas_hash_v1` (it is an edge reading aid, not a
//! frozen authority record), so adding/curating it never moves a frozen authority hash. Additive, read-only.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One HAZOP-guideword → DSFB-residual-analogue mapping entry. `&'static` so the bank is `const` (Serialize
/// only — `&'static` fields cannot be deserialized, matching the atlas authority-record convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HazopGuidewordEntryV1 {
    /// The HAZOP guideword.
    pub guideword: &'static str,
    /// The process deviation the guideword denotes (the engineer's reading).
    pub deviation: &'static str,
    /// The DSFB residual-stream analogue — how the deviation tends to imprint on `(r, δ, σ)` / the grammar.
    pub residual_analogue: &'static str,
    /// **Existing** DSFB witnesses/objects that would carry the evidence (grounds the analogy in real machinery,
    /// never invents a witness).
    pub dsfb_witnesses: &'static [&'static str],
    /// A concrete chemical-process example of the deviation.
    pub example: &'static str,
}

/// The eight-guideword bank, in canonical (sealed) order. Each `dsfb_witnesses` entry names an object that
/// already exists in the workspace, so the mapping is a *reading aid over real evidence*, not aspiration.
pub const HAZOP_GUIDEWORDS: &[HazopGuidewordEntryV1] = &[
    HazopGuidewordEntryV1 {
        guideword: "No / Not",
        deviation: "absence — no flow / no level change / no signal",
        residual_analogue: "flatline / zero-variance residual; classified SensorFault or routed to the \
                            'residual_degeneracy' unknown class (no discriminating drift/slew information)",
        dsfb_witnesses: &["SensorTrustDegradationLedgerV1 (frozen tag / dead channel)", "GrammarState::SensorFault", "unknown taxonomy: residual_degeneracy"],
        example: "frozen level transmitter; dead flow channel; loss of feed",
    },
    HazopGuidewordEntryV1 {
        guideword: "More",
        deviation: "excess — overfeed / over-temperature / over-pressure",
        residual_analogue: "sustained positive drift δ and/or an upper envelope breach (DriftAccum → EnvViolation high)",
        dsfb_witnesses: &["drift/slew/envelope grammar (DriftAccum, EnvViolation)", "BalanceWitnessV1 (closure surplus)"],
        example: "overfeed step; runaway exothermic temperature rise",
    },
    HazopGuidewordEntryV1 {
        guideword: "Less",
        deviation: "deficit — underfeed / under-temperature / reduced duty",
        residual_analogue: "sustained negative drift δ and/or a lower envelope breach",
        dsfb_witnesses: &["DriftAccum (δ<0)", "BalanceWitnessV1 (closure deficit)", "EquationResidualPassportV1 (heat-transfer duty)"],
        example: "underfeed; coolant loss; fouling-reduced heat-exchanger duty",
    },
    HazopGuidewordEntryV1 {
        guideword: "Reverse",
        deviation: "reversed sense — backflow / sign reversal",
        residual_analogue: "first-difference slew σ sign reversal / residual sign flip relative to baseline",
        dsfb_witnesses: &["SlewSpike (signed)", "SetpointResidualSeparationV1", "BalanceWitnessV1 (reversed flow leg)"],
        example: "backflow through a check valve; reversed differential pressure",
    },
    HazopGuidewordEntryV1 {
        guideword: "As well as",
        deviation: "additional — contaminant / extra component or pathway",
        residual_analogue: "unexpected co-drift across a variable group the nominal model does not predict; new \
                            correlated residual structure",
        dsfb_witnesses: &["DetectorDisagreementForensicsV1", "ProcessTopologyGraphV1 (unexpected coupling)", "NegativeWitnessV1"],
        example: "contaminant ingress; an unmodelled side reaction",
    },
    HazopGuidewordEntryV1 {
        guideword: "Part of",
        deviation: "incomplete — partial conversion / partial separation",
        residual_analogue: "balance-closure deficit; a yield/selectivity/component residual below the nominal band",
        dsfb_witnesses: &["BalanceWitnessV1 (Yield / Selectivity / component balance)", "EquationResidualPassportV1"],
        example: "incomplete reactor conversion; poor separation in a distillation column",
    },
    HazopGuidewordEntryV1 {
        guideword: "Other than",
        deviation: "wrong identity — wrong material / grade / recipe",
        residual_analogue: "out-of-regime: residuals reconcile only under a different regime/phase model; a \
                            material-lot or recipe boundary coincides with the structure",
        dsfb_witnesses: &["RegimeEnvelopeV1 (out-of-regime)", "MaterialLotWitnessV1", "unknown taxonomy: out_of_regime_envelope"],
        example: "wrong feed grade; mis-loaded recipe; an off-spec material lot",
    },
    HazopGuidewordEntryV1 {
        guideword: "Early / Late",
        deviation: "timing — residence-time or phase misalignment",
        residual_analogue: "upstream→downstream onset misaligned with the declared residence time; large \
                            residuals concentrated at phase boundaries",
        dsfb_witnesses: &["ResidenceTimeAlignmentV1", "StartupShutdownEnvelopeV1", "batch-phase heuristic (H5)"],
        example: "batch phase-step misalignment; residence-time drift after a throughput change",
    },
];

/// A sealed snapshot of the guideword mapping (schema v1) + its explicit non-claim — so a case file or paper
/// can cite the mapping by a stable digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HazopGuidewordMappingV1 {
    /// Number of guidewords in the sealed bank (8).
    pub n_guidewords: usize,
    /// The load-bearing non-claim (HAZOP-*inspired*, not a formal PHA).
    pub non_claim: String,
    /// SHA-256 (via [`CanonicalHasher`]) over the canonical bank + non-claim.
    pub mapping_hash: String,
}

impl HazopGuidewordMappingV1 {
    /// The standing non-claim, kept as a constant so it is part of the seal preimage.
    pub const NON_CLAIM: &'static str = "A residual-evidence mapping INSPIRED BY HAZOP guidewords — it is NOT a \
        formal process-hazard analysis (HAZOP/PHA), claims no hazard coverage, and asserts no root cause; each row \
        is a candidate reading bounded by its cited witness strength.";

    /// Canonical seal over the bank (fixed field order) + the non-claim.
    fn seal() -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"hazop_guideword_mapping_v1");
        for e in HAZOP_GUIDEWORDS {
            h.field("guideword", e.guideword.as_bytes());
            h.field("deviation", e.deviation.as_bytes());
            h.field("residual_analogue", e.residual_analogue.as_bytes());
            for w in e.dsfb_witnesses {
                h.field("witness", w.as_bytes());
            }
            h.field("example", e.example.as_bytes());
        }
        h.field("non_claim", Self::NON_CLAIM.as_bytes());
        h.finalize_hex()
    }

    /// Build the sealed mapping from the static bank.
    pub fn build() -> Self {
        HazopGuidewordMappingV1 {
            n_guidewords: HAZOP_GUIDEWORDS.len(),
            non_claim: Self::NON_CLAIM.to_string(),
            mapping_hash: Self::seal(),
        }
    }

    /// Re-derive the seal and confirm it matches (tamper-evident self-check).
    pub fn verify(&self) -> bool {
        self.n_guidewords == HAZOP_GUIDEWORDS.len()
            && self.non_claim == Self::NON_CLAIM
            && self.mapping_hash == Self::seal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_has_the_eight_guidewords_and_self_verifies() {
        let m = HazopGuidewordMappingV1::build();
        assert_eq!(
            m.n_guidewords, 8,
            "the classic HAZOP guideword set is eight entries"
        );
        assert!(m.verify(), "a freshly built mapping must self-verify");
    }

    #[test]
    fn seal_is_deterministic() {
        assert_eq!(
            HazopGuidewordMappingV1::build().mapping_hash,
            HazopGuidewordMappingV1::build().mapping_hash
        );
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let mut m = HazopGuidewordMappingV1::build();
        m.mapping_hash = "0".repeat(64);
        assert!(!m.verify(), "a mutated hash must fail verification");
    }

    #[test]
    fn every_guideword_grounds_in_at_least_one_witness() {
        // The mapping is a reading aid over REAL evidence: no row may be witness-less.
        for e in HAZOP_GUIDEWORDS {
            assert!(
                !e.dsfb_witnesses.is_empty(),
                "guideword '{}' must cite at least one DSFB witness",
                e.guideword
            );
        }
    }
}
