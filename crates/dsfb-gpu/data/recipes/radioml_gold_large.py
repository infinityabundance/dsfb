#!/usr/bin/env python3
# radioml_gold_large.py — large-fixture residual projection of the
# full RadioML 2018.01 GOLD_XYZ_OSC.0001_1024.hdf5 corpus.
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# The sibling recipe `radioml_2018_snr30_large.py` projects the
# 97 MB SNR=30 dB slice of RadioML 2018.01. This recipe projects
# the FULL 20 GB GOLD_XYZ_OSC.0001_1024.hdf5 corpus, which contains
# 2,555,904 examples × 1024 IQ time samples × 2 (real, imag) +
# 24-class modulation labels (`/Y`) and SNR-per-example (`/Z`).
#
# Both fixtures feed the same dispatcher via the saturation harness
# (tests/s_real_saturation_bench.rs). Two RadioML projections give
# the saturation sweep TWO independent samples from the same source
# corpus, which surfaces measurement variance vs. dataset variance:
# if the SNR=30 slice and the full GOLD corpus produce different
# GB/s numbers, the variance is dataset-dependent; if they match
# within thermal noise, the dispatcher's saturation regime is
# stable across SNR distributions.
#
# Provenance (upstream):
#   - Authors:  T. O'Shea, J. Corgan (DeepSig)
#   - File:     GOLD_XYZ_OSC.0001_1024.hdf5 (20 GB; full 2018.01
#               release with all SNR levels and modulation classes)
#   - Format:
#       /X  shape (2_555_904, 1024, 2)  float32   I/Q samples
#       /Y  shape (2_555_904, 24)       int       one-hot mod label
#       /Z  shape (2_555_904, 1)        int       SNR in dB
#   - License:  CC-BY-NC-SA-4.0 (research-use; non-commercial). Cite
#               https://www.deepsig.ai/datasets when redistributing.
#
# Projection law (panel-locked, deterministic; mirrors the sibling
# radioml_2018_snr30_large.py):
#   1. Read /X[:n_entities, :n_windows, :2] as float64 (promotion
#      from float32 happens at read time; matches sibling).
#   2. IQ magnitude: |z| = sqrt(I^2 + Q^2) per sample.
#   3. Reshape to (n_entities, n_windows) row-major.
#   4. Per-entity z-score against early-baseline (first
#      healthy_window_end samples of each row; Bessel-corrected
#      stddev). Drop-mask on baseline stddev < 1e-6.
#   5. 6-decimal fixed-point output; LF newlines.
#
# Usage:
#   python3 data/recipes/radioml_gold_large.py \
#       --hdf5 "/run/media/one/toshiba4TB/dsfb-rf datasets/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5" \
#       --out  data/fixtures/radioml_gold_1024x1024.tsv
#
# Panel-locked non-claims (in every audit_report.html):
#   - DSFB-GPU does NOT classify modulation type or recover the
#     RadioML mod-class labels.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     an RF-domain-truth claim.

import argparse
import math
import sys
from pathlib import Path

import h5py
import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project RadioML 2018 GOLD full corpus HDF5 → residual-projection v2 TSV"
    )
    ap.add_argument("--hdf5", required=True,
                    help="Path to GOLD_XYZ_OSC.0001_1024.hdf5")
    ap.add_argument("--out", default="data/fixtures/radioml_gold_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    hdf5_path = Path(args.hdf5)
    if not hdf5_path.is_file():
        print(f"FATAL: HDF5 not found at {hdf5_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with h5py.File(str(hdf5_path), "r") as f:
        if "X" not in f:
            print("FATAL: HDF5 has no /X member", file=sys.stderr)
            return 3
        ds = f["X"]
        if ds.shape[0] < n_entities or ds.shape[1] < n_windows:
            print(f"FATAL: /X shape {ds.shape} too small for {n_entities}x{n_windows}",
                  file=sys.stderr)
            return 4
        # Read I and Q components separately as float64 to keep the
        # magnitude math bit-stable across numpy versions.
        iq = ds[:n_entities, :n_windows, :2].astype(np.float64)

    # Magnitude per sample; reshape to (entities, windows).
    block = np.sqrt(iq[:, :, 0] ** 2 + iq[:, :, 1] ** 2)

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
        fh.write(f"# dataset_id=radioml_gold_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={hdf5_path.name}\n")
        fh.write("# upstream_source=DeepSig RadioML 2018.01 (GOLD_XYZ_OSC; https://www.deepsig.ai/datasets)\n")
        fh.write("# license=CC-BY-NC-SA-4.0 (research-use; non-commercial)\n")
        fh.write("# attribution=O'Shea, Corgan; DeepSig 2018\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write("# source_field=/X (real+imag IQ pairs); magnitude=sqrt(I^2+Q^2)\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip (audit ingest drops NaN cells)\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=Full-corpus RadioML 2018 GOLD I/Q magnitude residual; throughput witness, not modulation-class claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    byte_size = out_path.stat().st_size
    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {byte_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
