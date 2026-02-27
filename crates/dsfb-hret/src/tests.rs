use super::HretObserver;

fn make_observer() -> HretObserver {
    HretObserver::new(
        2,
        2,
        vec![0, 1],
        0.5,
        vec![0.5, 0.5],
        vec![1.0, 1.0],
        vec![1.0, 1.0],
        vec![vec![1.0, 1.0]],
    )
    .expect("observer construction should succeed")
}

#[test]
fn update_produces_convex_weights_and_expected_correction() {
    let mut obs = make_observer();
    let (delta_x, weights, s_k, s_g) = obs.update(vec![1.0, 1.0]).expect("update should succeed");

    assert_eq!(delta_x.len(), 1);
    assert!((delta_x[0] - 1.0).abs() < 1e-12);

    assert_eq!(weights.len(), 2);
    assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!((weights[0] - 0.5).abs() < 1e-12);
    assert!((weights[1] - 0.5).abs() < 1e-12);

    assert_eq!(s_k.len(), 2);
    assert_eq!(s_g.len(), 2);
}

#[test]
fn reset_envelopes_zeroes_envelope_state() {
    let mut obs = make_observer();
    let _ = obs.update(vec![0.5, -0.25]).expect("update should succeed");
    obs.reset_envelopes();

    let (_, _, s_k, s_g) = obs.update(vec![0.0, 0.0]).expect("update should succeed");
    assert!(s_k.iter().all(|&x| x.abs() < 1e-12));
    assert!(s_g.iter().all(|&x| x.abs() < 1e-12));
}

#[test]
fn constructor_rejects_invalid_group_mapping_length() {
    let error = HretObserver::new(
        2,
        1,
        vec![0],
        0.95,
        vec![0.9],
        vec![1.0, 1.0],
        vec![1.0],
        vec![vec![1.0, 1.0]],
    )
    .expect_err("constructor should reject invalid mapping length");

    assert!(error.to_string().contains("group_mapping"));
}

#[test]
fn constructor_rejects_out_of_range_group_indices() {
    let error = HretObserver::new(
        2,
        1,
        vec![0, 1],
        0.95,
        vec![0.9],
        vec![1.0, 1.0],
        vec![1.0],
        vec![vec![1.0, 1.0]],
    )
    .expect_err("constructor should reject out-of-range group index");

    assert!(error.to_string().contains("out of range"));
}

#[test]
fn constructor_rejects_invalid_forgetting_factor() {
    let error = HretObserver::new(
        2,
        1,
        vec![0, 0],
        1.0,
        vec![0.9],
        vec![1.0, 1.0],
        vec![1.0],
        vec![vec![1.0, 1.0]],
    )
    .expect_err("constructor should reject rho outside (0, 1)");

    assert!(error.to_string().contains("rho"));
}

#[test]
fn constructor_rejects_empty_gain_matrix() {
    let error = HretObserver::new(
        2,
        1,
        vec![0, 0],
        0.95,
        vec![0.9],
        vec![1.0, 1.0],
        vec![1.0],
        vec![],
    )
    .expect_err("constructor should reject empty gain matrix");

    assert!(error.to_string().contains("gain row"));
}

#[test]
fn constructor_rejects_non_finite_gains() {
    let error = HretObserver::new(
        2,
        1,
        vec![0, 0],
        0.95,
        vec![0.9],
        vec![1.0, 1.0],
        vec![1.0],
        vec![vec![1.0, f64::INFINITY]],
    )
    .expect_err("constructor should reject non-finite gains");

    assert!(error.to_string().contains("must be finite"));
}

#[test]
fn update_rejects_non_finite_residuals() {
    let mut obs = make_observer();
    let error = obs
        .update(vec![f64::NAN, 0.0])
        .expect_err("update should reject NaN residuals");

    assert!(error.to_string().contains("residuals"));
}

#[test]
fn update_uses_uniform_weights_when_trusts_underflow() {
    let mut obs = HretObserver::new(
        2,
        1,
        vec![0, 0],
        0.5,
        vec![0.5],
        vec![1e308, 1e308],
        vec![1e308],
        vec![vec![1.0, 1.0]],
    )
    .expect("constructor should succeed");

    let (_, weights, _, _) = obs
        .update(vec![1e308, 1e308])
        .expect("update should succeed with finite residuals");

    assert!((weights[0] - 0.5).abs() < 1e-12);
    assert!((weights[1] - 0.5).abs() < 1e-12);
    assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
}
