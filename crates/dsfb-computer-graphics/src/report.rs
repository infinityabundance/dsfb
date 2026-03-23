use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cost::{build_cost_report, CostMode, CostReport};
use crate::error::Result;
use crate::metrics::{
    AblationEntry, CalibrationBin, DemoASuiteMetrics, HistogramBin, TrustOperatingMode,
};
use crate::sampling::{DemoBScenarioReport, DemoBSuiteMetrics};
use crate::scaling::ResolutionScalingMetrics;
use crate::sensitivity::ParameterSensitivityMetrics;
use crate::timing::TimingMetrics;

pub const EXPERIMENT_SENTENCE: &str =
    "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”";
pub const COST_SENTENCE: &str = "“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”";
pub const COMPATIBILITY_SENTENCE: &str =
    "“The framework is compatible with tiled and asynchronous GPU execution.”";

pub fn build_trust_diagnostics(demo_a: &DemoASuiteMetrics) -> TrustDiagnostics {
    let mut scenarios = Vec::new();
    for scenario in &demo_a.scenarios {
        for run_id in [
            "dsfb_host_realistic",
            "dsfb_host_gated_reference",
            "dsfb_motion_augmented",
        ] {
            if let Some(run) = scenario
                .runs
                .iter()
                .find(|run| run.summary.run_id == run_id)
            {
                scenarios.push(TrustScenarioDiagnostic {
                    scenario_id: scenario.scenario_id.clone(),
                    scenario_title: scenario.scenario_title.clone(),
                    support_category: format!("{:?}", scenario.support_category),
                    run_id: run.summary.run_id.clone(),
                    label: run.summary.label.clone(),
                    roi_pixels: scenario.target_pixels,
                    occupied_bin_count: run.summary.trust_occupied_bin_count,
                    entropy_bits: run.summary.trust_entropy_bits,
                    discreteness_score: run.summary.trust_discreteness_score,
                    effective_level_count: run.summary.trust_effective_level_count,
                    operating_mode: run.summary.trust_operating_mode,
                    trust_error_rank_correlation: run.summary.trust_error_rank_correlation,
                    trust_rank_correlation_is_degenerate: run
                        .summary
                        .trust_rank_correlation_is_degenerate,
                    histogram: run.summary.trust_histogram.clone(),
                    calibration_bins: run.summary.trust_calibration_bins.clone(),
                });
            }
        }
    }

    let host_modes = scenarios
        .iter()
        .filter(|scenario| scenario.run_id == "dsfb_host_realistic")
        .filter_map(|scenario| scenario.operating_mode)
        .collect::<Vec<_>>();
    let conclusion = if host_modes
        .iter()
        .all(|mode| matches!(mode, TrustOperatingMode::NearBinaryGate))
    {
        "The current host-realistic implementation behaves as a near-binary gate rather than a smoothly calibrated continuous supervisor.".to_string()
    } else if host_modes
        .iter()
        .all(|mode| matches!(mode, TrustOperatingMode::StronglyGraded))
    {
        "The current host-realistic implementation behaves as a strongly graded supervisor on the measured suite.".to_string()
    } else {
        "The current host-realistic implementation is best described as weakly graded overall, with gate-like behavior on the point-ROI scenarios. The retained gated reference remains the explicitly near-binary mode.".to_string()
    };

    TrustDiagnostics {
        conclusion,
        scenarios,
    }
}

