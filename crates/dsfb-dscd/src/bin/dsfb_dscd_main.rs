use std::process;

use anyhow::{bail, Result};
use dsfb::sim::SimConfig;
use dsfb_add::SimulationConfig;
use dsfb_dscd::{create_timestamped_output_dir, run_trust_threshold_sweep, DscdSweepConfig};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("dsfb-dscd sweep failed: {error}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let cfg = parse_cli(std::env::args().skip(1))?;
    let dsfb_cfg = SimConfig {
        steps: cfg.num_events,
        ..SimConfig::default()
    };
    let add_cfg = SimulationConfig {
        steps_per_run: cfg.num_events,
        multi_steps_per_run: vec![cfg.num_events],
        ..SimulationConfig::default()
    };

    let output_paths = create_timestamped_output_dir()?;
    run_trust_threshold_sweep(&cfg, &dsfb_cfg, &add_cfg, &output_paths)?;

    println!("{}", output_paths.run_dir.display());
    Ok(())
}

fn parse_cli<I>(args: I) -> Result<DscdSweepConfig>
where
    I: IntoIterator<Item = String>,
{
    let mut cfg = DscdSweepConfig::default();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--num-events" => {
                cfg.num_events = parse_usize_arg(&arg, iter.next())?;
            }
            "--tau-min" => {
                cfg.tau_min = parse_f64_arg(&arg, iter.next())?;
            }
            "--tau-max" => {
                cfg.tau_max = parse_f64_arg(&arg, iter.next())?;
            }
            "--tau-steps" => {
                cfg.tau_steps = parse_usize_arg(&arg, iter.next())?;
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    cfg.validate()?;
    Ok(cfg)
}

fn parse_usize_arg(flag: &str, raw: Option<String>) -> Result<usize> {
    let raw = raw.ok_or_else(|| anyhow::anyhow!("missing value for {flag}"))?;
    Ok(raw.parse::<usize>()?)
}

fn parse_f64_arg(flag: &str, raw: Option<String>) -> Result<f64> {
    let raw = raw.ok_or_else(|| anyhow::anyhow!("missing value for {flag}"))?;
    Ok(raw.parse::<f64>()?)
}

fn print_help() {
    println!(
        "Usage: cargo run --release -p dsfb-dscd -- [--num-events 1024] [--tau-min 0.0] [--tau-max 1.0] [--tau-steps 101]"
    );
}
