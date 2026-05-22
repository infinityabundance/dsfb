#!/usr/bin/env python3
# sqlshare_large.py — large-fixture residual projection of the
# SQLShare 2015 data-release corpus (academic SQL workload from UW).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# SQLShare (Jain et al., 2016) is a public scientific-workload
# corpus: 24K user-submitted SQL queries + their underlying
# data tables, released by the University of Washington in 2015.
# The data tables under `sqlshare_data_release1/data/<user_id>/`
# include both numeric CSVs (oceanographic stats, microbial DNA
# tags) and materialized query-result snapshots.
#
# This recipe projects a single canonical numeric CSV (the
# `table_seaflow-Tokyo_1-stats.csv` oceanographic statistics
# table, 3.3 MB) into a 1024×1024 residual TSV. The CSV has
# ~21K rows × many numeric columns, enough to satisfy the
# saturation-class cell count.
#
# Provenance (upstream):
#   - Source:  SQLShare data release 1 (Jain et al., U. Washington
#              2015). https://uwescience.github.io/sqlshare/
#   - File:    sqlshare_data_release1.zip → sqlshare_data_release1/
#              data/1002/table_seaflow-Tokyo_1-stats.csv
#   - License: research-use; the upstream zip does not include
#              an SPDX LICENSE; recorded as `no-upstream-license`
#              per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic; mirrors the
# snowset_large recipe except for the source file).
#
# Usage:
#   python3 data/recipes/sqlshare_large.py \
#       --zip "/run/media/one/toshiba4TB/dsfb-database datasets/sqlshare_data_release1.zip" \
#       --out data/fixtures/sqlshare_1024x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT classify oceanographic samples, recover
#     species taxonomies, predict SQL workload patterns, or
#     interpret the SQLShare schema.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     an oceanography / database-workload domain claim.

import argparse
import io
import math
import sys
import zipfile
from pathlib import Path

import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
DEFAULT_INNER_CSV = "sqlshare_data_release1/data/1002/table_seaflow-Tokyo_1-stats.csv"
DEFAULT_BASE_COLS = 64
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project SQLShare numeric CSV → residual-projection v2 TSV"
    )
    ap.add_argument("--zip", required=True,
                    help="Path to sqlshare_data_release1.zip")
    ap.add_argument("--out", default="data/fixtures/sqlshare_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--inner-csv", default=DEFAULT_INNER_CSV)
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    ap.add_argument("--base-cols", type=int, default=DEFAULT_BASE_COLS)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    if n_entities % args.base_cols != 0:
        print(f"FATAL: n_entities ({n_entities}) must be a multiple of "
              f"base_cols ({args.base_cols})", file=sys.stderr)
        return 2
    n_blocks = n_entities // args.base_cols
    rows_needed = n_blocks * n_windows

    zip_path = Path(args.zip)
    if not zip_path.is_file():
        print(f"FATAL: zip not found at {zip_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    raw_data = np.empty((rows_needed, args.base_cols), dtype=np.float64)
    numeric_cols = None
    rows_read = 0

    with zipfile.ZipFile(str(zip_path), "r") as zf:
        try:
            info = zf.getinfo(args.inner_csv)
        except KeyError:
            print(f"FATAL: zip member not found: {args.inner_csv}",
                  file=sys.stderr)
            return 3
        with zf.open(args.inner_csv, "r") as fh:
            text = io.TextIOWrapper(fh, encoding="utf-8", errors="replace")
            header = text.readline().rstrip("\n").split(",")
            # First non-empty data row → identify numeric columns.
            first_data = None
            for line in text:
                line = line.rstrip("\n")
                if not line:
                    continue
                first_data = line.split(",")
                break
            if first_data is None:
                print("FATAL: CSV has no data rows", file=sys.stderr)
                return 4
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
                return 5
            # First row already consumed.
            for k, j in enumerate(numeric_cols):
                try:
                    raw_data[0, k] = float(first_data[j])
                except ValueError:
                    raw_data[0, k] = float("nan")
            rows_read = 1
            for line in text:
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

    # If the CSV is smaller than the saturation magnitude, tile
    # the available rows to fill rows_needed. This keeps the
    # output shape deterministic and saturation-class, with the
    # honest receipt field `csv_unique_rows` recording the actual
    # distinct-row count before tiling.
    csv_unique_rows = rows_read
    if rows_read < rows_needed:
        # Tile the read rows in canonical order until we have enough.
        if rows_read == 0:
            print("FATAL: CSV produced 0 numeric rows", file=sys.stderr)
            return 6
        for i in range(rows_read, rows_needed):
            raw_data[i, :] = raw_data[i % rows_read, :]
        rows_read = rows_needed

    block = np.empty((n_entities, n_windows), dtype=np.float64)
    for e in range(n_entities):
        bc = e % args.base_cols
        b_idx = e // args.base_cols
        block[e, :] = raw_data[b_idx * n_windows:(b_idx + 1) * n_windows, bc]

    healthy_window_end = n_windows // 4
    baseline = block[:, :healthy_window_end]
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
        fh.write(f"# dataset_id=sqlshare_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={zip_path.name}\n")
        fh.write(f"# upstream_inner_csv={args.inner_csv}\n")
        fh.write("# upstream_source=SQLShare 2015 (Jain et al., U.Washington); https://uwescience.github.io/sqlshare/\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=Jain, Howe, Yan, Stoddard; U. Washington 2015\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write(f"# base_cols={args.base_cols} (numeric col indices: {numeric_cols}; replicated {n_blocks}x)\n")
        fh.write(f"# csv_unique_rows={csv_unique_rows}\n")
        fh.write(f"# rows_total={rows_read}\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=SQLShare oceanographic numeric CSV residual; throughput witness, not domain claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    print(f"  csv_unique_rows  : {csv_unique_rows}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
