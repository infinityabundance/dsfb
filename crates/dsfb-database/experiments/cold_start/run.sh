#!/usr/bin/env bash
# Pass-2 N2: cold-start ablation.
#
# For each (fault, replication) tape under
# experiments/real_pg_eval/out/, drop the leading WARMUP seconds of
# residual samples and re-run the bake-off. Reports the per-detector
# TTD and recall as a function of warmup truncation.
#
# Replay-only — never captures a new tape, never touches the engine.
# The aggregator builds truncated tapes in tempdirs, recomputes the
# SHA-256 sidecar, patches a matching ground-truth JSON, and invokes
# replay_tape_baselines on the (truncated_tape, truncated_gt) pair.
#
# Wall-clock: ~5 minutes for 4 faults × 10 reps × 4 warmups × 4
# detectors = 640 bake-off invocations on the default settings; each
# invocation is < 1 second.
#
# Output: experiments/cold_start/out/cold_start.csv
#
# The §43 paragraph in paper/dsfb-database.tex cites this CSV.
#
# Pre-requisites:
#   * `experiments/real_pg_eval/run.sh` must have completed.
#   * `replay_tape_baselines` binary built; see `cargo build --release
#     --features "cli report live-postgres" --bin replay_tape_baselines`.
#   * Python 3.

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
PG_OUT="${PG_OUT:-${CRATE_DIR}/experiments/real_pg_eval/out}"
OUT_DIR="${CRATE_DIR}/experiments/cold_start/out"
WARMUPS="${WARMUPS:-0,10,20,30}"
MAX_REPS="${MAX_REPS:-10}"

mkdir -p "${OUT_DIR}"

if [ ! -d "${PG_OUT}" ]; then
    echo "[cold_start] PG_OUT not found: ${PG_OUT}" >&2
    echo "  run experiments/real_pg_eval/run.sh first." >&2
    exit 2
fi

cd "${CRATE_DIR}"
cargo build --release --features "cli report live-postgres" \
    --bin replay_tape_baselines --quiet

BIN="${CRATE_DIR}/target/release/replay_tape_baselines"
if [ ! -x "${BIN}" ]; then
    echo "[cold_start] missing binary: ${BIN}" >&2
    exit 2
fi

python3 "${CRATE_DIR}/experiments/cold_start/aggregate.py" \
    --pg-out "${PG_OUT}" \
    --bin "${BIN}" \
    --out "${OUT_DIR}" \
    --warmups "${WARMUPS}" \
    --max-reps "${MAX_REPS}"

echo "[cold_start] done. CSV: ${OUT_DIR}/cold_start.csv"
