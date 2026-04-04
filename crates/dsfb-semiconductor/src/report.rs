use crate::cohort::{
    cohort_report_section, rating_forecast_report_section, CohortDsaSummary, DeltaTargetAssessment,
    FeatureCohorts, OptimizationExecution, RatingDeltaForecast,
};
use crate::config::PipelineConfig;
use crate::dataset::phm2018::Phm2018SupportStatus;
use crate::dataset::secom::SecomArchiveLayout;
use crate::error::Result;
use crate::failure_driven::FailureDrivenArtifacts;
use crate::heuristics::HeuristicEntry;
use crate::metrics::{BenchmarkMetrics, MotifMetric};
use crate::plots::FigureManifest;
use crate::precursor::DsaEvaluation;
use crate::secom_addendum::SecomAddendumArtifacts;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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
    failure_driven: &FailureDrivenArtifacts,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
    secom_addendum: &SecomAddendumArtifacts,
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
            failure_driven,
            feature_cohorts,
            cohort_summary,
            rating_delta_forecast,
            secom_addendum,
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
            failure_driven,
            feature_cohorts,
            cohort_summary,
            rating_delta_forecast,
            secom_addendum,
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
    failure_driven: &FailureDrivenArtifacts,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
    secom_addendum: &SecomAddendumArtifacts,
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
    out.push_str(&executive_summary_markdown_section(
        optimization,
        secom_addendum,
    ));

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
    out.push_str(&non_intrusive_integration_markdown_section());
    out.push_str(&true_dsfb_structural_semiotics_markdown_section());
    out.push_str(&grouped_coordinated_semiotics_markdown_section());
    out.push_str(&missed_failure_analysis_markdown_section(failure_driven));
    out.push_str(&failure_priority_markdown_section(failure_driven));
    out.push_str(&feature_motif_grounding_markdown_section(failure_driven));
    out.push_str(&feature_role_validation_markdown_section(failure_driven));
    out.push_str(&minimal_heuristics_markdown_section(failure_driven));
    out.push_str(&heuristic_provenance_markdown_section(failure_driven));
    out.push_str(&group_validation_markdown_section(failure_driven));
    out.push_str(&negative_control_markdown_section(failure_driven));
    out.push_str(&dsfb_vs_ewma_markdown_section(failure_driven));
    out.push_str(&secom_limitation_markdown_section(secom_addendum));
    out.push_str(&metric_regrounding_markdown_section(secom_addendum));
    out.push_str(&target_d_regression_markdown_section(secom_addendum));
    out.push_str(&lead_time_explanation_markdown_section(secom_addendum));
    out.push_str(&which_delta_matters_markdown_section(optimization));
    out.push_str(&predeclared_operator_delta_targets_markdown_section(
        optimization,
    ));
    out.push_str(&operator_optimization_frontier_markdown_section(
        optimization,
    ));
    out.push_str(&recall_recovery_efficiency_markdown_section(optimization));
    out.push_str(&operator_target_attainment_markdown_section(optimization));
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
    out.push_str(&claims_intentionally_not_made_markdown_section());

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
    failure_driven: &FailureDrivenArtifacts,
    feature_cohorts: &FeatureCohorts,
    cohort_summary: &CohortDsaSummary,
    rating_delta_forecast: &RatingDeltaForecast,
    secom_addendum: &SecomAddendumArtifacts,
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
    out.push_str(&executive_summary_latex_section(
        optimization,
        secom_addendum,
    ));

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
    out.push_str(&non_intrusive_integration_latex_section());
    out.push_str(&true_dsfb_structural_semiotics_latex_section());
    out.push_str(&grouped_coordinated_semiotics_latex_section());
    out.push_str(&missed_failure_analysis_latex_section(failure_driven));
    out.push_str(&failure_priority_latex_section(failure_driven));
    out.push_str(&feature_motif_grounding_latex_section(failure_driven));
    out.push_str(&feature_role_validation_latex_section(failure_driven));
    out.push_str(&minimal_heuristics_latex_section(failure_driven));
    out.push_str(&heuristic_provenance_latex_section(failure_driven));
    out.push_str(&group_validation_latex_section(failure_driven));
    out.push_str(&negative_control_latex_section(failure_driven));
    out.push_str(&dsfb_vs_ewma_latex_section(failure_driven));
    out.push_str(&secom_limitation_latex_section(secom_addendum));
    out.push_str(&metric_regrounding_latex_section(secom_addendum));
    out.push_str(&target_d_regression_latex_section(secom_addendum));
    out.push_str(&lead_time_explanation_latex_section(secom_addendum));
    out.push_str(&operator_sections_latex(optimization));
    out.push_str(&optimization_sections_latex(
        optimization,
        delta_target_assessment,
    ));
    out.push_str(&rating_forecast_latex_section(rating_delta_forecast));
    out.push_str(&claims_intentionally_not_made_latex_section());

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

fn executive_summary_markdown_section(
    optimization: &OptimizationExecution,
    secom_addendum: &SecomAddendumArtifacts,
) -> String {
    let mut out = String::new();
    out.push_str("## Executive Summary\n\n");
    out.push_str("- DSFB remains a non-intrusive, read-only, deterministic companion layer over existing SPC/EWMA/controller residuals; it does not modify thresholds, actuation, timing, or certification boundaries.\n");
    out.push_str(&format!(
        "- {}\n- Required SECOM limitation statement supported by data: {}\n- Paper abstract artifact: `paper_abstract_artifact.txt`\n",
        secom_addendum.executive_summary_text,
        secom_addendum.required_tradeoff_statement_supported,
    ));
    out.push_str(&format!(
        "- Exact operator deltas in the selected SECOM row: investigation load {:.1}%, episode count {:.1}%, review points/pass-run {:.1}%, review episodes/pass-run {:.1}%, recall {}/{}, precursor quality {}, nuisance vs EWMA {:.1}%\n\n",
        optimization.operator_delta_targets.delta_investigation_load * 100.0,
        optimization.operator_delta_targets.delta_episode_count * 100.0,
        optimization.operator_delta_targets.delta_review_points_per_pass_run * 100.0,
        optimization.operator_delta_targets.delta_review_episodes_per_pass_run * 100.0,
        optimization.operator_delta_targets.selected_configuration.failure_recall,
        optimization.operator_delta_targets.selected_configuration.failure_runs,
        format_option_f64(
            optimization
                .operator_delta_targets
                .selected_configuration
                .precursor_quality,
        ),
        optimization.operator_delta_targets.delta_nuisance_vs_ewma * 100.0,
    ));
    out
}

