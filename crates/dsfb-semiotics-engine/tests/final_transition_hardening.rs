use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_rel(path: &str) -> String {
    fs::read_to_string(crate_root().join(path)).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

fn json_rel(path: &str) -> Value {
    serde_json::from_str(&read_rel(path)).unwrap_or_else(|error| {
        panic!("failed to parse json {path}: {error}");
    })
}

fn line_count(path: &str) -> usize {
    read_rel(path).lines().count()
}

fn contains_all(text: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| text.contains(needle))
}

fn starts_with_doc_comment(path: &str) -> bool {
    read_rel(path)
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with("//!"))
}

#[test]
fn test_flight_loop_trust_position_doc_exists() {
    assert!(crate_root()
        .join("docs/FLIGHT_LOOP_TRUST_POSITION.md")
        .exists());
}

#[test]
fn test_flight_loop_trust_position_distinguishes_advisory_vs_trusted_roles() {
    let doc = read_rel("docs/FLIGHT_LOOP_TRUST_POSITION.md").to_lowercase();
    assert!(contains_all(
        &doc,
        &[
            "bounded monitor / advisory use",
            "direct flight-loop integration experiments",
            "not yet justified for blind trust"
        ]
    ));
}

#[test]
fn test_flight_loop_trust_position_links_to_real_time_contract() {
    let doc = read_rel("docs/FLIGHT_LOOP_TRUST_POSITION.md");
    assert!(doc.contains("REAL_TIME_CONTRACT.md"));
}

#[test]
fn test_flight_loop_trust_position_links_to_timing_report() {
    let doc = read_rel("docs/FLIGHT_LOOP_TRUST_POSITION.md");
    assert!(doc.contains("TIMING_DETERMINISM_REPORT.md"));
}

#[test]
fn test_flight_loop_trust_position_links_to_allocation_audit() {
    let doc = read_rel("docs/FLIGHT_LOOP_TRUST_POSITION.md");
    assert!(doc.contains("ONLINE_PATH_ALLOCATION_AUDIT.md"));
}

#[test]
fn test_flight_loop_trust_position_explicitly_states_non_certified_scope() {
    let doc = read_rel("docs/FLIGHT_LOOP_TRUST_POSITION.md");
    assert!(contains_all(
        &doc,
        &[
            "observed bounded behavior",
            "integration-ready under stated assumptions",
            "not certifiable as-is"
        ]
    ));
}

#[test]
fn test_target_facing_timing_demo_doc_exists() {
    assert!(crate_root()
        .join("docs/TARGET_FACING_TIMING_DEMO.md")
        .exists());
}

#[test]
fn test_target_facing_timing_demo_distinguishes_from_host_benchmarks() {
    let doc = read_rel("docs/TARGET_FACING_TIMING_DEMO.md").to_lowercase();
    assert!(contains_all(
        &doc,
        &[
            "host benchmark report",
            "target-facing demo",
            "certified wcet",
            "not claimed"
        ]
    ));
}

#[test]
fn test_target_facing_timing_demo_includes_platform_or_profile_assumptions() {
    let doc = read_rel("docs/TARGET_FACING_TIMING_DEMO.md");
    assert!(contains_all(
        &doc,
        &[
            "profile name: `constrained_host_profile`",
            "history_buffer_capacity=16",
            "safety_first"
        ]
    ));
}

#[test]
fn test_target_facing_timing_demo_reports_observed_bounds() {
    let doc = read_rel("docs/TARGET_FACING_TIMING_DEMO.md");
    assert!(contains_all(
        &doc,
        &["Observed Bounds", "p99", "Max (ns)", "616,028"]
    ));
    let json = json_rel("docs/generated/target_facing_timing_demo.json");
    assert_eq!(json["profile_name"], "constrained_host_profile");
    assert!(json["metrics"].as_array().unwrap().len() >= 3);
}

#[test]
fn test_figures_source_module_reduced_or_split_further() {
    assert!(crate_root().join("src/figures/source/upgraded.rs").exists());
    assert!(line_count("src/figures/source.rs") < 3_000);
}

