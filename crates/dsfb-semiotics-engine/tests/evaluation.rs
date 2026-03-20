use dsfb_semiotics_engine::engine::bank::HeuristicBankRegistry;
use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::evaluation::sweeps::{SweepConfig, SweepFamily};
use tempfile::tempdir;

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
        .join("json/heuristic_bank_validation.json")
        .is_file());
}
