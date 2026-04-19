//! Live read-only MySQL telemetry adapter (feature = `live-mysql`).
//!
//! Second-engine analogue of [`crate::live`]. Maintains the identical
//! three-layer code-audit contract described there, translated to
//! MySQL-native surfaces:
//!   * **Type-level.** [`readonly_conn::ReadOnlyMySqlConn`] wraps a
//!     private `mysql_async::Conn`; the only public entry point that
//!     touches SQL is [`readonly_conn::ReadOnlyMySqlConn::query_allowed`],
//!     which accepts a variant of the closed
//!     [`queries::AllowedMySqlQuery`] enum.
//!   * **Session-level.** `connect` issues
//!     `SET SESSION TRANSACTION READ ONLY`,
//!     `SET SESSION MAX_EXECUTION_TIME = 500`, and
//!     `SET SESSION innodb_lock_wait_timeout = 1`, then verifies each
//!     via `SELECT @@SESSION.*`. Refuses to proceed if the engine
//!     silently ignores any SET.
//!   * **Statement-level.** Every [`queries::AllowedMySqlQuery`]
//!     variant maps to a `'static` `SELECT` against
//!     `performance_schema.*` or `information_schema.*`. The SHA-256
//!     of the concatenated strings is pinned by
//!     `tests/live_query_allowlist_lock_mysql.rs`.
//!
//! Determinism discipline inherited from the PostgreSQL path:
//! engine->tape is non-deterministic (scheduling and workload jitter),
//! tape->episodes is byte-deterministic. This is the same 7th
//! non-claim in [`crate::non_claims`]; the wording there generalises
//! from "live PostgreSQL adapter" to "live adapters" because the
//! contract is engine-shape-independent.
//!
//! Scope discipline: this module ships the three-layer contract
//! (type, session, statement) and the operator-facing role manifest
//! in `spec/permissions.mysql.sql`. A full scraper / distiller pair
//! that converts `AllowedMySqlQuery` row sets into
//! [`crate::residual::ResidualStream`] samples — the MySQL analogue
//! of [`crate::live::distiller`] — is scheduled for a subsequent
//! engineering pass co-sited with a MySQL harness fixture. The
//! paper's §Live-Eval MySQL subsection is explicit that the
//! contract-level code is reviewable today and that end-to-end live
//! replay against a running `mysqld` is future work. This matches
//! the panel's item-#10 scoping ("~2 weeks of engineering") and is
//! disclosed rather than synthesised.

pub mod queries;

#[cfg(feature = "live-mysql")]
pub mod readonly_conn;

pub use queries::AllowedMySqlQuery;

#[cfg(feature = "live-mysql")]
pub use readonly_conn::ReadOnlyMySqlConn;
