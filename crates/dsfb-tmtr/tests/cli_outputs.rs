use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn temp_output_root() -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "dsfb-tmtr-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_nanos()
    ));
    root
}

#[test]
fn cli_writes_expected_artifacts() {
    let output_root = temp_output_root();
    let binary = env!("CARGO_BIN_EXE_dsfb-tmtr");
    let status = Command::new(binary)
        .args([
            "--scenario",
            "disturbance-recovery",
            "--n-steps",
            "180",
            "--output-root",
            output_root.to_str().expect("utf-8 path"),
        ])
        .status()
        .expect("cli status");
    assert!(status.success());

    let run_dirs = fs::read_dir(&output_root)
        .expect("run dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    assert_eq!(run_dirs.len(), 1);
    let run_dir = run_dirs[0].path();
    for artifact in [
        "run_manifest.json",
        "config.json",
        "scenario_summary.csv",
        "trajectories.csv",
        "trust_timeseries.csv",
        "residuals.csv",
        "correction_events.csv",
        "prediction_tubes.csv",
        "causal_edges.csv",
        "causal_metrics.csv",
        "notebook_ready_summary.json",
    ] {
        assert!(run_dir.join(artifact).exists(), "missing {artifact}");
    }

    let _ = fs::remove_dir_all(output_root);
}
