//! Candidate-interval extraction.
//!
//! The candidate stage collapses contiguous runs of "interesting"
//! consensus cells along each entity's window axis into compact
//! `CandidateInterval` records. These are **not** semantic episodes —
//! the Semantic Non-Bypass Axiom forbids the pipeline from declaring an
//! `Episode` straight from accelerated detector evidence. The
//! `CandidateInterval` is the structured handoff to the bank stage
//! (Section F), which is the only point in the pipeline that can mint
//! an admissible `Episode`.
//!
//! Contiguity rule: a cell is "interesting" when `detector_count ≥
//! min_detector_count` OR `axis1_residual_q.raw() ≥ min_residual_q_raw`.
//! Two interesting cells belong to the same interval iff they are
//! adjacent in window order for the same entity. We do not merge across
//! a one-window gap in v0; that smoothing rule is bank-stage policy.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::consensus::ConsensusCell;
use crate::fixed::Q16;
use crate::motif::MotifClass;

/// Configuration for candidate-interval extraction.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CandidateConfig {
    /// Minimum detector count for a cell to be "interesting".
    pub min_detector_count: u32,
    /// Minimum Q16 residual norm for a cell to be "interesting".
    pub min_residual_q_raw: i32,
    /// Minimum length of an interval to be emitted. Intervals shorter
    /// than this are suppressed (treated as noise).
    pub min_length_windows: u32,
}

impl CandidateConfig {
    /// Canonical defaults. Chosen so the three fixture episodes (ramp:
    /// 16 windows, burst: 6 windows, shock + recovery: 6 windows) each
    /// produce a single candidate interval that the bank stage can
    /// admit. Single-window confuser candidates are surfaced separately
    /// (length-1 intervals) so the bank can apply the confuser
    /// suppression axis.
    pub const CANONICAL: Self = Self {
        min_detector_count: 2,
        min_residual_q_raw: 3 * 65_536, // 3 ms
        min_length_windows: 1,
    };
}

/// A contiguous run of "interesting" consensus cells for a single
/// entity, summarized.
///
/// `start_window..end_window` is inclusive of `start_window` and
/// exclusive of `end_window` (standard half-open range). The summary
/// fields are the peak observed within the interval — they are
/// information-only inputs to the bank stage and never become final
/// verdicts on their own.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct CandidateInterval {
    /// Entity this candidate belongs to.
    pub entity_id: u32,
    /// First window of the interval (inclusive).
    pub start_window: u32,
    /// One past the last window of the interval (exclusive).
    pub end_window: u32,
    /// Number of windows spanned. Convenient cached length so consumers
    /// don't have to recompute `end - start`.
    pub length_windows: u32,
    /// OR of the detector masks across all cells in the interval. Carried
    /// into the bank stage so the admissibility check can look at
    /// "which detectors fired somewhere in this window range".
    pub union_mask: u32,
    /// Peak `axis1_residual_q` observed within the interval.
    pub peak_residual_q: Q16,
    /// Peak `axis2_drift_q` observed within the interval.
    pub peak_drift_q: Q16,
    /// Peak `axis3_slew_q` observed within the interval.
    pub peak_slew_q: Q16,
    /// Peak `axis4_temporal_q` observed within the interval.
    pub peak_temporal_q: Q16,
    /// Peak `axis7_consensus_q` observed within the interval.
    pub peak_consensus_q: Q16,
    /// R.5 axis-5 entity-locality average. Precomputed as
    /// `sum(axis7_consensus_q for this entity over [start_window, end_window)) / span`,
    /// in Q16 raw units. The bank's axis-5 gate reads this directly
    /// instead of summing the consensus grid at admission time. Same
    /// integer arithmetic on CPU and GPU; pinned to byte-identical
    /// values by the cross-stage chain tests.
    pub entity_avg_q: Q16,
    /// R.5 axis-5 grid-locality average. Precomputed as
    /// `sum(axis7_consensus_q for all entities over [start_window, end_window)) / (n_entities * span)`,
    /// in Q16 raw units. Paired with `entity_avg_q` for the bank's
    /// "entity is exceptionally hot relative to grid" predicate.
    pub grid_avg_q: Q16,
}

impl CandidateInterval {
    /// Predicate: was a specific motif present anywhere in this interval?
    #[must_use]
    pub const fn covers(&self, class: MotifClass) -> bool {
        (self.union_mask & class.bit_mask()) != 0
    }
}

#[inline]
const fn flat(entity_id: u32, window_idx: u32, n_windows: u32) -> usize {
    (entity_id * n_windows + window_idx) as usize
}

