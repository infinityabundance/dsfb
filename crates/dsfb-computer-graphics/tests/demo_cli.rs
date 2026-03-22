use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
fn cli_demo_a_completes_and_writes_required_artifacts() {
    let output_dir = unique_output_dir("cli_demo_a");
    let binary = env!("CARGO_BIN_EXE_dsfb-computer-graphics");
    let status = Command::new(binary)
        .arg("run-demo-a")
        .arg("--output")
        .arg(&output_dir)
        .status()
        .expect("binary should execute");
    assert!(status.success(), "demo command should succeed");

    for relative in [
        "metrics.json",
        "report.md",
        "scene_manifest.json",
        "figures/fig_system_diagram.svg",
        "figures/fig_trust_map.svg",
        "figures/fig_before_after.svg",
        "figures/fig_trust_vs_error.svg",
    ] {
        let path = output_dir.join(relative);
        assert!(path.exists(), "expected artifact {}", path.display());
        let metadata = fs::metadata(&path).expect("artifact metadata should exist");
        assert!(
            metadata.len() > 0,
            "artifact {} should be non-empty",
            path.display()
        );
    }
}