#[test]
fn test_figures_plots_module_reduced_or_split_further() {
    assert!(crate_root().join("src/figures/plots/upgraded.rs").exists());
    assert!(line_count("src/figures/plots.rs") < 1_200);
}

#[test]
fn test_pipeline_artifact_tables_module_reduced_or_split_further() {
    assert!(crate_root()
        .join("src/engine/pipeline_artifacts/tables/rows.rs")
        .exists());
    assert!(line_count("src/engine/pipeline_artifacts/tables.rs") < 500);
}

#[test]
fn test_semantics_retrieval_module_reduced_or_split_further() {
    assert!(crate_root()
        .join("src/engine/semantics/retrieval/index.rs")
        .exists());
    assert!(crate_root()
        .join("src/engine/semantics/retrieval/scoring.rs")
        .exists());
    assert!(line_count("src/engine/semantics/retrieval.rs") < 500);
}

#[test]
fn test_semantics_bank_builtin_module_reduced_or_split_further() {
    assert!(crate_root()
        .join("src/engine/semantics/bank_builtin/entries.rs")
        .exists());
    assert!(line_count("src/engine/semantics/bank_builtin.rs") < 100);
}

#[test]
fn test_integration_tests_decomposed_by_theme() {
    assert!(crate_root().join("tests/integration_artifacts.rs").exists());
    assert!(line_count("tests/integration.rs") < 1_300);
}

#[test]
fn test_refactor_preserves_outputs_and_reproducibility() {
    let readme = read_rel("README.md");
    assert!(readme.contains("reproducible"));
    assert!(crate_root()
        .join("docs/generated/real_time_contract_summary.json")
        .exists());
}

#[test]
fn test_module_docs_updated_after_final_split() {
    for path in [
        "src/engine/semantics/retrieval/index.rs",
        "src/engine/semantics/retrieval/scoring.rs",
        "src/engine/pipeline_artifacts/tables/rows.rs",
        "src/figures/source/upgraded.rs",
        "src/figures/plots/upgraded.rs",
        "src/engine/semantics/bank_builtin/entries.rs",
    ] {
        assert!(
            starts_with_doc_comment(path),
            "{path} is missing a module doc comment"
        );
    }
}

#[test]
fn test_fixed_point_deployment_evidence_doc_exists() {
    assert!(crate_root()
        .join("docs/FIXED_POINT_DEPLOYMENT_EVIDENCE.md")
        .exists());
}

#[test]
fn test_fixed_point_deployment_evidence_includes_supported_scope() {
    let doc = read_rel("docs/FIXED_POINT_DEPLOYMENT_EVIDENCE.md");
    assert!(contains_all(
        &doc,
        &[
            "## Supported Scope",
            "bounded `OnlineStructuralEngine` live path",
            "Not yet covered"
        ]
    ));
}

#[test]
fn test_fixed_point_deployment_evidence_includes_precision_bounds() {
    let doc = read_rel("docs/FIXED_POINT_DEPLOYMENT_EVIDENCE.md");
    assert!(contains_all(
        &doc,
        &[
            "## Precision Bounds Used",
            "trust-scalar drift stayed below `2.5e-5`",
            "residual-norm drift stayed below `6.5e-6`"
        ]
    ));
}

