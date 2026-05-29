//! `ChemometricPassportV1` — a per-detector, hash-sealed provenance passport (P58).
//!
//! The CUDA court already seals a **case-level** passport (dataset, input hash, contract, atlas hash).
//! This is the finer-grained companion: **one passport per detector**, recording *how that detector
//! produced its residual stream* so the stream is auditable end-to-end. It pins three SHA-256 digests —
//! `baseline_window_hash` (the exact baseline rows the detector fit its control limit on),
//! `input_matrix_hash` (the exact input matrix it scored), and `output_hash` (the exact `DetectorOutput`
//! stream it emitted) — and records, as disclosed policy strings, the `threshold_policy`,
//! `normalization`, and `missingness` handling. A `passport_hash` seals all of it, so "detector X scored
//! matrix H_in, fit on baseline H_base, under threshold policy P, emitting stream H_out" is a single
//! verifiable claim.
//!
//! Additive + off the replay path: it records provenance for a computation the pipeline already does.

use serde::{Deserialize, Serialize};

use crate::detectors::DetectorOutput;
use crate::hashing::{sha256_hex, CanonicalHasher};

/// SHA-256 (hex) over a row-major `f64` matrix, hashing each value's raw IEEE-754 bits little-endian
/// — the same endian-stable convention as the dispatch input digest, so a value that round-trips
/// through `to_bits()` reproduces the hash byte-for-byte regardless of platform float formatting.
fn hash_rows(rows: &[Vec<f64>]) -> String {
    let mut buf: Vec<u8> = Vec::with_capacity(rows.iter().map(|r| r.len() * 8).sum());
    for row in rows {
        for &v in row {
            buf.extend_from_slice(&v.to_bits().to_le_bytes());
        }
    }
    sha256_hex(&buf)
}

/// SHA-256 (hex) over a detector's output stream — every field of every `DetectorOutput`, in order,
/// through the canonical length-prefixed hasher so the digest is unambiguous.
fn hash_outputs(outputs: &[DetectorOutput]) -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"detector_output_stream_v1");
    for o in outputs {
        h.field("detector_id", o.detector_id.as_bytes());
        h.u64("time_index", o.time_index as u64);
        h.field("variable_scope", o.variable_scope.as_bytes());
        h.f64q("raw_score", o.raw_score);
        h.f64q("normalized_score", o.normalized_score);
        h.f64q("threshold", o.threshold);
        h.f64q("signed_margin", o.signed_margin);
        h.u64("breach", o.breach as u64);
    }
    h.finalize_hex()
}

/// A per-detector provenance passport (schema v1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChemometricPassportV1 {
    /// Detector identifier (matches `Detector::id` / the atlas `detector_id`).
    pub detector_id: String,
    /// Detector family (e.g. `"ClassicalMSPC"`, `"ProcessStructure"`).
    pub family: String,
    /// SHA-256 of the baseline window rows the detector fit its control limit on.
    pub baseline_window_hash: String,
    /// How the threshold/control-limit was derived (disclosed policy, e.g.
    /// `"baseline 99th-percentile"`, `"k=3 sigma over baseline"`, `"fixed"`).
    pub threshold_policy: String,
    /// Normalization applied before scoring (e.g. `"z-score(baseline mean/std)"`, `"none"`).
    pub normalization: String,
    /// How missing / non-finite samples were handled (e.g. `"non-finite passed through as NaN, counted"`).
    pub missingness: String,
    /// SHA-256 of the input matrix the detector scored.
    pub input_matrix_hash: String,
    /// SHA-256 of the emitted `DetectorOutput` stream.
    pub output_hash: String,
    /// SHA-256 (via [`CanonicalHasher`]) sealing every field above — the passport seal.
    pub passport_hash: String,
}

