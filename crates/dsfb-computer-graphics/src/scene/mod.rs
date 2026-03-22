use serde::Serialize;

use crate::config::SceneConfig;
use crate::frame::{Color, ImageFrame};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SurfaceTag {
    Background,
    ThinStructure,
    ForegroundObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MotionVector {
    pub to_prev_x: i32,
    pub to_prev_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Clone, Debug)]
pub struct SceneFrame {
    pub index: usize,
    pub ground_truth: ImageFrame,
    pub layers: Vec<SurfaceTag>,
    pub motion: Vec<MotionVector>,
    pub disocclusion_mask: Vec<bool>,
    pub object_rect: Rect,
}

#[derive(Clone, Debug)]
pub struct SceneSequence {
    pub config: SceneConfig,
    pub frames: Vec<SceneFrame>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SceneManifest {
    pub config: SceneConfig,
    pub frame_count: usize,
    pub reveal_frame_guess: usize,
}

pub fn generate_sequence(config: &SceneConfig) -> SceneSequence {
    let object_positions = build_object_positions(config);
    let mut frames: Vec<SceneFrame> = Vec::with_capacity(config.frame_count);

    for frame_index in 0..config.frame_count {
        let object_rect = Rect {
            x: object_positions[frame_index],
            y: config.object_top_y,
            width: config.object_width as i32,
            height: config.object_height as i32,
        };
        let previous_object_x = if frame_index == 0 {
            object_rect.x
        } else {
            object_positions[frame_index - 1]
        };
        let object_dx = object_rect.x - previous_object_x;

        let mut ground_truth = ImageFrame::new(config.width, config.height);
        let mut layers = vec![SurfaceTag::Background; config.width * config.height];
        let mut motion = vec![
            MotionVector {
                to_prev_x: 0,
                to_prev_y: 0,
            };
            config.width * config.height
        ];

        for y in 0..config.height {
            for x in 0..config.width {
                let x_i = x as i32;
                let y_i = y as i32;
                let mut color = background_color(x, y, config);
                let mut layer = SurfaceTag::Background;

                if is_thin_structure(x_i, y_i, config) {
                    color = thin_structure_color(x_i, y_i, config);
                    layer = SurfaceTag::ThinStructure;
                }

                if object_rect.contains(x_i, y_i) {
                    color = object_color(x_i, y_i, object_rect);
                    layer = SurfaceTag::ForegroundObject;
                }

                let index = y * config.width + x;
                ground_truth.set(x, y, color);
                layers[index] = layer;
                if matches!(layer, SurfaceTag::ForegroundObject) {
                    motion[index] = MotionVector {
                        to_prev_x: -object_dx,
                        to_prev_y: 0,
                    };
                }
            }
        }

        let disocclusion_mask = if frame_index == 0 {
            vec![false; config.width * config.height]
        } else {
            let previous_layers = &frames[frame_index - 1].layers;
            let mut mask = vec![false; config.width * config.height];
            for y in 0..config.height {
                for x in 0..config.width {
                    let index = y * config.width + x;
                    let motion_vector = motion[index];
                    let prev_x = (x as i32 + motion_vector.to_prev_x)
                        .clamp(0, config.width as i32 - 1)
                        as usize;
                    let prev_y = (y as i32 + motion_vector.to_prev_y)
                        .clamp(0, config.height as i32 - 1)
                        as usize;
                    let previous_layer = previous_layers[prev_y * config.width + prev_x];
                    mask[index] = previous_layer != layers[index]
                        && !matches!(layers[index], SurfaceTag::ForegroundObject);
                }
            }
            mask
        };

        frames.push(SceneFrame {
            index: frame_index,
            ground_truth,
            layers,
            motion,
            disocclusion_mask,
            object_rect,
        });
    }

    SceneSequence {
        config: config.clone(),
        frames,
    }
}

pub fn build_manifest(sequence: &SceneSequence) -> SceneManifest {
    let reveal_frame_guess = sequence
        .frames
        .iter()
        .enumerate()
        .max_by_key(|(_, frame)| thin_disocclusion_pixels(frame))
        .and_then(|(index, frame)| (thin_disocclusion_pixels(frame) > 0).then_some(index))
        .unwrap_or(0);

    SceneManifest {
        config: sequence.config.clone(),
        frame_count: sequence.frames.len(),
        reveal_frame_guess,
    }
}

fn thin_disocclusion_pixels(frame: &SceneFrame) -> usize {
    frame
        .layers
        .iter()
        .zip(frame.disocclusion_mask.iter().copied())
        .filter(|(layer, disoccluded)| *disoccluded && matches!(*layer, SurfaceTag::ThinStructure))
        .count()
}

fn build_object_positions(config: &SceneConfig) -> Vec<i32> {
    let mut positions = Vec::with_capacity(config.frame_count);
    for frame_index in 0..config.frame_count {
        if frame_index < config.move_frames {
            let span = config.object_stop_x - config.object_start_x;
            let step = span as f32 / config.move_frames.max(1) as f32;
            let position = config.object_start_x as f32 + step * frame_index as f32;
            positions.push(position.round() as i32);
        } else {
            positions.push(config.object_stop_x);
        }
    }
    positions
}

fn background_color(x: usize, y: usize, config: &SceneConfig) -> Color {
    let xf = x as f32 / (config.width.saturating_sub(1).max(1)) as f32;
    let yf = y as f32 / (config.height.saturating_sub(1).max(1)) as f32;
    let checker = if ((x / 12) + (y / 12)) % 2 == 0 {
        1.0
    } else {
        0.0
    };
    let diagonal = if (x + 2 * y) % 22 < 6 { 1.0 } else { 0.0 };
    let vignette_x = (xf - 0.5).abs();
    let vignette_y = (yf - 0.5).abs();
    let vignette = 1.0 - (vignette_x * 0.35 + vignette_y * 0.4);

    Color::rgb(
        (0.12 + 0.16 * xf + 0.05 * checker + 0.03 * diagonal) * vignette,
        (0.15 + 0.11 * yf + 0.04 * diagonal) * vignette,
        (0.22 + 0.18 * (1.0 - xf) + 0.03 * checker) * vignette,
    )
}

fn is_thin_structure(x: i32, y: i32, config: &SceneConfig) -> bool {
    let vertical = x == config.thin_vertical_x && y >= 14 && y <= config.height as i32 - 14;
    let diagonal_line = {
        let diagonal = 0.58 * x as f32 + 10.0;
        (y as f32 - diagonal).abs() <= 0.55 && (28..=118).contains(&x)
    };
    vertical || diagonal_line
}

fn thin_structure_color(x: i32, y: i32, config: &SceneConfig) -> Color {
    if x == config.thin_vertical_x {
        let pulse = if y % 6 < 3 { 1.0 } else { 0.82 };
        return Color::rgb(0.95 * pulse, 0.96 * pulse, 0.98);
    }
    Color::rgb(0.64, 0.90, 0.96)
}

fn object_color(x: i32, y: i32, rect: Rect) -> Color {
    let local_x = (x - rect.x) as f32 / rect.width.max(1) as f32;
    let local_y = (y - rect.y) as f32 / rect.height.max(1) as f32;
    let stripe = if local_x > 0.36 && local_x < 0.46 {
        0.55
    } else {
        1.0
    };
    let rim = if !(2..=(rect.width - 3)).contains(&(x - rect.x))
        || !(2..=(rect.height - 3)).contains(&(y - rect.y))
    {
        1.12
    } else {
        1.0
    };

    Color::rgb(
        (0.82 + 0.10 * local_y) * stripe * rim,
        (0.35 + 0.12 * (1.0 - local_y)) * stripe * rim,
        (0.20 + 0.08 * local_x) * stripe * rim,
    )
    .clamp01()
}
