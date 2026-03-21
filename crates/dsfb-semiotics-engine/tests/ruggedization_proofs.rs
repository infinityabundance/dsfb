use std::fs;
use std::path::PathBuf;

use dsfb_semiotics_engine::demos::{
    synthetic_failure_injection_trace, vibration_to_thermal_drift_trace,
};
use dsfb_semiotics_engine::engine::bank::{HeuristicBankMetadata, HeuristicBankRegistry};
use dsfb_semiotics_engine::engine::config::{BankRunConfig, BankValidationMode, CommonRunConfig};
use dsfb_semiotics_engine::engine::grammar_layer::evaluate_grammar_layer;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::{EngineSettings, SmoothingMode, SmoothingSettings};
use dsfb_semiotics_engine::engine::types::{
    AdmissibilityRequirement, EnvelopeMode, GrammarReasonCode, HeuristicBankEntry,
    HeuristicProvenance, HeuristicScopeConditions, ResidualSample, ResidualTrajectory,
};
use dsfb_semiotics_engine::math::derivatives::compute_drift_trajectory;
use dsfb_semiotics_engine::math::envelope::{build_envelope, EnvelopeSpec};
use dsfb_semiotics_engine::math::metrics::scalar_derivative;
use dsfb_semiotics_engine::math::smoothing::{smooth_residual_trajectory, smooth_scalar_series};
use serde_json::Value;
use tempfile::tempdir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path(name: &str) -> PathBuf {
    crate_root().join("tests").join("fixtures").join(name)
}

fn minimal_scope() -> HeuristicScopeConditions {
    HeuristicScopeConditions {
        min_outward_drift_fraction: None,
        max_outward_drift_fraction: None,
        min_inward_drift_fraction: None,
        max_inward_drift_fraction: None,
        max_curvature_energy: None,
        min_curvature_energy: None,
        max_curvature_onset_score: None,
        min_curvature_onset_score: None,
        min_directional_persistence: None,
        min_sign_consistency: None,
        min_channel_coherence: None,
        min_aggregate_monotonicity: None,
        max_aggregate_monotonicity: None,
        min_slew_spike_count: None,
        max_slew_spike_count: None,
        min_slew_spike_strength: None,
        max_slew_spike_strength: None,
        min_boundary_grazing_episodes: None,
        max_boundary_grazing_episodes: None,
        min_boundary_recovery_count: None,
        min_coordinated_group_breach_fraction: None,
        max_coordinated_group_breach_fraction: None,
        require_group_breach: false,
    }
}

