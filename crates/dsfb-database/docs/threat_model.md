# Threat model — dsfb-database live adapters

This document is a STRIDE-style threat model for the live read-only
PostgreSQL and MySQL adapters shipped under `src/live/` and
`src/live_mysql/`. It complements (does not replace) the crate-root
[`SECURITY.md`](../SECURITY.md), which covers the offline parsing
surface and the artefact-integrity invariants.

The model documents the boundaries that the three-layer code-audit
contract enforces and — equally — the boundaries it does **not**
enforce. Operators deploying the crate against a production engine
should read both this document and the verbatim role manifests at
[`spec/permissions.postgres.sql`](../spec/permissions.postgres.sql) and
[`spec/permissions.mysql.sql`](../spec/permissions.mysql.sql).

## Scope

In scope:

- The `dsfb_observer` role on PostgreSQL 16 / 17 reading
  `pg_stat_statements`, `pg_stat_activity`, `pg_stat_io`.
- The equivalent MySQL 8 role reading
  `performance_schema.events_statements_summary_by_digest`,
  `performance_schema.threads`,
  `performance_schema.metadata_locks`,
  `information_schema.innodb_buffer_pool_stats`.
- The `ReadOnlyPgConn` / `ReadOnlyMySqlConn` wrapper types in
  `src/live/readonly_conn.rs` / `src/live_mysql/readonly_conn.rs`.
- The SHA-256-pinned allow-lists in `src/live/queries.rs` and
  `src/live_mysql/queries.rs`, locked by
  `tests/live_query_allowlist_lock.rs` and
  `tests/live_query_allowlist_lock_mysql.rs`.
- The persisted tape format under `src/live/tape.rs` (line-oriented
  JSONL with a SHA-256 manifest sidecar).

