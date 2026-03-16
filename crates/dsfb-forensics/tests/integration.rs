use std::path::PathBuf;

use dsfb_forensics::auditor::{infer_initial_state, ForensicAuditor, ForensicConfig};
use dsfb_forensics::cli::BaselineComparison;
use dsfb_forensics::fs::{create_run_directory_at, write_json, write_text};
use dsfb_forensics::input::load_trace;
use dsfb_forensics::report::render_markdown_report;
use tempfile::tempdir;

#[test]
fn fixture_trace_emits_artifacts_and_detects_structural_debt() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("fixtures/example_trace.csv");
    let trace = load_trace(&fixture).expect("fixture trace loads");
    let initial_state = infer_initial_state(&trace).expect("initial state");
    let mut auditor = ForensicAuditor::new(
        ForensicConfig {
            slew_threshold: 6.0,
            trust_alpha: 0.20,
            baseline_comparison: BaselineComparison::On,
        },
        &trace.channel_names,
        initial_state,
    );
    let run = auditor
        .audit_trace(&trace, fixture.to_string_lossy().as_ref())
        .expect("audit run succeeds");

    assert_eq!(run.summary.total_steps, trace.steps.len());
    assert!(run.summary.shatter_events >= 1);
    assert!(run.summary.pruned_updates >= 1);

    let workspace_root = tempdir().expect("tempdir");
    let run_dir = create_run_directory_at(workspace_root.path()).expect("run directory");
    write_json(&run_dir.run_dir.join("causal_trace.json"), &run.causal_trace).expect("trace json");
    write_text(
        &run_dir.run_dir.join("forensic_report.md"),
        &render_markdown_report(&run.summary),
    )
    .expect("markdown report");

    assert!(run_dir.run_dir.join("causal_trace.json").exists());
    assert!(run_dir.run_dir.join("forensic_report.md").exists());
}
