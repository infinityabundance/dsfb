use dsfb::TrustStats;

/// Single-channel residual-envelope state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResidualEnvelope {
    pub s: f64,
    pub rho: f64,
}

impl ResidualEnvelope {
    pub fn new(rho: f64, s0: f64) -> Self {
        assert!(
            rho.is_finite() && rho > 0.0 && rho < 1.0,
            "rho must be in (0, 1)"
        );
        assert!(s0.is_finite() && s0 >= 0.0, "s0 must be finite and >= 0");
        Self { s: s0, rho }
    }

    pub fn update(&mut self, residual: f64) -> f64 {
        assert!(residual.is_finite(), "residual must be finite");
        self.s = self.rho * self.s + (1.0 - self.rho) * residual.abs();
        self.s
    }

    /// Exposes the final envelope state in the same shape as the core DSFB trust API.
    pub fn as_dsfb_stats(&self, beta: f64) -> TrustStats {
        TrustStats {
            residual_ema: self.s,
            weight: TrustWeight::weight(beta, self.s),
        }
    }
}

/// Single-channel trust mapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrustWeight;

impl TrustWeight {
    pub fn weight(beta: f64, s: f64) -> f64 {
        assert!(
            beta.is_finite() && beta > 0.0,
            "beta must be finite and > 0"
        );
        assert!(s.is_finite() && s >= 0.0, "s must be finite and >= 0");
        1.0 / (1.0 + beta * s)
    }
}

#[cfg(test)]
mod tests {
    use super::{ResidualEnvelope, TrustWeight};

    #[test]
    fn envelope_update_matches_recursion() {
        let mut env = ResidualEnvelope::new(0.9, 0.0);
        let s = env.update(2.0);
        assert!((s - 0.2).abs() < 1e-12);
    }

    #[test]
    fn trust_weight_is_monotone() {
        let w_low = TrustWeight::weight(2.0, 0.1);
        let w_high = TrustWeight::weight(2.0, 0.6);
        assert!(w_low > w_high);
    }
}
