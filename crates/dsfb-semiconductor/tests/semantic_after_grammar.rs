// Invariant: semantic layer always operates downstream of the grammar layer.
// These tests verify the actual pipeline ordering constraints:
//
// 1. Every SemanticMatch has a corresponding GrammarState at the same
//    feature + timestamp (grammar must exist before semantics can fire).
// 2. Non-Silent semantic matches must come from non-Admissible grammar states.
// 3. Semantic matches for Admissible grammar states yield only Silent actions.
// 4. Non-Silent policy decisions require a non-Admissible grammar state or
//    a non-Silent semantic match.

use dsfb_semiconductor::input::residual_stream::ResidualSample;
use dsfb_semiconductor::interface::{DSFBObserver, ReadOnlyDsfbObserver};

fn persistent_drift_samples() -> Vec<ResidualSample> {
    (0..12)
        .map(|i| ResidualSample {
            timestamp: i as f64,
            feature_id: "S059".into(),
            value: 0.5 + (i as f64) * 0.35,
        })
        .collect()
}

fn admissible_samples() -> Vec<ResidualSample> {
    (0..6)
        .map(|i| ResidualSample {
            timestamp: i as f64,
            feature_id: "S099".into(),
            value: 0.02,
        })
        .collect()
}

/// Every SemanticMatch must have a corresponding GrammarState for the same
/// feature and timestamp — grammar layer ran before semantics can fire.
#[test]
fn every_semantic_match_has_a_corresponding_grammar_state() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in persistent_drift_samples() {
        observer.ingest(&sample);
    }
    let artifacts = observer.layered_output();

    for semantic in &artifacts.semantic_matches {
        let has_grammar = artifacts.grammar_states.iter().any(|g| {
            (g.timestamp - semantic.timestamp).abs() < f64::EPSILON
                && g.feature_id == semantic.feature_id
        });
        assert!(
            has_grammar,
            "semantic match for feature {} at t={} has no grammar state — \
             grammar layer must run before semantic layer",
            semantic.feature_id, semantic.timestamp
        );
    }
}

/// Non-Silent semantic matches must come from non-Admissible grammar states.
/// An Admissible state matches only the noise-suppress heuristic (action=Silent).
#[test]
fn non_silent_semantic_matches_require_non_admissible_grammar_state() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in persistent_drift_samples() {
        observer.ingest(&sample);
    }
    let artifacts = observer.layered_output();

    for semantic in &artifacts.semantic_matches {
        if semantic.action == "Silent" {
            continue;
        }
        let has_non_admissible = artifacts.grammar_states.iter().any(|g| {
            (g.timestamp - semantic.timestamp).abs() < f64::EPSILON
                && g.feature_id == semantic.feature_id
                && g.state != "Admissible"
        });
        assert!(
            has_non_admissible,
            "non-Silent semantic match (action={}) for feature {} at t={} \
             has no non-Admissible grammar state — violates pipeline ordering",
            semantic.action, semantic.feature_id, semantic.timestamp
        );
    }
}

/// Semantic matches for Admissible grammar states must have action = "Silent".
/// Admissible grammar cannot escalate through the semantic layer.
#[test]
fn admissible_grammar_state_yields_only_silent_semantic_matches() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in admissible_samples() {
        observer.ingest(&sample);
    }
    let artifacts = observer.layered_output();

    for gs in &artifacts.grammar_states {
        if gs.state != "Admissible" {
            continue;
        }
        for semantic in &artifacts.semantic_matches {
            if (semantic.timestamp - gs.timestamp).abs() < f64::EPSILON
                && semantic.feature_id == gs.feature_id
            {
                assert_eq!(
                    semantic.action, "Silent",
                    "Admissible grammar state for feature {} at t={} produced \
                     non-Silent semantic match (action={})",
                    gs.feature_id, gs.timestamp, semantic.action
                );
            }
        }
    }
}

/// Grammar states must appear at timestamps ≤ any semantic match for the same
/// feature. This is the causal ordering (temporal precedence) check.
#[test]
fn grammar_states_are_not_later_than_semantic_matches() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in persistent_drift_samples() {
        observer.ingest(&sample);
    }
    let artifacts = observer.layered_output();

    for semantic in &artifacts.semantic_matches {
        let earliest_grammar = artifacts
            .grammar_states
            .iter()
            .filter(|g| g.feature_id == semantic.feature_id)
            .map(|g| g.timestamp)
            .fold(f64::INFINITY, f64::min);

        assert!(
            earliest_grammar <= semantic.timestamp + f64::EPSILON,
            "grammar first appears at t={earliest_grammar} but semantic match \
             is at t={} — grammar must precede or coincide with semantics",
            semantic.timestamp
        );
    }
}

/// Non-Silent policy decisions must come from a non-Admissible grammar state
/// or a non-Silent semantic match. This verifies the end-to-end causal chain.
#[test]
fn non_silent_policy_requires_non_admissible_grammar_or_non_silent_semantic() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in persistent_drift_samples() {
        observer.ingest(&sample);
    }
    let artifacts = observer.layered_output();

    for decision in &artifacts.policy_decisions {
        if decision.decision == "Silent" {
            continue;
        }
        let has_non_admissible_grammar = artifacts.grammar_states.iter().any(|g| {
            (g.timestamp - decision.timestamp).abs() < f64::EPSILON && g.state != "Admissible"
        });
        let has_non_silent_semantic = artifacts.semantic_matches.iter().any(|m| {
            (m.timestamp - decision.timestamp).abs() < f64::EPSILON && m.action != "Silent"
        });
        assert!(
            has_non_admissible_grammar || has_non_silent_semantic,
            "non-Silent policy '{}' at t={} has no non-Admissible grammar state \
             and no non-Silent semantic match — cannot escalate without cause",
            decision.decision, decision.timestamp
        );
    }
}
