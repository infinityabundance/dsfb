#!/usr/bin/env bash
# S-PERF.ROOF-PREFLIGHT — Nsight Compute roofline measurement of
# the post-S-PERF.13 D64 compact-densor throughput pipeline.
#
# Panel-locked thesis (2026-05-18): the bench's `wide bytes/sec`
# metric is a logical byte count (264 B/cell × cells/sec), NOT
# physical DRAM / L2 / SM / PCIe / hash-pipeline traffic. Before
# any S-PERF.14+ device-side optimization, we MUST measure the
# actual fabric utilization so the next campaign attacks the real
# wall rather than a logical metric.
#
# This script:
#   1. Builds the S-PERF.12 D64 compact-densor stage-profile bench
#      in --release with `-C target-feature=+avx2` (matches the
#      S-PERF.13 production code path).
#   2. Invokes `ncu --set detailed` against the bench binary,
#      restricted via `--launch-skip` + `--launch-count` to a
#      single measurement iteration (skip 3 warmup × 12 kernels =
#      36 launches; profile next 12 kernels).
#   3. Writes the CSV metric matrix to
#      `reports/_ncu_d64_compact_densor_metrics.csv` (gitignored).
#   4. Builds a human-readable summary at
#      `reports/s_perf_roof_preflight_d64_nsight_metrics.txt`
#      with per-kernel SpeedOfLight metrics (DRAM throughput,
#      SM throughput, L2 throughput, achieved occupancy) and a
#      panel-locked decision-rule classification.
#
# Usage:
#
#   sudo bash scripts/s_perf_roof_preflight.sh
#
# `sudo` is REQUIRED because NVIDIA driver gates GPU performance
# counters (ERR_NVGPUCTRPERM) from non-root users by default.
# This is one-shot per measurement event; persistent fix is to
# set `options nvidia NVreg_RestrictProfilingToAdminUsers=0` in
# /etc/modprobe.d/nvidia.conf and reload the driver.
#
# Output artifacts (S-PERF.13-PREFLIGHT-style; one-shot audit
# records, no hash chain):
#
#   reports/_ncu_d64_compact_densor_metrics.csv  (gitignored;
#                                                 ncu raw CSV)
#   reports/s_perf_roof_preflight_d64_nsight_metrics.txt
#                                                (whitelisted;
#                                                 the receipt
#                                                 the S-PERF.14
#                                                 commit cites)
#
# Panel-locked non-claims (printed at end):
#   - Does NOT change any kernel.
#   - Does NOT claim bandwidth saturation; measures it.
#   - Does NOT add Rust dependencies.
#   - Does NOT mutate any hash anchor.
#   - Does NOT pre-commit to any S-PERF.14 design.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Flags. `--no-ncu` reuses the existing CSV at
# reports/_ncu_d64_compact_densor_metrics.csv and only
# re-runs the receipt-build step. Useful for iterating on
# the receipt format without burning GPU time.
skip_ncu=0
tag=""
kernel_filter=""
launch_count=65   # v4 default: 5 pipeline iterations per ROOF (panel-locked
                  # post-S-PERF.14b.1 HARD ENFORCEMENT cadence; was 13 = 1
                  # iteration; bumped so each sudo ROOF yields 5 measurements
                  # per kernel per stage, surfacing within-run thermal-noise
                  # band data without requiring 5 separate sudo invocations).
expect_tag_value=0
expect_kernel_filter_value=0
expect_launch_count_value=0
for arg in "$@"; do
    if [[ "$expect_tag_value" -eq 1 ]]; then
        tag="$arg"
        expect_tag_value=0
        continue
    fi
    if [[ "$expect_kernel_filter_value" -eq 1 ]]; then
        kernel_filter="$arg"
        expect_kernel_filter_value=0
        continue
    fi
    if [[ "$expect_launch_count_value" -eq 1 ]]; then
        launch_count="$arg"
        expect_launch_count_value=0
        continue
    fi
    case "$arg" in
        --no-ncu|--skip-ncu)
            skip_ncu=1 ;;
        --tag)
            expect_tag_value=1 ;;
        --tag=*)
            tag="${arg#--tag=}" ;;
        --kernel-filter)
            expect_kernel_filter_value=1 ;;
        --kernel-filter=*)
            kernel_filter="${arg#--kernel-filter=}" ;;
        --launch-count)
            expect_launch_count_value=1 ;;
        --launch-count=*)
            launch_count="${arg#--launch-count=}" ;;
        --help|-h)
            echo "usage: $0 [--no-ncu] [--tag NAME] [--kernel-filter NAME] [--launch-count N]"
            echo "  --no-ncu              : reuse existing CSV; only re-emit the receipt"
            echo "  --tag NAME            : suffix output paths with _NAME to keep"
            echo "                          multiple per-campaign ROOF receipts side-by-side."
            echo "                          e.g. --tag s_perf_14b_post"
            echo "  --kernel-filter NAME  : capture only kernels whose name matches NAME"
            echo "                          (passed to ncu as --kernel-id ::regex:NAME:).  Use"
            echo "                          this for per-kernel re-runs robust against"
            echo "                          iteration-shape drift; replaces the blind"
            echo "                          --launch-skip / --launch-count window."
            echo "  --launch-count N      : number of kernel launches to profile."
            echo "                          Default 65 = 5 iterations × 13 kernels (v4 HARD"
            echo "                          ENFORCEMENT multi-run cadence). Use 13 for 1-iter"
            echo "                          legacy behaviour; bump higher for more iterations."
            exit 0 ;;
        *)
            echo "warning: unknown arg $arg (ignored)" >&2 ;;
    esac
