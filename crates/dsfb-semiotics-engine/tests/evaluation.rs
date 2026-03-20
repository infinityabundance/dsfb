use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::{BankRunConfig, CommonRunConfig};
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::evaluation::sweeps::{SweepConfig, SweepFamily};
use dsfb_semiotics_engine::io::schema::HEURISTIC_BANK_SCHEMA_VERSION;
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct FigureIntegrityRecord {
    figure_id: String,
    png_present: bool,
    svg_present: bool,
    count_like_panels_integerlike: bool,
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn asymmetric_bank_json() -> &'static str {
    r#"{
  "metadata": {
    "schema_version": "dsfb-semiotics-engine-bank/v1",
    "bank_version": "asymmetric-bank/v1",
    "note": "strict symmetry fixture"
  },
  "entries": [
    {
      "heuristic_id": "H-A",
      "motif_label": "a",
      "short_label": "a",
      "scope_conditions": {
        "min_outward_drift_fraction": null,
        "max_outward_drift_fraction": 0.6,
        "min_inward_drift_fraction": 0.4,
        "max_inward_drift_fraction": 0.6,
        "max_curvature_energy": 0.00001,
        "min_curvature_energy": null,
        "max_curvature_onset_score": 0.2,
        "min_curvature_onset_score": null,
        "min_directional_persistence": null,
        "min_sign_consistency": null,
        "min_channel_coherence": null,
        "min_aggregate_monotonicity": null,
        "max_aggregate_monotonicity": 0.08,
        "min_slew_spike_count": null,
        "max_slew_spike_count": 1,
        "min_slew_spike_strength": null,
        "max_slew_spike_strength": 0.001,
        "min_boundary_grazing_episodes": null,
        "max_boundary_grazing_episodes": 0,
        "min_boundary_recovery_count": null,
        "min_coordinated_group_breach_fraction": null,
        "max_coordinated_group_breach_fraction": 0.0,
        "require_group_breach": false
      },
      "admissibility_requirements": "NoViolation",
      "regime_tags": ["fixed"],
      "provenance": { "source": "fixture", "note": "fixture" },
      "applicability_note": "fixture",
      "retrieval_priority": 1,
      "compatible_with": ["H-B"],
      "incompatible_with": []
    },
    {
      "heuristic_id": "H-B",
      "motif_label": "b",
      "short_label": "b",
      "scope_conditions": {
        "min_outward_drift_fraction": null,
        "max_outward_drift_fraction": 0.6,
        "min_inward_drift_fraction": 0.4,
        "max_inward_drift_fraction": 0.6,
        "max_curvature_energy": 0.00001,
        "min_curvature_energy": null,
        "max_curvature_onset_score": 0.2,
        "min_curvature_onset_score": null,
        "min_directional_persistence": null,
        "min_sign_consistency": null,
        "min_channel_coherence": null,
        "min_aggregate_monotonicity": null,
        "max_aggregate_monotonicity": 0.08,
        "min_slew_spike_count": null,
        "max_slew_spike_count": 1,
        "min_slew_spike_strength": null,
        "max_slew_spike_strength": 0.001,
        "min_boundary_grazing_episodes": null,
        "max_boundary_grazing_episodes": 0,
        "min_boundary_recovery_count": null,
        "min_coordinated_group_breach_fraction": null,
        "max_coordinated_group_breach_fraction": 0.0,
        "require_group_breach": false
      },
      "admissibility_requirements": "NoViolation",
      "regime_tags": ["fixed"],
      "provenance": { "source": "fixture", "note": "fixture" },
      "applicability_note": "fixture",
      "retrieval_priority": 1,
      "compatible_with": [],
      "incompatible_with": []
    }
  ]
}"#
}

fn write_builtin_bank_json(path: &std::path::Path) {
    let registry = HeuristicBankRegistry::builtin();
    std::fs::write(path, serde_json::to_string_pretty(&registry).unwrap()).unwrap();
}

