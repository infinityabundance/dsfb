//! Shared residual helper for balancing datasets (MIT Cheetah 3 /
//! Mini-Cheetah, IIT iCub push-recovery).
//!
//! Balancing platforms expose a **dual-channel** residual:
//!
//! - `r_F(k) = F_contact,measured(k) − F_contact,planned(k)` — the
//!   whole-body controller's planned ground-reaction / contact-wrench
//!   minus what was actually realised. Rolled forward into the next
//!   MPC horizon and discarded as "tracking error".
//!
//! - `r_ξ(k) = ξ_measured(k) − ξ_model(k)` — the centroidal-momentum
//!   (or full-body centre-of-mass) observer discrepancy between the
//!   IMU-fused estimate and the rigid-body model prediction. Fused
//!   into the state estimate and discarded once consumed.
//!
//! DSFB ingests both channels through the same `observe()` core by
//! combining them into a single scalar residual norm. This module
//! provides the combiner; the per-dataset adapters (Phase 3) supply
//! the raw channels from their respective controllers.

use crate::math;

/// Channel-combination strategy for the dual balancing residual.
///
/// `SumOfSquares` is the default and most conservative: treats both
/// channels as equally important and produces the Euclidean norm.
/// `WeightedSum` allows the caller to bias one channel more — for
/// example, weighting `r_F` higher during stance and `r_ξ` higher
/// during swing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BalancingCombine {
    /// `sqrt(r_F² + r_ξ²)` — Euclidean combination, equal weights.
    SumOfSquares,
    /// `sqrt(w_F · r_F² + w_xi · r_ξ²)` — weighted Euclidean.
    ///
    /// Both weights must be non-negative. A zero weight suppresses
    /// the corresponding channel; typical values are `w_F = 1.0`,
    /// `w_xi = 1.0` for parity with `SumOfSquares`.
    WeightedSum {
        /// Weight on the contact-force residual channel `r_F`.
        w_force: f64,
        /// Weight on the centroidal-momentum residual channel `r_ξ`.
        w_xi: f64,
    },
}

/// Combine a force-residual channel and a centroidal-momentum channel
/// into a single scalar residual norm.
///
/// Returns `None` if both inputs are non-finite or the strategy is
/// mis-configured. Missingness-aware: a finite channel combined with
/// a non-finite one degrades to the finite channel's magnitude.
#[must_use]
pub fn combine_channels(
    r_force: f64,
    r_xi: f64,
    strategy: BalancingCombine,
) -> Option<f64> {
    let f_finite = r_force.is_finite();
    let x_finite = r_xi.is_finite();
    if !f_finite && !x_finite {
        return None;
    }
    debug_assert!(f_finite || x_finite, "guarded above: at least one channel is finite");
    let rf = if f_finite { r_force } else { 0.0 };
    let rx = if x_finite { r_xi } else { 0.0 };
    debug_assert!(rf.is_finite() && rx.is_finite(), "post-degrade rf/rx must be finite");

    let ssq = match strategy {
        BalancingCombine::SumOfSquares => rf * rf + rx * rx,
        BalancingCombine::WeightedSum { w_force, w_xi } => {
            if w_force < 0.0 || w_xi < 0.0 || !w_force.is_finite() || !w_xi.is_finite() {
                return None;
            }
            debug_assert!(w_force >= 0.0 && w_xi >= 0.0, "weights validated above");
            w_force * rf * rf + w_xi * rx * rx
        }
    };
    debug_assert!(ssq >= 0.0, "sum-of-squares is non-negative by construction");

    math::sqrt_f64(ssq)
}

/// Vectorised variant: produce a streaming residual-norm sequence from
/// two aligned channel slices. Returns the number of finite samples
/// written into `out` (never exceeds `out.len()`). Non-finite entries
/// in either channel become zero in the combined residual (per
/// [`combine_channels`] missingness rule).
pub fn combine_stream(
    r_force: &[f64],
    r_xi: &[f64],
    out: &mut [f64],
    strategy: BalancingCombine,
) -> usize {
    debug_assert!(r_force.len() == r_xi.len(), "channels must have equal length");
    debug_assert!(!out.is_empty() || r_force.is_empty(), "non-empty output requires non-empty input");
    let n = r_force.len().min(r_xi.len()).min(out.len());
    debug_assert!(n <= out.len(), "n must respect destination capacity");
    let mut i = 0_usize;
    while i < n {
        let combined = combine_channels(r_force[i], r_xi[i], strategy).unwrap_or(0.0);
        debug_assert!(combined.is_finite(), "combined residual must be finite (non-finite inputs degrade to 0)");
        out[i] = combined;
        i += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_of_squares_zero_inputs_zero_out() {
        let r = combine_channels(0.0, 0.0, BalancingCombine::SumOfSquares).expect("finite");
        assert!(r.abs() < 1e-12);
    }

    #[test]
    fn sum_of_squares_3_4_5_triangle() {
        let r = combine_channels(3.0, 4.0, BalancingCombine::SumOfSquares).expect("finite");
        assert!((r - 5.0).abs() < 1e-12);
    }

    #[test]
    fn weighted_sum_zero_weight_suppresses_channel() {
        let r = combine_channels(
            10.0,
            0.1,
            BalancingCombine::WeightedSum { w_force: 0.0, w_xi: 1.0 },
        )
        .expect("finite");
        assert!((r - 0.1).abs() < 1e-6, "force channel zero-weighted → only xi shows through");
    }

    #[test]
    fn weighted_sum_rejects_negative_weights() {
        let r = combine_channels(
            1.0,
            1.0,
            BalancingCombine::WeightedSum { w_force: -1.0, w_xi: 1.0 },
        );
        assert!(r.is_none());
    }

    #[test]
    fn both_non_finite_is_none() {
        assert!(combine_channels(f64::NAN, f64::NAN, BalancingCombine::SumOfSquares).is_none());
    }

    #[test]
    fn one_non_finite_degrades_to_other() {
        let r = combine_channels(3.0, f64::NAN, BalancingCombine::SumOfSquares).expect("finite");
        assert!((r - 3.0).abs() < 1e-12);
    }

    #[test]
    fn stream_aligns_and_respects_capacity() {
        let rf = [3.0, 0.0, 1.0, 2.0];
        let rx = [4.0, 0.0, 0.0, 2.0];
        let mut out = [0.0_f64; 3];
        let n = combine_stream(&rf, &rx, &mut out, BalancingCombine::SumOfSquares);
        assert_eq!(n, 3);
        assert!((out[0] - 5.0).abs() < 1e-12);
        assert!(out[1].abs() < 1e-12);
        assert!((out[2] - 1.0).abs() < 1e-12);
    }
}
