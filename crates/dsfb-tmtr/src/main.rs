use anyhow::Result;
use clap::Parser;
use dsfb_tmtr::config::{Cli, SimulationConfig};
use dsfb_tmtr::output::{create_run_directory, write_run_outputs};
use dsfb_tmtr::simulation::run_simulation;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = SimulationConfig::from_cli(cli)?;
    let run = run_simulation(&config)?;
    let run_dir = create_run_directory(&config.output_root_path())?;
    write_run_outputs(&run, &run_dir)?;
    println!("Output directory: {}", run_dir.run_dir.display());
    Ok(())
}
