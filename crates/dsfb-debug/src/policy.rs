//! DSFB-Debug: policy engine — paper §5.
//!
//! Maps the tuple `(matched motif, DSA score, confirmed grammar
//! state)` to the developer-facing four-level `PolicyState`:
//!
//! | State | Meaning |
//! |-------|---------|
//! | `Silent` | No structural deviation; suppress dashboard |
//! | `Watch` | Deviation detected, below escalation threshold |
//! | `Review` | Sustained drift / low slew; surface to operator |
//! | `Escalate` | Hard envelope breach / high slew; page on-call |
//!
//! The policy state determines whether and how the typed episode
//! reaches the operator surface. `Silent` and `Watch` episodes are
//! suppressed from operator alerting (logged for the audit trail
//! only); `Review` and `Escalate` episodes flow through to dashboards.
//!
//! Policy decisions are pure functions of the motif + DSA score +
//! grammar state inputs; they preserve Theorem 9 deterministic
//! replay trivially.

use crate::types::*;

/// Apply policy based on grammar state, DSA score, motif match, and persistence.
///
/// - Silent:   gate failed or no motif
/// - Watch:    motif active, below escalation
/// - Review:   persistence >= K AND motif class == Review
/// - Escalate: persistence >= K AND Violation-class motif
#[inline]
pub fn apply_policy(
    confirmed_grammar: GrammarState,
    dsa_score: f64,
    consistency_gate_passed: bool,
    semantic: SemanticDisposition,
    persistence_count: usize,
    persistence_threshold: usize,
) -> PolicyState {
    // No structural evidence → Silent
    if confirmed_grammar == GrammarState::Admissible && dsa_score < 0.5 {
        return PolicyState::Silent;
    }

    // Consistency gate not passed → Silent
    if !consistency_gate_passed && confirmed_grammar != GrammarState::Violation {
        return PolicyState::Silent;
    }

    // Violation always escalates (regardless of persistence gate)
    if confirmed_grammar == GrammarState::Violation {
        return PolicyState::Escalate;
    }

    // Persistence gate: must have >= K consecutive boundary windows
    if persistence_count < persistence_threshold {
        // Activity present but not sustained → Watch
        if confirmed_grammar == GrammarState::Boundary {
            return PolicyState::Watch;
        }
        return PolicyState::Silent;
    }

    // Persistence gate passed + Boundary state
    match semantic {
        SemanticDisposition::Named(motif) => {
            // Check if the motif warrants Escalate or Review
            match motif {
                MotifClass::CascadingTimeoutSlew
                | MotifClass::DeploymentRegressionSlew
                | MotifClass::ErrorRateEscalation => PolicyState::Escalate,
                _ => PolicyState::Review,
            }
        }
        SemanticDisposition::Unknown => {
            // Endoductive: structure present but unnamed → Review
            PolicyState::Review
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admissible_is_silent() {
        let p = apply_policy(
            GrammarState::Admissible, 0.1, false,
            SemanticDisposition::Unknown, 0, 4,
        );
        assert_eq!(p, PolicyState::Silent);
    }

    #[test]
    fn test_violation_is_escalate() {
        let p = apply_policy(
            GrammarState::Violation, 3.0, true,
            SemanticDisposition::Unknown, 0, 4,
        );
        assert_eq!(p, PolicyState::Escalate);
    }

    #[test]
    fn test_boundary_below_persistence_is_watch() {
        let p = apply_policy(
            GrammarState::Boundary, 2.5, true,
            SemanticDisposition::Named(MotifClass::MemoryLeakDrift), 2, 4,
        );
        assert_eq!(p, PolicyState::Watch);
    }

    #[test]
    fn test_boundary_above_persistence_review() {
        let p = apply_policy(
            GrammarState::Boundary, 2.5, true,
            SemanticDisposition::Named(MotifClass::MemoryLeakDrift), 5, 4,
        );
        assert_eq!(p, PolicyState::Review);
    }
}