done

# Validate launch_count is a positive integer.
if ! [[ "$launch_count" =~ ^[0-9]+$ ]] || [[ "$launch_count" -lt 1 ]]; then
    echo "FATAL: --launch-count must be a positive integer (got '$launch_count')" >&2
    exit 2
fi

# Sanitize tag (alnum + underscore only) so it composes into
# safe file paths.
if [[ -n "$tag" ]]; then
    if ! [[ "$tag" =~ ^[A-Za-z0-9_]+$ ]]; then
        echo "FATAL: --tag must be [A-Za-z0-9_]+ (got '$tag')" >&2
        exit 2
    fi
    tag_suffix="_${tag}"
else
    tag_suffix=""
fi

# Tools.
ncu_bin="${NCU_BIN:-/usr/bin/ncu}"
cuda_bin="${CUDA_BIN:-/opt/cuda/bin}"
export PATH="$cuda_bin:$PATH"

if [[ ! -x "$ncu_bin" ]]; then
    echo "FATAL: ncu not found at $ncu_bin" >&2
    echo "       set NCU_BIN env var or install nsight-compute" >&2
    exit 2
fi

echo "=== S-PERF.ROOF-PREFLIGHT ==="
echo "ncu_bin    : $ncu_bin"
"$ncu_bin" --version 2>&1 | head -3
echo ""

# Step 1 — locate or build the bench binary in release + AVX2.
# We prefer to reuse an existing binary (the typical case after
# a recent `cargo test --release` invocation); the rebuild is
# skipped to avoid sudo+cargo permission complications.
echo "=== locate/build bench binary (release + AVX2) ==="

bench_bin="$(ls -t target/release/deps/s_perf_12_d64_compact_densor_stage_profile-* \
    2>/dev/null \
    | grep -v '\.d$' \
    | head -1)"

if [[ -z "$bench_bin" || ! -x "$bench_bin" ]]; then
    # Need to build. If invoked under sudo, drop privileges for
    # the cargo build so target/ stays user-owned. Resolve cargo
    # via the standard rustup install location since `sudo -u`
    # strips the user's PATH.
    echo "  no existing bench binary found; running cargo test --no-run"
    sudo_user="${SUDO_USER:-$(id -un)}"
    user_home="$(getent passwd "$sudo_user" | cut -d: -f6)"
    cargo_bin="${CARGO_BIN:-$user_home/.cargo/bin/cargo}"
    if [[ ! -x "$cargo_bin" ]]; then
        echo "FATAL: cargo not found at $cargo_bin" >&2
        echo "       set CARGO_BIN env var or install rustup" >&2
        exit 2
    fi
    build_cmd=(env "RUSTFLAGS=-C target-feature=+avx2" \
        "PATH=$user_home/.cargo/bin:$cuda_bin:/usr/bin:/bin" \
        "$cargo_bin" test \
        -p dsfb-gpu-debug-demo \
        --features cuda \
        --release \
        --test s_perf_12_d64_compact_densor_stage_profile \
        --no-run)
    if [[ "${EUID:-$(id -u)}" -eq 0 && -n "${SUDO_USER:-}" ]]; then
        echo "  (dropping privileges to $SUDO_USER for cargo build)"
        sudo -u "$SUDO_USER" "${build_cmd[@]}" 2>&1 | tail -5
    else
        "${build_cmd[@]}" 2>&1 | tail -5
    fi
    bench_bin="$(ls -t target/release/deps/s_perf_12_d64_compact_densor_stage_profile-* \
        2>/dev/null \
        | grep -v '\.d$' \
        | head -1)"
fi

if [[ -z "$bench_bin" || ! -x "$bench_bin" ]]; then
    echo "FATAL: bench binary not found under target/release/deps/" >&2
    echo "       run \`cargo test -p dsfb-gpu-debug-demo --features cuda --release \\" >&2
    echo "             --test s_perf_12_d64_compact_densor_stage_profile --no-run\`" >&2
    echo "       before this script" >&2
    exit 2
fi
echo "bench_bin  : $bench_bin"
echo ""

