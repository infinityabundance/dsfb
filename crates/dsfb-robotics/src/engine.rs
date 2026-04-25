//! `DsfbRoboticsEngine` — the streaming orchestrator that composes
//! [`crate::sign::SignWindow`], [`crate::grammar::GrammarEvaluator`], and [`crate::envelope::AdmissibilityEnvelope`]
//! into a `no_std` + `no_alloc` observer suitable for bare-metal MCU
//! deployment alongside a safety-rated controller.
//!
//! ## Type parameters
//!
//! - `W` — drift-window length (samples over which ṙ is estimated).
//! - `K` — persistence threshold for the recurrent-grazing reason
//!   code (also the length of the internal grazing history buffer).
//!
//! ## Canonical observe loop
//!
//! ```
//! use dsfb_robotics::engine::DsfbRoboticsEngine;
//! use dsfb_robotics::platform::RobotContext;
//! use dsfb_robotics::Episode;
//!
//! let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
//! let mut out = [Episode::empty(); 32];
//!
//! let residual_norm: f64 = 0.045; // ‖r(k)‖ from your upstream observer
//! let ep = eng.observe_one(residual_norm, false, RobotContext::ArmOperating, 0);
//! // advisory output: ep.grammar, ep.decision
//! let _ = (ep.grammar, ep.decision);
//! let _ = &out;
//! ```

use crate::envelope::AdmissibilityEnvelope;
use crate::episode::Episode;
use crate::grammar::GrammarEvaluator;
use crate::platform::RobotContext;
use crate::policy::PolicyDecision;
use crate::sign::SignWindow;

/// Streaming DSFB engine.
///
/// All state is stack-allocated. No heap, no `unsafe`, no `std`.
pub struct DsfbRoboticsEngine<const W: usize, const K: usize> {
    envelope: AdmissibilityEnvelope,
    sign_window: SignWindow<W>,
    grammar: GrammarEvaluator<K>,
}

impl<const W: usize, const K: usize> DsfbRoboticsEngine<W, K> {
    /// Create an engine from an envelope radius, using the paper
    /// defaults for the boundary fraction and slew threshold.
    #[must_use]
    pub const fn new(rho: f64) -> Self {
        Self {
            envelope: AdmissibilityEnvelope::new(rho),
            sign_window: SignWindow::<W>::new(),
            grammar: GrammarEvaluator::<K>::new(),
        }
    }

    /// Create an engine from an explicit envelope.
    #[must_use]
    pub const fn from_envelope(envelope: AdmissibilityEnvelope) -> Self {
        Self {
            envelope,
            sign_window: SignWindow::<W>::new(),
            grammar: GrammarEvaluator::<K>::new(),
        }
    }

    /// Replace the envelope (e.g. after online recalibration on a
    /// longer healthy window).
    pub fn set_envelope(&mut self, envelope: AdmissibilityEnvelope) {
        self.envelope = envelope;
    }

    /// Inspect the current envelope.
    #[inline]
    #[must_use]
    pub fn envelope(&self) -> AdmissibilityEnvelope {
        self.envelope
    }

    /// Observe a single residual norm and return the emitted episode.
    ///
    /// - `norm` — `‖r(k)‖` from the upstream observer.
    /// - `below_floor` — `true` if the sample is below the known
    ///   noise floor (forces drift and slew to zero for this sample).
    /// - `context` — current robot operating regime.
    /// - `index` — sample index within the caller's stream, passed
    ///   through to [`Episode::index`] for traceability.
    pub fn observe_one(
        &mut self,
        norm: f64,
        below_floor: bool,
        context: RobotContext,
        index: usize,
    ) -> Episode {
        let sign = self.sign_window.push(norm, below_floor);
        let state = self.grammar.evaluate(&sign, &self.envelope, context);
        let decision = PolicyDecision::from_grammar(state);
        Episode::new(index, norm * norm, sign.drift, state, decision)
    }

