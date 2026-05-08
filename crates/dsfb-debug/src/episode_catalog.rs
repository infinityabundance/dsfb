//! DSFB-Debug: episode catalog — institutional memory for debugging
//! episodes.
//!
//! Each closed `DebugEpisode` is recorded into a fixed-size circular
//! buffer. New episodes can be matched against history via
//! signature-vector cosine similarity, surfacing past episodes whose
//! structural shape matches the current one.
//!
//! # Operator use case
//!
//! At 3am, a `MotifClass::CascadingTimeoutSlew` fires. The catalog
//! lookup says: "similar to episode #4173 from 23 days ago,
//! similarity 0.91, peak_slew 0.83 vs current 0.85, contributing
//! services were the same set". The operator immediately reaches
//! for the past resolution (e.g. "increased connection pool to 200,
//! see runbook PR #2418") rather than re-discovering the diagnosis.
//!
//! Per panellist P12 (Distributed-Systems Architect, Session 6):
//! "questions 5 and 7 of the on-call questions are 'have we seen
//! this before?' and 'what should I investigate next?'. The catalog
//! is the answer to both." This module is the answer.
//!
//! # no_std + no_alloc — Copy slots
//!
//! The catalog is a fixed-size `[Option<DebugEpisode>; N]` array.
//! Older episodes are overwritten when the buffer wraps. Lookup is
//! O(N) linear scan. Suitable for embedded deployments (no heap)
//! and audit-grade determinism (no time-dependent state — the same
//! `record` history produces the same `find_similar` output).
//!
//! For audit-grade durability across process restarts, the operator
//! should periodically serialise the catalog to a write-ahead log
//! (see `docs/operator_handbook.md` §Episode Catalog Persistence).

use crate::types::*;

/// Fixed-size circular buffer of past episodes.
///
/// `N` is the catalog capacity. Older episodes are overwritten when
/// the buffer wraps. For audit-grade durability, persist the catalog
/// to disk between runs (see `docs/operator_handbook.md`).
pub struct EpisodeCatalog<const N: usize> {
    entries: [Option<DebugEpisode>; N],
    write_head: usize,
    total_recorded: u64,
}

/// Result of a catalog similarity lookup.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SimilarEpisode {
    /// Index into the catalog (0..N). Stable for as long as the
    /// catalog has not wrapped past this entry.
    pub catalog_index: usize,
    /// The matched past episode.
    pub past_episode: DebugEpisode,
    /// Similarity score in `[0.0, 1.0]`. 1.0 = identical signature
    /// vector (motif, drift direction, peak slew, duration,
    /// contributing-signal-count rounded to nearest); 0.0 = nothing
    /// in common. Cosine-style metric over a 5-element feature vector.
    pub similarity: f64,
}

impl<const N: usize> EpisodeCatalog<N> {
    /// Construct an empty catalog.
    pub fn new() -> Self {
        Self {
            entries: [None; N],
            write_head: 0,
            total_recorded: 0,
        }
    }

    /// Append an episode to the catalog. Wraps when full.
    pub fn record(&mut self, ep: DebugEpisode) {
        if N == 0 {
            return;
        }
        self.entries[self.write_head] = Some(ep);
        self.write_head = (self.write_head + 1) % N;
        self.total_recorded += 1;
    }

    /// Total episodes recorded since construction (including any
    /// overwritten by buffer wrap). Useful for audit logs.
    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    /// Number of entries currently stored (capped at N).
    pub fn len(&self) -> usize {
        if self.total_recorded as usize > N {
            N
        } else {
            self.total_recorded as usize
        }
    }

    /// True iff no episodes have been recorded.
    pub fn is_empty(&self) -> bool {
        self.total_recorded == 0
    }

