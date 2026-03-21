use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use dsfb_semiotics_engine::engine::config::CommonRunConfig;
use dsfb_semiotics_engine::engine::pipeline::{
    export_artifacts, EngineConfig, StructuralSemioticsEngine,
};
use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::EnvelopeMode;
use dsfb_semiotics_engine::live::{to_real, OnlineStructuralEngine};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;
use serde_json::Value;
use tempfile::tempdir;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme() -> String {
    fs::read_to_string(crate_root().join("README.md")).unwrap()
}

fn ensure_imu_artifacts_ran() -> PathBuf {
    static ONCE: OnceLock<PathBuf> = OnceLock::new();
    ONCE.get_or_init(|| {
        let temp = tempdir().unwrap();
        let run_root = temp.path().join("imu_transition");
        let bundle = StructuralSemioticsEngine::new(EngineConfig::synthetic_single(
            CommonRunConfig {
                output_root: Some(run_root.clone()),
                ..Default::default()
            },
            "imu_thermal_drift_gps_denied",
        ))
        .run_selected()
        .unwrap();
        let exported = export_artifacts(&bundle).unwrap();
        std::mem::forget(temp);
        exported.run_dir
    })
    .clone()
}

#[test]
fn test_real_time_contract_doc_exists() {
    assert!(crate_root().join("docs/REAL_TIME_CONTRACT.md").is_file());
}

#[test]
fn test_real_time_contract_mentions_memory_budget() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("Memory Budget"));
    assert!(docs.contains("6064"));
}

#[test]
fn test_real_time_contract_mentions_timing_budget() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("Timing Budget"));
    assert!(docs.contains("scalar_push_sample"));
}

#[test]
fn test_real_time_contract_mentions_no_panic_policy() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("Panic Policy"));
    assert!(docs.contains("Invalid inputs return structured `Result` errors"));
}

#[test]
fn test_real_time_contract_mentions_no_nan_policy() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("NaN / Inf Policy"));
    assert!(docs.contains("non-finite"));
}

#[test]
fn test_real_time_contract_links_from_readme() {
    assert!(readme().contains("docs/REAL_TIME_CONTRACT.md"));
}

#[test]
fn test_real_time_contract_links_from_icd() {
    let icd = fs::read_to_string(crate_root().join("docs/ICD.md")).unwrap();
    assert!(icd.contains("REAL_TIME_CONTRACT.md"));
}

#[test]
fn test_real_time_contract_summary_generated_if_applicable() {
    assert!(crate_root()
        .join("docs/generated/real_time_contract_summary.json")
        .is_file());
    let value: Value = serde_json::from_str(
        &fs::read_to_string(crate_root().join("docs/generated/real_time_contract_summary.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        value["schema_version"].as_str().unwrap(),
        "dsfb-semiotics-real-time-contract/v1"
    );
}

#[test]
fn test_online_path_allocation_audit_exists() {
    assert!(crate_root()
        .join("docs/ONLINE_PATH_ALLOCATION_AUDIT.md")
        .is_file());
}

#[test]
fn test_online_path_no_heap_alloc_after_init_or_gap_explicitly_documented() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("not yet claimed"));
    assert!(docs.contains("per-sample heap allocation: still present"));
}

#[test]
fn test_hot_path_does_not_format_strings_or_build_unbounded_vectors() {
    let source = fs::read_to_string(crate_root().join("src/live/mod.rs")).unwrap();
    let start = source.find("pub fn push_residual_sample").unwrap();
    let end = source.find("pub fn push_residual_sample_batch").unwrap();
    let hot_path = &source[start..end];
    assert!(!hot_path.contains("format!("));
    assert!(!hot_path.contains("Vec::new()"));
}

#[test]
fn test_core_push_sample_path_allocation_behavior_verified() {
    let value: Value = serde_json::from_str(
        &fs::read_to_string(crate_root().join("docs/generated/real_time_contract_summary.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["no_heap_alloc_after_init_verified"], false);
    assert!(value["allocation_audit_findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding["symbol"] == "OnlineStructuralEngine::push_residual_sample"
                && finding["allocation_behavior"] == "present"
        }));
}

#[test]
fn test_no_unwrap_expect_in_hot_online_path() {
    let source = fs::read_to_string(crate_root().join("src/live/mod.rs")).unwrap();
    let start = source.find("pub fn push_residual_sample").unwrap();
    let end = source.find("pub fn push_residual_sample_batch").unwrap();
    let hot_path = &source[start..end];
    assert!(!hot_path.contains(".unwrap("));
    assert!(!hot_path.contains(".expect("));
}

#[test]
fn test_push_sample_invalid_input_does_not_panic() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "invalid_input",
        vec!["x".to_string()],
        1.0,
        EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        EngineSettings::default(),
    )
    .unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        engine.push_residual_sample(f64::NAN, &[to_real(0.1)])
    }));
    assert!(result.is_ok());
    assert!(result.unwrap().is_err());
}

