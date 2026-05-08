//! DSFB-Debug: grammar evaluation — paper §5.5.
//!
//! Maps the per-(window, signal) `SignTuple` σ(k) = (‖r‖, ṙ, r̈)
//! to one of three `GrammarState` symbols:
//!
//! - **Admissible** — within envelope; drift inward or bounded
//! - **Boundary** — approaching envelope with sustained outward
//!   drift (early-warning state)
//! - **Violation** — exited envelope (structural fault / actionable
//!   incident)
//!
//! The decision is a pure function of σ(k) and the operator-supplied
//! grammar thresholds (`drift_threshold`, `slew_threshold`,
//! `envelope_radius`). The downstream `policy.rs` applies hysteresis
//! (n_confirm windows) before producing the operator-facing
//! `confirmed_grammar_state`.
//!
//! The grammar is the load-bearing structural-detectability mechanism
//! (paper Law 1): a trajectory that drifts persistently outward is
//! more detectable — and more interpretively significant — than one
//! that oscillates at larger amplitude within admissible bounds.

use crate::types::{GrammarState, ReasonCode, SignTuple};
use crate::config::EngineConfig;
use crate::envelope;

/// Evaluate raw grammar state from sign tuple and envelope parameters.
///
/// - Violation: norm > ρ
/// - Boundary: norm > boundary_fraction * ρ AND (sustained outward drift OR high slew)
/// - Admissible: otherwise
#[inline]
pub fn evaluate_raw_grammar(
    sign: &SignTuple,
    rho: f64,
    config: &EngineConfig,
    drift_persistence: f64,
) -> (GrammarState, ReasonCode) {
    // Violation check — hard exit, no hysteresis required
    if envelope::is_violation(sign.norm, rho) {
        if sign.slew > config.slew_delta || sign.slew < -config.slew_delta {
            return (GrammarState::Violation, ReasonCode::AbruptSlewViolation);
        }
        return (GrammarState::Violation, ReasonCode::EnvelopeViolation);
    }

    // Boundary check — in boundary zone with structural evidence
    if envelope::is_boundary_zone(sign.norm, rho, config.boundary_fraction) {
        // Sustained outward drift
        if sign.drift > 0.0 && drift_persistence > 0.5 {
            return (GrammarState::Boundary, ReasonCode::SustainedOutwardDrift);
        }
        // Abrupt slew while in boundary zone
        if sign.slew > config.slew_delta || sign.slew < -config.slew_delta {
            return (GrammarState::Boundary, ReasonCode::AbruptSlewViolation);
        }
        // Recurrent grazing (norm repeatedly in boundary zone)
        return (GrammarState::Boundary, ReasonCode::BoundaryApproach);
    }

    // Admissible
    (GrammarState::Admissible, ReasonCode::Admissible)
}

/// Apply hysteresis confirmation: raw state must persist for n_confirm
/// consecutive windows before being committed.
///
/// `recent_raw_states` should contain the last n_confirm raw states
/// (most recent last).
#[inline]
pub fn hysteresis_confirm(
    recent_raw_states: &[GrammarState],
    n_confirm: usize,
) -> GrammarState {
    if recent_raw_states.len() < n_confirm {
        return GrammarState::Admissible;
    }

    let latest = recent_raw_states[recent_raw_states.len() - 1];

    // Violation is immediate — no hysteresis (hard exit)
    if latest == GrammarState::Violation {
        return GrammarState::Violation;
    }

    // Boundary requires n_confirm consecutive Boundary-or-higher
    if latest == GrammarState::Boundary {
        let start = recent_raw_states.len() - n_confirm;
        let mut all_boundary_or_higher = true;
        let mut i = start;
        while i < recent_raw_states.len() {
            if recent_raw_states[i] == GrammarState::Admissible {
                all_boundary_or_higher = false;
            }
            i += 1;
        }
        if all_boundary_or_higher {
            return GrammarState::Boundary;
        }
    }

    GrammarState::Admissible
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_detection() {
        let sign = SignTuple { norm: 4.0, drift: 0.1, slew: 0.0 };
        let config = EngineConfig::default();
        let (state, _) = evaluate_raw_grammar(&sign, 3.0, &config, 0.5);
        assert_eq!(state, GrammarState::Violation);
    }

    #[test]
    fn test_boundary_with_drift() {
        let sign = SignTuple { norm: 2.0, drift: 0.5, slew: 0.0 };
        let config = EngineConfig::default();
        let (state, reason) = evaluate_raw_grammar(&sign, 3.0, &config, 0.8);
        assert_eq!(state, GrammarState::Boundary);
        assert_eq!(reason, ReasonCode::SustainedOutwardDrift);
    }

    #[test]
    fn test_admissible() {
        let sign = SignTuple { norm: 0.5, drift: 0.0, slew: 0.0 };
        let config = EngineConfig::default();
        let (state, _) = evaluate_raw_grammar(&sign, 3.0, &config, 0.0);
        assert_eq!(state, GrammarState::Admissible);
    }

    #[test]
    fn test_hysteresis_blocks_transient() {
        let states = [GrammarState::Admissible, GrammarState::Boundary];
        assert_eq!(hysteresis_confirm(&states, 2), GrammarState::Admissible);
    }

    #[test]
    fn test_hysteresis_confirms_sustained() {
        let states = [GrammarState::Boundary, GrammarState::Boundary];
        assert_eq!(hysteresis_confirm(&states, 2), GrammarState::Boundary);
    }

    #[test]
    fn test_violation_bypasses_hysteresis() {
        let states = [GrammarState::Admissible, GrammarState::Violation];
        assert_eq!(hysteresis_confirm(&states, 2), GrammarState::Violation);
    }
}
