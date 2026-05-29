//! Fuzz target: the `no_std` FIXED-POINT grammar path (`core::FixedEnvelope::eval` → `GrammarClassifier::classify`).
//!
//! The embedded `core` crate reimplements the grammar in scaled `i64` integers with `i128`-promoted
//! comparisons (no float, no heap, `#![forbid(unsafe_code)]`). `classify_axis` documents an overflow-safety
//! claim ("promoted to `i128` so the product never overflows"); this target backs that claim EMPIRICALLY by
//! feeding fully-arbitrary `i64` envelope bounds + triple coordinates — including `i64::MIN`/`MAX`, inverted
//! bounds (`lo > hi`), and out-of-range band fractions — and asserting the whole eval→classify path never
//! panics (no integer overflow, no divide, no out-of-range index). This is the embedded-sibling analogue of
//! the float `grammar_classify` target and of the Kani totality proof.
//!
//! Run: `cargo +nightly fuzz run core_fixedpoint_classify -- -max_total_time=60`

#![no_main]

use libfuzzer_sys::fuzz_target;

use dsfb_chemical_engineering_core::{FixedEnvelope, FixedTriple, GrammarClassifier};

/// Read 8 bytes at `off` as a little-endian `i64` (0 if the slice is too short).
#[inline]
fn i64_at(data: &[u8], off: usize) -> i64 {
    data.get(off..off + 8)
        .map(|b| i64::from_le_bytes(b.try_into().unwrap()))
        .unwrap_or(0)
}

fuzz_target!(|data: &[u8]| {
    // All seven envelope fields and the three triple coordinates are arbitrary i64 — deliberately including
    // degenerate geometries (inverted bounds, negative or > SCALE band fractions). The integer logic must
    // absorb every one of these without a panic.
    let env = FixedEnvelope {
        r_min: i64_at(data, 0),
        r_max: i64_at(data, 8),
        delta_min: i64_at(data, 16),
        delta_max: i64_at(data, 24),
        sigma_min: i64_at(data, 32),
        sigma_max: i64_at(data, 40),
        band_scaled: i64_at(data, 48),
    };
    let t = FixedTriple {
        r: i64_at(data, 56),
        delta: i64_at(data, 64),
        sigma: i64_at(data, 72),
    };
    // `valid` toggles the sensor-trust flag (the fixed-point classify's analogue of the float NaN path).
    let valid = data.first().map(|b| b & 1 == 1).unwrap_or(true);

    let ev = env.eval(&t);
    let mut g = GrammarClassifier::new();
    let _ = g.classify(&ev, &t, valid);
});
