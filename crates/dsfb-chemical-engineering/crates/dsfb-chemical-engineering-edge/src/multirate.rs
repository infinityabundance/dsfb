//! `MultiRateAlignmentCourtV1` + `ManualSampleBridgeV1` (Wave-4 historian layer) — reconcile ragged,
//! multi-rate plant-historian tags onto a common grid **deterministically and auditably**.
//!
//! A real historian holds tags at wildly different rates: a 1 s flow, a 1 min temperature, an hourly
//! analyser, a once-a-shift lab sample. To analyse them jointly they must land on one time grid — and *how*
//! that resampling is done (zero-order hold? linear interpolation? nearest within tolerance? dropped where
//! data is absent?) materially changes the residuals. A monitor that silently resamples is untrustworthy.
//! This module makes the alignment a **sealed receipt**: per tag, the native interval, the chosen policy, the
//! target grid, and exactly how many grid points were filled by an exact sample / a held value / an
//! interpolated value / left missing. Nothing is fabricated — out-of-range grid points are `missing`, not
//! extrapolated.
//!
//! [`ManualSampleBridgeV1`] handles the sparsest stream — manual/lab samples — by bridging each onto its
//! nearest grid point with the **alignment uncertainty** (the time gap) and a **chain-of-custody hash** made
//! explicit, so a sparse ground-truth point is never mistaken for a continuous measurement.
//!
//! Bounded (non-claims): an alignment is a documented *resampling*, not new information — a held or
//! interpolated value is an assumption, recorded as such; a manual sample is a sparse point with a stated
//! time uncertainty, not a continuous signal. Additive + off the replay path; deterministic, hash-sealed,
//! self-verifying.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// Absolute tolerance for "a native sample lands exactly on a grid point" (seconds, or the grid's time unit).
const EXACT_EPS: f64 = 1e-9;

/// How a tag's native samples are mapped onto the target grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignmentPolicy {
    /// Each grid point takes the most recent sample at or before it; before the first sample ⇒ missing.
    ZeroOrderHold,
    /// Linear interpolation between the bracketing samples; outside the sample range ⇒ missing (no extrapolation).
    Linear,
    /// The nearest sample within half a grid step ⇒ else missing (a snap, not a carry-forward).
    Nearest,
}

impl AlignmentPolicy {
    fn tag(self) -> &'static str {
        match self {
            AlignmentPolicy::ZeroOrderHold => "zero_order_hold",
            AlignmentPolicy::Linear => "linear",
            AlignmentPolicy::Nearest => "nearest",
        }
    }
}

/// The full result of aligning one tag (carries the aligned `values`, NaN where missing).
#[derive(Debug, Clone, PartialEq)]
pub struct AlignedTag {
    pub tag: String,
    pub policy: AlignmentPolicy,
    pub native_interval: f64,
    pub n_native: usize,
    pub n_grid: usize,
    pub n_exact: usize,
    pub n_held: usize,
    pub n_interpolated: usize,
    pub n_missing: usize,
    pub values: Vec<f64>,
}

/// Median of the consecutive timestamp differences in a sorted sample set (0 if fewer than 2 samples).
fn median_interval(samples: &[(f64, f64)]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mut d: Vec<f64> = samples
        .windows(2)
        .map(|w| w[1].0 - w[0].0)
        .filter(|x| x.is_finite())
        .collect();
    if d.is_empty() {
        return 0.0;
    }
    d.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let m = d.len() / 2;
    if d.len() % 2 == 1 {
        d[m]
    } else {
        0.5 * (d[m - 1] + d[m])
    }
}

