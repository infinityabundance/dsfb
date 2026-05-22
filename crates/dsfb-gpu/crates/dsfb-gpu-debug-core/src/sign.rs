//! Drift / slew sign tuples.
//!
//! For each `(window, entity)` residual cell, the sign stage emits the
//! L1 norm of the residual along with two time-domain derivatives:
//!
//! * `drift` — an exponentially weighted moving average of the norm
//!   along the entity's time series. The recurrence is the standard
//!   leaky-integrator form `drift_{w} = drift_{w-1} + α(norm_w −
//!   drift_{w-1})`, executed in Q16.16 with the contract-locked α
//!   (`ewma_alpha_q16_raw = 0x2000`, i.e. 0.125).
//! * `slew` — the first difference of the norm, `slew_w = norm_w −
//!   norm_{w-1}`. Slew captures sharp transitions that an EWMA would
//!   smear out. Both signed and unsigned slew variants are commonly
//!   useful in trace diagnostics; for the v0 detectors we keep slew
//!   signed and let downstream stages take the absolute value when they
//!   care.
//!
//! The two derivatives are computed **per entity, in window order**.
//! Each entity's time series is processed independently — there is no
//! cross-entity coupling at this stage. This matches the architecture
//! decision to keep cross-entity (axis-5 "entity locality") fusion in
//! the bank stage.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::fixed::Q16;
use crate::residual::ResidualCell;

/// One `(window, entity)` sign cell.
///
/// `norm_q` is the L1 sum of the residual latency and error components.
/// `drift_q` and `slew_q` are the EWMA and first-difference of the norm,
/// respectively. All three live in the same Q16.16 milliseconds-plus-
/// fraction domain so they can be compared head-to-head against
/// detector thresholds in Section D without per-axis scaling.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct SignCell {
    /// Window index this cell belongs to.
    pub window_idx: u32,
    /// Entity this cell belongs to.
    pub entity_id: u32,
    /// L1 norm of the residual.
    pub norm_q: Q16,
    /// EWMA drift of the norm along the entity's time series.
    pub drift_q: Q16,
    /// First-difference slew of the norm.
    pub slew_q: Q16,
}

