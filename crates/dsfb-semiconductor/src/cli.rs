use crate::calibration::{run_secom_calibration, run_secom_dsa_calibration, CalibrationGrid};
use crate::config::PipelineConfig;
use crate::dataset::phm2018;
use crate::dataset::secom;
use crate::error::Result;
use crate::non_intrusive::materialize_non_intrusive_artifacts;
use crate::output_paths::{default_data_root, default_output_root};
use crate::phm2018_loader::run_phm2018_benchmark;
use crate::pipeline::{run_secom_benchmark, PaperLockMetrics};
use crate::unified_value_figure::{render_unified_value_figure, resolve_latest_completed_run};
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
    CalibrateSecomDsa(CalibrateSecomDsaArgs),
    ProbePhm2018(ProbePhm2018Args),
    RunPhm2018(RunPhm2018Args),
    RenderNonIntrusiveArtifacts(RenderNonIntrusiveArtifactsArgs),
    RenderUnifiedValueFigure(RenderUnifiedValueFigureArgs),
    SbirDemo(SbirDemoArgs),
    /// Verify that the crate reproduces the paper headline numbers.
    ///
    /// Runs the SECOM benchmark with the fixed paper-lock configuration
    /// (all_features [compression_biased], W=5, K=20, tau=2.0, m=2) and
    /// checks that:
    ///   - episode count  == 71
    ///   - precision      >= 0.80 (paper value: 80.3 %)
    ///   - recall count   == 104 / 104
    ///
    /// Exits 0 on match, 1 on mismatch.
    PaperLock(PaperLockArgs),
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
    #[arg(long, default_value_t = 2)]
    state_confirmation_steps: usize,
    #[arg(long, default_value_t = 2)]
    persistent_state_steps: usize,
    #[arg(long, default_value_t = 10)]
    density_window: usize,
    #[arg(long, default_value_t = 0.2)]
    ewma_alpha: f64,
    #[arg(long, default_value_t = 3.0)]
    ewma_sigma_multiplier: f64,
    #[arg(long, default_value_t = 0.5)]
    cusum_kappa_sigma_multiplier: f64,
    #[arg(long, default_value_t = 5.0)]
    cusum_alarm_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    run_energy_sigma_multiplier: f64,
    #[arg(long, default_value_t = 0.95)]
    pca_variance_explained: f64,
    #[arg(long, default_value_t = 3.0)]
    pca_t2_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    pca_spe_sigma_multiplier: f64,
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
    #[arg(long, default_value_t = 5)]
    dsa_window: usize,
    #[arg(long, default_value_t = 2)]
    dsa_persistence_runs: usize,
    #[arg(long, default_value_t = 2.0)]
    dsa_alert_tau: f64,
    #[arg(long, default_value_t = 2)]
    dsa_corroborating_feature_count_min: usize,
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
    #[arg(long, value_delimiter = ',', default_value = "2")]
    state_confirmation_steps_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "2")]
    persistent_state_steps_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "10")]
    density_window_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "0.2")]
    ewma_alpha_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    ewma_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "0.5")]
    cusum_kappa_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "5.0")]
    cusum_alarm_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    run_energy_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "0.95")]
    pca_variance_explained_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    pca_t2_sigma_multiplier_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "3.0")]
    pca_spe_sigma_multiplier_grid: Vec<f64>,
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
struct CalibrateSecomDsaArgs {
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
    #[arg(long, default_value_t = 2)]
    state_confirmation_steps: usize,
    #[arg(long, default_value_t = 2)]
    persistent_state_steps: usize,
    #[arg(long, default_value_t = 10)]
    density_window: usize,
    #[arg(long, default_value_t = 0.2)]
    ewma_alpha: f64,
    #[arg(long, default_value_t = 3.0)]
    ewma_sigma_multiplier: f64,
    #[arg(long, default_value_t = 0.5)]
    cusum_kappa_sigma_multiplier: f64,
    #[arg(long, default_value_t = 5.0)]
    cusum_alarm_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    run_energy_sigma_multiplier: f64,
    #[arg(long, default_value_t = 0.95)]
    pca_variance_explained: f64,
    #[arg(long, default_value_t = 3.0)]
    pca_t2_sigma_multiplier: f64,
    #[arg(long, default_value_t = 3.0)]
    pca_spe_sigma_multiplier: f64,
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
    #[arg(long, value_delimiter = ',', default_value = "5,10,15")]
    dsa_window_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "2,3,4")]
    dsa_persistence_runs_grid: Vec<usize>,
    #[arg(long, value_delimiter = ',', default_value = "2.0,2.5,3.0")]
    dsa_alert_tau_grid: Vec<f64>,
    #[arg(long, value_delimiter = ',', default_value = "2,3,5")]
    dsa_corroborating_feature_count_min_grid: Vec<usize>,
}