/// Align one tag's `(timestamp, value)` samples (which **must be sorted by timestamp**) onto the regular grid
/// `g_j = grid_start + j·grid_step`, `j ∈ [0, n_grid)`, under `policy`. Deterministic and pure; never
/// extrapolates — a grid point outside the sample range (or with no qualifying sample) is `NaN` and counted
/// `missing`. Non-finite samples are ignored.
pub fn align_tag(
    tag: impl Into<String>,
    samples: &[(f64, f64)],
    grid_start: f64,
    grid_step: f64,
    n_grid: usize,
    policy: AlignmentPolicy,
) -> AlignedTag {
    let s: Vec<(f64, f64)> = samples
        .iter()
        .copied()
        .filter(|(t, v)| t.is_finite() && v.is_finite())
        .collect();
    let (mut n_exact, mut n_held, mut n_interpolated, mut n_missing) =
        (0usize, 0usize, 0usize, 0usize);
    let mut values = Vec::with_capacity(n_grid);
    for j in 0..n_grid {
        let g = grid_start + j as f64 * grid_step;
        // Index of the first sample strictly after g; `lo = hi-1` is the last sample at/before g.
        let hi = s.partition_point(|&(t, _)| t <= g + EXACT_EPS);
        let exact = hi > 0 && (s[hi - 1].0 - g).abs() <= EXACT_EPS;
        let (v, bucket) = if exact {
            (s[hi - 1].1, 0u8) // exact
        } else {
            match policy {
                AlignmentPolicy::ZeroOrderHold => {
                    if hi > 0 {
                        (s[hi - 1].1, 1) // held
                    } else {
                        (f64::NAN, 3) // before the first sample
                    }
                }
                AlignmentPolicy::Linear => {
                    if hi > 0 && hi < s.len() {
                        let (t0, v0) = s[hi - 1];
                        let (t1, v1) = s[hi];
                        let w = if (t1 - t0).abs() > EXACT_EPS {
                            (g - t0) / (t1 - t0)
                        } else {
                            0.0
                        };
                        (v0 + w * (v1 - v0), 2) // interpolated
                    } else {
                        (f64::NAN, 3) // outside the sample range ⇒ no extrapolation
                    }
                }
                AlignmentPolicy::Nearest => {
                    // Nearest of the two neighbours within half a grid step.
                    let mut best: Option<(f64, f64)> = None; // (distance, value)
                    for (t, val) in [hi.checked_sub(1).map(|i| s[i]), s.get(hi).copied()]
                        .into_iter()
                        .flatten()
                    {
                        let dist = (t - g).abs();
                        if dist <= 0.5 * grid_step.abs() && best.is_none_or(|(bd, _)| dist < bd) {
                            best = Some((dist, val));
                        }
                    }
                    match best {
                        Some((_, val)) => (val, 1), // snapped ⇒ counted as held
                        None => (f64::NAN, 3),
                    }
                }
            }
        };
        match bucket {
            0 => n_exact += 1,
            1 => n_held += 1,
            2 => n_interpolated += 1,
            _ => n_missing += 1,
        }
        values.push(v);
    }
    AlignedTag {
        tag: tag.into(),
        policy,
        native_interval: median_interval(&s),
        n_native: s.len(),
        n_grid,
        n_exact,
        n_held,
        n_interpolated,
        n_missing,
        values,
    }
}

/// The sealed per-tag alignment receipt (counts + a hash of the aligned stream; the full stream is hashed,
/// not stored, like the balance witness).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignedTagReceipt {
    pub tag: String,
    pub policy: String,
    pub native_interval: f64,
    pub n_native: usize,
    pub n_exact: usize,
    pub n_held: usize,
    pub n_interpolated: usize,
    pub n_missing: usize,
    pub values_hash: String,
}

fn hash_values(values: &[f64]) -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", b"aligned_values_v1");
    for &v in values {
        h.f64q("v", v);
    }
    h.finalize_hex()
}

impl AlignedTagReceipt {
    fn from(a: &AlignedTag) -> Self {
        AlignedTagReceipt {
            tag: a.tag.clone(),
            policy: a.policy.tag().to_string(),
            native_interval: a.native_interval,
            n_native: a.n_native,
            n_exact: a.n_exact,
            n_held: a.n_held,
            n_interpolated: a.n_interpolated,
            n_missing: a.n_missing,
            values_hash: hash_values(&a.values),
        }
    }
}

