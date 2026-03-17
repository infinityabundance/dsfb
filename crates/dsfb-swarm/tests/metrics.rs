use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use csv::Reader;

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

fn benchmark_like_run_config(scenario: ScenarioKind) -> RunConfig {
    RunConfig {
        scenario,
        steps: 120,
        agents: 20,
        dt: 0.08,
        interaction_radius: 1.45,
        k_neighbors: 4,
        base_gain: 1.0,
        noise_level: 0.01,
        warmup_steps: 24,
        multi_mode: true,
        monitored_modes: 4,
        mode_shapes: true,
        predictor: dsfb_swarm::config::PredictorKind::SmoothCorrective,
        trust_mode: dsfb_swarm::config::TrustGateMode::SmoothDecay,
        output_root: test_root("scenario-metrics"),
        report_pdf: true,
    }
}

#[test]
fn scenario_runner_produces_nonempty_diagnostics() -> Result<()> {
    let mut config = RunConfig::default_quickstart();
    config.scenario = ScenarioKind::Nominal;
    config.steps = 64;
    config.agents = 16;
    config.warmup_steps = 12;
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::Nominal, config.steps),
    )?;
    assert_eq!(run.time_series.len(), config.steps);
    assert!(!run.spectra.is_empty());
    assert!(!run.residuals.is_empty());
    assert!(!run.trust.is_empty());
    Ok(())
}

#[test]
fn nominal_run_keeps_false_positive_rate_low() -> Result<()> {
    let config = benchmark_like_run_config(ScenarioKind::Nominal);
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::Nominal, config.steps),
    )?;
    assert!(run.summary.scalar_false_positive_rate <= 0.02);
    assert!(run.summary.multimode_false_positive_rate <= 0.02);
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
    assert!(run_dir.join("hero_benchmark_summary.csv").exists());
    assert!(run_dir.join("detector_debug.csv").exists());
    for artifact in [
        "figures/scaling_curves.png",
        "figures/hero_leadtime_comparison.png",
        "figures/hero_benchmark_table.png",
        "figures/adversarial_trust_detection_focus.png",
    ] {
        let path = run_dir.join(artifact);
        assert!(path.exists(), "missing {artifact}");
        assert!(path.metadata()?.len() > 0, "empty {artifact}");
    }
    assert!(run_dir.join("hero_benchmark_summary.csv").metadata()?.len() > 0);
    assert!(run_dir.join("report/dsfb_swarm_report.pdf").exists());
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn hero_summary_contains_required_scenarios_and_winner_column() -> Result<()> {
    let root = test_root("hero-summary");
    let config = BenchmarkConfig {
        steps: 120,
        sizes: vec![20, 100],
        noise_levels: vec![0.01],
        scenarios: vec![
            ScenarioKind::GradualEdgeDegradation,
            ScenarioKind::AdversarialAgent,
            ScenarioKind::CommunicationLoss,
        ],
        multi_mode: true,
        monitored_modes: 4,
        mode_shapes: true,
        predictor: dsfb_swarm::config::PredictorKind::SmoothCorrective,
        trust_mode: dsfb_swarm::config::TrustGateMode::SmoothDecay,
        output_root: root.clone(),
    };
    let run_dir = run_benchmark_suite(config)?;
    let mut reader = Reader::from_path(run_dir.join("hero_benchmark_summary.csv"))?;
    let headers = reader.headers()?.clone();
    assert!(headers.iter().any(|header| header == "winner"));
    assert!(headers
        .iter()
        .any(|header| header == "dsfb_advantage_margin"));
    let rows = reader
        .deserialize::<std::collections::BTreeMap<String, String>>()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for scenario in [
        "gradual_edge_degradation",
        "adversarial_agent",
        "communication_loss",
    ] {
        let row = rows
            .iter()
            .find(|row| row.get("scenario").map(String::as_str) == Some(scenario))
            .expect("hero scenario row");
        assert!(!row.get("winner").unwrap_or(&String::new()).is_empty());
    }
    for scenario in ["gradual_edge_degradation", "communication_loss"] {
        let row = rows
            .iter()
            .find(|row| row.get("scenario").map(String::as_str) == Some(scenario))
            .expect("hero scenario row");
        assert!(!row
            .get("dsfb_advantage_margin")
            .unwrap_or(&String::new())
            .is_empty());
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn communication_loss_exports_positive_scalar_lead_time() -> Result<()> {
    let config = benchmark_like_run_config(ScenarioKind::CommunicationLoss);
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::CommunicationLoss, config.steps),
    )?;
    assert!(run.summary.scalar_detection_step.is_some());
    assert!(run.summary.scalar_detection_lead_time.unwrap_or(0.0) > 0.0);
    assert!(run.summary.scalar_true_positive_rate > 0.0);
    Ok(())
}

#[test]
fn large_scale_communication_loss_keeps_meaningful_detection() -> Result<()> {
    let mut config = benchmark_like_run_config(ScenarioKind::CommunicationLoss);
    config.agents = 100;
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::CommunicationLoss, config.steps),
    )?;
    assert!(
        run.summary.scalar_detection_lead_time.unwrap_or(0.0) > 0.0
            || run.summary.multimode_detection_lead_time.unwrap_or(0.0) > 0.0
    );
    assert!(run.summary.scalar_detection_step.is_some());
    Ok(())
}

