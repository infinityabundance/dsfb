use crate::config::PipelineConfig;
use crate::dataset::phm2018::Phm2018SupportStatus;
use crate::dataset::secom::SecomArchiveLayout;
use crate::error::Result;
use crate::heuristics::HeuristicEntry;
use crate::metrics::{BenchmarkMetrics, MotifMetric};
use crate::plots::FigureManifest;
use crate::precursor::{DsaEvaluation, DsaGridSummary};
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
    dsa_grid_summary: &DsaGridSummary,
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
            dsa_grid_summary,
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
            dsa_grid_summary,
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
    dsa_grid_summary: &DsaGridSummary,
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
        "- Nominal reference: healthy-window mean over first {} passing runs\n- Residual: x(k) - x_hat\n- Envelope radius rho: {:.1} * healthy-window residual std\n- Drift window W: {}\n- Boundary condition: |r| > {:.1} * rho and drift > {:.1} * healthy drift std\n- Slew threshold: {:.1} * healthy slew std\n- Recurrent-boundary grazing: {} hits in a {}-run window\n- Hysteresis confirmations: {}\n- Persistent-state minimum length: {}\n- Density window: {}\n- Baseline comparators: raw residual threshold, univariate EWMA on residual norms with alpha = {:.2} and threshold mean + {:.1} * healthy EWMA std, positive CUSUM on residual norms with kappa = {:.1} * healthy std and alarm threshold = {:.1} * healthy std, run-level residual energy with threshold mean + {:.1} * healthy run-energy std, and PCA T2/SPE multivariate FDC retaining {:.0}% healthy variance with thresholds mean + {:.1}/{:.1} * healthy sigma\n\n",
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
        config.cusum_kappa_sigma_multiplier,
        config.cusum_alarm_sigma_multiplier,
        config.run_energy_sigma_multiplier,
        config.pca_variance_explained * 100.0,
        config.pca_t2_sigma_multiplier,
        config.pca_spe_sigma_multiplier,
    ));
    out.push_str("The existing DSFB residual, drift, slew, grammar, envelope, and violation logic is unchanged in this pass.\n\n");
    out.push_str(&format!(
        "In this crate, `DSFB Violation` remains instantaneous hard envelope exit (`|r| > rho`). `Deterministic Structural Accumulator (DSA)` is additive and sits above the existing DSFB outputs. The feature-level DSA precursor itself remains persistence-constrained, and the run-level comparison signal is cross-feature corroboration: `{}`. The current DSA configuration uses `W = {}`, `K = {}`, `tau = {:.2}`, `m = {}`, fixed unit weights, and a consistency rule that rejects thresholded inward drift and thresholded drift-sign flips.\n\n",
        dsa.run_signals.primary_run_signal,
        config.dsa.window,
        config.dsa.persistence_runs,
        config.dsa.alert_tau,
        config.dsa.corroborating_feature_count_min,
    ));

    out.push_str("## Quantitative Summary\n\n");
    out.push_str(&format!(
        "- Analyzable features: {}\n- Threshold alarm points: {}\n- EWMA alarm points: {}\n- CUSUM alarm points: {}\n- Run-energy alarm points: {}\n- PCA T2/SPE alarm points: {}\n- DSFB raw boundary points: {}\n- DSFB persistent boundary points: {}\n- DSFB raw violation points: {}\n- DSFB persistent violation points: {}\n- DSA alert points: {}\n- DSA alert runs: {}\n- Failure runs with preceding DSA signal ({}-run lookback): {}\n- Failure runs with preceding DSFB Violation signal ({}-run lookback): {}\n- Failure runs with preceding raw DSFB boundary signal ({}-run lookback): {}\n- Failure runs with preceding EWMA signal ({}-run lookback): {}\n- Failure runs with preceding CUSUM signal ({}-run lookback): {}\n- Failure runs with preceding run-energy signal ({}-run lookback): {}\n- Failure runs with preceding PCA T2/SPE signal ({}-run lookback): {}\n- Failure runs with preceding threshold signal ({}-run lookback): {}\n\n",
        metrics.summary.analyzable_feature_count,
        metrics.summary.threshold_alarm_points,
        metrics.summary.ewma_alarm_points,
        metrics.summary.cusum_alarm_points,
        metrics.summary.run_energy_alarm_points,
        metrics.summary.pca_fdc_alarm_points,
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
        metrics.summary.failure_runs_with_preceding_cusum_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_run_energy_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_pca_fdc_signal,
        config.pre_failure_lookback_runs,
        metrics.summary.failure_runs_with_preceding_threshold_signal,
    ));
    out.push_str("The raw threshold baseline and the raw DSFB Violation state still share the same instantaneous envelope-exit condition. The feature-level DSA precursor is separate structural compression logic, and the run-level primary DSA signal requires cross-feature corroboration without redefining either frozen baseline.\n\n");

    out.push_str("## Lead-Time and Nuisance Proxies\n\n");
    out.push_str(&format!(
        "- Mean DSA lead (runs): {}\n- Median DSA lead (runs): {}\n- Mean raw DSFB boundary lead (runs): {}\n- Mean DSFB Violation lead (runs): {}\n- Mean EWMA lead (runs): {}\n- Mean CUSUM lead (runs): {}\n- Mean run-energy lead (runs): {}\n- Mean PCA T2/SPE lead (runs): {}\n- Mean threshold lead (runs): {}\n- Mean DSA minus CUSUM lead delta (runs): {}\n- Mean DSA minus run-energy lead delta (runs): {}\n- Mean DSA minus PCA T2/SPE lead delta (runs): {}\n- Mean DSA minus threshold lead delta (runs): {}\n- Mean DSA minus EWMA lead delta (runs): {}\n- Pass-run nuisance proxy, DSA: {:.4}\n- Pass-run nuisance proxy, raw DSFB boundary: {:.4}\n- Pass-run nuisance proxy, DSFB Violation: {:.4}\n- Pass-run nuisance proxy, EWMA: {:.4}\n- Pass-run nuisance proxy, CUSUM: {:.4}\n- Pass-run nuisance proxy, run energy: {:.4}\n- Pass-run nuisance proxy, PCA T2/SPE: {:.4}\n- Pass-run nuisance proxy, threshold: {:.4}\n\n",
        format_option_f64(dsa.summary.mean_lead_time_runs),
        format_option_f64(dsa.summary.median_lead_time_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_boundary_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_raw_violation_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_ewma_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_cusum_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_run_energy_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_pca_fdc_lead_runs),
        format_option_f64(metrics.lead_time_summary.mean_threshold_lead_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_cusum_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_run_energy_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_pca_fdc_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.summary.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        dsa.comparison_summary.dsfb_raw_boundary.pass_run_nuisance_proxy,
        dsa.comparison_summary.dsfb_violation.pass_run_nuisance_proxy,
        dsa.comparison_summary.ewma.pass_run_nuisance_proxy,
        dsa.comparison_summary.cusum.pass_run_nuisance_proxy,
        dsa.comparison_summary.run_energy.pass_run_nuisance_proxy,
        dsa.comparison_summary.pca_fdc.pass_run_nuisance_proxy,
        dsa.comparison_summary.threshold.pass_run_nuisance_proxy,
    ));
    out.push_str("These nuisance values are pass-run proxies on SECOM labels, not fab-certified false-alarm metrics.\n\n");

    out.push_str("## Deterministic Structural Accumulator (DSA)\n\n");
    out.push_str("- DSA is a persistence-constrained structural decision layer\n");
    out.push_str("- DSA is additive and sits above existing DSFB outputs\n");
    out.push_str("- DSFB Violation remains instantaneous envelope exit\n");
    out.push_str("- The feature-level DSA precursor is structural accumulation; the run-level primary signal is a corroborated feature-count decision\n");
    out.push_str("- DSA is intended to reduce nuisance and stabilize precursor regimes\n");
    out.push_str(&format!(
        "- Primary run-level comparison signal: `{}`\n- Primary run-level signal definition: `{}`\n- Secondary run-level signal emitted: `{}`\n- Tertiary run-level signal emitted: `{}`\n- Failure-run recall, DSA: {}/{}\n- Failure-run recall, threshold: {}/{}\n- Failure-run recall, EWMA: {}/{}\n- Failure-run recall, CUSUM: {}/{}\n- Failure-run recall, run energy: {}/{}\n- Failure-run recall, PCA T2/SPE: {}/{}\n- Failure-run recall, DSFB Violation: {}/{}\n- Mean lead time, DSA: {}\n- Median lead time, DSA: {}\n- Pass-run nuisance proxy, DSA: {:.4}\n- Lead delta vs CUSUM (runs): {}\n- Lead delta vs run energy (runs): {}\n- Lead delta vs PCA T2/SPE (runs): {}\n- Lead delta vs threshold (runs): {}\n- Lead delta vs EWMA (runs): {}\n- Nuisance delta vs threshold: {:.4}\n- Nuisance delta vs EWMA: {:.4}\n- Nuisance delta vs DSFB Violation: {:.4}\n- Nuisance delta vs CUSUM: {:.4}\n- Nuisance delta vs run energy: {:.4}\n- Nuisance delta vs PCA T2/SPE: {:.4}\n- Nuisance delta vs raw DSFB boundary: {:.4}\n- DSA episodes: {}\n- DSA episodes preceding failure: {}\n- Precursor quality: {}\n- Mean DSA episode length (runs): {}\n- Max DSA episode length (runs): {}\n- Raw boundary episodes: {}\n- Compression ratio (raw boundary / DSA): {}\n- Non-escalating DSA episode fraction: {}\n- Primary success condition met: {}\n- Success-condition failures: {}\n- Nuisance improved: {}\n- Lead time improved: {}\n- Recall preserved: {}\n- Compression improved: {}\n- Nothing improved: {}\n- Threshold recall gate passed: {}\n- Boundary nuisance gate passed: {}\n- Validation passed: {}\n\n{}\n\n",
        dsa.run_signals.primary_run_signal,
        dsa.parameter_manifest.primary_run_signal_definition,
        dsa.parameter_manifest.secondary_run_signal,
        dsa.parameter_manifest.tertiary_run_signal,
        dsa.comparison_summary.dsa.failure_run_recall,
        dsa.comparison_summary.dsa.failure_runs,
        dsa.comparison_summary.threshold.failure_run_recall,
        dsa.comparison_summary.threshold.failure_runs,
        dsa.comparison_summary.ewma.failure_run_recall,
        dsa.comparison_summary.ewma.failure_runs,
        dsa.comparison_summary.cusum.failure_run_recall,
        dsa.comparison_summary.cusum.failure_runs,
        dsa.comparison_summary.run_energy.failure_run_recall,
        dsa.comparison_summary.run_energy.failure_runs,
        dsa.comparison_summary.pca_fdc.failure_run_recall,
        dsa.comparison_summary.pca_fdc.failure_runs,
        dsa.comparison_summary.dsfb_violation.failure_run_recall,
        dsa.comparison_summary.dsfb_violation.failure_runs,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_time_runs),
        format_option_f64(dsa.comparison_summary.dsa.median_lead_time_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_cusum_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_run_energy_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_pca_fdc_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.pass_run_nuisance_delta_vs_threshold,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_violation,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_cusum,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_run_energy,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_pca_fdc,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_raw_boundary,
        dsa.episode_summary.dsa_episode_count,
        dsa.episode_summary.dsa_episodes_preceding_failure,
        format_option_f64(dsa.episode_summary.precursor_quality),
        format_option_f64(dsa.episode_summary.mean_dsa_episode_length_runs),
        dsa.episode_summary.max_dsa_episode_length_runs,
        dsa.episode_summary.raw_boundary_episode_count,
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.non_escalating_dsa_episode_fraction),
        dsa.comparison_summary.primary_success_condition_met,
        if dsa.comparison_summary.success_condition_failures.is_empty() {
            "none".to_string()
        } else {
            dsa.comparison_summary.success_condition_failures.join("; ")
        },
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

    out.push_str("## DSA Calibration Grid\n\n");
    out.push_str(&format!(
        "- Grid points evaluated: {}\n- Optimization priority order: {}\n- Primary success condition: {}\n- Success rows in bounded grid: {}\n- Cross-feature corroboration effect: {}\n- Limiting factor: {}\n",
        dsa_grid_summary.grid_point_count,
        dsa_grid_summary.optimization_priority_order.join(" | "),
        dsa_grid_summary.primary_success_condition_definition,
        dsa_grid_summary.success_row_count,
        dsa_grid_summary.cross_feature_corroboration_effect,
        dsa_grid_summary.limiting_factor,
    ));
    if let Some(row) = &dsa_grid_summary.closest_to_success {
        out.push_str(&format!(
            "- Closest to primary success: config_id={}, W={}, K={}, tau={:.2}, m={}, recall={}/{}, mean lead={}, nuisance={:.4}, precursor quality={}, compression ratio={}\n",
            row.config_id,
            row.window,
            row.persistence_runs,
            row.alert_tau,
            row.corroborating_feature_count_min,
            row.failure_run_recall,
            row.failure_runs,
            format_option_f64(row.mean_lead_time_runs),
            row.pass_run_nuisance_proxy,
            format_option_f64(row.precursor_quality),
            format_option_f64(row.compression_ratio),
        ));
    }
    if let Some(row) = &dsa_grid_summary.best_precursor_quality_row {
        out.push_str(&format!(
            "- Highest precursor-quality row: config_id={}, W={}, K={}, tau={:.2}, m={}, precursor quality={}\n",
            row.config_id,
            row.window,
            row.persistence_runs,
            row.alert_tau,
            row.corroborating_feature_count_min,
            format_option_f64(row.precursor_quality),
        ));
    }
    out.push_str("- Saved grid artifacts: `dsa_grid_results.csv` and `dsa_grid_summary.json`\n\n");

    out.push_str("## Density Summary\n\n");
    out.push_str(&format!(
        "- Density window: {} runs\n- Mean persistent boundary density, failure-labeled runs: {:.4}\n- Mean persistent boundary density, pass-labeled runs: {:.4}\n- Mean persistent violation density, failure-labeled runs: {:.4}\n- Mean persistent violation density, pass-labeled runs: {:.4}\n- Mean threshold density, failure-labeled runs: {:.4}\n- Mean threshold density, pass-labeled runs: {:.4}\n- Mean EWMA density, failure-labeled runs: {:.4}\n- Mean EWMA density, pass-labeled runs: {:.4}\n- Mean CUSUM density, failure-labeled runs: {:.4}\n- Mean CUSUM density, pass-labeled runs: {:.4}\n\n",
        metrics.density_summary.density_window,
        metrics.density_summary.mean_persistent_boundary_density_failure,
        metrics.density_summary.mean_persistent_boundary_density_pass,
        metrics.density_summary.mean_persistent_violation_density_failure,
        metrics.density_summary.mean_persistent_violation_density_pass,
        metrics.density_summary.mean_threshold_density_failure,
        metrics.density_summary.mean_threshold_density_pass,
        metrics.density_summary.mean_ewma_density_failure,
        metrics.density_summary.mean_ewma_density_pass,
        metrics.density_summary.mean_cusum_density_failure,
        metrics.density_summary.mean_cusum_density_pass,
    ));

    out.push_str(&drsc_dsa_combined_markdown_section(figures));
    out.push_str(&drsc_markdown_section(figures));
    out.push_str(&dsa_focus_markdown_section(figures));

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
        "- Official page: {}\n- Manual archive path: {}\n- Archive summary support implemented: {}\n- Implemented now: {}\n- Blocker: {}\n\n",
        phm_status.official_page,
        phm_status.manual_placement_path.display(),
        phm_status.archive_summary_supported,
        phm_status.fully_implemented,
        phm_status.blocker,
    ));

    out.push_str("## Limitations of This Run\n\n");
    out.push_str("- SECOM is anonymized and instance-level; this run does not validate chamber-mechanism attribution or run-to-failure prognostics.\n");
    out.push_str("- The comparator set is still intentionally bounded: raw threshold, EWMA, positive CUSUM, run-level residual energy, PCA T2/SPE multivariate FDC, DSFB boundary, and DSFB Violation. ML baselines are intentionally not claimed here.\n");
    out.push_str("- Lead-time and nuisance values are bounded proxy metrics derived from SECOM labels and a fixed lookback, not fab-qualified operational KPIs.\n");
    out.push_str("- PHM 2018 support is still limited to the manual-placement contract, archive probe, grouped CSV-schema summary, and archive-shape ingestion summary until the real archive is present and verified end to end.\n");
    out.push_str("- DRSC keeps the DSFB state semantics intact, but now overlays DSA/run-level comparator traces for the selected feature window.\n");
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
    dsa_grid_summary: &DsaGridSummary,
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
        "The nominal reference is the healthy-window mean over the first {} passing runs. Residuals are defined as x(k) - x_hat. The admissibility envelope radius is {:.1} sigma on the healthy residual distribution. The drift window is W = {}. The boundary rule in this implementation is |r| > {:.1} rho with drift above {:.1} healthy drift sigma. Abrupt slew tags use {:.1} healthy slew sigma. Hysteresis-confirmed state changes require {} confirmations, persistent-state alarms require {} consecutive confirmed steps, and density metrics use a {}-run sliding window. The comparator set contains a raw residual threshold, a univariate EWMA on residual norms with alpha = {:.2} and a threshold at the healthy-window EWMA mean plus {:.1} sigma, a positive CUSUM on residual norms with kappa = {:.1} healthy sigma and alarm threshold = {:.1} healthy sigma, a run-level residual-energy baseline with threshold mean plus {:.1} healthy run-energy sigma, and a PCA T2/SPE multivariate FDC baseline retaining {:.0}% healthy variance with T2/SPE thresholds at mean plus {:.1}/{:.1} healthy sigma. DSFB Violation remains the instantaneous hard envelope exit state. DSA is additive, sits above the existing DSFB outputs, and uses W = {}, K = {}, tau = {:.2}, m = {}, fixed unit weights, primary run signal {}, and a consistency rule that rejects thresholded inward drift and thresholded drift-sign flips.",
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
        config.cusum_kappa_sigma_multiplier,
        config.cusum_alarm_sigma_multiplier,
        config.run_energy_sigma_multiplier,
        config.pca_variance_explained * 100.0,
        config.pca_t2_sigma_multiplier,
        config.pca_spe_sigma_multiplier,
        config.dsa.window,
        config.dsa.persistence_runs,
        config.dsa.alert_tau,
        config.dsa.corroborating_feature_count_min,
        dsa.run_signals.primary_run_signal,
    )));
    out.push_str("\n\n");

    out.push_str("\\section*{Deterministic Structural Accumulator (DSA)}\n");
    out.push_str("\\begin{tabular}{lr}\n\\toprule\n");
    out.push_str(&format!(
        "Failure-run recall, DSA & {}/{} \\\\\nFailure-run recall, threshold & {}/{} \\\\\nFailure-run recall, EWMA & {}/{} \\\\\nFailure-run recall, CUSUM & {}/{} \\\\\nFailure-run recall, run energy & {}/{} \\\\\nFailure-run recall, PCA T2/SPE & {}/{} \\\\\nFailure-run recall, DSFB Violation & {}/{} \\\\\nMean lead time, DSA & {} \\\\\nMedian lead time, DSA & {} \\\\\nPass-run nuisance proxy, DSA & {:.4} \\\\\nLead delta vs CUSUM & {} \\\\\nLead delta vs run energy & {} \\\\\nLead delta vs PCA T2/SPE & {} \\\\\nLead delta vs threshold & {} \\\\\nLead delta vs EWMA & {} \\\\\nNuisance delta vs threshold & {:.4} \\\\\nNuisance delta vs EWMA & {:.4} \\\\\nNuisance delta vs DSFB Violation & {:.4} \\\\\nNuisance delta vs CUSUM & {:.4} \\\\\nNuisance delta vs run energy & {:.4} \\\\\nNuisance delta vs PCA T2/SPE & {:.4} \\\\\nNuisance delta vs raw boundary & {:.4} \\\\\nRaw boundary episodes & {} \\\\\nDSA episodes & {} \\\\\nDSA episodes preceding failure & {} \\\\\nPrecursor quality & {} \\\\\nCompression ratio & {} \\\\\nNon-escalating DSA episode fraction & {} \\\\\nPrimary success condition met & {} \\\\\nThreshold recall gate passed & {} \\\\\nBoundary nuisance gate passed & {} \\\\\nValidation passed & {} \\\\\n",
        dsa.comparison_summary.dsa.failure_run_recall,
        dsa.comparison_summary.dsa.failure_runs,
        dsa.comparison_summary.threshold.failure_run_recall,
        dsa.comparison_summary.threshold.failure_runs,
        dsa.comparison_summary.ewma.failure_run_recall,
        dsa.comparison_summary.ewma.failure_runs,
        dsa.comparison_summary.cusum.failure_run_recall,
        dsa.comparison_summary.cusum.failure_runs,
        dsa.comparison_summary.run_energy.failure_run_recall,
        dsa.comparison_summary.run_energy.failure_runs,
        dsa.comparison_summary.pca_fdc.failure_run_recall,
        dsa.comparison_summary.pca_fdc.failure_runs,
        dsa.comparison_summary.dsfb_violation.failure_run_recall,
        dsa.comparison_summary.dsfb_violation.failure_runs,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_time_runs),
        format_option_f64(dsa.comparison_summary.dsa.median_lead_time_runs),
        dsa.comparison_summary.dsa.pass_run_nuisance_proxy,
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_cusum_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_run_energy_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_pca_fdc_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_threshold_runs),
        format_option_f64(dsa.comparison_summary.dsa.mean_lead_delta_vs_ewma_runs),
        dsa.comparison_summary.pass_run_nuisance_delta_vs_threshold,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_ewma,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_violation,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_cusum,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_run_energy,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_pca_fdc,
        dsa.comparison_summary.pass_run_nuisance_delta_vs_raw_boundary,
        dsa.episode_summary.raw_boundary_episode_count,
        dsa.episode_summary.dsa_episode_count,
        dsa.episode_summary.dsa_episodes_preceding_failure,
        format_option_f64(dsa.episode_summary.precursor_quality),
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.non_escalating_dsa_episode_fraction),
        dsa.comparison_summary.primary_success_condition_met,
        dsa.comparison_summary.threshold_recall_gate_passed,
        dsa.comparison_summary.boundary_nuisance_gate_passed,
        dsa.comparison_summary.validation_passed,
    ));
    out.push_str("\\bottomrule\n\\end{tabular}\n\n");
    out.push_str(&latex_escape(&dsa.comparison_summary.conclusion));
    out.push_str("\n\n");

    out.push_str("\\section*{DSA calibration grid}\n");
    out.push_str(&latex_escape(&format!(
        "Grid points evaluated: {}. Optimization priority order: {}. Primary success condition: {}. Success rows in bounded grid: {}. Cross-feature corroboration effect: {}. Limiting factor: {}.",
        dsa_grid_summary.grid_point_count,
        dsa_grid_summary.optimization_priority_order.join(" | "),
        dsa_grid_summary.primary_success_condition_definition,
        dsa_grid_summary.success_row_count,
        dsa_grid_summary.cross_feature_corroboration_effect,
        dsa_grid_summary.limiting_factor,
    )));
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

    out.push_str(&drsc_dsa_combined_latex_section(figures));
    out.push_str(&drsc_latex_section(figures));
    out.push_str(&dsa_focus_latex_section(figures));

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
        "Official page: {}. Manual archive path: {}. Archive summary support implemented: {}. Implemented now: {}. Blocker: {}.",
        phm_status.official_page,
        phm_status.manual_placement_path.display(),
        phm_status.archive_summary_supported,
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
            role: "Baseline comparison summary across DSFB, threshold, EWMA, CUSUM, run energy, PCA T2/SPE, and DSA.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_vs_baselines.json".into(),
            role: "Saved DSA recall, lead-time, nuisance, validation, and compression summary.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_grid_results.csv".into(),
            role: "Full bounded DSA calibration grid with W, K, tau, and corroboration m.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_grid_summary.json".into(),
            role: "Saved DSA grid summary, closest-to-success row, corroboration effect, and limiting-factor analysis.".into(),
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
            role: "Flattened DSFB, threshold, EWMA, CUSUM, and run-energy lead-time table.".into(),
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
            path: "cusum_baseline.csv".into(),
            role: "Positive CUSUM baseline trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "run_energy_baseline.csv".into(),
            role: "Run-level residual-energy baseline trace export.".into(),
        },
        ArtifactInventoryEntry {
            path: "pca_fdc_baseline.csv".into(),
            role: "PCA T2/SPE multivariate FDC baseline trace export.".into(),
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
    if figures.drsc_dsa_combined.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "drsc_dsa_combined.csv".into(),
            role: "Aligned DRSC+DSA publication figure trace export for the selected feature window.".into(),
        });
    }
    if figures.dsa_focus.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "dsa_top_feature.csv".into(),
            role: "Aligned DSA structural-focus trace export for the selected feature window.".into(),
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

