#!/usr/bin/env bash
# Public-trace bake-off driver.
#
# Runs the `public_trace_bakeoff` binary on the five publicly-cited
# real-workload exemplars (Snowset, SQLShare, CEB, JOB, TPC-DS) and
# renders a paper table.
#
# The outputs are a workload-stress *upper bound* on false-alarm-per-
# hour — not a detection-quality claim. Those traces carry no fault
# annotations, so every emitted episode is counted as an FP by
# construction. See src/bin/public_trace_bakeoff.rs for the full
# honest-framing note.
#
# Usage: bash run.sh            # SEEDS=10 default
#        SEEDS=20 bash run.sh

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${CRATE_DIR}/experiments/public_trace/out"
SEEDS="${SEEDS:-10}"

mkdir -p "${OUT_DIR}"

BIN="${CRATE_DIR}/target/release/public_trace_bakeoff"
if [[ ! -x "${BIN}" ]]; then
  echo "public_trace_bakeoff binary missing at ${BIN}; run:"
  echo "  cargo build --release --bin public_trace_bakeoff --features \"cli report\""
  exit 1
fi

"${BIN}" \
  --seeds "${SEEDS}" \
  --out "${OUT_DIR}"

python3 "${CRATE_DIR}/experiments/public_trace/to_tex.py" \
  "${OUT_DIR}/public_trace_far.csv" \
  "${CRATE_DIR}/paper/tables/public_trace_far.tex"
