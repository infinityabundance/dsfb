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
    agent_count: usize,
    decay: f64,
    smoothing: f64,
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
    recent_signal_count: usize,
    candidate_streak: usize,
    signal_ema: f64,
    drift_ema: f64,
    slew_ema: f64,
    combined_ema: f64,
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
    pub candidate_flag: bool,
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
    pub recent_exceedance_count: usize,
    pub recent_signal_count: usize,
    pub candidate_streak: usize,
    pub score: f64,
    pub reasons: Vec<String>,
}

impl EnvelopeMonitor {
    pub fn new_scalar(warmup_steps: usize) -> Self {
        Self::new_scalar_for_agents(warmup_steps, 0)
    }

    pub fn new_scalar_for_agents(warmup_steps: usize, agent_count: usize) -> Self {
        Self {
            mode: MonitorMode::ScalarNegative,
            agent_count,
            decay: 0.94,
            smoothing: 0.80,
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
            recent_signal_count: 0,
            candidate_streak: 0,
            signal_ema: 0.0,
            drift_ema: 0.0,
            slew_ema: 0.0,
            combined_ema: 0.0,
            state: EnvelopeState::default(),
        }
    }

    pub fn new_multimode(warmup_steps: usize) -> Self {
        Self::new_multimode_for_agents(warmup_steps, 0)
    }

    pub fn new_multimode_for_agents(warmup_steps: usize, agent_count: usize) -> Self {
        Self {
            mode: MonitorMode::MultiStack,
            agent_count,
            decay: 0.94,
            smoothing: 0.70,
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
            recent_signal_count: 0,
            candidate_streak: 0,
            signal_ema: 0.0,
            drift_ema: 0.0,
            slew_ema: 0.0,
            combined_ema: 0.0,
            state: EnvelopeState::default(),
        }
    }

    pub fn is_calibrated(&self) -> bool {
        self.step >= self.warmup_steps
    }

