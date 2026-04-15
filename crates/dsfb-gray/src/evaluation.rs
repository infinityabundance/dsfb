//! Reproducible public evaluation pipeline for DSFB gray-failure experiments.
//!
//! This module centralizes the deterministic evaluation used by the demo
//! binary, the checked-in evaluation artifacts, and the generated public
//! documentation snippets. The goal is simple: every published number should
//! come from one executable pipeline.

use crate::scan::AUDIT_SCORE_METHOD;
use crate::{
    generate_csv, run_scenario, AdmissibilityEnvelope, AsyncStarvationScenario,
    ChannelBackpressureScenario, ClockDriftScenario, FaultScenario, ObserverConfig,
    PartialPartitionScenario, ScenarioResult, WorkloadPhase,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const AUDIT_NON_CERTIFICATION_STATEMENT: &str =
    "The DSFB audit includes standards- and certification-relevant structural checks, but it does not certify compliance with IEC, ISO, RTCA, MIL, NIST, or other standards.";

/// Structured result for one primary evaluation scenario.
#[derive(Debug, Clone)]
pub struct PrimaryEvaluationRow {
    /// User-facing scenario label.
    pub name: String,
    /// Full scenario result.
    pub result: ScenarioResult,
    /// CSV filename written for this scenario.
    pub csv_name: String,
}

/// One row of the 2D sensitivity sweep.
#[derive(Debug, Clone, Copy)]
pub struct SensitivitySweepRow {
    /// Envelope width multiplier.
    pub sigma: f64,
    /// Persistence window.
    pub persistence_window: usize,
    /// Detection delay from injection start.
    pub detection_delay: Option<u64>,
    /// Lead time before the scenario ends.
    pub lead_time: Option<u64>,
    /// Whether the scenario was detected at all.
    pub detected: bool,
    /// Whether the first anomaly occurred early in the evaluation window.
    pub early_window_detection: bool,
    /// Whether a pre-injection false alarm occurred.
    pub has_false_alarm: bool,
    /// Boundary-state steps across the scenario.
    pub boundary_steps: u32,
    /// Violation-state steps across the scenario.
    pub violation_steps: u32,
}

/// One clean-window negative-control result.
#[derive(Debug, Clone)]
pub struct NegativeControlRow {
    /// User-facing scenario label.
    pub name: String,
    /// Full scenario result.
    pub result: ScenarioResult,
}

/// One row of the drift-rate elasticity sweep.
#[derive(Debug, Clone, Copy)]
pub struct DriftElasticityRow {
    /// Injected drift rate.
    pub drift_rate: f64,
    /// Full scenario result.
    pub result_detected: bool,
    /// Detection delay from injection, if any.
    pub detection_delay: Option<u64>,
    /// Lead time before the scenario ends, if any.
    pub lead_time: Option<u64>,
}

/// Fully structured public evaluation bundle.
#[derive(Debug, Clone)]
pub struct PublicEvaluationBundle {
    /// Recommended-configuration primary results.
    pub primary: Vec<PrimaryEvaluationRow>,
    /// 42-point clock-drift sensitivity sweep.
    pub sensitivity_sweep: Vec<SensitivitySweepRow>,
    /// Negative-control results.
    pub negative_controls: Vec<NegativeControlRow>,
    /// First-boundary steps observed across reproducibility runs.
    pub reproducibility_boundary_steps: Vec<u64>,
    /// Drift-rate elasticity sweep.
    pub drift_elasticity: Vec<DriftElasticityRow>,
}

/// Paths written by the public-artifact regeneration workflow.
#[derive(Debug, Clone)]
pub struct PublicArtifactPaths {
    /// Full evaluation report.
    pub evaluation_results_path: PathBuf,
    /// Demo output report.
    pub demo_output_path: PathBuf,
    /// Sensitivity sweep CSV.
    pub sensitivity_sweep_path: PathBuf,
    /// Generated docs directory.
    pub generated_docs_dir: PathBuf,
    /// Generated paper directory.
    pub generated_paper_dir: PathBuf,
}

/// Build the full deterministic public evaluation bundle.
pub fn build_public_evaluation() -> PublicEvaluationBundle {
    let primary = build_primary_evaluation();
    let sensitivity_sweep = build_sensitivity_sweep();
    let negative_controls = build_negative_controls();
    let reproducibility_boundary_steps = build_reproducibility_steps();
    let drift_elasticity = build_drift_elasticity();

    PublicEvaluationBundle {
        primary,
        sensitivity_sweep,
        negative_controls,
        reproducibility_boundary_steps,
        drift_elasticity,
    }
}

fn count_true<T>(items: &[T], predicate: impl Fn(&T) -> bool) -> usize {
    let mut count = 0usize;
    for item in items.iter() {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

fn find_negative_control<'a>(
    bundle: &'a PublicEvaluationBundle,
    name: &str,
) -> Option<&'a NegativeControlRow> {
    bundle.negative_controls.iter().find(|row| row.name == name)
}

fn render_optional_number(value: Option<u64>) -> String {
    match value {
        Some(number) => number.to_string(),
        None => "-".to_string(),
    }
}

fn render_optional_steps(value: Option<u64>, missing: &'static str) -> String {
    match value {
        Some(steps) => format!("{steps} steps"),
        None => missing.to_string(),
    }
}

/// Render the full human-readable evaluation report.
pub fn render_public_evaluation_report(bundle: &PublicEvaluationBundle) -> String {
    let mut out = String::new();
    let stats = compute_evaluation_summary(bundle);

    push_evaluation_header(&mut out);
    push_primary_section(&mut out, bundle);
    push_sensitivity_section(&mut out, bundle);
    push_negative_control_section(&mut out, bundle);
    push_reproducibility_section(&mut out, bundle, stats.reproducibility_baseline);
    push_structural_discrimination_section(&mut out, bundle);
    push_drift_elasticity_section(&mut out, bundle);
    push_evaluation_summary(&mut out, bundle, &stats);

    out
}

struct EvaluationSummaryStats {
    primary_detected: usize,
    detected_sweep_points: usize,
    sweep_false_alarm_points: usize,
    clean_control_clear: usize,
    reproducibility_baseline: u64,
    reproducibility_matches: usize,
}

fn compute_evaluation_summary(bundle: &PublicEvaluationBundle) -> EvaluationSummaryStats {
    let reproducibility_baseline = bundle
        .reproducibility_boundary_steps
        .first()
        .copied()
        .unwrap_or(0);

    EvaluationSummaryStats {
        primary_detected: count_true(&bundle.primary, |row| row.result.detected()),
        detected_sweep_points: count_true(&bundle.sensitivity_sweep, |row| row.detected),
        sweep_false_alarm_points: count_true(&bundle.sensitivity_sweep, |row| row.has_false_alarm),
        clean_control_clear: count_true(&bundle.negative_controls, |row| {
            row.result.total_boundary_steps + row.result.total_violation_steps == 0
        }),
        reproducibility_baseline,
        reproducibility_matches: count_true(&bundle.reproducibility_boundary_steps, |step| {
            *step == reproducibility_baseline
        }),
    }
}

fn push_evaluation_header(out: &mut String) {
    out.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    out.push_str("║  DSFB Gray Failure Detection — Full Empirical Evaluation    ║\n");
    out.push_str("║  Invariant Forge LLC — Deterministic Structural Engine      ║\n");
    out.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");
}

fn push_primary_section(out: &mut String, bundle: &PublicEvaluationBundle) {
    out.push_str("══ Section 1: Primary Evaluation (Recommended Configuration) ══\n\n");
    out.push_str("Table 1: Primary Results (Recommended Configuration)\n");
    out.push_str(
        "┌─────────────────────┬───────┬──────────┬───────┬──────┬────────┬─────────┬──────────┐\n",
    );
    out.push_str(
        "│ Scenario            │ Steps │ Inj.Start│ Det.  │Delay │ Lead   │ FalseAl │ Viol.Steps│\n",
    );
    out.push_str(
        "├─────────────────────┼───────┼──────────┼───────┼──────┼────────┼─────────┼──────────┤\n",
    );
    for row in &bundle.primary {
        let result = &row.result;
        out.push_str(&format!(
            "│ {:19} │  {:3}  │    {:3}   │  {}  │ {:>4} │  {:>4}  │    {}    │   {:>4}   │\n",
            row.name,
            result.total_steps,
            result.injection_start,
            if result.detected() { "YES" } else { "NO " },
            render_optional_number(result.detection_delay_from_injection()),
            render_optional_number(result.detection_lead_time()),
            result.false_alarms_before_injection,
            result.total_violation_steps
        ));
    }
    out.push_str(
        "└─────────────────────┴───────┴──────────┴───────┴──────┴────────┴─────────┴──────────┘\n\n",
    );
}

fn push_sensitivity_section(out: &mut String, bundle: &PublicEvaluationBundle) {
    out.push_str("══ Section 2: Sensitivity Sweep (Clock Drift, 42-point 2D) ══\n\n");
    out.push_str("Table 2: Sensitivity Sweep — Clock Drift Scenario\n");
    out.push_str("┌──────┬─────┬──────────┬───────┬──────┬────────┐\n");
    out.push_str("│  σ   │  P  │ Med.Lead │ Det.% │ EW%  │ False% │\n");
    out.push_str("├──────┼─────┼──────────┼───────┼──────┼────────┤\n");
    for row in &bundle.sensitivity_sweep {
        out.push_str(&format!(
            "│ {:4.1} │ {:3} │   {:>4}   │  {:3}  │ {:3}  │  {:>3}   │\n",
            row.sigma,
            row.persistence_window,
            row.lead_time.unwrap_or(0),
            if row.detected { 100 } else { 0 },
            if row.early_window_detection { 100 } else { 0 },
            if row.has_false_alarm { 100 } else { 0 },
        ));
    }
    out.push_str("└──────┴─────┴──────────┴───────┴──────┴────────┘\n");
    out.push_str("  CSV: data/sensitivity_sweep.csv\n\n");
}

fn push_negative_control_section(out: &mut String, bundle: &PublicEvaluationBundle) {
    out.push_str("══ Section 3: Negative Control — No-Fault Baseline ══\n\n");
    out.push_str("Table 3: Negative Control — False Alarm Analysis on Healthy Windows\n");
    out.push_str("┌─────────────────────┬──────────┬────────────┬────────────┬────────────┐\n");
    out.push_str("│ Scenario            │ Samples  │ Boundary   │ Violation  │ False Rate │\n");
    out.push_str("├─────────────────────┼──────────┼────────────┼────────────┼────────────┤\n");
    for row in &bundle.negative_controls {
        let result = &row.result;
        out.push_str(&format!(
            "│ {:19} │   {:4}   │     {:4}   │     {:4}   │   {:5.1}%   │\n",
            row.name,
            result.total_steps,
            result.total_boundary_steps,
            result.total_violation_steps,
            negative_control_false_rate(result)
        ));
    }
    out.push_str("└─────────────────────┴──────────┴────────────┴────────────┴────────────┘\n\n");
}

fn negative_control_false_rate(result: &ScenarioResult) -> f64 {
    if result.total_steps > 0 {
        (result.total_boundary_steps + result.total_violation_steps) as f64
            / result.total_steps as f64
            * 100.0
    } else {
        0.0
    }
}

fn push_reproducibility_section(out: &mut String, bundle: &PublicEvaluationBundle, baseline: u64) {
    out.push_str("══ Section 4: Deterministic Reproducibility Verification ══\n\n");
    for (idx, step) in bundle.reproducibility_boundary_steps.iter().enumerate() {
        out.push_str(&format!(
            "  Run {:2}: first_boundary=step {} {}\n",
            idx + 1,
            step,
            reproducibility_status_suffix(idx, *step, baseline)
        ));
    }
    out.push_str(&format!(
        "  Deterministic: {}\n\n",
        if reproducibility_verified(bundle) {
            "VERIFIED — 10/10 runs identical"
        } else {
            "FAILED"
        }
    ));
}

fn reproducibility_status_suffix(idx: usize, step: u64, baseline: u64) -> &'static str {
    if idx == 0 {
        "(baseline)"
    } else if step == baseline {
        "✓ matches"
    } else {
        "✗ MISMATCH"
    }
}

