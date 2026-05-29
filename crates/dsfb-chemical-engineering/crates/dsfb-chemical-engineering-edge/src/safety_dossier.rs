//! `SafetyCertificationDossierV1` (Wave-7 impact) — auto-build a **traceability matrix** mapping the clauses
//! of safety / records standards (IEC 61511 · ISA-84 · FDA 21 CFR Part 11) to the DSFB evidence artifacts
//! that address them, lowering the adoption barrier for a regulated site.
//!
//! A regulated operator's first question is "how does this map to my standards?". This object answers it as a
//! sealed matrix: each standard clause → the requirement → the DSFB artifact that addresses it (or an honest
//! gap). It is a *traceability aid built from the Court Record*, not a certification.
//!
//! Bounded (non-claims, sealed): this is a **traceability matrix, NOT a certification, audit, or compliance
//! determination** — DSFB is advisory and read-only, with no SIS/safety authority (IEC 61511); the dossier
//! helps an operator's own qualified assessors, it does not replace them. Additive + off the replay path;
//! deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One traceability row: a standard clause and the DSFB artifact that addresses it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityRow {
    pub standard: String,
    pub clause: String,
    pub requirement: String,
    /// The DSFB artifact addressing it (e.g. `"ChemicalAuthoritySeparationLawV1"`), or `""` for a gap.
    pub evidence_artifact: String,
    pub addressed: bool,
}

/// A hash-sealed safety-certification traceability dossier (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyCertificationDossierV1 {
    pub rows: Vec<TraceabilityRow>,
    pub n_addressed: usize,
    pub n_gaps: usize,
    pub non_claim: String,
    pub dossier_hash: String,
}

impl SafetyCertificationDossierV1 {
    const NON_CLAIM: &'static str =
        "a traceability matrix built from the Court Record; NOT a certification, audit, or compliance determination. DSFB is advisory/read-only with no SIS/IEC-61511 safety authority";

    /// The default traceability matrix mapping standard clauses to DSFB artifacts. An artifact of `""` marks
    /// an honest gap (a clause DSFB does not address — e.g. anything requiring control/safety actuation).
    fn default_rows() -> Vec<TraceabilityRow> {
        let r = |std: &str, clause: &str, req: &str, art: &str| TraceabilityRow {
            standard: std.into(),
            clause: clause.into(),
            requirement: req.into(),
            evidence_artifact: art.into(),
            addressed: !art.is_empty(),
        };
        vec![
            r("IEC 61511", "independence of monitoring from the SIS",
              "the monitor must not be part of, or interfere with, the safety-instrumented function",
              "ChemicalAuthoritySeparationLawV1 (read-only; writes to no upstream register)"),
            r("IEC 61511", "no safety actuation authority",
              "a safety function's actuation must not depend on this layer",
              "claim-boundary banner + WitnessBurdenOfProof (advisory only; emits unknown, never an action)"),
            r("IEC 61511", "proof-test / safe-state demand handling", "periodic proof testing of the SIF",
              ""), // honest gap: DSFB is not the SIF and performs no proof test
            r("ISA-84", "alarm rationalisation (ISA-18.2/IEC 62682 alignment)",
              "alarms must be rationalised and flood-managed",
              "AlarmFloodCompressionReportV1"),
            r("FDA 21 CFR Part 11", "secure, computer-generated, time-stamped audit trails",
              "record creation/modification with an audit trail",
              "TamperEvidenceSealV1 + MerkleDagAmendmentChainV1 (append-only, hash-chained)"),
            r("FDA 21 CFR Part 11", "record integrity / tamper evidence",
              "detect alteration of electronic records",
              "CanonicalHasher seals + verify() on every evidence object; bundle_root reproducibility"),
            r("FDA 21 CFR Part 11", "electronic signatures binding", "signatures bound to records",
              ""), // honest gap: hash-chaining is provided; cryptographic signing is optional/out of scope
        ]
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"safety_certification_dossier_v1");
        for row in &self.rows {
            h.field("standard", row.standard.as_bytes());
            h.field("clause", row.clause.as_bytes());
            h.field("requirement", row.requirement.as_bytes());
            h.field("evidence_artifact", row.evidence_artifact.as_bytes());
            h.u64("addressed", row.addressed as u64);
        }
        h.u64("n_addressed", self.n_addressed as u64);
        h.u64("n_gaps", self.n_gaps as u64);
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    fn assemble(rows: Vec<TraceabilityRow>) -> Self {
        let n_addressed = rows.iter().filter(|r| r.addressed).count();
        let n_gaps = rows.len() - n_addressed;
        let mut d = SafetyCertificationDossierV1 {
            rows,
            n_addressed,
            n_gaps,
            non_claim: Self::NON_CLAIM.into(),
            dossier_hash: String::new(),
        };
        d.dossier_hash = d.seal();
        d
    }

    /// Build the default dossier (honest gaps included).
    pub fn build() -> Self {
        Self::assemble(Self::default_rows())
    }

    pub fn verify(&self) -> bool {
        let n_addressed = self.rows.iter().filter(|r| r.addressed).count();
        n_addressed == self.n_addressed
            && self.rows.len() - n_addressed == self.n_gaps
            && self
                .rows
                .iter()
                .all(|r| r.addressed != r.evidence_artifact.is_empty())
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.dossier_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dossier_maps_clauses_and_keeps_honest_gaps() {
        let d = SafetyCertificationDossierV1::build();
        assert!(d.n_addressed >= 4); // most clauses map to an artifact
        assert!(d.n_gaps >= 2); // proof-test + e-signature are honest gaps DSFB does not claim
                                // the proof-test clause is an unaddressed gap (DSFB is not the SIF)
        assert!(d
            .rows
            .iter()
            .any(|r| r.requirement.contains("proof testing") && !r.addressed));
        assert!(d.non_claim.contains("NOT a certification"));
        assert!(d.dossier_hash.len() == 64 && d.verify());
        assert_eq!(SafetyCertificationDossierV1::build(), d); // deterministic
    }

    #[test]
    fn tampering_addressed_or_nonclaim_breaks_the_seal() {
        let mut d = SafetyCertificationDossierV1::build();
        assert!(d.verify());
        // Forge a gap into "addressed" without supplying an artifact.
        if let Some(g) = d.rows.iter_mut().find(|r| !r.addressed) {
            g.addressed = true;
        }
        assert!(!d.verify());
    }
}
