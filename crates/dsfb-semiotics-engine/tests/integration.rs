use std::fs;

use clap::Parser;
use dsfb_semiotics_engine::cli::args::{CsvInputConfig, ScenarioSelection};
use dsfb_semiotics_engine::engine::grammar_layer::{
    evaluate_detectability, evaluate_grammar_layer,
};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::residual_layer::extract_residuals;
use dsfb_semiotics_engine::engine::semantics_layer::retrieve_semantics;
use dsfb_semiotics_engine::engine::sign_layer::construct_signs;
use dsfb_semiotics_engine::engine::syntax_layer::characterize_syntax;
use dsfb_semiotics_engine::engine::types::{
    EnvelopeMode, GrammarState, GrammarStatus, ObservedTrajectory, PredictedTrajectory,
    SemanticDisposition, SignSample, SignTrajectory, SyntaxCharacterization, VectorSample,
};
use dsfb_semiotics_engine::io::output::create_output_layout;
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::envelope::build_envelope;
use dsfb_semiotics_engine::math::metrics::sign_projection_metadata;
use dsfb_semiotics_engine::sim::generators::synthesize;
use dsfb_semiotics_engine::sim::scenarios::all_scenarios;
use tempfile::TempDir;

#[test]
fn residual_computation_matches_observation_minus_prediction() {
    let observed = trajectory(
        "unit",
        &["ch1", "ch2"],
        &[(0.0, &[1.0, 2.0]), (1.0, &[3.0, 5.0])],
    );
    let predicted = predicted_trajectory(
        "unit",
        &["ch1", "ch2"],
        &[(0.0, &[0.5, 1.0]), (1.0, &[2.5, 4.0])],
    );

    let residual = extract_residuals(&observed, &predicted, "unit");
    assert_eq!(residual.samples[0].values, vec![0.5, 1.0]);
    assert_eq!(residual.samples[1].values, vec![0.5, 1.0]);
}

#[test]
fn drift_and_slew_follow_discrete_construction() {
    let observed = trajectory(
        "derivatives",
        &["ch1"],
        &[(0.0, &[0.0]), (1.0, &[1.0]), (2.0, &[4.0])],
    );
    let predicted = predicted_trajectory(
        "derivatives",
        &["ch1"],
        &[(0.0, &[0.0]), (1.0, &[0.0]), (2.0, &[0.0])],
    );
    let residual = extract_residuals(&observed, &predicted, "derivatives");
    let drift = compute_drift_trajectory(&residual, 1.0, "derivatives");
    let slew = compute_slew_trajectory(&residual, 1.0, "derivatives");

    assert!((drift.samples[1].values[0] - 2.0).abs() < 1.0e-9);
    assert!((slew.samples[1].values[0] - 2.0).abs() < 1.0e-9);
}

#[test]
fn sign_projection_uses_aggregate_multi_channel_features() {
    let observed = trajectory(
        "sign",
        &["x", "y"],
        &[(0.0, &[0.0, 0.0]), (1.0, &[3.0, 4.0]), (2.0, &[6.0, 8.0])],
    );
    let predicted = predicted_trajectory(
        "sign",
        &["x", "y"],
        &[(0.0, &[0.0, 0.0]), (1.0, &[0.0, 0.0]), (2.0, &[0.0, 0.0])],
    );
    let residual = extract_residuals(&observed, &predicted, "sign");
    let drift = compute_drift_trajectory(&residual, 1.0, "sign");
    let slew = compute_slew_trajectory(&residual, 1.0, "sign");
    let sign = construct_signs(&residual, &drift, &slew);

    assert_eq!(sign.projection_metadata.axis_labels[0], "||r(t)||");
    assert_eq!(
        sign.projection_metadata.axis_labels[1],
        "signed aggregate drift"
    );
    assert!((sign.samples[1].projection[0] - 5.0).abs() < 1.0e-9);
    assert!((sign.samples[1].projection[1] - 5.0).abs() < 1.0e-9);
    assert_ne!(sign.samples[1].projection[0], sign.samples[1].residual[0]);
}

