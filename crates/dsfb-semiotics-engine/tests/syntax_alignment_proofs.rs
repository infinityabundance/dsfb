use std::fs;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use tempfile::tempdir;

#[test]
fn test_oscillatory_bounded_not_generic_when_evidence_strong() {
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "oscillatory_bounded",
    ));
    let bundle = engine.run_selected().unwrap();
    let scenario = bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == "oscillatory_bounded")
        .unwrap();

    assert_eq!(
        scenario.syntax.trajectory_label,
        "bounded-oscillatory-structured"
    );
}

#[test]
fn test_structured_noisy_not_generic_when_evidence_strong() {
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
        CommonRunConfig::default(),
        "noisy_structured",
    ));
    let bundle = engine.run_selected().unwrap();
    let scenario = bundle
        .scenario_outputs
        .iter()
        .find(|scenario| scenario.record.id == "noisy_structured")
        .unwrap();

    assert_eq!(
        scenario.syntax.trajectory_label,
        "structured-noisy-admissible"
    );
}

#[test]
fn test_report_explains_mixed_structured_noncommitment() {
    let temp = tempdir().unwrap();
    let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
        output_root: Some(temp.path().join("artifacts")),
        ..Default::default()
    }));
    let bundle = engine.run_selected().unwrap();
    let exported = export_artifacts(&bundle).unwrap();
    let report = fs::read_to_string(exported.report_markdown).unwrap();

    assert!(report.contains(
        "Labels such as `weakly-structured-baseline-like` and `mixed-structured` remain conservative summaries rather than health judgments."
    ));
}
