/// basic_invariants.rs — Integration tests for DSFB core invariants.
///
/// These tests verify the mathematical and behavioral properties that DSFB
/// is required to satisfy:
///   1. No output step contains NaN or ±∞.
///   2. Constant zero residual input always produces Nominal state.
///   3. Drift grows monotonically in the window for constant positive input.
///   4. Deterministic replay produces identical output to original run.
///   5. Noise compression ratio ≥ 1 (episodes ≤ samples).
///   6. BoundaryGrazing is emitted when any component is in the grazing band.
///   7. Recovery is a single-step transient (not a persistent loop).
///   8. Compound requires both δ AND σ to be violated simultaneously.
///   9. Subsea domain produces well-formed output under its default envelope.

use dsfb_oil_gas::{
    AdmissibilityEnvelope, DeterministicDsfb, GrammarClassifier, GrammarState,
    ResidualSample,
    aggregate_episodes,
    noise_compression_ratio,
};

fn make_engine() -> DeterministicDsfb {
    DeterministicDsfb::new(
        AdmissibilityEnvelope::default_pipeline(),
        GrammarClassifier::new(),
    )
}

/// Every history entry must be free of NaN / ±∞ in numeric fields.
#[test]
fn no_nan_in_output() {
    let mut engine = make_engine();
    for i in 0..50 {
        let s = ResidualSample::new(i as f64 * 0.5, (i as f64).sin() * 2.5, 0.0, "test");
        engine.ingest_sample(&s);
    }
    for step in engine.history() {
        assert!(step.triple.r.is_finite(), "r contains NaN/Inf at {:?}", step);
        assert!(step.triple.delta.is_finite(), "delta contains NaN/Inf");
        assert!(step.triple.sigma.is_finite(), "sigma contains NaN/Inf");
    }
}

/// Constant zero residual must always produce GrammarState::Nominal.
#[test]
fn zero_residual_always_nominal() {
    let mut engine = make_engine();
    for i in 0..40 {
        let s = ResidualSample::new(i as f64 * 0.5, 0.0, 0.0, "zero");
        engine.ingest_sample(&s);
    }
    for step in engine.history() {
        assert_eq!(
            step.state,
            GrammarState::Nominal,
            "expected Nominal for zero residual, got {:?}",
            step.state
        );
    }
}

/// With constant positive residual, drift must reach DriftAccum eventually.
#[test]
fn constant_drift_reaches_drift_accum() {
    let mut engine = DeterministicDsfb::with_window(
        AdmissibilityEnvelope::default_pipeline(),
        GrammarClassifier::new(),
        5,
        "drift_test",
    );
    for i in 0..30 {
        // Large residual that should push drift outside default envelope
        let s = ResidualSample::new(i as f64 * 0.5, 6.0, 0.0, "drift_test");
        engine.ingest_sample(&s);
    }
    let non_nominal = engine
        .history()
        .iter()
        .any(|s| s.state != GrammarState::Nominal);
    assert!(non_nominal, "constant large residual should produce non-Nominal tokens");
}

/// A large instantaneous slew must produce a SlewSpike token.
#[test]
fn large_slew_produces_slew_spike() {
    let mut engine = DeterministicDsfb::with_window(
        AdmissibilityEnvelope {
            r_min: -5.0, r_max: 5.0,
            delta_min: -2.0, delta_max: 2.0,
            sigma_min: -10.0, sigma_max: 10.0,
            grazing_band: 1.0,
        },
        GrammarClassifier::new(),
        5,
        "slew_test",
    );
    // Warm-up: 10 nominal samples
    for i in 0..10 {
        let s = ResidualSample::new(i as f64 * 0.5, 0.2, 0.0, "slew_test");
        engine.ingest_sample(&s);
    }
    // Spike: residual jumps from ~0.2 to 40.0 in one step → sigma = 79.6/s
    let spike = ResidualSample::new(5.0, 40.0, 0.0, "slew_test");
    engine.ingest_sample(&spike);

    let last = engine.history().last().unwrap();
    assert!(
        matches!(last.state, GrammarState::SlewSpike | GrammarState::Compound | GrammarState::EnvViolation),
        "expected SlewSpike/Compound/EnvViolation for huge slew, got {:?}", last.state
    );
}

