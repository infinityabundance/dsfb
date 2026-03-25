//! Admissibility envelope: regime-consistency characterisation.
//!
//! The admissibility envelope E_R defines the set of residual behaviours
//! expected under the nominal regime R. A residual sample is "admissible"
//! if it falls within the envelope; breaches indicate departure from
//! nominal dynamics.
//!
//! We estimate the envelope from the nominal window as a per-sample
//! tolerance band: mean_wf ± k * sqrt(waveform_variance), where k is
//! chosen from the configured quantile level.

use crate::baseline::NominalBaseline;

/// Per-sample envelope bounds.
#[derive(Debug, Clone)]
pub struct Envelope {
    /// Upper bound at each sample index.
    pub upper: Vec<f64>,
    /// Lower bound at each sample index.
    pub lower: Vec<f64>,
    /// Global scalar upper bound on residual magnitude (for windows
    /// longer than the model waveform).
    pub global_upper: f64,
    /// Global scalar lower bound.
    pub global_lower: f64,
}

/// Estimate the admissibility envelope from the nominal baseline.
///
/// The `quantile_level` (e.g., 0.99) determines the multiplier k
/// via the Chebyshev-style approximation k = 1 / sqrt(1 - q).
/// For q = 0.99, k ≈ 10.0 which is very conservative. In practice
/// we use a gentler mapping: k = quantile_to_k(q).
pub fn estimate_envelope(baseline: &NominalBaseline, quantile_level: f64) -> Envelope {
    let k = quantile_to_k(quantile_level);
    let n = baseline.mean_waveform.len();
    let mut upper = Vec::with_capacity(n);
    let mut lower = Vec::with_capacity(n);
    for i in 0..n {
        let sigma = baseline.waveform_variance[i].sqrt();
        upper.push(k * sigma);
        lower.push(-k * sigma);
    }
    let global_sigma = baseline.mean_variance.sqrt();
    Envelope {
        upper,
        lower,
        global_upper: k * global_sigma,
        global_lower: -k * global_sigma,
    }
}

/// Map a quantile level to a multiplier k.
///
/// Uses a heuristic motivated by the normal distribution:
/// for q in [0.9, 0.999], k ranges roughly from 1.65 to 3.3.
fn quantile_to_k(q: f64) -> f64 {
    // Approximate inverse-normal scaling.
    // For q = 0.95 → k ≈ 1.96, q = 0.99 → k ≈ 2.58, q = 0.999 → k ≈ 3.29.
    if q <= 0.5 {
        return 0.67;
    }
    if q >= 0.9999 {
        return 4.0;
    }
    // Simple rational approximation of the probit function for the upper tail.
    let p = 1.0 - q;
    let t = (-2.0 * p.ln()).sqrt();
    // Abramowitz & Stegun 26.2.23 approximation.
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t)
}

/// Compute the fraction of residual samples that fall outside the envelope.
pub fn breach_fraction(residual: &[f64], envelope: &Envelope) -> f64 {
    if residual.is_empty() {
        return 0.0;
    }
    let n = residual.len();
    let mut breaches = 0usize;
    for (i, &r) in residual.iter().enumerate() {
        let (lo, hi) = if i < envelope.lower.len() {
            (envelope.lower[i], envelope.upper[i])
        } else {
            (envelope.global_lower, envelope.global_upper)
        };
        if r < lo || r > hi {
            breaches += 1;
        }
    }
    breaches as f64 / n as f64
}
