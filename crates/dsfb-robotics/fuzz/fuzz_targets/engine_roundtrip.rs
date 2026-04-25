#![no_main]
//! Fuzz target: end-to-end `observe()` roundtrip.
//!
//! Takes arbitrary input bytes, interprets them as a residual f64
//! stream, and runs the DSFB engine. The target asserts no panic,
//! no out-of-bounds output, and structurally-valid episode labels.
//!
//! Run:
//!
//! ```bash
//! cd crates/dsfb-robotics/fuzz
//! cargo +nightly fuzz run engine_roundtrip -- -runs=1000000
//! ```

use libfuzzer_sys::fuzz_target;

use dsfb_robotics::{observe, Episode};

fn bytes_to_f64_stream(data: &[u8]) -> Vec<f64> {
    // Interpret 8-byte chunks as little-endian f64s. Truncate any
    // trailing fragment.
    data.chunks_exact(8)
        .map(|c| {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(c);
            f64::from_le_bytes(buf)
        })
        .collect()
}

fuzz_target!(|data: &[u8]| {
    let residuals = bytes_to_f64_stream(data);
    // Size the output buffer to the input; the invariant is that the
    // observer never writes past it regardless of input values.
    let mut out = vec![Episode::empty(); residuals.len()];
    let n = observe(&residuals, &mut out);
    assert!(n <= out.len());
    for e in &out[..n] {
        assert!(matches!(e.grammar, "Admissible" | "Boundary" | "Violation"));
        assert!(matches!(e.decision, "Silent" | "Review" | "Escalate"));
    }
});
