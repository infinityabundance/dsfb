use dsfb_semiotics_engine::engine::types::{ResidualSample, ResidualTrajectory};
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::metrics::{
    euclidean_norm, hash_serializable_hex, project_sign, residual_norm_path_monotonicity,
    scalar_derivative, sign_with_deadband, trend_aligned_increment_fraction,
};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed, TestRunner};

const PROPTEST_SEED: u64 = 0xD5FB_5EED_2026_0320;
const CASES: u32 = 64;

fn deterministic_runner() -> TestRunner {
    TestRunner::new(Config {
        cases: CASES,
        failure_persistence: None,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(PROPTEST_SEED),
        ..Config::default()
    })
}

fn increasing_times(len: usize, increments: &[u16]) -> Vec<f64> {
    let mut time = 0.0;
    let mut times = Vec::with_capacity(len.max(1));
    times.push(time);
    for increment in increments.iter().take(len.saturating_sub(1)) {
        time += f64::from(*increment) * 0.25;
        times.push(time);
    }
    times
}

fn residual_trajectory_from_values(values: &[f64], times: &[f64]) -> ResidualTrajectory {
    ResidualTrajectory {
        scenario_id: "proptest".to_string(),
        channel_names: vec!["x".to_string()],
        samples: values
            .iter()
            .enumerate()
            .map(|(step, value)| ResidualSample {
                step,
                time: times[step],
                values: vec![*value],
                norm: value.abs(),
            })
            .collect(),
    }
}

#[test]
fn proptest_sign_deadband_is_bounded_and_symmetric() {
    let strategy = (-10_000i32..=10_000, 1u32..=10_000u32);
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |(value_raw, deadband_raw)| {
            let value = f64::from(value_raw) * 1.0e-3;
            let deadband = f64::from(deadband_raw) * 1.0e-6;
            let sign = sign_with_deadband(value, deadband);
            let mirrored = sign_with_deadband(-value, deadband);

            prop_assert!(matches!(sign, -1..=1));
            prop_assert_eq!(sign, -mirrored);
            Ok(())
        })
        .unwrap();
}

#[test]
fn proptest_projection_norms_are_nonnegative_and_zero_residual_gives_zero_radial_component() {
    let strategy = (1usize..=6).prop_flat_map(|len| {
        (
            proptest::collection::vec(-200i32..=200, len),
            proptest::collection::vec(-200i32..=200, len),
        )
    });
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |(drift_raw, slew_raw)| {
            let residual = vec![0.0; drift_raw.len()];
            let drift = drift_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-2)
                .collect::<Vec<_>>();
            let slew = slew_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-3)
                .collect::<Vec<_>>();
            let projection = project_sign(&residual, &drift, &slew);

            prop_assert!(euclidean_norm(&residual) >= 0.0);
            prop_assert!(projection.iter().all(|value| value.is_finite()));
            prop_assert!(projection[0] >= 0.0);
            prop_assert!(projection[2] >= 0.0);
            prop_assert!(projection[1].abs() <= 1.0e-12);
            Ok(())
        })
        .unwrap();
}

#[test]
fn proptest_scalar_derivative_preserves_linear_paths_on_nonuniform_times() {
    let strategy = (2usize..=8, -200i32..=200, -100i32..=100).prop_flat_map(
        |(len, intercept_raw, slope_raw)| {
            proptest::collection::vec(1u16..=8, len - 1).prop_map(move |increments| {
                let times = increasing_times(len, &increments);
                let intercept = f64::from(intercept_raw) * 0.1;
                let slope = f64::from(slope_raw) * 0.05;
                (times, intercept, slope)
            })
        },
    );
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |(times, intercept, slope)| {
            let values = times
                .iter()
                .map(|time| intercept + slope * *time)
                .collect::<Vec<_>>();
            let derivative = scalar_derivative(&values, &times);

            prop_assert!(derivative.iter().all(|value| value.is_finite()));
            prop_assert!(derivative
                .iter()
                .all(|value| (*value - slope).abs() <= 1.0e-9));
            Ok(())
        })
        .unwrap();
}

#[test]
fn proptest_slew_is_zero_for_affine_paths() {
    let strategy = (3usize..=8, -200i32..=200, -100i32..=100).prop_flat_map(
        |(len, intercept_raw, slope_raw)| {
            proptest::collection::vec(1u16..=8, len - 1).prop_map(move |increments| {
                let times = increasing_times(len, &increments);
                let intercept = f64::from(intercept_raw) * 0.1;
                let slope = f64::from(slope_raw) * 0.05;
                (times, intercept, slope)
            })
        },
    );
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |(times, intercept, slope)| {
            let values = times
                .iter()
                .map(|time| intercept + slope * *time)
                .collect::<Vec<_>>();
            let residual = residual_trajectory_from_values(&values, &times);
            let drift = compute_drift_trajectory(&residual, 1.0, "proptest");
            let slew = compute_slew_trajectory(&residual, 1.0, "proptest");

            prop_assert!(drift.samples.iter().all(|sample| sample.norm.is_finite()));
            prop_assert!(slew.samples.iter().all(|sample| sample.norm.is_finite()));
            prop_assert!(slew
                .samples
                .iter()
                .all(|sample| sample.norm.abs() <= 1.0e-10));
            Ok(())
        })
        .unwrap();
}

#[test]
fn proptest_monotonicity_fractions_remain_bounded() {
    let strategy = proptest::collection::vec(-500i32..=500, 1..=12);
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |values_raw| {
            let values = values_raw
                .into_iter()
                .map(|value| f64::from(value) * 1.0e-2)
                .collect::<Vec<_>>();
            let path_monotonicity = residual_norm_path_monotonicity(&values);
            let trend_alignment = trend_aligned_increment_fraction(&values, 1.0e-9);

            prop_assert!(path_monotonicity.is_finite());
            prop_assert!(trend_alignment.is_finite());
            prop_assert!((0.0..=1.0).contains(&path_monotonicity));
            prop_assert!((0.0..=1.0).contains(&trend_alignment));
            Ok(())
        })
        .unwrap();
}

#[test]
fn proptest_serialized_hash_is_stable_for_repeated_materialization() {
    let strategy = (1usize..=4).prop_flat_map(|outer_len| {
        proptest::collection::vec(proptest::collection::vec(-200i32..=200, 0..=6), outer_len)
    });
    let mut runner = deterministic_runner();
    runner
        .run(&strategy, |nested_raw| {
            let nested = nested_raw
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|value| f64::from(value) * 1.0e-3)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let first = hash_serializable_hex("materialized", &nested).unwrap();
            let second = hash_serializable_hex("materialized", &nested).unwrap();

            prop_assert_eq!(first.fnv1a_64_hex, second.fnv1a_64_hex);
            Ok(())
        })
        .unwrap();
}