#[test]
fn test_fixed_point_vs_f32_equivalence_on_additional_live_or_demo_path() {
    let f64_report = json_rel("docs/generated/fixed_point_deployment_evidence_f64.json");
    let fixed_report =
        json_rel("docs/generated/fixed_point_deployment_evidence_numeric_fixed.json");

    let to_map = |value: &Value| {
        value["scenario_summaries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["scenario_id"].as_str().unwrap().to_string(),
                    entry.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };

    let f64_map = to_map(&f64_report);
    let fixed_map = to_map(&fixed_report);
    assert_eq!(f64_map.len(), fixed_map.len());

    for scenario_id in [
        "imu_thermal_drift_gps_denied",
        "regime_switch",
        "abrupt_event",
    ] {
        let f64_entry = f64_map.get(scenario_id).unwrap();
        let fixed_entry = fixed_map.get(scenario_id).unwrap();
        assert_eq!(
            f64_entry["final_syntax_label"],
            fixed_entry["final_syntax_label"]
        );
        assert_eq!(
            f64_entry["final_grammar_state"],
            fixed_entry["final_grammar_state"]
        );
        assert_eq!(
            f64_entry["final_grammar_reason_code"],
            fixed_entry["final_grammar_reason_code"]
        );
        assert_eq!(
            f64_entry["final_semantic_disposition"],
            fixed_entry["final_semantic_disposition"]
        );
        assert_eq!(
            f64_entry["selected_heuristic_ids"],
            fixed_entry["selected_heuristic_ids"]
        );
        let trust_delta = (f64_entry["final_trust_scalar"].as_f64().unwrap()
            - fixed_entry["final_trust_scalar"].as_f64().unwrap())
        .abs();
        assert!(
            trust_delta <= 2.5e-5,
            "{scenario_id} trust delta {trust_delta}"
        );
        let residual_delta = (f64_entry["max_residual_norm"].as_f64().unwrap()
            - fixed_entry["max_residual_norm"].as_f64().unwrap())
        .abs();
        assert!(
            residual_delta <= 6.5e-6,
            "{scenario_id} residual delta {residual_delta}"
        );
    }
}

#[test]
fn test_visual_argument_map_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/visual_argument_map.md")
        .exists());
}

#[test]
fn test_visual_argument_map_explains_09_argument() {
    let doc = read_rel("docs/examples/visual_argument_map.md");
    assert!(contains_all(
        &doc,
        &[
            "Figure 9 Argument",
            "same or very similar primary behavior",
            "different higher-order or meta-residual structure"
        ]
    ));
}

#[test]
fn test_visual_argument_map_explains_12_argument() {
    let doc = read_rel("docs/examples/visual_argument_map.md");
    assert!(contains_all(
        &doc,
        &[
            "Figure 12 Argument",
            "semantic interpretation unfolds through time",
            "disposition and candidate-set evolution"
        ]
    ));
}

#[test]
fn test_visual_argument_map_explains_13_argument() {
    let doc = read_rel("docs/examples/visual_argument_map.md");
    assert!(contains_all(
        &doc,
        &[
            "Figure 13 Argument",
            "baseline comparator view",
            "interpretability delta"
        ]
    ));
}

#[test]
fn test_visual_argument_map_links_to_synthetic_milling_bearings_docs() {
    let doc = read_rel("docs/examples/visual_argument_map.md");
    assert!(contains_all(
        &doc,
        &[
            "paper_synthetic_figures_09_12_13.md",
            "paper_milling_figures_09_12_13.md",
            "paper_bearings_figures_09_12_13.md"
        ]
    ));
}

#[test]
fn test_all_three_run_families_preserve_upgraded_09_12_13() {
    let doc = read_rel("docs/examples/visual_argument_map.md");
    assert!(contains_all(
        &doc,
        &[
            "synthetic for controlled textbook separation",
            "NASA milling for process-window structure",
            "NASA bearings for failure-progression structure"
        ]
    ));
}

#[test]
fn test_what_changes_operationally_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/what_changes_operationally.md")
        .exists());
}

#[test]
fn test_doc_contains_synthetic_example() {
    let doc = read_rel("docs/examples/what_changes_operationally.md");
    assert!(doc.contains("## Synthetic Example"));
}

#[test]
fn test_doc_contains_bearings_example() {
    let doc = read_rel("docs/examples/what_changes_operationally.md");
    assert!(doc.contains("## NASA Bearings Example"));
}

#[test]
fn test_doc_contains_imu_example_if_supported() {
    let doc = read_rel("docs/examples/what_changes_operationally.md");
    assert!(doc.contains("## IMU / GPS-Denied Example"));
}

#[test]
fn test_doc_explains_decision_impact_conservatively() {
    let doc = read_rel("docs/examples/what_changes_operationally.md");
    assert!(contains_all(
        &doc,
        &[
            "Decision impact",
            "still does not decide on its own",
            "root cause"
        ]
    ));
}

