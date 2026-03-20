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
}

#[test]
fn test_readme_mentions_property_tests_or_testing_discipline_docs() {
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
}

#[test]
fn test_readme_mentions_dashboard_if_dashboard_added() {
    let readme = readme_text();
    assert!(readme.contains("--dashboard-replay"));
    assert!(readme.contains("ratatui"));
}

#[test]
fn test_readme_mentions_colab_download_buttons() {
    let readme = readme_text();
    assert!(readme.contains("one-click download links"));
    assert!(readme.contains("PDF report and ZIP bundle"));
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
