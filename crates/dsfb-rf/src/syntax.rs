//! Syntax layer: classify sign tuples into named temporal motifs.
//!
//! The syntax layer sits between the sign tuple and the grammar state.
//! It maps σ(k) = (‖r‖, ṙ, r̈) onto a named `MotifClass`, which
//! the heuristics bank then looks up to retrieve operator-facing
//! semantic dispositions and provenance.
//!
//! ## RF Motif Classes (paper §V-F)
//!
//! | Class | Signature | RF Context |
//! |---|---|---|
//! | PreFailureSlowDrift | norm in [0.3ρ,ρ], ṙ>0 sustained | PA thermal drift, LO aging |
//! | TransientExcursion | norm > ρ for 1–2 obs, rapid recovery | Single-sample noise spike |
//! | RecurrentBoundaryApproach | periodic boundary entries | Cyclic interference |
//! | AbruptOnset | r̈ >> 0 with large norm jump | Jamming onset, HW fault |
//! | SpectralMaskApproach | norm drifting toward 1.0 (normalized mask) | TX power creep |
//! | PhaseNoiseExcursion | oscillatory ṙ with growing |ṙ| | Oscillator aging |
//! | FreqHopTransition | abrupt slew + recovery to new baseline | FHSS waveform boundary |
//! | Unknown | no pattern matches | Endoductive: return σ(k) for operator |

use crate::sign::SignTuple;
use crate::grammar::GrammarState;

/// Named temporal motif class.
///
/// The syntax layer maps (sign_tuple, grammar_state) → MotifClass.
/// This is the input to the heuristics bank lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MotifClass {
    /// Persistent positive ṙ while norm approaches ρ.
    /// Primary pre-transition precursor motif.
    PreFailureSlowDrift,
    /// Brief norm spike above ρ with rapid recovery (< 2 observations).
    TransientExcursion,
    /// Repeated near-boundary excursions in a rolling window.
    RecurrentBoundaryApproach,
    /// Abrupt large slew: |r̈| > δ_abrupt.
    /// Consistent with jamming onset or hardware fault.
    AbruptOnset,
    /// Monotone outward drift toward normalized mask boundary (norm → 1.0).
    SpectralMaskApproach,
    /// Oscillatory ṙ with growing amplitude.
    /// Consistent with phase noise or oscillator aging.
    PhaseNoiseExcursion,
    /// Abrupt slew followed by rapid stabilization at a new norm baseline.
    /// Consistent with FHSS waveform transition (should be suppressed by platform context).
    FreqHopTransition,
    /// No motif pattern matched. Endoductive regime.
    /// Operator receives the full σ(k) trajectory; DSFB returns semantic Unknown.
    Unknown,
    /// Monotone linear norm increase with near-zero second derivative.
    /// Signature: ṙ > threshold, |r̈| ≈ 0 (constant-rate gain ramp).
    /// RF context: LNA thermal runaway, progressive gain collapse.
    /// Structurally distinct from `PreFailureSlowDrift`: the gain ramp is
    /// linear (no acceleration) and starts below 30% ρ.
    LnaGainInstability,
    /// Recurrent boundary grazing with oscillatory slew pattern.
    /// Signature: `RecurrentBoundaryGrazing` reason code AND |r̈| > 0.
    /// RF context: LO phase noise excursion, oscillator aging or vibration.
    /// Carries an Allan-deviation instability character distinguishable from
    /// `RecurrentBoundaryApproach` (which has no oscillatory slew).
    LoInstabilityPrecursor,
}

/// Thresholds for syntax classification. All are dimensionless fractions
/// of ρ or absolute rates, set from the paper's Stage III protocol.
#[derive(Debug, Clone, Copy)]
pub struct SyntaxThresholds {
    /// Minimum drift rate to qualify as SlowDrift motif.
    pub drift_threshold: f32,
    /// Minimum |r̈| to qualify as AbruptOnset motif.
    pub abrupt_slew_threshold: f32,
    /// Norm fraction above which SpectralMaskApproach is considered.
    pub mask_approach_frac: f32,
    /// Maximum norm for TransientExcursion (above rho but recovers).
    pub transient_max_overshoot: f32,
}

impl Default for SyntaxThresholds {
    fn default() -> Self {
        Self {
            drift_threshold: 0.002,
            abrupt_slew_threshold: 0.05,
            mask_approach_frac: 0.80,
            transient_max_overshoot: 2.0, // up to 2× ρ for transient
        }
    }
}

