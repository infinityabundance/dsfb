//! DLR Rollin' Justin / LWR-III kinematics adapter.
//!
//! **Provenance.** DLR Institute of Robotics and Mechatronics
//! publications around the Light-Weight Robot III platform and the
//! mobile two-arm humanoid Rollin' Justin (Albu-Schäffer et al.).
//! Direct **link-side torque sensing** on every joint is the signature
//! DLR instrumentation choice. This adapter consumes **manifest-only**
//! data: full trajectories require a DLR research-access data-use
//! agreement and are never redistributed by this crate.
//!
//! **Residual DSFB structures.**
//!
//! ```text
//! r_τ(k) = τ_link,measured(k) − τ_link,predicted(q(k), q̇(k), q̈(k); θ̂_dlr)
//! ```
//!
//! **Paired contrast with [`super::panda_gaz`]**: both are 7-DoF
//! research arms with published identified-parameter vectors. DLR
//! measures link-side, Panda reconstructs from motor-side current.
//! The paper's §10.7 uses this pair to demonstrate that DSFB's
//! grammar is agnostic to the sensing modality.

use crate::kinematics::{self, TorqueSensorSide};

/// DLR LWR-III / Rollin' Justin arm DoF.
pub const NUM_JOINTS: usize = 7;

/// Direct link-side torque sensing.
pub const SENSOR_SIDE: TorqueSensorSide = TorqueSensorSide::Link;

/// Per-timestep sample (link-side torques).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Per-joint measured link-side torque, N·m.
    pub tau_measured: [f64; NUM_JOINTS],
    /// Per-joint predicted torque from the DLR identified model.
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
        tau_measured: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        tau_predicted: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    },
    Sample {
        tau_measured: [0.20, 0.10, -0.15, 0.08, 0.00, 0.00, 0.00],
        tau_predicted: [0.20, 0.10, -0.15, 0.08, 0.00, 0.00, 0.00],
    },
    Sample {
        tau_measured: [0.30, 0.15, -0.10, 0.15, 0.02, 0.01, 0.00],
        tau_predicted: [0.20, 0.10, -0.15, 0.08, 0.00, 0.00, 0.00],
    },
    Sample {
        tau_measured: [0.20, 0.10, -0.15, 0.08, 0.00, 0.00, 0.00],
        tau_predicted: [0.20, 0.10, -0.15, 0.08, 0.00, 0.00, 0.00],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_side_is_link() {
        assert_eq!(SENSOR_SIDE, TorqueSensorSide::Link);
    }

    #[test]
    fn perfectly_identified_sample_has_zero_residual() {
        let s = Sample {
            tau_measured: [1.0; NUM_JOINTS],
            tau_predicted: [1.0; NUM_JOINTS],
        };
        assert!(s.residual_norm().expect("finite").abs() < 1e-12);
    }

    #[test]
    fn fixture_peak_is_at_sample_2() {
        let mut out = [0.0_f64; 4];
        let n = residual_stream(&FIXTURE, &mut out);
        assert_eq!(n, 4);
        // Indexes 0, 1, 3 are perfect fits → zero residual.
        assert!(out[0].abs() < 1e-12);
        assert!(out[1].abs() < 1e-12);
        assert!(out[3].abs() < 1e-12);
        // Index 2 is the perturbation peak.
        assert!(out[2] > 0.05);
    }
}
