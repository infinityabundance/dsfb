#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

printf '[1/3] Building and running example...\n'
cargo run --release -p dsfb --example drift_impulse

latest_sim_csv="$(
  find output-dsfb -mindepth 2 -maxdepth 2 -type f -name 'sim-dsfb.csv' 2>/dev/null | sort | tail -n1
)"
if [[ -z "${latest_sim_csv}" ]]; then
  latest_sim_csv="$(
    find output-dsfb -mindepth 2 -maxdepth 2 -type f -name 'sim.csv' 2>/dev/null | sort | tail -n1
  )"
fi
if [[ -z "${latest_sim_csv}" ]]; then
  printf 'ERROR: expected output-dsfb/<timestamp>/sim-dsfb.csv after running the example.\n' >&2
  exit 1
fi

sim_csv="${latest_sim_csv}"
latest_run_dir="$(dirname "${sim_csv}")"
analysis_outdir="${latest_run_dir}/analysis"
printf '[2/3] Using simulation CSV: %s\n' "${sim_csv}"

printf '[3/3] Computing metrics and generating plots...\n'
python3 scripts/analyze_sim.py \
  --csv "${sim_csv}" \
  --outdir "${analysis_outdir}" \
  --impulse-start 300 \
  --impulse-duration 100 \
  "$@"

printf 'Analysis output directory: %s\n' "${analysis_outdir}"
