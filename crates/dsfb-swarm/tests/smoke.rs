use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use dsfb_swarm::config::{create_timestamped_run_directory, ResolvedCommand, RunConfig};
use dsfb_swarm::report::run_scenario_bundle;

fn test_root(name: &str) -> PathBuf {
    let unique = format!(
        "{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix epoch")
            .as_nanos()
    );
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-output")
        .join(unique)
}

#[test]
fn timestamped_output_directory_does_not_overwrite() -> Result<()> {
    let root = test_root("timestamp");
    let first = create_timestamped_run_directory(&root)?;
    let second = create_timestamped_run_directory(&root)?;
    assert_ne!(first.run_dir, second.run_dir);
    assert!(first.run_dir.exists());
    assert!(second.run_dir.exists());
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn quickstart_writes_expected_artifacts() -> Result<()> {
    let root = test_root("quickstart");
    let mut config = RunConfig::default_quickstart();
    config.output_root = root.clone();
    let run_dir = run_scenario_bundle(ResolvedCommand::Quickstart(config))?;
    for artifact in [
        "manifest.json",
        "run_config.json",
        "scenarios_summary.csv",
        "benchmark_summary.csv",
        "hero_benchmark_summary.csv",
        "time_series.csv",
        "detector_debug.csv",
        "spectra.csv",
        "residuals.csv",
        "trust.csv",
        "baselines.csv",
        "anomalies.json",
        "figures/lambda2_timeseries.png",
        "figures/residual_timeseries.png",
        "figures/drift_slew.png",
        "figures/trust_evolution.png",
        "figures/baseline_comparison.png",
        "figures/scaling_curves.png",
        "figures/noise_stress_curves.png",
        "figures/multimode_comparison.png",
        "figures/topology_snapshots.png",
        "figures/hero_leadtime_comparison.png",
        "figures/hero_benchmark_table.png",
        "report/dsfb_swarm_report.md",
        "report/dsfb_swarm_report.pdf",
    ] {
        assert!(run_dir.join(artifact).exists(), "missing {artifact}");
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}
