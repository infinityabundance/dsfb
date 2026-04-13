#!/usr/bin/env bash
# build_figures.sh — Full figure pipeline: Rust export → Python gen → PDF + zip
#
# Usage:
#   bash scripts/build_figures.sh
#
# Environment (optional):
#   DSFB_OUTPUT   Write outputs here; otherwise a fresh timestamped directory
#                 is created at <workspace-root>/output-dsfb-oil-gas/dsfb-oil-gas-<stamp>/
#
# Outputs (inside $DSFB_OUTPUT/figures/):
#   trace_data/real_*.csv      — per-step DSFB trace data (intermediate; also in crate)
#   fig_*.pdf                  — 20 individual figures
#   all_figures.pdf            — compiled figure booklet
#   dsfb_figures.zip           — zip of all PDFs
set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKSPACE_ROOT="$(cd "${CRATE_ROOT}/../.." && pwd)"

# ── Determine output directory ─────────────────────────────────────────────────
if [ -z "${DSFB_OUTPUT:-}" ]; then
    STAMP=$(date +%Y-%m-%d-%H%M%S)
    DSFB_OUTPUT="${WORKSPACE_ROOT}/output-dsfb-oil-gas/dsfb-oil-gas-${STAMP}"
fi
export DSFB_OUTPUT
FIG_OUT="${DSFB_OUTPUT}/figures"
mkdir -p "${FIG_OUT}"

echo "═══════════════════════════════════════════════════════"
echo "  DSFB Oil & Gas — Figure Pipeline"
echo "  Crate  : ${CRATE_ROOT}"
echo "  Output : ${DSFB_OUTPUT}"
echo "═══════════════════════════════════════════════════════"

cd "${CRATE_ROOT}"

# ── Step 1: Rust export (writes intermediate trace_data into crate) ────────────
echo ""
echo "► Step 1/4: Rust export (cargo run --example export_grammar_traces)"
cargo run --release --example export_grammar_traces

# ── Step 2: Python figure generation (writes fig_*.pdf to OUTPUT/figures/) ─────
echo ""
echo "► Step 2/4: Python figure generation (scripts/gen_figures.py)"
python3 scripts/gen_figures.py

# ── Step 3: LaTeX figure booklet ───────────────────────────────────────────────
echo ""
echo "► Step 3/4: LaTeX figure booklet (all_figures.tex)"
cp "${CRATE_ROOT}/figures/all_figures.tex" "${FIG_OUT}/"
cd "${FIG_OUT}"
if command -v pdflatex &>/dev/null; then
    pdflatex -interaction=nonstopmode all_figures.tex > /dev/null 2>&1 \
        && pdflatex -interaction=nonstopmode all_figures.tex > /dev/null 2>&1 \
        && echo "  all_figures.pdf compiled" \
        || echo "  WARNING: pdflatex failed — skipping booklet"
else
    echo "  WARNING: pdflatex not found — skipping booklet"
fi
cd "${CRATE_ROOT}"

# ── Step 4: Zip ────────────────────────────────────────────────────────────────
echo ""
echo "► Step 4/4: Creating zip archive"
cd "${FIG_OUT}"
zip -q dsfb_figures.zip fig_*.pdf 2>/dev/null || true
if [ -f all_figures.pdf ]; then
    zip -q dsfb_figures.zip all_figures.pdf
fi
cd "${CRATE_ROOT}"

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  DONE"
FIG_COUNT=$(ls "${FIG_OUT}"/fig_*.pdf 2>/dev/null | wc -l)
echo "  Individual figures : ${FIG_COUNT}  (${FIG_OUT}/fig_*.pdf)"
[ -f "${FIG_OUT}/all_figures.pdf" ] && echo "  Figure booklet    : ${FIG_OUT}/all_figures.pdf"
[ -f "${FIG_OUT}/dsfb_figures.zip" ] && echo "  Download archive  : ${FIG_OUT}/dsfb_figures.zip"
echo "═══════════════════════════════════════════════════════"