#[test]
fn nominal_tail_multimode_overfire_stays_low_at_scale() -> Result<()> {
    let mut config = benchmark_like_run_config(ScenarioKind::Nominal);
    config.agents = 100;
    config.noise_level = 0.10;
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::Nominal, config.steps),
    )?;
    assert!(run.summary.multimode_false_positive_rate <= 0.02);
    Ok(())
}

#[test]
fn gradual_degradation_exports_persistent_drift_and_lead_time() -> Result<()> {
    let config = benchmark_like_run_config(ScenarioKind::GradualEdgeDegradation);
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::GradualEdgeDegradation, config.steps),
    )?;
    assert!(
        run.summary.scalar_detection_step.is_some()
            || run.summary.multimode_detection_step.is_some()
    );
    assert!(
        run.summary.scalar_detection_lead_time.unwrap_or(0.0) > 0.0
            || run.summary.multimode_detection_lead_time.unwrap_or(0.0) > 0.0
    );
    assert!(run
        .time_series
        .iter()
        .skip(run.summary.onset_step)
        .any(|row| row.scalar_drift < -0.25 && row.scalar_combined_ratio > 0.5));
    Ok(())
}

#[test]
fn adversarial_run_shows_trust_delay_and_multimode_advantage() -> Result<()> {
    let config = benchmark_like_run_config(ScenarioKind::AdversarialAgent);
    let run = run_scenario(
        &config,
        ScenarioDefinition::from_kind(ScenarioKind::AdversarialAgent, config.steps),
    )?;
    assert!(run.summary.trust_suppression_delay.unwrap_or(0.0) > 0.0);
    assert!(run.summary.multimode_minus_scalar_seconds.unwrap_or(0.0) > 0.0);
    assert!(run
        .summary
        .multimode_detection_step
        .zip(run.summary.scalar_detection_step)
        .map(|(multi, scalar)| multi < scalar)
        .unwrap_or(false));
    Ok(())
}

#[test]
fn benchmark_summary_contains_calibrated_metric_columns() -> Result<()> {
    let root = test_root("benchmark-metrics");
    let config = BenchmarkConfig {
        steps: 120,
        sizes: vec![20],
        noise_levels: vec![0.01],
        scenarios: vec![
            ScenarioKind::GradualEdgeDegradation,
            ScenarioKind::AdversarialAgent,
            ScenarioKind::CommunicationLoss,
        ],
        multi_mode: true,
        monitored_modes: 4,
        mode_shapes: true,
        predictor: dsfb_swarm::config::PredictorKind::SmoothCorrective,
        trust_mode: dsfb_swarm::config::TrustGateMode::SmoothDecay,
        output_root: root.clone(),
    };
    let run_dir = run_benchmark_suite(config)?;
    let mut reader = Reader::from_path(run_dir.join("benchmark_summary.csv"))?;
    let headers = reader.headers()?.clone();
    for expected in [
        "visible_failure_step",
        "scalar_detection_step",
        "multimode_detection_step",
        "best_baseline_name",
        "best_baseline_lead_time",
        "lead_time_gain_vs_best_baseline",
        "dsfb_advantage_margin",
        "tpr_gain_vs_best_baseline",
        "fpr_delta_vs_best_baseline",
        "fpr_reduction_vs_best_baseline",
        "baseline_lambda2_lead_time",
        "multimode_minus_scalar_seconds",
        "trust_drop_step",
        "trust_suppression_delay",
        "peak_mode_shape_norm",
        "peak_stack_score",
    ] {
        assert!(
            headers.iter().any(|header| header == expected),
            "missing {expected}"
        );
    }

    let rows = reader
        .deserialize::<std::collections::BTreeMap<String, String>>()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let communication = rows
        .iter()
        .find(|row| row.get("scenario").map(String::as_str) == Some("communication_loss"))
        .expect("communication_loss row");
    assert!(!communication
        .get("scalar_detection_lead_time")
        .unwrap_or(&String::new())
        .is_empty());
    assert!(!communication
        .get("best_baseline_name")
        .unwrap_or(&String::new())
        .is_empty());
    assert!(!communication
        .get("lead_time_gain_vs_best_baseline")
        .unwrap_or(&String::new())
        .is_empty());
    assert!(!communication
        .get("dsfb_advantage_margin")
        .unwrap_or(&String::new())
        .is_empty());

    let adversarial = rows
        .iter()
        .find(|row| row.get("scenario").map(String::as_str) == Some("adversarial_agent"))
        .expect("adversarial_agent row");
    assert!(!adversarial
        .get("multimode_minus_scalar_seconds")
        .unwrap_or(&String::new())
        .is_empty());
    assert!(!adversarial
        .get("trust_suppression_delay")
        .unwrap_or(&String::new())
        .is_empty());

    let _ = fs::remove_dir_all(root);
    Ok(())
}
