#!/usr/bin/env bash
# DSFB-Database: real-engine evaluation harness (multi-fault).
#
# This script takes a PostgreSQL container, installs
# pg_stat_statements, provisions the least-privilege `dsfb_observer`
# role verbatim from spec/permissions.postgres.sql, runs pgbench
# (scale-10, TPC-B-like) while the live adapter polls the stats
# views, injects one of four *known faults* at t = 30 s, and
# re-collects pg_stat telemetry to t = 70 s. It does this N_REPS
# times per fault class, scores each captured tape against a per-
# replication ground-truth window via the replay_tape_baselines
# binary, and aggregates per-detector mean / stddev / 95 % CI across
# replications.
#
# Fault classes (FAULT env var):
#   drop_constraint  structural plan regression  (default, original)
#   stats_stale      cardinality mismatch from degraded statistics
#   lock_hold        contention ramp from sustained row-level lock
#   cache_evict      cache collapse from buffer-pool eviction
#   all              runs all four sequentially
#
# Container image version is pinned via PG_IMAGE + PG_IMAGE_DIGEST.
# experiments/real_pg_eval/containers.txt enumerates the validated
# (image, digest) pairs.
#
# The script is not CI-wired: requires podman, root SQL privileges
# inside the container, and ≈ 48 min wall-clock for the full
# (4-fault × N_REPS=10) sweep.
#
# Reproducibility discipline: container digest pinned, pgbench
# version captured, crate SHA captured, tape SHA-256 verified on
# load, summary CSV a deterministic function of per-rep bakeoff CSVs.
#
# Usage:
#   bash run.sh                         # FAULT=drop_constraint, N_REPS=10
#   FAULT=stats_stale bash run.sh
#   FAULT=all N_REPS=10 bash run.sh

set -euo pipefail

# --- configuration --------------------------------------------------
CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${OUT_DIR:-${CRATE_DIR}/experiments/real_pg_eval/out}"
PG_IMAGE="${PG_IMAGE:-docker.io/library/postgres:17}"
PG_IMAGE_DIGEST="${PG_IMAGE_DIGEST:-sha256:7ad98329d513dd497293b951c195ca354274a77f12ddbbbbf85e68a811823d72}"
PG_CONTAINER="dsfb-real-pg-eval"
PG_PORT="${PG_PORT:-15434}"
PG_USER_ADMIN="postgres"
PG_PASSWORD_ADMIN="postgres"
PG_USER_OBSERVER="dsfb_observer"
PG_PASSWORD_OBSERVER="observer"
PG_DB="bench"
PGBENCH_SCALE=10
PGBENCH_CLIENTS="${PGBENCH_CLIENTS:-16}"
PGBENCH_JOBS="${PGBENCH_JOBS:-4}"
N_REPS="${N_REPS:-10}"
DURATION_S=70
FAULT_AT_S=30
POLL_INTERVAL_MS=500
FAULT="${FAULT:-drop_constraint}"

FAULTS_DIR="${CRATE_DIR}/experiments/real_pg_eval/faults"

# --- helpers --------------------------------------------------------
log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

psql_admin() {
  if [[ -n "${PSQL_CMD:-}" ]]; then
    ${PSQL_CMD} --host=127.0.0.1 --port="${PG_PORT}" \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
      --no-psqlrc --set=ON_ERROR_STOP=1 "$@"
  else
    podman exec -i "${PG_CONTAINER}" psql \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
      --no-psqlrc --set=ON_ERROR_STOP=1 "$@"
  fi
}

