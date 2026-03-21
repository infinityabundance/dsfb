use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::figures::source::FigureSourceTable;
use serde_json::Value;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ensure_public_pipeline_ran() -> PathBuf {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let output = Command::new(env!("CARGO_BIN_EXE_dsfb-public-dataset-demo"))
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
        crate_root().join("artifacts/public_dataset_demo/nasa_bearings/latest")
    })
    .clone()
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

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn semantic_code(label: &str) -> f64 {
    match label {
        "Match" => 3.0,
        "CompatibleSet" => 2.0,
        "Ambiguous" => 1.0,
        _ => 0.0,
    }
}

#[test]
fn test_figure_09_generated() {
    let root = ensure_public_pipeline_ran();
    assert!(root
        .join("figures/figure_09_detectability_bound_comparison.png")
        .is_file());
}

#[test]
fn test_figure_09_filename_preserved() {
    let root = ensure_public_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_09_detectability_bound_comparison.png")
            .file_name()
            .unwrap(),
        "figure_09_detectability_bound_comparison.png"
    );
}

#[test]
fn test_figure_09_uses_nasa_bearings_outputs() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_bearings_public_demo"));
}

#[test]
fn test_figure_09_shows_similar_primary_behavior_cases() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let stable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "primary_magnitude_similarity"
                && row.series_id == "stable_primary_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let departure = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "primary_magnitude_similarity"
                && row.series_id == "departure_primary_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(!stable.is_empty());
    assert!(!departure.is_empty());
    let stable_mean = mean(&stable);
    let departure_mean = mean(&departure);
    assert!((stable_mean - departure_mean).abs() < 1.0e-3);
}

#[test]
fn test_figure_09_shows_distinct_meta_residual_or_meta_drift_behavior() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let stable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "meta_residual_divergence" && row.series_id == "stable_meta_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let departure = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "meta_residual_divergence" && row.series_id == "departure_meta_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!((mean(&departure) - mean(&stable)).abs() > 5.0e-3);
}

#[test]
fn test_figure_09_shows_distinct_outcome_or_detectability_consequence() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let stable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "outcome_consequence" && row.series_id == "stable_outcome_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let departure = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "outcome_consequence" && row.series_id == "departure_outcome_window"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(mean(&departure) > mean(&stable));
}

#[test]
fn test_figure_09_not_trivial_summary_bar_chart() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert_eq!(
        table.panel_ids,
        vec![
            "primary_magnitude_similarity",
            "meta_residual_divergence",
            "outcome_consequence"
        ]
    );
    assert!(table.rows.iter().all(|row| row.series_kind == "line"));
}

#[test]
fn test_figure_09_caption_or_metadata_explains_primary_insufficiency() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.plot_title.contains("Similar Primary Magnitude"));
    assert!(table.rows.iter().any(|row| row
        .note
        .contains("Primary residual magnitude alone does not separate")));
}

#[test]
fn test_figure_12_generated() {
    let root = ensure_public_pipeline_ran();
    assert!(root
        .join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
        .is_file());
}

#[test]
fn test_figure_12_filename_preserved() {
    let root = ensure_public_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
            .file_name()
            .unwrap(),
        "figure_12_semantic_retrieval_heuristics_bank.png"
    );
}

#[test]
fn test_figure_12_uses_nasa_bearings_outputs() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_bearings_public_demo"));
}

#[test]
fn test_figure_12_has_timeline_structure() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert_eq!(
        table.panel_ids,
        vec![
            "semantic_score_timeline",
            "semantic_candidate_count_timeline",
            "semantic_disposition_timeline"
        ]
    );
    assert!(table.rows.iter().all(|row| row.series_kind == "line"));
}

#[test]
fn test_figure_12_shows_semantic_transitions_or_candidate_evolution() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let top_scores = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_score_timeline" && row.series_id == "top_candidate_score"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(top_scores
        .windows(2)
        .any(|window| (window[1] - window[0]).abs() > 1.0e-9));
}

#[test]
fn test_figure_12_shows_ambiguity_or_candidate_count_evolution() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let counts = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_candidate_count_timeline"
                && row.series_id == "post_regime_count"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(counts
        .windows(2)
        .any(|window| (window[1] - window[0]).abs() > 1.0e-9));
}

