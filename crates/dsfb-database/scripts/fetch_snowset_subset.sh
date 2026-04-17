#!/usr/bin/env bash
# Fetch a Snowset subset.
#
# Snowset is the workload trace published with Vuppalapati et al.,
# "Building an Elastic Query Engine on Disaggregated Storage", NSDI 2020
# (CC-BY 4.0, https://github.com/resource-disaggregation/snowset).
#
# The original S3 bucket `snowset-trace.s3.us-west-2.amazonaws.com` that
# the Snowset GitHub README used to point at has been decommissioned.
# The live, canonical public mirror (maintained by Midhul Vuppalapati at
# Cornell, same author) is:
#
#   http://www.cs.cornell.edu/~midhul/snowset/snowset-main.csv.gz
#
# The full file is ~7.5 GB gzipped. Downloading and replaying 70 million
# queries is unnecessary for the dsfb-database demo: we stream the first
# `SUBSET_ROWS` rows (header-preserving) into a local CSV that is large
# enough to exhibit per-warehouse phase transitions, cache-miss drift,
# and plan-latency regression, and small enough to analyse in a few
# seconds. Bump `SUBSET_ROWS` if you want a larger shard.
#
# Usage:  ./scripts/fetch_snowset_subset.sh
# Output: data/snowset-main.csv.gz  (full dump, cached)
#         data/snowset_shard.csv    (subset consumed by the adapter)

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

SNOWSET_URL="http://www.cs.cornell.edu/~midhul/snowset/snowset-main.csv.gz"
SNOWSET_GZ="${DATA_DIR}/snowset-main.csv.gz"
SNOWSET_CSV="${DATA_DIR}/snowset_shard.csv"
SUBSET_ROWS="${SUBSET_ROWS:-200000}"

echo "Snowset URL:     ${SNOWSET_URL}"
echo "Snowset license: CC-BY 4.0"

if [[ ! -f "${SNOWSET_GZ}" ]]; then
  echo "downloading full Snowset dump (~7.5 GB gzipped, resumable)"
  curl -L -C - -o "${SNOWSET_GZ}" "${SNOWSET_URL}"
fi

if [[ ! -f "${SNOWSET_CSV}" ]]; then
  echo "extracting first ${SUBSET_ROWS} rows + header into ${SNOWSET_CSV}"
  # +1 for the header line.
  zcat "${SNOWSET_GZ}" | head -n "$((SUBSET_ROWS + 1))" > "${SNOWSET_CSV}"
fi

sha256sum "${SNOWSET_CSV}"
echo "OK: snowset subset ready at ${SNOWSET_CSV}"
