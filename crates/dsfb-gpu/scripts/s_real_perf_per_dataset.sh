#!/usr/bin/env bash
# S-REAL.3.bench — per-dataset performance receipt + optional Nsight
# Compute trace across the 20-dataset audit gauntlet.
#
# WHY THIS EXISTS (for the future engineer reading cold):
#
# S-REAL.3 emits 180 deterministic audit artifacts (9 per dataset
# × 20 datasets). Each `dsfb-gpu-debug s-real-audit --dataset <id>`
# call (canonical subcommand; `s-real-1-audit` historical alias
# preserved in main.rs match arm) already produces a `perf_profile.txt`
# with per-stage timing
# captured via `std::time::Instant` (ingest, CUDA dispatch run 1,
# CUDA dispatch run 2, casefile emit, episodes JSONL emit, audit
# report emit, total). This script does NOT add new timing; it
# RUNS every dataset, AGGREGATES the per-dataset receipts into a
# single comparison table, and OPTIONALLY captures a Nsight Compute
# trace for a chosen dataset under sudo.
#
# Honest framing (panel-locked, MUST hold in every receipt):
#   - These wall-time numbers reflect SHORT single-dispatch audits
#     on SMALL fixtures (128–656 events / dataset). Launch overhead
#     and artifact-write overhead dominate.
#   - DO NOT compare these numbers against S-PERF.16.a's saturation-
#     class bench (256×4096 K=1, 4M+ events / dispatch). The two
#     layers measure structurally different workloads.
#   - The Nsight trace, when captured, is FROM the same audit
#     binary on the same fixture; per-kernel verdicts will reflect
#     a low-occupancy short-dispatch regime, NOT a saturation
#     regime. This is honest data; do not overclaim from it.
#
# Outputs (whitelisted under reports/s_real_perf/ in .gitignore):
#   reports/s_real_perf/<dataset_id>/perf_receipt.txt
#   reports/s_real_perf/summary.txt
# Optional Nsight artifacts (sudo run, gitignored):
#   reports/s_real_perf/<dataset_id>/ncu_<dataset_id>.csv
#   reports/s_real_perf/<dataset_id>/ncu_<dataset_id>.txt
#
# Usage:
#
#   # 1. Per-dataset wall-time aggregation (no sudo; safe to repeat):
#   bash scripts/s_real_perf_per_dataset.sh
#
#   # 2. Single-dataset Nsight Compute trace (sudo; ~2-5 min):
#   sudo bash scripts/s_real_perf_per_dataset.sh --ncu cmapss_fd001_unit1
#
#   # 3. Limit which datasets to time (skip a known-slow set, etc.):
#   bash scripts/s_real_perf_per_dataset.sh --dataset tadbench_f11
#   bash scripts/s_real_perf_per_dataset.sh --dataset all
#
# `sudo` is required for --ncu because NVIDIA driver gates GPU
# performance counters (ERR_NVGPUCTRPERM) from non-root users by
# default; same constraint as scripts/s_perf_roof_preflight.sh.
#
# Panel-locked non-claims (printed at end of summary.txt):
#   - Does NOT change any kernel.
#   - Does NOT change any sealed audit artifact (replay verification
#     still gates byte-identity; this script only RE-runs the audit
#     and reads its emitted perf_profile.txt).
#   - Does NOT claim bandwidth saturation; small fixtures are launch-
#     overhead-dominated.
#   - Does NOT compare against the S-PERF.16.a saturation bench.

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# The canonical 20-dataset list, mirroring s_real_audit.rs::DATASETS
# in canonical order. Kept in sync MANUALLY rather than parsed from
# the Rust source — the source-of-truth is the Rust DATASETS const;
# this list is a thin run-loop driver.
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
target_dataset=""           # empty until we know whether --dataset
                            # was explicitly set; resolved after the
                            # argv loop so --ncu can supply a default.
target_dataset_explicit=0   # 1 iff --dataset was passed on argv;
                            # lets --ncu narrow the perf loop to one
                            # dataset when the user didn't say "all".
