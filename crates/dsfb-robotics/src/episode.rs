//! Canonical [`Episode`] struct — the DSFB observer's advisory output.
//!
//! Fields are byte-identical to `dsfb-semiconductor`'s `Episode` so
//! downstream tooling (dashboards, audit exporters, cross-domain
//! aggregators) can consume episodes uniformly across DSFB crates.

use crate::grammar::GrammarState;
use crate::policy::PolicyDecision;

/// A structured episode emitted by the DSFB observer.
///
/// Advisory only. No upstream state is modified by emitting one.
/// String fields are `&'static str` so an episode can be constructed
/// and passed through a `no_alloc` core without requiring an allocator.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Episode {
    /// Sample index within the input slice.
    pub index: usize,
    /// Squared residual norm `‖r‖²`. Squared rather than norm so the
    /// core never needs `sqrt` in the hot path.
    pub residual_norm_sq: f64,
    /// Rolling drift estimate (mean first-difference of absolute
    /// residuals over the drift window).
    pub drift: f64,
    /// Grammar-state label: `"Admissible"`, `"Boundary"`, or
    /// `"Violation"`.
    pub grammar: &'static str,
    /// Policy decision: `"Silent"`, `"Review"`, or `"Escalate"`.
    pub decision: &'static str,
}

impl Episode {
    /// A zero-valued episode suitable for seeding a fixed-capacity
    /// output buffer: `[Episode::empty(); N]`.
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            index: 0,
            residual_norm_sq: 0.0,
            drift: 0.0,
            grammar: "Admissible",
            decision: "Silent",
        }
    }

    /// Build an episode from the current grammar state, policy
    /// decision, and residual sign-tuple components.
    ///
    /// Callers typically pass `residual_norm_sq = norm * norm` where
    /// `norm` is `‖r‖`, matching the semantics of the field name. No
    /// check is made that the value is actually a squared quantity —
    /// the signature is a convention, not an enforcement.
    #[inline]
    #[must_use]
    pub const fn new(
        index: usize,
        residual_norm_sq: f64,
        drift: f64,
        grammar: GrammarState,
        decision: PolicyDecision,
    ) -> Self {
        Self {
            index,
            residual_norm_sq,
            drift,
            grammar: grammar.label(),
            decision: decision.label(),
        }
    }
}

impl Default for Episode {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarState, ReasonCode};
    use crate::policy::PolicyDecision;

    #[test]
    fn empty_is_admissible_silent() {
        let e = Episode::empty();
        assert_eq!(e.grammar, "Admissible");
        assert_eq!(e.decision, "Silent");
        assert_eq!(e.index, 0);
        assert_eq!(e.residual_norm_sq, 0.0);
        assert_eq!(e.drift, 0.0);
    }

    #[test]
    fn new_writes_expected_labels() {
        let e = Episode::new(
            42,
            0.01,
            0.001,
            GrammarState::Boundary(ReasonCode::SustainedOutwardDrift),
            PolicyDecision::Review,
        );
        assert_eq!(e.index, 42);
        assert_eq!(e.grammar, "Boundary");
        assert_eq!(e.decision, "Review");
    }

    #[test]
    fn default_equals_empty() {
        assert_eq!(Episode::default(), Episode::empty());
    }
}
