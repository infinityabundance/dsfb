use dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer;
use dsfb_semiotics_engine::engine::types::{
    AdmissibilityEnvelope, EnvelopeMode, EnvelopeSample, ResidualSample, ResidualTrajectory,
};
use dsfb_semiotics_engine::math::derivatives::{compute_drift_trajectory, compute_slew_trajectory};
use dsfb_semiotics_engine::math::metrics::{
    euclidean_norm, hash_serializable_hex, project_sign, residual_norm_path_monotonicity,
    scalar_derivative, sign_with_deadband, trend_aligned_increment_fraction,
};

#[test]
fn zero_residual_projection_reports_zero_signed_radial_component() {
    let projection = project_sign(&[0.0, 0.0], &[2.0, -1.0], &[3.0, 4.0]);
    assert_eq!(projection[0], 0.0);
    assert_eq!(projection[1], 0.0);
    assert!((projection[2] - 5.0).abs() <= 1.0e-12);
}

#[test]
fn monotone_residual_norm_path_has_consistent_summary_metrics() {
    let values = [0.0, 1.0, 2.0, 3.0, 4.0];
    assert!((residual_norm_path_monotonicity(&values) - 1.0).abs() <= 1.0e-12);
    assert!((trend_aligned_increment_fraction(&values, 1.0e-9) - 1.0).abs() <= 1.0e-12);
}

#[test]
fn hash_serialization_is_stable_for_identical_values() {
    let value = vec![1.0_f64, 2.5, 3.0];
    let first = hash_serializable_hex("first", &value).unwrap();
    let second = hash_serializable_hex("second", &value).unwrap();

    assert_eq!(first.fnv1a_64_hex, second.fnv1a_64_hex);
}

#[test]
fn sign_deadband_is_symmetric_around_zero() {
    assert_eq!(sign_with_deadband(1.0e-7, 1.0e-6), 0);
    assert_eq!(sign_with_deadband(-1.0e-7, 1.0e-6), 0);
    assert_eq!(sign_with_deadband(1.0e-3, 1.0e-6), 1);
    assert_eq!(sign_with_deadband(-1.0e-3, 1.0e-6), -1);
}

#[test]
fn euclidean_norm_is_nonnegative() {
    assert!(euclidean_norm(&[0.0, -3.0, 4.0]) >= 0.0);
}

#[test]
fn scalar_derivative_is_zero_for_constant_path() {
    let derivative = scalar_derivative(&[2.0, 2.0, 2.0, 2.0], &[0.0, 1.0, 2.0, 3.0]);
    assert!(derivative.iter().all(|value| value.abs() <= 1.0e-12));
}

#[test]
fn scalar_derivative_is_constant_for_linear_path() {
    let derivative = scalar_derivative(&[1.0, 3.0, 5.0, 7.0], &[0.0, 1.0, 2.0, 3.0]);
    assert!(derivative
        .iter()
        .all(|value| (*value - 2.0).abs() <= 1.0e-12));
}

#[test]
fn monotonicity_score_stays_within_unit_interval() {
    let score = residual_norm_path_monotonicity(&[0.0, 1.0, 0.5, 1.5, 1.0]);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn short_trajectories_keep_drift_and_slew_finite() {
    let one_step = ResidualTrajectory {
        scenario_id: "one_step".to_string(),
        channel_names: vec!["x".to_string()],
        samples: vec![ResidualSample {
            step: 0,
            time: 0.0,
            values: vec![1.0],
            norm: 1.0,
        }],
    };
    let two_step = ResidualTrajectory {
        scenario_id: "two_step".to_string(),
        channel_names: vec!["x".to_string()],
        samples: vec![
            ResidualSample {
                step: 0,
                time: 0.0,
                values: vec![1.0],
                norm: 1.0,
            },
            ResidualSample {
                step: 1,
                time: 1.0,
                values: vec![1.5],
                norm: 1.5,
            },
        ],
    };

    for residual in [one_step, two_step] {
        let drift = compute_drift_trajectory(&residual, 1.0, &residual.scenario_id);
        let slew = compute_slew_trajectory(&residual, 1.0, &residual.scenario_id);

        assert!(drift.samples.iter().all(|sample| sample.norm.is_finite()));
        assert!(slew.samples.iter().all(|sample| sample.norm.is_finite()));
        assert!(slew
            .samples
            .iter()
            .all(|sample| sample.norm.abs() <= 1.0e-12));
    }
}

#[test]
fn boundary_only_path_stays_boundary_without_violation() {
    let residual = ResidualTrajectory {
        scenario_id: "boundary_only".to_string(),
        channel_names: vec!["x".to_string()],
        samples: (0..4)
            .map(|step| ResidualSample {
                step,
                time: step as f64,
                values: vec![0.98],
                norm: 0.98,
            })
            .collect(),
    };
    let envelope = AdmissibilityEnvelope {
        scenario_id: "boundary_only".to_string(),
        name: "fixed_boundary".to_string(),
        mode: EnvelopeMode::Fixed,
        samples: (0..4)
            .map(|step| EnvelopeSample {
                step,
                time: step as f64,
                radius: 1.0,
                derivative_bound: 0.0,
                regime: "fixed".to_string(),
            })
            .collect(),
    };

    let grammar = evaluate_grammar_layer(&residual, &envelope);
    assert_eq!(grammar.len(), 4);
    assert!(grammar.iter().all(|status| {
        matches!(
            status.state,
            dsfb_semiotics_engine::engine::types::GrammarState::Boundary
        )
    }));
}
