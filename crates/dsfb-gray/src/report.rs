//! # DSFB Report Generator
//!
//! Produces human-readable, deterministic, audit-traced gray failure
//! analysis reports from scenario results.

use crate::{GrammarState, ScenarioResult, CONTRACT_VERSION, CRATE_VERSION};

/// Generate a plain-text report from scenario results.
pub fn generate_report(result: &ScenarioResult) -> String {
    let mut report = String::with_capacity(4096);
    push_report_header(&mut report, result);
    push_detection_summary(&mut report, result);
    let grammar_counts = grammar_state_counts(result);
    push_grammar_distribution(&mut report, result, grammar_counts);
    push_detection_point_state(&mut report, result);
    push_residual_trajectory(&mut report, result);
    push_non_interference_contract(&mut report);
    push_report_metadata(&mut report);
    report
}

fn push_report_header(report: &mut String, result: &ScenarioResult) {
    report.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    report.push_str("║         DSFB Gray Failure Detection Report                  ║\n");
    report.push_str("║         Deterministic Structural Semiotics Engine           ║\n");
    report.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");
    report.push_str(&format!("Scenario: {}\n", result.scenario_name));
    report.push_str(&format!("Total Steps: {}\n", result.total_steps));
    report.push_str(&format!(
        "Injection Start: step {}\n",
        result.injection_start
    ));
    report.push_str(&format!(
        "Expected Reason Code: {:?}\n",
        result.expected_reason_code
    ));
    match result.detected_reason_code {
        Some(reason) => report.push_str(&format!("Detected Reason Code: {:?}\n\n", reason)),
        None => report.push_str("Detected Reason Code: None\n\n"),
    }
}

fn push_detection_summary(report: &mut String, result: &ScenarioResult) {
    report.push_str("── Detection Summary ──────────────────────────────────────────\n\n");
    if result.detected() {
        push_detected_summary(report, result);
    } else {
        push_undetected_summary(report);
    }
    report.push_str(&format!(
        "\n  Total Boundary Steps: {}\n",
        result.total_boundary_steps
    ));
    report.push_str(&format!(
        "  Total Violation Steps: {}\n",
        result.total_violation_steps
    ));
}

fn push_detected_summary(report: &mut String, result: &ScenarioResult) {
    report.push_str("  ✓ FAULT DETECTED\n");
    if let Some(step) = result.first_anomaly_step {
        report.push_str(&format!("  First Anomaly:   step {}\n", step));
    }
    if let Some(step) = result.first_boundary_step {
        report.push_str(&format!("  First Boundary:  step {}\n", step));
    }
    if let Some(step) = result.first_violation_step {
        report.push_str(&format!("  First Violation: step {}\n", step));
    }
    match result.detection_delay_from_injection() {
        Some(delay) => {
            report.push_str(&format!(
                "  Detection Delay: {} steps after injection\n",
                delay
            ));
        }
        None => report.push_str("  Detection Delay: pre-injection false alarm\n"),
    }
    let detection_lead = result
        .detection_lead_time()
        .map_or_else(|| "unknown".to_string(), |value| value.to_string());
    report.push_str(&format!(
        "  Detection Lead:  {detection_lead} steps before scenario end\n"
    ));
    report.push_str(&format!(
        "  False Alarms:    {} (before injection)\n",
        result.false_alarms_before_injection
    ));
}

fn push_undetected_summary(report: &mut String) {
    report.push_str("  ✗ FAULT NOT DETECTED\n");
    report.push_str("  The DSFB observer did not transition to Boundary or Violation.\n");
    report.push_str("  Consider: envelope too wide, persistence window too large,\n");
    report.push_str("  or drift rate too low for the configured sensitivity.\n");
}

fn grammar_state_counts(result: &ScenarioResult) -> (usize, usize, usize) {
    let admissible = result
        .samples
        .iter()
        .filter(|sample| sample.grammar_state == GrammarState::Admissible)
        .count();
    let boundary = result
        .samples
        .iter()
        .filter(|sample| sample.grammar_state == GrammarState::Boundary)
        .count();
    let violation = result
        .samples
        .iter()
        .filter(|sample| sample.grammar_state == GrammarState::Violation)
        .count();
    (admissible, boundary, violation)
}

fn push_grammar_distribution(
    report: &mut String,
    result: &ScenarioResult,
    (admissible, boundary, violation): (usize, usize, usize),
) {
    report.push_str("\n── Grammar State Distribution ─────────────────────────────────\n\n");
    report.push_str(&format!(
        "  Admissible: {} ({:.1}%)\n",
        admissible,
        admissible as f64 / result.samples.len() as f64 * 100.0
    ));
    report.push_str(&format!(
        "  Boundary:   {} ({:.1}%)\n",
        boundary,
        boundary as f64 / result.samples.len() as f64 * 100.0
    ));
    report.push_str(&format!(
        "  Violation:  {} ({:.1}%)\n",
        violation,
        violation as f64 / result.samples.len() as f64 * 100.0
    ));
}