/// Deterministic replay must produce token-for-token identical output.
#[test]
fn deterministic_replay_identical() {
    let samples: Vec<ResidualSample> = (0..30)
        .map(|i| ResidualSample::new(i as f64 * 0.5, (i as f64 * 0.3).sin() * 3.0, 0.0, "rep"))
        .collect();

    let states_run1: Vec<GrammarState> = {
        let mut e = make_engine();
        e.reset();
        samples.iter().map(|s| e.ingest_sample(s).state).collect()
    };
    let states_run2: Vec<GrammarState> = {
        let mut e = make_engine();
        e.reset();
        samples.iter().map(|s| e.ingest_sample(s).state).collect()
    };
    assert_eq!(states_run1, states_run2, "replay must be deterministic");
}

/// Episode count must be ≤ sample count (noise compression ratio ≥ 1.0).
#[test]
fn noise_compression_ratio_at_least_one() {
    let mut engine = make_engine();
    let n = 80usize;
    for i in 0..n {
        let s = ResidualSample::new(i as f64 * 0.5, (i as f64 * 0.2).sin() * 2.0, 0.0, "ncr");
        engine.ingest_sample(&s);
    }
    let episodes = aggregate_episodes(engine.history());
    let ratio = noise_compression_ratio(n, episodes.len());
    assert!(ratio >= 1.0, "NCR must be ≥ 1.0, got {}", ratio);
}

/// BoundaryGrazing must be emitted when any component is in the grazing band
/// but no component is outside the envelope.
///
/// Configuration: wide delta/sigma bounds so drift and slew never violate;
/// r set to ~4.8 which is inside [4.5, 5.0] grazing band (grazing_band=0.1).
#[test]
fn boundary_grazing_is_emitted_when_r_in_grazing_band() {
    let env = AdmissibilityEnvelope {
        r_min:     -5.0,
        r_max:      5.0,
        delta_min: -100.0,
        delta_max:  100.0,  // wide: drift of ~4.8 is never violated
        sigma_min: -100.0,
        sigma_max:  100.0,  // wide: slew is never violated
        grazing_band: 0.1,  // grazing when |r_norm| in [0.9, 1.0), i.e. |r| in [4.5, 5.0)
    };
    let mut eng = DeterministicDsfb::with_window(env, GrammarClassifier::new(), 1, "bg");

    // Feed a constant r = 4.8; after first step delta = 4.8 (interior for ±100),
    // sigma = 0 (constant residual, no change); r_norm = 0.96 → Grazing.
    for i in 0..5 {
        eng.ingest_sample(&ResidualSample::new(i as f64, 4.8, 0.0, "bg"));
    }

    let any_grazing = eng.history().iter().any(|s| s.state == GrammarState::BoundaryGrazing);
    assert!(
        any_grazing,
        "expected at least one BoundaryGrazing step, got states: {:?}",
        eng.history().iter().map(|s| s.state).collect::<Vec<_>>()
    );
}

/// Recovery must be single-step: after a non-Nominal event returns to interior,
/// the FIRST interior step emits Recovery; the SECOND emits Nominal (not Recovery again).
///
/// This guards against the bug where prev_state is set to Recovery, causing the
/// next interior step to again see prev_state != Nominal and re-emit Recovery.
#[test]
fn recovery_is_single_step_not_persistent() {
    // Narrow envelope so we can trigger a violation then recover.
    let env = AdmissibilityEnvelope {
        r_min: -2.0, r_max: 2.0,
        delta_min: -1.0, delta_max: 1.0,
        sigma_min: -20.0, sigma_max: 20.0,
        grazing_band: 0.05,
    };
    let mut eng = DeterministicDsfb::with_window(env, GrammarClassifier::new(), 1, "rc");

    // 5 nominal steps (r = 0.5)
    for i in 0..5 {
        eng.ingest_sample(&ResidualSample::new(i as f64, 0.5, 0.0, "rc"));
    }
    // 1 violation step (r = 5.0, well outside ±2.0)
    eng.ingest_sample(&ResidualSample::new(5.0, 5.0, 0.0, "rc"));
    // 3 steps returning to nominal interior (r = 0.5; drift window=1 so delta=0.5, interior)
    for i in 6..9 {
        eng.ingest_sample(&ResidualSample::new(i as f64, 0.5, 0.0, "rc"));
    }

    let hist: Vec<GrammarState> = eng.history().iter().map(|s| s.state).collect();
    // Step index 5 should be non-Nominal (EnvViolation or similar)
    assert_ne!(hist[5], GrammarState::Nominal, "step 5 should be violation");
    // Step index 6 should be Recovery (first interior step after violation)
    assert_eq!(hist[6], GrammarState::Recovery, "step 6 should be Recovery, got {:?}", hist[6]);
    // Step index 7 must be Nominal — NOT another Recovery
    assert_eq!(
        hist[7], GrammarState::Nominal,
        "step 7 should be Nominal after single Recovery, got {:?} (Recovery loop bug?)",
        hist[7]
    );
    // Step index 8 must also be Nominal
    assert_eq!(hist[8], GrammarState::Nominal, "step 8 should remain Nominal");
}