fn secom_limitation_markdown_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let stats = &secom_addendum.recurrent_boundary_stats;
    let mut out = String::new();
    out.push_str("## SECOM Structural Limitation\n\n");
    out.push_str(&format!(
        "- recurrent_boundary_approach points: {}\n- Runs hit by recurrent_boundary_approach: {}\n- Pre-failure runs hit: {}\n- Pass runs hit: {}\n- Pre-failure precision: {:.4}\n- Pass-run precision: {:.4}\n\n",
        stats.total_boundary_points,
        stats.total_run_hits,
        stats.total_pre_failure_hits,
        stats.pass_run_hits,
        stats.precision_pre_failure,
        stats.precision_pass,
    ));
    out.push_str(&format!(
        "{} Supported by tradeoff sweep: {}.\n\n",
        secom_addendum.required_tradeoff_statement,
        secom_addendum.required_tradeoff_statement_supported,
    ));
    out.push_str("- Tradeoff artifacts: `recurrent_boundary_stats.json`, `recurrent_boundary_tradeoff_curve.csv`, and `recurrent_boundary_tradeoff_plot.png`\n\n");
    out
}

fn metric_regrounding_markdown_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Metric Re-Grounding\n\n");
    out.push_str("Every delta below is baseline-specific; no 40% claim is made without naming the comparator.\n\n");
    out.push_str("| Metric | Baseline | DSFB value | Baseline value | Delta % |\n");
    out.push_str("|---|---|---:|---:|---:|\n");
    for row in &secom_addendum.metric_regrounding {
        out.push_str(&format!(
            "| {} | {} | {:.4} | {:.4} | {:.2}% |\n",
            row.metric,
            row.baseline,
            row.dsfb_value,
            row.baseline_value,
            row.delta_percent * 100.0,
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "- Episode precision is promoted as the primary operator metric: {:.1}% vs a raw-boundary precision proxy of {:.2}%, a {:.1}x gain.\n\n",
        secom_addendum.episode_precision_metrics.dsfb_precision * 100.0,
        secom_addendum.episode_precision_metrics.raw_alarm_precision * 100.0,
        secom_addendum.episode_precision_metrics.precision_gain_factor,
    ));
    out
}

fn target_d_regression_markdown_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let analysis = &secom_addendum.target_d_regression_analysis;
    let mut out = String::new();
    out.push_str("## Target D Regression Analysis\n\n");
    out.push_str(&format!(
        "- Contributing features: {}\n- Contributing motifs: {}\n- Contributing heuristics: {}\n- Contributing policy rules: {}\n- Action taken: {}\n\n",
        join_or_none(&analysis.contributing_features),
        join_or_none(&analysis.contributing_motifs),
        join_or_none(&analysis.contributing_heuristics),
        join_or_none(&analysis.contributing_policy_rules),
        analysis.action_taken,
    ));
    out.push_str(&format!(
        "{}\n\n{}\n\n",
        analysis.why_regression_occurred, analysis.tradeoff_justification
    ));
    out.push_str("Causal chain:\n");
    for item in &analysis.causal_chain {
        out.push_str(&format!("- {}\n", item));
    }
    out.push('\n');
    out
}

fn lead_time_explanation_markdown_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let explanation = &secom_addendum.lead_time_explanation;
    let mut out = String::new();
    out.push_str("## Lead-Time Deficit Explanation\n\n");
    out.push_str(&format!(
        "- Mean DSA lead: {}\n- Mean threshold lead: {}\n- Mean earliest semantic-match lead: {}\n- Failures where threshold leads DSA: {}\n- Failures where DSA leads threshold: {}\n- Failures where semantic matches precede threshold: {}\n- Failures where motif emergence precedes threshold: {}\n\n{}\n\n{}\n\n",
        format_option_f64(explanation.mean_dsfb_lead_runs),
        format_option_f64(explanation.mean_threshold_lead_runs),
        format_option_f64(explanation.mean_semantic_match_lead_runs),
        explanation.threshold_earlier_failure_count,
        explanation.dsfb_earlier_failure_count,
        explanation.semantic_match_precedes_threshold_count,
        explanation.motif_emergence_precedes_threshold_count,
        explanation.explanation,
        explanation.validation_note,
    ));
    out.push_str(
        "- Saved artifacts: `lead_time_comparison.csv` and `lead_time_explanation.json`\n\n",
    );
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

fn non_intrusive_integration_markdown_section() -> String {
    let mut out = String::new();
    out.push_str("## Non-Intrusive Integration Model\n\n");
    out.push_str("- Integration mode: `read_only_side_channel`\n");
    out.push_str(
        "- Fixed layer order: `Residual -> Sign -> Syntax -> Grammar -> Semantics -> Policy`\n",
    );
    out.push_str("- Inputs consumed by DSFB: immutable residual streams, upstream alarm streams, and metadata only.\n");
    out.push_str("- Outputs emitted by DSFB: advisory interpretations only; no controller, threshold, recipe, or actuation API exists.\n");
    out.push_str(
        "- No feedback path exists from DSFB outputs back into SPC/EWMA/controller logic.\n",
    );
    out.push_str("- No upstream latency claim is made; the contract is that DSFB runs observer-style and must not change primary control timing.\n");
    out.push_str("- Replay is deterministic and fail-safe isolated: identical ordered inputs yield identical outputs, and DSFB failure leaves upstream behavior unchanged.\n");
    out.push_str("- Contract artifacts: `non_intrusive_interface_spec.md`, `figures/dsfb_non_intrusive_architecture.png`, and `figures/dsfb_non_intrusive_architecture.svg`\n\n");
    out
}

fn claims_intentionally_not_made_markdown_section() -> String {
    let mut out = String::new();
    out.push_str("## Claims intentionally not made\n\n");
    out.push_str(
        "- DSFB does not replace SPC, EWMA, threshold logic, APC, or controller actuation.\n",
    );
    out.push_str("- SECOM does not prove universal early-warning superiority.\n");
    out.push_str("- PHM 2018 does not prove burden reduction because PHM burden metrics are not computed here.\n");
    out.push_str("- No universal superiority claim is made against scalar baselines.\n");
    out.push_str(
        "- No SEMI compliance, completed qualification, or deployment readiness claim is made.\n\n",
    );
    out
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

fn executive_summary_latex_section(
    optimization: &OptimizationExecution,
    secom_addendum: &SecomAddendumArtifacts,
) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Executive summary}\n");
    out.push_str(&latex_escape(&format!(
        "DSFB remains a non-intrusive, read-only, deterministic companion layer over existing SPC/EWMA/controller residuals. {} Required SECOM limitation statement supported by data: {}. Exact operator deltas in the selected row are investigation load {:.1}%, episode count {:.1}%, review points/pass-run {:.1}%, review episodes/pass-run {:.1}%, recall {}/{}, precursor quality {}, and nuisance vs EWMA {:.1}%.",
        secom_addendum.executive_summary_text,
        secom_addendum.required_tradeoff_statement_supported,
        optimization.operator_delta_targets.delta_investigation_load * 100.0,
        optimization.operator_delta_targets.delta_episode_count * 100.0,
        optimization.operator_delta_targets.delta_review_points_per_pass_run * 100.0,
        optimization.operator_delta_targets.delta_review_episodes_per_pass_run * 100.0,
        optimization.operator_delta_targets.selected_configuration.failure_recall,
        optimization.operator_delta_targets.selected_configuration.failure_runs,
        format_option_f64(
            optimization
                .operator_delta_targets
                .selected_configuration
                .precursor_quality,
        ),
        optimization.operator_delta_targets.delta_nuisance_vs_ewma * 100.0,
    )));
    out.push_str("\n\n");
    out
}

