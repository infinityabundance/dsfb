use serde::Serialize;

use crate::config::{DemoConfig, SceneConfig};
use crate::dsfb::DsfbRun;
use crate::error::{Error, Result};
use crate::frame::{
    mean_abs_error, mean_abs_error_over_mask, BoundingBox, Color, ImageFrame, ScalarField,
};
use crate::metrics::DemoAAnalysis;
use crate::scene::{Rect, SceneFrame, SceneSequence};

#[derive(Clone, Debug, Serialize)]
pub struct DemoBMetrics {
    pub reveal_frame: usize,
    pub reference_spp: usize,
    pub uniform_spp: usize,
    pub guided_min_spp: usize,
    pub guided_max_spp: usize,
    pub total_pixels: usize,
    pub uniform_total_samples: usize,
    pub guided_total_samples: usize,
    pub uniform_mae: f32,
    pub guided_mae: f32,
    pub uniform_rmse: f32,
    pub guided_rmse: f32,
    pub uniform_roi_mae: f32,
    pub guided_roi_mae: f32,
    pub uniform_roi_rmse: f32,
    pub guided_roi_rmse: f32,
    pub roi_pixels: usize,
    pub mean_guided_spp: f32,
    pub roi_mean_guided_spp: f32,
    pub max_guided_spp: usize,
    pub trust_roi_mean: f32,
}

#[derive(Clone, Debug)]
pub struct DemoBRun {
    pub reference_frame: ImageFrame,
    pub uniform_frame: ImageFrame,
    pub guided_frame: ImageFrame,
    pub uniform_error: ScalarField,
    pub guided_error: ScalarField,
    pub guided_spp: ScalarField,
    pub trust: ScalarField,
    pub metrics: DemoBMetrics,
    pub focus_bbox: BoundingBox,
}

pub fn run_demo_b(
    config: &DemoConfig,
    sequence: &SceneSequence,
    dsfb: &DsfbRun,
    analysis: &DemoAAnalysis,
) -> Result<DemoBRun> {
    let reveal_frame = analysis.reveal_frame;
    let scene_frame = &sequence.frames[reveal_frame];
    let trust = &dsfb.supervision_frames[reveal_frame].trust;
    let width = sequence.config.width;
    let height = sequence.config.height;
    let total_pixels = width * height;

    let uniform_total_samples = config.demo_b_uniform_spp * total_pixels;
    let minimum_total_samples = config.demo_b_min_spp * total_pixels;
    if uniform_total_samples < minimum_total_samples {
        return Err(Error::Message(
            "Demo B uniform budget must be at least the guided minimum budget".to_string(),
        ));
    }
    if config.demo_b_max_spp < config.demo_b_min_spp {
        return Err(Error::Message(
            "Demo B max spp must be greater than or equal to min spp".to_string(),
        ));
    }

    let reference_counts = vec![config.demo_b_reference_spp; total_pixels];
    let uniform_counts = vec![config.demo_b_uniform_spp; total_pixels];
    let guided_counts = guided_allocation(
        trust,
        uniform_total_samples,
        config.demo_b_min_spp,
        config.demo_b_max_spp,
    )?;

    let reference_frame = render_with_counts(&sequence.config, scene_frame, &reference_counts);
    let uniform_frame = render_with_counts(&sequence.config, scene_frame, &uniform_counts);
    let guided_frame = render_with_counts(&sequence.config, scene_frame, &guided_counts);

    let uniform_error = build_error_field(&uniform_frame, &reference_frame);
    let guided_error = build_error_field(&guided_frame, &reference_frame);
    let guided_spp = build_count_field(&guided_counts, width, height);

    let metrics = DemoBMetrics {
        reveal_frame,
        reference_spp: config.demo_b_reference_spp,
        uniform_spp: config.demo_b_uniform_spp,
        guided_min_spp: config.demo_b_min_spp,
        guided_max_spp: config.demo_b_max_spp,
        total_pixels,
        uniform_total_samples,
        guided_total_samples: guided_counts.iter().sum(),
        uniform_mae: mean_abs_error(&uniform_frame, &reference_frame),
        guided_mae: mean_abs_error(&guided_frame, &reference_frame),
        uniform_rmse: rmse(&uniform_frame, &reference_frame, None),
        guided_rmse: rmse(&guided_frame, &reference_frame, None),
        uniform_roi_mae: mean_abs_error_over_mask(
            &uniform_frame,
            &reference_frame,
            &analysis.persistence_mask,
        ),
        guided_roi_mae: mean_abs_error_over_mask(
            &guided_frame,
            &reference_frame,
            &analysis.persistence_mask,
        ),
        uniform_roi_rmse: rmse(
            &uniform_frame,
            &reference_frame,
            Some(&analysis.persistence_mask),
        ),
        guided_roi_rmse: rmse(
            &guided_frame,
            &reference_frame,
            Some(&analysis.persistence_mask),
        ),
        roi_pixels: analysis
            .persistence_mask
            .iter()
            .filter(|value| **value)
            .count(),
        mean_guided_spp: guided_counts.iter().sum::<usize>() as f32 / total_pixels as f32,
        roi_mean_guided_spp: mean_count_over_mask(&guided_counts, &analysis.persistence_mask),
        max_guided_spp: guided_counts.iter().copied().max().unwrap_or(0),
        trust_roi_mean: trust.mean_over_mask(&analysis.persistence_mask),
    };

    Ok(DemoBRun {
        reference_frame,
        uniform_frame,
        guided_frame,
        uniform_error,
        guided_error,
        guided_spp,
        trust: trust.clone(),
        metrics,
        focus_bbox: analysis.persistence_bbox,
    })
}

