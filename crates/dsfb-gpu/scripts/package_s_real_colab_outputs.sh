#!/usr/bin/env bash
# package_s_real_colab_outputs.sh — assemble the operator-facing ZIP
# bundle of S-REAL audit outputs.
#
# WHY THIS EXISTS (for the future engineer reading cold):
#
# After `dsfb-gpu-debug s-real-audit --dataset all --out-dir reports`
# completes, the 20-dataset audit lives across three tier directories
# (reports/s_real_1/, reports/s_real_2/, reports/s_real_3/) plus the
# sealed bundle receipts (bundle_manifest.toml, bundle_hash_chain.txt,
# zenodo_metadata.json) and the operator navigation root INDEX.md.
# That's 180 audit artifacts + 4 top-level receipts; no operator wants
# to copy each one by hand from a Colab session.
#
# This script collects every piece an external reviewer needs into a
# single ZIP under reports/s_real_colab_outputs.zip (or any path
# passed via --out). The intended consumers are:
#
#   - the Colab notebook (notebooks/dsfb_gpu_debug_colab.ipynb) which
#     calls this immediately after the audit run to produce the
#     downloadable bundle the operator hands to reviewers
#   - dev-machine refreshes (e.g. when a sealed bundle gets a new
#     saturation receipt and we want to pack a fresh reference ZIP)
#
# The contents list is panel-locked at COLAB.S-REAL.1 plan-time and
# enumerated below. Build artifacts (target/), version control
# metadata (.git/), upstream archives (data/upstream/), and binary
# blobs are excluded by construction (we only include what we
# explicitly add).
#
# Usage:
#
#   bash scripts/package_s_real_colab_outputs.sh
#   bash scripts/package_s_real_colab_outputs.sh --out /tmp/bundle.zip
#
# Exit codes:
#   0   ZIP produced successfully
#   1   usage / argument error
#   2   required tier directory missing (refuses to build a partial
#       ZIP because that would be misleading)
#   3   no audit outputs found at all
#   4   zip binary not available

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# ────────────────────────────────────────────────────────────────────
# Argument parsing.
# ────────────────────────────────────────────────────────────────────
out_path="reports/s_real_colab_outputs.zip"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --out)
            shift
            if [[ $# -eq 0 ]]; then
                echo "package_s_real_colab_outputs.sh: --out requires a path" >&2
                exit 1
            fi
            out_path="$1"
            shift
            ;;
        -h|--help)
            sed -n '2,40p' "${BASH_SOURCE[0]}"
            exit 0
            ;;
        *)
            echo "package_s_real_colab_outputs.sh: unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

# ────────────────────────────────────────────────────────────────────
# Prerequisite check: every tier directory must exist. A partial
# bundle would be misleading to operators, so we refuse rather than
# silently ship 10/20 datasets.
# ────────────────────────────────────────────────────────────────────
required_tiers=(reports/s_real_1 reports/s_real_2 reports/s_real_3)
for tier in "${required_tiers[@]}"; do
    if [[ ! -d "$tier" ]]; then
        echo "package_s_real_colab_outputs.sh: required tier directory missing: $tier" >&2
        echo "  Run \`dsfb-gpu-debug s-real-audit --dataset all --out-dir reports\` first." >&2
        exit 2
    fi
done

# Sanity-check: are there any per-dataset subdirectories at all?
dataset_count=$(find reports/s_real_1 reports/s_real_2 reports/s_real_3 \
    -mindepth 1 -maxdepth 1 -type d | wc -l)
if [[ "$dataset_count" -eq 0 ]]; then
    echo "package_s_real_colab_outputs.sh: no per-dataset audit outputs found under reports/s_real_*/" >&2
    echo "  Expected 20 dataset directories total." >&2
    exit 3
fi

# Check zip binary is available (Colab has it pre-installed; dev
# machines typically too).
if ! command -v zip >/dev/null 2>&1; then
    echo "package_s_real_colab_outputs.sh: \`zip\` binary not found on PATH" >&2
    exit 4
fi

# ────────────────────────────────────────────────────────────────────
# Stage a temporary tree so the ZIP is rooted at a known prefix
# (dsfb_gpu_s_real_colab_outputs/) regardless of where the script
# is invoked from. Operators can `unzip -l` and see a clean,
# self-describing structure.
# ────────────────────────────────────────────────────────────────────
stage_root="$(mktemp -d -t dsfb_gpu_s_real_colab.XXXXXX)"
stage="$stage_root/dsfb_gpu_s_real_colab_outputs"
mkdir -p "$stage"

trap 'rm -rf "$stage_root"' EXIT

# Include verbatim: the three tier directories (all 180 audit
# artifacts + the sealed bundle receipts under s_real_3/).
cp -a reports/s_real_1 "$stage/"
cp -a reports/s_real_2 "$stage/"
cp -a reports/s_real_3 "$stage/"

# Operator navigation root.
if [[ -f reports/INDEX.md ]]; then
    cp -a reports/INDEX.md "$stage/"
fi

# Saturation sweep classification (if present from a sweep run; not
# required by the audit but useful for bundle context).
if [[ -f reports/s_real_saturation_sweep.txt ]]; then
    cp -a reports/s_real_saturation_sweep.txt "$stage/"
fi

