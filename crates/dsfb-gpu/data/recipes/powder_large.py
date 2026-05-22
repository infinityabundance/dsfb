#!/usr/bin/env python3
# powder_large.py — large-fixture residual projection of a POWDER
# LTE-band I/Q capture from the GlobecomPOWDER 2020 archive.
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# POWDER's `neu_m046tb444.zip` contains hundreds of 4G/5G LTE-band
# I/Q recordings collected by the University of Utah POWDER testbed.
# Each `4G_Day_*_<location>_s*.bin` file is `cf32` (complex float32:
# 4-byte real + 4-byte imag, interleaved) at a 7.69 MHz sample rate,
# typically 5.3M complex samples (42.4 MB).
#
# This recipe projects ONE canonical capture (the first BES s1
# station-1 file) as a 1024×1024 IQ-magnitude residual, matching
# the saturation-class shape used by RadioML / DeepBeam. The
# capture identity is hardcoded so two runs of the recipe against
# the same upstream zip produce the same TSV bytes.
#
# Provenance (upstream):
#   - Source:  University of Utah POWDER testbed (Globecom 2020)
#   - File:    GlobecomPOWDER/4G_Day_1_bes_s1.bin inside
#              neu_m046tb444.zip (sigmf-style cf32 binary).
#   - Format:  cf32 (interleaved float32 I/Q), 5_300_000 samples
#              per file at 7.69 MHz, LTE Band 7 center 2.685 GHz.
#   - License: research-use; the upstream zip does not include a
#              LICENSE file; recorded as `no-upstream-license`
#              per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic):
#   1. Open the zip read-only and stream the first
#      n_entities*n_windows*8 bytes of the named .bin file into a
#      float32 (2*n,) array. Reshape to (n, 2). Each row is one
#      complex sample (real, imag).
#   2. Magnitude: |z| = sqrt(I^2 + Q^2) per sample → 1-D array of
#      n complex samples.
#   3. Reshape to (n_entities, n_windows) row-major.
#   4. Per-entity z-score against early-baseline (first
#      healthy_window_end samples of each row; Bessel-corrected
#      stddev). Drop-mask on baseline stddev < 1e-6.
#   5. 6-decimal fixed-point output; LF newlines.
#
# Usage:
#   python3 data/recipes/powder_large.py \
#       --zip "/run/media/one/toshiba4TB/dsfb-rf datasets/POWDER/neu_m046tb444.zip" \
#       --out data/fixtures/powder_1024x1024.tsv
#
# Override the inner .bin file to project a different capture via
# --inner-bin (default: GlobecomPOWDER/4G_Day_1_bes_s1.bin).
#
# Panel-locked non-claims (in every audit_report.html):
#   - DSFB-GPU does NOT classify cell, sector, eNB, modulation,
#     or transmitter on POWDER data.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     an LTE-domain-truth claim.

import argparse
import math
import sys
import zipfile
from pathlib import Path

import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
DEFAULT_INNER_BIN = "GlobecomPOWDER/4G_Day_1_bes_s1.bin"
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project POWDER LTE I/Q .bin → residual-projection v2 TSV"
    )
    ap.add_argument("--zip", required=True,
                    help="Path to neu_m046tb444.zip")
    ap.add_argument("--out", default="data/fixtures/powder_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--inner-bin", default=DEFAULT_INNER_BIN,
                    help=f"Inner .bin name (default {DEFAULT_INNER_BIN})")
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities)
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows)
    args = ap.parse_args()
    n_entities, n_windows = args.n_entities, args.n_windows

    zip_path = Path(args.zip)
    if not zip_path.is_file():
        print(f"FATAL: zip not found at {zip_path}", file=sys.stderr)
        return 2
    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Number of complex samples needed; each is 8 bytes (cf32).
    needed_complex = n_entities * n_windows
    needed_bytes = needed_complex * 8

    with zipfile.ZipFile(str(zip_path), "r") as zf:
        try:
            info = zf.getinfo(args.inner_bin)
        except KeyError:
            print(f"FATAL: zip member not found: {args.inner_bin}", file=sys.stderr)
            return 3
        if info.file_size < needed_bytes:
            print(f"FATAL: {args.inner_bin} has {info.file_size} bytes; "
                  f"need {needed_bytes} ({needed_complex} cf32 samples)",
                  file=sys.stderr)
            return 4
        with zf.open(args.inner_bin, "r") as fh:
            raw = fh.read(needed_bytes)

    # Decode cf32: pairs of float32 (real, imag), little-endian.
    iq32 = np.frombuffer(raw, dtype="<f4", count=needed_complex * 2)
    iq32 = iq32.reshape(needed_complex, 2).astype(np.float64)

    magnitude = np.sqrt(iq32[:, 0] ** 2 + iq32[:, 1] ** 2)
    block = magnitude.reshape(n_entities, n_windows)

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
        fh.write(f"# dataset_id=powder_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={zip_path.name}\n")
        fh.write(f"# upstream_inner_bin={args.inner_bin}\n")
        fh.write("# upstream_source=University of Utah POWDER (Globecom 2020); neu_m046tb444\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=POWDER testbed; University of Utah\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write("# source_field=cf32 interleaved float32 IQ pairs (sigmf-style)\n")
        fh.write("# sample_rate_hz=7690000 (LTE Band 7, center 2.685 GHz)\n")
        fh.write("# magnitude_formula=sqrt(I^2+Q^2) per sample\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=POWDER 4G LTE Band-7 I/Q magnitude residual; throughput witness, not LTE-domain claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
