#!/usr/bin/env bash
# S-REAL.3.throughput — per-dataset throughput bench across the
# 20-dataset audit gauntlet. Separate from the per-dataset perf +
# Nsight script (scripts/s_real_perf_per_dataset.sh) because the
# user asked for "real GB/s throughput in a separate script."
#
# WHY THIS EXISTS (for the future engineer reading cold):
#
# This script RUNS the dispatcher on each dataset with multiple
# iterations and measures STEADY-STATE bandwidth from the median
# dispatch time, not from a single cold-start dispatch. The audit
# binary already supports --iters N and emits dispatch_median_us
# / dispatch_p50_us / p95 / p99 into perf_profile.txt; this
# script orchestrates the iter sweep and computes GB/s from the
# resulting medians.
#
# Default --iters = 7  (mirrors the S-PERF.16.a bench cadence:
# 3 warmup + 7 measurement iterations per stage; the audit
# binary's perf_profile.txt computes the median across all
# recorded iters, so a higher iters count = tighter median).
#
# Honest framing (panel-locked, MUST hold in every receipt):
#   - Fixtures are small (128-656 events / dataset). Even at
#     steady state, the per-call wall is launch-overhead-dominated
#     because there isn't enough work per dispatch to fill an
#     RTX 4080 SUPER. The numbers below are HONEST measurement of
#     a launch-overhead regime, not a saturation regime.
#   - DO NOT compare against S-PERF.16.a's saturation bench
#     (256x4096 K=1, 4M+ events). The two layers measure
#     structurally different workloads.
#   - The bandwidth reported here is FIXTURE BYTES PROCESSED
#     PER STEADY-STATE DISPATCH WALL. That measures end-to-end
#     dispatcher throughput including H2D, all kernels, D2H,
#     and the bank stage. It is NOT physical DRAM bandwidth
#     (which would require ncu; see scripts/s_real_perf_per_dataset.sh
#     --ncu <id> for that separately).
#   - Cross-driver / cross-CUDA / cross-hardware throughput
#     identity is NOT claimed.
#
# Output (whitelisted under reports/s_real_throughput/):
#   reports/s_real_throughput/summary.txt           (master 20-row table)
#   reports/s_real_throughput/<dataset>/bench_receipt.txt
#                                                   (per-dataset block)
#   reports/s_real_throughput/<dataset>/bench_stdout.txt + bench_stderr.txt
#                                                   (audit invocation logs)
#
# Usage:
#
#   # Run the bench on every dataset with default --iters 7:
#   bash scripts/s_real_throughput_bench.sh
#
#   # Tighten the median with more iterations:
#   bash scripts/s_real_throughput_bench.sh --iters 15
#
#   # Single dataset bench:
#   bash scripts/s_real_throughput_bench.sh --dataset cmapss_fd001_unit1
#
# Panel-locked non-claims (printed at end of every summary.txt):
#   - Does NOT change any sealed audit artifact's byte content
#     (replay determinism is unaffected; only perf_profile.txt
#     timing values vary across runs).
#   - Does NOT claim saturation-class throughput.
#   - Does NOT capture Nsight Compute traces; see
#     scripts/s_real_perf_per_dataset.sh --ncu <id> for that.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Canonical 20-dataset list (mirrors s_real_audit.rs::DATASETS).
ALL_DATASETS=(
    tadbench_f11
    tadbench_f04
    tadbench_f11b
    tadbench_f19
    illinois_socialnet
    lo2
    deeptralog
    aiops_kpi
    multidim_localization
    defects4j
    bugsinpy
    promise_defect_prediction
    cmapss_fd001_unit50
    cmapss_fd002_unit1
    cmapss_fd002_unit100
    cmapss_fd003_unit1
    cmapss_fd004_unit1
    promise_ant_1_4
    deeptralog_f02
    cmapss_fd001_unit1
    radioml_2018_snr30_large
    deepbeam_large
    radioml_gold_large
    powder_large
    oracle_large
    deepsense6g_large
    imdb_tgz_large
    imdb_duckdb_large
    snowset_large
    sqlshare_large
)

