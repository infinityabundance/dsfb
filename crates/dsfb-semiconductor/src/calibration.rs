use crate::baselines::compute_baselines;
use crate::config::PipelineConfig;
use crate::dataset::secom;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::evaluate_grammar;
use crate::metrics::compute_metrics;
use crate::nominal::build_nominal_model;
use crate::output_paths::{create_timestamped_run_dir, default_output_root};
use crate::preprocessing::prepare_secom;
use crate::residual::compute_residuals;
use crate::signs::compute_signs;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationGrid {
    pub healthy_pass_runs: Vec<usize>,
    pub drift_window: Vec<usize>,
    pub envelope_sigma: Vec<f64>,
    pub boundary_fraction_of_rho: Vec<f64>,
    pub ewma_alpha: Vec<f64>,
    pub ewma_sigma_multiplier: Vec<f64>,
    pub drift_sigma_multiplier: Vec<f64>,
    pub slew_sigma_multiplier: Vec<f64>,
    pub grazing_window: Vec<usize>,
    pub grazing_min_hits: Vec<usize>,
    pub pre_failure_lookback_runs: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationArtifacts {
    pub run_dir: PathBuf,
    pub grid_results_csv: PathBuf,
    pub summary_json: PathBuf,
    pub report_markdown: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationRunConfiguration {
    dataset: String,
    data_root: String,
    output_root: String,
    fetch_if_missing: bool,
    grid: CalibrationGrid,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationSummary {
    grid_point_count: usize,
    top_by_boundary_recall: Option<CalibrationResultRow>,
    top_by_boundary_mean_lead: Option<CalibrationResultRow>,
    top_by_low_boundary_nuisance: Option<CalibrationResultRow>,
    top_by_boundary_minus_ewma_delta: Option<CalibrationResultRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationResultRow {
    pub config_id: usize,
    pub healthy_pass_runs: usize,
    pub drift_window: usize,
    pub envelope_sigma: f64,
    pub boundary_fraction_of_rho: f64,
    pub ewma_alpha: f64,
    pub ewma_sigma_multiplier: f64,
    pub drift_sigma_multiplier: f64,
    pub slew_sigma_multiplier: f64,
    pub grazing_window: usize,
    pub grazing_min_hits: usize,
    pub pre_failure_lookback_runs: usize,
    pub analyzable_feature_count: usize,
    pub failure_runs: usize,
    pub dsfb_any_recall: usize,
    pub dsfb_boundary_recall: usize,
    pub dsfb_violation_recall: usize,
    pub ewma_recall: usize,
    pub threshold_recall: usize,
    pub mean_boundary_lead_runs: Option<f64>,
    pub mean_violation_lead_runs: Option<f64>,
    pub mean_ewma_lead_runs: Option<f64>,
    pub mean_threshold_lead_runs: Option<f64>,
    pub mean_boundary_minus_ewma_delta_runs: Option<f64>,
    pub mean_boundary_minus_threshold_delta_runs: Option<f64>,
    pub pass_run_dsfb_boundary_nuisance_rate: f64,
    pub pass_run_dsfb_violation_nuisance_rate: f64,
    pub pass_run_ewma_nuisance_rate: f64,
    pub pass_run_threshold_nuisance_rate: f64,
    pub boundary_episode_count: usize,
    pub mean_boundary_episode_length: Option<f64>,
    pub non_escalating_boundary_episode_fraction: Option<f64>,
    pub pre_failure_slow_drift_precision_proxy: Option<f64>,
    pub transient_excursion_precision_proxy: Option<f64>,
    pub recurrent_boundary_approach_precision_proxy: Option<f64>,
}

impl CalibrationGrid {
    pub fn validate(&self) -> Result<()> {
        let grid_point_count = self.grid_point_count();
        if grid_point_count == 0 {
            return Err(DsfbSemiconductorError::DatasetFormat(
                "calibration grid must contain at least one point".into(),
            ));
        }
        if grid_point_count > 4096 {
            return Err(DsfbSemiconductorError::DatasetFormat(format!(
                "calibration grid is too large ({grid_point_count} points); reduce the grid before running"
            )));
        }
        Ok(())
    }

    pub fn grid_point_count(&self) -> usize {
        [
            self.healthy_pass_runs.len(),
            self.drift_window.len(),
            self.envelope_sigma.len(),
            self.boundary_fraction_of_rho.len(),
            self.ewma_alpha.len(),
            self.ewma_sigma_multiplier.len(),
            self.drift_sigma_multiplier.len(),
            self.slew_sigma_multiplier.len(),
            self.grazing_window.len(),
            self.grazing_min_hits.len(),
            self.pre_failure_lookback_runs.len(),
        ]
        .into_iter()
        .product()
    }

    pub fn expand(&self) -> Vec<PipelineConfig> {
        let mut configs = Vec::new();
        for &healthy_pass_runs in &self.healthy_pass_runs {
            for &drift_window in &self.drift_window {
                for &envelope_sigma in &self.envelope_sigma {
                    for &boundary_fraction_of_rho in &self.boundary_fraction_of_rho {
                        for &ewma_alpha in &self.ewma_alpha {
                            for &ewma_sigma_multiplier in &self.ewma_sigma_multiplier {
                                for &drift_sigma_multiplier in &self.drift_sigma_multiplier {
                                    for &slew_sigma_multiplier in &self.slew_sigma_multiplier {
                                        for &grazing_window in &self.grazing_window {
                                            for &grazing_min_hits in &self.grazing_min_hits {
                                                for &pre_failure_lookback_runs in
                                                    &self.pre_failure_lookback_runs
                                                {
                                                    configs.push(PipelineConfig {
                                                        healthy_pass_runs,
                                                        drift_window,
                                                        envelope_sigma,
                                                        boundary_fraction_of_rho,
                                                        ewma_alpha,
                                                        ewma_sigma_multiplier,
                                                        drift_sigma_multiplier,
                                                        slew_sigma_multiplier,
                                                        grazing_window,
                                                        grazing_min_hits,
                                                        pre_failure_lookback_runs,
                                                        ..PipelineConfig::default()
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        configs
    }
}

pub fn run_secom_calibration(
    data_root: &Path,
    output_root: Option<&Path>,
    grid: CalibrationGrid,
    fetch_if_missing: bool,
) -> Result<CalibrationArtifacts> {
    grid.validate()?;

    let _paths = if fetch_if_missing {
        secom::fetch_if_missing(data_root)?
    } else {
        secom::ensure_present(data_root)?
    };
    let dataset = secom::load_from_root(data_root)?;

    let output_root = output_root
        .map(Path::to_path_buf)
        .unwrap_or_else(default_output_root);
    fs::create_dir_all(&output_root)?;
    let run_dir = create_timestamped_run_dir(&output_root, "secom_calibration")?;

    let expanded = grid.expand();
    let mut rows = Vec::with_capacity(expanded.len());
    for (config_id, config) in expanded.iter().enumerate() {
        config
            .validate()
            .map_err(DsfbSemiconductorError::DatasetFormat)?;
        let prepared = prepare_secom(&dataset, config)?;
        let nominal = build_nominal_model(&prepared, config);
        let residuals = compute_residuals(&prepared, &nominal);
        let signs = compute_signs(&prepared, &nominal, &residuals, config);
        let baselines = compute_baselines(&prepared, &nominal, &residuals, config);
        let grammar = evaluate_grammar(&residuals, &signs, &nominal, config);
        let metrics = compute_metrics(
            &prepared,
            &nominal,
            &residuals,
            &signs,
            &baselines,
            &grammar,
            config.pre_failure_lookback_runs,
        );

        rows.push(CalibrationResultRow {
            config_id,
            healthy_pass_runs: config.healthy_pass_runs,
            drift_window: config.drift_window,
            envelope_sigma: config.envelope_sigma,
            boundary_fraction_of_rho: config.boundary_fraction_of_rho,
            ewma_alpha: config.ewma_alpha,
            ewma_sigma_multiplier: config.ewma_sigma_multiplier,
            drift_sigma_multiplier: config.drift_sigma_multiplier,
            slew_sigma_multiplier: config.slew_sigma_multiplier,
            grazing_window: config.grazing_window,
            grazing_min_hits: config.grazing_min_hits,
            pre_failure_lookback_runs: config.pre_failure_lookback_runs,
            analyzable_feature_count: metrics.summary.analyzable_feature_count,
            failure_runs: metrics.summary.failure_runs,
            dsfb_any_recall: metrics.summary.failure_runs_with_preceding_dsfb_signal,
            dsfb_boundary_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_boundary_signal,
            dsfb_violation_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_violation_signal,
            ewma_recall: metrics.summary.failure_runs_with_preceding_ewma_signal,
            threshold_recall: metrics.summary.failure_runs_with_preceding_threshold_signal,
            mean_boundary_lead_runs: metrics.lead_time_summary.mean_boundary_lead_runs,
            mean_violation_lead_runs: metrics.lead_time_summary.mean_violation_lead_runs,
            mean_ewma_lead_runs: metrics.lead_time_summary.mean_ewma_lead_runs,
            mean_threshold_lead_runs: metrics.lead_time_summary.mean_threshold_lead_runs,
            mean_boundary_minus_ewma_delta_runs: metrics
                .lead_time_summary
                .mean_boundary_minus_ewma_delta_runs,
            mean_boundary_minus_threshold_delta_runs: metrics
                .lead_time_summary
                .mean_boundary_minus_threshold_delta_runs,
            pass_run_dsfb_boundary_nuisance_rate: metrics
                .summary
                .pass_run_dsfb_boundary_nuisance_rate,
            pass_run_dsfb_violation_nuisance_rate: metrics
                .summary
                .pass_run_dsfb_violation_nuisance_rate,
            pass_run_ewma_nuisance_rate: metrics.summary.pass_run_ewma_nuisance_rate,
            pass_run_threshold_nuisance_rate: metrics.summary.pass_run_threshold_nuisance_rate,
            boundary_episode_count: metrics.boundary_episode_summary.episode_count,
            mean_boundary_episode_length: metrics.boundary_episode_summary.mean_episode_length,
            non_escalating_boundary_episode_fraction: metrics
                .boundary_episode_summary
                .non_escalating_episode_fraction,
            pre_failure_slow_drift_precision_proxy: motif_precision(
                &metrics,
                "pre_failure_slow_drift",
            ),
            transient_excursion_precision_proxy: motif_precision(&metrics, "transient_excursion"),
            recurrent_boundary_approach_precision_proxy: motif_precision(
                &metrics,
                "recurrent_boundary_approach",
            ),
        });
    }

    let grid_results_csv = run_dir.join("calibration_grid_results.csv");
    let summary_json = run_dir.join("calibration_best_by_metric.json");
    let report_markdown = run_dir.join("calibration_report.md");

    write_json_pretty(
        &run_dir.join("calibration_run_configuration.json"),
        &CalibrationRunConfiguration {
            dataset: "SECOM".into(),
            data_root: data_root.display().to_string(),
            output_root: output_root.display().to_string(),
            fetch_if_missing,
            grid: grid.clone(),
        },
    )?;
    write_json_pretty(&run_dir.join("parameter_grid_manifest.json"), &grid)?;
    write_calibration_results_csv(&grid_results_csv, &rows)?;
    write_json_pretty(&summary_json, &build_calibration_summary(&rows))?;
    fs::write(&report_markdown, calibration_report_markdown(&rows))?;

    Ok(CalibrationArtifacts {
        run_dir,
        grid_results_csv,
        summary_json,
        report_markdown,
    })
}

fn motif_precision(metrics: &crate::metrics::BenchmarkMetrics, motif_name: &str) -> Option<f64> {
    metrics
        .motif_metrics
        .iter()
        .find(|metric| metric.motif_name == motif_name)
        .and_then(|metric| metric.pre_failure_window_precision_proxy)
}

fn build_calibration_summary(rows: &[CalibrationResultRow]) -> CalibrationSummary {
    CalibrationSummary {
        grid_point_count: rows.len(),
        top_by_boundary_recall: best_row_by(rows, |left, right| {
            left.dsfb_boundary_recall
                .cmp(&right.dsfb_boundary_recall)
                .then_with(|| {
                    cmp_option_f64(left.mean_boundary_lead_runs, right.mean_boundary_lead_runs)
                })
                .then_with(|| {
                    cmp_f64_smallest(
                        left.pass_run_dsfb_boundary_nuisance_rate,
                        right.pass_run_dsfb_boundary_nuisance_rate,
                    )
                })
        }),
        top_by_boundary_mean_lead: best_row_by(rows, |left, right| {
            cmp_option_f64(left.mean_boundary_lead_runs, right.mean_boundary_lead_runs)
                .then_with(|| left.dsfb_boundary_recall.cmp(&right.dsfb_boundary_recall))
                .then_with(|| {
                    cmp_f64_smallest(
                        left.pass_run_dsfb_boundary_nuisance_rate,
                        right.pass_run_dsfb_boundary_nuisance_rate,
                    )
                })
        }),
        top_by_low_boundary_nuisance: best_row_by(rows, |left, right| {
            cmp_f64_smallest(
                left.pass_run_dsfb_boundary_nuisance_rate,
                right.pass_run_dsfb_boundary_nuisance_rate,
            )
            .then_with(|| left.dsfb_boundary_recall.cmp(&right.dsfb_boundary_recall))
            .then_with(|| {
                cmp_option_f64(left.mean_boundary_lead_runs, right.mean_boundary_lead_runs)
            })
        }),
        top_by_boundary_minus_ewma_delta: best_row_by(rows, |left, right| {
            cmp_option_f64(
                left.mean_boundary_minus_ewma_delta_runs,
                right.mean_boundary_minus_ewma_delta_runs,
            )
            .then_with(|| left.dsfb_boundary_recall.cmp(&right.dsfb_boundary_recall))
            .then_with(|| {
                cmp_f64_smallest(
                    left.pass_run_dsfb_boundary_nuisance_rate,
                    right.pass_run_dsfb_boundary_nuisance_rate,
                )
            })
        }),
    }
}

fn best_row_by<F>(rows: &[CalibrationResultRow], compare: F) -> Option<CalibrationResultRow>
where
    F: Fn(&CalibrationResultRow, &CalibrationResultRow) -> std::cmp::Ordering,
{
    rows.iter().cloned().max_by(compare)
}

fn cmp_option_f64(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left
            .partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn cmp_f64_smallest(left: f64, right: f64) -> std::cmp::Ordering {
    right
        .partial_cmp(&left)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn write_calibration_results_csv(path: &Path, rows: &[CalibrationResultRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn calibration_report_markdown(rows: &[CalibrationResultRow]) -> String {
    let summary = build_calibration_summary(rows);
    let mut out = String::new();
    out.push_str("# DSFB SECOM Calibration Report\n\n");
    out.push_str(&format!("- Grid points evaluated: {}\n\n", rows.len()));

    append_top_row(
        &mut out,
        "Top by boundary recall",
        &summary.top_by_boundary_recall,
    );
    append_top_row(
        &mut out,
        "Top by boundary mean lead",
        &summary.top_by_boundary_mean_lead,
    );
    append_top_row(
        &mut out,
        "Top by low boundary nuisance",
        &summary.top_by_low_boundary_nuisance,
    );
    append_top_row(
        &mut out,
        "Top by boundary minus EWMA lead delta",
        &summary.top_by_boundary_minus_ewma_delta,
    );

    out.push_str("## Notes\n\n");
    out.push_str("- These calibration outputs are deterministic grid-search artifacts on SECOM.\n");
    out.push_str(
        "- The nuisance values are pass-run proxies, not fab-qualified false-alarm metrics.\n",
    );
    out.push_str("- The lead-time values are measured against fixed pre-failure lookback windows on the available labels.\n");
    out
}

fn append_top_row(out: &mut String, title: &str, row: &Option<CalibrationResultRow>) {
    out.push_str(&format!("## {title}\n\n"));
    if let Some(row) = row {
        out.push_str(&format!(
            "- config_id: {}\n- dsfb_boundary_recall: {}\n- mean_boundary_lead_runs: {}\n- mean_boundary_minus_ewma_delta_runs: {}\n- pass_run_dsfb_boundary_nuisance_rate: {:.4}\n\n",
            row.config_id,
            row.dsfb_boundary_recall,
            format_option_f64(row.mean_boundary_lead_runs),
            format_option_f64(row.mean_boundary_minus_ewma_delta_runs),
            row.pass_run_dsfb_boundary_nuisance_rate,
        ));
    } else {
        out.push_str("- none\n\n");
    }
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}
