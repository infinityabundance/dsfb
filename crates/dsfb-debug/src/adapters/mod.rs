//! DSFB-Debug: std-only adapters for real-world dataset ingestion.
//!
//! These modules are gated behind `#[cfg(feature = "std")]` and are
//! absent from the no_std core build. They form the only bridge
//! between the file system / fixture bytes and the no_std engine.
//!
//! # Modules
//!
//! - **`sha256`** — hand-rolled SHA-256 (FIPS 180-4 §6.2) used to
//!   verify vendored fixture bytes against `MANIFEST.toml` entries.
//!   Zero-dep policy preserved (no `sha2`/`ring`/`openssl` runtime
//!   dependency). NIST FIPS 180-4 test vectors validate the
//!   implementation.
//! - **`residual_projection`** — parser for the lightweight TSV
//!   "residual projection v1/v2" fixture format. The format is a
//!   deterministic projection of upstream Jaeger / OTLP spans (or
//!   any analogous time-series source) onto a per-window, per-signal
//!   residual matrix. Provenance (upstream DOI, archive SHA-256,
//!   extraction recipe, license) lives in the fixture header and is
//!   cross-verified against `MANIFEST.toml`.
//!
//! # Adapter contract
//!
//! The adapter is deliberately narrow: the no_std engine consumes a
//! flat `&[f64]` and is byte-stable regardless of which fixture is
//! loaded. The adapter's only job is to produce that `&[f64]` plus
//! the per-signal channel-name labels (v2 only) and the per-window
//! fault-label mask. No allocation, no parsing logic, leaks into the
//! engine.

#![cfg(feature = "std")]

pub mod sha256;
pub mod residual_projection;

pub use residual_projection::{parse_residual_projection, OwnedResidualMatrix};