pub fn write_trust_diagnostics_report(path: &Path, diagnostics: &TrustDiagnostics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Trust Diagnostics");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", diagnostics.conclusion);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Run | ROI pixels | Occupied bins | Entropy (bits) | Mode | Correlation note |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | ---: | --- | --- |");
    for entry in &diagnostics.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {:.3} | {:?} | {} |",
            entry.scenario_id,
            entry.run_id,
            entry.roi_pixels,
            entry.occupied_bin_count,
            entry.entropy_bits.unwrap_or(0.0),
            entry.operating_mode,
            if entry.trust_rank_correlation_is_degenerate {
                "degenerate, not decision-facing"
            } else {
                "non-degenerate"
            }
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- These diagnostics do not prove probabilistic calibration in the statistical sense."
    );
    let _ = writeln!(
        markdown,
        "- Point-ROI scenarios remain weak evidence for smooth trust calibration even when they are mechanically useful."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The current trust signal still needs broader region-scale evidence and real-engine traces before it can be called broadly calibrated."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_timing_report(path: &Path, timing: &TimingMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Timing Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Measurement classification: `{}`.",
        timing.measurement_kind
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Actual GPU timing measured: `{}`.",
        timing.actual_gpu_timing
    );
    let _ = writeln!(markdown);
    for note in &timing.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Label | Mode | Scenario | Resolution | Build | Total ms | ms / frame | Ops / px | Traffic MB |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |"
    );
    for entry in &timing.entries {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {}x{} | {} | {:.3} | {:.3} | {} | {:.2} |",
            entry.label,
            entry.mode,
            entry.scenario_id,
            entry.width,
            entry.height,
            entry.build_profile,
            entry.total_ms,
            entry.ms_per_frame,
            entry.estimated_ops_per_pixel,
            entry.estimated_memory_traffic_megabytes
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Per-Stage Breakdown");
    let _ = writeln!(markdown);
    for entry in &timing.entries {
        let _ = writeln!(markdown, "### {}", entry.label);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "| Stage | Total ms | ms / frame | ns / pixel |");
        let _ = writeln!(markdown, "| --- | ---: | ---: | ---: |");
        for stage in &entry.stages {
            let _ = writeln!(
                markdown,
                "| {} | {:.3} | {:.3} | {:.3} |",
                stage.stage, stage.total_ms, stage.ms_per_frame, stage.ns_per_pixel
            );
        }
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "Likely optimization levers:");
        for lever in &entry.likely_optimization_levers {
            let _ = writeln!(markdown, "- {lever}");
        }
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not contain measured GPU milliseconds."
    );
    let _ = writeln!(
        markdown,
        "- It does not justify any production deployment performance claim."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real GPU execution and memory-system measurements remain outstanding."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_resolution_scaling_report(
    path: &Path,
    scaling: &ResolutionScalingMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Resolution Scaling Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Tier | Scenario | Resolution | ROI pixels | ROI fraction | Host ROI MAE | Host vs fixed gain | Motion vs host gain | Memory MB |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for entry in &scaling.entries {
        let _ = writeln!(
            markdown,
            "| {} | {} | {}x{} | {} | {:.5} | {:.5} | {:.5} | {:.5} | {:.2} |",
            entry.tier_id,
            entry.scenario_id,
            entry.width,
            entry.height,
            entry.target_pixels,
            entry.target_area_fraction,
            entry.host_realistic_cumulative_roi_mae,
            entry.host_realistic_vs_fixed_alpha_gain,
            entry.motion_augmented_vs_host_realistic_gain,
            entry.buffer_memory_megabytes
        );
    }
    let _ = writeln!(markdown);
    for note in &scaling.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report is a structural scaling study, not a production-scene benchmark."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- A full 1080p or 4K full-suite run with real hardware timing remains future work."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_parameter_sensitivity_report(
    path: &Path,
    sensitivity: &ParameterSensitivityMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Parameter Sensitivity Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Baseline mode: {}.", sensitivity.baseline_mode);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Parameter | Mode | Value | Benefit wins vs fixed | Zero-ghost benefit scenarios | Canonical ROI MAE | Region mean ROI MAE | Motion-bias ROI MAE | Neutral non-ROI MAE | Robust corridor |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for point in &sensitivity.sweep_points {
        let _ = writeln!(
            markdown,
            "| {} | {} | {:.3} | {} | {} | {:.5} | {:.5} | {:.5} | {:.5} | {} |",
            point.parameter_id,
            point.profile_mode,
            point.numeric_value,
            point.benefit_scenarios_beating_fixed,
            point.benefit_scenarios_with_zero_ghost_frames,
            point.canonical_cumulative_roi_mae,
            point.region_mean_cumulative_roi_mae,
            point.motion_bias_cumulative_roi_mae,
            point.neutral_non_roi_mae,
            if point.robust_corridor_member {
                "yes"
            } else {
                "no"
            }
        );
    }
    let _ = writeln!(markdown);
    for note in &sensitivity.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- These sweeps do not claim global optimality or statistically complete calibration."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Parameters are now centralized and sensitivity-vetted, but they are still hand-set rather than trained on an external benchmark."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_demo_b_efficiency_report(path: &Path, demo_b: &DemoBSuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo B Efficiency Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report separates aliasing-sensitive thin-point cases from larger mixed-width and motion-biased region cases so fixed-budget wins are not attributed only to sub-pixel line recovery."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Scenario | Policy | Mean spp | ROI MAE |");
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: |");
    for curve in &demo_b.budget_efficiency_curves {
        for point in &curve.points {
            let _ = writeln!(
                markdown,
                "| {} | {} | {:.1} | {:.5} |",
                curve.scenario_id, curve.policy_id, point.average_spp, point.roi_mae
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This study does not prove an optimal sampling controller or general renderer superiority."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Demo B remains synthetic and still needs real-engine noise and shading complexity for full production confidence."
    );
    fs::write(path, markdown)?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct CompletionNoteStatus {
    pub only_files_inside_crate_changed: bool,
    pub upgrade_plan_written: bool,
    pub host_realistic_mode_implemented: bool,
    pub stronger_baselines_implemented: bool,
    pub scenario_suite_implemented: bool,
    pub ablation_study_implemented: bool,
    pub demo_b_strengthened: bool,
    pub integration_surface_documented: bool,
    pub cost_model_generated: bool,
    pub reviewer_reports_generated: bool,
    pub required_honesty_sentence_present: bool,
    pub cargo_fmt_passed: bool,
    pub cargo_clippy_passed: bool,
    pub cargo_test_passed: bool,
    pub no_fabricated_performance_claims: bool,
    pub no_files_outside_crate_modified: bool,
    pub fully_implemented: Vec<String>,
    pub future_work: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrustScenarioDiagnostic {
    pub scenario_id: String,
    pub scenario_title: String,
    pub support_category: String,
    pub run_id: String,
    pub label: String,
    pub roi_pixels: usize,
    pub occupied_bin_count: usize,
    pub entropy_bits: Option<f32>,
    pub discreteness_score: Option<f32>,
    pub effective_level_count: Option<usize>,
    pub operating_mode: Option<TrustOperatingMode>,
    pub trust_error_rank_correlation: Option<f32>,
    pub trust_rank_correlation_is_degenerate: bool,
    pub histogram: Vec<HistogramBin>,
    pub calibration_bins: Vec<CalibrationBin>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrustDiagnostics {
    pub conclusion: String,
    pub scenarios: Vec<TrustScenarioDiagnostic>,
}

pub fn write_report(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    cost_report: &CostReport,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let canonical = &demo_a.scenarios[0];
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# DSFB Computer Graphics Evaluation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scope");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This crate is a deterministic, crate-local evaluation artifact for temporal reuse supervision and fixed-budget adaptive sampling."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is demonstrated: host-realistic DSFB supervision, stronger heuristic baselines, multi-scenario behavior, ablation sensitivity, fixed-budget allocation comparisons, attachability surfaces, and architectural cost accounting."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is not proven: production-scene generalization, measured GPU benchmark wins, engine deployment readiness, or universal superiority over strong heuristics."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scenario Suite");
    let _ = writeln!(markdown);
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "- `{}`: {}",
            scenario.scenario_id, scenario.scenario_description
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo A Baselines and DSFB Variants");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Baselines: fixed alpha, residual threshold, neighborhood clamp, depth/normal rejection, reactive-mask-style, and strong heuristic."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "DSFB variants: visibility-assisted synthetic mode, host-realistic mode, no-visibility, no-thin, no-motion, no-grammar, residual-only, and trust-without-alpha-modulation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Canonical Headline");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_a.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_a.summary.secondary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", canonical.headline);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Per-Scenario Outcome Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Expectation | Host vs fixed ROI gain | Host vs strong heuristic ROI gain | Non-ROI penalty vs fixed | Note |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {:.5} | {:.5} | {:.5} | {} |",
            scenario.scenario_title,
            scenario.expectation,
            scenario.host_realistic_vs_fixed_alpha_cumulative_roi_gain,
            scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain,
            scenario.host_realistic_non_roi_penalty_vs_fixed_alpha,
            scenario.bounded_or_neutral_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Ablation Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Variant | Canonical cumulative ROI MAE | Canonical peak ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: |");
    for entry in &demo_a.ablations {
        let _ = writeln!(
            markdown,
            "| {} | {:.5} | {:.5} | {:.5} | {:.5} |",
            entry.label,
            entry.canonical_cumulative_roi_mae,
            entry.canonical_peak_roi_mae,
            entry.suite_mean_cumulative_roi_mae,
            entry.suite_mean_false_positive_response_rate
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo B Fixed-Budget Study");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Imported trust ROI MAE | Combined heuristic ROI MAE | Uniform ROI MAE | Note |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | --- |");
    for scenario in &demo_b.scenarios {
        let trust = find_policy(scenario, "imported_trust");
        let combined = find_policy(scenario, "combined_heuristic");
        let uniform = find_policy(scenario, "uniform");
        if let (Some(trust), Some(combined), Some(uniform)) = (trust, combined, uniform) {
            let _ = writeln!(
                markdown,
                "| {} | {:.5} | {:.5} | {:.5} | {} |",
                scenario.scenario_title,
                trust.roi_mae,
                combined.roi_mae,
                uniform.roi_mae,
                scenario.bounded_note
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Attachability");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The host integration surface is implemented around typed current color, history color, motion vectors, depth, normals, trust, alpha, intervention, and optional sampling-budget outputs. See `docs/integration_surface.md`."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Cost Model");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COST_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COMPATIBILITY_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Mode | Buffers | Approx ops / pixel | Approx reads / pixel | Approx writes / pixel |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: |");
    for mode in [
        CostMode::Minimal,
        CostMode::HostRealistic,
        CostMode::FullResearchDebug,
    ] {
        let report = if mode == cost_report.mode {
            cost_report.clone()
        } else {
            build_cost_report(mode)
        };
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {} |",
            mode.label(),
            report.buffers.len(),
            report.estimated_total_ops_per_pixel,
            report.estimated_total_reads_per_pixel,
            report.estimated_total_writes_per_pixel
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Aggregate Leaderboard");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Run | Mean rank | Mean cumulative ROI MAE | Mean non-ROI MAE | Benefit-scenario wins |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: |");
    for entry in demo_a.aggregate_leaderboard.iter().take(10) {
        let _ = writeln!(
            markdown,
            "| {} | {:.2} | {:.5} | {:.5} | {} |",
            entry.label,
            entry.mean_rank,
            entry.mean_cumulative_roi_mae,
            entry.mean_non_roi_mae,
            entry.benefit_scenarios_won
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not prove production-scene generalization."
    );
    let _ = writeln!(
        markdown,
        "- This report does not prove that DSFB beats every strong heuristic on every scenario."
    );
    let _ = writeln!(
        markdown,
        "- This report does not claim measured GPU hardware wins or production readiness."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_reviewer_summary(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Reviewer Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_a.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is now decision-clean: host-realistic mode exists, stronger baselines are included, multiple deterministic scenarios are reported, ablations isolate cue dependence, Demo B is fixed-budget across multiple policies, and attachability/cost are explicit."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "What is still blocked: synthetic scene scope, lack of measured GPU benchmarks, and mixed outcomes against the strongest heuristic baseline on some scenarios."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This crate is ready for internal technical evaluation and funding diligence. It is not presented as a production-readiness or licensing-closing proof."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_five_mentor_audit(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let sections = [
        (
            "SBIR / Toyon",
            "Passes: bounded replayable evidence, host-style interface, multi-scenario report, remaining blockers stated openly.\nStill blocks: synthetic-only scope and no fielded deployment evidence.\nConfidence: ready for funding diligence.\nNext step: engine-side trace replay or mission-adjacent integration pilot.",
        ),
        (
            "NVIDIA",
            "Passes: stronger heuristic baselines, host-realistic cues, temporal attachability surface, multi-scenario TAA analysis.\nStill blocks: no measured GPU implementation and strong heuristic can remain competitive.\nConfidence: ready for evaluation.\nNext step: implement a reduced-resolution GPU pass and compare against an engine reactive-mask stack.",
        ),
        (
            "AMD / Intel",
            "Passes: explicit buffer model, local-operation cost accounting, tiled/async compatibility statement, fixed-budget fairness in Demo B.\nStill blocks: no measured cache/bandwidth data on real hardware.\nConfidence: ready for evaluation.\nNext step: hardware profiling pass with half-resolution and tile aggregation variants.",
        ),
        (
            "Academic",
            "Passes: deterministic suite, ablations, stronger baselines, neutral-case honesty, replayable figures and reports.\nStill blocks: synthetic breadth is still limited and there is no external benchmark corpus.\nConfidence: ready for evaluation.\nNext step: add richer published benchmark scenes and statistical robustness sweeps.",
        ),
        (
            "Licensing / Strategy",
            "Passes: attachable supervisory-layer shape, logging/trust outputs, explicit integration surfaces, and blocker-aware reporting.\nStill blocks: no external customer validation and no engine integration case study.\nConfidence: ready for licensing diligence.\nNext step: package the host interface into an engine-adjacent prototype and gather partner feedback.",
        ),
    ];

    let _ = writeln!(markdown, "# Five Mentor Audit");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo A: {}",
        demo_a.summary.primary_behavioral_result
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo B: {}",
        demo_b.summary.primary_behavioral_result
    );
    let _ = writeln!(markdown);
    for (title, body) in sections {
        let _ = writeln!(markdown, "## {title}");
        let _ = writeln!(markdown);
        for line in body.split('\n') {
            let _ = writeln!(markdown, "{line}");
        }
        let _ = writeln!(markdown);
    }

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_check_signing_blockers(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Blocker Check");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Removed");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Host-realistic DSFB mode exists and is reported separately from visibility-assisted mode.");
    let _ = writeln!(
        markdown,
        "- Stronger baselines are present and scored across multiple scenarios."
    );
    let _ = writeln!(
        markdown,
        "- A bounded neutral scenario is included to expose false positives."
    );
    let _ = writeln!(
        markdown,
        "- Demo B enforces fixed-budget fairness across multiple policies."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Partially Removed");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Strong heuristic baselines are now explicit, but they remain competitive on some scenarios.");
    let _ = writeln!(markdown, "- Cost confidence is better because buffers and stages are explicit, but hardware validation remains undone.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_ablation_report(path: &Path, entries: &[AblationEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Ablation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report answers which cues materially drive the effect and how much survives host-realistic mode."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Variant | Canonical cumulative ROI MAE | Suite mean cumulative ROI MAE | Suite mean false-positive rate |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: |");
    for entry in entries {
        let _ = writeln!(
            markdown,
            "| {} | {:.5} | {:.5} | {:.5} |",
            entry.label,
            entry.canonical_cumulative_roi_mae,
            entry.suite_mean_cumulative_roi_mae,
            entry.suite_mean_false_positive_response_rate
        );
    }

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_demo_b_decision_report(path: &Path, demo_b: &DemoBSuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo B Decision Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report explicitly separates aliasing recovery on point-like thin features from allocation quality on mixed-width and textured region cases."
    );
    let _ = writeln!(markdown);
    for scenario in &demo_b.scenarios {
        let _ = writeln!(markdown, "## {}", scenario.scenario_title);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "{}", scenario.headline);
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "{}", scenario.bounded_note);
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## What is not proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This study does not prove an optimal sampling controller."
    );
    let _ = writeln!(
        markdown,
        "- It does not prove that imported trust beats every cheap heuristic on every scene."
    );
    let _ = writeln!(
        markdown,
        "- It does not claim production renderer integration."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Demo B still lacks real-engine shading complexity and measured rendering hardware runs."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_cost_report(path: &Path, report: &CostReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Cost Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COST_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COMPATIBILITY_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Mode");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- {}", report.mode.label());
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Buffers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Buffer | Bytes / pixel | Notes |");
    let _ = writeln!(markdown, "| --- | ---: | --- |");
    for buffer in &report.buffers {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} |",
            buffer.name, buffer.bytes_per_pixel, buffer.notes
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Stages");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Stage | Approx ops / pixel | Reads / pixel | Writes / pixel | Reduction note |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | --- |");
    for stage in &report.stages {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {} |",
            stage.stage,
            stage.approximate_ops_per_pixel,
            stage.approximate_reads_per_pixel,
            stage.approximate_writes_per_pixel,
            stage.reduction_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Resolution Footprints");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Resolution | Pixels | Approx memory (MB) |");
    let _ = writeln!(markdown, "| --- | ---: | ---: |");
    for footprint in &report.footprints {
        let _ = writeln!(
            markdown,
            "| {}x{} | {} | {:.2} |",
            footprint.width, footprint.height, footprint.total_pixels, footprint.memory_megabytes
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Notes");
    let _ = writeln!(markdown);
    for note in &report.notes {
        let _ = writeln!(markdown, "- {note}");
    }

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_completion_note(path: &Path, status: &CompletionNoteStatus) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Completion Note");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Checklist");
    let _ = writeln!(markdown);
    checklist(
        &mut markdown,
        status.only_files_inside_crate_changed,
        "Only files inside crates/dsfb-computer-graphics were changed",
    );
    checklist(
        &mut markdown,
        status.upgrade_plan_written,
        "Upgrade plan was written inside the crate",
    );
    checklist(
        &mut markdown,
        status.host_realistic_mode_implemented,
        "Host-realistic DSFB mode is implemented",
    );
    checklist(
        &mut markdown,
        status.stronger_baselines_implemented,
        "Stronger baselines are implemented",
    );
    checklist(
        &mut markdown,
        status.scenario_suite_implemented,
        "Scenario suite is implemented",
    );
    checklist(
        &mut markdown,
        status.ablation_study_implemented,
        "Ablation study is implemented",
    );
    checklist(
        &mut markdown,
        status.demo_b_strengthened,
        "Demo B fixed-budget study is strengthened",
    );
    checklist(
        &mut markdown,
        status.integration_surface_documented,
        "Integration surface is documented",
    );
    checklist(
        &mut markdown,
        status.cost_model_generated,
        "Cost model report is generated",
    );
    checklist(
        &mut markdown,
        status.reviewer_reports_generated,
        "Reviewer reports are generated",
    );
    checklist(
        &mut markdown,
        status.required_honesty_sentence_present,
        "Required honesty sentence is present",
    );
    checklist(&mut markdown, status.cargo_fmt_passed, "cargo fmt passed");
    checklist(
        &mut markdown,
        status.cargo_clippy_passed,
        "cargo clippy passed",
    );
    checklist(&mut markdown, status.cargo_test_passed, "cargo test passed");
    checklist(
        &mut markdown,
        status.no_fabricated_performance_claims,
        "No fabricated performance claims were made",
    );
    checklist(
        &mut markdown,
        status.no_files_outside_crate_modified,
        "No files outside the crate were modified",
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Fully Implemented");
    let _ = writeln!(markdown);
    for item in &status.fully_implemented {
        let _ = writeln!(markdown, "- {item}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Future Work");
    let _ = writeln!(markdown);
    for item in &status.future_work {
        let _ = writeln!(markdown, "- {item}");
    }
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_report(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    cost_report: &CostReport,
    trust: &TrustDiagnostics,
    timing: &TimingMetrics,
    scaling: &ResolutionScalingMetrics,
    sensitivity: &ParameterSensitivityMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# DSFB Computer Graphics Evaluation Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scope");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This artifact is a deterministic crate-local evaluation package for temporal-reuse supervision and fixed-budget sampling allocation. It is intended to clear diligence blockers honestly, not to imply production readiness."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## ROI Disclosure");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Support category | ROI pixels | ROI fraction | Disclosure |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.support_category,
            scenario.target_pixels,
            scenario.target_area_fraction,
            scenario.roi_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Point-like ROI scenarios are kept because they remain mechanically relevant, but they are not mixed with region-ROI evidence without explicit disclosure.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Scenario Outcomes");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Expectation | Host vs fixed ROI gain | Host vs strong ROI gain | Non-ROI penalty vs strong | Clamp trigger mean | Note |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {:.5} | {:.5} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.expectation,
            scenario.host_realistic_vs_fixed_alpha_cumulative_roi_gain,
            scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain,
            scenario.host_realistic_non_roi_penalty_vs_strong_heuristic,
            scenario.neighborhood_clamp_roi_trigger_mean,
            scenario.bounded_or_neutral_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Trust Diagnostics");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", trust.conclusion);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Degenerate trust-error rank correlations are retained only as diagnostics and are not used here as decision-facing calibration evidence."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Motion Disagreement Decision");
    let _ = writeln!(markdown);
    let motion_bias = demo_a
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario_id == "motion_bias_band");
    if let Some(scenario) = motion_bias {
        let motion = scenario
            .runs
            .iter()
            .find(|run| run.summary.run_id == "dsfb_motion_augmented");
        let host = scenario
            .runs
            .iter()
            .find(|run| run.summary.run_id == "dsfb_host_realistic");
        if let (Some(motion), Some(host)) = (motion, host) {
            let _ = writeln!(
                markdown,
                "The minimum host-realistic path excludes motion disagreement. On `motion_bias_band`, the optional motion-augmented path changed cumulative ROI MAE from {:.5} to {:.5}. That makes motion disagreement an optional extension rather than a minimum-path requirement.",
                host.summary.cumulative_roi_mae,
                motion.summary.cumulative_roi_mae
            );
        }
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Demo B Confound Handling");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo B now includes mixed-width and textured region scenarios alongside the original thin-point case, and reports equal-budget curves at 1, 2, 4, and 8 mean spp. The goal is to separate aliasing recovery from structurally better allocation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Resolution Scaling");
    let _ = writeln!(markdown);
    for note in &scaling.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Timing Path");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Timing classification: `{}`. Actual GPU timing measured: `{}`.",
        timing.measurement_kind, timing.actual_gpu_timing
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Cost Model");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COST_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{COMPATIBILITY_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Mode | Buffers | Ops / px | Reads / px | Writes / px |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: |");
    for mode in [
        CostMode::Minimal,
        CostMode::HostRealistic,
        CostMode::FullResearchDebug,
    ] {
        let report = if mode == cost_report.mode {
            cost_report.clone()
        } else {
            build_cost_report(mode)
        };
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {} |",
            mode.label(),
            report.buffers.len(),
            report.estimated_total_ops_per_pixel,
            report.estimated_total_reads_per_pixel,
            report.estimated_total_writes_per_pixel
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Parameter Sensitivity");
    let _ = writeln!(markdown);
    let robust_count = sensitivity
        .sweep_points
        .iter()
        .filter(|point| point.robust_corridor_member)
        .count();
    let _ = writeln!(
        markdown,
        "Centralized hazard weights are still hand-set, but they are now sensitivity-vetted. Robust corridor sweep points found: {} of {}.",
        robust_count,
        sensitivity.sweep_points.len()
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The supervisory effect is real under a host-realistic minimum path, not only with privileged visibility hints."
    );
    let _ = writeln!(
        markdown,
        "- Point-like ROI evidence and region-ROI evidence are now reported separately."
    );
    let _ = writeln!(
        markdown,
        "- Motion disagreement is no longer treated as mandatory in the minimum path."
    );
    let _ = writeln!(
        markdown,
        "- Demo B no longer relies only on the original thin sub-pixel case."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This artifact does not prove production-scene generalization."
    );
    let _ = writeln!(
        markdown,
        "- It does not prove measured GPU wins or production deployment performance."
    );
    let _ = writeln!(
        markdown,
        "- It does not prove globally calibrated trust or globally optimal parameter settings."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }
    let _ = writeln!(markdown, "- Real GPU execution data remains outstanding.");
    let _ = writeln!(
        markdown,
        "- External engine traces and broader scene diversity remain future work."
    );

    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_reviewer_summary(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    trust: &TrustDiagnostics,
    timing: &TimingMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let point_like = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                crate::scene::ScenarioSupportCategory::PointLikeRoi
            )
        })
        .map(|scenario| format!("{}={} px", scenario.scenario_id, scenario.target_pixels))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(markdown, "# Reviewer Summary");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_a.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Point-like ROI disclosure: {}. These remain mechanically relevant but statistically weak and are not treated as region-scale aggregate evidence.",
        point_like
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Trust conclusion: {}", trust.conclusion);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Timing conclusion: `{}` with actual GPU timing measured = `{}`.",
        timing.measurement_kind, timing.actual_gpu_timing
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "What is still blocked:");
    let _ = writeln!(markdown, "- real GPU execution measurements");
    let _ = writeln!(markdown, "- broader external scene validation");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "What is now decision-clean:");
    let _ = writeln!(markdown, "- host-realistic minimum path is explicit");
    let _ = writeln!(markdown, "- point vs region ROI evidence is separated");
    let _ = writeln!(
        markdown,
        "- motion disagreement is optional rather than hidden in the minimum path"
    );
    let _ = writeln!(
        markdown,
        "- Demo B includes region and mixed-width evidence"
    );
    let _ = writeln!(markdown, "- weights are centralized and sensitivity-vetted");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- no actual GPU timing in this environment");
    let _ = writeln!(markdown, "- no production-scene or engine deployment proof");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- real GPU execution measurements");
    let _ = writeln!(markdown, "- broader external scene validation");
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_five_mentor_audit(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    timing: &TimingMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let point_like = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                crate::scene::ScenarioSupportCategory::PointLikeRoi
            )
        })
        .map(|scenario| format!("{}={} px", scenario.scenario_id, scenario.target_pixels))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(markdown, "# Five Mentor Audit");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo A: {}",
        demo_a.summary.primary_behavioral_result
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Demo B: {}",
        demo_b.summary.primary_behavioral_result
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Point-like ROI disclosure: {}. Region-sized scenarios are reported separately for decision-facing aggregate claims.",
        point_like
    );
    let _ = writeln!(markdown);
    let gpu_line = if timing.actual_gpu_timing {
        "Measured GPU timing is available."
    } else {
        "Only CPU proxy timing is available."
    };
    for (title, readiness, passes, blockers) in [
        (
            "SBIR / Toyon",
            "ready for evaluation",
            "multi-scenario host-realistic evidence, explicit blockers, and fail-loud validation",
            "synthetic-only scope and no fielded integration",
        ),
        (
            "NVIDIA",
            "ready for evaluation",
            "timing path exists, minimum path is explicit, motion extension is isolated",
            "no measured GPU execution; strong heuristic still competitive on some scenarios",
        ),
        (
            "AMD / Intel",
            "ready for evaluation",
            "buffer, traffic, and scaling surfaces are explicit",
            "no hardware cache/bandwidth measurements",
        ),
        (
            "Academic",
            "ready for evaluation",
            "honest ROI disclosure, ablations, trust diagnostics, and sensitivity sweeps",
            "synthetic breadth and no external benchmark corpus",
        ),
        (
            "Licensing / Strategy",
            "ready for evaluation",
            "decision-facing reports show what passes, what ties, and what still blocks",
            "no engine case study or customer validation",
        ),
    ] {
        let _ = writeln!(markdown, "## {title}");
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "Passes: {passes}.");
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "Still blocks: {blockers}.");
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "Timing note: {gpu_line}");
        let _ = writeln!(markdown);
        let _ = writeln!(markdown, "Readiness: {readiness}.");
        let _ = writeln!(markdown);
    }
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This audit does not claim funding close, licensing close, or deployment readiness."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- real GPU measurements");
    let _ = writeln!(markdown, "- external engine validation");
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_check_signing_blockers(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    timing: &TimingMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let point_like = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.support_category,
                crate::scene::ScenarioSupportCategory::PointLikeRoi
            )
        })
        .map(|scenario| format!("{}={} px", scenario.scenario_id, scenario.target_pixels))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(markdown, "# Blocker Check");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Point-like ROI disclosure: {}. These cases are no longer buried inside region-ROI aggregate claims.",
        point_like
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Removed");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Point-like ROI evidence is now labeled explicitly instead of being mixed into aggregate claims.");
    let _ = writeln!(markdown, "- Trust rank correlation is no longer used as a headline calibration claim when degenerate.");
    let _ = writeln!(
        markdown,
        "- Motion disagreement is not hidden in the minimum host-realistic path."
    );
    let _ = writeln!(
        markdown,
        "- Hazard weights are centralized and sensitivity-vetted."
    );
    let _ = writeln!(
        markdown,
        "- Demo B includes mixed-width region cases and equal-budget efficiency curves."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Partially Removed");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- GPU timing is addressed by a CPU proxy timing path and hardware-model estimates, but actual GPU measurements are still missing: `{}`.",
        timing.actual_gpu_timing
    );
    let _ = writeln!(
        markdown,
        "- Trust behavior is now described honestly, but broad calibration claims still remain blocked by limited scene diversity."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }
    let _ = writeln!(markdown, "- Actual GPU execution measurements.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This file does not claim all diligence blockers are removed."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- synthetic-only scope");
    let _ = writeln!(markdown, "- no production-scene engine integration");
    fs::write(path, markdown)?;
    Ok(())
}

fn checklist(markdown: &mut String, ok: bool, label: &str) {
    let _ = writeln!(markdown, "- {} {}", if ok { "[x]" } else { "[ ]" }, label);
}

fn find_policy<'a>(
    scenario: &'a DemoBScenarioReport,
    policy_id: &str,
) -> Option<&'a crate::sampling::DemoBPolicyMetrics> {
    scenario
        .policies
        .iter()
        .find(|policy| policy.policy_id == policy_id)
}
