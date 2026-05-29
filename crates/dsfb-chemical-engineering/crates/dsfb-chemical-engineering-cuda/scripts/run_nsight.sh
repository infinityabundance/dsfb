#!/usr/bin/env bash
# run_nsight.sh — capture Nsight Systems (nsys) timelines and Nsight Compute (ncu) kernel metrics
# for the evidence factory, repeated MULTIPLE times across MULTIPLE size variants. Raw text
# summaries are written to reports/ and committed; the large .nsys-rep/.ncu-rep blobs are gitignored.
#
# Usage:  bash scripts/run_nsight.sh [runs]   (default 5 runs per variant)
set -euo pipefail
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${WORKSPACE_ROOT}"
if [[ -z "${CUDA_HOME:-}" && -d /opt/cuda ]]; then export CUDA_HOME=/opt/cuda; fi
if [[ -n "${CUDA_HOME:-}" ]]; then export PATH="${CUDA_HOME}/bin:${PATH}"; fi

RUNS="${1:-5}"
BIN=./target/release/dsfb-chem-cuda
REPORTS="${CRATE_DIR}/reports"
mkdir -p "${REPORTS}"
# Opt-in sudo for nsys on hosts that restrict profiling:  NSYS_SUDO=sudo bash run_nsight.sh
# (sudo strips the environment, so the launches below use `sudo env VAR=... nsys ...`).
SUDO="${NSYS_SUDO:-}"

# Always (re)build with the CUDA backend, then verify the binary is actually CUDA-enabled. A CPU-only
# binary makes `profile` a no-op ("profile requires --features cuda") and nsys then captures NO CUDA
# data (empty report). Abort loudly rather than silently emit empty captures.
bash "${CRATE_DIR}/scripts/build_cuda.sh"
if DSFB_BENCH_LANES=2 DSFB_BENCH_SAMPLES=2 "$BIN" profile 2>&1 | grep -qi "requires --features cuda"; then
  echo "ERROR: $BIN is not CUDA-enabled (profile is a no-op) — refusing to capture empty data." >&2
  echo "       Ensure nvcc is on PATH and rebuild: bash ${CRATE_DIR}/scripts/build_cuda.sh" >&2
  exit 1
fi

if ! command -v nsys >/dev/null 2>&1; then echo "nsys not found; skipping Nsight Systems"; fi

# Preflight: Nsight Compute needs GPU performance-counter access (admin-only by default via the
# NVreg_RestrictProfilingToAdminUsers kernel-module flag). Probe once; if denied, record the gate
# and skip ncu (nsys still runs — it does not need counter permissions).
NCU_OK=0
if command -v ncu >/dev/null 2>&1; then
  if DSFB_BENCH_LANES=512 DSFB_BENCH_SAMPLES=1024 \
       ncu --kernel-name evidence_kernel --launch-count 1 --metrics gpu__time_duration.sum \
       "$BIN" profile >/dev/null 2>"${REPORTS}/.ncu_preflight"; then
    NCU_OK=1
    # Counters available: record current state so the committed note never goes stale.
    {
      echo "Nsight Compute (ncu) counters were available and captured normally on this host."
      echo "Per-variant medians (occupancy, SM- and DRAM-throughput, L2 sector-hit) are distilled"
      echo "into reports/NSIGHT_SUMMARY.md and reports/nsight_summary.json by summarize_nsight.py."
      echo "If another host reports ERR_NVGPUCTRPERM (admin-restricted counters), enable them once"
      echo "(root): set NVreg_RestrictProfilingToAdminUsers=0, regenerate the initramfs (e.g."
      echo "mkinitcpio -P on Arch/CachyOS), and reboot; or run 'sudo ncu ...'."
    } > "${REPORTS}/ncu_permission_note.txt"
  else
    {
      echo "Nsight Compute (ncu) was unavailable on this host: GPU performance-counter access is"
      echo "admin-restricted (ERR_NVGPUCTRPERM). To capture occupancy / DRAM-throughput / warp"
      echo "metrics, enable counters once (root): set NVreg_RestrictProfilingToAdminUsers=0 (reboot)"
      echo "or run 'sudo ncu ...', then re-run this script. Nsight Systems (nsys) timings below do"
      echo "NOT require this and were captured normally."
      echo "--- preflight stderr ---"; cat "${REPORTS}/.ncu_preflight"
    } > "${REPORTS}/ncu_permission_note.txt"
    echo "ncu: counter access denied — see reports/ncu_permission_note.txt (nsys still runs)."
  fi
  rm -f "${REPORTS}/.ncu_preflight"
fi

# Size variants (lanes x samples). Two sets:
#  - STRESS (a/b/g, ~64 MB each): the throughput-roofline aspect-ratio sweep (the meaningful axis for the
#    per-lane-serial SHA kernel) — wide / deep / very-wide.
#  - REALISTIC (r1..r4): the actual evaluation-dataset shapes (5-590 lanes), where the kernel is tiny and
#    host<->device transfer dominates END TO END — so the GPU's value at deployment sizes is byte-exact
#    determinism, not throughput (B6). nsys --stats below emits a CUDA GPU MemOps Summary (HtoD+DtoH) + a
#    Kernel Summary per variant, so end-to-end = H2D + kernel + D2H is read straight from reports/nsys_*.txt;
#    the `bench` subcommand gives the kernel-only GB/s. r1=CSTR, r2=TEP IDV, r3=gas-sensor, r4=SECOM.
declare -A VARIANTS=(
  [a]="2048 4096" [b]="1024 8192" [g]="4096 2048"
  [r1]="5 800" [r2]="52 960" [r3]="128 600" [r4]="590 600"
)
NCU_METRICS="gpu__time_duration.sum,sm__throughput.avg.pct_of_peak_sustained_elapsed,dram__throughput.avg.pct_of_peak_sustained_elapsed,sm__warps_active.avg.pct_of_peak_sustained_active,lts__t_sector_hit_rate.pct,launch__occupancy_limit_warps"

for v in a b g r1 r2 r3 r4; do
  read -r LANES SAMPLES <<< "${VARIANTS[$v]}"
  for run in $(seq 1 "${RUNS}"); do
    tag="evidence_${LANES}x${SAMPLES}_variant_${v}_run${run}"
    echo "=== Nsight ${tag} ==="
    if [[ "${NCU_OK}" == "1" ]]; then
      DSFB_BENCH_LANES=$LANES DSFB_BENCH_SAMPLES=$SAMPLES \
        ncu --target-processes all --kernel-name evidence_kernel --launch-count 1 \
            --metrics "${NCU_METRICS}" --csv --log-file "${REPORTS}/ncu_${tag}.csv" \
            "$BIN" profile >/dev/null 2>&1 || echo "  (ncu run ${tag} returned non-zero; see log)"
    fi
    if command -v nsys >/dev/null 2>&1; then
      DSFB_BENCH_LANES=$LANES DSFB_BENCH_SAMPLES=$SAMPLES \
        nsys profile --force-overwrite true -o "${REPORTS}/nsys_${tag}" --stats=true \
            "$BIN" profile > "${REPORTS}/nsys_${tag}.txt" 2>&1 || echo "  (nsys run ${tag} returned non-zero)"
    fi
  done
done

echo ""
echo "Nsight raw summaries under ${REPORTS}/ (ncu_*.csv, nsys_*.txt). Large .rep blobs are gitignored."
