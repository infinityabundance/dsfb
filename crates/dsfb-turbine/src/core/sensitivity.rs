//! Parameter sensitivity analysis.
//!
//! Varies each configuration parameter independently to measure
//! grammar-transition stability. Reports how the first Boundary
//! and first Violation cycles change with configuration.

use crate::core::config::DsfbConfig;

/// A single sensitivity sweep point.
#[derive(Debug, Clone, Copy)]
pub struct SensitivityPoint {
    /// Parameter name (static str for no_alloc).
    pub parameter: &'static str,
    /// Parameter value at this sweep point.
    pub value: f64,
    /// First Boundary cycle under this configuration (None if never reached).
    pub first_boundary: Option<u32>,
    /// First Violation cycle under this configuration (None if never reached).
    pub first_violation: Option<u32>,
}

/// Maximum number of sweep points per parameter.
pub const MAX_SWEEP_POINTS: usize = 11;

/// Generates sweep values for a parameter.
///
/// Returns an array of `count` values linearly spaced between `min` and `max`.
/// `count` is capped at `MAX_SWEEP_POINTS`.
#[must_use]
pub fn sweep_values(min: f64, max: f64, count: usize) -> ([f64; MAX_SWEEP_POINTS], usize) {
    let n = count.min(MAX_SWEEP_POINTS);
    let mut values = [0.0; MAX_SWEEP_POINTS];
    if n <= 1 {
        values[0] = min;
        return (values, 1);
    }
    let step = (max - min) / (n - 1) as f64;
    let mut i = 0;
    while i < n {
        values[i] = min + step * i as f64;
        i += 1;
    }
    (values, n)
}

/// Generates a modified config with one parameter changed.
#[must_use]
pub fn config_with_healthy_window(base: &DsfbConfig, hw: usize) -> DsfbConfig {
    DsfbConfig { healthy_window: hw, ..*base }
}

/// Generates a modified config with drift_window changed.
#[must_use]
pub fn config_with_drift_window(base: &DsfbConfig, dw: usize) -> DsfbConfig {
    DsfbConfig { drift_window: dw, ..*base }
}

/// Generates a modified config with persistence_threshold changed.
#[must_use]
pub fn config_with_persistence(base: &DsfbConfig, pt: usize) -> DsfbConfig {
    DsfbConfig { persistence_threshold: pt, ..*base }
}

/// Generates a modified config with envelope_sigma changed.
#[must_use]
pub fn config_with_envelope_sigma(base: &DsfbConfig, sigma: f64) -> DsfbConfig {
    DsfbConfig { envelope_sigma: sigma, ..*base }
}
