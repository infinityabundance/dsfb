#!/usr/bin/env bash
# export_artifacts.sh — one command: fetch datasets (if missing), run the full edge demo over all
# 20 datasets, and render the publication figures into paper/figures/.
#
# Usage:  bash scripts/export_artifacts.sh [--no-fetch]
#
# Requires: cargo, python3 (+ numpy/scipy/pandas/sklearn/matplotlib for fetch/figures).
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_DIR}/../.." && pwd)"
cd "${WORKSPACE_ROOT}"

echo "================================================================"
echo "  DSFB-Chemical-Engineering — Edge artifact export"
echo "================================================================"

if [[ "${1:-}" != "--no-fetch" ]]; then
  if [[ ! -d "${CRATE_DIR}/data/slices" || -z "$(ls -A "${CRATE_DIR}/data/slices" 2>/dev/null)" ]]; then
    echo "=> Fetching dataset slices ..."
    python3 "${CRATE_DIR}/scripts/fetch_datasets.py"
  else
    echo "=> Dataset slices present (skipping fetch; pass --no-fetch to silence)."
  fi
fi

echo "=> Building release binary ..."
cargo build --release -p dsfb-chemical-engineering-edge

echo "=> Running full demo over all datasets ..."
DSFB_CHEM_DIR="${CRATE_DIR}" ./target/release/dsfb-chem-edge demo

echo "=> Rendering publication figures into paper/figures/ ..."
python3 "${CRATE_DIR}/scripts/gen_figures.py"

echo ""
echo "EXPORT COMPLETE."
echo "  artifacts: ${WORKSPACE_ROOT}/output-dsfb-chemical-engineering/<latest>/"
echo "  figures  : ${WORKSPACE_ROOT}/paper/figures/"
