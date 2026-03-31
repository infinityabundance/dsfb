use crate::baselines::compute_baselines;
use crate::config::{PipelineConfig, RunConfiguration};
use crate::dataset::phm2018::{support_status as phm_support_status, Phm2018SupportStatus};
use crate::dataset::secom::{self, SecomArchiveLayout};
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::evaluate_grammar;
use crate::heuristics::build_heuristics_bank;
use crate::metrics::{
    compute_metrics, BenchmarkMetrics, BoundaryEpisodeSummary, DensityMetricRecord, DensitySummary,
    LeadTimeSummary, PerFailureRunSignal,
};
use crate::nominal::build_nominal_model;
use crate::output_paths::{create_timestamped_run_dir, default_output_root};
use crate::plots::{generate_figures, FigureManifest};
use crate::precursor::{
    evaluate_dsa, DsaEvaluation, DsaRunSignals, DsaVsBaselinesSummary, PerFailureRunDsaSignal,
};
use crate::preprocessing::prepare_secom;
use crate::report::{write_reports, ReportArtifacts};
use crate::residual::compute_residuals;
use crate::signs::compute_signs;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, Serialize)]
pub struct SecomRunArtifacts {
    pub run_dir: PathBuf,
    pub report: ReportArtifacts,
    pub figures: FigureManifest,
    pub metrics_path: PathBuf,
    pub manifest_path: PathBuf,
    pub zip_path: PathBuf,
    pub phm2018_status: Phm2018SupportStatus,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactManifest {
    dataset: String,
    run_dir: String,
    metrics_summary_path: String,
    baseline_comparison_summary_path: String,
    dsa_vs_baselines_summary_path: String,
    dsa_parameter_manifest_path: String,
    lead_time_metrics_path: String,
    density_metrics_path: String,
    per_failure_run_signals_path: String,
    dsa_metrics_path: String,
    dsa_run_signals_path: String,
    per_failure_run_dsa_signals_path: String,
    secom_archive_layout_path: String,
    drsc_trace_path: Option<String>,
    drsc_figure_path: Option<String>,
    report_markdown_path: String,
    report_tex_path: String,
    report_pdf_path: Option<String>,
    zip_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct BaselineComparisonSummary {
    dataset: String,
    secom_archive_layout_note: String,
    feature_count_used_by_crate: usize,
    failure_runs: usize,
    analyzable_feature_count: usize,
    lookback_runs: usize,
    failure_run_recall: FailureRunRecallSummary,
    pass_run_nuisance_proxy: PassRunNuisanceSummary,
    lead_time_summary: LeadTimeSummary,
    density_summary: DensitySummary,
    boundary_episode_summary: BoundaryEpisodeSummary,
    dsa_comparison_summary: Option<DsaVsBaselinesSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct FailureRunRecallSummary {
    dsfb_raw_signal: usize,
    dsfb_persistent_signal: usize,
    dsfb_raw_boundary_signal: usize,
    dsfb_persistent_boundary_signal: usize,
    dsfb_raw_violation_signal: usize,
    dsfb_persistent_violation_signal: usize,
    dsfb_dsa_signal: usize,
    ewma_signal: usize,
    threshold_signal: usize,
}

#[derive(Debug, Clone, Serialize)]
struct PassRunNuisanceSummary {
    dsfb_raw_boundary_signal_runs: usize,
    dsfb_persistent_boundary_signal_runs: usize,
    dsfb_raw_violation_signal_runs: usize,
    dsfb_persistent_violation_signal_runs: usize,
    dsfb_dsa_signal_runs: usize,
    ewma_signal_runs: usize,
    threshold_signal_runs: usize,
    dsfb_raw_boundary_signal_rate: f64,
    dsfb_persistent_boundary_signal_rate: f64,
    dsfb_raw_violation_signal_rate: f64,
    dsfb_persistent_violation_signal_rate: f64,
    dsfb_dsa_signal_rate: f64,
    ewma_signal_rate: f64,
    threshold_signal_rate: f64,
}

pub fn run_secom_benchmark(
    data_root: &Path,
    output_root: Option<&Path>,
    config: PipelineConfig,
    fetch_if_missing: bool,
) -> Result<SecomRunArtifacts> {
    config
        .validate()
        .map_err(DsfbSemiconductorError::DatasetFormat)?;

    let paths = if fetch_if_missing {
        secom::fetch_if_missing(data_root)?
    } else {
        secom::ensure_present(data_root)?
    };
    let secom_archive_layout = secom::inspect_archive_layout(&paths)?;
    let dataset = secom::load_from_root(data_root)?;
    let prepared = prepare_secom(&dataset, &config)?;
    let nominal = build_nominal_model(&prepared, &config);
    let residuals = compute_residuals(&prepared, &nominal);
    let signs = compute_signs(&prepared, &nominal, &residuals, &config);
    let baselines = compute_baselines(&prepared, &nominal, &residuals, &config);
    let grammar = evaluate_grammar(&residuals, &signs, &nominal, &config);
    let mut metrics = compute_metrics(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &config,
    );
    let dsa = evaluate_dsa(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &config.dsa,
        config.pre_failure_lookback_runs,
    )?;
    metrics.dsa_summary = Some(dsa.summary.clone());
    let heuristics = build_heuristics_bank(&metrics, "SECOM");

    let output_root = output_root
        .map(Path::to_path_buf)
        .unwrap_or_else(default_output_root);
    fs::create_dir_all(&output_root)?;
    let run_dir = create_timestamped_run_dir(&output_root, "secom")?;

    write_json_pretty(&run_dir.join("dataset_summary.json"), &prepared.summary)?;
    write_json_pretty(&run_dir.join("parameter_manifest.json"), &config)?;
    write_json_pretty(
        &run_dir.join("run_configuration.json"),
        &RunConfiguration {
            dataset: "SECOM".into(),
            config: config.clone(),
            data_root: data_root.display().to_string(),
            output_root: output_root.display().to_string(),
            secom_fetch_if_missing: fetch_if_missing,
        },
    )?;
    write_json_pretty(&run_dir.join("benchmark_metrics.json"), &metrics)?;
    write_json_pretty(
        &run_dir.join("secom_archive_layout.json"),
        &secom_archive_layout,
    )?;
    write_json_pretty(
        &run_dir.join("phm2018_support_status.json"),
        &phm_support_status(data_root),
    )?;
    write_json_pretty(&run_dir.join("heuristics_bank.json"), &heuristics)?;
    write_json_pretty(
        &run_dir.join("baseline_comparison_summary.json"),
        &build_baseline_comparison_summary(
            &metrics,
            &dsa,
            &secom_archive_layout,
            &config,
        ),
    )?;
    write_json_pretty(
        &run_dir.join("dsa_vs_baselines.json"),
        &dsa.comparison_summary,
    )?;
    write_json_pretty(
        &run_dir.join("dsa_parameter_manifest.json"),
        &dsa.parameter_manifest,
    )?;

    write_feature_metrics_csv(&run_dir.join("feature_metrics.csv"), &metrics)?;
    write_per_failure_run_signals_csv(
        &run_dir.join("per_failure_run_signals.csv"),
        &metrics.per_failure_run_signals,
    )?;
    write_dsa_metrics_csv(
        &run_dir.join("dsa_metrics.csv"),
        &prepared,
        &nominal,
        &dsa,
    )?;
    write_dsa_run_signals_csv(
        &run_dir.join("dsa_run_signals.csv"),
        &prepared,
        &dsa.run_signals,
    )?;
    write_per_failure_run_dsa_signals_csv(
        &run_dir.join("per_failure_run_dsa_signals.csv"),
        &dsa.per_failure_run_signals,
    )?;
    write_lead_time_metrics_csv(
        &run_dir.join("lead_time_metrics.csv"),
        &metrics.per_failure_run_signals,
    )?;
    write_density_metrics_csv(
        &run_dir.join("density_metrics.csv"),
        &metrics.density_metrics,
    )?;
    write_trace_csvs(
        &run_dir, &prepared, &residuals, &signs, &baselines, &grammar,
    )?;
    let figures = generate_figures(
        &run_dir,
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &baselines,
        &grammar,
        &metrics,
        &dsa,
        &config,
    )?;
    let report = write_reports(
        &run_dir,
        &config,
        &metrics,
        &dsa,
        &figures,
        &heuristics,
        &phm_support_status(data_root),
        &secom_archive_layout,
    )?;

    let manifest_path = run_dir.join("artifact_manifest.json");
    let metrics_path = run_dir.join("benchmark_metrics.json");
    let phm2018_status = phm_support_status(data_root);
    let zip_path = run_dir.join("run_bundle.zip");
    write_json_pretty(
        &manifest_path,
        &ArtifactManifest {
            dataset: "SECOM".into(),
            run_dir: run_dir.display().to_string(),
            metrics_summary_path: metrics_path.display().to_string(),
            baseline_comparison_summary_path: run_dir
                .join("baseline_comparison_summary.json")
                .display()
                .to_string(),
            dsa_vs_baselines_summary_path: run_dir
                .join("dsa_vs_baselines.json")
                .display()
                .to_string(),
            dsa_parameter_manifest_path: run_dir
                .join("dsa_parameter_manifest.json")
                .display()
                .to_string(),
            lead_time_metrics_path: run_dir.join("lead_time_metrics.csv").display().to_string(),
            density_metrics_path: run_dir.join("density_metrics.csv").display().to_string(),
            per_failure_run_signals_path: run_dir
                .join("per_failure_run_signals.csv")
                .display()
                .to_string(),
            dsa_metrics_path: run_dir.join("dsa_metrics.csv").display().to_string(),
            dsa_run_signals_path: run_dir
                .join("dsa_run_signals.csv")
                .display()
                .to_string(),
            per_failure_run_dsa_signals_path: run_dir
                .join("per_failure_run_dsa_signals.csv")
                .display()
                .to_string(),
            secom_archive_layout_path: run_dir
                .join("secom_archive_layout.json")
                .display()
                .to_string(),
            drsc_trace_path: figures
                .drsc
                .as_ref()
                .map(|drsc| run_dir.join(&drsc.trace_csv).display().to_string()),
            drsc_figure_path: figures.drsc.as_ref().map(|drsc| {
                run_dir
                    .join("figures")
                    .join(&drsc.figure_file)
                    .display()
                    .to_string()
            }),
            report_markdown_path: report.markdown_path.display().to_string(),
            report_tex_path: report.tex_path.display().to_string(),
            report_pdf_path: report
                .pdf_path
                .as_ref()
                .map(|path| path.display().to_string()),
            zip_path: zip_path.display().to_string(),
        },
    )?;
    zip_directory(&run_dir, &zip_path)?;

    Ok(SecomRunArtifacts {
        run_dir,
        report,
        figures,
        metrics_path,
        manifest_path,
        zip_path,
        phm2018_status,
    })
}

fn build_baseline_comparison_summary(
    metrics: &BenchmarkMetrics,
    dsa: &DsaEvaluation,
    secom_archive_layout: &SecomArchiveLayout,
    config: &PipelineConfig,
) -> BaselineComparisonSummary {
    BaselineComparisonSummary {
        dataset: "SECOM".into(),
        secom_archive_layout_note: secom_archive_layout.note.clone(),
        feature_count_used_by_crate: metrics.summary.dataset_summary.feature_count,
        failure_runs: metrics.summary.failure_runs,
        analyzable_feature_count: metrics.summary.analyzable_feature_count,
        lookback_runs: config.pre_failure_lookback_runs,
        failure_run_recall: FailureRunRecallSummary {
            dsfb_raw_signal: metrics.summary.failure_runs_with_preceding_dsfb_raw_signal,
            dsfb_persistent_signal: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_signal,
            dsfb_raw_boundary_signal: metrics
                .summary
                .failure_runs_with_preceding_dsfb_raw_boundary_signal,
            dsfb_persistent_boundary_signal: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_boundary_signal,
            dsfb_raw_violation_signal: metrics
                .summary
                .failure_runs_with_preceding_dsfb_raw_violation_signal,
            dsfb_persistent_violation_signal: metrics
                .summary
                .failure_runs_with_preceding_dsfb_persistent_violation_signal,
            dsfb_dsa_signal: dsa.summary.failure_run_recall,
            ewma_signal: metrics.summary.failure_runs_with_preceding_ewma_signal,
            threshold_signal: metrics.summary.failure_runs_with_preceding_threshold_signal,
        },
        pass_run_nuisance_proxy: PassRunNuisanceSummary {
            dsfb_raw_boundary_signal_runs: metrics.summary.pass_runs_with_dsfb_raw_boundary_signal,
            dsfb_persistent_boundary_signal_runs: metrics
                .summary
                .pass_runs_with_dsfb_persistent_boundary_signal,
            dsfb_raw_violation_signal_runs: metrics.summary.pass_runs_with_dsfb_raw_violation_signal,
            dsfb_persistent_violation_signal_runs: metrics
                .summary
                .pass_runs_with_dsfb_persistent_violation_signal,
            dsfb_dsa_signal_runs: (dsa.summary.pass_run_nuisance_proxy
                * metrics.summary.pass_runs as f64)
                .round() as usize,
            ewma_signal_runs: metrics.summary.pass_runs_with_ewma_signal,
            threshold_signal_runs: metrics.summary.pass_runs_with_threshold_signal,
            dsfb_raw_boundary_signal_rate: metrics.summary.pass_run_dsfb_raw_boundary_nuisance_rate,
            dsfb_persistent_boundary_signal_rate: metrics
                .summary
                .pass_run_dsfb_persistent_boundary_nuisance_rate,
            dsfb_raw_violation_signal_rate: metrics
                .summary
                .pass_run_dsfb_raw_violation_nuisance_rate,
            dsfb_persistent_violation_signal_rate: metrics
                .summary
                .pass_run_dsfb_persistent_violation_nuisance_rate,
            dsfb_dsa_signal_rate: dsa.summary.pass_run_nuisance_proxy,
            ewma_signal_rate: metrics.summary.pass_run_ewma_nuisance_rate,
            threshold_signal_rate: metrics.summary.pass_run_threshold_nuisance_rate,
        },
        lead_time_summary: metrics.lead_time_summary.clone(),
        density_summary: metrics.density_summary.clone(),
        boundary_episode_summary: metrics.boundary_episode_summary.clone(),
        dsa_comparison_summary: Some(dsa.comparison_summary.clone()),
    }
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, json)?;
    Ok(())
}

fn write_feature_metrics_csv(path: &Path, metrics: &BenchmarkMetrics) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for feature in &metrics.feature_metrics {
        writer.serialize(feature)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_per_failure_run_signals_csv(path: &Path, records: &[PerFailureRunSignal]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_dsa_metrics_csv(
    path: &Path,
    prepared: &crate::preprocessing::PreparedDataset,
    nominal: &crate::nominal::NominalModel,
    dsa: &DsaEvaluation,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "feature_index",
        "feature_name",
        "run_index",
        "timestamp",
        "label",
        "boundary_basis_hit",
        "drift_outward_hit",
        "slew_hit",
        "motif_hit",
        "boundary_density_W",
        "drift_persistence_W",
        "slew_density_W",
        "ewma_occupancy_W",
        "motif_recurrence_W",
        "consistent",
        "dsa_score",
        "dsa_active",
        "dsa_alert",
    ])?;

    for trace in &dsa.traces {
        if !nominal.features[trace.feature_index].analyzable {
            continue;
        }
        for run_index in 0..trace.dsa_score.len() {
            writer.write_record([
                trace.feature_index.to_string(),
                trace.feature_name.clone(),
                run_index.to_string(),
                prepared.timestamps[run_index]
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                prepared.labels[run_index].to_string(),
                trace.boundary_basis_hit[run_index].to_string(),
                trace.drift_outward_hit[run_index].to_string(),
                trace.slew_hit[run_index].to_string(),
                trace.motif_hit[run_index].to_string(),
                trace.boundary_density_w[run_index].to_string(),
                trace.drift_persistence_w[run_index].to_string(),
                trace.slew_density_w[run_index].to_string(),
                trace.ewma_occupancy_w[run_index].to_string(),
                trace.motif_recurrence_w[run_index].to_string(),
                trace.consistent[run_index].to_string(),
                trace.dsa_score[run_index].to_string(),
                trace.dsa_active[run_index].to_string(),
                trace.dsa_alert[run_index].to_string(),
            ])?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn write_dsa_run_signals_csv(
    path: &Path,
    prepared: &crate::preprocessing::PreparedDataset,
    run_signals: &DsaRunSignals,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "run_index",
        "timestamp",
        "label",
        "primary_run_signal",
        "any_feature_dsa_alert",
        "feature_count_dsa_alert",
    ])?;

    for run_index in 0..prepared.labels.len() {
        writer.write_record([
            run_index.to_string(),
            prepared.timestamps[run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            prepared.labels[run_index].to_string(),
            run_signals.primary_run_signal.clone(),
            run_signals.any_feature_dsa_alert[run_index].to_string(),
            run_signals.feature_count_dsa_alert[run_index].to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_per_failure_run_dsa_signals_csv(
    path: &Path,
    records: &[PerFailureRunDsaSignal],
) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_lead_time_metrics_csv(path: &Path, records: &[PerFailureRunSignal]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "failure_run_index",
        "failure_timestamp",
        "earliest_dsfb_raw_boundary_run",
        "earliest_dsfb_persistent_boundary_run",
        "earliest_dsfb_raw_violation_run",
        "earliest_dsfb_persistent_violation_run",
        "earliest_threshold_run",
        "earliest_ewma_run",
        "dsfb_raw_boundary_lead_runs",
        "dsfb_persistent_boundary_lead_runs",
        "dsfb_raw_violation_lead_runs",
        "dsfb_persistent_violation_lead_runs",
        "threshold_lead_runs",
        "ewma_lead_runs",
        "dsfb_raw_boundary_minus_threshold_delta_runs",
        "dsfb_raw_boundary_minus_ewma_delta_runs",
        "dsfb_persistent_boundary_minus_threshold_delta_runs",
        "dsfb_persistent_boundary_minus_ewma_delta_runs",
        "dsfb_raw_violation_minus_threshold_delta_runs",
        "dsfb_raw_violation_minus_ewma_delta_runs",
        "dsfb_persistent_violation_minus_threshold_delta_runs",
        "dsfb_persistent_violation_minus_ewma_delta_runs",
    ])?;

    for record in records {
        writer.write_record([
            record.failure_run_index.to_string(),
            record.failure_timestamp.clone(),
            option_to_string(record.earliest_dsfb_raw_boundary_run),
            option_to_string(record.earliest_dsfb_persistent_boundary_run),
            option_to_string(record.earliest_dsfb_raw_violation_run),
            option_to_string(record.earliest_dsfb_persistent_violation_run),
            option_to_string(record.earliest_threshold_run),
            option_to_string(record.earliest_ewma_run),
            option_to_string(record.dsfb_raw_boundary_lead_runs),
            option_to_string(record.dsfb_persistent_boundary_lead_runs),
            option_to_string(record.dsfb_raw_violation_lead_runs),
            option_to_string(record.dsfb_persistent_violation_lead_runs),
            option_to_string(record.threshold_lead_runs),
            option_to_string(record.ewma_lead_runs),
            option_to_string(record.dsfb_raw_boundary_minus_threshold_delta_runs),
            option_to_string(record.dsfb_raw_boundary_minus_ewma_delta_runs),
            option_to_string(record.dsfb_persistent_boundary_minus_threshold_delta_runs),
            option_to_string(record.dsfb_persistent_boundary_minus_ewma_delta_runs),
            option_to_string(record.dsfb_raw_violation_minus_threshold_delta_runs),
            option_to_string(record.dsfb_raw_violation_minus_ewma_delta_runs),
            option_to_string(record.dsfb_persistent_violation_minus_threshold_delta_runs),
            option_to_string(record.dsfb_persistent_violation_minus_ewma_delta_runs),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_density_metrics_csv(path: &Path, records: &[DensityMetricRecord]) -> Result<()> {
    let mut writer = csv::Writer::from_path(path)?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_trace_csvs(
    run_dir: &Path,
    prepared: &crate::preprocessing::PreparedDataset,
    residuals: &crate::residual::ResidualSet,
    signs: &crate::signs::SignSet,
    baselines: &crate::baselines::BaselineSet,
    grammar: &crate::grammar::GrammarSet,
) -> Result<()> {
    let mut residual_writer = csv::Writer::from_path(run_dir.join("residuals.csv"))?;
    let mut drift_writer = csv::Writer::from_path(run_dir.join("drifts.csv"))?;
    let mut slew_writer = csv::Writer::from_path(run_dir.join("slews.csv"))?;
    let mut ewma_writer = csv::Writer::from_path(run_dir.join("ewma_baseline.csv"))?;
    let mut grammar_writer = csv::Writer::from_path(run_dir.join("grammar_states.csv"))?;

    residual_writer.write_record([
        "run_index",
        "timestamp",
        "label",
        "feature",
        "imputed_value",
        "residual",
        "residual_norm",
        "threshold_alarm",
    ])?;
    drift_writer.write_record(["run_index", "timestamp", "feature", "drift"])?;
    slew_writer.write_record(["run_index", "timestamp", "feature", "slew"])?;
    ewma_writer.write_record([
        "run_index",
        "timestamp",
        "feature",
        "ewma",
        "healthy_mean",
        "healthy_std",
        "threshold",
        "alarm",
    ])?;
    grammar_writer.write_record([
        "run_index",
        "timestamp",
        "feature",
        "raw_state",
        "confirmed_state",
        "persistent_boundary",
        "persistent_violation",
        "raw_reason",
        "confirmed_reason",
    ])?;

    for feature_index in 0..residuals.traces.len() {
        let residual_trace = &residuals.traces[feature_index];
        let sign_trace = &signs.traces[feature_index];
        let ewma_trace = &baselines.ewma[feature_index];
        let grammar_trace = &grammar.traces[feature_index];
        for run_index in 0..prepared.timestamps.len() {
            let timestamp = prepared.timestamps[run_index]
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            residual_writer.write_record([
                run_index.to_string(),
                timestamp.clone(),
                prepared.labels[run_index].to_string(),
                residual_trace.feature_name.clone(),
                residual_trace.imputed_values[run_index].to_string(),
                residual_trace.residuals[run_index].to_string(),
                residual_trace.norms[run_index].to_string(),
                residual_trace.threshold_alarm[run_index].to_string(),
            ])?;
            drift_writer.write_record([
                run_index.to_string(),
                timestamp.clone(),
                residual_trace.feature_name.clone(),
                sign_trace.drift[run_index].to_string(),
            ])?;
            slew_writer.write_record([
                run_index.to_string(),
                timestamp.clone(),
                residual_trace.feature_name.clone(),
                sign_trace.slew[run_index].to_string(),
            ])?;
            ewma_writer.write_record([
                run_index.to_string(),
                timestamp.clone(),
                residual_trace.feature_name.clone(),
                ewma_trace.ewma[run_index].to_string(),
                ewma_trace.healthy_mean.to_string(),
                ewma_trace.healthy_std.to_string(),
                ewma_trace.threshold.to_string(),
                ewma_trace.alarm[run_index].to_string(),
            ])?;
            grammar_writer.write_record([
                run_index.to_string(),
                timestamp,
                residual_trace.feature_name.clone(),
                format!("{:?}", grammar_trace.raw_states[run_index]),
                format!("{:?}", grammar_trace.states[run_index]),
                grammar_trace.persistent_boundary[run_index].to_string(),
                grammar_trace.persistent_violation[run_index].to_string(),
                format!("{:?}", grammar_trace.raw_reasons[run_index]),
                format!("{:?}", grammar_trace.reasons[run_index]),
            ])?;
        }
    }

    residual_writer.flush()?;
    drift_writer.flush()?;
    slew_writer.flush()?;
    ewma_writer.flush()?;
    grammar_writer.flush()?;
    Ok(())
}

fn option_to_string<T: ToString>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn zip_directory(run_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    add_directory_contents(&mut zip, run_dir, run_dir, zip_path, options)?;
    zip.finish()?;
    Ok(())
}

fn add_directory_contents(
    zip: &mut zip::ZipWriter<File>,
    root: &Path,
    current: &Path,
    zip_path: &Path,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path == zip_path {
            continue;
        }
        if path.is_dir() {
            add_directory_contents(zip, root, &path, zip_path, options)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|err| DsfbSemiconductorError::DatasetFormat(err.to_string()))?;
            zip.start_file(relative.to_string_lossy().replace('\\', "/"), options)?;
            let bytes = fs::read(&path)?;
            zip.write_all(&bytes)?;
        }
    }
    Ok(())
}
