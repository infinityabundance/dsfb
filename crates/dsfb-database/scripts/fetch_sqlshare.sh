#!/usr/bin/env bash
# Fetch / prepare the SQLShare ad-hoc human SQL workload (Jain et al.,
# SIGMOD 2016).
#
# Upstream status (verified 2026-04):
#   * The canonical public S3 bucket `shrquerylogs.s3-us-west-2.amazonaws.com`
#     referenced from https://uwescience.github.io/sqlshare/data_release.html
#     returns `NoSuchBucket` \u2014 the bucket has been decommissioned.
#   * The richer 2015 CSV (QueriesWithPlan.csv, carrying user_id,
#     runtime_seconds, submitted_at) is no longer publicly available.
#   * The remaining public artefact is `sqlshare_data_release1.zip`,
#     whose top-level `queries.txt` contains 11,136 raw SQL query texts
#     separated by 40-underscore dividers \u2014 no timing, no user id.
#
# Because the zip is no longer redistributable from a live URL, this
# crate ships the extracted `queries.txt` inside `examples/data/` and
# operates in `sqlshare-text` mode (workload-phase motif only, over
# ordinal-position buckets). That mode is explicitly labelled
# "structural-ordering JSD, not temporal drift" in every emitted stream.
#
# If you obtained `sqlshare_data_release1.zip` from another source
# (e.g. your institutional archive), place it under data/ and this
# script will extract queries.txt from it. Otherwise it simply copies
# the crate-bundled `examples/data/queries.txt` to `data/queries.txt`
# so downstream invocations work identically.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
CRATE="$(cd "${HERE}/.." && pwd)"
DATA_DIR="${CRATE}/data"
mkdir -p "${DATA_DIR}"

BUNDLED_TXT="${CRATE}/examples/data/queries.txt"
DATA_TXT="${DATA_DIR}/queries.txt"
ZIP_PATH="${DATA_DIR}/sqlshare_data_release1.zip"

if [[ -f "${DATA_TXT}" ]]; then
  echo "already extracted: ${DATA_TXT}"
elif [[ -f "${ZIP_PATH}" ]]; then
  echo "extracting queries.txt from ${ZIP_PATH}"
  unzip -j -o "${ZIP_PATH}" "sqlshare_data_release1/queries.txt" -d "${DATA_DIR}"
elif [[ -f "${BUNDLED_TXT}" ]]; then
  echo "using crate-bundled queries.txt (S3 bucket decommissioned)"
  cp "${BUNDLED_TXT}" "${DATA_TXT}"
else
  cat >&2 <<'MSG'
cannot locate sqlshare_data_release1.zip or queries.txt.
The SQLShare upstream S3 bucket shrquerylogs has been decommissioned
(NoSuchBucket as of 2026-04). Obtain the zip from an alternative source
(e.g. your institutional archive) and place it at data/sqlshare_data_release1.zip,
then re-run this script.
MSG
  exit 1
fi

sha256sum "${DATA_TXT}"
echo "OK: SQLShare text corpus ready at ${DATA_TXT}"
echo "Run: ./target/release/dsfb-database run --dataset sqlshare-text --path ${DATA_TXT}"
