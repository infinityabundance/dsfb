use crate::calibration::{run_secom_calibration, CalibrationGrid};
use crate::config::PipelineConfig;
use crate::dataset::phm2018;
use crate::dataset::secom;
use crate::error::Result;
use crate::output_paths::{default_data_root, default_output_root};
use crate::pipeline::run_secom_benchmark;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "dsfb-semiconductor")]
#[command(about = "DSFB semiconductor benchmark companion")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    FetchSecom(DataArgs),
    RunSecom(RunSecomArgs),
    CalibrateSecom(CalibrateSecomArgs),
    ProbePhm2018(ProbePhm2018Args),
}

#[derive(Debug, Args)]
struct DataArgs {
    #[arg(long)]
    data_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunSecomArgs {
    #[command(flatten)]
    data: DataArgs,
    #[arg(long)]
    output_root: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    fetch_if_missing: bool,
    #[arg(long, default_value_t = 100)]
    healthy_pass_runs: usize,
    #[arg(long, default_value_t = 5)]
    drift_window: usize,
    #[arg(long, default_value_t = 3.0)]
    envelope_sigma: f64,
    #[arg(long, default_value_t = 0.5)]
    boundary_fraction_of_rho: f64,
    #[arg(long, default_value_t = 0.2)]
    ewma_alpha: f64,
    #[arg(long, default_value_t = 3.0)]
    ewma_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    drift_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    slew_sigma_multiplier: f64,
    #[arg(long, default_value_t = 10)]
    grazing_window: usize,
    #[arg(long, default_value_t = 3)]
    grazing_min_hits: usize,
    #[arg(long, default_value_t = 20)]
    pre_failure_lookback_runs: usize,
}

#[derive(Debug, Args)]
struct CalibrateSecomArgs {
    #[command(flatten)]
    data: DataArgs,
    #[arg(long)]
    output_root: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    fetch_if_missing: bool,
    #[arg(long, value_delimiter = ',', default_value = "100")]
    healthy_pass_runs_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "5")]
    drift_window_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    envelope_sigma_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "0.5")]
    boundary_fraction_of_rho_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "0.2")]
    ewma_alpha_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    ewma_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    drift_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    slew_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "10")]
    grazing_window_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "3")]
    grazing_min_hits_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "20")]
    pre_failure_lookback_runs_grid: Vec<usize>,
}

#[derive(Debug, Args)]
struct ProbePhm2018Args {
    #[arg(long)]
    archive: Option<PathBuf>,
    #[arg(long)]
    data_root: Option<PathBuf>,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::FetchSecom(args) => {
            let data_root = args.data_root.unwrap_or_else(default_data_root);
            let paths = secom::fetch_if_missing(&data_root)?;
            println!("SECOM dataset ready at {}", paths.root.display());
            println!("Archive: {}", paths.archive.display());
            Ok(())
        }
        Command::RunSecom(args) => {
            let data_root = args.data.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);
            let config = PipelineConfig {
                healthy_pass_runs: args.healthy_pass_runs,
                drift_window: args.drift_window,
                envelope_sigma: args.envelope_sigma,
                boundary_fraction_of_rho: args.boundary_fraction_of_rho,
                ewma_alpha: args.ewma_alpha,
                ewma_sigma_multiplier: args.ewma_sigma_multiplier,
                drift_sigma_multiplier: args.drift_sigma_multiplier,
                slew_sigma_multiplier: args.slew_sigma_multiplier,
                grazing_window: args.grazing_window,
                grazing_min_hits: args.grazing_min_hits,
                pre_failure_lookback_runs: args.pre_failure_lookback_runs,
                ..PipelineConfig::default()
            };
            let artifacts = run_secom_benchmark(
                &data_root,
                Some(&output_root),
                config,
                args.fetch_if_missing,
            )?;
            println!("Run directory: {}", artifacts.run_dir.display());
            println!("Metrics: {}", artifacts.metrics_path.display());
            if let Some(pdf) = artifacts.report.pdf_path {
                println!("PDF report: {}", pdf.display());
            } else if let Some(error) = artifacts.report.pdf_error {
                println!(
                    "PDF report failed: {}",
                    error.lines().next().unwrap_or("unknown error")
                );
            }
            println!("ZIP bundle: {}", artifacts.zip_path.display());
            Ok(())
        }
        Command::CalibrateSecom(args) => {
            let data_root = args.data.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);
            let grid = CalibrationGrid {
                healthy_pass_runs: args.healthy_pass_runs_grid,
                drift_window: args.drift_window_grid,
                envelope_sigma: args.envelope_sigma_grid,
                boundary_fraction_of_rho: args.boundary_fraction_of_rho_grid,
                ewma_alpha: args.ewma_alpha_grid,
                ewma_sigma_multiplier: args.ewma_sigma_multiplier_grid,
                drift_sigma_multiplier: args.drift_sigma_multiplier_grid,
                slew_sigma_multiplier: args.slew_sigma_multiplier_grid,
                grazing_window: args.grazing_window_grid,
                grazing_min_hits: args.grazing_min_hits_grid,
                pre_failure_lookback_runs: args.pre_failure_lookback_runs_grid,
            };
            let artifacts =
                run_secom_calibration(&data_root, Some(&output_root), grid, args.fetch_if_missing)?;
            println!("Calibration run directory: {}", artifacts.run_dir.display());
            println!(
                "Calibration grid results: {}",
                artifacts.grid_results_csv.display()
            );
            println!("Calibration summary: {}", artifacts.summary_json.display());
            println!(
                "Calibration report: {}",
                artifacts.report_markdown.display()
            );
            Ok(())
        }
        Command::ProbePhm2018(args) => {
            let data_root = args.data_root.unwrap_or_else(default_data_root);
            let status = phm2018::support_status(&data_root);
            println!(
                "PHM 2018 manual archive path: {}",
                status.manual_placement_path.display()
            );
            println!("Official page: {}", status.official_page);
            println!("Official link: {}", status.official_download_link);
            println!("Implemented now: {}", status.fully_implemented);
            println!("Blocker: {}", status.blocker);
            if let Some(archive) = args.archive {
                let manifest = phm2018::inspect_archive(&archive)?;
                println!("{}", serde_json::to_string_pretty(&manifest)?);
            }
            Ok(())
        }
    }
}