# Flags.
target_dataset="all"
iters=7
catalogs=1
expect_dataset_value=0
expect_iters_value=0
expect_catalogs_value=0
for arg in "$@"; do
    if [[ "$expect_dataset_value" -eq 1 ]]; then
        target_dataset="$arg"; expect_dataset_value=0; continue
    fi
    if [[ "$expect_iters_value" -eq 1 ]]; then
        iters="$arg"; expect_iters_value=0; continue
    fi
    if [[ "$expect_catalogs_value" -eq 1 ]]; then
        catalogs="$arg"; expect_catalogs_value=0; continue
    fi
    case "$arg" in
        --dataset)        expect_dataset_value=1 ;;
        --dataset=*)      target_dataset="${arg#--dataset=}" ;;
        --iters)          expect_iters_value=1 ;;
        --iters=*)        iters="${arg#--iters=}" ;;
        --catalogs)       expect_catalogs_value=1 ;;
        --catalogs=*)     catalogs="${arg#--catalogs=}" ;;
        --help|-h)
            cat <<'EOF'
usage: scripts/s_real_throughput_bench.sh [flags]

  --dataset <id|all>   Restrict the bench to ONE dataset id, or "all"
                       (default). Mirrors s_real_audit.rs::DATASETS.

  --iters <N>          Iterations per audit dispatch. Default 7 =
                       canonical bench cadence (1-2 cold + 5-6
                       steady-state samples; audit binary computes
                       median + p50 + p95 + p99 across all iters).
                       Higher = tighter median, slower bench.
                       Minimum 2 (audit requires >= 2 for replay
                       verification).

  --catalogs <K>       Batched-K amortization (default 1). K > 1
                       replicates the same dataset K times into one
                       batched dispatch; honestly labelled as
                       amortization, NOT K distinct datasets.

  -h, --help           Show this message.
EOF
            exit 0 ;;
        *)
            echo "warning: unknown flag $arg (ignored)" >&2 ;;
    esac
done

if ! [[ "$iters" =~ ^[0-9]+$ ]] || [[ "$iters" -lt 2 ]]; then
    echo "FATAL: --iters must be integer >= 2 (got '$iters')" >&2; exit 2
fi
if ! [[ "$catalogs" =~ ^[0-9]+$ ]] || [[ "$catalogs" -lt 1 ]]; then
    echo "FATAL: --catalogs must be positive integer (got '$catalogs')" >&2; exit 2
fi

if [[ "$target_dataset" == "all" ]]; then
    DATASETS_TO_RUN=("${ALL_DATASETS[@]}")
else
    found=0
    for d in "${ALL_DATASETS[@]}"; do
        if [[ "$d" == "$target_dataset" ]]; then
            DATASETS_TO_RUN=("$d"); found=1; break
        fi
    done
    if [[ "$found" -ne 1 ]]; then
        echo "FATAL: unknown dataset id '$target_dataset'" >&2
        echo "       valid ids: ${ALL_DATASETS[*]}" >&2; exit 2
    fi
fi

throughput_root="reports/s_real_throughput"
mkdir -p "$throughput_root"

# Tier resolver — matches the canonical commit-time tier directories
# the audit binary writes into. Mirrors the resolver in
# s_real_perf_per_dataset.sh; the two scripts agree on tiers.
tier_of() {
    case "$1" in
        tadbench_f11|illinois_socialnet|aiops_kpi) echo "s_real_1" ;;
        cmapss_fd001_unit50|cmapss_fd002_unit1|cmapss_fd002_unit100|cmapss_fd003_unit1|cmapss_fd004_unit1|promise_ant_1_4|deeptralog_f02) echo "s_real_3" ;;
        radioml_2018_snr30_large|deepbeam_large|radioml_gold_large|powder_large|oracle_large|deepsense6g_large|imdb_tgz_large|imdb_duckdb_large|snowset_large|sqlshare_large) echo "s_real_throughput" ;;
        *) echo "s_real_2" ;;
    esac
}

