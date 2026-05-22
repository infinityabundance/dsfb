#!/usr/bin/env python3
# deepbeam_large.py — large-fixture residual projection of the
# DeepBeam HDF5 (neu_ww72bk394.h5, NEU IDB-DeepBeam).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# DeepBeam is a 56 GB HDF5 of continuous I/Q amplitudes recorded
# against varying RX/TX beam patterns. It's the second saturation-
# class real-data fixture for the S-REAL sweep, complementing the
# RadioML 2018 SNR=30 dB recipe (data/recipes/radioml_2018_snr30_large.py).
# Both feed into the same dispatcher via the residual-projection v2
# TSV format and the S-PERF.16.a-mirrored saturation harness at
# tests/s_real_saturation_bench.rs.
#
# Provenance (upstream):
#   - Source:    NEU Wireless Lab, DeepBeam Dataset
#                (https://genesys-lab.org/oracle ; tier "neu_ww72bk394")
#   - File:      neu_ww72bk394.h5 (56 GB on disk; 11.06 billion float64
#                IQ pairs from `iq` group; matching `gain`, `rx_beam`,
#                `tx_beam` per-sample arrays).
#   - Format:    four 1-D HDF5 datasets, each 11_059_200_000 entries:
#                   gain    (float64, dB)
#                   iq      (float64 × 2; real, imag at each sample)
#                   rx_beam (float64, beam-index label)
#                   tx_beam (float64, beam-index label)
#                Continuous time-series; no documented sample-rate
#                in the HDF5 itself; this recipe treats every sample
#                as 1 logical time-step.
#   - License:   DeepBeam is distributed by the NEU GeneSys Lab for
#                research use. The dataset's README does not include
#                an SPDX-style LICENSE token; we record the license
#                field below as "no-upstream-license" per the
#                research-fair-use convention already used for other
#                S-REAL fixtures without explicit LICENSE files.
#
# Projection law (panel-locked, deterministic):
#   1. Open the HDF5 read-only and slice the first
#      n_entities × n_windows samples from `/iq` as float64 pairs.
#      Default 1024 × 1024 = 1_048_576 cells (matches RadioML
#      large-fixture magnitude → directly comparable in the sweep).
#   2. Compute IQ magnitude: |z| = sqrt(I^2 + Q^2). This is a
#      deterministic scalar reduction; both float64 inputs and a
#      sqrt-of-sum-of-squares are bit-stable across CPython
#      versions for the same input bytes.
#   3. Reshape to (n_entities, n_windows). Slice ordering is
#      row-major: entity i has time samples [i*n_windows, (i+1)*n_windows).
#   4. Per-ENTITY z-score against an EARLY-BASELINE of the first
#      healthy_window_end samples of THAT entity row (Bessel-
#      corrected stddev). Matches cmapss / radioml_2018 byte-for-
#      byte; downstream audit treats [0, H) as baseline-phase.
#   5. Entities whose baseline-phase stddev < 1e-6 are dropped to
#      all-NaN rows (audit ingest skips NaN cells).
#   6. 6-decimal fixed-point output. LF newlines. No trailing
#      whitespace. Matches sibling recipes byte-for-byte.
#
# Determinism rules (panel-locked):
#   - The slice [0:n_entities*n_windows] of `/iq` is the same on
#     every run. No random seeds. No streaming/chunking heuristics
#     that change between Python versions.
#   - 6-decimal printing rounds half-to-even on float64 → bit-
#     stable across CPython for the same input.
#   - LF newlines; `.gitattributes` marks the projected TSV
#     `binary` so git's CRLF normalization can never mangle the
#     SHA-256-pinned bytes.
#
# Usage:
#   python3 data/recipes/deepbeam_large.py \
#       --hdf5 "/run/media/one/toshiba4TB/dsfb-rf datasets/DeepBeam Dataset/neu_ww72bk394.h5" \
#       --out  data/fixtures/deepbeam_1024x1024.tsv
#
# Override shape:
#   --n-entities  default 1024; max 11_059_200_000 / n_windows
#   --n-windows   default 1024; max 11_059_200_000 / n_entities
#
# Panel-locked non-claims (in every audit_report.html produced
# from the output of this recipe):
#   - DSFB-GPU does NOT classify beam-pattern, transmitter, or
#     modulation. It reports STRUCTURAL residual evidence under
#     the deterministic z-score projection.
#   - DeepBeam is admitted here as a THROUGHPUT WITNESS for the
#     saturation bench, NOT as a domain-truth claim about RF
#     beamforming or beam-class detection.

import argparse
import math
import sys
from pathlib import Path

import h5py
import numpy as np

# Default fixture shape: same as the RadioML large fixture
# (1024 × 1024 = 1,048,576 cells = ~1M events). DeepBeam's
# `/iq` has 11 billion entries available, so larger shapes are
# possible if a future operator wants to bench a multi-M-cell
# fixture — but 1M cells matches S-PERF.16.a magnitude and is
# the canonical saturation-class size.
DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024

