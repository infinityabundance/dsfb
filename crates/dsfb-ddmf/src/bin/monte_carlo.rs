use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use csv::Writer;
use dsfb_ddmf::monte_carlo::{
    run_monte_carlo, summarize_batch, trajectory_rows, MonteCarloConfig, DEFAULT_MONTE_CARLO_RUNS,
};

#[derive(Debug, Clone)]
struct CliConfig {
    runs: usize,
    steps: usize,
    seed: u64,
    rho: f64,
    beta: f64,
    epsilon_bound: f64,
    recovery_delta: f64,
}

impl Default for CliConfig {
    fn default() -> Self {
        let defaults = MonteCarloConfig::default();
        Self {
            runs: defaults.n_runs,
            steps: defaults.n_steps,
            seed: defaults.seed,
            rho: defaults.rho,
            beta: defaults.beta,
            epsilon_bound: defaults.epsilon_bound,
            recovery_delta: defaults.recovery_delta,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = parse_args(env::args().skip(1))?;
    let output_dir = create_output_dir()?;
    let config = MonteCarloConfig {
        n_runs: cli.runs,
        n_steps: cli.steps,
        seed: cli.seed,
        rho: cli.rho,
        beta: cli.beta,
        epsilon_bound: cli.epsilon_bound,
        recovery_delta: cli.recovery_delta,
    };
    let batch = run_monte_carlo(&config);
    let summary = summarize_batch(&config, &batch);

    write_results_csv(&output_dir.join("results.csv"), &batch.records)?;
    write_trajectory_csv(
        &output_dir.join("single_run_impulse.csv"),
        &batch.example_impulse,
    )?;
    write_trajectory_csv(
        &output_dir.join("single_run_persistent.csv"),
        &batch.example_persistent,
    )?;
    fs::write(
        output_dir.join("summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    println!("Output directory: {}", output_dir.display());
    Ok(())
}

fn parse_args<I>(args: I) -> Result<CliConfig, Box<dyn Error>>
where
    I: IntoIterator<Item = String>,
{
    let mut cli = CliConfig::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--runs" => cli.runs = parse_value(args.next(), "--runs")?,
            "--steps" => cli.steps = parse_value(args.next(), "--steps")?,
            "--seed" => cli.seed = parse_value(args.next(), "--seed")?,
            "--rho" => cli.rho = parse_value(args.next(), "--rho")?,
            "--beta" => cli.beta = parse_value(args.next(), "--beta")?,
            "--epsilon-bound" => cli.epsilon_bound = parse_value(args.next(), "--epsilon-bound")?,
            "--recovery-delta" => {
                cli.recovery_delta = parse_value(args.next(), "--recovery-delta")?
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
    }

    Ok(cli)
}

fn parse_value<T>(value: Option<String>, flag: &str) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    let raw = value.ok_or_else(|| format!("missing value for {flag}"))?;
    Ok(raw.parse()?)
}

fn print_help() {
    println!("Usage: cargo run --bin monte_carlo -- [OPTIONS]");
    println!("  --runs <usize>            default: {DEFAULT_MONTE_CARLO_RUNS} (x360)");
    println!("  --steps <usize>");
    println!("  --seed <u64>");
    println!("  --rho <f64>");
    println!("  --beta <f64>");
    println!("  --epsilon-bound <f64>");
    println!("  --recovery-delta <f64>");
}

fn create_output_dir() -> Result<PathBuf, Box<dyn Error>> {
    let output_root = repo_root().join("output-dsfb-ddmf");
    fs::create_dir_all(&output_root)?;

    let timestamp = timestamp_string()?;
    let output_dir = output_root.join(timestamp);
    fs::create_dir_all(&output_dir)?;
    Ok(output_dir)
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

fn timestamp_string() -> Result<String, Box<dyn Error>> {
    let output = Command::new("date").arg("+%Y%m%d_%H%M%S").output()?;
    if !output.status.success() {
        return Err("date command failed while building output path".into());
    }

    let timestamp = String::from_utf8(output.stdout)?.trim().to_string();
    if timestamp.is_empty() {
        return Err("date command returned an empty timestamp".into());
    }

    Ok(timestamp)
}

fn write_results_csv<P: AsRef<Path>, T: serde::Serialize>(
    path: P,
    rows: &[T],
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

fn write_trajectory_csv(
    path: &Path,
    result: &dsfb_ddmf::SimulationResult,
) -> Result<(), Box<dyn Error>> {
    let rows = trajectory_rows(result);
    write_results_csv(path, &rows)
}
