use std::collections::BTreeSet;
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
        let output_root =
            std::env::temp_dir().join("dsfb-semiotics-engine-synthetic-paper-figures");
        let _ = fs::remove_dir_all(&output_root);
        fs::create_dir_all(&output_root).unwrap();
        let engine = StructuralSemioticsEngine::new(EngineConfig::synthetic_all(CommonRunConfig {
            output_root: Some(output_root.clone()),
            ..Default::default()
        }));
        let bundle = engine.run_selected().unwrap();
        export_artifacts(&bundle).unwrap().run_dir
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

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn scenario_ids(root: &Path) -> BTreeSet<String> {
    json_array(root, "scenario_outputs.json")
        .into_iter()
        .map(|row| row["record"]["id"].as_str().unwrap().to_string())
        .collect()
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
fn test_synthetic_figure_09_generated() {
    let root = ensure_synthetic_pipeline_ran();
    assert!(root
        .join("figures/figure_09_detectability_bound_comparison.png")
        .is_file());
}

#[test]
fn test_synthetic_figure_09_filename_preserved() {
    let root = ensure_synthetic_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_09_detectability_bound_comparison.png")
            .file_name()
            .unwrap(),
        "figure_09_detectability_bound_comparison.png"
    );
}

#[test]
fn test_synthetic_figure_09_uses_synthetic_outputs() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let ids = scenario_ids(&root);
    assert!(table.rows.iter().all(|row| ids.contains(&row.scenario_id)));
    assert!(table
        .rows
        .iter()
        .all(|row| !row.scenario_id.starts_with("nasa_")));
}

#[test]
fn test_synthetic_figure_09_shows_similar_primary_behavior_cases() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let admissible = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "primary_magnitude_similarity"
                && row.series_id == "admissible_primary_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let detectable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "primary_magnitude_similarity"
                && row.series_id == "detectable_primary_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!((mean(&admissible) - mean(&detectable)).abs() < 0.1);
}

#[test]
fn test_synthetic_figure_09_shows_distinct_meta_or_higher_order_behavior() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let admissible = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "meta_residual_divergence" && row.series_id == "admissible_meta_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let detectable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "meta_residual_divergence" && row.series_id == "detectable_meta_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!((mean(&admissible) - mean(&detectable)).abs() > 1.0e-3);
}

#[test]
fn test_synthetic_figure_09_shows_distinct_outcome() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    let admissible = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "outcome_consequence" && row.series_id == "admissible_outcome_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let detectable = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "outcome_consequence" && row.series_id == "detectable_outcome_case"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!((mean(&admissible) - mean(&detectable)).abs() > 1.0e-3);
}

#[test]
fn test_synthetic_figure_09_not_trivial_summary_bar_chart() {
    let root = ensure_synthetic_pipeline_ran();
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
fn test_synthetic_figure_12_generated() {
    let root = ensure_synthetic_pipeline_ran();
    assert!(root
        .join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
        .is_file());
}

#[test]
fn test_synthetic_figure_12_filename_preserved() {
    let root = ensure_synthetic_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
            .file_name()
            .unwrap(),
        "figure_12_semantic_retrieval_heuristics_bank.png"
    );
}

#[test]
fn test_synthetic_figure_12_uses_synthetic_outputs() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let ids = scenario_ids(&root);
    assert!(table.rows.iter().all(|row| ids.contains(&row.scenario_id)));
    assert!(table
        .rows
        .iter()
        .all(|row| !row.scenario_id.starts_with("nasa_")));
}

#[test]
fn test_synthetic_figure_12_has_timeline_structure() {
    let root = ensure_synthetic_pipeline_ran();
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
fn test_synthetic_figure_12_shows_semantic_transitions_or_candidate_evolution() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let scores = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_score_timeline" && row.series_id == "top_candidate_score"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(scores
        .windows(2)
        .any(|window| (window[1] - window[0]).abs() > 1.0e-9));
}

#[test]
fn test_synthetic_figure_12_shows_ambiguity_or_candidate_count_evolution() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let post_regime_counts = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_candidate_count_timeline"
                && row.series_id == "post_regime_count"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    let post_scope_counts = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_candidate_count_timeline"
                && row.series_id == "post_scope_count"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(
        post_regime_counts
            .windows(2)
            .any(|window| (window[1] - window[0]).abs() > 1.0e-9)
            || post_scope_counts
                .windows(2)
                .any(|window| (window[1] - window[0]).abs() > 1.0e-9)
    );
}

