#!/usr/bin/env python3
# imdb_tgz_large.py — large-fixture residual projection of the
# IMDB Movie Database CSV dump (Join-Order-Benchmark dataset).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# The IMDB-from-JOB-benchmark corpus (Leis et al., VLDB 2015;
# "How Good Are Query Optimizers, Really?") ships as ~21 CSV
# files inside `imdb.tgz`. The largest single-table CSV is
# `cast_info.csv` (~36M rows × 7 cols) describing actor-movie-
# role relationships. Several columns are numeric (person_id,
# movie_id, person_role_id, nr_order, role_id, ID columns).
#
# This recipe projects the first n_windows rows of n_entities
# numeric columns from the largest CSV in the tarball,
# replicating columns the same way snowset_large.py does to
# fill the saturation-class cell count (1024×1024).
#
# Provenance (upstream):
#   - Source:  IMDB Movie Database, Join-Order-Benchmark
#              (Leis et al., VLDB 2015 / 2018 SIGMOD).
#              http://homepages.cwi.nl/~boncz/job/
#   - File:    imdb.tgz → cast_info.csv (~3 GB uncompressed,
#              36M rows, 7 cols: id, person_id, movie_id,
#              person_role_id, note, nr_order, role_id)
#   - License: research-use; the upstream tarball does not
#              include an SPDX LICENSE on the data; recorded as
#              `no-upstream-license` per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic):
#   1. Open tarball read-only; locate cast_info.csv.
#   2. Stream-parse the first n_windows*n_blocks rows. For each
#      row, extract the 5 numeric ID columns (id, person_id,
#      movie_id, person_role_id, role_id). Skip the textual
#      `note` column and the sparse `nr_order` integer.
#   3. Replicate to n_entities=1024 by tiling the 5 base columns
#      n_blocks=1024/5=204 times (with one truncated block of
#      4 columns). The recipe asserts n_entities is a multiple
#      of base_cols=5 OR clamps n_entities down to a multiple.
#      Default: n_entities=1020 (= 204 × 5), n_windows=1024
#      → 1_044_480 cells.
#   4. Per-entity z-score against early-baseline (first
#      healthy_window_end samples; Bessel-corrected stddev).
#      Drop-mask on baseline stddev < 1e-6.
#   5. 6-decimal fixed-point output; LF newlines.
#
# Usage:
#   python3 data/recipes/imdb_tgz_large.py \
#       --tgz "/run/media/one/toshiba4TB/dsfb-database datasets/imdb.tgz" \
#       --out data/fixtures/imdb_tgz_1020x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT predict join order, infer schema, or
#     recover actor/movie relationships from this fixture.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     a database-schema-domain-truth claim.

import argparse
import io
import math
import sys
import tarfile
from pathlib import Path

import numpy as np

# Defaults: 1020 entities × 1024 windows = 1_044_480 cells.
# 1020 = 204 × 5 (= base_cols replicated to fill saturation
# shape without partial blocks).
DEFAULT_n_entities = 1020
DEFAULT_n_windows = 1024
DEFAULT_INNER_CSV = "cast_info.csv"
# cast_info.csv columns in source order:
#   0=id, 1=person_id, 2=movie_id, 3=person_role_id, 4=note,
#   5=nr_order, 6=role_id
# Pick the 5 dense numeric ID columns (skip note + nr_order).
DEFAULT_NUMERIC_COL_IDX = [0, 1, 2, 3, 6]
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project imdb.tgz cast_info.csv → residual-projection v2 TSV"
    )
    ap.add_argument("--tgz", required=True, help="Path to imdb.tgz")
    ap.add_argument("--out", default="data/fixtures/imdb_tgz_1020x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--inner-csv", default=DEFAULT_INNER_CSV)
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows
    base_cols = len(DEFAULT_NUMERIC_COL_IDX)
    if n_entities % base_cols != 0:
        print(f"FATAL: n_entities ({n_entities}) must be a multiple of "
              f"base_cols ({base_cols})", file=sys.stderr)
        return 2
    n_blocks = n_entities // base_cols
    rows_needed = n_blocks * n_windows

    tgz_path = Path(args.tgz)
    if not tgz_path.is_file():
        print(f"FATAL: tgz not found at {tgz_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Stream-read cast_info.csv from the tarball. tarfile lets us
    # extract a member into a file-like without materialising the
    # full 3 GB on disk — important because the runner may not
    # have 3 GB free in /tmp.
    raw_data = np.empty((rows_needed, base_cols), dtype=np.float64)
    rows_read = 0
    with tarfile.open(str(tgz_path), "r:gz") as tar:
        try:
            member = tar.getmember(args.inner_csv)
        except KeyError:
            print(f"FATAL: tar member not found: {args.inner_csv}",
                  file=sys.stderr)
            return 3
        fh = tar.extractfile(member)
        if fh is None:
            print(f"FATAL: cannot open {args.inner_csv} in tarball",
                  file=sys.stderr)
            return 4
        # Wrap as text. The IMDB CSVs are UTF-8 with no BOM.
        text = io.TextIOWrapper(fh, encoding="utf-8")
        for line in text:
            if rows_read >= rows_needed:
                break
            line = line.rstrip("\n").rstrip("\r")
            if not line:
                continue
            parts = line.split(",")
            for k, j in enumerate(DEFAULT_NUMERIC_COL_IDX):
                try:
                    raw_data[rows_read, k] = float(parts[j])
                except (ValueError, IndexError):
                    raw_data[rows_read, k] = float("nan")
            rows_read += 1

    if rows_read < rows_needed:
        print(f"FATAL: CSV exhausted at {rows_read} rows; need {rows_needed}",
              file=sys.stderr)
        return 5

    # Replicate base_cols n_blocks times into the saturation shape.
    block = np.empty((n_entities, n_windows), dtype=np.float64)
    for e in range(n_entities):
        bc = e % base_cols
        b_idx = e // base_cols
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
        fh.write(f"# dataset_id=imdb_tgz_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={tgz_path.name}\n")
        fh.write(f"# upstream_inner_csv={args.inner_csv}\n")
        fh.write("# upstream_source=IMDB Movie Database; Join-Order-Benchmark (Leis et al. VLDB 2015)\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=IMDB Inc.; Join-Order-Benchmark (Leis et al.)\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write(f"# base_cols={base_cols} (numeric col indices: {DEFAULT_NUMERIC_COL_IDX}; replicated {n_blocks}x)\n")
        fh.write(f"# rows_read={rows_read}\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=IMDB cast_info.csv numeric-ID residual; throughput witness, not movie-database claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    print(f"  rows_read        : {rows_read}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
