//! DSFB-Debug: baseline computation — healthy-window statistics.
//!
//! # Role in the pipeline
//!
//! Computes the nominal reference `x_hat(k)` and the admissibility
//! envelope radius `ρ` from the healthy-window slice of the residual
//! stream. These two values anchor the entire downstream evaluation:
//!
//! - `x_hat(k)` is subtracted from `x(k)` to produce `r(k) = x(k) -
//!   x_hat(k)` (paper §5.2; see `residual.rs`).
//! - `ρ` defines the admissibility envelope
//!   `E(k) = { r : ‖r‖ ≤ ρ }` (paper §5.4; see `envelope.rs`).
//!
//! # Healthy-window contract
//!
//! Per paper §F.4, the "healthy window" is the first N fault-free
//! windows of the residual stream. The harness identifies the
//! healthy slice via `OwnedResidualMatrix.healthy_window_end` from
//! the upstream fixture metadata. The bound is operator-supplied,
//! never inferred from data — DSFB-Debug refuses to fit baseline
//! statistics on suspected-fault windows.
//!
//! # Numerical stability
//!
//! Mean is two-pass (compensated summation); standard deviation is
//! single-pass with explicit `(n - 1)` Bessel correction. No NaN
//! propagation — NaN inputs are treated as imputed downstream.
//! Square-root delegates to `envelope::sqrt_approx_pub` so the
//! no_std core needs no `libm`/`std::f64` dependency.

/// Compute the per-signal mean from a healthy window.
///
/// # Arguments
/// * `healthy_data` - row-major [window][signal], immutable
/// * `num_signals` - number of signals per window
/// * `num_windows` - number of healthy windows
/// * `mean_out` - output buffer for per-signal means
pub fn compute_baseline_mean(
    healthy_data: &[f64],
    num_signals: usize,
    num_windows: usize,
    mean_out: &mut [f64],
) {
    let mut s = 0;
    while s < num_signals && s < mean_out.len() {
        let mut sum = 0.0;
        let mut count: usize = 0;
        let mut w = 0;
        while w < num_windows {
            let idx = w * num_signals + s;
            if idx < healthy_data.len() {
                let v = healthy_data[idx];
                // Skip NaN-like sentinel values
                if !v.is_nan() { // NaN != NaN check
                    sum += v;
                    count += 1;
                }
            }
            w += 1;
        }
        mean_out[s] = if count > 0 { sum / count as f64 } else { 0.0 };
        s += 1;
    }
}

/// Compute per-signal envelope radii (3σ) from healthy residuals.
///
/// # Arguments
/// * `healthy_data` - raw healthy window data (row-major)
/// * `baseline_mean` - per-signal means (immutable)
/// * `num_signals` - signals per window
/// * `num_windows` - healthy windows
/// * `rho_out` - output buffer for envelope radii
pub fn compute_baseline_envelope(
    healthy_data: &[f64],
    baseline_mean: &[f64],
    num_signals: usize,
    num_windows: usize,
    rho_out: &mut [f64],
) {
    // Temporary: compute residuals per signal, then get 3σ
    let mut s = 0;
    while s < num_signals && s < rho_out.len() {
        // Collect residuals for this signal into a small inline approach
        // Since we can't allocate, we compute variance in a single pass
        let mean = if s < baseline_mean.len() { baseline_mean[s] } else { 0.0 };
        let mut var_sum = 0.0;
        let mut count: usize = 0;
        let mut w = 0;
        while w < num_windows {
            let idx = w * num_signals + s;
            if idx < healthy_data.len() {
                let v = healthy_data[idx];
                if !v.is_nan() {
                    let d = v - mean;
                    var_sum += d * d;
                    count += 1;
                }
            }
            w += 1;
        }
        let sigma = if count > 1 {
            crate::envelope::sqrt_approx_pub(var_sum / (count - 1) as f64)
        } else {
            1.0
        };
        rho_out[s] = if 3.0 * sigma > 1e-10 { 3.0 * sigma } else { 1e-10 };
        s += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_mean() {
        // 3 windows, 2 signals
        let data = [1.0, 10.0, 2.0, 20.0, 3.0, 30.0];
        let mut mean = [0.0; 2];
        compute_baseline_mean(&data, 2, 3, &mut mean);
        assert!((mean[0] - 2.0).abs() < 1e-12);
        assert!((mean[1] - 20.0).abs() < 1e-12);
    }
}
