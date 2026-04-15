//! The DSFB observer: complete pipeline from residual sample to episode.
//!
//! [`DsfbObserver`] is the top-level entry point. It wires together the
//! residual estimator, admissibility envelope, grammar state machine,
//! heuristics bank, episode builder, and audit trace into a single
//! `observe()` call that accepts an immutable sample and produces
//! a fully classified structural interpretation.
//!
//! ## Non-Interference Contract
//!
//! The observer accepts all inputs as immutable references. No mutable
//! reference to any upstream system component is created. If the observer
//! is removed, upstream behavior is unchanged.

use crate::adapter::TelemetryAdapter;
use crate::audit::{AuditEvent, AuditTrace};
use crate::envelope::{AdmissibilityEnvelope, EnvelopePosition};
use crate::episode::{Episode, EpisodeBuilder};
use crate::grammar::{GrammarMachine, GrammarState, GrammarTransition};
use crate::heuristics::{
    AppliedStaticPrior, HeuristicId, HeuristicsBank, MatchResult, StaticPriorSet,
};
use crate::residual::{ResidualEstimator, ResidualSample, ResidualSign, ResidualSource};
use crate::ReasonCode;

/// Configuration for the DSFB observer.
#[derive(Debug, Clone)]
pub struct ObserverConfig {
    /// Persistence window for drift/slew estimation.
    /// Recommended: 20–100 for 1-second sampling intervals.
    pub persistence_window: usize,
    /// Hysteresis count for grammar state transitions.
    /// Recommended: 3–10.
    pub hysteresis_count: u32,
    /// Default admissibility envelope (used when no regime-specific
    /// envelope is configured).
    pub default_envelope: AdmissibilityEnvelope,
    /// Optional static priors from the crate scanner or caller.
    pub static_priors: StaticPriorSet,
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

impl ObserverConfig {
    /// Balanced preset for general-purpose monitoring.
    pub fn balanced() -> Self {
        Self {
            persistence_window: 40,
            hysteresis_count: 5,
            default_envelope: AdmissibilityEnvelope::symmetric(
                10.0,
                1.0,
                0.5,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            static_priors: StaticPriorSet::default(),
        }
    }

    /// Lower-latency preset that favors earlier transitions.
    pub fn fast_response() -> Self {
        Self {
            persistence_window: 20,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                5.0,
                0.5,
                0.25,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            static_priors: StaticPriorSet::default(),
        }
    }

    /// Conservative preset that favors stability over early transitions.
    pub fn low_noise() -> Self {
        Self {
            persistence_window: 60,
            hysteresis_count: 6,
            default_envelope: AdmissibilityEnvelope::symmetric(
                12.0,
                1.2,
                0.6,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            static_priors: StaticPriorSet::default(),
        }
    }

    /// Return a copy of this configuration with static priors attached.
    pub fn with_static_priors(mut self, static_priors: StaticPriorSet) -> Self {
        self.static_priors = static_priors;
        self
    }
}

/// Observation result from a single `observe()` call.
#[derive(Debug, Clone)]
pub struct ObservationResult {
    /// The computed residual sign (r, ω, α).
    pub sign: ResidualSign,
    /// Current grammar state after this observation.
    pub grammar_state: GrammarState,
    /// Envelope position classification.
    pub envelope_position: EnvelopePosition,
    /// Heuristic match result (reason code + confidence).
    pub heuristic_match: MatchResult,
    /// Human-readable evidence for the emitted reason code.
    pub reason_evidence: ReasonEvidence,
    /// Grammar transition, if one occurred at this step.
    pub transition: Option<GrammarTransition>,
    /// Completed episode, if a grammar transition closed the previous one.
    pub completed_episode: Option<Episode>,
}

/// Human-readable explanation of the structural reason selected by DSFB.
#[derive(Debug, Clone, Copy)]
pub struct ReasonEvidence {
    /// Selected reason code.
    pub reason_code: ReasonCode,
    /// Heuristic that produced the reason code, if any.
    pub matched_heuristic: Option<HeuristicId>,
    /// Match confidence after bounded threshold scaling.
    pub confidence: f64,
    /// Human-readable description of the pattern.
    pub description: &'static str,
    /// Rust-specific provenance of the pattern.
    pub provenance: &'static str,
    /// Static prior applied to the winning heuristic, if any.
    pub applied_prior: Option<AppliedStaticPrior>,
}

/// The DSFB observer for a single residual source channel.
///
/// For multi-channel observation (e.g., monitoring latency + throughput +
/// heartbeat RTT simultaneously), create one `DsfbObserver` per channel
/// and correlate their outputs externally.
pub struct DsfbObserver {
    estimator: ResidualEstimator,
    grammar: GrammarMachine,
    heuristics: HeuristicsBank,
    episode_builder: EpisodeBuilder,
    audit: AuditTrace,
    envelope: AdmissibilityEnvelope,
    source: ResidualSource,
    static_priors: StaticPriorSet,
    observation_count: u64,
}

impl DsfbObserver {
    /// Create a new observer for the given source channel.
    pub fn new(source: ResidualSource, config: &ObserverConfig) -> Self {
        Self {
            estimator: ResidualEstimator::new(source, config.persistence_window),
            grammar: GrammarMachine::new(config.hysteresis_count),
            heuristics: HeuristicsBank::default_bank(),
            episode_builder: EpisodeBuilder::new(),
            audit: AuditTrace::new(),
            envelope: config.default_envelope,
            source,
            static_priors: config.static_priors,
            observation_count: 0,
        }
    }

    /// Create an observer with a custom heuristics bank.
    pub fn with_heuristics(
        source: ResidualSource,
        config: &ObserverConfig,
        heuristics: HeuristicsBank,
    ) -> Self {
        let mut obs = Self::new(source, config);
        obs.heuristics = heuristics;
        obs
    }

    /// Set the admissibility envelope (e.g., after regime classification).
    pub fn set_envelope(&mut self, envelope: AdmissibilityEnvelope) {
        self.envelope = envelope;
    }

    /// Replace the static priors used during heuristic matching.
    pub fn set_static_priors(&mut self, static_priors: StaticPriorSet) {
        self.static_priors = static_priors;
    }

    /// Adapt and observe one application-specific telemetry record.
    pub fn observe_adapted<T, A>(&mut self, adapter: &A, input: &T) -> ObservationResult
    where
        A: TelemetryAdapter<T>,
    {
        let sample = adapter.adapt(input);
        self.observe(&sample)
    }

    /// Process a single residual sample and return the full observation result.
    ///
    /// This is the primary API. It accepts an immutable reference to a sample
    /// and returns a complete structural interpretation.
    ///
    /// ## Non-Interference
    ///
    /// The sample is accepted as `&ResidualSample`. No mutation of the sample
    /// or its originating system is possible through this API.
    pub fn observe(&mut self, sample: &ResidualSample) -> ObservationResult {
        self.observation_count += 1;

        // Step 1: Compute residual sign (r, ω, α)
        let sign = self.estimator.observe(sample);

        // Step 2: Classify against admissibility envelope
        let envelope_position = self.envelope.classify(&sign);

        // Step 3: Update grammar state machine
        let (grammar_state, transition) = self.grammar.step(envelope_position, sample.timestamp_ns);

        // Step 4: Match against heuristics bank
        let heuristic_match =
            self.heuristics
                .match_sign_with_priors(&sign, grammar_state, &self.static_priors);
        let reason_evidence = ReasonEvidence {
            reason_code: heuristic_match.reason_code,
            matched_heuristic: heuristic_match.matched_heuristic,
            confidence: heuristic_match.confidence,
            description: heuristic_match.description,
            provenance: heuristic_match.provenance,
            applied_prior: heuristic_match.applied_prior,
        };

        // Step 5: Manage episodes
        let completed_episode =
            self.manage_episodes(&sign, grammar_state, &heuristic_match, transition.as_ref());

        // Step 6: Record audit event
        self.audit.record(AuditEvent {
            timestamp_ns: sample.timestamp_ns,
            residual: sign.residual,
            drift: sign.drift,
            slew: sign.slew,
            envelope_position: match envelope_position {
                EnvelopePosition::Interior => 0,
                EnvelopePosition::BoundaryZone => 1,
                EnvelopePosition::Exterior => 2,
            },
            grammar_state: grammar_state.severity(),
            transition_occurred: transition.is_some(),
        });

        ObservationResult {
            sign,
            grammar_state,
            envelope_position,
            heuristic_match,
            reason_evidence,
            transition,
            completed_episode,
        }
    }

    /// Manage episode lifecycle based on grammar transitions.
    fn manage_episodes(
        &mut self,
        sign: &ResidualSign,
        grammar_state: GrammarState,
        heuristic_match: &MatchResult,
        transition: Option<&GrammarTransition>,
    ) -> Option<Episode> {
        let mut completed = None;

        if let Some(trans) = transition {
            // Close the previous episode if one was open
            if self.episode_builder.is_open() {
                completed = self.episode_builder.close(trans.timestamp_ns);
            }

            // Open a new episode for the new state
            if grammar_state != GrammarState::Admissible || self.episode_builder.is_open() {
                self.episode_builder.open(
                    trans.timestamp_ns,
                    grammar_state,
                    heuristic_match.reason_code,
                    self.source,
                );
            }
        }

        // Update the current episode with this observation
        if self.episode_builder.is_open() {
            self.episode_builder
                .update(sign.residual, sign.drift, sign.slew);
        }

        completed
    }

    /// Current grammar state.
    pub fn grammar_state(&self) -> GrammarState {
        self.grammar.state()
    }

    /// Total observations processed.
    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }

    /// Reference to the audit trace.
    pub fn audit_trace(&self) -> &AuditTrace {
        &self.audit
    }

    /// Current open episode, if any.
    pub fn current_episode(&self) -> Option<&Episode> {
        self.episode_builder.current()
    }

    /// Reset the observer state. Used on system restart or phase transition.
    pub fn reset(&mut self) {
        self.estimator.reset();
        self.grammar.reset();
        self.audit.reset();
        self.episode_builder = EpisodeBuilder::new();
        self.observation_count = 0;
    }

    /// Source channel this observer tracks.
    pub fn source(&self) -> ResidualSource {
        self.source
    }
}

/// Multi-channel observer that tracks multiple residual sources simultaneously.
///
/// Provides a unified interface for observing all telemetry channels of a
/// distributed system and correlating their structural interpretations.
pub struct MultiChannelObserver {
    observers: [Option<DsfbObserver>; 16],
    active_count: usize,
}

impl MultiChannelObserver {
    /// Create a new multi-channel observer with no channels configured.
    pub fn new() -> Self {
        Self {
            observers: Default::default(),
            active_count: 0,
        }
    }

