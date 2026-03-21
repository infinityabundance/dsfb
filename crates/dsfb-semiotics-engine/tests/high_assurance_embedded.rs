use std::fs;
use std::path::PathBuf;
use std::process::Command;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingMode, SmoothingSettings};
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::live::{to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;
use dsfb_semiotics_engine::math::smoothing::smooth_scalar_series;
use tempfile::tempdir;

#[cfg(feature = "numeric-fixed")]
use dsfb_semiotics_engine::live::numeric_mode_label;
#[cfg(feature = "numeric-fixed")]
use dsfb_semiotics_engine::math::fixed_point::{
    fixed_point_overflow_policy, FIXED_POINT_FRACTIONAL_BITS, FIXED_POINT_NUMERIC_MODE,
};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme() -> String {
    fs::read_to_string(crate_root().join("README.md")).unwrap()
}

#[test]
fn test_fixed_point_feature_exists() {
    let cargo_toml = fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("numeric-fixed"));
}

#[test]
fn test_fixed_point_docs_explain_quantization_tradeoffs() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("q16.16"));
    assert!(docs.contains("saturating"));
    assert!(docs.contains("quantization"));
}

#[test]
fn test_safety_first_smoothing_profile_exists() {
    let profile = SmoothingSettings::safety_first();
    assert_eq!(profile.mode, SmoothingMode::SafetyFirst);
    assert_eq!(profile.profile_label(), "safety_first");
    assert!(profile.maximum_settling_samples() >= 1);
}

#[test]
fn test_safety_first_profile_exported_in_metadata() {
    let temp = tempdir().unwrap();
    let settings = EngineSettings {
        smoothing: SmoothingSettings::safety_first(),
        ..EngineSettings::default()
    };
    let bundle = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(
            CommonRunConfig {
                output_root: Some(temp.path().join("artifacts")),
                ..Default::default()
            },
            "gradual_degradation",
        ),
        settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let run_metadata = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(exported.manifest_path).unwrap(),
    )
    .unwrap();
    assert_eq!(
        run_metadata["numeric_mode"],
        bundle.run_metadata.numeric_mode.as_str()
    );
}

#[test]
fn test_safety_first_profile_reduces_high_frequency_noise_fixture() {
    let raw = vec![0.0, 0.08, -0.08, 0.08, -0.08, 0.08, -0.08];
    let default = smooth_scalar_series(
        &raw,
        &SmoothingSettings {
            mode: SmoothingMode::ExponentialMovingAverage,
            exponential_alpha: 0.25,
            causal_window: 5,
        },
    );
    let safety = smooth_scalar_series(&raw, &SmoothingSettings::safety_first());
    let default_tv = default
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>();
    let safety_tv = safety
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>();
    assert!(safety_tv <= default_tv);
}

#[test]
fn test_safety_first_profile_has_documented_lag_bound() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("estimated centroid lag"));
    assert!(docs.contains("maximum settling horizon"));
}

#[test]
fn test_safety_first_profile_does_not_break_canonical_structural_case() {
    let base = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "gradual_degradation",
    ))
    .run_selected()
    .unwrap();
    let settings = EngineSettings {
        smoothing: SmoothingSettings::safety_first(),
        ..EngineSettings::default()
    };
    let safety = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(CommonRunConfig::default(), "gradual_degradation"),
        settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();
    assert_eq!(
        format!("{:?}", base.scenario_outputs[0].semantics.disposition),
        format!("{:?}", safety.scenario_outputs[0].semantics.disposition)
    );
}

#[test]
fn test_ffi_batch_push_symbol_exists() {
    let header =
        fs::read_to_string(crate_root().join("ffi/include/dsfb_semiotics_engine.h")).unwrap();
    let source = fs::read_to_string(crate_root().join("ffi/src/lib.rs")).unwrap();
    assert!(header.contains("dsfb_semiotics_engine_push_sample_batch"));
    assert!(source.contains("dsfb_semiotics_engine_push_sample_batch"));
}

#[test]
fn test_ffi_batch_push_example_exists() {
    assert!(crate_root().join("ffi/examples/batch_ffi.c").is_file());
}

#[test]
fn test_docs_reference_batch_ingestion() {
    let docs = fs::read_to_string(crate_root().join("docs/examples/ffi_integration.md")).unwrap();
    assert!(docs.contains("push_sample_batch"));
    assert!(docs.contains("row-major"));
}

