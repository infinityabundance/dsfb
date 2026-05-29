//! Data-quality episodes (Wave-6 confidential-evaluation chain): `DataQualityEpisodeV1`,
//! `FrozenTagDetectorV1`, and `ClockSkewWitnessV1`.
//!
//! Before DSFB interprets a process it must establish that the *data itself* is trustworthy — a frozen
//! transmitter, a missingness burst, or non-monotonic timestamps manufacture "anomalies" that are really data
//! defects. These objects emit **data-quality episodes that are explicitly distinct from process episodes**,
//! so a case file can separate "the data is bad here" from "the process did something here".
//!
//!   * [`DataQualityEpisodeV1`] — one typed data-quality event (`FrozenTag` / `MissingnessBurst` /
//!     `TimestampDiscontinuity` / `UnitMismatch` / `OutOfRangeSensor` / `DuplicateRows` / `SparseLabSampleGap`).
//!   * [`FrozenTagDetectorV1`] — scans a tag's series for stuck (unchanging) runs → `FrozenTag` episodes.
//!   * [`ClockSkewWitnessV1`] — scans a timestamp series for backwards jumps + abnormal gaps.
//!
//! Bounded (non-claims): a data-quality episode flags a *data* defect, not a process fault and not a sensor
//! condemnation — it says "do not trust this span as a process signal", nothing about root cause. Additive +
//! off the replay path; deterministic, hash-sealed, self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The kind of data-quality defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataQualityKind {
    FrozenTag,
    MissingnessBurst,
    TimestampDiscontinuity,
    UnitMismatch,
    OutOfRangeSensor,
    DuplicateRows,
    SparseLabSampleGap,
}

impl DataQualityKind {
    pub fn tag(self) -> &'static str {
        match self {
            DataQualityKind::FrozenTag => "frozen_tag",
            DataQualityKind::MissingnessBurst => "missingness_burst",
            DataQualityKind::TimestampDiscontinuity => "timestamp_discontinuity",
            DataQualityKind::UnitMismatch => "unit_mismatch",
            DataQualityKind::OutOfRangeSensor => "out_of_range_sensor",
            DataQualityKind::DuplicateRows => "duplicate_rows",
            DataQualityKind::SparseLabSampleGap => "sparse_lab_sample_gap",
        }
    }
}

/// A hash-sealed data-quality episode (schema v1): one typed defect over a sample span of one tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataQualityEpisodeV1 {
    pub kind: String,
    pub tag: String,
    pub start: usize,
    pub end: usize,
    pub detail: String,
    pub episode_hash: String,
}

impl DataQualityEpisodeV1 {
    fn seal(kind: &str, tag: &str, start: usize, end: usize, detail: &str) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"data_quality_episode_v1");
        h.field("kind", kind.as_bytes());
        h.field("tag", tag.as_bytes());
        h.u64("start", start as u64);
        h.u64("end", end as u64);
        h.field("detail", detail.as_bytes());
        h.finalize_hex()
    }

    pub fn new(
        kind: DataQualityKind,
        tag: impl Into<String>,
        start: usize,
        end: usize,
        detail: impl Into<String>,
    ) -> Self {
        let (tag, detail) = (tag.into(), detail.into());
        let episode_hash = Self::seal(kind.tag(), &tag, start, end, &detail);
        DataQualityEpisodeV1 {
            kind: kind.tag().into(),
            tag,
            start,
            end,
            detail,
            episode_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(&self.kind, &self.tag, self.start, self.end, &self.detail) == self.episode_hash
    }
}

// ── FrozenTagDetectorV1 ─────────────────────────────────────────────────────────────────────────────

/// A hash-sealed frozen-tag scan (schema v1): the `FrozenTag` episodes found in one tag's series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrozenTagDetectorV1 {
    pub tag: String,
    pub min_run: usize,
    pub eps: f64,
    pub episodes: Vec<DataQualityEpisodeV1>,
    pub scan_hash: String,
}

impl FrozenTagDetectorV1 {
    fn seal(tag: &str, min_run: usize, eps: f64, episodes: &[DataQualityEpisodeV1]) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"frozen_tag_detector_v1");
        h.field("tag", tag.as_bytes());
        h.u64("min_run", min_run as u64);
        h.f64q("eps", eps);
        for e in episodes {
            h.field("episode_hash", e.episode_hash.as_bytes());
        }
        h.finalize_hex()
    }

    /// Scan `series` for maximal runs of `≥ min_run` consecutive samples whose successive change is `≤ eps`
    /// (a stuck/frozen transmitter). Non-finite samples break a run (handled as missingness elsewhere).
    pub fn scan(tag: impl Into<String>, series: &[f64], min_run: usize, eps: f64) -> Self {
        let tag = tag.into();
        let mut episodes = Vec::new();
        let mut run_start = 0usize;
        let mut run_len = 1usize;
        let flush = |episodes: &mut Vec<DataQualityEpisodeV1>, start: usize, len: usize| {
            if len >= min_run.max(2) {
                episodes.push(DataQualityEpisodeV1::new(
                    DataQualityKind::FrozenTag,
                    tag.clone(),
                    start,
                    start + len - 1,
                    format!("value unchanged for {len} samples (|Δ| ≤ {eps})"),
                ));
            }
        };
        for i in 1..series.len() {
            let frozen = series[i].is_finite()
                && series[i - 1].is_finite()
                && (series[i] - series[i - 1]).abs() <= eps;
            if frozen {
                run_len += 1;
            } else {
                flush(&mut episodes, run_start, run_len);
                run_start = i;
                run_len = 1;
            }
        }
        if !series.is_empty() {
            flush(&mut episodes, run_start, run_len);
        }
        let scan_hash = Self::seal(&tag, min_run, eps, &episodes);
        FrozenTagDetectorV1 {
            tag,
            min_run,
            eps,
            episodes,
            scan_hash,
        }
    }

    pub fn any_frozen(&self) -> bool {
        !self.episodes.is_empty()
    }

    pub fn verify(&self) -> bool {
        self.episodes.iter().all(|e| e.verify())
            && Self::seal(&self.tag, self.min_run, self.eps, &self.episodes) == self.scan_hash
    }
}

