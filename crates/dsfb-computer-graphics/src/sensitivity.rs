use serde::Serialize;

use crate::config::DemoConfig;
use crate::dsfb::run_profiled_taa;
use crate::error::{Error, Result};
use crate::frame::{mean_abs_error_over_mask, ImageFrame};
use crate::host::{
    default_host_realistic_profile, motion_augmented_profile, HostSupervisionProfile,
};
use crate::scene::{
    generate_sequence_for_definition, scenario_by_id, ScenarioExpectation, ScenarioId,
    SceneSequence,
};
use crate::taa::run_fixed_alpha_baseline;

const SENSITIVITY_SCENARIOS: &[ScenarioId] = &[
    ScenarioId::ThinReveal,
    ScenarioId::RevealBand,
    ScenarioId::MotionBiasBand,
    ScenarioId::ContrastPulse,
];

#[derive(Clone, Debug, Serialize)]
pub struct ParameterSweepPoint {
    pub parameter_id: String,
    pub profile_mode: String,
    pub setting_label: String,
    pub numeric_value: f32,
    pub benefit_scenarios_beating_fixed: usize,
    pub benefit_scenarios_with_zero_ghost_frames: usize,
    pub canonical_cumulative_roi_mae: f32,
    pub region_mean_cumulative_roi_mae: f32,
    pub motion_bias_cumulative_roi_mae: f32,
    pub neutral_non_roi_mae: f32,
    pub robust_corridor_member: bool,
    pub robustness_class: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParameterSensitivityMetrics {
    pub baseline_mode: String,
    pub sweep_points: Vec<ParameterSweepPoint>,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy)]
struct ScenarioEval {
    expectation: ScenarioExpectation,
    cumulative_roi_mae: f32,
    average_non_roi_mae: f32,
    ghost_persistence_frames: usize,
    beat_fixed: bool,
}

pub fn run_parameter_sensitivity_study(config: &DemoConfig) -> Result<ParameterSensitivityMetrics> {
    let baseline_profile =
        default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
    let baseline_eval = evaluate_profile(config, &baseline_profile)?;

    let mut sweep_points = Vec::new();
    for factor in [0.5f32, 0.75, 1.0, 1.25, 1.5] {
        let mut profile = baseline_profile.clone();
        profile.parameters.weights.depth *= factor;
        sweep_points.push(build_sweep_point(
            config,
            "depth_weight",
            "host_realistic",
            factor,
            &profile,
            &baseline_eval,
        )?);

        let mut profile = baseline_profile.clone();
        profile.parameters.weights.thin *= factor;
        sweep_points.push(build_sweep_point(
            config,
            "thin_weight",
            "host_realistic",
            factor,
            &profile,
            &baseline_eval,
        )?);

        let mut profile = baseline_profile.clone();
        profile.parameters.weights.grammar *= factor;
        sweep_points.push(build_sweep_point(
            config,
            "grammar_weight",
            "host_realistic",
            factor,
            &profile,
            &baseline_eval,
        )?);

        let mut profile = baseline_profile.clone();
        profile.parameters.thresholds.residual.low *= factor;
        profile.parameters.thresholds.residual.high *= factor;
        sweep_points.push(build_sweep_point(
            config,
            "residual_threshold_scale",
            "host_realistic",
            factor,
            &profile,
            &baseline_eval,
        )?);

        let mut motion_profile =
            motion_augmented_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max);
        motion_profile.parameters.weights.motion *= factor;
        sweep_points.push(build_sweep_point(
            config,
            "motion_weight",
            "motion_augmented",
            factor,
            &motion_profile,
            &baseline_eval,
        )?);
    }

    for alpha_min in [0.04f32, config.dsfb_alpha_range.min, 0.12f32] {
        let mut profile = baseline_profile.clone();
        profile.parameters.alpha_range.min = alpha_min;
        sweep_points.push(build_sweep_point(
            config,
            "alpha_min",
            "host_realistic",
            alpha_min,
            &profile,
            &baseline_eval,
        )?);
    }

    for alpha_max in [0.84f32, config.dsfb_alpha_range.max, 0.99f32] {
        let mut profile = baseline_profile.clone();
        profile.parameters.alpha_range.max = alpha_max;
        sweep_points.push(build_sweep_point(
            config,
            "alpha_max",
            "host_realistic",
            alpha_max,
            &profile,
            &baseline_eval,
        )?);
    }

    Ok(ParameterSensitivityMetrics {
        baseline_mode: baseline_profile.label,
        sweep_points,
        notes: vec![
            "These sweeps are one-at-a-time sensitivity checks around the centralized hand-set parameterization. They are intended to show robustness corridors, not to overclaim a global optimum.".to_string(),
            "The motion-weight sweep uses the optional motion-augmented profile because the minimum host-realistic path no longer includes motion disagreement by default.".to_string(),
        ],
    })
}

