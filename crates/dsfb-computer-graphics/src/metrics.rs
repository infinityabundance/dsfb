use serde::Serialize;

use crate::error::{Error, Result};
use crate::frame::{mean_abs_error, mean_abs_error_over_mask, ImageFrame, ScalarField};
use crate::scene::{ScenarioExpectation, ScenarioSupportCategory, SceneSequence};

const LOW_RESPONSE_THRESHOLD: f32 = 0.50;

#[derive(Clone, Debug)]
pub struct RunAnalysisInput<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub category: &'a str,
    pub resolved_frames: &'a [ImageFrame],
    pub reprojected_history_frames: &'a [ImageFrame],
    pub alpha_frames: &'a [ScalarField],
    pub response_frames: &'a [ScalarField],
    pub trust_frames: Option<&'a [ScalarField]>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CalibrationBin {
    pub lower: f32,
    pub upper: f32,
    pub sample_count: usize,
    pub mean_trust: f32,
    pub mean_error: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct HistogramBin {
    pub lower: f32,
    pub upper: f32,
    pub sample_count: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum TrustOperatingMode {
    NearBinaryGate,
    WeaklyGraded,
    StronglyGraded,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunFrameMetrics {
    pub frame_index: usize,
    pub overall_mae: f32,
    pub overall_rmse: f32,
    pub roi_mae: f32,
    pub roi_rmse: f32,
    pub non_roi_mae: f32,
    pub non_roi_rmse: f32,
    pub alpha_mean: f32,
    pub alpha_roi_mean: f32,
    pub alpha_non_roi_mean: f32,
    pub response_mean: f32,
    pub response_roi_mean: f32,
    pub response_non_roi_mean: f32,
    pub trust_mean: Option<f32>,
    pub trust_roi_mean: Option<f32>,
    pub trust_non_roi_mean: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub label: String,
    pub category: String,
    pub peak_roi_mae: f32,
    pub peak_roi_mae_frame: usize,
    pub cumulative_roi_mae: f32,
    pub cumulative_non_roi_mae: f32,
    pub average_overall_mae: f32,
    pub average_overall_rmse: f32,
    pub average_roi_mae: f32,
    pub average_non_roi_mae: f32,
    pub average_non_roi_rmse: f32,
    pub ghost_persistence_frames: usize,
    pub onset_response_latency_frames: Option<usize>,
    pub false_positive_response_rate: f32,
    pub intervention_sparsity: f32,
    pub mean_alpha: f32,
    pub onset_alpha_p90: f32,
    pub onset_alpha_max: f32,
    pub alpha_temporal_delta_mean: f32,
    pub response_temporal_delta_mean: f32,
    pub temporal_variance_non_roi: f32,
    pub trust_error_rank_correlation: Option<f32>,
    pub trust_rank_correlation_is_degenerate: bool,
    pub trust_calibration_bins: Vec<CalibrationBin>,
    pub trust_histogram: Vec<HistogramBin>,
    pub trust_occupied_bin_count: usize,
    pub trust_entropy_bits: Option<f32>,
    pub trust_discreteness_score: Option<f32>,
    pub trust_effective_level_count: Option<usize>,
    pub trust_operating_mode: Option<TrustOperatingMode>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioRunReport {
    pub summary: RunSummary,
    pub frame_metrics: Vec<RunFrameMetrics>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioReport {
    pub scenario_id: String,
    pub scenario_title: String,
    pub scenario_description: String,
    pub expectation: ScenarioExpectation,
    pub support_category: ScenarioSupportCategory,
    pub roi_note: String,
    pub sampling_taxonomy: String,
    pub realism_stress: bool,
    pub competitive_baseline_case: bool,
    pub bounded_loss_disclosure: bool,
    pub demo_b_taxonomy: String,
    pub target_label: String,
    pub onset_frame: usize,
    pub target_pixels: usize,
    pub target_area_fraction: f32,
    pub persistence_threshold: f32,
    pub runs: Vec<ScenarioRunReport>,
    pub headline: String,
    pub bounded_or_neutral_note: String,
    pub host_realistic_vs_fixed_alpha_cumulative_roi_gain: f32,
    pub host_realistic_vs_strong_heuristic_cumulative_roi_gain: f32,
    pub host_realistic_non_roi_penalty_vs_fixed_alpha: f32,
    pub host_realistic_non_roi_penalty_vs_strong_heuristic: f32,
    pub host_realistic_non_roi_penalty_ratio_vs_strong_heuristic: f32,
    pub neighborhood_clamp_roi_trigger_mean: f32,
    pub neighborhood_clamp_roi_silent_fraction: f32,
    pub neighborhood_clamp_history_inside_hull_fraction: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct AblationEntry {
    pub run_id: String,
    pub label: String,
    pub canonical_cumulative_roi_mae: f32,
    pub canonical_peak_roi_mae: f32,
    pub suite_mean_cumulative_roi_mae: f32,
    pub suite_mean_false_positive_response_rate: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct AggregateRunScore {
    pub run_id: String,
    pub label: String,
    pub category: String,
    pub mean_rank: f32,
    pub mean_cumulative_roi_mae: f32,
    pub mean_non_roi_mae: f32,
    pub mean_false_positive_response_rate: f32,
    pub benefit_scenarios_won: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoASuiteSummary {
    pub canonical_scenario_id: String,
    pub scenario_ids: Vec<String>,
    pub point_roi_scenarios: Vec<String>,
    pub region_roi_scenarios: Vec<String>,
    pub negative_control_scenarios: Vec<String>,
    pub baseline_ids: Vec<String>,
    pub dsfb_ids: Vec<String>,
    pub ablation_ids: Vec<String>,
    pub primary_behavioral_result: String,
    pub secondary_behavioral_result: String,
    pub host_realistic_beats_fixed_alpha_scenarios: usize,
    pub host_realistic_beats_strong_heuristic_scenarios: usize,
    pub mixed_or_neutral_scenarios: Vec<String>,
    pub remaining_blockers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoASuiteMetrics {
    pub summary: DemoASuiteSummary,
    pub scenarios: Vec<ScenarioReport>,
    pub ablations: Vec<AblationEntry>,
    pub aggregate_leaderboard: Vec<AggregateRunScore>,
}

pub fn analyze_demo_a_suite(
    scenario_runs: &[(SceneSequence, Vec<RunAnalysisInput<'_>>)],
) -> Result<DemoASuiteMetrics> {
    if scenario_runs.is_empty() {
        return Err(Error::Message(
            "Demo A suite analysis requires at least one scenario".to_string(),
        ));
    }

    let mut scenarios = Vec::with_capacity(scenario_runs.len());
    for (sequence, runs) in scenario_runs {
        scenarios.push(analyze_scenario(sequence, runs)?);
    }

    let canonical = &scenarios[0];
    let fixed_alpha = find_run(canonical, "fixed_alpha")?;
    let strong_heuristic = find_run(canonical, "strong_heuristic")?;
    let host_realistic = find_run(canonical, "dsfb_host_realistic")?;

    let primary_behavioral_result = format!(
        "On the canonical scenario, host-realistic DSFB reduced cumulative ROI MAE from {:.5} for fixed alpha to {:.5}.",
        fixed_alpha.cumulative_roi_mae, host_realistic.cumulative_roi_mae
    );
    let secondary_behavioral_result = format!(
        "Against the strong heuristic baseline, host-realistic DSFB changed cumulative ROI MAE from {:.5} to {:.5}; mixed outcomes are surfaced per scenario below.",
        strong_heuristic.cumulative_roi_mae, host_realistic.cumulative_roi_mae
    );

    let host_realistic_beats_fixed_alpha_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            let fixed = scenario
                .runs
                .iter()
                .find(|run| run.summary.run_id == "fixed_alpha");
            let host = scenario
                .runs
                .iter()
                .find(|run| run.summary.run_id == "dsfb_host_realistic");
            match (fixed, host) {
                (Some(fixed), Some(host)) => {
                    host.summary.cumulative_roi_mae + 1e-6 < fixed.summary.cumulative_roi_mae
                }
                _ => false,
            }
        })
        .count();
    let host_realistic_beats_strong_heuristic_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            let heuristic = scenario
                .runs
                .iter()
                .find(|run| run.summary.run_id == "strong_heuristic");
            let host = scenario
                .runs
                .iter()
                .find(|run| run.summary.run_id == "dsfb_host_realistic");
            match (heuristic, host) {
                (Some(heuristic), Some(host)) => {
                    host.summary.cumulative_roi_mae + 1e-6 < heuristic.summary.cumulative_roi_mae
                }
                _ => false,
            }
        })
        .count();

    let mixed_or_neutral_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            matches!(scenario.expectation, ScenarioExpectation::NeutralExpected)
                || scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain <= 0.0
        })
        .map(|scenario| scenario.scenario_id.clone())
        .collect::<Vec<_>>();

    let baseline_ids = vec![
        "fixed_alpha".to_string(),
        "residual_threshold".to_string(),
        "neighborhood_clamp".to_string(),
        "depth_normal_reject".to_string(),
        "reactive_mask".to_string(),
        "strong_heuristic".to_string(),
    ];
    let dsfb_ids = vec![
        "dsfb_synthetic_visibility".to_string(),
        "dsfb_host_realistic".to_string(),
    ];
    let ablation_ids = vec![
        "dsfb_synthetic_visibility".to_string(),
        "dsfb_host_realistic".to_string(),
        "dsfb_host_gated_reference".to_string(),
        "dsfb_motion_augmented".to_string(),
        "dsfb_no_visibility".to_string(),
        "dsfb_no_thin".to_string(),
        "dsfb_no_motion_edge".to_string(),
        "dsfb_no_grammar".to_string(),
        "dsfb_residual_only".to_string(),
        "dsfb_trust_no_alpha".to_string(),
    ];

    let ablations = ablation_ids
        .iter()
        .filter_map(|run_id| {
            let canonical_run = canonical
                .runs
                .iter()
                .find(|run| run.summary.run_id == *run_id)?;
            let suite_matches = scenarios
                .iter()
                .filter_map(|scenario| {
                    scenario
                        .runs
                        .iter()
                        .find(|run| run.summary.run_id == *run_id)
                        .map(|run| &run.summary)
                })
                .collect::<Vec<_>>();
            let suite_count = suite_matches.len().max(1) as f32;
            Some(AblationEntry {
                run_id: (*run_id).clone(),
                label: canonical_run.summary.label.clone(),
                canonical_cumulative_roi_mae: canonical_run.summary.cumulative_roi_mae,
                canonical_peak_roi_mae: canonical_run.summary.peak_roi_mae,
                suite_mean_cumulative_roi_mae: suite_matches
                    .iter()
                    .map(|summary| summary.cumulative_roi_mae)
                    .sum::<f32>()
                    / suite_count,
                suite_mean_false_positive_response_rate: suite_matches
                    .iter()
                    .map(|summary| summary.false_positive_response_rate)
                    .sum::<f32>()
                    / suite_count,
            })
        })
        .collect::<Vec<_>>();

    let aggregate_leaderboard = aggregate_leaderboard(&scenarios);
    let point_roi_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                ScenarioSupportCategory::PointLikeRoi
            )
        })
        .map(|scenario| scenario.scenario_id.clone())
        .collect::<Vec<_>>();
    let region_roi_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                ScenarioSupportCategory::RegionRoi
            )
        })
        .map(|scenario| scenario.scenario_id.clone())
        .collect::<Vec<_>>();
    let negative_control_scenarios = scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                ScenarioSupportCategory::NegativeControl
            )
        })
        .map(|scenario| scenario.scenario_id.clone())
        .collect::<Vec<_>>();
    let remaining_blockers = vec![
        "The scenario suite is still synthetic and does not prove production-scene generalization."
            .to_string(),
        "The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than blanket win claims."
            .to_string(),
        "Cost accounting is architectural and CPU-side within the crate; it is not a measured GPU benchmark."
            .to_string(),
        "Point-like ROI scenarios remain mechanically useful but statistically weak, so aggregate claims must stay separated from region-ROI evidence."
            .to_string(),
    ];

    Ok(DemoASuiteMetrics {
        summary: DemoASuiteSummary {
            canonical_scenario_id: canonical.scenario_id.clone(),
            scenario_ids: scenarios
                .iter()
                .map(|scenario| scenario.scenario_id.clone())
                .collect(),
            point_roi_scenarios,
            region_roi_scenarios,
            negative_control_scenarios,
            baseline_ids,
            dsfb_ids,
            ablation_ids,
            primary_behavioral_result,
            secondary_behavioral_result,
            host_realistic_beats_fixed_alpha_scenarios,
            host_realistic_beats_strong_heuristic_scenarios,
            mixed_or_neutral_scenarios,
            remaining_blockers,
        },
        scenarios,
        ablations,
        aggregate_leaderboard,
    })
}

