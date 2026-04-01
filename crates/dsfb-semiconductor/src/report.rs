use crate::cohort::{
    cohort_report_section, rating_forecast_report_section, CohortDsaSummary, DeltaTargetAssessment,
    FeatureCohorts, OptimizationExecution, RatingDeltaForecast,
};
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
    optimization: &OptimizationExecution,
    delta_target_assessment: &DeltaTargetAssessment,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
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
            optimization,
            delta_target_assessment,
            feature_cohorts,
            cohort_summary,
            rating_delta_forecast,
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
            optimization,
            delta_target_assessment,
            feature_cohorts,
            cohort_summary,
            rating_delta_forecast,
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
    optimization: &OptimizationExecution,
    delta_target_assessment: &DeltaTargetAssessment,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let mut out = String::new();
    let artifact_inventory = artifact_inventory(
        figures,
        cohort_summary.failure_analysis.is_some(),
        !rating_delta_forecast.primary_success_met,
    );

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
    out.push_str("Missing values remain explicit during dataset loading and are deterministically imputed with the healthy-window nominal mean before residual construction. Stage III treats those imputed observations as structurally invalid for DSFB drift, slew, grammar-state assignment, and the boundary/drift fractions used by DSA.\n\n");

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
    out.push_str("Stage III keeps the nominal reference, envelope, violation definition, hysteresis, motif definitions, and DSA weights fixed. The change in this pass is missingness-aware signal validity: drift and slew are zeroed across imputed gaps, grammar states are suppressed to Admissible on imputed runs, and DSA boundary/drift fractions exclude imputed runs from the window denominator.\n\n");
    out.push_str(&format!(
        "In this crate, `DSFB Violation` remains instantaneous hard envelope exit (`|r| > rho`). `Deterministic Structural Accumulator (DSA)` is additive and sits above the existing DSFB outputs. The feature-level DSA precursor itself remains persistence-constrained, and the run-level comparison signal is cross-feature corroboration: `{}`. The current DSA configuration uses `W = {}`, `K = {}`, `tau = {:.2}`, `m = {}`, fixed unit weights, and a consistency rule that rejects thresholded inward drift and thresholded drift-sign flips.\n\n",
        dsa.run_signals.primary_run_signal,
        dsa.summary.config.window,
        dsa.summary.config.persistence_runs,
        dsa.summary.config.alert_tau,
        dsa.summary.config.corroborating_feature_count_min,
    ));

    out.push_str("## Quantitative Summary\n\n");
    out.push_str(&format!(
        "- Analyzable features: {}\n- Grammar-state suppressions due to imputation: {}\n- Threshold alarm points: {}\n- EWMA alarm points: {}\n- CUSUM alarm points: {}\n- Run-energy alarm points: {}\n- PCA T2/SPE alarm points: {}\n- DSFB raw boundary points: {}\n- DSFB persistent boundary points: {}\n- DSFB raw violation points: {}\n- DSFB persistent violation points: {}\n- DSA alert points: {}\n- DSA alert runs: {}\n- Failure runs with preceding DSA signal ({}-run lookback): {}\n- Failure runs with preceding DSFB Violation signal ({}-run lookback): {}\n- Failure runs with preceding raw DSFB boundary signal ({}-run lookback): {}\n- Failure runs with preceding EWMA signal ({}-run lookback): {}\n- Failure runs with preceding CUSUM signal ({}-run lookback): {}\n- Failure runs with preceding run-energy signal ({}-run lookback): {}\n- Failure runs with preceding PCA T2/SPE signal ({}-run lookback): {}\n- Failure runs with preceding threshold signal ({}-run lookback): {}\n\n",
        metrics.summary.analyzable_feature_count,
        metrics.summary.grammar_imputation_suppression_points,
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
    out.push_str("- The predeclared 40% nuisance-reduction target is evaluated separately below; the legacy one-run nuisance/recall sweep gate reported here is not sufficient for target attainment\n");
    out.push_str(&format!(
        "- Primary run-level comparison signal: `{}`\n- Primary run-level signal definition: `{}`\n- Secondary run-level signal emitted: `{}`\n- Tertiary run-level signal emitted: `{}`\n- Failure-run recall, DSA: {}/{}\n- Failure-run recall, threshold: {}/{}\n- Failure-run recall, EWMA: {}/{}\n- Failure-run recall, CUSUM: {}/{}\n- Failure-run recall, run energy: {}/{}\n- Failure-run recall, PCA T2/SPE: {}/{}\n- Failure-run recall, DSFB Violation: {}/{}\n- Mean lead time, DSA: {}\n- Median lead time, DSA: {}\n- Pass-run nuisance proxy, DSA: {:.4}\n- Lead delta vs CUSUM (runs): {}\n- Lead delta vs run energy (runs): {}\n- Lead delta vs PCA T2/SPE (runs): {}\n- Lead delta vs threshold (runs): {}\n- Lead delta vs EWMA (runs): {}\n- Nuisance delta vs threshold: {:.4}\n- Nuisance delta vs EWMA: {:.4}\n- Nuisance delta vs DSFB Violation: {:.4}\n- Nuisance delta vs CUSUM: {:.4}\n- Nuisance delta vs run energy: {:.4}\n- Nuisance delta vs PCA T2/SPE: {:.4}\n- Nuisance delta vs raw DSFB boundary: {:.4}\n- DSA episodes: {}\n- DSA episodes preceding failure: {}\n- Precursor quality: {}\n- Mean DSA episode length (runs): {}\n- Max DSA episode length (runs): {}\n- Raw boundary episodes: {}\n- Compression ratio (raw boundary / DSA): {}\n- Non-escalating DSA episode fraction: {}\n- Legacy one-run nuisance/recall sweep gate met: {}\n- Legacy gate failures: {}\n- Nuisance improved: {}\n- Lead time improved: {}\n- Recall preserved: {}\n- Compression improved: {}\n- Nothing improved: {}\n- Legacy threshold-minus-one recall gate passed: {}\n- Legacy boundary nuisance gate passed: {}\n- Stricter validation passed: {}\n\n{}\n\n",
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
        "- Grid points evaluated: {}\n- Optimization priority order: {}\n- Legacy one-run nuisance/recall sweep gate: {}\n- Rows meeting the legacy gate in the bounded grid: {}\n- Cross-feature corroboration effect: {}\n- Limiting factor: {}\n",
        cohort_summary.grid_point_count,
        cohort_summary.optimization_priority_order.join(" | "),
        cohort_summary.primary_success_condition,
        cohort_summary
            .cohort_results
            .iter()
            .filter(|row| row.primary_success)
            .count(),
        cohort_summary.cross_feature_corroboration_effect,
        cohort_summary.limiting_factor,
    ));
    if let Some(row) = &cohort_summary.closest_to_success {
        out.push_str(&format!(
            "- Closest to the legacy sweep gate: grid_row_id={}, cohort={}, W={}, K={}, tau={:.2}, m={}, recall={}/{}, mean lead={}, nuisance={:.4}, precursor quality={}, compression ratio={}\n",
            row.grid_row_id,
            row.cohort_name,
            row.window,
            row.persistence_runs,
            row.alert_tau,
            row.corroborating_m,
            row.failure_recall,
            row.failure_runs,
            format_option_f64(row.mean_lead_time_runs),
            row.pass_run_nuisance_proxy,
            format_option_f64(row.precursor_quality),
            format_option_f64(row.compression_ratio),
        ));
    }
    if let Some(row) = &cohort_summary.best_precursor_quality_row {
        out.push_str(&format!(
            "- Highest precursor-quality row: grid_row_id={}, cohort={}, W={}, K={}, tau={:.2}, m={}, precursor quality={}\n",
            row.grid_row_id,
            row.cohort_name,
            row.window,
            row.persistence_runs,
            row.alert_tau,
            row.corroborating_m,
            format_option_f64(row.precursor_quality),
        ));
    }
    out.push_str("- Saved grid artifacts: `dsa_grid_results.csv` and `dsa_grid_summary.json`\n\n");
    out.push_str(&cohort_report_section(feature_cohorts, cohort_summary));
    out.push_str(&heuristics_policy_engine_markdown_section(
        heuristics,
        dsa,
        cohort_summary,
    ));
    out.push_str(&semantics_of_silence_markdown_section(metrics, dsa));
    out.push_str(&predeclared_delta_target_markdown_section(
        delta_target_assessment,
    ));
    out.push_str(&recall_recovery_diagnostics_markdown_section(optimization));
    out.push_str(&feature_aware_governance_markdown_section(optimization));
    out.push_str(&missed_failure_diagnostics_markdown_section(optimization));
    out.push_str(&two_stage_optimization_frontier_markdown_section(
        optimization,
        delta_target_assessment,
    ));
    out.push_str(&target_attainment_markdown_section(delta_target_assessment));
    out.push_str(&rating_forecast_report_section(rating_delta_forecast));

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
    out.push_str(
        "| Motif | Point hits | Run hits | Pre-failure window run hits | Precision proxy |\n",
    );
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
    out.push_str(
        "| Motif | Provenance | Contributes to DSA scoring | Severity | Recommended action |\n",
    );
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
    optimization: &OptimizationExecution,
    delta_target_assessment: &DeltaTargetAssessment,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
    figures: &FigureManifest,
    heuristics: &[HeuristicEntry],
    phm_status: &Phm2018SupportStatus,
    secom_layout: &SecomArchiveLayout,
) -> String {
    let mut out = String::new();
    let artifact_inventory = artifact_inventory(
        figures,
        cohort_summary.failure_analysis.is_some(),
        !rating_delta_forecast.primary_success_met,
    );

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
        dsa.summary.config.window,
        dsa.summary.config.persistence_runs,
        dsa.summary.config.alert_tau,
        dsa.summary.config.corroborating_feature_count_min,
        dsa.run_signals.primary_run_signal,
    )));
    out.push_str("\n\n");

    out.push_str("\\section*{Deterministic Structural Accumulator (DSA)}\n");
    out.push_str(&latex_escape(
        "The predeclared 40% nuisance-reduction target is evaluated separately below; the legacy one-run nuisance/recall sweep gate reported in this section is not sufficient for target attainment.",
    ));
    out.push_str("\n\n");
    out.push_str("\\begin{tabular}{lr}\n\\toprule\n");
    out.push_str(&format!(
        "Failure-run recall, DSA & {}/{} \\\\\nFailure-run recall, threshold & {}/{} \\\\\nFailure-run recall, EWMA & {}/{} \\\\\nFailure-run recall, CUSUM & {}/{} \\\\\nFailure-run recall, run energy & {}/{} \\\\\nFailure-run recall, PCA T2/SPE & {}/{} \\\\\nFailure-run recall, DSFB Violation & {}/{} \\\\\nMean lead time, DSA & {} \\\\\nMedian lead time, DSA & {} \\\\\nPass-run nuisance proxy, DSA & {:.4} \\\\\nLead delta vs CUSUM & {} \\\\\nLead delta vs run energy & {} \\\\\nLead delta vs PCA T2/SPE & {} \\\\\nLead delta vs threshold & {} \\\\\nLead delta vs EWMA & {} \\\\\nNuisance delta vs threshold & {:.4} \\\\\nNuisance delta vs EWMA & {:.4} \\\\\nNuisance delta vs DSFB Violation & {:.4} \\\\\nNuisance delta vs CUSUM & {:.4} \\\\\nNuisance delta vs run energy & {:.4} \\\\\nNuisance delta vs PCA T2/SPE & {:.4} \\\\\nNuisance delta vs raw boundary & {:.4} \\\\\nRaw boundary episodes & {} \\\\\nDSA episodes & {} \\\\\nDSA episodes preceding failure & {} \\\\\nPrecursor quality & {} \\\\\nCompression ratio & {} \\\\\nNon-escalating DSA episode fraction & {} \\\\\nLegacy one-run nuisance/recall sweep gate met & {} \\\\\nLegacy threshold-minus-one recall gate passed & {} \\\\\nLegacy boundary nuisance gate passed & {} \\\\\nStricter validation passed & {} \\\\\n",
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
        "Grid points evaluated: {}. Optimization priority order: {}. Legacy one-run nuisance/recall sweep gate: {}. Rows meeting the legacy gate in the bounded grid: {}. Cross-feature corroboration effect: {}. Limiting factor: {}.",
        cohort_summary.grid_point_count,
        cohort_summary.optimization_priority_order.join(" | "),
        cohort_summary.primary_success_condition,
        cohort_summary
            .cohort_results
            .iter()
            .filter(|row| row.primary_success)
            .count(),
        cohort_summary.cross_feature_corroboration_effect,
        cohort_summary.limiting_factor,
    )));
    out.push_str("\n\n");
    out.push_str(&feature_cohort_latex_section(
        feature_cohorts,
        cohort_summary,
    ));
    out.push_str(&heuristics_policy_engine_latex_section(
        heuristics,
        dsa,
        cohort_summary,
    ));
    out.push_str(&semantics_of_silence_latex_section(metrics, dsa));
    out.push_str(&optimization_sections_latex(
        optimization,
        delta_target_assessment,
    ));
    out.push_str(&rating_forecast_latex_section(rating_delta_forecast));

    out.push_str("\\section*{Motif metrics}\n");
    out.push_str("\\begin{longtable}{p{0.26\\linewidth}rrrr}\n\\toprule\n");
    out.push_str(
        "Motif & Point hits & Run hits & Pre-failure run hits & Precision proxy \\\\\n\\midrule\n",
    );
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
    out.push_str(
        "\\item No standards-compliance, completed qualification, or SEMI compatibility claim.\n",
    );
    out.push_str(
        "\\item No chamber-mechanism or physical root-cause attribution from SECOM alone.\n",
    );
    out.push_str(
        "\\item No PHM 2018 completion claim unless the real archive is staged and verified.\n",
    );
    out.push_str("\\item No Kani verification, no\\_alloc, SIMD, rayon, or parallel-acceleration claim for this crate.\n");
    out.push_str("\\end{itemize}\n\n");

    out.push_str(&figure_blocks(figures));
    out.push_str("\\end{document}\n");
    out
}

