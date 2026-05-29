//! `OperatorUncertaintyDashboardV1` (Wave-7 operator) — a quantitative **effort-to-resolve** score per
//! unresolved (`unknown` / structural-unmapped) episode, so an operator can triage *which unknown to chase
//! first* rather than facing an undifferentiated pile.
//!
//! Each unknown carries three deterministic difficulty signals: the **window length** (a long, persistent
//! unknown is more worth resolving), the **detector-disagreement entropy** (how split the detectors are —
//! high entropy = genuinely ambiguous), and the **closest-heuristic distance** (how far the episode shape is
//! from any known signature — far = novel). These combine into a normalised `effort_score`; the dashboard
//! ranks the unknowns by it.
//!
//! Bounded (non-claims): the effort score ranks *resolution difficulty*, it is not a severity or risk score
//! and does not imply a cause; a high score means "ambiguous + novel + persistent", not "important". Additive
//! + off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One unknown episode's difficulty signals + combined effort score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnknownEffort {
    pub episode_ref: String,
    pub window_len: usize,
    /// Detector-disagreement entropy, already normalised to `[0, 1]` by the caller (1 = maximally split).
    pub disagreement_entropy: f64,
    /// Distance to the nearest known heuristic signature, normalised to `[0, 1]` (1 = no near signature).
    pub closest_heuristic_distance: f64,
    /// Combined difficulty in `[0, 1]` (see [`OperatorUncertaintyDashboardV1::score`]).
    pub effort_score: f64,
}

/// A hash-sealed operator uncertainty dashboard (schema v1): unknowns ranked by effort-to-resolve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorUncertaintyDashboardV1 {
    /// Window length (samples) treated as the normalisation ceiling for the persistence term.
    pub window_norm: usize,
    /// Unknowns sorted by descending `effort_score` (ties broken by `episode_ref` for determinism).
    pub unknowns: Vec<UnknownEffort>,
    pub dashboard_hash: String,
}

impl OperatorUncertaintyDashboardV1 {
    /// Combine the three signals into an effort score in `[0, 1]`: the mean of the persistence term
    /// (`min(window_len/window_norm, 1)`), the disagreement entropy, and the closest-heuristic distance.
    /// Equal weights — disclosed, not tuned.
    fn score(window_len: usize, window_norm: usize, entropy: f64, distance: f64) -> f64 {
        let persistence = if window_norm == 0 {
            0.0
        } else {
            (window_len as f64 / window_norm as f64).min(1.0)
        };
        let e = entropy.clamp(0.0, 1.0);
        let d = distance.clamp(0.0, 1.0);
        (persistence + e + d) / 3.0
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"operator_uncertainty_dashboard_v1");
        h.u64("window_norm", self.window_norm as u64);
        for u in &self.unknowns {
            h.field("episode_ref", u.episode_ref.as_bytes());
            h.u64("window_len", u.window_len as u64);
            h.f64q("disagreement_entropy", u.disagreement_entropy);
            h.f64q("closest_heuristic_distance", u.closest_heuristic_distance);
            h.f64q("effort_score", u.effort_score);
        }
        h.finalize_hex()
    }

    /// Build the dashboard from `(episode_ref, window_len, disagreement_entropy, closest_heuristic_distance)`
    /// rows, scoring and ranking them by descending effort.
    pub fn build(rows: &[(String, usize, f64, f64)], window_norm: usize) -> Self {
        let mut unknowns: Vec<UnknownEffort> = rows
            .iter()
            .map(
                |(episode_ref, window_len, entropy, distance)| UnknownEffort {
                    episode_ref: episode_ref.clone(),
                    window_len: *window_len,
                    disagreement_entropy: entropy.clamp(0.0, 1.0),
                    closest_heuristic_distance: distance.clamp(0.0, 1.0),
                    effort_score: Self::score(*window_len, window_norm, *entropy, *distance),
                },
            )
            .collect();
        // Descending effort; ties broken by episode_ref ascending (deterministic).
        unknowns.sort_by(|a, b| {
            b.effort_score
                .partial_cmp(&a.effort_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.episode_ref.cmp(&b.episode_ref))
        });
        let mut d = OperatorUncertaintyDashboardV1 {
            window_norm,
            unknowns,
            dashboard_hash: String::new(),
        };
        d.dashboard_hash = d.seal();
        d
    }

    /// The hardest-to-resolve unknown (top of the ranking), if any.
    pub fn hardest(&self) -> Option<&UnknownEffort> {
        self.unknowns.first()
    }

    pub fn verify(&self) -> bool {
        // Re-derive scores + re-seal; also confirm the ranking is non-increasing.
        let scores_ok = self.unknowns.iter().all(|u| {
            (u.effort_score
                - Self::score(
                    u.window_len,
                    self.window_norm,
                    u.disagreement_entropy,
                    u.closest_heuristic_distance,
                ))
            .abs()
                <= 1e-12
        });
        let sorted = self
            .unknowns
            .windows(2)
            .all(|w| w[0].effort_score >= w[1].effort_score - 1e-12);
        scores_ok && sorted && self.seal() == self.dashboard_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn ranks_a_novel_ambiguous_persistent_unknown_highest() {
        // window_norm = 100. Episode "hard": long (90), split (entropy 0.9), novel (distance 0.9) → high.
        //                     "easy": short (5), agreed (0.1), near a known shape (0.1) → low.
        let rows = vec![
            (s("hard"), 90usize, 0.9, 0.9),
            (s("easy"), 5usize, 0.1, 0.1),
        ];
        let d = OperatorUncertaintyDashboardV1::build(&rows, 100);
        assert_eq!(d.hardest().unwrap().episode_ref, "hard");
        // hard score = (0.9 + 0.9 + 0.9)/3 = 0.9; easy = (0.05 + 0.1 + 0.1)/3 ≈ 0.083.
        assert!((d.unknowns[0].effort_score - 0.9).abs() < 1e-12);
        assert!(d.unknowns[1].effort_score < 0.2);
        assert!(d.dashboard_hash.len() == 64 && d.verify());
    }

    #[test]
    fn signals_are_clamped_and_persistence_capped() {
        // window_len beyond window_norm caps the persistence term at 1; out-of-range entropy/distance clamp.
        let rows = vec![(s("x"), 500usize, 1.5, -0.2)];
        let d = OperatorUncertaintyDashboardV1::build(&rows, 100);
        // persistence capped at 1, entropy→1, distance→0 → (1 + 1 + 0)/3 = 0.6667.
        assert!((d.unknowns[0].effort_score - (2.0 / 3.0)).abs() < 1e-12);
        assert!(d.verify());
    }

    #[test]
    fn tampering_a_score_breaks_the_seal() {
        let d = OperatorUncertaintyDashboardV1::build(&[(s("a"), 10, 0.5, 0.5)], 100);
        let mut t = d.clone();
        t.unknowns[0].effort_score = 0.99; // forge a higher difficulty
        assert!(!t.verify());
    }
}