// ── ClockSkewWitnessV1 ──────────────────────────────────────────────────────────────────────────────

/// A hash-sealed clock-skew witness (schema v1): timestamp monotonicity + gap anomalies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockSkewWitnessV1 {
    pub n_samples: usize,
    pub expected_step: f64,
    /// Samples where the timestamp went backwards (`t[i] < t[i-1]`).
    pub n_backwards: usize,
    /// Samples where the gap exceeded `expected_step · gap_factor`.
    pub n_large_gaps: usize,
    /// First offending sample index (backwards or large gap), if any.
    pub first_offending_index: Option<usize>,
    /// Largest observed gap.
    pub max_gap: f64,
    pub witness_hash: String,
}

impl ClockSkewWitnessV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"clock_skew_witness_v1");
        h.u64("n_samples", self.n_samples as u64);
        h.f64q("expected_step", self.expected_step);
        h.u64("n_backwards", self.n_backwards as u64);
        h.u64("n_large_gaps", self.n_large_gaps as u64);
        h.u64("has_first", self.first_offending_index.is_some() as u64);
        h.u64(
            "first_offending_index",
            self.first_offending_index.unwrap_or(0) as u64,
        );
        h.f64q("max_gap", self.max_gap);
        h.finalize_hex()
    }

    /// Scan a timestamp series: a backwards step is `t[i] < t[i-1]`; a large gap is
    /// `t[i] − t[i-1] > expected_step · gap_factor`. Non-finite timestamps are skipped.
    pub fn scan(timestamps: &[f64], expected_step: f64, gap_factor: f64) -> Self {
        let (mut n_backwards, mut n_large_gaps, mut first_offending_index, mut max_gap) =
            (0usize, 0usize, None, 0.0f64);
        let threshold = expected_step * gap_factor;
        for i in 1..timestamps.len() {
            let (a, b) = (timestamps[i - 1], timestamps[i]);
            if !a.is_finite() || !b.is_finite() {
                continue;
            }
            let d = b - a;
            if d < 0.0 {
                n_backwards += 1;
                first_offending_index.get_or_insert(i);
            } else {
                max_gap = max_gap.max(d);
                if d > threshold {
                    n_large_gaps += 1;
                    first_offending_index.get_or_insert(i);
                }
            }
        }
        let mut w = ClockSkewWitnessV1 {
            n_samples: timestamps.len(),
            expected_step,
            n_backwards,
            n_large_gaps,
            first_offending_index,
            max_gap,
            witness_hash: String::new(),
        };
        w.witness_hash = w.seal();
        w
    }

    pub fn is_clean(&self) -> bool {
        self.n_backwards == 0 && self.n_large_gaps == 0
    }

    pub fn verify(&self) -> bool {
        self.seal() == self.witness_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_tag_detects_a_stuck_run() {
        // A sensor stuck at 5.0 for samples 2..6, otherwise moving.
        let series = [1.0, 3.0, 5.0, 5.0, 5.0, 5.0, 5.0, 8.0];
        let d = FrozenTagDetectorV1::scan("TI-101", &series, 3, 1e-9);
        assert!(d.any_frozen());
        let ep = &d.episodes[0];
        assert_eq!(ep.kind, "frozen_tag");
        assert_eq!((ep.start, ep.end), (2, 6)); // 5 unchanged samples (indices 2..6)
        assert!(d.scan_hash.len() == 64 && d.verify());
        // No frozen run when the signal always moves.
        let moving = FrozenTagDetectorV1::scan("x", &[1.0, 2.0, 3.0, 4.0], 3, 1e-9);
        assert!(!moving.any_frozen());
    }

    #[test]
    fn clock_skew_flags_backwards_and_gaps() {
        // Regular step 1.0; a backwards jump at i=3 and a big gap at i=5.
        let ts = [0.0, 1.0, 2.0, 1.5, 2.5, 10.0, 11.0];
        let w = ClockSkewWitnessV1::scan(&ts, 1.0, 3.0); // gap threshold 3.0
        assert_eq!(w.n_backwards, 1); // 1.5 < 2.0
        assert_eq!(w.n_large_gaps, 1); // 10.0 − 2.5 = 7.5 > 3.0
        assert_eq!(w.first_offending_index, Some(3));
        assert!((w.max_gap - 7.5).abs() < 1e-12);
        assert!(!w.is_clean() && w.verify());
        // A regular clock is clean.
        let clean = ClockSkewWitnessV1::scan(&[0.0, 1.0, 2.0, 3.0], 1.0, 3.0);
        assert!(clean.is_clean() && clean.verify());
    }

    #[test]
    fn episodes_and_witness_are_tamper_evident() {
        let mut d = FrozenTagDetectorV1::scan("t", &[5.0, 5.0, 5.0, 5.0], 3, 1e-9);
        assert!(d.verify());
        d.episodes[0].end = 999; // forge the span without re-sealing the episode
        assert!(!d.verify());
        let mut w = ClockSkewWitnessV1::scan(&[0.0, 1.0, 2.0], 1.0, 3.0);
        assert!(w.verify());
        w.n_backwards = 5; // forge a count
        assert!(!w.verify());
    }
}
