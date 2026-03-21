use std::fs;
use std::path::PathBuf;

use clap::CommandFactory;
use dsfb_semiotics_engine::CliArgs;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme_text() -> String {
    fs::read_to_string(crate_root().join("README.md")).unwrap()
}

#[test]
fn test_readme_mentions_external_bank_mode() {
    let readme = readme_text();
    assert!(readme.contains("--bank-mode external"));
}

#[test]
fn test_readme_mentions_strict_bank_validation() {
    let readme = readme_text();
    assert!(readme.contains("--strict-bank-validation"));
    assert!(readme.contains("--bank-validation-mode permissive"));
    assert!(readme.contains("Strict governance is now the default posture"));
}

#[test]
fn test_readme_mentions_strict_validation_default() {
    let readme = readme_text();
    assert!(readme.contains("strict validation default"));
}

#[test]
fn test_readme_mentions_property_tests_or_testing_discipline_docs() {
    let readme = readme_text();
    assert!(readme.contains("cargo test --test proptest_invariants"));
}

#[test]
fn test_readme_mentions_property_tests() {
    let readme = readme_text();
    assert!(readme.contains("cargo test --test proptest_invariants"));
}

#[test]
fn test_help_mentions_bank_flags() {
    let mut command = CliArgs::command();
    let help = command.render_long_help().to_string();

    assert!(help.contains("--bank-mode"));
    assert!(help.contains("--bank-path"));
    assert!(help.contains("--strict-bank-validation"));
    assert!(help.contains("--bank-validation-mode"));
}

#[test]
fn test_help_mentions_sweep_mode_if_applicable() {
    let mut command = CliArgs::command();
    let help = command.render_long_help().to_string();

    assert!(help.contains("--sweep-family"));
    assert!(help.contains("--sweep-points"));
}

#[test]
fn test_docs_schema_file_exists() {
    assert!(crate_root().join("docs/schema.md").is_file());
    assert!(crate_root().join("docs/bank_schema.md").is_file());
}

#[test]
fn test_example_csv_workflow_exists() {
    assert!(crate_root()
        .join("docs/examples/illustrative_csv_example.md")
        .is_file());
    assert!(crate_root()
        .join("docs/examples/dashboard_replay.md")
        .is_file());
    assert!(crate_root()
        .join("docs/examples/ffi_integration.md")
        .is_file());
    assert!(crate_root()
        .join("docs/examples/synthetic_failure_injection.md")
        .is_file());
    assert!(crate_root()
        .join("docs/examples/vibration_to_thermal_drift.md")
        .is_file());
    assert!(crate_root().join("docs/examples/live_drop_in.md").is_file());
}

#[test]
fn test_readme_mentions_dashboard_if_dashboard_added() {
    let readme = readme_text();
    assert!(readme.contains("--dashboard-replay"));
    assert!(readme.contains("ratatui"));
}

#[test]
fn test_readme_mentions_dashboard() {
    let readme = readme_text();
    assert!(readme.contains("--dashboard-replay"));
    assert!(readme.contains("ratatui"));
}

#[test]
fn test_readme_mentions_dashboard_csv_replay() {
    let readme = readme_text();
    assert!(readme.contains("--dashboard-replay-csv"));
    assert!(readme.contains("CSV live replay"));
}

#[test]
fn test_readme_mentions_colab_download_buttons() {
    let readme = readme_text();
    assert!(readme.contains("one-click download"));
    assert!(readme.contains("PDF report and ZIP bundle"));
}

#[test]
fn test_readme_mentions_notebook_download_buttons() {
    let readme = readme_text();
    assert!(readme.contains("one-click download"));
    assert!(readme.contains("PDF report and ZIP bundle"));
}

#[test]
fn test_readme_mentions_bounded_online_history_and_numeric_mode() {
    let readme = readme_text();
    assert!(readme.contains("fixed-capacity ring buffer"));
    assert!(readme.contains("numeric-f32"));
}

#[test]
fn test_readme_mentions_f32_mode_if_added() {
    let readme = readme_text();
    assert!(readme.contains("numeric-f32"));
}

#[test]
fn test_readme_mentions_ring_buffer_or_bounded_memory_if_added() {
    let readme = readme_text();
    assert!(readme.contains("fixed-capacity ring buffer"));
}

#[test]
fn test_readme_mentions_ring_buffer_or_bounded_memory() {
    let readme = readme_text();
    assert!(readme.contains("fixed-capacity ring buffer"));
}

#[test]
fn test_readme_mentions_smoothing_if_added() {
    let readme = readme_text();
    assert!(readme.contains("smoothing"));
    assert!(readme.contains("low-latency smoothing"));
}

