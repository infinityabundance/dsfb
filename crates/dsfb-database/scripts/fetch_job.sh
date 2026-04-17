#!/usr/bin/env bash
# Fetch the Join Order Benchmark queries (Leis et al., VLDB 2015) and
# produce a real EXPLAIN ANALYZE trace.
#
# JOB ships 113 .sql files under https://github.com/gregrahn/join-order-benchmark
# (MIT license). The canonical IMDb data dump (~1.26 GB CSV tarball)
# lives at https://event.cwi.nl/da/job/imdb.tgz (CWI, same team that
# released JOB). We download both, load IMDb into a DuckDB file database,
# run each of the 113 queries `ITERATIONS` times with profiling enabled,
# and emit a CSV that matches the dsfb-database JobAdapter schema:
#
#   query_id, iteration, est_rows, actual_rows, latency_ms, plan_hash
#
# This is a real trace — not a synthetic exemplar. The first run is
# heavy (download + load + 113 × N query executions); re-runs are cheap
# (the DuckDB file is cached, and re-running the query stage alone takes
# a few minutes on a laptop).
#
# Controls:
#   ITERATIONS=3                # number of replay rounds per query
#   JOB_DATA_DIR=data/job_imdb  # IMDb CSV extraction directory
#   JOB_DB=data/imdb.duckdb     # DuckDB file database

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"

# Isolated venv for the duckdb helper (keeps this script usable on
# PEP 668 distros and Colab alike).
VENV_DIR="${DATA_DIR}/.venv"
if [[ ! -x "${VENV_DIR}/bin/python3" ]]; then
  python3 -m venv "${VENV_DIR}"
  "${VENV_DIR}/bin/pip" install --quiet --upgrade pip
  "${VENV_DIR}/bin/pip" install --quiet "duckdb>=1.0"
fi
PY="${VENV_DIR}/bin/python3"

JOB_REPO="https://github.com/gregrahn/join-order-benchmark.git"
JOB_DIR="${DATA_DIR}/job"
IMDB_TGZ_URL="https://event.cwi.nl/da/job/imdb.tgz"
IMDB_TGZ="${DATA_DIR}/imdb.tgz"
IMDB_CSV_DIR="${JOB_DATA_DIR:-${DATA_DIR}/job_imdb}"
DUCKDB_FILE="${JOB_DB:-${DATA_DIR}/imdb.duckdb}"
ITERATIONS="${ITERATIONS:-3}"
TRACE_CSV="${DATA_DIR}/job_trace.csv"

# Stage 1: clone the JOB query repository.
if [[ ! -d "${JOB_DIR}" ]]; then
  git clone --depth 1 "${JOB_REPO}" "${JOB_DIR}"
fi

# Stage 2: download + extract the IMDb CSV dump (resumable).
if [[ ! -d "${IMDB_CSV_DIR}" ]] || [[ -z "$(ls -A "${IMDB_CSV_DIR}" 2>/dev/null)" ]]; then
  mkdir -p "${IMDB_CSV_DIR}"
  if [[ ! -f "${IMDB_TGZ}" ]]; then
    echo "downloading ${IMDB_TGZ_URL} (~1.26 GB)"
    curl -L -C - -o "${IMDB_TGZ}" "${IMDB_TGZ_URL}"
  fi
  echo "extracting IMDb CSVs to ${IMDB_CSV_DIR}"
  tar -xzf "${IMDB_TGZ}" -C "${IMDB_CSV_DIR}"
fi

# Stage 3: load schema + CSVs into DuckDB (idempotent via duckdb file presence).
if [[ ! -f "${DUCKDB_FILE}" ]]; then
  echo "loading IMDb into DuckDB (${DUCKDB_FILE})"
  "${PY}" - "${JOB_DIR}" "${IMDB_CSV_DIR}" "${DUCKDB_FILE}" <<'PY'
import duckdb, os, sys, glob
job_dir, csv_dir, db_file = sys.argv[1], sys.argv[2], sys.argv[3]
con = duckdb.connect(db_file)
with open(os.path.join(job_dir, "schema.sql")) as f:
    con.execute(f.read())
# Load each CSV matching a schema table name.
tables = [r[0] for r in con.execute("SHOW TABLES").fetchall()]
for t in tables:
    csv = os.path.join(csv_dir, f"{t}.csv")
    if not os.path.exists(csv):
        sys.stderr.write(f"skip {t}: {csv} missing\n")
        continue
    sys.stderr.write(f"loading {t} <- {csv}\n")
    # PostgreSQL COPY uses backslash-escaped quotes ( \" ) in CSV text
    # fields, which is not standard CSV. DuckDB reads this correctly when
    # we set escape='\'.
    con.execute(
        "COPY " + t + " FROM '" + csv + "' "
        "(FORMAT CSV, HEADER FALSE, QUOTE '\"', ESCAPE '\\', NULL '', DELIMITER ',')"
    )
