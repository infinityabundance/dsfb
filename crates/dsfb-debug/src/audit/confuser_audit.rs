//! Confuser-pair utilisation audit (Phase ζ.7).
//!
//! For each declared {motif, confuser} pair in the canonical bank,
//! aggregate the observed competition across the 12 vendored
//! fixtures: how often did both motifs score >0 on the same closed
//! episode, and how often did the confuser-margin gate fire?
//!
//! Outputs a per-pair utilisation entry. The entries are sorted by
//! observed competition count: heavily-utilised pairs (the bank's
//! load-bearing disambiguators) come first; under-utilised pairs
//! (theatre — declared but never observed competing on the data)
//! come last.
//!
//! Documentation only; no bank field is mutated by this module.
//!
//! Implementation note: `MotifClass` does not derive `Ord`, so this
//! module uses linear-scan Vec<...> aggregation rather than BTreeMap.
//! Motif counts are at most 32 × 32 = 1024 pairs, so linear scan is
//! cheap and avoids forcing an Ord derive on a public type.

extern crate std;

use std::format;
use std::string::String;
use std::vec;
use std::vec::Vec;

use crate::types::MotifClass;
use crate::heuristics_bank::HeuristicsBank;

/// Per-{motif, confuser} pair audit record.
#[derive(Debug, Clone)]
pub struct ConfuserPairAuditEntry {
    pub motif: MotifClass,
    pub confuser: MotifClass,
    /// Number of episodes (across all fixtures) where this motif
    /// scored highest AND the runner-up matched the declared confuser.
    pub observed_competitions: u64,
    /// Number of episodes (across all fixtures) where this motif's
    /// margin against the declared confuser fell below
    /// `min_margin_vs_confuser`, surfacing as `ConfuserAmbiguous`.
    pub confuser_ambiguous_count: u64,
    /// Total number of fixtures where this motif was the
    /// highest-scoring motif on at least one episode.
    pub fixtures_where_motif_typed: usize,
}

#[derive(Debug, Clone)]
pub struct ConfuserAuditReport {
    pub entries: Vec<ConfuserPairAuditEntry>,
}

/// Single observation of a typed-episode motif decision.
///
/// Captured during a fusion run by inspecting the `MatchConfidence`
/// per closed episode. The audit aggregates many of these.
#[derive(Debug, Clone, Copy)]
pub struct EpisodeTypingObservation {
    pub fixture_name: &'static str,
    pub top_motif: MotifClass,
    pub runner_up_motif: Option<MotifClass>,
    pub margin_vs_confuser: f64,
    pub fell_below_confuser_threshold: bool,
}

/// Build a confuser-pair utilisation report from a slice of
/// `EpisodeTypingObservation` records aggregated across all 12
/// fixtures.
pub fn audit_confuser_pairs<const M: usize>(
    bank: &HeuristicsBank<M>,
    observations: &[EpisodeTypingObservation],
) -> ConfuserAuditReport {
    // Build motif → confuser list from the bank (linear; <= 32 entries).
    let mut declared_pairs: Vec<(MotifClass, MotifClass)> = Vec::new();
    for entry in bank.entries_iter() {
        if let Some(confuser) = entry.confuser_motif {
            declared_pairs.push((entry.motif_class, confuser));
        }
    }

    // Counts: parallel Vec aligned with declared_pairs.
    let mut observed_competitions: Vec<u64> = vec![0; declared_pairs.len()];
    let mut confuser_ambig_counts: Vec<u64> = vec![0; declared_pairs.len()];

    // Fixtures-where-motif-typed: per declared motif, set of fixture names.
    let mut motif_fixtures: Vec<Vec<&'static str>> =
        vec![Vec::new(); declared_pairs.len()];

    for obs in observations {
        // Find this motif in the declared_pairs list.
        for (i, (motif, confuser)) in declared_pairs.iter().enumerate() {
            if obs.top_motif == *motif {
                if !motif_fixtures[i].contains(&obs.fixture_name) {
                    motif_fixtures[i].push(obs.fixture_name);
                }
                let runner_up_is_declared_confuser = obs.runner_up_motif
                    .map(|r| r == *confuser).unwrap_or(false);
                if runner_up_is_declared_confuser {
                    observed_competitions[i] += 1;
                }
                if obs.fell_below_confuser_threshold {
                    confuser_ambig_counts[i] += 1;
                }
            }
        }
    }

    let mut entries: Vec<ConfuserPairAuditEntry> = declared_pairs.iter()
        .enumerate()
        .map(|(i, (motif, confuser))| ConfuserPairAuditEntry {
            motif: *motif,
            confuser: *confuser,
            observed_competitions: observed_competitions[i],
            confuser_ambiguous_count: confuser_ambig_counts[i],
            fixtures_where_motif_typed: motif_fixtures[i].len(),
        })
        .collect();

    // Sort by observed_competitions descending; tie-break stable by
    // existing index (preserve declaration order — Theorem 9
    // determinism without an Ord on MotifClass).
    entries.sort_by(|a, b| b.observed_competitions
        .cmp(&a.observed_competitions));

    ConfuserAuditReport { entries }
}