fn heuristics_policy_engine_markdown_section(
    heuristics: &[HeuristicEntry],
    dsa: &DsaEvaluation,
    cohort_summary: &CohortDsaSummary,
) -> String {
    let mut out = String::new();
    out.push_str("## Heuristics-Governed DSA Policy Engine\n\n");
    out.push_str("- Current motif set used: ");
    out.push_str(
        &heuristics
            .iter()
            .map(|entry| entry.motif_name.clone())
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("\n");
    out.push_str("- Policy fields used: `alert_class_default`, `requires_persistence`, `requires_corroboration`, `minimum_window`, `minimum_hits`, `maximum_allowed_fragmentation`, `suppresses_alert`, `promotes_alert`\n");
    out.push_str("- Legacy one-run-tolerance cohort gate used in the bounded sweep: ");
    out.push_str(&cohort_summary.primary_success_condition);
    out.push_str("\n\n");
    out.push_str("| Motif | Default class | Persistence | Corroboration | Min window | Min hits | Max fragmentation | Suppresses | Promotes |\n");
    out.push_str("|---|---|---|---|---:|---:|---:|---|---|\n");
    for entry in heuristics {
        out.push_str(&format!(
            "| {} | {:?} | {} | {} | {} | {} | {:.4} | {} | {} |\n",
            entry.motif_name,
            entry.alert_class_default,
            entry.requires_persistence,
            entry.requires_corroboration,
            entry.minimum_window,
            entry.minimum_hits,
            entry.maximum_allowed_fragmentation,
            entry.suppresses_alert,
            entry.promotes_alert,
        ));
    }
    out.push('\n');
    if let Some(selected) = &cohort_summary.selected_configuration {
        out.push_str(&format!(
            "- Best cohort result: {} with recall {}/{}, nuisance {:.4}, numeric-only nuisance {:.4}, mean lead {}, precursor quality {}, and compression {}\n",
            selected.cohort_name,
            selected.failure_recall,
            selected.failure_runs,
            selected.pass_run_nuisance_proxy,
            selected.numeric_pass_run_nuisance_proxy,
            format_option_f64(selected.mean_lead_time_runs),
            format_option_f64(selected.precursor_quality),
            format_option_f64(selected.compression_ratio),
        ));
    }
    out.push_str(&format!(
        "- Legacy one-run-tolerance cohort gate met: {}\n",
        cohort_summary.any_primary_success
    ));
    if let Some(analysis) = &cohort_summary.failure_analysis {
        out.push_str(&format!(
            "- Failure analysis: {}\n- Policy vs numeric-only DSA: {}\n- Nuisance-dominant motif class: {}\n- Useful precursor motif class: {}\n",
            analysis.limiting_factor,
            analysis.policy_vs_numeric_note,
            analysis.nuisance_motif_classes,
            analysis.useful_precursor_motif_classes,
        ));
    }
    out.push_str(&format!(
        "- Feature-state counts in selected evaluation: Watch={}, Review={}, Escalate={}, Silent suppression points={}\n\n",
        dsa.summary.watch_point_count,
        dsa.summary.review_point_count,
        dsa.summary.escalate_point_count,
        dsa.summary.silenced_point_count,
    ));
    out
}

fn semantics_of_silence_markdown_section(
    metrics: &BenchmarkMetrics,
    dsa: &DsaEvaluation,
) -> String {
    format!(
        "## Semantics of Silence\n\n- Silence rule: {}\n- Grammar-state suppressions due to imputation: {}\n- Numeric-only DSA alert points: {}\n- Policy-governed Review/Escalate alert points: {}\n- Explicitly silenced points: {}\n- Policy nuisance: {:.4} versus numeric-only DSA {:.4} and EWMA {:.4}\n- Policy recall: {}/{} versus numeric-only DSA {}/{}\n- Watch/Review/Escalate points: {}/{}/{}\n- Raw boundary episodes: {}\n- Policy-governed DSA episodes: {}\n- Compression ratio: {}\n- Precursor quality: {}\n\n",
        dsa.parameter_manifest.silence_rule,
        metrics.summary.grammar_imputation_suppression_points,
        dsa.summary.numeric_alert_point_count,
        dsa.summary.alert_point_count,
        dsa.summary.silenced_point_count,
        dsa.summary.pass_run_nuisance_proxy,
        dsa.summary.numeric_primary_pass_run_nuisance_proxy,
        dsa.comparison_summary.ewma.pass_run_nuisance_proxy,
        dsa.summary.failure_run_recall,
        dsa.summary.failure_runs,
        dsa.summary.numeric_primary_failure_run_recall,
        dsa.summary.failure_runs,
        dsa.summary.watch_point_count,
        dsa.summary.review_point_count,
        dsa.summary.escalate_point_count,
        dsa.episode_summary.raw_boundary_episode_count,
        dsa.episode_summary.dsa_episode_count,
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.precursor_quality),
    )
}