#[test]
fn test_timing_determinism_report_generation_exists() {
    let temp = tempdir().unwrap();
    let md = temp.path().join("timing.md");
    let json = temp.path().join("timing.json");
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-timing-determinism"))
        .args([
            "--iterations",
            "16",
            "--warmup",
            "4",
            "--output-md",
            md.to_str().unwrap(),
            "--output-json",
            json.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(md.is_file());
    assert!(json.is_file());
}

#[test]
fn test_benchmark_docs_explain_mean_and_tail_metrics() {
    let docs = fs::read_to_string(crate_root().join("docs/execution_budget.md")).unwrap();
    assert!(docs.contains("median"));
    assert!(docs.contains("p95"));
    assert!(docs.contains("p99"));
}

#[test]
fn test_readme_mentions_timing_determinism_report() {
    let docs = readme();
    assert!(docs.contains("Timing Determinism Report"));
}

#[test]
fn test_kani_config_exists() {
    assert!(crate_root().join("kani.toml").is_file());
}

#[test]
fn test_kani_harnesses_exist() {
    let harness = fs::read_to_string(crate_root().join("proofs/kani/trust_scalar.rs")).unwrap();
    assert!(harness.contains("proof_trust_scalar_in_unit_interval"));
    assert!(harness.contains("proof_trust_scalar_not_nan"));
}

#[test]
fn test_qa_includes_kani_or_proof_step() {
    let justfile = fs::read_to_string(crate_root().join("justfile")).unwrap();
    assert!(justfile.contains("proof-step"));
    assert!(justfile.contains("cargo kani"));
}

#[test]
fn test_docs_explain_verified_vs_tested_scope() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("not formally verified"));
    assert!(docs.contains("Kani harnesses currently target"));
}

#[test]
fn test_cargo_deny_config_exists() {
    assert!(crate_root().join("deny.toml").is_file());
}

#[test]
fn test_cargo_audit_docs_or_ci_step_exists() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("cargo-deny and cargo-audit guidance"));
    assert!(workflow.contains("cargo audit"));
}

#[test]
fn test_requirements_doc_exists() {
    assert!(crate_root().join("docs/REQUIREMENTS.md").is_file());
}

#[test]
fn test_requirements_doc_maps_claims_to_code_or_tests() {
    let docs = fs::read_to_string(crate_root().join("docs/REQUIREMENTS.md")).unwrap();
    assert!(docs.contains("REQ-BOUNDED-LIVE-001"));
    assert!(docs.contains("src/live/mod.rs"));
    assert!(docs.contains("tests/high_assurance_embedded.rs"));
}

#[test]
fn test_engine_state_snapshot_serialization_exists() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "snapshot_exists",
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
        EngineSettings::default(),
    )
    .unwrap();
    let _ = engine.push_residual_sample(0.0, &[to_real(0.1)]).unwrap();
    let bytes = engine.snapshot_binary().unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_engine_state_snapshot_roundtrip() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "snapshot_roundtrip",
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
        EngineSettings::default(),
    )
    .unwrap();
    for step in 0..4 {
        let _ = engine
            .push_residual_sample(step as f64, &[to_real(0.1 + step as f64 * 0.01)])
            .unwrap();
    }
    let restored =
        OnlineStructuralEngine::from_snapshot_binary(&engine.snapshot_binary().unwrap()).unwrap();
    assert_eq!(restored.online_history_len(), engine.online_history_len());
    assert_eq!(restored.history_capacity(), engine.history_capacity());
}

#[test]
fn test_single_step_replay_matches_original_transition() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "snapshot_step",
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
        EngineSettings::default(),
    )
    .unwrap();
    for step in 0..4 {
        let _ = engine
            .push_residual_sample(step as f64, &[to_real(0.1 + step as f64 * 0.01)])
            .unwrap();
    }
    let snapshot = engine.snapshot_binary().unwrap();
    let original = engine.push_residual_sample(4.0, &[to_real(0.22)]).unwrap();
    let temp = tempdir().unwrap();
    let snapshot_path = temp.path().join("state.dsfb");
    fs::write(&snapshot_path, snapshot).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dsfb-state-replay"))
        .args([
            "--snapshot-in",
            snapshot_path.to_str().unwrap(),
            "--sample-time",
            "4.0",
            "--sample-values",
            "0.22",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("step={}", original.step)));
    assert!(stdout.contains(&format!("syntax={}", original.syntax_label)));
}

#[test]
fn test_snapshot_versioning_documented() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("versioned snapshot"));
    assert!(docs.contains("state-exact replay"));
}

#[test]
fn test_docs_include_field_support_replay_workflow() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("Support workflow"));
    assert!(docs.contains("dsfb-state-replay"));
}

#[test]
fn test_readme_mentions_fixed_point_and_batch_ingestion() {
    let docs = readme();
    assert!(docs.contains("numeric-fixed"));
    assert!(docs.contains("batch ingestion"));
}

#[cfg(feature = "numeric-fixed")]
#[test]
fn test_canonical_online_scenario_runs_in_fixed_point_mode() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "fixed_point_online",
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
        EngineSettings::default(),
    )
    .unwrap();
    for step in 0..16 {
        let status = engine
            .push_residual_sample(step as f64, &[to_real(0.1 + step as f64 * 0.02)])
            .unwrap();
        assert_eq!(status.numeric_mode, FIXED_POINT_NUMERIC_MODE);
    }
    assert_eq!(numeric_mode_label(), FIXED_POINT_NUMERIC_MODE);
    assert_eq!(FIXED_POINT_FRACTIONAL_BITS, 16);
    assert!(fixed_point_overflow_policy().contains("saturating"));
}

#[cfg(feature = "numeric-fixed")]
#[test]
fn test_fixed_point_manifest_or_metadata_records_numeric_backend() {
    let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "nominal_stable",
    ))
    .run_selected()
    .unwrap();
    assert_eq!(bundle.run_metadata.numeric_mode, FIXED_POINT_NUMERIC_MODE);
}
