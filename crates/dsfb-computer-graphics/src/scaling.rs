use serde::Serialize;

use crate::config::{DemoConfig, SceneConfig};
use crate::cost::{build_cost_report, CostMode};
use crate::dsfb::run_profiled_taa;
use crate::error::{Error, Result};
use crate::host::{default_host_realistic_profile, motion_augmented_profile};
use crate::metrics::{analyze_demo_a_suite, RunAnalysisInput, ScenarioReport};
use crate::scene::{
    generate_sequence_for_definition, scenario_by_id, ScenarioExpectation, ScenarioId,
    ScenarioSupportCategory, SceneSequence,
};
use crate::taa::{run_fixed_alpha_baseline, run_strong_heuristic_baseline};

const RESOLUTION_TIERS: &[(&str, usize, usize, bool)] = &[
    ("default_full_suite", 160, 96, false),
    ("intermediate_selected_suite", 640, 360, false),
    ("high_resolution_proxy_selected_suite", 960, 540, true),
];

const SELECTED_SCENARIOS: &[ScenarioId] = &[
    ScenarioId::ThinReveal,
    ScenarioId::RevealBand,
    ScenarioId::MotionBiasBand,
    ScenarioId::ContrastPulse,
];

#[derive(Clone, Debug, Serialize)]
pub struct ResolutionScenarioMetrics {
    pub tier_id: String,
    pub width: usize,
    pub height: usize,
    pub selected_high_resolution_mode: bool,
    pub scenario_id: String,
    pub scenario_title: String,
    pub expectation: ScenarioExpectation,
    pub support_category: ScenarioSupportCategory,
    pub target_pixels: usize,
    pub target_area_fraction: f32,
    pub fixed_alpha_cumulative_roi_mae: f32,
    pub strong_heuristic_cumulative_roi_mae: f32,
    pub host_realistic_cumulative_roi_mae: f32,
    pub motion_augmented_cumulative_roi_mae: f32,
    pub host_realistic_vs_fixed_alpha_gain: f32,
    pub motion_augmented_vs_host_realistic_gain: f32,
    pub host_realistic_non_roi_mae: f32,
    pub buffer_memory_megabytes: f32,
    pub roi_note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolutionScalingMetrics {
    pub entries: Vec<ResolutionScenarioMetrics>,
    pub notes: Vec<String>,
}

pub fn run_resolution_scaling_study(config: &DemoConfig) -> Result<ResolutionScalingMetrics> {
    let mut entries = Vec::new();
    let host_cost = build_cost_report(CostMode::HostRealistic);
    let bytes_per_pixel = host_cost
        .buffers
        .iter()
        .map(|buffer| buffer.bytes_per_pixel)
        .sum::<usize>();

    for (tier_id, width, height, selected_high_resolution_mode) in RESOLUTION_TIERS {
        let scaled_scene = scaled_scene_config(&config.scene, *width, *height);
        let tier_config = DemoConfig {
            scene: scaled_scene,
            ..config.clone()
        };
        for scenario_id in SELECTED_SCENARIOS {
            if *width > config.scene.width && matches!(scenario_id, ScenarioId::ThinReveal) {
                continue;
            }
            let definition = scenario_by_id(&tier_config.scene, *scenario_id).ok_or_else(|| {
                Error::Message(format!(
                    "resolution scaling scenario {} was unavailable",
                    scenario_id.as_str()
                ))
            })?;
            let sequence = generate_sequence_for_definition(&definition);
            let scenario_report = run_resolution_scenario(&tier_config, &sequence)?;
            let fixed = find_run(&scenario_report, "fixed_alpha")?;
            let strong = find_run(&scenario_report, "strong_heuristic")?;
            let host = find_run(&scenario_report, "dsfb_host_realistic")?;
            let motion = find_run(&scenario_report, "dsfb_motion_augmented")?;
            let total_pixels = width * height;
            entries.push(ResolutionScenarioMetrics {
                tier_id: (*tier_id).to_string(),
                width: *width,
                height: *height,
                selected_high_resolution_mode: *selected_high_resolution_mode,
                scenario_id: scenario_report.scenario_id.clone(),
                scenario_title: scenario_report.scenario_title.clone(),
                expectation: scenario_report.expectation,
                support_category: scenario_report.support_category,
                target_pixels: scenario_report.target_pixels,
                target_area_fraction: scenario_report.target_area_fraction,
                fixed_alpha_cumulative_roi_mae: fixed.summary.cumulative_roi_mae,
                strong_heuristic_cumulative_roi_mae: strong.summary.cumulative_roi_mae,
                host_realistic_cumulative_roi_mae: host.summary.cumulative_roi_mae,
                motion_augmented_cumulative_roi_mae: motion.summary.cumulative_roi_mae,
                host_realistic_vs_fixed_alpha_gain: scenario_report
                    .host_realistic_vs_fixed_alpha_cumulative_roi_gain,
                motion_augmented_vs_host_realistic_gain: host.summary.cumulative_roi_mae
                    - motion.summary.cumulative_roi_mae,
                host_realistic_non_roi_mae: host.summary.average_non_roi_mae,
                buffer_memory_megabytes: bytes_per_pixel as f32 * total_pixels as f32
                    / (1024.0 * 1024.0),
                roi_note: scenario_report.roi_note.clone(),
            });
        }
    }

    Ok(ResolutionScalingMetrics {
        entries,
        notes: vec![
            "The high-resolution tier is a selected-scenario scalable proxy rather than a full 1080p sweep. It is intended to demonstrate structural persistence beyond the toy default resolution without pretending to be a shipping-engine benchmark.".to_string(),
            "The canonical thin_reveal point-ROI case is intentionally kept at the default resolution only. At higher resolutions its exact one-pixel disocclusion geometry becomes path-dependent and is not a stable scaling metric.".to_string(),
            "Memory footprint numbers are analytical host-realistic buffer estimates from the crate cost model.".to_string(),
        ],
    })
}

pub fn scaled_scene_config(base: &SceneConfig, width: usize, height: usize) -> SceneConfig {
    let scale_x = width as f32 / base.width.max(1) as f32;
    let scale_y = height as f32 / base.height.max(1) as f32;
    let scale_len = scale_x.min(scale_y);
    let clamp_x = |value: i32| ((value as f32 * scale_x).round() as i32).clamp(0, width as i32);
    let clamp_y = |value: i32| ((value as f32 * scale_y).round() as i32).clamp(0, height as i32);

    SceneConfig {
        width,
        height,
        frame_count: base.frame_count,
        object_width: ((base.object_width as f32 * scale_len).round() as usize).max(8),
        object_height: ((base.object_height as f32 * scale_len).round() as usize).max(8),
        object_start_x: clamp_x(base.object_start_x),
        object_stop_x: clamp_x(base.object_stop_x),
        object_top_y: clamp_y(base.object_top_y),
        move_frames: base.move_frames,
        thin_vertical_x: clamp_x(base.thin_vertical_x),
    }
}

fn run_resolution_scenario(
    config: &DemoConfig,
    sequence: &SceneSequence,
) -> Result<ScenarioReport> {
    let fixed = run_fixed_alpha_baseline(sequence, config.baseline.fixed_alpha);
    let strong = run_strong_heuristic_baseline(sequence, &config.baseline);
    let host = run_profiled_taa(
        sequence,
        &default_host_realistic_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max),
    );
    let motion = run_profiled_taa(
        sequence,
        &motion_augmented_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max),
    );
    let host_alpha = host
        .supervision_frames
        .iter()
        .map(|frame| frame.alpha.clone())
        .collect::<Vec<_>>();
    let host_response = host
        .supervision_frames
        .iter()
        .map(|frame| frame.intervention.clone())
        .collect::<Vec<_>>();
    let host_trust = host
        .supervision_frames
        .iter()
        .map(|frame| frame.trust.clone())
        .collect::<Vec<_>>();
    let motion_alpha = motion
        .supervision_frames
        .iter()
        .map(|frame| frame.alpha.clone())
        .collect::<Vec<_>>();
    let motion_response = motion
        .supervision_frames
        .iter()
        .map(|frame| frame.intervention.clone())
        .collect::<Vec<_>>();
    let motion_trust = motion
        .supervision_frames
        .iter()
        .map(|frame| frame.trust.clone())
        .collect::<Vec<_>>();

    let analysis = analyze_demo_a_suite(&[(
        sequence.clone(),
        vec![
            RunAnalysisInput {
                id: &fixed.id,
                label: &fixed.label,
                category: "baseline",
                resolved_frames: &fixed.taa.resolved_frames,
                reprojected_history_frames: &fixed.taa.reprojected_history_frames,
                alpha_frames: &fixed.alpha_frames,
                response_frames: &fixed.response_frames,
                trust_frames: None,
            },
            RunAnalysisInput {
                id: &strong.id,
                label: &strong.label,
                category: "baseline",
                resolved_frames: &strong.taa.resolved_frames,
                reprojected_history_frames: &strong.taa.reprojected_history_frames,
                alpha_frames: &strong.alpha_frames,
                response_frames: &strong.response_frames,
                trust_frames: None,
            },
            RunAnalysisInput {
                id: &host.profile.id,
                label: &host.profile.label,
                category: "dsfb",
                resolved_frames: &host.resolved_frames,
                reprojected_history_frames: &host.reprojected_history_frames,
                alpha_frames: &host_alpha,
                response_frames: &host_response,
                trust_frames: Some(&host_trust),
            },
            RunAnalysisInput {
                id: &motion.profile.id,
                label: &motion.profile.label,
                category: "dsfb",
                resolved_frames: &motion.resolved_frames,
                reprojected_history_frames: &motion.reprojected_history_frames,
                alpha_frames: &motion_alpha,
                response_frames: &motion_response,
                trust_frames: Some(&motion_trust),
            },
        ],
    )])?;
    analysis.scenarios.into_iter().next().ok_or_else(|| {
        Error::Message("resolution scaling analysis produced no scenario".to_string())
    })
}

fn find_run<'a>(
    scenario: &'a ScenarioReport,
    run_id: &str,
) -> Result<&'a crate::metrics::ScenarioRunReport> {
    scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == run_id)
        .ok_or_else(|| Error::Message(format!("resolution scaling run {run_id} missing")))
}
