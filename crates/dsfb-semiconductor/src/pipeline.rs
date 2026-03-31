use crate::config::{PipelineConfig, RunConfiguration};
use crate::dataset::phm2018::{support_status as phm_support_status, Phm2018SupportStatus};
use crate::dataset::secom;
use crate::error::{DsfbSemiconductorError, Result};
use crate::grammar::evaluate_grammar;
use crate::heuristics::build_heuristics_bank;
use crate::metrics::{compute_metrics, BenchmarkMetrics};
use crate::nominal::build_nominal_model;
use crate::output_paths::{create_timestamped_run_dir, default_output_root};
use crate::plots::{generate_figures, FigureManifest};
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
    report_markdown_path: String,
    report_tex_path: String,
    report_pdf_path: Option<String>,
    zip_path: String,
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
    let grammar = evaluate_grammar(&residuals, &signs, &nominal, &config);
    let metrics = compute_metrics(
        &prepared,
        &nominal,
        &residuals,
        &signs,
        &grammar,
        config.pre_failure_lookback_runs,
    );
    let heuristics = build_heuristics_bank(&grammar, "SECOM");

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
    write_json_pretty(&run_dir.join("phm2018_support_status.json"), &phm_support_status(data_root))?;
    write_json_pretty(&run_dir.join("heuristics_bank.json"), &heuristics)?;

    write_feature_metrics_csv(&run_dir.join("feature_metrics.csv"), &metrics)?;
    write_trace_csvs(&run_dir, &prepared, &residuals, &signs, &grammar)?;
    let figures = generate_figures(&run_dir, &prepared, &nominal, &residuals, &signs, &grammar, &metrics)?;
    let report = write_reports(
        &run_dir,
        &config,
        &metrics,
        &figures,
        &heuristics,
        &phm_support_status(data_root),
    )?;

    let zip_path = run_dir.join("run_bundle.zip");
    zip_directory(&run_dir, &zip_path)?;

    let manifest_path = run_dir.join("artifact_manifest.json");
    let metrics_path = run_dir.join("benchmark_metrics.json");
    let phm2018_status = phm_support_status(data_root);
    write_json_pretty(
        &manifest_path,
        &ArtifactManifest {
            dataset: "SECOM".into(),
            run_dir: run_dir.display().to_string(),
            metrics_summary_path: metrics_path.display().to_string(),
            report_markdown_path: report.markdown_path.display().to_string(),
            report_tex_path: report.tex_path.display().to_string(),
            report_pdf_path: report.pdf_path.as_ref().map(|path| path.display().to_string()),
            zip_path: zip_path.display().to_string(),
        },
    )?;

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

fn write_trace_csvs(
    run_dir: &Path,
    prepared: &crate::preprocessing::PreparedDataset,
    residuals: &crate::residual::ResidualSet,
    signs: &crate::signs::SignSet,
    grammar: &crate::grammar::GrammarSet,
) -> Result<()> {
    let mut residual_writer = csv::Writer::from_path(run_dir.join("residuals.csv"))?;
    let mut drift_writer = csv::Writer::from_path(run_dir.join("drifts.csv"))?;
    let mut slew_writer = csv::Writer::from_path(run_dir.join("slews.csv"))?;
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
    grammar_writer.write_record([
        "run_index",
        "timestamp",
        "feature",
        "state",
        "reason",
    ])?;

    for feature_index in 0..residuals.traces.len() {
        let residual_trace = &residuals.traces[feature_index];
        let sign_trace = &signs.traces[feature_index];
        let grammar_trace = &grammar.traces[feature_index];
        for run_index in 0..prepared.timestamps.len() {
            let timestamp = prepared.timestamps[run_index].format("%Y-%m-%d %H:%M:%S").to_string();
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
            grammar_writer.write_record([
                run_index.to_string(),
                timestamp,
                residual_trace.feature_name.clone(),
                format!("{:?}", grammar_trace.states[run_index]),
                format!("{:?}", grammar_trace.reasons[run_index]),
            ])?;
        }
    }

    residual_writer.flush()?;
    drift_writer.flush()?;
    slew_writer.flush()?;
    grammar_writer.flush()?;
    Ok(())
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
