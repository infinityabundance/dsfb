//! Observability honesty (Wave-6 confidential-evaluation chain): `InstrumentationCoverageMapV1`,
//! `ObservabilityNonClaimReceiptV1`, and `ResidualWitnessCoverageScoreV1`.
//!
//! The no-overclaim discipline extended to its sharpest form: **can the supplied data even see this fault?**
//! If the instrumentation cannot observe a fault class, DSFB must say so — never fabricate a label it has no
//! witness for. These three sealed objects make observability a first-class, auditable input gate:
//!
//!   * [`InstrumentationCoverageMapV1`] — per-fault-class observability given the available tags
//!     (`Observable` / `PartiallyObservable` / `NotObservable`).
//!   * [`ObservabilityNonClaimReceiptV1`] — when a requested label's required witnesses are missing, an
//!     explicit *"not observable from supplied data"* receipt naming what is missing.
//!   * [`ResidualWitnessCoverageScoreV1`] — available ÷ required witness families, so an operator can judge
//!     whether a dataset is Phase-I-adequate before investing in an evaluation.
//!
//! Bounded (non-claims): an `Observable` mark means the *required inputs are present*, not that a fault is
//! occurring or will be detected; a non-claim receipt is the honest refusal to label, not a statement that
//! the fault is absent. Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── InstrumentationCoverageMapV1 ────────────────────────────────────────────────────────────────────

/// Observability level of one fault class given the available tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Observability {
    Observable,
    PartiallyObservable,
    NotObservable,
}

impl Observability {
    fn tag(self) -> &'static str {
        match self {
            Observability::Observable => "observable",
            Observability::PartiallyObservable => "partially_observable",
            Observability::NotObservable => "not_observable",
        }
    }
}

/// One fault class's coverage entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageEntry {
    pub fault_class: String,
    pub n_required: usize,
    pub n_present: usize,
    /// Required witnesses that are missing from the available set (sorted, deterministic).
    pub missing: Vec<String>,
    pub observability: String,
}

/// A hash-sealed per-fault-class instrumentation-coverage map (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentationCoverageMapV1 {
    pub entries: Vec<CoverageEntry>,
    pub n_observable: usize,
    pub n_not_observable: usize,
    pub map_hash: String,
}

impl InstrumentationCoverageMapV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"instrumentation_coverage_map_v1");
        for e in &self.entries {
            h.field("fault_class", e.fault_class.as_bytes());
            h.u64("n_required", e.n_required as u64);
            h.u64("n_present", e.n_present as u64);
            for m in &e.missing {
                h.field("missing", m.as_bytes());
            }
            h.field("observability", e.observability.as_bytes());
        }
        h.u64("n_observable", self.n_observable as u64);
        h.u64("n_not_observable", self.n_not_observable as u64);
        h.finalize_hex()
    }

    /// Build the map from `(fault_class, required_witnesses)` requirements and the available witness set.
    /// All-present ⇒ `Observable`; some ⇒ `PartiallyObservable`; none ⇒ `NotObservable`.
    pub fn build(requirements: &[(String, Vec<String>)], available: &[String]) -> Self {
        let entries: Vec<CoverageEntry> = requirements
            .iter()
            .map(|(fault_class, required)| {
                let mut missing: Vec<String> = required
                    .iter()
                    .filter(|r| !available.contains(r))
                    .cloned()
                    .collect();
                missing.sort();
                missing.dedup();
                let n_required = required.len();
                let n_present = n_required - missing.len();
                let observability = if n_required == 0 || n_present == n_required {
                    Observability::Observable
                } else if n_present == 0 {
                    Observability::NotObservable
                } else {
                    Observability::PartiallyObservable
                };
                CoverageEntry {
                    fault_class: fault_class.clone(),
                    n_required,
                    n_present,
                    missing,
                    observability: observability.tag().into(),
                }
            })
            .collect();
        let n_observable = entries
            .iter()
            .filter(|e| e.observability == "observable")
            .count();
        let n_not_observable = entries
            .iter()
            .filter(|e| e.observability == "not_observable")
            .count();
        let mut m = InstrumentationCoverageMapV1 {
            entries,
            n_observable,
            n_not_observable,
            map_hash: String::new(),
        };
        m.map_hash = m.seal();
        m
    }

    pub fn verify(&self) -> bool {
        let n_observable = self
            .entries
            .iter()
            .filter(|e| e.observability == "observable")
            .count();
        let n_not_observable = self
            .entries
            .iter()
            .filter(|e| e.observability == "not_observable")
            .count();
        n_observable == self.n_observable
            && n_not_observable == self.n_not_observable
            && self.seal() == self.map_hash
    }
}

// ── ObservabilityNonClaimReceiptV1 ──────────────────────────────────────────────────────────────────

/// A hash-sealed non-claim receipt (schema v1): the honest refusal to label when witnesses are missing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityNonClaimReceiptV1 {
    pub requested_label: String,
    pub missing_inputs: Vec<String>,
    /// True iff `missing_inputs` is empty (the label IS observable from the supplied data).
    pub observable: bool,
    pub statement: String,
    pub receipt_hash: String,
}

