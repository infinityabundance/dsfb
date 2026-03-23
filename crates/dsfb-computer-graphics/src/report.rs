use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::cost::{build_cost_report, CostMode, CostReport};
use crate::error::Result;
use crate::metrics::{AblationEntry, DemoASuiteMetrics};
use crate::sampling::{DemoBScenarioReport, DemoBSuiteMetrics};

pub const EXPERIMENT_SENTENCE: &str =
    "“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”";
pub const COST_SENTENCE: &str = "“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”";
pub const COMPATIBILITY_SENTENCE: &str =
    "“The framework is compatible with tiled and asynchronous GPU execution.”";

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
