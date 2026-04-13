/// non_interference.rs — Tests verifying the non-intrusive integration contract.
///
/// The DSFB framework must satisfy the non-intrusive contract:
///   - It never modifies upstream data.
///   - ReadOnlySlice exposes no mutable access.
///   - Removing the DSFB engine from a computation leaves upstream state identical.

use dsfb_oil_gas::{
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, ReadOnlySlice,
    ResidualSample, process_read_only,
};

/// ReadOnlySlice must not expose any mutable reference to inner data.
/// Verified by type-level API: the only usable call is as_slice() → &[T].
#[test]
fn read_only_slice_values_unchanged() {
    let original = vec![1.0f64, 2.0, 3.0, 4.0, 5.0];
    let ro = ReadOnlySlice::wrap(original.clone());
    // The only way to access data is through a shared reference.
    let view = ro.as_slice();
    for (a, b) in original.iter().zip(view.iter()) {
        assert!((a - b).abs() < 1e-12, "data must be unchanged");
    }
}

/// process_read_only passes only a shared reference; sum computation
/// cannot modify the source.
#[test]
fn process_read_only_cannot_mutate_source() {
    let data = vec![10.0f64, 20.0, 30.0];
    let source = ReadOnlySlice::wrap(data);
    let sum = process_read_only(&source, |sl| sl.iter().sum::<f64>());
    assert!((sum - 60.0).abs() < 1e-10);
    // Source still has 3 elements — process did not consume or shorten it.
    assert_eq!(source.len(), 3);
}

/// Running DSFB on a dataset leaves the original samples unchanged.
#[test]
fn dsfb_does_not_modify_input_samples() {
    let samples: Vec<ResidualSample> = (0..20)
        .map(|i| ResidualSample::new(i as f64 * 0.5, (i as f64).sin() * 2.0, 0.0, "ni"))
        .collect();

    // Capture original values
    let orig_observed: Vec<f64> = samples.iter().map(|s| s.observed).collect();
    let orig_expected: Vec<f64> = samples.iter().map(|s| s.expected).collect();

    // Run engine
    let mut engine = DeterministicDsfb::new(
        AdmissibilityEnvelope::default_pipeline(),
        GrammarClassifier::new(),
    );
    for s in &samples {
        engine.ingest_sample(s);
    }

    // Verify input samples are unchanged after DSFB processing
    for (i, s) in samples.iter().enumerate() {
        assert!(
            (s.observed - orig_observed[i]).abs() < 1e-12,
            "sample {} observed was modified by DSFB", i
        );
        assert!(
            (s.expected - orig_expected[i]).abs() < 1e-12,
            "sample {} expected was modified by DSFB", i
        );
    }
}

/// The DSFB engine's history does not alias or share memory with input samples.
/// Verified by ensuring different timestamps → different history entries.
#[test]
fn history_does_not_alias_input() {
    let samples: Vec<ResidualSample> = (0..10)
        .map(|i| ResidualSample::new(i as f64, 1.0, 0.0, "alias"))
        .collect();

    let mut engine = DeterministicDsfb::new(
        AdmissibilityEnvelope::default_pipeline(),
        GrammarClassifier::new(),
    );
    for s in &samples {
        engine.ingest_sample(s);
    }

    let history = engine.history();
    assert_eq!(history.len(), samples.len(), "one history entry per sample");

    // Each history entry corresponds to the correct input timestamp
    for (step, sample) in history.iter().zip(samples.iter()) {
        assert!(
            (step.triple.timestamp - sample.timestamp).abs() < 1e-9,
            "history entry timestamp mismatch"
        );
    }
}

/// Two independent engine instances on the same input produce identical output.
/// Verifies isolation: engines do not share any global mutable state.
#[test]
fn independent_engines_produce_same_output() {
    let samples: Vec<ResidualSample> = (0..25)
        .map(|i| ResidualSample::new(i as f64 * 0.5, (i as f64 * 0.4).cos() * 3.0, 0.0, "iso"))
        .collect();

    let env = AdmissibilityEnvelope::default_pipeline();

    let mut e1 = DeterministicDsfb::new(env, GrammarClassifier::new());
    let mut e2 = DeterministicDsfb::new(env, GrammarClassifier::new());

    for s in &samples {
        e1.ingest_sample(s);
        e2.ingest_sample(s);
    }

    let h1 = e1.history();
    let h2 = e2.history();
    assert_eq!(h1.len(), h2.len());
    for (a, b) in h1.iter().zip(h2.iter()) {
        assert_eq!(a.state, b.state, "independent engines must produce identical states");
    }
}
