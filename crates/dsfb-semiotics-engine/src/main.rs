#![forbid(unsafe_code)]

use anyhow::Result;
use dsfb_semiotics_engine::cli::args::{CliArgs, ScenarioSelection};
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};

fn main() -> Result<()> {
    let args = CliArgs::parse_args();
    let selection = args.selection();
    let common = CommonRunConfig {
        seed: args.seed,
        steps: args.steps,
        dt: args.dt,
        output_root: args.output_dir.clone(),
    };
    let config = match selection.clone() {
        ScenarioSelection::All => EngineConfig::synthetic_all(common),
        ScenarioSelection::Single(id) => EngineConfig::synthetic_single(common, id),
        ScenarioSelection::Csv(input) => EngineConfig::csv(common, input),
        ScenarioSelection::Sweep(sweep) => EngineConfig::sweep(common, sweep),
    };

    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected()?;
    let exported = export_artifacts(&bundle)?;

    println!("run_dir={}", exported.run_dir.display());
    println!("manifest={}", exported.manifest_path.display());
    println!("report_pdf={}", exported.report_pdf.display());
    println!("zip_archive={}", exported.zip_path.display());
    println!("scenario_count={}", bundle.scenario_outputs.len());
    println!("input_mode={}", bundle.run_metadata.input_mode);

    if args.output_dir.is_some() {
        println!("output_root_override=true");
    }

    Ok(())
}