fn drsc_dsa_combined_markdown_section(figures: &FigureManifest) -> String {
    if let Some(combined) = &figures.drsc_dsa_combined {
        format!(
            "## Deterministic Residual Stateflow Chart with Structural Accumulation (DRSC+DSA)\n\nThe crate emits a publication-oriented combined figure and aligned trace CSV for the representative top boundary-activity feature in the current run (`{}`). The figure keeps the DSFB semantics frozen while making four aligned layers readable in one glance: normalized residual / drift / slew; the actual persistent DSFB state band (`Admissible`, `Boundary`, `Violation`); the DSA precursor layer rendered as feature-level plus corroborated run-level activation; and run-level threshold / EWMA scalar trigger timing. This figure is grayscale-safe, generated from crate outputs, and does not claim scalar lag unless it is visible in the saved traces.\n\n- Figure: figures/{}\n- Trace CSV: {}\n- Feature selection basis: {}\n- Normalization: {}\n- State display: {}\n- DSA rendering: {}\n- Baseline rendering: {}\n\n",
            combined.feature_name,
            combined.figure_file,
            combined.trace_csv,
            combined.feature_selection_basis,
            combined.normalization_note,
            combined.state_display_note,
            combined.dsa_rendering_note,
            combined.baseline_rendering_note,
        )
    } else {
        String::new()
    }
}

