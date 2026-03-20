use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::evaluation::sweeps::{SweepConfig, SweepFamily};
use serde::Deserialize;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct FigureIntegrityRecord {
    figure_id: String,
    png_present: bool,
    svg_present: bool,
    count_like_panels_integerlike: bool,
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
        bundle.scenario_outputs.len() * 4
    );
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