impl ObservabilityNonClaimReceiptV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"observability_non_claim_receipt_v1");
        h.field("requested_label", self.requested_label.as_bytes());
        for m in &self.missing_inputs {
            h.field("missing", m.as_bytes());
        }
        h.u64("observable", self.observable as u64);
        h.field("statement", self.statement.as_bytes());
        h.finalize_hex()
    }

    /// Build a receipt for a requested label given the inputs it requires that are missing.
    pub fn build(requested_label: impl Into<String>, mut missing_inputs: Vec<String>) -> Self {
        missing_inputs.sort();
        missing_inputs.dedup();
        let requested_label = requested_label.into();
        let observable = missing_inputs.is_empty();
        let statement = if observable {
            format!("'{requested_label}' is observable from the supplied data")
        } else {
            format!(
                "'{requested_label}' is NOT observable from the supplied data (missing: {})",
                missing_inputs.join(", ")
            )
        };
        let mut r = ObservabilityNonClaimReceiptV1 {
            requested_label,
            missing_inputs,
            observable,
            statement,
            receipt_hash: String::new(),
        };
        r.receipt_hash = r.seal();
        r
    }

    pub fn verify(&self) -> bool {
        self.observable == self.missing_inputs.is_empty() && self.seal() == self.receipt_hash
    }
}

// ── ResidualWitnessCoverageScoreV1 ──────────────────────────────────────────────────────────────────

/// A hash-sealed witness-coverage score (schema v1): available ÷ required witness families.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidualWitnessCoverageScoreV1 {
    /// The required witness families (e.g. tag / unit / topology / controller / lab).
    pub required_families: Vec<String>,
    pub available_families: Vec<String>,
    pub n_required: usize,
    pub n_available: usize,
    /// `n_available / n_required` in `[0, 1]` (1.0 when nothing is required).
    pub coverage: f64,
    pub score_hash: String,
}

impl ResidualWitnessCoverageScoreV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"residual_witness_coverage_score_v1");
        for f in &self.required_families {
            h.field("required", f.as_bytes());
        }
        for f in &self.available_families {
            h.field("available", f.as_bytes());
        }
        h.u64("n_required", self.n_required as u64);
        h.u64("n_available", self.n_available as u64);
        h.f64q("coverage", self.coverage);
        h.finalize_hex()
    }

    /// Build the score: how many of the required witness families are available.
    pub fn build(required_families: &[String], available_families: &[String]) -> Self {
        let n_required = required_families.len();
        let n_available = required_families
            .iter()
            .filter(|f| available_families.contains(f))
            .count();
        let coverage = if n_required == 0 {
            1.0
        } else {
            n_available as f64 / n_required as f64
        };
        let mut s = ResidualWitnessCoverageScoreV1 {
            required_families: required_families.to_vec(),
            available_families: available_families.to_vec(),
            n_required,
            n_available,
            coverage,
            score_hash: String::new(),
        };
        s.score_hash = s.seal();
        s
    }

    pub fn verify(&self) -> bool {
        let n_available = self
            .required_families
            .iter()
            .filter(|f| self.available_families.contains(f))
            .count();
        let coverage = if self.n_required == 0 {
            1.0
        } else {
            n_available as f64 / self.n_required as f64
        };
        n_available == self.n_available
            && (coverage - self.coverage).abs() <= 1e-12
            && self.seal() == self.score_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn coverage_map_grades_each_fault_class() {
        let reqs = vec![
            (s("valve_stiction"), vec![s("PV"), s("MV"), s("SP")]),
            (
                s("heat_exchanger_fouling"),
                vec![s("T_hot_in"), s("T_hot_out"), s("flow")],
            ),
            (s("bearing_fault"), vec![s("vibration")]),
        ];
        let available = vec![s("PV"), s("MV"), s("SP"), s("T_hot_in")]; // full valve loop; partial HX; no vibration
        let m = InstrumentationCoverageMapV1::build(&reqs, &available);
        assert_eq!(m.entries[0].observability, "observable"); // valve loop complete
        assert_eq!(m.entries[1].observability, "partially_observable"); // 1/3 HX tags
        assert_eq!(m.entries[2].observability, "not_observable"); // no vibration tag
        assert_eq!(m.entries[2].missing, vec![s("vibration")]);
        assert_eq!((m.n_observable, m.n_not_observable), (1, 1));
        assert!(m.map_hash.len() == 64 && m.verify());
    }

    #[test]
    fn non_claim_receipt_refuses_to_label_when_blind() {
        let r = ObservabilityNonClaimReceiptV1::build(
            "reactor_thermal_excursion",
            vec![s("reactor_temp"), s("coolant_flow")],
        );
        assert!(!r.observable);
        assert!(r.statement.contains("NOT observable"));
        assert!(r.verify());
        // An observable label: no missing inputs.
        let ok = ObservabilityNonClaimReceiptV1::build("level_drift", vec![]);
        assert!(ok.observable && ok.statement.contains("is observable"));
        // Tamper: forge observable=true while keeping missing inputs.
        let mut t = r.clone();
        t.observable = true;
        assert!(!t.verify());
    }

    #[test]
    fn coverage_score_is_available_over_required() {
        let req = vec![
            s("tag"),
            s("unit"),
            s("topology"),
            s("controller"),
            s("lab"),
        ];
        let avail = vec![s("tag"), s("unit"), s("controller")]; // 3 of 5
        let sc = ResidualWitnessCoverageScoreV1::build(&req, &avail);
        assert_eq!((sc.n_required, sc.n_available), (5, 3));
        assert!((sc.coverage - 0.6).abs() < 1e-12);
        assert!(sc.score_hash.len() == 64 && sc.verify());
        let mut t = sc.clone();
        t.coverage = 1.0; // forge a perfect score
        assert!(!t.verify());
    }
}
