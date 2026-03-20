use std::path::PathBuf;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer;
use dsfb_semiotics_engine::engine::types::{
    EnvelopeMode, GrammarState, ResidualSample, ResidualTrajectory,
};
use dsfb_semiotics_engine::io::input::load_csv_trajectories;
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::envelope::{build_envelope, EnvelopeSpec};
use dsfb_semiotics_engine::math::metrics::project_sign;
use tempfile::tempdir;

fn csv_config(
    observed_csv: PathBuf,
    predicted_csv: PathBuf,
    time_column: Option<&str>,
) -> CsvInputConfig {
    CsvInputConfig {
        observed_csv,
        predicted_csv,
        scenario_id: "edge_case".to_string(),
        channel_names: None,
        time_column: time_column.map(std::string::ToString::to_string),
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture".to_string(),
    }
}

fn residual(values: &[(f64, &[f64])]) -> ResidualTrajectory {
    ResidualTrajectory {
        scenario_id: "edge_case".to_string(),
        channel_names: vec!["x".to_string()],
        samples: values
            .iter()
            .enumerate()
            .map(|(step, (time, sample_values))| ResidualSample {
                step,
                time: *time,
                values: sample_values.to_vec(),
                norm: sample_values
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>()
                    .sqrt(),
            })
            .collect(),
    }
}

#[test]
fn test_length_one_trajectory() {
    let residual = residual(&[(0.0, &[0.0])]);
    let drift = compute_drift_trajectory(&residual, 1.0, "edge_case");
    let slew = compute_slew_trajectory(&residual, 1.0, "edge_case");

    assert_eq!(drift.samples.len(), 1);
    assert_eq!(slew.samples.len(), 1);
    assert_eq!(drift.samples[0].norm, 0.0);
    assert_eq!(slew.samples[0].norm, 0.0);
}

#[test]
fn test_length_two_trajectory() {
    let residual = residual(&[(0.0, &[0.0]), (1.0, &[1.0])]);
    let drift = compute_drift_trajectory(&residual, 1.0, "edge_case");
    let slew = compute_slew_trajectory(&residual, 1.0, "edge_case");

    assert_eq!(drift.samples.len(), 2);
    assert_eq!(slew.samples.len(), 2);
    assert!(drift.samples.iter().all(|sample| sample.norm.is_finite()));
    assert!(slew.samples.iter().all(|sample| sample.norm.is_finite()));
}

#[test]
fn test_zero_norm_residual_projection_safe() {
    let projection = project_sign(&[0.0, 0.0], &[1.0, -1.0], &[0.0, 0.0]);
    assert!(projection.iter().all(|value| value.is_finite()));
    assert_eq!(projection[0], 0.0);
    assert_eq!(projection[1], 0.0);
}

#[test]
fn test_boundary_only_trajectory() {
    let residual = residual(&[(0.0, &[0.98]), (1.0, &[0.98]), (2.0, &[0.98])]);
    let envelope = build_envelope(
        &residual,
        &EnvelopeSpec {
            name: "boundary".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        "edge_case",
    );
    let grammar = evaluate_grammar_layer(&residual, &envelope);

    assert!(grammar
        .iter()
        .all(|status| matches!(status.state, GrammarState::Boundary)));
}

#[test]
fn test_exact_zero_slew_case() {
    let residual = residual(&[(0.0, &[1.0]), (1.0, &[2.0]), (2.0, &[3.0]), (3.0, &[4.0])]);
    let slew = compute_slew_trajectory(&residual, 1.0, "edge_case");

    assert!(slew
        .samples
        .iter()
        .all(|sample| sample.norm.abs() <= 1.0e-10));
}

#[test]
fn test_empty_csv_fails_cleanly() {
    let temp = tempdir().unwrap();
    let observed = temp.path().join("observed.csv");
    let predicted = temp.path().join("predicted.csv");
    std::fs::write(&observed, "").unwrap();
    std::fs::write(&predicted, "").unwrap();

    let error = load_csv_trajectories(&csv_config(observed, predicted, None)).unwrap_err();
    let message = format!("{error:#}");
    assert!(
        message.contains("failed to parse observed CSV")
            || message.contains("must contain at least one channel column")
    );
}

#[test]
fn test_repeated_timestamp_policy() {
    let temp = tempdir().unwrap();
    let observed = temp.path().join("observed.csv");
    let predicted = temp.path().join("predicted.csv");
    let csv = "time,x\n0.0,0.0\n0.5,0.1\n0.5,0.2\n";
    std::fs::write(&observed, csv).unwrap();
    std::fs::write(&predicted, csv).unwrap();

    let error = load_csv_trajectories(&csv_config(observed, predicted, Some("time"))).unwrap_err();
    assert!(format!("{error:#}").contains("strictly increasing time values"));
}

#[test]
fn test_missing_time_column_with_dt_fallback() {
    let temp = tempdir().unwrap();
    let observed = temp.path().join("observed.csv");
    let predicted = temp.path().join("predicted.csv");
    let csv = "x\n1.0\n1.5\n2.0\n";
    std::fs::write(&observed, csv).unwrap();
    std::fs::write(&predicted, csv).unwrap();

    let (observed_traj, predicted_traj) =
        load_csv_trajectories(&csv_config(observed, predicted, None)).unwrap();

    assert_eq!(observed_traj.samples[1].time, 0.5);
    assert_eq!(predicted_traj.samples[2].time, 1.0);
}

#[test]
fn test_malformed_bank_file_fails_cleanly() {
    let temp = tempdir().unwrap();
    let bank_path = temp.path().join("malformed_bank.json");
    std::fs::write(&bank_path, "{not-json").unwrap();

    let error = HeuristicBankRegistry::load_external_json(bank_path.as_path(), true).unwrap_err();
    assert!(error.to_string().contains("parse heuristic bank JSON"));
}