ncu_arg=""                  # raw --ncu argv value; expanded below
                            # into ncu_datasets[] (single id, "all",
                            # or comma-separated list of ids).
iters=2          # mirrors s-real-audit's default --iters; 2 = one
                 # primary + one replay verification.
catalogs=1       # mirrors s-real-audit's default --catalogs.
expect_dataset_value=0
expect_ncu_value=0
expect_iters_value=0
expect_catalogs_value=0
for arg in "$@"; do
    if [[ "$expect_dataset_value" -eq 1 ]]; then
        target_dataset="$arg"; expect_dataset_value=0; continue
    fi
    if [[ "$expect_ncu_value" -eq 1 ]]; then
        ncu_arg="$arg"; expect_ncu_value=0; continue
    fi
    if [[ "$expect_iters_value" -eq 1 ]]; then
        iters="$arg"; expect_iters_value=0; continue
    fi
    if [[ "$expect_catalogs_value" -eq 1 ]]; then
        catalogs="$arg"; expect_catalogs_value=0; continue
    fi
    case "$arg" in
        --dataset)        expect_dataset_value=1; target_dataset_explicit=1 ;;
        --dataset=*)      target_dataset="${arg#--dataset=}"; target_dataset_explicit=1 ;;
        --ncu)            expect_ncu_value=1 ;;
        --ncu=*)          ncu_arg="${arg#--ncu=}" ;;
        --iters)          expect_iters_value=1 ;;
        --iters=*)        iters="${arg#--iters=}" ;;
        --catalogs)       expect_catalogs_value=1 ;;
        --catalogs=*)     catalogs="${arg#--catalogs=}" ;;
        --help|-h)
            cat <<'EOF'
usage: scripts/s_real_perf_per_dataset.sh [flags]

  --dataset <id|all>   Restrict the run-loop to ONE dataset id, or
                       "all" (default) to run all 20. The id list
                       lives in s_real_audit.rs::DATASETS.

  --iters <N>          Number of replay iterations per audit (>= 2).
                       Default 2 = one primary + one byte-identity
                       replay; higher values produce wider thermal-
                       noise bands without changing seal verdicts.

  --catalogs <K>       Batched-K amortization (default 1). K > 1
                       replicates the same dataset K times into one
                       batched dispatch; honestly labelled as
                       amortization, NOT K distinct datasets.

  --ncu <id|all|csv>   Capture a Nsight Compute trace per dataset.
                       Accepts a single id, "all" (every entry in
                       DATASETS), or a comma-separated list of ids.
                       REQUIRES sudo (driver gates the perf
                       counters). Writes ncu_<id>.csv + .txt +
                       .ncu-rep under reports/s_real_perf/<id>/
                       for each named dataset. Cost is real:
                       ~2-5 min and ~200-400 MB of binary trace
                       per dataset, so --ncu all is roughly 40-100
                       min and ~5-8 GB cumulative. Skip this flag
                       entirely for the plain wall-time aggregation.

  -h, --help           Show this message.
EOF
            exit 0 ;;
        *)
            echo "warning: unknown flag $arg (ignored)" >&2 ;;
    esac
done

# Validate flags.
if ! [[ "$iters" =~ ^[0-9]+$ ]] || [[ "$iters" -lt 2 ]]; then
    echo "FATAL: --iters must be an integer >= 2 (got '$iters')" >&2
    exit 2
fi
if ! [[ "$catalogs" =~ ^[0-9]+$ ]] || [[ "$catalogs" -lt 1 ]]; then
    echo "FATAL: --catalogs must be a positive integer (got '$catalogs')" >&2
    exit 2
fi

