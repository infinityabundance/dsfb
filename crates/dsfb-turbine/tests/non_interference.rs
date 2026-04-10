#![forbid(unsafe_code)]

//! API-level non-interference checks.
//!
//! These tests are intentionally narrow. They do not prove every semantic form
//! of non-interference. They verify two concrete properties that this crate
//! claims and that can be checked directly here:
//! 1. The public residual-processing entrypoints accept borrowed input data.
//! 2. The std-gated evaluation path leaves caller-owned inputs unchanged.

use dsfb_turbine::core::channels::ChannelId;
use dsfb_turbine::core::config::DsfbConfig;
use dsfb_turbine::core::residual::{
    compute_baseline, compute_drift, compute_residuals, compute_slew, sign_at, ResidualSign,
};
use dsfb_turbine::pipeline::engine_eval::{evaluate_engine, EngineEvalResult};

// Compile-time signature checks for the public observer-only entrypoints.
const _: fn(&[f64], &DsfbConfig) -> (f64, f64) = compute_baseline;
const _: fn(&[f64], f64, &mut [f64]) -> usize = compute_residuals;
const _: fn(&[f64], usize, &mut [f64]) -> usize = compute_drift;
const _: fn(&[f64], usize, &mut [f64]) -> usize = compute_slew;
const _: fn(&[f64], &[f64], &[f64], usize, u32) -> ResidualSign = sign_at;
const _: fn(u16, &[(ChannelId, Vec<f64>)], &DsfbConfig) -> EngineEvalResult = evaluate_engine;

#[test]
fn residual_pipeline_leaves_input_series_unchanged() {
    let config = DsfbConfig::default();
    let values = [10.0, 10.5, 10.75, 11.0, 11.25, 11.5];
    let original = values;

    let (mean, _) = compute_baseline(&values, &config);

    let mut residuals = [0.0; 6];
    let mut drift = [0.0; 6];
    let mut slew = [0.0; 6];

    compute_residuals(&values, mean, &mut residuals);
    compute_drift(&residuals, 2, &mut drift);
    compute_slew(&drift, 2, &mut slew);
    let _ = sign_at(&residuals, &drift, &slew, 5, 1);

    assert_eq!(values, original);
}

#[test]
fn engine_evaluation_observes_borrowed_channel_data_without_mutating_it() {
    let config = DsfbConfig::default();
    let channel_data = vec![
        (
            ChannelId::TempHpcOutlet,
            vec![100.0, 100.0, 100.1, 100.2, 100.4, 100.7, 101.1, 101.6],
        ),
        (
            ChannelId::PressureHpcOutlet,
            vec![80.0, 80.0, 79.9, 79.8, 79.7, 79.5, 79.3, 79.0],
        ),
    ];
    let original = channel_data.clone();

    let result = evaluate_engine(7, &channel_data, &config);

    assert_eq!(result.unit, 7);
    assert_eq!(channel_data, original);
}
