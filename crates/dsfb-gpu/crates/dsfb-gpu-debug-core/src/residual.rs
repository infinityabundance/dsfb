//! Residual extraction.
//!
//! Given a window-feature grid, this stage emits the deviation of each
//! cell from the configured baseline. The baseline is intentionally
//! simple in v0 — a global constant latency of 1 ms and zero baseline
//! error rate — because the prior-art claim is about the architecture
//! and the auditability of the chain, not about adaptive baselining.
//! A per-entity, learned baseline is straightforward future work.
//!
//! Conversion to Q16.16 happens here. The residual is the first artifact
//! whose values must live in the same numeric domain as the EWMA
//! recurrence in the sign stage; that domain is Q16.16 by contract.
//!
//! Rounding: integer truncation toward zero on the divide. The CUDA
//! kernel in Section H uses the same i64-widen-then-truncating-divide
//! pattern so the two backends produce identical cell values bit-for-
//! bit.

#![cfg(feature = "std")]

use std::vec::Vec;

use crate::fixed::Q16;
use crate::window::WindowFeature;

/// Baseline against which the residual is computed.
///
/// Fields are stored as integers so the baseline itself can be hashed
/// into the contract without floating-point ambiguity. The `_us` and
/// `_per_event_q16_raw` suffixes spell out the units explicitly because
/// they are easy to confuse otherwise.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Baseline {
    /// Baseline mean latency in microseconds. Subtracted from each cell's
    /// observed mean before the Q16 conversion.
    pub latency_us: u32,
    /// Baseline error rate as a raw Q16.16 fraction. `0` in v0; reserved
    /// so a future contract can declare a non-zero floor (e.g., for noisy
    /// background traffic).
    pub error_rate_q16_raw: i32,
}

impl Baseline {
    /// The canonical v0 baseline: 1 ms baseline latency, zero baseline
    /// error rate. Matches the fixture's clean-window posture.
    pub const CANONICAL: Self = Self {
        latency_us: 1_000,
        error_rate_q16_raw: 0,
    };
}

/// One `(window, entity)` residual cell.
///
/// `residual_latency_q` is in Q16.16 milliseconds (so the value `1.0`
/// means "one millisecond above baseline"). `residual_error_q` is a Q16
/// fraction in `[-1.0, 1.0]` — error rate minus baseline error rate.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ResidualCell {
    /// Window index this cell belongs to. Carried for downstream lookups
    /// even though the entity-major layout is enough to identify the cell.
    pub window_idx: u32,
    /// Entity this cell belongs to.
    pub entity_id: u32,
    /// Mean-latency deviation from baseline, in Q16.16 milliseconds.
    pub residual_latency_q: Q16,
    /// Error-rate deviation from baseline, as a Q16.16 fraction.
    pub residual_error_q: Q16,
}

/// Convert a signed microsecond delta to Q16.16 milliseconds.
///
/// The transformation is `q.raw = (us * 65_536) / 1_000`, executed in
/// `i64` to avoid mid-flight overflow, with truncation toward zero on
/// the divide and final saturation to `i32`. Integer truncation rather
/// than banker's rounding is chosen here because the divide is between
/// two integers and there is no fractional bit to round at — banker's
/// rounding would degenerate to truncation in every well-formed case.
#[must_use]
pub fn q16_ms_from_us(us: i64) -> Q16 {
    let numer = us.saturating_mul(1_i64 << 16);
    let q = numer / 1_000;
    let clamped = if q > i64::from(i32::MAX) {
        i32::MAX
    } else if q < i64::from(i32::MIN) {
        i32::MIN
    } else {
        q as i32
    };
    Q16::from_raw(clamped)
}

/// Convert an error fraction `(error_count, event_count)` to Q16.16
/// directly. If `event_count` is zero the result is `Q16::ZERO` because
/// an empty cell has no observed error rate.
#[must_use]
pub fn q16_error_rate(error_count: u32, event_count: u32) -> Q16 {
    if event_count == 0 {
        return Q16::ZERO;
    }
    let numer = i64::from(error_count).saturating_mul(1_i64 << 16);
    let q = numer / i64::from(event_count);
    Q16::from_raw(q as i32)
}

