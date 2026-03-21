#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::dashboard::{CsvReplayDriver, DashboardReplayConfig};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingSettings};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::io::json::write_pretty;
use dsfb_semiotics_engine::public_dataset::{
    clear_dir, ensure_dir, find_first_png, mirror_directory, write_replay_artifacts,
    PublicDatasetArtifactSummary, PublicDatasetKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum PublicDatasetPhase {
    Fetch,
    Preprocess,
    Run,
    All,
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Fetch, preprocess, run, and replay the real NASA public dataset demos"
)]
struct Args {
    #[arg(long = "dataset", value_enum)]
    datasets: Vec<PublicDatasetKind>,

    #[arg(long, value_enum, default_value_t = PublicDatasetPhase::All)]
    phase: PublicDatasetPhase,

    #[arg(long)]
    force_download: bool,

    #[arg(long)]
    force_regenerate: bool,

    #[arg(long)]
    dashboard_max_frames: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let datasets = if args.datasets.is_empty() {
        PublicDatasetKind::all().to_vec()
    } else {
        args.datasets.clone()
    };

    for dataset in datasets {
        if matches!(
            args.phase,
            PublicDatasetPhase::Fetch | PublicDatasetPhase::All
        ) {
            run_python_script(
                dataset.tools_fetch_script(),
                dataset,
                args.force_download,
                args.force_regenerate,
            )?;
        }
        if matches!(
            args.phase,
            PublicDatasetPhase::Preprocess | PublicDatasetPhase::All
        ) {
            run_python_script(
                dataset.tools_preprocess_script(),
                dataset,
                false,
                args.force_regenerate,
            )?;
        }
        if matches!(
            args.phase,
            PublicDatasetPhase::Run | PublicDatasetPhase::All
        ) {
            let summary = run_dataset_demo(dataset, args.dashboard_max_frames)?;
            println!(
                "dataset={} latest_dir={} sample_dir={} report_pdf={} zip_archive={}",
                dataset.as_slug(),
                dataset.latest_root().display(),
                dataset.sample_root().display(),
                summary.report_pdf,
                summary.zip_archive
            );
        }
    }

    Ok(())
}

fn run_python_script(
    script: PathBuf,
    dataset: PublicDatasetKind,
    force_download: bool,
    force_regenerate: bool,
) -> Result<()> {
    let mut command = Command::new("python3");
    command.arg(&script).arg("--dataset").arg(dataset.as_slug());
    if force_download {
        command.arg("--force-download");
    }
    if force_regenerate {
        command.arg("--force-regenerate");
    }
    let status = command
        .status()
        .with_context(|| format!("failed to execute {}", script.display()))?;
    if !status.success() {
        return Err(anyhow!(
            "{} exited with status {:?}",
            script.display(),
            status.code()
        ));
    }
    Ok(())
}

fn run_dataset_demo(
    dataset: PublicDatasetKind,
    dashboard_max_frames: Option<usize>,
) -> Result<PublicDatasetArtifactSummary> {
    ensure_processed_inputs(dataset)?;

    let input = CsvInputConfig {
        observed_csv: dataset.processed_observed_path(),
        predicted_csv: dataset.processed_predicted_path(),
        scenario_id: dataset.scenario_id().to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 1.0,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: envelope_base(dataset),
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: format!("{}_envelope", dataset.as_slug()),
    };
    let common = CommonRunConfig {
        output_root: Some(dataset.generated_root()),
        ..Default::default()
    };
    let settings = EngineSettings {
        smoothing: SmoothingSettings::safety_first(),
        ..EngineSettings::default()
    };

    let engine = StructuralSemioticsEngine::with_settings(
        EngineConfig::csv(common.clone(), input.clone()),
        settings,
    )?;
    let bundle = engine.run_selected()?;
    let exported = export_artifacts(&bundle)?;

    let replay = CsvReplayDriver::from_bundle_and_csv_input(
        &bundle,
        &common,
        &input,
        &EngineSettings {
            smoothing: SmoothingSettings::safety_first(),
            ..EngineSettings::default()
        },
        DashboardReplayConfig {
            max_frames: dashboard_max_frames,
            source_label: Some(dataset.as_label().to_string()),
            ..Default::default()
        },
    )?;
    let replay_ascii = replay.render_replay_ascii();
    let replay_dir = exported.run_dir.join("replay");
    let replay_paths = write_replay_artifacts(replay.stream(), &replay_ascii, &replay_dir)?;
    let replay_inputs_dir = exported.run_dir.join("replay_inputs");
    ensure_dir(&replay_inputs_dir)?;
    copy_file(
        &dataset.processed_observed_path(),
        &replay_inputs_dir.join("observed.csv"),
    )?;
    copy_file(
        &dataset.processed_predicted_path(),
        &replay_inputs_dir.join("predicted.csv"),
    )?;
    copy_file(
        &dataset.processed_metadata_path(),
        &replay_inputs_dir.join("metadata.json"),
    )?;
    copy_file(
        &dataset.raw_summary_path(),
        &replay_inputs_dir.join("raw_summary.csv"),
    )?;

    let first_png = find_first_png(&exported.run_dir)?;
    let summary = PublicDatasetArtifactSummary {
        schema_version: "dsfb-semiotics-public-dataset-demo/v1".to_string(),
        dataset: dataset.as_slug().to_string(),
        dataset_label: dataset.as_label().to_string(),
        source_url: dataset.source_url().to_string(),
        source_archive: display_relative(&dataset.source_archive_path()),
        raw_summary_csv: display_relative(&dataset.raw_summary_path()),
        processed_observed_csv: display_relative(&dataset.processed_observed_path()),
        processed_predicted_csv: display_relative(&dataset.processed_predicted_path()),
        replay_events_csv: replay_paths.replay_events_csv.display().to_string(),
        replay_events_json: replay_paths.replay_events_json.display().to_string(),
        replay_ascii: replay_paths.replay_ascii.display().to_string(),
        manifest_json: exported.manifest_path.display().to_string(),
        report_pdf: exported.report_pdf.display().to_string(),
        zip_archive: exported.zip_path.display().to_string(),
        first_png: first_png.as_ref().map(|path| path.display().to_string()),
    };
    write_pretty(
        &exported.run_dir.join("public_dataset_demo_summary.json"),
        &summary,
    )?;

    mirror_directory(&exported.run_dir, &dataset.latest_root())?;
    sync_sample_subset(
        &exported.run_dir,
        &dataset.sample_root(),
        first_png.as_deref(),
    )?;

    Ok(summary)
}

