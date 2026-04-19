//! Type-level read-only PostgreSQL connection wrapper.
//!
//! [`ReadOnlyPgConn`] owns a `tokio_postgres::Client` behind a private
//! field. The only public call that executes SQL is
//! [`ReadOnlyPgConn::query_allowed`], and it takes a
//! [`crate::live::queries::AllowedQuery`] enum variant — there is no way
//! to smuggle a raw SQL string through this type. `execute`, `prepare`,
//! `transaction`, `copy_in`, `copy_out`, `batch_execute`, and
//! `simple_query` are **not** re-exported.
//!
//! On top of the type-level contract, the constructor installs a
//! session-level read-only guard by issuing `SET
//! default_transaction_read_only = on;` and then verifies via a
//! `SELECT current_setting(...)` that the setting took effect. If the
//! engine silently ignores the SET (it should not, but a forked engine
//! might), we refuse to proceed rather than operate on an unguarded
//! session.
//!
//! Every error path is *explicit*. A malformed connection string, a
//! connection-level failure, a refusal of the read-only SET, and a
//! verification mismatch each produce a distinct message so that an
//! operator reading the log can tell which control failed.

use super::queries::AllowedQuery;
use anyhow::{anyhow, Context, Result};

/// Read-only PostgreSQL client wrapper.
///
/// The wrapped `tokio_postgres::Client` is private by construction.
/// Downstream code can only interact with the database via
/// [`Self::query_allowed`]. This is the type-level layer of the
/// code-audit contract documented in [`crate::live`].
pub struct ReadOnlyPgConn {
    client: tokio_postgres::Client,
}

impl ReadOnlyPgConn {
    /// Open a connection and install the session-level read-only guard.
    ///
    /// Spawns the tokio-postgres background connection driver on the
    /// current tokio runtime; if no runtime is active, spawning will
    /// panic — but the `live` subcommand always constructs a
    /// current-thread runtime before calling this, so that is a
    /// caller-side invariant documented in [`crate::live`].
    pub async fn connect(conn_str: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(conn_str, tokio_postgres::NoTls)
            .await
            .with_context(|| "tokio_postgres::connect failed (check conn string + reachability)")?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("tokio-postgres connection driver exited: {e}");
            }
        });
        client
            .execute("SET default_transaction_read_only = on", &[])
            .await
            .with_context(|| "failed to set default_transaction_read_only = on")?;
        let row = client
            .query_one(
                "SELECT current_setting('default_transaction_read_only')",
                &[],
            )
            .await
            .with_context(|| "failed to verify default_transaction_read_only")?;
        let setting: &str = row.get(0);
        if setting != "on" {
            return Err(anyhow!(
                "refusing to proceed: default_transaction_read_only = {:?}, expected \"on\"",
                setting
            ));
        }
        Ok(Self { client })
    }

    /// Execute an allow-listed query. No raw SQL surface is exposed:
    /// the caller chooses a variant of the closed [`AllowedQuery`]
    /// enum, the wrapper looks up the pinned SQL text, and returns
    /// the resulting rows.
    pub async fn query_allowed(&self, q: AllowedQuery) -> Result<Vec<tokio_postgres::Row>> {
        let sql = q.sql();
        self.client
            .query(sql, &[])
            .await
            .with_context(|| format!("query_allowed failed for {:?}", q))
    }
}

// NOTE: No `Deref<Target = tokio_postgres::Client>` impl. No
// `AsRef<tokio_postgres::Client>` impl. No getter. Those would each
// be a hole in the code-audit contract — the client is deliberately
// inaccessible outside this module.
