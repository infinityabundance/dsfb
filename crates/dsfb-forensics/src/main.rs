use anyhow::{bail, Result};
use clap::Parser;

use dsfb_forensics::auditor::{infer_initial_state, ForensicAuditor, ForensicConfig};
use dsfb_forensics::benchmark::BenchmarkConfig;
use dsfb_forensics::cli::Cli;
use dsfb_forensics::fs::{create_run_directory, write_json, write_text};
use dsfb_forensics::input::load_trace;
use dsfb_forensics::report::render_markdown_report;
use dsfb_forensics::{generate_benchmark_trace, write_benchmark_trace_csv};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let benchmark_config = benchmark_config_from_cli(&cli)?;
    let (trace, input_label) = if let Some(config) = benchmark_config.as_ref() {
        if cli.input_trace.is_some() {
            bail!("--input-trace cannot be combined with a built-in benchmark scenario");
        }
        (
            generate_benchmark_trace(config)?,
            format!("built-in benchmark: {}", config.scenario.as_str()),
        )
    } else {
        let input_trace = cli
            .input_trace
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--input-trace is required when --benchmark-scenario none"))?;
        (load_trace(input_trace)?, input_trace.display().to_string())
    };
    let initial_state = infer_initial_state(&trace)?;
    let mut auditor = ForensicAuditor::new(
        ForensicConfig {
            slew_threshold: cli.slew_threshold,
            trust_alpha: cli.trust_alpha,
            baseline_comparison: cli.baseline_comparison,
        },
        &trace.channel_names,
        initial_state,
    );
    let run = auditor.audit_trace_with_benchmark(&trace, &input_label, benchmark_config.as_ref())?;
    let run_dir = create_run_directory()?;
    let causal_trace_path = run_dir.run_dir.join("causal_trace.json");
    let markdown_path = run_dir.run_dir.join("forensic_report.md");

    if let Some(config) = benchmark_config.as_ref() {
        write_json(&run_dir.run_dir.join("benchmark_config.json"), config)?;
        if config.write_trace.enabled() {
            write_benchmark_trace_csv(&run_dir.run_dir.join("benchmark_trace.csv"), &trace)?;
        }
    }

    write_json(&causal_trace_path, &run.causal_trace)?;
    write_text(&markdown_path, &render_markdown_report(&run.summary))?;
    if cli.report_format.writes_json() {
        write_json(&run_dir.run_dir.join("forensic_report.json"), &run.summary)?;
    }

    println!("dsfb-forensics completed");
    println!("output: {}", run_dir.run_dir.display());
    println!("seal: {:?}", run.summary.seal);
    println!("shatter_events: {}", run.summary.shatter_events);
    println!("silent_failures: {}", run.summary.silent_failures);
    println!("reasoning_consistency: {:.3}", run.summary.reasoning_consistency);
    if let Some(step) = run.summary.dsfb_first_alert_step {
        println!("dsfb_first_alert_step: {}", step);
    }
    if let Some(step) = run.summary.conventional_qa_fail_step {
        println!("conventional_qa_fail_step: {}", step);
    }
    println!(
        "degradation_detected_early: {}",
        run.summary.degradation_detected_early
    );
    Ok(())
}

fn benchmark_config_from_cli(cli: &Cli) -> Result<Option<BenchmarkConfig>> {
    if !cli.benchmark_scenario.enabled() {
        return Ok(None);
    }
    Ok(Some(BenchmarkConfig {
        scenario: cli.benchmark_scenario,
        step_count: cli.benchmark_steps,
        dt: cli.benchmark_dt,
        channel_count: cli.benchmark_channel_count,
        drift_start_step: cli.benchmark_drift_start,
        drift_ramp_rate: cli.benchmark_drift_rate,
        drift_amplitude_ceiling: cli.benchmark_drift_max,
        conventional_qa_threshold: cli.benchmark_qa_threshold,
        jitter_level: cli.benchmark_jitter_level,
        anomaly_channels: cli.benchmark_anomaly_channels.clone(),
        recovery_step: cli.benchmark_recovery_step,
        alert_consecutive_steps: cli.benchmark_alert_consecutive_steps,
        write_trace: cli.benchmark_write_trace,
    }))
}