/// "Interesting" predicate. The candidate stage gathers contiguous
/// stretches of interesting cells per entity.
///
/// The detector stage's CleanWindowStability bit only fires when every
/// other motif is silent, so a cell whose detector_count is 1 with only
/// the clean bit set still trips `count_ok` for `min_detector_count = 1`.
/// The canonical config uses `min_detector_count = 2` to exclude those
/// clean cells without needing the underlying mask here.
fn is_interesting(cell: &ConsensusCell, config: &CandidateConfig) -> bool {
    let count_ok = cell.detector_count >= config.min_detector_count;
    let residual_ok = cell.axis1_residual_q.raw() >= config.min_residual_q_raw;
    count_ok || residual_ok
}

/// Sweep the consensus grid and emit candidate intervals in canonical
/// `(entity_id, start_window)` order.
#[must_use]
pub fn prepare(
    consensus: &[ConsensusCell],
    n_windows: u32,
    n_entities: u32,
    config: &CandidateConfig,
) -> Vec<CandidateInterval> {
    let mut out: Vec<CandidateInterval> = Vec::new();
    for entity_id in 0..n_entities {
        let mut run_start: Option<u32> = None;
        let mut acc = CandidateInterval {
            entity_id,
            start_window: 0,
            end_window: 0,
            length_windows: 0,
            union_mask: 0,
            peak_residual_q: Q16::ZERO,
            peak_drift_q: Q16::ZERO,
            peak_slew_q: Q16::ZERO,
            peak_temporal_q: Q16::ZERO,
            peak_consensus_q: Q16::ZERO,
            // Axis-5 averages are filled by the second pass in
            // `prepare_with_detectors`. `prepare()` alone (the unit-test
            // path) leaves them at zero — that is the documented v0
            // behaviour and the cause of the byte refresh in R.5.
            entity_avg_q: Q16::ZERO,
            grid_avg_q: Q16::ZERO,
        };

        for window_idx in 0..n_windows {
            let idx = flat(entity_id, window_idx, n_windows);
            let cell = &consensus[idx];
            let interesting = is_interesting(cell, config);

            if interesting {
                if run_start.is_none() {
                    run_start = Some(window_idx);
                    acc = CandidateInterval {
                        entity_id,
                        start_window: window_idx,
                        end_window: window_idx + 1,
                        length_windows: 1,
                        union_mask: 0,
                        peak_residual_q: Q16::ZERO,
                        peak_drift_q: Q16::ZERO,
                        peak_slew_q: Q16::ZERO,
                        peak_temporal_q: Q16::ZERO,
                        peak_consensus_q: Q16::ZERO,
                        // Axis-5 averages computed in second pass.
                        entity_avg_q: Q16::ZERO,
                        grid_avg_q: Q16::ZERO,
                    };
                } else {
                    acc.end_window = window_idx + 1;
                    acc.length_windows = acc.end_window - acc.start_window;
                }
                acc.union_mask |= 0u32; // detector_mask is not stored on the
                                        // consensus cell — we use detector_count for axis 7. The
                                        // bank stage receives the union mask from the candidate;
                                        // for v0 we synthesize it from the cell's non-clean
                                        // signature by reading the detector grid out-of-band. See
                                        // `prepare_with_detectors` below for the full version.
                acc.peak_residual_q = peak(acc.peak_residual_q, cell.axis1_residual_q);
                acc.peak_drift_q = peak(acc.peak_drift_q, cell.axis2_drift_q);
                acc.peak_slew_q = peak(acc.peak_slew_q, cell.axis3_slew_q);
                acc.peak_temporal_q = peak(acc.peak_temporal_q, cell.axis4_temporal_q);
                acc.peak_consensus_q = peak(acc.peak_consensus_q, cell.axis7_consensus_q);
            } else if run_start.is_some() {
                if acc.length_windows >= config.min_length_windows {
                    out.push(acc);
                }
                run_start = None;
            }
        }
        // Close out any open run at the entity's end-of-time.
        if run_start.is_some() && acc.length_windows >= config.min_length_windows {
            out.push(acc);
        }
    }
    out
}

