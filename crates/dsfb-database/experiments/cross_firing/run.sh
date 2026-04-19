#!/usr/bin/env bash
# Pass-2 N1: cross-motif cross-firing matrix on the existing
# multi-fault tape corpus.
#
# Quantifies what the §13 ¶2 prose-only observation already names: when
# fault X is planted, do detectors emit episodes for motifs other than
# the ground-truth motif for X? The §36 paragraph in paper/dsfb-database.tex
# cites this matrix.
#
# Replay-only experiment — never captures a new tape, never touches
# the engine. Reads the per-(fault, rep) bakeoff CSVs already produced
# by `experiments/real_pg_eval/run.sh`, partitions emitted episodes
# by (planted_fault, emitted_motif, detector), reports counts and
# FAR/hr per cell.
#
# Wall-clock: <30 seconds (Python aggregator over O(160) CSVs).
#
# Output: experiments/cross_firing/out/cross_firing.csv
#         paper/tables/cross_firing.tex
#
# Pre-requisites:
#   * `experiments/real_pg_eval/run.sh` must have completed for at
#     least one container/version (default: PG17 under
#     experiments/real_pg_eval/out/).
#   * Python 3 (no extra packages required).

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
PG_OUT="${PG_OUT:-${CRATE_DIR}/experiments/real_pg_eval/out}"
OUT_DIR="${CRATE_DIR}/experiments/cross_firing/out"
mkdir -p "${OUT_DIR}"

if [ ! -d "${PG_OUT}" ]; then
    echo "[cross_firing] PG_OUT not found: ${PG_OUT}" >&2
    echo "  run experiments/real_pg_eval/run.sh first, or override with PG_OUT=..." >&2
    exit 2
fi

python3 "${CRATE_DIR}/experiments/cross_firing/aggregate.py" \
    --pg-out "${PG_OUT}" \
    --csv "${OUT_DIR}/cross_firing.csv" \
    --tex "${CRATE_DIR}/paper/tables/cross_firing.tex"

echo "[cross_firing] done. CSV: ${OUT_DIR}/cross_firing.csv"