fn residual(values: &[f64]) -> ResidualTrajectory {
    ResidualTrajectory {
        scenario_id: "fixture".to_string(),
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

fn export_single(
    scenario_id: &str,
    settings: EngineSettings,
    bank: Option<BankRunConfig>,
) -> (
    dsfb_semiotics_engine::engine::types::EngineOutputBundle,
    PathBuf,
) {
    let output_root = std::env::temp_dir().join(format!(
        "dsfb-semiotics-ruggedization-{}-{}",
        scenario_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let common = CommonRunConfig {
        output_root: Some(output_root),
        bank: bank.unwrap_or_default(),
        ..Default::default()
    };
    let engine = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(common, scenario_id),
        settings,
    )
    .unwrap();
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    (bundle, exported.run_dir)
}

#[test]
fn test_strict_validation_is_default() {
    assert!(CommonRunConfig::default().bank.is_strict());
}

#[test]
fn test_permissive_requires_explicit_opt_in() {
    let strict = CommonRunConfig::default();
    let permissive = CommonRunConfig {
        bank: BankRunConfig::builtin_with_mode(BankValidationMode::Permissive),
        ..Default::default()
    };
    assert!(strict.bank.is_strict());
    assert!(!permissive.bank.is_strict());
}

#[test]
fn test_reports_mark_permissive_runs_as_not_governance_clean() {
    let temp = tempdir().unwrap();
    let common = CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        bank: BankRunConfig::builtin_with_mode(BankValidationMode::Permissive),
        ..Default::default()
    };
    let bundle =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_single(common, "nominal_stable"))
            .run_selected()
            .unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();
    assert!(report.contains("not governance-clean"));
}

#[test]
fn test_directional_exception_schema_allows_only_explicit_exceptions() {
    let temp = tempdir().unwrap();
    let allowed = temp.path().join("allowed.json");
    let denied = temp.path().join("denied.json");
    let allowed_json = r#"{
  "metadata": { "schema_version": "dsfb-semiotics-engine-bank/v1", "bank_version": "allowed/v1", "note": "allowed" },
  "entries": [
    {
      "heuristic_id": "H-A",
      "motif_label": "a",
      "short_label": "a",
      "scope_conditions": {
        "min_outward_drift_fraction": null, "max_outward_drift_fraction": null,
        "min_inward_drift_fraction": null, "max_inward_drift_fraction": null,
        "max_curvature_energy": null, "min_curvature_energy": null,
        "max_curvature_onset_score": null, "min_curvature_onset_score": null,
        "min_directional_persistence": null, "min_sign_consistency": null,
        "min_channel_coherence": null, "min_aggregate_monotonicity": null,
        "max_aggregate_monotonicity": null, "min_slew_spike_count": null,
        "max_slew_spike_count": null, "min_slew_spike_strength": null,
        "max_slew_spike_strength": null, "min_boundary_grazing_episodes": null,
        "max_boundary_grazing_episodes": null, "min_boundary_recovery_count": null,
        "min_coordinated_group_breach_fraction": null, "max_coordinated_group_breach_fraction": null,
        "require_group_breach": false
      },
      "admissibility_requirements": "Any",
      "regime_tags": ["fixed"],
      "provenance": { "source": "fixture", "note": "fixture" },
      "applicability_note": "fixture",
      "retrieval_priority": 1,
      "compatible_with": [],
      "incompatible_with": ["H-B"],
      "directional_incompatibility_exceptions": ["H-B"]
    },
    {
      "heuristic_id": "H-B",
      "motif_label": "b",
      "short_label": "b",
      "scope_conditions": {
        "min_outward_drift_fraction": null, "max_outward_drift_fraction": null,
        "min_inward_drift_fraction": null, "max_inward_drift_fraction": null,
        "max_curvature_energy": null, "min_curvature_energy": null,
        "max_curvature_onset_score": null, "min_curvature_onset_score": null,
        "min_directional_persistence": null, "min_sign_consistency": null,
        "min_channel_coherence": null, "min_aggregate_monotonicity": null,
        "max_aggregate_monotonicity": null, "min_slew_spike_count": null,
        "max_slew_spike_count": null, "min_slew_spike_strength": null,
        "max_slew_spike_strength": null, "min_boundary_grazing_episodes": null,
        "max_boundary_grazing_episodes": null, "min_boundary_recovery_count": null,
        "min_coordinated_group_breach_fraction": null, "max_coordinated_group_breach_fraction": null,
        "require_group_breach": false
      },
      "admissibility_requirements": "Any",
      "regime_tags": ["fixed"],
      "provenance": { "source": "fixture", "note": "fixture" },
      "applicability_note": "fixture",
      "retrieval_priority": 1,
      "compatible_with": [],
      "incompatible_with": []
    }
  ]
}"#;
    let denied_json = allowed_json.replace(
        "\n      \"directional_incompatibility_exceptions\": [\"H-B\"]",
        "",
    );
    fs::write(&allowed, allowed_json).unwrap();
    fs::write(&denied, denied_json).unwrap();

    assert!(HeuristicBankRegistry::load_external_json(&allowed, true).is_ok());
    assert!(HeuristicBankRegistry::load_external_json(&denied, true).is_err());
}

