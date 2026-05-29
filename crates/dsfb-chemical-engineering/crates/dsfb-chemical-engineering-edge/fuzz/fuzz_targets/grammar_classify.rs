//! Fuzz target: the float-path DSFB grammar classifier (`evaluate` → `GrammarClassifier::classify`).
//!
//! This is the EMPIRICAL companion to the Kani harness `proof_classify_is_total_on_finite`
//! (`../../src/kani_proofs.rs`). Kani proves totality over a *bounded symbolic* window
//! (`|r,δ,σ| < 1e6`, finite); this target hammers the SAME two functions over the **full** IEEE-754
//! domain — arbitrary fuzzer bytes are reinterpreted as `f64`, so NaN, ±∞, and subnormals all occur.
//! The invariant under test is the same one Kani states: `classify` must never panic, overflow, or
//! diverge — for any input it terminates and returns a `(GrammarState, ReasonCode)`. Non-finite axes
//! are expected to route to `SensorFault`, never to a trap.
//!
//! Run: `cargo +nightly fuzz run grammar_classify -- -max_total_time=60`

#![no_main]

use libfuzzer_sys::fuzz_target;

use dsfb_chemical_engineering_edge::dsfb_core::{
    evaluate, AdmissibilityEnvelope, GrammarClassifier, ResidualTriple,
};

/// Read 8 bytes at `off` as a little-endian `f64` (0.0 if the slice is too short).
#[inline]
fn f64_at(data: &[u8], off: usize) -> f64 {
    data.get(off..off + 8)
        .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
        .unwrap_or(0.0)
}

fuzz_target!(|data: &[u8]| {
    // The triple (r, δ, σ) is the untrusted input; bytes are reinterpreted raw so the fuzzer can reach
    // NaN/±∞/subnormal corners the bounded Kani proof deliberately excludes.
    let r = f64_at(data, 0);
    let delta = f64_at(data, 8);
    let sigma = f64_at(data, 16);

    // Derive a non-degenerate envelope from further bytes so we exercise many (bound, band) geometries —
    // `symmetric` is the same constructor the Kani harnesses use. Keep r_max finite-positive and the band
    // in a sane fraction so the envelope itself is well-formed; the *triple* is where the fuzzing happens.
    let raw_max = f64_at(data, 24).abs();
    let r_max = if raw_max.is_finite() && raw_max > 0.0 { raw_max.min(1e9) } else { 1.0 };
    let band = (data.get(32).copied().unwrap_or(26) as f64) / 255.0 * 0.5 + 1e-6;
    let env = AdmissibilityEnvelope::symmetric(r_max, band);

    let mut g = GrammarClassifier::new();
    let t = ResidualTriple { r, delta, sigma, timestamp: 0.0 };
    let ev = evaluate(&env, &t);
    // The whole point: this must return for every input, never panic.
    let _ = g.classify(&ev, &t);
});
