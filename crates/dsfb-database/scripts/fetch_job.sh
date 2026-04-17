#!/usr/bin/env bash
# Fetch the Join Order Benchmark queries (Leis et al., VLDB 2015).
#
# JOB ships 113 .sql files under https://github.com/gregrahn/join-order-benchmark
# (MIT license). The dsfb-database JobAdapter executes them via DuckDB's
# IMDb extension and writes a CSV with EXPLAIN ANALYZE rows.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"
JOB_REPO="https://github.com/gregrahn/join-order-benchmark.git"
JOB_DIR="${DATA_DIR}/job"

if [[ ! -d "${JOB_DIR}" ]]; then
  git clone --depth 1 "${JOB_REPO}" "${JOB_DIR}"
fi

# Build a CSV manifest of the 113 queries; the adapter consumes this
# manifest and runs each query against the IMDb sample DuckDB ships.
python3 - <<'PY' "${JOB_DIR}" "${DATA_DIR}/job_manifest.csv"
import os, sys, csv, glob
job_root = sys.argv[1]
out_csv  = sys.argv[2]
queries  = sorted(glob.glob(os.path.join(job_root, "*.sql")))
with open(out_csv, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["query_id", "sql_path"])
    for qpath in queries:
        qid = os.path.splitext(os.path.basename(qpath))[0]
        w.writerow([qid, qpath])
print(f"wrote {out_csv} ({len(queries)} queries)")
PY

sha256sum "${DATA_DIR}/job_manifest.csv"
echo "OK: JOB manifest ready at ${DATA_DIR}/job_manifest.csv"