fn heuristics_policy_engine_latex_section(
    heuristics: &[HeuristicEntry],
    dsa: &DsaEvaluation,
    cohort_summary: &CohortDsaSummary,
) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Heuristics-Governed DSA Policy Engine}\n");
    out.push_str(&latex_escape(&format!(
        "Current motif set: {}. Policy fields used: alert_class_default, requires_persistence, requires_corroboration, minimum_window, minimum_hits, maximum_allowed_fragmentation, suppresses_alert, and promotes_alert. Legacy one-run-tolerance cohort gate used in the bounded sweep: {}.",
        heuristics
            .iter()
            .map(|entry| entry.motif_name.clone())
            .collect::<Vec<_>>()
            .join(", "),
        cohort_summary.primary_success_condition,
    )));
    out.push_str("\n\n");
    out.push_str("\\begin{longtable}{p{0.22\\linewidth}p{0.12\\linewidth}ccrrcc}\n\\toprule\n");
    out.push_str("Motif & Default class & Persist & Corrob & Window & Hits & Suppress & Promote \\\\\n\\midrule\n");
    for entry in heuristics {
        out.push_str(&format!(
            "{} & {} & {} & {} & {} & {} & {} & {} \\\\\n",
            latex_escape(&entry.motif_name),
            latex_escape(&format!("{:?}", entry.alert_class_default)),
            latex_escape(&entry.requires_persistence.to_string()),
            latex_escape(&entry.requires_corroboration.to_string()),
            entry.minimum_window,
            entry.minimum_hits,
            latex_escape(&entry.suppresses_alert.to_string()),
            latex_escape(&entry.promotes_alert.to_string()),
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");
    if let Some(selected) = &cohort_summary.selected_configuration {
        out.push_str(&latex_escape(&format!(
            "Best cohort result: {} with recall {}/{}, nuisance {:.4}, numeric-only nuisance {:.4}, mean lead {}, precursor quality {}, and compression {}. Legacy one-run-tolerance cohort gate met: {}.",
            selected.cohort_name,
            selected.failure_recall,
            selected.failure_runs,
            selected.pass_run_nuisance_proxy,
            selected.numeric_pass_run_nuisance_proxy,
            format_option_f64(selected.mean_lead_time_runs),
            format_option_f64(selected.precursor_quality),
            format_option_f64(selected.compression_ratio),
            cohort_summary.any_primary_success,
        )));
        out.push_str("\n\n");
    }
    if let Some(analysis) = &cohort_summary.failure_analysis {
        out.push_str(&latex_escape(&format!(
            "Failure analysis: {}. Policy vs numeric-only DSA: {}. Nuisance-dominant motif class: {}. Useful precursor motif class: {}.",
            analysis.limiting_factor,
            analysis.policy_vs_numeric_note,
            analysis.nuisance_motif_classes,
            analysis.useful_precursor_motif_classes,
        )));
        out.push_str("\n\n");
    }
    out.push_str(&latex_escape(&format!(
        "Selected-evaluation state counts: Watch={}, Review={}, Escalate={}, Silent suppression points={}.",
        dsa.summary.watch_point_count,
        dsa.summary.review_point_count,
        dsa.summary.escalate_point_count,
        dsa.summary.silenced_point_count,
    )));
    out.push_str("\n\n");
    out
}

fn semantics_of_silence_latex_section(metrics: &BenchmarkMetrics, dsa: &DsaEvaluation) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Semantics of Silence}\n");
    out.push_str(&latex_escape(&format!(
        "Silence rule: {}. Grammar-state suppressions due to imputation: {}. Numeric-only DSA alert points: {}. Policy-governed Review/Escalate alert points: {}. Explicitly silenced points: {}. Policy nuisance is {:.4} versus numeric-only DSA {:.4} and EWMA {:.4}. Policy recall is {}/{} versus numeric-only DSA {}/{}. Watch/Review/Escalate points are {}/{}/{}. Raw boundary episodes: {}. Policy-governed DSA episodes: {}. Compression ratio: {}. Precursor quality: {}.",
        dsa.parameter_manifest.silence_rule,
        metrics.summary.grammar_imputation_suppression_points,
        dsa.summary.numeric_alert_point_count,
        dsa.summary.alert_point_count,
        dsa.summary.silenced_point_count,
        dsa.summary.pass_run_nuisance_proxy,
        dsa.summary.numeric_primary_pass_run_nuisance_proxy,
        dsa.comparison_summary.ewma.pass_run_nuisance_proxy,
        dsa.summary.failure_run_recall,
        dsa.summary.failure_runs,
        dsa.summary.numeric_primary_failure_run_recall,
        dsa.summary.failure_runs,
        dsa.summary.watch_point_count,
        dsa.summary.review_point_count,
        dsa.summary.escalate_point_count,
        dsa.episode_summary.raw_boundary_episode_count,
        dsa.episode_summary.dsa_episode_count,
        format_option_f64(dsa.episode_summary.compression_ratio),
        format_option_f64(dsa.episode_summary.precursor_quality),
    )));
    out.push_str("\n\n");
    out
}

