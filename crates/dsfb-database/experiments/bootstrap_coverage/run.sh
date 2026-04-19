#!/usr/bin/env bash
# Pass-2 N4: Monte-Carlo coverage of the percentile-bootstrap 95 % CI
# at small sample sizes.
#
# Pure synthetic experiment — no engine, no podman, no fixtures touched.
# Wall-clock: ~2 seconds for the default `--n-mc 2000` × 4 sample
# sizes × 3 distributions = 24000 bootstrap CIs, each with B=1000
# resamples.
#
# Output: experiments/bootstrap_coverage/out/coverage.csv
#         paper/figs/bootstrap_coverage.png
#
# The CSV is the source of the §39 paragraph in paper/dsfb-database.tex
# that quantifies the percentile-bootstrap under-coverage at n=10.
#
# This script does not stage, commit, or push paper/ — paper rendering
# happens in `paper/build.sh`, which the user runs locally per the
# durable "paper is local-only" invariant.

set -euo pipefail

CRATE_DIR=$(cd "$(dirname "$0")/../.." && pwd)
OUT_DIR="${CRATE_DIR}/experiments/bootstrap_coverage/out"
N_MC="${N_MC:-2000}"
N_LIST="${N_LIST:-5,10,20,50}"
SEED="${SEED:-42}"

mkdir -p "${OUT_DIR}"

cd "${CRATE_DIR}"
cargo build --release --features cli --bin bootstrap_coverage --quiet

./target/release/bootstrap_coverage \
    --out "${OUT_DIR}" \
    --n "${N_LIST}" \
    --n-mc "${N_MC}" \
    --seed "${SEED}"

python3 "${CRATE_DIR}/experiments/bootstrap_coverage/aggregate.py" \
    "${OUT_DIR}/coverage.csv" \
    "${CRATE_DIR}/paper/figs/bootstrap_coverage.png"

echo "[bootstrap_coverage] done. CSV: ${OUT_DIR}/coverage.csv"
