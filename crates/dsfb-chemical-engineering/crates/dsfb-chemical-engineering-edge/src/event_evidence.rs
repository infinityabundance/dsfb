//! Event semantics & evidence rigor (Wave-6 confidential-evaluation chain): `ChemicalEventOntologyV1`,
//! `EpisodeEvidenceLedgerV1`, `EvidenceMinimumsMatrixV1`, and `WitnessBurdenOfProofV1`.
//!
//! Turns each episode from a CSV row into a *case file*: a typed event class separate from its heuristic
//! label, a full ledger of which witnesses support/contradict/are-silent-on/are-missing-for it, and a
//! deterministic burden-of-proof so a weak heuristic that does not meet its evidence minimum is forced to
//! emit **unknown** rather than a tempting story.
//!
//!   * [`ChemicalEventOntologyV1`] — a typed process-event class per episode (Sensor / Controller / Material /
//!     Thermal / Hydraulic / Reaction / Separation / Cleaning / Maintenance / ExternalUtility /
//!     UnknownStructural), separate from the heuristic label so the *kind* of event is explicit.
//!   * [`EpisodeEvidenceLedgerV1`] — supporting / contradicting / silent / missing / confuser witnesses +
//!     an evidence-strength class.
//!   * [`EvidenceMinimumsMatrixV1`] + [`WitnessBurdenOfProofV1`] — required (+ optional) evidence per
//!     candidate label, and the verdict: the label *only if the burden is met*, else `unknown`.
//!
//! Bounded (non-claims): an ontology class is a *typing of the episode's shape*, not a root cause; an
//! evidence ledger accounts for witnesses, it does not prove causation; failing the burden yields `unknown`,
//! never a fabricated label. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── ChemicalEventOntologyV1 ─────────────────────────────────────────────────────────────────────────

/// Typed process-event class (the *kind* of event, separate from the heuristic label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOntologyClass {
    Sensor,
    Controller,
    Material,
    Thermal,
    Hydraulic,
    Reaction,
    Separation,
    Cleaning,
    Maintenance,
    ExternalUtility,
    UnknownStructural,
}

impl EventOntologyClass {
    pub fn tag(self) -> &'static str {
        match self {
            EventOntologyClass::Sensor => "sensor",
            EventOntologyClass::Controller => "controller",
            EventOntologyClass::Material => "material",
            EventOntologyClass::Thermal => "thermal",
            EventOntologyClass::Hydraulic => "hydraulic",
            EventOntologyClass::Reaction => "reaction",
            EventOntologyClass::Separation => "separation",
            EventOntologyClass::Cleaning => "cleaning",
            EventOntologyClass::Maintenance => "maintenance",
            EventOntologyClass::ExternalUtility => "external_utility",
            EventOntologyClass::UnknownStructural => "unknown_structural",
        }
    }

    /// Deterministic keyword map from a heuristic label to an ontology class (falls back to
    /// `UnknownStructural`). The map is conservative: an unrecognised label is *not* forced into a class.
    pub fn from_label(label: &str) -> Self {
        let l = label.to_lowercase();
        let has = |k: &str| l.contains(k);
        if has("sensor") || has("stiction") && has("sens") || has("drift") && has("bias") {
            EventOntologyClass::Sensor
        } else if has("valve") || has("controller") || has("setpoint") || has("control") {
            EventOntologyClass::Controller
        } else if has("thermal")
            || has("temperature")
            || has("temp")
            || has("cooling")
            || has("heat excursion")
        {
            EventOntologyClass::Thermal
        } else if has("pump")
            || has("flow")
            || has("hydraulic")
            || has("cavitation")
            || has("blockage")
        {
            EventOntologyClass::Hydraulic
        } else if has("reaction") || has("conversion") || has("runaway") || has("kinetic") {
            EventOntologyClass::Reaction
        } else if has("column") || has("separation") || has("flooding") || has("distill") {
            EventOntologyClass::Separation
        } else if has("clean") || has("cip") {
            EventOntologyClass::Cleaning
        } else if has("maintenance") || has("calibration") {
            EventOntologyClass::Maintenance
        } else if has("utility") || has("steam") || has("cooling water") {
            EventOntologyClass::ExternalUtility
        } else if has("material") || has("lot") || has("feed") || has("coa") {
            EventOntologyClass::Material
        } else {
            EventOntologyClass::UnknownStructural
        }
    }
}

/// A hash-sealed episode ontology classification (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalEventOntologyV1 {
    pub episode_ref: String,
    pub heuristic_label: String,
    pub ontology_class: String,
    pub ontology_hash: String,
}

