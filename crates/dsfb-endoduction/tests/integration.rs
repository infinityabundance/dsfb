//! Integration tests for dsfb-endoduction.

use dsfb_endoduction::admissibility;
use dsfb_endoduction::baseline;
use dsfb_endoduction::baselines;
use dsfb_endoduction::grammar;
use dsfb_endoduction::residual;
use dsfb_endoduction::trust;

/// Synthetic test signal: constant value.
fn constant_signal(val: f64, n: usize) -> Vec<f64> {
    vec![val; n]
}

/// Synthetic test signal: linear ramp.
fn ramp_signal(n: usize) -> Vec<f64> {
    (0..n).map(|i| i as f64 / n as f64).collect()
}

/// Synthetic deterministic oscillation.
fn sine_signal(n: usize, freq: f64) -> Vec<f64> {
    (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / n as f64).sin())
        .collect()
}

// ----- Residual tests -----

#[test]
fn residual_zero_for_identical_signal() {
    let sig = sine_signal(1024, 5.0);
    let windows: Vec<&[f64]> = vec![sig.as_slice(); 10];
    let bl = baseline::estimate_baseline(&windows);
    let resid = residual::compute_residual(&sig, &bl);
    let max_resid = resid.iter().map(|r| r.abs()).fold(0.0_f64, f64::max);
    assert!(
        max_resid < 1e-10,
        "Residual should be zero for identical signal, got max {max_resid}"
    );
}

#[test]
fn residual_nonzero_for_different_signal() {
    let sig = sine_signal(1024, 5.0);
    let windows: Vec<&[f64]> = vec![sig.as_slice(); 10];
    let bl = baseline::estimate_baseline(&windows);
    let different = sine_signal(1024, 10.0);
    let resid = residual::compute_residual(&different, &bl);
    let max_resid = resid.iter().map(|r| r.abs()).fold(0.0_f64, f64::max);
    assert!(max_resid > 0.01, "Residual should be nonzero for different signal");
}

// ----- Admissibility tests -----

#[test]
fn envelope_breach_zero_for_nominal() {
    let sig = constant_signal(1.0, 1024);
    let windows: Vec<&[f64]> = vec![sig.as_slice(); 20];
    let bl = baseline::estimate_baseline(&windows);
    let env = admissibility::estimate_envelope(&bl, 0.99);
    let resid = residual::compute_residual(&sig, &bl);
    let breach = admissibility::breach_fraction(&resid, &env);
    assert!(
        breach < 1e-10,
        "Breach should be zero for constant signal, got {breach}"
    );
}

#[test]
fn envelope_breach_high_for_outlier() {
    let sig = constant_signal(1.0, 1024);
    let windows: Vec<&[f64]> = vec![sig.as_slice(); 20];
    let bl = baseline::estimate_baseline(&windows);
    let env = admissibility::estimate_envelope(&bl, 0.99);
    // Signal far from baseline
    let outlier = constant_signal(100.0, 1024);
    let resid = residual::compute_residual(&outlier, &bl);
    let breach = admissibility::breach_fraction(&resid, &env);
    assert!(breach > 0.5, "Breach should be high for outlier, got {breach}");
}

// ----- Grammar tests -----

#[test]
fn drift_positive_for_ramp() {
    let ramp = ramp_signal(1024);
    let d = grammar::drift(&ramp);
    assert!(d > 0.0, "Drift should be positive for ramp, got {d}");
}

#[test]
fn drift_near_zero_for_constant() {
    let c = constant_signal(5.0, 1024);
    let d = grammar::drift(&c);
    assert!(d.abs() < 1e-10, "Drift should be zero for constant, got {d}");
}

#[test]
fn persistence_high_for_all_positive() {
    let pos = constant_signal(1.0, 1024);
    let p = grammar::persistence(&pos);
    assert!(
        (p - 1.0).abs() < 1e-10,
        "Persistence should be 1.0 for all-positive, got {p}"
    );
}

#[test]
fn slew_detects_jump() {
    let mut sig = constant_signal(0.0, 100);
    sig[50] = 100.0;
    let s = grammar::slew(&sig);
    assert!(s >= 100.0, "Slew should detect jump, got {s}");
}

#[test]
fn variance_growth_one_for_baseline() {
    let sig = sine_signal(1024, 5.0);
    let var = baseline::variance(&sig);
    let vg = grammar::variance_growth(&sig, var);
    assert!(
        (vg - 1.0).abs() < 1e-10,
        "Variance growth should be 1.0, got {vg}"
    );
}

// ----- Trust score tests -----

#[test]
fn trust_score_bounded_comprehensive() {
    // Extreme inputs should still produce scores in [0, 1].
    let extremes = vec![
        trust::TrustInputs {
            breach_fraction: 1.0,
            persistence: 1.0,
            autocorr_growth: 10.0,
            spectral_shift: 1.0,
            variance_growth: 100.0,
            drift_magnitude: 1.0,
            baseline_drift_scale: 1e-5,
            baseline_spectral_scale: 0.01,
        },
        trust::TrustInputs {
            breach_fraction: 0.0,
            persistence: 0.0,
            autocorr_growth: -10.0,
            spectral_shift: -1.0,
            variance_growth: 0.0,
            drift_magnitude: 0.0,
            baseline_drift_scale: 1e-5,
            baseline_spectral_scale: 0.01,
        },
    ];
    for inputs in &extremes {
        let s = trust::compute_trust_score(inputs);
        assert!(s >= 0.0 && s <= 1.0, "Trust score out of bounds: {s}");
    }
}

// ----- Baselines tests -----

#[test]
fn first_sustained_detection_basic() {
    let flags = vec![false, false, true, true, true, false, true];
    assert_eq!(baselines::first_sustained_detection(&flags, 3), Some(2));
    assert_eq!(baselines::first_sustained_detection(&flags, 4), None);
}

#[test]
fn first_sustained_detection_empty() {
    let flags: Vec<bool> = vec![];
    assert_eq!(baselines::first_sustained_detection(&flags, 1), None);
}

// ----- Baseline math tests -----

#[test]
fn rms_correct() {
    let v = vec![1.0, -1.0, 1.0, -1.0];
    assert!((baseline::rms(&v) - 1.0).abs() < 1e-10);
}

#[test]
fn kurtosis_normal_approx() {
    // For a uniform-like distribution, kurtosis should be near -1.2.
    let v: Vec<f64> = (0..10000).map(|i| (i as f64 / 10000.0) * 2.0 - 1.0).collect();
    let k = baseline::kurtosis(&v);
    assert!(
        (k - (-1.2)).abs() < 0.1,
        "Kurtosis of uniform should be near -1.2, got {k}"
    );
}

#[test]
fn crest_factor_sine() {
    let sig = sine_signal(10000, 5.0);
    let cf = baseline::crest_factor(&sig);
    // For a pure sine wave, crest factor = sqrt(2) ≈ 1.414
    assert!(
        (cf - std::f64::consts::SQRT_2).abs() < 0.05,
        "Crest factor of sine should be ~1.414, got {cf}"
    );
}
