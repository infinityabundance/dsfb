//! Universal Robots UR10 kinematics adapter — Kufieta 2014 (NTNU).
//!
//! **Provenance.** Kufieta, *"Force estimation in Robotic Manipulators:
//! Modeling, Simulation and Experiments: The UR5 Manipulator as a Case
//! Study"*, NTNU MSc thesis, 2014, and subsequent UR-series
//! identification literature (e.g. Kebria et al. 2016). Six-DoF
//! industrial cobot with **motor-side current sensing**. The
//! identification regresses a standard rigid-body dynamic model;
//! `θ̂_ur10` is the published identified parameter vector.
//!
//! **Residual DSFB structures.**
//!
//! ```text
//! r_τ(k) = τ_motor,measured(k) − τ_motor,predicted(q(k), q̇(k), q̈(k); θ̂_ur10)
//! ```
//!
//! Rounds out the four-arm kinematics cohort alongside KUKA LWR
//! (link-side), Panda (motor-side), and DLR Justin (link-side). The
//! UR10 is the widely-deployed industrial counterpart, important for
//! demonstrating that the framework is not academia-only.

use crate::kinematics::{self, TorqueSensorSide};

/// UR10 has six joints (shoulder / elbow / wrist triple × 2).
pub const NUM_JOINTS: usize = 6;

/// Motor-side sensing (reconstructed from joint current).
pub const SENSOR_SIDE: TorqueSensorSide = TorqueSensorSide::Motor;

/// Per-timestep sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Per-joint motor-side reconstructed torque, N·m.
    pub tau_measured: [f64; NUM_JOINTS],
    /// Per-joint predicted torque from `Y · θ̂_ur10`, N·m.
    pub tau_predicted: [f64; NUM_JOINTS],
}

impl Sample {
    /// Residual norm for this sample.
    #[inline]
    #[must_use]
    pub fn residual_norm(&self) -> Option<f64> {
        kinematics::tau_residual_norm(&self.tau_measured, &self.tau_predicted)
    }
}

/// Stream samples into a residual buffer.
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

/// Smoke-test micro-fixture (4 samples).
pub const FIXTURE: [Sample; 4] = [
    Sample {
        tau_measured: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Sample {
        tau_measured: [0.6, 0.1, -0.05, 0.02, 0.00, 0.00],
        tau_predicted: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Sample {
        tau_measured: [0.5, 0.2, -0.10, 0.05, 0.01, 0.00],
        tau_predicted: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Sample {
        tau_measured: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_joints_is_six() {
        assert_eq!(NUM_JOINTS, 6);
    }

    #[test]
    fn sensor_side_is_motor() {
        assert_eq!(SENSOR_SIDE, TorqueSensorSide::Motor);
    }

    #[test]
    fn nominal_sample_has_zero_residual() {
        let s = FIXTURE[0];
        assert!(s.residual_norm().expect("finite").abs() < 1e-12);
    }

    #[test]
    fn fixture_residual_rises_then_falls() {
        let mut out = [0.0_f64; 4];
        let n = residual_stream(&FIXTURE, &mut out);
        assert_eq!(n, 4);
        assert!(out[0] < out[1], "residual rising through fixture");
        assert!(out[1] < out[2]);
        assert!(out[2] > out[3], "and falling back to nominal");
    }
}