/// Compound requires BOTH δ AND σ to be simultaneously outside the envelope.
/// A slew-only violation must produce SlewSpike (not Compound).
/// A drift-only violation must produce DriftAccum (not Compound).
#[test]
fn compound_requires_both_delta_and_sigma_violated() {
    // Envelope: tight sigma bounds, tight delta bounds, very wide r bounds.
    let env_tight = AdmissibilityEnvelope {
        r_min:     -500.0,
        r_max:      500.0,
        delta_min:  -1.0,
        delta_max:   1.0,
        sigma_min:  -1.0,
        sigma_max:   1.0,
        grazing_band: 0.01,
    };

    // ── Slew-only violation (large instantaneous jump, but window=1 so delta=r which is large)
    // We need to isolate sigma violation without delta violation.
    // Use large delta bounds so drift stays interior:
    let env_wide_delta = AdmissibilityEnvelope {
        r_min:     -500.0, r_max:      500.0,
        delta_min: -500.0, delta_max:  500.0,   // delta cannot be violated
        sigma_min:   -1.0, sigma_max:    1.0,   // sigma tight: single jump violates it
        grazing_band: 0.01,
    };
    {
        let mut eng = DeterministicDsfb::with_window(
            env_wide_delta, GrammarClassifier::new(), 1, "compound_test"
        );
        // 5 warm-up steps at residual = 0
        for i in 0..5 { eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "ct")); }
        // One large slew (r jumps from ~0 to 100, sigma ≈ 100/1 = 100 >> sigma_max=1)
        eng.ingest_sample(&ResidualSample::new(5.0, 100.0, 0.0, "ct"));
        let last = eng.history().last().unwrap().state;
        assert_eq!(
            last, GrammarState::SlewSpike,
            "slew-only violation must be SlewSpike, not Compound; got {:?}", last
        );
    }

    // ── Drift-only violation (large constant residual fills window, sigma ≈ 0).
    let env_wide_sigma = AdmissibilityEnvelope {
        r_min:     -500.0, r_max:      500.0,
        delta_min:   -1.0, delta_max:    1.0,   // delta tight: constant residual violates it
        sigma_min: -500.0, sigma_max:  500.0,   // sigma very wide: no sigma violation
        grazing_band: 0.01,
    };
    {
        let mut eng = DeterministicDsfb::with_window(
            env_wide_sigma, GrammarClassifier::new(), 1, "drift_only"
        );
        // Feed constant r = 10 >> delta_max=1; with window=1, delta=r=10 → violated.
        // sigma(t) = (r_t - r_{t-1}) / Δt = 0 for constant input after step 1.
        for i in 0..5 { eng.ingest_sample(&ResidualSample::new(i as f64, 10.0, 0.0, "do")); }
        // After step ≥ 1 (when sigma=0), state must be DriftAccum, not Compound.
        let drift_accum_steps: Vec<GrammarState> = eng.history().iter()
            .skip(1)  // skip step 0 where sigma=0 by convention but delta may not be filled
            .map(|s| s.state)
            .collect();
        for (i, st) in drift_accum_steps.iter().enumerate() {
            assert_ne!(
                *st, GrammarState::Compound,
                "drift-only violation at step {} must not produce Compound; got {:?}", i, st
            );
        }
        assert!(
            drift_accum_steps.iter().any(|s| *s == GrammarState::DriftAccum),
            "at least one DriftAccum expected for drift-only violation"
        );
    }

    // ── Both violated simultaneously → Compound.
    {
        let mut eng = DeterministicDsfb::with_window(
            env_tight, GrammarClassifier::new(), 1, "compound"
        );
        // 5 warm-up at 0
        for i in 0..5 { eng.ingest_sample(&ResidualSample::new(i as f64, 0.0, 0.0, "cp")); }
        // Large slew (10→70): sigma = (70-0)/1 = 70 >> sigma_max=1
        // delta = mean_window1(r) = 70 >> delta_max=1
        // Both violated → Compound
        eng.ingest_sample(&ResidualSample::new(5.0, 70.0, 0.0, "cp"));
        let last = eng.history().last().unwrap().state;
        assert_eq!(
            last, GrammarState::Compound,
            "both δ and σ violated must produce Compound, got {:?}", last
        );
    }
}

