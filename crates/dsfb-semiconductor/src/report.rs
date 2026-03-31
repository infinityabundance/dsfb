use crate::config::PipelineConfig;
use crate::dataset::phm2018::Phm2018SupportStatus;
use crate::dataset::secom::SecomArchiveLayout;
use crate::error::Result;
use crate::heuristics::HeuristicEntry;
use crate::metrics::{BenchmarkMetrics, MotifMetric};
use crate::plots::FigureManifest;
use crate::precursor::DsaEvaluation;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
struct ArtifactInventoryEntry {
    path: String,
    role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportArtifacts {
    pub markdown_path: PathBuf,
    pub tex_path: PathBuf,
    pub pdf_path: Option<PathBuf>,
    pub pdf_error: Option<String>,
}

pub fn write_reports(
    run_dir: &Path,
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    dsa: &DsaEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> Result<ReportArtifacts> {
    let markdown_path = run_dir.join("engineering_report.md");
    let tex_path = run_dir.join("engineering_report.tex");
    fs::write(
        &markdown_path,
        markdown_report(
            config,
            metrics,
            dsa,
            figures,
            heuristics,
            phm_status,
            secom_layout,
        ),
    )?;
    fs::write(
        &tex_path,
        latex_report(
            config,
            metrics,
            dsa,
            figures,
            heuristics,
            phm_status,
            secom_layout,
        ),
    )?;

    let (pdf_path, pdf_error) = compile_pdf(&tex_path, run_dir);

    Ok(ReportArtifacts {
        markdown_path,
        tex_path,
        pdf_path,
        pdf_error,
    })
}

fn markdown_report(
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    dsa: &DsaEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let mut out = String::new();
    let artifact_inventory = artifact_inventory(figures);

    out.push_str("# DSFB Semiconductor Engineering Report\n\n");

    out.push_str("## Dataset\n\n");
    out.push_str("- Dataset: SECOM (UCI Machine Learning Repository)\n");
    out.push_str(
        "- Evidence class: Stage II public-benchmark evidence on real semiconductor data\n",
    );
    out.push_str("- Non-claim: this run does not establish SEMI compliance, production readiness, or chamber-level mechanism attribution\n\n");

    out.push_str("## Archive Layout Note\n\n");
    out.push_str(&format!(
        "- Numeric columns parsed from `secom.data`: {}\n- Metadata attribute count claimed in `secom.names`: {}\n- Label rows parsed from `secom_labels.data`: {}\n- Label file includes timestamps: {}\n\n{}\n\n",
        secom_layout.data_file_numeric_column_count,
        secom_layout
            .metadata_attribute_count_claim
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
        secom_layout.label_row_count,
        secom_layout.label_file_includes_timestamp,
        secom_layout.note,
    ));

    out.push_str("## Preprocessing Summary\n\n");
    out.push_str(&format!(
        "- Runs: {}\n- Features used by crate: {}\n- Passing runs: {}\n- Failure runs: {}\n- Dataset missing fraction: {:.4}\n- Healthy passing runs requested/found: {}/{}\n\n",
        metrics.summary.dataset_summary.run_count,
        metrics.summary.dataset_summary.feature_count,
        metrics.summary.dataset_summary.pass_count,
        metrics.summary.dataset_summary.fail_count,
        metrics.summary.dataset_summary.dataset_missing_fraction,
        metrics.summary.dataset_summary.healthy_pass_runs_requested,
        metrics.summary.dataset_summary.healthy_pass_runs_found,
    ));
    out.push_str("Missing values remain explicit during dataset loading and are deterministically imputed with the healthy-window nominal mean before residual construction.\n\n");

    out.push_str("## DSFB Instantiation\n\n");
    out.push_str(&format!(
        "- Nominal reference: healthy-window mean over first {} passing runs\n- Residual: x(k) - x_hat\n- Envelope radius rho: {:.1} * healthy-window residual std\n- Drift window W: {}\n- Boundary condition: |r| > {:.1} * rho and drift > {:.1} * healthy drift std\n- Slew threshold: {:.1} * healthy slew std\n- Recurrent-boundary grazing: {} hits in a {}-run window\n- Hysteresis confirmations: {}\n- Persistent-state minimum length: {}\n- Density window: {}\n- Baseline comparators: raw residual threshold plus univariate EWMA on residual norms with alpha = {:.2} and threshold mean + {:.1} * healthy EWMA std\n\n",
        config.healthy_pass_runs,
        config.envelope_sigma,
        config.drift_window,
        config.boundary_fraction_of_rho,
        config.drift_sigma_multiplier,
        config.slew_sigma_multiplier,
        config.grazing_min_hits,
        config.grazing_window,
        config.state_confirmation_steps,
        config.persistent_state_steps,
        config.density_window,
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
    ));
    out.push_str("The existing DSFB residual, drift, slew, grammar, envelope, and violation logic is unchanged in this pass.\n\n");
    out.push_str(&format!(
        "In this crate, `DSFB Violation` remains instantaneous hard envelope exit (`|r| > rho`). `Deterministic Structural Accumulator (DSA)` is additive and sits above the existing DSFB outputs. The current DSA configuration uses `W = {}`, `K = {}`, `tau = {:.2}`, fixed unit weights, primary run signal `{}`, and a consistency rule that rejects any window with inward drift or drift-sign flips.\n\n",
        config.dsa.window,
        config.dsa.persistence_runs,
        config.dsa.alert_tau,
        dsa.run_signals.primary_run_signal,
    ));

    out.push_str("## Quantitative Summary\n\n");
    out.push_str(&format!(
        "- Analyzable features: {}\n- Threshold alarm points: {}\n- EWMA alarm points: {}\n- DSFB raw boundary points: {}\n- DSFB persistent boundary points: {}\n- DSFB raw violation points: {}\n- DSFB persistent violation points: {}\n- DSA alert points: {}\n- DSA alert runs: {}\n- Failure runs with preceding DSA signal ({}-run lookback): {}\n- Failure runs with preceding DSFB Violation signal ({}-run lookback): {}\n- Failure runs with preceding raw DSFB boundary signal ({}-run lookback): {}\n- Failure runs with preceding EWMA signal ({}-run lookback): {}\n- Failure runs with preceding threshold signal ({}-run lookback): {}\n\n",
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.dsfb_raw_boundary_points,
        metrics.summary.dsfb_persistent_boundary_points,
        metrics.summary.dsfb_raw_violation_points,
        metrics.summary.dsfb_persistent_violation_points,
        dsa.summary.alert_point_count,
        dsa.summary.alert_run_count,
        config.pre_failure_lookback_runs,
        dsa.summary.failure_run_recall,
        config.pre_failure_lookback_runs,
        dsa.comparison_summary.dsfb_violation.failure_run_recall,
        config.pre_failure_lookback_runs,
        dsa.comparison_summary.dsfb_raw_boundary.failure_run_recall,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_ewma_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
    ));
    out.push_str("The raw threshold baseline and the raw DSFB Violation state still share the same instantaneous envelope-exit condition. DSA is a separate structural compression layer, not a redefinition of threshold or violation.\n\n");

    out.push_str("## Lead-Time and Nuisance Proxies\n\n");
    out.push_str(&format!(
        "- Mean DSA lead (runs): {}\n- Median DSA lead (runs): {}\n- Mean raw DSFB boundary lead (runs): {}\n- Mean DSFB Violation lead (runs): {}\n- Mean EWMA lead (runs): {}\n- Mean threshold lead (runs): {}\n- Mean DSA minus threshold lead delta (runs): {}\n- Mean DSA minus EWMA lead delta (runs): {}\n- Pass-run nuisance proxy, DSA: {:.4}\n- Pass-run nuisance proxy, raw DSFB boundary: {:.4}\n- Pass-run nuisance proxy, DSFB Violation: {:.4}\n- Pass-run nuisance proxy, EWMA: {:.4}\n- Pass-run nuisance proxy, threshold: {:.4}\n\n",
        format_option_f64(dsa.summary.mean_lead_time_runs),
        format_option_f64(dsa.summary.median_lead_time_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_ewma_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_threshold_lead_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        dsa.comparison_summary.dsfb_raw_boundary.pass_run_nuisance_proxy,
        dsa.comparison_summary.dsfb_violation.pass_run_nuisance_proxy,
        dsa.comparison_summary.ewma.pass_run_nuisance_proxy,
        dsa.comparison_summary.threshold.pass_run_nuisance_proxy,
    ));
    out.push_str("These nuisance values are pass-run proxies on SECOM labels, not fab-certified false-alarm metrics.\n\n");

    out.push_str("## Deterministic Structural Accumulator (DSA)\n\n");
    out.push_str("- DSA is a persistence-constrained structural decision layer\n");
    out.push_str("- DSA is additive and sits above existing DSFB outputs\n");
    out.push_str("- DSFB Violation remains instantaneous envelope exit\n");
    out.push_str("- DSA is intended to reduce nuisance and stabilize precursor regimes\n");
    out.push_str(&format!(
        "- Primary run-level comparison signal: `{}`\n- Secondary run-level signal emitted: `feature_count_dsa_alert(k)`\n- Failure-run recall, DSA: {}/{}\n- Failure-run recall, threshold: {}/{}\n- Failure-run recall, EWMA: {}/{}\n- Failure-run recall, DSFB Violation: {}/{}\n- Mean lead time, DSA: {}\n- Median lead time, DSA: {}\n- Pass-run nuisance proxy, DSA: {:.4}\n- Lead delta vs threshold (runs): {}\n- Lead delta vs EWMA (runs): {}\n- Nuisance delta vs threshold: {:.4}\n- Nuisance delta vs EWMA: {:.4}\n- Nuisance delta vs raw DSFB boundary: {:.4}\n- DSA episodes: {}\n- Mean DSA episode length (runs): {}\n- Max DSA episode length (runs): {}\n- Raw boundary episodes: {}\n- Compression ratio (raw boundary / DSA): {}\n- Non-escalating DSA episode fraction: {}\n- Nuisance improved: {}\n- Lead time improved: {}\n- Recall preserved: {}\n- Compression improved: {}\n- Nothing improved: {}\n- Threshold recall gate passed: {}\n- Boundary nuisance gate passed: {}\n- Validation passed: {}\n\n{}\n\n",
        dsa.run_signals.primary_run_signal,
        dsa.comparison_summary.dsa.failure_run_recall,
        dsa.comparison_summary.dsa.failure_runs,
        dsa.comparison_summary.threshold.failure_run_recall,
        dsa.comparison_summary.threshold.failure_runs,
        dsa.comparison_summary.ewma.failure_run_recall,
        dsa.comparison_summary.ewma.failure_runs,
        dsa.comparison_summary.dsfb_violation.failure_run_recall,
        dsa.comparison_summary.dsfb_violation.failure_runs,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_time_runs),
        format_option_f64(dsa.comparison_summary.dsa.median_lead_time_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.pass_run_nuisance_delta_vs_threshold,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_raw_boundary,
        dsa.episode_summary.dsa_episode_count,
        format_option_f64(dsa.episode_summary.mean_dsa_episode_length_runs),
        dsa.episode_summary.max_dsa_episode_length_runs,
        dsa.episode_summary.raw_boundary_episode_count,
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.non_escalating_dsa_episode_fraction),
        dsa.comparison_summary.nuisance_improved,
        dsa.comparison_summary.lead_time_improved,
        dsa.comparison_summary.recall_preserved,
        dsa.comparison_summary.compression_improved,
        dsa.comparison_summary.nothing_improved,
        dsa.comparison_summary.threshold_recall_gate_passed,
        dsa.comparison_summary.boundary_nuisance_gate_passed,
        dsa.comparison_summary.validation_passed,
        dsa.comparison_summary.conclusion,
    ));

    out.push_str("## Density Summary\n\n");
    out.push_str(&format!(
        "- Density window: {} runs\n- Mean persistent boundary density, failure-labeled runs: {:.4}\n- Mean persistent boundary density, pass-labeled runs: {:.4}\n- Mean persistent violation density, failure-labeled runs: {:.4}\n- Mean persistent violation density, pass-labeled runs: {:.4}\n- Mean threshold density, failure-labeled runs: {:.4}\n- Mean threshold density, pass-labeled runs: {:.4}\n- Mean EWMA density, failure-labeled runs: {:.4}\n- Mean EWMA density, pass-labeled runs: {:.4}\n\n",
        metrics.density_summary.density_window,
        metrics.density_summary.mean_persistent_boundary_density_failure,
        metrics.density_summary.mean_persistent_boundary_density_pass,
        metrics.density_summary.mean_persistent_violation_density_failure,
        metrics.density_summary.mean_persistent_violation_density_pass,
        metrics.density_summary.mean_threshold_density_failure,
        metrics.density_summary.mean_threshold_density_pass,
        metrics.density_summary.mean_ewma_density_failure,
        metrics.density_summary.mean_ewma_density_pass,
    ));

    out.push_str(&drsc_markdown_section(figures));

    out.push_str("## Motif Calibration Summary\n\n");
    out.push_str("| Motif | Point hits | Run hits | Pre-failure window run hits | Precision proxy |\n");
    out.push_str("|---|---:|---:|---:|---:|\n");
    for metric in &metrics.motif_metrics {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            metric.motif_name,
            metric.point_hits,
            metric.run_hits,
            metric.pre_failure_window_run_hits,
            format_option_f64(metric.pre_failure_window_precision_proxy),
        ));
    }
    out.push('\n');

