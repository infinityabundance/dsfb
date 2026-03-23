use crate::frame::{Color, ImageFrame, ScalarField};
use crate::scene::{Normal3, SceneSequence};

#[derive(Clone, Debug)]
pub struct TaaRun {
    pub resolved_frames: Vec<ImageFrame>,
    pub reprojected_history_frames: Vec<ImageFrame>,
}

#[derive(Clone, Debug)]
pub struct HeuristicRun {
    pub id: String,
    pub label: String,
    pub description: String,
    pub taa: TaaRun,
    pub alpha_frames: Vec<ScalarField>,
    pub response_frames: Vec<ScalarField>,
}

#[derive(Clone, Debug)]
pub struct ResidualThresholdRun {
    pub taa: TaaRun,
    pub alpha_frames: Vec<ScalarField>,
    pub trigger_frames: Vec<ScalarField>,
}

pub fn run_fixed_alpha(sequence: &SceneSequence, alpha: f32) -> TaaRun {
    run_fixed_alpha_baseline(sequence, alpha).taa
}

pub fn run_fixed_alpha_baseline(sequence: &SceneSequence, alpha: f32) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "fixed_alpha",
        "Fixed-alpha baseline",
        "Uniform temporal blend weight with no rejection or clamp logic.",
        move |context| (context.history, alpha, 0.0),
    )
}

pub fn run_residual_threshold(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
    threshold_low: f32,
    threshold_high: f32,
) -> ResidualThresholdRun {
    let heuristic = run_heuristic_baseline(
        sequence,
        "residual_threshold",
        "Residual-threshold baseline",
        "Per-pixel alpha increases when current vs history residual exceeds a threshold.",
        move |context| {
            let residual = context.current.abs_diff(context.history);
            let trigger = smoothstep(threshold_low, threshold_high, residual);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (context.history, alpha, trigger)
        },
    );

    ResidualThresholdRun {
        taa: heuristic.taa,
        alpha_frames: heuristic.alpha_frames,
        trigger_frames: heuristic.response_frames,
    }
}

pub fn run_residual_threshold_baseline(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
    threshold_low: f32,
    threshold_high: f32,
) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "residual_threshold",
        "Residual-threshold baseline",
        "Per-pixel alpha increases when current vs history residual exceeds a threshold.",
        move |context| {
            let residual = context.current.abs_diff(context.history);
            let trigger = smoothstep(threshold_low, threshold_high, residual);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (context.history, alpha, trigger)
        },
    )
}

pub fn run_neighborhood_clamp_baseline(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "neighborhood_clamp",
        "Neighborhood-clamped baseline",
        "History is clamped to the current 3x3 neighborhood before blending. Alpha rises with clamp distance.",
        move |context| {
            let clamped = clamp_to_current_neighborhood(context.scene_frame, context.history, context.x, context.y);
            let clamp_distance = clamped.abs_diff(context.history);
            let trigger = smoothstep(0.008, 0.10, clamp_distance);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (clamped, alpha, trigger)
        },
    )
}

pub fn run_depth_normal_rejection_baseline(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "depth_normal_reject",
        "Depth/normal rejection baseline",
        "Alpha rises with reprojected depth or normal disagreement.",
        move |context| {
            let depth_gate = smoothstep(
                0.01,
                0.08,
                (context.current_depth - context.reprojected_depth).abs(),
            );
            let normal_gate = smoothstep(
                0.01,
                0.16,
                1.0 - context
                    .current_normal
                    .dot(context.reprojected_normal)
                    .clamp(-1.0, 1.0),
            );
            let trigger = depth_gate.max(normal_gate);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (context.history, alpha, trigger)
        },
    )
}

pub fn run_reactive_mask_baseline(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "reactive_mask",
        "Reactive-mask-style baseline",
        "Residual, depth, and neighborhood disagreement combine into a reactive alpha increase.",
        move |context| {
            let residual_gate = smoothstep(0.015, 0.22, context.current.abs_diff(context.history));
            let depth_gate = smoothstep(
                0.01,
                0.08,
                (context.current_depth - context.reprojected_depth).abs(),
            );
            let neighborhood_gate = smoothstep(
                0.01,
                0.14,
                neighborhood_distance(context.scene_frame, context.history, context.x, context.y),
            );
            let trigger = residual_gate.max(depth_gate).max(neighborhood_gate);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (context.history, alpha, trigger)
        },
    )
}