#[test]
fn multi_channel_syntax_characterization_is_not_channel_zero_biased() {
    let sign = sign_trajectory(
        "multi",
        vec![
            sign_sample(0, 0.0, vec![0.10, 0.10], vec![-0.02, 0.05], vec![0.0, 0.0]),
            sign_sample(1, 1.0, vec![0.05, 0.20], vec![-0.02, 0.06], vec![0.0, 0.01]),
            sign_sample(2, 2.0, vec![0.00, 0.30], vec![-0.01, 0.07], vec![0.0, 0.01]),
            sign_sample(
                3,
                3.0,
                vec![-0.05, 0.40],
                vec![-0.01, 0.08],
                vec![0.0, 0.01],
            ),
        ],
    );
    let grammar =
        grammar_with_margins("multi", &[0.50, 0.38, 0.24, 0.10], GrammarState::Admissible);

    let syntax = characterize_syntax(&sign, &grammar);

    assert!(syntax.outward_drift_fraction > 0.70);
    assert!(syntax.inward_drift_fraction < 0.20);
    assert!(syntax.aggregate_monotonicity > 0.90);
    assert!(syntax.channel_coherence < 0.30);
}

#[test]
fn outward_and_inward_drift_are_distinguished_by_margin_evolution() {
    let outward_sign = sign_trajectory(
        "outward",
        vec![
            sign_sample(0, 0.0, vec![0.10, 0.00], vec![0.05, 0.00], vec![0.0, 0.0]),
            sign_sample(1, 1.0, vec![0.20, 0.00], vec![0.05, 0.00], vec![0.0, 0.0]),
            sign_sample(2, 2.0, vec![0.30, 0.00], vec![0.05, 0.00], vec![0.0, 0.0]),
        ],
    );
    let inward_sign = sign_trajectory(
        "inward",
        vec![
            sign_sample(0, 0.0, vec![0.30, 0.00], vec![-0.05, 0.00], vec![0.0, 0.0]),
            sign_sample(1, 1.0, vec![0.20, 0.00], vec![-0.05, 0.00], vec![0.0, 0.0]),
            sign_sample(2, 2.0, vec![0.10, 0.00], vec![-0.05, 0.00], vec![0.0, 0.0]),
        ],
    );
    let outward_grammar =
        grammar_with_margins("outward", &[0.30, 0.20, 0.10], GrammarState::Admissible);
    let inward_grammar =
        grammar_with_margins("inward", &[0.10, 0.20, 0.30], GrammarState::Admissible);

    let outward = characterize_syntax(&outward_sign, &outward_grammar);
    let inward = characterize_syntax(&inward_sign, &inward_grammar);

    assert!(outward.outward_drift_fraction > outward.inward_drift_fraction);
    assert!(inward.inward_drift_fraction > inward.outward_drift_fraction);
}

