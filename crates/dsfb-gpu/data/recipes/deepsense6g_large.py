#!/usr/bin/env python3
# deepsense6g_large.py — large-fixture residual projection of the
# Deepsense6G Scenario 23 mmWave-power telemetry corpus.
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# Deepsense6G is a multimodal 5G-mmWave sensing corpus (Alkhateeb
# et al., 2022) that pairs each timestamp with a 64-beam mmWave
# received-power vector, GPS, camera image, and IMU telemetry.
# Scenario 23 contains 11_388 timestamps with one mmWave power
# file per timestamp; each power file is 64 float64 values
# (one per beam).
#
# Unlike the I/Q-based RF datasets (DeepBeam / RadioML / POWDER /
# ORACLE), Deepsense6G is structurally TABULAR — there is no
# continuous time-series IQ stream. We project 64 beam-power
# values × 11_388 timestamps; reshape to 1024×1024 = 1_048_576
# cells, then z-score per entity against first 256 samples.
#
# Provenance (upstream):
#   - Source:  DeepSense6G Scenario 23 development split
#              (Alkhateeb et al., 2022)
#              https://deepsense6g.net/scenario-23/
#   - File:    scenario23_dev_w_resources.zip → scenario23_dev/
#              unit1/mmWave_data/mmWave_power_<N>.txt for
#              N ∈ [1, 11388]
#   - Format:  each .txt file = 64 lines of scientific-notation
#              floats (one beam-power per line). Linear scale,
#              units of received power (interpretation varies
#              by paper).
#   - License: research-use. The dataset README does not include
#              an SPDX LICENSE; recorded as `no-upstream-license`
#              per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic):
#   1. Open zip read-only. Read mmWave_power_1.txt through
#      mmWave_power_(n_files).txt. The number of files needed
#      = ceil(n_entities * n_windows / 64). Order is strictly
#      ascending integer suffix — deterministic across runs.
#   2. Parse each file as 64 float64 values. Stack into a
#      flat array of length n_files × 64.
#   3. Reshape to (n_entities, n_windows) row-major; require
#      n_entities × n_windows ≤ n_files × 64. The default
#      1024×1024 needs ceil(1048576/64) = 16384 files; we have
#      11388 → trim n_files to 11388 and shape down OR use
#      n_entities × n_windows ≤ 11388 × 64 = 728_832 cells.
#   4. Per-entity z-score against early-baseline (first
#      healthy_window_end samples of each row; Bessel-corrected
#      stddev). Drop-mask on baseline stddev < 1e-6.
#   5. 6-decimal fixed-point output; LF newlines.
#
# Default shape is 896×1024 = 917_504 cells — fits within
# 11388×64 = 728_832 cells if we trim to 896×813 = 728,448
# cells. Actually we shrink to a safe default that fits: use
# 768×896 = 688,128 cells (the largest 1024-divisible shape
# under 728,832).
#
# Even tighter default: 512×1024 = 524,288 cells (clean halves).
# That fits in 8192 files × 64 = 524,288 cells exactly. Pick
# that as the default — keeps the recipe deterministic without
# truncation surprises.
#
# Usage:
#   python3 data/recipes/deepsense6g_large.py \
#       --zip "/run/media/one/toshiba4TB/dsfb-rf datasets/Deepsense6G/scenario23_dev_w_resources.zip" \
#       --out data/fixtures/deepsense6g_512x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT predict beam index, classify channel
#     condition, recover GPS position, or process the camera/IMU
#     modalities from this fixture.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     a 5G-mmWave-domain-truth claim.

import argparse
import math
import sys
import zipfile
from pathlib import Path

import numpy as np

# Default fits cleanly in 8192 mmWave files × 64 beams = 524_288
# cells; smaller than the IQ-recipe defaults of 1024×1024
# (1_048_576 cells) because Deepsense6G has only ~11_388 files,
# capping the maximum at 11388*64 = 728_832 cells.
DEFAULT_n_entities = 512
DEFAULT_n_windows = 1024
DEFAULT_INNER_GLOB = "scenario23_dev/unit1/mmWave_data/mmWave_power_{i}.txt"
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project Deepsense6G mmWave-power files → residual-projection v2 TSV"
    )
    ap.add_argument("--zip", required=True,
                    help="Path to scenario23_dev_w_resources.zip")
    ap.add_argument("--out", default="data/fixtures/deepsense6g_512x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    total_cells = n_entities * n_windows
    # Each mmWave power file contains exactly 64 values.
    files_needed = (total_cells + 63) // 64

    zip_path = Path(args.zip)
    if not zip_path.is_file():
        print(f"FATAL: zip not found at {zip_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # The mmWave_power_<N>.txt files have GAPS in the numeric
    # suffix (the zip contains 11387 files but indices run up to
    # ~12004 with some skipped). We enumerate the actual member
    # list, sort by integer suffix, and take the first
    # `files_needed`. Deterministic across runs given the same
    # zip bytes.
    samples = np.empty(files_needed * 64, dtype=np.float64)
    with zipfile.ZipFile(str(zip_path), "r") as zf:
        all_members = zf.namelist()
        prefix = "scenario23_dev/unit1/mmWave_data/mmWave_power_"
        suffix = ".txt"
        candidates = []
        for name in all_members:
            if name.startswith(prefix) and name.endswith(suffix):
                idx_str = name[len(prefix):-len(suffix)]
                try:
                    candidates.append((int(idx_str), name))
                except ValueError:
                    continue
        candidates.sort()
        if len(candidates) < files_needed:
            print(f"FATAL: only {len(candidates)} mmWave files present; "
                  f"need {files_needed} for {n_entities}x{n_windows}",
                  file=sys.stderr)
            return 3
        for k, (idx, inner) in enumerate(candidates[:files_needed]):
            with zf.open(inner, "r") as fh:
                raw = fh.read().decode("utf-8").splitlines()
            if len(raw) < 64:
                print(f"FATAL: {inner} has {len(raw)} lines; expected 64",
                      file=sys.stderr)
                return 4
            for j in range(64):
                samples[k * 64 + j] = float(raw[j])

    block = samples[:total_cells].reshape(n_entities, n_windows)

    healthy_window_end = n_windows // 4
    baseline = block[:, :healthy_window_end]
    mean = baseline.mean(axis=1)
    std = baseline.std(axis=1, ddof=1)
    drop_mask = std < STDDEV_FLOOR
    with np.errstate(divide="ignore", invalid="ignore"):
        residual = (block - mean[:, None]) / std[:, None]
    residual[drop_mask, :] = np.nan

    n_finite = int(np.isfinite(residual).sum())
    n_dropped = int(drop_mask.sum())

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# residual-projection v2\n")
        fh.write(f"# dataset_id=deepsense6g_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={zip_path.name}\n")
        fh.write(f"# upstream_files=scenario23_dev/unit1/mmWave_data/mmWave_power_[1..{files_needed}].txt\n")
        fh.write("# upstream_source=Deepsense6G Scenario 23 (Alkhateeb et al. 2022); https://deepsense6g.net/scenario-23/\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=Alkhateeb, A.; et al.; Deepsense6G 2022\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write("# source_field=mmWave received-power (64 beams per timestamp; linear scale)\n")
        fh.write("# stacking_order=ascending timestamp index, ascending beam index within each file\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=Deepsense6G Scenario 23 mmWave-power residual; throughput witness, not beam-prediction claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{total_cells}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    print(f"  files_read       : {files_needed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
