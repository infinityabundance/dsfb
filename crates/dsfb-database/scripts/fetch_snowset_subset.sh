#!/usr/bin/env bash
# Fetch a small Snowset subset.
#
# Snowset is the workload trace published with Vuppalapati et al.,
# "Building an Elastic Query Engine on Disaggregated Storage", NSDI 2020.
# Full-trace sizes are 7-30 GB; for the dsfb-database paper we fetch the
# first per-day Parquet shard (~50 MB) which is enough to demonstrate the
# workload-phase and resource motifs. Update SNOWSET_SHARD_URL below if
# you want a different shard.
#
# Usage:  ./scripts/fetch_snowset_subset.sh
# Output: data/snowset_shard.parquet (and a CSV mirror data/snowset_shard.csv)

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

# The Snowset GitHub repo lists the canonical S3 prefix; the README
# explains the schema. We mirror the URL here so this script remains
# auditable without a live network handshake at script-read time.
SNOWSET_BASE="https://snowset-trace.s3.us-west-2.amazonaws.com"
SNOWSET_SHARD="2018-02-21.parquet"

echo "Snowset URL:  ${SNOWSET_BASE}/${SNOWSET_SHARD}"
echo "Snowset license: CC-BY 4.0 (https://github.com/resource-disaggregation/snowset)"
echo "If your workstation has no network access, place a copy at"
echo "  ${DATA_DIR}/snowset_shard.parquet"
echo "and skip this script."

if [[ ! -f "${DATA_DIR}/snowset_shard.parquet" ]]; then
  curl --fail -L -o "${DATA_DIR}/snowset_shard.parquet" \
    "${SNOWSET_BASE}/${SNOWSET_SHARD}"
fi

# DuckDB does the parquet -> csv conversion in-process (the dsfb-database
# adapter expects a CSV).
duckdb -c "COPY (SELECT * FROM '${DATA_DIR}/snowset_shard.parquet') TO '${DATA_DIR}/snowset_shard.csv' (HEADER, DELIMITER ',');"

sha256sum "${DATA_DIR}/snowset_shard.parquet" "${DATA_DIR}/snowset_shard.csv"
echo "OK: snowset subset ready at ${DATA_DIR}/snowset_shard.csv"