#[test]
fn boundary_grazing_episode_count_tracks_distinct_entries() {
    let sign = sign_trajectory(
        "grazing",
        vec![
            sign_sample(0, 0.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(1, 1.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(2, 2.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(3, 3.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(4, 4.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(5, 5.0, vec![0.1], vec![0.0], vec![0.0]),
            sign_sample(6, 6.0, vec![0.1], vec![0.0], vec![0.0]),
        ],
    );
    let grammar = vec![
        grammar_status("grazing", 0, 0.0, GrammarState::Admissible, 0.30),
        grammar_status("grazing", 1, 1.0, GrammarState::Boundary, 0.02),
        grammar_status("grazing", 2, 2.0, GrammarState::Boundary, 0.01),
        grammar_status("grazing", 3, 3.0, GrammarState::Admissible, 0.10),
        grammar_status("grazing", 4, 4.0, GrammarState::Boundary, 0.02),
        grammar_status("grazing", 5, 5.0, GrammarState::Admissible, 0.10),
        grammar_status("grazing", 6, 6.0, GrammarState::Boundary, 0.02),
    ];

    let syntax = characterize_syntax(&sign, &grammar);
    assert_eq!(syntax.boundary_grazing_episode_count, 3);
    assert_eq!(syntax.boundary_recovery_count, 2);
    assert_eq!(syntax.repeated_grazing_count, 2);
}

#[test]
fn compatible_semantic_multi_match_returns_ranked_shortlist() {
    let syntax = syntax_template("compatible")
        .with_outward(0.70)
        .with_persistence(0.80)
        .with_sign_consistency(0.78)
        .with_coherence(0.72)
        .with_monotonicity(0.82)
        .with_curvature(1.0e-10)
        .with_boundary_episodes(3)
        .with_boundary_recoveries(1);
    let grammar = vec![
        grammar_status("compatible", 0, 0.0, GrammarState::Boundary, 0.02),
        grammar_status("compatible", 1, 1.0, GrammarState::Admissible, 0.10),
        grammar_status("compatible", 2, 2.0, GrammarState::Boundary, 0.02),
    ];

    let result = retrieve_semantics("compatible", &syntax, &grammar, None);
    assert!(matches!(
        result.disposition,
        SemanticDisposition::CompatibleSet
    ));
    assert!(result.selected_labels.len() >= 2);
    assert_eq!(
        result.selected_labels.len(),
        result.selected_heuristic_ids.len()
    );
    assert!(result.compatibility_note.contains("compatible"));
}

#[test]
fn incompatible_semantic_multi_match_is_explicitly_ambiguous() {
    let syntax = syntax_template("ambiguous")
        .with_outward(0.20)
        .with_persistence(0.75)
        .with_sign_consistency(0.72)
        .with_coherence(0.65)
        .with_monotonicity(0.55)
        .with_curvature(0.010)
        .with_boundary_episodes(3)
        .with_boundary_recoveries(1);
    let mut syntax = syntax.0;
    syntax.inward_drift_fraction = 0.72;
    let grammar = vec![
        grammar_status("ambiguous", 0, 0.0, GrammarState::Boundary, 0.02),
        grammar_status("ambiguous", 1, 1.0, GrammarState::Admissible, 0.10),
        grammar_status("ambiguous", 2, 2.0, GrammarState::Boundary, 0.02),
    ];

    let result = retrieve_semantics("ambiguous", &syntax, &grammar, None);
    assert!(matches!(result.disposition, SemanticDisposition::Ambiguous));
    assert!(!result.conflict_notes.is_empty());
}

#[test]
fn monotonicity_is_not_equivalent_to_positive_drift_sign() {
    let sign = sign_trajectory(
        "monotone_inward",
        vec![
            sign_sample(0, 0.0, vec![0.9, 0.3], vec![-0.3, -0.1], vec![0.0, 0.0]),
            sign_sample(1, 1.0, vec![0.6, 0.2], vec![-0.3, -0.1], vec![0.0, 0.0]),
            sign_sample(2, 2.0, vec![0.3, 0.1], vec![-0.3, -0.1], vec![0.0, 0.0]),
            sign_sample(3, 3.0, vec![0.0, 0.0], vec![-0.2, -0.1], vec![0.0, 0.0]),
        ],
    );
    let grammar = grammar_with_margins(
        "monotone_inward",
        &[0.10, 0.22, 0.35, 0.48],
        GrammarState::Admissible,
    );

    let syntax = characterize_syntax(&sign, &grammar);
    assert!(syntax.aggregate_monotonicity > 0.95);
    assert!(syntax.mean_radial_drift < 0.0);
    assert!(syntax.inward_drift_fraction > syntax.outward_drift_fraction);
}

#[test]
fn curvature_case_does_not_collapse_into_monotone_drift_semantics() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 180,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::Single("curvature_onset".to_string()),
    });

    let bundle = engine.run_single("curvature_onset").unwrap();
    let scenario = &bundle.scenario_outputs[0];
    assert!(!scenario
        .semantics
        .selected_labels
        .iter()
        .any(|label| label == "gradual degradation candidate"));
    assert!(scenario.syntax.curvature_energy > 0.0);
}

#[test]
fn abrupt_event_scenario_produces_meaningful_slew_spikes() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 180,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::Single("abrupt_event".to_string()),
    });

    let bundle = engine.run_single("abrupt_event").unwrap();
    let scenario = &bundle.scenario_outputs[0];
    assert!(scenario.syntax.slew_spike_count >= 1);
    assert!(scenario.syntax.slew_spike_strength > 0.0);
    assert!(scenario.syntax.curvature_onset_score > 0.0);
}

