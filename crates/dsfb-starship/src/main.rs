use std::path::PathBuf;

use clap::Parser;
use dsfb_starship::config::SimConfig;
use dsfb_starship::run_simulation;

#[derive(Debug, Parser)]
#[command(author, version, about = "Starship 6-DoF re-entry DSFB demonstration")]
struct Cli {
    /// Output base directory (relative paths are resolved from workspace root)
    #[arg(long, default_value = "output-dsfb-starship")]
    output: PathBuf,

    /// Integration step in seconds
    #[arg(long)]
    dt: Option<f64>,

    /// Final simulation time in seconds
    #[arg(long)]
    t_final: Option<f64>,

    /// DSFB EMA factor
    #[arg(long)]
    rho: Option<f64>,

    /// Slew threshold for acceleration channels [m/s^3]
    #[arg(long)]
    slew_threshold: Option<f64>,

    /// Random seed
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut cfg = SimConfig::default();
    if let Some(v) = cli.dt {
        cfg.dt = v;
    }
    if let Some(v) = cli.t_final {
        cfg.t_final = v;
    }
    if let Some(v) = cli.rho {
        cfg.rho = v;
    }
    if let Some(v) = cli.slew_threshold {
        cfg.slew_threshold_accel = v;
        cfg.slew_threshold_gyro = (0.055 * v).max(0.15);
    }
    if let Some(v) = cli.seed {
        cfg.seed = v;
    }

    let summary = run_simulation(&cfg, &cli.output)?;

    println!(
        "Simulation complete. Samples: {} | Blackout: {:.1} s",
        summary.samples, summary.blackout_duration_s
    );
    println!("Run directory: {}", summary.outputs.output_dir.display());
    println!("CSV: {}", summary.outputs.csv_path.display());
    println!("Summary: {}", summary.outputs.summary_path.display());
    println!("Altitude plot: {}", summary.outputs.plot_altitude_path.display());
    println!("Error plot: {}", summary.outputs.plot_error_path.display());
    println!("Trust plot: {}", summary.outputs.plot_trust_path.display());

    println!(
        "DSFB RMSE pos/vel/att: {:.2} m | {:.3} m/s | {:.3} deg",
        summary.dsfb.rmse_position_m,
        summary.dsfb.rmse_velocity_mps,
        summary.dsfb.rmse_attitude_deg
    );

    Ok(())
}
