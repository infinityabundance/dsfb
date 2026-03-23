use serde::Serialize;

use crate::frame::{ImageFrame, ScalarField};
use crate::host::{
    default_host_realistic_profile, profile_residual_only, profile_without_alpha_modulation,
    profile_without_grammar, profile_without_motion, profile_without_thin,
    profile_without_visibility, supervise_temporal_reuse, synthetic_visibility_profile,
    HostSupervisionProfile, HostTemporalInputs,
};
use crate::scene::{MotionVector, Normal3, SceneFrame, SceneSequence};

#[derive(Clone, Debug)]
pub struct ProxyFields {
    pub residual_proxy: ScalarField,
    pub visibility_proxy: ScalarField,
    pub depth_proxy: ScalarField,
    pub normal_proxy: ScalarField,
    pub motion_proxy: ScalarField,
    pub neighborhood_proxy: ScalarField,
    pub thin_proxy: ScalarField,
    pub history_instability_proxy: ScalarField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum StructuralState {
    Nominal,
    DisocclusionLike,
    UnstableHistory,
    MotionEdge,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct StateCounts {
    pub nominal: usize,
    pub disocclusion_like: usize,
    pub unstable_history: usize,
    pub motion_edge: usize,
}

#[derive(Clone, Debug)]
pub struct StateField {
    width: usize,
    values: Vec<StructuralState>,
}

impl StateField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            values: vec![StructuralState::Nominal; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.values.len() / self.width.max(1)
    }

    pub fn set(&mut self, x: usize, y: usize, value: StructuralState) {
        self.values[y * self.width + x] = value;
    }

    pub fn values(&self) -> &[StructuralState] {
        &self.values
    }

    pub fn counts(&self) -> StateCounts {
        let mut counts = StateCounts::default();
        for state in &self.values {
            match state {
                StructuralState::Nominal => counts.nominal += 1,
                StructuralState::DisocclusionLike => counts.disocclusion_like += 1,
                StructuralState::UnstableHistory => counts.unstable_history += 1,
                StructuralState::MotionEdge => counts.motion_edge += 1,
            }
        }
        counts
    }

    pub fn counts_over_mask(&self, mask: &[bool]) -> StateCounts {
        let mut counts = StateCounts::default();
        for (state, include) in self.values.iter().zip(mask.iter().copied()) {
            if !include {
                continue;
            }
            match state {
                StructuralState::Nominal => counts.nominal += 1,
                StructuralState::DisocclusionLike => counts.disocclusion_like += 1,
                StructuralState::UnstableHistory => counts.unstable_history += 1,
                StructuralState::MotionEdge => counts.motion_edge += 1,
            }
        }
        counts
    }
}

#[derive(Clone, Debug)]
pub struct SupervisionFrame {
    pub residual: ScalarField,
    pub trust: ScalarField,
    pub alpha: ScalarField,
    pub intervention: ScalarField,
    pub proxies: ProxyFields,
    pub state: StateField,
}

#[derive(Clone, Debug)]
pub struct DsfbRun {
    pub profile: HostSupervisionProfile,
    pub resolved_frames: Vec<ImageFrame>,
    pub reprojected_history_frames: Vec<ImageFrame>,
    pub supervision_frames: Vec<SupervisionFrame>,
}

pub fn run_gated_taa(sequence: &SceneSequence, alpha_min: f32, alpha_max: f32) -> DsfbRun {
    run_profiled_taa(
        sequence,
        &default_host_realistic_profile(alpha_min, alpha_max),
    )
}

pub fn run_visibility_assisted_taa(
    sequence: &SceneSequence,
    alpha_min: f32,
    alpha_max: f32,
) -> DsfbRun {
    run_profiled_taa(
        sequence,
        &synthetic_visibility_profile(alpha_min, alpha_max),
    )
}

pub fn ablation_profiles(alpha_min: f32, alpha_max: f32) -> Vec<HostSupervisionProfile> {
    vec![
        synthetic_visibility_profile(alpha_min, alpha_max),
        default_host_realistic_profile(alpha_min, alpha_max),
        profile_without_visibility(alpha_min, alpha_max),
        profile_without_thin(alpha_min, alpha_max),
        profile_without_motion(alpha_min, alpha_max),
        profile_without_grammar(alpha_min, alpha_max),
        profile_residual_only(alpha_min, alpha_max),
        profile_without_alpha_modulation(alpha_min, alpha_max),
    ]
}

pub fn run_profiled_taa(sequence: &SceneSequence, profile: &HostSupervisionProfile) -> DsfbRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());
    let mut supervision_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        let width = scene_frame.ground_truth.width();
        let height = scene_frame.ground_truth.height();
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            supervision_frames.push(empty_supervision(width, height, 1.0, profile.alpha_min));
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let previous_scene_frame = &sequence.frames[frame_index - 1];
        let reprojected = reproject_frame(previous_resolved, scene_frame);
        let reprojected_depth = reproject_depth(previous_scene_frame, scene_frame);
        let reprojected_normals = reproject_normals(previous_scene_frame, scene_frame);
        let visibility_hint = profile
            .use_visibility_hint
            .then_some(scene_frame.disocclusion_mask.as_slice());
        let thin_hint_field = profile
            .use_visibility_hint
            .then(|| compute_thin_hint(scene_frame));
        let thin_hint = thin_hint_field.as_ref();

        let host_inputs = HostTemporalInputs {
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
        let outputs = supervise_temporal_reuse(&host_inputs, profile);
        let resolved = resolve_with_alpha(&reprojected, &scene_frame.ground_truth, &outputs.alpha);

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
        supervision_frames.push(SupervisionFrame {
            residual: outputs.residual,
            trust: outputs.trust,
            alpha: outputs.alpha,
            intervention: outputs.intervention,
            proxies: ProxyFields {
                residual_proxy: outputs.proxies.residual_proxy,
                visibility_proxy: outputs.proxies.visibility_proxy,
                depth_proxy: outputs.proxies.depth_proxy,
                normal_proxy: outputs.proxies.normal_proxy,
                motion_proxy: outputs.proxies.motion_proxy,
                neighborhood_proxy: outputs.proxies.neighborhood_proxy,
                thin_proxy: outputs.proxies.thin_proxy,
                history_instability_proxy: outputs.proxies.history_instability_proxy,
            },
            state: outputs.state,
        });
    }

    DsfbRun {
        profile: profile.clone(),
        resolved_frames,
        reprojected_history_frames,
        supervision_frames,
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
                previous_resolved
                    .sample_clamped(x as i32 + motion.to_prev_x, y as i32 + motion.to_prev_y),
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
            let prev_x = (x as i32 + motion.to_prev_x).clamp(0, width as i32 - 1) as usize;
            let prev_y = (y as i32 + motion.to_prev_y).clamp(0, height as i32 - 1) as usize;
            reprojected[index] = previous_scene_frame.normals[prev_y * width + prev_x];
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
            let prev_x = (x as i32 + vector.to_prev_x).clamp(0, width as i32 - 1) as usize;
            let prev_y = (y as i32 + vector.to_prev_y).clamp(0, height as i32 - 1) as usize;
            reprojected[index] = previous_values[prev_y * width + prev_x];
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
            let hint = matches!(
                scene_frame.layers[index],
                crate::scene::SurfaceTag::ThinStructure
            ) || neighbors(x, y, width, height).into_iter().any(|(nx, ny)| {
                matches!(
                    scene_frame.layers[ny * width + nx],
                    crate::scene::SurfaceTag::ThinStructure
                )
            });
            field.set(x, y, if hint { 1.0 } else { 0.0 });
        }
    }
    field
}

