use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use dsfb_lattice::{
    default_output_root, run_demo, DemoConfig, ExampleSelection,
    PressureTestSettings,
};
use dsfb_lattice::heuristics::HeuristicSettings;

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
    #[arg(long, default_value_t = true)]
    pressure_test_include_ambiguity_case: bool,
    #[arg(long, default_value_t = 1.08)]
    pressure_test_ambiguity_point_mass_scale: f64,
    #[arg(long, default_value_t = 0.96)]
    pressure_test_ambiguity_point_spring_scale: f64,
    #[arg(long, default_value_t = 0.14)]
    pressure_test_ambiguity_strain_strength: f64,
    #[arg(long, default_value_t = true)]
    heuristics_enabled: bool,
    #[arg(long, default_value_t = 0.18)]
    heuristics_ambiguity_tolerance: f64,
    #[arg(long, default_value_t = 0.01)]
    heuristics_low_noise_threshold: f64,
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
            include_ambiguity_case: cli.pressure_test_include_ambiguity_case,
            ambiguity_point_mass_scale: cli.pressure_test_ambiguity_point_mass_scale,
            ambiguity_point_spring_scale: cli.pressure_test_ambiguity_point_spring_scale,
            ambiguity_strain_strength: cli.pressure_test_ambiguity_strain_strength,
        },
        heuristics: HeuristicSettings {
            enabled: cli.heuristics_enabled,
            ambiguity_tolerance: cli.heuristics_ambiguity_tolerance,
            low_noise_threshold: cli.heuristics_low_noise_threshold,
            similarity_metric: "weighted_l1".to_string(),
        },
    };

    let outcome = run_demo(config)?;
    println!("RUN_DIRECTORY={}", outcome.run_dir.display());
    println!("SUMMARY_JSON={}", outcome.summary_json.display());
    println!("REPORT_PDF={}", outcome.report_pdf.display());
    println!("ZIP_ARCHIVE={}", outcome.zip_path.display());
    Ok(())
}