# Step 2 — invoke ncu against the bench binary.
#
# `--set detailed` gives us SpeedOfLight (DRAM / SM / L2
# throughput), Occupancy, MemoryWorkloadAnalysis, ComputeWorkload
# Analysis, WorkloadDistribution, and Roofline metrics — 1007
# metrics per kernel launch. This is the right cut for the
# decision-rule table without paying for `full`'s 7314-metric
# overhead.
#
# `--launch-skip 39` skips the 3 warmup iterations × 13 kernels
# per iteration = 39 launches, so the metric collection starts on
# the first MEASUREMENT iteration's first kernel. The 13-kernel
# count is post-S-PERF.14a: the pre-S-PERF.14a pipeline had 12
# kernels per iteration; S-PERF.14a split drift_slew_sign into a
# Pre-Alpha (drift_ewma_precompute_kernel) + cellpar
# (drift_slew_sign_kernel_cellpar) pair, adding one launch per
# iteration. The S-PERF.14a post-receipt was captured against the
# stale 36/12 calibration which is the source of its
# capture-window alignment caveat; this script's calibration is
# now correct for the post-S-PERF.14a iteration shape.
#
# `--launch-count 13` limits collection to one pipeline iteration
# (13 kernels). ncu replays each kernel multiple times to gather
# the full metric set deterministically; limiting to 1 iteration
# keeps the total runtime under ~5 minutes on RTX 4080 SUPER.
#
# `--csv --log-file` emits parseable CSV that the receipt-build
# step below converts into the human-readable summary.

csv_out="reports/_ncu_d64_compact_densor_metrics${tag_suffix}.csv"

if [[ "$skip_ncu" -eq 1 ]]; then
    echo "=== --no-ncu: reusing existing CSV ==="
    if [[ ! -s "$csv_out" ]]; then
        echo "FATAL: $csv_out is missing or empty; cannot --no-ncu" >&2
        exit 2
    fi
    echo "  CSV: $csv_out ($(wc -l < "$csv_out") lines)"
    echo ""
else
    if [[ -n "$kernel_filter" ]]; then
        echo "=== ncu run (sudo required) ==="
        echo "  --set detailed --kernel-id ::regex:$kernel_filter:"
        echo "  output: $csv_out"
        echo "  (per-kernel filter mode; --launch-skip / --launch-count not used)"
        echo "  expected runtime: 2-5 minutes on RTX 4080 SUPER"
        echo ""
        "$ncu_bin" \
            --set detailed \
            --kernel-id "::regex:${kernel_filter}:" \
            --csv \
            --log-file "$csv_out" \
            --target-processes all \
            "$bench_bin" \
            --ignored s_perf_12_d64_compact_densor_stage_profile_256x4096_k1 \
            2>&1 | tail -20
    else
        echo "=== ncu run (sudo required) ==="
        echo "  --set detailed --launch-skip 39 --launch-count $launch_count"
        echo "  output: $csv_out"
        echo "  expected runtime: 2-10 minutes on RTX 4080 SUPER (scales with launch_count)"
        echo ""

        # Note: ncu invokes the binary with positional args following
        # the binary path. The bench's #[ignore] attribute requires the
        # `--ignored` flag + the test name. Launch-skip 39 = 3 warmup ×
        # 13 kernels-per-iteration; --launch-count default 65 = 5 full
        # iterations (panel-locked v4 HARD ENFORCEMENT cadence per
        # post-S-PERF.14b.1-v3 reframing — single-iter capture is
        # blind to within-run thermal-noise band).
        "$ncu_bin" \
            --set detailed \
            --launch-skip 39 \
            --launch-count "$launch_count" \
            --csv \
            --log-file "$csv_out" \
            --target-processes all \
            "$bench_bin" \
            --ignored s_perf_12_d64_compact_densor_stage_profile_256x4096_k1 \
            2>&1 | tail -20
    fi

    echo ""
    echo "ncu run complete. CSV at: $csv_out"
    echo ""
fi

# Step 3 — build the human-readable receipt. The receipt is
# plain text; it scrapes the per-kernel SpeedOfLight numbers and
# applies the panel-locked decision-rule table to classify the
# dominant bottleneck.

receipt="reports/s_perf_roof_preflight_d64_nsight_metrics${tag_suffix}.txt"
echo "=== building receipt: $receipt ==="

python3 - <<'PYEOF' "$csv_out" "$receipt"
import csv
import sys
import os
from collections import OrderedDict, defaultdict

csv_path = sys.argv[1]
receipt_path = sys.argv[2]

if not os.path.exists(csv_path):
    print(f"FATAL: ncu CSV not found: {csv_path}", file=sys.stderr)
    sys.exit(2)

# ncu CSV row layout (verified against actual 2026.1.1.0
# `--set detailed` output): each row is one
# (kernel-instance × section × metric) sample. The columns are:
#   ID, Process ID, Process Name, Host Name, Kernel Name,
#   Context, Stream, Block Size, Grid Size, Device, CC,
#   Section Name, Metric Name, Metric Unit, Metric Value,
#   Rule Name, Rule Type, Rule Description,
#   Estimated Speedup Type, Estimated Speedup
#
# The ncu run also emits "==PROF==" banner lines before the
# header — skip those. We key by Kernel Name only (the
# compact-densor pipeline launches each kernel once per
# pipeline iteration; ncu replays for metric collection but
# emits one row per (kernel-instance × metric).

