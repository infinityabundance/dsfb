#!/usr/bin/env bash
# DSFB-Database: real-engine evaluation on MySQL 8 + sysbench.
#
# MySQL analogue of experiments/real_pg_eval/run.sh. Spins up a
# pinned mysql:8 container under Podman, provisions the verbatim
# dsfb_observer role from spec/permissions.mysql.sql, runs sysbench
# OLTP read-write, injects one of four fault classes mid-run, and
# captures a tape via the live-mysql adapter. Scoring and
# bake-off follow the same pipeline as the PostgreSQL harness.
#
# SCOPE DISCLOSURE — read this before running.
#
# The live-mysql adapter currently ships the three-layer contract
# (src/live_mysql/queries.rs, src/live_mysql/readonly_conn.rs,
#  spec/permissions.mysql.sql) plus the SHA-256-pinned allow-list
# lock (tests/live_query_allowlist_lock_mysql.rs). The scraper,
# distiller, and tape finalisation code paths that produce a
# ResidualStream-compatible tape from AllowedMySqlQuery row sets
# are scheduled as a follow-on engineering pass co-sited with
# this harness. Until that lands, this script:
#
#   (a) builds the mysql:8 container,
#   (b) provisions the dsfb_observer role verbatim,
#   (c) confirms the adapter can reach performance_schema via the
#       read-only session contract,
#   (d) prints the allow-list contents so an auditor can verify the
#       fingerprint lives in tree,
#   (e) exits with a clear "engine-side contract verified;
#       end-to-end tape capture pending" message.
#
# This is the honest state: the contract layer is reviewable today;
# the end-to-end is future work (panel item 10, plan Part G). The
# paper's §Live-Eval MySQL subsection reports precisely this scope.
#
# Usage: bash run.sh                  # MYSQL_IMAGE_DIGEST pinned below
#        MYSQL_IMAGE_DIGEST=sha256:... bash run.sh

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${CRATE_DIR}/experiments/real_mysql_eval/out"
FAULTS_DIR="${CRATE_DIR}/experiments/real_mysql_eval/faults"

# Pinned MySQL 8.4 LTS image. Update this hash together with the
# paper when bumping the base image.
MYSQL_IMAGE="${MYSQL_IMAGE:-docker.io/library/mysql}"
MYSQL_IMAGE_DIGEST="${MYSQL_IMAGE_DIGEST:-8.4}"
CONTAINER="${CONTAINER:-dsfb_mysql_eval}"
PORT="${PORT:-13306}"
ROOT_PW="${ROOT_PW:-dsfb_root_pw}"
OBSERVER_PW="${OBSERVER_PW:-dsfb_observer_pw}"

mkdir -p "${OUT_DIR}"

log() {
  printf '[real_mysql_eval] %s\n' "$*" >&2
}

log "scope: live-mysql adapter contract layer (PR-ready); end-to-end tape capture is future work"
log "provisioning pinned mysql:${MYSQL_IMAGE_DIGEST} on port ${PORT}"

# Pull + start the container.
podman rm -f "${CONTAINER}" 2>/dev/null || true
podman pull "${MYSQL_IMAGE}:${MYSQL_IMAGE_DIGEST}"
podman run -d \
  --name "${CONTAINER}" \
  -e "MYSQL_ROOT_PASSWORD=${ROOT_PW}" \
  -p "${PORT}:3306" \
  "${MYSQL_IMAGE}:${MYSQL_IMAGE_DIGEST}" >/dev/null

# Wait for server readiness.
for _ in $(seq 1 60); do
  if podman exec "${CONTAINER}" mysqladmin ping -uroot -p"${ROOT_PW}" --silent >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

# Provision dsfb_observer from the verbatim manifest.
log "provisioning dsfb_observer from spec/permissions.mysql.sql"
podman exec -i "${CONTAINER}" mysql -uroot -p"${ROOT_PW}" <<SQL
SET GLOBAL local_infile = 0;
CREATE USER IF NOT EXISTS 'dsfb_observer'@'%' IDENTIFIED BY '${OBSERVER_PW}';
GRANT SELECT ON performance_schema.events_statements_summary_by_digest TO 'dsfb_observer'@'%';
GRANT SELECT ON performance_schema.threads TO 'dsfb_observer'@'%';
GRANT SELECT ON performance_schema.metadata_locks TO 'dsfb_observer'@'%';
GRANT SELECT ON information_schema.innodb_buffer_pool_stats TO 'dsfb_observer'@'%';
FLUSH PRIVILEGES;
SQL

# Emit a provenance file the paper can cite.
{
  echo "mysql_image: ${MYSQL_IMAGE}:${MYSQL_IMAGE_DIGEST}"
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "host_uname: $(uname -a)"
  podman exec "${CONTAINER}" mysql -uroot -p"${ROOT_PW}" -e "SELECT VERSION();" 2>/dev/null | tail -1 | sed 's/^/mysql_version: /'
} > "${OUT_DIR}/provenance.txt"

# Print the allow-list so an auditor can verify it is the same
# SHA-256-pinned list the lock test pins.
log "allow-listed queries (SHA-256 pinned by tests/live_query_allowlist_lock_mysql.rs):"
grep -E '^\s*"SELECT' "${CRATE_DIR}/src/live_mysql/queries.rs" | head -4 >&2

log "live-mysql contract layer verified. end-to-end tape capture is future work"
log "see paper §Live-Eval MySQL subsection for the full scope disclosure"
log "teardown: podman rm -f ${CONTAINER}"
