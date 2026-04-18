//! BOCPD — Bayesian Online Change-Point Detection.
//!
//! Reference: Adams & MacKay, *Bayesian Online Changepoint Detection*,
//! arXiv:0710.3742 (2007).
//!
//! Core idea: at each time `t` maintain a posterior `P(r_t | x_{1..t})`
//! over the run length `r_t` (the number of samples since the last
//! change-point). Updates factorise into
//!
//! ```text
//!   P(r_t = r_{t-1}+1 | x_{1..t}) ∝ P(r_{t-1} | x_{1..t-1}) ·
//!                                   UPM(x_t | x_{1..t-1}, r_{t-1}) ·
//!                                   (1 − H(r_{t-1}+1))
//!
//!   P(r_t = 0 | x_{1..t}) ∝ Σ P(r_{t-1} | x_{1..t-1}) ·
//!                              UPM(x_t | x_{1..t-1}, r_{t-1}) ·
//!                              H(r_{t-1}+1)
//! ```
//!
//! with a constant hazard `H(r) = 1/λ` (geometric run-length prior) and
//! a Student-t UPM derived from a Normal-Gamma conjugate prior on
//! `(μ, σ²)`.
//!
//! We report a change-point at time `t` when the *MAP run length*
//! (`argmax_r P(r_t | x_{1..t})`) drops by more than one step from the
//! previous time. This is the standard point-decision rule given in
//! §2.1 of Adams & MacKay (and is what the cited Figure 3 plots): a
//! MAP drop means the posterior has collapsed back toward `r = 0`,
//! signalling a regime change. Under a constant hazard, a fixed
//! `P(r_t = 0) > τ` rule is pinned near the hazard rate even on
//! obvious shifts — the MAP rule avoids that degeneracy.

use super::ChangePointDetector;
use std::f64::consts::PI;

pub struct Bocpd {
    /// Expected run length. `hazard = 1 / expected_run_length`. 100 is
    /// a common default in the BOCPD literature for traces of a few
    /// hundred samples; we keep it here so the crate's bake-off lines up
    /// with published benchmarks.
    pub expected_run_length: f64,
    /// Minimum drop in MAP run length that triggers a change-point.
    /// 1 would fire on every slight argmax jitter; 2 damps that without
    /// missing real regime changes.
    pub map_drop_min: usize,
    /// Normal-Gamma prior hyper-parameters. Weakly informative defaults
    /// match Adams & MacKay's Figure 3 setup.
    pub mu0: f64,
    pub kappa0: f64,
    pub alpha0: f64,
    pub beta0: f64,
}

impl Default for Bocpd {
    fn default() -> Self {
        Self {
            expected_run_length: 100.0,
            map_drop_min: 2,
            mu0: 0.0,
            kappa0: 1.0,
            alpha0: 1.0,
            beta0: 1.0,
        }
    }
}

