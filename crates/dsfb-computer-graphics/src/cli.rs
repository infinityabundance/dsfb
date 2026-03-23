use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::DemoConfig;
use crate::datasets::{
    prepare_davis_dataset, prepare_sintel_dataset, validate_standard_external_package,
};
use crate::engine_native::{run_engine_native_import, run_engine_native_replay};
use crate::error::Result;
use crate::external_validation::probe_external_gpu_only;
use crate::mixed_regime::confirm_mixed_regime;
use crate::pipeline::{
    export_evaluator_handoff, export_minimal_report, generate_scene_artifacts, run_all,
    run_all_filtered, run_check_signing, run_demo_a, run_demo_a_filtered, run_demo_b,
    run_demo_b_efficiency_only, run_demo_b_filtered, run_engine_realistic_bridge,
    run_external_replay_only, run_gpu_path_only, run_realism_bridge_only,
    run_resolution_scaling_only, run_sensitivity_only, run_timing_only, validate_artifact_bundle,
    validate_engine_native_gates, validate_final_bundle,
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
    PrepareDavis {
        #[arg(long, default_value = "data/external/davis")]
        output: PathBuf,
    },
    PrepareSintel {
        #[arg(long, default_value = "data/external/sintel")]
        output: PathBuf,
    },
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
    RunScenario {
        scenario: String,
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunAll {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        #[arg(long)]
        scenario: Option<String>,
    },
    RunTiming {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunResolutionScaling {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunSensitivity {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunDemoBEfficiency {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    RunGpuPath {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    #[command(visible_aliases = ["run-external-replay", "replay-external"])]
    ImportExternal {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    #[command(visible_alias = "run-realism-bridge")]
    RunRealismSuite {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    ExportEvaluatorHandoff {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    Validate {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    ValidateFinal {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
        /// Allow pending engine-native gates (pass even if no real engine capture has been provided).
        /// Without this flag, validate-final fails if ENGINE_NATIVE_CAPTURE_MISSING=true.
        #[arg(long)]
        allow_pending_engine_native: bool,
    },
    /// Import and validate an engine-native temporal buffer capture.
    /// See docs/engine_capture_schema.md and examples/engine_native_capture_manifest.json.
    ImportEngineNative {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = "generated/engine_native")]
        output: PathBuf,
    },
    /// Run the full DSFB replay pipeline on an engine-native capture.
    /// Same pipeline as DAVIS/Sintel — no special-case path.
    RunEngineNativeReplay {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = "generated/engine_native")]
        output: PathBuf,
    },
    /// Confirm one mixed-regime case (aliasing + variance co-active in same ROI).
    /// Uses internal synthetic scenario data; engine-native confirmation remains pending.
    ConfirmMixedRegime {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    ValidateArtifacts {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    ExportMinimalReport {
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    /// Internal: run GPU probe in an isolated subprocess (used by run-external-replay)
    #[command(hide = true)]
    ProbeExternalGpu {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long, default_value = "generated")]
        output: PathBuf,
    },
    /// Generate the engine-realistic synthetic 1080p bridge and run the full external validation bundle on it.
    RunEngineRealisticBridge {
        #[arg(long, default_value = "generated/engine_realistic")]
        output: PathBuf,
    },
    /// Generate the check-signing evidence report answering all panel objections.
    RunCheckSigning {
        #[arg(long, default_value = "generated/final_bundle")]
        output: PathBuf,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    let config = DemoConfig::default();
    match cli.command {
        Command::PrepareDavis { output } => {
            let manifest = prepare_davis_dataset(&output)?;
            println!("DAVIS manifest: {}", manifest.display());
        }
        Command::PrepareSintel { output } => {
            let manifest = prepare_sintel_dataset(&output)?;
            println!("Sintel manifest: {}", manifest.display());
        }
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
        Command::RunScenario { scenario, output } => {
            let artifacts = run_all_filtered(&config, &output, Some(&scenario))?;
            println!("scenario output: {}", artifacts.output_dir.display());
            println!("report: {}", artifacts.demo_a.report_path.display());
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
        Command::RunTiming { output } => {
            let report = run_timing_only(&config, &output)?;
            println!("timing report: {}", report.display());
        }
        Command::RunResolutionScaling { output } => {
            let report = run_resolution_scaling_only(&config, &output)?;
            println!("resolution scaling report: {}", report.display());
        }
        Command::RunSensitivity { output } => {
            let report = run_sensitivity_only(&config, &output)?;
            println!("parameter sensitivity report: {}", report.display());
        }
        Command::RunDemoBEfficiency { output } => {
            let report = run_demo_b_efficiency_only(&config, &output)?;
            println!("demo b efficiency report: {}", report.display());
        }
        Command::RunGpuPath { output } => {
            let report = run_gpu_path_only(&config, &output)?;
            println!("gpu execution report: {}", report.display());
        }
        Command::ImportExternal { manifest, output } => {
            let report = run_external_replay_only(&config, &manifest, &output)?;
            println!("external replay report: {}", report.display());
        }
        Command::RunRealismSuite { output } => {
            let report = run_realism_bridge_only(&config, &output)?;
            println!("realism bridge report: {}", report.display());
        }
        Command::ExportEvaluatorHandoff { output } => {
            let report = export_evaluator_handoff(&config, &output)?;
            println!("evaluator handoff: {}", report.display());
        }
        Command::Validate { output } => {
            validate_standard_external_package(&output)?;
            println!(
                "validated standard external package at {}",
                output.display()
            );
        }
        Command::ValidateFinal {
            output,
            allow_pending_engine_native,
        } => {
            validate_final_bundle(&output)?;
            // Engine-native gates: check {output}/../engine_native/ (i.e., generated/engine_native/)
            let engine_native_dir = output
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("engine_native");
            validate_engine_native_gates(&engine_native_dir, allow_pending_engine_native)?;
            println!("validated final bundle at {}", output.display());
        }
        Command::ImportEngineNative { manifest, output } => {
            let artifacts = run_engine_native_import(&config, &manifest, &output)?;
            println!("engine-native import report: {}", artifacts.import_report_path.display());
            println!("resolved manifest: {}", artifacts.resolved_manifest_path.display());
            println!("ENGINE_NATIVE_CAPTURE_MISSING={}", artifacts.capture_missing);
        }
        Command::RunEngineNativeReplay { manifest, output } => {
            let artifacts = run_engine_native_replay(&config, &manifest, &output)?;
            println!("engine-native replay report: {}", artifacts.replay_report_path.display());
            println!("GPU report: {}", artifacts.gpu_report_path.display());
            println!("Demo A: {}", artifacts.demo_a_report_path.display());
            println!("Demo B: {}", artifacts.demo_b_report_path.display());
            println!("validation: {}", artifacts.validation_report_path.display());
            println!("ENGINE_NATIVE_CAPTURE_MISSING={}", artifacts.capture_missing);
        }
        Command::ConfirmMixedRegime { output } => {
            let report = confirm_mixed_regime(&config, &output)?;
            println!("mixed regime confirmation: {}", report.display());
        }
        Command::ValidateArtifacts { output } => {
            validate_artifact_bundle(&output)?;
            println!("validated artifact bundle at {}", output.display());
        }
        Command::ExportMinimalReport { output } => {
            let report = export_minimal_report(&config, &output)?;
            println!("minimal report: {}", report.display());
        }
        Command::ProbeExternalGpu { manifest, output } => {
            let metrics_path = probe_external_gpu_only(&config, &manifest, &output)?;
            println!("gpu probe metrics: {}", metrics_path.display());
        }
        Command::RunEngineRealisticBridge { output } => {
            let report = run_engine_realistic_bridge(&config, &output)?;
            println!("engine-realistic bridge report: {}", report.display());
        }
        Command::RunCheckSigning { output } => {
            let report = run_check_signing(&config, &output)?;
            println!("check-signing report: {}", report.display());
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
