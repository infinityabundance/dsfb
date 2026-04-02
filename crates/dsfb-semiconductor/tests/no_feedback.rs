use dsfb_semiconductor::input::residual_stream::ResidualSample;
use dsfb_semiconductor::interface::{DSFBObserver, ReadOnlyDsfbObserver};

#[test]
fn residual_ingest_is_read_only_and_advisory_only() {
    let observer = ReadOnlyDsfbObserver::new();
    let sample = ResidualSample {
        timestamp: 1.0,
        feature_id: "S059".into(),
        value: 2.5,
    };
    let original = sample.clone();
    observer.ingest(&sample);

    assert_eq!(sample, original);

    let serialized = serde_json::to_string(&observer.output()).unwrap();
    for forbidden in [
        "controller",
        "actuation",
        "recipe",
        "threshold_write",
        "feedback",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "unexpected feedback surface token {forbidden}"
        );
    }
}

#[test]
fn observer_output_is_deterministic_for_identical_inputs() {
    let observer = ReadOnlyDsfbObserver::new();
    for sample in [
        ResidualSample {
            timestamp: 0.0,
            feature_id: "S059".into(),
            value: 0.5,
        },
        ResidualSample {
            timestamp: 1.0,
            feature_id: "S059".into(),
            value: 1.5,
        },
        ResidualSample {
            timestamp: 2.0,
            feature_id: "S059".into(),
            value: 2.8,
        },
    ] {
        observer.ingest(&sample);
    }

    let first = observer.output();
    let second = observer.output();
    assert_eq!(first, second);
}
