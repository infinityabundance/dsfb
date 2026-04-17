#!/usr/bin/env bash
# Fetch the SQLShare ad-hoc human SQL workload (Jain et al., SIGMOD 2016).
#
# SQLShare is published as a CSV bundle at https://uwescience.github.io/sqlshare/data_release.html
# under CC-BY 4.0. We grab the small "queries.csv" dump (~30 MB) which is
# the only file the dsfb-database SqlShare adapter reads.
#
# Usage:  ./scripts/fetch_sqlshare.sh

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

SQLSHARE_URL="https://homes.cs.washington.edu/~bhowe/sqlshare/queries.csv"

echo "SQLShare URL:    ${SQLSHARE_URL}"
echo "SQLShare license: CC-BY 4.0"
echo "If your workstation has no network access, place a copy at"
echo "  ${DATA_DIR}/sqlshare_queries.csv"
echo "and skip this script."

if [[ ! -f "${DATA_DIR}/sqlshare_queries.csv" ]]; then
  curl --fail -L -o "${DATA_DIR}/sqlshare_queries.csv" "${SQLSHARE_URL}"
fi

sha256sum "${DATA_DIR}/sqlshare_queries.csv"
echo "OK: SQLShare subset ready at ${DATA_DIR}/sqlshare_queries.csv"