    /// Stream `residuals` into a caller-owned output buffer `out`,
    /// emitting one episode per input sample.
    ///
    /// Returns the number of episodes written. Never writes past
    /// `out.len()`: if `residuals.len() > out.len()` the function
    /// stops at capacity and returns `out.len()` (fail-closed,
    /// advisory-only semantics). Passing `context` applies uniformly
    /// to every sample in the call — callers that need to change
    /// context mid-stream should invoke [`Self::observe_one`] in a
    /// loop.
    pub fn observe(
        &mut self,
        residuals: &[f64],
        out: &mut [Episode],
        context: RobotContext,
    ) -> usize {
        debug_assert!(residuals.len() <= usize::MAX / 2, "residuals slice unreasonable");
        debug_assert!(out.len() <= usize::MAX / 2, "output buffer unreasonable");

        let mut written = 0_usize;
        let n = residuals.len().min(out.len());
        let mut i = 0_usize;
        while i < n {
            let r = residuals[i];
            // Treat non-finite residuals as below-floor (missingness-aware),
            // matching the semiconductor-crate behaviour.
            let below_floor = !r.is_finite();
            let norm = if r.is_finite() { crate::math::abs_f64(r) } else { 0.0 };
            out[written] = self.observe_one(norm, below_floor, context, i);
            written += 1;
            i += 1;
        }
        written
    }

    /// Reset the streaming state (sign window + grammar hysteresis)
    /// without touching the envelope.
    ///
    /// Use after a commissioning-to-operating transition to avoid
    /// pre-commissioning noise bleeding into the operating state.
    pub fn reset(&mut self) {
        self.sign_window.reset();
        self.grammar.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_stays_admissible_for_quiet_input() {
        let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
        let residuals = [0.01_f64; 32];
        let mut out = [Episode::empty(); 32];
        let n = eng.observe(&residuals, &mut out, RobotContext::ArmOperating);
        assert_eq!(n, 32);
        for e in &out[..n] {
            assert_eq!(e.grammar, "Admissible");
            assert_eq!(e.decision, "Silent");
        }
    }

    #[test]
    fn persistent_violation_produces_escalate() {
        let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
        let residuals = [0.5_f64; 32];
        let mut out = [Episode::empty(); 32];
        let n = eng.observe(&residuals, &mut out, RobotContext::ArmOperating);
        assert_eq!(n, 32);
        // Hysteresis: first sample is pending, from sample 2 onward it must be committed.
        let escalated = out[..n].iter().filter(|e| e.decision == "Escalate").count();
        assert!(escalated >= 30, "expected ≥30 Escalate episodes, got {}", escalated);
    }

    #[test]
    fn commissioning_suppresses_everything() {
        let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
        let residuals = [1_000.0_f64; 32];
        let mut out = [Episode::empty(); 32];
        let n = eng.observe(&residuals, &mut out, RobotContext::ArmCommissioning);
        assert_eq!(n, 32);
        for e in &out[..n] {
            assert_eq!(e.grammar, "Admissible");
            assert_eq!(e.decision, "Silent");
        }
    }

    #[test]
    fn observe_respects_output_capacity() {
        let mut eng = DsfbRoboticsEngine::<8, 4>::new(0.1);
        let residuals = [0.02_f64; 32];
        let mut small_out = [Episode::empty(); 4];
        let n = eng.observe(&residuals, &mut small_out, RobotContext::ArmOperating);
        assert_eq!(n, 4, "must never write past output capacity");
    }

    #[test]
    fn observe_one_preserves_sample_index() {
        let mut eng = DsfbRoboticsEngine::<4, 3>::new(0.1);
        for i in 0..10 {
            let e = eng.observe_one(0.02, false, RobotContext::ArmOperating, i);
            assert_eq!(e.index, i);
        }
    }

    #[test]
    fn nonfinite_residual_treated_as_below_floor() {
        let mut eng = DsfbRoboticsEngine::<4, 3>::new(0.1);
        let residuals = [0.02_f64, f64::NAN, 0.02, f64::INFINITY, 0.02];
        let mut out = [Episode::empty(); 5];
        let n = eng.observe(&residuals, &mut out, RobotContext::ArmOperating);
        assert_eq!(n, 5);
        for e in &out[..n] {
            assert_eq!(e.grammar, "Admissible");
            assert_eq!(e.decision, "Silent");
        }
    }

    #[test]
    fn reset_clears_streaming_state_but_keeps_envelope() {
        let mut eng = DsfbRoboticsEngine::<4, 3>::new(0.1);
        let before = eng.envelope().rho;
        for _ in 0..10 {
            eng.observe_one(0.5, false, RobotContext::ArmOperating, 0);
        }
        eng.reset();
        let after = eng.envelope().rho;
        assert_eq!(before, after);
        // Post-reset, the first observation re-enters pending state.
        let e = eng.observe_one(0.01, false, RobotContext::ArmOperating, 0);
        assert_eq!(e.grammar, "Admissible");
    }
}
