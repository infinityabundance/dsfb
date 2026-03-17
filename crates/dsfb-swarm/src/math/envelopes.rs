use serde::Serialize;

use crate::math::residuals::ResidualStack;

#[derive(Debug, Clone)]
pub struct EnvelopeMonitor {
    decay: f64,
    warmup_steps: usize,
    negative_drift_limit: f64,
    persistent_window: usize,
    step: usize,
    warmup_max_residual: f64,
    warmup_max_drift: f64,
    warmup_max_slew: f64,
    warmup_max_combined: f64,
    recent_negative_count: usize,
    state: EnvelopeState,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EnvelopeState {
    pub scalar_residual_envelope: f64,
    pub scalar_drift_envelope: f64,
    pub scalar_slew_envelope: f64,
    pub combined_envelope: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnomalyCertificate {
    pub flagged: bool,
    pub residual_violation: bool,
    pub drift_violation: bool,
    pub slew_violation: bool,
    pub combined_violation: bool,
    pub persistent_negative_drift: bool,
    pub scalar_limit: f64,
    pub drift_limit: f64,
    pub slew_limit: f64,
    pub combined_limit: f64,
    pub score: f64,
    pub reasons: Vec<String>,
}

impl EnvelopeMonitor {
    pub fn new(warmup_steps: usize) -> Self {
        Self {
            decay: 0.94,
            warmup_steps,
            negative_drift_limit: -0.035,
            persistent_window: 6,
            step: 0,
            warmup_max_residual: 0.0,
            warmup_max_drift: 0.0,
            warmup_max_slew: 0.0,
            warmup_max_combined: 0.0,
            recent_negative_count: 0,
            state: EnvelopeState::default(),
        }
    }

    pub fn update(&mut self, residuals: &ResidualStack) -> AnomalyCertificate {
        self.state.scalar_residual_envelope =
            (self.decay * self.state.scalar_residual_envelope).max(residuals.scalar_residual.abs());
        self.state.scalar_drift_envelope =
            (self.decay * self.state.scalar_drift_envelope).max(residuals.scalar_drift.abs());
        self.state.scalar_slew_envelope =
            (self.decay * self.state.scalar_slew_envelope).max(residuals.scalar_slew.abs());
        self.state.combined_envelope =
            (self.decay * self.state.combined_envelope).max(residuals.combined_score.abs());

        if residuals.scalar_drift < self.negative_drift_limit {
            self.recent_negative_count += 1;
        } else {
            self.recent_negative_count = 0;
        }

        self.warmup_max_residual = self
            .warmup_max_residual
            .max(residuals.scalar_residual.abs());
        self.warmup_max_drift = self.warmup_max_drift.max(residuals.scalar_drift.abs());
        self.warmup_max_slew = self.warmup_max_slew.max(residuals.scalar_slew.abs());
        self.warmup_max_combined = self.warmup_max_combined.max(residuals.combined_score.abs());

        let certificate = if self.step < self.warmup_steps {
            AnomalyCertificate {
                flagged: false,
                residual_violation: false,
                drift_violation: false,
                slew_violation: false,
                combined_violation: false,
                persistent_negative_drift: false,
                scalar_limit: self.warmup_max_residual.max(1.0e-6),
                drift_limit: self.warmup_max_drift.max(1.0e-6),
                slew_limit: self.warmup_max_slew.max(1.0e-6),
                combined_limit: self.warmup_max_combined.max(1.0e-6),
                score: residuals.combined_score,
                reasons: Vec::new(),
            }
        } else {
            let scalar_limit = 2.6 * self.warmup_max_residual.max(0.01);
            let drift_limit = 2.4 * self.warmup_max_drift.max(0.01);
            let slew_limit = 2.3 * self.warmup_max_slew.max(0.01);
            let combined_limit = 2.5 * self.warmup_max_combined.max(0.01);

            let residual_violation = residuals.scalar_residual.abs() > scalar_limit;
            let drift_violation = residuals.scalar_drift.abs() > drift_limit;
            let slew_violation = residuals.scalar_slew.abs() > slew_limit;
            let combined_violation = residuals.combined_score > combined_limit;
            let persistent_negative_drift = self.recent_negative_count >= self.persistent_window;

            let mut reasons = Vec::new();
            if residual_violation {
                reasons.push("residual envelope exceeded".to_string());
            }
            if drift_violation {
                reasons.push("residual drift envelope exceeded".to_string());
            }
            if slew_violation {
                reasons.push("residual slew envelope exceeded".to_string());
            }
            if combined_violation {
                reasons.push("combined spectral residual score exceeded".to_string());
            }
            if persistent_negative_drift {
                reasons.push("persistent negative residual drift".to_string());
            }

            AnomalyCertificate {
                flagged: residual_violation
                    || drift_violation
                    || slew_violation
                    || combined_violation
                    || persistent_negative_drift,
                residual_violation,
                drift_violation,
                slew_violation,
                combined_violation,
                persistent_negative_drift,
                scalar_limit,
                drift_limit,
                slew_limit,
                combined_limit,
                score: residuals.combined_score,
                reasons,
            }
        };
        self.step += 1;
        certificate
    }

    pub fn state(&self) -> &EnvelopeState {
        &self.state
    }
}
