#!/usr/bin/env bash
# Reproduce every figure and table in the dsfb-database paper.
#
# Inputs:    deterministic exemplars (no network required), plus optional
#            real-data CSVs under ./data/ if the corresponding fetch_*.sh
#            script has been run.
#
# Outputs:   ./out/                 — CSV, JSON, PNG figures
#            ./paper/dsfb-database.pdf  — built paper
#
# Exit status: 0 on success, non-zero if any check fails.
#
# This script is the single command an SBIR reviewer or licensing team
# should run to verify reproducibility. It is also the script the Colab
# notebook calls via subprocess.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="${HERE}/.."
cd "${ROOT}"

OUT="${ROOT}/out"
mkdir -p "${OUT}"

echo "==> 1/7  Building dsfb-database (release)"
cargo build --release --quiet

echo "==> 2/7  Running test suite (replay determinism + non-claim lock)"
cargo test --release --quiet

echo "==> 3/7  Running controlled-perturbation pipeline (TPC-DS-shaped)"
./target/release/dsfb-database reproduce --seed 42 --out "${OUT}"

echo "==> 4/7  Running per-dataset exemplars"
for d in snowset sqlshare ceb job tpcds; do
  ./target/release/dsfb-database exemplar --dataset "$d" --out "${OUT}"
done

echo "==> 5/7  Replay-determinism cross-check"
./target/release/dsfb-database replay-check --seed 42

echo "==> 6/7  Threshold elasticity sweep (+- 20 %)"
./target/release/dsfb-database elasticity --seed 42 --out "${OUT}"

# Optional: real-data Snowset run if the operator has fetched a shard.
# We do not bundle Snowset (CC-BY 4.0 but ~50 MB+); see
# scripts/fetch_snowset_subset.sh. If absent, we explicitly skip and the
# paper §6 records that this dataset is adapter-supported but exemplar-only
# in the bundled reproduction.
SNOWSET_CSV="${ROOT}/data/snowset_shard.csv"
if [[ -f "${SNOWSET_CSV}" ]]; then
  echo "==> 6.5/7  Real Snowset shard detected; running real-data ingest"
  ./target/release/dsfb-database run --dataset snowset --path "${SNOWSET_CSV}" \
    --out "${OUT}"
  sha256sum "${SNOWSET_CSV}" | tee "${OUT}/snowset.shard.sha256"
else
  echo "==> 6.5/7  Snowset shard not present at ${SNOWSET_CSV}; skipping real-data run"
  echo "          (run scripts/fetch_snowset_subset.sh to enable; the §6 paragraph"
  echo "           in the paper records this as adapter-supported but exemplar-only"
  echo "           in the bundled reproduction.)"
fi

# Optional: real-engine PostgreSQL ingest from the bundled redacted sample CSV.
# This is the bridge from controlled-trace evaluation to a real-engine view
# (T1.1 in the elevation plan); the sample is synthetic but mirrors the
# pg_stat_statements 14+ schema verbatim.
echo "==> 6.7/7  PostgreSQL pg_stat_statements ingest from bundled sample"
./target/release/dsfb-database ingest --engine postgres \
  --csv "${ROOT}/examples/data/pg_stat_statements_sample.csv" --out "${OUT}/pgss"

echo "==> 7/7  Building paper PDF"
if [[ -d "${ROOT}/paper" ]]; then
  # Refresh paper/figs/ from the just-generated out/ artefacts so every
  # figure cited from §5/§8 is the one the reviewer will rebuild.
  if [[ -f "${OUT}/tpcds.phase_portrait.png" ]]; then
    cp "${OUT}/tpcds.phase_portrait.png" "${ROOT}/paper/figs/"
  fi
  ( cd "${ROOT}/paper" && bash build.sh ) || \
    echo "(paper PDF build skipped — install latexmk to enable)"
fi

echo
echo "OK: figures + CSVs in ${OUT}; paper at ${ROOT}/paper/dsfb-database.pdf"
echo
echo "Reviewer-facing summary:"
column -t -s, "${OUT}/tpcds.metrics.csv" || cat "${OUT}/tpcds.metrics.csv"