# Expand --ncu into the ncu_datasets[] array. Three forms accepted:
#   --ncu all          → every entry in ALL_DATASETS
#   --ncu <id>         → single-element array
#   --ncu <id1>,<id2>  → comma-separated list (no spaces)
# An empty ncu_arg means the perf loop runs without any Nsight
# capture, which is the safe non-sudo default.
declare -a ncu_datasets=()
if [[ -n "$ncu_arg" ]]; then
    if [[ "$ncu_arg" == "all" ]]; then
        ncu_datasets=("${ALL_DATASETS[@]}")
    else
        IFS=',' read -r -a ncu_datasets <<< "$ncu_arg"
    fi
fi

# Resolve the default for --dataset. When --ncu was set without an
# explicit --dataset, the perf loop should match the ncu surface:
#   --ncu all     → perf loop over all 20 (matches ncu scope)
#   --ncu <id>    → perf loop over that single id (no waste)
#   --ncu id1,id2 → perf loop over the same list (no waste)
# Explicit --dataset always wins; if the operator wants ncu for one
# dataset but timing for all, they can pass --dataset all alongside.
if [[ "$target_dataset_explicit" -eq 0 ]]; then
    if [[ "${#ncu_datasets[@]}" -gt 0 ]]; then
        if [[ "${#ncu_datasets[@]}" -eq "${#ALL_DATASETS[@]}" ]]; then
            target_dataset="all"
        elif [[ "${#ncu_datasets[@]}" -eq 1 ]]; then
            target_dataset="${ncu_datasets[0]}"
            echo "==> --dataset not explicit; --ncu narrowed perf loop to '$target_dataset'" >&2
        else
            # Multiple-but-not-all ncu targets: we cannot pass a list
            # through the existing --dataset case-match, so default to
            # "all" so every named dataset is timed. The ncu loop
            # below still only captures the ones the operator named.
            target_dataset="all"
            echo "==> --dataset not explicit; --ncu names ${#ncu_datasets[@]} ids; perf loop runs over all 20" >&2
        fi
    else
        target_dataset="all"
    fi
fi

# Resolve which datasets we actually iterate over.
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
        echo "       valid ids: ${ALL_DATASETS[*]}" >&2
        exit 2
    fi
fi

# Working directory.
perf_root="reports/s_real_perf"
mkdir -p "$perf_root"

# CUDA tool discovery.
ncu_bin="${NCU_BIN:-/usr/bin/ncu}"
cuda_bin="${CUDA_BIN:-/opt/cuda/bin}"
export PATH="$cuda_bin:$PATH"

# If --ncu was requested, validate sudo + ncu binary + every named
# dataset id BEFORE we start the run-loop. Fail-fast on a typo or
# missing tool so we don't burn 40 minutes mid-loop only to die on
# an unknown id at dataset #15.
if [[ "${#ncu_datasets[@]}" -gt 0 ]]; then
    if [[ "$EUID" -ne 0 ]]; then
        echo "FATAL: --ncu requires sudo (NVIDIA driver gates GPU perf counters)" >&2
        echo "       re-invoke as: sudo bash $0 --ncu $ncu_arg" >&2
        exit 3
    fi
    if [[ ! -x "$ncu_bin" ]]; then
        echo "FATAL: ncu not found at $ncu_bin" >&2
        echo "       set NCU_BIN=<path/to/ncu> and re-invoke" >&2
        exit 3
    fi
    for nd in "${ncu_datasets[@]}"; do
        nd_found=0
        for d in "${ALL_DATASETS[@]}"; do
            if [[ "$d" == "$nd" ]]; then nd_found=1; break; fi
        done
        if [[ "$nd_found" -ne 1 ]]; then
            echo "FATAL: --ncu dataset '$nd' is not in DATASETS" >&2
            echo "       valid ids: ${ALL_DATASETS[*]}" >&2
            exit 3
        fi
    done
fi