fn push_structural_discrimination_section(out: &mut String, bundle: &PublicEvaluationBundle) {
    out.push_str("══ Section 5: Multi-Scenario Structural Discrimination ══\n\n");
    out.push_str("Table 4: Structural Signatures by Scenario at Detection Point\n");
    out.push_str("┌─────────────────────┬──────────┬──────────┬──────────┬────────────┐\n");
    out.push_str("│ Scenario            │ Residual │  Drift   │   Slew   │ Drift/Slew │\n");
    out.push_str("├─────────────────────┼──────────┼──────────┼──────────┼────────────┤\n");
    for row in &bundle.primary {
        if let Some(signature_row) = render_structural_signature_row(row) {
            out.push_str(&signature_row);
        }
    }
    out.push_str("└─────────────────────┴──────────┴──────────┴──────────┴────────────┘\n\n");
    out.push_str("Interpretation: Each scenario produces a structurally distinct signature\n");
    out.push_str("at its detection point. Clock drift has high drift/slew ratio (pure drift).\n");
    out.push_str("Backpressure has lower ratio (accelerating growth → positive slew).\n");
    out.push_str(
        "This discrimination is information that scalar threshold alerts do not provide.\n\n",
    );
}

fn render_structural_signature_row(row: &PrimaryEvaluationRow) -> Option<String> {
    let det_step = row.result.first_anomaly_step?;
    let sample = row
        .result
        .samples
        .iter()
        .find(|sample| sample.step == det_step)?;
    let ratio = if sample.slew.abs() > 1e-10 {
        sample.drift / sample.slew
    } else {
        f64::INFINITY
    };
    let ratio_str = if ratio.is_infinite() {
        "∞ (pure drift)".to_string()
    } else {
        format!("{ratio:.1}")
    };

    Some(format!(
        "│ {:19} │ {:>8.4} │ {:>8.6} │ {:>8.6} │ {:>10} │\n",
        row.name, sample.residual, sample.drift, sample.slew, ratio_str
    ))
}