    /// Add an observer for a new source channel. Returns the channel index.
    ///
    /// # Panics
    ///
    /// Panics if 16 channels are already configured.
    pub fn add_channel(&mut self, source: ResidualSource, config: &ObserverConfig) -> usize {
        assert!(self.active_count < 16, "Maximum 16 channels supported");
        let idx = self.active_count;
        self.observers[idx] = Some(DsfbObserver::new(source, config));
        self.active_count += 1;
        idx
    }

    /// Observe a sample on a specific channel.
    pub fn observe(
        &mut self,
        channel: usize,
        sample: &ResidualSample,
    ) -> Option<ObservationResult> {
        self.observers
            .get_mut(channel)
            .and_then(|opt| opt.as_mut())
            .map(|obs| obs.observe(sample))
    }

    /// Get the grammar state of a specific channel.
    pub fn channel_state(&self, channel: usize) -> Option<GrammarState> {
        self.observers
            .get(channel)
            .and_then(|opt| opt.as_ref())
            .map(|obs| obs.grammar_state())
    }

    /// Number of active channels.
    pub fn active_channels(&self) -> usize {
        self.active_count
    }

    /// Check if ANY channel is in Boundary or Violation state.
    pub fn any_anomalous(&self) -> bool {
        self.observers
            .iter()
            .filter_map(|opt| opt.as_ref())
            .any(|obs| obs.grammar_state() != GrammarState::Admissible)
    }