fn analyze_scenario(
    sequence: &SceneSequence,
    runs: &[RunAnalysisInput<'_>],
) -> Result<ScenarioReport> {
    if runs.is_empty() {
        return Err(Error::Message(format!(
            "scenario {} had no runs to analyze",
            sequence.scenario_id.as_str()
        )));
    }

    let non_roi_mask = invert_mask(&sequence.target_mask);
    let threshold = persistence_threshold(sequence);
    let mut reports = Vec::with_capacity(runs.len());
    for run in runs {
        reports.push(analyze_run(
            sequence,
            &sequence.target_mask,
            &non_roi_mask,
            threshold,
            run,
        ));
    }

    let fixed_alpha = reports
        .iter()
        .find(|run| run.summary.run_id == "fixed_alpha")
        .ok_or_else(|| Error::Message("fixed_alpha run missing from scenario".to_string()))?;
    let strong_heuristic = reports
        .iter()
        .find(|run| run.summary.run_id == "strong_heuristic")
        .ok_or_else(|| Error::Message("strong_heuristic run missing from scenario".to_string()))?;
    let host_realistic = reports
        .iter()
        .find(|run| run.summary.run_id == "dsfb_host_realistic")
        .ok_or_else(|| {
            Error::Message("dsfb_host_realistic run missing from scenario".to_string())
        })?;

    let headline = match sequence.expectation {
        ScenarioExpectation::BenefitExpected => format!(
            "{}: host-realistic DSFB changed cumulative ROI MAE from {:.5} (fixed alpha) and {:.5} (strong heuristic) to {:.5}.",
            sequence.scenario_title,
            fixed_alpha.summary.cumulative_roi_mae,
            strong_heuristic.summary.cumulative_roi_mae,
            host_realistic.summary.cumulative_roi_mae
        ),
        ScenarioExpectation::NeutralExpected => format!(
            "{}: neutral holdout with host-realistic non-ROI MAE {:.5} versus {:.5} for fixed alpha.",
            sequence.scenario_title,
            host_realistic.summary.average_non_roi_mae,
            fixed_alpha.summary.average_non_roi_mae
        ),
    };
    let bounded_or_neutral_note = match sequence.expectation {
        ScenarioExpectation::BenefitExpected => {
            if host_realistic.summary.cumulative_roi_mae
                > strong_heuristic.summary.cumulative_roi_mae
            {
                "Strong heuristic remains better on this scenario; the report surfaces that rather than hiding it."
                    .to_string()
            } else {
                "Host-realistic DSFB remains competitive without privileged visibility hints on this scenario."
                    .to_string()
            }
        }
        ScenarioExpectation::NeutralExpected => {
            "This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria."
                .to_string()
        }
    };

    Ok(ScenarioReport {
        scenario_id: sequence.scenario_id.as_str().to_string(),
        scenario_title: sequence.scenario_title.clone(),
        scenario_description: sequence.scenario_description.clone(),
        expectation: sequence.expectation,
        support_category: sequence.support_category,
        roi_note: sequence.roi_note.clone(),
        sampling_taxonomy: sequence.sampling_taxonomy.clone(),
        realism_stress: sequence.realism_stress,
        competitive_baseline_case: sequence.competitive_baseline_case,
        bounded_loss_disclosure: sequence.bounded_loss_disclosure,
        demo_b_taxonomy: sequence.demo_b_taxonomy.clone(),
        target_label: sequence.target_label.clone(),
        onset_frame: sequence.onset_frame,
        target_pixels: sequence.target_mask.iter().filter(|value| **value).count(),
        target_area_fraction: sequence.target_mask.iter().filter(|value| **value).count() as f32
            / (sequence.config.width * sequence.config.height).max(1) as f32,
        persistence_threshold: threshold,
        headline,
        bounded_or_neutral_note,
        host_realistic_vs_fixed_alpha_cumulative_roi_gain: fixed_alpha.summary.cumulative_roi_mae
            - host_realistic.summary.cumulative_roi_mae,
        host_realistic_vs_strong_heuristic_cumulative_roi_gain: strong_heuristic
            .summary
            .cumulative_roi_mae
            - host_realistic.summary.cumulative_roi_mae,
        host_realistic_non_roi_penalty_vs_fixed_alpha: host_realistic.summary.average_non_roi_mae
            - fixed_alpha.summary.average_non_roi_mae,
        host_realistic_non_roi_penalty_vs_strong_heuristic: host_realistic
            .summary
            .average_non_roi_mae
            - strong_heuristic.summary.average_non_roi_mae,
        host_realistic_non_roi_penalty_ratio_vs_strong_heuristic: ratio_or_identity(
            host_realistic.summary.average_non_roi_mae,
            strong_heuristic.summary.average_non_roi_mae,
        ),
        neighborhood_clamp_roi_trigger_mean: runs
            .iter()
            .find(|run| run.id == "neighborhood_clamp")
            .map(|run| {
                mean_field_over_mask_range(
                    run.response_frames,
                    &sequence.target_mask,
                    sequence.onset_frame,
                )
            })
            .unwrap_or(0.0),
        neighborhood_clamp_roi_silent_fraction: runs
            .iter()
            .find(|run| run.id == "neighborhood_clamp")
            .map(|run| {
                fraction_field_values_below_range(
                    run.response_frames,
                    &sequence.target_mask,
                    sequence.onset_frame,
                    LOW_RESPONSE_THRESHOLD,
                )
            })
            .unwrap_or(0.0),
        neighborhood_clamp_history_inside_hull_fraction: runs
            .iter()
            .find(|run| run.id == "neighborhood_clamp")
            .map(|run| {
                fraction_field_values_below_range(
                    run.response_frames,
                    &sequence.target_mask,
                    sequence.onset_frame,
                    1.0e-4,
                )
            })
            .unwrap_or(0.0),
        runs: reports,
    })
}

fn analyze_run(
    sequence: &SceneSequence,
    target_mask: &[bool],
    non_roi_mask: &[bool],
    threshold: f32,
    run: &RunAnalysisInput<'_>,
) -> ScenarioRunReport {
    let onset = sequence
        .onset_frame
        .min(sequence.frames.len().saturating_sub(1));
    let mut frame_metrics = Vec::with_capacity(sequence.frames.len());
    let mut cumulative_roi_mae = 0.0;
    let mut cumulative_non_roi_mae = 0.0;
    let mut average_overall_mae = 0.0;
    let mut average_overall_rmse = 0.0;
    let mut average_roi_mae = 0.0;
    let mut average_non_roi_mae = 0.0;
    let mut average_non_roi_rmse = 0.0;
    let mut peak_roi_mae = f32::NEG_INFINITY;
    let mut peak_roi_mae_frame = onset;
    let mut response_pixels = 0usize;
    let total_pixels = sequence.frames.len() * sequence.config.width * sequence.config.height;

    for frame_index in 0..sequence.frames.len() {
        let gt = &sequence.frames[frame_index].ground_truth;
        let resolved = &run.resolved_frames[frame_index];
        let alpha = &run.alpha_frames[frame_index];
        let response = &run.response_frames[frame_index];
        let trust = run.trust_frames.map(|fields| &fields[frame_index]);

        let overall_mae = mean_abs_error(resolved, gt);
        let overall_rmse = rmse(resolved, gt, None);
        let roi_mae = mean_abs_error_over_mask(resolved, gt, target_mask);
        let roi_rmse = rmse(resolved, gt, Some(target_mask));
        let non_roi_mae = mean_abs_error_over_mask(resolved, gt, non_roi_mask);
        let non_roi_rmse = rmse(resolved, gt, Some(non_roi_mask));
        let alpha_mean = alpha.mean();
        let alpha_roi_mean = alpha.mean_over_mask(target_mask);
        let alpha_non_roi_mean = alpha.mean_over_mask(non_roi_mask);
        let response_mean = response.mean();
        let response_roi_mean = response.mean_over_mask(target_mask);
        let response_non_roi_mean = response.mean_over_mask(non_roi_mask);
        let trust_mean = trust.map(ScalarField::mean);
        let trust_roi_mean = trust.map(|field| field.mean_over_mask(target_mask));
        let trust_non_roi_mean = trust.map(|field| field.mean_over_mask(non_roi_mask));

        average_overall_mae += overall_mae;
        average_overall_rmse += overall_rmse;
        average_roi_mae += roi_mae;
        average_non_roi_mae += non_roi_mae;
        average_non_roi_rmse += non_roi_rmse;
        cumulative_roi_mae += roi_mae;
        cumulative_non_roi_mae += non_roi_mae;
        response_pixels += count_field_above(response, LOW_RESPONSE_THRESHOLD);

        if roi_mae > peak_roi_mae {
            peak_roi_mae = roi_mae;
            peak_roi_mae_frame = frame_index;
        }

        frame_metrics.push(RunFrameMetrics {
            frame_index,
            overall_mae,
            overall_rmse,
            roi_mae,
            roi_rmse,
            non_roi_mae,
            non_roi_rmse,
            alpha_mean,
            alpha_roi_mean,
            alpha_non_roi_mean,
            response_mean,
            response_roi_mean,
            response_non_roi_mean,
            trust_mean,
            trust_roi_mean,
            trust_non_roi_mean,
        });
    }

    let frame_count = sequence.frames.len().max(1) as f32;
    let ghost_persistence_frames =
        compute_ghost_persistence(&frame_metrics, onset, threshold, |frame| frame.roi_mae);
    let onset_response_latency_frames =
        first_frame_at_or_above(&frame_metrics, onset, LOW_RESPONSE_THRESHOLD, |frame| {
            frame.response_roi_mean
        })
        .map(|frame| frame.saturating_sub(onset));
    let false_positive_response_rate = frame_metrics
        .iter()
        .skip(onset)
        .map(|frame| frame.response_non_roi_mean)
        .sum::<f32>()
        / (frame_metrics.len().saturating_sub(onset).max(1) as f32);
    let intervention_sparsity = response_pixels as f32 / total_pixels.max(1) as f32;
    let onset_alpha_values = run.alpha_frames[onset].values().to_vec();
    let onset_alpha_p90 = percentile(&onset_alpha_values, 0.90);
    let onset_alpha_max = onset_alpha_values.iter().copied().fold(0.0f32, f32::max);
    let temporal_variance_non_roi =
        temporal_variance_non_roi(sequence, run.resolved_frames, non_roi_mask);
    let alpha_temporal_delta_mean = temporal_scalar_delta_mean(run.alpha_frames);
    let response_temporal_delta_mean = temporal_scalar_delta_mean(run.response_frames);
    let (
        trust_error_rank_correlation,
        trust_rank_correlation_is_degenerate,
        trust_calibration_bins,
        trust_histogram,
        trust_occupied_bin_count,
        trust_entropy_bits,
        trust_discreteness_score,
        trust_effective_level_count,
        trust_operating_mode,
    ) = if let Some(fields) = run.trust_frames {
        let histogram = histogram_bins(&fields[onset], 10);
        let occupied_bin_count = histogram.iter().filter(|bin| bin.sample_count > 0).count();
        let entropy_bits = histogram_entropy_bits(&histogram);
        let discreteness_score =
            entropy_bits.map(|entropy| trust_discreteness_score(histogram.len(), entropy));
        let effective_level_count = entropy_bits.map(|entropy| {
            2.0f32
                .powf(entropy)
                .round()
                .clamp(1.0, histogram.len() as f32) as usize
        });
        let correlation = frame_spearman_correlation(fields, &frame_metrics, onset);
        let correlation_degenerate =
            trust_rank_correlation_is_degenerate(fields, onset, occupied_bin_count, entropy_bits);
        (
            Some(correlation),
            correlation_degenerate,
            calibration_bins(
                &fields[onset],
                &run.resolved_frames[onset],
                &sequence.frames[onset].ground_truth,
            ),
            histogram.clone(),
            occupied_bin_count,
            entropy_bits,
            discreteness_score,
            effective_level_count,
            classify_trust_operating_mode(occupied_bin_count, entropy_bits, discreteness_score),
        )
    } else {
        (
            None,
            false,
            Vec::new(),
            Vec::new(),
            0,
            None,
            None,
            None,
            None,
        )
    };

    ScenarioRunReport {
        summary: RunSummary {
            run_id: run.id.to_string(),
            label: run.label.to_string(),
            category: run.category.to_string(),
            peak_roi_mae,
            peak_roi_mae_frame,
            cumulative_roi_mae,
            cumulative_non_roi_mae,
            average_overall_mae: average_overall_mae / frame_count,
            average_overall_rmse: average_overall_rmse / frame_count,
            average_roi_mae: average_roi_mae / frame_count,
            average_non_roi_mae: average_non_roi_mae / frame_count,
            average_non_roi_rmse: average_non_roi_rmse / frame_count,
            ghost_persistence_frames,
            onset_response_latency_frames,
            false_positive_response_rate,
            intervention_sparsity,
            mean_alpha: run.alpha_frames.iter().map(ScalarField::mean).sum::<f32>() / frame_count,
            onset_alpha_p90,
            onset_alpha_max,
            alpha_temporal_delta_mean,
            response_temporal_delta_mean,
            temporal_variance_non_roi,
            trust_error_rank_correlation,
            trust_rank_correlation_is_degenerate,
            trust_calibration_bins,
            trust_histogram,
            trust_occupied_bin_count,
            trust_entropy_bits,
            trust_discreteness_score,
            trust_effective_level_count,
            trust_operating_mode,
        },
        frame_metrics,
    }
}

fn persistence_threshold(sequence: &SceneSequence) -> f32 {
    if sequence.onset_frame == 0 {
        return 0.02;
    }
    let previous = &sequence.frames[sequence.onset_frame - 1].ground_truth;
    let current = &sequence.frames[sequence.onset_frame].ground_truth;
    (mean_abs_error_over_mask(previous, current, &sequence.target_mask) * 0.15).max(0.02)
}

fn rmse(frame_a: &ImageFrame, frame_b: &ImageFrame, mask: Option<&[bool]>) -> f32 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for y in 0..frame_a.height() {
        for x in 0..frame_a.width() {
            let index = y * frame_a.width() + x;
            if mask.map(|values| values[index]).unwrap_or(true) {
                let diff = frame_a.get(x, y).abs_diff(frame_b.get(x, y));
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

fn invert_mask(mask: &[bool]) -> Vec<bool> {
    mask.iter().map(|value| !value).collect()
}

fn compute_ghost_persistence(
    frame_metrics: &[RunFrameMetrics],
    onset: usize,
    threshold: f32,
    metric: impl Fn(&RunFrameMetrics) -> f32,
) -> usize {
    frame_metrics
        .iter()
        .skip(onset)
        .filter(|frame| metric(frame) > threshold)
        .count()
}

fn first_frame_at_or_above(
    frame_metrics: &[RunFrameMetrics],
    start: usize,
    threshold: f32,
    metric: impl Fn(&RunFrameMetrics) -> f32,
) -> Option<usize> {
    frame_metrics
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, frame)| (metric(frame) >= threshold).then_some(index))
}

fn percentile(values: &[f32], quantile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    let index = ((sorted.len() - 1) as f32 * quantile.clamp(0.0, 1.0)).round() as usize;
    sorted[index]
}

fn temporal_variance_non_roi(
    sequence: &SceneSequence,
    resolved_frames: &[ImageFrame],
    non_roi_mask: &[bool],
) -> f32 {
    let width = sequence.config.width;
    let height = sequence.config.height;
    let frame_count = resolved_frames.len().max(1) as f32;
    let mut total_variance = 0.0f32;
    let mut pixel_count = 0usize;

    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            if !non_roi_mask[index] {
                continue;
            }
            let mean = resolved_frames
                .iter()
                .map(|frame| frame.get(x, y).luma())
                .sum::<f32>()
                / frame_count;
            let variance = resolved_frames
                .iter()
                .map(|frame| {
                    let diff = frame.get(x, y).luma() - mean;
                    diff * diff
                })
                .sum::<f32>()
                / frame_count;
            total_variance += variance;
            pixel_count += 1;
        }
    }

    if pixel_count == 0 {
        0.0
    } else {
        total_variance / pixel_count as f32
    }
}

