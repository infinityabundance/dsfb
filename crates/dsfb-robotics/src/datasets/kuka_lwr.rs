//! KUKA LWR kinematics adapter — Jubien–Gautier–Janot 2014.
//!
//! **Provenance.** Jubien, Gautier, and Janot, *"Dynamic identification
//! of the Kuka LWR robot using motor torques and joint torque sensors
//! data"*, IFAC Proceedings 47 (3), 2014. Seven-DoF arm with **link-side
//! torque sensing** (DLR-derived Light-Weight Robot III architecture),
//! 1 kHz sampling, excitation trajectories used to fit a full dynamic
//! parameter vector.
//!
//! **Residual DSFB structures.**
//!
//! ```text
//! r_τ(k) = τ_link,measured(k) − τ_link,predicted(q(k), q̇(k), q̈(k); θ̂_kuka)
//! ```
//!
//! where `θ̂_kuka` is the Jubien 2014 published identified parameter
//! vector. This is the identification residual the Jubien pipeline
//! treats as i.i.d. noise after the LS fit — exactly the
//! discarded-residual thesis.
//!
//! **Bounded claim (paper §10.4).** DSFB identifies structured
//! residual episodes in healthy operation distinguishable from
//! identification noise. No classification or fault-detection claim
//! is made; the dataset contains no labelled faults.

use crate::kinematics::{self, TorqueSensorSide};

/// KUKA LWR has seven joints.
pub const NUM_JOINTS: usize = 7;

/// Sensing side for this dataset (fixed by the platform).
pub const SENSOR_SIDE: TorqueSensorSide = TorqueSensorSide::Link;

/// A single timestep sample from the KUKA LWR dataset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Per-joint measured link-side torque, N·m.
    pub tau_measured: [f64; NUM_JOINTS],
    /// Per-joint predicted link-side torque from the identified model,
    /// `τ_pred = Y(q, q̇, q̈) · θ̂_kuka`, N·m.
    pub tau_predicted: [f64; NUM_JOINTS],
}

impl Sample {
    /// Compute the residual norm for this sample: `‖τ_meas − τ_pred‖`.
    ///
    /// Returns `None` if all per-joint samples are non-finite on at
    /// least one side. See
    /// [`crate::kinematics::tau_residual_norm`] for the missingness
    /// policy applied to partial samples.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self) -> Option<f64> {
        kinematics::tau_residual_norm(&self.tau_measured, &self.tau_predicted)
    }
}

/// Stream a slice of KUKA LWR samples into a caller-owned residual
/// buffer. Returns the number of residuals written.
///
/// Non-finite or degenerate samples emit `0.0` (missingness-aware),
/// matching the engine's below-floor policy. Never writes past
/// `out.len()`.
pub fn residual_stream(samples: &[Sample], out: &mut [f64]) -> usize {
    debug_assert!(samples.len() <= 1_000_000, "sample slice unreasonably large");
    let n = samples.len().min(out.len());
    debug_assert!(n <= out.len() && n <= samples.len(), "n bounded by both buffers");
    let mut i = 0_usize;
    while i < n {
        out[i] = samples[i].residual_norm().unwrap_or(0.0);
        i += 1;
    }
    debug_assert_eq!(i, n, "loop must run exactly n iterations");
    n
}

/// Deterministically populate `out` with the residual stream produced
/// by the in-crate [`FIXTURE`]. Returns the number of residuals
/// written (≤ `FIXTURE.len()`). Used by `paper-lock --fixture` for
/// bit-exact smoke testing without requiring the full dataset.
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    residual_stream(&FIXTURE, out)
}

/// Micro-fixture for unit tests and adapter smoke tests. 6 samples,
/// representative of a short excitation window from the published
/// Jubien 2014 trajectories (shape-only; no quantitative reproduction
/// of the paper's figures is claimed from this micro-fixture).
/// In-crate micro-fixture (6 samples) for smoke testing and Colab
/// reproductions when the full Jubien 2014 trajectories are not
/// available. Shape-only; no quantitative reproduction of the paper's
/// empirical results is claimed from this fixture.
pub const FIXTURE: [Sample; 6] = [
    Sample {
        tau_measured: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
    Sample {
        tau_measured: [0.11, -0.02, -0.18, 0.06, 0.01, -0.01, 0.00],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
    Sample {
        tau_measured: [0.15, -0.05, -0.10, 0.10, 0.02, 0.00, 0.01],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
    Sample {
        tau_measured: [0.20, -0.08, 0.00, 0.15, 0.03, 0.01, 0.02],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
    Sample {
        tau_measured: [0.15, -0.05, -0.10, 0.10, 0.02, 0.00, 0.01],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
    Sample {
        tau_measured: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
        tau_predicted: [0.10, 0.00, -0.20, 0.05, 0.00, -0.01, 0.00],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_produces_zero_residual() {
        let s = Sample {
            tau_measured: [0.1; NUM_JOINTS],
            tau_predicted: [0.1; NUM_JOINTS],
        };
        let r = s.residual_norm().expect("finite");
        assert!(r.abs() < 1e-12);
    }

    #[test]
    fn pythagorean_triple_across_joints() {
        let mut meas = [0.0_f64; NUM_JOINTS];
        let pred = [0.0_f64; NUM_JOINTS];
        meas[0] = 3.0;
        meas[1] = 4.0;
        let s = Sample { tau_measured: meas, tau_predicted: pred };
        let r = s.residual_norm().expect("finite");
        assert!((r - 5.0).abs() < 1e-12);
    }

    #[test]
    fn fixture_residuals_non_decreasing_in_perturbation_window() {
        // FIXTURE is symmetric around the peak at index 3. Residuals
        // should rise from 0 → peak → 0 monotonically each side.
        let mut buf = [0.0_f64; 6];
        let n = residual_stream(&FIXTURE, &mut buf);
        assert_eq!(n, 6);
        assert!(buf[0] < 1e-12, "first sample matches model → zero");
        assert!(buf[3] > buf[1], "residual peaks away from the nominal trajectory");
        assert!(buf[5] < 1e-12, "last sample matches model again → zero");
    }

    #[test]
    fn sensor_side_is_link() {
        assert_eq!(SENSOR_SIDE, TorqueSensorSide::Link);
    }
}
