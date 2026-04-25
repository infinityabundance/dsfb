//! MIT Cheetah 3 / Mini-Cheetah balancing adapter.
//!
//! **Provenance.** Katz, Di Carlo, Kim, *"Mini Cheetah: A Platform for
//! Pushing the Limits of Dynamic Quadruped Control"*, IEEE ICRA 2019,
//! DOI [10.1109/ICRA.2019.8794449](https://doi.org/10.1109/ICRA.2019.8794449);
//! Bledt, Powell, Katz, Di Carlo, Wensing, Kim, *"MIT Cheetah 3:
//! Design and Control of a Robust, Dynamic Quadruped Robot"*, IROS
//! 2018. Open-source platform with logs released under the MIT
//! license at
//! [github.com/mit-biomimetics/Cheetah-Software](https://github.com/mit-biomimetics/Cheetah-Software).
//!
//! **Residual DSFB structures — dual channel.**
//!
//! - **`r_F(k) = F_GRF,measured(k) − F_MPC,planned(k)`** — contact-
//!   force residual. The whole-body MPC plans a per-foot
//!   ground-reaction force each cycle; the measured force differs
//!   from the plan by a residual that the MPC rolls into the next
//!   horizon and discards.
//! - **`r_ξ(k) = c_CoM,IMU(k) − c_CoM,model(k)`** — centroidal-
//!   momentum observer residual. The IMU-fused CoM estimate differs
//!   from the rigid-body model prediction by a residual that the
//!   state estimator fuses and discards.
//!
//! DSFB combines the two channels through
//! [`crate::balancing::combine_channels`] into a single scalar
//! residual norm and then runs the standard grammar FSM. The paper's
//! §10.9 uses this dataset as the **quadruped** exemplar of the
//! balancing family (the humanoid exemplar is
//! [`super::icub_pushrecovery`]).

use crate::balancing::{self, BalancingCombine};

/// Per-timestep dual-channel sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Contact-force residual `r_F = F_GRF,meas − F_MPC,plan` (N).
    pub force_residual: f64,
    /// Centroidal-momentum residual `r_ξ = c_CoM,IMU − c_CoM,model`
    /// (unit: kg·m/s or m, depending on which momentum component the
    /// caller is streaming; the combiner only cares about magnitude).
    pub xi_residual: f64,
}

impl Sample {
    /// Combine the two channels with the supplied strategy.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self, strategy: BalancingCombine) -> Option<f64> {
        balancing::combine_channels(self.force_residual, self.xi_residual, strategy)
    }
}

/// The recommended default: equal-weighted Euclidean combination.
/// Matches the paper's §5.X specification.
pub const DEFAULT_COMBINE: BalancingCombine = BalancingCombine::SumOfSquares;

/// Stream samples into a residual buffer using [`DEFAULT_COMBINE`].
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

/// Smoke-test micro-fixture (6 samples: stance → swing → touchdown → stance).
pub const FIXTURE: [Sample; 6] = [
    // Stance: small residuals on both channels.
    Sample { force_residual: 0.5, xi_residual: 0.01 },
    Sample { force_residual: 0.8, xi_residual: 0.02 },
    // Swing: force channel quiet, centroidal channel rises.
    Sample { force_residual: 0.1, xi_residual: 0.05 },
    Sample { force_residual: 0.1, xi_residual: 0.08 },
    // Touchdown: both channels spike.
    Sample { force_residual: 3.0, xi_residual: 0.10 },
    // Return to stance quiet.
    Sample { force_residual: 0.6, xi_residual: 0.02 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_sample_is_zero_residual() {
        let s = Sample { force_residual: 0.0, xi_residual: 0.0 };
        assert!(s.residual_norm(DEFAULT_COMBINE).expect("finite").abs() < 1e-12);
    }

    #[test]
    fn euclidean_combination_is_norm_of_both() {
        let s = Sample { force_residual: 3.0, xi_residual: 4.0 };
        let r = s.residual_norm(BalancingCombine::SumOfSquares).expect("finite");
        assert!((r - 5.0).abs() < 1e-12);
    }

    #[test]
    fn weighted_combine_suppresses_channel_on_request() {
        // Zero weight on xi → residual is purely the force channel magnitude.
        let s = Sample { force_residual: 2.0, xi_residual: 100.0 };
        let r = s
            .residual_norm(BalancingCombine::WeightedSum { w_force: 1.0, w_xi: 0.0 })
            .expect("finite");
        assert!((r - 2.0).abs() < 1e-12);
    }

    #[test]
    fn fixture_touchdown_has_highest_residual() {
        let mut out = [0.0_f64; 6];
        let n = residual_stream(&FIXTURE, &mut out);
        assert_eq!(n, 6);
        let touchdown = out[4];
        for (i, r) in out[..n].iter().enumerate() {
            if i != 4 {
                assert!(touchdown > *r, "touchdown (idx=4) must be the largest residual, got out={out:?}");
            }
        }
    }
}
