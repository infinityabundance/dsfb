//! Window-level feature aggregation.
//!
//! The windowing stage collapses the raw trace-event stream into a
//! `(window, entity)` grid of summary statistics. It runs on the CPU in
//! v0 because a parallel GPU implementation that respects determinism
//! would require atomics or a multi-pass reduction; the spec explicitly
//! permits pre-windowing on the CPU as a bounded simplification.
//!
//! Output shape: an entity-major flat `Vec<WindowFeature>` of length
//! `n_windows * n_entities`. Entity-major (index = `entity_id *
//! n_windows + window_idx`) makes the per-entity time series contiguous,
//! which is what the residual and sign stages want.
//!
//! Numeric posture: aggregates are computed in raw integer arithmetic —
//! `event_count: u32`, `error_count: u32`, `sum_latency_us: u64` — and
//! only converted to Q16.16 at the residual boundary. This preserves
//! exact integer semantics through the aggregation, which is identical on
//! CPU and GPU and survives every reordering that a non-atomic GPU
//! reduction might still produce when integer addition is associative.

#![cfg(feature = "std")]

use std::vec;
use std::vec::Vec;

use crate::event::TraceEvent;

/// Summary statistics for one `(window, entity)` cell.
///
/// Carried through the pipeline as the canonical input to the residual
/// stage. The fields here are stored as plain unsigned integers; Q16.16
/// conversion happens in `residual::compute`. The struct is `#[repr(C)]`
/// so the CUDA-side mirror lays out the same fields in the same order.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct WindowFeature {
    /// Window index. Bounded by `n_windows` declared in the contract.
    pub window_idx: u32,
    /// Entity that produced the events aggregated into this cell.
    pub entity_id: u32,
    /// Number of trace events that fell into this cell. Zero is a
    /// well-formed value — empty cells are kept so the downstream grid
    /// is rectangular.
    pub event_count: u32,
    /// Subset of `event_count` whose `error_code` was non-zero.
    pub error_count: u32,
    /// Sum of `latency_us` across all events in this cell. Stored as
    /// `u64` so it cannot overflow at the bounded fixture scale —
    /// 10 000 events × `u32::MAX` µs comfortably fits.
    pub sum_latency_us: u64,
}

impl WindowFeature {
    /// Position in the entity-major flat layout. Used by the downstream
    /// stages to look up a specific cell directly without scanning.
    #[must_use]
    pub const fn flat_index(entity_id: u32, window_idx: u32, n_windows: u32) -> usize {
        (entity_id * n_windows + window_idx) as usize
    }

    /// Mean latency in microseconds for this cell, or `0` if the cell is
    /// empty. Integer truncation, deterministically. The Q16 conversion
    /// happens at the residual stage so the windowing output stays an
    /// integer artifact.
    #[must_use]
    pub const fn mean_latency_us(&self) -> u32 {
        if self.event_count == 0 {
            0
        } else {
            // sum / count cannot exceed u32::MAX because every individual
            // sample was already a u32. Safe truncation.
            (self.sum_latency_us / self.event_count as u64) as u32
        }
    }
}

