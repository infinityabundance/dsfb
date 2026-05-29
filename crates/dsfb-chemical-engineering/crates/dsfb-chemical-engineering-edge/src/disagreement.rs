//! `DetectorDisagreementForensicsV1` + `NegativeWitnessV1` — first-class disagreement forensics (P59).
//!
//! A `FusedEpisode` already records `participating_detectors`, `consensus_strength`, and a scalar
//! `disagreement_entropy`. That captures *who fired* and *how much they agreed*, but not the equally
//! forensic question of *who stayed silent and what that silence rules out*. This module promotes the
//! silent half of the evidence to a first-class object:
//!
//! - [`NegativeWitnessV1`] — a silent detector as evidence in its own right: which detector, *why* it
//!   was silent, and the **subspace implication** (what its silence lets the court rule out).
//! - [`DetectorDisagreementForensicsV1`] — the full per-episode report: the firing (participating)
//!   detectors, the silent ones (negative witnesses), the contradicting ones (firing but disagreeing on
//!   the grammar state), a `witness_diversity_score`, and the carried `disagreement_entropy` — sealed by
//!   a `forensics_hash`.
//!
//! Additive + off the replay path: it re-expresses the disagreement structure the fusion layer already
//! computes, and changes no existing sealed artifact.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Why a detector produced no firing within the episode window. Disclosed, not inferred as fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhySilent {
    /// The detector ran and its residual stayed within its admissibility envelope (nominal).
    NominalResidual,
    /// The detector ran but its score never crossed its control limit (sub-threshold).
    BelowThreshold,
    /// The detector is not applicable to this dataset's variables / process type.
    NotApplicable,
    /// The detector had no valid input (e.g. required channel missing or all-non-finite).
    NoInput,
}

impl WhySilent {
    fn tag(self) -> &'static str {
        match self {
            WhySilent::NominalResidual => "nominal_residual",
            WhySilent::BelowThreshold => "below_threshold",
            WhySilent::NotApplicable => "not_applicable",
            WhySilent::NoInput => "no_input",
        }
    }
}

/// A silent detector as first-class negative evidence (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegativeWitnessV1 {
    pub detector_id: String,
    pub family: String,
    pub why_silent: WhySilent,
    /// What this silence lets the court *rule out* (e.g. "no exceedance in the residual-energy
    /// subspace this detector covers"). A bounded, advisory implication — never a positive claim.
    pub subspace_implication: String,
}

/// Per-episode detector-disagreement forensics (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectorDisagreementForensicsV1 {
    /// Which episode this report is about (e.g. `"idx=120..168"` or an episode id).
    pub episode_ref: String,
    /// Firing detector ids (the positive witnesses).
    pub participating: Vec<String>,
    /// Silent detectors as first-class negative witnesses.
    pub silent: Vec<NegativeWitnessV1>,
    /// Detectors that fired but disagreed with the dominant grammar motif (contradicting witnesses).
    pub contradicting: Vec<String>,
    /// Total number of detectors considered.
    pub n_total_detectors: usize,
    /// Fraction of distinct detector *families* represented among the firing detectors, in `[0,1]`
    /// (1.0 = every family that exists fired at least one detector). Higher = broader corroboration.
    pub witness_diversity_score: f64,
    /// The episode's scalar disagreement entropy (carried through for one-stop forensics).
    pub disagreement_entropy: f64,
    /// SHA-256 (via [`CanonicalHasher`]) sealing the whole report.
    pub forensics_hash: String,
}