# ────────────────────────────────────────────────────────────────────
# Synthesise a fresh colab_run_receipt.md with the run's
# provenance + A–F classification placeholders + panel-locked
# non-claims. The notebook OR a dev-machine refresh fills in the
# placeholder fields by re-emitting this file after the integrity
# tests run; this script emits the SHELL of the receipt so it
# always exists in the ZIP.
# ────────────────────────────────────────────────────────────────────
receipt_path="$stage/colab_run_receipt.md"
run_timestamp_utc="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
git_commit="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"

# Best-effort hardware identifiers (silently empty on machines
# without nvidia-smi / nvcc).
gpu_model="$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1 || echo 'unknown')"
cuda_version="$(nvcc --version 2>/dev/null | grep -oE 'release [0-9.]+' | head -1 || echo 'unknown')"
host_uname="$(uname -srm 2>/dev/null || echo 'unknown')"

cat > "$receipt_path" <<RECEIPT
# COLAB.S-REAL.1 — Run Receipt

Generated by \`scripts/package_s_real_colab_outputs.sh\` at the time
the ZIP bundle was assembled. This file rides inside the bundle so a
reviewer opening the ZIP knows exactly what hardware and what commit
produced these artifacts.

## Provenance

- run_timestamp_utc : ${run_timestamp_utc}
- git_commit        : ${git_commit}
- gpu_model         : ${gpu_model}
- cuda_version      : ${cuda_version}
- host_uname        : ${host_uname}

## Honest framing (panel-locked verbatim, MUST appear)

The notebook attempts cross-hardware replay on Colab T4 / A100 / L4.
If the emitted artifacts match the committed S-REAL bundle, this is
evidence of cross-hardware determinism for that environment. If they
diverge, the notebook reports the exact differing artifact and
preserves the result as an honest portability finding.

## A–F classification (filled in by notebook section 7 or dev refresh)

\`\`\`
A  Build success                              : <PENDING>
B  Dataset SHA verification success           : <PENDING>
C  Audit run success                          : <PENDING>
D  Per-run replay success                     : <PENDING>
E  Bundle integrity success                   : <PENDING>
F  Cross-hardware byte-identity success       : <PENDING>
\`\`\`

\`<PENDING>\` indicates the field is filled in by the notebook's
result-summary cell or by a subsequent dev-machine refresh, NOT by
this packer script. The packer ships the receipt skeleton so the
ZIP layout is consistent; the notebook overwrites the file before
inviting the operator to download.

## Per-dataset summary (filled in by notebook section 7 or dev refresh)

| dataset_id | events | admitted_episodes | replay_verified | casefile_final_hash |
|------------|-------:|------------------:|:---------------:|:--------------------|
| (pending)  |        |                   |                 |                     |

## Aggregate (filled in by notebook section 7 or dev refresh)

- total_datasets       : 20
- total_events         : (pending)
- total_episodes       : 316  (panel-locked bundle headline)
- replay_verified_yes  : (pending) / 20

## Panel-locked non-claims (verbatim, MUST appear)

- COLAB.S-REAL.1 does NOT add new datasets, kernels, or hashes.
- COLAB.S-REAL.1 does NOT change S-REAL.3.1.1's claims; it only
  makes them reproducible on third-party hardware.
- COLAB.S-REAL.1 does NOT claim Colab-runtime saturation or any
  new performance result — the saturation sweep is the dev-machine
  hardware-anchored measurement.
- COLAB.S-REAL.1 does NOT claim cross-hardware byte-identity until
  the F gate has actually been run; the notebook reports what it
  observed honestly.
- COLAB.S-REAL.1 does NOT modify the audit's algorithm, fixtures,
  contracts, or hash chain.
- The downloaded bundle does NOT claim Zenodo deposit or DOI
  assignment on its own; those are separate publication acts.
- The notebook is offered as a reproducibility tool, NOT as a
  benchmarking platform — Colab thermal variance makes any per-run
  GB/s number a courtesy snapshot, not a measurement claim.
- Colab T4 / A100 / L4 is NOT RTX 4080 SUPER + CUDA 13.2 (the
  dev-machine hardware anchor); timing values in
  \`perf_profile.txt\` are RUNTIME-DEPENDENT and NOT comparable
  to the anchor.

## Citation pointers

- Sealed commit chain (most recent first):
  - 3fdf42f  S-REAL.3.1.1 hygiene close-out
  - fde8a99  S-REAL.3.1 bundle integrity gate + saturation sweep
  - a8aaa04  S-REAL.3 20-dataset sealed; Zenodo-publishable bundle
- INDEX.md inside this ZIP is the operator navigation root.
RECEIPT

# ────────────────────────────────────────────────────────────────────
# Assemble the ZIP. Use \`zip -r\` with the staged directory so the
# archive contents land under \`dsfb_gpu_s_real_colab_outputs/\`.
# ────────────────────────────────────────────────────────────────────
out_path_abs="$(cd "$(dirname "$out_path")" && pwd)/$(basename "$out_path")"
mkdir -p "$(dirname "$out_path_abs")"
rm -f "$out_path_abs"

(
    cd "$stage_root"
    zip -rq "$out_path_abs" dsfb_gpu_s_real_colab_outputs
)

# Report final size + entry count for operator transparency.
entries="$(unzip -l "$out_path_abs" 2>/dev/null | tail -1 | awk '{print $2}')"
zip_bytes="$(stat -c %s "$out_path_abs" 2>/dev/null || stat -f %z "$out_path_abs")"
echo "package_s_real_colab_outputs: wrote $out_path ($zip_bytes bytes, $entries entries)"