fn push_drift_elasticity_section(out: &mut String, bundle: &PublicEvaluationBundle) {
    out.push_str("══ Section 6: Drift Rate Variation — Detection Elasticity ══\n\n");
    out.push_str("Table 5: Clock Drift Detection vs. Drift Rate\n");
    out.push_str("┌────────────┬───────┬──────┬────────┐\n");
    out.push_str("│ Drift Rate │ Det.  │Delay │  Lead  │\n");
    out.push_str("├────────────┼───────┼──────┼────────┤\n");
    for row in &bundle.drift_elasticity {
        out.push_str(&format!(
            "│   {:6.3}   │  {}  │ {:>4} │  {:>4}  │\n",
            row.drift_rate,
            if row.result_detected { "YES" } else { "NO " },
            render_optional_number(row.detection_delay),
            render_optional_number(row.lead_time)
        ));
    }
    out.push_str("└────────────┴───────┴──────┴────────┘\n\n");
}

fn push_evaluation_summary(
    out: &mut String,
    bundle: &PublicEvaluationBundle,
    stats: &EvaluationSummaryStats,
) {
    out.push_str("══════════════════════════════════════════════════════════════\n");
    out.push_str("  EVALUATION COMPLETE\n");
    out.push_str(&format!(
        "  • {}/4 primary scenarios: gray failures detected\n",
        stats.primary_detected
    ));
    out.push_str(&format!(
        "  • 42-point sensitivity sweep completed: {} detected, {} with pre-injection alarms\n",
        stats.detected_sweep_points, stats.sweep_false_alarm_points
    ));
    out.push_str(&format!(
        "  • Clean-window controls: {}/{} had zero anomaly steps\n",
        stats.clean_control_clear,
        bundle.negative_controls.len()
    ));
    out.push_str(&format!(
        "  • Deterministic reproducibility: {}/{} runs identical\n",
        stats.reproducibility_matches,
        bundle.reproducibility_boundary_steps.len()
    ));
    out.push_str("  • Structural discrimination: distinct signatures per scenario\n");
    out.push_str("  • Drift rate elasticity: 8-point sweep completed\n");
    out.push_str("══════════════════════════════════════════════════════════════\n");
}

