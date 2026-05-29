//! Adversarial / anti-hallucination hardening (Wave-6 confidential-evaluation chain):
//! `AdversarialConfuserSuiteV1` and `FalseNarrativeRegressionTestV1`.
//!
//! Turns the framework's attack surface into a sealed artifact: instead of merely *claiming* DSFB does not
//! hallucinate, these record the outcome of running cases designed to fool it, so the resistance is testable
//! and auditable.
//!
//!   * [`AdversarialConfuserSuiteV1`] — confuser cases that intentionally *resemble* a real fault; the suite
//!     is `Robust` iff the system distinguished each one (emitted the true nature or `unknown`) rather than
//!     being fooled into the resembled label.
//!   * [`FalseNarrativeRegressionTestV1`] — cases asserting the system must NOT emit a tempting-but-
//!     unsupported story (a drift-only signal ⇒ `unknown structural episode`, never `reactor thermal
//!     excursion`). Any production of a forbidden narrative is a regression.
//!
//! The `produced` label fed to these objects comes from running the *real* bank (e.g. the burden-of-proof
//! gate in [`crate::event_evidence`]); the objects seal the outcome, they do not invent it.
//!
//! Bounded (non-claims): these prove resistance *on the supplied cases*, not a universal guarantee of
//! non-hallucination; a `Robust` verdict is "fooled by none of these confusers", not "cannot be fooled".
//! Additive + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

// ── AdversarialConfuserSuiteV1 ──────────────────────────────────────────────────────────────────────

/// One confuser case: a synthetic signal that *resembles* `resembles` but is not it; `produced` is what the
/// real bank emitted for it. The case is **distinguished** iff `produced != resembles` (the system was not
/// fooled into the mimicked label — it emitted the true nature or `unknown`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfuserCase {
    pub name: String,
    pub resembles: String,
    pub produced: String,
}

/// A hash-sealed adversarial confuser suite (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversarialConfuserSuiteV1 {
    pub cases: Vec<ConfuserCase>,
    pub n_distinguished: usize,
    pub n_fooled: usize,
    /// `"Robust"` iff no case was fooled, else `"Fooled"`.
    pub verdict: String,
    pub suite_hash: String,
}

impl AdversarialConfuserSuiteV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"adversarial_confuser_suite_v1");
        for c in &self.cases {
            h.field("name", c.name.as_bytes());
            h.field("resembles", c.resembles.as_bytes());
            h.field("produced", c.produced.as_bytes());
        }
        h.u64("n_distinguished", self.n_distinguished as u64);
        h.u64("n_fooled", self.n_fooled as u64);
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    pub fn build(cases: Vec<ConfuserCase>) -> Self {
        let n_fooled = cases.iter().filter(|c| c.produced == c.resembles).count();
        let n_distinguished = cases.len() - n_fooled;
        let verdict = if n_fooled == 0 { "Robust" } else { "Fooled" };
        let mut s = AdversarialConfuserSuiteV1 {
            cases,
            n_distinguished,
            n_fooled,
            verdict: verdict.into(),
            suite_hash: String::new(),
        };
        s.suite_hash = s.seal();
        s
    }

    pub fn is_robust(&self) -> bool {
        self.n_fooled == 0
    }

    pub fn verify(&self) -> bool {
        let n_fooled = self
            .cases
            .iter()
            .filter(|c| c.produced == c.resembles)
            .count();
        let n_distinguished = self.cases.len() - n_fooled;
        let verdict = if n_fooled == 0 { "Robust" } else { "Fooled" };
        n_fooled == self.n_fooled
            && n_distinguished == self.n_distinguished
            && verdict == self.verdict
            && self.seal() == self.suite_hash
    }
}

// ── FalseNarrativeRegressionTestV1 ──────────────────────────────────────────────────────────────────

/// One regression case: for `input_pattern`, the system must NOT emit `forbidden_narrative`; `produced` is
/// what it actually emitted. A **violation** is `produced == forbidden_narrative`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NarrativeCase {
    pub input_pattern: String,
    pub forbidden_narrative: String,
    pub produced: String,
}

/// A hash-sealed false-narrative regression test (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FalseNarrativeRegressionTestV1 {
    pub cases: Vec<NarrativeCase>,
    pub n_violations: usize,
    /// `"Pass"` iff no forbidden narrative was produced, else `"Regressed"`.
    pub verdict: String,
    pub test_hash: String,
}

