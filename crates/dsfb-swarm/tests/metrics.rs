use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use dsfb_swarm::config::{BenchmarkConfig, RunConfig, ScenarioKind};
use dsfb_swarm::report::run_benchmark_suite;
use dsfb_swarm::sim::runner::run_scenario;
use dsfb_swarm::sim::scenarios::ScenarioDefinition;

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
fn scenario_runner_produces_nonempty_diagnostics() -> Result<()> {
    let mut config = RunConfig::default_quickstart();
    config.scenario = ScenarioKind::Nominal;
    config.steps = 64;
    config.agents = 16;
    config.warmup_steps = 12;
    let run = run_scenario(&config, ScenarioDefinition::from_kind(ScenarioKind::Nominal, config.steps))?;
    assert_eq!(run.time_series.len(), config.steps);
    assert!(!run.spectra.is_empty());
    assert!(!run.residuals.is_empty());
    assert!(!run.trust.is_empty());
    Ok(())
}

#[test]
fn benchmark_artifact_generation_smoke() -> Result<()> {
    let root = test_root("benchmark");
    let config = BenchmarkConfig {
        steps: 48,
        sizes: vec![12, 18],
        noise_levels: vec![0.01],
        scenarios: vec![ScenarioKind::Nominal, ScenarioKind::CommunicationLoss],
        multi_mode: true,
        monitored_modes: 3,
        mode_shapes: true,
        predictor: dsfb_swarm::config::PredictorKind::SmoothCorrective,
        trust_mode: dsfb_swarm::config::TrustGateMode::SmoothDecay,
        output_root: root.clone(),
    };
    let run_dir = run_benchmark_suite(config)?;
    assert!(run_dir.join("benchmark_summary.csv").exists());
    assert!(run_dir.join("figures/scaling_curves.png").exists());
    assert!(run_dir.join("report/dsfb_swarm_report.pdf").exists());
    let _ = fs::remove_dir_all(root);
    Ok(())
}
