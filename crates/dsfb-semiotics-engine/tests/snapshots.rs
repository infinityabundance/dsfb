use std::collections::BTreeMap;

use serde::Serialize;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{EngineConfig, StructuralSemioticsEngine};

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
    let expected = include_str!("snapshots/canonical_snapshot.json");

    assert_eq!(actual, expected.trim_end());
}