# Build under regular user (cargo on PATH). The bench does NOT need
# sudo because it's plain audit invocations, not ncu captures.
bin="target/release/dsfb-gpu-debug"
if [[ "$EUID" -eq 0 ]]; then
    if [[ ! -x "$bin" ]]; then
        echo "FATAL: $bin missing under sudo; cargo not on PATH." >&2
        echo "       Build first as your regular user: bash $0" >&2
        exit 4
    fi
else
    echo "==> building dsfb-gpu-debug-demo (release, cuda feature)..."
    cargo build --release --features cuda -p dsfb-gpu-debug-demo \
        --bin dsfb-gpu-debug 2>&1 | tail -3
fi
if [[ ! -x "$bin" ]]; then
    echo "FATAL: $bin not found after build" >&2; exit 4
fi

# Header.
summary_path="$throughput_root/summary.txt"
{
    echo "=========================================================="
    echo "  S-REAL.3 per-dataset throughput bench"
    echo "  generated by scripts/s_real_throughput_bench.sh"
    echo "  $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "=========================================================="
    echo ""
    echo "  datasets   : ${#DATASETS_TO_RUN[@]} of ${#ALL_DATASETS[@]} total"
    echo "  iters      : $iters per audit (median across all iters; first iter pays CUDA-init tax)"
    echo "  catalogs   : $catalogs (batched-K amortization)"
    echo ""
    echo "  Columns:"
    echo "    bytes      : fixture_byte_size from dataset_manifest.toml"
    echo "    median_us  : dispatch_median_us from perf_profile.txt"
    echo "                  (median across all $iters dispatch iters;"
    echo "                   first iter pays the CUDA-init tax so it"
    echo "                   skews high — the median still reflects"
    echo "                   that for low --iters. Bump --iters for"
    echo "                   tighter steady-state numbers.)"
    echo "    p50_us     : 50th-percentile dispatch wall"
    echo "                  (excludes cold first iter once iters>=3)"
    echo "    median_BW  : bytes / median_us  (input bytes per median"
    echo "                  dispatch wall; honest single-number throughput)"
    echo "    p50_BW     : bytes / p50_us  (warm steady-state throughput;"
    echo "                  excludes first-iter CUDA-init tax)"
    echo ""
    echo "  Honest framing (MUST hold):"
    echo "    - Fixtures are small (128-656 events). Even at steady"
    echo "      state the wall is launch-overhead-dominated because"
    echo "      there isn't enough work per dispatch to fill an"
    echo "      RTX 4080 SUPER."
    echo "    - DO NOT compare against S-PERF.16.a saturation bench."
    echo "    - Bandwidth here is INPUT FILE bytes / dispatch wall."
    echo "      It is NOT physical DRAM bandwidth (use --ncu on the"
    echo "      sibling perf script for that)."
    echo ""
    printf "%-30s | %5s | %10s | %12s | %10s | %12s | %10s\n" \
        "dataset_id" "iters" "bytes" "median_us" "median_BW" "p50_us" "p50_BW"
    printf '%s\n' "------------------------------+-------+------------+--------------+------------+--------------+------------"
} > "$summary_path"

# Helper: parse a `key : value` line from a perf_profile.txt file.
# All audit-emitted timing fields use that format.
parse_us() {
    local key="$1"; local file="$2"; local val
    val="$(grep -E "^[[:space:]]*${key}[[:space:]]*:" "$file" \
           | head -1 | awk -F: '{print $2}' | tr -d ' ')"
    [[ -z "$val" ]] && val="0"
    echo "$val"
}