Out of scope (delegated to the operator's broader security posture):

- Engine-level RBAC beyond what `pg_hba.conf` / MySQL host filters
  enforce.
- Network-layer controls (TLS termination, mTLS, IP allow-listing).
- Supply-chain integrity of the binary in production (covered by
  `cargo-deny` + `cargo-audit` in `ci/gate.sh` at build time, but the
  deployed binary is operator-trusted at run time).
- Side channels on host-shared infrastructure (CPU-cache, hyperthread,
  memory-pressure leakage between tenants on the same host).

## STRIDE walk

### S — Spoofing
- *Engine impersonation.* The adapter authenticates against the engine
  using a connection string supplied by the operator. We do not pin the
  server certificate; the operator must use TLS with a trusted CA
  (rustls is the default TLS backend for both adapters; the MySQL
  adapter's `default-rustls` feature is set at
  [`Cargo.toml:119`](../Cargo.toml)).
- *Adapter impersonation.* Mitigated by the role manifests:
  `dsfb_observer` is a least-privilege role distinct from any operator
  account; revoking it isolates DSFB cleanly.

### T — Tampering
- *Query rewriting.* Mitigated at three layers. (1) Type-level:
  `ReadOnlyPgConn` / `ReadOnlyMySqlConn` do not expose `execute`,
  `prepare`, `transaction`, `copy_in`, `batch_execute` — proven by the
  five trybuild compile-fail tests under
  `tests/trybuild_readonly_conn/`. (2) Session-level: the connection
  is opened with `default_transaction_read_only = on` (PG) or
  `SET SESSION TRANSACTION READ ONLY` (MySQL), plus
  `statement_timeout = 500ms` (PG) or `MAX_EXECUTION_TIME = 500` (MySQL),
  and `lock_timeout = 1ms` / `innodb_lock_wait_timeout = 1`. (3)
  Statement-level: the closed allow-list of six PG queries / four MySQL
  queries is SHA-256-pinned, and the lock test asserts no new variant
  can be added without updating the hash.
- *Tape tampering.* The tape file's SHA-256 manifest sidecar is
  verified by `live::tape::load_and_verify` before replay. A modified
  tape fails the manifest check before any episode is emitted.
- *In-flight tampering.* Mitigated by TLS to the engine; not by the
  adapter itself.

### R — Repudiation
- The persisted tape carries a SHA-256 over its byte content; the
  episode stream a replayer derives is a deterministic function of the
  tape (locked by `tests/live_tape_replay_is_deterministic.rs`). An
  operator can therefore prove "this episode came from this tape" by
  re-running the replay; an auditor can prove "this tape had this
  content at this time" by checking the manifest signature against an
  out-of-band record.
- We do not provide non-repudiation for the *engine→tape* step (this
  is non-claim #7's load-bearing distinction). Two live invocations
  produce different tapes.

### I — Information disclosure
- *Tenant query-text leakage.* `pg_stat_statements` is an
  engine-global view by default. A `dsfb_observer` role with
  `pg_read_all_stats` reads query-text fingerprints (and on PG14+
  optionally the normalized text) for **all tenants on the engine**.
  An operator running a multi-tenant database must either:
  - restrict `dsfb_observer` per-tenant via row-level security on a
    custom view (PG14+), OR
  - deploy a per-tenant DSFB instance, OR
  - accept that DSFB output may carry cross-tenant query-shape
    information and route the output accordingly.

  This is documented in paper §38 (Pass-2 limitation) and is **not**
  mitigated by the adapter's three layers — the layers prevent writes,
  not cross-tenant reads. The same caveat applies to MySQL
  `performance_schema.events_statements_summary_by_digest`, which is
  global by design.
- *Statement-text-vs-fingerprint.* The PG adapter reads
  `pg_stat_statements.queryid` (a stable hash) and the *normalized*
  query text (`pg_stat_statements.query` with parameter literals
  replaced by `$N`). Literal user data (passwords, PII embedded as
  literals) does not enter the stream **unless** the operator has
  disabled normalisation; we recommend the default
  (`pg_stat_statements.track_utility = off` and parameter normalisation
  on).
- *Side channels.* Out of scope (see "Out of scope" above).

### D — Denial of service
- *Engine-load attack.* Mitigated by `statement_timeout=500ms` and
  `lock_timeout=1ms` (PG) / `MAX_EXECUTION_TIME=500ms` and
  `innodb_lock_wait_timeout=1` (MySQL). A pathological pinned query
  cannot stall the engine for longer than the timeout; an
  operator-side LIMIT clause in each allow-list query also bounds the
  result-set size.
- *Adapter-load attack.* The polling cadence is operator-configured
  (`--interval-ms`, default 500). At 500 ms × 4 queries, the adapter
  issues ~8 queries / sec to the engine — within the noise floor of
  any production workload.
- *Storage exhaustion.* The tape grows linearly with run length. The
  observer-load harness (Pass-1, `experiments/observer_load/`)
  measures per-rep tape size; an operator should set a tape rotation
  policy.

### E — Elevation of privilege
- The role manifests grant only `pg_read_all_stats` (PG) or four
  `SELECT` privileges on `performance_schema` / `information_schema`
  views (MySQL). No `SUPERUSER`, no `CREATE ROLE`, no
  `pg_signal_backend`, no `pg_terminate_backend`, no DDL.
- A compromised `dsfb_observer` cannot escalate within the engine
  via the layers shipped here. (A compromised host running the binary
  is out of scope; supply-chain integrity is the operator's
  responsibility once the binary is deployed.)

## Residual risks (not mitigated by the adapter contract)

These are documented as paper limitations and named here for an
operator's deployment review:

| # | Risk | Paper §  | Mitigation responsibility |
|---|---|---|---|
| 1 | Multi-tenant query-text leakage via shared `pg_stat_statements` | §38 | Operator (per-tenant view or per-tenant deployment) |
| 2 | Adversarial workload crafted to hit motif thresholds | §44 | Consumer of DSFB output (rate-limit alerts) |
| 3 | Replication-lag-induced read drift if pointed at a hot standby | §42 | Operator (deploy against primary OR add `pg_last_wal_replay_lsn` channel) |
| 4 | Schema-evolution false phase-transition motifs | §46 | Operator (add schema-version channel) |
| 5 | OS scheduler / CFS jitter on container with CPU limits | §47 | Operator (set CPU shares; non-claim #7 disclosure) |
| 6 | Cryptographic non-repudiation (the read-only contract is structural, not cryptographic) | §31 | Operator (combine with engine RBAC + supply-chain verification) |

## Reporting

Vulnerabilities in the live adapters fall under the same disclosure
policy as the rest of the crate. Contact `security@invariantforge.net`
per [`SECURITY.md`](../SECURITY.md).
