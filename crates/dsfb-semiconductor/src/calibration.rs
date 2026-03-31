use crate::baselines::compute_baselines;
use crate::config::PipelineConfig;
use crate::dataset::secom;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::evaluate_grammar;
use crate::metrics::compute_metrics;
use crate::nominal::build_nominal_model;
use crate::output_paths::{create_timestamped_run_dir, default_output_root};
use crate::preprocessing::prepare_secom;
use crate::precursor::{
    run_precursor_calibration_grid, PrecursorCalibrationGrid, PrecursorCalibrationRow,
};
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
    pub state_confirmation_steps: Vec<usize>,
    pub persistent_state_steps: Vec<usize>,
    pub density_window: Vec<usize>,
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
pub struct PrecursorCalibrationArtifacts {
    pub run_dir: PathBuf,
    pub grid_results_csv: PathBuf,
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
    top_by_persistent_boundary_recall: Option<CalibrationResultRow>,
    top_by_persistent_boundary_mean_lead: Option<CalibrationResultRow>,
    top_by_low_persistent_boundary_nuisance: Option<CalibrationResultRow>,
    top_by_persistent_boundary_minus_threshold_delta: Option<CalibrationResultRow>,
    top_by_persistent_boundary_minus_ewma_delta: Option<CalibrationResultRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalibrationResultRow {
    pub config_id: usize,
    pub healthy_pass_runs: usize,
    pub drift_window: usize,
    pub envelope_sigma: f64,
    pub boundary_fraction_of_rho: f64,
    pub state_confirmation_steps: usize,
    pub persistent_state_steps: usize,
    pub density_window: usize,
    pub ewma_alpha: f64,
    pub ewma_sigma_multiplier: f64,
    pub drift_sigma_multiplier: f64,
    pub slew_sigma_multiplier: f64,
    pub grazing_window: usize,
    pub grazing_min_hits: usize,
    pub pre_failure_lookback_runs: usize,
    pub analyzable_feature_count: usize,
    pub failure_runs: usize,
    pub dsfb_raw_recall: usize,
    pub dsfb_persistent_recall: usize,
    pub dsfb_raw_boundary_recall: usize,
    pub dsfb_persistent_boundary_recall: usize,
    pub dsfb_raw_violation_recall: usize,
    pub dsfb_persistent_violation_recall: usize,
    pub ewma_recall: usize,
    pub threshold_recall: usize,
    pub mean_raw_boundary_lead_runs: Option<f64>,
    pub mean_persistent_boundary_lead_runs: Option<f64>,
    pub mean_raw_violation_lead_runs: Option<f64>,
    pub mean_persistent_violation_lead_runs: Option<f64>,
    pub mean_ewma_lead_runs: Option<f64>,
    pub mean_threshold_lead_runs: Option<f64>,
    pub mean_persistent_boundary_minus_ewma_delta_runs: Option<f64>,
    pub mean_persistent_boundary_minus_threshold_delta_runs: Option<f64>,
    pub pass_run_dsfb_persistent_boundary_nuisance_rate: f64,
    pub pass_run_dsfb_persistent_violation_nuisance_rate: f64,
    pub pass_run_ewma_nuisance_rate: f64,
    pub pass_run_threshold_nuisance_rate: f64,
    pub persistent_boundary_episode_count: usize,
    pub mean_persistent_boundary_episode_length: Option<f64>,
    pub persistent_non_escalating_boundary_episode_fraction: Option<f64>,
    pub mean_persistent_boundary_density_failure: f64,
    pub mean_persistent_boundary_density_pass: f64,
    pub mean_persistent_violation_density_failure: f64,
    pub mean_persistent_violation_density_pass: f64,
    pub mean_threshold_density_failure: f64,
    pub mean_threshold_density_pass: f64,
    pub mean_ewma_density_failure: f64,
    pub mean_ewma_density_pass: f64,
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
            self.state_confirmation_steps.len(),
            self.persistent_state_steps.len(),
            self.density_window.len(),
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
                        for &state_confirmation_steps in &self.state_confirmation_steps {
                            for &persistent_state_steps in &self.persistent_state_steps {
                                for &density_window in &self.density_window {
                                    for &ewma_alpha in &self.ewma_alpha {
                                        for &ewma_sigma_multiplier in &self.ewma_sigma_multiplier {
                                            for &drift_sigma_multiplier in
                                                &self.drift_sigma_multiplier
                                            {
                                                for &slew_sigma_multiplier in
                                                    &self.slew_sigma_multiplier
                                                {
                                                    for &grazing_window in &self.grazing_window {
                                                        for &grazing_min_hits in
                                                            &self.grazing_min_hits
                                                        {
                                                            for &pre_failure_lookback_runs in
                                                                &self.pre_failure_lookback_runs
                                                            {
                                                                configs.push(PipelineConfig {
                                                                    healthy_pass_runs,
                                                                    drift_window,
                                                                    envelope_sigma,
                                                                    boundary_fraction_of_rho,
                                                                    state_confirmation_steps,
                                                                    persistent_state_steps,
                                                                    density_window,
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
            config,
        );

        rows.push(CalibrationResultRow {
            config_id,
            healthy_pass_runs: config.healthy_pass_runs,
            drift_window: config.drift_window,
            envelope_sigma: config.envelope_sigma,
            boundary_fraction_of_rho: config.boundary_fraction_of_rho,
            state_confirmation_steps: config.state_confirmation_steps,
            persistent_state_steps: config.persistent_state_steps,
            density_window: config.density_window,
            ewma_alpha: config.ewma_alpha,
            ewma_sigma_multiplier: config.ewma_sigma_multiplier,
            drift_sigma_multiplier: config.drift_sigma_multiplier,
            slew_sigma_multiplier: config.slew_sigma_multiplier,
            grazing_window: config.grazing_window,
            grazing_min_hits: config.grazing_min_hits,
            pre_failure_lookback_runs: config.pre_failure_lookback_runs,
            analyzable_feature_count: metrics.summary.analyzable_feature_count,
            failure_runs: metrics.summary.failure_runs,
            dsfb_raw_recall: metrics.summary.failure_runs_with_preceding_dsfb_raw_signal,
            dsfb_persistent_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_signal,
            dsfb_raw_boundary_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_raw_boundary_signal,
            dsfb_persistent_boundary_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_boundary_signal,
            dsfb_raw_violation_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_raw_violation_signal,
            dsfb_persistent_violation_recall: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_violation_signal,
            ewma_recall: metrics.summary.failure_runs_with_preceding_ewma_signal,
            threshold_recall: metrics.summary.failure_runs_with_preceding_threshold_signal,
            mean_raw_boundary_lead_runs: metrics.lead_time_summary.mean_raw_boundary_lead_runs,
            mean_persistent_boundary_lead_runs: metrics
                .lead_time_summary
                .mean_persistent_boundary_lead_runs,
            mean_raw_violation_lead_runs: metrics.lead_time_summary.mean_raw_violation_lead_runs,
            mean_persistent_violation_lead_runs: metrics
                .lead_time_summary
                .mean_persistent_violation_lead_runs,
            mean_ewma_lead_runs: metrics.lead_time_summary.mean_ewma_lead_runs,
            mean_threshold_lead_runs: metrics.lead_time_summary.mean_threshold_lead_runs,
            mean_persistent_boundary_minus_ewma_delta_runs: metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_ewma_delta_runs,
            mean_persistent_boundary_minus_threshold_delta_runs: metrics
                .lead_time_summary
                .mean_persistent_boundary_minus_threshold_delta_runs,
            pass_run_dsfb_persistent_boundary_nuisance_rate: metrics
                .summary
                .pass_run_dsfb_persistent_boundary_nuisance_rate,
            pass_run_dsfb_persistent_violation_nuisance_rate: metrics
                .summary
                .pass_run_dsfb_persistent_violation_nuisance_rate,
            pass_run_ewma_nuisance_rate: metrics.summary.pass_run_ewma_nuisance_rate,
            pass_run_threshold_nuisance_rate: metrics.summary.pass_run_threshold_nuisance_rate,
            persistent_boundary_episode_count: metrics
                .boundary_episode_summary
                .persistent_episode_count,
            mean_persistent_boundary_episode_length: metrics
                .boundary_episode_summary
                .mean_persistent_episode_length,
            persistent_non_escalating_boundary_episode_fraction: metrics
                .boundary_episode_summary
                .persistent_non_escalating_episode_fraction,
            mean_persistent_boundary_density_failure: metrics
                .density_summary
                .mean_persistent_boundary_density_failure,
            mean_persistent_boundary_density_pass: metrics
                .density_summary
                .mean_persistent_boundary_density_pass,
            mean_persistent_violation_density_failure: metrics
                .density_summary
                .mean_persistent_violation_density_failure,
            mean_persistent_violation_density_pass: metrics
                .density_summary
                .mean_persistent_violation_density_pass,
            mean_threshold_density_failure: metrics.density_summary.mean_threshold_density_failure,
            mean_threshold_density_pass: metrics.density_summary.mean_threshold_density_pass,
            mean_ewma_density_failure: metrics.density_summary.mean_ewma_density_failure,
            mean_ewma_density_pass: metrics.density_summary.mean_ewma_density_pass,
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

    write_grid_results(&grid_results_csv, &rows)?;
    write_summary(
        &summary_json,
        &CalibrationSummary {
            grid_point_count: rows.len(),
            top_by_persistent_boundary_recall: best_by_persistent_boundary_recall(&rows),
            top_by_persistent_boundary_mean_lead: best_by_persistent_boundary_mean_lead(&rows),
            top_by_low_persistent_boundary_nuisance: best_by_low_persistent_boundary_nuisance(
                &rows,
            ),
            top_by_persistent_boundary_minus_threshold_delta:
                best_by_persistent_boundary_minus_threshold_delta(&rows),
            top_by_persistent_boundary_minus_ewma_delta:
                best_by_persistent_boundary_minus_ewma_delta(&rows),
        },
    )?;
    fs::write(
        run_dir.join("calibration_run_configuration.json"),
        serde_json::to_string_pretty(&CalibrationRunConfiguration {
            dataset: "SECOM".into(),
            data_root: data_root.display().to_string(),
            output_root: output_root.display().to_string(),
            fetch_if_missing,
            grid: grid.clone(),
        })?,
    )?;
    fs::write(
        run_dir.join("parameter_grid_manifest.json"),
        serde_json::to_string_pretty(&grid)?,
    )?;
    fs::write(
        &report_markdown,
        calibration_report(
            &rows,
            best_by_persistent_boundary_recall(&rows).as_ref(),
            best_by_persistent_boundary_minus_threshold_delta(&rows).as_ref(),
            best_by_persistent_boundary_minus_ewma_delta(&rows).as_ref(),
            best_by_low_persistent_boundary_nuisance(&rows).as_ref(),
        ),
    )?;

    Ok(CalibrationArtifacts {
        run_dir,
        grid_results_csv,
        summary_json,
        report_markdown,
    })
}

pub fn run_secom_precursor_calibration(
    data_root: &Path,
    output_root: Option<&Path>,
    config: PipelineConfig,
    fetch_if_missing: bool,
) -> Result<PrecursorCalibrationArtifacts> {
    config
        .validate()
        .map_err(DsfbSemiconductorError::DatasetFormat)?;

    let _paths = if fetch_if_missing {
        secom::fetch_if_missing(data_root)?
    } else {
        secom::ensure_present(data_root)?
    };
    let dataset = secom::load_from_root(data_root)?;
    let prepared = prepare_secom(&dataset, &config)?;
    let nominal = build_nominal_model(&prepared, &config);
    let residuals = compute_residuals(&prepared, &nominal);
    let signs = compute_signs(&prepared, &nominal, &residuals, &config);
    let baselines = compute_baselines(&prepared, &nominal, &residuals, &config);
    let grammar = evaluate_grammar(&residuals, &signs, &nominal, &config);

    let output_root = output_root
        .map(Path::to_path_buf)
        .unwrap_or_else(default_output_root);
    fs::create_dir_all(&output_root)?;
    let run_dir = create_timestamped_run_dir(&output_root, "secom_precursor_calibration")?;
    let grid = PrecursorCalibrationGrid::bounded_default();
    let rows = run_precursor_calibration_grid(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &grid,
        config.pre_failure_lookback_runs,
    )?;

    let grid_results_csv = run_dir.join("precursor_calibration_grid.csv");
    write_precursor_grid_results(&grid_results_csv, &rows)?;
    fs::write(
        run_dir.join("precursor_calibration_run_configuration.json"),
        serde_json::to_string_pretty(&CalibrationRunConfiguration {
            dataset: "SECOM".into(),
            data_root: data_root.display().to_string(),
            output_root: output_root.display().to_string(),
            fetch_if_missing,
            grid: CalibrationGrid {
                healthy_pass_runs: vec![config.healthy_pass_runs],
                drift_window: vec![config.drift_window],
                envelope_sigma: vec![config.envelope_sigma],
                boundary_fraction_of_rho: vec![config.boundary_fraction_of_rho],
                state_confirmation_steps: vec![config.state_confirmation_steps],
                persistent_state_steps: vec![config.persistent_state_steps],
                density_window: vec![config.density_window],
                ewma_alpha: vec![config.ewma_alpha],
                ewma_sigma_multiplier: vec![config.ewma_sigma_multiplier],
                drift_sigma_multiplier: vec![config.drift_sigma_multiplier],
                slew_sigma_multiplier: vec![config.slew_sigma_multiplier],
                grazing_window: vec![config.grazing_window],
                grazing_min_hits: vec![config.grazing_min_hits],
                pre_failure_lookback_runs: vec![config.pre_failure_lookback_runs],
            },
        })?,
    )?;
    fs::write(
        run_dir.join("precursor_parameter_grid_manifest.json"),
        serde_json::to_string_pretty(&grid)?,
    )?;

    Ok(PrecursorCalibrationArtifacts {
        run_dir,
        grid_results_csv,
    })
}

fn write_grid_results(path: &Path, rows: &[CalibrationResultRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_precursor_grid_results(path: &Path, rows: &[PrecursorCalibrationRow]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_summary(path: &Path, summary: &CalibrationSummary) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(summary)?)?;
    Ok(())
}

fn best_by_persistent_boundary_recall(rows: &[CalibrationResultRow]) -> Option<CalibrationResultRow> {
    rows.iter().cloned().max_by(|left, right| {
        left.dsfb_persistent_boundary_recall
            .cmp(&right.dsfb_persistent_boundary_recall)
            .then_with(|| {
                cmp_option_f64(
                    left.mean_persistent_boundary_lead_runs,
                    right.mean_persistent_boundary_lead_runs,
                )
            })
            .then_with(|| {
                cmp_f64_ascending(
                    left.pass_run_dsfb_persistent_boundary_nuisance_rate,
                    right.pass_run_dsfb_persistent_boundary_nuisance_rate,
                )
            })
    })
}

fn best_by_persistent_boundary_mean_lead(
    rows: &[CalibrationResultRow],
) -> Option<CalibrationResultRow> {
    rows.iter().cloned().max_by(|left, right| {
        cmp_option_f64(
            left.mean_persistent_boundary_lead_runs,
            right.mean_persistent_boundary_lead_runs,
        )
        .then_with(|| {
            left.dsfb_persistent_boundary_recall
                .cmp(&right.dsfb_persistent_boundary_recall)
        })
        .then_with(|| {
            cmp_f64_ascending(
                left.pass_run_dsfb_persistent_boundary_nuisance_rate,
                right.pass_run_dsfb_persistent_boundary_nuisance_rate,
            )
        })
    })
}

fn best_by_low_persistent_boundary_nuisance(
    rows: &[CalibrationResultRow],
) -> Option<CalibrationResultRow> {
    rows.iter().cloned().max_by(|left, right| {
        cmp_f64_ascending(
            left.pass_run_dsfb_persistent_boundary_nuisance_rate,
            right.pass_run_dsfb_persistent_boundary_nuisance_rate,
        )
        .then_with(|| {
            left.dsfb_persistent_boundary_recall
                .cmp(&right.dsfb_persistent_boundary_recall)
        })
        .then_with(|| {
            cmp_option_f64(
                left.mean_persistent_boundary_lead_runs,
                right.mean_persistent_boundary_lead_runs,
            )
        })
    })
}

fn best_by_persistent_boundary_minus_threshold_delta(
    rows: &[CalibrationResultRow],
) -> Option<CalibrationResultRow> {
    rows.iter().cloned().max_by(|left, right| {
        cmp_option_f64(
            left.mean_persistent_boundary_minus_threshold_delta_runs,
            right.mean_persistent_boundary_minus_threshold_delta_runs,
        )
        .then_with(|| {
            left.dsfb_persistent_boundary_recall
                .cmp(&right.dsfb_persistent_boundary_recall)
        })
        .then_with(|| {
            cmp_f64_ascending(
                left.pass_run_dsfb_persistent_boundary_nuisance_rate,
                right.pass_run_dsfb_persistent_boundary_nuisance_rate,
            )
        })
    })
}

fn best_by_persistent_boundary_minus_ewma_delta(
    rows: &[CalibrationResultRow],
) -> Option<CalibrationResultRow> {
    rows.iter().cloned().max_by(|left, right| {
        cmp_option_f64(
            left.mean_persistent_boundary_minus_ewma_delta_runs,
            right.mean_persistent_boundary_minus_ewma_delta_runs,
        )
        .then_with(|| {
            left.dsfb_persistent_boundary_recall
                .cmp(&right.dsfb_persistent_boundary_recall)
        })
        .then_with(|| {
            cmp_f64_ascending(
                left.pass_run_dsfb_persistent_boundary_nuisance_rate,
                right.pass_run_dsfb_persistent_boundary_nuisance_rate,
            )
        })
    })
}

fn motif_precision(metrics: &crate::metrics::BenchmarkMetrics, motif_name: &str) -> Option<f64> {
    metrics
        .motif_metrics
        .iter()
        .find(|metric| metric.motif_name == motif_name)
        .and_then(|metric| metric.pre_failure_window_precision_proxy)
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

fn cmp_f64_ascending(left: f64, right: f64) -> std::cmp::Ordering {
    right
        .partial_cmp(&left)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn calibration_report(
    rows: &[CalibrationResultRow],
    best_recall: Option<&CalibrationResultRow>,
    best_threshold_delta: Option<&CalibrationResultRow>,
    best_ewma_delta: Option<&CalibrationResultRow>,
    best_low_nuisance: Option<&CalibrationResultRow>,
) -> String {
    let mut out = String::new();
    out.push_str("# SECOM calibration report\n\n");
    out.push_str(&format!("- Grid points evaluated: {}\n\n", rows.len()));
    out.push_str(
        "This report summarizes deterministic parameter-grid trade-offs over persistent DSFB boundary lead time, recall, and nuisance proxies. It is a calibration report, not a superiority claim.\n\n",
    );

    if let Some(row) = best_recall {
        out.push_str("## Best persistent-boundary recall\n\n");
        out.push_str(&format!(
            "- config_id: {}\n- persistent boundary recall: {}\n- mean persistent boundary lead runs: {}\n- persistent boundary minus threshold delta runs: {}\n- pass-run persistent boundary nuisance rate: {:.4}\n\n",
            row.config_id,
            row.dsfb_persistent_boundary_recall,
            format_option_f64(row.mean_persistent_boundary_lead_runs),
            format_option_f64(row.mean_persistent_boundary_minus_threshold_delta_runs),
            row.pass_run_dsfb_persistent_boundary_nuisance_rate,
        ));
    }

    if let Some(row) = best_threshold_delta {
        out.push_str("## Best persistent-boundary minus threshold delta\n\n");
        out.push_str(&format!(
            "- config_id: {}\n- mean persistent boundary minus threshold delta runs: {}\n- persistent boundary recall: {}\n- pass-run persistent boundary nuisance rate: {:.4}\n\n",
            row.config_id,
            format_option_f64(row.mean_persistent_boundary_minus_threshold_delta_runs),
            row.dsfb_persistent_boundary_recall,
            row.pass_run_dsfb_persistent_boundary_nuisance_rate,
        ));
    }

    if let Some(row) = best_ewma_delta {
        out.push_str("## Best persistent-boundary minus EWMA delta\n\n");
        out.push_str(&format!(
            "- config_id: {}\n- mean persistent boundary minus EWMA delta runs: {}\n- persistent boundary recall: {}\n- pass-run persistent boundary nuisance rate: {:.4}\n\n",
            row.config_id,
            format_option_f64(row.mean_persistent_boundary_minus_ewma_delta_runs),
            row.dsfb_persistent_boundary_recall,
            row.pass_run_dsfb_persistent_boundary_nuisance_rate,
        ));
    }

    if let Some(row) = best_low_nuisance {
        out.push_str("## Lowest persistent-boundary nuisance\n\n");
        out.push_str(&format!(
            "- config_id: {}\n- pass-run persistent boundary nuisance rate: {:.4}\n- persistent boundary recall: {}\n- mean persistent boundary lead runs: {}\n\n",
            row.config_id,
            row.pass_run_dsfb_persistent_boundary_nuisance_rate,
            row.dsfb_persistent_boundary_recall,
            format_option_f64(row.mean_persistent_boundary_lead_runs),
        ));
    }

    out.push_str("## Interpretation\n\n");
    out.push_str(
        "A positive persistent-boundary lead delta is meaningful only if it is paired with acceptable nuisance and bounded calibration sensitivity. In the current crate this grid is intended to surface trade-offs explicitly, not to imply that a favorable configuration is already deployment-ready.\n",
    );
    out
}

fn format_option_f64(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "n/a".into())
}
