use dsfb_semiotics_engine::engine::grammar_layer::{
    evaluate_detectability, evaluate_grammar_layer,
};
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};
use dsfb_semiotics_engine::engine::residual_layer::extract_residuals;
use dsfb_semiotics_engine::engine::semantics_layer::retrieve_semantics;
use dsfb_semiotics_engine::engine::sign_layer::construct_signs;
use dsfb_semiotics_engine::engine::types::{
    GrammarState, ObservedTrajectory, PredictedTrajectory, SemanticDisposition,
    SyntaxCharacterization, VectorSample,
};
use dsfb_semiotics_engine::io::output::create_output_layout;
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::envelope::build_envelope;
use dsfb_semiotics_engine::sim::generators::synthesize;
use dsfb_semiotics_engine::sim::scenarios::all_scenarios;
use tempfile::TempDir;

#[test]
fn residual_computation_matches_observation_minus_prediction() {
    let observed = ObservedTrajectory {
        scenario_id: "unit".to_string(),
        channel_names: vec!["ch1".to_string(), "ch2".to_string()],
        samples: vec![
            VectorSample {
                step: 0,
                time: 0.0,
                values: vec![1.0, 2.0],
            },
            VectorSample {
                step: 1,
                time: 1.0,
                values: vec![3.0, 5.0],
            },
        ],
    };
    let predicted = PredictedTrajectory {
        scenario_id: "unit".to_string(),
        channel_names: vec!["ch1".to_string(), "ch2".to_string()],
        samples: vec![
            VectorSample {
                step: 0,
                time: 0.0,
                values: vec![0.5, 1.0],
            },
            VectorSample {
                step: 1,
                time: 1.0,
                values: vec![2.5, 4.0],
            },
        ],
    };

    let residual = extract_residuals(&observed, &predicted, "unit");
    assert_eq!(residual.samples[0].values, vec![0.5, 1.0]);
    assert_eq!(residual.samples[1].values, vec![0.5, 1.0]);
}

#[test]
fn drift_and_slew_follow_discrete_construction() {
    let observed = ObservedTrajectory {
        scenario_id: "derivatives".to_string(),
        channel_names: vec!["ch1".to_string()],
        samples: vec![
            VectorSample {
                step: 0,
                time: 0.0,
                values: vec![0.0],
            },
            VectorSample {
                step: 1,
                time: 1.0,
                values: vec![1.0],
            },
            VectorSample {
                step: 2,
                time: 2.0,
                values: vec![4.0],
            },
        ],
    };
    let predicted = PredictedTrajectory {
        scenario_id: "derivatives".to_string(),
        channel_names: vec!["ch1".to_string()],
        samples: vec![
            VectorSample {
                step: 0,
                time: 0.0,
                values: vec![0.0],
            },
            VectorSample {
                step: 1,
                time: 1.0,
                values: vec![0.0],
            },
            VectorSample {
                step: 2,
                time: 2.0,
                values: vec![0.0],
            },
        ],
    };
    let residual = extract_residuals(&observed, &predicted, "derivatives");
    let drift = compute_drift_trajectory(&residual, 1.0, "derivatives");
    let slew = compute_slew_trajectory(&residual, 1.0, "derivatives");

    assert!((drift.samples[1].values[0] - 2.0).abs() < 1.0e-9);
    assert!((slew.samples[1].values[0] - 2.0).abs() < 1.0e-9);
}

#[test]
fn sign_construction_contains_residual_drift_and_slew() {
    let observed = ObservedTrajectory {
        scenario_id: "sign".to_string(),
        channel_names: vec!["ch1".to_string()],
        samples: vec![
            VectorSample {
                step: 0,
                time: 0.0,
                values: vec![0.0],
            },
            VectorSample {
                step: 1,
                time: 1.0,
                values: vec![0.3],
            },
            VectorSample {
                step: 2,
                time: 2.0,
                values: vec![0.8],
            },
        ],
    };
    let predicted = PredictedTrajectory {
        scenario_id: "sign".to_string(),
        channel_names: vec!["ch1".to_string()],
        samples: observed
            .samples
            .iter()
            .map(|sample| VectorSample {
                step: sample.step,
                time: sample.time,
                values: vec![0.0],
            })
            .collect(),
    };
    let residual = extract_residuals(&observed, &predicted, "sign");
    let drift = compute_drift_trajectory(&residual, 1.0, "sign");
    let slew = compute_slew_trajectory(&residual, 1.0, "sign");
    let sign = construct_signs(&residual, &drift, &slew);

    assert_eq!(sign.samples.len(), 3);
    assert!((sign.samples[1].residual[0] - 0.3).abs() < 1.0e-9);
    assert_eq!(sign.samples[1].projection.len(), 3);
}

