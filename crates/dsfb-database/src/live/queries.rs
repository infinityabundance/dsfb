//! Query allow-list for the live PostgreSQL adapter.
//!
//! Every SQL statement the live adapter can ever issue is enumerated
//! by [`AllowedQuery`] and maps to a `'static` SQL string via
//! [`AllowedQuery::sql`]. The concatenated SQL texts are
//! SHA-256-pinned by `tests/live_query_allowlist_lock.rs`: any edit to
//! a statement — even an added comment — forces an intentional lock
//! bump that must be co-authored with the paper's Appendix F.
//!
//! All four variants are pure `SELECT` against `pg_catalog.*` and
//! `pg_stat_*` views. None touch user tables. None issue DDL, DML, or
//! advisory locks. This is the statement-level layer of the
//! code-audit contract documented in [`crate::live`].

/// Closed enumeration of every SQL statement the live adapter will
/// ever execute. Adding a variant is a reviewable change that
/// simultaneously breaks [`Self::sql_concat_for_lock`] and therefore
/// the allow-list lock test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedQuery {
    /// Per-query cumulative latency / call counters.
    /// Source: `pg_stat_statements` extension (PostgreSQL 13+).
    /// Residual classes emitted: PlanRegression, WorkloadPhase.
    PgStatStatementsSnapshot,
    /// Per-session wait-event samples.
    /// Source: `pg_stat_activity` (PostgreSQL 9.6+).
    /// Residual class emitted: Contention.
    PgStatActivitySnapshot,
    /// Per-backend-type cumulative I/O counters.
    /// Source: `pg_stat_io` (PostgreSQL 16+).
    /// Residual class emitted: CacheIo.
    PgStatIoSnapshot,
    /// Per-database cumulative block-hit / block-read counters.
    /// Source: `pg_stat_database` (PostgreSQL 9.0+).
    /// Used as a CacheIo fallback when `pg_stat_io` is unavailable
    /// (PG < 16).
    PgStatDatabaseSnapshot,
}

impl AllowedQuery {
    /// Every variant the adapter knows about. Used by the lock test
    /// and by the scraper to enumerate a full poll cycle in a
    /// deterministic order.
    pub const ALL: [AllowedQuery; 4] = [
        Self::PgStatStatementsSnapshot,
        Self::PgStatActivitySnapshot,
        Self::PgStatIoSnapshot,
        Self::PgStatDatabaseSnapshot,
    ];

    /// Pinned SQL text for this variant. Every string is a
    /// `SELECT` against `pg_stat_*` or `pg_catalog.*`. Any edit
    /// invalidates the lock test and forces a paired paper update.
    pub fn sql(&self) -> &'static str {
        match self {
            Self::PgStatStatementsSnapshot => {
                // `md5(queryid::text)` anonymises the query id so the
                // emitted residual stream contains no query text and no
                // raw queryid (still a one-way function of the raw id,
                // so a malicious snapshot set could be replayed against
                // a rainbow table — the engine side, not the crate,
                // must enforce that the extension is only installed on
                // non-PII workloads).
                "SELECT \
                   extract(epoch from now())::float8 AS snapshot_t, \
                   md5(queryid::text) AS query_id, \
                   calls AS calls, \
                   total_exec_time AS total_exec_time_ms \
                 FROM pg_stat_statements"
            }
            Self::PgStatActivitySnapshot => {
                // Only wait_event + wait_event_type + pid + state
                // columns; no query text, no client_addr. We count
                // wait-event samples per category in the distiller,
                // so the PII surface collapses to "how many sessions
                // are waiting on which wait-event class".
                "SELECT \
                   extract(epoch from now())::float8 AS snapshot_t, \
                   coalesce(wait_event_type, 'None') AS wait_event_type, \
                   coalesce(wait_event, 'None') AS wait_event, \
                   state AS state \
                 FROM pg_stat_activity \
                 WHERE pid <> pg_backend_pid()"
            }
            Self::PgStatIoSnapshot => {
                // PG 16+. `pg_stat_io` aggregates reads/writes per
                // backend_type + object + context; we sum hits and
                // read-time for the CacheIo residual.
                "SELECT \
                   extract(epoch from now())::float8 AS snapshot_t, \
                   coalesce(backend_type, 'unknown') AS backend_type, \
                   coalesce(object, 'unknown') AS object, \
                   coalesce(context, 'unknown') AS context, \
                   coalesce(reads, 0) AS reads, \
                   coalesce(hits, 0) AS hits, \
                   coalesce(read_time, 0)::float8 AS read_time_ms \
                 FROM pg_stat_io"
            }
            Self::PgStatDatabaseSnapshot => {
                // PG 9.0+. Used as CacheIo fallback (hit-ratio only)
                // when pg_stat_io is unavailable.
                "SELECT \
                   extract(epoch from now())::float8 AS snapshot_t, \
                   coalesce(datname, '(null)') AS datname, \
                   coalesce(blks_hit, 0) AS blks_hit, \
                   coalesce(blks_read, 0) AS blks_read \
                 FROM pg_stat_database \
                 WHERE datname IS NOT NULL"
            }
        }
    }

    /// Concatenate every variant's SQL in [`Self::ALL`] order with a
    /// `\n---\n` separator. This is the input to the allow-list lock
    /// test's SHA-256.
    pub fn sql_concat_for_lock() -> String {
        let mut out = String::new();
        for (i, q) in Self::ALL.iter().enumerate() {
            if i > 0 {
                out.push_str("\n---\n");
            }
            out.push_str(q.sql());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_query_is_a_select_on_pg_catalog_or_pg_stat() {
        for q in AllowedQuery::ALL.iter() {
            let sql = q.sql();
            let trimmed = sql.trim_start();
            assert!(
                trimmed.starts_with("SELECT"),
                "allow-listed query {:?} is not a SELECT: {}",
                q,
                sql
            );
            assert!(
                sql.contains("pg_stat_") || sql.contains("pg_catalog."),
                "allow-listed query {:?} does not reference pg_stat_* or pg_catalog.*: {}",
                q,
                sql
            );
            // Paranoid negative assertions — not comprehensive, the lock
            // test is the real defense.
            for forbidden in [
                "INSERT",
                "UPDATE",
                "DELETE",
                "DROP",
                "ALTER",
                "CREATE",
                "TRUNCATE",
                "GRANT",
                "REVOKE",
                "COPY",
            ] {
                assert!(
                    !sql.to_uppercase().contains(forbidden),
                    "allow-listed query {:?} contains forbidden keyword {}",
                    q,
                    forbidden
                );
            }
        }
    }
}
