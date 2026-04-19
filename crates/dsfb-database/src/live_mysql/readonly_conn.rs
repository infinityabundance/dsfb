//! Type-level read-only MySQL connection wrapper.
//!
//! [`ReadOnlyMySqlConn`] owns a `mysql_async::Conn` behind a private
//! field. The only public call that executes SQL is
//! [`ReadOnlyMySqlConn::query_allowed`], and it takes a
//! [`crate::live_mysql::queries::AllowedMySqlQuery`] enum variant.
//! There is no way to smuggle a raw SQL string through this type.
//! The mysql_async methods `query_drop`, `query_iter`,
//! `query_first`, `exec_drop`, `exec_iter`, and `exec_batch` are
//! **not** re-exported and cannot be reached through a `Deref` or
//! `AsRef` (none is impl'd).
//!
//! On top of the type-level contract, the constructor installs three
//! session-level controls:
//!   * `SET SESSION TRANSACTION READ ONLY`  -- engine rejects writes.
//!   * `SET SESSION MAX_EXECUTION_TIME = 500`  -- 500 ms ceiling per
//!     statement (MySQL 5.7.8+). Prevents a runaway
//!     performance_schema query from impacting the engine.
//!   * `SET SESSION innodb_lock_wait_timeout = 1`  -- yield before
//!     blocking on any InnoDB row lock. Reads of
//!     performance_schema do not take InnoDB locks, but this is a
//!     belt-and-braces control in case a view is extended.
//!
//! After issuing the three SETs we re-read them via
//! `SHOW SESSION VARIABLES` and refuse to proceed if any did not
//! take effect — the engine must explicitly confirm the session
//! contract.

use super::queries::AllowedMySqlQuery;
use anyhow::{anyhow, Context, Result};
use mysql_async::prelude::*;
use mysql_async::{Conn, OptsBuilder, Row};

/// Read-only MySQL client wrapper.
///
/// The wrapped `mysql_async::Conn` is private by construction.
/// Downstream code can only interact with the database via
/// [`Self::query_allowed`]. This is the type-level layer of the
/// code-audit contract documented in [`crate::live_mysql`].
pub struct ReadOnlyMySqlConn {
    conn: Conn,
}

impl ReadOnlyMySqlConn {
    /// Open a connection and install the session-level read-only
    /// guards. `url` follows the mysql_async URL convention:
    /// `mysql://user:password@host:port/database`.
    pub async fn connect(url: &str) -> Result<Self> {
        let opts = OptsBuilder::from_opts(
            mysql_async::Opts::from_url(url)
                .with_context(|| "mysql_async::Opts::from_url failed")?,
        );
        let mut conn = Conn::new(opts)
            .await
            .with_context(|| "mysql_async::Conn::new failed")?;
        // Session-level controls. Executed as `query_drop` because
        // SET statements return no rows. We gate behind `query_drop`
        // locally — this single call site is auditable in isolation,
        // and the statements are fully enumerated below.
        for stmt in [
            "SET SESSION TRANSACTION READ ONLY",
            "SET SESSION MAX_EXECUTION_TIME = 500",
            "SET SESSION innodb_lock_wait_timeout = 1",
        ] {
            conn.query_drop(stmt)
                .await
                .with_context(|| format!("failed to issue {stmt}"))?;
        }
        // Verify the session-level guards took effect. If the engine
        // silently ignores any SET (forked engine, bad permissions,
        // etc.) we refuse to proceed.
        let tx_read_only: Option<String> = conn
            .query_first("SELECT @@SESSION.transaction_read_only")
            .await
            .with_context(|| "failed to read @@SESSION.transaction_read_only")?;
        match tx_read_only.as_deref() {
            Some("1") | Some("ON") => {}
            other => {
                return Err(anyhow!(
                    "refusing to proceed: @@SESSION.transaction_read_only = {:?}, expected 1/ON",
                    other
                ))
            }
        }
        let max_exec: Option<u64> = conn
            .query_first("SELECT @@SESSION.MAX_EXECUTION_TIME")
            .await
            .with_context(|| "failed to read @@SESSION.MAX_EXECUTION_TIME")?;
        if max_exec != Some(500) {
            return Err(anyhow!(
                "refusing to proceed: @@SESSION.MAX_EXECUTION_TIME = {:?}, expected 500",
                max_exec
            ));
        }
        let lock_wait: Option<u64> = conn
            .query_first("SELECT @@SESSION.innodb_lock_wait_timeout")
            .await
            .with_context(|| "failed to read @@SESSION.innodb_lock_wait_timeout")?;
        if lock_wait != Some(1) {
            return Err(anyhow!(
                "refusing to proceed: @@SESSION.innodb_lock_wait_timeout = {:?}, expected 1",
                lock_wait
            ));
        }
        Ok(Self { conn })
    }

    /// Execute an allow-listed query. No raw SQL surface is exposed:
    /// the caller chooses a variant of the closed
    /// [`AllowedMySqlQuery`] enum, the wrapper looks up the pinned
    /// SQL text, and returns the resulting rows.
    pub async fn query_allowed(&mut self, q: AllowedMySqlQuery) -> Result<Vec<Row>> {
        let sql = q.sql();
        self.conn
            .query(sql)
            .await
            .with_context(|| format!("query_allowed failed for {:?}", q))
    }

    /// Close the connection gracefully. mysql_async's `Conn` closes
    /// on drop; this is an explicit async path for the scraper to
    /// flush before tape finalisation.
    pub async fn disconnect(self) -> Result<()> {
        self.conn
            .disconnect()
            .await
            .with_context(|| "mysql_async Conn::disconnect failed")
    }
}

// NOTE: No `Deref<Target = mysql_async::Conn>` impl. No
// `AsRef<mysql_async::Conn>` impl. No getter. Those would each be a
// hole in the code-audit contract — the connection is deliberately
// inaccessible outside this module.
