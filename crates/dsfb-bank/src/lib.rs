pub mod cli;
pub mod csv_writer;
pub mod output;
pub mod registry;
pub mod run_summary;
pub mod runners;
pub mod sim;
pub mod timestamp;

use std::path::PathBuf;

use anyhow::Result;

use crate::cli::{Cli, RunSelection};
use crate::output::{
    collect_inventory, default_output_root, prepare_output_layout, write_logs, write_manifest,
};
use crate::registry::TheoremRegistry;
use crate::run_summary::write_run_summary;
use crate::runners::{run_selection, RunExecution};
use crate::timestamp::create_timestamped_run_dir;

pub fn execute(cli: &Cli) -> Result<Option<PathBuf>> {
    let registry = TheoremRegistry::load()?;
    if cli.list {
        print_listing(&registry);
        return Ok(None);
    }

    let selection = cli.selection()?;
    let output_root = cli.output.clone().unwrap_or_else(default_output_root);
    let output_root = if output_root.is_absolute() {
        output_root
    } else {
        crate::registry::workspace_root().join(output_root)
    };

    let run_dir = create_timestamped_run_dir(&output_root)?;
    let layout = prepare_output_layout(&run_dir)?;
    let execution = run_selection(&registry, &selection, &layout, cli.seed.unwrap_or(0))?;

    write_run_summary(
        &run_dir.run_dir.join("run_summary.md"),
        &run_dir,
        &selection,
        &execution,
    )?;
    write_logs(&run_dir.run_dir.join("logs.txt"), &execution.logs)?;

    let mut output_file_inventory = collect_inventory(&run_dir.run_dir)?;
    output_file_inventory.push(String::from("manifest.json"));
    output_file_inventory.sort();

    write_manifest(
        &run_dir.run_dir.join("manifest.json"),
        &run_dir,
        cli,
        &selection,
        &registry,
        &execution,
        output_file_inventory,
    )?;

    Ok(Some(run_dir.run_dir.clone()))
}

fn print_listing(registry: &TheoremRegistry) {
    println!("Theorem demos:");
    for theorem in registry.all_theorems() {
        println!(
            "  {} [{}] {}",
            theorem.id,
            theorem.component.as_str(),
            theorem.title
        );
    }

    println!("\nRealization outputs:");
    for component in registry.bank_components() {
        println!("  realizations/{}_realizations.csv", component.as_str());
    }
    println!("  realizations/all_realizations.csv");
}

pub fn execution_summary(execution: &RunExecution) -> String {
    format!(
        "ran {} theorem demos and {} realization exports",
        execution.theorem_results.len(),
        execution.realization_results.len()
    )
}

pub fn default_selection() -> RunSelection {
    RunSelection::All
}