#[derive(Debug, Args)]
struct ProbePhm2018Args {
    #[arg(long)]
    archive: Option<PathBuf>,
    #[arg(long)]
    data_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunPhm2018Args {
    #[arg(long)]
    data_root: Option<PathBuf>,
    #[arg(long)]
    output_root: Option<PathBuf>,
    #[arg(long)]
    secom_run_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct SbirDemoArgs {
    /// Optional data root; defaults to the crate-local data directory.
    #[arg(long)]
    data_root: Option<PathBuf>,
    /// Optional output root; defaults to the crate-local output directory.
    #[arg(long)]
    output_root: Option<PathBuf>,
    /// Fetch SECOM dataset automatically if absent (default: true).
    #[arg(long, default_value_t = true)]
    fetch_if_missing: bool,
}

#[derive(Debug, Args)]
struct PaperLockArgs {
    /// Optional data root; defaults to the crate-local data directory.
    #[arg(long)]
    data_root: Option<PathBuf>,
    /// Optional output root; defaults to the crate-local output directory.
    #[arg(long)]
    output_root: Option<PathBuf>,
    /// Fetch SECOM dataset automatically if absent (default: true).
    #[arg(long, default_value_t = true)]
    fetch_if_missing: bool,
}

#[derive(Debug, Args)]
struct RenderNonIntrusiveArtifactsArgs {
    #[arg(long)]
    run_dir: PathBuf,
}

#[derive(Debug, Args)]
struct RenderUnifiedValueFigureArgs {
    #[arg(long)]
    secom_run_dir: Option<PathBuf>,
    #[arg(long)]
    phm_run_dir: Option<PathBuf>,
    #[arg(long)]
    output_root: Option<PathBuf>,
    #[arg(long)]
    paper_tex: Option<PathBuf>,
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
                state_confirmation_steps: args.state_confirmation_steps,
                persistent_state_steps: args.persistent_state_steps,
                density_window: args.density_window,
                ewma_alpha: args.ewma_alpha,
                ewma_sigma_multiplier: args.ewma_sigma_multiplier,
                cusum_kappa_sigma_multiplier: args.cusum_kappa_sigma_multiplier,
                cusum_alarm_sigma_multiplier: args.cusum_alarm_sigma_multiplier,
                run_energy_sigma_multiplier: args.run_energy_sigma_multiplier,
                pca_variance_explained: args.pca_variance_explained,
                pca_t2_sigma_multiplier: args.pca_t2_sigma_multiplier,
                pca_spe_sigma_multiplier: args.pca_spe_sigma_multiplier,
                drift_sigma_multiplier: args.drift_sigma_multiplier,
                slew_sigma_multiplier: args.slew_sigma_multiplier,
                grazing_window: args.grazing_window,
                grazing_min_hits: args.grazing_min_hits,
                pre_failure_lookback_runs: args.pre_failure_lookback_runs,
                dsa: crate::precursor::DsaConfig {
                    window: args.dsa_window,
                    persistence_runs: args.dsa_persistence_runs,
                    alert_tau: args.dsa_alert_tau,
                    corroborating_feature_count_min: args.dsa_corroborating_feature_count_min,
                },
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
                state_confirmation_steps: args.state_confirmation_steps_grid,
                persistent_state_steps: args.persistent_state_steps_grid,
                density_window: args.density_window_grid,
                ewma_alpha: args.ewma_alpha_grid,
                ewma_sigma_multiplier: args.ewma_sigma_multiplier_grid,
                cusum_kappa_sigma_multiplier: args.cusum_kappa_sigma_multiplier_grid,
                cusum_alarm_sigma_multiplier: args.cusum_alarm_sigma_multiplier_grid,
                run_energy_sigma_multiplier: args.run_energy_sigma_multiplier_grid,
                pca_variance_explained: args.pca_variance_explained_grid,
                pca_t2_sigma_multiplier: args.pca_t2_sigma_multiplier_grid,
                pca_spe_sigma_multiplier: args.pca_spe_sigma_multiplier_grid,
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
        Command::CalibrateSecomDsa(args) => {
            let data_root = args.data.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);
            let config = PipelineConfig {
                healthy_pass_runs: args.healthy_pass_runs,
                drift_window: args.drift_window,
                envelope_sigma: args.envelope_sigma,
                boundary_fraction_of_rho: args.boundary_fraction_of_rho,
                state_confirmation_steps: args.state_confirmation_steps,
                persistent_state_steps: args.persistent_state_steps,
                density_window: args.density_window,
                ewma_alpha: args.ewma_alpha,
                ewma_sigma_multiplier: args.ewma_sigma_multiplier,
                cusum_kappa_sigma_multiplier: args.cusum_kappa_sigma_multiplier,
                cusum_alarm_sigma_multiplier: args.cusum_alarm_sigma_multiplier,
                run_energy_sigma_multiplier: args.run_energy_sigma_multiplier,
                pca_variance_explained: args.pca_variance_explained,
                pca_t2_sigma_multiplier: args.pca_t2_sigma_multiplier,
                pca_spe_sigma_multiplier: args.pca_spe_sigma_multiplier,
                drift_sigma_multiplier: args.drift_sigma_multiplier,
                slew_sigma_multiplier: args.slew_sigma_multiplier,
                grazing_window: args.grazing_window,
                grazing_min_hits: args.grazing_min_hits,
                pre_failure_lookback_runs: args.pre_failure_lookback_runs,
                ..PipelineConfig::default()
            };
            let artifacts = run_secom_dsa_calibration(
                &data_root,
                Some(&output_root),
                config,
                crate::precursor::DsaCalibrationGrid {
                    window: args.dsa_window_grid,
                    persistence_runs: args.dsa_persistence_runs_grid,
                    alert_tau: args.dsa_alert_tau_grid,
                    corroborating_feature_count_min: args.dsa_corroborating_feature_count_min_grid,
                },
                args.fetch_if_missing,
            )?;
            println!(
                "DSA calibration run directory: {}",
                artifacts.run_dir.display()
            );
            println!(
                "DSA calibration grid: {}",
                artifacts.grid_results_csv.display()
            );
            println!(
                "DSA calibration summary: {}",
                artifacts.summary_json.display()
            );
            println!(
                "DSA calibration report: {}",
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
            println!(
                "Archive summary support implemented: {}",
                status.archive_summary_supported
            );
            println!("Implemented now: {}", status.fully_implemented);
            println!("Blocker: {}", status.blocker);
            let archive = args.archive.or_else(|| {
                status
                    .manual_placement_path
                    .exists()
                    .then_some(status.manual_placement_path.clone())
            });
            if let Some(archive) = archive {
                println!("Inspecting archive: {}", archive.display());
                let manifest = phm2018::inspect_archive(&archive)?;
                println!("{}", serde_json::to_string_pretty(&manifest)?);
            }
            Ok(())
        }
        Command::RunPhm2018(args) => {
            let data_root = args.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);
            let artifacts =
                run_phm2018_benchmark(&data_root, &output_root, args.secom_run_dir.as_deref())?;
            println!("Run directory: {}", artifacts.run_dir.display());
            println!(
                "PHM lead-time metrics: {}",
                artifacts.lead_time_metrics_path.display()
            );
            println!(
                "PHM early-warning stats: {}",
                artifacts.early_warning_stats_path.display()
            );
            println!(
                "Claim alignment report: {}",
                artifacts.claim_alignment_report_path.display()
            );
            println!(
                "Engineering report (tex): {}",
                artifacts.tex_report_path.display()
            );
            if let Some(pdf) = &artifacts.pdf_path {
                println!("Engineering report (pdf): {}", pdf.display());
            }
            println!("ZIP bundle: {}", artifacts.zip_path.display());
            Ok(())
        }
        Command::RenderNonIntrusiveArtifacts(args) => {
            let artifacts = materialize_non_intrusive_artifacts(&args.run_dir)?;
            println!(
                "Non-intrusive interface spec: {}",
                artifacts.interface_spec_path.display()
            );
            println!(
                "Non-intrusive architecture PNG: {}",
                artifacts.architecture_png_path.display()
            );
            println!(
                "Non-intrusive architecture SVG: {}",
                artifacts.architecture_svg_path.display()
            );
            Ok(())
        }
        Command::RenderUnifiedValueFigure(args) => {
            let output_root = args.output_root.unwrap_or_else(default_output_root);
            let secom_root_candidates = [
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-semiconductor"),
                output_root.clone(),
            ];
            let secom_run_dir = match args.secom_run_dir {
                Some(path) => path,
                None => secom_root_candidates
                    .iter()
                    .find_map(|root| {
                        resolve_latest_completed_run(
                            root,
                            "_secom",
                            "dsa_operator_delta_targets.json",
                        )
                    })
                    .ok_or_else(|| {
                        crate::error::DsfbSemiconductorError::DatasetFormat(
                            "could not resolve a completed SECOM run directory".into(),
                        )
                    })?,
            };
            let phm_run_dir = match args.phm_run_dir {
                Some(path) => Some(path),
                None => {
                    let phm_root_candidates = [
                        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("output-dsfb-semiconductor"),
                        output_root.clone(),
                    ];
                    phm_root_candidates.iter().find_map(|root| {
                        resolve_latest_completed_run(
                            root,
                            "_phm2018",
                            "phm2018_early_warning_stats.json",
                        )
                    })
                }
            };
            let paper_tex = args.paper_tex.or_else(|| {
                Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("paper/semiconductor.tex"))
            });
            let artifacts = render_unified_value_figure(
                &secom_run_dir,
                phm_run_dir.as_deref(),
                paper_tex.as_deref(),
            )?;
            println!("SECOM run: {}", artifacts.secom_run_dir.display());
            if let Some(phm) = &artifacts.phm_run_dir {
                println!("PHM run: {}", phm.display());
            } else {
                println!("PHM run: unavailable; Panel C rendered as placeholder");
            }
            println!("Unified figure: {}", artifacts.figure_path.display());
            println!("Companion CSV: {}", artifacts.csv_path.display());
            println!("PHM panel available: {}", artifacts.phm_panel_available);
            println!("Paper updated: {}", artifacts.paper_updated);
            Ok(())
        }
        Command::SbirDemo(args) => {
            let data_root = args.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);