fn render_with_counts(
    config: &SceneConfig,
    scene_frame: &SceneFrame,
    counts: &[usize],
) -> ImageFrame {
    let mut frame = ImageFrame::new(config.width, config.height);

    for y in 0..config.height {
        for x in 0..config.width {
            let pixel_index = y * config.width + x;
            let sample_count = counts[pixel_index].max(1);
            let mut accum = Color::rgb(0.0, 0.0, 0.0);

            for sample_index in 0..sample_count {
                let (offset_x, offset_y) = sample_offset(pixel_index as u32, sample_index as u32);
                let sample = sample_scene_continuous(
                    config,
                    scene_frame.object_rect,
                    x as f32 + offset_x,
                    y as f32 + offset_y,
                );
                accum = Color::rgb(accum.r + sample.r, accum.g + sample.g, accum.b + sample.b);
            }

            let inv = 1.0 / sample_count as f32;
            frame.set(
                x,
                y,
                Color::rgb(accum.r * inv, accum.g * inv, accum.b * inv).clamp01(),
            );
        }
    }

    frame
}

fn guided_allocation(
    trust: &ScalarField,
    total_samples: usize,
    min_spp: usize,
    max_spp: usize,
) -> Result<Vec<usize>> {
    let total_pixels = trust.width() * trust.height();
    let min_total = min_spp * total_pixels;
    let max_total = max_spp * total_pixels;
    if total_samples < min_total || total_samples > max_total {
        return Err(Error::Message(
            "Demo B total sample budget is incompatible with the min/max spp bounds".to_string(),
        ));
    }

    let mut counts = vec![min_spp; total_pixels];
    let mut remaining = total_samples - min_total;
    if remaining == 0 {
        return Ok(counts);
    }

    let weights: Vec<f32> = trust
        .values()
        .iter()
        .map(|trust_value| {
            let hazard = (1.0 - trust_value).clamp(0.0, 1.0);
            0.03 + 0.97 * hazard.powi(3)
        })
        .collect();

    while remaining > 0 {
        let available_weight: f32 = counts
            .iter()
            .zip(weights.iter())
            .filter_map(|(count, weight)| (*count < max_spp).then_some(*weight))
            .sum();

        if available_weight <= f32::EPSILON {
            break;
        }

        let round_budget = remaining;
        let mut floor_assignments = vec![0usize; total_pixels];
        let mut fractional_parts = Vec::new();

        for (index, (count, weight)) in counts
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .enumerate()
        {
            if count >= max_spp {
                continue;
            }
            let capacity = max_spp - count;
            let target = round_budget as f32 * weight / available_weight;
            let whole = target.floor() as usize;
            let assigned = whole.min(capacity);
            floor_assignments[index] = assigned;
            if assigned < capacity {
                fractional_parts.push((target - assigned as f32, index));
            }
        }

        let mut assigned_this_round = 0usize;
        for (count, extra) in counts.iter_mut().zip(floor_assignments.iter().copied()) {
            *count += extra;
            assigned_this_round += extra;
        }
        remaining -= assigned_this_round.min(remaining);

        if remaining == 0 {
            break;
        }

        fractional_parts.sort_by(|left, right| right.0.total_cmp(&left.0));
        let mut assigned_fractional = 0usize;
        for (_, index) in fractional_parts {
            if remaining == 0 {
                break;
            }
            if counts[index] < max_spp {
                counts[index] += 1;
                remaining -= 1;
                assigned_fractional += 1;
            }
        }

        if assigned_this_round == 0 && assigned_fractional == 0 {
            let mut fallback: Vec<(f32, usize)> = weights
                .iter()
                .copied()
                .enumerate()
                .filter(|(index, _)| counts[*index] < max_spp)
                .map(|(index, weight)| (weight, index))
                .collect();
            fallback.sort_by(|left, right| right.0.total_cmp(&left.0));
            for (_, index) in fallback {
                if remaining == 0 {
                    break;
                }
                counts[index] += 1;
                remaining -= 1;
            }
        }
    }

    Ok(counts)
}

