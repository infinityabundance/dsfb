#![forbid(unsafe_code)]

//! Basic DSFB pipeline example — demonstrates core engine usage.
//!
//! This example shows how to use the DSFB core engine types
//! without relying on the crate's std-gated evaluation modules.
//! The example binary itself uses `std` for console output.

use dsfb_turbine::core::config::DsfbConfig;
use dsfb_turbine::core::residual::{compute_baseline, compute_residuals, compute_drift, compute_slew, sign_at};
use dsfb_turbine::core::envelope::AdmissibilityEnvelope;
use dsfb_turbine::core::grammar::{GrammarEngine, GrammarState};
use dsfb_turbine::core::heuristics::HeuristicsBank;
use dsfb_turbine::core::regime::OperatingRegime;

fn main() {
    println!("DSFB Gas Turbine — Basic Pipeline Example");
    println!("All core types: no_std, no_alloc, no_unsafe");
    println!();

    // Simulated HPC outlet temperature (T30) for 100 cycles
    // Healthy for 30 cycles, then gradual degradation
    let mut values = [0.0f64; 100];
    for i in 0..100 {
        let healthy = 1580.0;
        let degradation = if i > 30 { (i - 30) as f64 * 0.2 } else { 0.0 };
        let noise = ((i as f64 * 0.7).sin()) * 0.3;
        values[i] = healthy + degradation + noise;
    }

    let config = DsfbConfig::cmapss_fd001_default();
    
    // Step 1: Baseline from healthy window
    let (mean, std) = compute_baseline(&values, &config);
    println!("Baseline: mean={mean:.2}, std={std:.4}");

    // Step 2: Compute residuals
    let mut residuals = [0.0f64; 100];
    compute_residuals(&values, mean, &mut residuals);

    // Step 3: Compute drift and slew
    let mut drift = [0.0f64; 100];
    let mut slew = [0.0f64; 100];
    compute_drift(&residuals, config.drift_window, &mut drift);
    compute_slew(&drift, config.slew_window, &mut slew);

    // Step 4: Construct envelope
    let envelope = AdmissibilityEnvelope::from_baseline(
        mean, std, OperatingRegime::SeaLevelStatic, &config,
    );
    println!("Envelope: [{:.2}, {:.2}]", envelope.lower, envelope.upper);

    // Step 5: Run grammar engine
    let mut grammar = GrammarEngine::new();
    let bank = HeuristicsBank::default_gas_turbine();

    println!();
    println!("Cycle  Residual   Drift     Slew      State       Reason");
    println!("─────  ────────   ─────     ────      ─────       ──────");

    for k in 0..100 {
        let sign = sign_at(&residuals, &drift, &slew, k, 1);
        grammar.advance(&sign, &envelope, &config);

        let env_stressed = envelope.classify_position(sign.residual)
            != dsfb_turbine::core::envelope::EnvelopeStatus::Interior;
        let reason = bank.match_motif(sign.drift, sign.slew, grammar.state(), env_stressed);

        // Print every 10th cycle and all transition points
        if k % 10 == 0 || grammar.state() != GrammarState::Admissible {
            println!("{:5}  {:8.4}  {:8.5}  {:8.5}  {:11}  {}",
                k + 1, sign.residual, sign.drift, sign.slew,
                grammar.state().label(), reason.label());
        }
    }

    println!();
    if let Some(fb) = grammar.first_boundary_cycle() {
        println!("First Boundary: cycle {fb}");
    }
    if let Some(fv) = grammar.first_violation_cycle() {
        println!("First Violation: cycle {fv}");
    }

    println!();
    println!("Non-interference: all inputs were &[f64] immutable slices.");
    println!("No upstream system was modified.");
}