/// Compute the sign grid from residual cells.
///
/// `n_windows` and `n_entities` describe the input grid's shape; the
/// function asserts (in debug builds) that the input length matches
/// `n_windows * n_entities`. `alpha` is the locked EWMA coefficient,
/// usually fetched from `contract.numeric.ewma_alpha_q16_raw`.
///
/// The L1 norm is computed as `|residual_latency| + |residual_error|`.
/// L2 would require a square root; we avoid that in v0 because a
/// deterministic, cross-platform-bit-exact Q16.16 sqrt would need a
/// lookup table and complicate the GPU mirror. The L1 norm is
/// sufficient to drive the detector layer.
#[must_use]
pub fn compute(
    residuals: &[ResidualCell],
    alpha: Q16,
    n_windows: u32,
    n_entities: u32,
) -> Vec<SignCell> {
    debug_assert_eq!(
        residuals.len(),
        (n_windows as usize) * (n_entities as usize),
        "residual grid shape mismatch"
    );

    let mut out: Vec<SignCell> = Vec::with_capacity(residuals.len());
    // Entity-major iteration so the EWMA state can be carried in a single
    // Q16 across the inner window loop without any cross-entity bleed.
    for entity_id in 0..n_entities {
        let mut drift_state = Q16::ZERO;
        let mut prev_norm = Q16::ZERO;
        for window_idx in 0..n_windows {
            let idx = (entity_id * n_windows + window_idx) as usize;
            let cell = &residuals[idx];

            let norm = cell
                .residual_latency_q
                .abs()
                .sat_add(cell.residual_error_q.abs());

            // For the very first window of each entity the slew is zero
            // by convention: no preceding window means no first-difference.
            let slew = if window_idx == 0 {
                Q16::ZERO
            } else {
                norm.sat_sub(prev_norm)
            };

            // Update the EWMA drift before recording the cell so the
            // first window of an entity carries an EWMA equal to that
            // window's norm (because we lerp from zero with α applied to
            // the full norm). That matches what a fresh time series would
            // observe in the limit; an entity's history starts at zero
            // and the first observation contributes the first α-scaled
            // step.
            drift_state = drift_state.lerp(norm, alpha);

            out.push(SignCell {
                window_idx,
                entity_id,
                norm_q: norm,
                drift_q: drift_state,
                slew_q: slew,
            });

            prev_norm = norm;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
    use crate::residual::{compute as residual_compute, Baseline, ResidualCell};
    use crate::window::{compute_features, WindowFeature};

    const ALPHA: Q16 = Q16::from_raw(0x2000); // 0.125 — contract-locked.

    fn make_grid(
        n_windows: u32,
        n_entities: u32,
        cells: &[(u32, u32, i32, i32)],
    ) -> Vec<ResidualCell> {
        let mut grid: Vec<ResidualCell> = (0..n_entities)
            .flat_map(|e| {
                (0..n_windows).map(move |w| ResidualCell {
                    window_idx: w,
                    entity_id: e,
                    residual_latency_q: Q16::ZERO,
                    residual_error_q: Q16::ZERO,
                })
            })
            .collect();
        for &(e, w, lat_raw, err_raw) in cells {
            let idx = (e * n_windows + w) as usize;
            grid[idx].residual_latency_q = Q16::from_raw(lat_raw);
            grid[idx].residual_error_q = Q16::from_raw(err_raw);
        }
        grid
    }

    #[test]
    fn first_window_has_zero_slew() {
        let grid = make_grid(2, 1, &[(0, 0, 5 * 65_536, 0)]);
        let signs = compute(&grid, ALPHA, 2, 1);
        assert_eq!(signs[0].slew_q, Q16::ZERO);
    }

    #[test]
    fn norm_is_l1_sum_of_residuals() {
        let grid = make_grid(1, 1, &[(0, 0, 3 * 65_536, -2 * 65_536)]);
        let signs = compute(&grid, ALPHA, 1, 1);
        // |3| + |-2| = 5.
        assert_eq!(signs[0].norm_q.raw(), 5 * 65_536);
    }

    #[test]
    fn ewma_drift_converges_toward_constant_input() {
        // Three consecutive windows with norm=100. Drift should ramp up
        // each window: 0 → 12.5 → 23.44 → 33.0 approximately.
        let grid = make_grid(
            3,
            1,
            &[
                (0, 0, 100 * 65_536, 0),
                (0, 1, 100 * 65_536, 0),
                (0, 2, 100 * 65_536, 0),
            ],
        );
        let signs = compute(&grid, ALPHA, 3, 1);
        assert!(signs[0].drift_q.raw() < signs[1].drift_q.raw());
        assert!(signs[1].drift_q.raw() < signs[2].drift_q.raw());
        // First step is exactly α * norm = 0.125 * 100 = 12.5.
        // Q16 representation: 12.5 = 12 * 65_536 + 32_768 = 819_200.
        assert_eq!(signs[0].drift_q.raw(), 819_200);
    }

    #[test]
    fn slew_picks_up_step_changes() {
        let grid = make_grid(2, 1, &[(0, 0, 65_536, 0), (0, 1, 10 * 65_536, 0)]);
        let signs = compute(&grid, ALPHA, 2, 1);
        // Window 1 slew = 10 - 1 = 9 (in Q16).
        assert_eq!(signs[1].slew_q.raw(), 9 * 65_536);
    }

    #[test]
    fn entity_streams_do_not_bleed_into_each_other() {
        // Entity 0 sees a hot signal; entity 1 stays clean.
        let grid = make_grid(2, 2, &[(0, 0, 50 * 65_536, 0), (0, 1, 50 * 65_536, 0)]);
        let signs = compute(&grid, ALPHA, 2, 2);
        // Layout is entity-major: index = entity_id * n_windows + window_idx.
        // Entity 0, window 1 → index 1. Entity 1, window 1 → index 3.
        let e0_w1 = &signs[1];
        let e1_w1 = &signs[3];
        assert!(e0_w1.drift_q.raw() > 0);
        assert_eq!(e1_w1.drift_q.raw(), 0);
    }

    #[test]
    fn sign_pipeline_is_deterministic_over_synthesized_fixture() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let a = compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        let b = compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);
        assert_eq!(a, b);
    }

    #[test]
    fn shock_window_carries_the_highest_positive_slew_for_entity_eleven() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let residuals = residual_compute(&features, &Baseline::CANONICAL);
        let signs = compute(&residuals, ALPHA, N_WINDOWS, N_ENTITIES);

        // Find the maximum positive slew across entity 11. It should land
        // at the shock window (90), where the latency jumps two orders of
        // magnitude above baseline.
        let mut best = (0u32, i32::MIN);
        for window_idx in 0..N_WINDOWS {
            let idx = WindowFeature::flat_index(11, window_idx, N_WINDOWS);
            let slew = signs[idx].slew_q.raw();
            if slew > best.1 {
                best = (window_idx, slew);
            }
        }
        assert_eq!(best.0, 90);
    }
}
