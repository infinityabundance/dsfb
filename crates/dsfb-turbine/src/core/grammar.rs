//! Grammar-state machine: Admissible / Boundary / Violation.
//!
//! The grammar layer classifies the structural state of each residual
//! trajectory at each cycle. Transitions are governed by drift persistence,
//! slew persistence, and envelope position. All rules are deterministic.

use crate::core::config::DsfbConfig;
use crate::core::envelope::{AdmissibilityEnvelope, EnvelopeStatus};
use crate::core::residual::ResidualSign;

/// Grammar state: the structural classification of a residual trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarState {
    /// Residuals within envelope, no persistent outward drift.
    Admissible,
    /// Residuals approaching boundary or showing persistent outward drift.
    Boundary,
    /// Residuals have exited the envelope or show sustained acceleration.
    Violation,
}

impl GrammarState {
    /// Numeric severity for ordering (0 = safe, 2 = most severe).
    #[must_use]
    pub const fn severity(self) -> u8 {
        match self {
            Self::Admissible => 0,
            Self::Boundary => 1,
            Self::Violation => 2,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Admissible => "Admissible",
            Self::Boundary => "Boundary",
            Self::Violation => "Violation",
        }
    }
}

/// Single-channel grammar engine. Tracks state and persistence counters.
///
/// Stack-allocated, no heap. Designed for `no_alloc` environments.
#[derive(Debug, Clone, Copy)]
pub struct GrammarEngine {
    state: GrammarState,
    drift_persistence_count: u32,
    slew_persistence_count: u32,
    first_boundary_cycle: Option<u32>,
    first_violation_cycle: Option<u32>,
}

impl GrammarEngine {
    /// Creates a new grammar engine in the Admissible state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: GrammarState::Admissible,
            drift_persistence_count: 0,
            slew_persistence_count: 0,
            first_boundary_cycle: None,
            first_violation_cycle: None,
        }
    }

    /// Current grammar state.
    #[must_use]
    pub const fn state(&self) -> GrammarState {
        self.state
    }

    /// Cycle at which Boundary was first entered.
    #[must_use]
    pub const fn first_boundary_cycle(&self) -> Option<u32> {
        self.first_boundary_cycle
    }

    /// Cycle at which Violation was first entered.
    #[must_use]
    pub const fn first_violation_cycle(&self) -> Option<u32> {
        self.first_violation_cycle
    }

    /// Advances the grammar state by one cycle.
    ///
    /// This is the core deterministic transition function.
    /// Given the residual sign and envelope, it updates the grammar state.
    ///
    /// # Transition Rules
    ///
    /// - **Admissible → Boundary**: sustained outward drift exceeding `drift_floor`
    ///   for `persistence_threshold` consecutive cycles, OR envelope position is
    ///   `Approaching`.
    ///
    /// - **Boundary → Violation**: envelope position is `Exceeded`, OR sustained
    ///   positive slew exceeding `slew_floor` for `slew_persistence_threshold`
    ///   consecutive cycles.
    ///
    /// - **Boundary → Admissible**: drift reverses and envelope position returns
    ///   to `Interior` for `persistence_threshold` consecutive cycles.
    ///
    /// - **Violation**: terminal state within a single evaluation pass.
    ///   (In operational deployment, reset requires explicit maintenance event.)
    pub fn advance(
        &mut self,
        sign: &ResidualSign,
        envelope: &AdmissibilityEnvelope,
        config: &DsfbConfig,
    ) {
        let env_status = envelope.classify_position(sign.residual);
        let drift_outward = sign.drift.abs() > config.drift_floor;
        let slew_positive = sign.slew.abs() > config.slew_floor;

        match self.state {
            GrammarState::Admissible => {
                if env_status == EnvelopeStatus::Exceeded {
                    // Direct jump to Violation on envelope breach
                    self.transition_to(GrammarState::Violation, sign.cycle);
                } else if env_status == EnvelopeStatus::Approaching {
                    self.transition_to(GrammarState::Boundary, sign.cycle);
                } else if drift_outward {
                    self.drift_persistence_count += 1;
                    if self.drift_persistence_count >= config.persistence_threshold as u32 {
                        self.transition_to(GrammarState::Boundary, sign.cycle);
                    }
                } else {
                    self.drift_persistence_count = 0;
                }
            }
            GrammarState::Boundary => {
                if env_status == EnvelopeStatus::Exceeded {
                    self.transition_to(GrammarState::Violation, sign.cycle);
                } else if slew_positive {
                    self.slew_persistence_count += 1;
                    if self.slew_persistence_count >= config.slew_persistence_threshold as u32 {
                        self.transition_to(GrammarState::Violation, sign.cycle);
                    }
                } else if env_status == EnvelopeStatus::Interior && !drift_outward {
                    // Potential recovery: track consecutive non-drift cycles
                    self.drift_persistence_count += 1;
                    if self.drift_persistence_count >= config.persistence_threshold as u32 {
                        self.state = GrammarState::Admissible;
                        self.drift_persistence_count = 0;
                        self.slew_persistence_count = 0;
                    }
                } else {
                    self.drift_persistence_count = 0;
                    if !slew_positive {
                        self.slew_persistence_count = 0;
                    }
                }
            }
            GrammarState::Violation => {
                // Terminal state. No further transitions.
                // In operational deployment, reset requires explicit event.
            }
        }
    }

    /// Internal transition helper.
    fn transition_to(&mut self, new_state: GrammarState, cycle: u32) {
        if new_state == GrammarState::Boundary && self.first_boundary_cycle.is_none() {
            self.first_boundary_cycle = Some(cycle);
        }
        if new_state == GrammarState::Violation && self.first_violation_cycle.is_none() {
            self.first_violation_cycle = Some(cycle);
        }
        self.state = new_state;
        self.drift_persistence_count = 0;
        self.slew_persistence_count = 0;
    }
}