    /// Collect the worst grammar state across all channels.
    pub fn worst_state(&self) -> GrammarState {
        self.observers
            .iter()
            .filter_map(|opt| opt.as_ref())
            .map(|obs| obs.grammar_state())
            .max_by_key(|s| s.severity())
            .unwrap_or(GrammarState::Admissible)
    }
}

impl Default for MultiChannelObserver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(value: f64, baseline: f64, ts: u64) -> ResidualSample {
        ResidualSample {
            value,
            baseline,
            timestamp_ns: ts,
            source: ResidualSource::Latency,
        }
    }

    #[test]
    fn test_observer_starts_admissible() {
        let config = ObserverConfig::default();
        let obs = DsfbObserver::new(ResidualSource::Latency, &config);
        assert_eq!(obs.grammar_state(), GrammarState::Admissible);
        assert_eq!(obs.observation_count(), 0);
    }

    #[test]
    fn test_stable_system_stays_admissible() {
        let config = ObserverConfig {
            persistence_window: 10,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                10.0,
                1.0,
                0.5,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let mut obs = DsfbObserver::new(ResidualSource::Latency, &config);

        for i in 0..50u64 {
            let s = sample(100.0, 100.0, i * 1_000_000_000);
            let result = obs.observe(&s);
            assert_eq!(result.grammar_state, GrammarState::Admissible);
        }
    }

    #[test]
    fn test_sustained_drift_triggers_boundary() {
        let config = ObserverConfig {
            persistence_window: 10,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                5.0,
                0.5,
                0.3,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let mut obs = DsfbObserver::new(ResidualSource::Latency, &config);

        let mut found_transition = false;
        // Feed a linearly increasing residual that will eventually
        // breach the envelope boundary and trigger grammar transition
        for i in 0..100u64 {
            // value grows from 100 to 150 over 100 samples
            let value = 100.0 + 0.5 * i as f64;
            let s = sample(value, 100.0, i * 1_000_000_000);
            let result = obs.observe(&s);
            if result.grammar_state != GrammarState::Admissible {
                found_transition = true;
                break;
            }
        }
        assert!(
            found_transition,
            "Expected grammar transition from sustained drift"
        );
    }

    #[test]
    fn test_audit_trace_records_observations() {
        let config = ObserverConfig::default();
        let mut obs = DsfbObserver::new(ResidualSource::Latency, &config);

        for i in 0..10u64 {
            let s = sample(100.0, 100.0, i * 1_000_000_000);
            obs.observe(&s);
        }

        assert_eq!(obs.audit_trace().total_count(), 10);
    }

    #[test]
    fn test_multi_channel_worst_state() {
        let config = ObserverConfig {
            persistence_window: 5,
            hysteresis_count: 2,
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.3,
                0.2,
                crate::regime::WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let mut multi = MultiChannelObserver::new();
        let ch0 = multi.add_channel(ResidualSource::Latency, &config);
        let ch1 = multi.add_channel(ResidualSource::HeartbeatRtt, &config);

        // Feed stable data to both channels
        for i in 0..20u64 {
            let s0 = ResidualSample {
                value: 50.0,
                baseline: 50.0,
                timestamp_ns: i * 1_000_000_000,
                source: ResidualSource::Latency,
            };
            multi.observe(ch0, &s0);

            let s1 = ResidualSample {
                value: 10.0,
                baseline: 10.0,
                timestamp_ns: i * 1_000_000_000,
                source: ResidualSource::HeartbeatRtt,
            };
            multi.observe(ch1, &s1);
        }

        assert!(!multi.any_anomalous());
        assert_eq!(multi.worst_state(), GrammarState::Admissible);
    }

    #[test]
    fn test_nonintrusive_contract() {
        // This test verifies the non-interference contract:
        // The observer only accepts immutable references.
        let config = ObserverConfig::default();
        let mut obs = DsfbObserver::new(ResidualSource::Latency, &config);

        let original_value = 100.0f64;
        let s = ResidualSample {
            value: original_value,
            baseline: 95.0,
            timestamp_ns: 0,
            source: ResidualSource::Latency,
        };

        // The observer takes &ResidualSample — immutable reference.
        // After observation, the original sample is unchanged.
        let _result = obs.observe(&s);

        // Original sample is still accessible and unchanged
        assert_eq!(s.value, original_value);
        assert_eq!(s.baseline, 95.0);
    }

    #[test]
    fn test_observe_adapted_uses_adapter_output() {
        struct QueueDepthAdapter;

        impl TelemetryAdapter<u64> for QueueDepthAdapter {
            fn adapt(&self, input: &u64) -> ResidualSample {
                ResidualSample {
                    value: *input as f64,
                    baseline: 8.0,
                    timestamp_ns: 1_000,
                    source: ResidualSource::QueueDepth,
                }
            }
        }

        let mut observer =
            DsfbObserver::new(ResidualSource::QueueDepth, &ObserverConfig::fast_response());
        let result = observer.observe_adapted(&QueueDepthAdapter, &11);
        assert_eq!(result.sign.source, ResidualSource::QueueDepth);
        assert_eq!(result.sign.timestamp_ns, 1_000);
        assert_eq!(result.sign.residual, 3.0);
    }
}
