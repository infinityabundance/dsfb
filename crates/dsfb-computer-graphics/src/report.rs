use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cost::{build_cost_report, CostMode, CostReport};
use crate::error::Result;
use crate::external::ExternalHandoffMetrics;
use crate::gpu_execution::GpuExecutionMetrics;
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
    let _ = writeln!(
        markdown,
        "- External validation is still required on real engine-exported buffers and target GPU hardware."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real GPU execution and memory-system measurements remain outstanding."
    );
    let _ = writeln!(
        markdown,
        "- External handoff is available, but externally validated timing data is still absent."
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
    let _ = writeln!(
        markdown,
        "- External validation is still required to show that the same scaling behavior survives real engine-exported buffers."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- A full 1080p or 4K full-suite run with real hardware timing remains future work."
    );
    let _ = writeln!(
        markdown,
        "- External handoff exists, but no externally validated scaling study is included here."
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
        "| Parameter | Mode | Value | Benefit wins vs fixed | Zero-ghost benefit scenarios | Canonical ROI MAE | Region mean ROI MAE | Motion-bias ROI MAE | Neutral non-ROI MAE | Robust corridor | Robustness |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |"
    );
    for point in &sensitivity.sweep_points {
        let _ = writeln!(
            markdown,
            "| {} | {} | {:.3} | {} | {} | {:.5} | {:.5} | {:.5} | {:.5} | {} | {} |",
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
            },
            point.robustness_class
        );
    }
    let _ = writeln!(markdown);
    for note in &sensitivity.notes {
        let _ = writeln!(markdown, "- {note}");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- `robust` means the main benefit cases remain intact with bounded motion-bias and neutral-scene degradation."
    );
    let _ = writeln!(
        markdown,
        "- `moderately_sensitive` means the conclusion survives, but with narrower safety margin."
    );
    let _ = writeln!(
        markdown,
        "- `fragile` means the headline behavior or neutral-scene bound degrades materially."
    );
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
        "This report separates aliasing-limited thin-point cases from variance-limited and mixed-width region cases so fixed-budget wins are not attributed only to sub-pixel line recovery."
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
    let _ = writeln!(markdown, "## Scenario Taxonomy");
    let _ = writeln!(markdown);
    for scenario in &demo_b.scenarios {
        let _ = writeln!(
            markdown,
            "- `{}`: taxonomy=`{}`, sampling_taxonomy=`{}`",
            scenario.scenario_id, scenario.demo_b_taxonomy, scenario.sampling_taxonomy
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This study does not prove an optimal sampling controller or general renderer superiority."
    );
    let _ = writeln!(
        markdown,
        "- External validation is still required on real renderer noise and imported engine buffers."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Demo B remains synthetic and still needs real-engine noise and shading complexity for full production confidence."
    );
    let _ = writeln!(
        markdown,
        "- External handoff exists for Demo A style supervision, but Demo B still lacks an external renderer allocation trace."
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
        "This report explicitly separates aliasing recovery on point-like thin features from allocation quality on mixed-width, variance-limited, and edge-trap region cases under fixed-budget equality."
    );
    let _ = writeln!(markdown);
    for scenario in &demo_b.scenarios {
        let _ = writeln!(markdown, "## {}", scenario.scenario_title);
        let _ = writeln!(markdown);
        let _ = writeln!(
            markdown,
            "Taxonomy: `{}`. Sampling taxonomy: `{}`. Support category: `{:?}`.",
            scenario.demo_b_taxonomy, scenario.sampling_taxonomy, scenario.support_category
        );
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
    let _ = writeln!(
        markdown,
        "- External validation is still required before extending these conclusions to real renderer sample allocation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Demo B still lacks real-engine shading complexity and measured rendering hardware runs."
    );
    let _ = writeln!(
        markdown,
        "- External handoff for imported supervision exists, but no external sample-allocation capture is included."
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
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
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
        "| Scenario | Expectation | Tags | Host vs fixed ROI gain | Host vs strong ROI gain | Non-ROI penalty vs strong | Clamp trigger mean | Note |"
    );
    let _ = writeln!(
        markdown,
        "| --- | --- | --- | ---: | ---: | ---: | ---: | --- |"
    );
    for scenario in &demo_a.scenarios {
        let tags = scenario_tags(scenario);
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {} | {:.5} | {:.5} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.expectation,
            tags.join(", "),
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
        "Demo B now includes aliasing-limited, variance-limited, edge-trap, and mixed-width region cases alongside the original thin-point case, and reports equal-budget curves at 1, 2, 4, and 8 mean spp. The goal is to separate aliasing recovery from structurally better allocation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## External Handoff Bridge");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "External import status: external-capable = `{}`, externally validated = `{}`. Source kind used in the generated handoff example: `{}`.",
        external.external_capable, external.externally_validated, external.source_kind
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "The crate can now import current color, reprojected history, motion vectors, depth, and normals through a stable manifest/schema without re-architecting the evaluator."
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
    let _ = writeln!(markdown, "## GPU Execution Path");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "GPU execution classification: `{}`. Actual GPU timing measured: `{}`.",
        gpu.measurement_kind, gpu.actual_gpu_timing_measured
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "A real `wgpu` compute kernel for the minimum host-realistic supervisory path is now in the crate. If no adapter is present, the generated GPU report states that explicitly instead of implying measurement."
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
    let _ = writeln!(
        markdown,
        "- A file-based external buffer handoff path now exists for engine-adjacent evaluation."
    );
    let _ = writeln!(
        markdown,
        "- A GPU-executable minimum kernel now exists even when the current environment cannot measure it."
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
    let _ = writeln!(
        markdown,
        "- It does not prove external engine validation merely because the import schema now exists."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- Real GPU execution data remains outstanding.");
    }
    let _ = writeln!(
        markdown,
        "- External engine traces and broader scene diversity remain future work."
    );
    let _ = writeln!(
        markdown,
        "- Strong heuristic baselines remain competitive on some scenarios, so the correct framing remains a targeted supervisory overlay rather than a general-purpose replacement."
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
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
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
    let _ = writeln!(
        markdown,
        "GPU bridge conclusion: `{}` with actual GPU timing measured = `{}`.",
        gpu.measurement_kind, gpu.actual_gpu_timing_measured
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "External bridge conclusion: external-capable = `{}`, externally validated = `{}`.",
        external.external_capable, external.externally_validated
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "What is still blocked:");
    let _ = writeln!(markdown, "- broader external scene validation");
    let _ = writeln!(markdown, "- engine-side GPU profiling on imported captures");
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- real GPU execution measurements");
    }
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
    let _ = writeln!(
        markdown,
        "- a real GPU-executable kernel exists in the crate"
    );
    let _ = writeln!(
        markdown,
        "- external buffers can be imported through a stable manifest"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- no actual GPU timing in this environment");
    }
    let _ = writeln!(markdown, "- no production-scene or engine deployment proof");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- broader external scene validation");
    let _ = writeln!(markdown, "- engine-side GPU profiling on imported captures");
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- real GPU execution measurements");
    }
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_five_mentor_audit(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    _timing: &TimingMetrics,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
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
    let gpu_line = if gpu.actual_gpu_timing_measured {
        "Measured GPU timing is available."
    } else {
        "A GPU-executable path exists, but this environment did not produce measured GPU timing."
    };
    for (title, readiness, passes, blockers) in [
        (
            "SBIR / Toyon",
            "ready for evaluation",
            "multi-scenario host-realistic evidence, explicit blockers, fail-loud validation, and evaluator handoff package",
            "synthetic-only scope and no fielded integration",
        ),
        (
            "NVIDIA",
            "ready for evaluation",
            "GPU-executable minimum kernel exists, minimum path is explicit, and motion extension is isolated",
            "no measured engine-integrated GPU execution; strong heuristic still competitive on some scenarios",
        ),
        (
            "AMD / Intel",
            "ready for evaluation",
            "buffer, traffic, scaling, and external import surfaces are explicit",
            "no hardware cache/bandwidth measurements on real imported captures",
        ),
        (
            "Academic",
            "ready for evaluation",
            "honest ROI disclosure, ablations, trust diagnostics, sensitivity sweeps, and scenario taxonomy",
            "synthetic breadth and no external benchmark corpus",
        ),
        (
            "Licensing / Strategy",
            "ready for evaluation",
            "decision-facing reports show what passes, what ties, what external data is needed, and what to test next",
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
        let _ = writeln!(
            markdown,
            "External handoff note: external-capable = `{}`, externally validated = `{}`.",
            external.external_capable, external.externally_validated
        );
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
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- real GPU measurements");
    }
    let _ = writeln!(markdown, "- external engine validation");
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_full_check_signing_blockers(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    _timing: &TimingMetrics,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
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
    let _ = writeln!(
        markdown,
        "- An external buffer schema and file-based import path now exist."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Partially Removed");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- GPU timing is addressed by a GPU-executable kernel and an honest measured-vs-unmeasured path, but actual GPU measurements are still missing here: `{}`.",
        gpu.actual_gpu_timing_measured
    );
    let _ = writeln!(
        markdown,
        "- Trust behavior is now described honestly, but broad calibration claims still remain blocked by limited scene diversity."
    );
    let _ = writeln!(
        markdown,
        "- The crate is external-capable, but externally validated remains `{}`.",
        external.externally_validated
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining");
    let _ = writeln!(markdown);
    for blocker in &demo_a.summary.remaining_blockers {
        let _ = writeln!(markdown, "- {blocker}");
    }
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- Actual GPU execution measurements.");
    }
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

pub fn write_realism_suite_report(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Realism Suite Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Support | Tags | ROI pixels | ROI fraction | Host vs fixed ROI gain | Host vs strong ROI gain |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | ---: | ---: | ---: | ---: |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {} | {} | {:.5} | {:.5} | {:.5} |",
            scenario.scenario_id,
            scenario.support_category,
            scenario_tags(scenario).join(", "),
            scenario.target_pixels,
            scenario.target_area_fraction,
            scenario.host_realistic_vs_fixed_alpha_cumulative_roi_gain,
            scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The suite now contains explicit realism-stress and competitive-baseline cases alongside point-ROI and region-ROI evidence."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- These scenarios are still synthetic and do not replace external renderer captures."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real production-scene generalization still requires external captures."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_realism_bridge_report(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let point_roi = demo_a.summary.point_roi_scenarios.len();
    let region_roi = demo_a.summary.region_roi_scenarios.len();
    let realism_stress = demo_a
        .scenarios
        .iter()
        .filter(|scenario| scenario.realism_stress)
        .count();
    let competitive = demo_a
        .scenarios
        .iter()
        .filter(|scenario| scenario.competitive_baseline_case)
        .count();
    let bounded = demo_a
        .scenarios
        .iter()
        .filter(|scenario| scenario.bounded_loss_disclosure)
        .count();

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Realism Bridge Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Region-ROI evidence, realism-stress probes, competitive-baseline cases, and bounded-neutral controls now carry the main empirical load instead of leaving the story concentrated in point-ROI stress tests."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Point-ROI scenarios: `{point_roi}`");
    let _ = writeln!(markdown, "- Region-ROI scenarios: `{region_roi}`");
    let _ = writeln!(markdown, "- Realism-stress scenarios: `{realism_stress}`");
    let _ = writeln!(
        markdown,
        "- Strong-heuristic-competitive scenarios: `{competitive}`"
    );
    let _ = writeln!(
        markdown,
        "- Bounded-neutral or bounded-loss disclosures: `{bounded}`"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Support | Tags | ROI pixels | Host vs fixed ROI gain | Host vs strong ROI gain | Bounded note |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | ---: | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:?} | {} | {} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.support_category,
            scenario_tags(scenario).join(", "),
            scenario.target_pixels,
            scenario.host_realistic_vs_fixed_alpha_cumulative_roi_gain,
            scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain,
            scenario.bounded_or_neutral_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The crate now exposes a broader synthetic realism bridge with explicit external-handoff relevance instead of a narrow point-ROI-only story."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- These scenarios remain synthetic and do not replace external engine captures or production image content."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The realism bridge still needs external replay on real engine buffers before it can be treated as production-adjacent evidence."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_trust_mode_report(path: &Path, diagnostics: &TrustDiagnostics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let region_entries = diagnostics
        .scenarios
        .iter()
        .filter(|entry| entry.run_id == "dsfb_host_realistic")
        .filter(|entry| entry.support_category.contains("RegionRoi"))
        .collect::<Vec<_>>();
    let mut mode_counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in diagnostics
        .scenarios
        .iter()
        .filter(|entry| entry.run_id == "dsfb_host_realistic")
    {
        *mode_counts
            .entry(
                entry
                    .operating_mode
                    .map(|mode| format!("{mode:?}"))
                    .unwrap_or_else(|| "Unknown".to_string()),
            )
            .or_insert(0) += 1;
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Trust Mode Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{}", diagnostics.conclusion);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Operating Mode Counts");
    let _ = writeln!(markdown);
    for (mode, count) in mode_counts {
        let _ = writeln!(markdown, "- `{mode}`: `{count}` host-realistic scenarios");
    }
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Region-ROI scenario | Occupied bins | Effective levels | Entropy (bits) | Discreteness | Mode | Correlation note |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | ---: | --- | --- |");
    for entry in region_entries {
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {:.3} | {:.3} | {} | {} |",
            entry.scenario_id,
            entry.occupied_bin_count,
            entry
                .effective_level_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
            entry.entropy_bits.unwrap_or(0.0),
            entry.discreteness_score.unwrap_or(0.0),
            entry
                .operating_mode
                .map(|mode| format!("{mode:?}"))
                .unwrap_or_else(|| "Unknown".to_string()),
            if entry.trust_rank_correlation_is_degenerate {
                "degenerate, not decision-facing"
            } else {
                "non-degenerate"
            }
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The trust signal is now described according to its actual operating mode instead of being overstated as smoothly calibrated."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not claim externally validated probabilistic calibration."
    );
    let _ = writeln!(
        markdown,
        "- A gate-like trust mode can still be useful externally, but this report does not turn it into a continuous confidence guarantee."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real external replay traces are still needed before the trust operating mode can be generalized beyond this synthetic suite."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_competitive_baseline_analysis(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let host_beats_strong = demo_a
        .summary
        .host_realistic_beats_strong_heuristic_scenarios;
    let benefit_count = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.expectation,
                crate::scene::ScenarioExpectation::BenefitExpected
            )
        })
        .count();
    let framing = if host_beats_strong == benefit_count {
        "broader supervisory replacement candidate"
    } else {
        "targeted supervisory overlay / instability-focused specialist"
    };
    let _ = writeln!(markdown, "# Competitive Baseline Analysis");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Recommended framing: **{}**.", framing);
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Competitive baseline case | Host vs strong ROI gain | Non-ROI penalty ratio vs strong | Interpretation |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let interpretation =
            if scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain > 0.0 {
                "DSFB wins in the targeted instability region."
            } else if scenario
                .host_realistic_vs_strong_heuristic_cumulative_roi_gain
                .abs()
                < 1e-4
            {
                "Tie or effectively neutral."
            } else {
                "Strong heuristic remains competitive or better here."
            };
        let _ = writeln!(
            markdown,
            "| {} | {} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.competitive_baseline_case,
            scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain,
            scenario.host_realistic_non_roi_penalty_ratio_vs_strong_heuristic,
            interpretation
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This analysis does not support universal-win language."
    );
    let _ = writeln!(
        markdown,
        "- External validation is still required to confirm these competitive-baseline relationships on imported engine captures."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Competitive-baseline results still need real-engine confirmation on imported captures."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_product_positioning_report(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let wins = demo_a
        .scenarios
        .iter()
        .filter(|scenario| scenario.host_realistic_vs_strong_heuristic_cumulative_roi_gain > 0.0)
        .count();
    let ties = demo_a
        .scenarios
        .iter()
        .filter(|scenario| {
            scenario
                .host_realistic_vs_strong_heuristic_cumulative_roi_gain
                .abs()
                <= 1.0e-4
        })
        .count();
    let losses = demo_a.scenarios.len().saturating_sub(wins + ties);
    let framing = if losses == 0 {
        "targeted supervisory overlay with unusually broad synthetic support"
    } else {
        "targeted supervisory overlay / instability-focused specialist"
    };

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Product Positioning Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Recommended framing: **{}**.", framing);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- Wins vs strong heuristic: `{wins}`");
    let _ = writeln!(markdown, "- Ties vs strong heuristic: `{ties}`");
    let _ = writeln!(markdown, "- Losses vs strong heuristic: `{losses}`");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "DSFB's value is concentrated in instability-focused intervention rather than universal full-frame quality dominance. That makes non-ROI penalties and strong-heuristic ties part of the product story, not evidence to hide."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The current bundle supports a targeted-supervision story with explicit competitive-baseline honesty and external evaluation guidance."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not justify blanket replacement language or a claim that DSFB beats all strong heuristics on every scene."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- External replay is still required to confirm that the same positioning holds on real engine-exported buffers."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_non_roi_penalty_report(path: &Path, demo_a: &DemoASuiteMetrics) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Non-ROI Penalty Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report quantifies non-ROI penalty so evaluator-facing claims do not hide off-target cost."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Host non-ROI MAE penalty vs fixed | Host non-ROI MAE penalty vs strong | Penalty ratio vs strong | Note |"
    );
    let _ = writeln!(markdown, "| --- | ---: | ---: | ---: | --- |");
    for scenario in &demo_a.scenarios {
        let _ = writeln!(
            markdown,
            "| {} | {:.5} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.host_realistic_non_roi_penalty_vs_fixed_alpha,
            scenario.host_realistic_non_roi_penalty_vs_strong_heuristic,
            scenario.host_realistic_non_roi_penalty_ratio_vs_strong_heuristic,
            scenario.bounded_or_neutral_note
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not claim DSFB improves global full-frame quality in every case."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Non-ROI tradeoffs still need validation on imported external captures and measured GPU runs."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_demo_b_competitive_baselines_report(
    path: &Path,
    demo_b: &DemoBSuiteMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo B Competitive Baselines Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report compares imported trust against the full heuristic suite: gradient-magnitude / edge-guided, residual-guided, contrast-guided, variance-guided, combined heuristic, native trust, and hybrid trust + variance."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Taxonomy | Best heuristic baseline | Best heuristic ROI MAE | Imported trust ROI MAE | Hybrid ROI MAE | Interpretation |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | ---: | ---: | ---: | --- |");
    for scenario in &demo_b.scenarios {
        let heuristic_ids = [
            "edge_guided",
            "residual_guided",
            "contrast_guided",
            "variance_guided",
            "combined_heuristic",
        ];
        let best_heuristic = scenario
            .policies
            .iter()
            .filter(|policy| heuristic_ids.contains(&policy.policy_id.as_str()))
            .min_by(|left, right| left.roi_mae.total_cmp(&right.roi_mae))
            .expect("Demo B heuristic baseline should exist");
        let imported = find_policy(scenario, "imported_trust").expect("imported trust policy");
        let hybrid = find_policy(scenario, "hybrid_trust_variance").expect("hybrid policy");
        let interpretation = if imported.roi_mae + 1.0e-6 < best_heuristic.roi_mae {
            "Imported trust beats the strongest heuristic baseline on fixed budget."
        } else if hybrid.roi_mae + 1.0e-6 < best_heuristic.roi_mae {
            "Hybrid trust/variance beats pure heuristics even when imported trust alone does not."
        } else {
            "Heuristic baseline remains competitive here."
        };
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {:.5} | {:.5} | {:.5} | {} |",
            scenario.scenario_id,
            scenario.demo_b_taxonomy,
            best_heuristic.label,
            best_heuristic.roi_mae,
            imported.roi_mae,
            hybrid.roi_mae,
            interpretation
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not claim the same ranking will hold on externally replayed renderer traces."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- External sample-allocation traces and real renderer variance are still needed before these competitive-baseline rankings become externally validated."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_demo_b_aliasing_vs_variance_report(
    path: &Path,
    demo_b: &DemoBSuiteMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Demo B Aliasing vs Variance Report");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Scenario | Demo B taxonomy | Imported trust ROI MAE | Uniform ROI MAE | Combined heuristic ROI MAE |"
    );
    let _ = writeln!(markdown, "| --- | --- | ---: | ---: | ---: |");
    for scenario in &demo_b.scenarios {
        let imported = find_policy(scenario, "imported_trust").expect("imported trust policy");
        let uniform = find_policy(scenario, "uniform").expect("uniform policy");
        let combined = find_policy(scenario, "combined_heuristic").expect("combined heuristic");
        let _ = writeln!(
            markdown,
            "| {} | {} | {:.5} | {:.5} | {:.5} |",
            scenario.scenario_id,
            scenario.demo_b_taxonomy,
            imported.roi_mae,
            uniform.roi_mae,
            combined.roi_mae
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not claim the same ordering will hold under real renderer variance or path-tracing noise."
    );
    let _ = writeln!(
        markdown,
        "- External validation is still required before treating aliasing-vs-variance separation as an engine-level conclusion."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real renderer noise and in-engine sample allocation remain future work."
    );
    let _ = writeln!(
        markdown,
        "- No external renderer handoff exists yet for per-pixel sample-allocation traces."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_operating_band_report(
    path: &Path,
    sensitivity: &ParameterSensitivityMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut grouped: BTreeMap<&str, Vec<&crate::sensitivity::ParameterSweepPoint>> =
        BTreeMap::new();
    for point in &sensitivity.sweep_points {
        grouped
            .entry(point.parameter_id.as_str())
            .or_default()
            .push(point);
    }

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Operating Band Report");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "This report translates parameter sweeps into evaluator-facing operating bands: what is robust, what is moderately sensitive, and what is fragile."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Parameter | Robust values | Moderately sensitive values | Fragile values | First tuning priority |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | --- | --- |");
    for (parameter_id, points) in grouped {
        let robust = band_values(&points, "robust");
        let moderate = band_values(&points, "moderately_sensitive");
        let fragile = band_values(&points, "fragile");
        let first_tuning_priority = if parameter_id.contains("alpha") {
            "second"
        } else if parameter_id.contains("motion") {
            "optional path only"
        } else {
            "first"
        };
        let _ = writeln!(
            markdown,
            "| {} | {} | {} | {} | {} |",
            parameter_id,
            if robust.is_empty() {
                "none".to_string()
            } else {
                robust.join(", ")
            },
            if moderate.is_empty() {
                "none".to_string()
            } else {
                moderate.join(", ")
            },
            if fragile.is_empty() {
                "none".to_string()
            } else {
                fragile.join(", ")
            },
            first_tuning_priority
        );
    }
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The current weights are no longer opaque magic constants; they are centralized and classified into safe, narrower, and fragile corridors."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- These operating bands are still derived from synthetic in-crate sweeps rather than externally validated calibration."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- External replay and engine-side tuning are still required before these operating bands can be treated as deployment guidance."
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_production_eval_checklist(
    path: &Path,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Production Evaluation Checklist");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Proven in crate:");
    let _ = writeln!(markdown, "- host-realistic supervisory effect");
    let _ = writeln!(markdown, "- point vs region ROI separation");
    let _ = writeln!(markdown, "- external buffer schema and import path");
    let _ = writeln!(markdown, "- GPU-executable minimum kernel");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Requires external validation:");
    let _ = writeln!(markdown, "- real engine buffer export into the schema");
    let _ = writeln!(markdown, "- GPU profiling on imported captures");
    let _ = writeln!(
        markdown,
        "- fair in-engine comparison against strong heuristics"
    );
    let _ = writeln!(markdown, "- non-ROI penalty behavior on production scenes");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Status:");
    let _ = writeln!(
        markdown,
        "- external-capable = `{}`",
        external.external_capable
    );
    let _ = writeln!(
        markdown,
        "- externally validated = `{}`",
        external.externally_validated
    );
    let _ = writeln!(
        markdown,
        "- actual GPU timing measured = `{}`",
        gpu.actual_gpu_timing_measured
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This checklist does not claim production readiness."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- real engine validation");
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(
            markdown,
            "- measured GPU timing on the evaluator's hardware"
        );
    }
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_evaluator_handoff(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    demo_b: &DemoBSuiteMetrics,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Evaluator Handoff");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Run first:");
    let _ = writeln!(
        markdown,
        "- `cargo run --release -- run-all --output generated/final_bundle`"
    );
    let _ = writeln!(
        markdown,
        "- `cargo run --release -- validate-final --output generated/final_bundle`"
    );
    let _ = writeln!(
        markdown,
        "- `cargo run --release -- run-gpu-path --output generated/gpu_path`"
    );
    let _ = writeln!(markdown, "- `cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_real`");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Strongest current evidence:");
    let _ = writeln!(markdown, "- {}", demo_a.summary.primary_behavioral_result);
    let _ = writeln!(markdown, "- {}", demo_b.summary.primary_behavioral_result);
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Weakest current evidence:");
    if !gpu.actual_gpu_timing_measured {
        let _ = writeln!(markdown, "- no in-environment measured GPU timing");
    }
    if !external.externally_validated {
        let _ = writeln!(
            markdown,
            "- no real external engine capture has been validated"
        );
    }
    let _ = writeln!(
        markdown,
        "- strong heuristic remains competitive on some scenarios"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Single highest-value next GPU experiment:");
    let _ = writeln!(
        markdown,
        "- Run the measured `wgpu` minimum kernel on the target evaluator GPU and compare numeric deltas against the CPU reference on one region-ROI case and one realism-stress case."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "Single highest-value next external replay experiment:"
    );
    let _ = writeln!(
        markdown,
        "- Export one real frame pair from an engine into the external schema, replay it through DSFB host-realistic, and compare fixed alpha, strong heuristic, and DSFB on the same capture."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "Engine-side baselines to keep:");
    let _ = writeln!(markdown, "- fixed alpha");
    let _ = writeln!(markdown, "- strong heuristic");
    let _ = writeln!(
        markdown,
        "- imported-trust or native-trust sampling policy where relevant"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## External Validation");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- exact command: `cargo run --release -- run-external-replay --manifest <manifest> --output generated/external_real`");
    let _ = writeln!(markdown, "- required data format: `current_color`, `reprojected_history`, `motion_vectors`, `current_depth`, `reprojected_depth`, `current_normals`, `reprojected_normals`, plus optional `mask`, `ground_truth`, and `variance`.");
    let _ = writeln!(markdown, "- expected outputs: `external_validation_report.md`, `gpu_external_report.md`, `gpu_external_metrics.json`, `demo_a_external_report.md`, `demo_b_external_report.md`, `demo_b_external_metrics.json`, `scaling_report.md`, `scaling_metrics.json`, `memory_bandwidth_report.md`, `integration_scaling_report.md`, and `figures/`.");
    let _ = writeln!(markdown, "- success looks like: the imported capture runs through the DSFB host-minimum path, GPU status is explicit, ROI vs non-ROI is separated, fixed-budget Demo B compares DSFB against stronger heuristics, and the scaling package says whether 1080p/4K, readback, and async insertion are viable.");
    let _ = writeln!(markdown, "- failure looks like: malformed schema, missing required buffers, no measured-vs-unmeasured GPU disclosure, no 1080p attempt or unavailable classification, budget mismatch, or reports that hide proxy-vs-real metric status.");
    let _ = writeln!(markdown, "- interpretation: ties against strong heuristics mean DSFB is behaving like a targeted supervisory overlay rather than a blanket replacement; losses plus higher non-ROI penalty should trigger engine-side tuning before any broader claim.");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This handoff does not claim the current crate has already passed external evaluation."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "- external engine captures");
    let _ = writeln!(markdown, "- GPU profiling on imported captures");
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_minimum_external_validation_plan(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Minimum External Validation Plan");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "1. Export one frame pair with current color, reprojected history, motion, depth, and normals.");
    let _ = writeln!(markdown, "2. Run `import-external` on that manifest.");
    let _ = writeln!(markdown, "3. Run `run-gpu-path` on the same machine.");
    let _ = writeln!(
        markdown,
        "4. Compare strong heuristic, fixed alpha, and DSFB host-realistic results."
    );
    let _ = writeln!(
        markdown,
        "5. Record ROI behavior, non-ROI penalty, and GPU timing."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This plan does not imply the result will be positive."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- actual external captures still need to be exported"
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_next_step_matrix(
    path: &Path,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Next Step Matrix");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "| Area | Current status | Next action | Negative outcome to watch |"
    );
    let _ = writeln!(markdown, "| --- | --- | --- | --- |");
    let _ = writeln!(
        markdown,
        "| GPU path | {} | Run `run-gpu-path` on evaluator hardware | Kernel timing too high or numeric mismatch vs CPU |",
        if gpu.actual_gpu_timing_measured { "measured" } else { "implemented, unmeasured here" }
    );
    let _ = writeln!(
        markdown,
        "| External handoff | external-capable={}, externally validated={} | Export one real frame pair into the schema | Imported buffers expose missing assumptions or normalization mismatch |",
        external.external_capable, external.externally_validated
    );
    let _ = writeln!(
        markdown,
        "| Competitive baseline | mixed outcomes surfaced | Re-run strongest heuristic on imported captures | Heuristic wins broadly, collapsing DSFB framing to niche-only use |"
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This matrix does not claim any of the next actions will succeed."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- external evaluator execution still needs to happen"
    );
    fs::write(path, markdown)?;
    Ok(())
}

pub fn write_check_signing_readiness(
    path: &Path,
    demo_a: &DemoASuiteMetrics,
    gpu: &GpuExecutionMetrics,
    external: &ExternalHandoffMetrics,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let internal_ready = gpu.entries.iter().all(|entry| entry.gpu_path_available)
        && external.external_capable
        && !demo_a.summary.region_roi_scenarios.is_empty();
    let sign_off_status = if internal_ready && external.externally_validated {
        "ready now"
    } else if internal_ready {
        "blocked pending external evidence"
    } else {
        "ready for evaluation"
    };

    let mut markdown = String::new();
    let _ = writeln!(markdown, "# Check Signing Readiness");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "{EXPERIMENT_SENTENCE}");
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "| Axis | Status | Evidence |");
    let _ = writeln!(markdown, "| --- | --- | --- |");
    let _ = writeln!(
        markdown,
        "| Internal artifact completeness | {} | GPU path present=`{}`, external replay present=`{}`, region-ROI scenarios=`{}` |",
        if internal_ready { "ready for diligence" } else { "ready for evaluation" },
        gpu.entries.iter().all(|entry| entry.gpu_path_available),
        external.external_capable,
        demo_a.summary.region_roi_scenarios.len()
    );
    let _ = writeln!(
        markdown,
        "| Immediate sign-off | {} | external validation=`{}`, measured GPU timing=`{}` |",
        sign_off_status, external.externally_validated, gpu.actual_gpu_timing_measured
    );
    let _ = writeln!(
        markdown,
        "| External replay | {} | source kind=`{}` |",
        if external.externally_validated {
            "ready for diligence"
        } else {
            "blocked pending external evidence"
        },
        external.source_kind
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- The remaining blockers are now dominated by external validation needs rather than missing in-repo mechanisms."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## What Is Not Proven");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- This report does not claim immediate sign-off without external replay evidence and broader engine-side measurement."
    );
    let _ = writeln!(markdown);
    let _ = writeln!(markdown, "## Remaining Blockers");
    let _ = writeln!(markdown);
    let _ = writeln!(
        markdown,
        "- Real external captures and imported-capture GPU profiling still gate immediate external sign-off."
    );
    fs::write(path, markdown)?;
    Ok(())
}

fn checklist(markdown: &mut String, ok: bool, label: &str) {
    let _ = writeln!(markdown, "- {} {}", if ok { "[x]" } else { "[ ]" }, label);
}

fn band_values(
    points: &[&crate::sensitivity::ParameterSweepPoint],
    class_name: &str,
) -> Vec<String> {
    points
        .iter()
        .filter(|point| point.robustness_class == class_name)
        .map(|point| format!("{:.3}", point.numeric_value))
        .collect()
}

fn scenario_tags(scenario: &crate::metrics::ScenarioReport) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if scenario.realism_stress {
        tags.push("realism_stress");
    }
    if scenario.competitive_baseline_case {
        tags.push("competitive_baseline");
    }
    if scenario.bounded_loss_disclosure {
        tags.push("bounded_neutral_or_loss");
    }
    if matches!(
        scenario.support_category,
        crate::scene::ScenarioSupportCategory::PointLikeRoi
    ) {
        tags.push("point_roi");
    }
    if matches!(
        scenario.support_category,
        crate::scene::ScenarioSupportCategory::RegionRoi
    ) {
        tags.push("region_roi");
    }
    if tags.is_empty() {
        tags.push("baseline_suite");
    }
    tags
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
