#!/usr/bin/env python3
# snowset_large.py — large-fixture residual projection of the
# Snowset Snowflake-telemetry CSV (snowset-main.csv.gz, 92 cols).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# Snowset (Vuppalapati et al., NSDI 2020) is a public 14M-row
# anonymized query-execution-telemetry corpus released by
# Snowflake. Each row records one query's executor stats:
# duration, byte-counts, queue times, server counts, etc. 92
# columns; most are numeric.
#
# This recipe projects the first n_windows rows of n_entities
# numeric columns into a residual-projection v2 TSV, treating
# each numeric column as one ENTITY and each row as one WINDOW.
# Default 64×1024 = 65_536 cells (small saturation-class fit;
# 92 columns gives us 64 numeric columns easily, and 1024 rows
# is a thin stripe to keep the fixture under 10 MB on disk).
#
# To get the saturation magnitude (~1M cells) the recipe stacks
# multiple row-blocks of the SAME 64 numeric columns: with
# n_windows=1024 and n_blocks=16 we get a 1024×1024 fixture by
# reading 16384 rows total and reshaping. That gives the
# saturation-class cell count using the actual Snowflake
# telemetry numeric distribution.
#
# Provenance (upstream):
#   - Source:  Vuppalapati, Miron, Agarwal, Park, Patel, Stoica;
#              NSDI 2020 (Snowset public telemetry release).
#              https://github.com/resource-disaggregation/snowset
#   - File:    snowset-main.csv.gz (gzip-compressed 7.6 GB CSV,
#              92 columns, ~14M rows of anonymized telemetry).
#   - License: research-use; the upstream repo does not include
#              an SPDX LICENSE on the CSV itself; recorded as
#              `no-upstream-license` per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic):
#   1. gzip-stream the CSV. Parse the header to find the 64
#      LEFTMOST numeric columns (skip columns that fail
#      float-parse on the first non-empty data row).
#   2. Read the first n_windows*n_blocks rows. Stack them so
#      column-i of block-b becomes entity (b*64 + i) — i.e. each
#      numeric column gets cloned 16 times across n_blocks. This
#      gives the saturation cell-count without re-running the
#      gzip stream multiple times.
#   3. Per-entity z-score against early-baseline (first
#      healthy_window_end samples of each row; Bessel-corrected
#      stddev). Drop-mask on baseline stddev < 1e-6.
#   4. 6-decimal fixed-point output; LF newlines.
#
# Usage:
#   python3 data/recipes/snowset_large.py \
#       --csv-gz "/run/media/one/toshiba4TB/dsfb-database datasets/snowset-main.csv.gz" \
#       --out data/fixtures/snowset_1024x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT classify SQL queries, predict workload
#     latency, model Snowflake performance, or recover any
#     database-internal state from this fixture.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     a database-telemetry-domain-truth claim.

import argparse
import gzip
import math
import sys
from pathlib import Path

