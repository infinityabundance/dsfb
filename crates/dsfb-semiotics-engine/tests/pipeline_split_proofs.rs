use std::fs;
use std::path::PathBuf;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    run_scenario, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::math::metrics::hash_serializable_hex;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_pipeline_module_split_exists() {
    assert!(crate_root().join("src/engine/pipeline.rs").is_file());
    assert!(crate_root().join("src/engine/pipeline_core.rs").is_file());
    assert!(crate_root()
        .join("src/engine/pipeline_artifacts.rs")
        .is_file());
    assert!(crate_root()
        .join("src/engine/pipeline_evaluation.rs")
        .is_file());
}

#[test]
fn test_pipeline_core_module_present() {
    let text = fs::read_to_string(crate_root().join("src/engine/pipeline_core.rs")).unwrap();
    assert!(text.contains("pipeline core orchestration"));
}

#[test]
fn test_pipeline_artifacts_module_present() {
    let text = fs::read_to_string(crate_root().join("src/engine/pipeline_artifacts.rs")).unwrap();
    assert!(text.contains("artifact assembly"));
    assert!(text.contains("figure-source exports"));
}

#[test]
fn test_pipeline_evaluation_module_present() {
    let text = fs::read_to_string(crate_root().join("src/engine/pipeline_evaluation.rs")).unwrap();
    assert!(text.contains("reproducibility aggregation"));
    assert!(text.contains("output comparison"));
}

#[test]
fn test_pipeline_module_responsibilities_documented() {
    let facade = fs::read_to_string(crate_root().join("src/engine/pipeline.rs")).unwrap();
    assert!(facade.contains("artifact assembly"));
    assert!(facade.contains("reproducibility aggregation"));
}

#[test]
fn test_pipeline_refactor_preserves_canonical_outputs() {
    let config = EngineConfig::synthetic_single(CommonRunConfig::default(), "nominal_stable");
    let direct = StructuralSemioticsEngine::new(config.clone())
        .run_selected()
        .unwrap();
    let facade = run_scenario(config, "nominal_stable").unwrap();

    assert_eq!(direct.scenario_outputs.len(), 1);
    assert_eq!(facade.scenario_outputs.len(), 1);
    assert_eq!(
        direct.scenario_outputs[0].syntax.trajectory_label,
        facade.scenario_outputs[0].syntax.trajectory_label
    );
    assert_eq!(
        format!("{:?}", direct.scenario_outputs[0].semantics.disposition),
        format!("{:?}", facade.scenario_outputs[0].semantics.disposition)
    );
    assert_eq!(
        direct.scenario_outputs[0].semantics.selected_heuristic_ids,
        facade.scenario_outputs[0].semantics.selected_heuristic_ids
    );
}

#[test]
fn test_pipeline_refactor_preserves_reproducibility_hashes() {
    let config = EngineConfig::synthetic_single(CommonRunConfig::default(), "nominal_stable");
    let first = StructuralSemioticsEngine::new(config.clone())
        .run_selected()
        .unwrap();
    let second = StructuralSemioticsEngine::new(config)
        .run_selected()
        .unwrap();

    assert_eq!(
        first.run_metadata.run_configuration_hash,
        second.run_metadata.run_configuration_hash
    );
    assert_eq!(
        hash_serializable_hex("scenario_outputs", &first.scenario_outputs)
            .unwrap()
            .fnv1a_64_hex,
        hash_serializable_hex("scenario_outputs", &second.scenario_outputs)
            .unwrap()
            .fnv1a_64_hex
    );
}