    out.push_str("## Heuristics Bank\n\n");
    out.push_str("| Motif | Provenance | Contributes to DSA scoring | Severity | Recommended action |\n");
    out.push_str("|---|---|---|---|---|\n");
    for entry in heuristics {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            entry.motif_name,
            entry.provenance_status,
            entry.contributes_to_dsa_scoring,
            entry.severity,
            entry.recommended_action,
        ));
    }
    out.push('\n');

    out.push_str("## Figures\n\n");
    for file in &figures.files {
        out.push_str(&format!("- figures/{}\n", file));
    }
    out.push('\n');

    out.push_str("## Artifact Inventory\n\n");
    out.push_str("| Path | Role |\n|---|---|\n");
    for entry in &artifact_inventory {
        out.push_str(&format!("| {} | {} |\n", entry.path, entry.role));
    }
    out.push('\n');

    out.push_str("## PHM 2018 Status\n\n");
    out.push_str(&format!(
        "- Official page: {}\n- Manual archive path: {}\n- Implemented now: {}\n- Blocker: {}\n\n",
        phm_status.official_page,
        phm_status.manual_placement_path.display(),
        phm_status.fully_implemented,
        phm_status.blocker,
    ));

    out.push_str("## Limitations of This Run\n\n");
    out.push_str("- SECOM is anonymized and instance-level; this run does not validate chamber-mechanism attribution or run-to-failure prognostics.\n");
    out.push_str("- The comparator set is still narrow: raw threshold, EWMA, DSFB boundary, and DSFB Violation. Stronger multivariate FDC and ML baselines are intentionally not claimed here.\n");
    out.push_str("- Lead-time and nuisance values are bounded proxy metrics derived from SECOM labels and a fixed lookback, not fab-qualified operational KPIs.\n");
    out.push_str("- PHM 2018 support is still limited to the manual-placement contract and archive probe until the real archive is present and verified.\n");
    out.push_str("- DRSC remains unchanged in semantics in this pass; DSA is not separately visualized there.\n");
    out.push_str("- PDF generation depends on a local `pdflatex` installation.\n\n");

    out.push_str("## Explicit Non-Claims\n\n");
    out.push_str("- No universal superiority claim over SPC, EWMA, FDC, or ML baselines\n");
    out.push_str("- No standards-compliance or completed qualification claim\n");
    out.push_str("- No SEMI compatibility claim\n");
    out.push_str("- No chamber-mechanism or physical root-cause attribution from SECOM alone\n");
    out.push_str("- No PHM 2018 completion claim unless the real archive is staged and verified\n");
    out.push_str("- No Kani verification claim for this crate\n");
    out.push_str("- No no_alloc, SIMD, rayon, or parallel-acceleration claim for this crate\n");
    out
}