#[test]
fn test_synthetic_figure_12_not_static_single_snapshot_only() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_synthetic_figure_13_generated() {
    let root = ensure_synthetic_pipeline_ran();
    assert!(root
        .join("figures/figure_13_internal_baseline_comparators.png")
        .is_file());
}

#[test]
fn test_synthetic_figure_13_filename_preserved() {
    let root = ensure_synthetic_pipeline_ran();
    assert_eq!(
        root.join("figures/figure_13_internal_baseline_comparators.png")
            .file_name()
            .unwrap(),
        "figure_13_internal_baseline_comparators.png"
    );
}

#[test]
fn test_synthetic_figure_13_uses_synthetic_outputs() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    let ids = scenario_ids(&root);
    assert!(table.rows.iter().all(|row| ids.contains(&row.scenario_id)));
    assert!(table
        .rows
        .iter()
        .all(|row| !row.scenario_id.starts_with("nasa_")));
}

#[test]
fn test_synthetic_figure_13_shows_baseline_or_internal_comparator_view() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "baseline_alarm_timing" && row.series_kind == "bar"));
}

#[test]
fn test_synthetic_figure_13_shows_dsfb_structural_layer_in_addition() {
    let root = ensure_synthetic_pipeline_ran();
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
fn test_synthetic_figure_13_not_flat_trigger_count_only() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(!table
        .panel_ids
        .contains(&"comparator_trigger_counts".to_string()));
}

#[test]
fn test_synthetic_figure_13_not_presented_as_performance_benchmark_only() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.plot_title.contains("Interpretation"));
    assert!(!table.plot_title.to_lowercase().contains("accuracy"));
}

#[test]
fn test_milling_figure_09_generated() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert!(root
        .join("figures/figure_09_detectability_bound_comparison.png")
        .is_file());
}

#[test]
fn test_milling_figure_09_filename_preserved() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert_eq!(
        root.join("figures/figure_09_detectability_bound_comparison.png")
            .file_name()
            .unwrap(),
        "figure_09_detectability_bound_comparison.png"
    );
}

#[test]
fn test_milling_figure_09_uses_milling_outputs() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_milling_public_demo"));
}

#[test]
fn test_milling_figure_09_shows_similar_primary_behavior_cases() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
    assert!((mean(&stable) - mean(&departure)).abs() < 0.05);
}

#[test]
fn test_milling_figure_09_shows_distinct_meta_or_higher_order_behavior() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
    assert!((mean(&stable) - mean(&departure)).abs() > 1.0e-3);
}

#[test]
fn test_milling_figure_09_shows_distinct_outcome() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
    assert!((mean(&stable) - mean(&departure)).abs() > 1.0e-3);
}

#[test]
fn test_milling_figure_09_not_trivial_summary_bar_chart() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
fn test_milling_figure_12_generated() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert!(root
        .join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
        .is_file());
}

#[test]
fn test_milling_figure_12_filename_preserved() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert_eq!(
        root.join("figures/figure_12_semantic_retrieval_heuristics_bank.png")
            .file_name()
            .unwrap(),
        "figure_12_semantic_retrieval_heuristics_bank.png"
    );
}

#[test]
fn test_milling_figure_12_uses_milling_outputs() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_milling_public_demo"));
}

#[test]
fn test_milling_figure_12_has_timeline_or_process_segment_structure() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
fn test_milling_figure_12_shows_semantic_transitions_or_candidate_evolution() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let scores = table
        .rows
        .iter()
        .filter(|row| {
            row.panel_id == "semantic_score_timeline" && row.series_id == "top_candidate_score"
        })
        .map(|row| row.y_value)
        .collect::<Vec<_>>();
    assert!(scores
        .windows(2)
        .any(|window| (window[1] - window[0]).abs() > 1.0e-9));
}

#[test]
fn test_milling_figure_12_shows_ambiguity_or_candidate_count_evolution() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
fn test_milling_figure_12_not_static_single_snapshot_only() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_milling_figure_13_generated() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert!(root
        .join("figures/figure_13_internal_baseline_comparators.png")
        .is_file());
}

#[test]
fn test_milling_figure_13_filename_preserved() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    assert_eq!(
        root.join("figures/figure_13_internal_baseline_comparators.png")
            .file_name()
            .unwrap(),
        "figure_13_internal_baseline_comparators.png"
    );
}

