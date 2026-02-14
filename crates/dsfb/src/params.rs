//! DSFB Parameters
//!
//! Parameters for the DSFB observer algorithm

/// Parameters for the DSFB observer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DsfbParams {
    /// Gain for phi correction
    pub k_phi: f64,
    /// Gain for omega correction
    pub k_omega: f64,
    /// Gain for alpha correction
    pub k_alpha: f64,
    /// EMA smoothing factor (0 < rho < 1)
    pub rho: f64,
    /// Trust softness parameter
    pub sigma0: f64,
}

impl DsfbParams {
    /// Create new DSFB parameters
    pub fn new(k_phi: f64, k_omega: f64, k_alpha: f64, rho: f64, sigma0: f64) -> Self {
        Self {
            k_phi,
            k_omega,
            k_alpha,
            rho,
            sigma0,
        }
    }

    /// Create default parameters suitable for basic simulation
    pub fn default_params() -> Self {
        Self {
            k_phi: 0.5,
            k_omega: 0.1,
            k_alpha: 0.01,
            rho: 0.95,
            sigma0: 0.1,
        }
    }
}

impl Default for DsfbParams {
    fn default() -> Self {
        Self::default_params()
    }
}