            // 1. Fetch SECOM
            println!("[sbir-demo] Fetching SECOM dataset...");
            let secom_paths = secom::fetch_if_missing(&data_root)?;
            println!("[sbir-demo] SECOM ready: {}", secom_paths.root.display());

            // 2. Run SECOM benchmark
            println!("[sbir-demo] Running SECOM benchmark...");
            let secom_artifacts = run_secom_benchmark(
                &data_root,
                Some(&output_root),
                PipelineConfig::default(),
                args.fetch_if_missing,
            )?;
            println!(
                "[sbir-demo] SECOM run dir:    {}",
                secom_artifacts.run_dir.display()
            );
            if let Some(pdf) = &secom_artifacts.report.pdf_path {
                println!("[sbir-demo] SECOM PDF:         {}", pdf.display());
            }
            println!(
                "[sbir-demo] SECOM ZIP:         {}",
                secom_artifacts.zip_path.display()
            );

            // 3. Calibrate SECOM (single-point default grid)
            println!("[sbir-demo] Running SECOM calibration...");
            let cal_grid = CalibrationGrid {
                healthy_pass_runs: vec![100],
                drift_window: vec![5],
                envelope_sigma: vec![3.0],
                boundary_fraction_of_rho: vec![0.5],
                state_confirmation_steps: vec![2],
                persistent_state_steps: vec![2],
                density_window: vec![10],
                ewma_alpha: vec![0.2],
                ewma_sigma_multiplier: vec![3.0],
                cusum_kappa_sigma_multiplier: vec![0.5],
                cusum_alarm_sigma_multiplier: vec![5.0],
                run_energy_sigma_multiplier: vec![3.0],
                pca_variance_explained: vec![0.95],
                pca_t2_sigma_multiplier: vec![3.0],
                pca_spe_sigma_multiplier: vec![3.0],
                drift_sigma_multiplier: vec![3.0],
                slew_sigma_multiplier: vec![3.0],
                grazing_window: vec![10],
                grazing_min_hits: vec![3],
                pre_failure_lookback_runs: vec![20],
            };
            let cal_artifacts = run_secom_calibration(
                &data_root,
                Some(&output_root),
                cal_grid,
                args.fetch_if_missing,
            )?;
            println!(
                "[sbir-demo] Calibration dir:   {}",
                cal_artifacts.run_dir.display()
            );
            if let Some(pdf) = &cal_artifacts.pdf_path {
                println!("[sbir-demo] Calibration PDF:   {}", pdf.display());
            }
            println!(
                "[sbir-demo] Calibration ZIP:   {}",
                cal_artifacts.zip_path.display()
            );

