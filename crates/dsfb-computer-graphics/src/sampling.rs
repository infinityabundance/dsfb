use serde::Serialize;

use crate::config::DemoConfig;
use crate::dsfb::DsfbRun;
use crate::error::{Error, Result};
use crate::frame::{mean_abs_error, Color, ImageFrame, ScalarField};
use crate::scene::{Rect, ScenarioExpectation, ScenarioId, SceneFrame, SceneSequence};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum AllocationPolicyId {
    Uniform,
    EdgeGuided,
    ResidualGuided,
    ContrastGuided,
    VarianceGuided,
    CombinedHeuristic,
    NativeTrust,
    ImportedTrust,
    HybridTrustVariance,
}

impl AllocationPolicyId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::EdgeGuided => "edge_guided",
            Self::ResidualGuided => "residual_guided",
            Self::ContrastGuided => "contrast_guided",
            Self::VarianceGuided => "variance_guided",
            Self::CombinedHeuristic => "combined_heuristic",
            Self::NativeTrust => "native_trust",
            Self::ImportedTrust => "imported_trust",
            Self::HybridTrustVariance => "hybrid_trust_variance",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::EdgeGuided => "Edge-guided",
            Self::ResidualGuided => "Residual-guided",
            Self::ContrastGuided => "Contrast-guided",
            Self::VarianceGuided => "Variance-guided",
            Self::CombinedHeuristic => "Combined heuristic",
            Self::NativeTrust => "Native trust",
            Self::ImportedTrust => "Imported trust",
            Self::HybridTrustVariance => "Hybrid trust + variance",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BudgetCurvePoint {
    pub average_spp: f32,
    pub roi_mae: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct BudgetCurve {
    pub scenario_id: String,
    pub policy_id: String,
    pub points: Vec<BudgetCurvePoint>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoBPolicyMetrics {
    pub policy_id: String,
    pub label: String,
    pub total_samples: usize,
    pub overall_mae: f32,
    pub overall_rmse: f32,
    pub roi_mae: f32,
    pub roi_rmse: f32,
    pub non_roi_mae: f32,
    pub non_roi_rmse: f32,
    pub roi_mean_spp: f32,
    pub non_roi_mean_spp: f32,
    pub max_spp: usize,
    pub allocation_concentration: f32,
    pub extra_roi_samples_vs_uniform: f32,
    pub roi_error_reduction_per_extra_roi_sample: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoBScenarioReport {
    pub scenario_id: String,
    pub scenario_title: String,
    pub expectation: ScenarioExpectation,
    pub onset_frame: usize,
    pub target_label: String,
    pub target_pixels: usize,
    pub policies: Vec<DemoBPolicyMetrics>,
    pub headline: String,
    pub bounded_note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoBSummary {
    pub scenario_ids: Vec<String>,
    pub policy_ids: Vec<String>,
    pub primary_behavioral_result: String,
    pub imported_trust_beats_uniform_scenarios: usize,
    pub imported_trust_beats_combined_heuristic_scenarios: usize,
    pub neutral_or_mixed_scenarios: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DemoBSuiteMetrics {
    pub summary: DemoBSummary,
    pub scenarios: Vec<DemoBScenarioReport>,
    pub budget_efficiency_curves: Vec<BudgetCurve>,
}

#[derive(Clone, Debug)]
pub struct DemoBPolicyRun {
    pub policy_id: AllocationPolicyId,
    pub frame: ImageFrame,
    pub error: ScalarField,
    pub spp: ScalarField,
    pub metrics: DemoBPolicyMetrics,
}

#[derive(Clone, Debug)]
pub struct DemoBScenarioRun {
    pub reference_frame: ImageFrame,
    pub policy_runs: Vec<DemoBPolicyRun>,
    pub target_bbox: crate::frame::BoundingBox,
}

struct PolicyMetricsContext<'a> {
    counts: &'a [usize],
    frame: &'a ImageFrame,
    reference_frame: &'a ImageFrame,
    _error: &'a ScalarField,
    target_mask: &'a [bool],
    extra_roi_samples_vs_uniform: f32,
    uniform_roi_mae: f32,
}

pub fn run_demo_b_suite(
    config: &DemoConfig,
    scenarios: &[(SceneSequence, DsfbRun)],
) -> Result<(DemoBSuiteMetrics, Vec<(String, DemoBScenarioRun)>)> {
    if scenarios.is_empty() {
        return Err(Error::Message(
            "Demo B suite requires at least one scenario".to_string(),
        ));
    }

    let mut reports = Vec::with_capacity(scenarios.len());
    let mut runs = Vec::with_capacity(scenarios.len());
    let mut curves = Vec::new();

    for (sequence, dsfb_host_realistic) in scenarios {
        let (report, scenario_run, scenario_curves) =
            run_demo_b_scenario(config, sequence, dsfb_host_realistic)?;
        reports.push(report);
        runs.push((sequence.scenario_id.as_str().to_string(), scenario_run));
        curves.extend(scenario_curves);
    }

    let canonical = &reports[0];
    let canonical_uniform = find_policy(canonical, AllocationPolicyId::Uniform)?;
    let canonical_imported = find_policy(canonical, AllocationPolicyId::ImportedTrust)?;

    let imported_trust_beats_uniform_scenarios = reports
        .iter()
        .filter(|report| {
            let uniform = report
                .policies
                .iter()
                .find(|policy| policy.policy_id == AllocationPolicyId::Uniform.as_str());
            let trust = report
                .policies
                .iter()
                .find(|policy| policy.policy_id == AllocationPolicyId::ImportedTrust.as_str());
            match (uniform, trust) {
                (Some(uniform), Some(trust)) => trust.roi_mae + 1e-6 < uniform.roi_mae,
                _ => false,
            }
        })
        .count();
    let imported_trust_beats_combined_heuristic_scenarios = reports
        .iter()
        .filter(|report| {
            let combined = report
                .policies
                .iter()
                .find(|policy| policy.policy_id == AllocationPolicyId::CombinedHeuristic.as_str());
            let trust = report
                .policies
                .iter()
                .find(|policy| policy.policy_id == AllocationPolicyId::ImportedTrust.as_str());
            match (combined, trust) {
                (Some(combined), Some(trust)) => trust.roi_mae + 1e-6 < combined.roi_mae,
                _ => false,
            }
        })
        .count();

    let neutral_or_mixed_scenarios = reports
        .iter()
        .filter(|report| {
            matches!(report.expectation, ScenarioExpectation::NeutralExpected)
                || report
                    .policies
                    .iter()
                    .find(|policy| policy.policy_id == AllocationPolicyId::ImportedTrust.as_str())
                    .zip(report.policies.iter().find(|policy| {
                        policy.policy_id == AllocationPolicyId::CombinedHeuristic.as_str()
                    }))
                    .map(|(trust, combined)| trust.roi_mae > combined.roi_mae)
                    .unwrap_or(false)
        })
        .map(|report| report.scenario_id.clone())
        .collect::<Vec<_>>();

    Ok((
        DemoBSuiteMetrics {
            summary: DemoBSummary {
                scenario_ids: reports
                    .iter()
                    .map(|report| report.scenario_id.clone())
                    .collect(),
                policy_ids: [
                    AllocationPolicyId::Uniform,
                    AllocationPolicyId::EdgeGuided,
                    AllocationPolicyId::ResidualGuided,
                    AllocationPolicyId::ContrastGuided,
                    AllocationPolicyId::VarianceGuided,
                    AllocationPolicyId::CombinedHeuristic,
                    AllocationPolicyId::NativeTrust,
                    AllocationPolicyId::ImportedTrust,
                    AllocationPolicyId::HybridTrustVariance,
                ]
                .iter()
                .map(|policy| policy.as_str().to_string())
                .collect(),
                primary_behavioral_result: format!(
                    "On the canonical sampling scenario, imported trust reduced ROI MAE from {:.5} for uniform allocation to {:.5} under the same total budget.",
                    canonical_uniform.roi_mae, canonical_imported.roi_mae
                ),
                imported_trust_beats_uniform_scenarios,
                imported_trust_beats_combined_heuristic_scenarios,
                neutral_or_mixed_scenarios,
            },
            scenarios: reports,
            budget_efficiency_curves: curves,
        },
        runs,
    ))
}

fn run_demo_b_scenario(
    config: &DemoConfig,
    sequence: &SceneSequence,
    dsfb_host_realistic: &DsfbRun,
) -> Result<(DemoBScenarioReport, DemoBScenarioRun, Vec<BudgetCurve>)> {
    let onset = sequence
        .onset_frame
        .min(sequence.frames.len().saturating_sub(1));
    let scene_frame = &sequence.frames[onset];
    let width = sequence.config.width;
    let height = sequence.config.height;
    let total_pixels = width * height;
    let uniform_total_samples = config.demo_b_uniform_spp * total_pixels;
    let min_total = config.demo_b_min_spp * total_pixels;
    let max_total = config.demo_b_max_spp * total_pixels;
    if uniform_total_samples < min_total || uniform_total_samples > max_total {
        return Err(Error::Message(
            "Demo B uniform total sample budget is incompatible with the min/max spp bounds"
                .to_string(),
        ));
    }

    let reference_counts = vec![config.demo_b_reference_spp; total_pixels];
    let uniform_counts = vec![config.demo_b_uniform_spp; total_pixels];
    let reference_frame = render_with_counts(sequence, scene_frame, &reference_counts);
    let pilot_a = render_with_counts(sequence, scene_frame, &vec![1usize; total_pixels]);
    let pilot_b = render_with_offset_counts(sequence, scene_frame, &vec![1usize; total_pixels], 17);
    let target_bbox = crate::frame::bounding_box_from_mask(&sequence.target_mask, width, height)
        .ok_or_else(|| Error::Message("Demo B target mask was empty".to_string()))?;

    let imported_trust = invert_trust(&dsfb_host_realistic.supervision_frames[onset].trust);
    let edge_difficulty = gradient_field(&pilot_a);
    let residual_difficulty = residual_proxy_field(&pilot_a);
    let contrast_difficulty = local_contrast_field(&pilot_a);
    let variance_difficulty = pilot_variance_field(&pilot_a, &pilot_b);
    let combined_difficulty = combine_fields(
        &[
            (&edge_difficulty, 0.35),
            (&residual_difficulty, 0.25),
            (&contrast_difficulty, 0.25),
            (&variance_difficulty, 0.15),
        ],
        width,
        height,
    );
    let native_trust_difficulty = combine_fields(
        &[
            (&edge_difficulty, 0.18),
            (&residual_difficulty, 0.28),
            (&contrast_difficulty, 0.24),
            (&variance_difficulty, 0.30),
        ],
        width,
        height,
    );
    let hybrid_difficulty = combine_fields(
        &[(&imported_trust, 0.55), (&variance_difficulty, 0.45)],
        width,
        height,
    );

    let policies = [
        (AllocationPolicyId::Uniform, None),
        (AllocationPolicyId::EdgeGuided, Some(&edge_difficulty)),
        (
            AllocationPolicyId::ResidualGuided,
            Some(&residual_difficulty),
        ),
        (
            AllocationPolicyId::ContrastGuided,
            Some(&contrast_difficulty),
        ),
        (
            AllocationPolicyId::VarianceGuided,
            Some(&variance_difficulty),
        ),
        (
            AllocationPolicyId::CombinedHeuristic,
            Some(&combined_difficulty),
        ),
        (
            AllocationPolicyId::NativeTrust,
            Some(&native_trust_difficulty),
        ),
        (AllocationPolicyId::ImportedTrust, Some(&imported_trust)),
        (
            AllocationPolicyId::HybridTrustVariance,
            Some(&hybrid_difficulty),
        ),
    ];

    let uniform_frame = render_with_counts(sequence, scene_frame, &uniform_counts);
    let uniform_error = build_error_field(&uniform_frame, &reference_frame);
    let uniform_metrics = policy_metrics(
        AllocationPolicyId::Uniform,
        PolicyMetricsContext {
            counts: &uniform_counts,
            frame: &uniform_frame,
            reference_frame: &reference_frame,
            _error: &uniform_error,
            target_mask: &sequence.target_mask,
            extra_roi_samples_vs_uniform: 0.0,
            uniform_roi_mae: 0.0,
        },
    );
    let mut policy_runs = vec![DemoBPolicyRun {
        policy_id: AllocationPolicyId::Uniform,
        frame: uniform_frame,
        error: uniform_error,
        spp: build_count_field(&uniform_counts, width, height),
        metrics: uniform_metrics.clone(),
    }];

    for (policy_id, field) in policies.iter().skip(1) {
        let difficulty = field.expect("guided policies require a difficulty field");
        let counts = guided_allocation(
            difficulty,
            uniform_total_samples,
            config.demo_b_min_spp,
            config.demo_b_max_spp,
        )?;
        let frame = render_with_counts(sequence, scene_frame, &counts);
        let error = build_error_field(&frame, &reference_frame);
        let roi_mean_spp = mean_count_over_mask(&counts, &sequence.target_mask);
        let metrics = policy_metrics(
            *policy_id,
            PolicyMetricsContext {
                counts: &counts,
                frame: &frame,
                reference_frame: &reference_frame,
                _error: &error,
                target_mask: &sequence.target_mask,
                extra_roi_samples_vs_uniform: roi_mean_spp - uniform_metrics.roi_mean_spp,
                uniform_roi_mae: uniform_metrics.roi_mae,
            },
        );
        policy_runs.push(DemoBPolicyRun {
            policy_id: *policy_id,
            frame,
            error,
            spp: build_count_field(&counts, width, height),
            metrics,
        });
    }

    for run in &policy_runs {
        let total = run.metrics.total_samples;
        if total != uniform_total_samples {
            return Err(Error::Message(format!(
                "policy {} violated fixed-budget fairness: expected {}, got {}",
                run.policy_id.as_str(),
                uniform_total_samples,
                total
            )));
        }
    }

    let budget_levels = [1.0f32, config.demo_b_uniform_spp as f32, 4.0f32, 8.0f32];
    let mut curves = Vec::new();
    for policy_id in [
        AllocationPolicyId::Uniform,
        AllocationPolicyId::CombinedHeuristic,
        AllocationPolicyId::NativeTrust,
        AllocationPolicyId::ImportedTrust,
        AllocationPolicyId::HybridTrustVariance,
    ] {
        let mut points = Vec::new();
        for average_spp in budget_levels {
            let total_samples = (average_spp * total_pixels as f32).round() as usize;
            let counts = match policy_id {
                AllocationPolicyId::Uniform => {
                    vec![average_spp.round().max(1.0) as usize; total_pixels]
                }
                AllocationPolicyId::CombinedHeuristic => guided_allocation(
                    &combined_difficulty,
                    total_samples,
                    1,
                    config.demo_b_max_spp,
                )?,
                AllocationPolicyId::NativeTrust => guided_allocation(
                    &native_trust_difficulty,
                    total_samples,
                    1,
                    config.demo_b_max_spp,
                )?,
                AllocationPolicyId::ImportedTrust => {
                    guided_allocation(&imported_trust, total_samples, 1, config.demo_b_max_spp)?
                }
                AllocationPolicyId::HybridTrustVariance => {
                    guided_allocation(&hybrid_difficulty, total_samples, 1, config.demo_b_max_spp)?
                }
                _ => unreachable!(),
            };
            let frame = render_with_counts(sequence, scene_frame, &counts);
            points.push(BudgetCurvePoint {
                average_spp,
                roi_mae: mean_abs_error_over_mask(&frame, &reference_frame, &sequence.target_mask),
            });
        }
        curves.push(BudgetCurve {
            scenario_id: sequence.scenario_id.as_str().to_string(),
            policy_id: policy_id.as_str().to_string(),
            points,
        });
    }

    let imported_trust_metrics = policy_runs
        .iter()
        .find(|run| run.policy_id == AllocationPolicyId::ImportedTrust)
        .map(|run| &run.metrics)
        .ok_or_else(|| Error::Message("imported trust policy missing".to_string()))?;
    let combined_metrics = policy_runs
        .iter()
        .find(|run| run.policy_id == AllocationPolicyId::CombinedHeuristic)
        .map(|run| &run.metrics)
        .ok_or_else(|| Error::Message("combined heuristic policy missing".to_string()))?;

    let headline = format!(
        "{}: imported-trust ROI MAE {:.5}, combined-heuristic ROI MAE {:.5}, uniform ROI MAE {:.5}.",
        sequence.scenario_title,
        imported_trust_metrics.roi_mae,
        combined_metrics.roi_mae,
        uniform_metrics.roi_mae
    );
    let bounded_note = match sequence.expectation {
        ScenarioExpectation::BenefitExpected => {
            if imported_trust_metrics.roi_mae > combined_metrics.roi_mae {
                "Combined heuristic remains stronger on this scenario, which is surfaced explicitly in the decision report.".to_string()
            } else {
                "Imported trust remains competitive under equal budget on this scenario.".to_string()
            }
        }
        ScenarioExpectation::NeutralExpected => {
            "Neutral case: guidance is not expected to produce a large win, so non-ROI penalties and concentration behavior matter more than raw ROI gain.".to_string()
        }
    };

    Ok((
        DemoBScenarioReport {
            scenario_id: sequence.scenario_id.as_str().to_string(),
            scenario_title: sequence.scenario_title.clone(),
            expectation: sequence.expectation,
            onset_frame: onset,
            target_label: sequence.target_label.clone(),
            target_pixels: sequence.target_mask.iter().filter(|value| **value).count(),
            policies: policy_runs.iter().map(|run| run.metrics.clone()).collect(),
            headline,
            bounded_note,
        },
        DemoBScenarioRun {
            reference_frame,
            policy_runs,
            target_bbox,
        },
        curves,
    ))
}

fn policy_metrics(
    policy_id: AllocationPolicyId,
    context: PolicyMetricsContext<'_>,
) -> DemoBPolicyMetrics {
    let non_roi_mask = context
        .target_mask
        .iter()
        .map(|value| !value)
        .collect::<Vec<_>>();
    let total_samples = context.counts.iter().sum::<usize>();
    let roi_mean_spp = mean_count_over_mask(context.counts, context.target_mask);
    let non_roi_mean_spp = mean_count_over_mask(context.counts, &non_roi_mask);
    let roi_mae =
        mean_abs_error_over_mask(context.frame, context.reference_frame, context.target_mask);
    let policy_label = policy_id.label().to_string();
    let allocation_concentration = if non_roi_mean_spp <= f32::EPSILON {
        0.0
    } else {
        roi_mean_spp / non_roi_mean_spp
    };

    DemoBPolicyMetrics {
        policy_id: policy_id.as_str().to_string(),
        label: policy_label,
        total_samples,
        overall_mae: mean_abs_error(context.frame, context.reference_frame),
        overall_rmse: rmse(context.frame, context.reference_frame, None),
        roi_mae,
        roi_rmse: rmse(
            context.frame,
            context.reference_frame,
            Some(context.target_mask),
        ),
        non_roi_mae: mean_abs_error_over_mask(
            context.frame,
            context.reference_frame,
            &non_roi_mask,
        ),
        non_roi_rmse: rmse(context.frame, context.reference_frame, Some(&non_roi_mask)),
        roi_mean_spp,
        non_roi_mean_spp,
        max_spp: context.counts.iter().copied().max().unwrap_or(0),
        allocation_concentration,
        extra_roi_samples_vs_uniform: context.extra_roi_samples_vs_uniform,
        roi_error_reduction_per_extra_roi_sample: if context.extra_roi_samples_vs_uniform
            <= f32::EPSILON
        {
            0.0
        } else {
            (context.uniform_roi_mae - roi_mae) / context.extra_roi_samples_vs_uniform
        },
    }
}

fn find_policy(
    report: &DemoBScenarioReport,
    policy_id: AllocationPolicyId,
) -> Result<&DemoBPolicyMetrics> {
    report
        .policies
        .iter()
        .find(|policy| policy.policy_id == policy_id.as_str())
        .ok_or_else(|| Error::Message(format!("Demo B policy {} missing", policy_id.as_str())))
}

fn render_with_counts(
    sequence: &SceneSequence,
    scene_frame: &SceneFrame,
    counts: &[usize],
) -> ImageFrame {
    render_with_offset_counts(sequence, scene_frame, counts, 0)
}

fn render_with_offset_counts(
    sequence: &SceneSequence,
    scene_frame: &SceneFrame,
    counts: &[usize],
    seed_offset: u32,
) -> ImageFrame {
    let mut frame = ImageFrame::new(sequence.config.width, sequence.config.height);
    for y in 0..sequence.config.height {
        for x in 0..sequence.config.width {
            let pixel_index = y * sequence.config.width + x;
            let sample_count = counts[pixel_index].max(1);
            let mut accum = Color::rgb(0.0, 0.0, 0.0);

            for sample_index in 0..sample_count {
                let (offset_x, offset_y) =
                    sample_offset(pixel_index as u32 ^ seed_offset, sample_index as u32);
                let sample = sample_scene_continuous(
                    sequence,
                    scene_frame,
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

fn sample_scene_continuous(
    sequence: &SceneSequence,
    scene_frame: &SceneFrame,
    sample_x: f32,
    sample_y: f32,
) -> Color {
    let mut color =
        background_color_continuous(sample_x, sample_y, &sequence.config, sequence.scenario_id);
    if is_thin_structure_continuous(sample_x, sample_y, &sequence.config, sequence.scenario_id) {
        color = thin_structure_color_continuous(
            sample_x,
            sample_y,
            &sequence.config,
            sequence.scenario_id,
        );
    }
    if rect_contains_continuous(scene_frame.object_rect, sample_x, sample_y)
        && !matches!(
            sequence.scenario_id,
            ScenarioId::ContrastPulse | ScenarioId::StabilityHoldout
        )
    {
        color = object_color_continuous(sample_x, sample_y, scene_frame.object_rect);
    }
    if matches!(sequence.scenario_id, ScenarioId::ContrastPulse)
        && scene_frame.index >= sequence.onset_frame
        && contrast_pulse_rect(&sequence.config).contains(sample_x as i32, sample_y as i32)
    {
        color = Color::rgb(color.r * 1.22, color.g * 1.22, color.b * 1.22).clamp01();
    }
    color
}

fn background_color_continuous(
    sample_x: f32,
    sample_y: f32,
    config: &crate::config::SceneConfig,
    scenario_id: ScenarioId,
) -> Color {
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
    let stripes = if (3.0 * sample_x + sample_y).rem_euclid(17.0) < 5.0 {
        1.0
    } else {
        0.0
    };
    let vignette_x = (xf - 0.5).abs();
    let vignette_y = (yf - 0.5).abs();
    let vignette = 1.0 - (vignette_x * 0.35 + vignette_y * 0.4);

    match scenario_id {
        ScenarioId::ThinReveal | ScenarioId::StabilityHoldout => Color::rgb(
            (0.12 + 0.16 * xf + 0.05 * checker + 0.03 * diagonal) * vignette,
            (0.15 + 0.11 * yf + 0.04 * diagonal) * vignette,
            (0.22 + 0.18 * (1.0 - xf) + 0.03 * checker) * vignette,
        ),
        ScenarioId::FastPan => Color::rgb(
            (0.10 + 0.18 * xf + 0.08 * checker + 0.05 * stripes) * vignette,
            (0.11 + 0.15 * yf + 0.10 * diagonal + 0.04 * stripes) * vignette,
            (0.18 + 0.20 * (1.0 - xf) + 0.06 * checker) * vignette,
        ),
        ScenarioId::DiagonalReveal => Color::rgb(
            (0.08 + 0.24 * checker + 0.20 * diagonal + 0.05 * xf) * vignette,
            (0.08 + 0.18 * stripes + 0.07 * yf) * vignette,
            (0.12 + 0.25 * (1.0 - checker) + 0.04 * xf) * vignette,
        ),
        ScenarioId::RevealBand | ScenarioId::MotionBiasBand => {
            let micro = ((sample_x * 0.83 + sample_y * 1.91).sin() * 0.5 + 0.5)
                * ((sample_x * 1.37 - sample_y * 0.71).cos() * 0.5 + 0.5);
            let band = if (18.0..=(config.height as f32 - 18.0)).contains(&sample_y)
                && (26.0..=(config.width as f32 - 24.0)).contains(&sample_x)
            {
                1.0
            } else {
                0.0
            };
            Color::rgb(
                (0.10 + 0.14 * xf + 0.05 * checker + 0.10 * micro * band) * vignette,
                (0.12 + 0.13 * yf + 0.06 * diagonal + 0.08 * micro * band) * vignette,
                (0.16 + 0.18 * (1.0 - xf) + 0.07 * stripes + 0.12 * micro * band) * vignette,
            )
        }
        ScenarioId::ContrastPulse => {
            Color::rgb(0.18 + 0.06 * xf, 0.18 + 0.05 * yf, 0.24 + 0.06 * (1.0 - xf))
        }
    }
}

fn is_thin_structure_continuous(
    sample_x: f32,
    sample_y: f32,
    config: &crate::config::SceneConfig,
    scenario_id: ScenarioId,
) -> bool {
    if matches!(scenario_id, ScenarioId::ContrastPulse) {
        return false;
    }
    let vertical_center = config.thin_vertical_x as f32 + 0.5;
    let vertical = (sample_x - vertical_center).abs() <= 0.18
        && sample_y >= 14.0
        && sample_y <= config.height as f32 - 14.0;
    let diagonal_line =
        (sample_y - (0.58 * sample_x + 10.5)).abs() <= 0.20 && (28.0..=118.0).contains(&sample_x);
    let mixed_width_band = {
        let in_band = (18.0..=(config.height as f32 - 18.0)).contains(&sample_y)
            && (26.0..=(config.width as f32 - 24.0)).contains(&sample_x);
        let thin_slats = (sample_x - 28.0).rem_euclid(11.0) < 0.18;
        let medium_slats = (sample_x - 34.0).rem_euclid(19.0) < 1.10;
        let wide_slats = (sample_x - 48.0).rem_euclid(29.0) < 2.15;
        let diagonal = (sample_y - (0.44 * sample_x + 12.0)).abs() <= 1.15
            && (38.0..=(config.width as f32 - 32.0)).contains(&sample_x);
        in_band && (thin_slats || medium_slats || wide_slats || diagonal)
    };
    match scenario_id {
        ScenarioId::DiagonalReveal => diagonal_line,
        ScenarioId::RevealBand | ScenarioId::MotionBiasBand => mixed_width_band,
        _ => vertical || diagonal_line,
    }
}

fn thin_structure_color_continuous(
    sample_x: f32,
    sample_y: f32,
    config: &crate::config::SceneConfig,
    scenario_id: ScenarioId,
) -> Color {
    let vertical_center = config.thin_vertical_x as f32 + 0.5;
    if !matches!(scenario_id, ScenarioId::DiagonalReveal)
        && (sample_x - vertical_center).abs() <= 0.18
    {
        let pulse = if (sample_y / 3.0).floor() as i32 % 2 == 0 {
            1.0
        } else {
            0.84
        };
        return Color::rgb(0.95 * pulse, 0.96 * pulse, 0.98);
    }
    if matches!(scenario_id, ScenarioId::DiagonalReveal) {
        Color::rgb(0.24, 0.29, 0.35)
    } else if matches!(scenario_id, ScenarioId::RevealBand) {
        let phase = ((sample_x + 2.0 * sample_y).rem_euclid(9.0)) / 8.0;
        Color::rgb(
            0.22 + 0.48 * phase,
            0.58 + 0.26 * phase,
            0.84 + 0.12 * (1.0 - phase),
        )
    } else if matches!(scenario_id, ScenarioId::MotionBiasBand) {
        let phase = ((2.0 * sample_x + sample_y).rem_euclid(13.0)) / 12.0;
        Color::rgb(
            0.78 + 0.16 * phase,
            0.74 + 0.10 * (1.0 - phase),
            0.26 + 0.18 * phase,
        )
    } else {
        Color::rgb(0.64, 0.90, 0.96)
    }
}

fn contrast_pulse_rect(config: &crate::config::SceneConfig) -> Rect {
    Rect {
        x: (config.width as i32 / 2) - 18,
        y: (config.height as i32 / 2) - 18,
        width: 52,
        height: 36,
    }
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

fn guided_allocation(
    difficulty: &ScalarField,
    total_samples: usize,
    min_spp: usize,
    max_spp: usize,
) -> Result<Vec<usize>> {
    let total_pixels = difficulty.width() * difficulty.height();
    let min_total = min_spp * total_pixels;
    let max_total = max_spp * total_pixels;
    if total_samples < min_total || total_samples > max_total {
        return Err(Error::Message(
            "guided allocation budget is incompatible with the min/max spp bounds".to_string(),
        ));
    }

    let mut counts = vec![min_spp; total_pixels];
    let mut remaining = total_samples - min_total;
    if remaining == 0 {
        return Ok(counts);
    }

    let weights = difficulty
        .values()
        .iter()
        .map(|value| 0.05 + 0.95 * value.clamp(0.0, 1.0).powf(2.4))
        .collect::<Vec<_>>();

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
            let mut fallback = weights
                .iter()
                .copied()
                .enumerate()
                .filter(|(index, _)| counts[*index] < max_spp)
                .map(|(index, weight)| (weight, index))
                .collect::<Vec<_>>();
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

fn mean_abs_error_over_mask(frame_a: &ImageFrame, frame_b: &ImageFrame, mask: &[bool]) -> f32 {
    crate::frame::mean_abs_error_over_mask(frame_a, frame_b, mask)
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

fn invert_trust(trust: &ScalarField) -> ScalarField {
    let mut field = ScalarField::new(trust.width(), trust.height());
    for y in 0..trust.height() {
        for x in 0..trust.width() {
            field.set(x, y, (1.0 - trust.get(x, y)).clamp(0.0, 1.0));
        }
    }
    field
}

fn gradient_field(frame: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(frame.width(), frame.height());
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let center = frame.get(x, y).luma();
            let left = frame.sample_clamped(x as i32 - 1, y as i32).luma();
            let right = frame.sample_clamped(x as i32 + 1, y as i32).luma();
            let up = frame.sample_clamped(x as i32, y as i32 - 1).luma();
            let down = frame.sample_clamped(x as i32, y as i32 + 1).luma();
            let grad = (right - left)
                .abs()
                .max((down - up).abs())
                .max((center - left).abs());
            field.set(x, y, (grad / 0.25).clamp(0.0, 1.0));
        }
    }
    field
}

fn residual_proxy_field(frame: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(frame.width(), frame.height());
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let blurred = box_blur_luma(frame, x, y);
            let residual = (frame.get(x, y).luma() - blurred).abs();
            field.set(x, y, (residual / 0.22).clamp(0.0, 1.0));
        }
    }
    field
}

fn local_contrast_field(frame: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(frame.width(), frame.height());
    for y in 0..frame.height() {
        for x in 0..frame.width() {
            let center = frame.get(x, y).luma();
            let mut strongest = 0.0f32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let neighbor = frame.sample_clamped(x as i32 + dx, y as i32 + dy).luma();
                    strongest = strongest.max((center - neighbor).abs());
                }
            }
            field.set(x, y, (strongest / 0.18).clamp(0.0, 1.0));
        }
    }
    field
}

fn pilot_variance_field(pilot_a: &ImageFrame, pilot_b: &ImageFrame) -> ScalarField {
    let mut field = ScalarField::new(pilot_a.width(), pilot_a.height());
    for y in 0..pilot_a.height() {
        for x in 0..pilot_a.width() {
            let diff = pilot_a.get(x, y).abs_diff(pilot_b.get(x, y));
            field.set(x, y, (diff / 0.20).clamp(0.0, 1.0));
        }
    }
    field
}

fn combine_fields(fields: &[(&ScalarField, f32)], width: usize, height: usize) -> ScalarField {
    let mut field = ScalarField::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let value = fields
                .iter()
                .map(|(current, weight)| current.get(x, y) * *weight)
                .sum::<f32>()
                .clamp(0.0, 1.0);
            field.set(x, y, value);
        }
    }
    field
}

fn box_blur_luma(frame: &ImageFrame, x: usize, y: usize) -> f32 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let color = frame.sample_clamped(x as i32 + dx, y as i32 + dy);
            sum += color.luma();
            count += 1;
        }
    }
    sum / count as f32
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
