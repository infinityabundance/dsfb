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
