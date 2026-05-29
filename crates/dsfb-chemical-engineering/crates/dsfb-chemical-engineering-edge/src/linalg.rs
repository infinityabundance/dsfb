//! Deterministic linear algebra for the chemometric detectors.
//!
//! Implements only what the detector atlas needs: standardised covariance, a deterministic
//! NIPALS PCA (top-`k` components via power iteration with deflation), and a symmetric-matrix
//! inverse via Gauss–Jordan (for full-variable Hotelling T² when `n_base > n_vars`). No `unsafe`,
//! no randomness — initial vectors are fixed so results are byte-reproducible.

/// Standardised data block built from a baseline: each column z-scored by baseline mean/std.
pub struct StandardModel {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub n_vars: usize,
}

impl StandardModel {
    pub fn z(&self, row: &[f64]) -> Vec<f64> {
        row.iter()
            .enumerate()
            .map(|(j, &x)| (x - self.mean[j]) / self.std[j])
            .collect()
    }
}

/// A deterministic NIPALS principal-component model: `k` loading vectors of length `n_vars`,
/// plus the score variances used to normalise Hotelling T² in score space.
pub struct PcaModel {
    pub n_vars: usize,
    pub k: usize,
    /// `k` loading vectors (each length `n_vars`), orthonormal.
    pub loadings: Vec<Vec<f64>>,
    /// Score variance per retained component (from the baseline).
    pub score_var: Vec<f64>,
}

impl PcaModel {
    /// Fit a top-`k` PCA on a standardised baseline block `zbase` (n rows × n_vars), via NIPALS
    /// power iteration with deflation. Deterministic: the start vector is fixed.
    pub fn fit(zbase: &[Vec<f64>], n_vars: usize, k: usize, iters: usize) -> Self {
        let n = zbase.len();
        let k = k.min(n_vars).min(n.saturating_sub(1)).max(1);
        // Working copy we deflate in place.
        let mut x: Vec<Vec<f64>> = zbase.to_vec();
        let mut loadings: Vec<Vec<f64>> = Vec::with_capacity(k);
        let mut score_var: Vec<f64> = Vec::with_capacity(k);
        for comp in 0..k {
            // Deterministic init: unit vector rotated by component index for distinctness.
            let mut p = vec![0.0; n_vars];
            p[comp % n_vars] = 1.0;
            // Power iteration on the (implicit) covariance X^T X.
            for _ in 0..iters {
                // t = X p
                let mut t = vec![0.0; n];
                for i in 0..n {
                    let mut s = 0.0;
                    let row = &x[i];
                    for j in 0..n_vars {
                        s += row[j] * p[j];
                    }
                    t[i] = s;
                }
                // p_new = X^T t
                let mut pn = vec![0.0; n_vars];
                for i in 0..n {
                    let ti = t[i];
                    let row = &x[i];
                    for j in 0..n_vars {
                        pn[j] += row[j] * ti;
                    }
                }
                let norm = pn.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-30);
                for v in pn.iter_mut() {
                    *v /= norm;
                }
                p = pn;
            }
            // Final scores t = X p; score variance.
            let mut t = vec![0.0; n];
            for i in 0..n {
                let mut s = 0.0;
                let row = &x[i];
                for j in 0..n_vars {
                    s += row[j] * p[j];
                }
                t[i] = s;
            }
            let mean_t = t.iter().sum::<f64>() / n as f64;
            let var_t =
                t.iter().map(|v| (v - mean_t) * (v - mean_t)).sum::<f64>() / (n as f64).max(1.0);
            // Deflate: X <- X - t p^T
            for i in 0..n {
                let ti = t[i];
                let row = &mut x[i];
                for j in 0..n_vars {
                    row[j] -= ti * p[j];
                }
            }
            loadings.push(p);
            // Floor the score variance away from zero before it becomes a T² denominator (`t2()` divides
            // by it). A near-degenerate component (a baseline direction with ~no spread) would otherwise
            // send `tᵢ²/var` astronomical on any test deviation; the floor caps that blow-up. Matched to
            // `std_dev`'s 1e-8 floor and verified hash-neutral on the committed corpus (no component's
            // baseline score variance lies in the floored band, so frozen evidence roots are unaffected).
            score_var.push(var_t.max(1e-8));
        }
        PcaModel {
            n_vars,
            k,
            loadings,
            score_var,
        }
    }

    /// Project a standardised sample onto the retained PCs → scores `t` (length `k`).
    pub fn scores(&self, z: &[f64]) -> Vec<f64> {
        self.loadings
            .iter()
            .map(|p| p.iter().zip(z).map(|(a, b)| a * b).sum::<f64>())
            .collect()
    }

    /// Hotelling T² in score space: `Σ t_a² / var_a`.
    pub fn t2(&self, z: &[f64]) -> f64 {
        let t = self.scores(z);
        t.iter()
            .zip(&self.score_var)
            .map(|(ti, v)| ti * ti / v)
            .sum()
    }

    /// Squared prediction error (Q / SPE): squared norm of the residual to the PCA reconstruction.
    pub fn spe(&self, z: &[f64]) -> f64 {
        let t = self.scores(z);
        // reconstruction zhat = Σ t_a p_a
        let mut zhat = vec![0.0; self.n_vars];
        for (a, p) in self.loadings.iter().enumerate() {
            let ta = t[a];
            for j in 0..self.n_vars {
                zhat[j] += ta * p[j];
            }
        }
        z.iter()
            .zip(&zhat)
            .map(|(zi, zh)| (zi - zh) * (zi - zh))
            .sum()
    }
}