# Read entire CSV via csv.reader so multi-line quoted fields
# (ncu Rule Description sometimes spans multiple lines) parse
# correctly. Skip lines starting with the ==PROF== banner.
with open(csv_path, "r", encoding="utf-8", errors="replace") as fh:
    body = "".join(
        line for line in fh
        if not line.startswith("==PROF==")
    )
reader = list(csv.reader(body.splitlines()))
header = None
rows = []
for rec in reader:
    if not rec:
        continue
    if header is None and rec and rec[0] == "ID":
        header = rec
        continue
    if header is None:
        continue
    # ncu omits the trailing 5 Rule-related columns on ordinary
    # metric rows, so most rows have 15-16 fields against a
    # 20-column header. Pad short rows with empty strings (so
    # the (kernel × metric) data still lands in by_kernel) and
    # ignore rows that are too short to carry a kernel name.
    if len(rec) < 15:
        continue
    if len(rec) < len(header):
        rec = rec + [""] * (len(header) - len(rec))
    elif len(rec) > len(header):
        rec = rec[: len(header)]
    rows.append(dict(zip(header, rec)))

print(f"  parsed {len(rows)} metric rows; header={len(header) if header else 0} cols", file=sys.stderr)

# Index by kernel; each kernel collects a flat dict of
# `Metric Name` -> (value, unit). Insights (rows with no
# Metric Name but a Rule Description) are collected separately.
by_kernel = defaultdict(dict)         # kname -> {mname: (value, unit)}
insights_by_kernel = defaultdict(list)  # kname -> [rule descriptions]
for r in rows:
    k = r.get("Kernel Name", "(unknown)")
    m = r.get("Metric Name", "")
    v = r.get("Metric Value", "")
    u = r.get("Metric Unit", "")
    rule_desc = r.get("Rule Description", "")
    if m:
        by_kernel[k][m] = (v, u)
    elif rule_desc:
        insights_by_kernel[k].append(rule_desc)

# Panel-locked metric set — exact `--set detailed` Metric Name
# strings ncu emits.
panel_metrics = OrderedDict([
    # SpeedOfLight section
    ("DRAM Throughput",          "% peak"),
    ("L1/TEX Cache Throughput",  "% peak"),
    ("L2 Cache Throughput",      "% peak"),
    ("Compute (SM) Throughput",  "% peak"),
    ("Memory Throughput",        "% peak"),
    ("Duration",                 "ns"),
    ("Elapsed Cycles",           "cycle"),
    # Occupancy section
    ("Achieved Occupancy",       "%"),
    ("Theoretical Occupancy",    "%"),
    ("Achieved Active Warps Per SM", "warp"),
    # ComputeWorkloadAnalysis
    ("Executed Ipc Active",      "inst/cycle"),
    ("Executed Ipc Elapsed",     "inst/cycle"),
    # MemoryWorkloadAnalysis
    ("Memory Throughput",        "% peak"),
])

# Deduplicate keys (Memory Throughput appears twice).
panel_keys = list(OrderedDict.fromkeys(panel_metrics.keys()))

def lookup(metric_dict, key):
    return metric_dict.get(key, ("(missing)", ""))

def as_float(value):
    if value is None:
        return None
    s = str(value).replace(",", "").replace("%", "").strip()
    if not s or s == "(missing)":
        return None
    try:
        return float(s)
    except ValueError:
        return None

# Classify per-kernel bottleneck via the panel-locked rule
# table. Thresholds match the plan's decision-rule table.
def classify(metrics):
    dram, _ = lookup(metrics, "DRAM Throughput")
    l2,   _ = lookup(metrics, "L2 Cache Throughput")
    sm,   _ = lookup(metrics, "Compute (SM) Throughput")
    occ,  _ = lookup(metrics, "Achieved Occupancy")
    l1,   _ = lookup(metrics, "L1/TEX Cache Throughput")
    dram_v = as_float(dram)
    l2_v   = as_float(l2)
    sm_v   = as_float(sm)
    occ_v  = as_float(occ)
    l1_v   = as_float(l1)
    # Panel-locked decision rule (post-S-PERF.13 ROOF spec):
    #   DRAM-bound if DRAM >= 70% peak
    #   L2-bound if DRAM low + L2 >= 70%
    #   SM/compute-bound if SM >= 70% + low DRAM/L2
    #   occupancy-bound if Achieved Occupancy < 50%
    #   otherwise UNCLASSIFIED
    if dram_v is not None and dram_v >= 70.0:
        return "DRAM-bound", f"DRAM={dram_v:.1f}%"
    if l2_v is not None and l2_v >= 70.0:
        return "L2-bound", f"L2={l2_v:.1f}% (DRAM={dram_v if dram_v is not None else '?'}%)"
    if sm_v is not None and sm_v >= 70.0:
        return "SM/compute-bound", f"SM={sm_v:.1f}%"
    if occ_v is not None and occ_v < 50.0:
        return "occupancy-bound", f"Occ={occ_v:.1f}%"
    # Mid-range: classify by the largest fabric, even if below 70%.
    candidates = []
    if dram_v is not None: candidates.append(("DRAM-near", dram_v, f"DRAM={dram_v:.1f}%"))
    if l2_v is not None:   candidates.append(("L2-near",   l2_v,   f"L2={l2_v:.1f}%"))
    if sm_v is not None:   candidates.append(("SM-near",   sm_v,   f"SM={sm_v:.1f}%"))
    if candidates:
        candidates.sort(key=lambda x: -x[1])
        bucket, val, why = candidates[0]
        return bucket, why
    return "UNCLASSIFIED", f"DRAM={dram} L2={l2} SM={sm} Occ={occ}"