impl ChemicalEventOntologyV1 {
    fn seal(episode_ref: &str, heuristic_label: &str, ontology_class: &str) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"chemical_event_ontology_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        h.field("heuristic_label", heuristic_label.as_bytes());
        h.field("ontology_class", ontology_class.as_bytes());
        h.finalize_hex()
    }

    /// Classify an episode's heuristic label into its ontology class and seal.
    pub fn build(episode_ref: impl Into<String>, heuristic_label: impl Into<String>) -> Self {
        let (episode_ref, heuristic_label) = (episode_ref.into(), heuristic_label.into());
        let class = EventOntologyClass::from_label(&heuristic_label)
            .tag()
            .to_string();
        let ontology_hash = Self::seal(&episode_ref, &heuristic_label, &class);
        ChemicalEventOntologyV1 {
            episode_ref,
            heuristic_label,
            ontology_class: class,
            ontology_hash,
        }
    }

    pub fn verify(&self) -> bool {
        EventOntologyClass::from_label(&self.heuristic_label).tag() == self.ontology_class
            && Self::seal(
                &self.episode_ref,
                &self.heuristic_label,
                &self.ontology_class,
            ) == self.ontology_hash
    }
}

// ── EpisodeEvidenceLedgerV1 ─────────────────────────────────────────────────────────────────────────

/// A hash-sealed per-episode evidence ledger (schema v1): every witness accounted for by role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeEvidenceLedgerV1 {
    pub episode_ref: String,
    pub supporting: Vec<String>,
    pub contradicting: Vec<String>,
    pub silent: Vec<String>,
    pub missing: Vec<String>,
    pub confuser: Vec<String>,
    /// `Strong` / `Moderate` / `Weak` / `Contested` / `Insufficient` (see [`Self::derive_strength`]).
    pub strength: String,
    pub ledger_hash: String,
}

impl EpisodeEvidenceLedgerV1 {
    /// Derive the evidence-strength class from the witness counts (deterministic, conservative):
    /// any contradiction with support ⇒ `Contested`; no support at all ⇒ `Insufficient`; otherwise the
    /// support count buckets into `Strong` (≥3) / `Moderate` (2) / `Weak` (1).
    fn derive_strength(supporting: usize, contradicting: usize) -> &'static str {
        if supporting == 0 {
            "Insufficient"
        } else if contradicting > 0 {
            "Contested"
        } else if supporting >= 3 {
            "Strong"
        } else if supporting == 2 {
            "Moderate"
        } else {
            "Weak"
        }
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"episode_evidence_ledger_v1");
        h.field("episode_ref", self.episode_ref.as_bytes());
        for (label, list) in [
            ("supporting", &self.supporting),
            ("contradicting", &self.contradicting),
            ("silent", &self.silent),
            ("missing", &self.missing),
            ("confuser", &self.confuser),
        ] {
            for w in list {
                h.field(label, w.as_bytes());
            }
        }
        h.field("strength", self.strength.as_bytes());
        h.finalize_hex()
    }

    pub fn build(
        episode_ref: impl Into<String>,
        supporting: Vec<String>,
        contradicting: Vec<String>,
        silent: Vec<String>,
        missing: Vec<String>,
        confuser: Vec<String>,
    ) -> Self {
        let strength = Self::derive_strength(supporting.len(), contradicting.len()).to_string();
        let mut l = EpisodeEvidenceLedgerV1 {
            episode_ref: episode_ref.into(),
            supporting,
            contradicting,
            silent,
            missing,
            confuser,
            strength,
            ledger_hash: String::new(),
        };
        l.ledger_hash = l.seal();
        l
    }

    pub fn verify(&self) -> bool {
        Self::derive_strength(self.supporting.len(), self.contradicting.len()) == self.strength
            && self.seal() == self.ledger_hash
    }
}

// ── EvidenceMinimumsMatrixV1 + WitnessBurdenOfProofV1 ───────────────────────────────────────────────

/// One candidate label's evidence minimums.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceMinimum {
    pub label: String,
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

/// A hash-sealed evidence-minimums matrix (schema v1): the burden each candidate label must meet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceMinimumsMatrixV1 {
    pub minimums: Vec<EvidenceMinimum>,
    pub matrix_hash: String,
}

impl EvidenceMinimumsMatrixV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"evidence_minimums_matrix_v1");
        for m in &self.minimums {
            h.field("label", m.label.as_bytes());
            for r in &m.required {
                h.field("required", r.as_bytes());
            }
            for o in &m.optional {
                h.field("optional", o.as_bytes());
            }
        }
        h.finalize_hex()
    }

    pub fn build(minimums: Vec<EvidenceMinimum>) -> Self {
        let mut m = EvidenceMinimumsMatrixV1 {
            minimums,
            matrix_hash: String::new(),
        };
        m.matrix_hash = m.seal();
        m
    }

    /// The required evidence for a label (empty if the label is not in the matrix).
    pub fn required_for(&self, label: &str) -> Vec<String> {
        self.minimums
            .iter()
            .find(|m| m.label == label)
            .map(|m| m.required.clone())
            .unwrap_or_default()
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.matrix_hash
    }
}

/// A hash-sealed burden-of-proof verdict (schema v1) for one candidate label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WitnessBurdenOfProofV1 {
    pub candidate_label: String,
    pub required: Vec<String>,
    pub missing_required: Vec<String>,
    pub meets_burden: bool,
    /// The candidate label iff the burden is met, else `"unknown"` (the forced honest fallback).
    pub verdict: String,
    pub burden_hash: String,
}

