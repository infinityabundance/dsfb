use dsfb_semiotics_engine::math::metrics::{
    hash_serializable_hex, project_sign, residual_norm_path_monotonicity,
    trend_aligned_increment_fraction,
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
