//! DSFB Observer implementation
//!
//! Implements the Drift-Slew Fusion Bootstrap algorithm

use crate::params::DsfbParams;
use crate::state::DsfbState;
use crate::trust::{calculate_trust_weights, TrustStats};

/// DSFB Observer
pub struct DsfbObserver {
    /// Observer parameters
    params: DsfbParams,
    /// Number of measurement channels
    channels: usize,
    /// Current state estimate
    state: DsfbState,
    /// EMA residuals for each channel
    ema_residuals: Vec<f64>,
    /// Trust statistics for each channel
    trust_stats: Vec<TrustStats>,
}

impl DsfbObserver {
    /// Create a new DSFB observer
    pub fn new(params: DsfbParams, channels: usize) -> Self {
        Self {
            params,
            channels,
            state: DsfbState::zero(),
            ema_residuals: vec![0.0; channels],
            trust_stats: vec![TrustStats::new(); channels],
        }
    }

    /// Initialize the state
    pub fn init(&mut self, initial_state: DsfbState) {
        self.state = initial_state;
    }

    /// Perform one step of the DSFB algorithm
    /// 
    /// # Arguments
    /// * `measurements` - Measurement vector y_k for each channel
    /// * `dt` - Time step
    /// 
    /// # Returns
    /// The corrected state estimate
    pub fn step(&mut self, measurements: &[f64], dt: f64) -> DsfbState {
        assert_eq!(measurements.len(), self.channels, "Measurement count mismatch");

        // Predict step
        let phi_pred = self.state.phi + self.state.omega * dt;
        let omega_pred = self.state.omega + self.state.alpha * dt;
        let alpha_pred = self.state.alpha;

        // Measurement function h_k(phi^-) = phi^- (identity)
        let h_pred = phi_pred;

        // Compute residuals: r_k = y_k - h_k(phi^-)
        let residuals: Vec<f64> = measurements.iter().map(|&y| y - h_pred).collect();

        // Calculate trust weights
        let weights = calculate_trust_weights(
            &residuals,
            &mut self.ema_residuals,
            self.params.rho,
            self.params.sigma0,
        );

        // Store trust stats
        for k in 0..self.channels {
            self.trust_stats[k].residual_ema = self.ema_residuals[k];
            self.trust_stats[k].weight = weights[k];
        }

        // Aggregate residual: R = sum_k w_k * r_k
        let aggregate_residual: f64 = residuals
            .iter()
            .zip(weights.iter())
            .map(|(&r, &w)| w * r)
            .sum();

        // Correct step
        let phi = phi_pred + self.params.k_phi * aggregate_residual;
        let omega = omega_pred + self.params.k_omega * aggregate_residual;
        let alpha = alpha_pred + self.params.k_alpha * aggregate_residual;

        self.state = DsfbState::new(phi, omega, alpha);
        self.state
    }

    /// Get the current state
    pub fn state(&self) -> DsfbState {
        self.state
    }

    /// Get trust statistics for all channels
    pub fn trust_stats(&self) -> &[TrustStats] {
        &self.trust_stats
    }

    /// Get trust weight for a specific channel
    pub fn trust_weight(&self, channel: usize) -> f64 {
        self.trust_stats[channel].weight
    }

    /// Get EMA residual for a specific channel
    pub fn ema_residual(&self, channel: usize) -> f64 {
        self.trust_stats[channel].residual_ema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer_creation() {
        let params = DsfbParams::default();
        let observer = DsfbObserver::new(params, 2);
        assert_eq!(observer.channels, 2);
        assert_eq!(observer.state.phi, 0.0);
    }

    #[test]
    fn test_observer_step_no_residual() {
        let params = DsfbParams::new(0.5, 0.1, 0.01, 0.9, 0.1);
        let mut observer = DsfbObserver::new(params, 2);
        observer.init(DsfbState::new(1.0, 0.1, 0.0));
        
        let dt = 0.1;
        let measurements = vec![1.01, 1.01]; // Close to predicted value
        let state = observer.step(&measurements, dt);
        
        // State should be updated
        assert!(state.phi > 1.0);
    }

    #[test]
    fn test_observer_trust_weights_sum() {
        let params = DsfbParams::default();
        let mut observer = DsfbObserver::new(params, 3);
        
        let measurements = vec![0.5, 1.5, 2.5];
        observer.step(&measurements, 0.1);
        
        let sum: f64 = (0..3).map(|i| observer.trust_weight(i)).sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }
}