/// Classify a sign tuple and grammar state into a named motif.
///
/// This is a pure deterministic function: identical inputs always
/// produce identical outputs (Theorem 9 of the paper).
pub fn classify(
    sign: &SignTuple,
    grammar: GrammarState,
    rho: f32,
    thresholds: &SyntaxThresholds,
) -> MotifClass {
    try_violation_motif(sign, grammar, rho, thresholds)
        .or_else(|| try_recurrent_grazing_motif(sign, grammar, thresholds))
        .or_else(|| try_boundary_drift_motif(sign, grammar, rho, thresholds))
        .or_else(|| try_boundary_slew_motif(sign, grammar, rho, thresholds))
        .unwrap_or(MotifClass::Unknown)
}

fn try_violation_motif(
    sign: &SignTuple, grammar: GrammarState, rho: f32, thresholds: &SyntaxThresholds,
) -> Option<MotifClass> {
    if grammar.is_violation() && sign.slew.abs() > thresholds.abrupt_slew_threshold {
        return Some(MotifClass::AbruptOnset);
    }
    if grammar.is_violation()
        && sign.norm < rho * thresholds.transient_max_overshoot
        && sign.drift.abs() < thresholds.drift_threshold * 5.0
    {
        return Some(MotifClass::TransientExcursion);
    }
    None
}

fn try_recurrent_grazing_motif(
    sign: &SignTuple, grammar: GrammarState, thresholds: &SyntaxThresholds,
) -> Option<MotifClass> {
    if let GrammarState::Boundary(crate::grammar::ReasonCode::RecurrentBoundaryGrazing) = grammar {
        if sign.slew.abs() > thresholds.drift_threshold * 1.5 {
            return Some(MotifClass::LoInstabilityPrecursor);
        }
        return Some(MotifClass::RecurrentBoundaryApproach);
    }
    None
}

fn try_boundary_drift_motif(
    sign: &SignTuple, grammar: GrammarState, rho: f32, thresholds: &SyntaxThresholds,
) -> Option<MotifClass> {
    if !grammar.is_boundary() { return None; }
    if sign.drift > thresholds.drift_threshold
        && sign.norm < rho * 0.30
        && sign.slew.abs() < thresholds.drift_threshold * 0.5
    {
        return Some(MotifClass::LnaGainInstability);
    }
    if sign.drift > thresholds.drift_threshold
        && sign.norm > rho * 0.30
        && sign.norm <= rho
    {
        return Some(MotifClass::PreFailureSlowDrift);
    }
    if sign.norm > rho * thresholds.mask_approach_frac && sign.drift > 0.0 {
        return Some(MotifClass::SpectralMaskApproach);
    }
    None
}

fn try_boundary_slew_motif(
    sign: &SignTuple, grammar: GrammarState, rho: f32, thresholds: &SyntaxThresholds,
) -> Option<MotifClass> {
    if !grammar.is_boundary() { return None; }
    if sign.slew.abs() > thresholds.drift_threshold * 2.0 && sign.norm > rho * 0.3 {
        return Some(MotifClass::PhaseNoiseExcursion);
    }
    if sign.slew.abs() > thresholds.abrupt_slew_threshold * 0.5 {
        return Some(MotifClass::FreqHopTransition);
    }
    None
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarState, ReasonCode};

    fn thresh() -> SyntaxThresholds { SyntaxThresholds::default() }

    #[test]
    fn slow_drift_classified() {
        let sign = SignTuple::new(0.07, 0.005, 0.0001);
        let grammar = GrammarState::Boundary(ReasonCode::SustainedOutwardDrift);
        let motif = classify(&sign, grammar, 0.1, &thresh());
        assert_eq!(motif, MotifClass::PreFailureSlowDrift);
    }

    #[test]
    fn abrupt_onset_classified() {
        let sign = SignTuple::new(0.15, 0.01, 0.1);
        let grammar = GrammarState::Violation;
        let motif = classify(&sign, grammar, 0.1, &thresh());
        assert_eq!(motif, MotifClass::AbruptOnset);
    }

    #[test]
    fn admissible_with_no_drift_is_unknown() {
        let sign = SignTuple::new(0.02, 0.0, 0.0);
        let grammar = GrammarState::Admissible;
        let motif = classify(&sign, grammar, 0.1, &thresh());
        assert_eq!(motif, MotifClass::Unknown);
    }

    #[test]
    fn recurrent_grazing_classified() {
        let sign = SignTuple::new(0.06, 0.001, 0.0);
        let grammar = GrammarState::Boundary(ReasonCode::RecurrentBoundaryGrazing);
        let motif = classify(&sign, grammar, 0.1, &thresh());
        assert_eq!(motif, MotifClass::RecurrentBoundaryApproach);
    }
}
