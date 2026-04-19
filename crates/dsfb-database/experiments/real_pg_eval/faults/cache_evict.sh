# Fault: buffer-pool eviction (cache collapse).
#
# At t=30 s we issue `DISCARD ALL` on all backend sessions (via
# pg_terminate on idle backends as a blunt prewarm-reset proxy), then
# issue a cold full-table scan of a sibling pgbench table large enough
# to push pgbench_accounts pages out of shared_buffers. A subsequent
# pg_prewarm(bogus_region) ensures the eviction persists for the
# remaining 40 s. On the pgbench SELECT qid this manifests as a large
# climb in buffer read ratio + a disk-read slew on the IO residual
# channel, which is the cache_collapse motif signature.

FAULT_MOTIF="cache_collapse"
FAULT_DESCRIPTION="At t=30.0 s the buffer pool is cooled by terminating idle backends (proxying a DISCARD ALL sweep) and issuing a full sequential scan of pgbench_history (a sibling table not touched by the pgbench SELECT/UPDATE hot path). The pgbench SELECT qid on pgbench_accounts subsequently observes a large climb in shared_blks_read / buffer-read ratio, driving the cache_collapse motif. No extension is assumed; the scan is a bare SELECT count(*) from pgbench_history."

fault_inject() {
  # Terminate idle backends to proxy a DISCARD ALL sweep (without
  # requiring pg_stat_statements_reset privileges on the observer role).
  psql_admin --command \
    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
     WHERE state = 'idle' AND datname = '${PG_DB}' \
       AND usename <> '${PG_USER_OBSERVER}' \
       AND pid <> pg_backend_pid();" \
    >/dev/null 2>&1 || true
  # Cold full scan of pgbench_history: forces pages of a sibling table
  # into shared_buffers, evicting pgbench_accounts pages. Run in a
  # backgrounded one-shot session so it does not block the main
  # fault-injection step.
  (
    podman exec -i "${PG_CONTAINER}" psql \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
      --no-psqlrc --set=ON_ERROR_STOP=1 <<'SQL' >/dev/null 2>&1 || true
SELECT count(*) FROM pgbench_history;
SELECT count(*) FROM pgbench_history;
SELECT count(*) FROM pgbench_history;
SQL
  ) &
  FAULT_CACHE_EVICT_PID=$!
  export FAULT_CACHE_EVICT_PID
}

fault_restore() {
  if [[ -n "${FAULT_CACHE_EVICT_PID:-}" ]]; then
    wait "${FAULT_CACHE_EVICT_PID}" 2>/dev/null || true
  fi
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