/// Compute the `(window, entity)` feature grid from a slice of trace
/// events.
///
/// Events whose `window_index` or `entity_id` falls outside the bounds
/// declared by the contract are silently dropped. This is the only
/// out-of-bounds tolerance in the pipeline — every other stage rejects
/// out-of-shape inputs.
///
/// Determinism: the output cells are produced in canonical
/// `(entity, window)` order regardless of input order, so two calls with
/// the same event slice yield byte-identical output buffers.
#[must_use]
pub fn compute_features(
    events: &[TraceEvent],
    n_windows: u32,
    n_entities: u32,
    window_size_ns: u64,
) -> Vec<WindowFeature> {
    let len = (n_windows as usize) * (n_entities as usize);
    let mut grid: Vec<WindowFeature> = vec![WindowFeature::default(); len];

    // Seed the position metadata in canonical order so even empty cells
    // carry the right `(window_idx, entity_id)` for downstream lookups.
    for entity_id in 0..n_entities {
        for window_idx in 0..n_windows {
            let idx = WindowFeature::flat_index(entity_id, window_idx, n_windows);
            grid[idx].entity_id = entity_id;
            grid[idx].window_idx = window_idx;
        }
    }

    for event in events {
        let w = event.window_index(window_size_ns);
        if w >= n_windows || event.entity_id >= n_entities {
            continue;
        }
        let idx = WindowFeature::flat_index(event.entity_id, w, n_windows);
        // Saturating add: even though overflow at v0 scale is impossible,
        // saturating keeps the function total and audit-friendly.
        grid[idx].event_count = grid[idx].event_count.saturating_add(1);
        if event.error_code != 0 {
            grid[idx].error_count = grid[idx].error_count.saturating_add(1);
        }
        grid[idx].sum_latency_us = grid[idx]
            .sum_latency_us
            .saturating_add(u64::from(event.latency_us));
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};

    fn fresh(events: &[TraceEvent]) -> Vec<WindowFeature> {
        compute_features(events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS)
    }

    #[test]
    fn empty_input_yields_rectangular_zero_grid() {
        let grid = compute_features(&[], 4, 2, 1_000_000_000);
        assert_eq!(grid.len(), 8);
        for cell in &grid {
            assert_eq!(cell.event_count, 0);
            assert_eq!(cell.error_count, 0);
            assert_eq!(cell.sum_latency_us, 0);
        }
    }

    #[test]
    fn grid_metadata_is_seeded_in_entity_major_order() {
        let grid = compute_features(&[], 3, 2, 1_000_000_000);
        // Entity 0, windows 0..3, then entity 1, windows 0..3.
        assert_eq!((grid[0].entity_id, grid[0].window_idx), (0, 0));
        assert_eq!((grid[1].entity_id, grid[1].window_idx), (0, 1));
        assert_eq!((grid[2].entity_id, grid[2].window_idx), (0, 2));
        assert_eq!((grid[3].entity_id, grid[3].window_idx), (1, 0));
        assert_eq!((grid[4].entity_id, grid[4].window_idx), (1, 1));
        assert_eq!((grid[5].entity_id, grid[5].window_idx), (1, 2));
    }

    #[test]
    fn events_route_to_their_window_and_entity() {
        let events = [
            TraceEvent::new(0, 0, 0, 1, 0, 100, 200, 0, 0, 0),
            TraceEvent::new(500_000_000, 0, 0, 2, 1, 200, 200, 0, 0, 0),
            TraceEvent::new(1_500_000_000, 1, 0, 3, 0, 300, 500, 500, 0, 0),
        ];
        let grid = compute_features(&events, 2, 2, 1_000_000_000);
        // Cell (entity=0, window=0): two events, sum=300, zero errors.
        let cell00 = &grid[WindowFeature::flat_index(0, 0, 2)];
        assert_eq!(cell00.event_count, 2);
        assert_eq!(cell00.error_count, 0);
        assert_eq!(cell00.sum_latency_us, 300);
        // Cell (entity=1, window=1): one event, error.
        let cell11 = &grid[WindowFeature::flat_index(1, 1, 2)];
        assert_eq!(cell11.event_count, 1);
        assert_eq!(cell11.error_count, 1);
        assert_eq!(cell11.sum_latency_us, 300);
    }

    #[test]
    fn out_of_bounds_events_are_dropped() {
        // Window 5 doesn't exist for n_windows=2; entity 9 doesn't exist
        // for n_entities=2. Both events are silently dropped.
        let events = [
            TraceEvent::new(5_500_000_000, 0, 0, 1, 0, 100, 200, 0, 0, 0),
            TraceEvent::new(0, 9, 0, 2, 0, 200, 200, 0, 0, 0),
        ];
        let grid = compute_features(&events, 2, 2, 1_000_000_000);
        for cell in &grid {
            assert_eq!(cell.event_count, 0);
        }
    }

    #[test]
    fn synthesized_fixture_distributes_across_all_windows() {
        let events = synthesize(DEFAULT_SEED);
        let grid = fresh(&events);
        let total: u64 = grid.iter().map(|c| u64::from(c.event_count)).sum();
        assert_eq!(total as usize, events.len());
        // Every window should have at least some events from at least
        // one entity, given the round-robin distribution.
        for w in 0..N_WINDOWS {
            let in_window: u32 = grid
                .iter()
                .filter(|c| c.window_idx == w)
                .map(|c| c.event_count)
                .sum();
            assert!(in_window > 0, "window {w} is empty");
        }
    }

    #[test]
    fn ramp_episode_elevates_mean_latency() {
        use crate::fixture::{N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
        let events = synthesize(DEFAULT_SEED);
        let grid = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        // Ramp lives on entity 3, windows 20..36. The last ramp window
        // must have notably higher mean latency than the first.
        let cell_first = &grid[WindowFeature::flat_index(3, 20, N_WINDOWS)];
        let cell_last = &grid[WindowFeature::flat_index(3, 35, N_WINDOWS)];
        let mean_first = cell_first.mean_latency_us();
        let mean_last = cell_last.mean_latency_us();
        assert!(
            mean_last > mean_first + 30_000,
            "ramp cell means: first={mean_first} last={mean_last}"
        );
    }

    #[test]
    fn windowing_is_deterministic() {
        let events = synthesize(DEFAULT_SEED);
        let a = fresh(&events);
        let b = fresh(&events);
        assert_eq!(a, b);
    }
}