/// Multi-channel grammar aggregation.
///
/// Aggregates grammar states from multiple channels using majority vote
/// or configurable fraction threshold.
///
/// Fixed-size array: supports up to `MAX_CHANNELS` channels.
pub const MAX_CHANNELS: usize = 21;

/// Multi-channel grammar result at a single cycle.
#[derive(Debug, Clone, Copy)]
pub struct MultiChannelGrammar {
    /// Per-channel grammar states.
    pub channel_states: [GrammarState; MAX_CHANNELS],
    /// Number of active channels.
    pub active_channels: usize,
    /// Aggregated grammar state (majority vote).
    pub aggregate_state: GrammarState,
    /// Number of channels in Boundary or Violation.
    pub channels_alarming: usize,
}

/// Aggregates per-channel grammar states into a single engine-level state.
#[must_use]
pub fn aggregate_grammar(
    states: &[GrammarState],
    vote_fraction: f64,
) -> GrammarState {
    if states.is_empty() {
        return GrammarState::Admissible;
    }

    let n = states.len();
    let mut violation_count = 0u32;
    let mut boundary_count = 0u32;

    let mut i = 0;
    while i < n {
        match states[i] {
            GrammarState::Violation => violation_count += 1,
            GrammarState::Boundary => boundary_count += 1,
            GrammarState::Admissible => {}
        }
        i += 1;
    }

    let threshold = (n as f64 * vote_fraction) as u32;
    let threshold = if threshold == 0 { 1 } else { threshold };

    if violation_count >= threshold {
        GrammarState::Violation
    } else if (violation_count + boundary_count) >= threshold {
        GrammarState::Boundary
    } else {
        GrammarState::Admissible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let engine = GrammarEngine::new();
        assert_eq!(engine.state(), GrammarState::Admissible);
    }

    #[test]
    fn test_aggregate_all_admissible() {
        let states = [GrammarState::Admissible; 5];
        assert_eq!(aggregate_grammar(&states, 0.3), GrammarState::Admissible);
    }

    #[test]
    fn test_aggregate_majority_boundary() {
        let states = [
            GrammarState::Admissible,
            GrammarState::Boundary,
            GrammarState::Boundary,
            GrammarState::Admissible,
            GrammarState::Admissible,
        ];
        assert_eq!(aggregate_grammar(&states, 0.3), GrammarState::Boundary);
    }
}
