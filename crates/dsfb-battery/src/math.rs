// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// DSFB Battery Health Monitoring — Mathematical core
//
// All functions correspond to named equations, definitions, or theorems in:
//   "DSFB Structural Semiotics Engine for Battery Health Monitoring"
//   by Riaan de Beer, Version 1.0.

use alloc::vec;
use alloc::vec::Vec;
use crate::types::EnvelopeParams;
use thiserror::Error;

/// Errors arising from math operations on battery residual data.
#[derive(Debug, Error)]
pub enum MathError {
    #[error("insufficient data: need at least {need} points, got {got}")]
    InsufficientData { need: usize, got: usize },
    #[error("window size {window} exceeds available data length {len}")]
    WindowTooLarge { window: usize, len: usize },
    #[error("zero standard deviation in healthy window — envelope undefined")]
    ZeroStdDev,
    #[error("net drift η − κ must be positive for Theorem 1 (got η={eta}, κ={kappa})")]
    NonPositiveNetDrift { eta: f64, kappa: f64 },
}

/// Compute residual at cycle k.
///
/// **Definition 1 (Paper):** r_k = y_k − ŷ_k
///
/// Here ŷ_k is the nominal prediction (healthy-window mean μ), so:
///   r_k = capacity_k − μ
///
/// A negative residual indicates capacity below nominal (degradation).
pub fn compute_residual(capacity: f64, nominal: f64) -> f64 {
    capacity - nominal
}

/// Compute windowed drift at index `k` over the residual sequence.
///
/// **Paper drift estimator:**
///   drift_k = (1/W) Σ_{i=0}^{W−1} (r_{k−i} − r_{k−i−1})
///
/// This simplifies to:
///   drift_k = (r_k − r_{k−W}) / W
///
/// Returns the average per-cycle change in residual over the window.
/// Units: Ah/cycle.
pub fn compute_drift(residuals: &[f64], k: usize, window: usize) -> Result<f64, MathError> {
    if window == 0 {
        return Err(MathError::InsufficientData { need: 1, got: 0 });
    }
    if k < window {
        return Err(MathError::WindowTooLarge { window, len: k });
    }
    if k >= residuals.len() {
        return Err(MathError::InsufficientData {
            need: k + 1,
            got: residuals.len(),
        });
    }
    // Telescoping sum: (1/W) Σ (r_{k-i} - r_{k-i-1}) = (r_k - r_{k-W}) / W
    Ok((residuals[k] - residuals[k - window]) / window as f64)
}

/// Compute windowed slew at index `k` over the drift sequence.
///
/// **Paper slew estimator:**
///   slew_k = (1/W) Σ_{i=0}^{W−1} (drift_{k−i} − drift_{k−i−1})
///
/// This simplifies to:
///   slew_k = (drift_k − drift_{k−W}) / W
///
/// Units: Ah/cycle².
pub fn compute_slew(drifts: &[f64], k: usize, window: usize) -> Result<f64, MathError> {
    if window == 0 {
        return Err(MathError::InsufficientData { need: 1, got: 0 });
    }
    if k < window {
        return Err(MathError::WindowTooLarge { window, len: k });
    }
    if k >= drifts.len() {
        return Err(MathError::InsufficientData {
            need: k + 1,
            got: drifts.len(),
        });
    }
    Ok((drifts[k] - drifts[k - window]) / window as f64)
}

/// Compute admissibility envelope from the healthy baseline window.
///
/// **Definition 3 (Paper), Stage II formula:**
///   μ_y^(0) = (1/N_0) Σ_{k=1}^{N_0} y_k
///   σ_y^(0) = sqrt( (1/(N_0−1)) Σ_{k=1}^{N_0} (y_k − μ_y^(0))² )
///   ρ_y = 3 σ_y^(0)
///   Admissible iff |r_k^(y)| ≤ ρ_y
///
/// `healthy_data` is the slice of capacity values over the first N_h cycles.
pub fn compute_envelope(healthy_data: &[f64]) -> Result<EnvelopeParams, MathError> {
    let n = healthy_data.len();
    if n < 2 {
        return Err(MathError::InsufficientData { need: 2, got: n });
    }

    // μ_y^(0) = (1/N_0) Σ y_k
    let mu: f64 = healthy_data.iter().sum::<f64>() / n as f64;

    // σ_y^(0) = sqrt( (1/(N_0−1)) Σ (y_k − μ)² )
    let variance: f64 = healthy_data.iter().map(|y| (y - mu).powi(2)).sum::<f64>() / (n - 1) as f64;
    let sigma = variance.sqrt();

    if sigma < f64::EPSILON {
        return Err(MathError::ZeroStdDev);
    }

    // ρ_y = 3 σ_y^(0)
    let rho = 3.0 * sigma;

    Ok(EnvelopeParams { mu, sigma, rho })
}

