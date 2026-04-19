//! Live read-only PostgreSQL telemetry adapter (feature = `live-postgres`).
//!
//! This module is the crate's *only* live-connection code path. Every other
//! adapter reads a file and returns a [`crate::residual::ResidualStream`]
//! deterministically; here we connect to a running PostgreSQL server, poll
//! `pg_stat_*` views at a configurable cadence, convert cumulative-counter
//! deltas into residual samples on the fly, write a SHA-256-finalised
//! *tape* artefact, and re-run the batch [`crate::grammar::MotifEngine`]
//! on a growing in-memory buffer so newly-closed episodes can be emitted
//! incrementally.
//!
//! The honesty budget for this module is unusual. Every offline code path
//! in the crate enjoys seed-deterministic reproducibility: given the same
//! input CSV, every byte of the resulting stream and every byte of the
//! resulting episode list is fixed and pinned by a fingerprint test. The
//! live path cannot inherit that guarantee because the engine's response
//! time, sampling jitter, and concurrent workload shape the stream.
//! Determinism migrates to the *tape*: given a tape, the replayed episode
//! stream is byte-stable; two live invocations against the same engine
//! workload will produce different tapes. This asymmetry is explicit in
//! the 7th non-claim in [`crate::non_claims`], in §Live in the paper, and
//! is enforced by [`tape::Tape::finalize`] (which SHA-256s the tape bytes
//! and pins the hash in a sidecar manifest).
//!
//! ## Code-audit contract (read-only by construction)
//!
//! Three layered controls together constitute the "software data diode"
//! the paper refers to. They are *structural*: they do not require the
//! operator to trust this crate's runtime behaviour beyond what a static
//! code audit can verify.
//!
//! 1. **Type-level.** [`readonly_conn::ReadOnlyPgConn`] wraps a private
//!    `tokio_postgres::Client`. The only public entry point that touches
//!    SQL is [`readonly_conn::ReadOnlyPgConn::query_allowed`], which
//!    accepts a variant of the closed [`queries::AllowedQuery`] enum.
//!    `execute`, `prepare`, `transaction`, `copy_in`, `copy_out`, and
//!    `batch_execute` are not re-exported. A compile-fail test
//!    (`tests/live_readonly_conn_surface.rs`) pins this surface.
//! 2. **Session-level.** [`readonly_conn::ReadOnlyPgConn::connect`]
//!    issues `SET default_transaction_read_only = on;` on the freshly
//!    opened session and verifies via a `SELECT
//!    current_setting(...)` that the setting took effect. If the engine
//!    refuses, connection fails hard.
//! 3. **Statement-level.** Every variant of [`queries::AllowedQuery`]
//!    maps to a `'static` SQL string that is a pure `SELECT` against
//!    `pg_stat_*` / `pg_catalog.*`. The SHA-256 of the concatenated
//!    strings is pinned by `tests/live_query_allowlist_lock.rs` so that
//!    an editor cannot silently add a write.
//!
//! These three controls are layered, not redundant: the type-level
//! control pins *what code paths exist* at compile time; the
//! session-level control defends against a bug in the type-level
//! control by asking PostgreSQL to reject any mutating statement that
//! somehow reaches the wire; the statement-level control ensures the
//! only statements that can reach the wire are an auditable, pinned
//! list. The paper's Appendix F documents the minimum-privilege role
//! that the engine operator should provision; together these are a
//! *code-path contract*, not an unforgeable cryptographic proof. The
//! 7th non-claim is explicit about that distinction.

pub mod distiller;
pub mod emitter;
pub mod queries;
pub mod readonly_conn;
pub mod scraper;
pub mod tape;

pub use distiller::{DistillerState, Snapshot};
pub use emitter::LiveEmitter;
pub use queries::AllowedQuery;
pub use readonly_conn::ReadOnlyPgConn;
pub use scraper::{BackpressureState, Budget, Scraper};
pub use tape::{Tape, TapeManifest};