#[test]
fn test_validation_artifact_contains_all_detected_violations() {
    let registry = HeuristicBankRegistry {
        metadata: HeuristicBankMetadata {
            schema_version: "dsfb-semiotics-engine-bank/v1".to_string(),
            bank_version: "invalid/v1".to_string(),
            note: "invalid".to_string(),
        },
        entries: vec![
            HeuristicBankEntry {
                heuristic_id: "H-DUP".to_string(),
                motif_label: "dup".to_string(),
                short_label: "dup".to_string(),
                scope_conditions: minimal_scope(),
                admissibility_requirements: AdmissibilityRequirement::Any,
                regime_tags: vec!["fixed".to_string()],
                provenance: HeuristicProvenance {
                    source: "".to_string(),
                    note: "".to_string(),
                },
                applicability_note: "".to_string(),
                retrieval_priority: 0,
                compatible_with: vec!["H-DUP".to_string()],
                incompatible_with: vec![],
                directional_incompatibility_exceptions: vec![],
            },
            HeuristicBankEntry {
                heuristic_id: "H-DUP".to_string(),
                motif_label: "dup".to_string(),
                short_label: "dup".to_string(),
                scope_conditions: minimal_scope(),
                admissibility_requirements: AdmissibilityRequirement::Any,
                regime_tags: vec!["fixed".to_string()],
                provenance: HeuristicProvenance {
                    source: "fixture".to_string(),
                    note: "fixture".to_string(),
                },
                applicability_note: "fixture".to_string(),
                retrieval_priority: 1,
                compatible_with: vec![],
                incompatible_with: vec!["H-MISSING".to_string()],
                directional_incompatibility_exceptions: vec![],
            },
        ],
    };
    let report = registry.validation_report();
    let joined = report.violations.join(" | ");
    assert!(joined.contains("H-DUP"));
    assert!(joined.contains("missing complete provenance"));
    assert!(joined.contains("unknown incompatible target"));
}

#[test]
fn test_smoother_configuration_exposed() {
    let settings = EngineSettings::default();
    assert!(!settings.smoothing.enabled());
    assert_eq!(settings.smoothing.mode, SmoothingMode::Disabled);
}

#[test]
fn test_smoothed_constant_signal_preserves_zero_drift() {
    let residual = residual(&[1.25, 1.25, 1.25, 1.25]);
    let smoothed = smooth_residual_trajectory(
        &residual,
        &SmoothingSettings {
            mode: SmoothingMode::ExponentialMovingAverage,
            exponential_alpha: 0.25,
        },
    );
    let drift = compute_drift_trajectory(&smoothed, 1.0, "constant");
    assert!(drift
        .samples
        .iter()
        .all(|sample| sample.norm.abs() <= 1.0e-12));
}

#[test]
fn test_smoothing_reduces_high_frequency_noise_fixture() {
    let raw = vec![0.0, 0.08, -0.08, 0.08, -0.08, 0.08];
    let smoothed = smooth_scalar_series(
        &raw,
        &SmoothingSettings {
            mode: SmoothingMode::ExponentialMovingAverage,
            exponential_alpha: 0.25,
        },
    );
    let times = (0..raw.len()).map(|index| index as f64).collect::<Vec<_>>();
    let raw_derivative = scalar_derivative(&raw, &times);
    let smoothed_derivative = scalar_derivative(&smoothed, &times);
    let raw_tv = raw_derivative
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>();
    let smooth_tv = smoothed_derivative
        .windows(2)
        .map(|window| (window[1] - window[0]).abs())
        .sum::<f64>();
    assert!(smooth_tv < raw_tv);
}

#[test]
fn test_smoothing_does_not_break_canonical_structural_case() {
    let strict_common = CommonRunConfig::default();
    let unsmoothed = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(strict_common.clone(), "gradual_degradation"),
        EngineSettings::default(),
    )
    .unwrap()
    .run_selected()
    .unwrap();
    let mut smoothed_settings = EngineSettings::default();
    smoothed_settings.smoothing.mode = SmoothingMode::ExponentialMovingAverage;
    smoothed_settings.smoothing.exponential_alpha = 0.28;
    let smoothed = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(strict_common, "gradual_degradation"),
        smoothed_settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();

    assert_eq!(
        format!("{:?}", unsmoothed.scenario_outputs[0].semantics.disposition),
        format!("{:?}", smoothed.scenario_outputs[0].semantics.disposition)
    );
}

