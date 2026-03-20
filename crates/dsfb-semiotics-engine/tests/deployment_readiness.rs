use std::fs;
use std::path::PathBuf;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::{
    EnvelopeMode, GrammarReasonCode, GrammarState, ResidualSample, ResidualTrajectory,
};
use dsfb_semiotics_engine::live::{numeric_mode_label, OnlineStructuralEngine, Real};
use dsfb_semiotics_engine::math::envelope::{build_envelope, EnvelopeSpec};
use tempfile::TempDir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn simple_residual(values: &[f64]) -> ResidualTrajectory {
    ResidualTrajectory {
        scenario_id: "grammar".to_string(),
        channel_names: vec!["x".to_string()],
        samples: values
            .iter()
            .enumerate()
            .map(|(step, value)| ResidualSample {
                step,
                time: step as f64,
                values: vec![*value],
                norm: value.abs(),
            })
            .collect(),
    }
}

#[test]
fn test_grammar_report_type_exists() {
    let reason = GrammarReasonCode::Admissible;
    assert_eq!(format!("{reason:?}"), "Admissible");
}

#[test]
fn test_admissible_case_returns_admissible_report() {
    let residual = simple_residual(&[0.1, 0.12, 0.08]);
    let envelope = build_envelope(
        &residual,
        &EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        "grammar",
    );
    let grammar =
        dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer(&residual, &envelope);
    assert!(grammar
        .iter()
        .all(|status| status.reason_code == GrammarReasonCode::Admissible));
}

#[test]
fn test_outward_violation_returns_reasoned_report() {
    let residual = simple_residual(&[0.3, 0.75, 0.79, 0.83]);
    let envelope = build_envelope(
        &residual,
        &EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.8,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        "grammar",
    );
    let grammar =
        dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer(&residual, &envelope);
    assert_eq!(grammar.last().unwrap().state, GrammarState::Violation);
    assert!(matches!(
        grammar.last().unwrap().reason_code,
        GrammarReasonCode::SustainedOutwardDrift | GrammarReasonCode::EnvelopeViolation
    ));
}

#[test]
fn test_abrupt_slew_case_returns_reasoned_report() {
    let residual = simple_residual(&[0.1, 0.15, 1.4]);
    let envelope = build_envelope(
        &residual,
        &EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 0.9,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        "grammar",
    );
    let grammar =
        dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer(&residual, &envelope);
    assert_eq!(
        grammar.last().unwrap().reason_code,
        GrammarReasonCode::AbruptSlewViolation
    );
}

#[test]
fn test_grammar_report_exported_to_json_csv() {
    let temp = TempDir::new().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().to_path_buf()),
        ..Default::default()
    };
    let bundle =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_single(common, "nominal_stable"))
            .run_selected()
            .unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let grammar_csv = fs::read_to_string(exported.run_dir.join("csv/grammar_events.csv")).unwrap();
    let scenario_json =
        fs::read_to_string(exported.run_dir.join("json/scenario_outputs.json")).unwrap();
    assert!(grammar_csv.contains("reason_code"));
    assert!(grammar_csv.contains("reason_text"));
    assert!(scenario_json.contains("\"reason_code\""));
}

#[test]
fn test_report_includes_grammar_reason_text() {
    let temp = TempDir::new().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().to_path_buf()),
        ..Default::default()
    };
    let bundle =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_single(common, "abrupt_event"))
            .run_selected()
            .unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    assert!(report.contains("Grammar reason"));
}

#[test]
fn test_manifest_contains_history_buffer_capacity() {
    let common = CommonRunConfig::default();
    let bundle =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_single(common, "nominal_stable"))
            .run_selected()
            .unwrap();
    assert_eq!(
        bundle.run_metadata.online_history_buffer_capacity,
        bundle
            .run_metadata
            .engine_settings
            .online
            .history_buffer_capacity
    );
    assert_eq!(
        bundle.run_metadata.numeric_mode,
        bundle.run_metadata.engine_settings.online.numeric_mode
    );
}

#[test]
fn test_ffi_crate_exists() {
    assert!(crate_root().join("ffi/Cargo.toml").is_file());
}

#[test]
fn test_c_header_generated_or_present() {
    let header = crate_root().join("ffi/include/dsfb_semiotics_engine.h");
    assert!(header.is_file());
    let text = fs::read_to_string(header).unwrap();
    assert!(text.contains("dsfb_semiotics_engine_create"));
    assert!(text.contains("dsfb_semiotics_engine_current_status"));
}

#[test]
fn test_ffi_symbols_exported() {
    let source = fs::read_to_string(crate_root().join("ffi/src/lib.rs")).unwrap();
    assert!(source.contains("dsfb_semiotics_engine_create"));
    assert!(source.contains("dsfb_semiotics_engine_push_sample"));
    assert!(source.contains("dsfb_semiotics_engine_current_status"));
    assert!(source.contains("dsfb_semiotics_engine_reset"));
}

#[test]
fn test_ffi_examples_present() {
    assert!(crate_root().join("ffi/examples/minimal_ffi.c").is_file());
    assert!(crate_root().join("ffi/examples/minimal_ffi.cpp").is_file());
}

#[test]
fn test_synthetic_failure_injection_example_exists() {
    assert!(crate_root()
        .join("examples/synthetic_failure_injection.rs")
        .is_file());
}

#[test]
fn test_synthetic_failure_injection_example_documented() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    assert!(readme.contains("synthetic_failure_injection"));
    assert!(crate_root()
        .join("docs/examples/synthetic_failure_injection.md")
        .is_file());
}

#[test]
fn test_external_bank_examples_exist_for_multiple_mock_domains() {
    assert!(crate_root()
        .join("tests/fixtures/external_bank_minimal.json")
        .is_file());
    assert!(crate_root()
        .join("tests/fixtures/external_bank_mock_actuation.json")
        .is_file());
}

#[test]
fn test_feature_numeric_f32_declared_and_documented() {
    let cargo_toml = fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    assert!(cargo_toml.contains("numeric-f32"));
    assert!(readme.contains("numeric-f32"));
}

#[test]
fn test_online_engine_memory_history_bounded() {
    let mut settings = EngineSettings::default();
    settings.online.history_buffer_capacity = 5;
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "bounded",
        vec!["x".to_string()],
        1.0,
        EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        settings,
    )
    .unwrap();
    for step in 0..32 {
        engine
            .push_residual_sample(step as f64, &[(0.1 + step as f64 * 0.01) as Real])
            .unwrap();
    }
    assert_eq!(engine.online_history_len(), 5);
}

#[test]
fn test_numeric_mode_label_exposed() {
    assert!(matches!(numeric_mode_label(), "f32" | "f64"));
}