fn build_error_field(frame: &ImageFrame, reference: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(frame.width(), frame.height());
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            field.set(x, y, frame.get(x, y).abs_diff(reference.get(x, y)));
        }
    }
    field
}

fn build_count_field(counts: &[usize], width: usize, height: usize) -> ScalarField {
    let mut field = ScalarField::new(width, height);
    for y in 0..height {
        for x in 0..width {
            field.set(x, y, counts[y * width + x] as f32);
        }
    }
    field
}

fn mean_count_over_mask(counts: &[usize], mask: &[bool]) -> f32 {
    let mut sum = 0usize;
    let mut count = 0usize;
    for (spp, include) in counts.iter().copied().zip(mask.iter().copied()) {
        if include {
            sum += spp;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum as f32 / count as f32
    }
}

fn rmse(frame: &ImageFrame, reference: &ImageFrame, mask: Option<&[bool]>) -> f32 {
    let mut sum = 0.0;
    let mut count = 0usize;

    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let index = y * frame.width() + x;
            if mask.map(|values| values[index]).unwrap_or(true) {
                let diff = frame.get(x, y).abs_diff(reference.get(x, y));
                sum += diff * diff;
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        (sum / count as f32).sqrt()
    }
}

fn sample_scene_continuous(
    config: &SceneConfig,
    object_rect: Rect,
    sample_x: f32,
    sample_y: f32,
) -> Color {
    let mut color = background_color_continuous(sample_x, sample_y, config);
    if is_thin_structure_continuous(sample_x, sample_y, config) {
        color = thin_structure_color_continuous(sample_x, sample_y, config);
    }
    if rect_contains_continuous(object_rect, sample_x, sample_y) {
        color = object_color_continuous(sample_x, sample_y, object_rect);
    }
    color
}

fn background_color_continuous(sample_x: f32, sample_y: f32, config: &SceneConfig) -> Color {
    let xf = sample_x / config.width.max(1) as f32;
    let yf = sample_y / config.height.max(1) as f32;
    let checker = if ((sample_x / 12.0).floor() + (sample_y / 12.0).floor()) as i32 % 2 == 0 {
        1.0
    } else {
        0.0
    };
    let diagonal = if (sample_x + 2.0 * sample_y).rem_euclid(22.0) < 6.0 {
        1.0
    } else {
        0.0
    };
    let vignette_x = (xf - 0.5).abs();
    let vignette_y = (yf - 0.5).abs();
    let vignette = 1.0 - (vignette_x * 0.35 + vignette_y * 0.4);

    Color::rgb(
        (0.12 + 0.16 * xf + 0.05 * checker + 0.03 * diagonal) * vignette,
        (0.15 + 0.11 * yf + 0.04 * diagonal) * vignette,
        (0.22 + 0.18 * (1.0 - xf) + 0.03 * checker) * vignette,
    )
}

fn is_thin_structure_continuous(sample_x: f32, sample_y: f32, config: &SceneConfig) -> bool {
    let vertical_center = config.thin_vertical_x as f32 + 0.5;
    let vertical = (sample_x - vertical_center).abs() <= 0.18
        && sample_y >= 14.0
        && sample_y <= config.height as f32 - 14.0;
    let diagonal_line =
        (sample_y - (0.58 * sample_x + 10.5)).abs() <= 0.20 && (28.0..=118.0).contains(&sample_x);
    vertical || diagonal_line
}

fn thin_structure_color_continuous(sample_x: f32, sample_y: f32, config: &SceneConfig) -> Color {
    let vertical_center = config.thin_vertical_x as f32 + 0.5;
    if (sample_x - vertical_center).abs() <= 0.18 {
        let pulse = if (sample_y / 3.0).floor() as i32 % 2 == 0 {
            1.0
        } else {
            0.84
        };
        return Color::rgb(0.95 * pulse, 0.96 * pulse, 0.98);
    }
    Color::rgb(0.64, 0.90, 0.96)
}

fn rect_contains_continuous(rect: Rect, sample_x: f32, sample_y: f32) -> bool {
    sample_x >= rect.x as f32
        && sample_x < (rect.x + rect.width) as f32
        && sample_y >= rect.y as f32
        && sample_y < (rect.y + rect.height) as f32
}

fn object_color_continuous(sample_x: f32, sample_y: f32, rect: Rect) -> Color {
    let local_x = (sample_x - rect.x as f32) / rect.width.max(1) as f32;
    let local_y = (sample_y - rect.y as f32) / rect.height.max(1) as f32;
    let stripe = if (0.36..0.46).contains(&local_x) {
        0.55
    } else {
        1.0
    };
    let rim = if !(0.05..=0.95).contains(&local_x) || !(0.05..=0.95).contains(&local_y) {
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

fn sample_offset(pixel_seed: u32, sample_index: u32) -> (f32, f32) {
    let shift_x = unit_hash(pixel_seed ^ 0x9e37_79b9);
    let shift_y = unit_hash(pixel_seed ^ 0x85eb_ca6b);
    let u = (radical_inverse(sample_index + 1, 2) + shift_x).fract();
    let v = (radical_inverse(sample_index + 1, 3) + shift_y).fract();
    (u, v)
}

fn unit_hash(value: u32) -> f32 {
    let mixed = value.wrapping_mul(0x045d_9f3b).rotate_left(7) ^ 0xa511_e9b3;
    (mixed as f32 / u32::MAX as f32).fract()
}

fn radical_inverse(mut index: u32, base: u32) -> f32 {
    let mut reversed = 0.0;
    let mut inv_base = 1.0 / base as f32;
    while index > 0 {
        let digit = index % base;
        reversed += digit as f32 * inv_base;
        index /= base;
        inv_base /= base as f32;
    }
    reversed
}
