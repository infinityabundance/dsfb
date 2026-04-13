#!/usr/bin/env bash
# scripts/run_notebook.sh — Execute the DSFB demonstration notebook.
#
# Usage: bash scripts/run_notebook.sh [--html]
#
# Environment (optional):
#   DSFB_OUTPUT   Directory containing notebook/dsfb_oil_gas.ipynb.
#                 If not set, the bundled crate notebook is used, or the notebook
#                 is regenerated via gen_notebook.py.
#
# Requires: jupyter (pip install jupyter nbconvert)
# Output:   <DSFB_OUTPUT>/notebook/dsfb_oil_gas.ipynb  (executed, in-place)
#           <DSFB_OUTPUT>/notebook/dsfb_oil_gas.html    (if --html flag passed)

set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_ROOT}/../.." && pwd)"

# ── Locate the notebook ────────────────────────────────────────────────────────
if [ -n "${DSFB_OUTPUT:-}" ]; then
    NOTEBOOK="${DSFB_OUTPUT}/notebook/dsfb_oil_gas.ipynb"
else
    # Prefer the bundled crate notebook
    BUNDLED="${CRATE_ROOT}/notebook/dsfb_oil_gas.ipynb"
    if [ -f "${BUNDLED}" ]; then
        NOTEBOOK="${BUNDLED}"
        DSFB_OUTPUT="${CRATE_ROOT}"
    else
        # Fall back: regenerate notebook into a new timestamped output dir
        echo "No existing notebook found — regenerating via gen_notebook.py..."
        export DSFB_OUTPUT="${WORKSPACE_ROOT}/output-dsfb-oil-gas/dsfb-oil-gas-$(date +%Y-%m-%d-%H%M%S)"
        cd "${CRATE_ROOT}"
        python3 scripts/gen_notebook.py
        NOTEBOOK="${DSFB_OUTPUT}/notebook/dsfb_oil_gas.ipynb"
    fi
fi

echo "=== DSFB Notebook Execution ==="
echo "Notebook: ${NOTEBOOK}"

if ! command -v jupyter &>/dev/null; then
    echo "ERROR: jupyter not found. Install with: pip install jupyter nbconvert"
    exit 1
fi

echo ""
echo "--- Executing notebook (this may take a minute) ---"
jupyter nbconvert \
    --to notebook \
    --execute \
    --inplace \
    --ExecutePreprocessor.timeout=120 \
    "${NOTEBOOK}"

echo ""
echo "Execution complete: ${NOTEBOOK}"

if [[ "${1:-}" == "--html" ]]; then
    echo ""
    echo "--- Converting to HTML ---"
    jupyter nbconvert --to html "${NOTEBOOK}"
    HTML="${NOTEBOOK%.ipynb}.html"
    echo "HTML output: ${HTML}"
fi
