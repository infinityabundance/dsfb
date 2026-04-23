//! Grammar FSM: Admissible | Boundary[ReasonCode] | Violation
//!
//! ## Mathematical Definition (paper §B.4, §V-C)
//!
//! State assignment rules (per observation k):
//! - Violation:  ‖r(k)‖ > ρ_eff
//! - Boundary:   ‖r(k)‖ > 0.5ρ_eff  AND  (ṙ(k) > 0 OR |r̈(k)| > δ_s)
//!   OR:         recurrent near-boundary hits ≥ K in window W
//! - Admissible: otherwise
//!
//! Hysteresis: 2 consecutive confirmations required before a state
//! change is committed. Sub-threshold observations forced to Admissible.
//!
//! ## Design Note
//!
//! This is a single canonical FSM implementation. The semiconductor crate
//! had the dual-FSM defect (3-state batch vs. 6-state streaming).
//! This crate has exactly one FSM: the 3-state typed grammar above.
//! No alternative implementations exist in this module.

use crate::envelope::AdmissibilityEnvelope;
use crate::sign::SignTuple;
use crate::platform::WaveformState;

/// Reason code qualifying a Boundary grammar state.
///
/// Typed reason codes allow operators to distinguish classes of structural
/// behavior without modulation classification (paper Table II).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReasonCode {
    /// Persistent positive ṙ over W consecutive observations.
    /// RF contexts: PA thermal drift, LO aging, slow interference buildup.
    SustainedOutwardDrift,
    /// Abrupt |r̈| > δ_s event.
    /// RF contexts: jamming onset, hardware fault, LO phase jump.
    AbruptSlewViolation,
    /// Recurrent near-boundary hits ≥ K in window W.
    /// RF contexts: cyclic interference, periodic spectral sharing.
    RecurrentBoundaryGrazing,
    /// Confirmed ‖r(k)‖ > ρ_eff.
    EnvelopeViolation,
}

/// The DSFB grammar state — the typed intermediate representation.
///
/// This is what operators see instead of a scalar alarm count.
/// The typed state encodes both the severity and the structural character
/// of the observed residual trajectory.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GrammarState {
    /// Residual within envelope, drift inward or bounded. Nominal operation.
    Admissible,
    /// Residual approaching envelope boundary with sustained outward drift
    /// or recurrent grazing. Early-warning state.
    Boundary(ReasonCode),
    /// Residual has exited envelope. Structural fault state.
    Violation,
}

impl GrammarState {
    /// Returns true if this state warrants any operator attention.
    #[inline]
    pub fn requires_attention(&self) -> bool {
        !matches!(self, GrammarState::Admissible)
    }

    /// Returns true if this is a Violation state.
    #[inline]
    pub fn is_violation(&self) -> bool {
        matches!(self, GrammarState::Violation)
    }

    /// Returns true if this is a Boundary state.
    #[inline]
    pub fn is_boundary(&self) -> bool {
        matches!(self, GrammarState::Boundary(_))
    }

    /// Severity level: 0=Admissible, 1=Boundary, 2=Violation.
    #[inline]
    pub fn severity(&self) -> u8 {
        match self {
            GrammarState::Admissible => 0,
            GrammarState::Boundary(_) => 1,
            GrammarState::Violation => 2,
        }
    }

    /// Severity-based trust scalar T ∈ [0, 1].
    ///
    /// Returns a deterministic, bounded trust weight that downstream stages
    /// can use to *down-weight* grammar evidence that is already at
    /// boundary or violation.  This is the semiotics-engine `trust_scalar_for()`
    /// severity dimension (de Beer 2026, §IV):
    ///
    /// - Admissible  → 1.0  (full trust: nominal region, no structural concern)
    /// - Boundary    → 0.5  (half trust: approach region, evidence partial)
    /// - Violation   → 0.0  (no trust: outside envelope, evidence suppressed)
    ///
    /// Use `geometry_trust()` for a continuous, geometry-aware version.
    #[inline]
    pub fn severity_trust(&self) -> f32 {
        match self {
            GrammarState::Admissible       => 1.0,
            GrammarState::Boundary(_)      => 0.5,
            GrammarState::Violation        => 0.0,
        }
    }

    /// Geometry-based grammar trust scalar T ∈ [0, 1].
    ///
    /// Provides a *continuous* trust measure based on how far inside the
    /// admissibility envelope the current residual norm lies, within the
    /// boundary band.  Derived from semiotics-engine eq. (trust_scalar_for):
    ///
    /// ```text
    /// margin        = (ρ − ‖r‖) / ρ              (normalised inward distance)
    /// T             = clamp(margin / band_frac, 0, 1)
    /// ```
    ///
    /// `band_frac` is the boundary band width as a fraction of ρ
    /// (semiotics-engine default: 0.04 = 4 %).
    ///
    /// ## Semantics
    ///
    /// - T = 1.0: residual deep inside envelope — full confidence
    /// - T ≈ 0.5: residual halfway through the boundary band
    /// - T = 0.0: residual at or outside the envelope boundary — suppressed
    ///
    /// Independent of grammar state: can be used even when the FSM is in
    /// Admissible but the norm is close to ρ.
    #[inline]
    pub fn geometry_trust(norm: f32, rho: f32, band_frac: f32) -> f32 {
        if rho <= 1e-30 { return 0.0; }
        let margin = (rho - norm) / rho;
        if band_frac < 1e-12 {
            return if margin >= 0.0 { 1.0 } else { 0.0 };
        }
        let t = margin / band_frac;
        t.max(0.0).min(1.0)
    }

