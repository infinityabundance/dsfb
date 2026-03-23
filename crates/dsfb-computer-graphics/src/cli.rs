use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::DemoConfig;
use crate::error::Result;
use crate::pipeline::{
    generate_scene_artifacts, run_all, run_all_filtered, run_demo_a, run_demo_a_filtered,
    run_demo_b, run_demo_b_filtered, validate_artifact_bundle,
};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "DSFB computer-graphics evaluation and artifact pipeline"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    GenerateScene {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        #[arg(long)]
        scenario: Option<String>,
    },
    RunDemoA {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        #[arg(long)]
        scenario: Option<String>,
    },
    RunDemoB {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        #[arg(long)]
        scenario: Option<String>,
    },
    RunAblations {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunAll {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        #[arg(long)]
        scenario: Option<String>,
    },
    ValidateArtifacts {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    let config = DemoConfig::default();
    match cli.command {
        Command::GenerateScene { output, .. } => {
            let manifest = generate_scene_artifacts(&config, &output)?;
            println!(
                "generated canonical scene manifest for {} at {}",
                manifest.scenario_id,
                output.display()
            );
        }
        Command::RunDemoA { output, scenario } => {
            let artifacts = if let Some(scenario) = scenario.as_deref() {
                run_demo_a_filtered(&config, &output, Some(scenario))?
            } else {
                run_demo_a(&config, &output)?
            };
            print_demo_a_artifacts(&artifacts);
        }
        Command::RunDemoB { output, scenario } => {
            let artifacts = if let Some(scenario) = scenario.as_deref() {
                run_demo_b_filtered(&config, &output, Some(scenario))?
            } else {
                run_demo_b(&config, &output)?
            };
            print_demo_b_artifacts(&artifacts);
        }
        Command::RunAblations { output } => {
            let artifacts = run_demo_a_filtered(&config, &output, Some("thin_reveal"))?;
            print_demo_a_artifacts(&artifacts);
        }
        Command::RunAll { output, scenario } => {
            let artifacts = if let Some(scenario) = scenario.as_deref() {
                run_all_filtered(&config, &output, Some(scenario))?
            } else {
                run_all(&config, &output)?
            };
            println!("run output: {}", artifacts.output_dir.display());
            println!("manifest: {}", artifacts.manifest_path.display());
            println!("report: {}", artifacts.demo_a.report_path.display());
            println!("demo b report: {}", artifacts.demo_b.report_path.display());
            println!(
                "mentor audit: {}",
                artifacts.five_mentor_audit_path.display()
            );
            println!(
                "blocker report: {}",
                artifacts.blocker_report_path.display()
            );
            println!(
                "demo b decision report: {}",
                artifacts.demo_b_decision_report_path.display()
            );
        }
        Command::ValidateArtifacts { output } => {
            validate_artifact_bundle(&output)?;
            println!("validated artifact bundle at {}", output.display());
        }
    }
    Ok(())
}

fn print_demo_a_artifacts(artifacts: &crate::pipeline::DemoAArtifacts) {
    println!("demo output: {}", artifacts.output_dir.display());
    println!("metrics: {}", artifacts.metrics_path.display());
    println!("report: {}", artifacts.report_path.display());
    println!(
        "reviewer summary: {}",
        artifacts.reviewer_summary_path.display()
    );
    println!(
        "ablation report: {}",
        artifacts.ablation_report_path.display()
    );
    println!("cost report: {}", artifacts.cost_report_path.display());
    println!(
        "completion note: {}",
        artifacts.completion_note_path.display()
    );
    for figure in &artifacts.figure_paths {
        println!("figure: {}", figure.display());
    }
}

fn print_demo_b_artifacts(artifacts: &crate::pipeline::DemoBArtifacts) {
    println!("demo output: {}", artifacts.output_dir.display());
    println!("metrics: {}", artifacts.metrics_path.display());
    println!("report: {}", artifacts.report_path.display());
    for figure in &artifacts.figure_paths {
        println!("figure: {}", figure.display());
    }
}