/// One tag's input to the court: its name, its `(timestamp, value)` native samples (sorted by timestamp),
/// and the alignment policy chosen for it.
pub type TagSampleStream = (String, Vec<(f64, f64)>, AlignmentPolicy);

/// A hash-sealed multi-rate alignment court (schema v1) over a set of tags aligned to one grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiRateAlignmentCourtV1 {
    pub grid_start: f64,
    pub grid_step: f64,
    pub n_grid: usize,
    pub receipts: Vec<AlignedTagReceipt>,
    /// Total grid cells left missing across all tags (the headline "how much was not real data").
    pub total_missing: usize,
    pub court_hash: String,
}

impl MultiRateAlignmentCourtV1 {
    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"multi_rate_alignment_court_v1");
        h.f64q("grid_start", self.grid_start);
        h.f64q("grid_step", self.grid_step);
        h.u64("n_grid", self.n_grid as u64);
        for r in &self.receipts {
            h.field("tag", r.tag.as_bytes());
            h.field("policy", r.policy.as_bytes());
            h.f64q("native_interval", r.native_interval);
            h.u64("n_native", r.n_native as u64);
            h.u64("n_exact", r.n_exact as u64);
            h.u64("n_held", r.n_held as u64);
            h.u64("n_interpolated", r.n_interpolated as u64);
            h.u64("n_missing", r.n_missing as u64);
            h.field("values_hash", r.values_hash.as_bytes());
        }
        h.u64("total_missing", self.total_missing as u64);
        h.finalize_hex()
    }

    /// Align every `(tag, samples, policy)` onto the shared grid and seal the receipts. Returns both the court
    /// (sealed receipts) and the aligned matrix (one `Vec<f64>` per tag, in input order) for downstream use.
    pub fn build(
        grid_start: f64,
        grid_step: f64,
        n_grid: usize,
        tags: &[TagSampleStream],
    ) -> (Self, Vec<AlignedTag>) {
        let aligned: Vec<AlignedTag> = tags
            .iter()
            .map(|(name, samples, policy)| {
                align_tag(
                    name.clone(),
                    samples,
                    grid_start,
                    grid_step,
                    n_grid,
                    *policy,
                )
            })
            .collect();
        let receipts: Vec<AlignedTagReceipt> =
            aligned.iter().map(AlignedTagReceipt::from).collect();
        let total_missing = receipts.iter().map(|r| r.n_missing).sum();
        let mut court = MultiRateAlignmentCourtV1 {
            grid_start,
            grid_step,
            n_grid,
            receipts,
            total_missing,
            court_hash: String::new(),
        };
        court.court_hash = court.seal();
        (court, aligned)
    }

    /// Re-derive `total_missing` + re-seal and compare the whole record.
    pub fn verify(&self) -> bool {
        let total_missing: usize = self.receipts.iter().map(|r| r.n_missing).sum();
        total_missing == self.total_missing && self.seal() == self.court_hash
    }
}

/// One bridged manual/lab sample onto the grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgedSample {
    /// The grid index this sample is associated with (the nearest grid point).
    pub grid_index: usize,
    pub sample_time: f64,
    pub value: f64,
    /// `|sample_time − grid_time|` — the time gap the bridge introduces (the alignment uncertainty).
    pub alignment_uncertainty: f64,
    /// SHA-256 over `(custody_id, sample_time, value)` — a chain-of-custody seal for this point.
    pub custody_hash: String,
}

/// A hash-sealed manual-sample bridge (schema v1): sparse lab/manual samples mapped onto the grid with their
/// alignment uncertainty and chain-of-custody hashes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManualSampleBridgeV1 {
    pub tag: String,
    pub grid_start: f64,
    pub grid_step: f64,
    pub n_grid: usize,
    pub samples: Vec<BridgedSample>,
    /// Largest alignment uncertainty across the bridged samples (the worst-case time gap).
    pub max_alignment_uncertainty: f64,
    pub bridge_hash: String,
}