    pub fn update(&mut self, residuals: &ResidualStack) -> AnomalyCertificate {
        let raw_signal_value = match self.mode {
            MonitorMode::ScalarNegative => (-residuals.scalar_residual).max(0.0),
            MonitorMode::MultiStack => {
                let mode_count = residuals.residuals.len().max(1) as f64;
                let stack_rms = residuals.stack_norm / mode_count.sqrt();
                let mode_shape_rms = residuals.mode_shape_norm / mode_count.sqrt();
                let scalar_support = (-residuals.scalar_residual).max(0.0);
                let mode_shape_term = if stack_rms > 0.45 {
                    0.08 * mode_shape_rms
                } else {
                    0.04 * mode_shape_rms
                };
                0.80 * stack_rms + mode_shape_term + 0.28 * scalar_support
            }
        };
        let raw_negative_drift_value = (-residuals.scalar_drift).max(0.0).sqrt();
        let raw_slew_value = residuals.scalar_slew.abs().sqrt();
        let raw_combined_value = match self.mode {
            MonitorMode::ScalarNegative => {
                0.78 * raw_signal_value + 0.22 * raw_negative_drift_value
            }
            MonitorMode::MultiStack => 0.88 * raw_signal_value + 0.12 * raw_negative_drift_value,
        };
        let signal_value = ema_step(&mut self.signal_ema, raw_signal_value, self.smoothing);
        let negative_drift_value = ema_step(
            &mut self.drift_ema,
            raw_negative_drift_value,
            self.smoothing,
        );
        let slew_value = ema_step(&mut self.slew_ema, raw_slew_value, self.smoothing);
        let combined_value = ema_step(&mut self.combined_ema, raw_combined_value, self.smoothing);

        self.state.scalar_residual_envelope =
            (self.decay * self.state.scalar_residual_envelope).max(signal_value);
        self.state.scalar_drift_envelope =
            (self.decay * self.state.scalar_drift_envelope).max(negative_drift_value);
        self.state.scalar_slew_envelope =
            (self.decay * self.state.scalar_slew_envelope).max(slew_value);
        self.state.combined_envelope =
            (self.decay * self.state.combined_envelope).max(combined_value);

        let certificate = if self.step < self.warmup_steps {
            push_bounded(&mut self.warmup_signal_samples, signal_value, 48);
            push_bounded(&mut self.warmup_drift_samples, negative_drift_value, 48);
            push_bounded(&mut self.warmup_slew_samples, slew_value, 48);
            push_bounded(&mut self.warmup_combined_samples, combined_value, 48);
            AnomalyCertificate {
                flagged: false,
                candidate_flag: false,
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
                recent_exceedance_count: 0,
                recent_signal_count: 0,
                candidate_streak: 0,
                score: combined_value,
                reasons: Vec::new(),
            }
        } else {
            if self.step == self.warmup_steps {
                self.refresh_limits();
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
            let mode_count = residuals.residuals.len().max(1) as f64;
            let mode_shape_support = residuals.mode_shape_norm / mode_count.sqrt();

            if combined_ratio < 0.82 && scalar_ratio < 0.82 {
                push_bounded(&mut self.warmup_signal_samples, signal_value, 48);
                push_bounded(&mut self.warmup_drift_samples, negative_drift_value, 48);
                push_bounded(&mut self.warmup_slew_samples, slew_value, 48);
                push_bounded(&mut self.warmup_combined_samples, combined_value, 48);
                self.refresh_limits();
            }

            update_counter(
                &mut self.recent_negative_count,
                drift_ratio > 0.92,
                drift_ratio < 0.55,
            );
            update_counter(
                &mut self.recent_exceedance_count,
                combined_ratio > 0.90 || scalar_ratio > 0.95,
                combined_ratio < 0.88 && scalar_ratio < 1.0,
            );
            update_counter(
                &mut self.recent_signal_count,
                scalar_ratio > 1.02,
                scalar_ratio < 0.84 && combined_ratio < 0.80,
            );
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

            let candidate_flag = match self.mode {
                MonitorMode::ScalarNegative => {
                    (self.recent_exceedance_count >= 4
                        && combined_ratio > 1.20
                        && scalar_ratio > 1.50)
                        || (self.recent_exceedance_count >= 3
                            && combined_ratio > 1.30
                            && scalar_ratio > 1.60)
                        || (self.recent_exceedance_count >= 8
                            && combined_ratio > 0.90
                            && scalar_ratio > 4.0)
                        || (self.recent_exceedance_count >= 3
                            && combined_ratio > 1.26
                            && scalar_ratio > 1.50
                            && persistent_negative_drift)
                        || (self.recent_signal_count
                            >= scalar_signal_streak_requirement(self.agent_count)
                            && scalar_ratio > 1.12
                            && combined_ratio > 0.46)
                }
                MonitorMode::MultiStack => {
                    let structured_support = scalar_ratio > 0.82
                        || persistent_negative_drift
                        || mode_shape_support > 0.14;
                    (combined_ratio > 1.24
                        && self.recent_exceedance_count >= 3
                        && structured_support)
                        || (combined_ratio > 1.30
                            && (scalar_ratio > 0.82
                                || persistent_negative_drift
                                || mode_shape_support > 0.10))
                        || (combined_ratio > 1.00
                            && scalar_ratio > 1.04
                            && self.recent_exceedance_count >= 6)
                        || (persistent_negative_drift
                            && scalar_ratio > 0.88
                            && combined_ratio > 1.08
                            && self.recent_exceedance_count >= 3)
                }
            };
            self.candidate_streak = if candidate_flag {
                self.candidate_streak + 1
            } else {
                0
            };
            let required_streak = match self.mode {
                MonitorMode::ScalarNegative => 1,
                MonitorMode::MultiStack => multimode_required_streak(self.agent_count),
            };
            let flagged = candidate_flag && self.candidate_streak >= required_streak;

            AnomalyCertificate {
                flagged,
                candidate_flag,
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
                recent_exceedance_count: self.recent_exceedance_count,
                recent_signal_count: self.recent_signal_count,
                candidate_streak: self.candidate_streak,
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

impl EnvelopeMonitor {
    fn refresh_limits(&mut self) {
        let signal_samples = tail_window(&self.warmup_signal_samples, 18);
        let drift_samples = tail_window(&self.warmup_drift_samples, 18);
        let slew_samples = tail_window(&self.warmup_slew_samples, 18);
        let combined_samples = tail_window(&self.warmup_combined_samples, 18);
        let large_scale = self.agent_count >= 100;
        let (
            signal_factor,
            drift_factor,
            slew_factor,
            combined_factor,
            signal_floor,
            drift_floor,
            slew_floor,
            combined_floor,
        ) = match (self.mode, large_scale) {
            (MonitorMode::ScalarNegative, true) => (2.2, 2.1, 2.2, 2.15, 0.005, 0.028, 0.06, 0.038),
            (MonitorMode::ScalarNegative, false) => (2.4, 2.2, 2.2, 2.3, 0.006, 0.03, 0.06, 0.04),
            (MonitorMode::MultiStack, true) => (2.1, 2.0, 2.0, 2.1, 0.055, 0.03, 0.06, 0.075),
            (MonitorMode::MultiStack, false) => (2.0, 2.0, 2.0, 2.0, 0.05, 0.03, 0.06, 0.07),
        };
        self.cached_signal_limit = calibrated_limit(signal_samples, signal_factor, signal_floor);
        self.cached_drift_limit = calibrated_limit(drift_samples, drift_factor, drift_floor);
        self.cached_slew_limit = calibrated_limit(slew_samples, slew_factor, slew_floor);
        self.cached_combined_limit =
            calibrated_limit(combined_samples, combined_factor, combined_floor);
    }
}

fn calibrated_limit(samples: &[f64], factor: f64, floor: f64) -> f64 {
    if samples.is_empty() {
        return floor;
    }
    let center = median(samples);
    let deviations = samples
        .iter()
        .map(|value| (value - center).abs())
        .collect::<Vec<_>>();
    let mad = median(&deviations);
    let robust_scale = (1.4826 * mad).max(0.15 * center.abs());
    (center + factor * robust_scale).max(floor)
}

fn tail_window(samples: &[f64], max_len: usize) -> &[f64] {
    let start = samples.len().saturating_sub(max_len);
    &samples[start..]
}

fn ema_step(state: &mut f64, sample: f64, smoothing: f64) -> f64 {
    if *state == 0.0 {
        *state = sample;
    } else {
        *state = smoothing * *state + (1.0 - smoothing) * sample;
    }
    *state
}

fn push_bounded(samples: &mut Vec<f64>, sample: f64, max_len: usize) {
    samples.push(sample);
    if samples.len() > max_len {
        let excess = samples.len() - max_len;
        samples.drain(..excess);
    }
}

fn update_counter(counter: &mut usize, positive: bool, reset: bool) {
    if positive {
        *counter += 1;
    } else if reset {
        *counter = 0;
    } else {
        *counter = counter.saturating_sub(1);
    }
}

fn median(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by(|left, right| left.total_cmp(right));
    let middle = ordered.len() / 2;
    if ordered.len() % 2 == 0 {
        0.5 * (ordered[middle - 1] + ordered[middle])
    } else {
        ordered[middle]
    }
}

fn scalar_signal_streak_requirement(agent_count: usize) -> usize {
    if agent_count >= 100 {
        7
    } else {
        8
    }
}

fn multimode_required_streak(agent_count: usize) -> usize {
    if agent_count >= 100 {
        4
    } else {
        3
    }
}
