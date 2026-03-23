use std::time::{Duration, Instant};

use serde::Serialize;

use crate::config::DemoConfig;
use crate::cost::{build_cost_report, CostMode};
use crate::error::{Error, Result};
use crate::frame::{ImageFrame, ScalarField};
use crate::host::{
    default_host_realistic_profile, motion_augmented_profile, supervise_temporal_reuse,
    synthetic_visibility_profile, HostSupervisionProfile, HostTemporalInputs,
};
use crate::scaling::scaled_scene_config;
use crate::scene::{
    generate_sequence_for_definition, scenario_by_id, MotionVector, Normal3, ScenarioId,
    SceneFrame, SceneSequence, SurfaceTag,
};

#[derive(Clone, Debug, Serialize)]
pub struct TimingStageMetrics {
    pub stage: String,
    pub total_ms: f64,
    pub ms_per_frame: f64,
    pub ns_per_pixel: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TimingEntry {
    pub label: String,
    pub measurement_kind: String,
    pub actual_gpu_timing: bool,
    pub mode: String,
    pub scenario_id: String,
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    pub iterations: usize,
    pub build_profile: String,
    pub stages: Vec<TimingStageMetrics>,
    pub total_ms: f64,
    pub ms_per_frame: f64,
    pub estimated_ops_per_pixel: usize,
    pub estimated_reads_per_pixel: usize,
    pub estimated_writes_per_pixel: usize,
    pub estimated_memory_traffic_megabytes: f64,
    pub likely_optimization_levers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TimingMetrics {
    pub measurement_kind: String,
    pub actual_gpu_timing: bool,
    pub entries: Vec<TimingEntry>,
    pub notes: Vec<String>,
}

pub fn run_timing_study(config: &DemoConfig) -> Result<TimingMetrics> {
    let (high_width, high_height) = if cfg!(debug_assertions) {
        (1280usize, 720usize)
    } else {
        (1920usize, 1080usize)
    };
    let entries = vec![
        measure_entry(
            "minimum_host_path_default_res",
            CostMode::Minimal,
            scenario_sequence(config, ScenarioId::ThinReveal, 160, 96)?,
            &default_host_realistic_profile(
                config.dsfb_alpha_range.min,
                config.dsfb_alpha_range.max,
            ),
            6,
        )?,
        measure_entry(
            "motion_augmented_region_mid_res",
            CostMode::HostRealistic,
            scenario_sequence(config, ScenarioId::MotionBiasBand, 640, 360)?,
            &motion_augmented_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max),
            4,
        )?,
        measure_entry(
            "full_debug_region_mid_res",
            CostMode::FullResearchDebug,
            scenario_sequence(config, ScenarioId::RevealBand, 640, 360)?,
            &synthetic_visibility_profile(config.dsfb_alpha_range.min, config.dsfb_alpha_range.max),
            4,
        )?,
        measure_entry(
            "minimum_host_path_high_res_proxy",
            CostMode::HostRealistic,
            timing_high_resolution_sequence(
                config,
                ScenarioId::RevealBand,
                high_width,
                high_height,
            )?,
            &default_host_realistic_profile(
                config.dsfb_alpha_range.min,
                config.dsfb_alpha_range.max,
            ),
            1,
        )?,
    ];

    Ok(TimingMetrics {
        measurement_kind: "cpu_only_proxy".to_string(),
        actual_gpu_timing: false,
        entries,
        notes: vec![
            "No actual GPU timing was measured in this environment. These timings are CPU-side proxy measurements of the same per-pixel supervisory structure and are paired with analytical op and memory estimates.".to_string(),
            "The highest-resolution entry is a selected-scenario host-realistic proxy, not a full-suite production benchmark.".to_string(),
        ],
    })
}

fn timing_high_resolution_sequence(
    config: &DemoConfig,
    scenario_id: ScenarioId,
    width: usize,
    height: usize,
) -> Result<SceneSequence> {
    let mut sequence = scenario_sequence(config, scenario_id, width, height)?;
    if cfg!(debug_assertions) && sequence.frames.len() > 8 {
        sequence.frames.truncate(8);
        sequence.onset_frame = sequence
            .onset_frame
            .min(sequence.frames.len().saturating_sub(2));
    }
    Ok(sequence)
}

fn scenario_sequence(
    config: &DemoConfig,
    scenario_id: ScenarioId,
    width: usize,
    height: usize,
) -> Result<SceneSequence> {
    let scaled_scene = scaled_scene_config(&config.scene, width, height);
    let definition = scenario_by_id(&scaled_scene, scenario_id).ok_or_else(|| {
        Error::Message(format!(
            "timing scenario {} is unavailable",
            scenario_id.as_str()
        ))
    })?;
    Ok(generate_sequence_for_definition(&definition))
}

fn measure_entry(
    label: &str,
    cost_mode: CostMode,
    sequence: SceneSequence,
    profile: &HostSupervisionProfile,
    iterations: usize,
) -> Result<TimingEntry> {
    let measured = measure_cpu_proxy(&sequence, profile, iterations.max(1));
    let cost = build_cost_report(cost_mode);
    let pixels_per_frame = (sequence.config.width * sequence.config.height) as f64;
    let active_frames = sequence.frames.len().saturating_sub(1).max(1) as f64;
    let total_pixels = pixels_per_frame * active_frames * iterations.max(1) as f64;
    let estimated_memory_traffic_megabytes =
        (cost.estimated_total_reads_per_pixel + cost.estimated_total_writes_per_pixel) as f64
            * 4.0
            * total_pixels
            / (1024.0 * 1024.0);

    Ok(TimingEntry {
        label: label.to_string(),
        measurement_kind: "cpu_only_proxy".to_string(),
        actual_gpu_timing: false,
        mode: cost_mode.as_str().to_string(),
        scenario_id: sequence.scenario_id.as_str().to_string(),
        width: sequence.config.width,
        height: sequence.config.height,
        frame_count: sequence.frames.len(),
        iterations,
        build_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        stages: vec![
            stage_metrics(
                "reproject",
                measured.reproject,
                active_frames,
                pixels_per_frame,
                iterations,
            ),
            stage_metrics(
                "supervise",
                measured.supervise,
                active_frames,
                pixels_per_frame,
                iterations,
            ),
            stage_metrics(
                "resolve",
                measured.resolve,
                active_frames,
                pixels_per_frame,
                iterations,
            ),
        ],
        total_ms: measured.total.as_secs_f64() * 1000.0,
        ms_per_frame: measured.total.as_secs_f64() * 1000.0 / active_frames / iterations as f64,
        estimated_ops_per_pixel: cost.estimated_total_ops_per_pixel,
        estimated_reads_per_pixel: cost.estimated_total_reads_per_pixel,
        estimated_writes_per_pixel: cost.estimated_total_writes_per_pixel,
        estimated_memory_traffic_megabytes,
        likely_optimization_levers: optimization_levers(cost_mode),
    })
}

#[derive(Default)]
struct StageDurations {
    reproject: Duration,
    supervise: Duration,
    resolve: Duration,
    total: Duration,
}

fn measure_cpu_proxy(
    sequence: &SceneSequence,
    profile: &HostSupervisionProfile,
    iterations: usize,
) -> StageDurations {
    let mut totals = StageDurations::default();
    for _ in 0..iterations {
        let total_start = Instant::now();
        let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
        for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
            let width = scene_frame.ground_truth.width();
            let height = scene_frame.ground_truth.height();
            if frame_index == 0 {
                resolved_frames.push(scene_frame.ground_truth.clone());
                continue;
            }

            let previous_resolved = &resolved_frames[frame_index - 1];
            let previous_scene = &sequence.frames[frame_index - 1];

            let start = Instant::now();
            let reprojected = reproject_frame(previous_resolved, scene_frame);
            let reprojected_depth = reproject_depth(previous_scene, scene_frame);
            let reprojected_normals = reproject_normals(previous_scene, scene_frame);
            totals.reproject += start.elapsed();

            let visibility_hint = profile
                .use_visibility_hint
                .then_some(scene_frame.disocclusion_mask.as_slice());
            let thin_hint_field = profile
                .use_visibility_hint
                .then(|| compute_thin_hint(scene_frame));
            let thin_hint = thin_hint_field.as_ref();
            let inputs = HostTemporalInputs {
                current_color: &scene_frame.ground_truth,
                reprojected_history: &reprojected,
                motion_vectors: &scene_frame.motion,
                current_depth: &scene_frame.depth,
                reprojected_depth: &reprojected_depth,
                current_normals: &scene_frame.normals,
                reprojected_normals: &reprojected_normals,
                visibility_hint,
                thin_hint,
            };

            let start = Instant::now();
            let outputs = supervise_temporal_reuse(&inputs, profile);
            totals.supervise += start.elapsed();

            let start = Instant::now();
            let resolved =
                resolve_with_alpha(&reprojected, &scene_frame.ground_truth, &outputs.alpha);
            totals.resolve += start.elapsed();
            resolved_frames.push(resolved);

            let _ = (width, height);
        }
        totals.total += total_start.elapsed();
    }
    totals
}