fn predeclared_delta_target_markdown_section(assessment: &DeltaTargetAssessment) -> String {
    let mut out = String::new();
    out.push_str("## Predeclared Delta Target\n\n");
    out.push_str(&format!(
        "- Primary target: {}\n- Secondary target: {}\n- EWMA nuisance baseline: {:.6}\n- Current policy-governed DSA nuisance baseline: {:.6}\n- Primary nuisance ceiling implied by the 40% target: {:.6}\n- Secondary nuisance ceiling implied by the 40% target: {:.6}\n\n{}\n\n",
        assessment.primary_target_definition,
        assessment.secondary_target_definition,
        assessment.ewma_nuisance_baseline,
        assessment.current_policy_dsa_nuisance_baseline,
        assessment.primary_target_nuisance_ceiling,
        assessment.secondary_target_nuisance_ceiling,
        assessment.assessment_note,
    ));
    out
}

fn recall_recovery_diagnostics_markdown_section(optimization: &OptimizationExecution) -> String {
    let baseline = optimization
        .baseline_execution
        .summary
        .selected_configuration
        .as_ref();
    let optimized = optimization
        .optimized_execution
        .summary
        .selected_configuration
        .as_ref();
    let mut out = String::new();
    out.push_str("## Recall Recovery Diagnostics\n\n");
    if let Some(baseline) = baseline {
        out.push_str(&format!(
            "- Previous limiting result: {} with recall {}/{}, nuisance {:.4}, mean lead {}\n",
            baseline.cohort_name,
            baseline.failure_recall,
            baseline.failure_runs,
            baseline.pass_run_nuisance_proxy,
            format_option_f64(baseline.mean_lead_time_runs),
        ));
    }
    if let Some(optimized) = optimized {
        out.push_str(&format!(
            "- Optimized result: {} [{}] with recall {}/{}, nuisance {:.4}, mean lead {}, rescued points {}, Watch->Review rescues {}\n",
            optimized.cohort_name,
            optimized.ranking_strategy,
            optimized.failure_recall,
            optimized.failure_runs,
            optimized.pass_run_nuisance_proxy,
            format_option_f64(optimized.mean_lead_time_runs),
            optimized.rescued_point_count,
            optimized.rescued_watch_to_review_points,
        ));
    }
    out.push_str("- Rescue rules added: bounded feature-level near-miss rescue on explicit override features only; no global threshold reduction.\n");
    if let (Some(baseline), Some(optimized)) = (baseline, optimized) {
        out.push_str(&format!(
            "- Recall delta: {} -> {} (change {}). Nuisance delta: {:.4} -> {:.4}.\n\n",
            baseline.failure_recall,
            optimized.failure_recall,
            optimized.failure_recall as i64 - baseline.failure_recall as i64,
            baseline.pass_run_nuisance_proxy,
            optimized.pass_run_nuisance_proxy,
        ));
    } else {
        out.push('\n');
    }
    out
}

