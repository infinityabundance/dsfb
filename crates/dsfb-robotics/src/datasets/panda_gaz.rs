//! Franka Emika Panda kinematics adapter — Gaz, Cognetti, Oliva,
//! Robuffo Giordano, De Luca, 2019.
//!
//! **Provenance.** Gaz, Cognetti, Oliva, Robuffo Giordano, De Luca,
//! *"Dynamic identification of the Franka Emika Panda robot with
//! retrieval of feasible parameters using penalty-based optimization"*,
//! IEEE RA-L 4 (4), 2019, DOI
//! [10.1109/LRA.2019.2931248](https://doi.org/10.1109/LRA.2019.2931248).
//! Seven-DoF research manipulator with **motor-side current sensing**
//! (torque reconstructed via motor constant post-transmission), 1 kHz
//! sampling. The paper provides a feasible dynamic parameter vector
//! `θ̂_panda` that the adapter treats as the published oracle.
//!
//! **Residual DSFB structures.**
//!
//! ```text
//! r_τ(k) = τ_motor,measured(k) − τ_motor,predicted(q(k), q̇(k), q̈(k); θ̂_panda)
//! ```
//!
//! Complement to [`super::kuka_lwr`]: same residual class, **different
//! sensing side** (motor vs. link), different vendor, different
//! identification methodology. The paper's §10.6 uses the Panda/KUKA
//! pair to show that the grammar behaviour is consistent across
//! sensing modalities.

use crate::kinematics::{self, TorqueSensorSide};

/// Franka Panda has seven joints.
pub const NUM_JOINTS: usize = 7;

/// Sensing side (motor — torque reconstructed from joint current).
pub const SENSOR_SIDE: TorqueSensorSide = TorqueSensorSide::Motor;

/// Per-timestep sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Per-joint motor-side torque reconstructed from measured current
    /// via the published torque constant, N·m.
    pub tau_measured: [f64; NUM_JOINTS],
    /// Per-joint predicted torque from `Y(q, q̇, q̈) · θ̂_panda`, N·m.
    pub tau_predicted: [f64; NUM_JOINTS],
}

impl Sample {
    /// `‖τ_meas − τ_pred‖` for this sample.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self) -> Option<f64> {
        kinematics::tau_residual_norm(&self.tau_measured, &self.tau_predicted)
    }
}

/// Stream samples into a residual buffer. See [`super::kuka_lwr::residual_stream`].
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

/// Fill `out` with the residual stream from the in-crate [`FIXTURE`].
pub fn fixture_residuals(out: &mut [f64]) -> usize {
    debug_assert!(!out.is_empty(), "fixture buffer must be non-empty");
    residual_stream(&FIXTURE, out)
}

/// Smoke-test micro-fixture (5 samples).
pub const FIXTURE: [Sample; 5] = [
    Sample {
        tau_measured: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Sample {
        tau_measured: [0.05, 0.02, -0.04, 0.03, 0.01, 0.00, 0.00],
        tau_predicted: [0.05, 0.02, -0.04, 0.03, 0.01, 0.00, 0.00],
    },
    Sample {
        tau_measured: [0.08, 0.05, -0.02, 0.06, 0.02, 0.01, 0.00],
        tau_predicted: [0.05, 0.02, -0.04, 0.03, 0.01, 0.00, 0.00],
    },
    Sample {
        tau_measured: [0.05, 0.02, -0.04, 0.03, 0.01, 0.00, 0.00],
        tau_predicted: [0.05, 0.02, -0.04, 0.03, 0.01, 0.00, 0.00],
    },
    Sample {
        tau_measured: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_match_yields_zero() {
        let s = Sample {
            tau_measured: [0.05; NUM_JOINTS],
            tau_predicted: [0.05; NUM_JOINTS],
        };
        assert!(s.residual_norm().expect("finite").abs() < 1e-12);
    }

    #[test]
    fn one_joint_off_scales_as_difference() {
        let mut meas = [0.0_f64; NUM_JOINTS];
        meas[2] = 0.5;
        let s = Sample { tau_measured: meas, tau_predicted: [0.0; NUM_JOINTS] };
        assert!((s.residual_norm().expect("finite") - 0.5).abs() < 1e-12);
    }

    #[test]
    fn residual_stream_respects_output_capacity() {
        let mut out = [0.0_f64; 3];
        let n = residual_stream(&FIXTURE, &mut out);
        assert_eq!(n, 3, "should stop at output capacity");
    }

    #[test]
    fn sensor_side_is_motor() {
        assert_eq!(SENSOR_SIDE, TorqueSensorSide::Motor);
    }
}
