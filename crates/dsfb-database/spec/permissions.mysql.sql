-- DSFB-Database: least-privilege MySQL role for the live-mysql adapter.
--
-- This file is the verbatim operator-facing contract for the crate's
-- MySQL read-only adapter. It is the MySQL analogue of
-- spec/permissions.postgres.sql. The adapter's session-level and
-- statement-level controls are pinned in
-- src/live_mysql/readonly_conn.rs and src/live_mysql/queries.rs
-- respectively; this file pins the *engine-side* controls the
-- operator must apply.
--
-- Three-layer contract (code-audit, not cryptographic):
--   1. Type-level:      ReadOnlyMySqlConn + AllowedMySqlQuery
--                       (src/live_mysql/).
--   2. Session-level:   SET SESSION TRANSACTION READ ONLY,
--                       SET SESSION MAX_EXECUTION_TIME = 500,
--                       SET SESSION innodb_lock_wait_timeout = 1
--                       (issued on every session and re-verified via
--                        SHOW SESSION VARIABLES).
--   3. Statement-level: every wire-bound statement is a variant of
--                       AllowedMySqlQuery, pure SELECT against
--                       performance_schema.* / information_schema.*,
--                       SHA-256-pinned by
--                       tests/live_query_allowlist_lock_mysql.rs.
--
-- Together these mean a static audit of the three files plus this
-- manifest is sufficient to conclude that the adapter cannot mutate
-- engine state. The crate's 7th non-claim is explicit that this is a
-- *code-audit* contract, not a cryptographic proof.

-- Step 1: create the observer account. Replace the password below
-- with a per-site secret and store it in the operator's secrets
-- manager. The 'requireSSL' clause is recommended for any remote
-- connection; adjust to site policy.
CREATE USER 'dsfb_observer'@'%' IDENTIFIED BY 'REPLACE_ME_PER_SITE' REQUIRE SSL;

-- Step 2: grant SELECT on the four performance_schema /
-- information_schema views the adapter reads. These are the MySQL
-- analogues of PostgreSQL's pg_stat_statements / pg_stat_activity /
-- pg_stat_io / pg_stat_database.
--   * events_statements_summary_by_digest:  per-digest cumulative
--                                           counters (plan regression,
--                                           workload phase surfaces).
--   * threads:                              per-session wait-event
--                                           samples (contention).
--   * metadata_locks:                       per-object lock waits
--                                           (contention, no PG analog).
--   * innodb_buffer_pool_stats:             per-pool cache counters
--                                           (cache-io surface).
GRANT SELECT ON performance_schema.events_statements_summary_by_digest
    TO 'dsfb_observer'@'%';
GRANT SELECT ON performance_schema.threads
    TO 'dsfb_observer'@'%';
GRANT SELECT ON performance_schema.metadata_locks
    TO 'dsfb_observer'@'%';
GRANT SELECT ON information_schema.innodb_buffer_pool_stats
    TO 'dsfb_observer'@'%';

-- Step 3: deny everything else by omission. MySQL default privileges
-- for a freshly-created user are no-privileges. The adapter cannot
-- reach user tables, cannot issue DML, cannot issue DDL, cannot
-- install UDFs, cannot read mysql.* or sys.*, cannot use
-- performance_schema.setup_* (which would let it toggle
-- instrumentation). Any such attempt will be rejected by MySQL at
-- the access-check layer.

-- Step 4: commit the grant.
FLUSH PRIVILEGES;

-- Step 5: operator-side session controls. These are re-issued by
-- ReadOnlyMySqlConn::connect on every connection and re-verified
-- before any query is allowed to run, but we document them here so
-- the operator sees the full surface.
--   SET SESSION TRANSACTION READ ONLY           -- engine rejects writes
--   SET SESSION MAX_EXECUTION_TIME = 500        -- 500 ms ceiling
--   SET SESSION innodb_lock_wait_timeout = 1    -- yield before blocking

-- References:
--   * MySQL Performance Schema manual, chapter 29.
--   * MySQL Access Control, chapter 8.2.
--   * dsfb-database paper §Live read-only telemetry adapter,
--     §Live Evaluation MySQL subsection.
