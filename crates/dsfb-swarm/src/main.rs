use clap::Parser;
use dsfb_swarm::cli::Cli;
use dsfb_swarm::error::DsfbSwarmResult;

fn main() -> DsfbSwarmResult<()> {
    let cli = Cli::parse();
    let run_dir = dsfb_swarm::run_cli(cli)?;
    println!("{}", run_dir.display());
    Ok(())
}