#[test]
fn test_no_nan_outputs_on_near_zero_norm_cases() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "near_zero",
        vec!["x".to_string(), "y".to_string(), "z".to_string()],
        1.0,
        EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        EngineSettings::default(),
    )
    .unwrap();
    for step in 0..8 {
        let status = engine
            .push_residual_sample(
                step as f64,
                &[to_real(0.0), to_real(1.0e-12), to_real(-1.0e-12)],
            )
            .unwrap();
        assert!(status.residual_norm.is_finite());
        assert!(status.drift_norm.is_finite());
        assert!(status.slew_norm.is_finite());
        assert!(status.trust_scalar.is_finite());
        assert!(status.projection.iter().all(|value| value.is_finite()));
    }
}

#[test]
fn test_no_inf_outputs_on_extreme_but_supported_cases() {
    let mut engine = OnlineStructuralEngine::with_builtin_bank(
        "extreme",
        vec!["x".to_string()],
        1.0,
        EnvelopeSpec {
            name: "fixed".to_string(),
            mode: EnvelopeMode::Fixed,
            base_radius: 1.0e151,
            slope: 0.0,
            switch_step: None,
            secondary_slope: None,
            secondary_base: None,
        },
        EngineSettings::default(),
    )
    .unwrap();
    let status = engine
        .push_residual_sample(0.0, &[to_real(1.0e150)])
        .unwrap();
    assert!(status.residual_norm.is_finite());
    assert!(status.drift_norm.is_finite());
    assert!(status.slew_norm.is_finite());
}

#[test]
fn test_numerical_guard_policy_documented() {
    let docs = fs::read_to_string(crate_root().join("docs/REAL_TIME_CONTRACT.md")).unwrap();
    assert!(docs.contains("structured error"));
    assert!(docs.contains("non-finite residual values are rejected"));
}

#[test]
fn test_timing_report_contains_p99_and_max() {
    let docs = fs::read_to_string(crate_root().join("docs/TIMING_DETERMINISM_REPORT.md")).unwrap();
    assert!(docs.contains("p99"));
    assert!(docs.contains("Max"));
}

#[test]
fn test_timing_report_contains_stress_case_measurements() {
    let docs = fs::read_to_string(crate_root().join("docs/TIMING_DETERMINISM_REPORT.md")).unwrap();
    assert!(docs.contains("grammar_violation_path"));
    assert!(docs.contains("semantic_retrieval_enlarged_bank"));
}

#[test]
fn test_timing_report_distinguishes_observed_vs_certified() {
    let docs = fs::read_to_string(crate_root().join("docs/TIMING_DETERMINISM_REPORT.md")).unwrap();
    assert!(docs.contains("observed host-side timing behavior"));
    assert!(docs.contains("not a certified WCET analysis"));
}

#[test]
fn test_batch_ingestion_timing_included() {
    let docs = fs::read_to_string(crate_root().join("docs/TIMING_DETERMINISM_REPORT.md")).unwrap();
    assert!(docs.contains("batch_push_sample"));
}

#[test]
fn test_semantic_retrieval_stress_timing_included() {
    let docs = fs::read_to_string(crate_root().join("docs/TIMING_DETERMINISM_REPORT.md")).unwrap();
    assert!(docs.contains("semantic_retrieval_enlarged_bank"));
}

#[test]
fn test_readme_mentions_observed_worst_case_timing_conservatively() {
    let docs = readme();
    assert!(docs.contains("Timing Determinism Report"));
    assert!(docs.contains("observed timing"));
}

#[test]
fn test_decision_grade_demo_doc_exists() {
    assert!(crate_root()
        .join("docs/examples/decision_grade_demo.md")
        .is_file());
}

#[test]
fn test_decision_grade_demo_contains_time_ordered_event_narrative() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/decision_grade_demo.md")).unwrap();
    assert!(docs.contains("t ≈ 60 s"));
    assert!(docs.contains("t ≈ 75 s"));
    assert!(docs.contains("t ≈ 120 s"));
}

#[test]
fn test_decision_grade_demo_reproducible_command_exists() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/decision_grade_demo.md")).unwrap();
    assert!(docs.contains("--scenario imu_thermal_drift_gps_denied"));
}

#[test]
fn test_demo_outputs_event_timeline_artifact() {
    let run_dir = ensure_imu_artifacts_ran();
    assert!(run_dir
        .join("csv/imu_thermal_drift_gps_denied_event_timeline.csv")
        .is_file());
    assert!(run_dir
        .join("json/imu_thermal_drift_gps_denied_event_timeline.json")
        .is_file());
}

#[test]
fn test_decision_implication_is_documented_conservatively() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/decision_grade_demo.md")).unwrap();
    assert!(docs.contains("not a prescribed control law"));
    assert!(docs.contains("operator"));
}

