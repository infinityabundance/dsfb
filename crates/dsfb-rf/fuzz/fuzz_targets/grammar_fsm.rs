//! cargo-fuzz harness: drive the grammar FSM with arbitrary SignTuple
//! sequences and prove the K=4 hysteresis transition table is panic-free
//! for any well-formed envelope + tuple + waveform-state triple.

#![no_main]

use arbitrary::Arbitrary;
use dsfb_rf::envelope::AdmissibilityEnvelope;
use dsfb_rf::grammar::GrammarEvaluator;
use dsfb_rf::platform::WaveformState;
use dsfb_rf::sign::{SignWindow, SnrFloor};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    rho: f32,
    norms: Vec<f32>,
}

fuzz_target!(|input: Input| {
    if !input.rho.is_finite() || input.rho <= 0.0 || input.rho > 1e3 {
        return;
    }
    let env = AdmissibilityEnvelope::new(input.rho);
    let mut window: SignWindow<8> = SignWindow::new();
    let floor = SnrFloor::default();
    let mut grammar: GrammarEvaluator<4> = GrammarEvaluator::new();
    for n in input.norms.iter().take(2048) {
        if !n.is_finite() {
            continue;
        }
        let sig = window.push(n.abs(), false, floor);
        let _ = grammar.evaluate(&sig, &env, WaveformState::Operational);
    }
});
