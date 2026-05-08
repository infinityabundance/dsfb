//! Per-motif affinity-tier and named-witness refinement (Phases ζ.4 + ζ.8).
//!
//! For each motif that types on a confirmed positive fixture (i.e.
//! the fixture matched the motif's `evidence_dataset` field), the
//! refinement pass:
//!
//! 1. Captures which tier bits were active during the matched
//!    episode's window range (Phase ζ.4 — affinity refinement).
//! 2. Captures which named detectors fired most frequently during
//!    that episode (Phase ζ.8 — named-witness refinement).
//!
//! The output is a *recommendation report* rendered to markdown.
//! The canonical bank in `src/heuristics_bank.rs` is NOT mutated by
//! this module — Phase ζ.9 separately gates any merge of the
//! recommendations through leave-one-fixture-out cross-validation.
//!
//! Discipline: real-data informed; hand-curation stands. The
//! recommendation surfaces divergence; the operator (or Phase ζ.9
//! gate) decides whether to apply.

extern crate std;

use std::collections::BTreeMap;
use std::format;
use std::string::String;
use std::vec::Vec;

use crate::types::MotifClass;

/// Per-motif refinement entry.
#[derive(Debug, Clone)]
pub struct MotifRefinementEntry {
    pub motif: MotifClass,
    pub fixture_observed: &'static str,
    /// Hand-curated affinity tier bitmask (current bank value).
    pub current_affinity_tiers: u32,
    /// Bitmask derived from per-cell tier-firing on the matched
    /// episode window range. May be ⊆, ⊋, or disjoint with the
    /// current curation.
    pub observed_affinity_tiers: u32,
    /// Hand-curated named witnesses (current bank value).
    pub current_named_witnesses: Vec<&'static str>,
    /// Top-K detectors by firing frequency on the matched episode.
    /// K = 5 by default.
    pub observed_top_witnesses: Vec<(&'static str, f64)>,
    /// Divergence rationale for the affinity refinement.
    pub affinity_divergence: AffinityDivergence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffinityDivergence {
    /// Observed ⊆ Current: hand-curated includes some bits never
    /// seen on this fixture's matched episode (potential dead
    /// weight).
    Subset,
    /// Observed ⊋ Current: real-data fires bits not in the
    /// hand-curation (potential missed routes).
    Superset,
    /// Observed ∩ Current = ∅: anti-data — flag for manual review.
    Disjoint,
    /// Observed = Current: hand-curation aligns exactly with
    /// observed firings.
    ExactMatch,
    /// Observed ∩ Current ≠ ∅ but neither subset nor superset.
    Overlap,
}

impl AffinityDivergence {
    pub fn classify(observed: u32, current: u32) -> Self {
        if observed == current { return Self::ExactMatch; }
        if observed & current == 0 { return Self::Disjoint; }
        if observed & !current == 0 { return Self::Subset; }
        if current & !observed == 0 { return Self::Superset; }
        Self::Overlap
    }
}

#[derive(Debug, Clone)]
pub struct MotifRefinementReport {
    pub entries: Vec<MotifRefinementEntry>,
}

/// Build a refinement report from per-fixture motif observations.
///
/// `observations`: iterator of (motif, fixture_name,
///                              current_affinity_tiers,
///                              observed_tier_mask,
///                              current_named_witnesses,
///                              top_detector_firings).
///
/// The `top_detector_firings` is the operator-side captured top-K
/// detectors-by-firing-rate during the motif's matched episode
/// (sorted descending by rate).
pub fn build_motif_refinement(
    observations: Vec<MotifRefinementEntry>,
) -> MotifRefinementReport {
    // Already populated by the caller; this function exists for
    // symmetry with the other audit modules and to allow future
    // cross-fixture aggregation.
    MotifRefinementReport { entries: observations }
}

/// Phase ζ.4 — single-episode observation.
///
/// Captured during a fusion run: for each typed-confirmed episode,
/// (top motif, fixture name, observed tier mask, observed top-K
/// witnesses).
#[derive(Debug, Clone)]
pub struct EpisodeMotifObservation {
    pub motif: MotifClass,
    pub fixture_name: &'static str,
    pub observed_tier_mask: u32,
    pub observed_top_witnesses: Vec<(&'static str, u64)>,
}

/// Build a refinement report from per-episode observations against
/// the canonical bank.
///
/// For each observation, looks up the motif's `affinity_tiers` and
/// `primary_witness_detectors` from the bank and computes the
/// divergence classification. The result is per-episode (not per-
/// motif), so the same motif typed on two fixtures contributes two
/// entries — useful for cross-fixture stability analysis.
pub fn build_motif_refinement_from_observations<const M: usize>(
    bank: &crate::heuristics_bank::HeuristicsBank<M>,
    observations: &[EpisodeMotifObservation],
) -> MotifRefinementReport {
    let mut entries: Vec<MotifRefinementEntry> = Vec::new();
    for obs in observations {
        let entry_opt = bank.entries_iter()
            .find(|e| e.motif_class == obs.motif);
        if let Some(bank_entry) = entry_opt {
            let curated = bank_entry.affinity_tiers;
            let witnesses: Vec<&'static str> = bank_entry.primary_witness_detectors
                .iter().copied().collect();
            // Convert observed top-K from (name, count) to (name, rate-as-f64)
            // for display. The denominator is the observed max so the rate
            // is a relative-density score in [0, 1] within the episode.
            let max_count = obs.observed_top_witnesses.iter()
                .map(|(_, c)| *c).max().unwrap_or(1).max(1);
            let observed_witnesses: Vec<(&'static str, f64)> = obs.observed_top_witnesses
                .iter().map(|(n, c)| (*n, *c as f64 / max_count as f64)).collect();

            entries.push(MotifRefinementEntry {
                motif: obs.motif,
                fixture_observed: obs.fixture_name,
                current_affinity_tiers: curated,
                observed_affinity_tiers: obs.observed_tier_mask,
                current_named_witnesses: witnesses,
                observed_top_witnesses: observed_witnesses,
                affinity_divergence: AffinityDivergence::classify(
                    obs.observed_tier_mask, curated),
            });
        }
    }
    MotifRefinementReport { entries }
}

/// Render the refinement report as markdown.
pub fn render_motif_refinement_md(report: &MotifRefinementReport) -> String {
    let mut out = String::new();
    out.push_str("# Per-motif refinement report (Phases ζ.4 + ζ.8)\n\n");
    out.push_str("For each motif that typed on a confirmed positive fixture,\n");
    out.push_str("the table reports: current hand-curated affinity-tier mask\n");
    out.push_str("vs observed tier-firing on the matched episode; current\n");
    out.push_str("named-witness list vs observed top-K detectors-by-firing.\n\n");
    out.push_str("**Refinements are RECOMMENDATIONS, not bank mutations.**\n");
    out.push_str("Phase ζ.9 separately gates any merge through leave-one-\n");
    out.push_str("fixture-out cross-validation (`audit::loo_cv::refinement_passes_gate`).\n\n");
    out.push_str("Source: Phase ζ.4 + ζ.8 audit harness.\n\n");
    out.push_str("| Motif | Fixture | Curated mask | Observed mask | Divergence | Top witnesses (observed) |\n");
    out.push_str("|-------|---------|-------------:|--------------:|-----------|-----|\n");

    for e in &report.entries {
        let witnesses_str: Vec<String> = e.observed_top_witnesses.iter()
            .map(|(n, f)| format!("`{}` ({:.2})", n, f))
            .collect();
        out.push_str(&format!(
            "| `{:?}` | `{}` | 0x{:08x} | 0x{:08x} | {:?} | {} |\n",
            e.motif,
            e.fixture_observed,
            e.current_affinity_tiers,
            e.observed_affinity_tiers,
            e.affinity_divergence,
            witnesses_str.join(", "),
        ));
    }

    out.push_str("\n## Summary by divergence\n\n");
    let mut by_div: BTreeMap<&'static str, u32> = BTreeMap::new();
    for e in &report.entries {
        let key: &'static str = match e.affinity_divergence {
            AffinityDivergence::ExactMatch => "ExactMatch",
            AffinityDivergence::Subset => "Subset (curation includes dead bits)",
            AffinityDivergence::Superset => "Superset (curation misses observed bits)",
            AffinityDivergence::Disjoint => "Disjoint (anti-data)",
            AffinityDivergence::Overlap => "Overlap (partial)",
        };
        *by_div.entry(key).or_insert(0) += 1;
    }
    for (k, v) in &by_div {
        out.push_str(&format!("- **{}**: {} motif(s)\n", k, v));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divergence_subset() {
        // Curated has bits 0, 1, 2; observed has bits 0, 1.
        assert_eq!(
            AffinityDivergence::classify(0b011, 0b111),
            AffinityDivergence::Subset);
    }

    #[test]
    fn divergence_superset() {
        // Curated has bits 0, 1; observed has bits 0, 1, 2.
        assert_eq!(
            AffinityDivergence::classify(0b111, 0b011),
            AffinityDivergence::Superset);
    }

    #[test]
    fn divergence_disjoint() {
        assert_eq!(
            AffinityDivergence::classify(0b100, 0b011),
            AffinityDivergence::Disjoint);
    }

    #[test]
    fn divergence_exact_match() {
        assert_eq!(
            AffinityDivergence::classify(0b101, 0b101),
            AffinityDivergence::ExactMatch);
    }

    #[test]
    fn divergence_overlap() {
        assert_eq!(
            AffinityDivergence::classify(0b110, 0b011),
            AffinityDivergence::Overlap);
    }
}
