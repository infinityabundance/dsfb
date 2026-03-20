use std::path::PathBuf;

use dsfb_semiotics_engine::cli::args::CsvInputConfig;
use dsfb_semiotics_engine::engine::config::{BankRunConfig, CommonRunConfig};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::io::input::load_csv_trajectories;
use dsfb_semiotics_engine::io::schema::ARTIFACT_SCHEMA_VERSION;
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn csv_input_fixture() -> CsvInputConfig {
    CsvInputConfig {
        observed_csv: fixture_path("observed_fixture.csv"),
        predicted_csv: fixture_path("predicted_fixture.csv"),
        scenario_id: "fixture_csv_case".to_string(),
        channel_names: None,
        time_column: Some("time".to_string()),
        dt_fallback: 0.5,
        envelope_mode: EnvelopeMode::Fixed,
        envelope_base: 0.6,
        envelope_slope: 0.0,
        envelope_switch_step: None,
        envelope_secondary_slope: None,
        envelope_secondary_base: None,
        envelope_name: "fixture_envelope".to_string(),
    }
}

fn external_bank_fixture() -> PathBuf {
    fixture_path("external_bank_minimal.json")
}

#[test]
fn typed_synthetic_config_runs_selected_scenario() {
    let temp = tempdir().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        ..Default::default()
    };
    let config = EngineConfig::synthetic_single(common, "nominal_stable");
    config.validate().unwrap();

    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected().unwrap();

    assert_eq!(bundle.scenario_outputs.len(), 1);
    assert_eq!(bundle.scenario_outputs[0].record.id, "nominal_stable");
    assert_eq!(bundle.run_metadata.schema_version, ARTIFACT_SCHEMA_VERSION);
    assert_eq!(bundle.run_metadata.input_mode, "synthetic");
}

#[test]
fn typed_csv_config_runs_fixture_and_exports_schema_metadata() {
    let temp = tempdir().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        ..Default::default()
    };
    let input = csv_input_fixture();
    let config = EngineConfig::csv(common, input.clone());
    config.validate().unwrap();

    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let manifest = std::fs::read_to_string(&exported.manifest_path).unwrap();
    let report = std::fs::read_to_string(&exported.report_markdown).unwrap();

    assert_eq!(bundle.scenario_outputs.len(), 1);
    assert_eq!(bundle.scenario_outputs[0].record.id, input.scenario_id);
    assert_eq!(bundle.run_metadata.input_mode, "csv");
    assert!(manifest.contains(ARTIFACT_SCHEMA_VERSION));
    assert!(report.contains("Artifact schema"));
    assert!(report.contains("Input mode: `csv`"));
}

#[test]
fn typed_external_bank_config_runs_and_records_bank_provenance() {
    let temp = tempdir().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        bank: BankRunConfig::external(external_bank_fixture(), true),
        ..Default::default()
    };
    let config = EngineConfig::synthetic_single(common, "nominal_stable");
    config.validate().unwrap();

    let engine = StructuralSemioticsEngine::new(config);
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let manifest = std::fs::read_to_string(&exported.manifest_path).unwrap();

    assert_eq!(bundle.run_metadata.bank.source_kind.as_label(), "external");
    assert_eq!(
        bundle.run_metadata.bank.bank_version,
        "external-fixture-bank/v1"
    );
    assert!(bundle.evaluation.bank_validation.valid);
    assert!(exported
        .run_dir
        .join("json/loaded_heuristic_bank_descriptor.json")
        .is_file());
    assert!(manifest.contains("external-fixture-bank/v1"));
}

#[test]
fn csv_fixture_loader_preserves_headers_and_times() {
    let (observed, predicted) = load_csv_trajectories(&csv_input_fixture()).unwrap();
    assert_eq!(
        observed.channel_names,
        vec!["ax".to_string(), "ay".to_string()]
    );
    assert_eq!(predicted.channel_names, observed.channel_names);
    assert_eq!(observed.samples.len(), 5);
    assert!((observed.samples[3].time - 1.5).abs() <= 1.0e-12);
}

#[test]
fn envelope_validation_rejects_incomplete_regime_switch() {
    let spec = EnvelopeSpec {
        name: "bad_regime_switch".to_string(),
        mode: EnvelopeMode::RegimeSwitched,
        base_radius: 1.0,
        slope: 0.0,
        switch_step: Some(8),
        secondary_slope: None,
        secondary_base: Some(1.2),
    };

    let error = spec.validate().unwrap_err();
    assert!(error.to_string().contains("secondary slope"));
}