fn drsc_markdown_section(figures: &FigureManifest) -> String {
    if let Some(drsc) = &figures.drsc {
        format!(
            "## Deterministic Residual Stateflow Chart (DRSC)\n\nThe crate emits a DRSC figure and aligned trace CSV for the top persistent-boundary feature in the current run (`{}`). The chart keeps the DSFB state semantics intact: top layer residual/drift/slew, middle persistent DSFB states, bottom admissibility and comparator occupancy. DSA and run-level comparator overlays are added to the same selected feature window without redefining DSFB state semantics.\n\n- Figure: figures/{}\n- Trace CSV: {}\n\n",
            drsc.feature_name, drsc.figure_file, drsc.trace_csv,
        )
    } else {
        String::new()
    }
}

fn dsa_focus_markdown_section(figures: &FigureManifest) -> String {
    if let Some(dsa_focus) = &figures.dsa_focus {
        format!(
            "## DSA Structural Focus Figure\n\nThe crate emits a DSA-specific figure and aligned trace CSV for the selected feature window (`{}`). This separate chart exposes the rolling structural inputs, DSA score, persistence gate, and feature-level comparator band, including run energy and PCA T2/SPE overlays.\n\n- Figure: figures/{}\n- Trace CSV: {}\n\n",
            dsa_focus.feature_name, dsa_focus.figure_file, dsa_focus.trace_csv,
        )
    } else {
        String::new()
    }
}