fn non_intrusive_integration_latex_section() -> String {
    let mut out = String::new();
    out.push_str("\\section*{Non-Intrusive Integration Model}\n");
    out.push_str("Integration mode: \\texttt{read\\_only\\_side\\_channel}. Fixed layer order: \\texttt{Residual -> Sign -> Syntax -> Grammar -> Semantics -> Policy}. Inputs consumed by DSFB are immutable residual streams, upstream alarm streams, and metadata only. Outputs are advisory interpretations only; no controller, threshold, recipe, or actuation API exists. No feedback path exists from DSFB outputs back into SPC/EWMA/controller logic. No upstream latency claim is made; the contract is that DSFB runs observer-style and must not change primary control timing. Replay is deterministic and fail-safe isolated: identical ordered inputs yield identical outputs, and DSFB failure leaves upstream behavior unchanged. Contract artifacts: \\texttt{non\\_intrusive\\_interface\\_spec.md}, \\texttt{figures/dsfb\\_non\\_intrusive\\_architecture.png}, and \\texttt{figures/dsfb\\_non\\_intrusive\\_architecture.svg}.\n\n");
    out
}

fn claims_intentionally_not_made_latex_section() -> String {
    let mut out = String::new();
    out.push_str("\\section*{Claims intentionally not made}\n");
    out.push_str("\\begin{itemize}\n");
    out.push_str(
        "\\item DSFB does not replace SPC, EWMA, threshold logic, APC, or controller actuation.\n",
    );
    out.push_str("\\item SECOM does not prove universal early-warning superiority.\n");
    out.push_str("\\item PHM 2018 does not prove burden reduction because PHM burden metrics are not computed here.\n");
    out.push_str("\\item No universal superiority claim is made against scalar baselines.\n");
    out.push_str("\\item No SEMI compliance, completed qualification, or deployment readiness claim is made.\n");
    out.push_str("\\end{itemize}\n\n");
    out
}

fn secom_limitation_latex_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let stats = &secom_addendum.recurrent_boundary_stats;
    let mut out = String::new();
    out.push_str("\\section*{SECOM structural limitation}\n");
    out.push_str(&latex_escape(&format!(
        "recurrent_boundary_approach points: {}. Runs hit: {}. Pre-failure runs hit: {}. Pass runs hit: {}. Pre-failure precision: {:.4}. Pass-run precision: {:.4}. {} Supported by tradeoff sweep: {}.",
        stats.total_boundary_points,
        stats.total_run_hits,
        stats.total_pre_failure_hits,
        stats.pass_run_hits,
        stats.precision_pre_failure,
        stats.precision_pass,
        secom_addendum.required_tradeoff_statement,
        secom_addendum.required_tradeoff_statement_supported,
    )));
    out.push_str("\n\n");
    out
}

fn metric_regrounding_latex_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Metric re-grounding}\n");
    out.push_str("\\begin{longtable}{p{0.20\\linewidth}p{0.14\\linewidth}rrr}\n\\toprule\n");
    out.push_str("Metric & Baseline & DSFB value & Baseline value & Delta \\% \\\\\n\\midrule\n");
    for row in &secom_addendum.metric_regrounding {
        out.push_str(&format!(
            "{} & {} & {:.4} & {:.4} & {:.2} \\\\\n",
            latex_escape(&row.metric),
            latex_escape(&row.baseline),
            row.dsfb_value,
            row.baseline_value,
            row.delta_percent * 100.0,
        ));
    }
    out.push_str("\\bottomrule\n\\end{longtable}\n\n");
    out.push_str(&latex_escape(&format!(
        "Episode precision is promoted as the primary operator metric: {:.1}\\% versus a raw-boundary precision proxy of {:.2}\\%, a {:.1}x gain.",
        secom_addendum.episode_precision_metrics.dsfb_precision * 100.0,
        secom_addendum.episode_precision_metrics.raw_alarm_precision * 100.0,
        secom_addendum.episode_precision_metrics.precision_gain_factor,
    )));
    out.push_str("\n\n");
    out
}

fn target_d_regression_latex_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let analysis = &secom_addendum.target_d_regression_analysis;
    let mut out = String::new();
    out.push_str("\\section*{Target D regression analysis}\n");
    out.push_str(&latex_escape(&format!(
        "Contributing features: {}. Contributing motifs: {}. Contributing heuristics: {}. Contributing policy rules: {}. Action taken: {}. {} {}",
        join_or_none(&analysis.contributing_features),
        join_or_none(&analysis.contributing_motifs),
        join_or_none(&analysis.contributing_heuristics),
        join_or_none(&analysis.contributing_policy_rules),
        analysis.action_taken,
        analysis.why_regression_occurred,
        analysis.tradeoff_justification,
    )));
    out.push_str("\n\n");
    out
}