fn stage_metrics(
    stage: &str,
    duration: Duration,
    active_frames: f64,
    pixels_per_frame: f64,
    iterations: usize,
) -> TimingStageMetrics {
    let total_ms = duration.as_secs_f64() * 1000.0;
    let pixel_count = active_frames * pixels_per_frame * iterations as f64;
    TimingStageMetrics {
        stage: stage.to_string(),
        total_ms,
        ms_per_frame: total_ms / active_frames.max(1.0) / iterations as f64,
        ns_per_pixel: duration.as_secs_f64() * 1.0e9 / pixel_count.max(1.0),
    }
}

fn optimization_levers(mode: CostMode) -> Vec<String> {
    match mode {
        CostMode::Minimal => vec![
            "Fuse alpha modulation into the temporal resolve.".to_string(),
            "Compute trust/intervention at half resolution if only gating is needed.".to_string(),
        ],
        CostMode::HostRealistic => vec![
            "Fuse reprojection fetches across color, depth, and normal buffers.".to_string(),
            "Evaluate trust at half resolution or per tile, then upsample alpha.".to_string(),
            "Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.".to_string(),
        ],
        CostMode::FullResearchDebug => vec![
            "Drop synthetic visibility and debug exports outside analysis mode.".to_string(),
            "Compress trust/alpha/intervention into narrower formats once calibration work stabilizes.".to_string(),
        ],
    }
}

