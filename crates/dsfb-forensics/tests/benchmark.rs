use std::path::PathBuf;

use dsfb_forensics::auditor::{infer_initial_state, ForensicAuditor, ForensicConfig};
use dsfb_forensics::benchmark::{
    generate_trace, write_trace_csv, BenchmarkConfig, BenchmarkScenario, BenchmarkWriteTrace,
};
use dsfb_forensics::cli::BaselineComparison;
use dsfb_forensics::fs::{create_run_directory_at, write_json, write_text};
use dsfb_forensics::input::load_trace;
use dsfb_forensics::report::render_markdown_report;
use tempfile::tempdir;

#[test]
fn benchmark_trace_generation_is_reproducible() {
    let config = latent_config();
    let left = generate_trace(&config).expect("first trace");
    let right = generate_trace(&config).expect("second trace");
    assert_eq!(left, right);
}

#[test]
fn generated_benchmark_trace_round_trips_through_loader() {
    let config = latent_config();
    let trace = generate_trace(&config).expect("trace");
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("trace.csv");
    write_trace_csv(&path, &trace).expect("write trace");
    let loaded = load_trace(&path).expect("load trace");
    assert_eq!(trace.channel_names, loaded.channel_names);
    assert_eq!(trace.steps.len(), loaded.steps.len());
    assert_eq!(trace.steps[0].measurements.len(), loaded.steps[0].measurements.len());
}

#[test]
fn healthy_benchmark_fixture_does_not_claim_early_warning() {
    let fixture = fixture_path("benchmark_healthy_reference.csv");
    let trace = load_trace(&fixture).expect("fixture trace");
    let run = audit_fixture(&trace, "benchmark_healthy_reference.csv", healthy_config());
    assert_eq!(run.summary.benchmark_scenario.as_deref(), Some("healthy-reference"));
    assert_eq!(run.summary.conventional_qa_fail_step, None);
    assert_eq!(run.summary.dsfb_first_alert_step, None);
    assert!(!run.summary.degradation_detected_early);
}

#[test]
fn latent_drift_fixture_yields_early_warning() {
    let fixture = fixture_path("benchmark_latent_signature_drift.csv");
    let trace = load_trace(&fixture).expect("fixture trace");
    let run = audit_fixture(&trace, "benchmark_latent_signature_drift.csv", latent_config());
    assert_eq!(run.summary.dsfb_first_alert_step, Some(26));
    assert_eq!(run.summary.conventional_qa_fail_step, Some(38));
    assert_eq!(run.summary.dsfb_lead_time_steps, Some(12));
    assert!(run.summary.degradation_detected_early);
}

#[test]
fn fragmentation_ramp_fixture_detects_quickly() {
    let fixture = fixture_path("benchmark_channel_fragmentation_ramp.csv");
    let trace = load_trace(&fixture).expect("fixture trace");
    let run = audit_fixture(
        &trace,
        "benchmark_channel_fragmentation_ramp.csv",
        obvious_failure_config(),
    );
    assert_eq!(run.summary.dsfb_first_alert_step, Some(14));
    assert_eq!(run.summary.conventional_qa_fail_step, Some(15));
    assert_eq!(run.summary.dsfb_lead_time_steps, Some(1));
    assert!(run.summary.degradation_detected_early);
}

#[test]
fn benchmark_artifacts_are_written() {
    let config = latent_config();
    let trace = generate_trace(&config).expect("trace");
    let run = audit_fixture(&trace, "latent-generated", config.clone());
    let workspace_root = tempdir().expect("tempdir");
    let run_dir = create_run_directory_at(workspace_root.path()).expect("run dir");

    write_json(&run_dir.run_dir.join("benchmark_config.json"), &config).expect("benchmark config");
    write_trace_csv(&run_dir.run_dir.join("benchmark_trace.csv"), &trace).expect("benchmark trace");
    write_json(&run_dir.run_dir.join("causal_trace.json"), &run.causal_trace).expect("causal trace");
    write_json(&run_dir.run_dir.join("forensic_report.json"), &run.summary).expect("report json");
    write_text(
        &run_dir.run_dir.join("forensic_report.md"),
        &render_markdown_report(&run.summary),
    )
    .expect("report markdown");

    assert!(run_dir.run_dir.join("benchmark_config.json").exists());
    assert!(run_dir.run_dir.join("benchmark_trace.csv").exists());
    assert!(run_dir.run_dir.join("causal_trace.json").exists());
    assert!(run_dir.run_dir.join("forensic_report.json").exists());
    assert!(run_dir.run_dir.join("forensic_report.md").exists());
}

fn audit_fixture(
    trace: &dsfb_forensics::TraceDocument,
    label: &str,
    config: BenchmarkConfig,
) -> dsfb_forensics::AuditRun {
    let initial_state = infer_initial_state(trace).expect("initial state");
    let mut auditor = ForensicAuditor::new(
        ForensicConfig {
            slew_threshold: 6.0,
            trust_alpha: 0.20,
            baseline_comparison: BaselineComparison::On,
        },
        &trace.channel_names,
        initial_state,
    );
    auditor
        .audit_trace_with_benchmark(trace, label, Some(&config))
        .expect("audit run")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(name)
}

fn healthy_config() -> BenchmarkConfig {
    BenchmarkConfig {
        scenario: BenchmarkScenario::HealthyReference,
        step_count: 30,
        dt: 0.25,
        channel_count: 4,
        drift_start_step: 12,
        drift_ramp_rate: 0.02,
        drift_amplitude_ceiling: 0.35,
        conventional_qa_threshold: 0.40,
        jitter_level: 0.015,
        anomaly_channels: vec![2],
        recovery_step: None,
        alert_consecutive_steps: 3,
        write_trace: BenchmarkWriteTrace::On,
    }
}

fn latent_config() -> BenchmarkConfig {
    BenchmarkConfig {
        scenario: BenchmarkScenario::LatentSignatureDrift,
        step_count: 40,
        dt: 0.25,
        channel_count: 4,
        drift_start_step: 12,
        drift_ramp_rate: 0.02,
        drift_amplitude_ceiling: 0.35,
        conventional_qa_threshold: 0.40,
        jitter_level: 0.015,
        anomaly_channels: vec![2],
        recovery_step: None,
        alert_consecutive_steps: 3,
        write_trace: BenchmarkWriteTrace::On,
    }
}

fn obvious_failure_config() -> BenchmarkConfig {
    BenchmarkConfig {
        scenario: BenchmarkScenario::ChannelFragmentationRamp,
        step_count: 30,
        dt: 0.25,
        channel_count: 4,
        drift_start_step: 8,
        drift_ramp_rate: 0.05,
        drift_amplitude_ceiling: 0.65,
        conventional_qa_threshold: 0.40,
        jitter_level: 0.015,
        anomaly_channels: vec![2],
        recovery_step: None,
        alert_consecutive_steps: 3,
        write_trace: BenchmarkWriteTrace::On,
    }
}