#[test]
fn builtin_bank_registry_validates_cleanly() {
    let registry = HeuristicBankRegistry::builtin();
    let report = registry.validate().unwrap();

    assert!(report.valid);
    assert!(report.duplicate_ids.is_empty());
    assert!(report.unknown_link_targets.is_empty());
}

#[test]
fn bank_registry_duplicate_detection_is_explicit() {
    let mut registry = HeuristicBankRegistry::builtin();
    let duplicate = registry.entries.first().cloned().unwrap();
    registry.entries.push(duplicate);

    let report = registry.validation_report();
    assert!(!report.valid);
    assert!(!report.duplicate_ids.is_empty());
}

#[test]
fn builtin_bank_registry_has_no_self_links_or_overlap() {
    let registry = HeuristicBankRegistry::builtin();

    for entry in &registry.entries {
        assert!(!entry.compatible_with.contains(&entry.heuristic_id));
        assert!(!entry.incompatible_with.contains(&entry.heuristic_id));
        for target in &entry.compatible_with {
            assert!(!entry.incompatible_with.contains(target));
        }
    }
}

#[test]
fn external_bank_fixture_loads_and_validates_strictly() {
    let (registry, descriptor, report) = HeuristicBankRegistry::load_external_json(
        fixture_path("external_bank_minimal.json").as_path(),
        true,
    )
    .unwrap();

    assert_eq!(
        registry.metadata.schema_version,
        HEURISTIC_BANK_SCHEMA_VERSION
    );
    assert_eq!(descriptor.source_kind.as_label(), "external");
    assert!(report.valid);
    assert!(report.strict_validation);
    assert_eq!(descriptor.validation_mode, "strict");
}

