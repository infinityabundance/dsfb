use serde::Serialize;

use crate::math::residuals::ResidualStack;

#[derive(Debug, Clone, Copy)]
enum MonitorMode {
    ScalarNegative,
    MultiStack,
}

#[derive(Debug, Clone)]
pub struct EnvelopeMonitor {
    mode: MonitorMode,
    decay: f64,
    warmup_steps: usize,
    persistent_window: usize,
    step: usize,
    warmup_signal_samples: Vec<f64>,
    warmup_drift_samples: Vec<f64>,
    warmup_slew_samples: Vec<f64>,
    warmup_combined_samples: Vec<f64>,
    cached_signal_limit: f64,
    cached_drift_limit: f64,
    cached_slew_limit: f64,
    cached_combined_limit: f64,
    recent_negative_count: usize,
    recent_exceedance_count: usize,
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
    pub signal_value: f64,
    pub scalar_limit: f64,
    pub drift_limit: f64,
    pub slew_limit: f64,
    pub combined_limit: f64,
    pub scalar_ratio: f64,
    pub drift_ratio: f64,
    pub combined_ratio: f64,
    pub score: f64,
    pub reasons: Vec<String>,
}

impl EnvelopeMonitor {
    pub fn new_scalar(warmup_steps: usize) -> Self {
        Self {
            mode: MonitorMode::ScalarNegative,
            decay: 0.94,
            warmup_steps,
            persistent_window: 2,
            step: 0,
            warmup_signal_samples: Vec::new(),
            warmup_drift_samples: Vec::new(),
            warmup_slew_samples: Vec::new(),
            warmup_combined_samples: Vec::new(),
            cached_signal_limit: 0.015,
            cached_drift_limit: 0.04,
            cached_slew_limit: 0.10,
            cached_combined_limit: 0.06,
            recent_negative_count: 0,
            recent_exceedance_count: 0,
            state: EnvelopeState::default(),
        }
    }

    pub fn new_multimode(warmup_steps: usize) -> Self {
        Self {
            mode: MonitorMode::MultiStack,
            decay: 0.94,
            warmup_steps,
            persistent_window: 2,
            step: 0,
            warmup_signal_samples: Vec::new(),
            warmup_drift_samples: Vec::new(),
            warmup_slew_samples: Vec::new(),
            warmup_combined_samples: Vec::new(),
            cached_signal_limit: 0.08,
            cached_drift_limit: 0.04,
            cached_slew_limit: 0.10,
            cached_combined_limit: 0.14,
            recent_negative_count: 0,
            recent_exceedance_count: 0,
            state: EnvelopeState::default(),
        }
    }

    pub fn is_calibrated(&self) -> bool {
        self.step >= self.warmup_steps
    }

