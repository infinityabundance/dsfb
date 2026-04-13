#!/usr/bin/env bash
# scripts/export_artifacts.sh -- Full artifact export pipeline.
#
# Creates a single timestamped directory and runs all generation steps,
# so figures, paper, and notebook all share the same output folder.
#
# Usage:
#   bash scripts/export_artifacts.sh [STAMP]
#
#   STAMP  Optional timestamp (default: YYYY-MM-DD-HHMMSS).
#
# Output:
#   <workspace-root>/output-dsfb-oil-gas/dsfb-oil-gas-<STAMP>/
#     figures/  -- 20 individual PDFs + booklet + zip
#     paper/    -- LaTeX source + compiled PDF
#     colab/    -- Jupyter notebook

set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_ROOT}/../.." && pwd)"

STAMP="${1:-$(date +%Y-%m-%d-%H%M%S)}"
export DSFB_OUTPUT="${WORKSPACE_ROOT}/output-dsfb-oil-gas/dsfb-oil-gas-${STAMP}"
mkdir -p "${DSFB_OUTPUT}"

echo "================================================================"
echo "  DSFB Oil & Gas -- Full Artifact Export"
echo "  Crate  : ${CRATE_ROOT}"
echo "  Output : ${DSFB_OUTPUT}"
echo "================================================================"

cd "${CRATE_ROOT}"

echo ""
echo "=> Phase 1/3: Figure pipeline"
bash scripts/build_figures.sh

echo ""
echo "=> Phase 2/3: Paper"
bash scripts/build_paper.sh

echo ""
echo "=> Phase 3/3: Notebook"
python3 scripts/gen_notebook.py

echo ""
echo "================================================================"
echo "  EXPORT COMPLETE"
echo "  Output directory: ${DSFB_OUTPUT}"
echo ""
find "${DSFB_OUTPUT}" -type f | sort
echo "================================================================"
