//! Shared residual helper for kinematic-identification datasets
//! (KUKA LWR, Franka Panda, DLR Rollin' Justin / LWR-III, UR10).
//!
//! All four datasets share the same residual form:
//!
//! ```text
//! r_τ(k) = τ_measured(k) − τ_predicted(q(k), q̇(k), q̈(k); θ̂)
//! ```
//!
//! where `θ̂` is the **published identified parameter vector** for the
//! specific arm. Each per-dataset adapter embeds its own `θ̂` as a
//! `const` so the residual is reproducible without an in-tree
//! identification step.
//!
//! This module provides the joint-aggregation primitive that turns a
//! per-joint torque residual sample into a scalar residual norm that
//! the core `observe()` pipeline ingests. Phase 2 provides the
//! function; Phase 3 wires it into each per-dataset adapter with the
//! appropriate `θ̂`.

use crate::math;

/// Side of the torque-sensing instrument.
///
/// KUKA LWR and DLR Rollin' Justin instrument **link-side** torque
/// directly. Franka Panda and UR10 measure **motor-side** current and
/// reconstruct torque post-transmission. The side changes the noise
/// floor and the residual's expected amplitude, but not its structural
/// grammar — DSFB treats both uniformly once the sample is in
/// `f64` newton-metres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TorqueSensorSide {
    /// Direct link-side torque sensing (e.g. KUKA LWR, DLR LWR-III).
    Link,
    /// Motor-side current × torque constant (e.g. Franka Panda, UR10).
    Motor,
}

impl TorqueSensorSide {
    /// Stable label for logging / JSON emission.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Link => "LinkSide",
            Self::Motor => "MotorSide",
        }
    }
}

/// Aggregate a per-joint torque-residual sample into a scalar residual
/// norm.
///
/// Accepts a slice of per-joint `r_τ(k)` values (N-DoF arm → N-length
/// slice) and returns the 2-norm `‖r_τ(k)‖ = sqrt(Σ r_τ_i²)`.
///
/// Returns `None` if the slice is empty or contains only non-finite
/// samples. Non-finite per-joint samples are **skipped**
/// (missingness-aware); a sample with *some* finite joints is still
/// aggregated over the finite ones.
#[must_use]
pub fn joint_residual_norm(per_joint_tau_residual: &[f64]) -> Option<f64> {
    debug_assert!(per_joint_tau_residual.len() <= 32, "per-joint slice fits a typical 7-DoF arm + safety margin");
    let mut ssq = 0.0_f64;
    let mut n = 0_usize;
    for &r in per_joint_tau_residual {
        if r.is_finite() {
            debug_assert!(r * r >= 0.0, "squared finite f64 is non-negative");
            ssq += r * r;
            n += 1;
        }
    }
    debug_assert!(n <= per_joint_tau_residual.len(), "finite-count cannot exceed slice length");
    if n == 0 {
        return None;
    }
    debug_assert!(ssq.is_finite() && ssq >= 0.0, "ssq must be a finite non-negative aggregate");
    math::sqrt_f64(ssq)
}

/// Compute the residual for one timestep, given measured and predicted
/// per-joint torques.
///
/// Returns `Some(‖τ_meas − τ_pred‖)` when the two slices have matching
/// length and at least one joint is finite on both sides; `None`
/// otherwise. This is the small helper that per-dataset adapters
/// compose into a streaming residual sequence.
#[must_use]
pub fn tau_residual_norm(tau_measured: &[f64], tau_predicted: &[f64]) -> Option<f64> {
    debug_assert!(tau_measured.len() <= 32, "torque slice within typical arm DoF + margin");
    debug_assert!(tau_predicted.len() <= 32, "torque slice within typical arm DoF + margin");
    if tau_measured.len() != tau_predicted.len() {
        return None;
    }
    if tau_measured.is_empty() {
        return None;
    }
    debug_assert_eq!(tau_measured.len(), tau_predicted.len(), "lengths matched after early returns");
    let mut ssq = 0.0_f64;
    let mut n = 0_usize;
    let mut i = 0_usize;
    while i < tau_measured.len() {
        let m = tau_measured[i];
        let p = tau_predicted[i];
        if m.is_finite() && p.is_finite() {
            let d = m - p;
            debug_assert!(d.is_finite(), "diff of finite operands is finite");
            ssq += d * d;
            n += 1;
        }
        i += 1;
    }
    debug_assert!(n <= tau_measured.len(), "finite-count bounded by slice length");
    if n == 0 {
        return None;
    }
    debug_assert!(ssq >= 0.0, "ssq is a sum of non-negative squares");
    math::sqrt_f64(ssq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice_is_none() {
        assert!(joint_residual_norm(&[]).is_none());
    }

    #[test]
    fn all_nan_slice_is_none() {
        assert!(joint_residual_norm(&[f64::NAN, f64::NAN]).is_none());
    }

    #[test]
    fn single_joint_norm_is_absolute_value() {
        let r = joint_residual_norm(&[0.3]).expect("finite");
        assert!((r - 0.3).abs() < 1e-12);
        let r2 = joint_residual_norm(&[-0.3]).expect("finite");
        assert!((r2 - 0.3).abs() < 1e-12);
    }

    #[test]
    fn three_joint_norm_is_euclidean() {
        // 3-4-5 triangle.
        let r = joint_residual_norm(&[3.0, 4.0, 0.0]).expect("finite");
        assert!((r - 5.0).abs() < 1e-12);
    }

    #[test]
    fn tau_residual_respects_lengths() {
        assert!(tau_residual_norm(&[1.0, 2.0], &[1.0]).is_none());
        assert!(tau_residual_norm(&[], &[]).is_none());
    }

    #[test]
    fn tau_residual_is_difference_norm() {
        let meas = [1.0, 2.0, 3.0];
        let pred = [1.0, 2.0, 3.0];
        let r = tau_residual_norm(&meas, &pred).expect("finite");
        assert!(r.abs() < 1e-12);
        let pred_off = [1.3, 2.4, 3.0];
        let r2 = tau_residual_norm(&meas, &pred_off).expect("finite");
        // diff = (0.3, 0.4, 0) → 0.5
        assert!((r2 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn tau_residual_skips_non_finite_joints() {
        let meas = [1.0, f64::NAN, 3.0];
        let pred = [1.0, 2.0, 3.0];
        let r = tau_residual_norm(&meas, &pred).expect("at least one finite joint");
        assert!(r.abs() < 1e-12);
    }

    #[test]
    fn torque_side_labels_are_stable() {
        assert_eq!(TorqueSensorSide::Link.label(), "LinkSide");
        assert_eq!(TorqueSensorSide::Motor.label(), "MotorSide");
    }
}