    /// Combined grammar trust scalar: minimum of severity_trust and geometry_trust.
    ///
    /// Takes the more conservative of the two trust dimensions.
    /// This is the recommended scalar for downstream weighting (e.g., DSA
    /// score blending, HRET combination).
    #[inline]
    pub fn combined_trust(&self, norm: f32, rho: f32, band_frac: f32) -> f32 {
        let st = self.severity_trust();
        let gt = GrammarState::geometry_trust(norm, rho, band_frac);
        st.min(gt)
    }
}

/// Grammar evaluator with hysteresis and boundary-grazing history.
///
/// Generic `W` = drift window, `K` = persistence threshold.
/// All state is stack-allocated; no heap, no unsafe.
pub struct GrammarEvaluator<const K: usize> {
    /// Pending (unconfirmed) grammar state awaiting hysteresis confirmation.
    pending: GrammarState,
    /// Confirmation counter for current pending state (0..=2).
    confirmations: u8,
    /// Confirmed (committed) grammar state.
    committed: GrammarState,
    /// Circular buffer of recent boundary-approach flags for grazing detection.
    boundary_hits: [bool; K],
    /// Write head for boundary_hits buffer.
    hit_head: usize,
    /// Number of boundary hits inserted so far (saturates at K).
    hit_count: usize,
}

impl<const K: usize> GrammarEvaluator<K> {
    /// Create a new evaluator initialized to Admissible.
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

    /// Current committed grammar state (after hysteresis).
    #[inline]
    pub fn state(&self) -> GrammarState {
        self.committed
    }

    /// Evaluate the grammar state for one observation.
    ///
    /// Returns the committed grammar state after applying hysteresis.
    /// If the waveform state is suppressed, forces Admissible.
    pub fn evaluate(
        &mut self,
        sign: &SignTuple,
        envelope: &AdmissibilityEnvelope,
        waveform_state: WaveformState,
    ) -> GrammarState {
        // Suppressed window: force Admissible (paper §XIV-C, §B.4)
        if waveform_state.is_suppressed() {
            self.committed = GrammarState::Admissible;
            self.pending = GrammarState::Admissible;
            self.confirmations = 0;
            return GrammarState::Admissible;
        }

        let multiplier = waveform_state.admissibility_multiplier();
        let raw_state = self.compute_raw_state(sign, envelope, multiplier);

        // Update boundary-grazing history
        let is_boundary_approach = envelope.is_boundary_approach(sign.norm, multiplier)
            && !envelope.is_violation(sign.norm, multiplier);
        self.boundary_hits[self.hit_head] = is_boundary_approach;
        self.hit_head = (self.hit_head + 1) % K;
        if self.hit_count < K { self.hit_count += 1; }

        // Apply hysteresis: require 2 consecutive confirmations (paper §B.4)
        if raw_state == self.pending {
            if self.confirmations < 2 {
                self.confirmations += 1;
            }
            if self.confirmations >= 2 {
                self.committed = raw_state;
            }
        } else {
            self.pending = raw_state;
            self.confirmations = 1;
        }

        self.committed
    }

    /// Compute raw grammar state (before hysteresis).
    fn compute_raw_state(
        &self,
        sign: &SignTuple,
        envelope: &AdmissibilityEnvelope,
        multiplier: f32,
    ) -> GrammarState {
        // Violation check first (hardest condition)
        if envelope.is_violation(sign.norm, multiplier) {
            return GrammarState::Violation;
        }

        // Boundary: outward drift
        if envelope.is_boundary_approach(sign.norm, multiplier) {
            if sign.is_outward_drift() {
                return GrammarState::Boundary(ReasonCode::SustainedOutwardDrift);
            }
            if sign.is_abrupt_slew(envelope.delta_s) {
                return GrammarState::Boundary(ReasonCode::AbruptSlewViolation);
            }
        }

        // Boundary: recurrent grazing — K hits in the last K observations
        let grazing_hits = self.boundary_hits.iter().filter(|&&h| h).count();
        if self.hit_count >= K && grazing_hits >= K {
            return GrammarState::Boundary(ReasonCode::RecurrentBoundaryGrazing);
        }

        GrammarState::Admissible
    }

    /// Reset the evaluator (e.g., after a post-transition guard expires).
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl<const K: usize> Default for GrammarEvaluator<K> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::AdmissibilityEnvelope;
    use crate::sign::SignTuple;
    use crate::platform::WaveformState;

    fn make_envelope() -> AdmissibilityEnvelope {
        AdmissibilityEnvelope::new(0.1)
    }

