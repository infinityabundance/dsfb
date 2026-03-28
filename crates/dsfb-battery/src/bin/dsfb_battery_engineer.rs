// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use dsfb_battery::{
    build_adaptive_residual_handoff_note, build_engineer_integration_artifact,
    build_external_residual_evaluation, build_partial_observability_scaffold_note,
    build_shadow_mode_integration_spec, estimate_dsfb_update_complexity, load_b0005_csv,
    load_external_residual_csv, resolve_helper_output_dir, run_dsfb_pipeline,
    write_complexity_report, PipelineConfig,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Opt-in engineer-facing integration and complexity helper"
)]
struct Cli {
    #[arg(long)]
    data: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = 0.88)]
    tactical_margin_fraction: f64,
    #[arg(long)]
    external_residual_csv: Option<PathBuf>,
    #[arg(long, default_value_t = 0.05)]
    external_envelope_rho: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = resolve_helper_output_dir(
        &crate_dir,
        "engineer_extensions/engineer",
        "dsfb_battery_engineer",
        cli.output,
    );
    std::fs::create_dir_all(&output_dir)?;

    let data_path = cli
        .data
        .unwrap_or_else(|| crate_dir.join("data").join("nasa_b0005_capacity.csv"));
    let raw_data = load_b0005_csv(&data_path)?;
    let capacities: Vec<f64> = raw_data.iter().map(|(_, value)| *value).collect();
    let config = PipelineConfig::default();
    let (_, trajectory) = run_dsfb_pipeline(&capacities, &config)?;

    let integration = build_engineer_integration_artifact(
        &capacities,
        &trajectory,
        &config,
        cli.tactical_margin_fraction,
    );
    std::fs::write(
        output_dir.join("integration_helper.json"),
        serde_json::to_string_pretty(&integration)?,
    )?;
    std::fs::write(
        output_dir.join("shadow_mode_integration_spec.md"),
        build_shadow_mode_integration_spec(),
    )?;
    std::fs::write(
        output_dir.join("adaptive_residual_handoff.md"),
        build_adaptive_residual_handoff_note(),
    )?;
    std::fs::write(
        output_dir.join("partial_observability_scaffold.md"),
        build_partial_observability_scaffold_note(),
    )?;

    if let Some(path) = cli.external_residual_csv {
        let samples = load_external_residual_csv(&path)?;
        let evaluation =
            build_external_residual_evaluation(&samples, cli.external_envelope_rho, &config);
        std::fs::write(
            output_dir.join("external_residual_evaluation.json"),
            serde_json::to_string_pretty(&evaluation)?,
        )?;
    }

    let complexity = estimate_dsfb_update_complexity(&config);
    write_complexity_report(&complexity, &output_dir.join("complexity_report.txt"))?;

    println!(
        "Engineer helper artifacts written to: {}",
        output_dir.display()
    );
    Ok(())
}
