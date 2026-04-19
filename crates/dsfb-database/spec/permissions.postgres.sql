-- DSFB-Database live adapter: minimum-privilege PostgreSQL role.
--
-- This script provisions the role that the `dsfb-database live`
-- subcommand connects as. It grants only the `pg_read_all_stats`
-- predefined role (PostgreSQL 14+) — the read-only privilege that
-- exposes `pg_stat_statements`, `pg_stat_activity.wait_event`,
-- `pg_stat_io`, and `pg_stat_database` without exposing any user
-- data. The role is pinned into a read-only transaction mode by
-- default and has aggressive session-level timeouts so that a
-- misbehaving observer cannot hold resources.
--
-- This manifest is a *starting point* for hardening, not a complete
-- security posture. A real deployment should additionally constrain:
--   * network reachability (`pg_hba.conf` host rules, firewall)
--   * connection-count caps (`max_connections` per-role)
--   * engine-side logging retention (statement logging, audit log)
-- all of which are engine-level controls outside this crate's scope.
--
-- The crate's §10 non-claim #7 and Appendix F of the paper state the
-- boundary of this manifest explicitly: the three layered controls
-- (type-level, session-level, statement-level) constitute a
-- *code-path contract*, not an unforgeable cryptographic proof.

-- 1. Create the observer role. Pick a strong password in your
--    deployment; the placeholder below MUST NOT be used in
--    production.
CREATE ROLE dsfb_observer WITH LOGIN PASSWORD 'change-me-in-production';

-- 2. Grant the predefined read-all-stats role. This is the least
--    privilege that exposes the pg_stat_* views the adapter polls.
--    It does NOT grant SELECT on user tables; the allow-listed
--    queries never touch them.
GRANT pg_read_all_stats TO dsfb_observer;

-- 3. Pin this role's default transaction mode to read-only. This is
--    the session-level layer of the code-audit contract: even if a
--    client bug bypassed the type-level surface, the engine would
--    refuse any write.
ALTER ROLE dsfb_observer SET default_transaction_read_only = on;

-- 4. Statement-level timeout. A poll that hangs past this budget is
--    aborted by the engine. 500 ms is ample for pg_stat_statements
--    snapshots on databases with up to ~1e6 distinct queries; tune
--    upward only after measuring actual poll distributions on your
--    instance.
ALTER ROLE dsfb_observer SET statement_timeout = '500ms';

-- 5. Idle-in-transaction timeout. The adapter never holds an
--    explicit transaction, but a buggy client could. This bound
--    ensures that even in the pathological case the observer
--    releases locks within one second.
ALTER ROLE dsfb_observer SET idle_in_transaction_session_timeout = '1s';

-- 6. No GRANT on any user-created schema. This is a non-action, but
--    documented here to make the omission explicit: the observer
--    has no access to user data, only to the stats views.

-- OPTIONAL hardening: restrict connections to the local socket.
-- Add this to `pg_hba.conf` to forbid TCP connections for this role:
--
--     local   all   dsfb_observer   peer
--     host    all   dsfb_observer   0.0.0.0/0   reject
--
-- Combined with the above role settings, this yields a three-layer
-- access posture: type-level (crate code), session-level
-- (PostgreSQL role), and network-level (pg_hba.conf).