# Build the receipt.
lines = []
lines.append("=" * 72)
lines.append(" S-PERF.ROOF-PREFLIGHT — D64 compact-densor Nsight Compute receipt")
lines.append(" (panel-verified post-S-PERF.13; one-shot audit; no hash chain)")
lines.append("=" * 72)
lines.append("")
lines.append("Hardware: RTX 4080 SUPER, CUDA 13.2, Nsight Compute 2026.1.1.0")
lines.append("Bench:    s_perf_12_d64_compact_densor_stage_profile_256x4096_k1")
lines.append("Profile:  --set detailed --launch-skip 36 --launch-count 12")
lines.append("          (skip 3 warmup x 12 kernels = 36 launches; profile")
lines.append("          next 12 kernels = one measurement iteration)")
lines.append(f"Source:   {os.path.basename(csv_path)}")
lines.append("")
lines.append("Panel-locked thesis: the bench's `wide bytes/sec` metric is a")
lines.append("LOGICAL byte count (264 B/cell x cells/sec), NOT physical")
lines.append("fabric traffic. This receipt names the ACTUAL saturated fabric.")
lines.append("")

# Per-kernel matrix.
kernel_names = list(by_kernel.keys())
lines.append("-" * 72)
lines.append(" Per-kernel SpeedOfLight matrix (launch order; one row per metric)")
lines.append("-" * 72)
lines.append("")
short_keys = [
    "Duration",
    "DRAM Throughput",
    "L2 Cache Throughput",
    "L1/TEX Cache Throughput",
    "Compute (SM) Throughput",
    "Memory Throughput",
    "Achieved Occupancy",
    "Theoretical Occupancy",
    "Executed Ipc Active",
]
header_row = f"  {'kernel':46s} | {'Dur(ns)':>8s} | {'DRAM%':>6s} | {'L2%':>5s} | {'L1%':>5s} | {'SM%':>5s} | {'Occ%':>5s} | {'Bucket':>10s}"
lines.append(header_row)
lines.append("  " + "-" * (len(header_row) - 2))
for kname in kernel_names:
    metrics = by_kernel[kname]
    short = kname.split("(", 1)[0].strip()[:46]

    def fmt(key, width=6, decimals=1):
        v, _ = lookup(metrics, key)
        f = as_float(v)
        if f is None:
            return f"{'?':>{width}s}"
        return f"{f:>{width}.{decimals}f}"

    bucket, _ = classify(metrics)
    bucket_short = {
        "DRAM-bound": "DRAM!",
        "L2-bound": "L2!",
        "SM/compute-bound": "SM!",
        "occupancy-bound": "OCC!",
        "DRAM-near": "DRAM",
        "L2-near": "L2",
        "SM-near": "SM",
        "UNCLASSIFIED": "?",
    }.get(bucket, bucket[:10])
    lines.append(
        f"  {short:46s} | {fmt('Duration', 8, 0)} | "
        f"{fmt('DRAM Throughput')} | {fmt('L2 Cache Throughput', 5)} | "
        f"{fmt('L1/TEX Cache Throughput', 5)} | {fmt('Compute (SM) Throughput', 5)} | "
        f"{fmt('Achieved Occupancy', 5)} | {bucket_short:>10s}"
    )
lines.append("")

# Per-kernel detail (full metrics + ncu insights).
lines.append("-" * 72)
lines.append(" Per-kernel detail (full metrics + ncu rule insights)")
lines.append("-" * 72)
for kname in kernel_names:
    metrics = by_kernel[kname]
    insights = insights_by_kernel.get(kname, [])
    lines.append("")
    lines.append(f"kernel: {kname[:140]}")
    for key in panel_keys:
        v, u = lookup(metrics, key)
        if v == "(missing)":
            continue
        lines.append(f"  {key:30s} = {v} {u}")
    bucket, why = classify(metrics)
    lines.append(f"  CLASSIFICATION: {bucket}  ({why})")
    if insights:
        lines.append(f"  ncu insights:")
        for ins in insights[:4]:
            lines.append(f"    - {ins[:140]}")

# Aggregate.
lines.append("")
lines.append("-" * 72)
lines.append(" Aggregate bottleneck classification (per-kernel)")
lines.append("-" * 72)
buckets = defaultdict(int)
for kname in kernel_names:
    bucket, _ = classify(by_kernel[kname])
    buckets[bucket] += 1
