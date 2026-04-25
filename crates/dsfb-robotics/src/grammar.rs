//! Grammar FSM: `Admissible | Boundary[ReasonCode] | Violation`.
//!
//! The grammar is the typed intermediate representation DSFB emits in
//! place of a scalar alarm count. Operators see *why* a residual
//! trajectory is drifting, not merely *that* a threshold was crossed.
//!
//! ## State assignment
//!
//! For each observation `k`:
//!
//! - **Violation**: `‖r(k)‖ > ρ_eff` (confirmed envelope exit).
//! - **Boundary**: `‖r(k)‖ > boundary_frac × ρ_eff` with a qualifying
//!   reason — sustained outward drift (ṙ > 0 over the drift window) or
//!   abrupt slew (|r̈| > δ_s) — **or** recurrent boundary grazing (K
//!   near-boundary hits in a K-long history buffer).
//! - **Admissible**: otherwise.
//!
//! ## Hysteresis
//!
//! Two consecutive confirmations are required before a state change is
//! committed, matching dsfb-rf's canonical FSM. This prevents
//! single-sample transients from flipping the grammar. During a
//! suppressed robot context (commissioning, maintenance) the FSM is
//! force-reset to `Admissible` so violations cannot occur.

use crate::envelope::AdmissibilityEnvelope;
use crate::platform::RobotContext;
use crate::sign::SignTuple;

/// Reason code qualifying a `Boundary` grammar state.
///
/// Typed reason codes let an operator distinguish classes of structural
/// behaviour without the observer making a fault-classification claim.
/// For a robotics deployment, the reason codes map to recognisable
/// failure *modes* (collision, friction drift, payload step, cyclic
/// loading) without DSFB having to commit to the *cause*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReasonCode {
    /// Persistent positive drift (ṙ > 0) across the drift window.
    ///
    /// Robotics mapping: friction/gravity-comp bias accumulating,
    /// thermal drift of joint encoders, slow payload mass change.
    SustainedOutwardDrift,

    /// Abrupt slew event (|r̈| > δ_s).
    ///
    /// Robotics mapping: collision onset, actuator saturation, sudden
    /// payload step, commanded-mode transition not flagged as suppressed.
    AbruptSlewViolation,

    /// `K` recurrent near-boundary hits within the last `K`
    /// observations.
    ///
    /// Robotics mapping: cyclic loading (periodic pick-and-place
    /// rhythm, gait cycle near limit), mechanical resonance, repetitive
    /// approach of a kinematic limit.
    RecurrentBoundaryGrazing,

    /// Confirmed envelope violation (`‖r‖ > ρ_eff`). Used as a reason
    /// qualifier when referring to the transition rather than the
    /// `Violation` state itself.
    EnvelopeViolation,
}

impl ReasonCode {
    /// Stable human-readable label for logging and JSON emission.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SustainedOutwardDrift => "SustainedOutwardDrift",
            Self::AbruptSlewViolation => "AbruptSlewViolation",
            Self::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
            Self::EnvelopeViolation => "EnvelopeViolation",
        }
    }
}

/// The typed grammar state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GrammarState {
    /// Residual inside envelope and not drifting outward. Nominal.
    #[default]
    Admissible,
    /// Residual in the boundary band with a qualifying reason code.
    Boundary(ReasonCode),
    /// Residual has exited the envelope.
    Violation,
}

impl GrammarState {
    /// `true` for any state that warrants operator attention.
    #[inline]
    #[must_use]
    pub const fn requires_attention(&self) -> bool {
        !matches!(self, Self::Admissible)
    }

    /// `true` iff this is the `Violation` state.
    #[inline]
    #[must_use]
    pub const fn is_violation(&self) -> bool {
        matches!(self, Self::Violation)
    }

    /// `true` iff this is a `Boundary[_]` state.
    #[inline]
    #[must_use]
    pub const fn is_boundary(&self) -> bool {
        matches!(self, Self::Boundary(_))
    }

    /// Severity level: `0 = Admissible`, `1 = Boundary`, `2 = Violation`.
    #[inline]
    #[must_use]
    pub const fn severity(&self) -> u8 {
        match self {
            Self::Admissible => 0,
            Self::Boundary(_) => 1,
            Self::Violation => 2,
        }
    }

