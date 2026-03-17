pub mod cli;
pub mod config;
pub mod error;
pub mod math;
pub mod report;
pub mod sim;

use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::config::{
    BenchmarkArgsPatch, BenchmarkConfig, ReportArgsPatch, ResolvedCommand, RunArgsPatch, RunConfig,
};
use crate::error::DsfbSwarmResult;
use crate::report::{generate_report_for_existing_run, run_benchmark_suite, run_scenario_bundle};
use anyhow::Context;

pub fn run_cli(cli: Cli) -> DsfbSwarmResult<PathBuf> {
    match cli.command {
        Command::Run(args) => {
            let config_path = args.config.clone();
            let config =
                RunConfig::resolve_with_patch(config_path.as_deref(), RunArgsPatch::from(args))
                    .context("failed to resolve run configuration")?;
            run_scenario_bundle(ResolvedCommand::Run(config))
        }
        Command::Scenario(args) => {
            let config_path = args.inner.config.clone();
            let config =
                RunConfig::resolve_with_patch(config_path.as_deref(), RunArgsPatch::from(args))
                    .context("failed to resolve scenario configuration")?;
            run_scenario_bundle(ResolvedCommand::Run(config))
        }
        Command::Quickstart(args) => {
            let mut config = RunConfig::default_quickstart();
            if let Some(path) = args.output_root {
                config.output_root = path;
            }
            run_scenario_bundle(ResolvedCommand::Quickstart(config))
        }
        Command::Benchmark(args) => {
            let config_path = args.config.clone();
            let patch = BenchmarkArgsPatch::try_from_args(args)
                .context("failed to parse benchmark CLI arguments")?;
            let config = BenchmarkConfig::resolve_with_patch(config_path.as_deref(), patch)
                .context("failed to resolve benchmark configuration")?;
            run_benchmark_suite(config)
        }
        Command::Report(args) => {
            let patch = ReportArgsPatch::from(args);
            generate_report_for_existing_run(patch)
        }
    }
}
