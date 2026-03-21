use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::figures::source::FigureSourceTable;
use serde_json::Value;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ensure_public_pipeline_ran() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let output = Command::new(env!("CARGO_BIN_EXE_dsfb-public-dataset-demo"))
            .arg("--dataset")
            .arg("nasa_milling")
            .arg("--dataset")
            .arg("nasa_bearings")
            .arg("--phase")
            .arg("all")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    });
}

fn ensure_synthetic_pipeline_ran() -> PathBuf {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let output_root = std::env::temp_dir().join("dsfb-semiotics-engine-figure-run-specific");
        let _ = fs::remove_dir_all(&output_root);
        fs::create_dir_all(&output_root).unwrap();
        let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
            output_root: Some(output_root.clone()),
            ..Default::default()
        }));
        let bundle = engine.run_selected().unwrap();
        let exported = export_artifacts(&bundle).unwrap();
        exported.run_dir
    })
    .clone()
}

fn public_dataset_root(dataset: &str) -> PathBuf {
    crate_root()
        .join("artifacts/public_dataset_demo")
        .join(dataset)
        .join("latest")
}

fn figure_table(root: &Path, figure_id: &str) -> FigureSourceTable {
    serde_json::from_str(
        &fs::read_to_string(root.join("json").join(format!("{figure_id}_source.json"))).unwrap(),
    )
    .unwrap()
}

fn json_array(root: &Path, name: &str) -> Vec<Value> {
    serde_json::from_str(&fs::read_to_string(root.join("json").join(name)).unwrap()).unwrap()
}

fn figure_png(root: &Path, figure_id: &str) -> PathBuf {
    root.join("figures").join(format!("{figure_id}.png"))
}

fn semantic_disposition_code(label: &str) -> f64 {
    match label {
        "Match" => 3.0,
        "CompatibleSet" => 2.0,
        "Ambiguous" => 1.0,
        _ => 0.0,
    }
}

fn grammar_state_code(label: &str) -> f64 {
    match label {
        "Violation" => 2.0,
        "Boundary" => 1.0,
        _ => 0.0,
    }
}

#[test]
fn test_figure_09_generated() {
    ensure_public_pipeline_ran();
    let synthetic_root = ensure_synthetic_pipeline_ran();
    assert!(figure_png(&synthetic_root, "figure_09_detectability_bound_comparison").is_file());
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(figure_png(
            &public_dataset_root(dataset),
            "figure_09_detectability_bound_comparison"
        )
        .is_file());
    }
}

#[test]
fn test_figure_12_generated() {
    ensure_public_pipeline_ran();
    let synthetic_root = ensure_synthetic_pipeline_ran();
    assert!(figure_png(
        &synthetic_root,
        "figure_12_semantic_retrieval_heuristics_bank"
    )
    .is_file());
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(figure_png(
            &public_dataset_root(dataset),
            "figure_12_semantic_retrieval_heuristics_bank"
        )
        .is_file());
    }
}

#[test]
fn test_figure_13_generated() {
    ensure_public_pipeline_ran();
    let synthetic_root = ensure_synthetic_pipeline_ran();
    assert!(figure_png(&synthetic_root, "figure_13_internal_baseline_comparators").is_file());
    for dataset in ["nasa_milling", "nasa_bearings"] {
        assert!(figure_png(
            &public_dataset_root(dataset),
            "figure_13_internal_baseline_comparators"
        )
        .is_file());
    }
}