#[test]
fn test_manifest_contains_smoothing_settings() {
    let mut settings = EngineSettings::default();
    settings.smoothing.mode = SmoothingMode::ExponentialMovingAverage;
    settings.smoothing.exponential_alpha = 0.22;
    let (_bundle, run_dir) = export_single("nominal_stable", settings, None);
    let manifest = serde_json::from_str::<Value>(
        &fs::read_to_string(run_dir.join("json/run_metadata.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest["engine_settings"]["smoothing"]["mode"],
        "exponential_moving_average"
    );
}

#[test]
fn test_trust_scalar_bounded_in_unit_interval() {
    let (bundle, _) = export_single("abrupt_event", EngineSettings::default(), None);
    for status in &bundle.scenario_outputs[0].grammar {
        let trust = status.trust_scalar.value();
        assert!((0.0..=1.0).contains(&trust));
    }
}

#[test]
fn test_grammar_severity_reduces_trust_monotonically_for_configured_fixture() {
    let residual = residual(&[0.1, 0.98, 1.5]);
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
        "fixture",
    );
    let grammar = evaluate_grammar_layer(&residual, &envelope);
    assert_eq!(grammar[0].reason_code, GrammarReasonCode::Admissible);
    assert_eq!(grammar[1].reason_code, GrammarReasonCode::Boundary);
    assert!(grammar[0].trust_scalar.value() > grammar[1].trust_scalar.value());
    assert!(grammar[1].trust_scalar.value() > grammar[2].trust_scalar.value());
}

#[test]
fn test_report_contains_operator_legible_comparator_section() {
    let (_bundle, run_dir) = export_single("nominal_stable", EngineSettings::default(), None);
    let report =
        fs::read_to_string(run_dir.join("report/dsfb_semiotics_engine_report.md")).unwrap();
    assert!(report.contains("EKF innovation monitoring"));
    assert!(report.contains("chi-squared-style gating"));
}

#[test]
fn test_indexed_retrieval_matches_linear_retrieval_for_fixture_bank() {
    let common = CommonRunConfig::default();
    let mut linear_settings = EngineSettings::default();
    linear_settings.retrieval_index.enabled = false;
    let linear = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(common.clone(), "outward_exit_case_a"),
        linear_settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();

    let mut indexed_settings = EngineSettings::default();
    indexed_settings.retrieval_index.enabled = true;
    indexed_settings.retrieval_index.minimum_bank_size = 1;
    let indexed = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(common, "outward_exit_case_a"),
        indexed_settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();

    assert_eq!(
        format!("{:?}", linear.scenario_outputs[0].semantics.disposition),
        format!("{:?}", indexed.scenario_outputs[0].semantics.disposition)
    );
    assert_eq!(
        linear.scenario_outputs[0].semantics.selected_heuristic_ids,
        indexed.scenario_outputs[0].semantics.selected_heuristic_ids
    );
}

#[test]
fn test_retrieval_latency_report_generated_if_enabled() {
    let mut settings = EngineSettings::default();
    settings.retrieval_index.enabled = true;
    settings.retrieval_index.minimum_bank_size = 1;
    let (_bundle, run_dir) = export_single("nominal_stable", settings, None);
    let rows = serde_json::from_str::<Vec<Value>>(
        &fs::read_to_string(run_dir.join("json/retrieval_latency_report.json")).unwrap(),
    )
    .unwrap();
    assert!(!rows.is_empty());
    assert!(rows[0].get("bank_size").is_some());
    assert!(rows[0].get("indexed_prefilter_candidate_count").is_some());
}

#[test]
fn test_small_bank_can_use_fallback_without_behavior_change() {
    let bank = BankRunConfig::external_with_mode(
        fixture_path("external_bank_minimal.json"),
        BankValidationMode::Strict,
    );
    let common = CommonRunConfig {
        bank: bank.clone(),
        ..Default::default()
    };
    let mut linear_settings = EngineSettings::default();
    linear_settings.retrieval_index.enabled = false;
    let linear = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(common.clone(), "nominal_stable"),
        linear_settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();
    let mut indexed_settings = EngineSettings::default();
    indexed_settings.retrieval_index.enabled = true;
    indexed_settings.retrieval_index.minimum_bank_size = 99;
    let indexed = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(common, "nominal_stable"),
        indexed_settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();

    assert_eq!(
        linear.scenario_outputs[0].semantics.selected_heuristic_ids,
        indexed.scenario_outputs[0].semantics.selected_heuristic_ids
    );
    assert!(indexed.scenario_outputs[0]
        .semantics
        .retrieval_audit
        .retrieval_path
        .starts_with("linear"));
}

#[test]
fn test_index_invalidates_or_rebuilds_when_bank_changes() {
    let mut settings = EngineSettings::default();
    settings.retrieval_index.enabled = true;
    settings.retrieval_index.minimum_bank_size = 1;
    let builtin = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(CommonRunConfig::default(), "nominal_stable"),
        settings.clone(),
    )
    .unwrap()
    .run_selected()
    .unwrap();
    let external = StructuralSemioticsEngine::with_settings(
        EngineConfig::synthetic_single(
            CommonRunConfig {
                bank: BankRunConfig::external_with_mode(
                    fixture_path("external_bank_mock_actuation.json"),
                    BankValidationMode::Strict,
                ),
                ..Default::default()
            },
            "nominal_stable",
        ),
        settings,
    )
    .unwrap()
    .run_selected()
    .unwrap();

    assert_ne!(
        builtin.run_metadata.bank.content_hash,
        external.run_metadata.bank.content_hash
    );
    assert_ne!(
        serde_json::to_string(&builtin.evaluation.retrieval_latency_report).unwrap(),
        serde_json::to_string(&external.evaluation.retrieval_latency_report).unwrap()
    );
}