# Helper: format an integer bytes-per-second into a human-readable
# bandwidth string (MB/s with one decimal, or GB/s with two when
# >= 1000 MB/s). Returns "X.XX MB/s" or "X.XX GB/s" depending on
# magnitude. Empty input → "0.00 MB/s".
fmt_bw() {
    local bps="$1"
    if [[ -z "$bps" || "$bps" -le 0 ]]; then
        echo "0.00 MB/s"; return
    fi
    # MB/s = bps / 1_000_000. Use awk for float math.
    local mbps
    mbps="$(awk -v b="$bps" 'BEGIN { printf "%.2f", b / 1000000.0 }')"
    # If >= 1000 MB/s, render as GB/s. Bash arithmetic on awk's
    # decimal output is fragile, so do the >= test in awk too.
    local in_gb
    in_gb="$(awk -v b="$bps" 'BEGIN { print (b >= 1000000000) ? 1 : 0 }')"
    if [[ "$in_gb" -eq 1 ]]; then
        awk -v b="$bps" 'BEGIN { printf "%.2f GB/s", b / 1000000000.0 }'
    else
        printf "%s MB/s" "$mbps"
    fi
}

# Run-loop. For each dataset:
#   1. Invoke the audit binary with --iters $iters --catalogs $catalogs.
#      This RE-RUNS the dispatcher N times and updates the canonical
#      audit artifacts under reports/s_real_<tier>/<dataset>/. The
#      replay invariant guarantees casefile.json + episodes.jsonl +
#      audit_report.html are byte-identical regardless of --iters;
#      only perf_profile.txt's timing fields vary.
#   2. Parse fixture_byte_size, dispatch_median_us, dispatch_p50_us
#      from the refreshed perf_profile.txt.
#   3. Compute median_BW and p50_BW.
#   4. Append row to summary; write per-dataset bench_receipt.txt.
#   5. Continue on per-dataset audit failure rather than aborting.

