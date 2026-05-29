//! Fuzz target: the engineering-unit string parser (`unit_consistency::parse_unit`).
//!
//! `parse_unit` is a pure `&str -> Option<Unit>` lexer that normalises (lower-cases, strips `^`/`°`) and
//! matches against a fixed unit table. It is on the data-ingestion boundary (unit strings come from
//! historian column headers / roles sidecars — i.e. untrusted text). The invariant: for ANY input string
//! — arbitrary UTF-8, multibyte glyphs, degenerate prefixes — it returns `Some`/`None` without panicking
//! (no slicing on non-char-boundaries, no overflow).
//!
//! Run: `cargo +nightly fuzz run parse_unit -- -max_total_time=60`

#![no_main]

use libfuzzer_sys::fuzz_target;

use dsfb_chemical_engineering_edge::unit_consistency::parse_unit;

fuzz_target!(|data: &[u8]| {
    // Only well-formed UTF-8 reaches `parse_unit` in production (CSV/JSON are decoded upstream), so we
    // mirror that: feed the bytes as a string when valid. `from_utf8` rejects invalid sequences, which is
    // the real contract; the fuzzer still explores every valid-UTF-8 corner (combining marks, the `°`
    // degree sign the parser strips, mixed case, empty/whitespace).
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_unit(s);
    }
});
