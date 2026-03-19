use anyhow::Result;
use dsfb_semiotics_engine::cli::args::{CliArgs, ScenarioSelection};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};

fn main() -> Result<()> {
    let args = CliArgs::parse_args();
    let selection = args.selection();
    let config = EngineConfig {
        seed: args.seed,
        steps: args.steps,
        dt: args.dt,
        output_root: args.output_dir.clone(),
        scenario_selection: selection.clone(),
    };

    let engine = StructuralSemioticsEngine::new(config);
    let bundle = match selection {
        ScenarioSelection::Single(id) => engine.run_single(&id)?,
        ScenarioSelection::Csv(input) => engine.run_csv(&input)?,
        ScenarioSelection::All => engine.run_all()?,
    };
    let exported = export_artifacts(&bundle)?;

    println!("run_dir={}", exported.run_dir.display());
    println!("manifest={}", exported.manifest_path.display());
    println!("report_pdf={}", exported.report_pdf.display());
    println!("zip_archive={}", exported.zip_path.display());
    println!("scenario_count={}", bundle.scenario_outputs.len());

    if args.output_dir.is_some() {
        println!("output_root_override=true");
    }

    Ok(())
}