fn lead_time_explanation_latex_section(secom_addendum: &SecomAddendumArtifacts) -> String {
    let explanation = &secom_addendum.lead_time_explanation;
    let mut out = String::new();
    out.push_str("\\section*{Lead-time deficit explanation}\n");
    out.push_str(&latex_escape(&format!(
        "Mean DSA lead: {}. Mean threshold lead: {}. Mean earliest semantic-match lead: {}. Failures where threshold leads DSA: {}. Failures where DSA leads threshold: {}. Failures where semantic matches precede threshold: {}. Failures where motif emergence precedes threshold: {}. {} {}",
        format_option_f64(explanation.mean_dsfb_lead_runs),
        format_option_f64(explanation.mean_threshold_lead_runs),
        format_option_f64(explanation.mean_semantic_match_lead_runs),
        explanation.threshold_earlier_failure_count,
        explanation.dsfb_earlier_failure_count,
        explanation.semantic_match_precedes_threshold_count,
        explanation.motif_emergence_precedes_threshold_count,
        explanation.explanation,
        explanation.validation_note,
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

fn which_delta_matters_markdown_section(optimization: &OptimizationExecution) -> String {
    let targets = &optimization.operator_delta_targets;
    let mut out = String::new();
    out.push_str("## Which Delta Matters on SECOM\n\n");
    out.push_str("On the current SECOM evidence, the operator-facing delta is investigation burden on structurally active pass windows, not binary run-level nuisance alone. This crate therefore evaluates Review/Escalate burden, episode fragmentation, precursor quality, and recall-recovery efficiency before lead-time claims.\n\n");
    out.push_str(&format!(
        "- Baseline structural investigation points: {}\n- Optimized Review/Escalate points: {}\n- Baseline episode count: {}\n- Optimized episode count: {}\n- Baseline review burden per pass run: {:.4}\n- Optimized review burden per pass run: {:.4}\n\n",
        optimization.operator_baselines.baseline_investigation_points,
        targets.optimized_review_escalate_points,
        optimization.operator_baselines.baseline_episode_count,
        targets.optimized_episode_count,
        optimization
            .operator_baselines
            .baseline_review_escalate_points_per_pass_run,
        targets.optimized_review_points_per_pass_run,
    ));
    out
}

fn true_dsfb_structural_semiotics_markdown_section() -> String {
    let mut out = String::new();
    out.push_str("## True DSFB Structural Semiotics Instantiation\n\n");
    out.push_str("This pass preserves the DSFB Structural Semiotics Engine as an explicit layered architecture: `Residual -> Sign -> Syntax -> Grammar -> Semantics -> Policy`.\n\n");
    out.push_str(
        "- Residual: deterministic discrepancy from the healthy-window nominal reference.\n",
    );
    out.push_str("- Sign: per-run tuples `sigma_i(t) = (r_i(t), d_i(t), s_i(t))`, saved in `dsfb_signs.csv` and `dsfb_feature_signs.csv`.\n");
    out.push_str("- Syntax: deterministic temporal motifs over sign trajectories, saved in `dsfb_motifs.csv`, `dsfb_motif_labels_per_time.csv`, and `dsfb_feature_motif_timeline.csv`.\n");
    out.push_str("- Grammar: admissibility-envelope states, saved in `dsfb_grammar_states.csv`, `dsfb_feature_grammar_states.csv`, and `dsfb_envelope_interaction_summary.csv`.\n");
    out.push_str("- Semantics: grammar-qualified heuristic retrieval only after syntax and grammar, saved in `dsfb_semantic_matches.csv` and `dsfb_semantic_ranked_candidates.csv`.\n");
    out.push_str("- Policy: operator burden control and bounded recall rescue using semantic outputs plus grammar-qualified persistence, with decisions logged in `dsfb_feature_policy_decisions.csv`.\n\n");
    out.push_str("No semantic label in this crate is assigned directly from raw feature magnitude, EWMA, or threshold behavior alone.\n\n");
    out
}

fn grouped_coordinated_semiotics_markdown_section() -> String {
    let mut out = String::new();
    out.push_str("## Grouped / Coordinated Semiotics\n\n");
    out.push_str("Candidate grouped-semiotics structures are logged in `dsfb_group_definitions.json`, with grouped signs, grammar states, and semantic matches emitted only for groups that survive strict validation.\n\n");
    out.push_str("- Candidate Group A: `S059`, `S133`\n");
    out.push_str("- Candidate Group B: `S123`, `S540`, `S128`\n");
    out.push_str("- Candidate Group C: `S104`\n\n");
    out.push_str("A group is accepted only when feature co-activation is visible in failure windows and absent in pass runs; otherwise grouped semiotics is explicitly rejected rather than assumed.\n\n");
    out
}

fn missed_failure_analysis_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Missed Failure Analysis\n\n");
    out.push_str(&format!(
        "- Baseline missed failures indexed: {}\n",
        failure_driven.failures_index.missed_failure_ids.len()
    ));
    for case in &failure_driven.failure_cases {
        out.push_str(&format!(
            "- Failure {}: stage=`{}`, exact_miss_rule=`{}`, optimized_detected=`{}`, artifact=`failure_case_{}.json`\n",
            case.failure_id,
            case.failure_stage,
            case.exact_miss_rule,
            case.optimized_detected_by_dsa,
            case.failure_id,
        ));
    }
    out.push('\n');
    out
}

fn failure_priority_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Failure Priority Ranking\n\n");
    if failure_driven.missed_failure_priority.is_empty() {
        out.push_str("- No missed-failure priority rows were emitted.\n\n");
        return out;
    }
    for row in failure_driven.missed_failure_priority.iter().take(8) {
        out.push_str(&format!(
            "- Failure {}: priority_score={:.4}, top_feature={:?}, signal_strength={:.4}, feature_concentration={:.4}, separation_from_noise={:.4}, recoverability_estimate={:.4}, exact_miss_rule=`{}`\n",
            row.failure_id,
            row.priority_score,
            row.top_feature_name,
            row.signal_strength,
            row.feature_concentration,
            row.separation_from_noise,
            row.recoverability_estimate,
            row.exact_miss_rule,
        ));
    }
    out.push('\n');
    out
}

fn feature_motif_grounding_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Feature -> Motif Grounding\n\n");
    for row in failure_driven.feature_motif_grounding.iter().take(10) {
        out.push_str(&format!(
            "- {}: motif=`{}`, dominant_dsfb_motif=`{}`, failure_semantic_hits={}, pass_semantic_hits={}\n",
            row.feature_name,
            row.motif_type,
            row.dominant_dsfb_motif,
            row.failure_window_semantic_hits,
            row.pass_run_semantic_hits,
        ));
    }
    out.push('\n');
    out
}

fn feature_role_validation_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Feature Role Validation\n\n");
    for row in &failure_driven.feature_role_validation {
        out.push_str(&format!(
            "- {}: initial_role=`{}`, initial_motif=`{}`, validation=`{}`, final_role=`{}`, final_motif=`{}`\n",
            row.feature_id,
            row.initial_role,
            row.initial_motif,
            row.supported_or_revised_or_rejected,
            row.final_role,
            row.final_motif,
        ));
    }
    out.push('\n');
    out
}

fn minimal_heuristics_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Heuristics With Justification\n\n");
    out.push_str(&format!(
        "- Minimal heuristic count: {}\n",
        failure_driven.minimal_heuristics_bank.len()
    ));
    for row in &failure_driven.minimal_heuristics_bank {
        out.push_str(&format!(
            "- {}: target=`{}` / `{}`, status=`{}`, action={}\n",
            row.heuristic_id,
            row.target_problem_type,
            row.target_identifier,
            row.status,
            row.policy_action,
        ));
    }
    out.push('\n');
    out
}

fn heuristic_provenance_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Heuristic Provenance\n\n");
    if failure_driven.heuristic_provenance.is_empty() {
        out.push_str("- No heuristic provenance rows were emitted.\n\n");
        return out;
    }
    for row in &failure_driven.heuristic_provenance {
        out.push_str(&format!(
            "- {}: failures=`{}`, features=`{}`, intended_effect=`{}`, nuisance_class=`{}`, motif_signature=`{}`, grammar_states=`{}`\n",
            row.heuristic_id,
            row.derived_from_failures,
            row.uses_features,
            row.intended_effect,
            row.targets_nuisance_class,
            row.motif_signature,
            row.allowed_grammar_states,
        ));
    }
    out.push('\n');
    out
}

fn group_validation_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## Grouped Semiotics Validation\n\n");
    if failure_driven.group_validation.is_empty() {
        out.push_str("- No group-validation rows were emitted.\n\n");
        return out;
    }
    for row in &failure_driven.group_validation {
        out.push_str(&format!(
            "- {} [{}]: failure_coactivation={}, pass_coactivation={}, decision=`{}`, reason={}\n",
            row.group_id,
            row.group_members,
            row.failure_coactivation_count,
            row.pass_coactivation_count,
            row.retained_or_rejected,
            row.reason,
        ));
    }
    out.push('\n');
    out
}

