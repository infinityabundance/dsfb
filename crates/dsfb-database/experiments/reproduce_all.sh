#!/usr/bin/env bash
# Heavy-path reproduction: chains every live, podman-based, time-consuming
# experiment behind the paper's auxiliary tables, plus the synthetic-data
# reproduce_paper.sh, plus the PDF build. Wall-time is dominated by the
# real_pg_eval matrix (4 faults x 10 reps x 2 PG versions) and the
# baseline_tune sweep over the same 40-tape corpus.
#
# Inputs:
#   - podman with internet access (pulls postgres:16, postgres:17, mysql:8)
#   - ~3.5 hours wall on a recent x86_64 laptop with NVMe storage
#
# Outputs:
#   - experiments/{real_pg_eval,real_pg_eval_pg16,observer_load,
#       baseline_tune,public_trace,real_mysql_eval}/out/ are populated
#       with bakeoff CSVs, tapes, and summary CSVs
#   - paper/tables/{live_eval_mean_ci,observer_self_load,baseline_tuned,
#       public_trace_far,pg_version_compat}.tex are regenerated
#   - paper/dsfb-database.pdf is rebuilt
#
# This script is the entry point named in §Reproducibility of the paper
# and the README's "Multi-engine, multi-fault evaluation" subsection.
# scripts/reproduce_paper.sh is the fast path (no podman, ~minutes) that
# regenerates only the synthetic-data tables; experiments/reproduce_all.sh
# is the heavy path (podman, ~hours) that regenerates everything.

set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="${HERE}/.."
cd "${ROOT}"

PG17_DIGEST="${PG17_DIGEST:-sha256:7ad98329d513dd497293b951c195ca354274a77f12ddbbbbf85e68a811823d72}"
PG16_DIGEST="${PG16_DIGEST:-sha256:01710bb7d42744d53c02d61d7b265b2901aa5fc21ed3f7e35e726b29af5deeb6}"

log() { echo "[$(date -u +%Y-%m-%dT%H:%M:%SZ)] $*"; }

log "==> 1/8  Build (release, all live features)"
cargo build --release --features "cli report live-postgres live-mysql" --quiet

log "==> 2/8  Test (replay determinism + non-claim lock + allow-list locks)"
cargo test --release --features "cli report live-postgres live-mysql" --quiet

log "==> 3/8  Real PG17 multi-fault matrix (4 faults x 10 reps, ~50 min)"
for f in drop_constraint stats_stale lock_hold cache_evict; do
  log "    fault=${f} (PG17)"
  FAULT="$f" N_REPS=10 \
    bash experiments/real_pg_eval/run.sh
done

log "==> 4/8  Real PG16 compat matrix (4 faults x 10 reps, ~50 min)"
for f in drop_constraint stats_stale lock_hold cache_evict; do
  log "    fault=${f} (PG16)"
  OUT_DIR="${ROOT}/experiments/real_pg_eval/out_pg16" \
  PG_IMAGE=docker.io/library/postgres:16 \
  PG_IMAGE_DIGEST="${PG16_DIGEST}" \
  FAULT="$f" N_REPS=10 \
    bash experiments/real_pg_eval/run.sh
done
log "    rendering pg_version_compat.tex"
python3 experiments/real_pg_eval/compat_to_tex.py \
  experiments/real_pg_eval/out/summary.csv \
  experiments/real_pg_eval/out_pg16/summary.csv \
  "${PG17_DIGEST}" "${PG16_DIGEST}" \
  paper/tables/pg_version_compat.tex

log "==> 5/8  Observer self-load (pgbench latency CDF, ~15 min)"
bash experiments/observer_load/run.sh

log "==> 6/8  Held-out baseline tuning (~10 min on cached PG17 tapes)"
bash experiments/baseline_tune/run.sh

log "==> 7/8  Public-trace FAR bake-off (~5 min)"
bash experiments/public_trace/run.sh

log "==> 8/8  MySQL contract layer (~5 min, ships contract not full tape)"
bash experiments/real_mysql_eval/run.sh || \
  log "    note: MySQL run failed; contract test still passes via cargo"

log "==> Synthetic-data tables + PDF build"
bash scripts/reproduce_paper.sh

log "done. paper at paper/dsfb-database.pdf"
