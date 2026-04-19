//! Query allow-list for the live MySQL adapter.
//!
//! This file is the MySQL analogue of `src/live/queries.rs`. Every
//! SQL statement the live-mysql adapter can ever issue is enumerated
//! by [`AllowedMySqlQuery`] and maps to a `'static` SQL string via
//! [`AllowedMySqlQuery::sql`]. The concatenated SQL texts are
//! SHA-256-pinned by `tests/live_query_allowlist_lock_mysql.rs`: any
//! edit — even an added comment — forces an intentional lock bump
//! that must be co-authored with the paper's §Live-Eval MySQL
//! subsection and the `spec/permissions.mysql.sql` manifest.
//!
//! All four variants are pure `SELECT` against `performance_schema`
//! and `information_schema`. None touch user tables. None issue DDL,
//! DML, or advisory locks. This is the statement-level layer of the
//! code-audit contract documented in [`crate::live_mysql`].
//!
//! The residual-class mapping mirrors the PostgreSQL path:
//!   * DigestSnapshot  -> PlanRegression / WorkloadPhase
//!   * ThreadsSnapshot -> Contention (wait-event samples)
//!   * MetadataLocksSnapshot -> Contention (per-object lock waits)
//!   * BufferPoolSnapshot    -> CacheIo

/// Closed enumeration of every SQL statement the live-mysql adapter
/// will ever execute. Adding a variant is a reviewable change that
/// simultaneously breaks [`Self::sql_concat_for_lock`] and therefore
/// the allow-list lock test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedMySqlQuery {
    /// Per-digest cumulative latency / call / row counters.
    /// Source: `performance_schema.events_statements_summary_by_digest`
    /// (MySQL 5.6+; column names stable through 8.4).
    /// Residual classes emitted: PlanRegression, WorkloadPhase.
    DigestSnapshot,
    /// Per-thread wait-event sample.
    /// Source: `performance_schema.threads` (MySQL 5.6+).
    /// Residual class emitted: Contention.
    ThreadsSnapshot,
    /// Per-object metadata-lock wait snapshot.
    /// Source: `performance_schema.metadata_locks` (MySQL 5.7+).
    /// Residual class emitted: Contention (no PostgreSQL analog).
    MetadataLocksSnapshot,
    /// Per-pool InnoDB buffer-pool cumulative counters.
    /// Source: `information_schema.innodb_buffer_pool_stats`
    /// (MySQL 5.6+).
    /// Residual class emitted: CacheIo.
    BufferPoolSnapshot,
}

impl AllowedMySqlQuery {
    /// Every variant the adapter knows about. Used by the lock test
    /// and by the scraper to enumerate a full poll cycle in a
    /// deterministic order.
    pub const ALL: [AllowedMySqlQuery; 4] = [
        Self::DigestSnapshot,
        Self::ThreadsSnapshot,
        Self::MetadataLocksSnapshot,
        Self::BufferPoolSnapshot,
    ];