fn ensure_processed_inputs(dataset: PublicDatasetKind) -> Result<()> {
    for required in [
        dataset.raw_summary_path(),
        dataset.processed_observed_path(),
        dataset.processed_predicted_path(),
        dataset.processed_metadata_path(),
    ] {
        if !required.is_file() {
            return Err(anyhow!(
                "public dataset demo requires {}; run the fetch and preprocess phases first",
                required.display()
            ));
        }
    }
    Ok(())
}

fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    let Some(parent) = dst.parent() else {
        return Err(anyhow!("{} has no parent directory", dst.display()));
    };
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

fn envelope_base(dataset: PublicDatasetKind) -> f64 {
    match dataset {
        PublicDatasetKind::NasaMilling => 0.30,
        PublicDatasetKind::NasaBearings => 0.35,
    }
}

fn display_relative(path: &Path) -> String {
    let crate_root = PublicDatasetKind::crate_root();
    match path.strip_prefix(&crate_root) {
        Ok(relative) => relative.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

fn sync_sample_subset(run_dir: &Path, sample_root: &Path, first_png: Option<&Path>) -> Result<()> {
    clear_dir(sample_root)?;
    copy_file(
        &run_dir.join("manifest.json"),
        &sample_root.join("manifest.json"),
    )?;
    copy_file(
        &run_dir.join("public_dataset_demo_summary.json"),
        &sample_root.join("public_dataset_demo_summary.json"),
    )?;
    copy_file(
        &run_dir.join("report/dsfb_semiotics_engine_report.pdf"),
        &sample_root.join("report/dsfb_semiotics_engine_report.pdf"),
    )?;
    copy_file(
        &run_dir.join("report/dsfb_semiotics_engine_report.md"),
        &sample_root.join("report/dsfb_semiotics_engine_report.md"),
    )?;
    copy_file(
        &run_dir.join("replay/replay_events.csv"),
        &sample_root.join("replay/replay_events.csv"),
    )?;
    copy_file(
        &run_dir.join("replay/replay_ascii.txt"),
        &sample_root.join("replay/replay_ascii.txt"),
    )?;
    copy_file(
        &run_dir.join("replay_inputs/observed.csv"),
        &sample_root.join("replay_inputs/observed.csv"),
    )?;
    copy_file(
        &run_dir.join("replay_inputs/predicted.csv"),
        &sample_root.join("replay_inputs/predicted.csv"),
    )?;
    copy_file(
        &run_dir.join("replay_inputs/raw_summary.csv"),
        &sample_root.join("replay_inputs/raw_summary.csv"),
    )?;
    copy_file(
        &run_dir.join("replay_inputs/metadata.json"),
        &sample_root.join("replay_inputs/metadata.json"),
    )?;
    if let Some(first_png) = first_png {
        copy_file(
            first_png,
            &sample_root.join("figures").join(
                first_png
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("figure.png")),
            ),
        )?;
    }
    Ok(())
}
