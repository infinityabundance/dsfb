use crate::sim::deterministic_signal::{
    first_differences, residuals_against_reference, second_differences, SignalTrace,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectorThresholds {
    pub residual: f64,
    pub difference: f64,
    pub curvature: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectorPoint {
    pub time_step: usize,
    pub signal_value: f64,
    pub residual_value: f64,
    pub first_difference: f64,
    pub second_difference: f64,
    pub residual_trigger: bool,
    pub difference_trigger: bool,
    pub curvature_trigger: bool,
}

pub fn evaluate_signal(
    signal: &SignalTrace,
    thresholds: DetectorThresholds,
    reference: impl Fn(f64) -> f64 + Copy,
) -> Vec<DetectorPoint> {
    let residuals = residuals_against_reference(&signal.values, reference);
    let first = first_differences(&signal.values);
    let second = second_differences(&signal.values);
    signal
        .values
        .iter()
        .enumerate()
        .map(|(time_step, signal_value)| {
            let residual_value = residuals[time_step];
            let first_difference = first[time_step];
            let second_difference = second[time_step];
            DetectorPoint {
                time_step,
                signal_value: *signal_value,
                residual_value,
                first_difference,
                second_difference,
                residual_trigger: residual_value.abs() > thresholds.residual,
                difference_trigger: first_difference.abs() > thresholds.difference,
                curvature_trigger: second_difference.abs() > thresholds.curvature,
            }
        })
        .collect()
}
