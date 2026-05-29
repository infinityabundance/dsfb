//! `SensitivitySweepReceiptV1` — a deterministic threshold-grid sensitivity receipt (P63).
//!
//! A reviewer's fair question of any thresholded method is "how much does the answer move if you nudge
//! the thresholds?". This runs a **deterministic Cartesian grid** over named threshold axes (envelope
//! `k`, quorum `min_families`, drift window, …), evaluates a caller-supplied metric at every grid point
//! in a fixed order, and seals the whole grid + results under a `receipt_hash`. The summary
//! `metric_range` (max − min over the grid) is the headline robustness number: a small range means the
//! reported metric is insensitive to threshold choice; a large range discloses the opposite honestly.
//!
//! Deterministic by construction (fixed axis order, row-major product, no RNG); additive + off the
//! replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// One threshold axis of the sweep: a name + the discrete values swept on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweepAxis {
    pub name: String,
    pub values: Vec<f64>,
}

/// One evaluated grid point: the coordinate (one value per axis, in axis order) and the metric there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SweepPoint {
    pub coords: Vec<f64>,
    pub metric: f64,
}

/// A hash-sealed sensitivity-sweep receipt (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensitivitySweepReceiptV1 {
    /// What the metric measures (e.g. `"baseline_fp_rate"`, `"n_episodes"`, `"detection_delay"`).
    pub metric_name: String,
    pub axes: Vec<SweepAxis>,
    pub points: Vec<SweepPoint>,
    pub metric_min: f64,
    pub metric_max: f64,
    /// `metric_max − metric_min` — the headline sensitivity (small ⇒ robust to threshold choice).
    pub metric_range: f64,
    pub receipt_hash: String,
}

impl SensitivitySweepReceiptV1 {
    fn seal(
        metric_name: &str,
        axes: &[SweepAxis],
        points: &[SweepPoint],
        lo: f64,
        hi: f64,
    ) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"sensitivity_sweep_receipt_v1");
        h.field("metric_name", metric_name.as_bytes());
        for a in axes {
            h.field("axis", a.name.as_bytes());
            for &v in &a.values {
                h.f64q("axis_value", v);
            }
        }
        for p in points {
            for &c in &p.coords {
                h.f64q("coord", c);
            }
            h.f64q("metric", p.metric);
        }
        h.f64q("metric_min", lo);
        h.f64q("metric_max", hi);
        h.finalize_hex()
    }

    /// Run the sweep: evaluate `eval(coords)` at every point of the Cartesian product of the axes, in
    /// row-major order (the last axis varies fastest), and seal the receipt. `eval` must be a pure
    /// deterministic function of the coordinate for the receipt to be reproducible.
    pub fn run(
        metric_name: impl Into<String>,
        axes: Vec<SweepAxis>,
        eval: impl Fn(&[f64]) -> f64,
    ) -> Self {
        let metric_name = metric_name.into();
        // Cartesian product, row-major (last axis fastest). Empty-axis product = a single empty point.
        let mut points: Vec<SweepPoint> = Vec::new();
        let total: usize = axes.iter().map(|a| a.values.len().max(1)).product();
        for idx in 0..total {
            let mut rem = idx;
            let mut coords = vec![0.0f64; axes.len()];
            // Decode the flat index into per-axis indices, last axis fastest.
            for ai in (0..axes.len()).rev() {
                let n = axes[ai].values.len().max(1);
                let k = rem % n;
                rem /= n;
                coords[ai] = axes[ai].values.get(k).copied().unwrap_or(0.0);
            }
            let metric = eval(&coords);
            points.push(SweepPoint { coords, metric });
        }
        let finite: Vec<f64> = points
            .iter()
            .map(|p| p.metric)
            .filter(|x| x.is_finite())
            .collect();
        let metric_min = finite.iter().copied().fold(f64::INFINITY, f64::min);
        let metric_max = finite.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (metric_min, metric_max) = if finite.is_empty() {
            (0.0, 0.0)
        } else {
            (metric_min, metric_max)
        };
        let metric_range = metric_max - metric_min;
        let receipt_hash = Self::seal(&metric_name, &axes, &points, metric_min, metric_max);
        SensitivitySweepReceiptV1 {
            metric_name,
            axes,
            points,
            metric_min,
            metric_max,
            metric_range,
            receipt_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.metric_name,
            &self.axes,
            &self.points,
            self.metric_min,
            self.metric_max,
        ) == self.receipt_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_covers_cartesian_product_in_fixed_order() {
        let axes = vec![
            SweepAxis {
                name: "k".into(),
                values: vec![2.0, 3.0],
            },
            SweepAxis {
                name: "min_families".into(),
                values: vec![1.0, 2.0, 3.0],
            },
        ];
        // metric = k + min_families (deterministic).
        let r = SensitivitySweepReceiptV1::run("toy", axes, |c| c[0] + c[1]);
        assert_eq!(r.points.len(), 6); // 2 × 3
                                       // Row-major, last axis fastest: first point (2,1), second (2,2), ...
        assert_eq!(r.points[0].coords, vec![2.0, 1.0]);
        assert_eq!(r.points[1].coords, vec![2.0, 2.0]);
        assert_eq!(r.points[3].coords, vec![3.0, 1.0]);
        assert!((r.metric_min - 3.0).abs() < 1e-12 && (r.metric_max - 6.0).abs() < 1e-12);
        assert!((r.metric_range - 3.0).abs() < 1e-12);
        assert!(r.verify());
    }

    #[test]
    fn deterministic_and_tamper_evident() {
        let axes = vec![SweepAxis {
            name: "t".into(),
            values: vec![0.0, 1.0, 2.0],
        }];
        let a = SensitivitySweepReceiptV1::run("m", axes.clone(), |c| c[0] * c[0]);
        let b = SensitivitySweepReceiptV1::run("m", axes, |c| c[0] * c[0]);
        assert_eq!(a.receipt_hash, b.receipt_hash);
        let mut t = a.clone();
        t.points[2].metric = 999.0;
        assert!(!t.verify());
    }
}
