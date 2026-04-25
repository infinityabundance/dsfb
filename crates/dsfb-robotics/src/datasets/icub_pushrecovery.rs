//! IIT iCub push-recovery balancing adapter.
//!
//! **Provenance.** IIT iCub Tech publications on whole-body balancing
//! and push-recovery control (Nori, Traversaro, Natale et al.;
//! Parmiggiani et al. iCub-3 releases). Full experiment logs are
//! typically released under an IIT data-use agreement; this adapter
//! consumes **manifest-only** slices. The Phase 1 `paper-lock` path
//! hard-errors if the full dataset is absent, pointing the user to
//! `docs/icub_pushrecovery_protocol.md` for the DUA fetch path.
//!
//! **Residual DSFB structures — dual channel.**
//!
//! - **`r_W(k) = W_contact,measured(k) − W_contact,planned(k)`** —
//!   contact-wrench residual from the whole-body controller (WBC).
//!   The WBC plans a per-contact 6-D wrench each cycle; the 6-axis
//!   F/T sensor-measured wrench differs from the plan. The WBC
//!   discards this residual post-balance-recovery.
//! - **`r_ξ(k) = ξ_measured(k) − ξ_planned(k)`** — centroidal-momentum
//!   tracking residual. Used by the balance controller as an error
//!   signal; discarded once the feedback loop stabilises.
//!
//! DSFB ingests both channels via
//! [`crate::balancing::combine_channels`]. The paper's §10.10 uses
//! iCub as the **humanoid** exemplar of the balancing family, paired
//! with the quadruped exemplar [`super::cheetah3`] to span
//! morphologies.

use crate::balancing::{self, BalancingCombine};

/// Per-timestep dual-channel sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Contact-wrench residual norm `‖W_meas − W_planned‖`.
    ///
    /// Supplied as a scalar magnitude by the caller — typically the
    /// 2-norm of the six-axis wrench error aggregated across contacts.
    pub wrench_residual: f64,
    /// Centroidal-momentum tracking residual `‖ξ_meas − ξ_planned‖`.
    ///
    /// Caller provides a scalar magnitude aggregated across linear +
    /// angular components with whatever weighting the paper's
    /// protocol fixes.
    pub xi_residual: f64,
}

impl Sample {
    /// Combine the two channels into a single residual norm.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self, strategy: BalancingCombine) -> Option<f64> {
        balancing::combine_channels(self.wrench_residual, self.xi_residual, strategy)
    }
}

/// Default combine strategy for iCub: equal-weighted Euclidean
/// (paper §5.X specification).
pub const DEFAULT_COMBINE: BalancingCombine = BalancingCombine::SumOfSquares;

/// Stream samples into a residual buffer with [`DEFAULT_COMBINE`].
pub fn residual_stream(samples: &[Sample], out: &mut [f64]) -> usize {
    debug_assert!(samples.len() <= 1_000_000, "sample slice unreasonably large");
    debug_assert!(matches!(DEFAULT_COMBINE, BalancingCombine::SumOfSquares), "default combine pinned by paper §5");
    residual_stream_with(samples, out, DEFAULT_COMBINE)
}

/// Stream samples with a caller-supplied combine strategy.
pub fn residual_stream_with(
    samples: &[Sample],
    out: &mut [f64],
    strategy: BalancingCombine,
) -> usize {
    debug_assert!(samples.len() <= 1_000_000, "sample slice unreasonably large");
    let n = samples.len().min(out.len());
    debug_assert!(n <= out.len() && n <= samples.len(), "n bounded by both buffers");
    let mut i = 0_usize;
    while i < n {
        out[i] = samples[i].residual_norm(strategy).unwrap_or(0.0);
        i += 1;
    }
    debug_assert_eq!(i, n, "loop must run exactly n iterations");
    n
}

/// Fill `out` with the residual stream from the in-crate [`FIXTURE`],
/// using [`DEFAULT_COMBINE`]. Used by `paper-lock --fixture`.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    residual_stream(&FIXTURE, out)
}

/// Smoke-test micro-fixture (6 samples: pre-push → push → recovery).
pub const FIXTURE: [Sample; 6] = [
    // Pre-push: standing, residuals quiet.
    Sample { wrench_residual: 0.8, xi_residual: 0.05 },
    Sample { wrench_residual: 0.9, xi_residual: 0.04 },
    // Push onset: wrench residual spikes as contact distribution shifts.
    Sample { wrench_residual: 4.0, xi_residual: 0.20 },
    // Peak disturbance: both channels elevated.
    Sample { wrench_residual: 6.5, xi_residual: 0.45 },
    // Recovery: residuals decaying back.
    Sample { wrench_residual: 2.0, xi_residual: 0.15 },
    Sample { wrench_residual: 1.0, xi_residual: 0.06 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_sample_is_zero_residual() {
        let s = Sample { wrench_residual: 0.0, xi_residual: 0.0 };
        assert!(s.residual_norm(DEFAULT_COMBINE).expect("finite").abs() < 1e-12);
    }

    #[test]
    fn push_peak_is_the_largest_residual() {
        let mut out = [0.0_f64; 6];
        let n = residual_stream(&FIXTURE, &mut out);
        assert_eq!(n, 6);
        let peak = out[3];
        for (i, r) in out[..n].iter().enumerate() {
            if i != 3 {
                assert!(peak > *r, "push peak (idx=3) must be largest, out={out:?}");
            }
        }
    }

    #[test]
    fn trajectory_is_bell_shaped() {
        // Fixture is monotonically increasing then decreasing around the peak.
        let mut out = [0.0_f64; 6];
        let _ = residual_stream(&FIXTURE, &mut out);
        assert!(out[0] < out[1], "rising phase");
        assert!(out[1] < out[2]);
        assert!(out[2] < out[3]);
        assert!(out[3] > out[4], "falling phase");
        assert!(out[4] > out[5]);
    }

    #[test]
    fn weighted_combine_routes_attention_to_wrench() {
        // Heavy weight on wrench → the push peak is dominated by r_W.
        let s = Sample { wrench_residual: 10.0, xi_residual: 0.1 };
        let r = s
            .residual_norm(BalancingCombine::WeightedSum { w_force: 10.0, w_xi: 0.1 })
            .expect("finite");
        // Expected ≈ sqrt(10·100 + 0.1·0.01) ≈ sqrt(1000.001) ≈ 31.623.
        assert!(r > 31.0 && r < 32.0, "weighted combine gave {r}");
    }
}