impl DetectorDisagreementForensicsV1 {
    /// Build the forensics report from the full detector roster and the episode's firing/contradicting
    /// sets. Silent detectors (roster − firing) become negative witnesses; their `why_silent` is taken
    /// from `silent_reason` (defaulting to `NominalResidual` for any not listed), and the
    /// `subspace_implication` is derived from the detector's family. `witness_diversity_score` =
    /// distinct firing families ÷ distinct total families.
    pub fn build(
        episode_ref: impl Into<String>,
        roster: &[(String, String)], // (detector_id, family) for every detector considered
        firing: &[String],
        contradicting: &[String],
        silent_reason: &std::collections::BTreeMap<String, WhySilent>,
        disagreement_entropy: f64,
    ) -> Self {
        use std::collections::BTreeSet;
        let firing_set: BTreeSet<&String> = firing.iter().collect();
        let total_families: BTreeSet<&String> = roster.iter().map(|(_, f)| f).collect();
        let firing_families: BTreeSet<&String> = roster
            .iter()
            .filter(|(id, _)| firing_set.contains(id))
            .map(|(_, f)| f)
            .collect();
        let witness_diversity_score = if total_families.is_empty() {
            0.0
        } else {
            firing_families.len() as f64 / total_families.len() as f64
        };
        // Silent = roster − firing; each becomes a negative witness.
        let mut silent: Vec<NegativeWitnessV1> = roster
            .iter()
            .filter(|(id, _)| !firing_set.contains(id))
            .map(|(id, family)| {
                let why_silent = silent_reason
                    .get(id)
                    .copied()
                    .unwrap_or(WhySilent::NominalResidual);
                NegativeWitnessV1 {
                    detector_id: id.clone(),
                    family: family.clone(),
                    why_silent,
                    subspace_implication: format!(
                        "no firing in the {family} subspace this detector covers"
                    ),
                }
            })
            .collect();
        silent.sort_by(|a, b| a.detector_id.cmp(&b.detector_id));

        let mut participating: Vec<String> = firing.to_vec();
        participating.sort();
        let mut contradicting_v: Vec<String> = contradicting.to_vec();
        contradicting_v.sort();

        let episode_ref = episode_ref.into();
        let forensics_hash = Self::seal(
            &episode_ref,
            &participating,
            &silent,
            &contradicting_v,
            roster.len(),
            witness_diversity_score,
            disagreement_entropy,
        );
        DetectorDisagreementForensicsV1 {
            episode_ref,
            participating,
            silent,
            contradicting: contradicting_v,
            n_total_detectors: roster.len(),
            witness_diversity_score,
            disagreement_entropy,
            forensics_hash,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn seal(
        episode_ref: &str,
        participating: &[String],
        silent: &[NegativeWitnessV1],
        contradicting: &[String],
        n_total: usize,
        diversity: f64,
        entropy: f64,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"detector_disagreement_forensics_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        for p in participating {
            h.field("firing", p.as_bytes());
        }
        for s in silent {
            h.field("silent_id", s.detector_id.as_bytes());
            h.field("silent_family", s.family.as_bytes());
            h.field("silent_why", s.why_silent.tag().as_bytes());
            h.field("silent_implication", s.subspace_implication.as_bytes());
        }
        for c in contradicting {
            h.field("contradicting", c.as_bytes());
        }
        h.u64("n_total", n_total as u64);
        h.f64q("diversity", diversity);
        h.f64q("entropy", entropy);
        h.finalize_hex()
    }

    /// Re-derive the seal from the record's own fields and check it matches `forensics_hash`.
    pub fn verify(&self) -> bool {
        Self::seal(
            &self.episode_ref,
            &self.participating,
            &self.silent,
            &self.contradicting,
            self.n_total_detectors,
            self.witness_diversity_score,
            self.disagreement_entropy,
        ) == self.forensics_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn roster() -> Vec<(String, String)> {
        vec![
            ("pca_t2".into(), "ClassicalMSPC".into()),
            ("ewma_spe".into(), "DynamicTemporal".into()),
            ("psi".into(), "NonlinearDistributional".into()),
            ("mass_balance".into(), "ProcessStructure".into()),
        ]
    }

    #[test]
    fn forensics_partitions_roster_into_firing_and_silent() {
        let firing = vec!["pca_t2".to_string(), "ewma_spe".to_string()];
        let contradicting = vec!["ewma_spe".to_string()];
        let reasons = BTreeMap::from([("psi".to_string(), WhySilent::BelowThreshold)]);
        let f = DetectorDisagreementForensicsV1::build(
            "idx=120..168",
            &roster(),
            &firing,
            &contradicting,
            &reasons,
            0.42,
        );
        assert_eq!(f.participating.len(), 2);
        assert_eq!(
            f.silent.len(),
            2,
            "the 2 non-firing detectors are negative witnesses"
        );
        // psi's why_silent came from the map; mass_balance defaulted to NominalResidual.
        let psi = f.silent.iter().find(|w| w.detector_id == "psi").unwrap();
        assert_eq!(psi.why_silent, WhySilent::BelowThreshold);
        let mb = f
            .silent
            .iter()
            .find(|w| w.detector_id == "mass_balance")
            .unwrap();
        assert_eq!(mb.why_silent, WhySilent::NominalResidual);
        // 2 firing families out of 4 total → diversity 0.5.
        assert!((f.witness_diversity_score - 0.5).abs() < 1e-12);
        assert!(f.verify());
        assert_eq!(f.forensics_hash.len(), 64);
    }

    #[test]
    fn deterministic_and_tamper_evident() {
        let firing = vec!["pca_t2".to_string()];
        let a = DetectorDisagreementForensicsV1::build(
            "e",
            &roster(),
            &firing,
            &[],
            &BTreeMap::new(),
            0.1,
        );
        let b = DetectorDisagreementForensicsV1::build(
            "e",
            &roster(),
            &firing,
            &[],
            &BTreeMap::new(),
            0.1,
        );
        assert_eq!(a.forensics_hash, b.forensics_hash);
        let mut t = a.clone();
        t.silent[0].subspace_implication = "tampered".into();
        assert!(!t.verify());
    }
}
