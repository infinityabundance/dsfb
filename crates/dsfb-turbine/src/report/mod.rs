//! Report generation — structured text output for evaluation results.
//!
//! This module is std-gated and alloc-using. It is not part of the
//! crate's embedded `no_std` / `no_alloc` core surface.

use crate::pipeline::engine_eval::EngineEvalResult;
use crate::pipeline::metrics::FleetMetrics;
use std::fmt::Write;

/// Generates a full-text evaluation report.
pub fn generate_report(
    results: &[EngineEvalResult],
    metrics: &FleetMetrics,
    dataset_name: &str,
) -> String {
    let mut report = String::with_capacity(16384);

    let _ = writeln!(report, "═══════════════════════════════════════════════════════════════");
    let _ = writeln!(report, "  DSFB Structural Semiotics Engine — Gas Turbine Evaluation");
    let _ = writeln!(report, "  Dataset: {dataset_name}");
    let _ = writeln!(report, "  Crate version: {}", crate::VERSION);
    let _ = writeln!(report, "  Non-interference contract: {}", crate::NON_INTERFERENCE_CONTRACT);
    let _ = writeln!(report, "═══════════════════════════════════════════════════════════════");
    let _ = writeln!(report);

    // Fleet summary
    let _ = writeln!(report, "── Fleet Summary ──────────────────────────────────────────────");
    let _ = writeln!(report, "  Engines evaluated:           {}", metrics.total_engines);
    let _ = writeln!(report, "  Engines with Boundary:       {} ({:.1}%)", metrics.engines_with_boundary,
        100.0 * metrics.engines_with_boundary as f64 / metrics.total_engines.max(1) as f64);
    let _ = writeln!(report, "  Engines with Violation:       {} ({:.1}%)", metrics.engines_with_violation,
        100.0 * metrics.engines_with_violation as f64 / metrics.total_engines.max(1) as f64);
    let _ = writeln!(report, "  Mean structural lead time:   {:.1} cycles", metrics.mean_lead_time);
    let _ = writeln!(report, "  Median structural lead time: {:.1} cycles", metrics.median_lead_time);
    let _ = writeln!(report, "  Min / Max lead time:         {} / {} cycles", metrics.min_lead_time, metrics.max_lead_time);
    let _ = writeln!(report, "  Total episodes:              {}", metrics.total_episodes);
    let _ = writeln!(report, "  Theorem 1 satisfaction:      {:.1}%", 100.0 * metrics.theorem_satisfaction_rate);
    let _ = writeln!(report, "  Early warning (>30 RUL):     {} ({:.1}%)", metrics.early_warning_count,
        100.0 * metrics.early_warning_count as f64 / metrics.total_engines.max(1) as f64);
    let _ = writeln!(report);

    // Per-engine trace chain (first 5 engines)
    let _ = writeln!(report, "── Trace-Chain Walkthrough (First 5 Engines) ──────────────────");
    for result in results.iter().take(5) {
        let _ = writeln!(report, "\n  Engine Unit {}: {} cycles, {} episodes", result.unit, result.total_cycles, result.episodes.len());
        if let Some(fb) = result.first_boundary_cycle {
            let _ = writeln!(report, "    First Boundary:  cycle {fb} (RUL = {})", result.total_cycles.saturating_sub(fb));
        }
        if let Some(fv) = result.first_violation_cycle {
            let _ = writeln!(report, "    First Violation: cycle {fv} (RUL = {})", result.total_cycles.saturating_sub(fv));
        }
        if let Some(lt) = result.structural_lead_time {
            let _ = writeln!(report, "    Structural lead time: {lt} cycles");
        }
        for ep in &result.episodes {
            let _ = writeln!(report, "    Episode: cycles {}-{}, state={}, reason={}",
                ep.start_cycle, ep.end_cycle, ep.peak_state.label(), ep.reason_code.label());
        }
        if let Some(ref tb) = result.theorem_bound {
            let _ = writeln!(report, "    Theorem 1: bound={} cycles, satisfied={}", tb.exit_bound_cycles, tb.bound_satisfied);
        }
    }

    let _ = writeln!(report);
    let _ = writeln!(report, "── Non-Interference Verification ──────────────────────────────");
    let _ = writeln!(report, "  DSFB operates as a read-only observer.");
    let _ = writeln!(report, "  No upstream EHM/GPA/FADEC system was modified.");
    let _ = writeln!(report, "  All inputs consumed as immutable slices (&[f64]).");
    let _ = writeln!(report, "  Removing DSFB produces zero change in upstream behavior.");
    let _ = writeln!(report);
    let _ = writeln!(report, "── Claims (Strictly Limited) ──────────────────────────────────");
    let _ = writeln!(report, "  1. DSFB does NOT predict RUL.");
    let _ = writeln!(report, "  2. DSFB does NOT claim superiority over any incumbent method.");
    let _ = writeln!(report, "  3. DSFB does NOT modify any engine control or protection logic.");
    let _ = writeln!(report, "  4. Results are from simulated C-MAPSS data only.");
    let _ = writeln!(report, "  5. Real-engine validation requires Phase I site data.");

    report
}
