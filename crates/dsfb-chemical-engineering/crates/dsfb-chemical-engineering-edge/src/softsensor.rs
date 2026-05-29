//! `SoftSensorWitnessV1` — a first-class soft-sensor witness (P61).
//!
//! The corpus crate frames the soft-sensor thesis (cheap sensors → hard-to-measure target) but the
//! soft-sensor's *output* was never a witness object. This makes it one: a hash-sealed record of a soft
//! sensor's `measured` target (where lab/online truth exists), its `prediction`, the `residual`
//! (measured − prediction — the DSFB-admissible error stream), an uncertainty `interval_half_width`,
//! the `model_family` (PLS / OLS / deterministic-envelope / neural / …), and a `training_scope_hash`
//! sealing *what the model was fit on*. DSFB then reads the residual stream as evidence exactly like any
//! other detector residual — turning a soft sensor's error into court-admissible, replayable testimony,
//! while the model family + training scope are disclosed (no probabilistic claim is made by this crate).
//!
//! Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::{sha256_hex, CanonicalHasher};

/// A hash-sealed soft-sensor witness (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftSensorWitnessV1 {
    /// The hard-to-measure target channel this soft sensor estimates.
    pub channel: String,
    /// The model family (disclosed, not a quality claim): e.g. `"PLS"`, `"OLS"`, `"deterministic-envelope"`.
    pub model_family: String,
    /// SHA-256 of the training-scope descriptor — *what* the model was fit on (window, datasets, vars).
    pub training_scope_hash: String,
    /// Measured target where truth exists (may carry `NaN` where unavailable; not faked).
    pub measured: Vec<f64>,
    /// Soft-sensor prediction per sample.
    pub prediction: Vec<f64>,
    /// Residual stream `measured − prediction` (the DSFB-admissible error; `NaN` where unmeasured).
    pub residual: Vec<f64>,
    /// Per-sample uncertainty interval half-width (0 if the model reports none).
    pub interval_half_width: Vec<f64>,
    pub n_samples: usize,
    /// SHA-256 sealing the whole record.
    pub witness_hash: String,
}

impl SoftSensorWitnessV1 {
    fn seal(
        channel: &str,
        model_family: &str,
        training_scope_hash: &str,
        measured: &[f64],
        prediction: &[f64],
        residual: &[f64],
        interval_half_width: &[f64],
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"soft_sensor_witness_v1");
        h.field("channel", channel.as_bytes());
        h.field("model_family", model_family.as_bytes());
        h.field("training_scope_hash", training_scope_hash.as_bytes());
        let mut put = |label: &str, xs: &[f64]| {
            h.u64(label, xs.len() as u64);
            for &v in xs {
                h.f64q("v", v);
            }
        };
        put("measured", measured);
        put("prediction", prediction);
        put("residual", residual);
        put("interval", interval_half_width);
        h.finalize_hex()
    }

    /// Build + seal a soft-sensor witness. The residual is computed as `measured − prediction` per
    /// sample over the common length (so an unmeasured `NaN` propagates into the residual rather than
    /// being silently zeroed). `training_scope` is a free-form descriptor of what the model was fit on;
    /// it is hashed into `training_scope_hash`.
    pub fn build(
        channel: impl Into<String>,
        model_family: impl Into<String>,
        training_scope: &str,
        measured: &[f64],
        prediction: &[f64],
        interval_half_width: &[f64],
    ) -> Self {
        let channel = channel.into();
        let model_family = model_family.into();
        let training_scope_hash = sha256_hex(training_scope.as_bytes());
        let n = measured.len().min(prediction.len());
        let residual: Vec<f64> = (0..n).map(|i| measured[i] - prediction[i]).collect();
        let witness_hash = Self::seal(
            &channel,
            &model_family,
            &training_scope_hash,
            measured,
            prediction,
            &residual,
            interval_half_width,
        );
        SoftSensorWitnessV1 {
            channel,
            model_family,
            training_scope_hash,
            measured: measured.to_vec(),
            prediction: prediction.to_vec(),
            residual,
            interval_half_width: interval_half_width.to_vec(),
            n_samples: n,
            witness_hash,
        }
    }

    /// Re-derive the seal from the record's own fields and check it matches `witness_hash`.
    pub fn verify(&self) -> bool {
        Self::seal(
            &self.channel,
            &self.model_family,
            &self.training_scope_hash,
            &self.measured,
            &self.prediction,
            &self.residual,
            &self.interval_half_width,
        ) == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residual_is_measured_minus_prediction_and_self_verifies() {
        let measured = vec![10.0, 12.0, 11.0];
        let prediction = vec![9.5, 12.5, 11.2];
        let interval = vec![0.5, 0.5, 0.5];
        let w = SoftSensorWitnessV1::build(
            "C4_content",
            "deterministic-envelope",
            "debutanizer baseline rows 0..500; 7 cheap inputs",
            &measured,
            &prediction,
            &interval,
        );
        assert_eq!(w.n_samples, 3);
        assert!((w.residual[0] - 0.5).abs() < 1e-12);
        assert!((w.residual[1] - (-0.5)).abs() < 1e-12);
        assert_eq!(w.training_scope_hash.len(), 64);
        assert!(w.verify());
    }

    #[test]
    fn training_scope_and_tamper_change_the_seal() {
        let m = vec![1.0, 2.0];
        let p = vec![1.1, 1.9];
        let a = SoftSensorWitnessV1::build("y", "PLS", "scope A", &m, &p, &[0.0, 0.0]);
        let b = SoftSensorWitnessV1::build("y", "PLS", "scope B", &m, &p, &[0.0, 0.0]);
        assert_ne!(
            a.training_scope_hash, b.training_scope_hash,
            "training scope is sealed"
        );
        assert_ne!(a.witness_hash, b.witness_hash);
        let mut t = a.clone();
        t.prediction[0] = 99.0;
        assert!(!t.verify());
    }
}