# Build once before the run-loop. `--features cuda` is REQUIRED for
# the audit subcommand to invoke the GPU dispatcher; without it the
# audit either skips or errors with `GpuError::CudaUnavailable`.
#
# Sudo discipline: when invoked under sudo (for --ncu), the user's
# shell PATH is stripped → `cargo` is not visible to root. We do
# NOT try to compile under sudo. Build the binary as the regular
# user FIRST; under sudo we only invoke the pre-built binary +
# ncu. If the binary is missing under sudo, error out with a
# helpful message instead of crashing on `cargo: not found`.
bin="target/release/dsfb-gpu-debug"
if [[ "$EUID" -eq 0 ]]; then
    if [[ ! -x "$bin" ]]; then
        echo "FATAL: running under sudo but $bin does not exist." >&2
        echo "       sudo's stripped PATH cannot find cargo. Build" >&2
        echo "       the binary first as your regular user, then" >&2
        echo "       re-invoke this script under sudo:" >&2
        echo "" >&2
        echo "         bash $0 --dataset <id>     # builds + runs once" >&2
        echo "         sudo bash $0 --ncu <id>    # uses cached binary" >&2
        echo "" >&2
        exit 4
    fi
    echo "==> using pre-built $bin (sudo run; cargo build skipped)..."
else
    echo "==> building dsfb-gpu-debug-demo (release, cuda feature)..."
    cargo build --release --features cuda -p dsfb-gpu-debug-demo \
        --bin dsfb-gpu-debug 2>&1 | tail -3
    if [[ ! -x "$bin" ]]; then
        echo "FATAL: $bin not found after build" >&2
        exit 4
    fi
fi

# Master summary header. ALWAYS truncates and rewrites on every
# invocation — by design, the summary mirrors the LAST run of the
# script, not an aggregate across runs. If you need the prior run's
# numbers, the per-dataset perf_receipt.txt files under
# reports/s_real_perf/<id>/ are kept on disk regardless and are
# overwritten only when that specific dataset is re-audited.
summary_path="$perf_root/summary.txt"
{
    echo "=========================================================="
    echo "  S-REAL.3 per-dataset performance receipt"
    echo "  generated by scripts/s_real_perf_per_dataset.sh"
    echo "  $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "=========================================================="
    echo ""
    echo "  iters    : $iters (per-audit replay iteration count)"
    echo "  catalogs : $catalogs (batched-K amortization factor)"
    echo "  datasets : ${#DATASETS_TO_RUN[@]} of ${#ALL_DATASETS[@]} total"
    echo ""
    echo "  Honest framing:"
    echo "    - These are SHORT single-dispatch audits on SMALL"
    echo "      fixtures (128-656 events). Launch overhead and"
    echo "      artifact-write overhead dominate."
    echo "    - DO NOT compare against S-PERF.16.a saturation"
    echo "      bench (256x4096 K=1, 4M+ events / dispatch)."
    echo "    - Numbers below are honest measurements of what the"
    echo "      audit wall actually is on this hardware, not"
    echo "      saturation-class throughput."
    echo ""
    printf "%-30s | %10s | %10s | %10s | %10s | %12s | %10s\n" \
        "dataset_id" "ingest_us" "disp1_us" "disp2_us" "emit_us" "total_us" "ev/sec"
    printf '%s\n' "------------------------------+------------+------------+------------+------------+--------------+-----------"
} > "$summary_path"

# Run-loop. For each dataset:
#   1. Pick the tier directory matching the dataset's seal tier so
#      the receipts we emit do NOT collide with the canonical
#      reports/s_real_<tier>/ artifacts. We write our perf
#      receipt under reports/s_real_perf/<dataset>/ separately.
#   2. Invoke the audit with the chosen iters / catalogs. This
#      RE-RUNS the dispatcher and OVERWRITES the canonical audit
#      artifacts at reports/s_real_<tier>/<dataset>/. That is
#      INTENDED: re-running an audit on the same sealed fixture
#      must produce byte-identical artifacts (replay invariant).
#   3. Parse perf_profile.txt for the timing fields.
#   4. Compute derived metrics (events/sec, fixture-bytes/sec).
#   5. Append one row to summary.txt and write a per-dataset
#      receipt block under reports/s_real_perf/<dataset>/.