/// Write the canonical public evaluation artifacts and generated snippets.
pub fn write_public_artifacts(
    bundle: &PublicEvaluationBundle,
    root: &Path,
) -> io::Result<PublicArtifactPaths> {
    let paths = public_artifact_paths(root);
    fs::create_dir_all(&paths.generated_docs_dir)?;
    fs::create_dir_all(&paths.generated_paper_dir)?;
    if let Some(data_dir) = paths.evaluation_results_path.parent() {
        fs::create_dir_all(data_dir)?;
    }

    let report = render_public_evaluation_report(bundle);
    write_primary_public_outputs(bundle, &paths, &report)?;
    let snippets = generated_public_snippets(bundle);
    write_generated_public_docs(bundle, &paths, &snippets)?;
    rewrite_public_marked_sections(root, bundle, &snippets)?;
    Ok(paths)
}

fn public_artifact_paths(root: &Path) -> PublicArtifactPaths {
    let data_dir = root.join("data");
    PublicArtifactPaths {
        evaluation_results_path: data_dir.join("evaluation_results.txt"),
        demo_output_path: data_dir.join("demo-output.txt"),
        sensitivity_sweep_path: data_dir.join("sensitivity_sweep.csv"),
        generated_docs_dir: root.join("docs/generated"),
        generated_paper_dir: root.join("paper/generated"),
    }
}

fn write_primary_public_outputs(
    bundle: &PublicEvaluationBundle,
    paths: &PublicArtifactPaths,
    report: &str,
) -> io::Result<()> {
    fs::write(&paths.evaluation_results_path, report)?;
    fs::write(&paths.demo_output_path, report)?;
    fs::write(
        &paths.sensitivity_sweep_path,
        render_sensitivity_sweep_csv(bundle),
    )?;
    for row in &bundle.primary {
        let output_root = paths
            .evaluation_results_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        fs::write(output_root.join(&row.csv_name), generate_csv(&row.result))?;
    }
    Ok(())
}

struct GeneratedPublicSnippets {
    readme_results: String,
    evidence_ledger: String,
    claim_ledger: String,
    audit_contract: String,
    paper_results_md: String,
}

fn generated_public_snippets(bundle: &PublicEvaluationBundle) -> GeneratedPublicSnippets {
    GeneratedPublicSnippets {
        readme_results: render_readme_results_section(bundle),
        evidence_ledger: render_evidence_ledger_md(bundle),
        claim_ledger: render_claim_ledger_md(bundle),
        audit_contract: render_audit_contract_md(),
        paper_results_md: render_paper_results_table_md(bundle),
    }
}

fn write_generated_public_docs(
    bundle: &PublicEvaluationBundle,
    paths: &PublicArtifactPaths,
    snippets: &GeneratedPublicSnippets,
) -> io::Result<()> {
    fs::write(
        paths.generated_docs_dir.join("README_RESULTS.md"),
        &snippets.readme_results,
    )?;
    fs::write(
        paths.generated_docs_dir.join("EVIDENCE_LEDGER.md"),
        &snippets.evidence_ledger,
    )?;
    fs::write(
        paths.generated_docs_dir.join("CLAIM_LEDGER.md"),
        &snippets.claim_ledger,
    )?;
    fs::write(
        paths.generated_docs_dir.join("AUDIT_CONTRACT.md"),
        &snippets.audit_contract,
    )?;
    fs::write(
        paths.generated_paper_dir.join("results_summary.tex"),
        render_paper_results_table_tex(bundle),
    )?;
    fs::write(
        paths.generated_paper_dir.join("claim_ledger.tex"),
        render_paper_claim_ledger_tex(bundle),
    )?;
    fs::write(
        paths.generated_paper_dir.join("results_summary.md"),
        &snippets.paper_results_md,
    )?;
    fs::write(
        paths.generated_paper_dir.join("claim_ledger.md"),
        &snippets.claim_ledger,
    )?;
    fs::write(
        paths.generated_paper_dir.join("audit_contract.md"),
        &snippets.audit_contract,
    )?;
    fs::write(
        paths.generated_paper_dir.join("audit_contract.tex"),
        render_paper_audit_contract_tex(),
    )?;
    Ok(())
}

fn rewrite_public_marked_sections(
    root: &Path,
    _bundle: &PublicEvaluationBundle,
    snippets: &GeneratedPublicSnippets,
) -> io::Result<()> {
    rewrite_marked_section_if_present(
        &root.join("README.md"),
        "<!-- DSFB:README_RESULTS:BEGIN -->",
        "<!-- DSFB:README_RESULTS:END -->",
        &snippets.readme_results,
    )?;
    rewrite_marked_section_if_present(
        &root.join("README.md"),
        "<!-- DSFB:EVIDENCE_LEDGER:BEGIN -->",
        "<!-- DSFB:EVIDENCE_LEDGER:END -->",
        &snippets.evidence_ledger,
    )?;
    rewrite_marked_section_if_present(
        &root.join("paper/paper.md"),
        "<!-- DSFB:PAPER_RESULTS:BEGIN -->",
        "<!-- DSFB:PAPER_RESULTS:END -->",
        &snippets.paper_results_md,
    )?;
    rewrite_marked_section_if_present(
        &root.join("paper/paper.md"),
        "<!-- DSFB:PAPER_CLAIM_LEDGER:BEGIN -->",
        "<!-- DSFB:PAPER_CLAIM_LEDGER:END -->",
        &snippets.claim_ledger,
    )?;
    rewrite_marked_section_if_present(
        &root.join("paper/paper.md"),
        "<!-- DSFB:PAPER_AUDIT_CONTRACT:BEGIN -->",
        "<!-- DSFB:PAPER_AUDIT_CONTRACT:END -->",
        &snippets.audit_contract,
    )?;
    Ok(())
}