fn build_sweep_point(
    config: &DemoConfig,
    parameter_id: &str,
    profile_mode: &str,
    numeric_value: f32,
    profile: &HostSupervisionProfile,
    baseline_eval: &[(&'static str, ScenarioEval)],
) -> Result<ParameterSweepPoint> {
    let current = evaluate_profile(config, profile)?;
    let baseline_motion = scenario_metric(baseline_eval, "motion_bias_band")?;
    let baseline_neutral = scenario_metric(baseline_eval, "contrast_pulse")?;
    let motion = scenario_metric(&current, "motion_bias_band")?;
    let neutral = scenario_metric(&current, "contrast_pulse")?;

    let benefit_scenarios_beating_fixed = current
        .iter()
        .filter(|(_, metric)| {
            matches!(metric.expectation, ScenarioExpectation::BenefitExpected) && metric.beat_fixed
        })
        .count();
    let benefit_scenarios_with_zero_ghost_frames = current
        .iter()
        .filter(|(_, metric)| {
            matches!(metric.expectation, ScenarioExpectation::BenefitExpected)
                && metric.ghost_persistence_frames == 0
        })
        .count();

    Ok(ParameterSweepPoint {
        parameter_id: parameter_id.to_string(),
        profile_mode: profile_mode.to_string(),
        setting_label: format!("{parameter_id}={numeric_value:.3}"),
        numeric_value,
        benefit_scenarios_beating_fixed,
        benefit_scenarios_with_zero_ghost_frames,
        canonical_cumulative_roi_mae: scenario_metric(&current, "thin_reveal")?.cumulative_roi_mae,
        region_mean_cumulative_roi_mae: mean_region_roi_mae(&current),
        motion_bias_cumulative_roi_mae: motion.cumulative_roi_mae,
        neutral_non_roi_mae: neutral.average_non_roi_mae,
        robust_corridor_member: benefit_scenarios_beating_fixed >= 2
            && motion.cumulative_roi_mae <= baseline_motion.cumulative_roi_mae * 1.20
            && neutral.average_non_roi_mae <= baseline_neutral.average_non_roi_mae * 1.25,
        robustness_class: classify_robustness(
            benefit_scenarios_beating_fixed,
            motion.cumulative_roi_mae,
            baseline_motion.cumulative_roi_mae,
            neutral.average_non_roi_mae,
            baseline_neutral.average_non_roi_mae,
        )
        .to_string(),
    })
}

fn evaluate_profile(
    config: &DemoConfig,
    profile: &HostSupervisionProfile,
) -> Result<Vec<(&'static str, ScenarioEval)>> {
    let mut results = Vec::new();
    for scenario_id in SENSITIVITY_SCENARIOS {
        let definition = scenario_by_id(&config.scene, *scenario_id).ok_or_else(|| {
            Error::Message(format!(
                "parameter sensitivity scenario {} unavailable",
                scenario_id.as_str()
            ))
        })?;
        let sequence = generate_sequence_for_definition(&definition);
        let fixed = run_fixed_alpha_baseline(&sequence, config.baseline.fixed_alpha);
        let profiled = run_profiled_taa(&sequence, profile);
        let fixed_metric = evaluate_run(&sequence, &fixed.taa.resolved_frames);
        let profiled_metric = evaluate_run(&sequence, &profiled.resolved_frames);
        results.push((
            scenario_id.as_str(),
            ScenarioEval {
                expectation: sequence.expectation,
                cumulative_roi_mae: profiled_metric.cumulative_roi_mae,
                average_non_roi_mae: profiled_metric.average_non_roi_mae,
                ghost_persistence_frames: profiled_metric.ghost_persistence_frames,
                beat_fixed: profiled_metric.cumulative_roi_mae + 1.0e-6
                    < fixed_metric.cumulative_roi_mae,
            },
        ));
    }
    Ok(results)
}

#[derive(Clone, Copy)]
struct RunEval {
    cumulative_roi_mae: f32,
    average_non_roi_mae: f32,
    ghost_persistence_frames: usize,
}

fn evaluate_run(sequence: &SceneSequence, resolved_frames: &[ImageFrame]) -> RunEval {
    let target_mask = &sequence.target_mask;
    let non_roi_mask = target_mask.iter().map(|value| !value).collect::<Vec<_>>();
    let onset = sequence
        .onset_frame
        .min(sequence.frames.len().saturating_sub(1));
    let threshold = persistence_threshold(sequence);
    let mut cumulative_roi_mae = 0.0;
    let mut average_non_roi_mae = 0.0;
    let mut ghost_persistence_frames = 0usize;

    for frame_index in 0..sequence.frames.len() {
        let gt = &sequence.frames[frame_index].ground_truth;
        let resolved = &resolved_frames[frame_index];
        let roi_mae = mean_abs_error_over_mask(resolved, gt, target_mask);
        let non_roi_mae = mean_abs_error_over_mask(resolved, gt, &non_roi_mask);
        cumulative_roi_mae += roi_mae;
        average_non_roi_mae += non_roi_mae;
        if frame_index >= onset && roi_mae > threshold {
            ghost_persistence_frames += 1;
        }
    }

    RunEval {
        cumulative_roi_mae,
        average_non_roi_mae: average_non_roi_mae / sequence.frames.len().max(1) as f32,
        ghost_persistence_frames,
    }
}

fn persistence_threshold(sequence: &SceneSequence) -> f32 {
    if sequence.onset_frame == 0 {
        return 0.02;
    }
    let previous = &sequence.frames[sequence.onset_frame - 1].ground_truth;
    let current = &sequence.frames[sequence.onset_frame].ground_truth;
    (mean_abs_error_over_mask(previous, current, &sequence.target_mask) * 0.15).max(0.02)
}

fn scenario_metric<'a>(
    values: &'a [(&'static str, ScenarioEval)],
    scenario_id: &str,
) -> Result<&'a ScenarioEval> {
    values
        .iter()
        .find(|(current, _)| *current == scenario_id)
        .map(|(_, metric)| metric)
        .ok_or_else(|| Error::Message(format!("sensitivity metric {scenario_id} missing")))
}

fn mean_region_roi_mae(values: &[(&'static str, ScenarioEval)]) -> f32 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for (scenario_id, metric) in values {
        if matches!(*scenario_id, "reveal_band" | "motion_bias_band") {
            sum += metric.cumulative_roi_mae;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

fn classify_robustness(
    benefit_scenarios_beating_fixed: usize,
    motion_roi_mae: f32,
    baseline_motion_roi_mae: f32,
    neutral_non_roi_mae: f32,
    baseline_neutral_non_roi_mae: f32,
) -> &'static str {
    if benefit_scenarios_beating_fixed >= 2
        && motion_roi_mae <= baseline_motion_roi_mae * 1.20
        && neutral_non_roi_mae <= baseline_neutral_non_roi_mae * 1.25
    {
        "robust"
    } else if benefit_scenarios_beating_fixed >= 2
        && motion_roi_mae <= baseline_motion_roi_mae * 1.35
        && neutral_non_roi_mae <= baseline_neutral_non_roi_mae * 1.40
    {
        "moderately_sensitive"
    } else {
        "fragile"
    }
}