fn temporal_scalar_delta_mean(fields: &[ScalarField]) -> f32 {
    if fields.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for window in fields.windows(2) {
        total += mean_abs_scalar_delta(&window[0], &window[1]);
    }
    total / (fields.len() - 1) as f32
}

fn mean_abs_scalar_delta(left: &ScalarField, right: &ScalarField) -> f32 {
    if left.values().is_empty() || right.values().is_empty() {
        return 0.0;
    }
    left.values()
        .iter()
        .zip(right.values().iter())
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>()
        / left.values().len().min(right.values().len()).max(1) as f32
}

fn frame_spearman_correlation(
    trust_frames: &[ScalarField],
    frame_metrics: &[RunFrameMetrics],
    onset: usize,
) -> f32 {
    let trust_values = trust_frames
        .iter()
        .skip(onset)
        .map(|field| field.mean())
        .collect::<Vec<_>>();
    let error_values = frame_metrics
        .iter()
        .skip(onset)
        .map(|frame| frame.roi_mae)
        .collect::<Vec<_>>();
    spearman(&trust_values, &error_values)
}

fn histogram_bins(field: &ScalarField, bin_count: usize) -> Vec<HistogramBin> {
    let safe_bin_count = bin_count.max(1);
    let mut counts = vec![0usize; safe_bin_count];
    for value in field.values().iter().copied() {
        let index = ((value.clamp(0.0, 1.0) * safe_bin_count as f32).floor() as usize)
            .min(safe_bin_count - 1);
        counts[index] += 1;
    }
    counts
        .into_iter()
        .enumerate()
        .map(|(index, sample_count)| HistogramBin {
            lower: index as f32 / safe_bin_count as f32,
            upper: (index + 1) as f32 / safe_bin_count as f32,
            sample_count,
        })
        .collect()
}

