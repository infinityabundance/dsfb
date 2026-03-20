use anyhow::Result;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};

fn main() -> Result<()> {
    let config = EngineConfig::synthetic_single(
        CommonRunConfig {
            output_root: Some(std::env::temp_dir().join("dsfb-semiotics-engine-example")),
            ..Default::default()
        },
        "nominal_stable",
    );
    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected()?;
    let artifacts = export_artifacts(&bundle)?;

    println!("run_dir={}", artifacts.run_dir.display());
    Ok(())
}