    #[test]
    fn clean_signal_is_admissible() {
        let mut eval = GrammarEvaluator::<4>::new();
        let env = make_envelope();
        for _ in 0..5 {
            let sig = SignTuple::new(0.02, 0.0, 0.0);
            let state = eval.evaluate(&sig, &env, WaveformState::Operational);
            assert_eq!(state, GrammarState::Admissible);
        }
    }

    #[test]
    fn violation_detected_after_hysteresis() {
        let mut eval = GrammarEvaluator::<4>::new();
        let env = make_envelope();
        // First observation: violation — pending but not yet confirmed
        let sig = SignTuple::new(0.15, 0.02, 0.001);
        let s1 = eval.evaluate(&sig, &env, WaveformState::Operational);
        // After 2 confirmations: committed Violation
        let s2 = eval.evaluate(&sig, &env, WaveformState::Operational);
        assert_eq!(s2, GrammarState::Violation, "s1={:?} s2={:?}", s1, s2);
    }

    #[test]
    fn transient_spike_dismissed_by_hysteresis() {
        let mut eval = GrammarEvaluator::<4>::new();
        let env = make_envelope();
        // Single violation then immediate recovery
        let above = SignTuple::new(0.15, 0.02, 0.0);
        let below = SignTuple::new(0.02, 0.0, 0.0);
        eval.evaluate(&above, &env, WaveformState::Operational);
        // Recovery before 2nd confirmation: hysteresis resets
        let state = eval.evaluate(&below, &env, WaveformState::Operational);
        assert_eq!(state, GrammarState::Admissible,
            "single transient should be dismissed by hysteresis");
    }

    #[test]
    fn transition_suppresses_violation() {
        let mut eval = GrammarEvaluator::<4>::new();
        let env = make_envelope();
        let huge = SignTuple::new(1000.0, 100.0, 10.0);
        // Even a massive residual must produce Admissible during transition
        for _ in 0..5 {
            let state = eval.evaluate(&huge, &env, WaveformState::Transition);
            assert_eq!(state, GrammarState::Admissible);
        }
    }

    #[test]
    fn sustained_outward_drift_detected() {
        let mut eval = GrammarEvaluator::<4>::new();
        let env = make_envelope();
        // Norm in boundary zone (> 0.05), positive drift
        let sig = SignTuple::new(0.07, 0.005, 0.0001);
        eval.evaluate(&sig, &env, WaveformState::Operational);
        let state = eval.evaluate(&sig, &env, WaveformState::Operational);
        assert_eq!(state,
            GrammarState::Boundary(ReasonCode::SustainedOutwardDrift));
    }

    #[test]
    fn grammar_state_severity_ordering() {
        assert!(GrammarState::Violation.severity() >
                GrammarState::Boundary(ReasonCode::SustainedOutwardDrift).severity());
        assert!(GrammarState::Boundary(ReasonCode::EnvelopeViolation).severity() >
                GrammarState::Admissible.severity());
    }

    #[test]
    fn severity_trust_bounded_and_ordered() {
        let t_adm = GrammarState::Admissible.severity_trust();
        let t_bnd = GrammarState::Boundary(ReasonCode::SustainedOutwardDrift).severity_trust();
        let t_vio = GrammarState::Violation.severity_trust();
        assert!((t_adm - 1.0).abs() < 1e-6);
        assert!((t_bnd - 0.5).abs() < 1e-6);
        assert!((t_vio - 0.0).abs() < 1e-6);
        assert!(t_adm > t_bnd);
        assert!(t_bnd > t_vio);
    }

    #[test]
    fn geometry_trust_deep_inside() {
        // norm = 0, rho = 0.10, band = 4% → margin = 1.0 → T = 1.0 / 0.04 → clamped 1.0
        let t = GrammarState::geometry_trust(0.0, 0.10, 0.04);
        assert!((t - 1.0).abs() < 1e-6, "deep inside → T=1.0, got {}", t);
    }

    #[test]
    fn geometry_trust_at_boundary() {
        // norm = rho → margin = 0 → T = 0
        let t = GrammarState::geometry_trust(0.10, 0.10, 0.04);
        assert!((t - 0.0).abs() < 1e-6, "at boundary → T=0.0, got {}", t);
    }

    #[test]
    fn geometry_trust_interpolates() {
        // norm = 0.098, rho = 0.10, band = 4%
        // margin = (0.10 - 0.098) / 0.10 = 0.02
        // T = 0.02 / 0.04 = 0.5
        let t = GrammarState::geometry_trust(0.098, 0.10, 0.04);
        assert!((t - 0.5).abs() < 1e-5, "midpoint of band → T=0.5, got {}", t);
    }

    #[test]
    fn combined_trust_takes_minimum() {
        // Outside envelope: geometry = 0, severity_trust(Violation) = 0 → combined = 0
        let t = GrammarState::Violation.combined_trust(0.15, 0.10, 0.04);
        assert!((t - 0.0).abs() < 1e-6);
        // Deep inside, Admissible → combined = 1.0
        let t2 = GrammarState::Admissible.combined_trust(0.01, 0.10, 0.04);
        assert!((t2 - 1.0).abs() < 1e-6);
    }
}