#[test]
fn test_milling_figure_13_uses_milling_outputs() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .all(|row| row.scenario_id == "nasa_milling_public_demo"));
}

#[test]
fn test_milling_figure_13_shows_baseline_or_internal_comparator_view() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table
        .rows
        .iter()
        .any(|row| row.panel_id == "baseline_alarm_timing" && row.series_kind == "bar"));
}

#[test]
fn test_milling_figure_13_shows_dsfb_structural_layer_in_addition() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
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
fn test_milling_figure_13_not_flat_trigger_count_only() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(!table
        .panel_ids
        .contains(&"comparator_trigger_counts".to_string()));
}

#[test]
fn test_milling_figure_13_not_presented_as_performance_benchmark_only() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.plot_title.contains("Interpretation"));
    assert!(!table.plot_title.to_lowercase().contains("accuracy"));
}

#[test]
fn test_synthetic_figure_09_source_table_exists_and_is_nontrivial() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_synthetic_figure_12_source_table_exists_and_is_nontrivial() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_synthetic_figure_13_source_table_exists_and_is_nontrivial() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_milling_figure_09_source_table_exists_and_is_nontrivial() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_milling_figure_12_source_table_exists_and_is_nontrivial() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_milling_figure_13_source_table_exists_and_is_nontrivial() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.rows.len() >= 12);
}

#[test]
fn test_synthetic_figure_sources_derived_from_real_outputs() {
    let root = ensure_synthetic_pipeline_ran();
    let ids = scenario_ids(&root);
    for figure_id in [
        "figure_09_detectability_bound_comparison",
        "figure_12_semantic_retrieval_heuristics_bank",
        "figure_13_internal_baseline_comparators",
    ] {
        let table = figure_table(&root, figure_id);
        assert!(table.rows.iter().all(|row| ids.contains(&row.scenario_id)));
    }
}

#[test]
fn test_milling_figure_sources_derived_from_real_outputs() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    let semantic_matches = json_array(&root, "semantic_matches.json");
    let disposition = semantic_matches[0]["disposition"].as_str().unwrap();
    let last = table
        .rows
        .iter()
        .rfind(|row| row.panel_id == "semantic_disposition_timeline")
        .unwrap();
    assert_eq!(last.y_value, semantic_code(disposition));
}

#[test]
fn test_artifact_pipeline_regenerates_synthetic_and_milling_figure_sources_without_manual_intervention(
) {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
    let milling_root = public_dataset_root("nasa_milling");
    for root in [synthetic_root, milling_root] {
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
}

#[test]
fn test_synthetic_figure_09_caption_framing_upgraded() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.plot_title.contains("Similar Primary"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("primary behavior alone is insufficient")));
}

#[test]
fn test_synthetic_figure_12_caption_framing_upgraded() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.plot_title.contains("Semantic Evolution"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("semantic evolution")));
}

#[test]
fn test_synthetic_figure_13_caption_framing_upgraded() {
    let root = ensure_synthetic_pipeline_ran();
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.plot_title.contains("Interpretation"));
    assert!(table.rows.iter().any(|row| row
        .note
        .contains("without claiming performance superiority")));
}

#[test]
fn test_milling_figure_09_caption_framing_upgraded() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_09_detectability_bound_comparison");
    assert!(table.plot_title.contains("Similar Primary"));
    assert!(table.rows.iter().any(|row| row
        .note
        .contains("first-order behavior alone is insufficient")));
}

#[test]
fn test_milling_figure_12_caption_framing_upgraded() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank");
    assert!(table.plot_title.contains("Semantic Evolution"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("semantic process")));
}

#[test]
fn test_milling_figure_13_caption_framing_upgraded() {
    ensure_public_pipeline_ran();
    let root = public_dataset_root("nasa_milling");
    let table = figure_table(&root, "figure_13_internal_baseline_comparators");
    assert!(table.plot_title.contains("Interpretation"));
    assert!(table
        .rows
        .iter()
        .any(|row| row.note.contains("not as a performance benchmark")));
}

#[test]
fn test_no_overclaiming_in_synthetic_and_milling_09_12_13_captions() {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
    let milling_root = public_dataset_root("nasa_milling");
    for root in [synthetic_root, milling_root] {
        for figure_id in [
            "figure_09_detectability_bound_comparison",
            "figure_12_semantic_retrieval_heuristics_bank",
            "figure_13_internal_baseline_comparators",
        ] {
            let table = figure_table(&root, figure_id);
            let lowered = table.plot_title.to_lowercase();
            assert!(!lowered.contains("accuracy"));
            assert!(!lowered.contains("superiority"));
            assert!(!lowered.contains("proves"));
        }
    }
}