fn empty_supervision(
    width: usize,
    height: usize,
    trust_value: f32,
    alpha_value: f32,
) -> SupervisionFrame {
    let mut trust = ScalarField::new(width, height);
    let mut alpha = ScalarField::new(width, height);
    let mut intervention = ScalarField::new(width, height);
    let mut state = StateField::new(width, height);
    for y in 0..height {
        for x in 0..width {
            trust.set(x, y, trust_value);
            alpha.set(x, y, alpha_value);
            intervention.set(x, y, 1.0 - trust_value);
            state.set(x, y, StructuralState::Nominal);
        }
    }
    SupervisionFrame {
        residual: ScalarField::new(width, height),
        trust,
        alpha,
        intervention,
        proxies: ProxyFields {
            residual_proxy: ScalarField::new(width, height),
            visibility_proxy: ScalarField::new(width, height),
            depth_proxy: ScalarField::new(width, height),
            normal_proxy: ScalarField::new(width, height),
            motion_proxy: ScalarField::new(width, height),
            neighborhood_proxy: ScalarField::new(width, height),
            thin_proxy: ScalarField::new(width, height),
            history_instability_proxy: ScalarField::new(width, height),
        },
        state,
    }
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut values = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                values.push((nx as usize, ny as usize));
            }
        }
    }
    values
}