#[test]
fn test_smoothing_sweep_runs() {
    let mut settings = EngineSettings::default();
    settings.smoothing.mode = SmoothingMode::ExponentialMovingAverage;
    settings.smoothing.exponential_alpha = 0.25;
    let (bundle, _) = export_single("noisy_structured", settings, None);
    assert!(!bundle.evaluation.smoothing_comparison_report.is_empty());
}

#[test]
fn test_retrieval_scaling_sweep_runs() {
    let mut settings = EngineSettings::default();
    settings.retrieval_index.enabled = true;
    settings.retrieval_index.minimum_bank_size = 1;
    settings.retrieval_index.benchmark_scaling_points = vec![8, 32, 96];
    let (bundle, _) = export_single("nominal_stable", settings.clone(), None);
    assert_eq!(
        bundle.evaluation.retrieval_latency_report.len(),
        settings.retrieval_index.benchmark_scaling_points.len()
    );
}

#[test]
fn test_ffi_returns_codes_not_only_strings() {
    let ffi_source = fs::read_to_string(crate_root().join("ffi/src/lib.rs")).unwrap();
    let header =
        fs::read_to_string(crate_root().join("ffi/include/dsfb_semiotics_engine.h")).unwrap();
    assert!(ffi_source.contains("semantic_disposition_code"));
    assert!(header.contains("typedef enum DsfbSemanticDisposition"));
}

#[test]
fn test_vibration_to_thermal_drift_example_exists() {
    assert!(crate_root()
        .join("examples/vibration_to_thermal_drift.rs")
        .is_file());
}

#[test]
fn test_vibration_to_thermal_drift_example_runs() {
    let trace = vibration_to_thermal_drift_trace().unwrap();
    assert!(trace.contains("Vibration to thermal drift trace"));
    assert!(trace.contains("mm/s"));
}

#[test]
fn test_synthetic_failure_injection_example_runs() {
    let trace = synthetic_failure_injection_trace().unwrap();
    assert!(trace.contains("Synthetic failure injection trace"));
}

#[test]
fn test_synthetic_failure_injection_example_prints_interpretation_trace() {
    let trace = synthetic_failure_injection_trace().unwrap();
    assert!(trace.contains("T+"));
    assert!(trace.contains("Semantic Interpretation"));
}

#[test]
fn test_readme_mentions_physical_units_and_vibration_example() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    assert!(readme.contains("vibration_to_thermal_drift"));
    assert!(readme.contains("millimeters/second"));
}

#[test]
fn test_docs_mention_vibration_example() {
    let text = fs::read_to_string(crate_root().join("docs/examples/vibration_to_thermal_drift.md"))
        .unwrap();
    assert!(text.contains("millimeters/second"));
}