# Bessel-corrected sample stddev floor below which an entity row
# is emitted as NaN. Matches every sibling recipe byte-for-byte.
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project DeepBeam HDF5 → residual-projection v2 TSV"
    )
    # --hdf5 path is required because the upstream lives off-repo
    # on an external drive (56 GB; not vendored into git). The
    # full path is recorded in the audit's dataset_manifest.toml
    # provenance block at audit time.
    ap.add_argument(
        "--hdf5",
        required=True,
        help="Path to neu_ww72bk394.h5 (upstream DeepBeam I/Q recording).",
    )
    ap.add_argument(
        "--out",
        default="data/fixtures/deepbeam_1024x1024.tsv",
        help="Output TSV path (residual-projection v2).",
    )
    ap.add_argument(
        "--n-entities",
        type=int,
        default=DEFAULT_n_entities,
        help=f"Number of contiguous entity rows (default {DEFAULT_n_entities})",
    )
    ap.add_argument(
        "--n-windows",
        type=int,
        default=DEFAULT_n_windows,
        help=f"Number of time samples per entity (default {DEFAULT_n_windows})",
    )
    args = ap.parse_args()
    n_entities = args.n_entities
    n_windows = args.n_windows
    if n_entities < 1 or n_windows < 1:
        print("FATAL: --n-entities and --n-windows must be positive", file=sys.stderr)
        return 2

    hdf5_path = Path(args.hdf5)
    if not hdf5_path.is_file():
        print(f"FATAL: HDF5 not found at {hdf5_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Read the first n_entities × n_windows pairs from `/iq` as
    # float64. Total memory load = n_entities*n_windows*16 bytes
    # (2 floats × 8 bytes); at the 1024×1024 default that's 16 MB
    # — trivially small. Larger shapes scale linearly; we deliberately
    # DO NOT load the entire 176 GB `/iq` array.
    needed = n_entities * n_windows
    with h5py.File(str(hdf5_path), "r") as f:
        if "iq" not in f:
            print(
                "FATAL: HDF5 has no top-level 'iq' member; "
                "this recipe expects DeepBeam neu_ww72bk394.h5 schema",
                file=sys.stderr,
            )
            return 3
        ds = f["iq"]
        if ds.shape[0] < needed:
            print(
                f"FATAL: HDF5 /iq has {ds.shape[0]} samples, "
                f"need {needed} ({n_entities}*{n_windows})",
                file=sys.stderr,
            )
            return 4
        # Read 2-column slice: shape (needed, 2). dtype float64.
        iq = ds[:needed, :].astype(np.float64)

    # IQ magnitude per sample: |z| = sqrt(I^2 + Q^2). This is the
    # deterministic scalar reduction that produces a single residual
    # quantity per time sample. Reshape to (n_entities, n_windows)
    # row-major so entity i carries time samples in [i*n_windows,
    # (i+1)*n_windows).
    magnitude = np.sqrt(iq[:, 0] ** 2 + iq[:, 1] ** 2)
    block = magnitude.reshape(n_entities, n_windows)

    # Per-entity z-score against early baseline. H = n_windows / 4
    # is the panel-locked default; matches the radioml recipe so
    # both fixtures use the same baseline / active split semantics.
    healthy_window_end = n_windows // 4
    if healthy_window_end < 2:
        print(
            f"FATAL: healthy_window_end={healthy_window_end} too small for "
            f"a meaningful baseline (need >= 2 samples per entity row)",
            file=sys.stderr,
        )
        return 5
    baseline = block[:, :healthy_window_end]
    mean = baseline.mean(axis=1)
    std = baseline.std(axis=1, ddof=1)

    # Constant-row mask. Entities with baseline stddev < floor
    # produce divide-by-zero residuals → emit all-NaN row. Audit
    # ingest treats NaN cells as missingness; downstream stages
    # skip them. Same posture as every sibling recipe.
    drop_mask = std < STDDEV_FLOOR
    with np.errstate(divide="ignore", invalid="ignore"):
        residual = (block - mean[:, None]) / std[:, None]
    residual[drop_mask, :] = np.nan

    n_finite = int(np.isfinite(residual).sum())
    n_dropped = int(drop_mask.sum())

    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        # Header block. Records EVERY piece of provenance an auditor
        # needs to reproduce the projection: source basename, schema
        # field used (`iq`), magnitude formula, slice shape, axis
        # semantics, NaN handling, finite-cell count, healthy_window_end.
        # Matches the radioml_2018 recipe byte-for-byte except for
        # the upstream-source + formula lines.
        fh.write("# residual-projection v2\n")
        fh.write(f"# dataset_id=deepbeam_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={hdf5_path.name}\n")
        fh.write("# upstream_source=NEU GeneSys Lab DeepBeam (https://genesys-lab.org/oracle)\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=NEU GeneSys Lab; DeepBeam dataset (neu_ww72bk394)\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write("# source_field=/iq (column 0 = real, column 1 = imag; both float64)\n")
        fh.write("# magnitude_formula=sqrt(I^2 + Q^2) per sample\n")
        fh.write("# projection=z-score per entity (mean, sample-stddev over first healthy_window_end samples; Bessel correction)\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip (audit ingest drops NaN cells)\n")
        fh.write("# axis0=entity (signal); axis1=window (time-sample within IQ recording)\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=Large-fixture z-score residual projection of DeepBeam I/Q magnitudes. DSFB does NOT classify beam-pattern; it reports STRUCTURAL residual evidence under the deterministic projection.\n")
        # Data rows. NO column-header row, NO leading window-idx
        # column — same format as every other vendored TSV.
        for w in range(n_windows):
            row = []
            for e in range(n_entities):
                v = residual[e, w]
                if math.isnan(v):
                    row.append("NaN")
                else:
                    row.append(f"{v:.6f}")
            fh.write("\t".join(row) + "\n")

    byte_size = out_path.stat().st_size
    print(f"wrote: {out_path}")
    print(f"  shape           : {n_entities} entities × {n_windows} windows")
    print(f"  finite cells    : {n_finite} of {n_entities * n_windows}")
    print(f"  dropped rows    : {n_dropped} (all-NaN; baseline stddev < {STDDEV_FLOOR})")
    print(f"  byte_size       : {byte_size} bytes")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
