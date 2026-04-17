#!/usr/bin/env bash
# Build a TPC-DS scale-1 trace via the DuckDB tpcds extension.
#
# DuckDB ships a tpcds extension that generates the schema, data, and
# canonical query set in-process — no Java toolchain required. We run
# the 99 canonical queries with EXPLAIN ANALYZE and write a CSV in the
# format expected by the dsfb-database TpcDsAdapter.
#
# Usage:  ./scripts/build_tpcds.sh   (writes data/tpcds_trace.csv)

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${HERE}/../data"
mkdir -p "${DATA_DIR}"
OUT="${DATA_DIR}/tpcds_trace.csv"

duckdb -c "
INSTALL tpcds; LOAD tpcds;
CALL dsdgen(sf=1);
"

# DuckDB's tpcds extension exposes tpcds_queries(query_nr) — we run all
# 99 with EXPLAIN ANALYZE and parse latencies + estimated/actual rows.
# This is intentionally a small bash glue script; the heavy lifting is
# in the DuckDB engine.
duckdb -c "
INSTALL tpcds; LOAD tpcds;
CALL dsdgen(sf=1);
COPY (
  SELECT query_nr AS query_id,
         seq AS t_seconds,
         exec_time_ms AS latency_ms,
         estimated_rows AS est_rows,
         actual_rows
  FROM tpcds_run_all_explain_analyze()
) TO '${OUT}' (HEADER, DELIMITER ',');
"

sha256sum "${OUT}"
echo "OK: TPC-DS trace ready at ${OUT}"
