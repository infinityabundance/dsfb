/// DSFB Oil & Gas — Grammar Automaton
///
/// Implements the finite-state transition function δ_G of the structural
/// residual grammar G = (Σ, Q, q_0, δ_G, Λ).
///
/// Precedence (highest first): Compound > EnvViolation > SlewSpike >
/// DriftAccum > BoundaryGrazing > Recovery > Nominal.

use crate::{
    envelope::EnvelopeEval,
    types::{GrammarState, ReasonCode, ResidualTriple},
};
#[cfg(feature = "alloc")]
use crate::envelope::evaluate;
#[cfg(feature = "alloc")]
use crate::types::{AdmissibilityEnvelope, AnnotatedStep};
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Stateful grammar classifier.  Holds only the previous state.
pub struct GrammarClassifier {
    prev_state: GrammarState,
}

impl Default for GrammarClassifier {
    fn default() -> Self {
        GrammarClassifier { prev_state: GrammarState::Nominal }
    }
}

impl GrammarClassifier {
    pub fn new() -> Self { Self::default() }

    /// Apply the transition function for one time step.
    /// Returns the (new_state, reason_code).
    pub fn classify(
        &mut self,
        eval: &EnvelopeEval,
        triple: &ResidualTriple,
    ) -> (GrammarState, ReasonCode) {
        use GrammarState::*;

        // ── OOB meta-token ────────────────────────────────────────────────────
        // Non-finite residual components indicate a sensor fault or a historian
        // gap encoded as IEEE 754 NaN/±∞.  Emit SensorFault without disturbing
        // the automaton's internal state: Recovery continuity is preserved.
        if !triple.r.is_finite() || !triple.delta.is_finite() || !triple.sigma.is_finite() {
            return (GrammarState::SensorFault, ReasonCode::oob_sensor());
        }

        let new_state = if eval.delta_violated() && eval.sigma_violated() {
            Compound
        } else if eval.r_violated() {
            EnvViolation
        } else if eval.sigma_violated() {
            SlewSpike
        } else if eval.delta_violated() {
            DriftAccum
        } else if eval.any_grazing() {
            BoundaryGrazing
        } else if self.prev_state != Nominal {
            Recovery
        } else {
            Nominal
        };

        let reason = match new_state {
            Nominal         => ReasonCode::nominal(),
            DriftAccum      => ReasonCode::drift(triple.delta),
            SlewSpike       => ReasonCode::slew(triple.sigma),
            EnvViolation    => ReasonCode::violation(),
            BoundaryGrazing => ReasonCode::grazing(),
            Recovery        => ReasonCode::recovery(),
            Compound        => ReasonCode::compound(),
            // Unreachable via early return above, but required for exhaustiveness.
            SensorFault     => ReasonCode::oob_sensor(),
        };

        // Recovery is a transient state: after emitting Recovery we reset to
        // Nominal so the *next* interior step is not also classified Recovery.
        // Without this, any step following a non-Nominal event would emit
        // Recovery indefinitely until another violation occurs.
        self.prev_state = if new_state == GrammarState::Recovery {
            GrammarState::Nominal
        } else {
            new_state
        };
        (new_state, reason)
    }

    pub fn reset(&mut self) {
        self.prev_state = GrammarState::Nominal;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DsfbEngine: ties together processor, envelope, and classifier
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "alloc")]
use crate::{
    residual::ResidualProcessor,
    types::{DsfbDomainFrame, ResidualSample},
};

/// Convenience trait alias for naming the engine in public docs.
#[cfg(feature = "alloc")]
pub trait DsfbEngine {
    fn ingest_sample(&mut self, sample: &ResidualSample) -> AnnotatedStep;
}

/// Full DSFB engine for one channel: residual → triple → envelope → grammar → annotated step.
#[cfg(feature = "alloc")]
pub struct DeterministicDsfb {
    processor: ResidualProcessor,
    envelope: AdmissibilityEnvelope,
    classifier: GrammarClassifier,
    history: Vec<AnnotatedStep>,
    channel: String,
}

#[cfg(feature = "alloc")]
impl DeterministicDsfb {
    pub fn new(
        envelope: AdmissibilityEnvelope,
        classifier: GrammarClassifier,
    ) -> Self {
        Self::with_window(envelope, classifier, 10, "channel")
    }

    pub fn with_window(
        envelope: AdmissibilityEnvelope,
        classifier: GrammarClassifier,
        drift_window: usize,
        channel: impl Into<String>,
    ) -> Self {
        DeterministicDsfb {
            processor: ResidualProcessor::new(drift_window),
            envelope,
            classifier,
            history: Vec::new(),
            channel: channel.into(),
        }
    }

    /// Ingest a domain frame (any DsfbDomainFrame implementor).
    /// The resulting AnnotatedStep is appended to `history()` and a reference returned.
    pub fn ingest<F: DsfbDomainFrame>(&mut self, frame: F) -> &AnnotatedStep {
        let sample = frame.to_residual_sample();
        self.ingest_sample(&sample);
        self.history.last().unwrap()
    }

    /// Ingest a raw residual sample directly.
    /// The resulting AnnotatedStep is appended to `history()` and returned.
    pub fn ingest_sample(&mut self, sample: &ResidualSample) -> AnnotatedStep {
        let triple = self.processor.process(sample);
        let eval = evaluate(&self.envelope, &triple);
        let (state, reason) = self.classifier.classify(&eval, &triple);
        let step = AnnotatedStep {
            triple,
            state,
            reason,
            channel: self.channel.clone(),
        };
        self.history.push(step.clone());
        step
    }

    /// Immutable view of the annotation history.
    pub fn history(&self) -> &[AnnotatedStep] { &self.history }

