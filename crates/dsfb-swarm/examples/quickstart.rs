use dsfb_swarm::config::{ResolvedCommand, RunConfig};
use dsfb_swarm::report::run_scenario_bundle;

fn main() -> anyhow::Result<()> {
    let run_dir = run_scenario_bundle(ResolvedCommand::Quickstart(RunConfig::default_quickstart()))?;
    println!("{}", run_dir.display());
    Ok(())
}