/// Variant of `prepare` that also folds the per-cell detector masks into
/// the interval's `union_mask`. Takes the detector grid alongside the
/// consensus grid; the two are guaranteed to be the same shape by the
/// upstream stages.
#[must_use]
pub fn prepare_with_detectors(
    consensus: &[ConsensusCell],
    detector_masks: &[u32],
    n_windows: u32,
    n_entities: u32,
    config: &CandidateConfig,
) -> Vec<CandidateInterval> {
    debug_assert_eq!(consensus.len(), detector_masks.len());

    let mut intervals = prepare(consensus, n_windows, n_entities, config);
    // Second pass: fold the union mask in. Mostly equivalent to inlining
    // the logic above, kept as a second pass so the primary `prepare`
    // stays single-purpose and easy to read.
    //
    // R.5: the same second pass now also precomputes the axis-5
    // entity-locality and grid-locality averages over the candidate's
    // window range. The arithmetic is intentionally identical to what
    // `bank::try_admit` did inline before R.5 (i64 sums, i32 result
    // via integer division). After R.5 the bank reads these
    // precomputed Q16 fields instead of re-summing the consensus grid
    // at admission time. Byte equivalence between this CPU path and
    // the CUDA `candidate_collapse_kernel` axis-5 computation is
    // pinned by the cross-stage chain tests.
    for interval in &mut intervals {
        let mut mask = 0u32;
        let mut entity_sum: i64 = 0;
        let mut grid_sum: i64 = 0;
        let mut grid_count: i64 = 0;
        for w in interval.start_window..interval.end_window {
            mask |= detector_masks[flat(interval.entity_id, w, n_windows)];
            for entity_id in 0..n_entities {
                let idx = flat(entity_id, w, n_windows);
                let q = i64::from(consensus[idx].axis7_consensus_q.raw());
                grid_sum += q;
                grid_count += 1;
                if entity_id == interval.entity_id {
                    entity_sum += q;
                }
            }
        }
        let span = i64::from(interval.end_window - interval.start_window).max(1);
        let entity_avg_raw = (entity_sum / span) as i32;
        let grid_avg_raw = if grid_count > 0 {
            (grid_sum / grid_count) as i32
        } else {
            0
        };
        interval.union_mask = mask;
        interval.entity_avg_q = Q16::from_raw(entity_avg_raw);
        interval.grid_avg_q = Q16::from_raw(grid_avg_raw);
    }
    intervals
}

#[inline]
fn peak(a: Q16, b: Q16) -> Q16 {
    if b.raw() > a.raw() {
        b
    } else {
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::form as consensus_form;
    use crate::detector::{evaluate as detector_evaluate, DetectorThresholds};
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
    use crate::residual::{compute as residual_compute, Baseline};
    use crate::sign::compute as sign_compute;
    use crate::window::compute_features;

    const ALPHA: Q16 = Q16::from_raw(0x2000);

    fn full_pipeline() -> (Vec<ConsensusCell>, Vec<u32>) {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = sign_compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let detectors = detector_evaluate(
            &residuals,
            &signs,
            &DetectorThresholds::CANONICAL,
            N_WINDOWS,
            N_ENTITIES,
        );
        let consensus = consensus_form(&signs, &detectors, N_WINDOWS, N_ENTITIES);
        let masks: Vec<u32> = detectors.iter().map(|d| d.detector_mask).collect();
        (consensus, masks)
    }

    #[test]
    fn candidate_extraction_is_deterministic() {
        let (consensus, masks) = full_pipeline();
        let a = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        let b = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn ramp_episode_yields_a_candidate_on_entity_three() {
        let (consensus, masks) = full_pipeline();
        let intervals = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        // At least one candidate on entity 3 whose start is at or before
        // window 25 and whose end is at or after window 30.
        let any_ramp = intervals
            .iter()
            .any(|c| c.entity_id == 3 && c.start_window <= 25 && c.end_window >= 30);
        assert!(
            any_ramp,
            "no ramp candidate found among {} intervals",
            intervals.len()
        );
    }

    #[test]
    fn burst_episode_yields_a_candidate_on_entity_seven() {
        let (consensus, masks) = full_pipeline();
        let intervals = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        let any_burst = intervals
            .iter()
            .any(|c| c.entity_id == 7 && c.start_window <= 62 && c.end_window >= 65);
        assert!(any_burst, "no burst candidate found");
    }

    #[test]
    fn shock_episode_yields_a_candidate_on_entity_eleven() {
        let (consensus, masks) = full_pipeline();
        let intervals = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        let any_shock = intervals
            .iter()
            .any(|c| c.entity_id == 11 && c.start_window <= 90 && c.end_window >= 91);
        assert!(any_shock, "no shock candidate found");
    }

    #[test]
    fn candidates_carry_the_union_mask() {
        let (consensus, masks) = full_pipeline();
        let intervals = prepare_with_detectors(
            &consensus,
            &masks,
            N_WINDOWS,
            N_ENTITIES,
            &CandidateConfig::CANONICAL,
        );
        // The ramp interval must cover both ResidualSpike and DriftRamp.
        for interval in &intervals {
            if interval.entity_id == 3 && interval.start_window <= 25 && interval.end_window >= 30 {
                assert!(interval.covers(MotifClass::ResidualSpike));
                assert!(interval.covers(MotifClass::DriftRamp));
                return;
            }
        }
        panic!("ramp interval not found");
    }
}
