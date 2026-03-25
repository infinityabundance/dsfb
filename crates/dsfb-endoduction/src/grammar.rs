//! Structural grammar over residuals: motif detectors.
//!
//! Each detector operates on a single window's residual to produce a
//! scalar characterising one structural motif. These are the building
//! blocks of the DSFB trust / precursor score.
//!
//! Motifs implemented:
//! - drift: linear trend magnitude
//! - slew: maximum rate of change
//! - persistence: fraction of sign-consistent runs
//! - variance growth: ratio of local variance to baseline
//! - autocorrelation growth: shift in lag-1 autocorrelation
//! - spectral redistribution: shift in spectral centroid
//! - envelope breach density: fraction of residual outside envelope

use crate::baseline;

/// Drift: slope of a linear fit to the residual across the window.
///
/// A large absolute drift indicates the residual is systematically
/// increasing or decreasing, consistent with a slowly evolving
/// departure from nominal.
pub fn drift(residual: &[f64]) -> f64 {
    let n = residual.len();
    if n < 2 {
        return 0.0;
    }
    // Least-squares slope: sum((i - i_mean)(r_i - r_mean)) / sum((i - i_mean)^2)
    let n_f = n as f64;
    let i_mean = (n_f - 1.0) / 2.0;
    let r_mean = baseline::mean(residual);
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &r) in residual.iter().enumerate() {
        let di = i as f64 - i_mean;
        num += di * (r - r_mean);
        den += di * di;
    }
    if den.abs() < 1e-30 {
        return 0.0;
    }
    num / den
}

/// Slew: maximum absolute first-difference in the residual.
///
/// A large slew indicates abrupt jumps or transients that depart
/// from smooth nominal variability.
pub fn slew(residual: &[f64]) -> f64 {
    if residual.len() < 2 {
        return 0.0;
    }
    residual
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0_f64, f64::max)
}

/// Persistence: fraction of samples in the longest sign-consistent run.
///
/// For a nominal, roughly symmetric residual, runs of consistent sign
/// should be short. Long runs indicate persistent bias (critical
/// slowing down signature).
pub fn persistence(residual: &[f64]) -> f64 {
    let n = residual.len();
    if n == 0 {
        return 0.0;
    }
    let mut max_run = 0usize;
    let mut current_run = 1usize;
    for i in 1..n {
        if (residual[i] >= 0.0) == (residual[i - 1] >= 0.0) {
            current_run += 1;
        } else {
            max_run = max_run.max(current_run);
            current_run = 1;
        }
    }
    max_run = max_run.max(current_run);
    max_run as f64 / n as f64
}

/// Variance growth: ratio of this window's residual variance to the
/// baseline nominal variance.
///
/// Values > 1 indicate increased fluctuation amplitude relative to
/// the nominal regime.
pub fn variance_growth(residual: &[f64], baseline_variance: f64) -> f64 {
    if baseline_variance < 1e-30 {
        return 0.0;
    }
    baseline::variance(residual) / baseline_variance
}

/// Autocorrelation growth: difference between this window's lag-1
/// autocorrelation and the nominal baseline autocorrelation.
///
/// Positive values indicate increased persistence / critical slowing.
pub fn autocorrelation_growth(residual: &[f64], baseline_autocorr: f64) -> f64 {
    baseline::lag1_autocorrelation(residual) - baseline_autocorr
}

/// Spectral centroid shift relative to nominal.
///
/// A decrease suggests energy moving to lower frequencies (critical slowing).
/// An increase suggests broadening or high-frequency activity emergence.
pub fn spectral_centroid_shift(residual: &[f64], baseline_centroid: f64) -> f64 {
    baseline::spectral_centroid(residual) - baseline_centroid
}