fn negative_control_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let report = &failure_driven.negative_control_report;
    let mut out = String::new();
    out.push_str("## Negative Control\n\n");
    out.push_str(&format!(
        "- Pass runs: false_activation_rate={:.4}, false_episode_rate={:.4}, review_escalate_points={}, review_escalate_episodes={} ({}/{}, {}/{})\n- Clean windows: false_activation_rate={:.4}, false_episode_rate={:.4}, review_escalate_points={}, review_escalate_episodes={} ({}/{}, {}/{})\n\n",
        report.false_activation_rate,
        report.false_episode_rate,
        report.review_escalate_points_on_pass_runs,
        report.review_escalate_episodes_on_pass_runs,
        report.pass_run_false_activation_count,
        report.pass_run_count,
        report.pass_run_false_episode_count,
        report.pass_run_count,
        report.clean_window_false_activation_rate,
        report.clean_window_false_episode_rate,
        report.review_escalate_points_on_clean_runs,
        report.review_escalate_episodes_on_clean_runs,
        report.clean_window_false_activation_count,
        report.clean_window_count,
        report.clean_window_false_episode_count,
        report.clean_window_count,
    ));
    out
}

fn dsfb_vs_ewma_markdown_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("## DSFB vs EWMA Separation\n\n");
    if failure_driven.dsfb_vs_ewma_cases.is_empty() {
        out.push_str(
            "- No recovered failure produced a DSFB-vs-EWMA case artifact in this run.\n\n",
        );
        return out;
    }
    for case in &failure_driven.dsfb_vs_ewma_cases {
        out.push_str(&format!(
            "- Failure {} recovered on feature `{}`; artifact=`dsfb_vs_ewma_case_{}.json`; EWMA_detected=`{}`.\n",
            case.failure_id,
            case.recovered_feature_name,
            case.failure_id,
            case.ewma_detected,
        ));
    }
    out.push('\n');
    out
}

fn predeclared_operator_delta_targets_markdown_section(
    optimization: &OptimizationExecution,
) -> String {
    let targets = &optimization.operator_delta_targets;
    let mut out = String::new();
    out.push_str("## Predeclared Operator Delta Targets\n\n");
    out.push_str("- Target A: `delta_investigation_load >= 0.40`\n");
    out.push_str("- Target B: `delta_episode_count >= 0.40`\n");
    out.push_str("- Target C: `delta_review_points_per_pass_run >= 0.40`\n");
    out.push_str("- Target D: `delta_review_episodes_per_pass_run >= 0.40`\n");
    out.push_str("- Target E: precursor quality preserved or improved\n");
    out.push_str("- Target F: recall `>= 103/104`\n");
    out.push_str("- Target G: recall `= 104/104`\n");
    out.push_str("- Target H: nuisance delta thresholds `>= 0.15`, `>= 0.25`, `>= 0.40`\n");
    out.push_str(&format!(
        "\nBaseline layers used:\n- Investigation load baseline: `{}` ({})\n- Episode baseline: `{}` ({})\n- Review-burden baseline: `{}` ({:.4} points/pass-run)\n\n",
        optimization.operator_baselines.investigation_baseline_layer,
        optimization.operator_baselines.baseline_investigation_points,
        optimization.operator_baselines.episode_baseline_layer,
        optimization.operator_baselines.baseline_episode_count,
        optimization.operator_baselines.review_burden_baseline_layer,
        optimization
            .operator_baselines
            .baseline_review_escalate_points_per_pass_run,
    ));
    out.push_str(&format!(
        "Selected-row operator deltas:\n- delta_investigation_load = {:.4}\n- delta_episode_count = {:.4}\n- delta_review_points_per_pass_run = {:.4}\n- delta_review_episodes_per_pass_run = {:.4}\n- precursor quality status = {}\n- recall equals threshold = {}\n- recall within tolerance = {}\n\n",
        targets.delta_investigation_load,
        targets.delta_episode_count,
        targets.delta_review_points_per_pass_run,
        targets.delta_review_episodes_per_pass_run,
        targets.precursor_quality_status,
        targets.recall_equals_threshold,
        targets.recall_within_tolerance,
    ));
    out
}

fn operator_optimization_frontier_markdown_section(optimization: &OptimizationExecution) -> String {
    let mut out = String::new();
    out.push_str("## Optimization Frontier\n\n");
    out.push_str(&format!(
        "- Pareto frontier rows: {}\n- Stage 1 burden-reduction candidates: {}\n- Stage 2 recall-recovery candidates: {}\n- Single-change iterations logged: {} (accepted: {})\n",
        optimization.pareto_frontier.len(),
        optimization.stage1_candidates.len(),
        optimization.stage2_candidates.len(),
        optimization.single_change_iteration_log.len(),
        optimization
            .single_change_iteration_log
            .iter()
            .filter(|row| row.accepted)
            .count(),
    ));
    if let Some(selected) = &optimization
        .optimized_execution
        .summary
        .selected_configuration
    {
        out.push_str(&format!(
            "- Best optimized configuration: {} [{}], W={}, K={}, tau={:.2}, m={}, recall={}/{}, Review/Escalate points={}, episodes={}, precursor quality={}, nuisance={:.4}\n\n",
            selected.cohort_name,
            selected.ranking_strategy,
            selected.window,
            selected.persistence_runs,
            selected.alert_tau,
            selected.corroborating_m,
            selected.failure_recall,
            selected.failure_runs,
            selected.investigation_point_count,
            selected.dsa_episode_count,
            format_option_f64(selected.precursor_quality),
            selected.pass_run_nuisance_proxy,
        ));
    } else {
        out.push('\n');
    }
    out
}

fn recall_recovery_efficiency_markdown_section(optimization: &OptimizationExecution) -> String {
    let mut out = String::new();
    out.push_str("## Recall Recovery Efficiency\n\n");
    if optimization.recall_recovery_efficiency.is_empty() {
        out.push_str("- No recall-recovery efficiency rows were emitted.\n\n");
        return out;
    }
    for row in &optimization.recall_recovery_efficiency {
        out.push_str(&format!(
            "- {} -> {}: recovered_failures={}, added_review_escalate_points={}, added_episode_count={}, added_review_points_per_pass_run={:.4}, added_review_episodes_per_pass_run={:.4}, added_nuisance_runs={}, recovered_failures_per_added_review_escalate_point={}, recovered_failures_per_added_episode={}, recovered_failures_per_added_pass_run_burden={}, recovered_failures_per_added_nuisance_run={}\n",
            row.baseline_configuration,
            row.optimized_configuration,
            row.recovered_failures,
            row.added_review_escalate_points,
            row.added_episode_count,
            row.added_review_points_per_pass_run,
            row.added_review_episodes_per_pass_run,
            row.added_nuisance_runs,
            format_option_f64(row.recovered_failures_per_added_review_escalate_point),
            format_option_f64(row.recovered_failures_per_added_episode),
            format_option_f64(row.recovered_failures_per_added_pass_run_burden),
            format_option_f64(row.recovered_failures_per_added_nuisance_run),
        ));
    }
    out.push('\n');
    out
}