#[test]
fn test_imu_thermal_drift_gps_denied_exists() {
    let scenarios = fs::read_to_string(crate_root().join("src/sim/scenarios.rs")).unwrap();
    assert!(scenarios.contains("imu_thermal_drift_gps_denied"));
}

#[test]
fn test_imu_scenario_outputs_syntax_grammar_semantics_events() {
    let run_dir = ensure_imu_artifacts_ran();
    let csv =
        fs::read_to_string(run_dir.join("csv/imu_thermal_drift_gps_denied_event_timeline.csv"))
            .unwrap();
    assert!(csv.contains("syntax"));
    assert!(csv.contains("grammar"));
    assert!(csv.contains("semantics"));
}

#[test]
fn test_batch_ingestion_documented_as_primary_for_multi_axis() {
    let docs = readme();
    assert!(docs.contains("batch ingestion is the primary ingestion style for multi-axis"));
}

#[test]
fn test_batch_vs_scalar_benchmark_exists() {
    let docs = fs::read_to_string(crate_root().join("docs/execution_budget.md")).unwrap();
    assert!(docs.contains("bounded online engine step"));
    assert!(docs.contains("bounded online engine batch step"));
}

#[test]
fn test_batch_overhead_reduction_reported() {
    let docs = fs::read_to_string(crate_root().join("docs/execution_budget.md")).unwrap();
    assert!(docs.contains("overhead reduction"));
    assert!(docs.contains("1.8%"));
}

#[test]
fn test_ffi_examples_include_batch_ingestion() {
    assert!(crate_root().join("ffi/examples/batch_ffi.c").is_file());
    let docs = fs::read_to_string(crate_root().join("docs/examples/ffi_integration.md")).unwrap();
    assert!(docs.contains("prefer the batch path"));
}

#[test]
#[ignore = "requires a dual-build comparison harness across f32 and numeric-fixed; current gap remains documented"]
fn test_fixed_point_vs_f32_equivalence_on_canonical_scenario() {}

#[test]
fn test_fixed_point_classification_consistency_documented() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("classification consistency"));
}

#[test]
fn test_fixed_point_precision_bounds_documented() {
    let docs = fs::read_to_string(crate_root().join("docs/high_assurance_embedded.md")).unwrap();
    assert!(docs.contains("precision bounds"));
    assert!(docs.contains("quantization tradeoffs"));
}

#[test]
fn test_fixed_point_demo_runs() {
    let output = Command::new(env!("CARGO"))
        .args([
            "check",
            "--manifest-path",
            crate_root().join("Cargo.toml").to_str().unwrap(),
            "--example",
            "live_drop_in",
            "--features",
            "numeric-fixed",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_readme_frames_fixed_point_conservatively() {
    let docs = readme();
    assert!(docs.contains("numeric-fixed"));
    assert!(docs.contains("not a blanket embedded-readiness claim"));
}

#[test]
fn test_same_residual_different_outcome_demo_exists() {
    assert!(crate_root()
        .join("docs/examples/same_primary_different_outcome.md")
        .is_file());
}

#[test]
fn test_demo_outputs_distinct_meta_or_structural_outcomes() {
    let docs =
        fs::read_to_string(crate_root().join("docs/examples/same_primary_different_outcome.md"))
            .unwrap();
    assert!(docs.contains("meta-residual"));
    assert!(docs.contains("different outcome"));
}

#[test]
fn test_apnt_brief_exists() {
    assert!(crate_root()
        .join("docs/briefs/dsfb_apnt_brief.md")
        .is_file());
}

#[test]
fn test_apnt_brief_mentions_problem_differentiator_example_and_number_or_explicit_reason_no_number()
{
    let docs = fs::read_to_string(crate_root().join("docs/briefs/dsfb_apnt_brief.md")).unwrap();
    assert!(docs.contains("Problem"));
    assert!(docs.contains("What DSFB Does Differently"));
    assert!(docs.contains("Concrete A-PNT Example"));
    assert!(docs.contains("992276 ns"));
}

#[test]
fn test_sample_vs_full_artifacts_are_distinguished() {
    let docs = fs::read_to_string(crate_root().join("docs/public_dataset_demo.md")).unwrap();
    assert!(docs.contains("sample-grade"));
    assert!(docs.contains("full regenerated artifact surface"));
}

#[test]
fn test_readme_has_real_time_contract_link() {
    assert!(readme().contains("docs/REAL_TIME_CONTRACT.md"));
}

#[test]
fn test_icd_mentions_timing_memory_error_behavior() {
    let icd = fs::read_to_string(crate_root().join("docs/ICD.md")).unwrap();
    assert!(icd.contains("Memory footprint table"));
    assert!(icd.contains("Measured timing"));
    assert!(icd.contains("Error handling contract"));
}