#[test]
fn grouped_correlated_scenario_produces_coordinated_semantics() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 240,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::Single("grouped_correlated".to_string()),
    });

    let bundle = engine.run_single("grouped_correlated").unwrap();
    let scenario = &bundle.scenario_outputs[0];
    assert!(scenario.coordinated.is_some());
    assert!(scenario.syntax.channel_coherence > 0.55);
    assert!(scenario
        .semantics
        .selected_labels
        .iter()
        .any(|label| label.contains("correlated degradation")));
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
fn reproducibility_is_checked_for_every_selected_scenario() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::All,
    });

    let bundle = engine.run_all().unwrap();
    assert_eq!(
        bundle.reproducibility_checks.len(),
        bundle.scenario_outputs.len()
    );
    assert!(bundle.reproducibility_summary.all_identical);
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
fn artifact_bundle_contains_manifest_report_zip_and_reproducibility_schema() {
    let temp = TempDir::new().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::Single("gradual_degradation".to_string()),
    });

    let bundle = engine.run_single("gradual_degradation").unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    assert!(exported.manifest_path.exists());
    assert!(exported.report_pdf.exists());
    assert!(exported.zip_path.exists());
    assert!(exported.figure_paths.len() >= 24);

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&exported.manifest_path).unwrap()).unwrap();
    assert!(manifest.get("figure_paths").is_some());
    assert!(manifest.get("json_paths").is_some());
    let pdf_bytes = fs::read(&exported.report_pdf).unwrap();
    assert!(pdf_bytes.len() > 1_000_000);

    let reproducibility_summary = exported.run_dir.join("json/reproducibility_summary.json");
    let reproducibility_checks = exported.run_dir.join("json/reproducibility_checks.json");
    assert!(reproducibility_summary.exists());
    assert!(reproducibility_checks.exists());
}

#[test]
fn csv_ingest_mode_runs_through_same_pipeline() {
    let temp = TempDir::new().unwrap();
    let observed_csv = temp.path().join("observed.csv");
    let predicted_csv = temp.path().join("predicted.csv");
    fs::write(&observed_csv, "time,x,y\n0,1.0,2.0\n1,1.4,2.4\n2,1.9,2.9\n").unwrap();
    fs::write(
        &predicted_csv,
        "time,x,y\n0,0.9,1.9\n1,1.0,2.0\n2,1.1,2.1\n",
    )
    .unwrap();

    let input = CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "csv_case".to_string(),
        channel_names: None,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "csv_env".to_string(),
    };
    let engine = StructuralSemioticsEngine::new(EngineConfig {
        seed: 123,
        steps: 80,
        dt: 1.0,
        output_root: Some(temp.path().join("artifacts")),
        scenario_selection: ScenarioSelection::Csv(input.clone()),
    });

    let bundle = engine.run_csv(&input).unwrap();
    let scenario = &bundle.scenario_outputs[0];
    assert_eq!(scenario.record.id, "csv_case");
    assert_eq!(scenario.observed.channel_names, vec!["x", "y"]);
    assert_eq!(scenario.residual.samples.len(), 3);
}

#[test]
fn csv_cli_mode_exposes_external_data_surface() {
    let args = dsfb_semiotics_engine::cli::args::CliArgs::try_parse_from([
        "dsfb-semiotics-engine",
        "--input-mode",
        "csv",
        "--observed-csv",
        "observed.csv",
        "--predicted-csv",
        "predicted.csv",
        "--scenario-id",
        "csv_case",
        "--channel-names",
        "x,y",
    ])
    .unwrap();

    let selection = args.selection();
    match selection {
        ScenarioSelection::Csv(config) => {
            assert_eq!(config.scenario_id, "csv_case");
            assert_eq!(config.channel_names.unwrap(), vec!["x", "y"]);
        }
        other => panic!("expected CSV selection, got {other:?}"),
    }
}

#[test]
fn semantics_layer_can_return_unknown() {
    let syntax = syntax_template("unknown");
    let result = retrieve_semantics("unknown", &syntax, &[], None);
    assert!(matches!(result.disposition, SemanticDisposition::Unknown));
}

fn trajectory(
    scenario_id: &str,
    channels: &[&str],
    samples: &[(f64, &[f64])],
) -> ObservedTrajectory {
    ObservedTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: channels.iter().map(|name| (*name).to_string()).collect(),
        samples: samples
            .iter()
            .enumerate()
            .map(|(step, (time, values))| VectorSample {
                step,
                time: *time,
                values: values.to_vec(),
            })
            .collect(),
    }
}

fn predicted_trajectory(
    scenario_id: &str,
    channels: &[&str],
    samples: &[(f64, &[f64])],
) -> PredictedTrajectory {
    PredictedTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: channels.iter().map(|name| (*name).to_string()).collect(),
        samples: samples
            .iter()
            .enumerate()
            .map(|(step, (time, values))| VectorSample {
                step,
                time: *time,
                values: values.to_vec(),
            })
            .collect(),
    }
}