    /// Stable label used in the canonical `Episode::grammar` field.
    #[inline]
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Admissible => "Admissible",
            Self::Boundary(_) => "Boundary",
            Self::Violation => "Violation",
        }
    }
}

/// Grammar evaluator with 2-confirmation hysteresis and `K`-long
/// boundary-grazing history.
///
/// All state is stack-allocated; no heap, no `unsafe`, no `std`.
pub struct GrammarEvaluator<const K: usize> {
    pending: GrammarState,
    confirmations: u8,
    committed: GrammarState,
    boundary_hits: [bool; K],
    hit_head: usize,
    hit_count: usize,
}

impl<const K: usize> GrammarEvaluator<K> {
    /// Construct an evaluator initialised to `Admissible`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: GrammarState::Admissible,
            confirmations: 0,
            committed: GrammarState::Admissible,
            boundary_hits: [false; K],
            hit_head: 0,
            hit_count: 0,
        }
    }

    /// The currently committed grammar state.
    #[inline]
    #[must_use]
    pub fn state(&self) -> GrammarState {
        self.committed
    }

    /// Evaluate the grammar state for one observation and return the
    /// committed state after applying hysteresis.
    pub fn evaluate(
        &mut self,
        sign: &SignTuple,
        envelope: &AdmissibilityEnvelope,
        context: RobotContext,
    ) -> GrammarState {
        debug_assert!(envelope.rho >= 0.0, "envelope radius must be non-negative");
        debug_assert!((0.0..=1.0).contains(&envelope.boundary_frac), "boundary_frac out of [0,1]");
        // Suppressed context (commissioning / maintenance): force and hold Admissible.
        if context.is_suppressed() {
            self.committed = GrammarState::Admissible;
            self.pending = GrammarState::Admissible;
            self.confirmations = 0;
            // Also clear grazing history so a resumption does not inherit
            // pre-commissioning boundary hits.
            self.boundary_hits = [false; K];
            self.hit_head = 0;
            self.hit_count = 0;
            return GrammarState::Admissible;
        }

        let multiplier = context.admissibility_multiplier();
        debug_assert!(multiplier >= 0.0, "admissibility multiplier must be non-negative");
        let raw = self.compute_raw_state(sign, envelope, multiplier);

        // Update boundary-grazing history.
        if K > 0 {
            let is_approach = envelope.is_boundary_approach(sign.norm, multiplier)
                && !envelope.is_violation(sign.norm, multiplier);
            self.boundary_hits[self.hit_head] = is_approach;
            self.hit_head = (self.hit_head + 1) % K;
            if self.hit_count < K {
                self.hit_count += 1;
            }
        }

        // 2-confirmation hysteresis.
        if raw == self.pending {
            if self.confirmations < 2 {
                self.confirmations += 1;
            }
            if self.confirmations >= 2 {
                self.committed = raw;
            }
        } else {
            self.pending = raw;
            self.confirmations = 1;
        }

        self.committed
    }

    fn compute_raw_state(
        &self,
        sign: &SignTuple,
        envelope: &AdmissibilityEnvelope,
        multiplier: f64,
    ) -> GrammarState {
        debug_assert!(envelope.rho >= 0.0);
        debug_assert!(multiplier >= 0.0);
        debug_assert!(self.hit_count <= K, "hit_count must never exceed K");
        if envelope.is_violation(sign.norm, multiplier) {
            return GrammarState::Violation;
        }

        if envelope.is_boundary_approach(sign.norm, multiplier) {
            if sign.is_outward_drift() {
                return GrammarState::Boundary(ReasonCode::SustainedOutwardDrift);
            }
            if sign.is_abrupt_slew(envelope.delta_s) {
                return GrammarState::Boundary(ReasonCode::AbruptSlewViolation);
            }
        }

        if K > 0 && self.hit_count >= K {
            let grazing_hits = self.boundary_hits.iter().filter(|&&h| h).count();
            debug_assert!(grazing_hits <= K, "grazing_hits bounded by buffer length");
            if grazing_hits >= K {
                return GrammarState::Boundary(ReasonCode::RecurrentBoundaryGrazing);
            }
        }

        GrammarState::Admissible
    }

    /// Reset the evaluator.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl<const K: usize> Default for GrammarEvaluator<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::AdmissibilityEnvelope;
    use crate::platform::RobotContext;
    use crate::sign::SignTuple;

    fn env() -> AdmissibilityEnvelope {
        AdmissibilityEnvelope::new(0.1)
    }

    #[test]
    fn clean_signal_is_admissible() {
        let mut e = GrammarEvaluator::<4>::new();
        for _ in 0..5 {
            let s = SignTuple::new(0.02, 0.0, 0.0);
            assert_eq!(e.evaluate(&s, &env(), RobotContext::ArmOperating), GrammarState::Admissible);
        }
    }

    #[test]
    fn violation_committed_after_hysteresis() {
        let mut e = GrammarEvaluator::<4>::new();
        let big = SignTuple::new(0.15, 0.0, 0.0);
        e.evaluate(&big, &env(), RobotContext::ArmOperating);
        let s = e.evaluate(&big, &env(), RobotContext::ArmOperating);
        assert_eq!(s, GrammarState::Violation);
    }

    #[test]
    fn single_transient_dismissed_by_hysteresis() {
        let mut e = GrammarEvaluator::<4>::new();
        let big = SignTuple::new(0.15, 0.0, 0.0);
        let small = SignTuple::new(0.02, 0.0, 0.0);
        e.evaluate(&big, &env(), RobotContext::ArmOperating);
        let s = e.evaluate(&small, &env(), RobotContext::ArmOperating);
        assert_eq!(s, GrammarState::Admissible, "single transient must be dismissed");
    }

    #[test]
    fn commissioning_suppresses_violations() {
        let mut e = GrammarEvaluator::<4>::new();
        let huge = SignTuple::new(1_000.0, 50.0, 5.0);
        for _ in 0..5 {
            assert_eq!(e.evaluate(&huge, &env(), RobotContext::ArmCommissioning), GrammarState::Admissible);
        }
    }

    #[test]
    fn sustained_outward_drift_is_boundary() {
        let mut e = GrammarEvaluator::<4>::new();
        let drift = SignTuple::new(0.07, 0.005, 0.0);
        e.evaluate(&drift, &env(), RobotContext::ArmOperating);
        let s = e.evaluate(&drift, &env(), RobotContext::ArmOperating);
        assert_eq!(s, GrammarState::Boundary(ReasonCode::SustainedOutwardDrift));
    }

    #[test]
    fn abrupt_slew_is_boundary_when_in_approach_band() {
        let mut e = GrammarEvaluator::<4>::new();
        // Norm 0.08 > 0.5·ρ(=0.05) and slew magnitude > δ_s (0.05).
        let s_in = SignTuple::new(0.08, 0.0, 0.2);
        e.evaluate(&s_in, &env(), RobotContext::ArmOperating);
        let s = e.evaluate(&s_in, &env(), RobotContext::ArmOperating);
        assert_eq!(s, GrammarState::Boundary(ReasonCode::AbruptSlewViolation));
    }

    #[test]
    fn recurrent_grazing_detected_after_k_hits() {
        let mut e = GrammarEvaluator::<3>::new();
        // Boundary approach without outward drift → only grazing triggers Boundary.
        let graze = SignTuple::new(0.07, 0.0, 0.0);
        // Need ≥ K = 3 approaches in history, then confirmation.
        for _ in 0..5 {
            e.evaluate(&graze, &env(), RobotContext::ArmOperating);
        }
        assert_eq!(e.state(), GrammarState::Boundary(ReasonCode::RecurrentBoundaryGrazing));
    }

    #[test]
    fn severity_monotone_with_state() {
        assert!(GrammarState::Violation.severity() > GrammarState::Boundary(ReasonCode::EnvelopeViolation).severity());
        assert!(GrammarState::Boundary(ReasonCode::SustainedOutwardDrift).severity() > GrammarState::Admissible.severity());
    }

    #[test]
    fn labels_are_stable() {
        assert_eq!(GrammarState::Admissible.label(), "Admissible");
        assert_eq!(GrammarState::Boundary(ReasonCode::SustainedOutwardDrift).label(), "Boundary");
        assert_eq!(GrammarState::Violation.label(), "Violation");
    }
}
