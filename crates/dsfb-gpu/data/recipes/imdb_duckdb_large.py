#!/usr/bin/env python3
# imdb_duckdb_large.py — large-fixture residual projection of the
# imdb.duckdb binary file via deterministic byte-frequency residuals.
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# `imdb.duckdb` is a DuckDB binary database of the IMDB
# Join-Order-Benchmark schema. The full SQL projection
# (via the python `duckdb` library) is the cleanest way to
# extract structured numeric residuals, but the runner has
# python-duckdb pip-blocked (PEP 668; pacman package
# `python-duckdb` requires sudo). To stay non-blocking we use a
# DIFFERENT projection: deterministic byte-frequency residuals of
# the binary file itself.
#
# For each "entity" (row of the residual matrix), we compute the
# byte-frequency histogram (256 bins) over a contiguous 32 KB
# block of the .duckdb file. Each "window" (column) is one bin
# of that histogram, replicated 4× to fill 1024 columns. Two
# nearby blocks of the same .duckdb file produce SIMILAR byte
# distributions (DuckDB's binary format has repeating column-
# layout headers); larger byte offsets produce DIFFERENT
# distributions (different table pages). The z-score residual
# surfaces both regimes.
#
# This is NOT a SQL-projection of the database — it's a
# byte-frequency-residual projection of the binary file. Honest
# in every receipt. The dataset is admitted as a throughput
# witness ONLY; the audit cannot infer SQL semantics from these
# bytes.
#
# Provenance (upstream):
#   - Source:  IMDB Join-Order-Benchmark database, DuckDB dump.
#              Built locally; same underlying schema as
#              imdb.tgz (sister recipe).
#   - File:    imdb.duckdb (~2.7 GB DuckDB v1.x binary)
#   - License: research-use; no SPDX LICENSE on the binary
#              dump itself; recorded as `no-upstream-license`.
#
# Projection law (panel-locked, deterministic):
#   1. mmap or open .duckdb read-only. Read n_entities sequential
#      32 KB byte blocks starting at byte offset 0.
#   2. For each block, compute a 256-bin byte-frequency histogram
#      (uint16; one bin per possible byte value 0..255).
#   3. Replicate each row 4 times across the column axis so the
#      256-bin histogram fills 1024 windows. The audit's per-
#      entity z-score against early-baseline runs on these
#      replicated values.
#   4. Per-entity z-score against early-baseline (first
#      healthy_window_end=256 samples of each row; Bessel-
#      corrected stddev). Constant rows (e.g. a block of all-
#      zero padding) emit NaN.
#   5. 6-decimal fixed-point output; LF newlines.
#
# Usage:
#   python3 data/recipes/imdb_duckdb_large.py \
#       --db "/run/media/one/toshiba4TB/dsfb-database datasets/imdb.duckdb" \
#       --out data/fixtures/imdb_duckdb_1024x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT execute SQL, decode DuckDB pages, or
#     recover any database schema from this fixture.
#   - This is a deterministic BYTE-FREQUENCY residual of the
#     binary file, NOT a SQL-query projection.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     a database-content-domain-truth claim.

import argparse
import math
import sys
from pathlib import Path

import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
BYTES_PER_BLOCK = 32 * 1024  # 32 KB per entity row
HIST_BINS = 256              # uint8 byte distribution
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project imdb.duckdb byte-frequency → residual-projection v2 TSV"
    )
    ap.add_argument("--db", required=True, help="Path to imdb.duckdb")
    ap.add_argument("--out", default="data/fixtures/imdb_duckdb_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    # n_windows must be a multiple of HIST_BINS so the replicated
    # histogram fills the window axis evenly. Default 1024 = 4×256.
    if n_windows % HIST_BINS != 0:
        print(f"FATAL: n_windows ({n_windows}) must be a multiple of "
              f"HIST_BINS ({HIST_BINS})", file=sys.stderr)
        return 2
    replicate = n_windows // HIST_BINS

    db_path = Path(args.db)
    if not db_path.is_file():
        print(f"FATAL: db not found at {db_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    needed_bytes = n_entities * BYTES_PER_BLOCK
    if db_path.stat().st_size < needed_bytes:
        print(f"FATAL: {db_path.name} is {db_path.stat().st_size} bytes; "
              f"need {needed_bytes}", file=sys.stderr)
        return 4

    # Read the prefix as one buffer; this is ~32 MB at the default
    # shape (1024 × 32 KB), trivially small.
    with open(db_path, "rb") as fh:
        raw = fh.read(needed_bytes)

    # Compute histograms per entity. numpy.bincount is the fastest
    # path; minlength ensures 256-element output even when bytes
    # don't cover all 256 values.
    hist = np.empty((n_entities, HIST_BINS), dtype=np.float64)
    for e in range(n_entities):
        block = np.frombuffer(raw, dtype=np.uint8,
                              count=BYTES_PER_BLOCK,
                              offset=e * BYTES_PER_BLOCK)
        hist[e, :] = np.bincount(block, minlength=HIST_BINS).astype(np.float64)

    # Replicate to fill n_windows columns. Each histogram column
    # appears `replicate` times in a row. Final shape: (n_entities,
    # n_windows). The replication makes the saturation-shape
    # bench compare like-for-like against the IQ fixtures (same
    # n_entities × n_windows), without inventing data.
    block = np.tile(hist, (1, replicate))

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
        fh.write(f"# dataset_id=imdb_duckdb_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={db_path.name}\n")
        fh.write("# upstream_source=IMDB Join-Order-Benchmark; DuckDB dump (same schema as imdb.tgz)\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=IMDB Inc.; Join-Order-Benchmark (Leis et al.); DuckDB\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write(f"# bytes_per_entity_block={BYTES_PER_BLOCK}\n")
        fh.write(f"# hist_bins={HIST_BINS}; replicate={replicate} (replicated across window axis)\n")
        fh.write("# projection=byte-frequency histogram per 32KB block; z-score per entity vs first healthy_window_end samples\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=DuckDB binary byte-frequency residual; NOT a SQL projection. Throughput witness, not database-content claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