    pub fn update(&mut self, residuals: &ResidualStack) -> AnomalyCertificate {
        let signal_value = match self.mode {
            MonitorMode::ScalarNegative => (-residuals.scalar_residual).max(0.0),
            MonitorMode::MultiStack => residuals.stack_norm + 1.10 * residuals.mode_shape_norm,
        };
        let negative_drift_value = (-residuals.scalar_drift).max(0.0);
        let slew_value = residuals.scalar_slew.abs();
        let combined_value = match self.mode {
            MonitorMode::ScalarNegative => 0.65 * signal_value + 0.35 * negative_drift_value,
            MonitorMode::MultiStack => signal_value,
        };

        self.state.scalar_residual_envelope =
            (self.decay * self.state.scalar_residual_envelope).max(signal_value);
        self.state.scalar_drift_envelope =
            (self.decay * self.state.scalar_drift_envelope).max(negative_drift_value);
        self.state.scalar_slew_envelope =
            (self.decay * self.state.scalar_slew_envelope).max(slew_value);
        self.state.combined_envelope =
            (self.decay * self.state.combined_envelope).max(combined_value);

        let certificate = if self.step < self.warmup_steps {
            self.warmup_signal_samples.push(signal_value);
            self.warmup_drift_samples.push(negative_drift_value);
            self.warmup_slew_samples.push(slew_value);
            self.warmup_combined_samples.push(combined_value);
            AnomalyCertificate {
                flagged: false,
                residual_violation: false,
                drift_violation: false,
                slew_violation: false,
                combined_violation: false,
                persistent_negative_drift: false,
                signal_value,
                scalar_limit: self.cached_signal_limit,
                drift_limit: self.cached_drift_limit,
                slew_limit: self.cached_slew_limit,
                combined_limit: self.cached_combined_limit,
                scalar_ratio: 0.0,
                drift_ratio: 0.0,
                combined_ratio: 0.0,
                score: combined_value,
                reasons: Vec::new(),
            }
        } else {
            if self.step == self.warmup_steps {
                let signal_samples = tail_window(&self.warmup_signal_samples, 14);
                let drift_samples = tail_window(&self.warmup_drift_samples, 14);
                let slew_samples = tail_window(&self.warmup_slew_samples, 14);
                let combined_samples = tail_window(&self.warmup_combined_samples, 14);
                self.cached_signal_limit = calibrated_limit(signal_samples, 1.55, 0.006);
                self.cached_drift_limit = calibrated_limit(drift_samples, 1.35, 0.03);
                self.cached_slew_limit = calibrated_limit(slew_samples, 1.6, 0.06);
                self.cached_combined_limit = calibrated_limit(combined_samples, 1.45, 0.04);
            }

            let scalar_limit = self.cached_signal_limit.max(1.0e-6);
            let drift_limit = self.cached_drift_limit.max(1.0e-6);
            let slew_limit = self.cached_slew_limit.max(1.0e-6);
            let combined_limit = self.cached_combined_limit.max(1.0e-6);

            let scalar_ratio = signal_value / scalar_limit;
            let drift_ratio = negative_drift_value / drift_limit;
            let combined_ratio = combined_value / combined_limit;

            let residual_violation = scalar_ratio > 1.0;
            let drift_violation = drift_ratio > 1.0;
            let slew_violation = slew_value > slew_limit;
            let combined_violation = combined_ratio > 1.0;

            if drift_violation {
                self.recent_negative_count += 1;
            } else {
                self.recent_negative_count = 0;
            }
            if combined_ratio > 0.85 || scalar_ratio > 0.85 {
                self.recent_exceedance_count += 1;
            } else {
                self.recent_exceedance_count = 0;
            }
            let persistent_negative_drift = self.recent_negative_count >= self.persistent_window;

            let mut reasons = Vec::new();
            if residual_violation {
                reasons.push("detector signal exceeded calibrated limit".to_string());
            }
            if drift_violation {
                reasons.push("negative residual drift exceeded calibrated limit".to_string());
            }
            if slew_violation {
                reasons.push("residual slew envelope exceeded".to_string());
            }
            if combined_violation {
                reasons.push("combined decision score exceeded calibrated limit".to_string());
            }
            if persistent_negative_drift {
                reasons.push("persistent negative residual drift".to_string());
            }

            let flagged = match self.mode {
                MonitorMode::ScalarNegative => {
                    (persistent_negative_drift && (scalar_ratio > 0.22 || combined_ratio > 0.24))
                        || (combined_ratio > 1.0 && self.recent_exceedance_count >= 2)
                        || (scalar_ratio > 1.15)
                }
                MonitorMode::MultiStack => {
                    (combined_ratio > 0.88 && self.recent_exceedance_count >= 2)
                        || (combined_ratio > 1.08)
                        || (persistent_negative_drift && scalar_ratio > 0.52)
                }
            };

            AnomalyCertificate {
                flagged,
                residual_violation,
                drift_violation,
                slew_violation,
                combined_violation,
                persistent_negative_drift,
                signal_value,
                scalar_limit,
                drift_limit,
                slew_limit,
                combined_limit,
                scalar_ratio,
                drift_ratio,
                combined_ratio,
                score: combined_value,
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

fn calibrated_limit(samples: &[f64], factor: f64, floor: f64) -> f64 {
    if samples.is_empty() {
        return floor;
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    (mean + factor * variance.sqrt()).max(floor)
}

fn tail_window(samples: &[f64], max_len: usize) -> &[f64] {
    let start = samples.len().saturating_sub(max_len);
    &samples[start..]
}