fn drsc_dsa_combined_latex_section(figures: &FigureManifest) -> String {
    if let Some(combined) = &figures.drsc_dsa_combined {
        format!(
            "\\section*{{Deterministic Residual Stateflow Chart with Structural Accumulation (DRSC+DSA)}}\nThe crate emits a publication-oriented combined figure and aligned trace CSV for the representative top boundary-activity feature in the current run (\\texttt{{{}}}). The figure keeps the DSFB semantics frozen while making four aligned layers readable in one glance: normalized residual / drift / slew; the actual persistent DSFB state band (\\texttt{{Admissible}}, \\texttt{{Boundary}}, \\texttt{{Violation}}); the DSA precursor layer rendered as feature-level plus corroborated run-level activation; and run-level threshold / EWMA scalar trigger timing. The figure is generated from crate outputs and does not claim scalar lag unless it is visible in the saved traces. The aligned trace CSV is \\texttt{{{}}}.\n\n",
            latex_escape(&combined.feature_name),
            latex_escape(&combined.trace_csv),
        )
    } else {
        String::new()
    }
}

fn drsc_latex_section(figures: &FigureManifest) -> String {
    if let Some(drsc) = &figures.drsc {
        format!(
            "\\section*{{Deterministic Residual Stateflow Chart (DRSC)}}\nThe crate emits a DRSC figure and aligned trace CSV for the top persistent-boundary feature in the current run (\\texttt{{{}}}). The chart keeps the DSFB state semantics intact: top layer residual, drift, and slew; middle layer persistent DSFB states; bottom layer admissibility and comparator occupancy. DSA and run-level comparator overlays are added to the same selected feature window without redefining DSFB state semantics. The aligned trace CSV is \\texttt{{{}}}.\n\n",
            latex_escape(&drsc.feature_name),
            latex_escape(&drsc.trace_csv),
        )
    } else {
        String::new()
    }
}