fn histogram_entropy_bits(histogram: &[HistogramBin]) -> Option<f32> {
    let total = histogram.iter().map(|bin| bin.sample_count).sum::<usize>();
    if total == 0 {
        return None;
    }
    let total_f = total as f32;
    Some(
        histogram
            .iter()
            .filter(|bin| bin.sample_count > 0)
            .map(|bin| {
                let p = bin.sample_count as f32 / total_f;
                -p * p.log2()
            })
            .sum::<f32>(),
    )
}

fn trust_discreteness_score(bin_count: usize, entropy_bits: f32) -> f32 {
    let max_entropy = (bin_count.max(2) as f32).log2().max(f32::EPSILON);
    (1.0 - entropy_bits / max_entropy).clamp(0.0, 1.0)
}

fn trust_rank_correlation_is_degenerate(
    trust_frames: &[ScalarField],
    onset: usize,
    occupied_bin_count: usize,
    entropy_bits: Option<f32>,
) -> bool {
    let Some(onset_field) = trust_frames.get(onset) else {
        return true;
    };
    let (min_value, max_value) = onset_field.values().iter().copied().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(min_value, max_value), value| (min_value.min(value), max_value.max(value)),
    );
    let entropy_low = entropy_bits.unwrap_or(0.0) < 1.0;
    let near_flat = (max_value - min_value).abs() < 0.15;
    let too_few_post_onset_frames = trust_frames.len().saturating_sub(onset) < 4;
    occupied_bin_count < 4 || entropy_low || near_flat || too_few_post_onset_frames
}

