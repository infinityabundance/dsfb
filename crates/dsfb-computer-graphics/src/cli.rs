use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::DemoConfig;
use crate::error::Result;
use crate::pipeline::{generate_scene_artifacts, run_demo_a, run_demo_b};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Minimal DSFB computer graphics research artifact"
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
    },
    RunDemoA {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunDemoB {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    MakeFigures {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    MakeReport {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    let config = DemoConfig::default();
    match cli.command {
        Command::GenerateScene { output } => {
            let manifest = generate_scene_artifacts(&config, &output)?;
            println!(
                "generated scene with {} frames at {}",
                manifest.frame_count,
                output.display()
            );
        }
        Command::RunDemoA { output }
        | Command::MakeFigures { output }
        | Command::MakeReport { output } => {
            let artifacts = run_demo_a(&config, &output)?;
            println!("demo output: {}", artifacts.output_dir.display());
            println!("metrics: {}", artifacts.metrics_path.display());
            println!("report: {}", artifacts.report_path.display());
            println!(
                "reviewer summary: {}",
                artifacts.reviewer_summary_path.display()
            );
            println!(
                "completion note: {}",
                artifacts.completion_note_path.display()
            );
            for figure in artifacts.figure_paths {
                println!("figure: {}", figure.display());
            }
        }
        Command::RunDemoB { output } => {
            let artifacts = run_demo_b(&config, &output)?;
            println!("demo output: {}", artifacts.output_dir.display());
            println!("metrics: {}", artifacts.metrics_path.display());
            println!("report: {}", artifacts.report_path.display());
            for figure in artifacts.figure_paths {
                println!("figure: {}", figure.display());
            }
        }
    }
    Ok(())
}