import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
DEFAULT_BASE_COLS = 64       # number of distinct numeric columns to use
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project snowset Snowflake-telemetry CSV → residual-projection v2 TSV"
    )
    ap.add_argument("--csv-gz", required=True,
                    help="Path to snowset-main.csv.gz")
    ap.add_argument("--out", default="data/fixtures/snowset_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    ap.add_argument("--base-cols", type=int, default=DEFAULT_BASE_COLS,
                    help=f"Distinct numeric columns to use (default {DEFAULT_BASE_COLS}). "
                         f"n_entities should be a multiple of base-cols; the "
                         f"recipe replicates each column n_entities/base-cols times "
                         f"to fill the saturation-class shape.")
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    if n_entities % args.base_cols != 0:
        print(f"FATAL: n_entities ({n_entities}) must be a multiple of "
              f"base-cols ({args.base_cols})", file=sys.stderr)
        return 2
    n_blocks = n_entities // args.base_cols
    rows_needed = n_blocks * n_windows

    gz_path = Path(args.csv_gz)
    if not gz_path.is_file():
        print(f"FATAL: gz not found at {gz_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Stream the gzipped CSV. We deliberately DO NOT decompress the
    # whole 60+ GB CSV — we read row-by-row until we have
    # rows_needed numeric rows.
    numeric_cols = None
    raw_data = np.empty((rows_needed, args.base_cols), dtype=np.float64)
    rows_read = 0
    with gzip.open(str(gz_path), "rt", encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split(",")
        # First non-empty data row → infer which columns parse as
        # floats. Cache the selected column indices for all
        # subsequent rows. Deterministic across runs given the same
        # file bytes.
        first_data = None
        for line in fh:
            line = line.rstrip("\n")
            if not line:
                continue
            first_data = line.split(",")
            break
        if first_data is None:
            print("FATAL: CSV has no data rows", file=sys.stderr)
            return 3
        numeric_cols = []
        for j, v in enumerate(first_data):
            try:
                float(v)
                numeric_cols.append(j)
                if len(numeric_cols) >= args.base_cols:
                    break
            except ValueError:
                continue
        if len(numeric_cols) < args.base_cols:
            print(f"FATAL: found only {len(numeric_cols)} numeric cols; "
                  f"need {args.base_cols}", file=sys.stderr)
            return 4
        # First row already consumed; record its values.
        for k, j in enumerate(numeric_cols):
            try:
                raw_data[0, k] = float(first_data[j])
            except ValueError:
                raw_data[0, k] = float("nan")
        rows_read = 1
        # Stream remaining rows. Skip rows where any selected
        # column fails to parse — they're rare in the snowset CSV
        # but exist; recorded as NaN. The audit ingest treats NaN
        # cells as missingness.
        for line in fh:
            if rows_read >= rows_needed:
                break
            line = line.rstrip("\n")
            if not line:
                continue
            parts = line.split(",")
            for k, j in enumerate(numeric_cols):
                try:
                    raw_data[rows_read, k] = float(parts[j])
                except (ValueError, IndexError):
                    raw_data[rows_read, k] = float("nan")
            rows_read += 1

    if rows_read < rows_needed:
        print(f"FATAL: CSV exhausted at {rows_read} rows; need {rows_needed}",
              file=sys.stderr)
        return 5

    # Replicate the base-cols n_blocks times so each row of the
    # final block is (col_0, col_1, ..., col_63, col_0, col_1, ...).
    # block[w, e] = raw_data[w + (e // base_cols) * n_windows, e % base_cols].
    block = np.empty((n_entities, n_windows), dtype=np.float64)
    for e in range(n_entities):
        base_col = e % args.base_cols
        block_idx = e // args.base_cols
        block[e, :] = raw_data[block_idx * n_windows:(block_idx + 1) * n_windows,
                               base_col]

    healthy_window_end = n_windows // 4
    baseline = block[:, :healthy_window_end]
    # Some snowset columns are constant (e.g. compilationTime can be
    # 0 across long runs). The NaN-aware stats here keep the
    # recipe robust: if baseline has NaN, mean/std propagate NaN
    # and the row gets dropped.
    mean = baseline.mean(axis=1)
    std = baseline.std(axis=1, ddof=1)
    drop_mask = (std < STDDEV_FLOOR) | ~np.isfinite(std) | ~np.isfinite(mean)
    with np.errstate(divide="ignore", invalid="ignore"):
        residual = (block - mean[:, None]) / std[:, None]
    residual[drop_mask, :] = np.nan

    n_finite = int(np.isfinite(residual).sum())
    n_dropped = int(drop_mask.sum())

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# residual-projection v2\n")
        fh.write(f"# dataset_id=snowset_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={gz_path.name}\n")
        fh.write("# upstream_source=Snowset (Vuppalapati et al., NSDI 2020); github.com/resource-disaggregation/snowset\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=Vuppalapati, Miron, Agarwal, Park, Patel, Stoica; Snowflake/UC Berkeley 2020\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write(f"# base_cols={args.base_cols} (distinct numeric columns; replicated {n_blocks}x to fill n_entities)\n")
        fh.write(f"# numeric_col_indices={','.join(str(c) for c in numeric_cols)}\n")
        fh.write(f"# rows_read={rows_read}\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=Snowset Snowflake-telemetry numeric-column residual; throughput witness, not database-telemetry claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    print(f"  base_cols        : {args.base_cols} (replicated {n_blocks}x)")
    print(f"  rows_read        : {rows_read}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
