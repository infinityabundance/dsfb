use std::path::PathBuf;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::types::{EnvelopeMode, ResidualSample, ResidualTrajectory};
use dsfb_semiotics_engine::io::input::load_csv_trajectories;
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::metrics::{
    euclidean_norm, hash_serializable_hex, project_sign, residual_norm_path_monotonicity,
    scalar_derivative, sign_with_deadband, trend_aligned_increment_fraction,
};
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn sign_deadband_grid_is_symmetric_and_bounded() {
    for deadband in [1.0e-9, 1.0e-6, 1.0e-3] {
        for value in [
            -10.0,
            -1.0e-2,
            -deadband,
            -deadband / 2.0,
            0.0,
            deadband / 2.0,
            deadband,
            1.0e-2,
            10.0,
        ] {
            let sign = sign_with_deadband(value, deadband);
            let mirrored = sign_with_deadband(-value, deadband);
            assert!(matches!(sign, -1..=1));
            assert_eq!(sign, -mirrored);
        }
    }
}

#[test]
fn norms_and_sign_projection_stay_finite_on_deterministic_grid() {
    for left in -3..=3 {
        for right in -3..=3 {
            let residual = [left as f64 * 0.5, right as f64 * -0.25];
            let drift = [right as f64 * 0.1, left as f64 * -0.2];
            let slew = [left as f64 * 0.01, right as f64 * 0.02];
            let norm = euclidean_norm(&residual);
            let projection = project_sign(&residual, &drift, &slew);

            assert!(norm.is_finite());
            assert!(norm >= 0.0);
            assert!(projection.iter().all(|value| value.is_finite()));
            assert!(projection[0] >= 0.0);
            assert!(projection[2] >= 0.0);
        }
    }
}

#[test]
fn scalar_derivative_preserves_constant_and_linear_paths_on_grids() {
    let constant_times = [0.0, 1.0, 2.0, 3.0];
    for constant in [-5.0, -1.0, 0.0, 3.5] {
        let derivative =
            scalar_derivative(&[constant, constant, constant, constant], &constant_times);
        assert!(derivative.iter().all(|value| value.abs() <= 1.0e-12));
    }

    let nonuniform_times = [0.0, 0.5, 1.5, 3.0];
    for intercept in [-2.0, 0.0, 1.5] {
        for slope in [-3.0, -0.5, 0.0, 2.0] {
            let values = nonuniform_times
                .iter()
                .map(|time| intercept + slope * time)
                .collect::<Vec<_>>();
            let derivative = scalar_derivative(&values, &nonuniform_times);
            assert!(derivative
                .iter()
                .all(|value| (*value - slope).abs() <= 1.0e-9));
        }
    }
}

#[test]
fn slew_is_zero_for_constant_and_affine_residual_paths() {
    for values in [
        vec![0.0, 0.0, 0.0, 0.0],
        vec![1.0, 2.0, 3.0, 4.0],
        vec![-3.0, -1.0, 1.0, 3.0],
    ] {
        let residual = ResidualTrajectory {
            scenario_id: "affine_case".to_string(),
            channel_names: vec!["x".to_string()],
            samples: values
                .iter()
                .enumerate()
                .map(|(step, value)| ResidualSample {
                    step,
                    time: step as f64 * 0.5,
                    values: vec![*value],
                    norm: value.abs(),
                })
                .collect(),
        };
        let drift = compute_drift_trajectory(&residual, 0.5, "affine_case");
        let slew = compute_slew_trajectory(&residual, 0.5, "affine_case");

        assert!(drift.samples.iter().all(|sample| sample.norm.is_finite()));
        assert!(slew
            .samples
            .iter()
            .all(|sample| sample.norm.abs() <= 1.0e-10));
    }
}

#[test]
fn monotonicity_summaries_stay_within_unit_interval_on_grid_sequences() {
    let sequences = [
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 2.0, 3.0],
        vec![3.0, 2.0, 1.0, 0.0],
        vec![0.0, 1.0, 0.5, 1.5, 1.0],
        vec![0.0, 0.5, 0.0, 0.5, 0.0],
    ];
    for sequence in sequences {
        let monotonicity = residual_norm_path_monotonicity(&sequence);
        let trend_alignment = trend_aligned_increment_fraction(&sequence, 1.0e-9);
        assert!((0.0..=1.0).contains(&monotonicity));
        assert!((0.0..=1.0).contains(&trend_alignment));
    }
}

#[test]
fn hash_stability_and_external_bank_loading_are_repeatable() {
    let first = HeuristicBankRegistry::load_external_json(
        fixture_path("external_bank_minimal.json").as_path(),
        true,
    )
    .unwrap();
    let second = HeuristicBankRegistry::load_external_json(
        fixture_path("external_bank_minimal.json").as_path(),
        true,
    )
    .unwrap();
    let builtin_hash_a = hash_serializable_hex("builtin", &HeuristicBankRegistry::builtin())
        .unwrap()
        .fnv1a_64_hex;
    let builtin_hash_b = hash_serializable_hex("builtin", &HeuristicBankRegistry::builtin())
        .unwrap()
        .fnv1a_64_hex;

    assert_eq!(first.1.content_hash, second.1.content_hash);
    assert_eq!(builtin_hash_a, builtin_hash_b);
    assert_eq!(first.2.valid, second.2.valid);
}

#[test]
fn csv_loader_rejects_repeated_explicit_timestamps() {
    let temp = tempdir().unwrap();
    let observed = temp.path().join("observed.csv");
    let predicted = temp.path().join("predicted.csv");
    let csv = "time,ax,ay\n0.0,0.0,0.0\n0.5,0.1,0.0\n0.5,0.2,0.1\n";
    std::fs::write(&observed, csv).unwrap();
    std::fs::write(&predicted, csv).unwrap();

    let error = load_csv_trajectories(&CsvInputConfig {
        observed_csv: observed,
        predicted_csv: predicted,
        scenario_id: "repeated_time".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 1.0,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture".to_string(),
    })
    .unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("must have strictly increasing time values"));
}
