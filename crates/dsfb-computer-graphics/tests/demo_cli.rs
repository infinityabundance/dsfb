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
        .arg("validate-final")
        .arg("--output")
        .arg(&output_dir)
        .status()
        .expect("binary should execute");
    assert!(validate.success(), "validate-final should succeed");

    for relative in [
        "artifact_manifest.json",
        "metrics.json",
        "report.md",
        "five_mentor_audit.md",
        "check_signing_blockers.md",
        "trust_mode_report.md",
        "gpu_execution_report.md",
        "gpu_execution_metrics.json",
        "external_replay_report.md",
        "external_handoff_report.md",
        "realism_suite_report.md",
        "realism_bridge_report.md",
        "scenario_taxonomy.json",
        "competitive_baseline_analysis.md",
        "non_roi_penalty_report.md",
        "product_positioning_report.md",
        "operating_band_report.md",
        "demo_b_decision_report.md",
        "demo_b_competitive_baselines_report.md",
        "demo_b_aliasing_vs_variance_report.md",
        "production_eval_checklist.md",
        "evaluator_handoff.md",
        "minimum_external_validation_plan.md",
        "next_step_matrix.md",
        "check_signing_readiness.md",
        "demo_b/metrics.json",
        "demo_b/report.md",
        "external_demo/resolved_external_capture_manifest.json",
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

#[test]
fn cli_gpu_external_realism_and_handoff_commands_succeed() {
    let binary = env!("CARGO_BIN_EXE_dsfb-computer-graphics");

    let gpu_dir = unique_output_dir("cli_gpu_path");
    let gpu_status = Command::new(binary)
        .arg("run-gpu-path")
        .arg("--output")
        .arg(&gpu_dir)
        .status()
        .expect("binary should execute");
    assert!(gpu_status.success(), "run-gpu-path should succeed");
    assert!(gpu_dir.join("gpu_execution_report.md").exists());
    assert!(gpu_dir.join("gpu_execution_metrics.json").exists());

    let realism_dir = unique_output_dir("cli_realism_suite");
    let realism_status = Command::new(binary)
        .arg("run-realism-bridge")
        .arg("--output")
        .arg(&realism_dir)
        .status()
        .expect("binary should execute");
    assert!(realism_status.success(), "run-realism-bridge should succeed");
    assert!(realism_dir.join("realism_suite_report.md").exists());
    assert!(realism_dir.join("realism_bridge_report.md").exists());
    assert!(realism_dir.join("scenario_taxonomy.json").exists());

    let external_dir = unique_output_dir("cli_external_import");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("external_capture_manifest.json");
    let external_status = Command::new(binary)
        .arg("run-external-replay")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--output")
        .arg(&external_dir)
        .status()
        .expect("binary should execute");
    assert!(external_status.success(), "run-external-replay should succeed");
    assert!(external_dir.join("external_replay_report.md").exists());
    assert!(external_dir.join("external_handoff_report.md").exists());
    assert!(
        external_dir
            .join("resolved_external_capture_manifest.json")
            .exists()
    );

    let handoff_dir = unique_output_dir("cli_evaluator_handoff");
    let handoff_status = Command::new(binary)
        .arg("export-evaluator-handoff")
        .arg("--output")
        .arg(&handoff_dir)
        .status()
        .expect("binary should execute");
    assert!(
        handoff_status.success(),
        "export-evaluator-handoff should succeed"
    );
    assert!(handoff_dir.join("evaluator_handoff.md").exists());
    assert!(handoff_dir.join("production_eval_checklist.md").exists());
    assert!(handoff_dir.join("next_step_matrix.md").exists());
}