# Tier resolver: matches the canonical commit-time tier directories.
tier_of() {
    case "$1" in
        tadbench_f11|illinois_socialnet|aiops_kpi) echo "s_real_1" ;;
        cmapss_fd001_unit50|cmapss_fd002_unit1|cmapss_fd002_unit100|cmapss_fd003_unit1|cmapss_fd004_unit1|promise_ant_1_4|deeptralog_f02) echo "s_real_3" ;;
        radioml_2018_snr30_large|deepbeam_large|radioml_gold_large|powder_large|oracle_large|deepsense6g_large|imdb_tgz_large|imdb_duckdb_large|snowset_large|sqlshare_large) echo "s_real_throughput" ;;
        *) echo "s_real_2" ;;
    esac
}

for d in "${DATASETS_TO_RUN[@]}"; do
    tier="$(tier_of "$d")"
    out_dir="reports/${tier}"
    dataset_dir="$out_dir/$d"
    perf_dir="$perf_root/$d"
    mkdir -p "$perf_dir"

    echo "==> [$d] running audit (tier=$tier, iters=$iters, catalogs=$catalogs)..."
    "$bin" s-real-audit \
        --dataset "$d" \
        --out-dir "$out_dir" \
        --iters "$iters" \
        --catalogs "$catalogs" \
        > "$perf_dir/audit_stdout.txt" 2> "$perf_dir/audit_stderr.txt" || {
            echo "  AUDIT FAILED for $d (see $perf_dir/audit_stderr.txt)" >&2
            printf "%-30s | %10s | %10s | %10s | %10s | %12s | %10s\n" \
                "$d" "FAIL" "FAIL" "FAIL" "FAIL" "FAIL" "FAIL" \
                >> "$summary_path"
            continue
        }

    perf_profile="$dataset_dir/perf_profile.txt"
    if [[ ! -f "$perf_profile" ]]; then
        echo "  perf_profile.txt missing for $d at $perf_profile" >&2
        continue
    fi

    # Parse perf_profile.txt. Format (deterministic, from
    # emit_perf_profile_txt): "  ingest_us              : N\n", etc.
    parse_us() {
        local key="$1"
        local file="$2"
        local val
        val="$(grep -E "^[[:space:]]*${key}[[:space:]]*:" "$file" \
               | head -1 | awk -F: '{print $2}' | tr -d ' ')"
        if [[ -z "$val" ]]; then val="0"; fi
        echo "$val"
    }

    ingest_us=$(parse_us 'ingest_us' "$perf_profile")
    disp1_us=$(parse_us 'cuda_dispatch_run1_us' "$perf_profile")
    disp2_us=$(parse_us 'cuda_dispatch_run2_us' "$perf_profile")
    casefile_us=$(parse_us 'casefile_emit_us' "$perf_profile")
    episodes_us=$(parse_us 'episodes_jsonl_emit_us' "$perf_profile")
    report_us=$(parse_us 'audit_report_emit_us' "$perf_profile")
    # total_us may be a derived field; if absent, sum the parts.
    total_us=$(parse_us 'total_us' "$perf_profile")
    if [[ "$total_us" == "0" || -z "$total_us" ]]; then
        total_us=$(( ingest_us + disp1_us + disp2_us + casefile_us + episodes_us + report_us ))
    fi
    emit_us=$(( casefile_us + episodes_us + report_us ))

    # events_per_second + events_emitted live in perf_profile.txt's
    # Throughput block (the audit binary computes them end-to-end).
    # We parse them directly rather than recomputing from disp1_us +
    # a manifest count, because (a) the audit's measurement is the
    # canonical one and (b) dataset_manifest.toml does not carry an
    # event_count field.
    manifest="$dataset_dir/dataset_manifest.toml"
    event_count=$(parse_us 'events_emitted' "$perf_profile")
    ev_per_sec=$(parse_us 'events_per_second' "$perf_profile")
    if [[ -z "$event_count" ]]; then event_count=0; fi
    if [[ -z "$ev_per_sec" ]]; then ev_per_sec=0; fi

    printf "%-30s | %10s | %10s | %10s | %10s | %12s | %10s\n" \
        "$d" "$ingest_us" "$disp1_us" "$disp2_us" "$emit_us" "$total_us" "$ev_per_sec" \
        >> "$summary_path"

    # Per-dataset receipt block. Captures everything from
    # perf_profile.txt plus the derived metrics and the chosen
    # iters/catalogs context, so a future reader can identify
    # under what regime this measurement was taken.
    {
        echo "=========================================================="
        echo "  S-REAL.3 per-dataset performance receipt — $d"
        echo "  generated by scripts/s_real_perf_per_dataset.sh"
        echo "  $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
        echo "=========================================================="
        echo ""
        echo "  source perf_profile.txt: $perf_profile"
        echo "  source dataset_manifest: $manifest"
        echo "  iters                  : $iters"
        echo "  catalogs               : $catalogs"
        echo "  tier                   : $tier"
        echo ""
        echo "  Timing breakdown (microseconds, host Instant):"
        echo "    ingest_us              : $ingest_us"
        echo "    cuda_dispatch_run1_us  : $disp1_us"
        echo "    cuda_dispatch_run2_us  : $disp2_us"
        echo "    casefile_emit_us       : $casefile_us"
        echo "    episodes_jsonl_emit_us : $episodes_us"
        echo "    audit_report_emit_us   : $report_us"
        echo "    total_us               : $total_us"
        echo ""
        echo "  Derived (run-1 only; run-2 is replay verification):"
        echo "    event_count            : $event_count"
        echo "    events_per_second      : $ev_per_sec"
        echo ""
        echo "  Honest framing:"
        echo "    - Single-dispatch audit on $event_count events;"
        echo "      launch overhead + artifact-write overhead dominate."
        echo "    - DO NOT compare against S-PERF.16.a saturation bench."
        echo "    - run2 is replay-verification, NOT a second sample;"
        echo "      the bench harness does not aggregate it into ev/sec."
    } > "$perf_dir/perf_receipt.txt"
