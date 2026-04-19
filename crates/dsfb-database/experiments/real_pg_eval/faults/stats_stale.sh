# Fault: statistics staleness → cardinality mismatch.
#
# Prior to the replication, we dirty the pg_statistic samples for
# pgbench_accounts by executing a large asymmetric UPDATE that rewrites
# half the rows (abalance on even aid). At t=30s we kill planner
# accuracy harder by setting the column statistics target to 1 and
# re-running ANALYZE, then issuing an aggressive UPDATE that produces
# row-count variance the planner cannot estimate. The cardinality-
# mismatch motif fires on qids whose observed row-counts diverge from
# the stored pg_class.reltuples estimate under sustained load.

FAULT_MOTIF="cardinality_mismatch_regime"
FAULT_DESCRIPTION="At t=30.0 s under c=16 j=4 pgbench scale-10 load, column statistics on pgbench_accounts were degraded to statistics_target=1 and ANALYZE was re-run, then a bulk asymmetric UPDATE on pgbench_accounts was issued. The planner's row-count estimates diverge from the observed cardinality on the two pgbench account qids, driving the cardinality_mismatch_regime motif."

fault_inject() {
  psql_admin --command \
    "ALTER TABLE pgbench_accounts ALTER COLUMN aid SET STATISTICS 1; \
     ALTER TABLE pgbench_accounts ALTER COLUMN abalance SET STATISTICS 1; \
     ANALYZE pgbench_accounts; \
     UPDATE pgbench_accounts SET abalance = abalance + 1 WHERE aid % 2 = 0;"
}

fault_restore() {
  psql_admin --command \
    "ALTER TABLE pgbench_accounts ALTER COLUMN aid SET STATISTICS -1; \
     ALTER TABLE pgbench_accounts ALTER COLUMN abalance SET STATISTICS -1; \
     ANALYZE pgbench_accounts;" \
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
  podman exec "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" --no-psqlrc \
    --tuples-only --no-align --command \
    "SELECT md5(queryid::text) FROM pg_stat_statements \
     WHERE query LIKE 'SELECT abalance FROM pgbench_accounts WHERE aid = %' \
     ORDER BY calls DESC LIMIT 1;" 2>/dev/null | tr -d '[:space:]'
  echo
}