for bucket, count in sorted(buckets.items(), key=lambda x: -x[1]):
    lines.append(f"  {bucket:30s} : {count} kernel(s)")

# Recommended S-PERF.14 lever.
lines.append("")
lines.append("-" * 72)
lines.append(" Recommended S-PERF.14 lever (panel-locked decision rule)")
lines.append("-" * 72)
recommendations = {
    "DRAM-bound":          "Reduce DRAM traffic: layout change, compact-projection refinement, or non-temporal stores. The structural floor is set by source-byte read volume.",
    "L2-bound":            "Improve L2 locality: kernel fusion + shared-memory tiling. Adjacent stages can share a single L2-resident pass.",
    "SM/compute-bound":    "Reduce integer-ALU / hash-pipeline work: vectorized hash primitives, algorithmic compression of the digest topology.",
    "occupancy-bound":     "Reduce register pressure or shrink block size to admit more concurrent warps.",
    "DRAM-near":           "Approach DRAM ceiling: investigate streaming/non-temporal stores; consider layout-change before kernel-fusion.",
    "L2-near":             "Near L2 ceiling: kernel fusion is the right lever; share L2-resident state across adjacent stages.",
    "SM-near":             "Near SM ceiling: vectorize hash math; consider integer-ALU compression.",
    "UNCLASSIFIED":        "Multi-front: collect `ncu --set full` warp-stall breakdown for finer attribution.",
}
dominant = max(buckets.items(), key=lambda x: x[1])[0] if buckets else "UNCLASSIFIED"
lines.append(f"  Dominant bucket: {dominant}")
lines.append(f"  Lever:           {recommendations.get(dominant, '(unknown)')}")