done

# Footer + non-claims.
{
    echo ""
    echo "=========================================================="
    echo "  Panel-locked non-claims"
    echo "=========================================================="
    echo ""
    echo "  - These wall-time numbers do NOT claim memory-bandwidth"
    echo "    saturation. The audit dispatches are LAUNCH-OVERHEAD-"
    echo "    DOMINATED on these small fixtures."
    echo "  - These numbers do NOT compare against S-PERF.16.a's"
    echo "    saturation-class bench (256x4096 K=1, 4M+ events)."
    echo "    Both layers measure structurally different workloads."
    echo "  - These numbers do NOT change any sealed audit artifact."
    echo "    Re-running an audit on the same fixture produces byte-"
    echo "    identical artifacts (replay invariant); only the per-"
    echo "    receipt timing values are non-deterministic."
    echo "  - cross-driver / cross-CUDA / cross-hardware byte-identity"
    echo "    is NOT claimed."
    echo "  - For per-kernel Nsight evidence on a single dataset,"
    echo "    re-invoke with: sudo bash $0 --ncu <dataset_id>"
    echo ""
} >> "$summary_path"

# Optional Nsight Compute capture. Loops over every dataset named
# in ncu_datasets[] (single id, "all", or comma-separated list).
# We invoke the audit binary directly under ncu so the trace covers
# the same kernel-launch sequence the audit produces.
#
# Cost discipline: each capture takes ~2-5 min and produces a
# 200-400 MB .ncu-rep binary trace. --ncu all is therefore ~40-100
# min wall + ~5-8 GB cumulative disk. The trace files are
# .gitignored; only the .csv + .txt receipts ship in commits.
#
# Failure mode: if one dataset's ncu fails (driver hiccup, OOM,
# unsupported metric on this arch), the loop records the failure
# in summary.txt and continues to the next dataset rather than
# aborting the whole run.
if [[ "${#ncu_datasets[@]}" -gt 0 ]]; then
    {
        echo ""
        echo "=========================================================="
        echo "  Nsight Compute traces (per dataset)"
        echo "=========================================================="
        echo "  ncu binary  : $ncu_bin"
        echo "  count       : ${#ncu_datasets[@]} of ${#ALL_DATASETS[@]}"
        echo ""
        printf "%-30s | %-10s | %s\n" "dataset_id" "status" "artifact path"
        printf '%s\n' "------------------------------+------------+--------------------------------------------------"
    } >> "$summary_path"

    ncu_idx=0
    ncu_total="${#ncu_datasets[@]}"
    for nd in "${ncu_datasets[@]}"; do
        ncu_idx=$(( ncu_idx + 1 ))
        tier="$(tier_of "$nd")"
        out_dir="reports/${tier}"
        perf_dir="$perf_root/$nd"
        mkdir -p "$perf_dir"

        ncu_csv="$perf_dir/ncu_${nd}.csv"
        ncu_txt="$perf_dir/ncu_${nd}.txt"
        ncu_rep="$perf_dir/ncu_${nd}.ncu-rep"
        ncu_tmp_rep="$ncu_csv.tmp.ncu-rep"

        echo ""
        echo "==> [ncu $ncu_idx/$ncu_total: $nd] capturing trace..."
        echo "    csv: $ncu_csv"
        echo "    rep: $ncu_rep"

        # ncu captures every kernel launch in the process by default;
        # the audit's two dispatcher invocations + their constituent
        # kernels are all captured. --set detailed gives the full
        # SpeedOfLight matrix; --csv emits machine-readable rows.
        if "$ncu_bin" \
                --set detailed \
                --csv \
                --target-processes all \
                --print-units base \
                -o "$ncu_csv.tmp" \
                "$bin" s-real-audit \
                --dataset "$nd" \
                --out-dir "$out_dir" \
                --iters "$iters" \
                --catalogs "$catalogs" \
                > "$perf_dir/ncu_audit_stdout.txt" 2> "$perf_dir/ncu_stderr.txt"; then
            # ncu writes a binary .ncu-rep when -o is given; we want a
            # text/CSV view. Use `ncu --import` to render the trace.
            if [[ -f "$ncu_tmp_rep" ]]; then
                "$ncu_bin" --import "$ncu_tmp_rep" --csv \
                    > "$ncu_csv" 2>> "$perf_dir/ncu_stderr.txt" || true
                "$ncu_bin" --import "$ncu_tmp_rep" --print-summary per-kernel \
                    > "$ncu_txt" 2>> "$perf_dir/ncu_stderr.txt" || true
                mv -f "$ncu_tmp_rep" "$ncu_rep"
            fi
            printf "%-30s | %-10s | %s\n" "$nd" "OK" "$perf_dir/" >> "$summary_path"
            echo "    OK"
        else
            printf "%-30s | %-10s | %s\n" "$nd" "FAIL" "$perf_dir/ncu_stderr.txt" >> "$summary_path"
            echo "    FAIL (see $perf_dir/ncu_stderr.txt)" >&2
            # Continue to next dataset rather than aborting; a single
            # ncu failure should not waste an in-flight --ncu all run.
        fi
    done

    {
        echo ""
        echo "  Note: ncu rows reflect ALL kernel launches in each"
        echo "  audit process, including the replay-verification"
        echo "  second dispatch. Per-kernel durations are honest"
        echo "  but reflect a LAUNCH-OVERHEAD-DOMINATED regime on"
        echo "  these short single-dispatch audits."
    } >> "$summary_path"
fi

echo ""
echo "==> summary written to: $summary_path"
echo "==> per-dataset receipts under: $perf_root/<dataset_id>/perf_receipt.txt"
if [[ "${#ncu_datasets[@]}" -gt 0 ]]; then
    echo "==> nsight traces (${#ncu_datasets[@]} datasets) under: $perf_root/<dataset_id>/ncu_<dataset_id>.{csv,txt,ncu-rep}"
fi