/// Render the confuser-pair audit as markdown.
pub fn render_confuser_audit_md(report: &ConfuserAuditReport) -> String {
    let mut out = String::new();
    out.push_str("# Confuser-pair utilisation audit\n\n");
    out.push_str("For each declared {motif, confuser} pair in the canonical bank,\n");
    out.push_str("the table reports: observed competition count (runner-up matched\n");
    out.push_str("the declared confuser), confuser-margin-gate firings (margin fell\n");
    out.push_str("below `min_margin_vs_confuser`, surfacing as `ConfuserAmbiguous`),\n");
    out.push_str("and the count of fixtures where this motif typed at all.\n\n");
    out.push_str("Source: Phase ζ.7 audit harness (`src/audit/confuser_audit.rs`).\n");
    out.push_str("Data: aggregated `EpisodeTypingObservation` records over all 12\n");
    out.push_str("vendored fixtures.\n\n");
    out.push_str("Sorted by observed competition (most-utilised first).\n");
    out.push_str("Pairs with 0 observed competitions are theatre on the current\n");
    out.push_str("12-fixture surface; partner-data may activate them.\n\n");
    out.push_str("| Motif | Confuser | Observed competitions | Confuser-ambig events | Fixtures typed |\n");
    out.push_str("|-------|----------|---------------------:|---------------------:|---------------:|\n");

    for e in &report.entries {
        out.push_str(&format!(
            "| `{:?}` | `{:?}` | {} | {} | {} |\n",
            e.motif,
            e.confuser,
            e.observed_competitions,
            e.confuser_ambiguous_count,
            e.fixtures_where_motif_typed,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;
    use crate::heuristics_bank::HeuristicsBank;

    #[test]
    fn empty_observations_produces_zero_counts() {
        let bank: HeuristicsBank<64> = HeuristicsBank::with_canonical_motifs();
        let report = audit_confuser_pairs(&bank, &[]);
        // Every declared pair appears with zero observations.
        for e in &report.entries {
            assert_eq!(e.observed_competitions, 0);
            assert_eq!(e.confuser_ambiguous_count, 0);
            assert_eq!(e.fixtures_where_motif_typed, 0);
        }
        assert!(!report.entries.is_empty(),
                "canonical bank declares some confusers");
    }

    #[test]
    fn observation_increments_correct_pair() {
        let bank: HeuristicsBank<64> = HeuristicsBank::with_canonical_motifs();
        // MemoryLeakDrift's confuser is ConnectionPoolExhaustionDrift
        // (per docs/heuristics_bank.md and src/heuristics_bank.rs).
        let obs = vec![
            EpisodeTypingObservation {
                fixture_name: "test_fix",
                top_motif: MotifClass::MemoryLeakDrift,
                runner_up_motif: Some(MotifClass::ConnectionPoolExhaustionDrift),
                margin_vs_confuser: 0.05,
                fell_below_confuser_threshold: true,
            },
        ];
        let report = audit_confuser_pairs(&bank, &obs);
        let entry = report.entries.iter()
            .find(|e| e.motif == MotifClass::MemoryLeakDrift)
            .expect("memory leak pair should exist");
        assert_eq!(entry.observed_competitions, 1);
        assert_eq!(entry.confuser_ambiguous_count, 1);
        assert_eq!(entry.fixtures_where_motif_typed, 1);
    }
}