# Per-kernel quick recommendation. The generic bucket map
# captures the default lever; the kernel-specific overrides
# (KERNEL_SPECIFIC_RECOMMENDATIONS) replace the generic text
# for kernels whose true bottleneck is named in the plan,
# preventing misleading "shrink register pressure" hints on
# kernels where the actual issue is launch geometry or a
# structural recurrence floor by design.
KERNEL_SPECIFIC_RECOMMENDATIONS = {
    # S-PERF.14b target: one-thread/block serial SHA over
    # [header || leaves]. Tree-Merkle aggregation would change
    # root bytes -> CompactDensorDigestV2. Within the byte-
    # identity contract the legal fix is exact-stream
    # cooperative staging + single-thread SHA (Path 1a).
    "compact_densor_digest_v1_root_kernel": (
        "replace one-thread/block root reduction with "
        "deterministic parallel scratch staging + single-"
        "thread SHA preserving byte-identical "
        "CompactDensorDigestV1 root bytes (Path 1a; "
        "see plan)"
    ),
    # S-PERF.14b post-swap kernel: cooperative scratch
    # staging (Phase 1 header + Phase 2 byte-granular
    # cooperative leaf copy + Phase 3 single-thread SHA).
    # Per-call wall dropped 2.38 ms -> 0.82 ms (-65%) at
    # canonical 256x4096 K=1 D64 (post-S-PERF.14b ROOF
    # measurement). Occupancy stays low at K=1 because
    # only one block runs (structural limit; multi-catalog
    # K would amortize). Wall-time lever from here requires
    # streaming SHA-256 init/update/finalize (Path 1b;
    # deferred to S-PERF.14b.1 follow-on).
    "compact_densor_digest_v1_root_kernel_blockcoop": (
        "post-S-PERF.14b cooperative-staging kernel; "
        "per-call wall already -65% vs legacy single-thread "
        "(2.38 ms -> 0.82 ms); occupancy structurally limited "
        "at K=1 (one block runs); next lever is streaming "
        "SHA-256 (Path 1b -> S-PERF.14b.1)"
    ),
    # TreeSha256V1 mirror; same single-thread root pattern,
    # different canonical header. Deferred to a follow-on
    # commit (S-PERF.11b mirror) once Path 1a validates.
    "tree_digest_root_kernel": (
        "same Path 1a launch-geometry pattern as S-PERF.14b "
        "(deferred follow-on; TreeSha256V1 not active in "
        "production default path)"
    ),
    # S-PERF.14a Pre-Alpha kernel: one thread per entity by
    # design (serial EWMA recurrence). Low Occ here is the
    # structural recurrence floor, not a launch-geometry
    # failure. Do not 'shrink register pressure' here.
    "drift_ewma_precompute_kernel": (
        "structural recurrence floor (one thread per entity "
        "by design for serial EWMA carry); low Occ here is "
        "NOT a launch-geometry failure"
    ),
    # S-PERF.14c Pre-Alpha kernel: same shape as S-PERF.14a's
    # drift EWMA precompute — one thread per entity by design
    # (serial fired[] walk producing run boundaries with
    # length + max-per-entity filter). Low Occ here is the
    # structural recurrence floor; the wall-time win comes
    # from the cellpar emit kernel that publishes surviving
    # runs into the legacy boundaries[] slot table.
    "candidate_boundary_precompute_kernel": (
        "S-PERF.14c Pre-Alpha kernel: structural recurrence "
        "floor (one thread per entity by design for serial "
        "fired[] walk); low Occ here is NOT a launch-geometry "
        "failure. The cellpar emit kernel is the occupancy "
        "lever."
    ),
    # S-PERF.14c cellpar emit kernel: one thread per
    # (entity, slot, catalog). At canonical 256 × 4 096 K=1
    # the launch is 256 blocks × 16 threads = 4 096 (entity,
    # slot) threads vs the legacy 8 blocks × 32 threads. The
    # 32x block-count increase breaks the 2.1 % occupancy
    # ceiling the legacy candidate_boundary_kernel_wide paid.
    "candidate_boundary_cellpar_emit_kernel": (
        "S-PERF.14c cellpar emit kernel: 256 blocks x 16 "
        "threads at canonical scale (vs legacy 8 blocks x 32 "
        "threads = 32x block-count increase). Replaces the "
        "legacy candidate_boundary_kernel_wide which the "
        "post-S-PERF.14b ROOF flagged as the last remaining "
        "low-occupancy offender (286 us / 2.1 % Occ). Memcpy "
        "from run_buffer to boundaries[]; thread 0 publishes "
        "count_per_entity[]."
    ),
    # S-PERF.15.a fused detector_motif + digest_pack kernel.
    # Replaces the legacy 2-kernel sequence
    # (detector_motif_kernel_wide_d64 +
    # detector_wide_digest_pack_kernel_v1) that post-S-PERF.14c
    # ROOF flagged as the dominant L2-bound pair (2.72 ms /
    # 62 % of the L2 bucket at canonical 256x4096 K=1 D64).
    # Phase 1 detector evaluation + Phase 2 wide-cell store +
    # Phase 3 register-resident 18-byte compact pack in one
    # cell-parallel pass. Eliminates the ~277 MB L2 round-trip
    # on the wide-detector arena. Byte-identity preserved by
    # construction (4 PINNED_PRE_S_PERF_15_A_* constants pin
    # DetectorCellWide arena SHA-256, compact-pack arena
    # SHA-256, TreeSha256V1 detector-stage root, and
    # CompactDensorDigestV1 detector-stage root). Legacy
    # kernels remain callable from D128/D205 dispatchers
    # (which use wide_mask_words_used=2; fusion is
    # D64-specific). D64 _timed dispatchers only.
    "detector_motif_fused_d64_kernel": (
        "S-PERF.15.a fused detector_motif + digest_pack kernel: "
        "eliminates the ~277 MB L2 round-trip on the wide-detector "
        "arena by writing DetectorCellWide AND its 18-byte compact "
        "pack from registers in one cell-parallel pass. Replaces the "
        "legacy 2-kernel pair (detector_motif_kernel_wide_d64 + "
        "detector_wide_digest_pack_kernel_v1) which post-S-PERF.14c "
        "ROOF flagged as the dominant L2-bound pair (62 % of the L2 "
        "bucket). Byte-identity preserved (4 PINNED_PRE_S_PERF_15_A_* "
        "constants). D64 _timed dispatchers only; D128/D205 retain "
        "the unfused 2-kernel pair (wide_mask_words_used=2)."
    ),
    "consensus_axis5_fused_kernel": (
        "S-PERF.15.b direct-fusion consensus + axis5 kernel: "
        "absorbs axis5_grid_sum_kernel_wide's per-(window,catalog) "
        "i64 reduction into the consensus pass via shared-memory "
        "staging. One block per (window, catalog), one thread per "
        "entity; thread e writes axis7 to shm[e] in canonical lane "
        "order; thread 0 performs the canonical entity-ascending "
        "serial sum (byte-identical to the legacy serial loop). "
        "Eliminates the consensus -> axis5 L2 round-trip on the "
        "~32 MB ConsensusCell arena while preserving ConsensusCell "
        "bytes, axis5_grid_sum i64 values, candidate_fired cascade, "
        "and final case-file hash (4 PINNED_PRE_S_PERF_15_B_* "
        "constants). D64 _timed dispatchers only; D128/D205 retain "
        "the unfused consensus_grid_kernel_wide_d128/_d205 + "
        "axis5_grid_sum_kernel_wide pair."
    ),
    # S-PERF.15.c block-cooperative candidate-pack kernel.
    # Replaces the legacy candidate_pack_kernel_wide whose
    # post-S-PERF.15.b ROOF flagged it as the largest unaddressed
    # occupancy-bound wall (873 us @ 5.7 % Occ; structural OCC
    # ceiling because legacy launched only ceil(max_per_entity/32)
    # = 1 block-x with most threads early-returning when slot >=
    # local_count). New launch is one warp per (slot, entity,
    # catalog) — at canonical 256 x max_per_entity 16 K=1 D64 this
    # is 16 x 256 x 32 = 4 096 blocks (16x block-count increase
    # over legacy 256). Per-thread partials walk strided windows
    # across [start_w, end_w); block-cooperative pairwise shared-
    # memory reduction in canonical descending-stride order on 5
    # peaks (max), union_mask (OR), and entity/grid sums (i64-add)
    # — all associative + commutative, so byte-identical to the
    # legacy serial loop by construction. Thread 0 emits the
    # CandidateInterval at the canonical slot. Byte-identity
    # preserved (4 PINNED_PRE_S_PERF_15_C_* constants pin
    # CandidatePack arena SHA-256, CandidateCount arena SHA-256,
    # final case-file hash, episode-summary SHA-256). D64 _timed
    # dispatcher only; D128/D205 retain the legacy kernel (which
    # uses different projection helpers). Measured outcome
    # post-S-PERF.15.c ROOF: 873 -> 148.8 us (-83 %, 5.87x faster),
    # achieved occupancy 5.66 -> 38.68 % (+33 pp); kernel now
    # surfaces 65.8 % DRAM as the dominant fabric so the next
    # lever (if needed) is layout/coalescing, not launch
    # geometry.
    "candidate_pack_kernel_wide_blockcoop": (
        "S-PERF.15.c block-cooperative candidate-pack kernel: "
        "one warp per (slot, entity, catalog) replaces the legacy "
        "ceil(max_per_entity/32)=1 block-x launch (16x block-count "
        "increase at canonical 256 x max_per_entity 16 K=1 D64). "
        "Block-cooperative shared-memory pairwise reduction in "
        "canonical descending-stride order on 5 peaks (max), union "
        "mask (OR), and entity/grid sums (i64-add); all "
        "associative + commutative -> byte-identical to legacy "
        "serial walk. Thread 0 emits CandidateInterval at canonical "
        "slot. Byte-identity preserved (4 PINNED_PRE_S_PERF_15_C_* "
        "constants). D64 _timed dispatcher only; D128/D205 retain "
        "legacy candidate_pack_kernel_wide. Measured: 873 -> 148.8 "
        "us (-83 %, 5.87x faster); achieved occupancy 5.66 -> 38.68 "
        "% (+33 pp); next-lever bucket DRAM (65.8 %)."
    ),
}
lines.append("")
lines.append("  Per-kernel recommendation:")
for kname in kernel_names:
    short = kname.split("(", 1)[0].strip()
    bucket, why = classify(by_kernel[kname])
    rec_short = KERNEL_SPECIFIC_RECOMMENDATIONS.get(short) or {
        "DRAM-bound":       "reduce DRAM traffic",
        "L2-bound":         "kernel fusion + shared-mem",
        "SM/compute-bound": "vectorize hash math",
        "occupancy-bound":  "shrink register pressure",
        "DRAM-near":        "watch DRAM; layout-change candidate",
        "L2-near":          "kernel fusion candidate",
        "SM-near":          "compute-bound candidate",
        "UNCLASSIFIED":     "(needs full ncu set)",
    }.get(bucket, bucket)
    lines.append(f"    {short:46s}  [{bucket:>17s}]  -> {rec_short}")

