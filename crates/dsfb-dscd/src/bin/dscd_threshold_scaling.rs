use std::path::PathBuf;
use std::process;

use anyhow::{bail, Result};
use dsfb_dscd::{
    create_timestamped_output_dir_in, run_threshold_scaling, workspace_root_dir, DscdScalingConfig,
    EventId,
};

fn main() {
    if let Err(error) = try_main() {
        eprintln!("dscd-threshold-scaling failed: {error}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut cfg = DscdScalingConfig::default();
    let mut output_root = PathBuf::from("output-dsfb-dscd");
    let mut tau_steps = 201_usize;

    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--event-counts" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for --event-counts"))?;
                cfg.event_counts = parse_usize_list(&raw)?;
            }
            "--tau-steps" => {
                tau_steps = parse_usize_arg("--tau-steps", iter.next())?;
            }
            "--output-root" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for --output-root"))?;
                output_root = PathBuf::from(raw);
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    cfg.tau_grid = build_tau_grid(tau_steps);
    cfg.initial_event = EventId(0);
    cfg.max_path_length = usize::MAX;
    cfg.critical_fraction = 0.5;
    cfg.validate()?;

    let resolved_output_root = if output_root.is_absolute() {
        output_root
    } else {
        workspace_root_dir().join(output_root)
    };

    let output_paths = create_timestamped_output_dir_in(&resolved_output_root)?;
    run_threshold_scaling(&cfg, &output_paths.run_dir)?;

    println!("{}", output_paths.run_dir.display());
    Ok(())
}

fn parse_usize_list(raw: &str) -> Result<Vec<usize>> {
    let mut out = Vec::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        out.push(token.parse::<usize>()?);
    }

    if out.is_empty() {
        bail!("--event-counts must contain at least one integer");
    }

    Ok(out)
}

fn parse_usize_arg(flag: &str, raw: Option<String>) -> Result<usize> {
    let raw = raw.ok_or_else(|| anyhow::anyhow!("missing value for {flag}"))?;
    Ok(raw.parse::<usize>()?)
}

fn build_tau_grid(steps: usize) -> Vec<f64> {
    if steps <= 1 {
        return vec![0.0];
    }

    let denom = (steps - 1) as f64;
    (0..steps).map(|idx| idx as f64 / denom).collect()
}

fn print_help() {
    println!(
        "Usage: cargo run -p dsfb-dscd --bin dscd_threshold_scaling -- \\
         --event-counts 2048,4096,8192,16384,32768 --tau-steps 201 --output-root output-dsfb-dscd"
    );
}
