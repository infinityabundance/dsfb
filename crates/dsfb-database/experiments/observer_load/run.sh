#!/usr/bin/env bash
# DSFB-Database: observer self-load measurement.
#
# Measures the per-transaction latency impact of running
# `dsfb-database live` against a PostgreSQL 17 container while a
# pgbench workload executes. Two conditions, N_REPS replications each:
#
#   without_scraper  pgbench alone at c=16 j=4 T=90
#   with_scraper     identical pgbench + dsfb-database live at
#                    --interval-ms 500 in the same container network
#
# Each replication captures pgbench's per-transaction log
# (pgbench -l). The log is parsed for p50 / p95 / p99 / p99.9 of
# transaction latency in microseconds. A bootstrap 95% CI on the
# per-replication percentile is reported across N_REPS.
#
# The script is not CI-wired: it requires podman, ~18 min of
# wall-clock time, and the release dsfb-database binary to be built
# with --features "cli report live-postgres".
#
# Usage: bash run.sh   (uses defaults; N_REPS=5, T=90)

set -euo pipefail

# --- configuration --------------------------------------------------
CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${CRATE_DIR}/experiments/observer_load/out"
PG_IMAGE="docker.io/library/postgres:17"
PG_IMAGE_DIGEST="${PG_IMAGE_DIGEST:-sha256:7ad98329d513dd497293b951c195ca354274a77f12ddbbbbf85e68a811823d72}"
PG_CONTAINER="dsfb-observer-load"
PG_PORT="${PG_PORT:-15436}"
PG_USER_ADMIN="postgres"
PG_PASSWORD_ADMIN="postgres"
PG_USER_OBSERVER="dsfb_observer"
PG_PASSWORD_OBSERVER="observer"
PG_DB="bench"
PGBENCH_SCALE=10
PGBENCH_CLIENTS="${PGBENCH_CLIENTS:-16}"
PGBENCH_JOBS="${PGBENCH_JOBS:-4}"
N_REPS="${N_REPS:-5}"
DURATION_S="${DURATION_S:-90}"
POLL_INTERVAL_MS=500

# --- helpers --------------------------------------------------------
log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

psql_admin() {
  podman exec -i "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
    --no-psqlrc --set=ON_ERROR_STOP=1 "$@"
}

