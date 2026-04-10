//! Residual sign computation — the primary inferential object.
//!
//! The residual sign `σ_k = (r_k, d_k, s_k)` captures:
//! - `r_k`: structured deviation from nominal health behavior
//! - `d_k`: direction and persistence of degradation movement (drift)
//! - `s_k`: acceleration or curvature of the degradation trajectory (slew)
//!
//! All computation uses immutable input slices. No heap allocation.

use crate::core::config::DsfbConfig;

/// Residual sign at a single cycle: the typed triple `(residual, drift, slew)`.
#[derive(Debug, Clone, Copy)]
pub struct ResidualSign {
    /// The residual value: `r_k = y_k - ŷ_k`.
    pub residual: f64,
    /// First discrete difference (drift): direction and persistence.
    pub drift: f64,
    /// Second discrete difference (slew): acceleration / curvature.
    pub slew: f64,
    /// Cycle index (1-based).
    pub cycle: u32,
}

/// Computes the healthy-window baseline (mean and standard deviation)
/// from the first `config.healthy_window` values of a sensor channel.
///
/// # Arguments
/// - `values`: immutable slice of sensor readings for one channel, one engine.
/// - `config`: DSFB configuration (healthy_window length).
///
/// # Returns
/// `(mean, std_dev)` of the healthy window, or `(0.0, 1.0)` if insufficient data.
#[must_use]
pub fn compute_baseline(values: &[f64], config: &DsfbConfig) -> (f64, f64) {
    let n = values.len().min(config.healthy_window);
    if n == 0 {
        return (0.0, 1.0);
    }

    let mut sum = 0.0;
    let mut i = 0;
    while i < n {
        sum += values[i];
        i += 1;
    }
    let mean = sum / n as f64;

    let mut var_sum = 0.0;
    i = 0;
    while i < n {
        let d = values[i] - mean;
        var_sum += d * d;
        i += 1;
    }
    let std_dev = if n > 1 {
        libm_sqrt(var_sum / (n - 1) as f64)
    } else {
        1.0
    };

    // Guard against zero std_dev (constant signal)
    let std_dev = if std_dev < 1e-12 { 1.0 } else { std_dev };

    (mean, std_dev)
}

/// Computes the residual time series: `r_k = value_k - baseline_mean`.
///
/// Writes into `output` slice. Caller provides pre-allocated storage.
/// Returns the number of residuals written.
pub fn compute_residuals(
    values: &[f64],
    baseline_mean: f64,
    output: &mut [f64],
) -> usize {
    let n = values.len().min(output.len());
    let mut i = 0;
    while i < n {
        output[i] = values[i] - baseline_mean;
        i += 1;
    }
    n
}

/// Computes windowed drift (first discrete difference) for a residual series.
///
/// `drift_k = (1/W) * Σ_{i=0}^{W-1} (r_{k-i} - r_{k-i-1})`
///
/// Writes into `output`. Returns number of valid drift values.
pub fn compute_drift(
    residuals: &[f64],
    window: usize,
    output: &mut [f64],
) -> usize {
    let n = residuals.len();
    if n < 2 || window == 0 {
        return 0;
    }
    let w = window.min(n - 1);

    // First (w) values: not enough history, set to 0.0
    let mut i = 0;
    while i < w {
        output[i] = 0.0;
        i += 1;
    }

    // Windowed drift
    while i < n {
        let mut sum = 0.0;
        let mut j = 0;
        while j < w {
            let idx = i - j;
            if idx > 0 {
                sum += residuals[idx] - residuals[idx - 1];
            }
            j += 1;
        }
        output[i] = sum / w as f64;
        i += 1;
    }
    n
}

/// Computes windowed slew (second discrete difference) for a drift series.
///
/// `slew_k = (1/W) * Σ_{i=0}^{W-1} (drift_{k-i} - drift_{k-i-1})`
///
/// Writes into `output`. Returns number of valid slew values.
pub fn compute_slew(
    drift: &[f64],
    window: usize,
    output: &mut [f64],
) -> usize {
    // Slew is just drift-of-drift.
    compute_drift(drift, window, output)
}

/// Assembles a `ResidualSign` at cycle `k` from pre-computed arrays.
#[must_use]
pub fn sign_at(
    residuals: &[f64],
    drift: &[f64],
    slew: &[f64],
    k: usize,
    cycle_offset: u32,
) -> ResidualSign {
    ResidualSign {
        residual: if k < residuals.len() { residuals[k] } else { 0.0 },
        drift: if k < drift.len() { drift[k] } else { 0.0 },
        slew: if k < slew.len() { slew[k] } else { 0.0 },
        cycle: cycle_offset + k as u32,
    }
}

/// Pure `no_std` square root using Newton's method.
/// Avoids dependency on `libm` crate for minimal footprint.
#[must_use]
fn libm_sqrt(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut guess = x;
    let mut i = 0;
    while i < 50 {
        let next = 0.5 * (guess + x / guess);
        if (next - guess).abs() < 1e-15 {
            return next;
        }
        guess = next;
        i += 1;
    }
    guess
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_computation() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let config = DsfbConfig { healthy_window: 5, ..DsfbConfig::default() };
        let (mean, std) = compute_baseline(&values, &config);
        assert!((mean - 3.0).abs() < 1e-10);
        assert!(std > 0.0);
    }

    #[test]
    fn test_residual_computation() {
        let values = [10.0, 11.0, 12.0, 13.0];
        let mut output = [0.0; 4];
        let n = compute_residuals(&values, 10.0, &mut output);
        assert_eq!(n, 4);
        assert!((output[0] - 0.0).abs() < 1e-10);
        assert!((output[3] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_drift_monotone_signal() {
        // Linear increase: residuals = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        let residuals: [f64; 10] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let mut drift = [0.0; 10];
        compute_drift(&residuals, 3, &mut drift);
        // After warmup, drift should be ~1.0 (constant slope)
        assert!((drift[9] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_slew_zero_for_linear() {
        // Linear signal: drift is constant, slew should be ~0
        let residuals: [f64; 20] = {
            let mut arr = [0.0; 20];
            let mut i = 0;
            while i < 20 { arr[i] = i as f64; i += 1; }
            arr
        };
        let mut drift = [0.0; 20];
        let mut slew = [0.0; 20];
        compute_drift(&residuals, 5, &mut drift);
        compute_slew(&drift, 5, &mut slew);
        // Slew at end should be near zero for linear signal
        assert!(slew[19].abs() < 0.1);
    }

    #[test]
    fn test_sqrt() {
        assert!((libm_sqrt(4.0) - 2.0).abs() < 1e-10);
        assert!((libm_sqrt(9.0) - 3.0).abs() < 1e-10);
        assert!((libm_sqrt(0.0)).abs() < 1e-10);
    }
}