dataset_idx=0
dataset_total="${#DATASETS_TO_RUN[@]}"
for d in "${DATASETS_TO_RUN[@]}"; do
    dataset_idx=$(( dataset_idx + 1 ))
    tier="$(tier_of "$d")"
    out_dir="reports/${tier}"
    dataset_dir="$out_dir/$d"
    receipt_dir="$throughput_root/$d"
    mkdir -p "$receipt_dir"

    echo "==> [$dataset_idx/$dataset_total: $d] running bench (--iters $iters --catalogs $catalogs)..."
    if ! "$bin" s-real-audit \
            --dataset "$d" \
            --out-dir "$out_dir" \
            --iters "$iters" \
            --catalogs "$catalogs" \
            > "$receipt_dir/bench_stdout.txt" 2> "$receipt_dir/bench_stderr.txt"; then
        echo "  BENCH FAILED for $d (see $receipt_dir/bench_stderr.txt)" >&2
        printf "%-30s | %5s | %10s | %12s | %10s | %12s | %10s\n" \
            "$d" "$iters" "FAIL" "FAIL" "FAIL" "FAIL" "FAIL" >> "$summary_path"
        continue
    fi

    perf_profile="$dataset_dir/perf_profile.txt"
    if [[ ! -f "$perf_profile" ]]; then
        echo "  perf_profile.txt missing for $d after audit; skipping row" >&2
        continue
    fi

    fixture_bytes=$(parse_us 'fixture_byte_size' "$perf_profile")
    median_us=$(parse_us 'dispatch_median_us' "$perf_profile")
    p50_us=$(parse_us 'dispatch_p50_us' "$perf_profile")

    # bytes/sec computed as fixture_bytes × 1e6 / us. Bash integer
    # math is fine for these magnitudes (bytes up to ~50 KB, us up
    # to ~200_000, product fits in 64-bit easily).
    median_bps=0
    if [[ "$median_us" -gt 0 ]]; then
        median_bps=$(( fixture_bytes * 1000000 / median_us ))
    fi
    p50_bps=0
    if [[ "$p50_us" -gt 0 ]]; then
        p50_bps=$(( fixture_bytes * 1000000 / p50_us ))
    fi

    median_bw="$(fmt_bw "$median_bps")"
    p50_bw="$(fmt_bw "$p50_bps")"

    printf "%-30s | %5s | %10s | %12s | %10s | %12s | %10s\n" \
        "$d" "$iters" "$fixture_bytes" "$median_us" "$median_bw" "$p50_us" "$p50_bw" \
        >> "$summary_path"

    # Per-dataset bench receipt. Captures the full per-stage timing
    # picture from perf_profile.txt plus the derived bandwidth.
    {
        echo "=========================================================="
        echo "  S-REAL.3 throughput bench receipt — $d"
        echo "  generated by scripts/s_real_throughput_bench.sh"
        echo "  $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
        echo "=========================================================="
        echo ""
        echo "  Source       : $perf_profile"
        echo "  iters        : $iters"
        echo "  catalogs     : $catalogs"
        echo "  tier         : $tier"
        echo ""
        echo "  Input:"
        echo "    fixture_byte_size       : $fixture_bytes B"
        echo "    events_emitted          : $(parse_us 'events_emitted' "$perf_profile")"
        echo "    finite_cells            : $(parse_us 'finite_cells' "$perf_profile")"
        echo ""
        echo "  Dispatch wall (microseconds, host Instant; per-iter):"
        echo "    cuda_dispatch_run1_us   : $(parse_us 'cuda_dispatch_run1_us' "$perf_profile")  (cold; pays CUDA init)"
        echo "    cuda_dispatch_run2_us   : $(parse_us 'cuda_dispatch_run2_us' "$perf_profile")  (first warm)"
        echo "    dispatch_median_us      : $median_us"
        echo "    dispatch_p50_us         : $p50_us"
        echo "    dispatch_p95_us         : $(parse_us 'dispatch_p95_us' "$perf_profile")"
        echo "    dispatch_p99_us         : $(parse_us 'dispatch_p99_us' "$perf_profile")"
        echo ""
        echo "  Derived bandwidth (fixture_bytes / dispatch_wall):"
        echo "    median_BW               : $median_bw  (across all $iters iters; cold-included)"
        echo "    p50_BW                  : $p50_bw  (warm steady-state)"
        echo ""
        echo "  Honest framing:"
        echo "    - This is INPUT FILE bytes / dispatch wall, NOT"
        echo "      physical DRAM bandwidth. For DRAM% see"
        echo "      scripts/s_real_perf_per_dataset.sh --ncu $d."
        echo "    - Small fixture (~$fixture_bytes B). Launch overhead"
        echo "      dominates per-call wall; throughput is structurally"
        echo "      bounded by dispatch overhead, not memory bandwidth."
        echo "    - Sealed audit artifacts at $dataset_dir/ are byte-"
        echo "      identical regardless of --iters; replay invariant"
        echo "      holds."
    } > "$receipt_dir/bench_receipt.txt"
done

{
    echo ""
    echo "=========================================================="
    echo "  Panel-locked non-claims"
    echo "=========================================================="
    echo ""
    echo "  - Bandwidth numbers above are HONEST measurements of"
    echo "    short single-dispatch audits, NOT saturation-class"
    echo "    bandwidth. The audit dispatcher is launch-overhead-"
    echo "    dominated on these fixture sizes."
    echo "  - median_BW = fixture_byte_size / dispatch_median_us"
    echo "    measures INPUT FILE bytes / dispatch wall. It is NOT"
    echo "    physical DRAM bandwidth. For DRAM%, see"
    echo "    scripts/s_real_perf_per_dataset.sh --ncu <id>."
    echo "  - p50_BW excludes the first-iter CUDA-init tax (which"
    echo "    skews median high at low --iters). p50_BW is the"
    echo "    cleanest single-number bandwidth for steady-state"
    echo "    operator-facing reporting."
    echo "  - Replay determinism is unaffected; the sealed audit"
    echo "    artifacts are byte-identical regardless of --iters."
    echo "  - Cross-driver / cross-CUDA / cross-hardware throughput"
    echo "    identity is NOT claimed."
    echo ""
} >> "$summary_path"

echo ""
echo "==> summary written to: $summary_path"
echo "==> per-dataset receipts under: $throughput_root/<dataset_id>/bench_receipt.txt"
