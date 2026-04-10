//! Detailed trace-chain walkthrough for a single engine.
//!
//! Produces a complete audit trace from raw sensor value →
//! residual → drift → slew → envelope position → grammar state →
//! reason code for every cycle of one engine. This is the
//! "show don't tell" artifact for the paper.

use crate::pipeline::engine_eval::EngineEvalResult;
use crate::core::grammar::GrammarState;
use std::fmt::Write;

/// Generates a complete trace-chain walkthrough for one engine.
///
/// This is the paper's Section 10.7 equivalent: a fully resolved
/// trace showing every step of the DSFB interpretive pipeline
/// from raw residual to typed conclusion.
pub fn trace_chain_report(result: &EngineEvalResult) -> String {
    let mut out = String::with_capacity(16384);

    let _ = writeln!(out, "══════════════════════════════════════════════════════════════════════");
    let _ = writeln!(out, "  DSFB Trace-Chain Walkthrough — Engine Unit {}", result.unit);
    let _ = writeln!(out, "  Total lifetime: {} cycles", result.total_cycles);
    let _ = writeln!(out, "══════════════════════════════════════════════════════════════════════");
    let _ = writeln!(out);

    // Summary
    let _ = writeln!(out, "── Summary ────────────────────────────────────────────────────────");
    if let Some(fb) = result.first_boundary_cycle {
        let rul_at_fb = result.total_cycles.saturating_sub(fb);
        let _ = writeln!(out, "  First Boundary:    cycle {} (RUL = {} cycles remaining)", fb, rul_at_fb);
    } else {
        let _ = writeln!(out, "  First Boundary:    never reached");
    }
    if let Some(fv) = result.first_violation_cycle {
        let rul_at_fv = result.total_cycles.saturating_sub(fv);
        let _ = writeln!(out, "  First Violation:   cycle {} (RUL = {} cycles remaining)", fv, rul_at_fv);
    } else {
        let _ = writeln!(out, "  First Violation:   never reached");
    }
    if let Some(lt) = result.structural_lead_time {
        let _ = writeln!(out, "  Structural lead:   {} cycles before end-of-life", lt);
    }
    let _ = writeln!(out, "  Total episodes:    {}", result.episodes.len());
    let _ = writeln!(out);

    // Theorem 1 bound analysis
    if let Some(ref tb) = result.theorem_bound {
        let _ = writeln!(out, "── Theorem 1 Bound Analysis ───────────────────────────────────");
        let _ = writeln!(out, "  Initial admissibility gap: {:.4}", tb.initial_gap);
        let _ = writeln!(out, "  Observed drift rate (eta): {:.6}", tb.drift_rate);
        let _ = writeln!(out, "  Envelope expansion (kappa): {:.6}", tb.envelope_expansion_rate);
        let _ = writeln!(out, "  Computed exit bound:       {} cycles from drift onset", tb.exit_bound_cycles);
        if let Some(onset) = tb.drift_onset_cycle {
            let _ = writeln!(out, "  Drift onset cycle:         {}", onset);
        }
        if let Some(obs) = tb.observed_transition_cycle {
            let _ = writeln!(out, "  Observed transition:       cycle {}", obs);
        }
        let _ = writeln!(out, "  Bound satisfied:           {}", if tb.bound_satisfied { "YES" } else { "NO" });
        if !tb.bound_satisfied {
            let _ = writeln!(out, "  Note: Boundary triggered by envelope-approach before");
            let _ = writeln!(out, "        drift-persistence threshold was reached. This is");
            let _ = writeln!(out, "        a known discrepancy (see paper Section 6.2).");
        }
        let _ = writeln!(out);
    }

    // Per-channel first-boundary cycles
    if !result.channel_boundary_cycles.is_empty() {
        let _ = writeln!(out, "── Per-Channel First Boundary Cycles ──────────────────────────");
        for (ch, fb) in &result.channel_boundary_cycles {
            let _ = writeln!(out, "  {:35} cycle {}",
                ch.label(),
                fb.map_or("never".to_string(), |c| c.to_string()));
        }
        let _ = writeln!(out);
    }

    // Episodes
    if !result.episodes.is_empty() {
        let _ = writeln!(out, "── Episodes ───────────────────────────────────────────────────");
        for (i, ep) in result.episodes.iter().enumerate() {
            let _ = writeln!(out, "  Episode {}: cycles {}-{} ({} cycles)",
                i + 1, ep.start_cycle, ep.end_cycle, ep.duration_cycles);
            let _ = writeln!(out, "    Peak state:  {}", ep.peak_state.label());
            let _ = writeln!(out, "    Reason code: {}", ep.reason_code.label());
            let _ = writeln!(out, "    Max drift:   {:.6}", ep.max_drift);
            let _ = writeln!(out, "    Max slew:    {:.6}", ep.max_slew);
        }
        let _ = writeln!(out);
    }

    // Full audit trace (every cycle)
    let _ = writeln!(out, "── Full Audit Trace (Primary Channel) ─────────────────────────");
    let _ = writeln!(out, "  {:>5} {:>10} {:>10} {:>10} {:>8} {:>10} {:>11} {}",
        "Cycle", "Residual", "Drift", "Slew", "EnvPos", "EnvStatus", "Grammar", "ReasonCode");
    let _ = writeln!(out, "  {}", "─".repeat(90));

    let entries = result.primary_audit.entries();
    let mut prev_state = GrammarState::Admissible;

    for entry in entries {
        let env_status_str = match entry.envelope_status {
            crate::core::envelope::EnvelopeStatus::Interior => "Interior",
            crate::core::envelope::EnvelopeStatus::Approaching => "Approach",
            crate::core::envelope::EnvelopeStatus::Exceeded => "EXCEEDED",
        };

        // Mark transitions with >>>
        let transition_marker = if entry.grammar_state != prev_state {
            " >>>"
        } else {
            ""
        };

        let _ = writeln!(out, "  {:5} {:10.4} {:10.6} {:10.6} {:8.3} {:>10} {:>11} {}{}",
            entry.cycle,
            entry.residual,
            entry.drift,
            entry.slew,
            entry.envelope_position,
            env_status_str,
            entry.grammar_state.label(),
            entry.reason_code.label(),
            transition_marker,
        );

        prev_state = entry.grammar_state;
    }

    let _ = writeln!(out);
    let _ = writeln!(out, "── Interpretive Summary ────────────────────────────────────────");
    let _ = writeln!(out, "  This trace shows the complete DSFB interpretive pipeline:");
    let _ = writeln!(out, "    raw sensor → residual (deviation from baseline) →");
    let _ = writeln!(out, "    drift (first difference) → slew (second difference) →");
    let _ = writeln!(out, "    envelope position → grammar state → reason code.");
    let _ = writeln!(out, "  Every value is deterministic and reproducible.");
    let _ = writeln!(out, "  Transitions marked with >>> indicate grammar state changes.");
    let _ = writeln!(out);
    let _ = writeln!(out, "  DSFB does not predict RUL. It classifies the structural");
    let _ = writeln!(out, "  state of the degradation trajectory at each cycle.");
    let _ = writeln!(out, "  The information it provides — typed reason codes, drift");
    let _ = writeln!(out, "  direction, slew acceleration — is structure that existing");
    let _ = writeln!(out, "  EHM trending methods compute but do not formalize.");

    out
}