#[test]
fn external_bank_roundtrip_preserves_metadata_and_ids() {
    let (registry, _, _) = HeuristicBankRegistry::load_external_json(
        fixture_path("external_bank_minimal.json").as_path(),
        true,
    )
    .unwrap();
    let serialized = serde_json::to_string_pretty(&registry).unwrap();
    let reparsed: HeuristicBankRegistry = serde_json::from_str(&serialized).unwrap();

    assert_eq!(
        reparsed.metadata.schema_version,
        registry.metadata.schema_version
    );
    assert_eq!(
        reparsed.metadata.bank_version,
        registry.metadata.bank_version
    );
    assert_eq!(
        reparsed
            .entries
            .iter()
            .map(|entry| entry.heuristic_id.clone())
            .collect::<Vec<_>>(),
        registry
            .entries
            .iter()
            .map(|entry| entry.heuristic_id.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn malformed_external_bank_reports_validation_failure() {
    let temp = tempdir().unwrap();
    let bank_path = temp.path().join("bad_bank.json");
    std::fs::write(
        &bank_path,
        r#"{
  "metadata": {
    "schema_version": "dsfb-semiotics-engine-bank/v1",
    "bank_version": "bad-bank/v1",
    "note": "malformed"
  },
  "entries": [
    {
      "heuristic_id": "H-BAD",
      "motif_label": "bad",
      "short_label": "bad",
      "scope_conditions": {
        "min_outward_drift_fraction": 0.7,
        "max_outward_drift_fraction": 0.2,
        "min_inward_drift_fraction": null,
        "max_inward_drift_fraction": null,
        "max_curvature_energy": null,
        "min_curvature_energy": null,
        "max_curvature_onset_score": null,
        "min_curvature_onset_score": null,
        "min_directional_persistence": null,
        "min_sign_consistency": null,
        "min_channel_coherence": null,
        "min_aggregate_monotonicity": null,
        "max_aggregate_monotonicity": null,
        "min_slew_spike_count": null,
        "max_slew_spike_count": null,
        "min_slew_spike_strength": null,
        "max_slew_spike_strength": null,
        "min_boundary_grazing_episodes": null,
        "max_boundary_grazing_episodes": null,
        "min_boundary_recovery_count": null,
        "min_coordinated_group_breach_fraction": null,
        "max_coordinated_group_breach_fraction": null,
        "require_group_breach": false
      },
      "admissibility_requirements": "Any",
      "regime_tags": [],
      "provenance": {
        "source": "",
        "note": ""
      },
      "applicability_note": "",
      "retrieval_priority": 1,
      "compatible_with": ["H-MISSING"],
      "incompatible_with": ["H-MISSING"]
    }
  ]
}"#,
    )
    .unwrap();

    let error = HeuristicBankRegistry::load_external_json(bank_path.as_path(), true).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("failed validation"));
}

#[test]
fn strict_external_bank_validation_rejects_asymmetric_graph_links() {
    let temp = tempdir().unwrap();
    let bank_path = temp.path().join("asymmetric_bank.json");
    std::fs::write(&bank_path, asymmetric_bank_json()).unwrap();

    let strict_error =
        HeuristicBankRegistry::load_external_json(bank_path.as_path(), true).unwrap_err();
    assert!(strict_error.to_string().contains("failed validation"));

    let (_, _, report) =
        HeuristicBankRegistry::load_external_json(bank_path.as_path(), false).unwrap();
    assert!(report.valid);
    assert!(!report.missing_compatibility_links.is_empty());
    assert_eq!(report.validation_mode, "permissive");
}

#[test]
fn builtin_and_external_bank_with_same_content_produce_same_semantics() {
    let temp = tempdir().unwrap();
    let external_bank = temp.path().join("builtin_equivalent_bank.json");
    write_builtin_bank_json(&external_bank);

    let builtin_engine =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
            output_root: Some(temp.path().join("builtin_artifacts")),
            ..Default::default()
        }));
    let external_engine =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
            output_root: Some(temp.path().join("external_artifacts")),
            bank: BankRunConfig::external(external_bank, false),
            ..Default::default()
        }));

    let builtin_bundle = builtin_engine.run_selected().unwrap();
    let external_bundle = external_engine.run_selected().unwrap();

    let builtin = builtin_bundle
        .scenario_outputs
        .iter()
        .map(|scenario| {
            (
                scenario.record.id.clone(),
                (
                    format!("{:?}", scenario.semantics.disposition),
                    scenario.semantics.selected_heuristic_ids.clone(),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let external = external_bundle
        .scenario_outputs
        .iter()
        .map(|scenario| {
            (
                scenario.record.id.clone(),
                (
                    format!("{:?}", scenario.semantics.disposition),
                    scenario.semantics.selected_heuristic_ids.clone(),
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    assert_eq!(builtin, external);
}

#[test]
fn permissive_external_bank_run_exports_graph_violations_clearly() {
    let temp = tempdir().unwrap();
    let bank_path = temp.path().join("asymmetric_bank.json");
    std::fs::write(&bank_path, asymmetric_bank_json()).unwrap();

    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig {
            output_root: Some(temp.path().join("artifacts")),
            bank: BankRunConfig::external(bank_path, false),
            ..Default::default()
        },
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let validation_json =
        std::fs::read_to_string(exported.run_dir.join("json/heuristic_bank_validation.json"))
            .unwrap();
    let descriptor_json = std::fs::read_to_string(
        exported
            .run_dir
            .join("json/loaded_heuristic_bank_descriptor.json"),
    )
    .unwrap();

    assert!(validation_json.contains("\"validation_mode\": \"permissive\""));
    assert!(validation_json.contains("missing_compatibility_links"));
    assert!(descriptor_json.contains("\"validation_mode\": \"permissive\""));
}

#[test]
fn evaluation_summary_and_baselines_cover_every_scenario() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        ..Default::default()
    }));
    let bundle = engine.run_all().unwrap();

    assert_eq!(
        bundle.evaluation.summary.scenario_count,
        bundle.scenario_outputs.len()
    );
    assert_eq!(
        bundle.evaluation.baseline_results.len(),
        bundle.scenario_outputs.len() * 6
    );
    let comparator_ids = bundle
        .evaluation
        .baseline_results
        .iter()
        .map(|result| result.comparator_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(comparator_ids.contains("baseline_residual_threshold"));
    assert!(comparator_ids.contains("baseline_moving_average_trend"));
    assert!(comparator_ids.contains("baseline_slew_spike"));
    assert!(comparator_ids.contains("baseline_envelope_interaction"));
    assert!(comparator_ids.contains("baseline_cusum"));
    assert!(comparator_ids.contains("baseline_innovation_chi_squared_style"));
    assert!(bundle.evaluation.summary.all_reproducible);
    assert!(bundle.evaluation.bank_validation.valid);
}

#[test]
fn sweep_mode_produces_stable_evaluation_outputs() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::sweep(
        CommonRunConfig {
            output_root: Some(temp.path().join("artifacts")),
            ..Default::default()
        },
        SweepConfig {
            family: SweepFamily::GradualDriftSlope,
            points: 4,
        },
    ));
    let bundle = engine.run_selected().unwrap();

    assert_eq!(bundle.run_metadata.input_mode, "synthetic-sweep");
    assert_eq!(bundle.scenario_outputs.len(), 4);
    assert_eq!(bundle.evaluation.sweep_results.len(), 4);
    assert_eq!(
        bundle
            .evaluation
            .sweep_summary
            .as_ref()
            .unwrap()
            .sweep_family,
        "gradual_drift_slope"
    );
}

#[test]
fn export_writes_evaluation_and_artifact_completeness_outputs() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig {
            output_root: Some(temp.path().join("artifacts")),
            ..Default::default()
        },
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();

    assert!(exported
        .run_dir
        .join("csv/evaluation_summary.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("csv/baseline_comparators.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("csv/artifact_completeness.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("csv/figure_12_semantic_retrieval_source.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("csv/figure_01_residual_prediction_observation_overview_source.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("json/figure_10_deterministic_pipeline_flow_source.json")
        .is_file());
    assert!(exported
        .run_dir
        .join("json/figure_integrity_checks.json")
        .is_file());
    assert!(exported
        .run_dir
        .join("json/heuristic_bank_validation.json")
        .is_file());
}

#[test]
fn figure_integrity_covers_all_rendered_publication_figures() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig {
            output_root: Some(temp.path().join("artifacts")),
            ..Default::default()
        },
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let integrity =
        std::fs::read_to_string(exported.run_dir.join("json/figure_integrity_checks.json"))
            .unwrap();
    let rows: Vec<FigureIntegrityRecord> = serde_json::from_str(&integrity).unwrap();

    assert_eq!(rows.len(), 13);
    assert!(rows.iter().all(|row| row.png_present && row.svg_present));
    assert!(
        rows.iter()
            .find(|row| row.figure_id == "figure_12_semantic_retrieval_heuristics_bank")
            .unwrap()
            .count_like_panels_integerlike
    );
    assert!(exported
        .run_dir
        .join("csv/figure_09_detectability_bound_comparison_source.csv")
        .is_file());
    assert!(exported
        .run_dir
        .join("csv/figure_10_deterministic_pipeline_flow_source.csv")
        .is_file());
}

#[test]
fn external_bank_pipeline_run_records_external_source_in_manifest() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig {
            output_root: Some(temp.path().join("artifacts")),
            bank: BankRunConfig::external(fixture_path("external_bank_minimal.json"), true),
            ..Default::default()
        },
        "nominal_stable",
    ));
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let manifest = std::fs::read_to_string(&exported.manifest_path).unwrap();

    assert_eq!(bundle.run_metadata.bank.source_kind.as_label(), "external");
    assert!(manifest.contains("\"source_kind\": \"external\""));
    assert!(exported
        .run_dir
        .join("json/loaded_heuristic_bank_descriptor.json")
        .is_file());
}