    /// Pinned SQL text for this variant. Every string is a `SELECT`
    /// against `performance_schema` or `information_schema`. Any edit
    /// invalidates the lock test and forces a paired paper update.
    pub fn sql(&self) -> &'static str {
        match self {
            Self::DigestSnapshot => {
                // MD5 of the digest canonicalises the identifier so
                // the emitted residual stream contains no raw digest
                // text. Analogous to the md5(queryid::text) choice in
                // the PostgreSQL path.
                "SELECT \
                   UNIX_TIMESTAMP(NOW(6)) AS snapshot_t, \
                   MD5(DIGEST) AS digest_id, \
                   COUNT_STAR AS calls, \
                   SUM_TIMER_WAIT / 1000000000.0 AS total_exec_time_ms \
                 FROM performance_schema.events_statements_summary_by_digest \
                 WHERE DIGEST IS NOT NULL"
            }
            Self::ThreadsSnapshot => {
                // No query-text columns, no client-host columns.
                // Counts wait-event samples per category in the
                // distiller, so the PII surface collapses to "how
                // many sessions are waiting on which wait-event
                // class".
                "SELECT \
                   UNIX_TIMESTAMP(NOW(6)) AS snapshot_t, \
                   COALESCE(PROCESSLIST_STATE, 'None') AS wait_event_type, \
                   COALESCE(PROCESSLIST_COMMAND, 'None') AS wait_event, \
                   COUNT(*) AS n_threads \
                 FROM performance_schema.threads \
                 WHERE PROCESSLIST_ID IS NOT NULL \
                 GROUP BY PROCESSLIST_STATE, PROCESSLIST_COMMAND"
            }
            Self::MetadataLocksSnapshot => {
                // Aggregates per-status counts to avoid emitting raw
                // object identifiers. A pure read: performance_schema
                // exposes lock metadata via SELECT, not a lock-taking
                // call.
                "SELECT \
                   UNIX_TIMESTAMP(NOW(6)) AS snapshot_t, \
                   COALESCE(LOCK_STATUS, 'None') AS lock_status, \
                   COALESCE(LOCK_TYPE, 'None') AS lock_type, \
                   COUNT(*) AS n_waiters \
                 FROM performance_schema.metadata_locks \
                 GROUP BY LOCK_STATUS, LOCK_TYPE"
            }
            Self::BufferPoolSnapshot => {
                // Per-pool cache counters. InnoDB exposes them via
                // information_schema; the adapter consumes the delta
                // between adjacent snapshots in the distiller.
                "SELECT \
                   UNIX_TIMESTAMP(NOW(6)) AS snapshot_t, \
                   POOL_ID AS pool_id, \
                   PAGES_DATA AS pages_data, \
                   PAGES_MISC AS pages_misc, \
                   PAGES_FREE AS pages_free, \
                   PAGES_MADE_YOUNG AS pages_made_young, \
                   PAGES_READ AS pages_read, \
                   PAGES_CREATED AS pages_created, \
                   PAGES_WRITTEN AS pages_written \
                 FROM information_schema.innodb_buffer_pool_stats"
            }
        }
    }

    /// Concatenation of every variant's SQL text, in `ALL`-order,
    /// separated by a single newline. The SHA-256 of this string is
    /// pinned by `tests/live_query_allowlist_lock_mysql.rs`.
    pub fn sql_concat_for_lock() -> String {
        let mut s = String::new();
        for (i, q) in Self::ALL.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            s.push_str(q.sql());
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_enumerated_once() {
        let n = AllowedMySqlQuery::ALL.len();
        let unique: std::collections::HashSet<_> =
            AllowedMySqlQuery::ALL.iter().copied().collect();
        assert_eq!(n, unique.len(), "duplicate variant in AllowedMySqlQuery::ALL");
    }

    #[test]
    fn every_variant_is_pure_select() {
        for q in AllowedMySqlQuery::ALL {
            let sql = q.sql();
            let head = sql.trim_start().to_uppercase();
            assert!(
                head.starts_with("SELECT"),
                "AllowedMySqlQuery::{:?} does not start with SELECT: {}",
                q,
                sql
            );
            let tokens: Vec<&str> = head
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .filter(|t| !t.is_empty())
                .collect();
            for kw in &[
                "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", "ALTER",
                "GRANT", "REVOKE", "TRUNCATE", "LOCK", "UNLOCK", "CALL",
                "LOAD", "HANDLER",
            ] {
                assert!(
                    !tokens.iter().any(|t| t == kw),
                    "AllowedMySqlQuery::{:?} contains forbidden keyword {}: {}",
                    q,
                    kw,
                    sql
                );
            }
        }
    }

    #[test]
    fn sql_concat_is_deterministic() {
        let a = AllowedMySqlQuery::sql_concat_for_lock();
        let b = AllowedMySqlQuery::sql_concat_for_lock();
        assert_eq!(a, b);
    }
}