fn dsa_focus_latex_section(figures: &FigureManifest) -> String {
    if let Some(dsa_focus) = &figures.dsa_focus {
        format!(
            "\\section*{{DSA structural focus figure}}\nThe crate emits a DSA-specific figure and aligned trace CSV for the selected feature window (\\texttt{{{}}}). This separate chart exposes the rolling structural inputs, DSA score, persistence gate, and feature-level comparator band, including run energy and PCA T2/SPE overlays. The aligned trace CSV is \\texttt{{{}}}.\n\n",
            latex_escape(&dsa_focus.feature_name),
            latex_escape(&dsa_focus.trace_csv),
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
                .drsc_dsa_combined
                .as_ref()
                .map(|combined| combined.figure_file == *file)
                .unwrap_or(false)
            {
                "Deterministic Residual Stateflow Chart with Structural Accumulation (DRSC+DSA) for the representative SECOM feature selected by the current run. Top: normalized residual, drift, and slew. Second: persistent deterministic DSFB state evolution. Third: feature-level and corroborated run-level DSA activation. Bottom: run-level threshold and EWMA trigger timing. The figure is generated from crate outputs and is intended to expose the difference between raw structural activity and persistence-constrained precursor regimes."
                    .to_string()
            } else if figures
                .drsc
                .as_ref()
                .map(|drsc| drsc.figure_file == *file)
                .unwrap_or(false)
            {
                "Deterministic Residual Stateflow Chart (DRSC) for the selected feature window."
                    .to_string()
            } else if figures
                .dsa_focus
                .as_ref()
                .map(|dsa_focus| dsa_focus.figure_file == *file)
                .unwrap_or(false)
            {
                "DSA structural focus figure for the selected feature window.".to_string()
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

    for _ in 0..3 {
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
                let pass_output = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stderr),
                    String::from_utf8_lossy(&output.stdout)
                );
                let needs_rerun = pass_output.contains("Rerun to get outlines right")
                    || pass_output.contains("Label(s) may have changed")
                    || pass_output.contains("Rerun to get cross-references right");
                combined_output.push_str(&String::from_utf8_lossy(&output.stderr));
                combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
                if output.status.success() {
                    any_success = true;
                    if !needs_rerun {
                        break;
                    }
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