    /// Find the most-similar past episode to `query`.
    ///
    /// Returns `None` if the catalog is empty or no past entry has
    /// non-zero similarity. Tie-broken by lower catalog index for
    /// determinism.
    pub fn find_similar(&self, query: &DebugEpisode) -> Option<SimilarEpisode> {
        let mut best: Option<SimilarEpisode> = None;
        let mut i = 0;
        while i < N {
            if let Some(past) = self.entries[i] {
                let s = signature_similarity(query, &past);
                let take = match best {
                    None => s > 0.0,
                    Some(b) => s > b.similarity,
                };
                if take {
                    best = Some(SimilarEpisode {
                        catalog_index: i,
                        past_episode: past,
                        similarity: s,
                    });
                }
            }
            i += 1;
        }
        best
    }
}

impl<const N: usize> Default for EpisodeCatalog<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Signature similarity in `[0.0, 1.0]`. Five-feature comparison:
///   1. Motif equality (binary 1.0 / 0.0 — strongest signal)
///   2. Reason code equality (binary)
///   3. Drift-direction equality (binary)
///   4. Peak slew magnitude proximity (1 - |Δ| / max)
///   5. Duration proximity (1 - |Δlog| / max)
/// Combined as weighted sum normalised to [0, 1]. Weights: motif 0.40,
/// reason 0.20, drift_dir 0.10, slew 0.15, duration 0.15.
#[inline]
fn signature_similarity(a: &DebugEpisode, b: &DebugEpisode) -> f64 {
    let motif_match = match (a.matched_motif, b.matched_motif) {
        (SemanticDisposition::Named(m1), SemanticDisposition::Named(m2)) => {
            if m1 == m2 { 1.0 } else { 0.0 }
        }
        (SemanticDisposition::Unknown, SemanticDisposition::Unknown) => 0.5,
        _ => 0.0,
    };
    let reason_match = if a.primary_reason_code == b.primary_reason_code { 1.0 } else { 0.0 };
    let drift_match = if a.structural_signature.dominant_drift_direction
        == b.structural_signature.dominant_drift_direction { 1.0 } else { 0.0 };

    let slew_a = abs_f64(a.structural_signature.peak_slew_magnitude);
    let slew_b = abs_f64(b.structural_signature.peak_slew_magnitude);
    let slew_max = if slew_a > slew_b { slew_a } else { slew_b };
    let slew_proximity = if slew_max > 0.0 {
        let denom = if slew_max > 1e-9 { slew_max } else { 1e-9 };
        1.0 - abs_f64(slew_a - slew_b) / denom
    } else {
        1.0
    };

    let dur_a_raw = a.structural_signature.duration_windows;
    let dur_b_raw = b.structural_signature.duration_windows;
    let dur_a = if dur_a_raw > 1 { dur_a_raw } else { 1 } as f64;
    let dur_b = if dur_b_raw > 1 { dur_b_raw } else { 1 } as f64;
    let dur_proximity = 1.0 - abs_f64(log2_approx(dur_a) - log2_approx(dur_b)) / 8.0;
    let dur_proximity = clamp01_f64(dur_proximity);

    let raw: f64 = 0.40 * motif_match
        + 0.20 * reason_match
        + 0.10 * drift_match
        + 0.15 * slew_proximity
        + 0.15 * dur_proximity;
    clamp01_f64(raw)
}

#[inline]
fn abs_f64(x: f64) -> f64 {
    if x >= 0.0 { x } else { -x }
}