fn classify_trust_operating_mode(
    occupied_bin_count: usize,
    entropy_bits: Option<f32>,
    discreteness_score: Option<f32>,
) -> Option<TrustOperatingMode> {
    let entropy_bits = entropy_bits?;
    let discreteness_score = discreteness_score?;
    Some(
        if occupied_bin_count <= 3 || discreteness_score >= 0.72 || entropy_bits < 1.0 {
            TrustOperatingMode::NearBinaryGate
        } else if occupied_bin_count <= 5 || discreteness_score >= 0.42 || entropy_bits < 1.8 {
            TrustOperatingMode::WeaklyGraded
        } else {
            TrustOperatingMode::StronglyGraded
        },
    )
}

fn calibration_bins(
    trust: &ScalarField,
    resolved: &ImageFrame,
    ground_truth: &ImageFrame,
) -> Vec<CalibrationBin> {
    let mut bins = vec![
        (0.0f32, 0.2f32, 0usize, 0.0f32, 0.0f32),
        (0.2, 0.4, 0, 0.0, 0.0),
        (0.4, 0.6, 0, 0.0, 0.0),
        (0.6, 0.8, 0, 0.0, 0.0),
        (0.8, 1.01, 0, 0.0, 0.0),
    ];
    for y in 0..trust.height() {
        for x in 0..trust.width() {
            let trust_value = trust.get(x, y);
            let error_value = resolved.get(x, y).abs_diff(ground_truth.get(x, y));
            for bin in &mut bins {
                if trust_value >= bin.0 && trust_value < bin.1 {
                    bin.2 += 1;
                    bin.3 += trust_value;
                    bin.4 += error_value;
                    break;
                }
            }
        }
    }

    bins.into_iter()
        .map(
            |(lower, upper, sample_count, trust_sum, error_sum)| CalibrationBin {
                lower,
                upper: upper.min(1.0),
                sample_count,
                mean_trust: if sample_count == 0 {
                    0.0
                } else {
                    trust_sum / sample_count as f32
                },
                mean_error: if sample_count == 0 {
                    0.0
                } else {
                    error_sum / sample_count as f32
                },
            },
        )
        .collect()
}