impl ManualSampleBridgeV1 {
    fn custody_hash(custody_id: &str, t: f64, v: f64) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"manual_sample_custody_v1");
        h.field("custody_id", custody_id.as_bytes());
        h.f64q("sample_time", t);
        h.f64q("value", v);
        h.finalize_hex()
    }

    fn seal(&self) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"manual_sample_bridge_v1");
        h.field("tag", self.tag.as_bytes());
        h.f64q("grid_start", self.grid_start);
        h.f64q("grid_step", self.grid_step);
        h.u64("n_grid", self.n_grid as u64);
        for b in &self.samples {
            h.u64("grid_index", b.grid_index as u64);
            h.f64q("sample_time", b.sample_time);
            h.f64q("value", b.value);
            h.f64q("alignment_uncertainty", b.alignment_uncertainty);
            h.field("custody_hash", b.custody_hash.as_bytes());
        }
        h.f64q("max_alignment_uncertainty", self.max_alignment_uncertainty);
        h.finalize_hex()
    }

    /// Bridge `(sample_time, value, custody_id)` lab/manual samples onto the grid by nearest grid point.
    /// Non-finite samples are skipped. A sample beyond the grid is clamped to the nearest end index, and the
    /// (larger) uncertainty reflects that gap honestly.
    pub fn build(
        tag: impl Into<String>,
        grid_start: f64,
        grid_step: f64,
        n_grid: usize,
        raw: &[(f64, f64, String)],
    ) -> Self {
        let mut samples = Vec::new();
        let mut max_alignment_uncertainty = 0.0f64;
        for (t, v, custody) in raw {
            if !t.is_finite() || !v.is_finite() || n_grid == 0 || grid_step == 0.0 {
                continue;
            }
            let raw_idx = ((t - grid_start) / grid_step).round();
            let grid_index = raw_idx.clamp(0.0, (n_grid - 1) as f64) as usize;
            let grid_time = grid_start + grid_index as f64 * grid_step;
            let unc = (t - grid_time).abs();
            max_alignment_uncertainty = max_alignment_uncertainty.max(unc);
            samples.push(BridgedSample {
                grid_index,
                sample_time: *t,
                value: *v,
                alignment_uncertainty: unc,
                custody_hash: Self::custody_hash(custody, *t, *v),
            });
        }
        let mut b = ManualSampleBridgeV1 {
            tag: tag.into(),
            grid_start,
            grid_step,
            n_grid,
            samples,
            max_alignment_uncertainty,
            bridge_hash: String::new(),
        };
        b.bridge_hash = b.seal();
        b
    }

    pub fn verify(&self) -> bool {
        let max_unc = self
            .samples
            .iter()
            .map(|s| s.alignment_uncertainty)
            .fold(0.0f64, f64::max);
        (max_unc - self.max_alignment_uncertainty).abs() <= 1e-12 && self.seal() == self.bridge_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_order_hold_holds_and_marks_pre_first_sample_missing() {
        // Samples at t=2 (10) and t=6 (20); grid 0,2,4,6,8.
        let s = vec![(2.0, 10.0), (6.0, 20.0)];
        let a = align_tag("flow", &s, 0.0, 2.0, 5, AlignmentPolicy::ZeroOrderHold);
        // g0=0 → missing (before first); g1=2 → exact 10; g2=4 → held 10; g3=6 → exact 20; g4=8 → held 20.
        assert!(a.values[0].is_nan());
        assert_eq!(a.values[1], 10.0);
        assert_eq!(a.values[2], 10.0);
        assert_eq!(a.values[3], 20.0);
        assert_eq!(a.values[4], 20.0);
        assert_eq!(
            (a.n_exact, a.n_held, a.n_interpolated, a.n_missing),
            (2, 2, 0, 1)
        );
        assert!((a.native_interval - 4.0).abs() < 1e-12);
    }

    #[test]
    fn linear_interpolates_and_never_extrapolates() {
        let s = vec![(2.0, 10.0), (6.0, 30.0)]; // slope 5/unit
        let a = align_tag("temp", &s, 0.0, 2.0, 5, AlignmentPolicy::Linear);
        assert!(a.values[0].is_nan()); // g0=0 before range
        assert_eq!(a.values[1], 10.0); // exact
        assert_eq!(a.values[2], 20.0); // interpolated midpoint
        assert_eq!(a.values[3], 30.0); // exact
        assert!(a.values[4].is_nan()); // g4=8 beyond range ⇒ no extrapolation
        assert_eq!((a.n_exact, a.n_interpolated, a.n_missing), (2, 1, 2));
    }

    #[test]
    fn court_seals_receipts_and_self_verifies() {
        let tags = vec![
            (
                "flow".to_string(),
                vec![(2.0, 10.0), (6.0, 20.0)],
                AlignmentPolicy::ZeroOrderHold,
            ),
            (
                "temp".to_string(),
                vec![(2.0, 10.0), (6.0, 30.0)],
                AlignmentPolicy::Linear,
            ),
        ];
        let (court, aligned) = MultiRateAlignmentCourtV1::build(0.0, 2.0, 5, &tags);
        assert_eq!(court.receipts.len(), 2);
        assert_eq!(court.total_missing, 1 + 2); // ZOH 1 missing + Linear 2 missing
        assert!(court.court_hash.len() == 64 && court.verify());
        // The receipt's values_hash matches the aligned stream it summarises.
        assert_eq!(
            court.receipts[0].values_hash,
            hash_values(&aligned[0].values)
        );
        // Determinism.
        let (court2, _) = MultiRateAlignmentCourtV1::build(0.0, 2.0, 5, &tags);
        assert_eq!(court2, court);
    }

    #[test]
    fn manual_sample_bridge_records_uncertainty_and_custody() {
        // Grid 0,1,2,3,4 (step 1). Lab sample at t=2.3 → nearest grid index 2, uncertainty 0.3.
        let raw = vec![
            (2.3, 99.0, "LIMS-001".to_string()),
            (0.1, 50.0, "LIMS-002".to_string()),
        ];
        let b = ManualSampleBridgeV1::build("assay", 0.0, 1.0, 5, &raw);
        assert_eq!(b.samples.len(), 2);
        assert_eq!(b.samples[0].grid_index, 2);
        assert!((b.samples[0].alignment_uncertainty - 0.3).abs() < 1e-12);
        assert!((b.max_alignment_uncertainty - 0.3).abs() < 1e-12);
        assert_eq!(b.samples[0].custody_hash.len(), 64);
        assert!(b.verify());
        // A different custody id ⇒ a different custody hash (chain-of-custody is sealed).
        let b2 =
            ManualSampleBridgeV1::build("assay", 0.0, 1.0, 5, &[(2.3, 99.0, "OTHER".to_string())]);
        assert_ne!(b2.samples[0].custody_hash, b.samples[0].custody_hash);
    }

    #[test]
    fn tampering_breaks_the_seals() {
        let tags = vec![(
            "f".to_string(),
            vec![(0.0, 1.0), (1.0, 2.0)],
            AlignmentPolicy::Linear,
        )];
        let (mut court, _) = MultiRateAlignmentCourtV1::build(0.0, 1.0, 3, &tags);
        assert!(court.verify());
        court.receipts[0].n_missing += 1; // forge a worse missing count without re-sealing
        assert!(!court.verify());

        let mut b = ManualSampleBridgeV1::build("a", 0.0, 1.0, 3, &[(1.2, 5.0, "c".into())]);
        assert!(b.verify());
        b.samples[0].alignment_uncertainty = 0.0; // forge away the time gap
        assert!(!b.verify());
    }
}
