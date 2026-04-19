# Fault: DROP CONSTRAINT pgbench_accounts_pkey (structural plan regression).
#
# Removing the PK forces sequential scans on every SELECT abalance and
# every UPDATE on pgbench_accounts. Under c=16 j=4 load this produces a
# large per-statement execution-time slew on the two pgbench account
# qids: the plan_regression_onset motif.
#
# Contract:
#   FAULT_MOTIF         - motif name for ground truth
#   FAULT_DESCRIPTION   - prose for ground_truth.json fault_description
#   fault_inject        - bash function; injects fault via psql_admin
#   fault_restore       - bash function; restores baseline state
#   fault_channels      - bash function; echoes one qid-md5 per line
#                         (the channels the ground-truth windows will
#                          be pinned to). Empty output ⇒ no windows.

FAULT_MOTIF="plan_regression_onset"
FAULT_DESCRIPTION="ALTER TABLE pgbench_accounts DROP CONSTRAINT pgbench_accounts_pkey executed at t=30.0 s under c=16 j=4 pgbench scale-10 TPC-B-like load. Removing the PK forces sequential scans on every SELECT abalance FROM pgbench_accounts WHERE aid = \$1 and every UPDATE pgbench_accounts, causing a per-statement execution-time slew on the two pgbench account qids."

fault_inject() {
  psql_admin --command \
    "ALTER TABLE pgbench_accounts DROP CONSTRAINT pgbench_accounts_pkey;"
}

fault_restore() {
  psql_admin --command \
    "ALTER TABLE pgbench_accounts ADD CONSTRAINT pgbench_accounts_pkey PRIMARY KEY (aid);" \
    >/dev/null 2>&1 || true
}

fault_channels() {
  podman exec "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" --no-psqlrc \
    --tuples-only --no-align --command \
    "SELECT md5(queryid::text) FROM pg_stat_statements \
     WHERE query LIKE 'SELECT abalance FROM pgbench_accounts WHERE aid = %' \
     ORDER BY calls DESC LIMIT 1;" 2>/dev/null | tr -d '[:space:]'
  echo
  podman exec "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" --no-psqlrc \
    --tuples-only --no-align --command \
    "SELECT md5(queryid::text) FROM pg_stat_statements \
     WHERE query LIKE 'UPDATE pgbench_accounts SET abalance = %' \
     ORDER BY calls DESC LIMIT 1;" 2>/dev/null | tr -d '[:space:]'
  echo
}
