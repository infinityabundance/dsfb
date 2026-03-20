use std::path::PathBuf;

use anyhow::Result;
use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;

fn example_data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("data")
        .join(name)
}

fn main() -> Result<()> {
    let input = CsvInputConfig {
        observed_csv: example_data_path("illustrative_observed.csv"),
        predicted_csv: example_data_path("illustrative_predicted.csv"),
        scenario_id: "illustrative_csv_example".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 0.6,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "illustrative_csv_envelope".to_string(),
    };
    let config = EngineConfig::csv(
        CommonRunConfig {
            output_root: Some(std::env::temp_dir().join("dsfb-semiotics-engine-csv-example")),
            ..Default::default()
        },
        input,
    );
    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected()?;
    let artifacts = export_artifacts(&bundle)?;

    println!("run_dir={}", artifacts.run_dir.display());
    Ok(())
}
