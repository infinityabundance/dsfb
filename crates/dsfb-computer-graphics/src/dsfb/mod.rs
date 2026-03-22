use crate::frame::{ImageFrame, ScalarField};
use crate::scene::{SceneFrame, SceneSequence, SurfaceTag};

#[derive(Clone, Debug)]
pub struct ProxyFields {
    pub residual_proxy: ScalarField,
    pub visibility_proxy: ScalarField,
    pub motion_edge_proxy: ScalarField,
    pub thin_proxy: ScalarField,
}

#[derive(Clone, Debug)]
pub struct SupervisionFrame {
    pub residual: ScalarField,
    pub trust: ScalarField,
    pub alpha: ScalarField,
    pub proxies: ProxyFields,
}

#[derive(Clone, Debug)]
pub struct DsfbRun {
    pub resolved_frames: Vec<ImageFrame>,
    pub reprojected_history_frames: Vec<ImageFrame>,
    pub supervision_frames: Vec<SupervisionFrame>,
}

pub fn run_gated_taa(sequence: &SceneSequence, alpha_min: f32, alpha_max: f32) -> DsfbRun {
    let mut resolved_frames = Vec::with_capacity(sequence.frames.len());
    let mut reprojected_history_frames = Vec::with_capacity(sequence.frames.len());
    let mut supervision_frames = Vec::with_capacity(sequence.frames.len());

    for (frame_index, scene_frame) in sequence.frames.iter().enumerate() {
        let width = scene_frame.ground_truth.width();
        let height = scene_frame.ground_truth.height();
        if frame_index == 0 {
            resolved_frames.push(scene_frame.ground_truth.clone());
            reprojected_history_frames.push(scene_frame.ground_truth.clone());
            supervision_frames.push(empty_supervision(width, height, 1.0, alpha_min));
            continue;
        }

        let previous_resolved = &resolved_frames[frame_index - 1];
        let previous_scene_frame = &sequence.frames[frame_index - 1];
        let motion_edge_proxy = compute_motion_edge_proxy(scene_frame);
        let thin_proxy = compute_thin_proxy(scene_frame);

        let mut reprojected = ImageFrame::new(width, height);
        let mut resolved = ImageFrame::new(width, height);
        let mut residual = ScalarField::new(width, height);
        let mut residual_proxy = ScalarField::new(width, height);
        let mut visibility_proxy = ScalarField::new(width, height);
        let mut trust = ScalarField::new(width, height);
        let mut alpha = ScalarField::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                let motion = scene_frame.motion[index];
                let prev_x = (x as i32 + motion.to_prev_x).clamp(0, width as i32 - 1) as usize;
                let prev_y = (y as i32 + motion.to_prev_y).clamp(0, height as i32 - 1) as usize;

                let history = previous_resolved.get(prev_x, prev_y);
                let current = scene_frame.ground_truth.get(x, y);
                let previous_layer = previous_scene_frame.layers[prev_y * width + prev_x];
                let current_layer = scene_frame.layers[index];
                let residual_value = (current.luma() - history.luma()).abs();
                let residual_gate = smoothstep(0.02, 0.28, residual_value);
                let visibility_gate = if previous_layer != current_layer {
                    1.0
                } else {
                    0.0
                };
                let structural_gate =
                    0.35 * motion_edge_proxy.get(x, y) + 0.25 * thin_proxy.get(x, y);
                let hazard = (0.55 * residual_gate
                    + 0.15 * structural_gate
                    + 0.10 * residual_gate * structural_gate)
                    .max(0.88 * visibility_gate)
                    .clamp(0.0, 1.0);
                let trust_value = 1.0 - hazard;
                let alpha_value = alpha_min + (alpha_max - alpha_min) * (1.0 - trust_value);

                reprojected.set(x, y, history);
                residual.set(x, y, residual_value);
                residual_proxy.set(x, y, residual_gate);
                visibility_proxy.set(x, y, visibility_gate);
                trust.set(x, y, trust_value);
                alpha.set(x, y, alpha_value);
                resolved.set(x, y, history.lerp(current, alpha_value));
            }
        }

        reprojected_history_frames.push(reprojected);
        resolved_frames.push(resolved);
        supervision_frames.push(SupervisionFrame {
            residual,
            trust,
            alpha,
            proxies: ProxyFields {
                residual_proxy,
                visibility_proxy,
                motion_edge_proxy,
                thin_proxy,
            },
        });
    }

    DsfbRun {
        resolved_frames,
        reprojected_history_frames,
        supervision_frames,
    }
}

fn compute_motion_edge_proxy(scene_frame: &SceneFrame) -> ScalarField {
    let width = scene_frame.ground_truth.width();
    let height = scene_frame.ground_truth.height();
    let mut field = ScalarField::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let base_layer = scene_frame.layers[index];
            let base_motion = scene_frame.motion[index];
            let mut score = 0.0f32;
            for (nx, ny) in neighbors(x, y, width, height) {
                let neighbor_index = ny * width + nx;
                let neighbor_layer = scene_frame.layers[neighbor_index];
                let neighbor_motion = scene_frame.motion[neighbor_index];
                if neighbor_layer != base_layer {
                    score = 1.0;
                    break;
                }
                let dx = (base_motion.to_prev_x - neighbor_motion.to_prev_x).abs();
                let dy = (base_motion.to_prev_y - neighbor_motion.to_prev_y).abs();
                score = score.max(((dx + dy) as f32 / 4.0).clamp(0.0, 1.0));
            }
            field.set(x, y, score);
        }
    }

    field
}

fn compute_thin_proxy(scene_frame: &SceneFrame) -> ScalarField {
    let width = scene_frame.ground_truth.width();
    let height = scene_frame.ground_truth.height();
    let mut field = ScalarField::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            let is_thin = matches!(scene_frame.layers[index], SurfaceTag::ThinStructure)
                || neighbors(x, y, width, height).into_iter().any(|(nx, ny)| {
                    matches!(
                        scene_frame.layers[ny * width + nx],
                        SurfaceTag::ThinStructure
                    )
                });
            field.set(x, y, if is_thin { 1.0 } else { 0.0 });
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
    for y in 0..height {
        for x in 0..width {
            trust.set(x, y, trust_value);
            alpha.set(x, y, alpha_value);
        }
    }
    SupervisionFrame {
        residual: ScalarField::new(width, height),
        trust,
        alpha,
        proxies: ProxyFields {
            residual_proxy: ScalarField::new(width, height),
            visibility_proxy: ScalarField::new(width, height),
            motion_edge_proxy: ScalarField::new(width, height),
            thin_proxy: ScalarField::new(width, height),
        },
    }
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn neighbors(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut values = Vec::with_capacity(4);
    if x > 0 {
        values.push((x - 1, y));
    }
    if x + 1 < width {
        values.push((x + 1, y));
    }
    if y > 0 {
        values.push((x, y - 1));
    }
    if y + 1 < height {
        values.push((x, y + 1));
    }
    values
}
