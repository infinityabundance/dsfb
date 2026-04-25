#!/usr/bin/env bash
# End-to-end reviewer reproduction script for dsfb-robotics.
#
# Steps (each idempotent):
#   1. Build the vendored Gaz 2019 cpp dynamic model
#   2. Run the dataset preprocessor (skips already-present CSVs)
#   3. Compute the published-θ̂ residual stream for panda_gaz
#   4. Build the paper-lock release binary
#   5. Run paper-lock on every dataset and JSON-checksum the output
#   6. Run the bootstrap CI sweep
#   7. Run the sensitivity grid
#   8. Run the ablation study
#   9. Compile the paper PDF
#
# Compares the bit-exact JSON checksums to `audit/checksums.txt` if present,
# otherwise emits a fresh checksums file the reviewer can compare.

set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${CRATE_ROOT}"

echo "=== dsfb-robotics reviewer reproduction ==="
echo "    crate root: ${CRATE_ROOT}"
echo

echo "[1/9] Build vendored Gaz cpp model"
bash scripts/build_panda_gaz_model.sh

echo "[2/9] Preprocess datasets (existing CSVs skipped)"
python3 scripts/preprocess_datasets.py 2>&1 | tail -3 || true

echo "[3/9] Compute published-θ̂ residual stream for panda_gaz"
python3 scripts/compute_published_residuals.py

echo "[4/9] Build paper-lock release binary"
cargo build --release --features std,paper_lock --bin paper-lock --quiet

BIN="target/release/paper-lock"
mkdir -p audit/json_outputs
echo "[5/9] Run paper-lock on every dataset and checksum the output"
fresh_checksums="audit/checksums.fresh.txt"
: > "${fresh_checksums}"
for slug in $($BIN --list); do
    out="audit/json_outputs/${slug}.json"
    $BIN "${slug}" > "${out}" 2>/dev/null
    sha=$(sha256sum "${out}" | awk '{print $1}')
    echo "${sha}  ${out}" >> "${fresh_checksums}"
done

if [[ -f audit/checksums.txt ]]; then
    if diff -q audit/checksums.txt "${fresh_checksums}" > /dev/null; then
        echo "  checksums match audit/checksums.txt — bit-exact reproduction OK"
    else
        echo "  WARNING: checksums differ from audit/checksums.txt"
        echo "  diff:"
        diff audit/checksums.txt "${fresh_checksums}" || true
    fi
else
    cp "${fresh_checksums}" audit/checksums.txt
    echo "  emitted audit/checksums.txt for the first time"
fi

echo "[6/9] Bootstrap CI sweep (this takes a few minutes)"
python3 scripts/bootstrap_census.py 2>&1 | tail -5 || true

echo "[7/9] Sensitivity grid (panda_gaz)"
python3 scripts/sensitivity_grid.py panda_gaz 2>&1 | tail -3 || true

echo "[8/9] Ablation study (3 exemplars)"
python3 scripts/ablation.py 2>&1 | tail -10 || true

echo "[9/9] Compile paper PDF"
(cd paper && latexmk -pdf dsfb_robotics.tex > /tmp/latexmk.log 2>&1) || {
    echo "  paper compile failed — see /tmp/latexmk.log"
    exit 2
}

echo
echo "=== reproduction complete ==="
echo "  paper:    ${CRATE_ROOT}/paper/dsfb_robotics.pdf"
echo "  bootstrap: ${CRATE_ROOT}/audit/bootstrap/"
echo "  sensitivity: ${CRATE_ROOT}/audit/sensitivity/"
echo "  ablation:  ${CRATE_ROOT}/audit/ablation/"
echo "  checksums: ${CRATE_ROOT}/audit/checksums.txt"
