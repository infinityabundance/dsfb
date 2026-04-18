#![no_main]

//! Fuzz target for the SQLShare text adapter.
//!
//! `SqlShareText::load` parses a text file of raw SQL queries separated
//! by a 40-underscore divider (see `src/adapters/sqlshare_text.rs`).
//! The skeletoniser toggles string state on every `'` or `"`; the
//! bucketer walks ordinal positions. Malformed or adversarial input
//! (unbalanced quotes, gigabyte strings, Unicode edge cases, huge
//! numbers of consecutive dividers) must never panic, overflow, or
//! hang the adapter.

use dsfb_database::adapters::sqlshare_text::SqlShareText;
use dsfb_database::adapters::DatasetAdapter;
use libfuzzer_sys::fuzz_target;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    let Ok(mut tmp) = tempfile::NamedTempFile::new() else {
        return;
    };
    if tmp.write_all(data).is_err() {
        return;
    }
    if tmp.flush().is_err() {
        return;
    }
    let adapter = SqlShareText;
    let _ = adapter.load(tmp.path());
});
