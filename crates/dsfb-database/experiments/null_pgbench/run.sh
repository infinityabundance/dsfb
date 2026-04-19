#!/usr/bin/env bash
# Pass-2 N3: null pgbench harness — no planted fault, all-detector
# false-alarm rate measurement.
#
# This is the *baseline-against-which-cross-firing-is-measured* tape
# capture. The §13 prose names DSFB's contention_ramp false-positives
# during the DROP CONSTRAINT cascade; that observation is meaningful
# only relative to a known-quiet tape. This script captures that
# quiet tape — pgbench at the same shape (-c 16 -j 4 -T 90) as
# real_pg_eval, with the live observer running, but with no fault
# injection at any point.
#
# Output: experiments/null_pgbench/out/r{01..NN}/{live.tape.jsonl,
#         live.tape.jsonl.hash, ground_truth.json, bakeoff.csv}
#         experiments/null_pgbench/out/summary_far.csv
#
# Cited by paper §44 (adversarial workload bound) and the §36
# cross-firing paragraph as the no-fault FAR floor.
#
# Pre-requisites:
#   * podman with rootless support
#   * Python 3
#   * Cargo workspace builds with --features "cli report live-postgres"
#
# Live capture is non-deterministic by design (per non-claim #7);
# replay of the captured tape is. The summary FAR/hr therefore has
# a CI from across replications, not from re-runs of the same tape.

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${OUT_DIR:-${CRATE_DIR}/experiments/null_pgbench/out}"
PG_IMAGE="${PG_IMAGE:-docker.io/library/postgres:17}"
PG_IMAGE_DIGEST="${PG_IMAGE_DIGEST:-sha256:7ad98329d513dd497293b951c195ca354274a77f12ddbbbbf85e68a811823d72}"
PG_CONTAINER="dsfb-null-pgbench"
PG_PORT="${PG_PORT:-15435}"
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

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }
psql_admin() {
  podman exec -i "${PG_CONTAINER}" psql \
    --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" \
    --no-psqlrc --set=ON_ERROR_STOP=1 "$@"
}
pgbench_cmd() {
  podman exec "${PG_CONTAINER}" pgbench \
    --username="${PG_USER_ADMIN}" "$@"
}

cleanup() {
  log "stopping container ${PG_CONTAINER}"
  podman stop --time 1 "${PG_CONTAINER}" >/dev/null 2>&1 || true
  podman rm --force "${PG_CONTAINER}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

mkdir -p "${OUT_DIR}"

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

for _ in $(seq 1 120); do
  if podman exec "${PG_CONTAINER}" pg_isready \
      --username="${PG_USER_ADMIN}" --dbname="${PG_DB}" >/dev/null 2>&1; then
    break
  fi
  sleep 0.5
done
log "postgres ready on port ${PG_PORT}"

psql_admin --command "CREATE EXTENSION IF NOT EXISTS pg_stat_statements;"
OBSERVER_SQL="$(sed -e "s/change-me-in-production/${PG_PASSWORD_OBSERVER}/" \
                    "${CRATE_DIR}/spec/permissions.postgres.sql")"
echo "${OBSERVER_SQL}" | psql_admin --file -

log "pgbench -i -s ${PGBENCH_SCALE}"
pgbench_cmd "${PG_DB}" -i -s "${PGBENCH_SCALE}" -q >/dev/null 2>&1 || true

cd "${CRATE_DIR}"
cargo build --release --features "cli report live-postgres" --quiet

DSFB_BIN="${CRATE_DIR}/target/release/dsfb-database"
REPLAY_BIN="${CRATE_DIR}/target/release/replay_tape_baselines"
CONN_STR="host=127.0.0.1 port=${PG_PORT} user=${PG_USER_OBSERVER} password=${PG_PASSWORD_OBSERVER} dbname=${PG_DB}"

CRATE_SHA="$(git -C "${CRATE_DIR}/../.." rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
{
  echo "container_digest = ${PG_IMAGE}@${PG_IMAGE_DIGEST}"
  echo "crate_sha        = ${CRATE_SHA}"
  echo "pgbench_clients  = ${PGBENCH_CLIENTS}"
  echo "pgbench_jobs     = ${PGBENCH_JOBS}"
  echo "duration_s       = ${DURATION_S}"
  echo "poll_interval_ms = ${POLL_INTERVAL_MS}"
  echo "n_reps           = ${N_REPS}"
  echo "fault_class      = none"
} > "${OUT_DIR}/provenance.txt"

for r in $(seq 1 "${N_REPS}"); do
  rep=$(printf "r%02d" "$r")
  rep_dir="${OUT_DIR}/${rep}"
  mkdir -p "${rep_dir}"

  log "${rep}: pgbench -c ${PGBENCH_CLIENTS} -j ${PGBENCH_JOBS} -T ${DURATION_S}"
  pgbench_cmd "${PG_DB}" -c "${PGBENCH_CLIENTS}" -j "${PGBENCH_JOBS}" \
    -T "${DURATION_S}" -s "${PGBENCH_SCALE}" \
    > "${rep_dir}/pgbench.log" 2>&1 &
  PGBENCH_PID=$!

  log "${rep}: starting dsfb-database live"
  "${DSFB_BIN}" live \
    --conn "${CONN_STR}" \
    --interval-ms "${POLL_INTERVAL_MS}" \
    --max-duration-sec "${DURATION_S}" \
    --tape "${rep_dir}/live.tape.jsonl" \
    --out "${rep_dir}/live" \
    > "${rep_dir}/live.log" 2>&1 &
  DSFB_PID=$!

  wait "${DSFB_PID}" || true
  kill "${PGBENCH_PID}" >/dev/null 2>&1 || true
  wait "${PGBENCH_PID}" >/dev/null 2>&1 || true

  TAPE_SHA=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["sha256"])' \
    "${rep_dir}/live.tape.jsonl.hash")

  cat > "${rep_dir}/ground_truth.json" <<EOF
{
  "tape_sha256": "${TAPE_SHA}",
  "fault_description": "no fault planted; pgbench-only baseline",
  "windows": [],
  "notes": "Empty windows: every detector emission is a false alarm by construction."
}
EOF

  "${REPLAY_BIN}" \
    --tape "${rep_dir}/live.tape.jsonl" \
    --ground-truth "${rep_dir}/ground_truth.json" \
    --out "${rep_dir}" \
    > "${rep_dir}/bakeoff.log" 2>&1 || \
    log "${rep}: bakeoff failed (see bakeoff.log)"
done

log "aggregating no-fault FAR/hr → summary_far.csv"
python3 "${CRATE_DIR}/experiments/null_pgbench/aggregate.py" \
  --in "${OUT_DIR}" \
  --out "${OUT_DIR}/summary_far.csv"

log "done. summary at ${OUT_DIR}/summary_far.csv"
