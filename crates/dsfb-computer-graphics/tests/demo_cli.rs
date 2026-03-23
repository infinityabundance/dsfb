use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn unique_output_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("generated")
        .join("test_runs")
        .join(format!("{name}_{stamp}"));
    fs::create_dir_all(&dir).expect("test output directory should be creatable");
    dir
}

#[test]
fn cli_run_all_and_validate_artifacts_succeed() {
    let output_dir = unique_output_dir("cli_run_all");
    let binary = env!("CARGO_BIN_EXE_dsfb-computer-graphics");

    let status = Command::new(binary)
        .arg("run-all")
        .arg("--output")
        .arg(&output_dir)
        .status()
        .expect("binary should execute");
    assert!(status.success(), "run-all command should succeed");

    let validate = Command::new(binary)
        .arg("validate-artifacts")
        .arg("--output")
        .arg(&output_dir)
        .status()
        .expect("binary should execute");
    assert!(validate.success(), "validate-artifacts should succeed");

    for relative in [
        "artifact_manifest.json",
        "metrics.json",
        "report.md",
        "five_mentor_audit.md",
        "check_signing_blockers.md",
        "demo_b_decision_report.md",
        "demo_b/metrics.json",
        "demo_b/report.md",
    ] {
        assert!(
            output_dir.join(relative).exists(),
            "expected artifact {}",
            output_dir.join(relative).display()
        );
    }
}

#[test]
fn cli_single_scenario_and_ablation_commands_succeed() {
    let binary = env!("CARGO_BIN_EXE_dsfb-computer-graphics");

    let single_dir = unique_output_dir("cli_single_scenario");
    let status = Command::new(binary)
        .arg("run-demo-a")
        .arg("--scenario")
        .arg("thin_reveal")
        .arg("--output")
        .arg(&single_dir)
        .status()
        .expect("binary should execute");
    assert!(
        status.success(),
        "single-scenario run-demo-a should succeed"
    );

    let metrics_text =
        fs::read_to_string(single_dir.join("metrics.json")).expect("metrics should be readable");
    let metrics: Value = serde_json::from_str(&metrics_text).expect("metrics should be valid json");
    let scenario_ids = metrics["summary"]["scenario_ids"]
        .as_array()
        .expect("scenario ids should be an array");
    assert_eq!(
        scenario_ids.len(),
        1,
        "single-scenario run should stay scoped"
    );
    assert_eq!(scenario_ids[0].as_str(), Some("thin_reveal"));

    let ablation_dir = unique_output_dir("cli_ablations");
    let ablation_status = Command::new(binary)
        .arg("run-ablations")
        .arg("--output")
        .arg(&ablation_dir)
        .status()
        .expect("binary should execute");
    assert!(ablation_status.success(), "run-ablations should succeed");
    assert!(ablation_dir.join("ablation_report.md").exists());
}

#[test]
fn cli_run_demo_b_single_scenario_succeeds() {
    let output_dir = unique_output_dir("cli_demo_b_single");
    let binary = env!("CARGO_BIN_EXE_dsfb-computer-graphics");
    let status = Command::new(binary)
        .arg("run-demo-b")
        .arg("--scenario")
        .arg("thin_reveal")
        .arg("--output")
        .arg(&output_dir)
        .status()
        .expect("binary should execute");
    assert!(
        status.success(),
        "single-scenario run-demo-b should succeed"
    );

    for relative in [
        "demo_b/metrics.json",
        "demo_b/report.md",
        "demo_b/figures/fig_demo_b_sampling.svg",
        "demo_b/images/reference.png",
        "demo_b/images/uniform.png",
        "demo_b/images/imported_trust.png",
    ] {
        assert!(
            output_dir.join(relative).exists(),
            "expected artifact {}",
            output_dir.join(relative).display()
        );
    }
}
