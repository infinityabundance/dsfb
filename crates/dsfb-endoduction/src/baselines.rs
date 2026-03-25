//! Classical diagnostic baselines for comparison.
//!
//! These implement conventional scalar metrics applied to the raw signal
//! so that DSFB structural residual analysis can be compared against
//! standard approaches on equal footing.

use crate::baseline;

/// Compute all classical baseline metrics for one window of raw signal.
#[derive(Debug, Clone)]
pub struct ClassicalMetrics {
    pub rms: f64,
    pub kurtosis: f64,
    pub crest_factor: f64,
    pub rolling_variance: f64,
    pub lag1_autocorrelation: f64,
    pub spectral_band_energy: f64,
    pub spectral_centroid: f64,
}

/// Compute classical metrics from a raw signal window.
pub fn compute_classical(signal: &[f64]) -> ClassicalMetrics {
    ClassicalMetrics {
        rms: baseline::rms(signal),
        kurtosis: baseline::kurtosis(signal),
        crest_factor: baseline::crest_factor(signal),
        rolling_variance: baseline::variance(signal),
        lag1_autocorrelation: baseline::lag1_autocorrelation(signal),
        spectral_band_energy: baseline::spectral_band_energy(signal),
        spectral_centroid: baseline::spectral_centroid(signal),
    }
}

/// Determine whether a classical metric exceeds its baseline threshold.
///
/// Uses a simple rule: metric > mean + k * std.
pub fn exceeds_threshold(value: f64, mean: f64, std: f64, k: f64) -> bool {
    value > mean + k * std
}

/// First sustained detection: index of the first window where the condition
/// is true for `sustained` consecutive windows.
pub fn first_sustained_detection(flags: &[bool], sustained: usize) -> Option<usize> {
    if sustained == 0 || flags.is_empty() {
        return None;
    }
    let mut run = 0usize;
    for (i, &f) in flags.iter().enumerate() {
        if f {
            run += 1;
            if run >= sustained {
                return Some(i + 1 - sustained);
            }
        } else {
            run = 0;
        }
    }
    None
}
