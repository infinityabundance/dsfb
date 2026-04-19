# Fault: sustained row-level lock hold (contention ramp).
#
# At t=30 s a separate psql session begins a transaction that takes a
# FOR UPDATE lock on a single pgbench_accounts row and holds it for
# 30 s. The c=16 j=4 pgbench load includes UPDATEs that must wait on
# that row with some probability. The wait-event distribution on the
# UPDATE qid shifts toward Lock/tuple and idle_in_transaction_session,
# which produces a sustained residual on the contention_ramp motif's
# channels.

FAULT_MOTIF="contention_ramp"
FAULT_DESCRIPTION="At t=30.0 s a background psql session holds a row-level lock (BEGIN; SELECT ... FROM pgbench_accounts WHERE aid = 1 FOR UPDATE;) for 30 s under c=16 j=4 pgbench scale-10 load. UPDATEs on pgbench_accounts that collide with aid=1 accumulate Lock:tuple and idle_in_transaction_session wait events, driving the contention_ramp motif on the UPDATE qid."

fault_inject() {
  # Background psql process that holds the lock for 30s, then commits.
  (
    podman exec -i "${PG_CONTAINER}" psql \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
      --no-psqlrc --set=ON_ERROR_STOP=1 <<'SQL' >/dev/null 2>&1 || true
BEGIN;
SELECT abalance FROM pgbench_accounts WHERE aid = 1 FOR UPDATE;
SELECT pg_sleep(30);
COMMIT;
SQL
  ) &
  FAULT_LOCK_HOLDER_PID=$!
  export FAULT_LOCK_HOLDER_PID
}

fault_restore() {
  if [[ -n "${FAULT_LOCK_HOLDER_PID:-}" ]]; then
    wait "${FAULT_LOCK_HOLDER_PID}" 2>/dev/null || true
  fi
  # Defensive: terminate any remaining backend holding a lock on aid=1.
  psql_admin --command \
    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
     WHERE state = 'idle in transaction' AND datname = '${PG_DB}';" \
    >/dev/null 2>&1 || true
}

fault_channels() {
  podman exec "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" --no-psqlrc \
    --tuples-only --no-align --command \
    "SELECT md5(queryid::text) FROM pg_stat_statements \
     WHERE query LIKE 'UPDATE pgbench_accounts SET abalance = %' \
     ORDER BY calls DESC LIMIT 1;" 2>/dev/null | tr -d '[:space:]'
  echo
}