/// Compute the Theorem 1 exit bound.
///
/// **Theorem 1 (Paper): Discrete-Time Finite Envelope Exit Under Sustained
/// Outward Drift.**
///
/// Given:
///   g_{k_0} = initial admissibility gap = ρ − |r_{k_0}|
///   η = minimum outward drift per cycle (sustained)
///   κ = maximum envelope expansion per cycle (for static envelope, κ = 0)
///
/// Then the first envelope exit time satisfies:
///   k* − k_0 ≤ ⌈ g_{k_0} / (η − κ) ⌉
///
/// For the simplified Stage II case with static envelope (κ = 0):
///   k* − k_0 ≤ ⌈ ρ / η ⌉
///
/// `initial_gap` is g_{k_0} (the initial margin between residual magnitude
/// and envelope radius).
/// `min_drift` is η (minimum outward drift per cycle).
/// `max_envelope_expansion` is κ (set to 0.0 for static envelopes).
pub fn theorem1_exit_bound(
    initial_gap: f64,
    min_drift: f64,
    max_envelope_expansion: f64,
) -> Result<usize, MathError> {
    let net_drift = min_drift - max_envelope_expansion;
    if net_drift <= 0.0 {
        return Err(MathError::NonPositiveNetDrift {
            eta: min_drift,
            kappa: max_envelope_expansion,
        });
    }
    // ⌈ g_{k_0} / (η − κ) ⌉
    let bound = (initial_gap / net_drift).ceil() as usize;
    Ok(bound)
}

/// Compute all residuals from a capacity sequence and a nominal value.
///
/// **Definition 1 (Paper):** r_k = y_k − ŷ_k for all k.
///
/// Returns a vector of residuals with the same length as `capacities`.
pub fn compute_all_residuals(capacities: &[f64], nominal: f64) -> Vec<f64> {
    capacities
        .iter()
        .map(|c| compute_residual(*c, nominal))
        .collect()
}

/// Compute all drifts from a residual sequence.
///
/// **Paper drift estimator** applied at every valid index.
///
/// Returns a vector of length `residuals.len()`. Indices where drift cannot
/// be computed (k < window) are filled with 0.0.
pub fn compute_all_drifts(residuals: &[f64], window: usize) -> Vec<f64> {
    let mut drifts = vec![0.0; residuals.len()];
    for k in window..residuals.len() {
        if let Ok(d) = compute_drift(residuals, k, window) {
            drifts[k] = d;
        }
    }
    drifts
}

/// Compute all slews from a drift sequence.
///
/// **Paper slew estimator** applied at every valid index.
///
/// Returns a vector of length `drifts.len()`. Indices where slew cannot
/// be computed (k < window) are filled with 0.0.
pub fn compute_all_slews(drifts: &[f64], window: usize) -> Vec<f64> {
    let mut slews = vec![0.0; drifts.len()];
    for k in window..drifts.len() {
        if let Ok(s) = compute_slew(drifts, k, window) {
            slews[k] = s;
        }
    }
    slews
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_residual() {
        let r = compute_residual(1.80, 1.85);
        assert!((r - (-0.05)).abs() < 1e-10);
    }

    #[test]
    fn test_compute_drift_telescoping() {
        // residuals: [0.0, -0.01, -0.02, -0.03, -0.04, -0.05]
        // drift at k=5 with W=5: (r_5 - r_0) / 5 = (-0.05 - 0.0) / 5 = -0.01
        let residuals = vec![0.0, -0.01, -0.02, -0.03, -0.04, -0.05];
        let d = compute_drift(&residuals, 5, 5).unwrap();
        assert!((d - (-0.01)).abs() < 1e-10);
    }

    #[test]
    fn test_compute_envelope_basic() {
        // Data with known mean and std
        let data = vec![2.0, 2.01, 1.99, 2.0, 2.02, 1.98, 2.0, 2.01, 1.99, 2.0];
        let env = compute_envelope(&data).unwrap();
        assert!((env.mu - 2.0).abs() < 0.01);
        assert!(env.rho > 0.0);
        assert!((env.rho - 3.0 * env.sigma).abs() < 1e-10);
    }

    #[test]
    fn test_theorem1_bound() {
        // ρ = 0.03, η = 0.005, κ = 0.0
        // ⌈ 0.03 / 0.005 ⌉ = 6
        let bound = theorem1_exit_bound(0.03, 0.005, 0.0).unwrap();
        assert_eq!(bound, 6);
    }

    #[test]
    fn test_theorem1_rejects_non_positive_net_drift() {
        let result = theorem1_exit_bound(0.03, 0.001, 0.002);
        assert!(result.is_err());
    }
}
