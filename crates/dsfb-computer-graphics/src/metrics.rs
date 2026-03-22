use serde::Serialize;

use crate::dsfb::DsfbRun;
use crate::error::{Error, Result};
use crate::frame::{
    bounding_box_from_mask, mean_abs_error, mean_abs_error_over_mask, BoundingBox, ImageFrame,
};
use crate::scene::{SceneSequence, SurfaceTag};
use crate::taa::TaaRun;

#[derive(Clone, Debug, Serialize)]
pub struct FrameMetrics {
    pub frame_index: usize,
    pub overall_mae_baseline: f32,
    pub overall_mae_dsfb: f32,
    pub persistence_roi_mae_baseline: f32,
    pub persistence_roi_mae_dsfb: f32,
    pub mean_residual: f32,
    pub mean_trust: f32,
    pub persistence_roi_trust: f32,
    pub disocclusion_pixels: usize,
    pub thin_disocclusion_pixels: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct SummaryMetrics {
    pub reveal_frame: usize,
    pub persistence_threshold: f32,
    pub persistence_mask_pixels: usize,
    pub baseline_ghost_persistence_frames: usize,
    pub dsfb_ghost_persistence_frames: usize,
    pub average_overall_mae_baseline: f32,
    pub average_overall_mae_dsfb: f32,
    pub cumulative_persistence_roi_mae_baseline: f32,
    pub cumulative_persistence_roi_mae_dsfb: f32,
    pub trust_error_correlation: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MetricsReport {
    pub frame_metrics: Vec<FrameMetrics>,
    pub summary: SummaryMetrics,
}

#[derive(Clone, Debug)]
pub struct DemoAAnalysis {
    pub report: MetricsReport,
    pub persistence_mask: Vec<bool>,
    pub persistence_bbox: BoundingBox,
    pub trust_map_bbox: BoundingBox,
    pub reveal_frame: usize,
    pub trust_map_frame: usize,
    pub comparison_frame: usize,
}

pub fn analyze_demo_a(
    sequence: &SceneSequence,
    baseline: &TaaRun,
    dsfb: &DsfbRun,
    trust_map_offset: usize,
    comparison_offset: usize,
) -> Result<DemoAAnalysis> {
    let reveal_frame = find_reveal_frame(sequence)?;
    let persistence_mask = build_persistence_mask(sequence, reveal_frame)?;
    let persistence_bbox = bounding_box_from_mask(
        &persistence_mask,
        sequence.config.width,
        sequence.config.height,
    )
    .ok_or_else(|| Error::Message("persistence mask was empty".to_string()))?
    .expand(sequence.config.width, sequence.config.height, 4);

    let trust_map_frame = (reveal_frame + trust_map_offset).min(sequence.frames.len() - 1);
    let trust_map_bbox = bounding_box_from_mask(
        &sequence.frames[trust_map_frame].disocclusion_mask,
        sequence.config.width,
        sequence.config.height,
    )
    .unwrap_or(persistence_bbox)
    .expand(sequence.config.width, sequence.config.height, 6);
    let comparison_frame = (reveal_frame + comparison_offset).min(sequence.frames.len() - 1);

    let threshold = persistence_threshold(
        &sequence.frames[reveal_frame - 1].ground_truth,
        &sequence.frames[reveal_frame].ground_truth,
        &persistence_mask,
    );

    let trust_error_correlation = compute_trust_error_correlation(sequence, dsfb, reveal_frame);

    let mut frame_metrics = Vec::with_capacity(sequence.frames.len());
    let mut average_baseline = 0.0;
    let mut average_dsfb = 0.0;
    let mut cumulative_roi_baseline = 0.0;
    let mut cumulative_roi_dsfb = 0.0;

    for frame_index in 0..sequence.frames.len() {
        let overall_mae_baseline = mean_abs_error(
            &baseline.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
        );
        let overall_mae_dsfb = mean_abs_error(
            &dsfb.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
        );
        let roi_mae_baseline = mean_abs_error_over_mask(
            &baseline.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
            &persistence_mask,
        );
        let roi_mae_dsfb = mean_abs_error_over_mask(
            &dsfb.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
            &persistence_mask,
        );
        let mean_residual = dsfb.supervision_frames[frame_index].residual.mean();
        let mean_trust = dsfb.supervision_frames[frame_index].trust.mean();
        let persistence_roi_trust = dsfb.supervision_frames[frame_index]
            .trust
            .mean_over_mask(&persistence_mask);
        let disocclusion_pixels = sequence.frames[frame_index]
            .disocclusion_mask
            .iter()
            .filter(|value| **value)
            .count();
        let thin_disocclusion_pixels = sequence.frames[frame_index]
            .layers
            .iter()
            .zip(
                sequence.frames[frame_index]
                    .disocclusion_mask
                    .iter()
                    .copied(),
            )
            .filter(|(layer, disoccluded)| {
                *disoccluded && matches!(*layer, SurfaceTag::ThinStructure)
            })
            .count();

        average_baseline += overall_mae_baseline;
        average_dsfb += overall_mae_dsfb;
        cumulative_roi_baseline += roi_mae_baseline;
        cumulative_roi_dsfb += roi_mae_dsfb;

        frame_metrics.push(FrameMetrics {
            frame_index,
            overall_mae_baseline,
            overall_mae_dsfb,
            persistence_roi_mae_baseline: roi_mae_baseline,
            persistence_roi_mae_dsfb: roi_mae_dsfb,
            mean_residual,
            mean_trust,
            persistence_roi_trust,
            disocclusion_pixels,
            thin_disocclusion_pixels,
        });
    }

    average_baseline /= sequence.frames.len() as f32;
    average_dsfb /= sequence.frames.len() as f32;

    let report = MetricsReport {
        frame_metrics,
        summary: SummaryMetrics {
            reveal_frame,
            persistence_threshold: threshold,
            persistence_mask_pixels: persistence_mask.iter().filter(|value| **value).count(),
            baseline_ghost_persistence_frames: compute_ghost_persistence(
                &baseline.resolved_frames,
                sequence,
                reveal_frame,
                &persistence_mask,
                threshold,
            ),
            dsfb_ghost_persistence_frames: compute_ghost_persistence(
                &dsfb.resolved_frames,
                sequence,
                reveal_frame,
                &persistence_mask,
                threshold,
            ),
            average_overall_mae_baseline: average_baseline,
            average_overall_mae_dsfb: average_dsfb,
            cumulative_persistence_roi_mae_baseline: cumulative_roi_baseline,
            cumulative_persistence_roi_mae_dsfb: cumulative_roi_dsfb,
            trust_error_correlation,
        },
    };

    Ok(DemoAAnalysis {
        report,
        persistence_mask,
        persistence_bbox,
        trust_map_bbox,
        reveal_frame,
        trust_map_frame,
        comparison_frame,
    })
}

fn find_reveal_frame(sequence: &SceneSequence) -> Result<usize> {
    sequence
        .frames
        .iter()
        .enumerate()
        .skip(1)
        .max_by_key(|(_, frame)| {
            frame
                .layers
                .iter()
                .zip(frame.disocclusion_mask.iter().copied())
                .filter(|(layer, disoccluded)| {
                    *disoccluded && matches!(*layer, SurfaceTag::ThinStructure)
                })
                .count()
        })
        .and_then(|(index, frame)| {
            let thin_pixels = frame
                .layers
                .iter()
                .zip(frame.disocclusion_mask.iter().copied())
                .filter(|(layer, disoccluded)| {
                    *disoccluded && matches!(*layer, SurfaceTag::ThinStructure)
                })
                .count();
            (thin_pixels > 0).then_some(index)
        })
        .ok_or_else(|| Error::Message("could not find a thin-structure reveal frame".to_string()))
}

fn build_persistence_mask(sequence: &SceneSequence, reveal_frame: usize) -> Result<Vec<bool>> {
    let frame = &sequence.frames[reveal_frame];
    let mask: Vec<bool> = frame
        .layers
        .iter()
        .zip(frame.disocclusion_mask.iter().copied())
        .map(|(layer, disoccluded)| disoccluded && matches!(*layer, SurfaceTag::ThinStructure))
        .collect();

    if mask.iter().any(|value| *value) {
        Ok(mask)
    } else {
        Err(Error::Message(
            "thin-structure persistence mask was empty at reveal frame".to_string(),
        ))
    }
}

fn persistence_threshold(previous: &ImageFrame, current: &ImageFrame, mask: &[bool]) -> f32 {
    let onset_contrast = mean_abs_error_over_mask(previous, current, mask);
    (onset_contrast * 0.15).max(0.02)
}

fn compute_ghost_persistence(
    resolved_frames: &[ImageFrame],
    sequence: &SceneSequence,
    reveal_frame: usize,
    mask: &[bool],
    threshold: f32,
) -> usize {
    let mut count = 0usize;
    for (frame_index, resolved_frame) in resolved_frames
        .iter()
        .enumerate()
        .take(sequence.frames.len())
        .skip(reveal_frame)
    {
        let error = mean_abs_error_over_mask(
            resolved_frame,
            &sequence.frames[frame_index].ground_truth,
            mask,
        );
        if error > threshold {
            count += 1;
        } else {
            break;
        }
    }
    count
}

fn compute_trust_error_correlation(
    sequence: &SceneSequence,
    dsfb: &DsfbRun,
    reveal_frame: usize,
) -> f32 {
    let trust = &dsfb.supervision_frames[reveal_frame].trust;
    let reprojected = &dsfb.reprojected_history_frames[reveal_frame];
    let current = &sequence.frames[reveal_frame].ground_truth;
    pearson_correlation(
        &trust
            .values()
            .iter()
            .map(|value| 1.0 - *value)
            .collect::<Vec<_>>(),
        &current
            .pixels()
            .iter()
            .zip(reprojected.pixels())
            .map(|(current_pixel, history_pixel)| current_pixel.abs_diff(*history_pixel))
            .collect::<Vec<_>>(),
    )
}

fn pearson_correlation(xs: &[f32], ys: &[f32]) -> f32 {
    if xs.len() != ys.len() || xs.is_empty() {
        return 0.0;
    }
    let mean_x = xs.iter().sum::<f32>() / xs.len() as f32;
    let mean_y = ys.iter().sum::<f32>() / ys.len() as f32;
    let mut numerator = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = *x - mean_x;
        let dy = *y - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }
    if denom_x <= f32::EPSILON || denom_y <= f32::EPSILON {
        0.0
    } else {
        numerator / (denom_x.sqrt() * denom_y.sqrt())
    }
}
