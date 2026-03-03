use std::path::PathBuf;
use std::process;

use anyhow::{bail, Result};
use clap::{ArgAction, Parser};
use dsfb_dscd::{
    create_timestamped_output_dir_in, linspace, run_dscd_simulation, workspace_root_dir, DscdConfig,
};

#[derive(Debug, Parser)]
#[command(
    name = "dsfb-dscd",
    version,
    about = "Deterministic Structural Causal Dynamics sweep + finite-size scaling"
)]
struct Cli {
    /// Run the quick profile (default when neither --quick nor --full are specified).
    #[arg(long, action = ArgAction::SetTrue)]
    quick: bool,
    /// Run the full profile for larger workstation runs.
    #[arg(long, action = ArgAction::SetTrue)]
    full: bool,
    /// Override event count for the main N run.
    #[arg(long)]
    num_events: Option<usize>,
    /// Override scaling event counts as comma-separated integers.
    #[arg(long)]
    scaling_ns: Option<String>,
    /// Override tau sampling count directly.
    #[arg(long)]
    num_tau_samples: Option<usize>,
    /// Alternative tau density knob; mapped deterministically to sample count.
    #[arg(long)]
    taus_per_decade: Option<usize>,
    /// Output root directory (timestamped run dir is created inside this root).
    #[arg(long, default_value = "output-dsfb-dscd")]
    output_root: PathBuf,
    /// Reachability root event id.
    #[arg(long, default_value_t = 0)]
    root_event_id: u64,
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("dsfb-dscd failed: {error}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();
    if cli.quick && cli.full {
        bail!("--quick and --full are mutually exclusive");
    }

    let quick_mode = !cli.full;
    let (default_num_events, default_scaling, default_tau_samples) = if quick_mode {
        (10_000_usize, vec![2_000, 5_000, 10_000], 201_usize)
    } else {
        (
            100_000_usize,
            vec![4_096, 8_192, 16_384, 32_768, 65_536, 100_000],
            1_001_usize,
        )
    };

    let num_events = cli.num_events.unwrap_or(default_num_events);
    let scaling_ns = cli
        .scaling_ns
        .as_deref()
        .map(parse_usize_list)
        .transpose()?
        .unwrap_or(default_scaling);

    let tau_samples = match (cli.num_tau_samples, cli.taus_per_decade) {
        (Some(_), Some(_)) => bail!("use either --num-tau-samples or --taus-per-decade"),
        (Some(samples), None) => samples.max(2),
        (None, Some(per_decade)) => (per_decade.saturating_mul(10) + 1).max(2),
        (None, None) => default_tau_samples,
    };
    let taus = linspace(0.0, 1.0, tau_samples);

    let output_root = if cli.output_root.is_absolute() {
        cli.output_root
    } else {
        workspace_root_dir().join(cli.output_root)
    };
    let output_paths = create_timestamped_output_dir_in(&output_root)?;

    let cfg = DscdConfig {
        num_events,
        taus,
        root_event_id: cli.root_event_id,
        output_dir: output_paths.run_dir.clone(),
        scaling_ns,
        quick_mode,
    };
    run_dscd_simulation(&cfg)?;

    println!("{}", output_paths.run_dir.display());
    Ok(())
}

fn parse_usize_list(raw: &str) -> Result<Vec<usize>> {
    let mut values = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        values.push(token.parse::<usize>()?);
    }

    if values.is_empty() {
        bail!("--scaling-ns must contain at least one integer");
    }
    Ok(values)
}