    /// All annotated steps where the grammar state is non-Nominal.
    pub fn events(&self) -> Vec<&AnnotatedStep> {
        self.history.iter().filter(|s| s.state.is_non_nominal()).collect()
    }

    pub fn reset(&mut self) {
        self.processor.reset();
        self.classifier.reset();
        self.history.clear();
    }
}

#[cfg(feature = "alloc")]
impl DsfbEngine for DeterministicDsfb {
    fn ingest_sample(&mut self, sample: &ResidualSample) -> AnnotatedStep {
        DeterministicDsfb::ingest_sample(self, sample)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::types::{AdmissibilityEnvelope, ResidualSample};

    fn engine() -> DeterministicDsfb {
        DeterministicDsfb::with_window(
            AdmissibilityEnvelope::default_pipeline(),
            GrammarClassifier::new(),
            5,
            "test_channel",
        )
    }

    #[test]
    fn constant_zero_is_always_nominal() {
        let mut eng = engine();
        for i in 0..50 {
            let s = ResidualSample::new(i as f64, 0.0, 0.0, "x");
            let step = eng.ingest_sample(&s);
            assert_eq!(step.state, GrammarState::Nominal, "step {i} should be Nominal");
        }
    }

    #[test]
    fn large_slew_produces_slew_spike() {
        let mut eng = engine();
        // baseline
        for i in 0..10 {
            eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "x"));
        }
        // large jump in residual → large slew; residual also exceeds envelope → Compound
        let step = eng.ingest_sample(&ResidualSample::new(11.0, 1000.0, 0.0, "x"));
        // Compound takes precedence over SlewSpike when envelope is also violated
        assert!(
            matches!(step.state, GrammarState::SlewSpike | GrammarState::Compound | GrammarState::EnvViolation),
            "expected slew-related token, got {:?}", step.state
        );
    }

    #[test]
    fn large_drift_produces_drift_accum() {
        let mut eng = DeterministicDsfb::with_window(
            AdmissibilityEnvelope::default_pipeline(),
            GrammarClassifier::new(),
            3,   // short window so drift fills quickly
            "x",
        );
        // sustained residual well above delta_max
        let samples: Vec<_> = (0..20)
            .map(|i| ResidualSample::new(i as f64, 50.0, 0.0, "x"))
            .collect();
        let mut last_state = GrammarState::Nominal;
        for s in &samples {
            last_state = eng.ingest_sample(s).state;
        }
        // After sustained large residual, drift should be outside bounds
        assert!(
            last_state == GrammarState::DriftAccum
                || last_state == GrammarState::EnvViolation
                || last_state == GrammarState::Compound,
            "expected non-Nominal from sustained large residual, got {:?}", last_state
        );
    }

    #[test]
    fn deterministic_replay_identical() {
        let samples: Vec<_> = (0..30)
            .map(|i| {
                let v = if i == 15 { 100.0 } else { (i as f64).sin() };
                ResidualSample::new(i as f64, v, 0.0, "x")
            })
            .collect();

        let run = |s: &[ResidualSample]| -> Vec<GrammarState> {
            let mut eng = engine();
            s.iter().map(|r| eng.ingest_sample(r).state).collect()
        };

        let run1 = run(&samples);
        let run2 = run(&samples);
        assert_eq!(run1, run2, "deterministic replay produced different results");
    }

    // ── OOB / SensorFault tests ───────────────────────────────────────────────

    #[test]
    fn nan_residual_emits_sensor_fault() {
        let mut eng = engine();
        // Establish nominal baseline
        for i in 0..5 {
            eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "x"));
        }
        // NaN observed → NaN residual propagates through drift/slew → SensorFault
        let step = eng.ingest_sample(&ResidualSample::new(5.0, f64::NAN, 0.0, "x"));
        assert_eq!(step.state, GrammarState::SensorFault,
            "NaN observed should emit SensorFault, got {:?}", step.state);
    }

    #[test]
    fn inf_residual_emits_sensor_fault() {
        let mut eng = engine();
        for i in 0..5 {
            eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "x"));
        }
        // Infinite residual (sensor rail-out / historian gap coded as +Inf)
        let step = eng.ingest_sample(&ResidualSample::new(5.0, f64::INFINITY, 0.0, "x"));
        assert_eq!(step.state, GrammarState::SensorFault,
            "Infinity observed should emit SensorFault, got {:?}", step.state);
    }

    #[test]
    fn sensor_fault_does_not_corrupt_recovery_state() {
        let mut eng = engine();
        // Trigger non-Nominal build-up
        for i in 0..10 {
            eng.ingest_sample(&ResidualSample::new(i as f64, 1000.0, 0.0, "x"));
        }
        // NaN step must emit SensorFault without poisoning the drift ring buffer
        let sf_step = eng.ingest_sample(&ResidualSample::new(10.0, f64::NAN, 0.0, "x"));
        assert_eq!(sf_step.state, GrammarState::SensorFault,
            "NaN after non-Nominal must emit SensorFault, got {:?}", sf_step.state);
        // After the NaN step, internal ring buffer must NOT be NaN-poisoned:
        // the next clean sample must produce a finite (non-SensorFault) state.
        let next = eng.ingest_sample(&ResidualSample::new(11.0, 0.0, 0.0, "x"));
        assert_ne!(next.state, GrammarState::SensorFault,
            "Clean sample after SensorFault must not be SensorFault (ring buffer poisoned?), got {:?}", next.state);
        // All three components of the returned triple must be finite
        assert!(next.triple.r.is_finite(), "r must be finite after clean sample");
        assert!(next.triple.delta.is_finite(), "delta must be finite; ring buffer was poisoned");
        assert!(next.triple.sigma.is_finite(), "sigma must be finite after clean sample");
    }
}
