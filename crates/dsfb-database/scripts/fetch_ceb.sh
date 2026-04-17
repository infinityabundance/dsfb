#!/usr/bin/env bash
# Fetch CEB (Cardinality Estimation Benchmark, Negi et al.).
#
# CEB ships PostgreSQL true / estimated cardinalities as Python pickles
# under https://github.com/learnedsystems/CEB (MIT license). The
# dsfb-database CebAdapter expects a CSV with columns:
#   query_id, subplan_id, true_rows, est_rows
# This script clones the repo, installs CEB locally, and runs a small
# Python helper that exports the IMDb subset to CSV.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

CEB_REPO="https://github.com/learnedsystems/CEB.git"
CEB_DIR="${DATA_DIR}/CEB"

if [[ ! -d "${CEB_DIR}" ]]; then
  git clone --depth 1 "${CEB_REPO}" "${CEB_DIR}"
fi

# CEB ships the pickles under queries/imdb/. Use a tiny inline Python
# script — keeps this fetch self-contained.
python3 - <<'PY' "${CEB_DIR}" "${DATA_DIR}/ceb_subset.csv"
import os, sys, glob, pickle, csv
ceb_root = sys.argv[1]
out_csv  = sys.argv[2]
pickles  = sorted(glob.glob(os.path.join(ceb_root, "queries", "imdb", "*.pkl")))[:200]
with open(out_csv, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["query_id", "subplan_id", "true_rows", "est_rows"])
    for qpath in pickles:
        qid = os.path.splitext(os.path.basename(qpath))[0]
        with open(qpath, "rb") as fp:
            q = pickle.load(fp)
        # Each query pickle yields (subplan_id, true, est) triples.
        for sp_id, true, est in q.get("subplans", []):
            w.writerow([qid, sp_id, true, est])
print(f"wrote {out_csv}")
PY

sha256sum "${DATA_DIR}/ceb_subset.csv"
echo "OK: CEB subset ready at ${DATA_DIR}/ceb_subset.csv"