fn spearman(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let left_ranks = ranks(left);
    let right_ranks = ranks(right);
    pearson(&left_ranks, &right_ranks)
}

fn ranks(values: &[f32]) -> Vec<f32> {
    let mut indexed = values.iter().copied().enumerate().collect::<Vec<_>>();
    indexed.sort_by(|left, right| left.1.total_cmp(&right.1));
    let mut result = vec![0.0; values.len()];
    for (rank, (index, _)) in indexed.into_iter().enumerate() {
        result[index] = rank as f32;
    }
    result
}

fn pearson(left: &[f32], right: &[f32]) -> f32 {
    let n = left.len().max(1) as f32;
    let mean_left = left.iter().sum::<f32>() / n;
    let mean_right = right.iter().sum::<f32>() / n;
    let mut numerator = 0.0;
    let mut denom_left = 0.0;
    let mut denom_right = 0.0;
    for (l, r) in left.iter().copied().zip(right.iter().copied()) {
        let dl = l - mean_left;
        let dr = r - mean_right;
        numerator += dl * dr;
        denom_left += dl * dl;
        denom_right += dr * dr;
    }
    let denom = (denom_left * denom_right).sqrt().max(f32::EPSILON);
    numerator / denom
}

fn count_field_above(field: &ScalarField, threshold: f32) -> usize {
    field
        .values()
        .iter()
        .filter(|value| **value >= threshold)
        .count()
}

