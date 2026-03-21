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
    let scenario_outputs = json_array(&root, "scenario_outputs.json");
    let observed = scenario_outputs[0]["detectability"]["observed_crossing_time"]
        .as_f64()
        .unwrap();
    let segment = table
        .rows
        .iter()
        .find(|row| {
            row.panel_id == "detectability_context" && row.series_id == "first_boundary_time"
        })
        .unwrap();
    assert!((segment.x_value - observed).abs() < 1.0e-9);
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "detectability_context" && row.series_id == "residual_norm"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "detectability_context" && row.series_id == "envelope_radius"));
}

#[test]
fn test_figure_12_uses_actual_retrieval_outputs_from_current_run() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let semantic_matches = json_array(&root, "semantic_matches.json");
    let first = &semantic_matches[0];
    let audit = &first["retrieval_audit"];
    let prefilter = audit["prefilter_candidate_count"].as_u64().unwrap() as f64;
    let post_regime = audit["heuristic_candidates_post_regime"].as_u64().unwrap() as f64;
    let funnel_prefilter = table
        .rows
        .iter()
        .find(|row| row.panel_id == "retrieval_filter_funnel" && row.series_id == "prefilter_count")
        .unwrap();
    let funnel_regime = table
        .rows
        .iter()
        .find(|row| {
            row.panel_id == "retrieval_filter_funnel" && row.series_id == "post_regime_count"
        })
        .unwrap();
    assert_eq!(funnel_prefilter.y_value, prefilter);
    assert_eq!(funnel_regime.y_value, post_regime);
    let ranked_candidates = audit["ranked_candidates_post_regime"]
        .as_array()
        .unwrap()
        .len();
    assert!(table
        .rows
        .iter()
        .filter(|row| row.panel_id == "post_regime_candidate_scores" && row.series_kind == "bar")
        .count()
        >= ranked_candidates.min(4));
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
            row.panel_id == "comparator_first_trigger_time"
                && row.series_id == "baseline_residual_threshold_first_trigger_time"
        })
        .unwrap();
    assert_eq!(timing_bar.y_value, first_trigger);
}

#[test]
fn test_figure_09_not_low_information_when_more_detectability_structure_exists() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert_eq!(
        table.panel_ids,
        vec!["detectability_context", "detectability_window_ratio"]
    );
    assert!(table.rows.len() >= 10);
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "detectability_window_ratio" && row.series_kind == "bar")
            .count()
            >= 4
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
            "post_regime_candidate_scores",
            "retrieval_filter_funnel",
            "retrieval_stage_rejections"
        ]
    );
    assert!(
        table
            .rows
            .iter()
            .filter(|row| row.panel_id == "retrieval_filter_funnel" && row.series_kind == "bar")
            .count()
            >= 5
    );
    assert!(table
        .rows
        .iter()
        .filter(|row| row.panel_id == "post_regime_candidate_scores" && row.series_kind == "bar")
        .count()
        >= 3);
}

#[test]
fn test_figure_13_not_low_information_when_more_comparator_structure_exists() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert_eq!(
        table.panel_ids,
        vec![
            "comparator_first_trigger_time",
            "comparator_onset_rank",
            "comparator_trigger_counts"
        ]
    );
    assert!(table
        .rows
        .iter()
        .filter(|row| row.panel_id == "comparator_first_trigger_time")
        .any(|row| row.y_value > 0.0));
    assert!(table
        .rows
        .iter()
        .filter(|row| row.panel_id == "comparator_onset_rank")
        .any(|row| row.y_value > 0.0));
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
        &public_dataset_root("nasa_milling"),
        "figure_13_internal_baseline_comparators",
    );
    assert!(table.rows.len() >= 18);
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