fn operator_target_attainment_markdown_section(optimization: &OptimizationExecution) -> String {
    let targets = &optimization.operator_delta_targets;
    let mut out = String::new();
    out.push_str("## Target Attainment Assessment\n\n");
    out.push_str(&format!(
        "- Target A (`delta_investigation_load >= 0.40`): {}\n- Target B (`delta_episode_count >= 0.40`): {}\n- Target C (`delta_review_points_per_pass_run >= 0.40`): {}\n- Target D (`delta_review_episodes_per_pass_run >= 0.40`): {}\n- Target E (precursor quality preserved or improved): {}\n- Target F (`recall >= 103/104`): {}\n- Target G (`recall = 104/104`): {}\n- Target H nuisance thresholds: >=0.15={}, >=0.25={}, >=0.40={}\n- Mean lead >= EWMA: {}\n- Mean lead >= threshold: {}\n\n",
        targets.delta_investigation_load >= 0.40,
        targets.delta_episode_count >= 0.40,
        targets.delta_review_points_per_pass_run >= 0.40,
        targets.delta_review_episodes_per_pass_run >= 0.40,
        targets.precursor_quality_status != "degraded",
        targets.recall_ge_103,
        targets.recall_eq_104,
        targets.delta_nuisance_vs_ewma >= 0.15,
        targets.delta_nuisance_vs_ewma >= 0.25,
        targets.delta_nuisance_vs_ewma >= 0.40,
        targets.mean_lead_delta_vs_ewma.unwrap_or(f64::NEG_INFINITY) >= 0.0,
        targets.mean_lead_delta_vs_threshold.unwrap_or(f64::NEG_INFINITY) >= 0.0,
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
    out.push_str("## Legacy Nuisance Target Assessment\n\n");
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
    out.push_str("\\section*{Legacy Nuisance Target Assessment}\n");
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

fn true_dsfb_structural_semiotics_latex_section() -> String {
    let mut out = String::new();
    out.push_str("\\section*{True DSFB Structural Semiotics Instantiation}\n");
    out.push_str(&latex_escape(
        "This pass preserves the DSFB Structural Semiotics Engine as an explicit layered architecture: Residual -> Sign -> Syntax -> Grammar -> Semantics -> Policy. Residuals are saved first, sign tuples sigma_i(t) = (r_i(t), d_i(t), s_i(t)) are exported in dsfb_signs.csv and dsfb_feature_signs.csv, syntax motifs are exported in dsfb_motifs.csv and dsfb_feature_motif_timeline.csv, grammar states are exported in dsfb_grammar_states.csv and dsfb_feature_grammar_states.csv, semantics are exported in dsfb_semantic_matches.csv and dsfb_semantic_ranked_candidates.csv, and policy decisions are saved in dsfb_feature_policy_decisions.csv. No semantic label is assigned directly from raw feature magnitude or scalar baselines alone.",
    ));
    out.push_str("\n\n");
    out
}

fn grouped_coordinated_semiotics_latex_section() -> String {
    let mut out = String::new();
    out.push_str("\\section*{Grouped / Coordinated Semiotics}\n");
    out.push_str(&latex_escape(
        "Candidate grouped semiotics is logged in dsfb_group_definitions.json, and grouped signs, grouped grammar states, and grouped semantic matches are exported only for groups that survive strict failure-vs-pass validation. Grouped structure is therefore tested rather than assumed and does not assert unique causal meaning.",
    ));
    out.push_str("\n\n");
    out
}

fn missed_failure_analysis_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Missed Failure Analysis}\n");
    out.push_str(&latex_escape(&format!(
        "Baseline missed failures indexed: {}. Failure-case artifacts were written as failure_case_<id>.json for each baseline miss.",
        failure_driven.failures_index.missed_failure_ids.len()
    )));
    out.push_str("\n\n");
    out
}

fn failure_priority_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Failure Priority Ranking}\n");
    if let Some(row) = failure_driven.missed_failure_priority.first() {
        out.push_str(&latex_escape(&format!(
            "Highest-priority missed failure: {} with priority score {:.4}, top feature {:?}, and miss rule {}.",
            row.failure_id, row.priority_score, row.top_feature_name, row.exact_miss_rule
        )));
    } else {
        out.push_str("No missed-failure priority rows were produced.");
    }
    out.push_str("\n\n");
    out
}

fn feature_motif_grounding_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Feature to Motif Grounding}\n");
    if let Some(row) = failure_driven.feature_motif_grounding.first() {
        out.push_str(&latex_escape(&format!(
            "Grounding artifacts were written to feature_motif_grounding.json. Example: {} grounded as {} with dominant DSFB motif {}.",
            row.feature_name, row.motif_type, row.dominant_dsfb_motif
        )));
    } else {
        out.push_str("No feature-to-motif grounding rows were produced.");
    }
    out.push_str("\n\n");
    out
}

fn feature_role_validation_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Feature Role Validation}\n");
    if let Some(row) = failure_driven.feature_role_validation.first() {
        out.push_str(&latex_escape(&format!(
            "Feature-role validation rows produced: {}. Example: {} starts as role {} with initial motif {}, is marked {}, and ends at role {} with motif {}.",
            failure_driven.feature_role_validation.len(),
            row.feature_id,
            row.initial_role,
            row.initial_motif,
            row.supported_or_revised_or_rejected,
            row.final_role,
            row.final_motif,
        )));
    } else {
        out.push_str("No feature-role validation rows were produced.");
    }
    out.push_str("\n\n");
    out
}

fn minimal_heuristics_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Heuristics With Justification}\n");
    out.push_str(&latex_escape(&format!(
        "Failure-driven minimal heuristics bank size: {}. The bank is written to dsfb_heuristics_bank_minimal.json and each entry targets one missed failure or one nuisance class.",
        failure_driven.minimal_heuristics_bank.len()
    )));
    out.push_str("\n\n");
    out
}

fn heuristic_provenance_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Heuristic Provenance}\n");
    out.push_str(&latex_escape(&format!(
        "Heuristic provenance rows produced: {}. Every minimal heuristic is linked to explicit failure IDs, feature IDs, nuisance class, intended effect, and constraints in dsfb_heuristic_provenance.csv.",
        failure_driven.heuristic_provenance.len()
    )));
    out.push_str("\n\n");
    out
}

fn group_validation_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{Grouped Semiotics Validation}\n");
    out.push_str(&latex_escape(&format!(
        "Grouped semiotics validation rows produced: {}. Each candidate group records failure co-activation, pass co-activation, retain/reject status, and reason in dsfb_group_validation.csv.",
        failure_driven.group_validation.len()
    )));
    out.push_str("\n\n");
    out
}