fn mean_field_over_mask_range(fields: &[ScalarField], mask: &[bool], start: usize) -> f32 {
    if fields.is_empty() || start >= fields.len() {
        return 0.0;
    }
    fields
        .iter()
        .skip(start)
        .map(|field| field.mean_over_mask(mask))
        .sum::<f32>()
        / fields.len().saturating_sub(start).max(1) as f32
}

fn fraction_field_values_below_range(
    fields: &[ScalarField],
    mask: &[bool],
    start: usize,
    threshold: f32,
) -> f32 {
    if fields.is_empty() || start >= fields.len() {
        return 0.0;
    }
    let mut total = 0usize;
    let mut hits = 0usize;
    for field in fields.iter().skip(start) {
        for (value, include) in field.values().iter().zip(mask.iter().copied()) {
            if !include {
                continue;
            }
            total += 1;
            if *value <= threshold {
                hits += 1;
            }
        }
    }
    if total == 0 {
        0.0
    } else {
        hits as f32 / total as f32
    }
}

fn ratio_or_identity(numerator: f32, denominator: f32) -> f32 {
    if denominator.abs() <= f32::EPSILON {
        if numerator.abs() <= f32::EPSILON {
            1.0
        } else {
            numerator
        }
    } else {
        numerator / denominator
    }
}