/// Compute the residual grid from window features.
///
/// Output shape and order mirror the input: entity-major, length
/// `n_windows * n_entities`. The function is pure — no I/O, no shared
/// state — so two calls with the same arguments produce byte-identical
/// output, which is the property the case-file hash chain depends on.
#[must_use]
pub fn compute(features: &[WindowFeature], baseline: &Baseline) -> Vec<ResidualCell> {
    let mut out: Vec<ResidualCell> = Vec::with_capacity(features.len());
    for cell in features {
        let mean_us = cell.mean_latency_us();
        let delta_us = i64::from(mean_us) - i64::from(baseline.latency_us);
        let residual_latency_q = q16_ms_from_us(delta_us);

        let observed_error_q = q16_error_rate(cell.error_count, cell.event_count);
        let baseline_error_q = Q16::from_raw(baseline.error_rate_q16_raw);
        let residual_error_q = observed_error_q.sat_sub(baseline_error_q);

        out.push(ResidualCell {
            window_idx: cell.window_idx,
            entity_id: cell.entity_id,
            residual_latency_q,
            residual_error_q,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{synthesize, DEFAULT_SEED, N_ENTITIES, N_WINDOWS, WINDOW_SIZE_NS};
    use crate::window::compute_features;

    #[test]
    fn baseline_cell_yields_zero_residual() {
        // A cell with exactly the baseline mean latency and zero errors
        // must produce a zero residual on both axes.
        let feature = WindowFeature {
            window_idx: 0,
            entity_id: 0,
            event_count: 10,
            error_count: 0,
            sum_latency_us: 10_000, // 10 events × 1 000 µs each = baseline.
        };
        let cells = compute(&[feature], &Baseline::CANONICAL);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].residual_latency_q, Q16::ZERO);
        assert_eq!(cells[0].residual_error_q, Q16::ZERO);
    }

    #[test]
    fn ten_ms_mean_latency_produces_residual_of_nine() {
        // 10 ms mean − 1 ms baseline = 9 ms, which in Q16 is 9 * 65_536.
        let feature = WindowFeature {
            window_idx: 0,
            entity_id: 0,
            event_count: 10,
            error_count: 0,
            sum_latency_us: 100_000,
        };
        let cells = compute(&[feature], &Baseline::CANONICAL);
        assert_eq!(cells[0].residual_latency_q.raw(), 9 * 65_536);
    }

    #[test]
    fn full_error_window_produces_residual_one() {
        // All 10 events errored → error_rate = 1.0 → Q16::ONE.
        let feature = WindowFeature {
            window_idx: 0,
            entity_id: 0,
            event_count: 10,
            error_count: 10,
            sum_latency_us: 10_000,
        };
        let cells = compute(&[feature], &Baseline::CANONICAL);
        assert_eq!(cells[0].residual_error_q, Q16::ONE);
    }

    #[test]
    fn half_error_window_produces_residual_half() {
        let feature = WindowFeature {
            window_idx: 0,
            entity_id: 0,
            event_count: 10,
            error_count: 5,
            sum_latency_us: 10_000,
        };
        let cells = compute(&[feature], &Baseline::CANONICAL);
        // 5/10 = 0.5. In Q16 that's raw = 32_768.
        assert_eq!(cells[0].residual_error_q.raw(), 0x8000);
    }

    #[test]
    fn empty_cell_produces_negative_baseline_latency_residual() {
        // An empty cell has mean = 0, so the residual is -baseline = -1 ms.
        let feature = WindowFeature {
            window_idx: 0,
            entity_id: 0,
            event_count: 0,
            error_count: 0,
            sum_latency_us: 0,
        };
        let cells = compute(&[feature], &Baseline::CANONICAL);
        assert_eq!(cells[0].residual_latency_q.raw(), -65_536);
        assert_eq!(cells[0].residual_error_q, Q16::ZERO);
    }

    #[test]
    fn residuals_are_deterministic_over_synthesized_fixture() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let a = compute(&features, &Baseline::CANONICAL);
        let b = compute(&features, &Baseline::CANONICAL);
        assert_eq!(a, b);
    }

    #[test]
    fn ramp_residual_is_strongly_positive_on_entity_three() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let cells = compute(&features, &Baseline::CANONICAL);
        // Last window of the ramp (35) on entity 3.
        let idx = WindowFeature::flat_index(3, 35, N_WINDOWS);
        assert!(
            cells[idx].residual_latency_q.raw() > 30 * 65_536,
            "ramp cell residual latency raw={}",
            cells[idx].residual_latency_q.raw()
        );
    }

    #[test]
    fn burst_residual_lights_up_error_axis_on_entity_seven() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let cells = compute(&features, &Baseline::CANONICAL);
        // Middle of the burst (62) on entity 7.
        let idx = WindowFeature::flat_index(7, 62, N_WINDOWS);
        assert!(
            cells[idx].residual_error_q.raw() > 0x4000,
            "burst cell residual error raw={}",
            cells[idx].residual_error_q.raw()
        );
    }

    #[test]
    fn shock_residual_spikes_then_recovers_on_entity_eleven() {
        let events = synthesize(DEFAULT_SEED);
        let features = compute_features(&events, N_WINDOWS, N_ENTITIES, WINDOW_SIZE_NS);
        let cells = compute(&features, &Baseline::CANONICAL);
        let shock_idx = WindowFeature::flat_index(11, 90, N_WINDOWS);
        let recovery_end_idx = WindowFeature::flat_index(11, 95, N_WINDOWS);
        // Spike must be >>recovery-end residual.
        assert!(
            cells[shock_idx].residual_latency_q.raw()
                > cells[recovery_end_idx].residual_latency_q.raw() * 4,
            "shock raw={}, recovery raw={}",
            cells[shock_idx].residual_latency_q.raw(),
            cells[recovery_end_idx].residual_latency_q.raw()
        );
    }
}
