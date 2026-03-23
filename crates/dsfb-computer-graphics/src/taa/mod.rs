use crate::frame::{ImageFrame, ScalarField};
use crate::scene::SceneSequence;

#[derive(Clone, Debug)]
pub struct TaaRun {
    pub resolved_frames: Vec<ImageFrame>,
    pub reprojected_history_frames: Vec<ImageFrame>,
}

#[derive(Clone, Debug)]
pub struct ResidualThresholdRun {
    pub taa: TaaRun,
    pub alpha_frames: Vec<ScalarField>,
    pub trigger_frames: Vec<ScalarField>,
}

pub fn run_fixed_alpha(sequence: &SceneSequence, alpha: f32) -> TaaRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let mut reprojected = ImageFrame::new(
            scene_frame.ground_truth.width(),
            scene_frame.ground_truth.height(),
        );
        let mut resolved = ImageFrame::new(
            scene_frame.ground_truth.width(),
            scene_frame.ground_truth.height(),
        );

        for y in 0..scene_frame.ground_truth.height() {
            for x in 0..scene_frame.ground_truth.width() {
                let history = reproject_history(previous_resolved, scene_frame, x, y);
                let current = scene_frame.ground_truth.get(x, y);
                reprojected.set(x, y, history);
                resolved.set(x, y, history.lerp(current, alpha));
            }
        }

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
    }

    TaaRun {
        resolved_frames,
        reprojected_history_frames,
    }
}

pub fn run_residual_threshold(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
    threshold_low: f32,
    threshold_high: f32,
) -> ResidualThresholdRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());
    let mut alpha_frames = Vec::with_capacity(sequence.frames.len());
    let mut trigger_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        let width = scene_frame.ground_truth.width();
        let height = scene_frame.ground_truth.height();
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            alpha_frames.push(fill_scalar(width, height, alpha_low));
            trigger_frames.push(ScalarField::new(width, height));
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let mut reprojected = ImageFrame::new(width, height);
        let mut resolved = ImageFrame::new(width, height);
        let mut alpha_frame = ScalarField::new(width, height);
        let mut trigger_frame = ScalarField::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let history = reproject_history(previous_resolved, scene_frame, x, y);
                let current = scene_frame.ground_truth.get(x, y);
                let residual = (current.luma() - history.luma()).abs();
                let trigger = smoothstep(threshold_low, threshold_high, residual);
                let alpha = alpha_low + (alpha_high - alpha_low) * trigger;

                reprojected.set(x, y, history);
                alpha_frame.set(x, y, alpha);
                trigger_frame.set(x, y, trigger);
                resolved.set(x, y, history.lerp(current, alpha));
            }
        }

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
        alpha_frames.push(alpha_frame);
        trigger_frames.push(trigger_frame);
    }

    ResidualThresholdRun {
        taa: TaaRun {
            resolved_frames,
            reprojected_history_frames,
        },
        alpha_frames,
        trigger_frames,
    }
}

fn reproject_history(
    previous_resolved: &ImageFrame,
    scene_frame: &crate::scene::SceneFrame,
    x: usize,
    y: usize,
) -> ColorProxy {
    let motion = scene_frame.motion[y * scene_frame.ground_truth.width() + x];
    previous_resolved.sample_clamped(x as i32 + motion.to_prev_x, y as i32 + motion.to_prev_y)
}

type ColorProxy = crate::frame::Color;

fn fill_scalar(width: usize, height: usize, value: f32) -> ScalarField {
    let mut field = ScalarField::new(width, height);
    for y in 0..height {
        for x in 0..width {
            field.set(x, y, value);
        }
    }
    field
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
