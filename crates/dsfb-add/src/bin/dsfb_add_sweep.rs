use std::fs;
use std::path::{Path, PathBuf};

use dsfb_add::{create_timestamped_output_dir, run_sweeps_into_dir, AddError, SimulationConfig};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("dsfb-add sweep failed: {error}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), AddError> {
    let cli = parse_cli(std::env::args().skip(1))?;
    let mut config = load_config(cli.config_path.as_deref())?;
    if let Some(multi_steps_per_run) = cli.multi_steps_per_run {
        config.multi_steps_per_run = multi_steps_per_run;
    }
    config.validate()?;

    let output_dir = create_timestamped_output_dir()?;
    run_sweeps_into_dir(&config, &output_dir)?;

    println!("Output directory: {}", output_dir.display());
    Ok(())
}

struct CliArgs {
    config_path: Option<PathBuf>,
    multi_steps_per_run: Option<Vec<usize>>,
}

fn parse_cli<I>(args: I) -> Result<CliArgs, AddError>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter();
    let mut config_path = None;
    let mut multi_steps_per_run = None;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                let path = iter.next().ok_or_else(|| {
                    AddError::InvalidConfig("missing value for --config".to_string())
                })?;
                config_path = Some(PathBuf::from(path));
            }
            "--multi-steps" => {
                let raw = iter.next().ok_or_else(|| {
                    AddError::InvalidConfig("missing value for --multi-steps".to_string())
                })?;
                multi_steps_per_run = Some(parse_multi_steps(&raw)?);
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(AddError::InvalidConfig(format!(
                    "unknown argument: {other}"
                )));
            }
        }
    }

    Ok(CliArgs {
        config_path,
        multi_steps_per_run,
    })
}

fn load_config(path: Option<&Path>) -> Result<SimulationConfig, AddError> {
    if let Some(path) = path {
        return load_config_file(path);
    }

    let cwd_config = PathBuf::from("config.json");
    if cwd_config.exists() {
        return load_config_file(&cwd_config);
    }

    Ok(SimulationConfig::default())
}

fn load_config_file(path: &Path) -> Result<SimulationConfig, AddError> {
    let raw = fs::read_to_string(path)?;
    let config: SimulationConfig = serde_json::from_str(&raw)?;
    Ok(config)
}

fn parse_multi_steps(raw: &str) -> Result<Vec<usize>, AddError> {
    let mut out = Vec::new();
    for chunk in raw.split(',') {
        let token = chunk.trim();
        if token.is_empty() {
            continue;
        }

        let steps = token.parse::<usize>().map_err(|_| {
            AddError::InvalidConfig(format!(
                "invalid steps_per_run value in --multi-steps: {token}"
            ))
        })?;
        if steps == 0 {
            return Err(AddError::InvalidConfig(
                "--multi-steps values must be greater than zero".to_string(),
            ));
        }
        out.push(steps);
    }

    if out.is_empty() {
        return Err(AddError::InvalidConfig(
            "--multi-steps must include at least one positive integer".to_string(),
        ));
    }

    Ok(out)
}

fn print_help() {
    println!(
        "Usage: cargo run -p dsfb-add --bin dsfb_add_sweep -- [--config path/to/config.json] [--multi-steps 5000,10000,20000]"
    );
    println!("If config.json exists in the current directory, it is loaded automatically.");
    println!("Otherwise the built-in deterministic sweep configuration is used.");
    println!(
        "When --multi-steps is provided, per-N sweep files are written with _N{{steps}} suffixes."
    );
}
