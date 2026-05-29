//! `PlantTwinReplayV1` (Wave-4 historian layer) — a **replay-consistency** witness comparing a recorded run
//! against a operator-supplied "twin" reference (a model output, a prior good run, or a sister unit), so a
//! divergence from the expected trajectory is court-admissible evidence.
//!
//! Operators increasingly keep a digital twin or a golden reference run. DSFB does not build or run that
//! twin — it ingests the twin's predicted trajectory as just another input series and reports, deterministically
//! and sealed, how far the live run diverged from it (per-sample residual summary, where it first crossed a
//! tolerance, how long it stayed out). This makes "the plant is no longer tracking its twin" a first-class,
//! reproducible observation.
//!
//! Bounded (non-claims): this is a **replay-consistency witness, NOT a controller, a simulator, or a
//! calibrated state estimate**. DSFB neither produces the twin nor vouches for it; a divergence is candidate
//! evidence that live ≠ reference, never a statement about which one is right or why. Additive + off the
//! replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// A hash-sealed plant-twin replay-consistency witness (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlantTwinReplayV1 {
    pub variable: String,
    /// Name/provenance of the twin reference (e.g. `"first-principles twin r3"`, `"golden run 2025-11"`).
    pub twin_ref: String,
    /// `|live − twin|` above which a sample is judged diverged.
    pub tolerance: f64,
    pub n_samples: usize,
    /// Samples whose `|live − twin|` exceeded `tolerance`.
    pub n_diverged: usize,
    /// First sample index that diverged (the onset), if any.
    pub first_divergence_index: Option<usize>,
    /// Longest run of consecutive diverged samples (how long the plant stayed off its twin).
    pub max_consecutive_divergence: usize,
    /// Largest `|live − twin|` across the run.
    pub peak_divergence: f64,
    /// Root-mean-square divergence over the finite paired samples.
    pub rms_divergence: f64,
    /// SHA-256 of the divergence stream `live − twin`.
    pub divergence_hash: String,
    pub non_claim: String,
    pub witness_hash: String,
}

impl PlantTwinReplayV1 {
    const NON_CLAIM: &'static str =
        "replay-consistency witness; NOT a controller, simulator, or calibrated state estimate. DSFB does not produce or vouch for the twin; a divergence means live != reference, not which is right or why";

    fn hash_divergence(d: &[f64]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"plant_twin_divergence_stream_v1");
        for &v in d {
            h.f64q("d", v);
        }
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"plant_twin_replay_v1");
        h.field("variable", self.variable.as_bytes());
        h.field("twin_ref", self.twin_ref.as_bytes());
        h.f64q("tolerance", self.tolerance);
        h.u64("n_samples", self.n_samples as u64);
        h.u64("n_diverged", self.n_diverged as u64);
        h.u64("has_first", self.first_divergence_index.is_some() as u64);
        h.u64(
            "first_divergence_index",
            self.first_divergence_index.unwrap_or(0) as u64,
        );
        h.u64(
            "max_consecutive_divergence",
            self.max_consecutive_divergence as u64,
        );
        h.f64q("peak_divergence", self.peak_divergence);
        h.f64q("rms_divergence", self.rms_divergence);
        h.field("divergence_hash", self.divergence_hash.as_bytes());
        h.field("non_claim", self.non_claim.as_bytes());
        h.finalize_hex()
    }

    /// Compare a live series against the twin reference and seal the divergence summary. A sample with a
    /// non-finite live or twin value contributes a non-finite divergence (excluded from peak/RMS and from the
    /// diverged count, but sealed in the stream so the gap is visible).
    pub fn build(
        variable: impl Into<String>,
        twin_ref: impl Into<String>,
        live: &[f64],
        twin: &[f64],
        tolerance: f64,
    ) -> Self {
        let n = live.len().min(twin.len());
        let divergence: Vec<f64> = (0..n).map(|k| live[k] - twin[k]).collect();
        let (mut n_diverged, mut first_divergence_index) = (0usize, None);
        let (mut run, mut max_run) = (0usize, 0usize);
        let (mut peak, mut sumsq, mut n_finite) = (0.0f64, 0.0f64, 0usize);
        for (k, &d) in divergence.iter().enumerate() {
            if !d.is_finite() {
                run = 0;
                continue;
            }
            n_finite += 1;
            sumsq += d * d;
            peak = peak.max(d.abs());
            if d.abs() > tolerance {
                n_diverged += 1;
                first_divergence_index.get_or_insert(k);
                run += 1;
                max_run = max_run.max(run);
            } else {
                run = 0;
            }
        }
        let rms_divergence = if n_finite == 0 {
            0.0
        } else {
            (sumsq / n_finite as f64).sqrt()
        };
        let mut w = PlantTwinReplayV1 {
            variable: variable.into(),
            twin_ref: twin_ref.into(),
            tolerance,
            n_samples: n,
            n_diverged,
            first_divergence_index,
            max_consecutive_divergence: max_run,
            peak_divergence: peak,
            rms_divergence,
            divergence_hash: Self::hash_divergence(&divergence),
            non_claim: Self::NON_CLAIM.to_string(),
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    /// True iff the live run ever diverged from its twin beyond tolerance.
    pub fn diverged(&self) -> bool {
        self.n_diverged > 0
    }

    pub fn verify(&self, live: &[f64], twin: &[f64]) -> bool {
        let n = live.len().min(twin.len());
        let divergence: Vec<f64> = (0..n).map(|k| live[k] - twin[k]).collect();
        Self::hash_divergence(&divergence) == self.divergence_hash
            && self.non_claim == Self::NON_CLAIM
            && self.seal() == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_twin_shows_no_divergence() {
        let live = vec![1.0, 2.0, 3.0, 4.0];
        let twin = vec![1.01, 1.99, 3.0, 4.02];
        let w = PlantTwinReplayV1::build("reactor_T", "golden run 2025-11", &live, &twin, 0.1);
        assert!(!w.diverged());
        assert_eq!(w.n_diverged, 0);
        assert_eq!(w.first_divergence_index, None);
        assert!(w.witness_hash.len() == 64 && w.verify(&live, &twin));
        assert!(w.non_claim.contains("NOT a controller, simulator"));
    }

    #[test]
    fn sustained_divergence_is_measured() {
        // Live runs 5 above the twin from k=2 onward → diverges, onset 2, run length 3, peak 5.
        let live = vec![10.0, 10.0, 15.0, 15.0, 15.0];
        let twin = vec![10.0, 10.0, 10.0, 10.0, 10.0];
        let w = PlantTwinReplayV1::build("level", "twin r3", &live, &twin, 1.0);
        assert!(w.diverged());
        assert_eq!(w.n_diverged, 3);
        assert_eq!(w.first_divergence_index, Some(2));
        assert_eq!(w.max_consecutive_divergence, 3);
        assert!((w.peak_divergence - 5.0).abs() < 1e-12);
        assert!(w.verify(&live, &twin));
        assert!(!w.verify(&[0.0; 5], &twin)); // a different live stream fails verification
    }

    #[test]
    fn tampering_a_count_breaks_the_seal() {
        let live = vec![1.0, 5.0];
        let twin = vec![1.0, 1.0];
        let mut w = PlantTwinReplayV1::build("x", "t", &live, &twin, 1.0);
        assert!(w.verify(&live, &twin));
        w.n_diverged = 0; // forge away the divergence
        assert!(!w.verify(&live, &twin));
    }
}