impl FalseNarrativeRegressionTestV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"false_narrative_regression_test_v1");
        for c in &self.cases {
            h.field("input_pattern", c.input_pattern.as_bytes());
            h.field("forbidden_narrative", c.forbidden_narrative.as_bytes());
            h.field("produced", c.produced.as_bytes());
        }
        h.u64("n_violations", self.n_violations as u64);
        h.field("verdict", self.verdict.as_bytes());
        h.finalize_hex()
    }

    pub fn build(cases: Vec<NarrativeCase>) -> Self {
        let n_violations = cases
            .iter()
            .filter(|c| c.produced == c.forbidden_narrative)
            .count();
        let verdict = if n_violations == 0 {
            "Pass"
        } else {
            "Regressed"
        };
        let mut t = FalseNarrativeRegressionTestV1 {
            cases,
            n_violations,
            verdict: verdict.into(),
            test_hash: String::new(),
        };
        t.test_hash = t.seal();
        t
    }

    pub fn passed(&self) -> bool {
        self.n_violations == 0
    }

    pub fn verify(&self) -> bool {
        let n_violations = self
            .cases
            .iter()
            .filter(|c| c.produced == c.forbidden_narrative)
            .count();
        let verdict = if n_violations == 0 {
            "Pass"
        } else {
            "Regressed"
        };
        n_violations == self.n_violations
            && verdict == self.verdict
            && self.seal() == self.test_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_evidence::WitnessBurdenOfProofV1;

    fn s(x: &str) -> String {
        x.to_string()
    }

    /// The "system under test": run the burden-of-proof gate for a candidate label given the present
    /// evidence, returning its verdict (the label or `"unknown"`). This is the real anti-hallucination gate.
    fn system_verdict(label: &str, required: &[String], present: &[String]) -> String {
        WitnessBurdenOfProofV1::evaluate(label, required, present).verdict
    }

    #[test]
    fn confuser_suite_is_robust_when_the_bank_emits_unknown() {
        // A drift-only confuser resembles a thermal excursion, but with only the drift witness present the
        // burden for "reactor_thermal_excursion" is not met → the bank emits "unknown" → not fooled.
        let required = vec![s("reactor_temp_breach"), s("coolant_anomaly")];
        let produced = system_verdict("reactor_thermal_excursion", &required, &[s("slow_drift")]);
        let suite = AdversarialConfuserSuiteV1::build(vec![ConfuserCase {
            name: s("drift_mimics_thermal"),
            resembles: s("reactor_thermal_excursion"),
            produced,
        }]);
        assert!(suite.is_robust() && suite.verdict == "Robust" && suite.n_fooled == 0);
        assert!(suite.suite_hash.len() == 64 && suite.verify());
    }

    #[test]
    fn confuser_suite_detects_a_fooled_case() {
        let suite = AdversarialConfuserSuiteV1::build(vec![ConfuserCase {
            name: s("c"),
            resembles: s("reactor_thermal_excursion"),
            produced: s("reactor_thermal_excursion"), // the bank WAS fooled
        }]);
        assert!(!suite.is_robust() && suite.verdict == "Fooled" && suite.n_fooled == 1);
        assert!(suite.verify());
        let mut t = suite.clone();
        t.verdict = "Robust".into();
        assert!(!t.verify());
    }

    #[test]
    fn false_narrative_regression_passes_when_story_is_not_emitted() {
        // Drift-only input must NOT yield "reactor_thermal_excursion"; the burden gate returns "unknown".
        let required = vec![s("reactor_temp_breach")];
        let produced = system_verdict("reactor_thermal_excursion", &required, &[s("drift")]);
        let test = FalseNarrativeRegressionTestV1::build(vec![NarrativeCase {
            input_pattern: s("drift_only"),
            forbidden_narrative: s("reactor_thermal_excursion"),
            produced,
        }]);
        assert!(test.passed() && test.verdict == "Pass" && test.n_violations == 0);
        assert!(test.verify());
        // A regression: the forbidden narrative was produced.
        let regressed = FalseNarrativeRegressionTestV1::build(vec![NarrativeCase {
            input_pattern: s("drift_only"),
            forbidden_narrative: s("reactor_thermal_excursion"),
            produced: s("reactor_thermal_excursion"),
        }]);
        assert!(!regressed.passed() && regressed.verdict == "Regressed");
    }
}
