#!/usr/bin/env bash
# DSFB-Database: baseline hyperparameter tuning with held-out
# replication discipline.
#
# Reads the multi-fault real_pg_eval output layout
# (experiments/real_pg_eval/out/<fault>/r*/live.tape.jsonl +
# ground_truth.json), sweeps a small grid of hyperparameters for each
# published baseline on replication TRAIN_REP (default r01 across all
# faults), picks the best macro-F1 config, freezes it, evaluates on
# the remaining replications, and reports mean + 95 % bootstrap CI.
#
# DSFB is evaluated at defaults on the same held-out test tapes.
#
# Usage: bash run.sh            # TRAIN_REP=1, reads out/ under real_pg_eval
#        TRAIN_REP=1 bash run.sh

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${CRATE_DIR}/experiments/baseline_tune/out"
ROOT="${ROOT:-${CRATE_DIR}/experiments/real_pg_eval/out}"
TRAIN_REP="${TRAIN_REP:-1}"

mkdir -p "${OUT_DIR}"

BIN="${CRATE_DIR}/target/release/baseline_tune"
if [[ ! -x "${BIN}" ]]; then
  echo "baseline_tune binary missing at ${BIN}; run:"
  echo "  cargo build --release --bin baseline_tune --features \"cli report live-postgres\""
  exit 1
fi

"${BIN}" \
  --root "${ROOT}" \
  --train-rep "${TRAIN_REP}" \
  --out "${OUT_DIR}"

python3 "${CRATE_DIR}/experiments/baseline_tune/to_tex.py" \
  "${OUT_DIR}/tuned_summary.csv" \
  "${CRATE_DIR}/paper/tables/baseline_tuned.tex"