con.close()
PY
fi

# Stage 4: run each JOB query for ITERATIONS replays, emit the trace.
echo "running 113 JOB queries x ${ITERATIONS} iterations"
"${PY}" - "${JOB_DIR}" "${DUCKDB_FILE}" "${ITERATIONS}" "${TRACE_CSV}" <<'PY'
import duckdb, os, sys, glob, json, csv, hashlib, time

job_dir, db_file, iterations, out_csv = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
con = duckdb.connect(db_file, read_only=True)
con.execute("PRAGMA enable_profiling='json'")

def walk(node, acc):
    acc.append(node)
    for c in node.get("children") or []:
        walk(c, acc)

def plan_hash(node):
    parts = []
    def rec(n):
        parts.append(n.get("operator_type") or n.get("operator_name") or "")
        parts.append("(")
        for c in n.get("children") or []:
            rec(c)
        parts.append(")")
    rec(node)
    return hashlib.sha1("|".join(parts).encode()).hexdigest()[:12]

def pick_representative(prof):
    # The top wrapper is RESULT_COLLECTOR / EXPLAIN_ANALYZE. Dive to the
    # meaningful operator subtree first.
    cur = prof
    while (cur.get("operator_type") or cur.get("operator_name") or "") in (
        "RESULT_COLLECTOR", "EXPLAIN_ANALYZE", ""
    ) and cur.get("children"):
        cur = cur["children"][0]
    acc = []
    walk(cur, acc)
    # Prefer joins (where cardinality estimation actually matters).
    join_types = {
        "HASH_JOIN", "NESTED_LOOP_JOIN", "PIECEWISE_MERGE_JOIN",
        "IE_JOIN", "BLOCKWISE_NL_JOIN", "CROSS_PRODUCT",
    }
    joins = [n for n in acc if (n.get("operator_type") or "") in join_types]
    candidate = max(joins, key=lambda n: n.get("operator_cardinality", 0), default=None)
    if candidate is None:
        candidate = cur  # no joins — use root
    est_str = (candidate.get("extra_info") or {}).get("Estimated Cardinality")
    try:
        est = float(est_str) if est_str is not None else float(candidate.get("operator_cardinality") or 0)
    except ValueError:
        est = float(candidate.get("operator_cardinality") or 0)
    actual = float(candidate.get("operator_cardinality") or 0)
    return est, actual

sql_files = sorted(glob.glob(os.path.join(job_dir, "*.sql")))
sql_files = [p for p in sql_files if os.path.basename(p) != "schema.sql"
             and os.path.basename(p) != "fkindexes.sql"]

with open(out_csv, "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["query_id", "iteration", "est_rows", "actual_rows", "latency_ms", "plan_hash"])
    f.flush()
    written = 0
    import re
    def duckdb_rewrite(sql):
        # Queries 15a-d use `aka_title AS at` but `at` is a DuckDB
        # reserved keyword (AT TIME ZONE). Rename the alias without
        # changing any join/filter semantics.
        sql = re.sub(r"\bAS at\b", "AS at_", sql)
        sql = re.sub(r"\bat\.", "at_.", sql)
        return sql

    for it in range(iterations):
        for qpath in sql_files:
            qid = os.path.splitext(os.path.basename(qpath))[0]
            with open(qpath) as fp:
                sql = duckdb_rewrite(fp.read())
            t0 = time.perf_counter()
            try:
                res = con.execute(f"EXPLAIN ANALYZE {sql}").fetchall()
            except Exception as e:
                sys.stderr.write(f"FAIL {qid} iter={it}: {e}\n")
                continue
            wall_ms = (time.perf_counter() - t0) * 1000.0
            try:
                prof = json.loads(res[0][1])
            except Exception as e:
                sys.stderr.write(f"parse-fail {qid} iter={it}: {e}\n")
                continue
            est, actual = pick_representative(prof)
            latency_ms = (prof.get("latency") or 0.0) * 1000.0 or wall_ms
            ph = plan_hash(prof)
            w.writerow([qid, it, int(est), int(actual), f"{latency_ms:.3f}", ph])
            written += 1
            if written % 20 == 0:
                f.flush()
                sys.stderr.write(f"  progress: {written} rows\n")
print(f"wrote {out_csv} ({written} rows over {iterations} iterations)")
PY

sha256sum "${TRACE_CSV}"
echo "OK: JOB trace ready at ${TRACE_CSV}"