fn feature_aware_governance_markdown_section(optimization: &OptimizationExecution) -> String {
    let mut out = String::new();
    out.push_str("## Feature-Aware Heuristic Governance\n\n");
    out.push_str("- Motif defaults remain the global policy baseline.\n");
    out.push_str(&format!(
        "- Explicit feature overrides written: {}\n",
        optimization.feature_policy_overrides.len()
    ));
    if optimization.feature_policy_overrides.is_empty() {
        out.push_str("- No feature-specific overrides met the deterministic selection rule.\n\n");
        return out;
    }
    out.push_str("| Feature | Rescue priority | Persistence override | Corroboration override | Window | Hits | Max fragmentation | Review-without-escalate | Suppress-if-isolated |\n");
    out.push_str("|---|---:|---|---|---:|---:|---:|---|---|\n");
    for row in &optimization.feature_policy_summary {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            row.feature_name,
            row.rescue_priority,
            row.requires_persistence_override
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
            row.requires_corroboration_override
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
            row.minimum_window_override
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
            row.minimum_hits_override
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
            row.maximum_allowed_fragmentation_override
                .map(|value| format!("{value:.4}"))
                .unwrap_or_else(|| "default".into()),
            row.allow_review_without_escalate
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
            row.suppress_if_isolated
                .map(|value| value.to_string())
                .unwrap_or_else(|| "default".into()),
        ));
    }
    out.push('\n');
    for row in &optimization.feature_policy_summary {
        out.push_str(&format!(
            "- {}: {}\n",
            row.feature_name, row.override_reason
        ));
    }
    out.push('\n');
    out
}