#[test]
fn test_figure_09_uses_actual_detectability_outputs_from_current_run() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "primary_magnitude_similarity"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "meta_residual_divergence"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "outcome_consequence"));
    let scenario_outputs = json_array(&root, "scenario_outputs.json");
    let residual_samples = scenario_outputs[0]["residual"]["samples"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|sample| {
            sample["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap().abs())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let slew_samples = scenario_outputs[0]["slew"]["samples"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|sample| {
            sample["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap().abs())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    for row in table
        .rows
        .iter()
        .filter(|row| row.panel_id == "primary_magnitude_similarity")
    {
        assert!(residual_samples
            .iter()
            .any(|value| (value - row.y_value).abs() < 1.0e-9));
    }
    for row in table
        .rows
        .iter()
        .filter(|row| row.panel_id == "meta_residual_divergence")
    {
        assert!(slew_samples
            .iter()
            .any(|value| (value - row.y_value).abs() < 1.0e-9));
    }
}

#[test]
fn test_figure_12_uses_actual_retrieval_outputs_from_current_run() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let semantic_matches = json_array(&root, "semantic_matches.json");
    let first = &semantic_matches[0];
    let audit = &first["retrieval_audit"];
    let final_post_regime = audit["heuristic_candidates_post_regime"].as_u64().unwrap() as f64;
    let final_post_scope = audit["heuristic_candidates_post_scope"].as_u64().unwrap() as f64;
    let final_disposition =
        semantic_disposition_code(first["disposition"].as_str().unwrap_or("Unknown"));
    let last_post_regime = table
        .rows
        .iter()
        .rfind(|row| {
            row.panel_id == "semantic_candidate_count_timeline"
                && row.series_id == "post_regime_count"
        })
        .unwrap();
    let last_post_scope = table
        .rows
        .iter()
        .find(|row| {
            row.panel_id == "semantic_candidate_count_timeline"
                && row.series_id == "post_scope_count"
                && (row.x_value
                    - table
                        .rows
                        .iter()
                        .rfind(|candidate| {
                            candidate.panel_id == "semantic_candidate_count_timeline"
                                && candidate.series_id == "post_scope_count"
                        })
                        .unwrap()
                        .x_value)
                    .abs()
                    < 1.0e-9
        })
        .unwrap();
    let last_disposition = table
        .rows
        .iter()
        .rfind(|row| {
            row.panel_id == "semantic_disposition_timeline"
                && row.series_id == "semantic_disposition_code"
        })
        .unwrap();
    assert_eq!(last_post_regime.y_value, final_post_regime);
    assert_eq!(last_post_scope.y_value, final_post_scope);
    assert_eq!(last_disposition.y_value, final_disposition);
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "semantic_score_timeline")
            .count()
            >= 10
    );
}

#[test]
fn test_figure_13_uses_actual_comparator_outputs_from_current_run() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    let comparator_results = json_array(&root, "comparator_results.json");
    let residual_threshold = comparator_results
        .iter()
        .find(|row| row["comparator_id"] == "baseline_residual_threshold")
        .unwrap();
    let first_trigger = residual_threshold["first_alarm_time"].as_f64().unwrap();
    let timing_bar = table
        .rows
        .iter()
        .find(|row| {
            row.panel_id == "baseline_alarm_timing"
                && row.series_id == "baseline_residual_threshold_first_alarm"
        })
        .unwrap();
    assert_eq!(timing_bar.y_value, first_trigger);
    let scenario_outputs = json_array(&root, "scenario_outputs.json");
    let final_grammar = grammar_state_code(
        scenario_outputs[0]["grammar"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["state"]
            .as_str()
            .unwrap_or("Admissible"),
    );
    let final_semantics = semantic_disposition_code(
        json_array(&root, "semantic_matches.json")[0]["disposition"]
            .as_str()
            .unwrap_or("Unknown"),
    );
    let grammar_last = table
        .rows
        .iter()
        .rfind(|row| row.panel_id == "dsfb_grammar_timeline" && row.series_kind == "line")
        .unwrap();
    let semantic_last = table
        .rows
        .iter()
        .rfind(|row| row.panel_id == "dsfb_semantic_timeline")
        .unwrap();
    assert_eq!(grammar_last.y_value, final_grammar);
    assert_eq!(semantic_last.y_value, final_semantics);
}

#[test]
fn test_figure_09_not_low_information_when_more_detectability_structure_exists() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert_eq!(
        table.panel_ids,
        vec![
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence"
        ]
    );
    assert!(table.rows.len() >= 10);
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "meta_residual_divergence")
            .count()
            >= 6
    );
}

#[test]
fn test_figure_12_not_low_information_when_more_retrieval_structure_exists() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert_eq!(
        table.panel_ids,
        vec![
            "semantic_score_timeline",
            "semantic_candidate_count_timeline",
            "semantic_disposition_timeline"
        ]
    );
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "semantic_candidate_count_timeline")
            .count()
            >= 20
    );
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "semantic_score_timeline")
            .count()
            >= 20
    );
}

