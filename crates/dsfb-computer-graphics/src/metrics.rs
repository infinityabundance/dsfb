use serde::Serialize;

use crate::dsfb::{DsfbRun, StateCounts};
use crate::error::{Error, Result};
use crate::frame::{
    bounding_box_from_mask, mean_abs_error, mean_abs_error_over_mask, BoundingBox, ImageFrame,
    ScalarField,
};
use crate::scene::{SceneSequence, SurfaceTag};
use crate::taa::{ResidualThresholdRun, TaaRun};

const LOW_TRUST_THRESHOLD: f32 = 0.50;
const RESIDUAL_BASELINE_RESPONSE_THRESHOLD: f32 = 0.50;

#[derive(Clone, Debug, Serialize)]
pub struct FrameMetrics {
    pub frame_index: usize,
    pub overall_mae_baseline: f32,
    pub overall_mae_residual_baseline: f32,
    pub overall_mae_dsfb: f32,
    pub persistence_roi_mae_baseline: f32,
    pub persistence_roi_mae_residual_baseline: f32,
    pub persistence_roi_mae_dsfb: f32,
    pub mean_residual: f32,
    pub mean_residual_proxy: f32,
    pub mean_visibility_proxy: f32,
    pub mean_motion_edge_proxy: f32,
    pub mean_thin_proxy: f32,
    pub mean_trust: f32,
    pub mean_alpha: f32,
    pub mean_intervention: f32,
    pub persistence_roi_trust: f32,
    pub low_trust_pixels: usize,
    pub low_trust_roi_pixels: usize,
    pub intervention_pixels: usize,
    pub residual_baseline_trigger_mean: f32,
    pub residual_baseline_roi_trigger_mean: f32,
    pub state_counts: StateCounts,
    pub disocclusion_pixels: usize,
    pub thin_disocclusion_pixels: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct SummaryMetrics {
    pub reveal_frame: usize,
    pub trust_map_frame: usize,
    pub comparison_frame: usize,
    pub trust_drop_frame: usize,
    pub trust_min_frame: usize,
    pub residual_baseline_response_frame: usize,
    pub persistence_threshold: f32,
    pub low_trust_threshold: f32,
    pub persistence_mask_pixels: usize,
    pub baseline_ghost_persistence_frames: usize,
    pub residual_baseline_ghost_persistence_frames: usize,
    pub dsfb_ghost_persistence_frames: usize,
    pub ghost_persistence_reduction_vs_baseline: isize,
    pub ghost_persistence_reduction_vs_residual_baseline: isize,
    pub average_overall_mae_baseline: f32,
    pub average_overall_mae_residual_baseline: f32,
    pub average_overall_mae_dsfb: f32,
    pub cumulative_persistence_roi_mae_baseline: f32,
    pub cumulative_persistence_roi_mae_residual_baseline: f32,
    pub cumulative_persistence_roi_mae_dsfb: f32,
    pub cumulative_roi_error_reduction_vs_baseline: f32,
    pub cumulative_roi_error_reduction_vs_residual_baseline: f32,
    pub baseline_peak_roi_error: f32,
    pub residual_baseline_peak_roi_error: f32,
    pub dsfb_peak_roi_error: f32,
    pub peak_roi_error_reduction_vs_baseline: f32,
    pub peak_roi_error_reduction_vs_residual_baseline: f32,
    pub baseline_peak_roi_error_frame: usize,
    pub residual_baseline_peak_roi_error_frame: usize,
    pub dsfb_peak_roi_error_frame: usize,
    pub trust_error_correlation: f32,
    pub reveal_frame_persistence_roi_trust: f32,
    pub reveal_frame_mean_residual: f32,
    pub reveal_frame_low_trust_pixels: usize,
    pub reveal_frame_intervention_pixels: usize,
    pub mean_low_trust_pixels: f32,
    pub trust_response_lead_vs_residual_baseline_frames: isize,
    pub primary_behavioral_result: String,
    pub secondary_behavioral_result: Option<String>,
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
    pub motion_edge_bbox: BoundingBox,
    pub reveal_frame: usize,
    pub trust_map_frame: usize,
    pub comparison_frame: usize,
}

pub fn analyze_demo_a(
    sequence: &SceneSequence,
    baseline: &TaaRun,
    residual_baseline: &ResidualThresholdRun,
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
    let motion_edge_bbox = bounding_box_from_mask(
        &mask_from_scalar_threshold(
            &dsfb.supervision_frames[trust_map_frame]
                .proxies
                .motion_edge_proxy,
            0.5,
        ),
        sequence.config.width,
        sequence.config.height,
    )
    .unwrap_or(trust_map_bbox)
    .expand(sequence.config.width, sequence.config.height, 4);
    let comparison_frame = (reveal_frame + comparison_offset).min(sequence.frames.len() - 1);

    let threshold = persistence_threshold(
        &sequence.frames[reveal_frame - 1].ground_truth,
        &sequence.frames[reveal_frame].ground_truth,
        &persistence_mask,
    );
    let trust_error_correlation = compute_trust_error_correlation(sequence, dsfb, reveal_frame);

    let mut frame_metrics = Vec::with_capacity(sequence.frames.len());
    let mut average_baseline = 0.0;
    let mut average_residual_baseline = 0.0;
    let mut average_dsfb = 0.0;
    let mut cumulative_roi_baseline = 0.0;
    let mut cumulative_roi_residual_baseline = 0.0;
    let mut cumulative_roi_dsfb = 0.0;
    let mut low_trust_sum = 0usize;

    for frame_index in 0..sequence.frames.len() {
        let supervision = &dsfb.supervision_frames[frame_index];
        let overall_mae_baseline = mean_abs_error(
            &baseline.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
        );
        let overall_mae_residual_baseline = mean_abs_error(
            &residual_baseline.taa.resolved_frames[frame_index],
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
        let roi_mae_residual_baseline = mean_abs_error_over_mask(
            &residual_baseline.taa.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
            &persistence_mask,
        );
        let roi_mae_dsfb = mean_abs_error_over_mask(
            &dsfb.resolved_frames[frame_index],
            &sequence.frames[frame_index].ground_truth,
            &persistence_mask,
        );
        let mean_residual = supervision.residual.mean();
        let mean_residual_proxy = supervision.proxies.residual_proxy.mean();
        let mean_visibility_proxy = supervision.proxies.visibility_proxy.mean();
        let mean_motion_edge_proxy = supervision.proxies.motion_edge_proxy.mean();
        let mean_thin_proxy = supervision.proxies.thin_proxy.mean();
        let mean_trust = supervision.trust.mean();
        let mean_alpha = supervision.alpha.mean();
        let mean_intervention = supervision.intervention.mean();
        let persistence_roi_trust = supervision.trust.mean_over_mask(&persistence_mask);
        let low_trust_pixels = count_field_below(&supervision.trust, LOW_TRUST_THRESHOLD);
        let low_trust_roi_pixels =
            count_field_below_over_mask(&supervision.trust, LOW_TRUST_THRESHOLD, &persistence_mask);
        let intervention_pixels =
            count_field_above(&supervision.intervention, 1.0 - LOW_TRUST_THRESHOLD);
        let residual_baseline_trigger_mean = residual_baseline.trigger_frames[frame_index].mean();
        let residual_baseline_roi_trigger_mean =
            residual_baseline.trigger_frames[frame_index].mean_over_mask(&persistence_mask);
        let state_counts = supervision.state.counts();
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
        average_residual_baseline += overall_mae_residual_baseline;
        average_dsfb += overall_mae_dsfb;
        cumulative_roi_baseline += roi_mae_baseline;
        cumulative_roi_residual_baseline += roi_mae_residual_baseline;
        cumulative_roi_dsfb += roi_mae_dsfb;
        low_trust_sum += low_trust_pixels;

        frame_metrics.push(FrameMetrics {
            frame_index,
            overall_mae_baseline,
            overall_mae_residual_baseline,
            overall_mae_dsfb,
            persistence_roi_mae_baseline: roi_mae_baseline,
            persistence_roi_mae_residual_baseline: roi_mae_residual_baseline,
            persistence_roi_mae_dsfb: roi_mae_dsfb,
            mean_residual,
            mean_residual_proxy,
            mean_visibility_proxy,
            mean_motion_edge_proxy,
            mean_thin_proxy,
            mean_trust,
            mean_alpha,
            mean_intervention,
            persistence_roi_trust,
            low_trust_pixels,
            low_trust_roi_pixels,
            intervention_pixels,
            residual_baseline_trigger_mean,
            residual_baseline_roi_trigger_mean,
            state_counts,
            disocclusion_pixels,
            thin_disocclusion_pixels,
        });
    }

    average_baseline /= sequence.frames.len() as f32;
    average_residual_baseline /= sequence.frames.len() as f32;
    average_dsfb /= sequence.frames.len() as f32;

    let (baseline_peak_roi_error_frame, baseline_peak_roi_error) =
        peak_error_frame(&frame_metrics, |frame| frame.persistence_roi_mae_baseline);
    let (residual_baseline_peak_roi_error_frame, residual_baseline_peak_roi_error) =
        peak_error_frame(&frame_metrics, |frame| {
            frame.persistence_roi_mae_residual_baseline
        });
    let (dsfb_peak_roi_error_frame, dsfb_peak_roi_error) =
        peak_error_frame(&frame_metrics, |frame| frame.persistence_roi_mae_dsfb);
    let trust_drop_frame =
        first_frame_at_or_below(&frame_metrics, reveal_frame, LOW_TRUST_THRESHOLD, |frame| {
            frame.persistence_roi_trust
        });
    let trust_min_frame = min_frame_from(&frame_metrics, reveal_frame, |frame| {
        frame.persistence_roi_trust
    });
    let residual_baseline_response_frame = first_frame_at_or_above(
        &frame_metrics,
        reveal_frame,
        RESIDUAL_BASELINE_RESPONSE_THRESHOLD,
        |frame| frame.residual_baseline_roi_trigger_mean,
    );

    let baseline_ghost_persistence_frames = compute_ghost_persistence(
        &baseline.resolved_frames,
        sequence,
        reveal_frame,
        &persistence_mask,
        threshold,
    );
    let residual_baseline_ghost_persistence_frames = compute_ghost_persistence(
        &residual_baseline.taa.resolved_frames,
        sequence,
        reveal_frame,
        &persistence_mask,
        threshold,
    );
    let dsfb_ghost_persistence_frames = compute_ghost_persistence(
        &dsfb.resolved_frames,
        sequence,
        reveal_frame,
        &persistence_mask,
        threshold,
    );

    let ghost_persistence_reduction_vs_baseline =
        baseline_ghost_persistence_frames as isize - dsfb_ghost_persistence_frames as isize;
    let ghost_persistence_reduction_vs_residual_baseline =
        residual_baseline_ghost_persistence_frames as isize
            - dsfb_ghost_persistence_frames as isize;
    let cumulative_roi_error_reduction_vs_baseline = cumulative_roi_baseline - cumulative_roi_dsfb;
    let cumulative_roi_error_reduction_vs_residual_baseline =
        cumulative_roi_residual_baseline - cumulative_roi_dsfb;
    let peak_roi_error_reduction_vs_baseline = baseline_peak_roi_error - dsfb_peak_roi_error;
    let peak_roi_error_reduction_vs_residual_baseline =
        residual_baseline_peak_roi_error - dsfb_peak_roi_error;
    let trust_response_lead_vs_residual_baseline_frames =
        residual_baseline_response_frame as isize - trust_drop_frame as isize;

    let primary_behavioral_result = primary_behavioral_result(
        baseline_ghost_persistence_frames,
        dsfb_ghost_persistence_frames,
        baseline_peak_roi_error,
        dsfb_peak_roi_error,
        cumulative_roi_baseline,
        cumulative_roi_dsfb,
    );
    let secondary_behavioral_result = secondary_behavioral_result(
        residual_baseline_ghost_persistence_frames,
        dsfb_ghost_persistence_frames,
        trust_drop_frame,
        residual_baseline_response_frame,
        residual_baseline_peak_roi_error,
        dsfb_peak_roi_error,
    );

    let reveal_frame_persistence_roi_trust = frame_metrics[reveal_frame].persistence_roi_trust;
    let reveal_frame_mean_residual = frame_metrics[reveal_frame].mean_residual;
    let reveal_frame_low_trust_pixels = frame_metrics[reveal_frame].low_trust_pixels;
    let reveal_frame_intervention_pixels = frame_metrics[reveal_frame].intervention_pixels;

    let report = MetricsReport {
        frame_metrics,
        summary: SummaryMetrics {
            reveal_frame,
            trust_map_frame,
            comparison_frame,
            trust_drop_frame,
            trust_min_frame,
            residual_baseline_response_frame,
            persistence_threshold: threshold,
            low_trust_threshold: LOW_TRUST_THRESHOLD,
            persistence_mask_pixels: persistence_mask.iter().filter(|value| **value).count(),
            baseline_ghost_persistence_frames,
            residual_baseline_ghost_persistence_frames,
            dsfb_ghost_persistence_frames,
            ghost_persistence_reduction_vs_baseline,
            ghost_persistence_reduction_vs_residual_baseline,
            average_overall_mae_baseline: average_baseline,
            average_overall_mae_residual_baseline: average_residual_baseline,
            average_overall_mae_dsfb: average_dsfb,
            cumulative_persistence_roi_mae_baseline: cumulative_roi_baseline,
            cumulative_persistence_roi_mae_residual_baseline: cumulative_roi_residual_baseline,
            cumulative_persistence_roi_mae_dsfb: cumulative_roi_dsfb,
            cumulative_roi_error_reduction_vs_baseline,
            cumulative_roi_error_reduction_vs_residual_baseline,
            baseline_peak_roi_error,
            residual_baseline_peak_roi_error,
            dsfb_peak_roi_error,
            peak_roi_error_reduction_vs_baseline,
            peak_roi_error_reduction_vs_residual_baseline,
            baseline_peak_roi_error_frame,
            residual_baseline_peak_roi_error_frame,
            dsfb_peak_roi_error_frame,
            trust_error_correlation,
            reveal_frame_persistence_roi_trust,
            reveal_frame_mean_residual,
            reveal_frame_low_trust_pixels,
            reveal_frame_intervention_pixels,
            mean_low_trust_pixels: low_trust_sum as f32 / sequence.frames.len() as f32,
            trust_response_lead_vs_residual_baseline_frames,
            primary_behavioral_result,
            secondary_behavioral_result,
        },
    };

    Ok(DemoAAnalysis {
        report,
        persistence_mask,
        persistence_bbox,
        trust_map_bbox,
        motion_edge_bbox,
        reveal_frame,
        trust_map_frame,
        comparison_frame,
    })
}

fn primary_behavioral_result(
    baseline_ghost_persistence_frames: usize,
    dsfb_ghost_persistence_frames: usize,
    baseline_peak_roi_error: f32,
    dsfb_peak_roi_error: f32,
    cumulative_roi_baseline: f32,
    cumulative_roi_dsfb: f32,
) -> String {
    if baseline_ghost_persistence_frames > dsfb_ghost_persistence_frames {
        format!(
            "In this bounded synthetic setting, DSFB reduced ghost persistence duration from {} to {} frames relative to the fixed-alpha baseline.",
            baseline_ghost_persistence_frames, dsfb_ghost_persistence_frames
        )
    } else if baseline_peak_roi_error > dsfb_peak_roi_error {
        format!(
            "In this bounded synthetic setting, DSFB reduced peak persistence-ROI error from {:.4} to {:.4} relative to the fixed-alpha baseline.",
            baseline_peak_roi_error, dsfb_peak_roi_error
        )
    } else {
        format!(
            "In this bounded synthetic setting, the fixed-alpha baseline accumulated {:.4} persistence-ROI error versus {:.4} for DSFB.",
            cumulative_roi_baseline, cumulative_roi_dsfb
        )
    }
}

fn secondary_behavioral_result(
    residual_baseline_ghost_persistence_frames: usize,
    dsfb_ghost_persistence_frames: usize,
    trust_drop_frame: usize,
    residual_baseline_response_frame: usize,
    residual_baseline_peak_roi_error: f32,
    dsfb_peak_roi_error: f32,
) -> Option<String> {
    if residual_baseline_ghost_persistence_frames > dsfb_ghost_persistence_frames {
        Some(format!(
            "Against the residual-threshold baseline, DSFB reduced ghost persistence duration from {} to {} frames.",
            residual_baseline_ghost_persistence_frames, dsfb_ghost_persistence_frames
        ))
    } else if trust_drop_frame < residual_baseline_response_frame {
        Some(format!(
            "The DSFB supervisory signal crossed the low-trust threshold at frame {}, earlier than the residual-threshold baseline response at frame {}.",
            trust_drop_frame, residual_baseline_response_frame
        ))
    } else if residual_baseline_peak_roi_error > dsfb_peak_roi_error {
        Some(format!(
            "Against the residual-threshold baseline, DSFB reduced peak persistence-ROI error from {:.4} to {:.4}.",
            residual_baseline_peak_roi_error, dsfb_peak_roi_error
        ))
    } else {
        None
    }
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

fn count_field_below(field: &ScalarField, threshold: f32) -> usize {
    field
        .values()
        .iter()
        .filter(|value| **value < threshold)
        .count()
}

fn count_field_below_over_mask(field: &ScalarField, threshold: f32, mask: &[bool]) -> usize {
    field
        .values()
        .iter()
        .zip(mask.iter().copied())
        .filter(|(value, include)| *include && **value < threshold)
        .count()
}

fn count_field_above(field: &ScalarField, threshold: f32) -> usize {
    field
        .values()
        .iter()
        .filter(|value| **value > threshold)
        .count()
}

fn mask_from_scalar_threshold(field: &ScalarField, threshold: f32) -> Vec<bool> {
    field
        .values()
        .iter()
        .map(|value| *value >= threshold)
        .collect()
}

fn peak_error_frame(
    metrics: &[FrameMetrics],
    selector: impl Fn(&FrameMetrics) -> f32,
) -> (usize, f32) {
    metrics
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| selector(left).total_cmp(&selector(right)))
        .map(|(index, frame)| (index, selector(frame)))
        .unwrap_or((0, 0.0))
}

fn first_frame_at_or_below(
    metrics: &[FrameMetrics],
    start_frame: usize,
    threshold: f32,
    selector: impl Fn(&FrameMetrics) -> f32,
) -> usize {
    metrics
        .iter()
        .skip(start_frame)
        .find(|frame| selector(frame) <= threshold)
        .map(|frame| frame.frame_index)
        .unwrap_or(start_frame)
}

fn first_frame_at_or_above(
    metrics: &[FrameMetrics],
    start_frame: usize,
    threshold: f32,
    selector: impl Fn(&FrameMetrics) -> f32,
) -> usize {
    metrics
        .iter()
        .skip(start_frame)
        .find(|frame| selector(frame) >= threshold)
        .map(|frame| frame.frame_index)
        .unwrap_or(start_frame)
}

fn min_frame_from(
    metrics: &[FrameMetrics],
    start_frame: usize,
    selector: impl Fn(&FrameMetrics) -> f32,
) -> usize {
    metrics
        .iter()
        .skip(start_frame)
        .min_by(|left, right| selector(left).total_cmp(&selector(right)))
        .map(|frame| frame.frame_index)
        .unwrap_or(start_frame)
}
