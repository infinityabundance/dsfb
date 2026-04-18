//! PELT — Pruned Exact Linear Time change-point detection.
//!
//! Reference: Killick, Fearnhead & Eckley, *Optimal Detection of
//! Changepoints With a Linear Computational Cost*, JASA 107 (500), 2012.
//!
//! PELT solves the optimal-partition DP
//!
//! ```text
//!   F(t) = min_{τ < t} [ F(τ) + C(y_{τ+1..t}) + β ]
//! ```
//!
//! where `C` is a segment cost (here L2 on the mean, i.e.
//! `Σ (yᵢ − ȳ)²`) and `β` is the per-change-point penalty. The
//! pruning step removes candidates `τ` for which
//! `F(τ) + C(τ+1:t) + K ≥ F(t)` — those can never be optimal later —
//! and gives the algorithm its linear expected-time behaviour under a
//! Poisson-rate change-point assumption.
//!
//! `K` is the cost-reduction constant; for the L2 cost `K = 0` is
//! valid (see Killick et al. §2.2).

use super::ChangePointDetector;

pub struct Pelt {
    /// BIC penalty coefficient. The BIC default is `β = k · ln(n) · σ²`
    /// with `k = 1`; we compute `σ²` from the input series' variance so
    /// the penalty adapts to scale.
    pub penalty_k: f64,
    /// Minimum segment length. Killick recommends ≥ 2; 5 damps noise
    /// for small series.
    pub min_seg_len: usize,
}

impl Default for Pelt {
    fn default() -> Self {
        Self {
            penalty_k: 2.0,
            min_seg_len: 5,
        }
    }
}

impl ChangePointDetector for Pelt {
    fn name(&self) -> &'static str {
        "pelt"
    }

    fn detect(&self, series: &[(f64, f64)]) -> Vec<f64> {
        let n = series.len();
        if n < 2 * self.min_seg_len {
            return Vec::new();
        }

        // Prefix sums for O(1) segment cost:
        // cost(i..=j) = Σ y² − (Σ y)² / (j - i + 1)
        let mut prefix_sum = vec![0.0f64; n + 1];
        let mut prefix_sq = vec![0.0f64; n + 1];
        for i in 0..n {
            prefix_sum[i + 1] = prefix_sum[i] + series[i].1;
            prefix_sq[i + 1] = prefix_sq[i] + series[i].1 * series[i].1;
        }
        let mean = prefix_sum[n] / n as f64;
        let var = (prefix_sq[n] / n as f64 - mean * mean).max(f64::MIN_POSITIVE);
        let beta = self.penalty_k * (n as f64).ln() * var;

        let cost = |i: usize, j: usize| -> f64 {
            debug_assert!(i < j, "nonempty segment");
            let k = (j - i) as f64;
            let s = prefix_sum[j] - prefix_sum[i];
            let sq = prefix_sq[j] - prefix_sq[i];
            (sq - s * s / k).max(0.0)
        };

        // DP tables.
        let mut f = vec![f64::INFINITY; n + 1];
        let mut prev = vec![0usize; n + 1];
        f[0] = -beta;
        let mut candidates: Vec<usize> = vec![0];

        for t in self.min_seg_len..=n {
            let (mut best_cost, mut best_tau) = (f64::INFINITY, 0usize);
            let mut next_cands: Vec<usize> = Vec::with_capacity(candidates.len() + 1);
            for &tau in &candidates {
                if t - tau < self.min_seg_len {
                    next_cands.push(tau);
                    continue;
                }
                let seg = cost(tau, t);
                let c = f[tau] + seg + beta;
                if c < best_cost {
                    best_cost = c;
                    best_tau = tau;
                }
                // Pruning: if f[τ] + cost(τ..t) ≥ f[t], τ cannot be the
                // optimal last change-point at any future time either
                // (Killick et al. §2.2, with K = 0 for L2).
                if f[tau] + seg < f[t].min(best_cost) {
                    next_cands.push(tau);
                }
            }
            f[t] = best_cost;
            prev[t] = best_tau;
            // Always keep `t` as a future candidate (segment boundary).
            next_cands.push(t);
            candidates = next_cands;
        }

        // Backtrack.
        let mut cuts: Vec<usize> = Vec::new();
        let mut t = n;
        while t > 0 {
            let p = prev[t];
            if p > 0 {
                cuts.push(p);
            }
            if p == t {
                break;
            }
            t = p;
        }
        cuts.sort_unstable();
        cuts.dedup();
        cuts.into_iter().map(|i| series[i].0).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_one_shift() {
        let mut series = Vec::new();
        for i in 0..60 {
            series.push((i as f64, 0.0));
        }
        for i in 60..120 {
            series.push((i as f64, 3.0));
        }
        let cps = Pelt::default().detect(&series);
        assert!(!cps.is_empty(), "PELT should flag the obvious shift");
        assert!(cps.iter().any(|&t| (t - 60.0).abs() < 10.0));
    }

    #[test]
    fn quiet_on_flat() {
        let series: Vec<(f64, f64)> = (0..200).map(|i| (i as f64, 0.0)).collect();
        let cps = Pelt::default().detect(&series);
        assert!(cps.is_empty(), "PELT should not invent changes");
    }
}