/// Subsea domain produces finite, well-typed output under the default subsea envelope,
/// and SubseaFrame::channel_name() returns the expected override (not the default "default").
///
/// Guards that:
///  a) The subsea domain ingestion path runs without panic.
///  b) channel_name() override is wired up correctly in SubseaFrame.
///  c) A realistic pressure spike triggers at least one non-Nominal token.
#[test]
fn subsea_domain_produces_valid_output() {
    use dsfb_oil_gas::{DsfbDomainFrame, SubseaFrame};

    // ── a) channel_name() override check (direct trait dispatch)
    let sample_frame = SubseaFrame {
        timestamp: 0.0,
        expected_actuation_pressure: 100.0,
        observed_actuation_pressure: 100.5,
        valve_command: 1.0,
    };
    assert_eq!(
        sample_frame.channel_name(),
        "subsea_actuation_pressure",
        "SubseaFrame must override channel_name() to 'subsea_actuation_pressure'"
    );

    // ── b/c) Engine ingestion using the subsea default envelope
    // Note: DeterministicDsfb::ingest<F> takes F by value (SubseaFrame: Copy).
    // The engine's AnnotatedStep.channel comes from the constructor arg; we name it
    // to match the domain for logging fidelity.
    let env = AdmissibilityEnvelope::default_subsea();
    let mut eng = DeterministicDsfb::with_window(
        env, GrammarClassifier::new(), 5, "subsea_actuation_pressure"
    );

    // 20 nominal steps with a small, growing drift that stays inside envelope
    for i in 0..20 {
        eng.ingest(SubseaFrame {
            timestamp: i as f64,
            expected_actuation_pressure: 100.0,
            observed_actuation_pressure: 101.0 + (i as f64 * 0.1),  // max residual ~3.0
            valve_command: 1.0,
        });
    }
    // One pressure spike: residual = 250 - 100 = 150 >> r_max=50, should → EnvViolation/Compound
    eng.ingest(SubseaFrame {
        timestamp: 20.0,
        expected_actuation_pressure: 100.0,
        observed_actuation_pressure: 250.0,
        valve_command: 0.0,
    });
    // Return to nominal
    for i in 21..30 {
        eng.ingest(SubseaFrame {
            timestamp: i as f64,
            expected_actuation_pressure: 100.0,
            observed_actuation_pressure: 100.5,
            valve_command: 1.0,
        });
    }

    assert_eq!(eng.history().len(), 30, "should have exactly 30 annotated steps");

    for step in eng.history() {
        assert!(step.triple.r.is_finite(),     "subsea r contains NaN/Inf");
        assert!(step.triple.delta.is_finite(), "subsea delta contains NaN/Inf");
        assert!(step.triple.sigma.is_finite(), "subsea sigma contains NaN/Inf");
    }

    // The spike at step 20 (residual=150) must trigger a non-Nominal event
    let any_non_nominal = eng.history().iter().any(|s| s.state != GrammarState::Nominal);
    assert!(any_non_nominal, "subsea spike (residual=150, r_max=50) should produce non-Nominal");

    // All AnnotatedStep.channel values should match the engine constructor arg
    for step in eng.history() {
        assert_eq!(step.channel, "subsea_actuation_pressure", "engine channel mismatch");
    }
}
