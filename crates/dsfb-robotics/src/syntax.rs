//! Syntax layer — classify residual sign-tuple sequences into named
//! structural motifs from the heuristics bank.
//!
//! Phase 2 provides a minimal classifier that maps a sign tuple and
//! grammar state to either an exact [`crate::heuristics::RoboticsMotif`] match or
//! `RoboticsMotif::Unknown`. Full motif-pattern recognition (periodic
//! grazing detection, Stribeck-plateau fitting, BPFI harmonic
//! classification, GRF-desync autocorrelation, CoM-drift integration)
//! lands incrementally in Phase 3 alongside the dataset adapters that
//! exercise each motif.

use crate::grammar::{GrammarState, ReasonCode};
use crate::heuristics::RoboticsMotif;
use crate::sign::SignTuple;

/// Classify a sign tuple + grammar state into a named motif.
///
/// The Phase 2 classifier makes the minimal, obvious assignments:
///
/// - `GrammarState::Admissible` → `RoboticsMotif::Unknown` (nothing to
///   classify).
/// - `GrammarState::Boundary(RecurrentBoundaryGrazing)` with zero net
///   drift → `RoboticsMotif::BacklashRing` (characteristic of gear
///   backlash at velocity reversals).
/// - `GrammarState::Boundary(SustainedOutwardDrift)` with positive
///   slew → `RoboticsMotif::BpfiGrowth` (degradation trajectory).
/// - Everything else → `RoboticsMotif::Unknown`.
///
/// The `Unknown` fallback is **not** a failure — it is a first-class
/// output that tells the operator "DSFB observed structure but does
/// not have a named motif for it," which is precisely the
/// augment-not-classify posture of the framework.
#[must_use]
pub fn classify(state: GrammarState, sign: &SignTuple) -> RoboticsMotif {
    debug_assert!(sign.norm.is_finite() || sign.norm.is_nan(), "sign norm must be finite or NaN");
    debug_assert!(sign.drift.is_finite() || sign.drift.is_nan(), "sign drift must be finite or NaN");
    debug_assert!(sign.slew.is_finite() || sign.slew.is_nan(), "sign slew must be finite or NaN");
    match state {
        GrammarState::Admissible => RoboticsMotif::Unknown,
        GrammarState::Violation => RoboticsMotif::Unknown,
        GrammarState::Boundary(reason) => match reason {
            ReasonCode::RecurrentBoundaryGrazing => {
                // Grazing with near-zero net drift → backlash ring.
                if crate::math::abs_f64(sign.drift) < 1e-6 {
                    RoboticsMotif::BacklashRing
                } else {
                    RoboticsMotif::Unknown
                }
            }
            ReasonCode::SustainedOutwardDrift => {
                // Outward drift with positive slew (degrading direction) → BPFI growth.
                if sign.slew > 0.0 {
                    RoboticsMotif::BpfiGrowth
                } else {
                    RoboticsMotif::Unknown
                }
            }
            ReasonCode::AbruptSlewViolation | ReasonCode::EnvelopeViolation => {
                RoboticsMotif::Unknown
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admissible_is_unknown() {
        let s = SignTuple::zero();
        assert_eq!(classify(GrammarState::Admissible, &s), RoboticsMotif::Unknown);
    }

    #[test]
    fn grazing_with_zero_drift_is_backlash_ring() {
        let s = SignTuple::new(0.05, 0.0, 0.001);
        let m = classify(GrammarState::Boundary(ReasonCode::RecurrentBoundaryGrazing), &s);
        assert_eq!(m, RoboticsMotif::BacklashRing);
    }

    #[test]
    fn outward_drift_with_positive_slew_is_bpfi_growth() {
        let s = SignTuple::new(0.05, 0.01, 0.002);
        let m = classify(GrammarState::Boundary(ReasonCode::SustainedOutwardDrift), &s);
        assert_eq!(m, RoboticsMotif::BpfiGrowth);
    }

    #[test]
    fn outward_drift_with_flat_slew_is_unknown() {
        let s = SignTuple::new(0.05, 0.01, 0.0);
        let m = classify(GrammarState::Boundary(ReasonCode::SustainedOutwardDrift), &s);
        assert_eq!(m, RoboticsMotif::Unknown);
    }

    #[test]
    fn violation_state_is_unknown_motif() {
        let s = SignTuple::new(0.2, 0.05, 0.01);
        assert_eq!(classify(GrammarState::Violation, &s), RoboticsMotif::Unknown);
    }
}