fn missed_failure_diagnostics_markdown_section(optimization: &OptimizationExecution) -> String {
    let mut out = String::new();
    out.push_str("## Missed-Failure Diagnostics\n\n");
    if optimization.missed_failure_diagnostics.is_empty() {
        out.push_str("- No baseline-missed failures remained to diagnose.\n\n");
        return out;
    }
    for row in &optimization.missed_failure_diagnostics {
        out.push_str(&format!(
            "- Failure {}: nearest feature {:?}, score {}, policy_state {:?}, resolved_class {:?}, consistent={}, fragmentation={}, exact miss rule `{}`, recovered_after_optimization={}, bounded_rescue_would_recover={}\n",
            row.failure_run_index,
            row.nearest_feature_name,
            format_option_f64(row.nearest_feature_score),
            row.nearest_feature_policy_state,
            row.nearest_feature_resolved_alert_class,
            format_option_bool(row.nearest_feature_consistent),
            format_option_f64(row.nearest_feature_fragmentation_proxy_w),
            row.exact_miss_rule,
            row.recovered_after_optimization,
            row.bounded_rescue_would_recover,
        ));
    }
    out.push('\n');
    if !optimization.recall_critical_features.is_empty() {
        out.push_str(&format!(
            "- Recall-critical feature rows written: {} (`dsa_recall_critical_features.csv`).\n\n",
            optimization.recall_critical_features.len()
        ));
    }
    out
}

fn two_stage_optimization_frontier_markdown_section(
    optimization: &OptimizationExecution,
    assessment: &DeltaTargetAssessment,
) -> String {
    let mut out = String::new();
    out.push_str("## Two-Stage Optimization Frontier\n\n");
    out.push_str(&format!(
        "- Pareto frontier rows: {}\n- Stage A nuisance-first candidates: {}\n- Stage B recall-recovery candidates: {}\n",
        optimization.pareto_frontier.len(),
        optimization.stage_a_candidates.len(),
        optimization.stage_b_candidates.len(),
    ));
    if let Some(selected) = &optimization
        .optimized_execution
        .summary
        .selected_configuration
    {
        out.push_str(&format!(
            "- Best achieved configuration: {} [{}], W={}, K={}, tau={:.2}, m={}, recall={}/{}, nuisance {:.4}, mean lead {}, precursor quality {}, compression {}\n",
            selected.cohort_name,
            selected.ranking_strategy,
            selected.window,
            selected.persistence_runs,
            selected.alert_tau,
            selected.corroborating_m,
            selected.failure_recall,
            selected.failure_runs,
            selected.pass_run_nuisance_proxy,
            format_option_f64(selected.mean_lead_time_runs),
            format_option_f64(selected.precursor_quality),
            format_option_f64(selected.compression_ratio),
        ));
    }
    if let Some(best_stage_a) = &assessment.best_stage_a_delta_candidate {
        out.push_str(&format!(
            "- Best Stage A nuisance-collapse candidate: {} with delta_nuisance_vs_ewma {:.4}, delta_nuisance_vs_current_dsa {:.4}, recall {}/{}\n",
            best_stage_a.configuration,
            best_stage_a.delta_nuisance_vs_ewma,
            best_stage_a.delta_nuisance_vs_current_dsa,
            best_stage_a.failure_recall,
            best_stage_a.failure_runs,
        ));
    }
    if let Some(best_recall_103) = &assessment.best_recall_103_candidate {
        out.push_str(&format!(
            "- Best recall-preserving candidate (>=103/104): {} with delta_nuisance_vs_ewma {:.4}, nuisance {:.4}, precursor quality {}, compression {}\n\n",
            best_recall_103.configuration,
            best_recall_103.delta_nuisance_vs_ewma,
            best_recall_103.pass_run_nuisance_proxy,
            format_option_f64(best_recall_103.precursor_quality),
            format_option_f64(best_recall_103.compression_ratio),
        ));
    } else {
        out.push('\n');
    }
    out
}

fn target_attainment_markdown_section(assessment: &DeltaTargetAssessment) -> String {
    let mut out = String::new();
    out.push_str("## Target Attainment Assessment\n\n");
    out.push_str(&format!(
        "- Primary target reached: {}\n- Ideal target reached: {}\n- Secondary target reached: {}\n- Selected delta vs EWMA: {:.4}\n- Selected delta vs current policy DSA: {:.4}\n- Mean lead >= EWMA: {}\n- Mean lead >= threshold: {}\n\n{}\n\n",
        assessment.primary_target_met,
        assessment.ideal_target_met,
        assessment.secondary_target_met,
        assessment.selected_configuration.delta_nuisance_vs_ewma,
        assessment.selected_configuration.delta_nuisance_vs_current_dsa,
        assessment.mean_lead_time_ge_ewma,
        assessment.mean_lead_time_ge_threshold,
        assessment.assessment_note,
    ));
    out
}

