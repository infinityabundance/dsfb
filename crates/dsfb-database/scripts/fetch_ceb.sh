#!/usr/bin/env bash
# Fetch CEB (Cardinality Estimation Benchmark, Negi et al.).
#
# CEB's GitHub repo (https://github.com/learnedsystems/CEB, MIT) ships only
# four example-query pickles; the full ~3k-query CEB-IMDb workload is
# distributed as a Dropbox tarball pulled by the upstream helper
# `scripts/download_imdb_workload.sh`. This script clones the repo if
# absent, runs that helper (from inside the clone so its relative
# `queries/` paths work), and then exports `(query_id, subplan_id,
# true_rows, est_rows)` rows from every pickle into a single CSV that
# matches the dsfb-database CebAdapter schema.
#
# The pickle schema (verified 2026-04) is a dict whose `subset_graph`
# holds `nodes: [{ cardinality: {actual, expected}, id: (alias, ...) }]`
# — the subplan id is the sorted-tuple of table aliases.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

CEB_REPO="https://github.com/learnedsystems/CEB.git"
CEB_DIR="${DATA_DIR}/CEB"

if [[ ! -d "${CEB_DIR}" ]]; then
  git clone --depth 1 "${CEB_REPO}" "${CEB_DIR}"
fi

# Pull the full workload if not already present. The upstream helper
# writes into $PWD/queries, so run it from inside the clone.
if [[ ! -d "${CEB_DIR}/queries/imdb" ]]; then
  (cd "${CEB_DIR}" && bash scripts/download_imdb_workload.sh)
  # The helper names the extracted directory `ceb-imdb` (3k-query
  # subset). Relocate to `imdb/` to match this adapter's expected path.
  if [[ -d "${CEB_DIR}/queries/ceb-imdb" ]]; then
    mv "${CEB_DIR}/queries/ceb-imdb" "${CEB_DIR}/queries/imdb"
  fi
fi

python3 - <<'PY' "${CEB_DIR}" "${DATA_DIR}/ceb_subset.csv"
import os, sys, glob, pickle, csv
ceb_root = sys.argv[1]
out_csv  = sys.argv[2]
pickles  = sorted(glob.glob(os.path.join(ceb_root, "queries", "imdb", "**", "*.pkl"), recursive=True))[:400]
rows = 0
with open(out_csv, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["query_id", "subplan_id", "true_rows", "est_rows"])
    for qpath in pickles:
        qid = os.path.splitext(os.path.basename(qpath))[0]
        try:
            with open(qpath, "rb") as fp:
                q = pickle.load(fp)
        except Exception as e:
            sys.stderr.write(f"skip {qpath}: {e}\n")
            continue
        sg = q.get("subset_graph") or {}
        for node in sg.get("nodes") or []:
            card = node.get("cardinality") or {}
            true_rows = card.get("actual")
            est_rows  = card.get("expected")
            if true_rows is None or est_rows is None:
                continue
            sp_id = "_".join(sorted(node.get("id") or ()))
            w.writerow([qid, sp_id, int(true_rows), int(est_rows)])
            rows += 1
print(f"wrote {rows} rows from {len(pickles)} pickles to {out_csv}")
PY

sha256sum "${DATA_DIR}/ceb_subset.csv"
echo "OK: CEB subset ready at ${DATA_DIR}/ceb_subset.csv"