            // 4. DSA calibration (minimal default grid)
            println!("[sbir-demo] Running DSA calibration...");
            let dsa_artifacts = run_secom_dsa_calibration(
                &data_root,
                Some(&output_root),
                PipelineConfig::default(),
                crate::precursor::DsaCalibrationGrid {
                    window: vec![5],
                    persistence_runs: vec![2],
                    alert_tau: vec![2.0],
                    corroborating_feature_count_min: vec![2],
                },
                args.fetch_if_missing,
            )?;
            println!(
                "[sbir-demo] DSA cal dir:       {}",
                dsa_artifacts.run_dir.display()
            );
            if let Some(pdf) = &dsa_artifacts.pdf_path {
                println!("[sbir-demo] DSA cal PDF:       {}", pdf.display());
            }
            println!(
                "[sbir-demo] DSA cal ZIP:       {}",
                dsa_artifacts.zip_path.display()
            );

            // 5. PHM 2018 (skip with warning if neither archive nor extracted dataset found)
            let phm_status = phm2018::support_status(&data_root);
            if phm_status.fully_implemented || phm_status.manual_placement_path.exists() {
                println!("[sbir-demo] Running PHM 2018 benchmark...");
                match run_phm2018_benchmark(
                    &data_root,
                    &output_root,
                    Some(&secom_artifacts.run_dir),
                ) {
                    Ok(phm_artifacts) => {
                        println!(
                            "[sbir-demo] PHM run dir:       {}",
                            phm_artifacts.run_dir.display()
                        );
                        println!(
                            "[sbir-demo] PHM tex report:    {}",
                            phm_artifacts.tex_report_path.display()
                        );
                        if let Some(pdf) = &phm_artifacts.pdf_path {
                            println!("[sbir-demo] PHM PDF:           {}", pdf.display());
                        }
                        println!(
                            "[sbir-demo] PHM ZIP:           {}",
                            phm_artifacts.zip_path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("[sbir-demo] PHM 2018 run failed (skipping): {e}");
                    }
                }
            } else {
                println!(
                    "[sbir-demo] PHM 2018 dataset not found. Checked for:"
                );
                println!(
                    "[sbir-demo]   extracted dir: {}",
                    phm_status.extracted_dataset_path.display()
                );
                println!(
                    "[sbir-demo]   archive:       {}",
                    phm_status.manual_placement_path.display()
                );
                println!("[sbir-demo] Place either and re-run sbir-demo.");
            }

            println!("[sbir-demo] All artifacts generated.");
            Ok(())
        }
        Command::PaperLock(args) => {
            let data_root = args.data_root.unwrap_or_else(default_data_root);
            let output_root = args.output_root.unwrap_or_else(default_output_root);

            println!("[paper-lock] Running SECOM benchmark with fixed paper-lock config...");
            let artifacts = run_secom_benchmark(
                &data_root,
                Some(&output_root),
                PipelineConfig::default(),
                args.fetch_if_missing,
            )?;

            let PaperLockMetrics {
                episode_count,
                precision,
                detected_failures,
                total_failures,
            } = artifacts.paper_lock_metrics;

            const EXPECTED_EPISODES: usize = 71;
            const EXPECTED_MIN_PRECISION: f64 = 0.80;
            const EXPECTED_RECALL: usize = 104;

            let episode_ok  = episode_count == EXPECTED_EPISODES;
            let precision_ok = precision >= EXPECTED_MIN_PRECISION;
            let recall_ok   = detected_failures == EXPECTED_RECALL
                           && total_failures     == EXPECTED_RECALL;

            println!("[paper-lock] episode count : {episode_count:>4}  (expected {EXPECTED_EPISODES})  {}",
                if episode_ok  { "OK" } else { "FAIL" });
            println!("[paper-lock] precision     : {:>7.1}%  (expected >= {:.0}%)  {}",
                precision * 100.0,
                EXPECTED_MIN_PRECISION * 100.0,
                if precision_ok { "OK" } else { "FAIL" });
            println!("[paper-lock] recall        : {detected_failures}/{total_failures}  (expected {EXPECTED_RECALL}/{EXPECTED_RECALL})  {}",
                if recall_ok   { "OK" } else { "FAIL" });
            println!("[paper-lock] run dir       : {}", artifacts.run_dir.display());

            if episode_ok && precision_ok && recall_ok {
                println!("[paper-lock] PASS — headline numbers reproduced.");
                Ok(())
            } else {
                eprintln!("[paper-lock] FAIL — one or more headline numbers did not match.");
                std::process::exit(1);
            }
        }
    }
}