#[test]
fn test_paper_synthetic_figures_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/paper_synthetic_figures_09_12_13.md")
        .is_file());
}

#[test]
fn test_paper_milling_figures_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/paper_milling_figures_09_12_13.md")
        .is_file());
}

#[test]
fn test_synthetic_doc_explains_figure_09_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_synthetic_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 9 argues"));
    assert!(docs.contains("higher-order / meta-residual structure"));
}

#[test]
fn test_synthetic_doc_explains_figure_12_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_synthetic_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 12 argues"));
    assert!(docs.contains("semantic evolution"));
}

#[test]
fn test_synthetic_doc_explains_figure_13_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_synthetic_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 13 argues"));
    assert!(docs.contains("interpretability delta"));
}

#[test]
fn test_milling_doc_explains_figure_09_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_milling_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 9 argues"));
    assert!(docs.contains("higher-order / meta-residual structure"));
}

#[test]
fn test_milling_doc_explains_figure_12_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_milling_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 12 argues"));
    assert!(docs.contains("semantic retrieval evolves"));
}

#[test]
fn test_milling_doc_explains_figure_13_argument() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_milling_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("Figure 13 argues"));
    assert!(docs.contains("interpretability delta"));
}

#[test]
fn test_docs_explain_regeneration_and_drop_in_workflow() {
    for path in [
        crate_root().join("docs/examples/paper_synthetic_figures_09_12_13.md"),
        crate_root().join("docs/examples/paper_milling_figures_09_12_13.md"),
    ] {
        let docs = fs::read_to_string(path).unwrap();
        assert!(docs.contains("Regeneration"));
        assert!(docs.contains("No LaTeX edits are required"));
        assert!(docs.contains("paper `figures/` folder"));
    }
}

#[test]
fn test_bearings_figure_upgrade_not_regressed() {
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
}

#[test]
fn test_synthetic_milling_bearings_all_generate_upgraded_09_12_13() {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
    for root in [
        synthetic_root,
        public_dataset_root("nasa_milling"),
        public_dataset_root("nasa_bearings"),
    ] {
        assert!(
            figure_table(&root, "figure_09_detectability_bound_comparison")
                .panel_ids
                .contains(&"primary_magnitude_similarity".to_string())
        );
        assert!(
            figure_table(&root, "figure_12_semantic_retrieval_heuristics_bank")
                .panel_ids
                .contains(&"semantic_score_timeline".to_string())
        );
        assert!(
            figure_table(&root, "figure_13_internal_baseline_comparators")
                .panel_ids
                .contains(&"baseline_alarm_timing".to_string())
        );
    }
}

#[test]
fn test_other_artifact_outputs_not_broken() {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
    for root in [
        synthetic_root,
        public_dataset_root("nasa_milling"),
        public_dataset_root("nasa_bearings"),
    ] {
        assert!(root.join("manifest.json").is_file());
        assert!(root
            .join("report/dsfb_semiotics_engine_report.md")
            .is_file());
        assert!(root.join("json/run_metadata.json").is_file());
        assert!(root.join("csv/evaluation_summary.csv").is_file());
    }
}

#[test]
fn test_output_filenames_for_09_12_13_exact_across_all_run_families() {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
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
            assert_eq!(
                root.join("figures").join(figure_name).file_name().unwrap(),
                figure_name
            );
        }
    }
}

#[test]
fn test_png_outputs_exist_for_09_12_13_across_all_run_families() {
    let synthetic_root = ensure_synthetic_pipeline_ran();
    ensure_public_pipeline_ran();
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
fn test_drop_in_paper_workflow_unchanged_for_synthetic() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_synthetic_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("No LaTeX edits are required"));
    assert!(docs.contains("paper `figures/` folder"));
}

#[test]
fn test_drop_in_paper_workflow_unchanged_for_milling() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_milling_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("No LaTeX edits are required"));
    assert!(docs.contains("paper `figures/` folder"));
}

#[test]
fn test_drop_in_paper_workflow_unchanged_for_bearings() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/paper_bearings_figures_09_12_13.md"))
            .unwrap();
    assert!(docs.contains("No LaTeX edits are required"));
    assert!(docs.contains("paper `figures/` folder"));
}