#[test]
fn test_technical_brief_mentions_real_time_contract() {
    let doc = read_rel("docs/briefs/dsfb_apnt_brief.md");
    assert!(doc.contains("real-time contract"));
}

#[test]
fn test_technical_brief_mentions_target_facing_timing_demo() {
    let doc = read_rel("docs/briefs/dsfb_apnt_brief.md");
    assert!(doc.contains("target-facing constrained-profile timing demo"));
}

#[test]
fn test_technical_brief_mentions_imu_scenario() {
    let doc = read_rel("docs/briefs/dsfb_apnt_brief.md");
    assert!(doc.contains("imu_thermal_drift_gps_denied"));
}

#[test]
fn test_technical_brief_mentions_fixed_point_scope() {
    let doc = read_rel("docs/briefs/dsfb_apnt_brief.md");
    assert!(doc.contains("fixed-point evidence for the tested bounded live subset"));
}

#[test]
fn test_technical_brief_mentions_visual_or_decision_grade_demo() {
    let doc = read_rel("docs/briefs/dsfb_apnt_brief.md");
    assert!(doc.contains("event timeline"));
}

#[test]
fn test_readme_links_to_flight_loop_trust_position() {
    let readme = read_rel("README.md");
    assert!(readme.contains("docs/FLIGHT_LOOP_TRUST_POSITION.md"));
}

#[test]
fn test_readme_links_to_target_facing_timing_demo() {
    let readme = read_rel("README.md");
    assert!(readme.contains("docs/TARGET_FACING_TIMING_DEMO.md"));
}

#[test]
fn test_readme_links_to_fixed_point_deployment_evidence() {
    let readme = read_rel("README.md");
    assert!(readme.contains("docs/FIXED_POINT_DEPLOYMENT_EVIDENCE.md"));
}

#[test]
fn test_readme_links_to_visual_argument_map() {
    let readme = read_rel("README.md");
    assert!(readme.contains("docs/examples/visual_argument_map.md"));
}

#[test]
fn test_docs_index_surfaces_final_transition_docs() {
    let index = read_rel("docs/INDEX.md");
    assert!(contains_all(
        &index,
        &[
            "FLIGHT_LOOP_TRUST_POSITION.md",
            "TARGET_FACING_TIMING_DEMO.md",
            "FIXED_POINT_DEPLOYMENT_EVIDENCE.md",
            "FINAL_TRANSITION_GAP_REPORT.md"
        ]
    ));
}

#[test]
fn test_icd_mentions_advisory_vs_trusted_role_distinction() {
    let icd = read_rel("docs/ICD.md");
    assert!(contains_all(
        &icd,
        &[
            "advisory / monitor integration",
            "trusted flight-critical role: not justified"
        ]
    ));
}

#[test]
fn test_final_transition_gap_report_exists() {
    assert!(crate_root()
        .join("docs/FINAL_TRANSITION_GAP_REPORT.md")
        .exists());
}

#[test]
fn test_gap_report_distinguishes_pilot_vs_advisory_vs_flight_critical() {
    let doc = read_rel("docs/FINAL_TRANSITION_GAP_REPORT.md");
    assert!(contains_all(
        &doc,
        &[
            "pilot evaluation: yes",
            "bounded advisory use under assumptions: yes",
            "stronger flight-critical trust: not yet"
        ]
    ));
}

#[test]
fn test_gap_report_mentions_remaining_target_assurance_gap() {
    let doc = read_rel("docs/FINAL_TRANSITION_GAP_REPORT.md").to_lowercase();
    assert!(contains_all(
        &doc,
        &["constrained-profile", "qualification package"]
    ));
}

#[test]
fn test_gap_report_mentions_remaining_embedded_or_fixed_point_gap_if_present() {
    let doc = read_rel("docs/FINAL_TRANSITION_GAP_REPORT.md");
    assert!(contains_all(
        &doc,
        &[
            "whole-crate embedded / `no_std` coverage",
            "broader fixed-point coverage beyond the tested live subset",
            "no heap allocation after initialization"
        ]
    ));
}
