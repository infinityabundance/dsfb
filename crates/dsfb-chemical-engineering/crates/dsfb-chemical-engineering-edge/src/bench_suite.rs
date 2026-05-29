//! `DsfbBenchV1` (Wave-7 ecosystem) — a **provenance-locked challenge suite** definition: the adversarial,
//! non-stationary, multi-fault, and partial-instrumentation cases a monitor *should* be stress-tested on,
//! each tied to its data provenance, with the honest metric to report.
//!
//! This is a sealed *definition* of the benchmark (what to run, on what data, and which metric is honest for
//! it) — not a leaderboard. It exists so a third party can reproduce the same challenge set and so DSFB's own
//! claims are pinned to named, provenance-locked cases rather than cherry-picked runs.
//!
//! Bounded (non-claims): the suite defines challenges + the honest metric per challenge; it makes **no claim
//! that DSFB wins** any of them — the value is a reproducible, provenance-locked battery and honest metrics
//! (false-positive rate, detection delay, unknown-rate), never a single headline number. Additive + off the
//! replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The stress category a challenge exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeCategory {
    AdversarialNonStationary,
    MultiFault,
    PartialInstrumentation,
    ConfuserResistance,
    DataQuality,
}

impl ChallengeCategory {
    pub fn tag(self) -> &'static str {
        match self {
            ChallengeCategory::AdversarialNonStationary => "adversarial_non_stationary",
            ChallengeCategory::MultiFault => "multi_fault",
            ChallengeCategory::PartialInstrumentation => "partial_instrumentation",
            ChallengeCategory::ConfuserResistance => "confuser_resistance",
            ChallengeCategory::DataQuality => "data_quality",
        }
    }
}

/// One provenance-locked challenge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchChallenge {
    pub name: String,
    pub category: String,
    /// Data provenance (dataset + how the challenge is constructed) — what makes it reproducible.
    pub provenance: String,
    /// The honest metric to report for this challenge (e.g. `"baseline false-positive rate"`).
    pub honest_metric: String,
}

/// A hash-sealed DSFB-Bench challenge-suite definition (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DsfbBenchV1 {
    pub challenges: Vec<BenchChallenge>,
    pub non_claim: String,
    pub suite_hash: String,
}

impl DsfbBenchV1 {
    const NON_CLAIM: &'static str =
        "a reproducible, provenance-locked challenge battery with honest metrics; it makes NO claim that DSFB wins any challenge";

    /// The default provenance-locked challenge set (drawn from the committed datasets + documented constructions).
    fn default_challenges() -> Vec<BenchChallenge> {
        let c = |name: &str, cat: ChallengeCategory, prov: &str, metric: &str| BenchChallenge {
            name: name.into(),
            category: cat.tag().into(),
            provenance: prov.into(),
            honest_metric: metric.into(),
        };
        vec![
            c(
                "penicillin_fed_batch_nonstationary",
                ChallengeCategory::AdversarialNonStationary,
                "IndPenSim fed-batch (non-stationary baseline); phase-aligned regime envelope",
                "baseline false-positive rate",
            ),
            c(
                "tep_idv_multi",
                ChallengeCategory::MultiFault,
                "Tennessee Eastman IDV(1)/(4)/(6) executed; remaining IDVs catalogued",
                "per-fault detection delay (samples)",
            ),
            c(
                "swat_partial_instrumentation",
                ChallengeCategory::PartialInstrumentation,
                "SWaT stage-1 (agreement-gated; subset of tags) — observability-limited",
                "false-positive rate + observability non-claims",
            ),
            c(
                "confuser_battery",
                ChallengeCategory::ConfuserResistance,
                "synthetic confusers that mimic real faults (AdversarialConfuserSuiteV1)",
                "fooled count (target 0) + unknown-rate",
            ),
            c(
                "frozen_and_skewed_inputs",
                ChallengeCategory::DataQuality,
                "injected frozen tags + timestamp skew (FrozenTagDetectorV1 / ClockSkewWitnessV1)",
                "data-quality episodes recovered vs injected",
            ),
        ]
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"dsfb_bench_v1");
        for c in &self.challenges {
            h.field("name", c.name.as_bytes());
            h.field("category", c.category.as_bytes());
            h.field("provenance", c.provenance.as_bytes());
            h.field("honest_metric", c.honest_metric.as_bytes());
        }
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Build the default provenance-locked suite.
    pub fn build() -> Self {
        let mut s = DsfbBenchV1 {
            challenges: Self::default_challenges(),
            non_claim: Self::NON_CLAIM.into(),
            suite_hash: String::new(),
        };
        s.suite_hash = s.seal();
        s
    }

    /// Build a custom suite (for extension by the corpus crate / a third party).
    pub fn with_challenges(challenges: Vec<BenchChallenge>) -> Self {
        let mut s = DsfbBenchV1 {
            challenges,
            non_claim: Self::NON_CLAIM.into(),
            suite_hash: String::new(),
        };
        s.suite_hash = s.seal();
        s
    }

    pub fn verify(&self) -> bool {
        self.non_claim == Self::NON_CLAIM && self.seal() == self.suite_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_suite_covers_every_category_and_self_verifies() {
        let s = DsfbBenchV1::build();
        assert_eq!(s.challenges.len(), 5);
        // every category appears exactly once
        let mut cats: Vec<&str> = s.challenges.iter().map(|c| c.category.as_str()).collect();
        cats.sort();
        cats.dedup();
        assert_eq!(cats.len(), 5);
        assert!(s.non_claim.contains("NO claim that DSFB wins"));
        assert!(s.suite_hash.len() == 64 && s.verify());
        // deterministic
        assert_eq!(DsfbBenchV1::build(), s);
    }

    #[test]
    fn tampering_a_challenge_or_nonclaim_breaks_the_seal() {
        let mut s = DsfbBenchV1::build();
        assert!(s.verify());
        s.challenges[0].honest_metric = "win rate".into(); // forge a leaderboard metric
        assert!(!s.verify());
        let mut s2 = DsfbBenchV1::build();
        s2.non_claim = "DSFB wins all".into();
        assert!(!s2.verify());
    }
}