fn sign_trajectory(scenario_id: &str, samples: Vec<SignSample>) -> SignTrajectory {
    SignTrajectory {
        scenario_id: scenario_id.to_string(),
        channel_names: vec!["x".to_string(), "y".to_string()],
        projection_metadata: sign_projection_metadata(),
        samples,
    }
}

fn sign_sample(
    step: usize,
    time: f64,
    residual: Vec<f64>,
    drift: Vec<f64>,
    slew: Vec<f64>,
) -> SignSample {
    let residual_norm = residual
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    let drift_norm = drift.iter().map(|value| value * value).sum::<f64>().sqrt();
    let slew_norm = slew.iter().map(|value| value * value).sum::<f64>().sqrt();
    SignSample {
        step,
        time,
        residual: residual.clone(),
        drift: drift.clone(),
        slew: slew.clone(),
        residual_norm,
        drift_norm,
        slew_norm,
        projection: [
            residual_norm,
            if residual_norm > 0.0 {
                residual.iter().zip(&drift).map(|(r, d)| r * d).sum::<f64>() / residual_norm
            } else {
                0.0
            },
            slew_norm,
        ],
    }
}

fn grammar_with_margins(
    scenario_id: &str,
    margins: &[f64],
    default_state: GrammarState,
) -> Vec<GrammarStatus> {
    margins
        .iter()
        .enumerate()
        .map(|(step, margin)| {
            let state = if *margin < 0.0 {
                GrammarState::Violation
            } else if *margin < 0.04 {
                GrammarState::Boundary
            } else {
                default_state
            };
            grammar_status(scenario_id, step, step as f64, state, *margin)
        })
        .collect()
}

fn grammar_status(
    scenario_id: &str,
    step: usize,
    time: f64,
    state: GrammarState,
    margin: f64,
) -> GrammarStatus {
    GrammarStatus {
        scenario_id: scenario_id.to_string(),
        step,
        time,
        state,
        margin,
        radius: 1.0,
        residual_norm: 1.0 - margin,
        regime: "fixed".to_string(),
    }
}

#[derive(Clone)]
struct SyntaxTemplate(SyntaxCharacterization);

impl SyntaxTemplate {
    fn with_outward(mut self, value: f64) -> Self {
        self.0.outward_drift_fraction = value;
        self
    }

    fn with_persistence(mut self, value: f64) -> Self {
        self.0.directional_persistence = value;
        self
    }

    fn with_sign_consistency(mut self, value: f64) -> Self {
        self.0.sign_consistency = value;
        self
    }

    fn with_coherence(mut self, value: f64) -> Self {
        self.0.channel_coherence = value;
        self
    }

    fn with_monotonicity(mut self, value: f64) -> Self {
        self.0.aggregate_monotonicity = value;
        self.0.monotone_drift_fraction = value;
        self
    }

    fn with_curvature(mut self, value: f64) -> Self {
        self.0.curvature_energy = value;
        self
    }

    fn with_boundary_episodes(mut self, value: usize) -> Self {
        self.0.boundary_grazing_episode_count = value;
        self.0.repeated_grazing_count = value.saturating_sub(1);
        self
    }

    fn with_boundary_recoveries(mut self, value: usize) -> Self {
        self.0.boundary_recovery_count = value;
        self
    }
}

impl std::ops::Deref for SyntaxTemplate {
    type Target = SyntaxCharacterization;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn syntax_template(scenario_id: &str) -> SyntaxTemplate {
    SyntaxTemplate(SyntaxCharacterization {
        scenario_id: scenario_id.to_string(),
        outward_drift_fraction: 0.2,
        inward_drift_fraction: 0.2,
        sign_consistency: 0.2,
        directional_persistence: 0.2,
        channel_coherence: 0.2,
        aggregate_monotonicity: 0.2,
        monotone_drift_fraction: 0.2,
        curvature_energy: 0.01,
        curvature_onset_score: 0.1,
        mean_radial_drift: 0.0,
        min_margin: 0.1,
        mean_margin_delta: 0.0,
        max_slew_norm: 0.05,
        slew_spike_count: 0,
        slew_spike_strength: 0.0,
        boundary_grazing_episode_count: 0,
        boundary_recovery_count: 0,
        repeated_grazing_count: 0,
        trajectory_label: "mixed-structured".to_string(),
    })
}
