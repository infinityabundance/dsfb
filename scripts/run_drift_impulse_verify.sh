#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

printf '[1/3] Building and running example...\n'
cargo run --release -p dsfb --example drift_impulse

if [[ ! -f out/sim.csv ]]; then
  printf 'ERROR: expected out/sim.csv after running the example.\n' >&2
  exit 1
fi

cp out/sim.csv sim.csv
printf '[2/3] Wrote sim.csv\n'

printf '[3/3] Computing metrics and generating plots...\n'
python3 scripts/analyze_sim.py \
  --csv sim.csv \
  --impulse-start 300 \
  --impulse-duration 100 \
  "$@"
