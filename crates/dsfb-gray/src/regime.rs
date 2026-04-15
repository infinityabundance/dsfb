//! Workload phase classification for regime-conditioned envelopes.
//!
//! Different workload phases (warmup, steady-state, burst, cooldown)
//! require different admissibility envelopes. This module classifies
//! the current phase from observable signals.

/// Workload phase for envelope conditioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkloadPhase {
    /// System is warming up (caches cold, connections establishing).
    Warmup,
    /// Normal operating conditions.
    SteadyState,
    /// High-load burst (e.g., traffic spike, batch job).
    Burst,
    /// System is draining or shutting down.
    Cooldown,
    /// Custom phase with a static label.
    Custom(&'static str),
}

/// Classifier that determines the current workload phase from
/// throughput and latency signals.
pub struct RegimeClassifier {
    current_phase: WorkloadPhase,
    warmup_samples_remaining: u32,
    throughput_ema: f64,
    throughput_ema_alpha: f64,
    burst_threshold_multiplier: f64,
}

impl RegimeClassifier {
    /// Create a new classifier.
    ///
    /// - `warmup_samples`: number of samples to stay in Warmup phase
    /// - `burst_threshold`: multiplier above steady-state EMA to trigger Burst
    pub fn new(warmup_samples: u32, burst_threshold: f64) -> Self {
        Self {
            current_phase: WorkloadPhase::Warmup,
            warmup_samples_remaining: warmup_samples,
            throughput_ema: 0.0,
            throughput_ema_alpha: 0.1,
            burst_threshold_multiplier: burst_threshold.max(1.1),
        }
    }

    /// Update the classifier with a new throughput observation.
    pub fn observe_throughput(&mut self, throughput: f64) -> WorkloadPhase {
        if self.warmup_samples_remaining > 0 {
            self.warmup_samples_remaining -= 1;
            self.throughput_ema = throughput; // Initialize EMA
            if self.warmup_samples_remaining == 0 {
                self.current_phase = WorkloadPhase::SteadyState;
            }
            return self.current_phase;
        }

        // Update EMA
        self.throughput_ema = self.throughput_ema_alpha * throughput
            + (1.0 - self.throughput_ema_alpha) * self.throughput_ema;

        // Classify based on throughput relative to EMA
        if throughput > self.throughput_ema * self.burst_threshold_multiplier {
            self.current_phase = WorkloadPhase::Burst;
        } else if throughput < self.throughput_ema * 0.1 {
            self.current_phase = WorkloadPhase::Cooldown;
        } else {
            self.current_phase = WorkloadPhase::SteadyState;
        }

        self.current_phase
    }

    /// Force a specific phase (e.g., on system restart).
    pub fn set_phase(&mut self, phase: WorkloadPhase) {
        self.current_phase = phase;
    }

    /// Current phase.
    pub fn phase(&self) -> WorkloadPhase {
        self.current_phase
    }

    /// Reset to warmup phase.
    pub fn reset(&mut self, warmup_samples: u32) {
        self.current_phase = WorkloadPhase::Warmup;
        self.warmup_samples_remaining = warmup_samples;
        self.throughput_ema = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_in_warmup() {
        let classifier = RegimeClassifier::new(10, 2.0);
        assert_eq!(classifier.phase(), WorkloadPhase::Warmup);
    }

    #[test]
    fn test_transitions_to_steady_state() {
        let mut classifier = RegimeClassifier::new(3, 2.0);
        classifier.observe_throughput(100.0);
        classifier.observe_throughput(100.0);
        assert_eq!(classifier.phase(), WorkloadPhase::Warmup);
        classifier.observe_throughput(100.0);
        assert_eq!(classifier.phase(), WorkloadPhase::SteadyState);
    }
}
