#![no_main]
//! Fuzz target: grammar-FSM transitions under arbitrary sign-tuple
//! sequences.
//!
//! Exercises the core grammar state machine directly, bypassing the
//! engine's envelope calibration, to stress the 2-confirmation
//! hysteresis and boundary-grazing history paths.
//!
//! Run:
//!
//! ```bash
//! cd crates/dsfb-robotics/fuzz
//! cargo +nightly fuzz run grammar_fsm -- -runs=1000000
//! ```

use libfuzzer_sys::fuzz_target;

use dsfb_robotics::envelope::AdmissibilityEnvelope;
use dsfb_robotics::grammar::{GrammarEvaluator, GrammarState};
use dsfb_robotics::platform::RobotContext;
use dsfb_robotics::sign::SignTuple;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    // First 8 bytes parameterise the envelope radius.
    let mut rho_bytes = [0u8; 8];
    rho_bytes.copy_from_slice(&data[..8]);
    let rho = f64::from_le_bytes(rho_bytes).abs().min(1e6);
    if !rho.is_finite() {
        return;
    }
    let env = AdmissibilityEnvelope::new(rho);

    // Remaining bytes form 24-byte (norm, drift, slew) triples.
    let mut eval = GrammarEvaluator::<4>::new();
    let mut chunk = &data[8..];
    while chunk.len() >= 24 {
        let n_bytes: [u8; 8] = chunk[0..8].try_into().unwrap();
        let d_bytes: [u8; 8] = chunk[8..16].try_into().unwrap();
        let s_bytes: [u8; 8] = chunk[16..24].try_into().unwrap();
        let norm = f64::from_le_bytes(n_bytes);
        let drift = f64::from_le_bytes(d_bytes);
        let slew = f64::from_le_bytes(s_bytes);
        if !norm.is_finite() || !drift.is_finite() || !slew.is_finite() {
            chunk = &chunk[24..];
            continue;
        }
        let sig = SignTuple::new(norm, drift, slew);
        let state = eval.evaluate(&sig, &env, RobotContext::ArmOperating);
        // FSM totality: `evaluate` must always return a defined state.
        let _ = matches!(
            state,
            GrammarState::Admissible | GrammarState::Boundary(_) | GrammarState::Violation
        );
        chunk = &chunk[24..];
    }
});