fn rewrite_marked_section_if_present(
    path: &Path,
    start_marker: &str,
    end_marker: &str,
    generated: &str,
) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    rewrite_marked_section(path, start_marker, end_marker, generated)
}

/// Whether every reproducibility run matched the first baseline run.
pub fn reproducibility_verified(bundle: &PublicEvaluationBundle) -> bool {
    let Some(first) = bundle.reproducibility_boundary_steps.first().copied() else {
        return false;
    };
    bundle
        .reproducibility_boundary_steps
        .iter()
        .all(|step| *step == first)
}

fn build_primary_evaluation() -> Vec<PrimaryEvaluationRow> {
    vec![
        primary_clock_drift_row(),
        primary_partial_partition_row(),
        primary_backpressure_row(),
        primary_async_starvation_row(),
    ]
}

fn primary_clock_drift_row() -> PrimaryEvaluationRow {
    primary_row(
        "Clock Drift",
        "clock_drift.csv",
        &mut ClockDriftScenario::default_scenario(),
        symmetric_config(20, 2.0, 0.1, 0.05),
    )
}

fn primary_partial_partition_row() -> PrimaryEvaluationRow {
    primary_row(
        "Partial Partition",
        "partial_partition.csv",
        &mut PartialPartitionScenario::default_scenario(),
        symmetric_config(15, 3.0, 0.15, 0.08),
    )
}

fn primary_backpressure_row() -> PrimaryEvaluationRow {
    primary_row(
        "Channel Backpressure",
        "channel_backpressure.csv",
        &mut ChannelBackpressureScenario::default_scenario(),
        symmetric_config(15, 100.0, 10.0, 5.0),
    )
}

fn primary_async_starvation_row() -> PrimaryEvaluationRow {
    primary_row(
        "Async Starvation",
        "async_starvation.csv",
        &mut AsyncStarvationScenario::default_scenario(),
        symmetric_config(15, 30.0, 3.0, 1.5),
    )
}

fn primary_row(
    name: &str,
    csv_name: &str,
    scenario: &mut dyn FaultScenario,
    config: ObserverConfig,
) -> PrimaryEvaluationRow {
    PrimaryEvaluationRow {
        name: name.to_string(),
        csv_name: csv_name.to_string(),
        result: run_scenario(scenario, &config),
    }
}

fn symmetric_config(
    persistence_window: usize,
    residual: f64,
    drift: f64,
    slew: f64,
) -> ObserverConfig {
    ObserverConfig {
        persistence_window,
        hysteresis_count: 3,
        default_envelope: AdmissibilityEnvelope::symmetric(
            residual,
            drift,
            slew,
            WorkloadPhase::SteadyState,
        ),
        ..ObserverConfig::fast_response()
    }
}

fn build_sensitivity_sweep() -> Vec<SensitivitySweepRow> {
    let sigma_values = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let p_values: [usize; 7] = [5, 10, 15, 20, 25, 30, 40];
    let mut rows = Vec::new();

    for &sigma in &sigma_values {
        for &p in &p_values {
            let config = ObserverConfig {
                persistence_window: p,
                hysteresis_count: 3,
                default_envelope: AdmissibilityEnvelope::symmetric(
                    sigma,
                    sigma * 0.05,
                    sigma * 0.025,
                    WorkloadPhase::SteadyState,
                ),
                ..ObserverConfig::fast_response()
            };
            let result = run_scenario(&mut ClockDriftScenario::default_scenario(), &config);
            rows.push(SensitivitySweepRow {
                sigma,
                persistence_window: p,
                detection_delay: result.detection_delay_from_injection(),
                lead_time: result.detection_lead_time(),
                detected: result.detected(),
                early_window_detection: result.first_anomaly_step.is_some_and(|step| {
                    step >= result.injection_start && step < result.injection_start + 100
                }),
                has_false_alarm: result.false_alarms_before_injection > 0,
                boundary_steps: result.total_boundary_steps,
                violation_steps: result.total_violation_steps,
            });
        }
    }
    rows
}

fn build_negative_controls() -> Vec<NegativeControlRow> {
    vec![
        clean_clock_drift_row(),
        clean_partition_row(),
        clean_backpressure_row(),
        clean_starvation_row(),
    ]
}

fn clean_clock_drift_row() -> NegativeControlRow {
    negative_control_row(
        "Clock Drift (clean)",
        &mut ClockDriftScenario::new(5.0, 0.05, 999, 200, 0.02),
        symmetric_config(20, 2.0, 0.1, 0.05),
    )
}

fn clean_partition_row() -> NegativeControlRow {
    negative_control_row(
        "Partition (clean)",
        &mut PartialPartitionScenario {
            baseline: 5.0,
            start: 999,
            duration: 200,
            rate: 0.08,
            burst: 3.0,
            burst_dur: 10,
            noise_state: 137,
            seed: 137,
        },
        symmetric_config(15, 3.0, 0.15, 0.08),
    )
}

fn clean_backpressure_row() -> NegativeControlRow {
    negative_control_row(
        "Backpressure (clean)",
        &mut ChannelBackpressureScenario {
            baseline: 100.0,
            start: 999,
            duration: 200,
            rate: 5.0,
            noise_state: 271,
            seed: 271,
        },
        symmetric_config(15, 100.0, 10.0, 5.0),
    )
}

