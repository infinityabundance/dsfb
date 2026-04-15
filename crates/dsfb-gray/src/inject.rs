#![allow(missing_docs)]

//! # DSFB Fault Injection — Deterministic Chaos for Distributed Rust Systems
//!
//! Provides reproducible, seed-controlled fault injection scenarios that
//! produce gray failures: states where a system isn't "down" but is
//! performing incorrectly in ways that standard health checks miss.

use crate::{
    DsfbObserver, GrammarState, ObserverConfig, ReasonCode, ResidualSample, ResidualSource,
};

/// A deterministic fault scenario producing a time-series of residual samples.
pub trait FaultScenario {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn next_sample(&mut self, step: u64) -> Option<ResidualSample>;
    fn expected_reason_code(&self) -> ReasonCode;
    fn total_steps(&self) -> u64;
    fn reset(&mut self);
    fn injection_start(&self) -> u64;
}

/// Deterministic xorshift64 noise generator.
fn xorshift_noise(state: &mut u64, amplitude: f64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    let normalized = (*state as f64) / (u64::MAX as f64) * 2.0 - 1.0;
    normalized * amplitude
}

/// Clock drift injection scenario.
pub struct ClockDriftScenario {
    pub baseline_rtt_ms: f64,
    pub drift_rate: f64,
    pub injection_start_step: u64,
    pub duration: u64,
    pub noise_amp: f64,
    noise_state: u64,
    seed: u64,
}

impl ClockDriftScenario {
    pub fn new(baseline: f64, drift_rate: f64, start: u64, duration: u64, noise: f64) -> Self {
        Self {
            baseline_rtt_ms: baseline,
            drift_rate,
            injection_start_step: start,
            duration,
            noise_amp: noise * baseline,
            noise_state: 42,
            seed: 42,
        }
    }
    pub fn default_scenario() -> Self {
        Self::new(5.0, 0.05, 50, 200, 0.02)
    }
}

impl FaultScenario for ClockDriftScenario {
    fn name(&self) -> &str {
        "Clock Drift Injection"
    }
    fn description(&self) -> &str {
        "Monotonic clock divergence producing increasing apparent heartbeat RTT"
    }
    fn injection_start(&self) -> u64 {
        self.injection_start_step
    }
    fn next_sample(&mut self, step: u64) -> Option<ResidualSample> {
        if step >= self.duration {
            return None;
        }
        let drift = if step >= self.injection_start_step {
            self.drift_rate * (step - self.injection_start_step) as f64
        } else {
            0.0
        };
        let noise = xorshift_noise(&mut self.noise_state, self.noise_amp);
        Some(ResidualSample {
            value: self.baseline_rtt_ms + drift + noise,
            baseline: self.baseline_rtt_ms,
            timestamp_ns: step * 1_000_000_000,
            source: ResidualSource::HeartbeatRtt,
        })
    }
    fn expected_reason_code(&self) -> ReasonCode {
        ReasonCode::ClockDriftDivergence
    }
    fn total_steps(&self) -> u64 {
        self.duration
    }
    fn reset(&mut self) {
        self.noise_state = self.seed;
    }
}

/// Partial network partition scenario.
pub struct PartialPartitionScenario {
    pub baseline: f64,
    pub start: u64,
    pub duration: u64,
    pub rate: f64,
    pub burst: f64,
    pub burst_dur: u64,
    pub noise_state: u64,
    pub seed: u64,
}

impl PartialPartitionScenario {
    pub fn default_scenario() -> Self {
        Self {
            baseline: 5.0,
            start: 40,
            duration: 200,
            rate: 0.08,
            burst: 3.0,
            burst_dur: 10,
            noise_state: 137,
            seed: 137,
        }
    }
}

impl FaultScenario for PartialPartitionScenario {
    fn name(&self) -> &str {
        "Partial Network Partition"
    }
    fn description(&self) -> &str {
        "Selective packet loss producing burst-then-drift latency signature"
    }
    fn injection_start(&self) -> u64 {
        self.start
    }
    fn next_sample(&mut self, step: u64) -> Option<ResidualSample> {
        if step >= self.duration {
            return None;
        }
        let pert = if step >= self.start {
            let e = (step - self.start) as f64;
            let b = if (step - self.start) < self.burst_dur {
                self.burst * (1.0 - e / self.burst_dur as f64)
            } else {
                0.0
            };
            b + self.rate * e
        } else {
            0.0
        };
        let noise = xorshift_noise(&mut self.noise_state, self.baseline * 0.03);
        Some(ResidualSample {
            value: self.baseline + pert + noise,
            baseline: self.baseline,
            timestamp_ns: step * 1_000_000_000,
            source: ResidualSource::Latency,
        })
    }
    fn expected_reason_code(&self) -> ReasonCode {
        ReasonCode::PartialPartitionSignature
    }
    fn total_steps(&self) -> u64 {
        self.duration
    }
    fn reset(&mut self) {
        self.noise_state = self.seed;
    }
}

