//! Advisory policy layer: map a grammar state to a
//! [`PolicyDecision`] (`Silent`, `Review`, `Escalate`).
//!
//! DSFB is an observer, not a controller. The policy layer emits an
//! **advisory** decision for the operator review surface. It never
//! drives actuation.

use crate::grammar::GrammarState;

/// Operator-facing advisory decision.
///
/// `Silent` is the default: no attention warranted. `Review` is a
/// structured event that merits inspection in the operator dashboard.
/// `Escalate` is a confirmed envelope exit — the operator should treat
/// it as evidence that the upstream incumbent's nominal-operation
/// assumption is no longer valid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PolicyDecision {
    /// No operator attention required.
    #[default]
    Silent,
    /// Structured event: inspect in the review surface.
    Review,
    /// Confirmed envelope exit: operator should treat as evidence
    /// of a nominal-assumption invalidation.
    Escalate,
}

impl PolicyDecision {
    /// Stable label for the canonical `Episode::decision` field.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Silent => "Silent",
            Self::Review => "Review",
            Self::Escalate => "Escalate",
        }
    }

    /// Map a grammar state to its default advisory decision.
    ///
    /// Deterministic: no per-call state; identical inputs map to
    /// identical outputs.
    #[inline]
    #[must_use]
    pub const fn from_grammar(state: GrammarState) -> Self {
        match state {
            GrammarState::Admissible => Self::Silent,
            GrammarState::Boundary(_) => Self::Review,
            GrammarState::Violation => Self::Escalate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarState, ReasonCode};

    #[test]
    fn admissible_maps_to_silent() {
        assert_eq!(PolicyDecision::from_grammar(GrammarState::Admissible), PolicyDecision::Silent);
    }

    #[test]
    fn boundary_maps_to_review_regardless_of_reason() {
        for r in [
            ReasonCode::SustainedOutwardDrift,
            ReasonCode::AbruptSlewViolation,
            ReasonCode::RecurrentBoundaryGrazing,
            ReasonCode::EnvelopeViolation,
        ] {
            assert_eq!(PolicyDecision::from_grammar(GrammarState::Boundary(r)), PolicyDecision::Review);
        }
    }

    #[test]
    fn violation_maps_to_escalate() {
        assert_eq!(PolicyDecision::from_grammar(GrammarState::Violation), PolicyDecision::Escalate);
    }

    #[test]
    fn labels_are_canonical() {
        assert_eq!(PolicyDecision::Silent.label(), "Silent");
        assert_eq!(PolicyDecision::Review.label(), "Review");
        assert_eq!(PolicyDecision::Escalate.label(), "Escalate");
    }
}