fn push_detection_point_state(report: &mut String, result: &ScenarioResult) {
    let Some(det_step) = result.first_anomaly_step else {
        return;
    };
    let Some(sample) = result.samples.iter().find(|sample| sample.step == det_step) else {
        return;
    };
    report.push_str("\n── State at Detection Point ───────────────────────────────────\n\n");
    report.push_str(&format!("  Residual: {:.4}\n", sample.residual));
    report.push_str(&format!("  Drift:    {:.6}\n", sample.drift));
    report.push_str(&format!("  Slew:     {:.6}\n", sample.slew));
    report.push_str(&format!(
        "  Value:    {:.4} (baseline: {:.4})\n",
        sample.value, sample.baseline
    ));
}

fn push_residual_trajectory(report: &mut String, result: &ScenarioResult) {
    report.push_str("\n── Residual Trajectory (ASCII) ────────────────────────────────\n\n");
    let step_size = (result.samples.len() / 60).max(1);
    let max_r = result
        .samples
        .iter()
        .map(|sample| sample.residual.abs())
        .fold(0.0f64, f64::max)
        .max(0.001);

    for chunk in result.samples.chunks(step_size).take(60) {
        report.push_str(&render_residual_chunk(chunk, max_r));
    }
}

fn render_residual_chunk(chunk: &[crate::SampleRecord], max_r: f64) -> String {
    let avg_r: f64 = chunk.iter().map(|sample| sample.residual).sum::<f64>() / chunk.len() as f64;
    let bar_len = ((avg_r.abs() / max_r) * 40.0) as usize;
    let state_char = match chunk.last().map(|sample| sample.grammar_state) {
        Some(GrammarState::Admissible) => '·',
        Some(GrammarState::Boundary) => '▸',
        Some(GrammarState::Violation) => '█',
        None => ' ',
    };
    let bar: String = std::iter::repeat_n(state_char, bar_len).collect();
    format!("  {:>4} │{}\n", chunk[0].step, bar)
}

fn push_non_interference_contract(report: &mut String) {
    report.push_str("\n── Non-Interference Contract ──────────────────────────────────\n\n");
    report.push_str("  Contract Version: 1.0\n");
    report.push_str("  All inputs accepted as immutable references (&ResidualSample).\n");
    report.push_str("  No mutable reference to upstream system created.\n");
    report.push_str("  Observer removal produces zero behavioral change.\n");
}

fn push_report_metadata(report: &mut String) {
    report.push_str("\n── Report Metadata ────────────────────────────────────────────\n\n");
    report.push_str(&format!("  DSFB Version:          {}\n", CRATE_VERSION));
    report.push_str(&format!("  Contract Version:      {}\n", CONTRACT_VERSION));
    report.push_str("  Invariant Forge LLC — riaan@invariantforge.net\n");
}

/// Generate CSV output from scenario results.
pub fn generate_csv(result: &ScenarioResult) -> String {
    let mut csv = String::with_capacity(result.samples.len() * 80);
    csv.push_str("step,value,baseline,residual,drift,slew,grammar_state\n");
    for s in &result.samples {
        let state_str = match s.grammar_state {
            GrammarState::Admissible => "Admissible",
            GrammarState::Boundary => "Boundary",
            GrammarState::Violation => "Violation",
        };
        csv.push_str(&format!(
            "{},{:.6},{:.6},{:.6},{:.8},{:.8},{}\n",
            s.step, s.value, s.baseline, s.residual, s.drift, s.slew, state_str
        ));
    }
    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        run_scenario, AdmissibilityEnvelope, ClockDriftScenario, ObserverConfig, WorkloadPhase,
    };

    #[test]
    fn test_report_generation() {
        let mut scenario = ClockDriftScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 20,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.1,
                0.05,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let result = run_scenario(&mut scenario, &config);
        let report = generate_report(&result);
        assert!(report.contains("DSFB Gray Failure Detection Report"));
        assert!(report.contains("FAULT DETECTED"));
    }

    #[test]
    fn test_csv_generation() {
        let mut scenario = ClockDriftScenario::default_scenario();
        let config = ObserverConfig {
            persistence_window: 20,
            hysteresis_count: 3,
            default_envelope: AdmissibilityEnvelope::symmetric(
                2.0,
                0.1,
                0.05,
                WorkloadPhase::SteadyState,
            ),
            ..ObserverConfig::fast_response()
        };
        let result = run_scenario(&mut scenario, &config);
        let csv = generate_csv(&result);
        assert!(csv.starts_with("step,value,baseline,residual,drift,slew,grammar_state\n"));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 201); // header + 200 data rows
    }
}