fn aggregate_leaderboard(scenarios: &[ScenarioReport]) -> Vec<AggregateRunScore> {
    let mut entries = std::collections::BTreeMap::<String, AggregateRunScore>::new();
    for scenario in scenarios {
        let mut ranked = scenario
            .runs
            .iter()
            .map(|run| {
                let score = match scenario.expectation {
                    ScenarioExpectation::BenefitExpected => run.summary.cumulative_roi_mae,
                    ScenarioExpectation::NeutralExpected => {
                        run.summary.average_non_roi_mae
                            + 0.5 * run.summary.false_positive_response_rate
                    }
                };
                (score, run)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| left.0.total_cmp(&right.0));

        for (rank, (_, run)) in ranked.into_iter().enumerate() {
            let entry = entries
                .entry(run.summary.run_id.clone())
                .or_insert_with(|| AggregateRunScore {
                    run_id: run.summary.run_id.clone(),
                    label: run.summary.label.clone(),
                    category: run.summary.category.clone(),
                    mean_rank: 0.0,
                    mean_cumulative_roi_mae: 0.0,
                    mean_non_roi_mae: 0.0,
                    mean_false_positive_response_rate: 0.0,
                    benefit_scenarios_won: 0,
                });
            entry.mean_rank += rank as f32;
            entry.mean_cumulative_roi_mae += run.summary.cumulative_roi_mae;
            entry.mean_non_roi_mae += run.summary.average_non_roi_mae;
            entry.mean_false_positive_response_rate += run.summary.false_positive_response_rate;
            if rank == 0 && matches!(scenario.expectation, ScenarioExpectation::BenefitExpected) {
                entry.benefit_scenarios_won += 1;
            }
        }
    }

    let scenario_count = scenarios.len().max(1) as f32;
    let mut values = entries
        .into_values()
        .map(|mut entry| {
            entry.mean_rank /= scenario_count;
            entry.mean_cumulative_roi_mae /= scenario_count;
            entry.mean_non_roi_mae /= scenario_count;
            entry.mean_false_positive_response_rate /= scenario_count;
            entry
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| left.mean_rank.total_cmp(&right.mean_rank));
    values
}

fn find_run<'a>(scenario: &'a ScenarioReport, run_id: &str) -> Result<&'a RunSummary> {
    scenario
        .runs
        .iter()
        .find(|run| run.summary.run_id == run_id)
        .map(|run| &run.summary)
        .ok_or_else(|| Error::Message(format!("run {run_id} missing from scenario report")))
}