#[test]
fn test_readme_mentions_retrieval_indexing_if_added() {
    let readme = readme_text();
    assert!(readme.contains("prefilter index"));
    assert!(readme.contains("indexed or linear path"));
}

#[test]
fn test_readme_mentions_grammar_trust_if_added() {
    let readme = readme_text();
    assert!(readme.contains("trust scalar"));
    assert!(readme.contains("grammar severity"));
}

#[test]
fn test_readme_mentions_ffi_and_failure_injection_example() {
    let readme = readme_text();
    assert!(readme.contains("ffi/include/dsfb_semiotics_engine.h"));
    assert!(readme.contains("synthetic_failure_injection"));
}

#[test]
fn test_readme_mentions_ffi_if_added() {
    let readme = readme_text();
    assert!(readme.contains("ffi/include/dsfb_semiotics_engine.h"));
}

#[test]
fn test_readme_mentions_ffi() {
    let readme = readme_text();
    assert!(readme.contains("ffi/include/dsfb_semiotics_engine.h"));
    assert!(readme.contains("caller-owned buffers"));
}

#[test]
fn test_readme_mentions_drop_in_example_if_added() {
    let readme = readme_text();
    assert!(readme.contains("live_drop_in"));
    assert!(readme.contains("one-sample-at-a-time bounded loop"));
}

#[test]
fn test_csv_replay_example_command_present() {
    let docs = fs::read_to_string(crate_root().join("docs/examples/dashboard_replay.md")).unwrap();
    assert!(docs.contains("--dashboard-replay-csv"));
    assert!(docs.contains("--dashboard-playback-speed"));
}

#[test]
fn test_readme_mentions_operator_legible_comparator_context() {
    let readme = readme_text();
    assert!(readme.contains("EKF innovation monitoring"));
    assert!(readme.contains("chi-squared-style gating"));
}

#[test]
fn test_docs_include_embedded_core_roadmap() {
    let roadmap = crate_root().join("docs/embedded_core_roadmap.md");
    assert!(roadmap.is_file());
    let text = fs::read_to_string(roadmap).unwrap();
    assert!(text.contains("no_std"));
    assert!(text.contains("core"));
}

#[test]
fn test_core_candidate_modules_do_not_import_filesystem_or_plotting_layers() {
    let files = [
        "src/math/metrics.rs",
        "src/math/derivatives.rs",
        "src/engine/syntax_layer.rs",
        "src/engine/grammar_layer.rs",
        "src/engine/semantics_layer.rs",
    ];

    for file in files {
        let text = fs::read_to_string(crate_root().join(file)).unwrap();
        assert!(!text.contains("std::fs"));
        assert!(!text.contains("crate::figures"));
        assert!(!text.contains("crate::report"));
    }
}

#[test]
fn test_ci_workflow_file_present() {
    assert!(crate_root()
        .join(".github/workflows/crate-quality-gate.yml")
        .is_file());
}

#[test]
fn test_ci_workflow_mentions_external_bank_run() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("External-bank smoke run"));
    assert!(workflow.contains("--bank-mode external"));
}

#[test]
fn test_ci_workflow_mentions_property_tests_or_full_test_suite() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("Property invariants"));
    assert!(workflow.contains("cargo test --test proptest_invariants"));
}

#[test]
fn test_ci_workflow_mentions_figure_integrity_or_artifact_validation() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("Figure integrity smoke check"));
    assert!(workflow.contains("figure_integrity_report.json"));
}

#[test]
fn test_ci_workflow_mentions_dashboard_smoke_if_dashboard_added() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("Dashboard replay smoke run"));
    assert!(workflow.contains("--dashboard-replay"));
}

#[test]
fn test_ci_workflow_mentions_numeric_f32_and_ffi_smoke() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("numeric-f32 compile smoke"));
    assert!(workflow.contains("cargo check --features numeric-f32"));
    assert!(workflow.contains("FFI smoke compile"));
    assert!(workflow.contains("ffi/examples/minimal_ffi.c"));
    assert!(workflow.contains("ffi/examples/minimal_ffi.cpp"));
}

#[test]
fn test_ci_workflow_mentions_dashboard_csv_replay_and_forensics_smoke() {
    let workflow =
        fs::read_to_string(crate_root().join(".github/workflows/crate-quality-gate.yml")).unwrap();
    assert!(workflow.contains("Dashboard CSV replay smoke run"));
    assert!(workflow.contains("--dashboard-replay-csv"));
    assert!(workflow.contains("Forensics CLI smoke run"));
    assert!(workflow.contains("dsfb-forensics-gen"));
}