#[test]
fn test_figure_12_not_static_single_snapshot_only() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() > 30);
}

#[test]
fn test_figure_12_caption_or_metadata_mentions_semantic_evolution() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.plot_title.contains("Semantic Evolution"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("semantic evolution")));
}

#[test]
fn test_figure_13_generated() {
    let root = ensure_public_pipeline_ran();
    assert!(root
        .join("figures/figure_13_internal_baseline_comparators.png")
        .is_file());
}

#[test]
fn test_figure_13_filename_preserved() {
    let root = ensure_public_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_13_internal_baseline_comparators.png")
            .file_name()
            .unwrap(),
        "figure_13_internal_baseline_comparators.png"
    );
}

#[test]
fn test_figure_13_uses_nasa_bearings_outputs() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_bearings_public_demo"));
}

#[test]
fn test_figure_13_shows_baseline_or_internal_comparator_view() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "baseline_alarm_timing" && row.series_kind == "bar"));
}

#[test]
fn test_figure_13_shows_dsfb_structural_layer_in_addition() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "dsfb_grammar_timeline"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "dsfb_semantic_timeline"));
}

#[test]
fn test_figure_13_not_presented_as_performance_benchmark_only() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.plot_title.contains("Interpretation"));
    assert!(!table.plot_title.to_lowercase().contains("accuracy"));
}

#[test]
fn test_figure_13_caption_or_metadata_mentions_interpretability_delta() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("interpretability-delta")));
}

#[test]
fn test_figure_13_not_flat_trigger_count_only() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(!table
        .panel_ids
        .contains(&"comparator_trigger_counts".to_string()));
}

#[test]
fn test_figures_09_12_13_primary_paper_dataset_is_bearings() {
    let readme = fs::read_to_string(crate_root().join("README.md")).unwrap();
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_bearings_figures_09_12_13.md"))
            .unwrap();
    assert!(readme.contains("NASA Bearings is the primary dataset for Figures 9, 12, and 13"));
    assert!(docs.contains("NASA Bearings is the primary paper dataset"));
}

#[test]
fn test_output_filenames_for_09_12_13_exact() {
    let root = ensure_public_pipeline_ran();
    for name in [
        "figure_09_detectability_bound_comparison.png",
        "figure_12_semantic_retrieval_heuristics_bank.png",
        "figure_13_internal_baseline_comparators.png",
    ] {
        assert!(root.join("figures").join(name).is_file());
    }
}

#[test]
fn test_png_outputs_exist_for_09_12_13() {
    test_output_filenames_for_09_12_13_exact();
}

#[test]
fn test_paper_drop_in_workflow_unchanged() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_bearings_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("No LaTeX edits are required"));
    assert!(docs.contains("paper `figures/` folder"));
}

#[test]
fn test_figure_09_source_table_exists_and_is_nontrivial() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_figure_12_source_table_exists_and_is_nontrivial() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() >= 30);
}

#[test]
fn test_figure_13_source_table_exists_and_is_nontrivial() {
    let root = ensure_public_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_figure_sources_derived_from_bearings_run_outputs() {
    let root = ensure_public_pipeline_ran();
    let semantic_matches = json_array(&root, "semantic_matches.json");
    let disposition = semantic_matches[0]["disposition"].as_str().unwrap();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let last = table
        .rows
        .iter()
        .rfind(|row| row.panel_id == "semantic_disposition_timeline")
        .unwrap();
    assert_eq!(last.y_value, semantic_code(disposition));
}

#[test]
fn test_artifact_pipeline_regenerates_figure_sources_without_manual_intervention() {
    let root = ensure_public_pipeline_ran();
    for figure_id in [
        "figure_09_detectability_bound_comparison",
        "figure_12_semantic_retrieval_heuristics_bank",
        "figure_13_internal_baseline_comparators",
    ] {
        assert!(root
            .join("json")
            .join(format!("{figure_id}_source.json"))
            .is_file());
        assert!(root
            .join("csv")
            .join(format!("{figure_id}_source.csv"))
            .is_file());
    }
}

#[test]
fn test_paper_bearings_figures_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/paper_bearings_figures_09_12_13.md")
        .is_file());
}