fn latex_report(
    config: &PipelineConfig,
    metrics: &BenchmarkMetrics,
    dsa: &DsaEvaluation,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let mut out = String::new();
    let artifact_inventory = artifact_inventory(figures);

    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage[margin=1in]{geometry}\n");
    out.push_str("\\usepackage{booktabs}\n");
    out.push_str("\\usepackage{graphicx}\n");
    out.push_str("\\usepackage{longtable}\n");
    out.push_str("\\usepackage{hyperref}\n");
    out.push_str("\\begin{document}\n");
    out.push_str("\\title{DSFB Semiconductor Engineering Report}\n");
    out.push_str("\\author{Automatically generated by dsfb-semiconductor}\n");
    out.push_str("\\date{}\n\\maketitle\n\n");

    out.push_str("\\section*{Dataset}\n");
    out.push_str("This report documents a real-data DSFB run on the SECOM dataset from the UCI Machine Learning Repository. It is a Stage II public-benchmark artifact, not a deployment or qualification report.\n\n");

    out.push_str("\\section*{Archive layout note}\n");
    out.push_str(&format!(
        "The current distributed archive parses as {} numeric columns in \\texttt{{secom.data}}. The \\texttt{{secom.names}} metadata text claims {} attributes. The crate uses the numeric columns actually present in \\texttt{{secom.data}} and reads labels and timestamps separately from \\texttt{{secom\\_labels.data}}.\n\n",
        secom_layout.data_file_numeric_column_count,
        secom_layout
            .metadata_attribute_count_claim
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
    ));

    out.push_str("\\section*{Preprocessing summary}\n");
    out.push_str("\\begin{tabular}{lr}\n\\toprule\n");
    out.push_str(&format!(
        "Runs & {} \\\\\nFeatures used by crate & {} \\\\\nPassing runs & {} \\\\\nFailure runs & {} \\\\\nDataset missing fraction & {:.4} \\\\\nHealthy passing runs requested & {} \\\\\nHealthy passing runs found & {} \\\\\n",
        metrics.summary.dataset_summary.run_count,
        metrics.summary.dataset_summary.feature_count,
        metrics.summary.dataset_summary.pass_count,
        metrics.summary.dataset_summary.fail_count,
        metrics.summary.dataset_summary.dataset_missing_fraction,
        metrics.summary.dataset_summary.healthy_pass_runs_requested,
        metrics.summary.dataset_summary.healthy_pass_runs_found,
    ));
    out.push_str("\\bottomrule\n\\end{tabular}\n\n");

    out.push_str("\\section*{DSFB instantiation}\n");
    out.push_str(&latex_escape(&format!(
        "The nominal reference is the healthy-window mean over the first {} passing runs. Residuals are defined as x(k) - x_hat. The admissibility envelope radius is {:.1} sigma on the healthy residual distribution. The drift window is W = {}. The boundary rule in this implementation is |r| > {:.1} rho with drift above {:.1} healthy drift sigma. Abrupt slew tags use {:.1} healthy slew sigma. Hysteresis-confirmed state changes require {} confirmations, persistent-state alarms require {} consecutive confirmed steps, and density metrics use a {}-run sliding window. The scalar comparator set contains a raw residual threshold and a univariate EWMA on residual norms with alpha = {:.2} and a threshold at the healthy-window EWMA mean plus {:.1} sigma. DSFB Violation remains the instantaneous hard envelope exit state. DSA is additive, sits above the existing DSFB outputs, and uses W = {}, K = {}, tau = {:.2}, fixed unit weights, primary run signal {}, and a consistency rule that rejects any inward drift or drift-sign flip.",
        config.healthy_pass_runs,
        config.envelope_sigma,
        config.drift_window,
        config.boundary_fraction_of_rho,
        config.drift_sigma_multiplier,
        config.slew_sigma_multiplier,
        config.state_confirmation_steps,
        config.persistent_state_steps,
        config.density_window,
        config.ewma_alpha,
        config.ewma_sigma_multiplier,
        config.dsa.window,
        config.dsa.persistence_runs,
        config.dsa.alert_tau,
        dsa.run_signals.primary_run_signal,
    )));
    out.push_str("\n\n");

    out.push_str("\\section*{Deterministic Structural Accumulator (DSA)}\n");
    out.push_str("\\begin{tabular}{lr}\n\\toprule\n");
    out.push_str(&format!(
        "Failure-run recall, DSA & {}/{} \\\\\nFailure-run recall, threshold & {}/{} \\\\\nFailure-run recall, EWMA & {}/{} \\\\\nFailure-run recall, DSFB Violation & {}/{} \\\\\nMean lead time, DSA & {} \\\\\nMedian lead time, DSA & {} \\\\\nPass-run nuisance proxy, DSA & {:.4} \\\\\nLead delta vs threshold & {} \\\\\nLead delta vs EWMA & {} \\\\\nNuisance delta vs threshold & {:.4} \\\\\nNuisance delta vs EWMA & {:.4} \\\\\nNuisance delta vs raw boundary & {:.4} \\\\\nRaw boundary episodes & {} \\\\\nDSA episodes & {} \\\\\nCompression ratio & {} \\\\\nNon-escalating DSA episode fraction & {} \\\\\nNuisance improved & {} \\\\\nLead time improved & {} \\\\\nRecall preserved & {} \\\\\nCompression improved & {} \\\\\nNothing improved & {} \\\\\nThreshold recall gate passed & {} \\\\\nBoundary nuisance gate passed & {} \\\\\nValidation passed & {} \\\\\n",
        dsa.comparison_summary.dsa.failure_run_recall,
        dsa.comparison_summary.dsa.failure_runs,
        dsa.comparison_summary.threshold.failure_run_recall,
        dsa.comparison_summary.threshold.failure_runs,
        dsa.comparison_summary.ewma.failure_run_recall,
        dsa.comparison_summary.ewma.failure_runs,
        dsa.comparison_summary.dsfb_violation.failure_run_recall,
        dsa.comparison_summary.dsfb_violation.failure_runs,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_time_runs),
        format_option_f64(dsa.comparison_summary.dsa.median_lead_time_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.pass_run_nuisance_delta_vs_threshold,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_raw_boundary,
        dsa.episode_summary.raw_boundary_episode_count,
        dsa.episode_summary.dsa_episode_count,
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.non_escalating_dsa_episode_fraction),
        dsa.comparison_summary.nuisance_improved,
        dsa.comparison_summary.lead_time_improved,
        dsa.comparison_summary.recall_preserved,
        dsa.comparison_summary.compression_improved,
        dsa.comparison_summary.nothing_improved,
        dsa.comparison_summary.threshold_recall_gate_passed,
        dsa.comparison_summary.boundary_nuisance_gate_passed,
        dsa.comparison_summary.validation_passed,
    ));
    out.push_str("\\bottomrule\n\\end{tabular}\n\n");
    out.push_str(&latex_escape(&dsa.comparison_summary.conclusion));
    out.push_str("\n\n");

    out.push_str("\\section*{Motif metrics}\n");
    out.push_str("\\begin{longtable}{p{0.26\\linewidth}rrrr}\n\\toprule\n");
    out.push_str("Motif & Point hits & Run hits & Pre-failure run hits & Precision proxy \\\\\n\\midrule\n");
    for metric in &metrics.motif_metrics {
        out.push_str(&motif_row(metric));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");

    out.push_str("\\section*{Heuristics bank}\n");
    out.push_str("\\begin{longtable}{p{0.18\\linewidth}p{0.15\\linewidth}p{0.12\\linewidth}p{0.37\\linewidth}}\n\\toprule\n");
    out.push_str("Motif & Provenance & In DSA score & Recommended action \\\\\n\\midrule\n");
    for entry in heuristics {
        out.push_str(&format!(
            "{} & {} & {} & {} \\\\\n",
            latex_escape(&entry.motif_name),
            latex_escape(&entry.provenance_status),
            latex_escape(&entry.contributes_to_dsa_scoring.to_string()),
            latex_escape(&entry.recommended_action),
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");

    out.push_str(&drsc_latex_section(figures));

    out.push_str("\\section*{Artifact inventory}\n");
    out.push_str("\\begin{longtable}{p{0.38\\linewidth}p{0.52\\linewidth}}\n\\toprule\n");
    out.push_str("Path & Role \\\\\n\\midrule\n");
    for entry in &artifact_inventory {
        out.push_str(&format!(
            "{} & {} \\\\\n",
            latex_escape(&entry.path),
            latex_escape(&entry.role),
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");

    out.push_str("\\section*{PHM 2018 status}\n");
    out.push_str(&latex_escape(&format!(
        "Official page: {}. Manual archive path: {}. Implemented now: {}. Blocker: {}.",
        phm_status.official_page,
        phm_status.manual_placement_path.display(),
        phm_status.fully_implemented,
        phm_status.blocker,
    )));
    out.push_str("\n\n");

    out.push_str("\\section*{Explicit non-claims}\n");
    out.push_str("\\begin{itemize}\n");
    out.push_str("\\item No universal superiority claim over SPC, EWMA, FDC, or ML baselines.\n");
    out.push_str("\\item No standards-compliance, completed qualification, or SEMI compatibility claim.\n");
    out.push_str("\\item No chamber-mechanism or physical root-cause attribution from SECOM alone.\n");
    out.push_str("\\item No PHM 2018 completion claim unless the real archive is staged and verified.\n");
    out.push_str("\\item No Kani verification, no\\_alloc, SIMD, rayon, or parallel-acceleration claim for this crate.\n");
    out.push_str("\\end{itemize}\n\n");

    out.push_str(&figure_blocks(figures));
    out.push_str("\\end{document}\n");
    out
}

fn artifact_inventory(figures: &FigureManifest) -> Vec<ArtifactInventoryEntry> {
    let mut entries = vec![
        ArtifactInventoryEntry {
            path: "dataset_summary.json".into(),
            role: "Dataset summary and healthy-window counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "parameter_manifest.json".into(),
            role: "Saved deterministic DSFB parameter values.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_parameter_manifest.json".into(),
            role: "Saved deterministic DSA parameter values, fixed weights, run-level signal choice, and consistency rule.".into(),
        },
        ArtifactInventoryEntry {
            path: "run_configuration.json".into(),
            role: "CLI, data-root, and output-root configuration.".into(),
        },
        ArtifactInventoryEntry {
            path: "benchmark_metrics.json".into(),
            role: "Top-level benchmark metrics, summaries, and feature metrics.".into(),
        },
        ArtifactInventoryEntry {
            path: "baseline_comparison_summary.json".into(),
            role: "Baseline comparison summary across DSFB, threshold, EWMA, and DSA.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_vs_baselines.json".into(),
            role: "Saved DSA recall, lead-time, nuisance, validation, and compression summary.".into(),
        },
        ArtifactInventoryEntry {
            path: "feature_metrics.csv".into(),
            role: "Per-feature DSFB and baseline point counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_metrics.csv".into(),
            role: "Per-feature, per-run DSA structural features, score inputs, consistency flags, and alerts.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_run_signals.csv".into(),
            role: "Run-level DSA primary signal and alerting-feature counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "per_failure_run_signals.csv".into(),
            role: "Per-failure DSFB state-layer earliest-signal and lead-time records.".into(),
        },
        ArtifactInventoryEntry {
            path: "per_failure_run_dsa_signals.csv".into(),
            role: "Per-failure DSA earliest-signal and lead-time records.".into(),
        },
        ArtifactInventoryEntry {
            path: "lead_time_metrics.csv".into(),
            role: "Flattened DSFB, threshold, and EWMA lead-time table.".into(),
        },
        ArtifactInventoryEntry {
            path: "density_metrics.csv".into(),
            role: "Sliding-window density metrics per run.".into(),
        },
        ArtifactInventoryEntry {
            path: "residuals.csv".into(),
            role: "Residual trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "drifts.csv".into(),
            role: "Drift trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "slews.csv".into(),
            role: "Slew trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "ewma_baseline.csv".into(),
            role: "EWMA baseline trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "grammar_states.csv".into(),
            role: "Raw and confirmed DSFB grammar states per feature and run.".into(),
        },
        ArtifactInventoryEntry {
            path: "heuristics_bank.json".into(),
            role: "Provenance-aware heuristic guidance and DSA motif participation flags.".into(),
        },
        ArtifactInventoryEntry {
            path: "secom_archive_layout.json".into(),
            role: "Archive-layout inspection and metadata mismatch note.".into(),
        },
        ArtifactInventoryEntry {
            path: "phm2018_support_status.json".into(),
            role: "PHM 2018 manual-placement and support-status record.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.md".into(),
            role: "Markdown engineering report.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.tex".into(),
            role: "LaTeX source for the report PDF.".into(),
        },
        ArtifactInventoryEntry {
            path: "engineering_report.pdf".into(),
            role: "Compiled report PDF when pdflatex is available.".into(),
        },
        ArtifactInventoryEntry {
            path: "artifact_manifest.json".into(),
            role: "Machine-readable manifest of output artifact paths.".into(),
        },
        ArtifactInventoryEntry {
            path: "run_bundle.zip".into(),
            role: "ZIP archive containing the complete run directory.".into(),
        },
    ];

    if figures.drsc.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "drsc_top_feature.csv".into(),
            role: "Aligned DRSC trace export for the selected feature window.".into(),
        });
    }

    for file in &figures.files {
        entries.push(ArtifactInventoryEntry {
            path: format!("figures/{file}"),
            role: "Crate-generated PNG figure.".into(),
        });
    }

    entries
}

fn drsc_markdown_section(figures: &FigureManifest) -> String {
    if let Some(drsc) = &figures.drsc {
        format!(
            "## Deterministic Residual Stateflow Chart (DRSC)\n\nThe crate emits a DRSC figure and aligned trace CSV for the top persistent-boundary feature in the current run (`{}`). The chart is unchanged in semantics in this pass: top layer residual/drift/slew, middle persistent DSFB states, bottom admissibility and EWMA occupancy. DSA is not separately visualized here.\n\n- Figure: figures/{}\n- Trace CSV: {}\n\n",
            drsc.feature_name, drsc.figure_file, drsc.trace_csv,
        )
    } else {
        String::new()
    }
}

fn drsc_latex_section(figures: &FigureManifest) -> String {
    if let Some(drsc) = &figures.drsc {
        format!(
            "\\section*{{Deterministic Residual Stateflow Chart (DRSC)}}\nThe crate emits a DRSC figure and aligned trace CSV for the top persistent-boundary feature in the current run (\\texttt{{{}}}). The chart is unchanged in semantics in this pass: top layer residual, drift, and slew; middle layer persistent DSFB states; bottom layer admissibility and EWMA occupancy. DSA is not separately visualized here. The aligned trace CSV is \\texttt{{{}}}.\n\n",
            latex_escape(&drsc.feature_name),
            latex_escape(&drsc.trace_csv),
        )
    } else {
        String::new()
    }
}

fn figure_blocks(figures: &FigureManifest) -> String {
    figures
        .files
        .iter()
        .map(|file| {
            let caption = if figures
                .drsc
                .as_ref()
                .map(|drsc| drsc.figure_file == *file)
                .unwrap_or(false)
            {
                "Deterministic Residual Stateflow Chart (DRSC) for the selected feature window."
                    .to_string()
            } else {
                format!("Generated artifact: {}", file)
            };
            format!(
                "\\begin{{figure}}[htbp]\n\\centering\n\\includegraphics[width=0.92\\linewidth]{{figures/{}}}\n\\caption{{{}}}\n\\end{{figure}}\n",
                latex_escape(file),
                latex_escape(&caption),
            )
        })
        .collect::<String>()
}

fn motif_row(metric: &MotifMetric) -> String {
    format!(
        "{} & {} & {} & {} & {} \\\\\n",
        latex_escape(&metric.motif_name),
        metric.point_hits,
        metric.run_hits,
        metric.pre_failure_window_run_hits,
        latex_escape(&format_option_f64(metric.pre_failure_window_precision_proxy)),
    )
}

fn compile_pdf(tex_path: &Path, output_dir: &Path) -> (Option<PathBuf>, Option<String>) {
    let filename = tex_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "engineering_report.tex".into());
    let pdf_path = output_dir.join(filename.replace(".tex", ".pdf"));
    let mut combined_output = String::new();
    let mut any_success = false;

    for _ in 0..2 {
        match Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-output-directory")
            .arg(".")
            .arg(&filename)
            .current_dir(output_dir)
            .output()
        {
            Ok(output) => {
                combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                if output.status.success() {
                    any_success = true;
                }
            }
            Err(err) => {
                if pdf_path.exists() {
                    return (Some(pdf_path), Some(err.to_string()));
                }
                return (None, Some(err.to_string()));
            }
        }
    }

    if any_success && pdf_path.exists() {
        return (Some(pdf_path), None);
    }
    if pdf_path.exists() {
        return (
            Some(pdf_path),
            (!combined_output.trim().is_empty()).then_some(combined_output),
        );
    }

    (None, (!combined_output.trim().is_empty()).then_some(combined_output))
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

fn latex_escape(input: &str) -> String {
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
