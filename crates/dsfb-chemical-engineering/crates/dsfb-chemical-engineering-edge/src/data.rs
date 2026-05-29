//! Data container and baseline model shared by all detectors.
//!
//! A `DataMatrix` is a row-major `n_samples × n_vars` block of `f64`. Detectors are *fit* on a
//! normal-operating-condition baseline window and then *score* every sample, emitting a scalar
//! monitoring statistic per time step. All operations are deterministic.

use serde::{Deserialize, Serialize};

/// Row-major dense matrix of process measurements: `n_samples` rows × `n_vars` columns.
#[derive(Debug, Clone)]
pub struct DataMatrix {
    pub n_samples: usize,
    pub n_vars: usize,
    pub var_names: Vec<String>,
    /// Row-major: element `(i, j)` at `data[i * n_vars + j]`.
    pub data: Vec<f64>,
    /// Optional per-sample fault label (`0` = nominal/unknown, `>0` = fault class id).
    pub labels: Option<Vec<u32>>,
    /// Sample index at which the labelled fault is introduced (if known), for lead-time metrics.
    pub fault_onset: Option<usize>,
    /// Optional per-sample batch-phase id, for phase-aligned envelopes.
    pub phase: Option<Vec<u32>>,
}

impl DataMatrix {
    /// Build a row-major matrix from a list of equal-length rows.
    ///
    /// # Contract / panics
    /// - **Rectangularity is required:** every row must have the same length, and that length defines
    ///   `n_vars` (taken from `rows[0]` when non-empty, else from `var_names.len()` for the 0-row case).
    ///   A ragged row triggers `assert_eq!(... "ragged rows in DataMatrix")` — this is a caller bug
    ///   (the loaders in `datasets.rs` always emit rectangular rows), not a recoverable input error.
    /// - **Empty / single-row matrices are accepted here** (no panic): a 0- or 1-row matrix is a valid
    ///   `DataMatrix`. The *pipeline* (`pipeline::analyze`) is what requires `n_samples >= 2` for a
    ///   meaningful baseline; it guards that degenerate case explicitly rather than relying on this
    ///   constructor to reject it.
    pub fn new(var_names: Vec<String>, rows: Vec<Vec<f64>>) -> Self {
        let n_samples = rows.len();
        let n_vars = if n_samples > 0 {
            rows[0].len()
        } else {
            var_names.len()
        };
        let mut data = Vec::with_capacity(n_samples * n_vars);
        for row in &rows {
            assert_eq!(row.len(), n_vars, "ragged rows in DataMatrix");
            data.extend_from_slice(row);
        }
        DataMatrix {
            n_samples,
            n_vars,
            var_names,
            data,
            labels: None,
            fault_onset: None,
            phase: None,
        }
    }

    #[inline]
    pub fn row(&self, i: usize) -> &[f64] {
        &self.data[i * self.n_vars..(i + 1) * self.n_vars]
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.n_vars + j]
    }

    /// Column `j` as a freshly-allocated vector.
    pub fn column(&self, j: usize) -> Vec<f64> {
        (0..self.n_samples).map(|i| self.get(i, j)).collect()
    }

    /// A baseline view = the first `n` rows (normal operating condition window).
    pub fn baseline_rows(&self, n: usize) -> usize {
        n.min(self.n_samples)
    }
}

/// First/second-moment baseline + standardisation parameters from a normal window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub n: usize,
    pub n_vars: usize,
}