fn negative_control_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let report = &failure_driven.negative_control_report;
    let mut out = String::new();
    out.push_str("\\section*{Negative Control}\n");
    out.push_str(&latex_escape(&format!(
        "Pass-run false activation rate {:.4}, pass-run false episode rate {:.4}, pass-run Review/Escalate points {}, pass-run Review/Escalate episodes {}, clean-window false activation rate {:.4}, clean-window false episode rate {:.4}, clean-window Review/Escalate points {}, and clean-window Review/Escalate episodes {}.",
        report.false_activation_rate,
        report.false_episode_rate,
        report.review_escalate_points_on_pass_runs,
        report.review_escalate_episodes_on_pass_runs,
        report.clean_window_false_activation_rate,
        report.clean_window_false_episode_rate,
        report.review_escalate_points_on_clean_runs,
        report.review_escalate_episodes_on_clean_runs,
    )));
    out.push_str("\n\n");
    out
}

fn dsfb_vs_ewma_latex_section(failure_driven: &FailureDrivenArtifacts) -> String {
    let mut out = String::new();
    out.push_str("\\section*{DSFB vs EWMA Separation}\n");
    out.push_str(&latex_escape(&format!(
        "Recovered DSFB-vs-EWMA case files produced: {}.",
        failure_driven.dsfb_vs_ewma_cases.len()
    )));
    out.push_str("\n\n");
    out
}