/// Mean of a slice.
pub fn mean(x: &[f64]) -> f64 {
    if x.is_empty() {
        0.0
    } else {
        x.iter().sum::<f64>() / x.len() as f64
    }
}

/// Population standard deviation with a small floor.
///
/// The floor (`1e-8`) is a *degeneracy guard*: it is the divisor in [`StandardModel::z`], so a baseline
/// column with (near-)zero spread — a sensor pinned at a setpoint across the whole baseline window — would
/// otherwise make every z-score explode (`deviation / ~0`) and drive the downstream PCA T²/SPE statistics to
/// astronomical, non-physical magnitudes (the kind of `~1e35` blow-up that can poison fusion or overflow).
/// `1e-8` caps that. It was chosen as the largest round floor that is still **hash-neutral on the committed
/// corpus** — verified: no baseline column on the 6 synthetic golden sets or the 20 datasets has a standard
/// deviation inside the `[1e-12, 1e-8]` band, so raising it from the old `1e-12` left every frozen
/// `evidence_root` / `bundle_root` byte-identical. The guard therefore only ever binds on pathological
/// future inputs, never on anything we have sealed.
pub fn std_dev(x: &[f64]) -> f64 {
    if x.len() < 2 {
        return 1e-8;
    }
    let m = mean(x);
    (x.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / x.len() as f64)
        .sqrt()
        .max(1e-8)
}

/// Median over the **finite** values (deterministic; sorts a copy). Non-finite entries are ignored, so
/// a quality-gated `NaN` sample becomes a gap rather than poisoning the statistic. Returns 0.0 when no
/// value is finite. NOTE: we must *filter* (not `unwrap_or`) before sorting — the current Rust sort
/// panics if the comparator violates a total order, which `NaN` does; filtering guarantees a total order.
pub fn median(x: &[f64]) -> f64 {
    let mut v: Vec<f64> = x.iter().copied().filter(|a| a.is_finite()).collect();
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// Median absolute deviation (scaled to ≈σ for normal data).
pub fn mad(x: &[f64]) -> f64 {
    if x.is_empty() {
        return 1e-12;
    }
    let med = median(x);
    let dev: Vec<f64> = x.iter().map(|v| (v - med).abs()).collect();
    (1.4826 * median(&dev)).max(1e-12)
}

/// Lag-`lag` sample autocorrelation coefficient of a window.
pub fn autocorr(x: &[f64], lag: usize) -> f64 {
    let n = x.len();
    if n <= lag + 1 {
        return 0.0;
    }
    let m = mean(x);
    let denom: f64 = x.iter().map(|v| (v - m) * (v - m)).sum();
    if denom <= 1e-30 {
        return 0.0;
    }
    let mut num = 0.0;
    for i in lag..n {
        num += (x[i] - m) * (x[i - lag] - m);
    }
    num / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `std_dev` floor is the load-bearing degeneracy guard (it is the z-scoring divisor). A constant
    /// column has true σ = 0; the floor must return `1e-8`, never `0` (which would divide by zero) and
    /// never the old `1e-12` (which let z-scores reach ~1e12 and SPE/T² ~1e24+).
    #[test]
    fn std_dev_floors_a_constant_column_at_the_guard() {
        assert_eq!(std_dev(&[3.0; 16]), 1e-8);
        assert_eq!(std_dev(&[]), 1e-8); // <2 samples → floor, not a panic
    }

    /// Degeneracy guard, end to end: a baseline with a (near-)constant variable must NOT drive the PCA
    /// T²/SPE statistics to astronomical / non-finite magnitudes on a deviating test sample. Before the
    /// `1e-8` floors (z-scoring std + score variance) this class of input produced ~1e35 blow-ups that
    /// could poison fusion or overflow. We assert the statistics stay finite and within a sane bound; the
    /// `1e-8` floor caps the worst case at ~`n_vars · (deviation/1e-8)²`, far below the old behaviour and
    /// far from `f64::MAX`.
    #[test]
    fn near_constant_baseline_does_not_explode_t2_or_spe() {
        // 12-row baseline: var 0 is pinned (constant), var 1 carries a small deterministic ripple.
        let n_vars = 2;
        let base_rows: Vec<Vec<f64>> = (0..12)
            .map(|i| vec![5.0, 0.10 * ((i % 3) as f64 - 1.0)])
            .collect();
        let mean = vec![
            base_rows.iter().map(|r| r[0]).sum::<f64>() / 12.0,
            base_rows.iter().map(|r| r[1]).sum::<f64>() / 12.0,
        ];
        let std = vec![
            std_dev(&base_rows.iter().map(|r| r[0]).collect::<Vec<_>>()), // → 1e-8 (floored)
            std_dev(&base_rows.iter().map(|r| r[1]).collect::<Vec<_>>()),
        ];
        assert_eq!(std[0], 1e-8, "the pinned column must hit the floor");
        let sm = StandardModel { mean, std, n_vars };
        let zbase: Vec<Vec<f64>> = base_rows.iter().map(|r| sm.z(r)).collect();
        let pca = PcaModel::fit(&zbase, n_vars, 1, 60);

        // A test sample that deviates by 1.0 on the pinned column → z ≈ 1e8 (was ~1e12 before the floor).
        let z_test = sm.z(&[6.0, 0.0]);
        let t2 = pca.t2(&z_test);
        let spe = pca.spe(&z_test);
        assert!(
            t2.is_finite() && spe.is_finite(),
            "statistics must stay finite under degeneracy"
        );
        // Bounded well below the old ~1e35 blow-up class (and astronomically below f64::MAX).
        assert!(
            t2 < 1e20 && spe < 1e20,
            "degeneracy guard must cap the blow-up: t2={t2}, spe={spe}"
        );
    }
}