impl Baseline {
    /// Fit per-variable mean and (population) std over the first `n_base` rows.
    pub fn fit(m: &DataMatrix, n_base: usize) -> Self {
        let n = m.baseline_rows(n_base).max(1);
        let v = m.n_vars;
        let mut mean = vec![0.0; v];
        for i in 0..n {
            for (j, mj) in mean.iter_mut().enumerate() {
                *mj += m.get(i, j);
            }
        }
        mean.iter_mut().for_each(|x| *x /= n as f64);
        let mut var = vec![0.0; v];
        for i in 0..n {
            for j in 0..v {
                let d = m.get(i, j) - mean[j];
                var[j] += d * d;
            }
        }
        // Population standard deviation per channel, with a *degeneracy guard*.
        //
        // A baseline channel with (near-)zero variance carries no usable z-score: dividing a post-baseline
        // deviation by a ~1e-12 std produces absurd standardised values — this is exactly what made the
        // CSTR SPE/Q residual explode to ~1.4e35 (a constant baseline column standardised a post-fault
        // deviation of order 1 into z ≈ 1e12, then SPE = Σz² blew up). The fix: when a channel is
        // *effectively constant over the baseline* — its std is below a small **relative** threshold of its
        // own magnitude (constant to ~9 significant figures, i.e. below sensor resolution) — fall back to
        // its **raw deviation** (std = 1.0). That bounds the standardised residual to a physical magnitude.
        //
        // Crucially this is *targeted*: every well-conditioned channel (std above the relative threshold) is
        // byte-identical to before, so non-degenerate datasets' sealed roots are unchanged; only genuinely
        // degenerate datasets (a constant baseline column) shift — those are re-minted as a governed change.
        let std: Vec<f64> = var
            .iter()
            .enumerate()
            .map(|(j, s)| {
                let raw = (s / n as f64).sqrt();
                let rel_floor = 1e-9 * (1.0 + mean[j].abs());
                if raw < rel_floor {
                    1.0 // effectively-constant baseline channel → raw deviation, no amplification
                } else {
                    raw
                }
            })
            .collect();
        Baseline {
            mean,
            std,
            n,
            n_vars: v,
        }
    }

    /// Standardise (z-score) a single sample row against the baseline.
    pub fn standardise(&self, row: &[f64]) -> Vec<f64> {
        row.iter()
            .enumerate()
            .map(|(j, &x)| (x - self.mean[j]) / self.std[j])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for the PCA/standardisation degeneracy class (the CSTR SPE/Q ~1.4e35 explosion).
    ///
    /// Column 0 is **constant** across the baseline window (a dead / setpoint-held channel); column 1 has
    /// genuine variation. After the baseline, both columns take a deviation of order 1. Without the
    /// degeneracy guard, column 0's std would floor at 1e-12 and standardise its post-baseline deviation to
    /// z ≈ 1e12 (and SPE = Σz² would explode). With the guard, the constant channel falls back to raw
    /// deviation (std = 1.0), so |z| stays of order 1; the well-conditioned channel is untouched.
    #[test]
    fn constant_baseline_channel_does_not_explode_the_zscore() {
        let n_base = 16;
        let mut rows: Vec<Vec<f64>> = Vec::new();
        for i in 0..n_base {
            // col0 constant at 5.0; col1 a real sawtooth around 0.
            rows.push(vec![5.0, ((i % 4) as f64) - 1.5]);
        }
        // A post-baseline sample deviating by ~1 on both columns.
        rows.push(vec![6.0, 0.5]);
        let m = DataMatrix::new(vec!["dead".into(), "live".into()], rows);
        let b = Baseline::fit(&m, n_base);

        // Constant channel → guard fires → std == 1.0 (raw deviation, no amplification).
        assert_eq!(
            b.std[0], 1.0,
            "constant baseline channel must fall back to raw deviation"
        );
        // Well-conditioned channel → std is its genuine (non-degenerate) value, unchanged by the guard.
        assert!(
            b.std[1] > 1e-3 && b.std[1].is_finite(),
            "live channel keeps its real std: {}",
            b.std[1]
        );

        let z = b.standardise(&[6.0, 0.5]);
        assert!(
            z[0].abs() < 10.0,
            "constant-channel z must be bounded (was {}), not ~1e12",
            z[0]
        );
        assert!(
            z.iter().all(|v| v.is_finite()),
            "all standardised values finite"
        );
    }

    /// The guard must be *targeted*: a channel with small-but-real variation is left byte-identical (its
    /// std is its true value, not the raw-deviation fallback), so non-degenerate datasets are unaffected.
    #[test]
    fn small_but_real_variation_is_preserved() {
        let n_base = 32;
        let rows: Vec<Vec<f64>> = (0..n_base)
            .map(|i| vec![1.0 + 1e-3 * ((i % 8) as f64 - 3.5)])
            .collect();
        let m = DataMatrix::new(vec!["x".into()], rows);
        let b = Baseline::fit(&m, n_base);
        // std is ~1e-3 (real, above the 1e-9 relative floor) → NOT replaced by the fallback 1.0.
        assert!(
            b.std[0] > 1e-4 && b.std[0] < 1e-2,
            "small real std preserved: {}",
            b.std[0]
        );
        assert_ne!(b.std[0], 1.0);
    }
}
