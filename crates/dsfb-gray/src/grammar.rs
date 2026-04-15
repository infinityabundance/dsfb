//! Grammar state machine: structural classification of residual trajectories.
//!
//! The grammar layer maps envelope positions to interpretable states
//! (`Admissible`, `Boundary`, `Violation`) with hysteresis to prevent
//! chattering at envelope boundaries.

use crate::envelope::EnvelopePosition;

/// Grammar state: the structural classification output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrammarState {
    /// Residual trajectory is within the envelope interior.
    /// No action required.
    Admissible,
    /// Residual trajectory is in the boundary zone.
    /// Structural shift detected; monitoring escalated.
    Boundary,
    /// Residual trajectory has exited the envelope.
    /// Structural violation confirmed; intervention warranted.
    Violation,
}

impl GrammarState {
    /// Returns the severity level (0=Admissible, 1=Boundary, 2=Violation).
    pub fn severity(&self) -> u8 {
        match self {
            Self::Admissible => 0,
            Self::Boundary => 1,
            Self::Violation => 2,
        }
    }
}

/// A grammar state transition event.
#[derive(Debug, Clone, Copy)]
pub struct GrammarTransition {
    /// Previous grammar state.
    pub from: GrammarState,
    /// New grammar state.
    pub to: GrammarState,
    /// Timestamp of the transition (nanoseconds).
    pub timestamp_ns: u64,
    /// Number of consecutive observations in the new position before
    /// the transition was confirmed (hysteresis count).
    pub confirmation_count: u32,
}

/// Grammar state machine with hysteresis.
///
/// Transitions require `hysteresis_count` consecutive observations in
/// the new position before the grammar state changes. This prevents
/// chattering at envelope boundaries (Failure Mode FM-02).
pub struct GrammarMachine {
    current_state: GrammarState,
    pending_position: Option<EnvelopePosition>,
    consecutive_count: u32,
    hysteresis_count: u32,
    last_transition_ts: u64,
}

impl GrammarMachine {
    /// Create a new grammar machine starting in `Admissible` state.
    ///
    /// `hysteresis_count` is the number of consecutive observations required
    /// to confirm a state transition. Recommended: 3–10 depending on
    /// sampling rate and noise characteristics.
    pub fn new(hysteresis_count: u32) -> Self {
        Self {
            current_state: GrammarState::Admissible,
            pending_position: None,
            consecutive_count: 0,
            hysteresis_count: hysteresis_count.max(1),
            last_transition_ts: 0,
        }
    }

    /// Process an envelope position and return the current grammar state
    /// and any transition that occurred.
    pub fn step(
        &mut self,
        position: EnvelopePosition,
        timestamp_ns: u64,
    ) -> (GrammarState, Option<GrammarTransition>) {
        let target_state = match position {
            EnvelopePosition::Interior => GrammarState::Admissible,
            EnvelopePosition::BoundaryZone => GrammarState::Boundary,
            EnvelopePosition::Exterior => GrammarState::Violation,
        };

        if target_state == self.current_state {
            // Already in the correct state; reset pending
            self.pending_position = None;
            self.consecutive_count = 0;
            return (self.current_state, None);
        }

        // Escalation (toward Violation) requires hysteresis
        // De-escalation (toward Admissible) also requires hysteresis
        // to prevent flickering during recovery
        match self.pending_position {
            Some(pending) if position_to_state(pending) == target_state => {
                self.consecutive_count += 1;
            }
            None
            | Some(EnvelopePosition::Interior)
            | Some(EnvelopePosition::BoundaryZone)
            | Some(EnvelopePosition::Exterior) => {
                self.pending_position = Some(position);
                self.consecutive_count = 1;
            }
        }

        if self.consecutive_count >= self.hysteresis_count {
            let transition = GrammarTransition {
                from: self.current_state,
                to: target_state,
                timestamp_ns,
                confirmation_count: self.consecutive_count,
            };
            self.current_state = target_state;
            self.pending_position = None;
            self.consecutive_count = 0;
            self.last_transition_ts = timestamp_ns;
            (self.current_state, Some(transition))
        } else {
            (self.current_state, None)
        }
    }

    /// Current grammar state.
    pub fn state(&self) -> GrammarState {
        self.current_state
    }

    /// Reset the machine to Admissible. Used on system restart or
    /// phase transition.
    pub fn reset(&mut self) {
        self.current_state = GrammarState::Admissible;
        self.pending_position = None;
        self.consecutive_count = 0;
    }
}

fn position_to_state(pos: EnvelopePosition) -> GrammarState {
    match pos {
        EnvelopePosition::Interior => GrammarState::Admissible,
        EnvelopePosition::BoundaryZone => GrammarState::Boundary,
        EnvelopePosition::Exterior => GrammarState::Violation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteresis_prevents_premature_transition() {
        let mut machine = GrammarMachine::new(3);

        // Two boundary observations — not enough for transition
        let (state, trans) = machine.step(EnvelopePosition::BoundaryZone, 1);
        assert_eq!(state, GrammarState::Admissible);
        assert!(trans.is_none());

        let (state, trans) = machine.step(EnvelopePosition::BoundaryZone, 2);
        assert_eq!(state, GrammarState::Admissible);
        assert!(trans.is_none());

        // Third boundary observation — transition confirmed
        let (state, trans) = machine.step(EnvelopePosition::BoundaryZone, 3);
        assert_eq!(state, GrammarState::Boundary);
        assert!(trans.is_some());
        let t = trans.unwrap();
        assert_eq!(t.from, GrammarState::Admissible);
        assert_eq!(t.to, GrammarState::Boundary);
    }

    #[test]
    fn test_interrupted_hysteresis_resets() {
        let mut machine = GrammarMachine::new(3);

        machine.step(EnvelopePosition::BoundaryZone, 1);
        machine.step(EnvelopePosition::BoundaryZone, 2);
        // Interior interrupts the pending transition
        machine.step(EnvelopePosition::Interior, 3);

        // Restart boundary sequence
        machine.step(EnvelopePosition::BoundaryZone, 4);
        machine.step(EnvelopePosition::BoundaryZone, 5);
        let (state, _) = machine.step(EnvelopePosition::BoundaryZone, 6);
        assert_eq!(state, GrammarState::Boundary);
    }
}