fn resolve_with_alpha(
    history: &ImageFrame,
    current: &ImageFrame,
    alpha: &ScalarField,
) -> ImageFrame {
    let mut resolved = ImageFrame::new(history.width(), history.height());
    for y in 0..history.height() {
        for x in 0..history.width() {
            resolved.set(
                x,
                y,
                history.get(x, y).lerp(current.get(x, y), alpha.get(x, y)),
            );
        }
    }
    resolved
}

fn reproject_frame(previous_resolved: &ImageFrame, scene_frame: &SceneFrame) -> ImageFrame {
    let mut reprojected = ImageFrame::new(
        scene_frame.ground_truth.width(),
        scene_frame.ground_truth.height(),
    );
    for y in 0..scene_frame.ground_truth.height() {
        for x in 0..scene_frame.ground_truth.width() {
            let motion = scene_frame.motion[y * scene_frame.ground_truth.width() + x];
            reprojected.set(
                x,
                y,
                previous_resolved.sample_bilinear_clamped(
                    x as f32 + motion.to_prev_x,
                    y as f32 + motion.to_prev_y,
                ),
            );
        }
    }
    reprojected
}

fn reproject_depth(previous_scene_frame: &SceneFrame, scene_frame: &SceneFrame) -> Vec<f32> {
    reproject_scalar_buffer(
        &previous_scene_frame.depth,
        scene_frame.ground_truth.width(),
        scene_frame.ground_truth.height(),
        &scene_frame.motion,
    )
}

fn reproject_normals(previous_scene_frame: &SceneFrame, scene_frame: &SceneFrame) -> Vec<Normal3> {
    let width = scene_frame.ground_truth.width();
    let height = scene_frame.ground_truth.height();
    let mut reprojected = vec![Normal3::new(0.0, 0.0, 1.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let motion = scene_frame.motion[index];
            reprojected[index] = sample_normal_bilinear_clamped(
                &previous_scene_frame.normals,
                width,
                height,
                x as f32 + motion.to_prev_x,
                y as f32 + motion.to_prev_y,
            );
        }
    }
    reprojected
}

fn reproject_scalar_buffer(
    previous_values: &[f32],
    width: usize,
    height: usize,
    motion: &[MotionVector],
) -> Vec<f32> {
    let mut reprojected = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let vector = motion[index];
            reprojected[index] = sample_scalar_bilinear_clamped(
                previous_values,
                width,
                height,
                x as f32 + vector.to_prev_x,
                y as f32 + vector.to_prev_y,
            );
        }
    }
    reprojected
}

fn compute_thin_hint(scene_frame: &SceneFrame) -> ScalarField {
    let width = scene_frame.ground_truth.width();
    let height = scene_frame.ground_truth.height();
    let mut field = ScalarField::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let hint = matches!(scene_frame.layers[index], SurfaceTag::ThinStructure)
                || neighbors(x, y, width, height).into_iter().any(|(nx, ny)| {
                    matches!(
                        scene_frame.layers[ny * width + nx],
                        SurfaceTag::ThinStructure
                    )
                });
            field.set(x, y, if hint { 1.0 } else { 0.0 });
        }
    }
    field
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(8);
    for offset_y in -1..=1 {
        for offset_x in -1..=1 {
            if offset_x == 0 && offset_y == 0 {
                continue;
            }
            let nx = x as i32 + offset_x;
            let ny = y as i32 + offset_y;
            if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                result.push((nx as usize, ny as usize));
            }
        }
    }
    result
}

fn sample_scalar_bilinear_clamped(
    values: &[f32],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);

    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };

    let top = sample(x0, y0) * (1.0 - tx) + sample(x1, y0) * tx;
    let bottom = sample(x0, y1) * (1.0 - tx) + sample(x1, y1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn sample_normal_bilinear_clamped(
    values: &[Normal3],
    width: usize,
    height: usize,
    x: f32,
    y: f32,
) -> Normal3 {
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let tx = (x - x0).clamp(0.0, 1.0);
    let ty = (y - y0).clamp(0.0, 1.0);

    let sample = |sample_x: f32, sample_y: f32| {
        let sx = sample_x.clamp(0.0, width.saturating_sub(1) as f32) as usize;
        let sy = sample_y.clamp(0.0, height.saturating_sub(1) as f32) as usize;
        values[sy * width + sx]
    };

    let mix = |a: Normal3, b: Normal3, t: f32| {
        Normal3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    };
    mix(
        mix(sample(x0, y0), sample(x1, y0), tx),
        mix(sample(x0, y1), sample(x1, y1), tx),
        ty,
    )
    .normalized()
}