cleanup() {
  log "stopping container ${PG_CONTAINER}"
  podman stop --time 1 "${PG_CONTAINER}" >/dev/null 2>&1 || true
  podman rm --force "${PG_CONTAINER}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

mkdir -p "${OUT_DIR}"

# --- 1. container ---------------------------------------------------
log "pulling ${PG_IMAGE}@${PG_IMAGE_DIGEST}"
podman pull "${PG_IMAGE}@${PG_IMAGE_DIGEST}" >/dev/null

log "starting container ${PG_CONTAINER} on port ${PG_PORT}"
podman rm --force "${PG_CONTAINER}" >/dev/null 2>&1 || true
podman run --detach \
  --name "${PG_CONTAINER}" \
  --publish "127.0.0.1:${PG_PORT}:5432" \
  --env POSTGRES_PASSWORD="${PG_PASSWORD_ADMIN}" \
  --env POSTGRES_DB="${PG_DB}" \
  "${PG_IMAGE}@${PG_IMAGE_DIGEST}" \
  postgres -c shared_preload_libraries=pg_stat_statements \
           -c pg_stat_statements.track=all \
           -c pg_stat_statements.max=5000 \
           -c track_io_timing=on \
  >/dev/null

for i in $(seq 1 120); do
  if podman exec "${PG_CONTAINER}" pg_isready \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" >/dev/null 2>&1 \
    && podman exec "${PG_CONTAINER}" psql \
      --username="${PG_USER_ADMIN}" --dbname=postgres --no-psqlrc \
      --tuples-only --command \
      "SELECT 1 FROM pg_database WHERE datname='${PG_DB}';" 2>/dev/null \
      | grep -q '1'; then
    break
  fi
  sleep 0.5
done
log "postgres ready on port ${PG_PORT}"

# --- 2. extension + observer role -----------------------------------
psql_admin --command "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"
OBSERVER_SQL="$(sed -e "s/change-me-in-production/${PG_PASSWORD_OBSERVER}/" \
                    "${CRATE_DIR}/spec/permissions.postgres.sql")"
echo "${OBSERVER_SQL}" | psql_admin --file -

# --- 3. pgbench init ------------------------------------------------
log "pgbench -i -s ${PGBENCH_SCALE}"
podman exec "${PG_CONTAINER}" pgbench \
  --username="${PG_USER_ADMIN}" "${PG_DB}" -i -s "${PGBENCH_SCALE}" -q \
  >/dev/null 2>&1 || true

# --- 4. provenance --------------------------------------------------
CRATE_SHA="$(git -C "${CRATE_DIR}/../.." rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
PGBENCH_VERSION="$(podman exec "${PG_CONTAINER}" pgbench --version 2>&1 | head -1)"
{
  echo "container_digest = ${PG_IMAGE}@${PG_IMAGE_DIGEST}"
  echo "pgbench_version  = ${PGBENCH_VERSION}"
  echo "crate_sha        = ${CRATE_SHA}"
  echo "pgbench_clients  = ${PGBENCH_CLIENTS}"
  echo "pgbench_jobs     = ${PGBENCH_JOBS}"
  echo "duration_s       = ${DURATION_S}"
  echo "n_reps           = ${N_REPS}"
  echo "poll_interval_ms = ${POLL_INTERVAL_MS}"
} > "${OUT_DIR}/provenance.txt"

DSFB_BIN="${CRATE_DIR}/target/release/dsfb-database"
CONN_STR="host=127.0.0.1 port=${PG_PORT} user=${PG_USER_OBSERVER} password=${PG_PASSWORD_OBSERVER} dbname=${PG_DB}"

CSV="${OUT_DIR}/pgbench_latency_deltas.csv"
echo "condition,rep,n_tx,p50_us,p95_us,p99_us,p99_9_us" > "${CSV}"

# --- 5. replications ------------------------------------------------
run_rep() {
  local condition="$1"
  local rep="$2"
  local rep_dir="${OUT_DIR}/${condition}_r${rep}"
  mkdir -p "${rep_dir}"

  log "${condition} r${rep}: pgbench -c ${PGBENCH_CLIENTS} -j ${PGBENCH_JOBS} -T ${DURATION_S} -l"
  podman exec "${PG_CONTAINER}" bash -c \
    "cd /tmp && rm -f pgbench_log.* && pgbench \
      --username=${PG_USER_ADMIN} \
      -c ${PGBENCH_CLIENTS} -j ${PGBENCH_JOBS} \
      -T ${DURATION_S} -P 2 -l -s ${PGBENCH_SCALE} \
      ${PG_DB}" > "${rep_dir}/pgbench.log" 2>&1 &
  local PGBENCH_PID=$!

  if [[ "${condition}" == "with_scraper" ]]; then
    log "${condition} r${rep}: starting dsfb-database live"
    "${DSFB_BIN}" live \
      --conn "${CONN_STR}" \
      --interval-ms "${POLL_INTERVAL_MS}" \
      --max-duration-sec "${DURATION_S}" \
      --tape "${rep_dir}/live.tape.jsonl" \
      --out "${rep_dir}/live" \
      > "${rep_dir}/live.log" 2>&1 &
    local DSFB_PID=$!
    wait "${DSFB_PID}" || true
  fi

  wait "${PGBENCH_PID}" || true

  podman exec "${PG_CONTAINER}" bash -c "cat /tmp/pgbench_log.*" \
    > "${rep_dir}/tx_log.raw" 2>/dev/null || true

  python3 "${CRATE_DIR}/experiments/observer_load/parse_pgbench_log.py" \
    "${rep_dir}/tx_log.raw" "${condition}" "${rep}" >> "${CSV}"
}

for c in without_scraper with_scraper; do
  for r in $(seq 1 "${N_REPS}"); do
    run_rep "${c}" "${r}"
  done
done

# --- 6. render table + figure ---------------------------------------
python3 "${CRATE_DIR}/experiments/observer_load/to_tex.py" \
  "${CSV}" \
  "${CRATE_DIR}/paper/tables/observer_self_load.tex" \
  "${CRATE_DIR}/paper/figs/observer_self_load_cdf.png"

log "done. deltas CSV: ${CSV}"