pub fn run_strong_heuristic_baseline(
    sequence: &SceneSequence,
    alpha_low: f32,
    alpha_high: f32,
) -> HeuristicRun {
    run_heuristic_baseline(
        sequence,
        "strong_heuristic",
        "Strong heuristic baseline",
        "Neighborhood clamp plus combined residual/depth/normal/neighborhood trigger.",
        move |context| {
            let clamped = clamp_to_current_neighborhood(
                context.scene_frame,
                context.history,
                context.x,
                context.y,
            );
            let clamp_distance = clamped.abs_diff(context.history);
            let residual_gate = smoothstep(0.015, 0.22, context.current.abs_diff(clamped));
            let depth_gate = smoothstep(
                0.01,
                0.08,
                (context.current_depth - context.reprojected_depth).abs(),
            );
            let normal_gate = smoothstep(
                0.01,
                0.16,
                1.0 - context
                    .current_normal
                    .dot(context.reprojected_normal)
                    .clamp(-1.0, 1.0),
            );
            let neighborhood_gate = smoothstep(0.01, 0.14, clamp_distance);
            let trigger = residual_gate
                .max(depth_gate)
                .max(normal_gate)
                .max(neighborhood_gate);
            let alpha = alpha_low + (alpha_high - alpha_low) * trigger;
            (clamped, alpha, trigger)
        },
    )
}

#[derive(Clone, Copy)]
struct PixelContext<'a> {
    scene_frame: &'a crate::scene::SceneFrame,
    current: Color,
    history: Color,
    current_depth: f32,
    reprojected_depth: f32,
    current_normal: Normal3,
    reprojected_normal: Normal3,
    x: usize,
    y: usize,
}

fn run_heuristic_baseline(
    sequence: &SceneSequence,
    id: &str,
    label: &str,
    description: &str,
    mut policy: impl FnMut(PixelContext<'_>) -> (Color, f32, f32),
) -> HeuristicRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());
    let mut alpha_frames = Vec::with_capacity(sequence.frames.len());
    let mut response_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        let width = scene_frame.ground_truth.width();
        let height = scene_frame.ground_truth.height();
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            alpha_frames.push(fill_scalar(width, height, 0.0));
            response_frames.push(ScalarField::new(width, height));
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let previous_scene = &sequence.frames[frame_index - 1];
        let mut reprojected = ImageFrame::new(width, height);
        let mut resolved = ImageFrame::new(width, height);
        let mut alpha_frame = ScalarField::new(width, height);
        let mut response_frame = ScalarField::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let motion = scene_frame.motion[y * width + x];
                let prev_x = (x as i32 + motion.to_prev_x).clamp(0, width as i32 - 1) as usize;
                let prev_y = (y as i32 + motion.to_prev_y).clamp(0, height as i32 - 1) as usize;
                let history = previous_resolved.get(prev_x, prev_y);
                let current = scene_frame.ground_truth.get(x, y);
                let context = PixelContext {
                    scene_frame,
                    current,
                    history,
                    current_depth: scene_frame.depth[y * width + x],
                    reprojected_depth: previous_scene.depth[prev_y * width + prev_x],
                    current_normal: scene_frame.normals[y * width + x],
                    reprojected_normal: previous_scene.normals[prev_y * width + prev_x],
                    x,
                    y,
                };
                let (history_used, alpha, response) = policy(context);

                reprojected.set(x, y, history_used);
                resolved.set(x, y, history_used.lerp(current, alpha));
                alpha_frame.set(x, y, alpha);
                response_frame.set(x, y, response);
            }
        }

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
        alpha_frames.push(alpha_frame);
        response_frames.push(response_frame);
    }

    HeuristicRun {
        id: id.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        taa: TaaRun {
            resolved_frames,
            reprojected_history_frames,
        },
        alpha_frames,
        response_frames,
    }
}

fn clamp_to_current_neighborhood(
    scene_frame: &crate::scene::SceneFrame,
    history: Color,
    x: usize,
    y: usize,
) -> Color {
    let mut min_r = f32::INFINITY;
    let mut min_g = f32::INFINITY;
    let mut min_b = f32::INFINITY;
    let mut max_r = f32::NEG_INFINITY;
    let mut max_g = f32::NEG_INFINITY;
    let mut max_b = f32::NEG_INFINITY;

    for (nx, ny) in neighbors(
        x,
        y,
        scene_frame.ground_truth.width(),
        scene_frame.ground_truth.height(),
    ) {
        let color = scene_frame.ground_truth.get(nx, ny);
        min_r = min_r.min(color.r);
        min_g = min_g.min(color.g);
        min_b = min_b.min(color.b);
        max_r = max_r.max(color.r);
        max_g = max_g.max(color.g);
        max_b = max_b.max(color.b);
    }
    let current = scene_frame.ground_truth.get(x, y);
    min_r = min_r.min(current.r);
    min_g = min_g.min(current.g);
    min_b = min_b.min(current.b);
    max_r = max_r.max(current.r);
    max_g = max_g.max(current.g);
    max_b = max_b.max(current.b);

    Color::rgb(
        history.r.clamp(min_r, max_r),
        history.g.clamp(min_g, max_g),
        history.b.clamp(min_b, max_b),
    )
}

fn neighborhood_distance(
    scene_frame: &crate::scene::SceneFrame,
    history: Color,
    x: usize,
    y: usize,
) -> f32 {
    clamp_to_current_neighborhood(scene_frame, history, x, y).abs_diff(history)
}

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