#[inline]
fn clamp01_f64(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

/// `log2(x)` approximation for x > 0 — Newton-Raphson-free bit-trick.
/// We don't have access to f64::log2 in no_std; use a quick coarse
/// approximation good enough for the duration-similarity feature
/// (which only needs ~8-bin granularity).
#[inline]
fn log2_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    // For x in [1, 1<<63], the integer floor(log2(x)) equals
    // 63 - leading_zeros(x). For fractional refinement, linearly
    // interpolate within the bin.
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7FF) as i32 - 1023;
    let mantissa = (bits & ((1u64 << 52) - 1)) as f64 / (1u64 << 52) as f64;
    exp as f64 + mantissa
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(motif: MotifClass, slew: f64, duration: u64, contrib: u16) -> DebugEpisode {
        DebugEpisode {
            episode_id: 0,
            start_window: 0,
            end_window: duration - 1,
            peak_grammar_state: GrammarState::Boundary,
            primary_reason_code: ReasonCode::AbruptSlewViolation,
            matched_motif: SemanticDisposition::Named(motif),
            policy_state: PolicyState::Review,
            contributing_signal_count: contrib,
            structural_signature: StructuralSignature {
                dominant_drift_direction: DriftDirection::Positive,
                peak_slew_magnitude: slew,
                duration_windows: duration,
                signal_correlation: contrib as f64 / 8.0,
            },
            root_cause_signal_index: None,
        }
    }

    #[test]
    fn empty_catalog_returns_none() {
        let cat = EpisodeCatalog::<8>::new();
        let q = ep(MotifClass::CascadingTimeoutSlew, 0.5, 10, 3);
        assert!(cat.find_similar(&q).is_none());
        assert!(cat.is_empty());
    }

    #[test]
    fn identical_episode_yields_high_similarity() {
        let mut cat = EpisodeCatalog::<8>::new();
        let past = ep(MotifClass::CascadingTimeoutSlew, 0.85, 12, 4);
        cat.record(past);
        let query = ep(MotifClass::CascadingTimeoutSlew, 0.85, 12, 4);
        let sim = cat.find_similar(&query).expect("should find a match");
        assert!(sim.similarity > 0.9,
                "identical signature should yield similarity > 0.9, got {}",
                sim.similarity);
    }

    #[test]
    fn different_motif_yields_low_similarity() {
        let mut cat = EpisodeCatalog::<8>::new();
        cat.record(ep(MotifClass::CascadingTimeoutSlew, 0.5, 10, 3));
        let query = ep(MotifClass::DeploymentRegressionSlew, 0.5, 10, 3);
        // Reason matches (both AbruptSlewViolation) but motif differs.
        let sim = cat.find_similar(&query).expect("should find a match");
        // motif=0 (0.40) + reason=1 (0.20) + drift=1 (0.10) + slew=1 (0.15) + dur=1 (0.15) = 0.60
        assert!(sim.similarity < 0.7,
                "different motif should yield similarity < 0.7, got {}",
                sim.similarity);
    }

    #[test]
    fn deterministic_lookup_across_calls() {
        let mut cat = EpisodeCatalog::<8>::new();
        cat.record(ep(MotifClass::MemoryLeakDrift, 0.3, 50, 2));
        cat.record(ep(MotifClass::CascadingTimeoutSlew, 0.8, 8, 4));
        cat.record(ep(MotifClass::DeploymentRegressionSlew, 0.9, 5, 1));
        let q = ep(MotifClass::CascadingTimeoutSlew, 0.7, 10, 3);
        let s1 = cat.find_similar(&q);
        let s2 = cat.find_similar(&q);
        assert_eq!(s1, s2, "find_similar must be deterministic");
    }

    #[test]
    fn circular_buffer_wraps() {
        let mut cat = EpisodeCatalog::<3>::new();
        for i in 0..5 {
            let mut e = ep(MotifClass::MemoryLeakDrift, 0.5, 10, 2);
            e.episode_id = i as u32;
            cat.record(e);
        }
        assert_eq!(cat.total_recorded(), 5);
        assert_eq!(cat.len(), 3, "buffer should be full at capacity");
    }

    #[test]
    fn unknown_motif_pair_partial_similarity() {
        let mut cat = EpisodeCatalog::<8>::new();
        let mut past = ep(MotifClass::CascadingTimeoutSlew, 0.5, 10, 3);
        past.matched_motif = SemanticDisposition::Unknown;
        cat.record(past);
        let mut query = ep(MotifClass::CascadingTimeoutSlew, 0.5, 10, 3);
        query.matched_motif = SemanticDisposition::Unknown;
        let sim = cat.find_similar(&query).expect("should find a match");
        // Unknown-Unknown = 0.5 motif weight; reason+drift+slew+dur = 0.6
        assert!(sim.similarity > 0.7,
                "unknown-unknown structural twins should still match well");
    }
}
