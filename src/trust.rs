//! Trust weight calculation for DSFB
//!
//! Implements the trust-adaptive mechanism using EMA residuals

/// Trust statistics for a single channel
#[derive(Debug, Clone, PartialEq)]
pub struct TrustStats {
    /// EMA of absolute residuals
    pub residual_ema: f64,
    /// Trust weight (normalized)
    pub weight: f64,
}

impl TrustStats {
    /// Create new trust statistics
    pub fn new() -> Self {
        Self {
            residual_ema: 0.0,
            weight: 1.0,
        }
    }
}

impl Default for TrustStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate trust weights from residuals
pub fn calculate_trust_weights(
    residuals: &[f64],
    ema_residuals: &mut [f64],
    rho: f64,
    sigma0: f64,
) -> Vec<f64> {
    let n = residuals.len();
    let mut raw_weights = vec![0.0; n];

    // Update EMA and calculate raw trust weights
    for k in 0..n {
        // Update EMA: s_k = rho*s_k + (1-rho)*|r_k|
        ema_residuals[k] = rho * ema_residuals[k] + (1.0 - rho) * residuals[k].abs();

        // Trust softness: wtilde_k = 1 / (sigma0 + s_k)
        raw_weights[k] = 1.0 / (sigma0 + ema_residuals[k]);
    }

    // Normalize weights: w_k = wtilde_k / sum_j wtilde_j
    let sum: f64 = raw_weights.iter().sum();
    if sum > 0.0 {
        for w in raw_weights.iter_mut() {
            *w /= sum;
        }
    } else {
        // Fallback to uniform weights
        let uniform = 1.0 / n as f64;
        for w in raw_weights.iter_mut() {
            *w = uniform;
        }
    }

    raw_weights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_weights_uniform() {
        let residuals = vec![0.1, 0.1, 0.1];
        let mut ema_residuals = vec![0.0, 0.0, 0.0];
        let weights = calculate_trust_weights(&residuals, &mut ema_residuals, 0.9, 0.1);

        // All weights should be equal for equal residuals
        assert!((weights[0] - 1.0 / 3.0).abs() < 1e-10);
        assert!((weights[1] - 1.0 / 3.0).abs() < 1e-10);
        assert!((weights[2] - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_trust_weights_sum_to_one() {
        let residuals = vec![0.1, 1.0, 0.5];
        let mut ema_residuals = vec![0.0, 0.0, 0.0];
        let weights = calculate_trust_weights(&residuals, &mut ema_residuals, 0.9, 0.1);

        let sum: f64 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}
