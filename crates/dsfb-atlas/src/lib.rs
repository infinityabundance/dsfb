//! `dsfb-atlas` library surface.
//!
//! The crate is primarily a binary, but the `dedup`, `schema`,
//! `generator`, `bib_emit`, and `index_emit` modules are exported as a
//! library so that external test harnesses (and the cargo-fuzz target
//! under `audit/fuzz/`) can consume them directly.
//!
//! See `audit/AUDIT.md` for the safety audit posture, including the
//! Kani proof harness in `dedup` and the cargo-fuzz target for the YAML
//! parser.

#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_qualifications,
    clippy::all,
    clippy::pedantic
)]
#![allow(
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
#![forbid(unsafe_code)]

pub mod bib_emit;
pub mod dedup;
pub mod generator;
pub mod index_emit;
pub mod schema;