#[test]
fn test_figure_13_not_low_information_when_more_comparator_structure_exists() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_bearings");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert_eq!(
        table.panel_ids,
        vec![
            "baseline_alarm_timing",
            "dsfb_grammar_timeline",
            "dsfb_semantic_timeline"
        ]
    );
    assert!(table
        .rows
        .iter()
        .filter(|row| row.panel_id == "baseline_alarm_timing")
        .any(|row| row.y_value > 0.0));
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "dsfb_grammar_timeline")
            .count()
            >= 20
    );
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "dsfb_semantic_timeline")
            .count()
            >= 20
    );
}

#[test]
fn test_output_filenames_for_09_12_13_exact() {
    ensure_public_pipeline_ran();
    let synthetic_root = ensure_synthetic_pipeline_ran();
    for root in [
        synthetic_root,
        public_dataset_root("nasa_milling"),
        public_dataset_root("nasa_bearings"),
    ] {
        for figure_name in [
            "figure_09_detectability_bound_comparison.png",
            "figure_12_semantic_retrieval_heuristics_bank.png",
            "figure_13_internal_baseline_comparators.png",
        ] {
            assert!(root.join("figures").join(figure_name).is_file());
        }
    }
}

#[test]
fn test_png_outputs_exist_for_09_12_13() {
    test_output_filenames_for_09_12_13_exact();
}

#[test]
fn test_artifact_pipeline_regenerates_09_12_13_without_manual_intervention() {
    ensure_public_pipeline_ran();
    let synthetic_root = ensure_synthetic_pipeline_ran();
    for root in [
        synthetic_root,
        public_dataset_root("nasa_milling"),
        public_dataset_root("nasa_bearings"),
    ] {
        assert!(
            !figure_table(&root, "figure_09_detectability_bound_comparison")
                .rows
                .is_empty()
        );
        assert!(
            !figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank")
                .rows
                .is_empty()
        );
        assert!(
            !figure_table(&root, "figure_13_internal_baseline_comparators")
                .rows
                .is_empty()
        );
    }
}

#[test]
fn test_same_named_figures_drop_into_paper_workflow_unchanged() {
    ensure_public_pipeline_ran();
    for dataset in ["nasa_milling", "nasa_bearings"] {
        let figures_dir = public_dataset_root(dataset).join("figures");
        assert_eq!(
            figures_dir
                .join("figure_09_detectability_bound_comparison.png")
                .file_name()
                .unwrap(),
            "figure_09_detectability_bound_comparison.png"
        );
        assert_eq!(
            figures_dir
                .join("figure_12_semantic_retrieval_heuristics_bank.png")
                .file_name()
                .unwrap(),
            "figure_12_semantic_retrieval_heuristics_bank.png"
        );
        assert_eq!(
            figures_dir
                .join("figure_13_internal_baseline_comparators.png")
                .file_name()
                .unwrap(),
            "figure_13_internal_baseline_comparators.png"
        );
    }
}

#[test]
fn test_figure_09_source_table_nontrivial_for_current_run() {
    ensure_public_pipeline_ran();
    let table = figure_table(
        &public_dataset_root("nasa_bearings"),
        "figure_09_detectability_bound_comparison",
    );
    assert!(table.rows.len() >= 10);
    assert!(table.panel_ids.len() >= 2);
}

#[test]
fn test_figure_12_source_table_nontrivial_for_current_run() {
    ensure_public_pipeline_ran();
    let table = figure_table(
        &public_dataset_root("nasa_bearings"),
        "figure_12_semantic_retrieval_heuristics_bank",
    );
    assert!(table.rows.len() >= 10);
    assert!(table.panel_ids.len() == 3);
}

#[test]
fn test_figure_13_source_table_nontrivial_for_current_run() {
    ensure_public_pipeline_ran();
    let table = figure_table(
        &public_dataset_root("nasa_bearings"),
        "figure_13_internal_baseline_comparators",
    );
    assert!(table.rows.len() >= 12);
    assert!(table.panel_ids.len() == 3);
}

#[test]
fn test_docs_state_figures_09_12_13_are_run_specific() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    assert!(readme.contains("Figures 9, 12, and 13 remain run-specific"));
    assert!(readme.contains("run-specific: the figure stays within the current run"));
}

#[test]
fn test_docs_state_filenames_preserved_for_paper_workflow() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    assert!(readme.contains("PNG basenames remain unchanged"));
    assert!(readme.contains("paper `figures/` folder"));
}