/// Channel backpressure scenario.
pub struct ChannelBackpressureScenario {
    pub baseline: f64,
    pub start: u64,
    pub duration: u64,
    pub rate: f64,
    pub noise_state: u64,
    pub seed: u64,
}
impl ChannelBackpressureScenario {
    pub fn default_scenario() -> Self {
        Self {
            baseline: 100.0,
            start: 30,
            duration: 200,
            rate: 5.0,
            noise_state: 271,
            seed: 271,
        }
    }
}
impl FaultScenario for ChannelBackpressureScenario {
    fn name(&self) -> &str {
        "Channel Backpressure Onset"
    }
    fn description(&self) -> &str {
        "Bounded mpsc channel depth growing toward capacity"
    }
    fn injection_start(&self) -> u64 {
        self.start
    }
    fn next_sample(&mut self, step: u64) -> Option<ResidualSample> {
        if step >= self.duration {
            return None;
        }
        let growth = if step >= self.start {
            let e = (step - self.start) as f64;
            self.rate * e + 0.05 * e * e
        } else {
            0.0
        };
        let noise = xorshift_noise(&mut self.noise_state, 5.0);
        Some(ResidualSample {
            value: (self.baseline + growth + noise).min(1000.0),
            baseline: self.baseline,
            timestamp_ns: step * 1_000_000_000,
            source: ResidualSource::QueueDepth,
        })
    }
    fn expected_reason_code(&self) -> ReasonCode {
        ReasonCode::ChannelBackpressureOnset
    }
    fn total_steps(&self) -> u64 {
        self.duration
    }
    fn reset(&mut self) {
        self.noise_state = self.seed;
    }
}

/// Async runtime starvation scenario.
pub struct AsyncStarvationScenario {
    pub baseline: f64,
    pub start: u64,
    pub duration: u64,
    pub rate: f64,
    pub noise_state: u64,
    pub seed: u64,
}
impl AsyncStarvationScenario {
    pub fn default_scenario() -> Self {
        Self {
            baseline: 50.0,
            start: 60,
            duration: 200,
            rate: 2.0,
            noise_state: 313,
            seed: 313,
        }
    }
}
impl FaultScenario for AsyncStarvationScenario {
    fn name(&self) -> &str {
        "Async Runtime Starvation"
    }
    fn description(&self) -> &str {
        "Tokio task poll duration increasing from blocking in async context"
    }
    fn injection_start(&self) -> u64 {
        self.start
    }
    fn next_sample(&mut self, step: u64) -> Option<ResidualSample> {
        if step >= self.duration {
            return None;
        }
        let starv = if step >= self.start {
            self.rate * (step - self.start) as f64
        } else {
            0.0
        };
        let noise = xorshift_noise(&mut self.noise_state, 3.0);
        Some(ResidualSample {
            value: self.baseline + starv + noise,
            baseline: self.baseline,
            timestamp_ns: step * 1_000_000_000,
            source: ResidualSource::PollDuration,
        })
    }
    fn expected_reason_code(&self) -> ReasonCode {
        ReasonCode::AsyncRuntimeStarvation
    }
    fn total_steps(&self) -> u64 {
        self.duration
    }
    fn reset(&mut self) {
        self.noise_state = self.seed;
    }
}

/// Run a scenario through a DSFB observer and collect results.
pub fn run_scenario(scenario: &mut dyn FaultScenario, config: &ObserverConfig) -> ScenarioResult {
    scenario.reset();
    let first = scenario.next_sample(0);
    scenario.reset();
    let src = first.map(|s| s.source).unwrap_or(ResidualSource::Latency);
    let mut observer = DsfbObserver::new(src, config);
    let injection_start = scenario.injection_start();
    let mut stats = ScenarioRunStats::default();
    let mut samples = Vec::with_capacity(scenario.total_steps() as usize);

    for step in 0..scenario.total_steps() {
        if let Some(sample) = scenario.next_sample(step) {
            let observation = observer.observe(&sample);
            samples.push(sample_record(step, &sample, &observation));
            update_scenario_run_stats(
                &mut stats,
                step,
                injection_start,
                observation.grammar_state,
                observation.heuristic_match.reason_code,
            );
        }
    }
    build_scenario_result(scenario, injection_start, stats, samples)
}

#[derive(Default)]
struct ScenarioRunStats {
    first_boundary: Option<u64>,
    first_violation: Option<u64>,
    first_anomaly: Option<u64>,
    detected_reason_code: Option<ReasonCode>,
    boundary_count: u32,
    violation_count: u32,
    false_alarms: u32,
}

