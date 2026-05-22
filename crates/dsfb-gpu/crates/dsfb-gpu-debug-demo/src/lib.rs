//! Library target for `dsfb-gpu-debug-demo`.
//!
//! WHY THIS LIB EXISTS (for the future engineer reading cold):
//!
//! The demo crate is primarily a bin (`dsfb-gpu-debug` at
//! `src/main.rs`), but the `cli::ingest` module — which parses the
//! `# residual-projection v2` TSV format and lowers cells into
//! deterministic `TraceEvent[]` — is useful from integration
//! tests too (the S-REAL saturation bench needs to feed real TSV
//! events into the same dispatcher path S-PERF.16.a measures).
//!
//! Rust's integration-test target (under `tests/`) can only import
//! public items from a CRATE LIBRARY, not from a `[[bin]]`. So we
//! expose `cli` as a library here; `main.rs` consumes it via
//! `use dsfb_gpu_debug_demo::cli;`. The bin and the lib share the
//! same source tree under `src/`; only the entry shape differs.
//!
//! Nothing else changes — every cli subcommand's existing logic
//! continues to live in `src/cli/*.rs`, and the only consumer
//! today (besides main.rs) is `tests/s_real_saturation_bench.rs`.
//!
//! Clippy posture: the cli modules below are bin-style helpers
//! (process-exit consumers, one-shot fixture-failure unwraps in
//! their inline test scaffolding). Exposing them as a library
//! surfaces lints that are appropriate for library APIs but not
//! for bin-style code. The allows below restore the bin posture:
//!
//! - `must_use_candidate`: cli subcommand entrypoints return
//!   `ExitCode`/`u8` that the bin always consumes; the cli
//!   helpers return scalars the bin always reads. Annotating
//!   ~20 helpers with `#[must_use]` is noise without callers
//!   that would benefit from the warning.
//! - `expect_used` / `unwrap_used`: only the inline `#[cfg(test)]`
//!   scaffolding in cli modules uses them, and only on
//!   panic-on-fixture-failure paths where a loud abort is the
//!   correct test posture.

#![allow(
    clippy::must_use_candidate,
    clippy::expect_used,
    clippy::unwrap_used,
    // The cli modules are bin-style helpers; pub-item docs are
    // not required because no external library consumer reads
    // them (the only library consumer is the in-tree integration
    // test). Forcing them would either require ~100 single-line
    // doc comments on bin-style fields or a wide rustdoc-noise
    // pass; neither matches the panel-locked "lib is a delivery
    // vehicle for the integration test, not a public surface"
    // posture stated above.
    missing_docs
)]

pub mod cli;
