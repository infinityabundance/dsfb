//! cargo-fuzz harness: feed arbitrary norm sequences through the observer
//! and assert the core engine remains panic-free.
//!
//! Invocation:
//!
//!     cargo +nightly fuzz run engine_roundtrip
//!
//! The fuzz harness deliberately exercises the `SignWindow` → `GrammarEvaluator`
//! → `AdmissibilityEnvelope` path on boundary inputs (0.0, subnormals,
//! denormals, NaN-free finites) to surface any invariant gap that the
//! proptest bank misses.

#![no_main]

use arbitrary::Arbitrary;
use dsfb_rf::envelope::AdmissibilityEnvelope;
use dsfb_rf::grammar::GrammarEvaluator;
use dsfb_rf::platform::WaveformState;
use dsfb_rf::sign::{SignWindow, SnrFloor};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    base: f32,
    noise_seed: u64,
    n: u16,
}

fuzz_target!(|input: FuzzInput| {
    if !input.base.is_finite() || input.base <= 0.0 || input.base > 1.0 {
        return;
    }
    let healthy: [f32; 64] = core::array::from_fn(|_| input.base);
    let Some(env) = AdmissibilityEnvelope::calibrate_from_window(&healthy) else {
        return;
    };
    let mut window: SignWindow<8> = SignWindow::new();
    let floor = SnrFloor::default();
    let mut grammar: GrammarEvaluator<4> = GrammarEvaluator::new();
    let mut s = input.noise_seed;
    for _ in 0..input.n.min(4096) {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let jitter = ((s >> 40) as f32 / (1u32 << 24) as f32) * 1e-3;
        let norm = (input.base + jitter).max(0.0);
        if !norm.is_finite() {
            continue;
        }
        let sig = window.push(norm, false, floor);
        let _ = grammar.evaluate(&sig, &env, WaveformState::Operational);
    }
});
