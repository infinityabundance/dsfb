use anyhow::Result;
use clap::Parser;

use dsfb_forensics::auditor::{infer_initial_state, ForensicAuditor, ForensicConfig};
use dsfb_forensics::cli::Cli;
use dsfb_forensics::fs::{create_run_directory, write_json, write_text};
use dsfb_forensics::input::load_trace;
use dsfb_forensics::report::render_markdown_report;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let trace = load_trace(&cli.input_trace)?;
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
    let run = auditor.audit_trace(&trace, &cli.input_trace.display().to_string())?;
    let run_dir = create_run_directory()?;
    let causal_trace_path = run_dir.run_dir.join("causal_trace.json");
    let markdown_path = run_dir.run_dir.join("forensic_report.md");

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
    Ok(())
}
