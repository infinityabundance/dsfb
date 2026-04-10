//! Episode formation: the operator-facing output object.
//!
//! An episode is a contiguous interval of non-Admissible grammar state,
//! annotated with reason codes, drift/slew statistics, and audit trace.

use crate::core::grammar::GrammarState;
use crate::core::heuristics::EngineReasonCode;

/// A DSFB episode: the operator-facing output object.
///
/// Each episode represents a contiguous interval of structural concern.
/// It is the primary deliverable to maintenance analysts and fleet managers.
#[derive(Debug, Clone, Copy)]
pub struct Episode {
    /// Engine unit number.
    pub unit: u16,
    /// First cycle of the episode.
    pub start_cycle: u32,
    /// Last cycle of the episode (or current cycle if ongoing).
    pub end_cycle: u32,
    /// Peak grammar state reached during the episode.
    pub peak_state: GrammarState,
    /// Primary reason code.
    pub reason_code: EngineReasonCode,
    /// Maximum absolute drift observed during the episode.
    pub max_drift: f64,
    /// Maximum absolute slew observed during the episode.
    pub max_slew: f64,
    /// Number of cycles in the episode.
    pub duration_cycles: u32,
    /// Channel index that first triggered the episode.
    pub trigger_channel: u8,
}

impl Episode {
    /// Whether this episode represents a structural anomaly worth reviewing.
    #[must_use]
    pub fn is_reviewable(&self) -> bool {
        self.peak_state.severity() >= GrammarState::Boundary.severity()
    }

    /// Whether this episode should be escalated to maintenance.
    #[must_use]
    pub fn is_escalation(&self) -> bool {
        self.peak_state == GrammarState::Violation
    }
}