pgbench_cmd() {
  if [[ -n "${PGBENCH_CMD:-}" ]]; then
    ${PGBENCH_CMD} --host=127.0.0.1 --port="${PG_PORT}" \
      --username="${PG_USER_ADMIN}" "$@"
  else
    podman exec "${PG_CONTAINER}" pgbench \
      --username="${PG_USER_ADMIN}" "$@"
  fi
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
log "postgres ready on port ${PG_PORT}; database '${PG_DB}' exists"

# --- 2. extension + dsfb_observer role ------------------------------
psql_admin --command "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"
OBSERVER_SQL="$(sed -e "s/change-me-in-production/${PG_PASSWORD_OBSERVER}/" \
                    "${CRATE_DIR}/spec/permissions.postgres.sql")"
echo "${OBSERVER_SQL}" | psql_admin --file -

# --- 3. pgbench initialisation --------------------------------------
log "pgbench -i -s ${PGBENCH_SCALE} (TPC-B-like)"
pgbench_cmd "${PG_DB}" -i -s "${PGBENCH_SCALE}" -q >/dev/null 2>&1 || true

# --- 4. per-fault run_one -------------------------------------------
CRATE_SHA="$(git -C "${CRATE_DIR}/../.." rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
PGBENCH_VERSION="$(podman exec "${PG_CONTAINER}" pgbench --version 2>&1 | head -1)"
DSFB_BIN="${CRATE_DIR}/target/release/dsfb-database"
REPLAY_BIN="${CRATE_DIR}/target/release/replay_tape_baselines"
CONN_STR="host=127.0.0.1 port=${PG_PORT} user=${PG_USER_OBSERVER} password=${PG_PASSWORD_OBSERVER} dbname=${PG_DB}"

run_one_fault() {
  local fault="$1"
  local fault_script="${FAULTS_DIR}/${fault}.sh"
  if [[ ! -f "${fault_script}" ]]; then
    log "unknown fault: ${fault} (expected ${fault_script})"
    return 1
  fi
  # shellcheck disable=SC1090
  source "${fault_script}"

  local fault_out="${OUT_DIR}/${fault}"
  mkdir -p "${fault_out}"
  {
    echo "container_digest = ${PG_IMAGE}@${PG_IMAGE_DIGEST}"
    echo "pgbench_version  = ${PGBENCH_VERSION}"
    echo "crate_sha        = ${CRATE_SHA}"
    echo "pg_scale         = ${PGBENCH_SCALE}"
    echo "fault_class      = ${fault}"
    echo "fault_motif      = ${FAULT_MOTIF}"
    echo "fault_desc       = ${FAULT_DESCRIPTION}"
    echo "pgbench_clients  = ${PGBENCH_CLIENTS}"
    echo "pgbench_jobs     = ${PGBENCH_JOBS}"
    echo "poll_interval_ms = ${POLL_INTERVAL_MS}"
    echo "duration_s       = ${DURATION_S}"
    echo "n_reps           = ${N_REPS}"
  } > "${fault_out}/provenance.txt"

  for r in $(seq 1 "${N_REPS}"); do
    local rep=$(printf "r%02d" "$r")
    local rep_dir="${fault_out}/${rep}"
    mkdir -p "${rep_dir}"

    log "${fault} ${rep}: pgbench -c ${PGBENCH_CLIENTS} -j ${PGBENCH_JOBS} -T ${DURATION_S}"
    pgbench_cmd "${PG_DB}" -c "${PGBENCH_CLIENTS}" -j "${PGBENCH_JOBS}" \
      -T "${DURATION_S}" -s "${PGBENCH_SCALE}" \
      > "${rep_dir}/pgbench.log" 2>&1 &
    local PGBENCH_PID=$!

    log "${fault} ${rep}: starting dsfb-database live"
    "${DSFB_BIN}" live \
      --conn "${CONN_STR}" \
      --interval-ms "${POLL_INTERVAL_MS}" \
      --max-duration-sec "${DURATION_S}" \
      --tape "${rep_dir}/live.tape.jsonl" \
      --out "${rep_dir}/live" \
      > "${rep_dir}/live.log" 2>&1 &
    local DSFB_PID=$!

    sleep "${FAULT_AT_S}"
    log "${fault} ${rep}: injecting fault"
    fault_inject >> "${rep_dir}/fault.log" 2>&1 || \
      log "${fault} ${rep}: fault_inject failed (see fault.log)"

    wait "${DSFB_PID}" || true
    kill "${PGBENCH_PID}" >/dev/null 2>&1 || true
    wait "${PGBENCH_PID}" >/dev/null 2>&1 || true

    log "${fault} ${rep}: restoring baseline"
    fault_restore

    # Build per-rep ground-truth JSON from the fault's channels.
    log "${fault} ${rep}: resolving ground-truth channels"
    local channels_raw
    channels_raw="$(fault_channels || true)"
    local TAPE_SHA
    TAPE_SHA=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["sha256"])' \
      "${rep_dir}/live.tape.jsonl.hash")

    FAULT="${fault}" FAULT_MOTIF="${FAULT_MOTIF}" \
      FAULT_DESCRIPTION="${FAULT_DESCRIPTION}" \
      CHANNELS_RAW="${channels_raw}" \
      TAPE_SHA="${TAPE_SHA}" \
      GT_OUT="${rep_dir}/ground_truth.json" \
      python3 "${CRATE_DIR}/experiments/real_pg_eval/make_ground_truth.py"

    "${REPLAY_BIN}" \
      --tape "${rep_dir}/live.tape.jsonl" \
      --ground-truth "${rep_dir}/ground_truth.json" \
      --out "${rep_dir}" \
      > "${rep_dir}/bakeoff.log" 2>&1 || \
      log "${fault} ${rep}: bakeoff failed (see bakeoff.log)"
  done
}

# --- 5. dispatch ----------------------------------------------------
if [[ "${FAULT}" == "all" ]]; then
  for f in drop_constraint stats_stale lock_hold cache_evict; do
    log "=== FAULT=${f} ==="
    run_one_fault "${f}"
  done
else
  run_one_fault "${FAULT}"
fi

# --- 6. aggregation --------------------------------------------------
log "aggregating bakeoff CSVs → summary.csv"
export OUT_DIR
python3 "${CRATE_DIR}/experiments/real_pg_eval/aggregate.py"

# --- 7. render table -------------------------------------------------
# Only the PG17 headline sweep (default OUT_DIR) updates the paper
# table. Non-default OUT_DIRs (e.g. PG16 compat sweep → out_pg16)
# are rendered in place by an outer compat script to avoid clobbering
# the headline.
DEFAULT_OUT_DIR="${CRATE_DIR}/experiments/real_pg_eval/out"
if [[ "${OUT_DIR}" == "${DEFAULT_OUT_DIR}" ]]; then
  python3 "${CRATE_DIR}/experiments/real_pg_eval/summary_to_tex.py" \
    "${OUT_DIR}/summary.csv" \
    "${CRATE_DIR}/paper/tables/live_eval_mean_ci.tex"
else
  log "non-default OUT_DIR; skipping paper-table render"
fi

log "done. summary at ${OUT_DIR}/summary.csv"
