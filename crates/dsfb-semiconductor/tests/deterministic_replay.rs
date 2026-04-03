use dsfb_semiconductor::input::residual_stream::ResidualSample;
use dsfb_semiconductor::interface::{DSFBObserver, ReadOnlyDsfbObserver};

fn fixture_samples() -> [ResidualSample; 4] {
    [
        ResidualSample {
            timestamp: 3.0,
            feature_id: "S059".into(),
            value: 2.8,
        },
        ResidualSample {
            timestamp: 1.0,
            feature_id: "S059".into(),
            value: 0.5,
        },
        ResidualSample {
            timestamp: 2.0,
            feature_id: "S059".into(),
            value: 1.5,
        },
        ResidualSample {
            timestamp: 2.0,
            feature_id: "S104".into(),
            value: 0.7,
        },
    ]
}

#[test]
fn identical_inputs_replay_to_identical_outputs() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in fixture_samples() {
        observer.ingest(&sample);
    }

    let first = observer.layered_output();
    let second = observer.layered_output();

    assert_eq!(first.signs, second.signs);
    assert_eq!(first.motif_timeline, second.motif_timeline);
    assert_eq!(first.grammar_states, second.grammar_states);
    assert_eq!(first.semantic_matches, second.semantic_matches);
    assert_eq!(first.policy_decisions, second.policy_decisions);
}

#[test]
fn input_order_does_not_change_replayed_output() {
    let observer_a = ReadOnlyDsfbObserver::new();
    for sample in fixture_samples() {
        observer_a.ingest(&sample);
    }

    let observer_b = ReadOnlyDsfbObserver::new();
    for sample in fixture_samples().into_iter().rev() {
        observer_b.ingest(&sample);
    }

    assert_eq!(observer_a.layered_output().policy_decisions, observer_b.layered_output().policy_decisions);
    assert_eq!(observer_a.layered_output().semantic_matches, observer_b.layered_output().semantic_matches);
}
