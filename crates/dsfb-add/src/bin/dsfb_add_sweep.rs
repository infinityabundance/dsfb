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
    let config_path = parse_config_path(std::env::args().skip(1))?;
    let config = load_config(config_path.as_deref())?;
    config.validate()?;

    let output_dir = create_timestamped_output_dir()?;
    run_sweeps_into_dir(&config, &output_dir)?;

    println!("Output directory: {}", output_dir.display());
    Ok(())
}

fn parse_config_path<I>(args: I) -> Result<Option<PathBuf>, AddError>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = args.into_iter();
    let mut config_path = None;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => {
                let path = iter.next().ok_or_else(|| {
                    AddError::InvalidConfig("missing value for --config".to_string())
                })?;
                config_path = Some(PathBuf::from(path));
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

    Ok(config_path)
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

fn print_help() {
    println!("Usage: cargo run -p dsfb-add --bin dsfb_add_sweep -- [--config path/to/config.json]");
    println!("If config.json exists in the current directory, it is loaded automatically.");
    println!("Otherwise the built-in 360-point deterministic sweep configuration is used.");
}
