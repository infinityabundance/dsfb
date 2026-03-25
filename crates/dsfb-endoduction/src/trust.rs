//! Trust / precursor score aggregation.
//!
//! The trust score is a bounded scalar (0..1) that aggregates
//! multiple structural motif indicators into a single precursor
//! proximity measure. It is NOT a probability of failure. It is
//! a departure-from-nominal structure score inspired by the
//! Thermodynamic Precursor Visibility Principle.
//!
//! Higher values indicate greater structural departure from the
//! nominal regime.

/// Inputs to the trust score computation for one window.
#[derive(Debug, Clone)]
pub struct TrustInputs {
    /// Envelope breach fraction (0..1).
    pub breach_fraction: f64,
    /// Persistence score (0..1).
    pub persistence: f64,
    /// Autocorrelation growth (can be negative; we use abs or positive part).
    pub autocorr_growth: f64,
    /// Spectral centroid shift (can be negative).
    pub spectral_shift: f64,
    /// Variance growth ratio (1.0 = nominal).
    pub variance_growth: f64,
    /// Absolute drift magnitude.
    pub drift_magnitude: f64,
    /// Normalisation references from baseline.
    pub baseline_drift_scale: f64,
    pub baseline_spectral_scale: f64,
}

/// Compute the trust / precursor score from structural motif indicators.
///
/// The score is a weighted, bounded aggregation of normalised indicator
/// values. Each indicator is normalised to roughly [0,1] using
/// sigmoidal or linear clamping, then combined via a weighted sum
/// and clamped to [0,1].
///
/// Weights reflect the paper's emphasis on multiple concurrent
/// structural departures as stronger evidence than any single indicator.
pub fn compute_trust_score(inputs: &TrustInputs) -> f64 {
    // Normalise each component to [0, 1].

    // Breach fraction is already in [0, 1].
    let n_breach = inputs.breach_fraction.clamp(0.0, 1.0);

    // Persistence: subtract baseline expectation (~0.02 for random),
    // scale so that persistence of 0.1 maps to ~ 0.5.
    let n_persist = sigmoid((inputs.persistence - 0.02) * 20.0);

    // Autocorrelation growth: positive values indicate increased persistence.
    let n_autocorr = sigmoid(inputs.autocorr_growth * 5.0);

    // Spectral shift: use absolute value, normalised by baseline scale.
    let spec_scale = if inputs.baseline_spectral_scale > 1e-10 {
        inputs.baseline_spectral_scale
    } else {
        0.01
    };
    let n_spectral = sigmoid(inputs.spectral_shift.abs() / spec_scale - 1.0);

    // Variance growth: values > 1 indicate departure.
    let n_variance = sigmoid((inputs.variance_growth - 1.0) * 2.0);

    // Drift: normalised by baseline drift scale.
    let drift_scale = if inputs.baseline_drift_scale > 1e-10 {
        inputs.baseline_drift_scale
    } else {
        1e-6
    };
    let n_drift = sigmoid(inputs.drift_magnitude / drift_scale - 1.0);

    // Weighted combination.
    const W_BREACH: f64 = 0.20;
    const W_PERSIST: f64 = 0.15;
    const W_AUTOCORR: f64 = 0.20;
    const W_SPECTRAL: f64 = 0.10;
    const W_VARIANCE: f64 = 0.20;
    const W_DRIFT: f64 = 0.15;

    let raw = W_BREACH * n_breach
        + W_PERSIST * n_persist
        + W_AUTOCORR * n_autocorr
        + W_SPECTRAL * n_spectral
        + W_VARIANCE * n_variance
        + W_DRIFT * n_drift;

    raw.clamp(0.0, 1.0)
}

/// Standard logistic sigmoid: 1 / (1 + exp(-x)).
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_score_bounded() {
        let inputs = TrustInputs {
            breach_fraction: 0.0,
            persistence: 0.02,
            autocorr_growth: 0.0,
            spectral_shift: 0.0,
            variance_growth: 1.0,
            drift_magnitude: 0.0,
            baseline_drift_scale: 1e-5,
            baseline_spectral_scale: 0.01,
        };
        let s = compute_trust_score(&inputs);
        assert!(s >= 0.0 && s <= 1.0, "Score out of bounds: {s}");
    }

    #[test]
    fn trust_score_increases_with_breach() {
        let base = TrustInputs {
            breach_fraction: 0.0,
            persistence: 0.02,
            autocorr_growth: 0.0,
            spectral_shift: 0.0,
            variance_growth: 1.0,
            drift_magnitude: 0.0,
            baseline_drift_scale: 1e-5,
            baseline_spectral_scale: 0.01,
        };
        let high = TrustInputs {
            breach_fraction: 0.8,
            persistence: 0.3,
            autocorr_growth: 0.5,
            spectral_shift: 0.05,
            variance_growth: 5.0,
            drift_magnitude: 1e-3,
            ..base.clone()
        };
        assert!(compute_trust_score(&high) > compute_trust_score(&base));
    }
}