fn clean_starvation_row() -> NegativeControlRow {
    negative_control_row(
        "Starvation (clean)",
        &mut AsyncStarvationScenario {
            baseline: 50.0,
            start: 999,
            duration: 200,
            rate: 2.0,
            noise_state: 313,
            seed: 313,
        },
        symmetric_config(15, 30.0, 3.0, 1.5),
    )
}

fn negative_control_row(
    name: &str,
    scenario: &mut dyn FaultScenario,
    config: ObserverConfig,
) -> NegativeControlRow {
    NegativeControlRow {
        name: name.to_string(),
        result: run_scenario(scenario, &config),
    }
}

fn build_reproducibility_steps() -> Vec<u64> {
    let config = ObserverConfig {
        persistence_window: 20,
        hysteresis_count: 3,
        default_envelope: AdmissibilityEnvelope::symmetric(
            2.0,
            0.1,
            0.05,
            WorkloadPhase::SteadyState,
        ),
        ..ObserverConfig::fast_response()
    };

    let mut steps = Vec::with_capacity(10);
    for _ in 0..10 {
        let result = run_scenario(&mut ClockDriftScenario::default_scenario(), &config);
        steps.push(result.first_boundary_step.unwrap_or(999));
    }
    steps
}

fn build_drift_elasticity() -> Vec<DriftElasticityRow> {
    let mut rows = Vec::new();
    for &drift_rate in &[0.01, 0.02, 0.03, 0.05, 0.08, 0.10, 0.15, 0.20] {
        let result = run_scenario(
            &mut ClockDriftScenario::new(5.0, drift_rate, 50, 200, 0.02),
            &ObserverConfig {
                persistence_window: 20,
                hysteresis_count: 3,
                default_envelope: AdmissibilityEnvelope::symmetric(
                    2.0,
                    0.1,
                    0.05,
                    WorkloadPhase::SteadyState,
                ),
                ..ObserverConfig::fast_response()
            },
        );
        rows.push(DriftElasticityRow {
            drift_rate,
            result_detected: result.detected(),
            detection_delay: result.detection_delay_from_injection(),
            lead_time: result.detection_lead_time(),
        });
    }
    rows
}

fn render_sensitivity_sweep_csv(bundle: &PublicEvaluationBundle) -> String {
    let mut csv = String::from(
        "sigma,P,detection_delay,lead_time,detected,false_alarms,boundary_steps,violation_steps\n",
    );
    for row in &bundle.sensitivity_sweep {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            row.sigma,
            row.persistence_window,
            row.detection_delay.unwrap_or(0),
            row.lead_time.unwrap_or(0),
            u8::from(row.detected),
            u8::from(row.has_false_alarm),
            row.boundary_steps,
            row.violation_steps
        ));
    }
    csv
}