# Non-claims.
lines.append("")
lines.append("-" * 72)
lines.append(" Panel-locked non-claims")
lines.append("-" * 72)
lines.append("  - Does NOT change any kernel.")
lines.append("  - Does NOT claim bandwidth saturation; it measures it.")
lines.append("  - Does NOT add Rust-side code or dependencies.")
lines.append("  - Does NOT mutate any hash anchor.")
lines.append("  - Does NOT pre-commit to any S-PERF.14 design; the")
lines.append("    measured verdict drives the next campaign.")
lines.append("")
lines.append("Receipt is one-shot; no hash chain. Subsequent S-PERF.14")
lines.append("commit cites this verdict as load-bearing scope input.")
lines.append("")

# Write to a temp file in the same dir, then atomic-rename.
# Handles the case where a stale root-owned receipt blocks
# direct overwrite when the script is re-run under sudo.
tmp_path = receipt_path + ".tmp"
with open(tmp_path, "w", encoding="utf-8") as fh:
    fh.write("\n".join(lines))
    fh.write("\n")
os.replace(tmp_path, receipt_path)

print(f"  receipt written to {receipt_path}", file=sys.stderr)
PYEOF

# Chown outputs back to the invoking user when running under
# sudo, so subsequent non-root tooling (cargo, git, the user's
# editor) can read/write them without permission errors.
if [[ "${EUID:-$(id -u)}" -eq 0 && -n "${SUDO_USER:-}" ]]; then
    chown "$SUDO_USER:$SUDO_USER" "$csv_out" "$receipt" 2>/dev/null || true
    echo "  (chowned outputs back to $SUDO_USER)"
fi

echo ""
echo "=== done ==="
echo "  CSV    : reports/_ncu_d64_compact_densor_metrics.csv"
echo "  Receipt: reports/s_perf_roof_preflight_d64_nsight_metrics.txt"
echo ""
echo "Next: review the receipt; the dominant bottleneck bucket"
echo "drives the S-PERF.14 lever choice per the plan's decision-"
echo "rule table."
