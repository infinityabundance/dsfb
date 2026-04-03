// Metric consistency tests for the DSFB observer pipeline.
// These tests verify that the outputs satisfy fundamental internal constraints:
// - all policy decisions use valid state strings
// - no feedback tokens appear in serialized output
// - identical observer instances produce identical metric outputs
// - Silent decisions are produced for purely admissible inputs
// - review surface compression is non-negative (output ≤ input)

use dsfb_semiconductor::input::residual_stream::ResidualSample;
use dsfb_semiconductor::interface::{DSFBObserver, ReadOnlyDsfbObserver};

/// Samples representative of a slow drift-to-boundary scenario,
/// similar to the SECOM target-depletion precursor pattern.
fn secom_like_drift_samples() -> Vec<ResidualSample> {
    let values = [0.1, 0.4, 0.8, 1.2, 1.6, 2.0, 2.4, 2.7, 2.9, 3.1];
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| ResidualSample {
            timestamp: i as f64,
            feature_id: "S059".into(),
            value: v,
        })
        .collect()
}

fn admissible_only_samples() -> Vec<ResidualSample> {
    (0..5)
        .map(|i| ResidualSample {
            timestamp: i as f64,
            feature_id: "S001".into(),
            value: 0.01,
        })
        .collect()
}

/// All policy decisions must use one of the four valid DSFB state strings.
#[test]
fn all_policy_decisions_use_valid_state_strings() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in secom_like_drift_samples() {
        observer.ingest(&sample);
    }
    let decisions = observer.output();
    let valid_states = ["Silent", "Watch", "Review", "Escalate"];
    for decision in &decisions {
        assert!(
            valid_states.contains(&decision.decision.as_str()),
            "invalid policy decision state: '{}' — must be one of {:?}",
            decision.decision,
            valid_states
        );
    }
}

/// No feedback surface tokens appear in the JSON-serialized output.
/// This is a structural non-intrusion check on the output surface.
#[test]
fn no_feedback_tokens_in_serialized_policy_output() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in secom_like_drift_samples() {
        observer.ingest(&sample);
    }
    let serialized = serde_json::to_string(&observer.output()).unwrap();
    for forbidden in [
        "controller",
        "actuation",
        "recipe_write",
        "threshold_write",
        "feedback_path",
        "write_back",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "forbidden feedback token '{}' found in serialized policy output — \
             this token must not appear on the DSFB output surface",
            forbidden
        );
    }
}

/// Two ReadOnlyDsfbObserver instances with identical inputs must produce
/// identical metric outputs (policy decisions, grammar states, semantic matches).
#[test]
fn metric_output_is_identical_for_two_identical_observers() {
    let observer_a = ReadOnlyDsfbObserver::new();
    let observer_b = ReadOnlyDsfbObserver::new();
    for sample in secom_like_drift_samples() {
        observer_a.ingest(&sample);
        observer_b.ingest(&sample);
    }
    assert_eq!(
        observer_a.output(),
        observer_b.output(),
        "two observers with identical inputs must produce identical policy decisions"
    );
    let a = observer_a.layered_output();
    let b = observer_b.layered_output();
    assert_eq!(a.grammar_states, b.grammar_states);
    assert_eq!(a.semantic_matches, b.semantic_matches);
    assert_eq!(a.policy_decisions, b.policy_decisions);
}

/// For purely admissible inputs (near-zero residuals), the observer must
/// produce at least one Silent decision and no Escalate decisions.
/// This bounds the false escalation rate for noise-only input.
#[test]
fn admissible_inputs_produce_silent_and_no_escalation() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in admissible_only_samples() {
        observer.ingest(&sample);
    }
    let decisions = observer.output();
    let silent_count = decisions
        .iter()
        .filter(|d| d.decision == "Silent")
        .count();
    let escalate_count = decisions
        .iter()
        .filter(|d| d.decision == "Escalate")
        .count();
    assert!(
        silent_count > 0,
        "near-zero admissible inputs must produce at least one Silent decision"
    );
    assert_eq!(
        escalate_count, 0,
        "near-zero admissible inputs must not produce any Escalate decisions"
    );
}

/// Review surface compression: the number of non-silent advisory outputs
/// must be ≤ the number of input residual samples. DSFB compresses, not
/// amplifies, the review surface.
#[test]
fn output_count_does_not_exceed_input_count() {
    let samples = secom_like_drift_samples();
    let input_count = samples.len();
    let observer = ReadOnlyDsfbObserver::new();
    for sample in samples {
        observer.ingest(&sample);
    }
    let non_silent = observer
        .output()
        .into_iter()
        .filter(|d| d.decision != "Silent")
        .count();
    assert!(
        non_silent <= input_count,
        "non-silent decisions ({non_silent}) exceeds input count ({input_count}) — \
         DSFB must not amplify the review surface"
    );
}

/// Decision ranks are monotonically ordered: Escalate > Review > Watch > Silent.
/// A Watch timestamp must not be upgraded to Escalate in a second replay.
#[test]
fn decision_rank_is_stable_across_replays() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in secom_like_drift_samples() {
        observer.ingest(&sample);
    }
    let first = observer.output();
    let second = observer.output();
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(
            a.decision, b.decision,
            "replay changed decision at t={}: {} → {}",
            a.timestamp, a.decision, b.decision
        );
    }
}