impl ChangePointDetector for Bocpd {
    fn name(&self) -> &'static str {
        "bocpd"
    }

    fn detect(&self, series: &[(f64, f64)]) -> Vec<f64> {
        if series.len() < 2 {
            return Vec::new();
        }
        let hazard = 1.0 / self.expected_run_length.max(1.0);

        // Per-run-length sufficient statistics. mu[i], kappa[i], alpha[i],
        // beta[i] are the posterior Normal-Gamma parameters for the run
        // of length i.
        let mut mu = vec![self.mu0];
        let mut kappa = vec![self.kappa0];
        let mut alpha = vec![self.alpha0];
        let mut beta = vec![self.beta0];
        // Run-length posterior; starts with all mass at r=0.
        let mut run_posterior: Vec<f64> = vec![1.0];

        let mut cps = Vec::new();
        let mut prev_map: Option<usize> = None;
        for (t, &(ts, x)) in series.iter().enumerate() {
            // Predictive Student-t density at each extant run length.
            let mut pred = Vec::with_capacity(run_posterior.len());
            for i in 0..run_posterior.len() {
                pred.push(student_t_pdf(
                    x,
                    mu[i],
                    beta[i] * (kappa[i] + 1.0) / (alpha[i] * kappa[i]),
                    2.0 * alpha[i],
                ));
            }

            // Growth probabilities (no change).
            let mut new_post = vec![0.0; run_posterior.len() + 1];
            for i in 0..run_posterior.len() {
                new_post[i + 1] = run_posterior[i] * pred[i] * (1.0 - hazard);
            }
            // Change probability (collapse all mass to r=0).
            new_post[0] = run_posterior
                .iter()
                .zip(pred.iter())
                .map(|(r, p)| r * p * hazard)
                .sum();

            // Normalise — protects against numerical underflow over
            // long series.
            let z: f64 = new_post.iter().sum();
            if z > 0.0 {
                for p in &mut new_post {
                    *p /= z;
                }
            } else {
                new_post[0] = 1.0;
            }

            // Update sufficient statistics: extend every existing run by
            // 1, and prepend a fresh prior for the r=0 branch.
            let mut new_mu = vec![self.mu0];
            let mut new_kappa = vec![self.kappa0];
            let mut new_alpha = vec![self.alpha0];
            let mut new_beta = vec![self.beta0];
            for i in 0..run_posterior.len() {
                let k1 = kappa[i] + 1.0;
                new_mu.push((kappa[i] * mu[i] + x) / k1);
                new_kappa.push(k1);
                new_alpha.push(alpha[i] + 0.5);
                let diff = x - mu[i];
                new_beta.push(beta[i] + kappa[i] * diff * diff / (2.0 * k1));
            }

            mu = new_mu;
            kappa = new_kappa;
            alpha = new_alpha;
            beta = new_beta;
            run_posterior = new_post;

            // Decision. argmax run length is the MAP; a drop ≥
            // `map_drop_min` between consecutive steps flags a change.
            // Skip t=0 because MAP is trivially 0 at the boundary.
            let (map, _) = run_posterior
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, v)| (i, *v))
                .unwrap_or((0, 0.0));
            if t > 0 {
                if let Some(pm) = prev_map {
                    if pm >= self.map_drop_min && map + self.map_drop_min <= pm {
                        cps.push(ts);
                    }
                }
            }
            prev_map = Some(map);

            // Truncation: beyond a horizon the tail contributes negligible
            // posterior mass. 200 samples is standard in the literature
            // and keeps the inner loop O(min(t, 200)) per step.
            const MAX_RUN: usize = 200;
            if run_posterior.len() > MAX_RUN {
                run_posterior.truncate(MAX_RUN);
                mu.truncate(MAX_RUN);
                kappa.truncate(MAX_RUN);
                alpha.truncate(MAX_RUN);
                beta.truncate(MAX_RUN);
                let z: f64 = run_posterior.iter().sum();
                if z > 0.0 {
                    for p in &mut run_posterior {
                        *p /= z;
                    }
                }
            }
        }
        cps
    }
}

/// Student-t PDF at `x` with location `mu`, scale² `s2`, and `nu` df,
/// computed in log-space to avoid overflow under the tight posteriors
/// BOCPD develops over long runs of similar observations.
fn student_t_pdf(x: f64, mu: f64, s2: f64, nu: f64) -> f64 {
    let s = s2.max(f64::MIN_POSITIVE).sqrt();
    let z = (x - mu) / s;
    let log_num = ln_gamma((nu + 1.0) * 0.5) - ln_gamma(nu * 0.5);
    let log_den = 0.5 * (nu * PI).ln() + ((nu + 1.0) * 0.5) * (1.0 + z * z / nu).ln();
    (log_num - log_den).exp() / s
}

/// Lanczos `ln Γ(x)` approximation, g = 7, n = 9. Standard coefficients
/// (Numerical Recipes 3e §6.1). Accurate to ~15 digits for x > 0.5, and
/// we use the reflection formula below for the small-x tail. BOCPD's
/// updates only ever evaluate at α-type values that climb from the
/// prior's α₀ upward, so we stay in the accurate regime in practice.
fn ln_gamma(x: f64) -> f64 {
    const P: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885,
        -1_259.139_216_722_403,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_1,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_7e-7,
    ];
    const G: f64 = 7.0;
    if x < 0.5 {
        // Reflection: Γ(x)Γ(1−x) = π / sin(πx).
        (PI / (PI * x).sin()).ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = P[0];
        for (i, &p) in P.iter().enumerate().skip(1) {
            a += p / (x + i as f64);
        }
        let t = x + G + 0.5;
        0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_mean_shift() {
        let mut series = Vec::new();
        for i in 0..80 {
            series.push((i as f64, 0.0));
        }
        for i in 80..160 {
            series.push((i as f64, 5.0));
        }
        let cps = Bocpd::default().detect(&series);
        assert!(!cps.is_empty(), "BOCPD should flag a 5σ shift");
    }

    #[test]
    fn quiet_on_constant() {
        let series: Vec<(f64, f64)> = (0..100).map(|i| (i as f64, 0.0)).collect();
        let cps = Bocpd::default().detect(&series);
        // A completely constant series may still trigger at t=1 due to
        // the prior being updated, but should not emit a deluge.
        assert!(
            cps.len() <= 2,
            "BOCPD should be quiet on constants, got {cps:?}"
        );
    }
}
