use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use dsfb_lattice::{default_output_root, run_demo, DemoConfig, ExampleSelection, PressureTestSettings};

#[derive(Parser, Debug)]
#[command(
    name = "dsfb-lattice",
    about = "Bounded DSFB lattice and phonon toy-model demonstrator"
)]
struct Cli {
    #[arg(long, default_value_os_t = default_output_root())]
    output_root: PathBuf,
    #[arg(long, value_enum, default_value_t = ExampleSelection::All)]
    example: ExampleSelection,
    #[arg(long, default_value_t = 12)]
    sites: usize,
    #[arg(long, default_value_t = 320)]
    steps: usize,
    #[arg(long, default_value_t = 0.04)]
    dt: f64,
    #[arg(long, default_value_t = 0.06)]
    damping: f64,
    #[arg(long, default_value_t = 4)]
    observed_modes: usize,
    #[arg(long, default_value_t = 4)]
    baseline_runs: usize,
    #[arg(long, default_value_t = 3.0)]
    envelope_sigma: f64,
    #[arg(long, default_value_t = 0.003)]
    envelope_floor: f64,
    #[arg(long, default_value_t = 3)]
    consecutive_crossings: usize,
    #[arg(long, default_value_t = 1.0e-6)]
    normalization_epsilon: f64,
    #[arg(long, default_value_t = true)]
    pressure_test_enabled: bool,
    #[arg(long, default_value_t = 0.018)]
    pressure_test_noise_std: f64,
    #[arg(long, default_value_t = 0.97)]
    pressure_test_predictor_spring_scale: f64,
    #[arg(long, default_value_t = 20_260_318)]
    pressure_test_seed: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = DemoConfig {
        output_root: cli.output_root,
        example: cli.example,
        sites: cli.sites,
        steps: cli.steps,
        dt: cli.dt,
        damping: cli.damping,
        observed_modes: cli.observed_modes,
        baseline_runs: cli.baseline_runs,
        envelope_sigma: cli.envelope_sigma,
        envelope_floor: cli.envelope_floor,
        consecutive_crossings: cli.consecutive_crossings,
        normalization_epsilon: cli.normalization_epsilon,
        pressure_test: PressureTestSettings {
            enabled: cli.pressure_test_enabled,
            observation_noise_std: cli.pressure_test_noise_std,
            predictor_spring_scale: cli.pressure_test_predictor_spring_scale,
            rng_seed: cli.pressure_test_seed,
        },
    };

    let outcome = run_demo(config)?;
    println!("RUN_DIRECTORY={}", outcome.run_dir.display());
    println!("SUMMARY_JSON={}", outcome.summary_json.display());
    println!("REPORT_PDF={}", outcome.report_pdf.display());
    println!("ZIP_ARCHIVE={}", outcome.zip_path.display());
    Ok(())
}
