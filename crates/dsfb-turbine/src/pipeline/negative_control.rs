//! P4: Negative control — false-alarm analysis on known-healthy windows.
//!
//! For each engine, the first `healthy_window` cycles are used to construct
//! the baseline. By construction, these cycles are "healthy." Any DSFB
//! grammar transition within this window is a false alarm.
//!
//! This module also computes the clean-window false-episode rate:
//! across all engines, what fraction emit a Boundary or Violation state
//! during cycles 1..healthy_window?

use crate::pipeline::engine_eval::EngineEvalResult;
use crate::core::grammar::GrammarState;
use crate::core::config::DsfbConfig;
use std::fmt::Write;

/// Negative control result for the fleet.
#[derive(Debug, Clone)]
pub struct NegativeControlResult {
    /// Total engines evaluated.
    pub total_engines: usize,
    /// Engines with false Boundary in healthy window.
    pub engines_with_false_boundary: usize,
    /// Engines with false Violation in healthy window.
    pub engines_with_false_violation: usize,
    /// False Boundary rate (fraction of engines).
    pub false_boundary_rate: f64,
    /// False Violation rate (fraction of engines).
    pub false_violation_rate: f64,
    /// Total cycles in healthy windows across all engines.
    pub total_healthy_cycles: usize,
    /// Total false-alarm cycles (non-Admissible in healthy window).
    pub total_false_alarm_cycles: usize,
    /// Per-cycle false-alarm rate.
    pub per_cycle_false_alarm_rate: f64,
    /// Healthy window size used.
    pub healthy_window: usize,
}

/// Computes negative control metrics from evaluation results.
pub fn compute_negative_control(
    results: &[EngineEvalResult],
    config: &DsfbConfig,
) -> NegativeControlResult {
    let n = results.len();
    if n == 0 {
        return NegativeControlResult {
            total_engines: 0,
            engines_with_false_boundary: 0,
            engines_with_false_violation: 0,
            false_boundary_rate: 0.0,
            false_violation_rate: 0.0,
            total_healthy_cycles: 0,
            total_false_alarm_cycles: 0,
            per_cycle_false_alarm_rate: 0.0,
            healthy_window: config.healthy_window,
        };
    }

    let hw = config.healthy_window;
    let mut false_boundary = 0usize;
    let mut false_violation = 0usize;
    let mut total_healthy_cycles = 0usize;
    let mut total_false_cycles = 0usize;

    for result in results {
        let check_len = result.grammar_trajectory.len().min(hw);
        total_healthy_cycles += check_len;

        let mut has_false_boundary = false;
        let mut has_false_violation = false;

        for k in 0..check_len {
            match result.grammar_trajectory[k] {
                GrammarState::Boundary => {
                    has_false_boundary = true;
                    total_false_cycles += 1;
                }
                GrammarState::Violation => {
                    has_false_violation = true;
                    total_false_cycles += 1;
                }
                GrammarState::Admissible => {}
            }
        }

        if has_false_boundary { false_boundary += 1; }
        if has_false_violation { false_violation += 1; }
    }

    NegativeControlResult {
        total_engines: n,
        engines_with_false_boundary: false_boundary,
        engines_with_false_violation: false_violation,
        false_boundary_rate: false_boundary as f64 / n as f64,
        false_violation_rate: false_violation as f64 / n as f64,
        total_healthy_cycles,
        total_false_alarm_cycles: total_false_cycles,
        per_cycle_false_alarm_rate: if total_healthy_cycles > 0 {
            total_false_cycles as f64 / total_healthy_cycles as f64
        } else { 0.0 },
        healthy_window: hw,
    }
}

/// Formats the negative control result as text.
pub fn negative_control_report(nc: &NegativeControlResult) -> String {
    let mut out = String::with_capacity(1024);
    let _ = writeln!(out, "── P4: Negative Control (Clean-Window False-Alarm Analysis) ────");
    let _ = writeln!(out, "  Healthy window size:         {} cycles", nc.healthy_window);
    let _ = writeln!(out, "  Total engines:               {}", nc.total_engines);
    let _ = writeln!(out, "  Engines with false Boundary: {} ({:.1}%)",
        nc.engines_with_false_boundary, nc.false_boundary_rate * 100.0);
    let _ = writeln!(out, "  Engines with false Violation: {} ({:.1}%)",
        nc.engines_with_false_violation, nc.false_violation_rate * 100.0);
    let _ = writeln!(out, "  Total healthy-window cycles:  {}", nc.total_healthy_cycles);
    let _ = writeln!(out, "  False-alarm cycles:          {} ({:.3}%)",
        nc.total_false_alarm_cycles, nc.per_cycle_false_alarm_rate * 100.0);
    let _ = writeln!(out);
    if nc.false_boundary_rate == 0.0 && nc.false_violation_rate == 0.0 {
        let _ = writeln!(out, "  RESULT: Zero false alarms in healthy window. ✓");
    } else {
        let _ = writeln!(out, "  RESULT: False alarms detected. Envelope calibration may need adjustment.");
        let _ = writeln!(out, "  Note: This does not indicate a DSFB failure. It indicates that the");
        let _ = writeln!(out, "  envelope_sigma or persistence_threshold is too sensitive for the");
        let _ = writeln!(out, "  noise characteristics of this dataset at this configuration.");
    }
    out
}