impl ChemometricPassportV1 {
    #[allow(clippy::too_many_arguments)]
    fn compute_seal(
        detector_id: &str,
        family: &str,
        baseline_window_hash: &str,
        threshold_policy: &str,
        normalization: &str,
        missingness: &str,
        input_matrix_hash: &str,
        output_hash: &str,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"chemometric_passport_v1");
        h.field("detector_id", detector_id.as_bytes());
        h.field("family", family.as_bytes());
        h.field("baseline_window_hash", baseline_window_hash.as_bytes());
        h.field("threshold_policy", threshold_policy.as_bytes());
        h.field("normalization", normalization.as_bytes());
        h.field("missingness", missingness.as_bytes());
        h.field("input_matrix_hash", input_matrix_hash.as_bytes());
        h.field("output_hash", output_hash.as_bytes());
        h.finalize_hex()
    }

    /// Build + seal a passport from the detector's actual inputs and outputs. The component hashes are
    /// computed here from the real data; the policy strings are the disclosed calibration choices.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        detector_id: impl Into<String>,
        family: impl Into<String>,
        baseline_window: &[Vec<f64>],
        input_matrix: &[Vec<f64>],
        outputs: &[DetectorOutput],
        threshold_policy: impl Into<String>,
        normalization: impl Into<String>,
        missingness: impl Into<String>,
    ) -> Self {
        let detector_id = detector_id.into();
        let family = family.into();
        let threshold_policy = threshold_policy.into();
        let normalization = normalization.into();
        let missingness = missingness.into();
        let baseline_window_hash = hash_rows(baseline_window);
        let input_matrix_hash = hash_rows(input_matrix);
        let output_hash = hash_outputs(outputs);
        let passport_hash = Self::compute_seal(
            &detector_id,
            &family,
            &baseline_window_hash,
            &threshold_policy,
            &normalization,
            &missingness,
            &input_matrix_hash,
            &output_hash,
        );
        ChemometricPassportV1 {
            detector_id,
            family,
            baseline_window_hash,
            threshold_policy,
            normalization,
            missingness,
            input_matrix_hash,
            output_hash,
            passport_hash,
        }
    }

    /// Re-derive the seal from the record's own fields and check it matches `passport_hash`.
    pub fn verify(&self) -> bool {
        Self::compute_seal(
            &self.detector_id,
            &self.family,
            &self.baseline_window_hash,
            &self.threshold_policy,
            &self.normalization,
            &self.missingness,
            &self.input_matrix_hash,
            &self.output_hash,
        ) == self.passport_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::DetectorFamily;

    fn sample_output(i: usize, m: f64) -> DetectorOutput {
        DetectorOutput {
            detector_id: "pca_t2".into(),
            family: DetectorFamily::ClassicalMspc,
            time_index: i,
            variable_scope: "scores".into(),
            raw_score: m + 1.0,
            normalized_score: m + 1.0,
            threshold: 1.0,
            signed_margin: m,
            breach: m > 0.0,
        }
    }

    #[test]
    fn passport_is_deterministic_and_self_verifies() {
        let baseline = vec![vec![0.0, 1.0], vec![0.1, 0.9], vec![-0.1, 1.1]];
        let input = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
        let outs: Vec<DetectorOutput> = (0..2).map(|i| sample_output(i, i as f64)).collect();
        let a = ChemometricPassportV1::build(
            "pca_t2",
            "ClassicalMspc",
            &baseline,
            &input,
            &outs,
            "baseline 99th-percentile",
            "z-score(baseline mean/std)",
            "non-finite->NaN, counted",
        );
        let b = ChemometricPassportV1::build(
            "pca_t2",
            "ClassicalMspc",
            &baseline,
            &input,
            &outs,
            "baseline 99th-percentile",
            "z-score(baseline mean/std)",
            "non-finite->NaN, counted",
        );
        assert_eq!(a, b, "passport must be deterministic");
        assert_eq!(a.passport_hash.len(), 64);
        assert!(a.verify());
    }

    #[test]
    fn changing_any_input_changes_the_seal() {
        let baseline = vec![vec![0.0, 1.0]];
        let input = vec![vec![0.0, 1.0]];
        let outs = vec![sample_output(0, 0.5)];
        let base = ChemometricPassportV1::build("d", "F", &baseline, &input, &outs, "p", "n", "m");
        // A different baseline window → different baseline_window_hash → different seal.
        let diff_baseline =
            ChemometricPassportV1::build("d", "F", &[vec![9.0, 9.0]], &input, &outs, "p", "n", "m");
        assert_ne!(base.passport_hash, diff_baseline.passport_hash);
        // A different threshold policy → different seal (policy is sealed too).
        let diff_policy =
            ChemometricPassportV1::build("d", "F", &baseline, &input, &outs, "k=3 sigma", "n", "m");
        assert_ne!(base.passport_hash, diff_policy.passport_hash);
    }

    #[test]
    fn tampering_breaks_verification() {
        let outs = vec![sample_output(0, 0.5)];
        let mut p = ChemometricPassportV1::build(
            "d",
            "F",
            &[vec![1.0]],
            &[vec![1.0]],
            &outs,
            "p",
            "n",
            "m",
        );
        assert!(p.verify());
        p.threshold_policy = "tampered".into();
        assert!(!p.verify());
    }
}