fn sample_record(
    step: u64,
    sample: &ResidualSample,
    observation: &crate::ObservationResult,
) -> SampleRecord {
    SampleRecord {
        step,
        value: sample.value,
        baseline: sample.baseline,
        residual: observation.sign.residual,
        drift: observation.sign.drift,
        slew: observation.sign.slew,
        grammar_state: observation.grammar_state,
    }
}

fn update_scenario_run_stats(
    stats: &mut ScenarioRunStats,
    step: u64,
    injection_start: u64,
    grammar_state: GrammarState,
    reason_code: ReasonCode,
) {
    if matches!(
        grammar_state,
        GrammarState::Boundary | GrammarState::Violation
    ) && stats.first_anomaly.is_none()
    {
        stats.first_anomaly = Some(step);
        stats.detected_reason_code = Some(reason_code);
        if step < injection_start {
            stats.false_alarms += 1;
        }
    }

    match grammar_state {
        GrammarState::Boundary => {
            stats.boundary_count += 1;
            if stats.first_boundary.is_none() {
                stats.first_boundary = Some(step);
            }
        }
        GrammarState::Violation => {
            stats.violation_count += 1;
            if stats.first_violation.is_none() {
                stats.first_violation = Some(step);
            }
        }
        GrammarState::Admissible => {}
    }
}

fn build_scenario_result(
    scenario: &dyn FaultScenario,
    injection_start: u64,
    stats: ScenarioRunStats,
    samples: Vec<SampleRecord>,
) -> ScenarioResult {
    ScenarioResult {
        scenario_name: scenario.name().into(),
        total_steps: scenario.total_steps(),
        injection_start,
        first_anomaly_step: stats.first_anomaly,
        first_boundary_step: stats.first_boundary,
        first_violation_step: stats.first_violation,
        detected_reason_code: stats.detected_reason_code,
        false_alarms_before_injection: stats.false_alarms,
        total_boundary_steps: stats.boundary_count,
        total_violation_steps: stats.violation_count,
        expected_reason_code: scenario.expected_reason_code(),
        samples,
    }
}

#[derive(Debug, Clone)]
pub struct SampleRecord {
    pub step: u64,
    pub value: f64,
    pub baseline: f64,
    pub residual: f64,
    pub drift: f64,
    pub slew: f64,
    pub grammar_state: GrammarState,
}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub total_steps: u64,
    pub injection_start: u64,
    pub first_anomaly_step: Option<u64>,
    pub first_boundary_step: Option<u64>,
    pub first_violation_step: Option<u64>,
    pub detected_reason_code: Option<ReasonCode>,
    pub false_alarms_before_injection: u32,
    pub total_boundary_steps: u32,
    pub total_violation_steps: u32,
    pub expected_reason_code: ReasonCode,
    pub samples: Vec<SampleRecord>,
}

impl ScenarioResult {
    pub fn detection_lead_time(&self) -> Option<u64> {
        self.first_anomaly_step
            .map(|s| self.total_steps.saturating_sub(s))
    }

    pub fn detected(&self) -> bool {
        self.first_anomaly_step.is_some()
    }

    pub fn detection_delay_from_injection(&self) -> Option<u64> {
        self.first_anomaly_step
            .filter(|step| *step >= self.injection_start)
            .map(|step| step - self.injection_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdmissibilityEnvelope, WorkloadPhase};

    #[test]
    fn test_clock_drift_detected() {
        let mut s = ClockDriftScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 20,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.1,
                0.05,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let r = run_scenario(&mut s, &config);
        assert!(r.detected(), "Clock drift must be detected");
        assert!(
            r.detection_lead_time().unwrap() > 10,
            "Must detect with lead time > 10 steps"
        );
    }

    #[test]
    fn test_partial_partition_detected() {
        let mut s = PartialPartitionScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 15,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                3.0,
                0.15,
                0.08,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        assert!(run_scenario(&mut s, &config).detected());
    }

    #[test]
    fn test_backpressure_detected() {
        let mut s = ChannelBackpressureScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 15,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                100.0,
                10.0,
                5.0,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        assert!(run_scenario(&mut s, &config).detected());
    }

    #[test]
    fn test_async_starvation_detected() {
        let mut s = AsyncStarvationScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 15,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                30.0,
                3.0,
                1.5,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        assert!(run_scenario(&mut s, &config).detected());
    }

    #[test]
    fn test_deterministic_replay() {
        let config = ObserverConfig {
            persistence_window: 20,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.1,
                0.05,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let mut s1 = ClockDriftScenario::default_scenario();
        let r1 = run_scenario(&mut s1, &config);
        let mut s2 = ClockDriftScenario::default_scenario();
        let r2 = run_scenario(&mut s2, &config);
        assert_eq!(
            r1.first_anomaly_step, r2.first_anomaly_step,
            "Must be deterministically reproducible"
        );
    }
}