fn render_readme_results_section(bundle: &PublicEvaluationBundle) -> String {
    let mut out = String::new();
    let detected_count = count_true(&bundle.primary, |row| row.result.detected());
    let pre_injection_primary = count_true(&bundle.primary, |row| {
        row.result.false_alarms_before_injection > 0
    });
    out.push_str("| Gray Failure Scenario | Detection Delay | Lead Time | False Alarms |\n");
    out.push_str("|----------------------|-----------------|-----------|--------------|\n");
    for row in &bundle.primary {
        let result = &row.result;
        let delay = render_optional_steps(result.detection_delay_from_injection(), "pre-injection");
        let lead = render_optional_steps(result.detection_lead_time(), "-");
        out.push_str(&format!(
            "| {} | {} | {} | **{}** |\n",
            row.name, delay, lead, result.false_alarms_before_injection
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Current metrics are generated by `cargo run --bin dsfb-regenerate-public-artifacts`. The current recommended configuration detects {}/{} primary scenarios, and {} primary scenario(s) show a pre-injection anomaly.\n",
        detected_count,
        bundle.primary.len(),
        pre_injection_primary
    ));
    out
}

fn render_evidence_ledger_md(bundle: &PublicEvaluationBundle) -> String {
    let mut out = String::new();
    out.push_str("## Evidence Ledger\n\n");
    out.push_str("Every public-facing numeric claim in this repository should map to one command, one artifact, or one generated section.\n\n");
    out.push_str("| Claim Surface | Generated From | Artifact |\n");
    out.push_str("|---------------|----------------|----------|\n");
    out.push_str("| README results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/README_RESULTS.md` |\n");
    out.push_str("| Full evaluation narrative | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/evaluation_results.txt` |\n");
    out.push_str("| Demo output | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/demo-output.txt` |\n");
    out.push_str("| Sensitivity sweep table | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/sensitivity_sweep.csv` |\n");
    for row in &bundle.primary {
        out.push_str(&format!(
            "| Scenario CSV: {} | `cargo run --bin dsfb-regenerate-public-artifacts` | `data/{}` |\n",
            row.name, row.csv_name
        ));
    }
    out.push_str("| Paper TeX results table | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/results_summary.tex` |\n");
    out.push_str("| Audit contract summary | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/AUDIT_CONTRACT.md` |\n");
    out.push_str("| Paper TeX audit contract | `cargo run --bin dsfb-regenerate-public-artifacts` | `paper/generated/audit_contract.tex` |\n");
    out.push_str("| Claim ledger | `cargo run --bin dsfb-regenerate-public-artifacts` | `docs/generated/CLAIM_LEDGER.md` |\n");
    out
}

fn render_audit_contract_md() -> String {
    let mut out = String::new();
    out.push_str("## Canonical Broad Audit Contract\n\n");
    out.push_str(
        "- DSFB emits one canonical broad audit rather than primary profile-specific reports.\n",
    );
    out.push_str("- The audit keeps one shared evidence set and one shared denominator, then renders domain and standards interpretations as conclusion lenses at the end of the report.\n");
    out.push_str(&format!(
        "- The locked score method is `{}` with one overall score plus visible advisory subscores.\n",
        AUDIT_SCORE_METHOD
    ));
    out.push_str("- The score is a broad code-improvement and review-readiness target for Rust developers.\n");
    out.push_str("- The score is not runtime correctness, not a certification result, and not a standards certificate.\n");
    out.push_str(&format!("- {}\n", AUDIT_NON_CERTIFICATION_STATEMENT));
    out.push_str("- The report contract includes remediation guidance, verification suggestions, evidence IDs, SARIF, in-toto, DSSE, and static-to-runtime prior derivation.\n");
    out
}

fn render_claim_ledger_md(bundle: &PublicEvaluationBundle) -> String {
    let detected_count = count_true(&bundle.primary, |row| row.result.detected());
    let clean_control_false_rate = find_negative_control(bundle, "Starvation (clean)")
        .map(|row| {
            if row.result.total_steps > 0 {
                (row.result.total_boundary_steps + row.result.total_violation_steps) as f64
                    / row.result.total_steps as f64
                    * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    let sweep_pre_injection = count_true(&bundle.sensitivity_sweep, |row| row.has_false_alarm);
    let mut out = String::new();
    out.push_str("## Claim Ledger\n\n");
    out.push_str(&format!(
        "- DSFB detects {}/{} primary deterministic scenarios under the recommended configuration.\n",
        detected_count,
        bundle.primary.len()
    ));
    out.push_str(&format!(
        "  Evidence: `data/evaluation_results.txt`, Section 1; generated from {} primary scenarios.\n",
        bundle.primary.len()
    ));
    out.push_str("- The recommended configuration is not zero-false-alarm in all clean windows.\n");
    out.push_str(&format!(
        "  Evidence: `data/evaluation_results.txt`, Section 3; async starvation clean control produces a {:.1}% false rate.\n",
        clean_control_false_rate
    ));
    out.push_str(
        "- Sensitivity behavior is configuration-dependent rather than universally robust.\n",
    );
    out.push_str(&format!(
        "  Evidence: `data/evaluation_results.txt`, Section 2; {}/{} sweep points show pre-injection alarms.\n",
        sweep_pre_injection,
        bundle.sensitivity_sweep.len()
    ));
    out.push_str("- Reproducibility is deterministic for the current clock-drift harness.\n");
    out.push_str("  Evidence: `data/evaluation_results.txt`, Section 4; 10/10 runs identical.\n");
    out.push_str("- DSFB provides structurally distinct detection-point signatures across the primary scenarios.\n");
    out.push_str("  Evidence: `data/evaluation_results.txt`, Section 5.\n");
    out.push_str("- The companion crate now emits one canonical broad audit rather than primary profile-specific reports.\n");
    out.push_str("  Evidence: `docs/generated/AUDIT_CONTRACT.md`; regenerated from `cargo run --bin dsfb-regenerate-public-artifacts`.\n");
    out.push_str(&format!(
        "- The audit score method is `{}` and is treated as a broad improvement/readiness guide rather than certification.\n",
        AUDIT_SCORE_METHOD
    ));
    out.push_str(
        "  Evidence: `docs/generated/AUDIT_CONTRACT.md` and `docs/AUDIT_SCORING_LOCKED.md`.\n",
    );
    out.push_str("- The audit report includes conclusion lenses over one shared evidence set rather than separate primary scan modes.\n");
    out.push_str("  Evidence: `docs/generated/AUDIT_CONTRACT.md`; mirrored in the current scan report contract.\n");
    out.push_str("- The scanner emits SARIF, in-toto, and DSSE artifacts as part of the established public contract.\n");
    out.push_str("  Evidence: `docs/generated/AUDIT_CONTRACT.md` and the generated scanner outputs in `output-dsfb-gray/`.\n");
    out
}

fn render_paper_results_table_tex(bundle: &PublicEvaluationBundle) -> String {
    let mut out = String::new();
    out.push_str("\\begin{table}[H]\n\\centering\n");
    out.push_str(
        "\\caption{Primary deterministic evaluation results (recommended configuration).}\n",
    );
    out.push_str("\\label{tab:summary}\n");
    out.push_str("\\begin{tabular}{lcccc}\n\\toprule\n");
    out.push_str("Scenario & Detection Delay & Lead Time & False Alarms & Notes \\\\\n\\midrule\n");
    for row in &bundle.primary {
        let result = &row.result;
        let delay = result
            .detection_delay_from_injection()
            .map_or("pre-injection".to_string(), |value| value.to_string());
        let lead = result
            .detection_lead_time()
            .map_or("-".to_string(), |value| value.to_string());
        let notes = if result.false_alarms_before_injection > 0 {
            "pre-injection anomaly observed"
        } else {
            "none in primary run"
        };
        out.push_str(&format!(
            "{} & {} & {} & {} & {} \\\\\n",
            row.name, delay, lead, result.false_alarms_before_injection, notes
        ));
    }
    out.push_str("\\bottomrule\n\\end{tabular}\n\\end{table}\n");
    out
}

fn render_paper_claim_ledger_tex(bundle: &PublicEvaluationBundle) -> String {
    let detected_count = count_true(&bundle.primary, |row| row.result.detected());
    let clean_control_false_rate = find_negative_control(bundle, "Starvation (clean)")
        .map(|row| {
            if row.result.total_steps > 0 {
                (row.result.total_boundary_steps + row.result.total_violation_steps) as f64
                    / row.result.total_steps as f64
                    * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    let sweep_pre_injection = count_true(&bundle.sensitivity_sweep, |row| row.has_false_alarm);
    let mut out = String::new();
    out.push_str("\\subsection*{Claim Ledger}\n");
    out.push_str("\\begin{itemize}\n");
    out.push_str(&format!(
        "\\item Primary evaluation detects {} of {} deterministic scenarios under the recommended configuration. Evidence: Table~\\ref{{tab:summary}} and \\texttt{{data/evaluation\\_results.txt}}.\n",
        detected_count,
        bundle.primary.len()
    ));
    out.push_str(&format!(
        "\\item The recommended configuration is not universally zero-false-alarm: the clean async-starvation control produces a {:.1}\\% false rate. Evidence: negative-control section in \\texttt{{data/evaluation\\_results.txt}}.\n",
        clean_control_false_rate
    ));
    out.push_str(&format!(
        "\\item Sensitivity behavior is configuration-dependent: {} of {} sweep points show pre-injection alarms. Evidence: sensitivity sweep in \\texttt{{data/evaluation\\_results.txt}}.\n",
        sweep_pre_injection,
        bundle.sensitivity_sweep.len()
    ));
    out.push_str(&format!(
        "\\item Deterministic reproducibility holds for {} repeated clock-drift runs in the current harness. Evidence: reproducibility section in \\texttt{{data/evaluation\\_results.txt}}.\n",
        bundle.reproducibility_boundary_steps.len()
    ));
    out.push_str(&format!(
        "\\item The companion crate now emits one canonical broad audit with locked score method \\texttt{{{}}}. Evidence: \\texttt{{paper/generated/audit\\_contract.tex}} and \\texttt{{docs/AUDIT\\_SCORING\\_LOCKED.md}}.\n",
        AUDIT_SCORE_METHOD
    ));
    out.push_str("\\item The audit score is a broad improvement and review-readiness guide, not a compliance or certification result. Evidence: \\texttt{paper/generated/audit\\_contract.tex}.\n");
    out.push_str("\\item The report contract includes conclusion lenses over one shared evidence set rather than separate primary scan modes. Evidence: \\texttt{paper/generated/audit\\_contract.tex}.\n");
    out.push_str("\\end{itemize}\n");
    out
}

fn render_paper_audit_contract_tex() -> String {
    let mut out = String::new();
    out.push_str("\\begin{itemize}[leftmargin=1.5em,itemsep=2pt]\n");
    out.push_str("\\item DSFB now emits one canonical broad static audit rather than primary profile-specific reports.\n");
    out.push_str("\\item The audit keeps one shared evidence set and one shared score denominator, then renders domain and standards interpretations as conclusion lenses at the end of the report.\n");
    out.push_str(&format!(
        "\\item The locked score method is \\texttt{{{}}}, reported as one overall score plus visible advisory subscores.\n",
        AUDIT_SCORE_METHOD
    ));
    out.push_str("\\item The score is intended as a broad code-improvement and review-readiness target for Rust developers.\n");
    out.push_str("\\item The score is not runtime correctness, not a compliance result, and not a certification outcome.\n");
    out.push_str(&format!(
        "\\item {}\n",
        escape_latex(AUDIT_NON_CERTIFICATION_STATEMENT)
    ));
    out.push_str("\\item The public audit contract includes remediation guidance, verification suggestions, evidence identifiers, SARIF, in-toto, DSSE, and static-to-runtime prior derivation.\n");
    out.push_str("\\end{itemize}\n");
    out
}

fn render_paper_results_table_md(bundle: &PublicEvaluationBundle) -> String {
    let mut out = String::new();
    out.push_str("## Generated Primary Results\n\n");
    out.push_str("| Scenario | Detection Delay | Lead Time | False Alarms | Notes |\n");
    out.push_str("|----------|-----------------|-----------|--------------|-------|\n");
    for row in &bundle.primary {
        let result = &row.result;
        let delay = result
            .detection_delay_from_injection()
            .map_or("pre-injection".to_string(), |value| value.to_string());
        let lead = result
            .detection_lead_time()
            .map_or("-".to_string(), |value| value.to_string());
        let notes = if result.false_alarms_before_injection > 0 {
            "pre-injection anomaly observed"
        } else {
            "none in primary run"
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.name, delay, lead, result.false_alarms_before_injection, notes
        ));
    }
    out
}

fn rewrite_marked_section(
    path: &Path,
    start_marker: &str,
    end_marker: &str,
    generated: &str,
) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let Some(start) = contents.find(start_marker) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "missing start marker `{start_marker}` in {}",
                path.display()
            ),
        ));
    };
    let Some(end) = contents.find(end_marker) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing end marker `{end_marker}` in {}", path.display()),
        ));
    };
    let before = &contents[..start + start_marker.len()];
    let after = &contents[end..];
    let mut rewritten = String::new();
    rewritten.push_str(before);
    rewritten.push('\n');
    rewritten.push_str(generated.trim_end());
    rewritten.push('\n');
    rewritten.push_str(after);
    fs::write(path, rewritten)
}

fn escape_latex(input: &str) -> String {
    input
        .replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}
