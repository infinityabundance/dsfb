use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tempfile::tempdir;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};

#[derive(Serialize)]
struct CanonicalScenarioSnapshot {
    scenario_id: String,
    syntax_label: String,
    semantic_disposition: String,
    selected_heuristic_ids: Vec<String>,
}

#[derive(Serialize)]
struct CanonicalSnapshot {
    input_mode: String,
    evaluation_dispositions: BTreeMap<String, usize>,
    scenarios: Vec<CanonicalScenarioSnapshot>,
}

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(name)
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_path(name);
    if std::env::var_os("DSFB_UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, format!("{actual}\n")).unwrap();
    }
    let expected = fs::read_to_string(path).unwrap();
    assert_eq!(actual, expected.trim_end());
}

#[test]
fn canonical_snapshot_remains_stable() {
    let engine =
        StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig::default()));
    let bundle = engine.run_all().unwrap();
    let scenarios = ["nominal_stable", "abrupt_event", "outward_exit_case_a"]
        .into_iter()
        .map(|scenario_id| {
            let scenario = bundle
                .scenario_outputs
                .iter()
                .find(|scenario| scenario.record.id == scenario_id)
                .unwrap();
            CanonicalScenarioSnapshot {
                scenario_id: scenario.record.id.clone(),
                syntax_label: scenario.syntax.trajectory_label.clone(),
                semantic_disposition: format!("{:?}", scenario.semantics.disposition),
                selected_heuristic_ids: scenario.semantics.selected_heuristic_ids.clone(),
            }
        })
        .collect::<Vec<_>>();
    let snapshot = CanonicalSnapshot {
        input_mode: bundle.run_metadata.input_mode.clone(),
        evaluation_dispositions: bundle
            .evaluation
            .summary
            .semantic_disposition_counts
            .clone(),
        scenarios,
    };
    let actual = serde_json::to_string_pretty(&snapshot).unwrap();

    assert_snapshot("canonical_snapshot.json", &actual);
}

#[test]
fn canonical_csv_exports_remain_stable() {
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
    let evaluation_summary =
        fs::read_to_string(exported.run_dir.join("csv/evaluation_summary.csv")).unwrap();
    let semantic_matches =
        fs::read_to_string(exported.run_dir.join("csv/semantic_matches.csv")).unwrap();
    let semantic_retrieval_source = fs::read_to_string(
        exported
            .run_dir
            .join("csv/figure_12_semantic_retrieval_source.csv"),
    )
    .unwrap();

    assert_snapshot(
        "nominal_evaluation_summary.csv",
        evaluation_summary.trim_end(),
    );
    assert_snapshot("nominal_semantic_matches.csv", semantic_matches.trim_end());
    assert_snapshot(
        "nominal_figure_12_semantic_retrieval_source.csv",
        semantic_retrieval_source.trim_end(),
    );
}
