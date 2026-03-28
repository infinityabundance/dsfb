// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Riaan de Beer / Invariant Forge LLC
//
// Narrow FFI helper for conservative C integration.

use crate::detection::{
    build_dsfb_detection, build_threshold_detection, run_dsfb_pipeline, verify_theorem1,
};
use crate::types::{GrammarState, PipelineConfig};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DsfbBatteryConfig {
    pub healthy_window: usize,
    pub drift_window: usize,
    pub drift_persistence: usize,
    pub slew_persistence: usize,
    pub drift_threshold: f64,
    pub slew_threshold: f64,
    pub eol_fraction: f64,
    pub boundary_fraction: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DsfbBatterySummary {
    pub dsfb_alarm_cycle: usize,
    pub threshold_85pct_cycle: usize,
    pub eol_80pct_cycle: usize,
    pub first_boundary_cycle: usize,
    pub first_violation_cycle: usize,
    pub t_star: usize,
}

impl From<DsfbBatteryConfig> for PipelineConfig {
    fn from(value: DsfbBatteryConfig) -> Self {
        PipelineConfig {
            healthy_window: value.healthy_window,
            drift_window: value.drift_window,
            drift_persistence: value.drift_persistence,
            slew_persistence: value.slew_persistence,
            drift_threshold: value.drift_threshold,
            slew_threshold: value.slew_threshold,
            eol_fraction: value.eol_fraction,
            boundary_fraction: value.boundary_fraction,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dsfb_battery_default_config() -> DsfbBatteryConfig {
    let config = PipelineConfig::default();
    DsfbBatteryConfig {
        healthy_window: config.healthy_window,
        drift_window: config.drift_window,
        drift_persistence: config.drift_persistence,
        slew_persistence: config.slew_persistence,
        drift_threshold: config.drift_threshold,
        slew_threshold: config.slew_threshold,
        eol_fraction: config.eol_fraction,
        boundary_fraction: config.boundary_fraction,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dsfb_battery_evaluate_grammar_state(
    residual: f64,
    envelope_rho: f64,
    drift: f64,
    slew: f64,
    drift_counter: usize,
    slew_counter: usize,
    config: DsfbBatteryConfig,
) -> i32 {
    let config: PipelineConfig = config.into();
    let state = crate::detection::evaluate_grammar_state(
        residual,
        &crate::types::EnvelopeParams {
            mu: 0.0,
            sigma: envelope_rho / 3.0,
            rho: envelope_rho,
        },
        drift,
        slew,
        drift_counter,
        slew_counter,
        &config,
    );
    match state {
        GrammarState::Admissible => 0,
        GrammarState::Boundary => 1,
        GrammarState::Violation => 2,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dsfb_battery_run_capacity_summary(
    capacities_ptr: *const f64,
    len: usize,
    config: DsfbBatteryConfig,
    out_summary: *mut DsfbBatterySummary,
) -> i32 {
    if capacities_ptr.is_null() || out_summary.is_null() || len == 0 {
        return -1;
    }

    let capacities = unsafe { std::slice::from_raw_parts(capacities_ptr, len) };
    let config: PipelineConfig = config.into();
    let Ok((envelope, trajectory)) = run_dsfb_pipeline(capacities, &config) else {
        return -2;
    };
    let eol_capacity = config.eol_fraction * capacities[0];
    let dsfb_detection = build_dsfb_detection(&trajectory, capacities, eol_capacity);
    let threshold_detection = build_threshold_detection(capacities, 0.85, eol_capacity);
    let theorem1 = verify_theorem1(&envelope, &trajectory, &config);
    let summary = DsfbBatterySummary {
        dsfb_alarm_cycle: dsfb_detection.alarm_cycle.unwrap_or(0),
        threshold_85pct_cycle: threshold_detection.alarm_cycle.unwrap_or(0),
        eol_80pct_cycle: dsfb_detection.eol_cycle.unwrap_or(0),
        first_boundary_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Boundary)
            .map(|sample| sample.cycle)
            .unwrap_or(0),
        first_violation_cycle: trajectory
            .iter()
            .find(|sample| sample.grammar_state == GrammarState::Violation)
            .map(|sample| sample.cycle)
            .unwrap_or(0),
        t_star: theorem1.t_star,
    };
    unsafe {
        *out_summary = summary;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_state_eval_returns_boundary_code() {
        let config = dsfb_battery_default_config();
        let state = dsfb_battery_evaluate_grammar_state(0.05, 0.05, -0.003, -0.001, 12, 8, config);
        assert!(state == 1 || state == 2);
    }

    #[test]
    fn ffi_capacity_summary_runs() {
        let capacities = [2.0, 2.01, 1.99, 1.95, 1.90, 1.82, 1.74, 1.60];
        let mut summary = DsfbBatterySummary::default();
        let config = DsfbBatteryConfig {
            healthy_window: 3,
            drift_window: 1,
            drift_persistence: 1,
            slew_persistence: 1,
            drift_threshold: 0.002,
            slew_threshold: 0.001,
            eol_fraction: 0.80,
            boundary_fraction: 0.80,
        };
        let code = unsafe {
            dsfb_battery_run_capacity_summary(
                capacities.as_ptr(),
                capacities.len(),
                config,
                &mut summary as *mut DsfbBatterySummary,
            )
        };
        assert_eq!(code, 0);
        assert!(summary.t_star > 0);
    }
}