fn operator_sections_latex(optimization: &OptimizationExecution) -> String {
    let targets = &optimization.operator_delta_targets;
    let mut out = String::new();
    out.push_str("\\section*{Which Delta Matters on SECOM}\n");
    out.push_str(&latex_escape(
        "On the current SECOM evidence, the operator-facing delta is investigation burden on structurally active pass windows, not binary run-level nuisance alone. This report therefore prioritizes Review/Escalate burden, episode fragmentation, precursor quality, and recall-recovery efficiency before lead-time claims.",
    ));
    out.push_str("\n\n");
    out.push_str("\\section*{Predeclared Operator Delta Targets}\n");
    out.push_str(&latex_escape(&format!(
        "Target A: delta_investigation_load >= 0.40. Target B: delta_episode_count >= 0.40. Target C: delta_review_points_per_pass_run >= 0.40. Target D: delta_review_episodes_per_pass_run >= 0.40. Target E: precursor quality preserved or improved. Target F: recall >= 103/104. Target G: recall = 104/104. Investigation baseline {} = {}. Episode baseline {} = {}. Review-burden baseline {} = {:.4} points/pass-run.",
        optimization.operator_baselines.investigation_baseline_layer,
        optimization.operator_baselines.baseline_investigation_points,
        optimization.operator_baselines.episode_baseline_layer,
        optimization.operator_baselines.baseline_episode_count,
        optimization.operator_baselines.review_burden_baseline_layer,
        optimization
            .operator_baselines
            .baseline_review_escalate_points_per_pass_run,
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Optimization Frontier}\n");
    out.push_str(&latex_escape(&format!(
        "Pareto frontier rows: {}. Stage 1 burden-reduction candidates: {}. Stage 2 recall-recovery candidates: {}. Single-change iterations logged: {} with {} accepted.",
        optimization.pareto_frontier.len(),
        optimization.stage1_candidates.len(),
        optimization.stage2_candidates.len(),
        optimization.single_change_iteration_log.len(),
        optimization
            .single_change_iteration_log
            .iter()
            .filter(|row| row.accepted)
            .count(),
    )));
    out.push_str("\n\n");
    out.push_str("\\section*{Recall Recovery Efficiency}\n");
    if let Some(row) = optimization.recall_recovery_efficiency.first() {
        out.push_str(&latex_escape(&format!(
            "{} to {} recovered {} failures with added Review/Escalate points {}, added episodes {}, added pass-run review burden {:.4}, and added nuisance runs {}.",
            row.baseline_configuration,
            row.optimized_configuration,
            row.recovered_failures,
            row.added_review_escalate_points,
            row.added_episode_count,
            row.added_review_points_per_pass_run,
            row.added_nuisance_runs,
        )));
        out.push_str("\n\n");
    }
    out.push_str("\\section*{Target Attainment Assessment}\n");
    out.push_str(&latex_escape(&format!(
        "delta_investigation_load {:.4}. delta_episode_count {:.4}. delta_review_points_per_pass_run {:.4}. delta_review_episodes_per_pass_run {:.4}. precursor quality status {}. recall >= 103/104 {}. recall = 104/104 {}. delta_nuisance_vs_ewma {:.4}.",
        targets.delta_investigation_load,
        targets.delta_episode_count,
        targets.delta_review_points_per_pass_run,
        targets.delta_review_episodes_per_pass_run,
        targets.precursor_quality_status,
        targets.recall_ge_103,
        targets.recall_eq_104,
        targets.delta_nuisance_vs_ewma,
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
            path: "dsa_feature_ranking_dsfb_aware.csv".into(),
            role: "DSFB-semantics-aware ranking emphasizing grammar-qualified semantic persistence, grouped support, and bounded burden penalties.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_ranking_burden_aware.csv".into(),
            role: "Operator-burden-aware feature ranking that penalizes pass-run Review/Escalate burden.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_feature_ranking_comparison.csv".into(),
            role: "Side-by-side comparison of compression-biased, recall-aware, burden-aware, and DSFB-semantics-aware ranking positions.".into(),
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
            path: "dsa_stage1_candidates.csv".into(),
            role: "Stage 1 burden-reduction candidates under the operator-priority objective.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_stage2_candidates.csv".into(),
            role: "Stage 2 recall-recovery candidates under the operator-priority objective.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_missed_failure_diagnostics.csv".into(),
            role: "Per-failure diagnostic table for baseline-missed failures and rescue recoverability.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_operator_baselines.json".into(),
            role: "Explicit operator-burden baseline layers: numeric-only DSA, current policy DSA, and raw boundary.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_operator_delta_targets.json".into(),
            role: "Predeclared operator delta targets and selected-row attainment numbers.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_operator_delta_attainment_matrix.csv".into(),
            role: "Explicit pass/fail matrix for the operator-facing burden, recall, nuisance, and lead thresholds.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_policy_operator_burden_contributions.csv".into(),
            role: "Motif- and feature-level contributions to operator burden and recovered recall.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_recall_recovery_efficiency.csv".into(),
            role: "Recovered failures per added Review/Escalate point, per added episode, per added burden, and per added nuisance run.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_single_change_iteration_log.csv".into(),
            role: "Failure- or nuisance-justified single-change optimization log with per-iteration metric deltas and acceptance decisions.".into(),
        },
        ArtifactInventoryEntry {
            path: "optimization_log.json".into(),
            role: "JSON mirror of the single-change optimization log for deterministic audit and replay.".into(),
        },
        ArtifactInventoryEntry {
            path: "missed_failure_priority.csv".into(),
            role: "Priority-ranked missed failures using signal strength, feature concentration, separation from noise, and recoverability estimate.".into(),
        },
        ArtifactInventoryEntry {
            path: "feature_to_motif.json".into(),
            role: "Hard-locked feature-to-motif assignment for the top failure-relevant SECOM features.".into(),
        },
        ArtifactInventoryEntry {
            path: "negative_control_report.json".into(),
            role: "Pass-run and clean-window false-activation and false-episode rates used as anti-overfit controls.".into(),
        },
        ArtifactInventoryEntry {
            path: "non_intrusive_interface_spec.md".into(),
            role: "Run-local advisory-only interface contract documenting the read-only side-channel integration model.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_heuristic_provenance.csv".into(),
            role: "Explicit failure IDs, feature IDs, nuisance class, intended effect, and constraints for each minimal heuristic.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_feature_role_validation.csv".into(),
            role: "Empirical support, revision, or rejection of the locked SECOM feature-role scaffold.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_group_validation.csv".into(),
            role: "Failure/pass co-activation validation table for the locked grouped-semiotics candidates.".into(),
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
            path: "dsa_cohort_results_dsfb_aware.csv".into(),
            role: "DSFB-semantics-aware cohort results under the optimized deterministic rescue policy.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_results_burden_aware.csv".into(),
            role: "Burden-aware cohort results under the optimized deterministic rescue policy.".into(),
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
            path: "dsa_cohort_summary_dsfb_aware.json".into(),
            role: "DSFB-semantics-aware cohort summary for direct comparison with the other ranking strategies.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsa_cohort_summary_burden_aware.json".into(),
            role: "Burden-aware cohort summary for direct comparison with the other ranking strategies.".into(),
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
            path: "dsfb_signs.csv".into(),
            role: "Residual, drift, and slew sign tuples exported per feature and run.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_feature_signs.csv".into(),
            role: "Scaffolded top-feature sign tuples r(t), d(t), s(t) for the mapped SECOM features.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_motifs.csv".into(),
            role: "Deterministic temporal motif summary built from residual, drift, and slew trajectories.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_motif_labels_per_time.csv".into(),
            role: "Per-feature, per-run temporal motif labels from the syntax layer.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_feature_motif_timeline.csv".into(),
            role: "Scaffolded per-feature motif timeline for the mapped SECOM features.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_grammar_states.csv".into(),
            role: "DSFB admissibility grammar states per feature and run.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_feature_grammar_states.csv".into(),
            role: "Feature-scaffold grammar labels for the mapped SECOM features.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_envelope_interaction_summary.csv".into(),
            role: "Per-feature envelope interaction summary over boundary grazing, drift pressure, violations, and recovery.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_heuristics_bank_expanded.json".into(),
            role: "Expanded DSFB heuristics bank with grammar constraints, feature scope, ambiguity notes, and burden classes.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_semantic_matches.csv".into(),
            role: "Grammar-filtered heuristic semantic matches.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_semantic_ranked_candidates.csv".into(),
            role: "Ranked semantic candidates after grammar and motif filtering.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_feature_policy_decisions.csv".into(),
            role: "Feature-scaffold policy decisions produced only after grammar-qualified semantic retrieval.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_traceability.json".into(),
            role: "End-to-end event traceability chain from residual and sign through motif, grammar, semantic label, and policy state.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_group_definitions.json".into(),
            role: "Data-grounded grouped-semiotics scaffold definitions with member roles, preferred motifs, and empirical support counts.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_group_signs.csv".into(),
            role: "Grouped residual sign tuples for the scaffolded coordinated-semiotics pass.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_group_grammar_states.csv".into(),
            role: "Grouped admissibility states for scaffolded coordinated semiotics.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_group_semantic_matches.csv".into(),
            role: "Grouped semantic matches for coordinated DSFB scaffold motifs.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_structural_delta_metrics.json".into(),
            role: "Grammar violation precision, motif precision pre-failure, structural separation, precursor stability, episode precision, and compression ratio metrics.".into(),
        },
        ArtifactInventoryEntry {
            path: "figures/dsfb_non_intrusive_architecture.png".into(),
            role: "Operator-facing grayscale side-channel architecture figure proving read-only DSFB integration.".into(),
        },
        ArtifactInventoryEntry {
            path: "figures/dsfb_non_intrusive_architecture.svg".into(),
            role: "Vector version of the non-intrusive DSFB side-channel architecture figure.".into(),
        },
        ArtifactInventoryEntry {
            path: "recurrent_boundary_stats.json".into(),
            role: "SECOM structural-limitation summary for recurrent_boundary_approach as both precursor and nuisance source.".into(),
        },
        ArtifactInventoryEntry {
            path: "recurrent_boundary_tradeoff_curve.csv".into(),
            role: "Suppression sweep over recurrent_boundary_approach showing recall versus nuisance reduction.".into(),
        },
        ArtifactInventoryEntry {
            path: "recurrent_boundary_tradeoff_plot.png".into(),
            role: "Plot of the recurrent_boundary_approach suppression tradeoff curve.".into(),
        },
        ArtifactInventoryEntry {
            path: "dsfb_metric_regrounding.csv".into(),
            role: "Metric table with deltas regrounded separately against EWMA, threshold, and numeric-only DSA baselines.".into(),
        },
        ArtifactInventoryEntry {
            path: "target_d_regression_analysis.json".into(),
            role: "Causal analysis of the review-episodes-per-pass-run regression, with either a bounded fix or a formal tradeoff justification.".into(),
        },
        ArtifactInventoryEntry {
            path: "missed_failure_root_cause.json".into(),
            role: "Root-cause artifact for the former 103/104 limiting failure and its bounded recovery path.".into(),
        },
        ArtifactInventoryEntry {
            path: "lead_time_comparison.csv".into(),
            role: "Per-failure comparison of threshold lead, DSA lead, and earliest semantic-match lead.".into(),
        },
        ArtifactInventoryEntry {
            path: "lead_time_explanation.json".into(),
            role: "Why DSFB fires later than threshold on SECOM, validated against semantic-emergence timing.".into(),
        },
        ArtifactInventoryEntry {
            path: "episode_precision_metrics.json".into(),
            role: "Primary operator-facing episode-precision summary, including the raw-boundary precision proxy and gain factor.".into(),
        },
        ArtifactInventoryEntry {
            path: "paper_abstract_artifact.txt".into(),
            role: "Bounded abstract-ready value statement grounded in the saved SECOM artifacts.".into(),
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
            } else if file == "dsfb_non_intrusive_architecture.png" {
                "Non-intrusive DSFB side-channel architecture. The primary SPC/EWMA/controller path remains authoritative; DSFB reads residual and alarm taps only, emits advisory interpretations, and has no feedback arrow into control."
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
        latex_escape(&format_option_f64(
            metric.pre_failure_window_precision_proxy
        )),
    )
}

fn compile_pdf(tex_path: &Path, output_dir: &Path) -> (Option<PathBuf>, Option<String>) {
    crate::output_paths::compile_pdf(tex_path, output_dir)
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values.join(", ")
    }
}

fn format_option_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "n/a".into())
}

fn latex_escape(input: &str) -> String {
    input
        .replace('≥', "$\\geq$")
        .replace('≤', "$\\leq$")
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