#[test]
fn envelope_crossing_occurs_for_theorem_aligned_exit_case() {
    let definition = all_scenarios()
        .into_iter()
        .find(|scenario| scenario.record.id == "outward_exit_case_a")
        .unwrap();
    let synthesis = synthesize(&definition, 240, 1.0, 123);
    let residual = extract_residuals(
        &synthesis.observed,
        &synthesis.predicted,
        &definition.record.id,
    );
    let envelope = build_envelope(&residual, &definition.envelope_spec, &definition.record.id);
    let grammar = evaluate_grammar_layer(&residual, &envelope);

    assert!(grammar
        .iter()
        .any(|status| matches!(status.state, GrammarState::Violation)));
}

#[test]
fn detectability_bound_is_respected_for_configured_exit_case() {
    let scenarios = all_scenarios();
    let definition = scenarios
        .iter()
        .find(|scenario| scenario.record.id == "outward_exit_case_a")
        .unwrap();
    let nominal = scenarios
        .iter()
        .find(|scenario| scenario.record.id == "nominal_stable")
        .unwrap();
    let synthesis = synthesize(definition, 240, 1.0, 123);
    let nominal_synthesis = synthesize(nominal, 240, 1.0, 123);
    let residual = extract_residuals(
        &synthesis.observed,
        &synthesis.predicted,
        &definition.record.id,
    );
    let nominal_residual = extract_residuals(
        &nominal_synthesis.observed,
        &nominal_synthesis.predicted,
        &nominal.record.id,
    );
    let envelope = build_envelope(&residual, &definition.envelope_spec, &definition.record.id);
    let grammar = evaluate_grammar_layer(&residual, &envelope);
    let detectability = evaluate_detectability(
        &residual,
        &grammar,
        definition.detectability_inputs.clone(),
        Some(&nominal_residual),
    );

    assert_eq!(detectability.bound_satisfied, Some(true));
    let dt = residual.samples[1].time - residual.samples[0].time;
    assert!(
        detectability.predicted_upper_bound.unwrap() + dt
            >= detectability.observed_crossing_time.unwrap()
    );
}

#[test]
fn deterministic_reproducibility_check_is_true() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 120,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: dsfb_semiotics_engine::ScenarioSelection::Single(
            "nominal_stable".to_string(),
        ),
    });

    let bundle = engine.run_single("nominal_stable").unwrap();
    assert!(bundle.reproducibility_check.identical);
}

#[test]
fn output_path_creation_builds_expected_subdirectories() {
    let temp = TempDir::new().unwrap();
    let layout = create_output_layout(temp.path()).unwrap();

    assert!(layout.run_dir.starts_with(temp.path()));
    assert!(layout.figures_dir.exists());
    assert!(layout.csv_dir.exists());
    assert!(layout.json_dir.exists());
    assert!(layout.report_dir.exists());
}

#[test]
fn semantics_layer_can_return_unknown() {
    let syntax = SyntaxCharacterization {
        scenario_id: "unknown".to_string(),
        outward_drift_fraction: 0.2,
        inward_drift_fraction: 0.2,
        monotone_drift_fraction: 0.2,
        curvature_energy: 0.01,
        max_slew_norm: 0.05,
        slew_spike_count: 0,
        repeated_grazing_count: 0,
        trajectory_label: "mixed-structured".to_string(),
    };
    let result = retrieve_semantics("unknown", &syntax, &[], None);
    assert!(matches!(result.disposition, SemanticDisposition::Unknown));
}
