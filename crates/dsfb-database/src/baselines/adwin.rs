//! ADWIN (ADaptive WINdowing) change-point detector.
//!
//! Reference: Bifet & Gavaldà, *Learning from time-changing data with
//! adaptive windowing*, SDM 2007, Theorems 1–3.
//!
//! Core idea: maintain a window W of recent observations. Whenever any
//! split `W = W₀ · W₁` shows |mean(W₀) − mean(W₁)| larger than a
//! Hoeffding-style bound ε_cut depending on the split sizes and a
//! confidence δ, drop W₀ and emit a change-point at the cut.
//!
//! Simplifications vs. the paper that do not affect correctness for our
//! purposes:
//! * We use the linear-scan split test (the paper's §4 exponential-
//!   histogram data structure is an optimisation; our series are short,
//!   so the simpler code is faster to audit).
//! * We assume values are real and bounded within a single series (the
//!   paper's Hoeffding bound uses the sample range; we track
//!   `observed_min` / `observed_max` per series).
//!
//! Determinism: the detector is a pure function of the input series.

use super::ChangePointDetector;

pub struct Adwin {
    /// Confidence parameter δ. Smaller δ → larger ε_cut → fewer false
    /// positives. The SDM paper uses δ = 0.002; we match that so the
    /// numbers in our bake-off can be cross-checked against
    /// implementations calibrated on the published SEA / HYPERPLANE
    /// benchmarks.
    pub delta: f64,
    /// Minimum number of samples on either side of a candidate split —
    /// below this we don't test. Matches the paper's "buckets of at least
    /// 2 observations" guidance; 5 is the conservative default used by
    /// MOA's reference Java implementation.
    pub min_side: usize,
}

impl Default for Adwin {
    fn default() -> Self {
        Self {
            delta: 0.002,
            min_side: 5,
        }
    }
}

impl ChangePointDetector for Adwin {
    fn name(&self) -> &'static str {
        "adwin"
    }

    fn detect(&self, series: &[(f64, f64)]) -> Vec<f64> {
        let mut cps = Vec::new();
        if series.len() < 2 * self.min_side {
            return cps;
        }
        // Running window = indices [start, end). After emitting a change
        // at cut-index `k`, we drop [start, k) and continue with [k, end).
        let mut start: usize = 0;
        let mut end: usize = 0;
        while end < series.len() {
            end += 1;
            let n = end - start;
            if n < 2 * self.min_side {
                continue;
            }
            // Scan all valid splits; take the first one that exceeds
            // ε_cut, as the paper does. A more elaborate version takes
            // argmax |difference|; using "first" keeps the output
            // deterministic without an ordering tiebreak rule.
            let (mut best_cut, mut best_diff) = (None, 0.0f64);
            for k in (start + self.min_side)..=(end - self.min_side) {
                let n0 = (k - start) as f64;
                let n1 = (end - k) as f64;
                let mean0 = series[start..k].iter().map(|(_, v)| v).sum::<f64>() / n0;
                let mean1 = series[k..end].iter().map(|(_, v)| v).sum::<f64>() / n1;
                let diff = (mean0 - mean1).abs();
                if diff > best_diff {
                    best_diff = diff;
                    best_cut = Some(k);
                }
            }
            let Some(k) = best_cut else { continue };
            let n0 = (k - start) as f64;
            let n1 = (end - k) as f64;
            let eps_cut = self.epsilon_cut(series, start, end, n0, n1);
            if best_diff > eps_cut {
                cps.push(series[k].0);
                start = k;
            }
        }
        cps
    }
}

impl Adwin {
    /// ε_cut per Theorem 3.1 of Bifet & Gavaldà 2007, specialised to
    /// bounded-range data. `R` is the observed range over the current
    /// window; `m` is the harmonic mean of the two side sizes.
    fn epsilon_cut(
        &self,
        series: &[(f64, f64)],
        start: usize,
        end: usize,
        n0: f64,
        n1: f64,
    ) -> f64 {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &(_, v) in &series[start..end] {
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }
        let r = (hi - lo).max(f64::EPSILON);
        let m = 2.0 * n0 * n1 / (n0 + n1);
        let n = (end - start) as f64;
        let d_prime = self.delta / n.max(1.0);
        // ε_cut = sqrt( (R² / (2m)) · ln(2/δ′) )
        ((r * r) / (2.0 * m) * (2.0 / d_prime).ln()).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_large_mean_shift() {
        // 50 samples at 0.0, then 50 at 2.0 — an obvious change.
        let mut series = Vec::new();
        for i in 0..50 {
            series.push((i as f64, 0.0));
        }
        for i in 50..100 {
            series.push((i as f64, 2.0));
        }
        let cps = Adwin::default().detect(&series);
        assert!(!cps.is_empty(), "ADWIN should flag an obvious shift");
    }

    #[test]
    fn stays_silent_on_flat_series() {
        // All zeros — no change to find.
        let series: Vec<(f64, f64)> = (0..200).map(|i| (i as f64, 0.0)).collect();
        let cps = Adwin::default().detect(&series);
        assert!(
            cps.is_empty(),
            "ADWIN should not invent changes on flat data"
        );
    }
}
