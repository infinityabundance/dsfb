#!/usr/bin/env bash
# run_bench.sh — GB/s throughput benchmarks, repeated multiple times (the bench itself does >=5
# internal runs per size; this script repeats the whole sweep N times for run-to-run variance).
set -euo pipefail
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${WORKSPACE_ROOT}"
if [[ -z "${CUDA_HOME:-}" && -d /opt/cuda ]]; then export CUDA_HOME=/opt/cuda; fi
if [[ -n "${CUDA_HOME:-}" ]]; then export PATH="${CUDA_HOME}/bin:${PATH}"; fi

REPEATS="${1:-3}"
BIN=./target/release/dsfb-chem-cuda
[[ -x "$BIN" ]] || bash "${CRATE_DIR}/scripts/build_cuda.sh"

echo "=== GB/s benchmark: ${REPEATS} sweep repeats (each size does 5 internal runs) ==="
for r in $(seq 1 "${REPEATS}"); do
  echo "--- sweep $r/${REPEATS} ---"
  DSFB_CHEM_CUDA_DIR="${CRATE_DIR}" "$BIN" bench
done
echo ""
echo "JSON reports written under ${CRATE_DIR}/reports/"
