#!/usr/bin/env python3
# oracle_large.py — large-fixture residual projection of an ORACLE
# IoT-fingerprinting Wi-Fi I/Q capture (SigMF cf32 from NEU/KRI).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# ORACLE's `neu_m044q5210.zip` contains 16 distinct USRP X310
# transmitters' over-the-air IEEE 802.11a recordings collected at
# multiple TX-RX distances (2 ft, 8 ft, 14 ft, etc.). Each capture
# is a SigMF pair (.sigmf-data + .sigmf-meta), with the .sigmf-data
# file holding 40M cf32 IQ samples (320 MB).
#
# This recipe projects ONE canonical capture (8ft / X310 device id
# 3123D7D / run1) as a 1024×1024 IQ-magnitude residual. Pinned by
# inner-file name so two runs against the same zip produce the
# same TSV bytes.
#
# Provenance (upstream):
#   - Source:  NEU Kostas Research Institute (KRI), 16-device
#              fingerprinting dataset (Sankhe et al., INFOCOM 2019)
#   - File:    KRI-16Devices-RawData/8ft/WiFi_air_X310_3123D7D_8ft_run1.sigmf-data
#              inside neu_m044q5210.zip
#   - Format:  SigMF v0.02, cf32 (interleaved float32 I/Q),
#              5 MHz sample rate, 40,012,800 complex samples
#   - License: research-use; the upstream zip does not include
#              an SPDX LICENSE; recorded as `no-upstream-license`
#              per the S-REAL.2c convention.
#
# Projection law (panel-locked, deterministic; mirrors POWDER
# and DeepBeam recipes byte-for-byte except for the source file).
#
# Usage:
#   python3 data/recipes/oracle_large.py \
#       --zip "/run/media/one/toshiba4TB/dsfb-rf datasets/ORACLE/neu_m044q5210.zip" \
#       --out data/fixtures/oracle_1024x1024.tsv
#
# Panel-locked non-claims:
#   - DSFB-GPU does NOT identify the transmitter (device id),
#     classify WiFi frames, or decode 802.11a packets.
#   - The audit reports STRUCTURAL residual evidence under the
#     deterministic projection; it is a throughput witness, not
#     a device-fingerprinting claim.

import argparse
import math
import sys
import zipfile
from pathlib import Path

import numpy as np

DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024
DEFAULT_INNER_SIGMF = "KRI-16Devices-RawData/8ft/WiFi_air_X310_3123D7D_8ft_run1.sigmf-data"
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project ORACLE SigMF cf32 → residual-projection v2 TSV"
    )
    ap.add_argument("--zip", required=True,
                    help="Path to neu_m044q5210.zip")
    ap.add_argument("--out", default="data/fixtures/oracle_1024x1024.tsv",
                    help="Output TSV path")
    ap.add_argument("--inner-sigmf-data", default=DEFAULT_INNER_SIGMF,
                    help=f"Inner .sigmf-data name (default {DEFAULT_INNER_SIGMF})")
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

    needed_complex = n_entities * n_windows
    needed_bytes = needed_complex * 8  # cf32: 2 floats × 4 bytes

    with zipfile.ZipFile(str(zip_path), "r") as zf:
        try:
            info = zf.getinfo(args.inner_sigmf_data)
        except KeyError:
            print(f"FATAL: zip member not found: {args.inner_sigmf_data}", file=sys.stderr)
            return 3
        if info.file_size < needed_bytes:
            print(f"FATAL: {args.inner_sigmf_data} has {info.file_size} bytes; "
                  f"need {needed_bytes}", file=sys.stderr)
            return 4
        with zf.open(args.inner_sigmf_data, "r") as fh:
            raw = fh.read(needed_bytes)

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
        fh.write(f"# dataset_id=oracle_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={zip_path.name}\n")
        fh.write(f"# upstream_inner_sigmf={args.inner_sigmf_data}\n")
        fh.write("# upstream_source=NEU/KRI 16-Device ORACLE Wi-Fi fingerprinting (Sankhe et al. INFOCOM 2019); neu_m044q5210\n")
        fh.write("# license=no-upstream-license (research-fair-use convention)\n")
        fh.write("# attribution=Sankhe, K.; Belgiovine, M.; Zhou, F.; Riyaz, S.; Ioannidis, S.; Chowdhury, K.; NEU 2019\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write(f"# n_dropped_rows={n_dropped}\n")
        fh.write("# source_field=SigMF cf32 (interleaved float32 IQ), 5 MHz sample rate\n")
        fh.write("# magnitude_formula=sqrt(I^2+Q^2) per sample\n")
        fh.write("# projection=z-score per entity against first healthy_window_end samples; Bessel correction\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=ORACLE 802.11a I/Q magnitude residual; throughput witness, not device-fingerprinting claim.\n")
        for w in range(n_windows):
            row = ["NaN" if math.isnan(residual[e, w]) else f"{residual[e, w]:.6f}"
                   for e in range(n_entities)]
            fh.write("\t".join(row) + "\n")

    print(f"wrote: {out_path}")
    print(f"  shape {n_entities}x{n_windows}  finite {n_finite}/{n_entities*n_windows}  dropped {n_dropped}  bytes {out_path.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