impl WitnessBurdenOfProofV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"witness_burden_of_proof_v1");
        h.field("candidate_label", self.candidate_label.as_bytes());
        for r in &self.required {
            h.field("required", r.as_bytes());
        }
        for m in &self.missing_required {
            h.field("missing", m.as_bytes());
        }
        h.u64("meets_burden", self.meets_burden as u64);
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    /// Evaluate a candidate label against its required evidence and the present evidence. The burden is met
    /// iff every required item is present; otherwise the verdict is forced to `unknown`.
    pub fn evaluate(
        candidate_label: impl Into<String>,
        required: &[String],
        present: &[String],
    ) -> Self {
        let candidate_label = candidate_label.into();
        let mut missing_required: Vec<String> = required
            .iter()
            .filter(|r| !present.contains(r))
            .cloned()
            .collect();
        missing_required.sort();
        missing_required.dedup();
        let meets_burden = missing_required.is_empty();
        let verdict = if meets_burden {
            candidate_label.clone()
        } else {
            "unknown".to_string()
        };
        let mut b = WitnessBurdenOfProofV1 {
            candidate_label,
            required: required.to_vec(),
            missing_required,
            meets_burden,
            verdict,
            burden_hash: String::new(),
        };
        b.burden_hash = b.seal();
        b
    }

    pub fn verify(&self) -> bool {
        let meets = self.missing_required.is_empty();
        let verdict = if meets {
            self.candidate_label.clone()
        } else {
            "unknown".to_string()
        };
        meets == self.meets_burden && verdict == self.verdict && self.seal() == self.burden_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn ontology_types_labels_and_self_verifies() {
        assert_eq!(
            ChemicalEventOntologyV1::build("e1", "valve stiction").ontology_class,
            "controller"
        );
        assert_eq!(
            ChemicalEventOntologyV1::build("e2", "reactor thermal excursion").ontology_class,
            "thermal"
        );
        assert_eq!(
            ChemicalEventOntologyV1::build("e3", "pump cavitation").ontology_class,
            "hydraulic"
        );
        let o = ChemicalEventOntologyV1::build("e4", "weird unmapped thing");
        assert_eq!(o.ontology_class, "unknown_structural");
        assert!(o.ontology_hash.len() == 64 && o.verify());
        let mut t = o.clone();
        t.ontology_class = "thermal".into(); // forge a class the label doesn't map to
        assert!(!t.verify());
    }

    #[test]
    fn evidence_ledger_strength_classes() {
        // 3 supporting, no contradiction → Strong.
        let strong = EpisodeEvidenceLedgerV1::build(
            "e",
            vec![s("a"), s("b"), s("c")],
            vec![],
            vec![s("d")],
            vec![],
            vec![],
        );
        assert_eq!(strong.strength, "Strong");
        // supporting + contradicting → Contested.
        let contested = EpisodeEvidenceLedgerV1::build(
            "e",
            vec![s("a")],
            vec![s("x")],
            vec![],
            vec![],
            vec![s("cf")],
        );
        assert_eq!(contested.strength, "Contested");
        // no support → Insufficient.
        let insufficient =
            EpisodeEvidenceLedgerV1::build("e", vec![], vec![], vec![s("z")], vec![s("m")], vec![]);
        assert_eq!(insufficient.strength, "Insufficient");
        assert!(strong.verify() && contested.verify() && insufficient.verify());
    }

    #[test]
    fn burden_forces_unknown_when_required_evidence_missing() {
        let matrix = EvidenceMinimumsMatrixV1::build(vec![EvidenceMinimum {
            label: s("reactor_thermal_excursion"),
            required: vec![s("reactor_temp_breach"), s("coolant_anomaly")],
            optional: vec![s("rate_rise")],
        }]);
        assert!(matrix.verify());
        let required = matrix.required_for("reactor_thermal_excursion");
        // Only one of two required witnesses present → burden NOT met → verdict "unknown".
        let weak = WitnessBurdenOfProofV1::evaluate(
            "reactor_thermal_excursion",
            &required,
            &[s("reactor_temp_breach")],
        );
        assert!(!weak.meets_burden && weak.verdict == "unknown");
        assert_eq!(weak.missing_required, vec![s("coolant_anomaly")]);
        // Both present → burden met → the label stands.
        let strong = WitnessBurdenOfProofV1::evaluate(
            "reactor_thermal_excursion",
            &required,
            &[s("reactor_temp_breach"), s("coolant_anomaly")],
        );
        assert!(strong.meets_burden && strong.verdict == "reactor_thermal_excursion");
        assert!(weak.verify() && strong.verify());
        // Tamper: forge meets_burden true while a requirement is missing.
        let mut t = weak.clone();
        t.meets_burden = true;
        t.verdict = "reactor_thermal_excursion".into();
        assert!(!t.verify());
    }
}