fn optimization_sections_latex(
    optimization: &OptimizationExecution,
    assessment: &DeltaTargetAssessment,
) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Predeclared Delta Target}\n");
    out.push_str(&latex_escape(&format!(
        "Primary target: {}. Secondary target: {}. EWMA nuisance baseline {:.6}; current policy-governed DSA nuisance baseline {:.6}. Primary nuisance ceiling {:.6}; secondary nuisance ceiling {:.6}. {}",
        assessment.primary_target_definition,
        assessment.secondary_target_definition,
        assessment.ewma_nuisance_baseline,
        assessment.current_policy_dsa_nuisance_baseline,
        assessment.primary_target_nuisance_ceiling,
        assessment.secondary_target_nuisance_ceiling,
        assessment.assessment_note,
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Recall Recovery Diagnostics}\n");
    if let Some(baseline) = &optimization
        .baseline_execution
        .summary
        .selected_configuration
    {
        out.push_str(&latex_escape(&format!(
            "Previous limiting result: {} with recall {}/{}, nuisance {:.4}, mean lead {}.",
            baseline.cohort_name,
            baseline.failure_recall,
            baseline.failure_runs,
            baseline.pass_run_nuisance_proxy,
            format_option_f64(baseline.mean_lead_time_runs),
        )));
        out.push_str("\n\n");
    }
    if let Some(optimized_row) = &optimization
        .optimized_execution
        .summary
        .selected_configuration
    {
        out.push_str(&latex_escape(&format!(
            "Optimized result: {} [{}] with recall {}/{}, nuisance {:.4}, mean lead {}, rescued points {}, Watch-to-Review rescues {}.",
            optimized_row.cohort_name,
            optimized_row.ranking_strategy,
            optimized_row.failure_recall,
            optimized_row.failure_runs,
            optimized_row.pass_run_nuisance_proxy,
            format_option_f64(optimized_row.mean_lead_time_runs),
            optimized_row.rescued_point_count,
            optimized_row.rescued_watch_to_review_points,
        )));
        out.push_str("\n\n");
    }
    out.push_str("\\section*{Feature-Aware Heuristic Governance}\n");
    out.push_str(&latex_escape(&format!(
        "Explicit feature overrides written: {}. Recall-critical feature rows written: {}.",
        optimization.feature_policy_overrides.len(),
        optimization.recall_critical_features.len(),
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Missed-Failure Diagnostics}\n");
    out.push_str(&latex_escape(&format!(
        "Baseline missed failures diagnosed: {}.",
        optimization.missed_failure_diagnostics.len(),
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Two-Stage Optimization Frontier}\n");
    out.push_str(&latex_escape(&format!(
        "Pareto frontier rows: {}. Stage A nuisance-first candidates: {}. Stage B recall-recovery candidates: {}.",
        optimization.pareto_frontier.len(),
        optimization.stage_a_candidates.len(),
        optimization.stage_b_candidates.len(),
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Target Attainment Assessment}\n");
    out.push_str(&latex_escape(&format!(
        "Primary target reached: {}. Ideal target reached: {}. Secondary target reached: {}. Selected delta vs EWMA {:.4}; selected delta vs current policy DSA {:.4}. Mean lead >= EWMA: {}. Mean lead >= threshold: {}.",
        assessment.primary_target_met,
        assessment.ideal_target_met,
        assessment.secondary_target_met,
        assessment.selected_configuration.delta_nuisance_vs_ewma,
        assessment.selected_configuration.delta_nuisance_vs_current_dsa,
        assessment.mean_lead_time_ge_ewma,
        assessment.mean_lead_time_ge_threshold,
    )));
    out.push_str("\n\n");
    out
}

fn artifact_inventory(
    figures: &FigureManifest,
    include_cohort_failure_analysis: bool,
    include_rating_failure_analysis: bool,
) -> Vec<ArtifactInventoryEntry> {
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
            role: "Full bounded cohort DSA grid with cohort, W, K, tau, and corroboration m.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_grid_summary.json".into(),
            role: "Saved cohort-grid summary, closest-to-success row, corroboration effect, and limiting-factor analysis.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_ranking.csv".into(),
            role: "Deterministic analyzable-feature ranking used for cohort selection.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_ranking_recall_aware.csv".into(),
            role: "Recall-aware deterministic feature ranking emphasizing pre-failure coverage.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_ranking_comparison.csv".into(),
            role: "Side-by-side comparison of compression-biased and recall-aware ranking positions.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_seed_feature_check.json".into(),
            role: "Standalone seed-feature inclusion report for S059, S044, S061, S222, S354, and S173.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_cohorts.json".into(),
            role: "Explicit top_4, top_8, top_16, and all-feature cohorts plus seed-feature inclusion report.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_policy_overrides.json".into(),
            role: "Explicit feature-aware heuristic override table and rescue eligibility.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_policy_summary.csv".into(),
            role: "Feature-level policy summary with override rationale and ranking positions.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_recall_rescue_results.csv".into(),
            role: "Per-configuration rescue activation counts and recovered-alert summaries.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_recall_critical_features.csv".into(),
            role: "Per-missed-failure recall-critical feature table with closest structural candidates and bounded override recoverability.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_pareto_frontier.csv".into(),
            role: "Nuisance-versus-recall Pareto frontier across the optimized search.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_stage_a_candidates.csv".into(),
            role: "Stage A nuisance-first candidate set with recall kept at or above 100/104.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_stage_b_candidates.csv".into(),
            role: "Stage B recall-recovery candidates selected from the nuisance-first frontier.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_missed_failure_diagnostics.csv".into(),
            role: "Per-failure diagnostic table for baseline-missed failures and rescue recoverability.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_results.csv".into(),
            role: "Cohort-level DSA nuisance, recall, lead-time, episode, compression, and corroboration sweep results.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_results_recall_aware.csv".into(),
            role: "Recall-aware cohort results under the optimized deterministic rescue policy.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_summary.json".into(),
            role: "Saved cohort-level DSA summary, closest-to-success row, and best cohort when present.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_summary_recall_aware.json".into(),
            role: "Recall-aware cohort summary for direct comparison with the compression-biased ranking.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_precursor_quality.csv".into(),
            role: "Cohort-level precursor-quality table across the corroboration sweep.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_motif_policy_contributions.csv".into(),
            role: "Per-grid motif contributions to Watch/Review/Escalate and explicit silent suppression.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_policy_contribution_analysis.csv".into(),
            role: "Best-configuration contribution analysis for nuisance suppression, rescued recall, and rescue transitions.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_rating_delta_forecast.json".into(),
            role: "Bounded rating-delta forecast grounded in the saved DSA nuisance, recall, lead-time, and cohort metrics.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_delta_target_assessment.json".into(),
            role: "Explicit predeclared 40% delta-target evaluation against EWMA and the prior policy-governed DSA baseline.".into(),
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
            role: "Provenance-aware heuristic guidance and active DSA policy-engine defaults.".into(),
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

    if include_cohort_failure_analysis {
        entries.push(ArtifactInventoryEntry {
            path: "dsa_cohort_failure_analysis.md".into(),
            role: "Closest-cohort, corroboration, ranking-quality, and all-feature-vs-cohort failure analysis.".into(),
        });
        entries.push(ArtifactInventoryEntry {
            path: "dsa_heuristic_policy_failure_analysis.md".into(),
            role: "Heuristics-governed DSA policy failure analysis, including policy-vs-numeric and motif-class diagnostics.".into(),
        });
    }
    if include_rating_failure_analysis {
        entries.push(ArtifactInventoryEntry {
            path: "dsa_rating_delta_failure_analysis.md".into(),
            role: "Failure analysis for the rating-delta primary success condition.".into(),
        });
    }

    if figures.drsc.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "drsc_top_feature.csv".into(),
            role: "Aligned DRSC trace export for the selected feature window.".into(),
        });
    }
    if figures.drsc_dsa_combined.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "drsc_dsa_combined.csv".into(),
            role:
                "Aligned DRSC+DSA publication figure trace export for the selected feature window."
                    .into(),
        });
    }
    if figures.dsa_focus.is_some() {
        entries.push(ArtifactInventoryEntry {
            path: "dsa_top_feature.csv".into(),
            role: "Aligned DSA structural-focus trace export for the selected feature window."
                .into(),
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

fn feature_cohort_latex_section(
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Feature-Cohort DSA Selection}\n");
    out.push_str(&latex_escape(&format!(
        "Ranking formula: {}. Missingness penalty: {:.1} when missing_fraction > {:.2}. Legacy one-run-tolerance cohort gate used inside the bounded sweep: {}.",
        cohort_summary.ranking_formula,
        feature_cohorts.missingness_penalty_value,
        feature_cohorts.missingness_penalty_threshold,
        cohort_summary.primary_success_condition,
    )));
    out.push_str("\n\n");
    out.push_str(&latex_escape(&format!(
        "Selected cohorts: top_4={}, top_8={}, top_16={}, all_features={}.",
        feature_cohorts.top_4.len(),
        feature_cohorts.top_8.len(),
        feature_cohorts.top_16.len(),
        feature_cohorts.all_features.len(),
    )));
    out.push_str("\n\n");
    out.push_str("\\begin{longtable}{p{0.14\\linewidth}rccc}\n\\toprule\n");
    out.push_str("Seed feature & Rank & Top 4 & Top 8 & Top 16 \\\\\n\\midrule\n");
    for seed in &feature_cohorts.seed_feature_report {
        out.push_str(&format!(
            "{} & {} & {} & {} & {} \\\\\n",
            latex_escape(&seed.feature_name),
            latex_escape(
                &seed
                    .rank
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".into())
            ),
            latex_escape(&seed.in_top_4.to_string()),
            latex_escape(&seed.in_top_8.to_string()),
            latex_escape(&seed.in_top_16.to_string()),
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");
    out.push_str("\\begin{longtable}{p{0.15\\linewidth}rrrrrr}\n\\toprule\n");
    out.push_str(
        "Cohort & m & Recall & Mean lead & Nuisance & Compression & Legacy gate \\\\\n\\midrule\n",
    );
    for best in &cohort_summary.best_by_cohort {
        let result = &best.best_row;
        out.push_str(&format!(
            "{} & {} & {}/{} & {} & {:.4} & {} & {} \\\\\n",
            latex_escape(&result.cohort_name),
            result.corroborating_m,
            result.failure_recall,
            result.failure_runs,
            latex_escape(&format_option_f64(result.mean_lead_time_runs)),
            result.pass_run_nuisance_proxy,
            latex_escape(&format_option_f64(result.compression_ratio)),
            latex_escape(&result.primary_success.to_string()),
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");
    if let Some(best_row) = &cohort_summary.selected_configuration {
        out.push_str(&latex_escape(&format!(
            "Best cohort/grid result: {} with recall {}/{}, nuisance {:.4}, mean lead {}, compression {}, and precursor quality {}.",
            best_row.cohort_name,
            best_row.failure_recall,
            best_row.failure_runs,
            best_row.pass_run_nuisance_proxy,
            format_option_f64(best_row.mean_lead_time_runs),
            format_option_f64(best_row.compression_ratio),
            format_option_f64(best_row.precursor_quality),
        )));
        out.push_str("\n\n");
    }
    if let Some(failure_analysis) = &cohort_summary.failure_analysis {
        out.push_str(&latex_escape(&format!(
            "Failure analysis: closest cohort {} at {}. Limiting factor: {}. Corroboration effect: {}. Ranking quality: {}. All-feature vs cohort: {}.",
            failure_analysis.closest_cohort,
            failure_analysis.closest_grid_point,
            failure_analysis.limiting_factor,
            failure_analysis.corroboration_effect,
            failure_analysis.ranking_quality_note,
            failure_analysis.all_feature_dsa_vs_cohort_note,
        )));
        out.push_str("\n\n");
    }
    out
}

fn rating_forecast_latex_section(rating_delta_forecast: &RatingDeltaForecast) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Rating Delta Forecast}\n");
    out.push_str(&latex_escape(&format!(
        "Primary success condition: {}. Primary success met: {}. Forecast score if primary success only: {:.1}. Forecast score if primary plus secondary success: {:.1}. Achieved forecast under the measured result: {:.1}. This is a forecast, not an achieved score.",
        rating_delta_forecast.primary_success_condition,
        rating_delta_forecast.primary_success_met,
        rating_delta_forecast.forecast_score_if_primary_success_only,
        rating_delta_forecast.forecast_score_if_primary_plus_secondary_success,
        rating_delta_forecast.achieved_forecast_score,
    )));
    out.push_str("\n\n");
    out.push_str(&latex_escape(&rating_delta_forecast.forecast_justification));
    out.push_str("\n\n");
    out
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
        latex_escape(&format_option_f64(
            metric.pre_failure_window_precision_proxy
        )),
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

    (
        None,
        (!combined_output.trim().is_empty()).then_some(combined_output),
    )
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

fn format_option_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
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
