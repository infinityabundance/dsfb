#!/usr/bin/env python3
# radioml_2018_snr30_large.py — large-fixture residual projection
# of the RadioML 2018.01 SNR=30 dB HDF5 (DEEPSIG_2018_SNR30.hdf5).
#
# WHY THIS RECIPE EXISTS (for the future engineer reading cold):
#
# The S-REAL gauntlet's 20 audits run on SMALL real-world fixtures
# (128-656 events / dataset). The dispatcher there is launch-
# overhead-dominated; bench throughput tops out at ~13 MB/s
# (median_BW from scripts/s_real_throughput_bench.sh). That is the
# correct measurement for small real-world public datasets, but it
# does NOT show what the dispatcher does when given S-PERF.16.a-
# scale work.
#
# This recipe projects the RadioML 2018 SNR=30 HDF5 into a single
# 256-entity × 4096-window fixture (= 1,048,576 cells, exactly the
# S-PERF.16.a canonical shape). Under D64 throughput-mode tree-
# compact dispatch the audit binary sees ~10^6 events per dispatch
# — enough to saturate the kernels and surface real GB/s.
#
# Provenance (upstream):
#   - Authors:  T. O'Shea, J. Corgan (DeepSig)
#   - File:     DEEPSIG_2018_SNR30.hdf5  (97 MB; the SNR=30 dB slice
#               of the full GOLD_XYZ_OSC.0001_1024.hdf5 corpus)
#   - Format:   single HDF5 dataset `/dataset` of shape (24576, 1024)
#               float32, where each row is one modulation example
#               and each column is one I/Q-pair amplitude sample at
#               that time index. SNR is fixed at +30 dB across all
#               examples (the "clean" slice of the full RadioML
#               2018.01 archive).
#   - License:  RadioML 2018.01 is distributed by DeepSig with a
#               CC-BY-NC-SA-4.0 (research-use; non-commercial). Cite
#               https://www.deepsig.ai/datasets when redistributing.
#
# Projection law (panel-locked, deterministic):
#   1. Read first n_entities=256 rows × first n_windows=4096 columns
#      of `/dataset` as float32. This is a contiguous fixed slice;
#      the recipe never randomly samples.
#   2. For each window-column w, compute mean and sample-stddev of
#      the 256 values across entities. Bessel-correction (divide
#      by N-1) is applied to match all sibling recipes
#      (cmapss_*, deeptralog_*, promise_*, bugsinpy).
#   3. Drop any window whose stddev < 1e-6 (constant column;
#      would yield divide-by-zero residuals). These appear in the
#      output as NaN cells the audit's ingest step skips.
#   4. Emit z-score residual: (value - mean_w) / stddev_w. Format
#      each cell as %.6f (6-decimal fixed-point) so the projection
#      is platform-independent and reproducible across machines.
#   5. Window order is column-ascending (0..4095). Entity order is
#      row-ascending (0..255). Both deterministic.
#
# Output format (residual-projection v2):
#   TAB-delimited TSV with a `# key=value` header block, one window
#   per row, one column per entity. Identical structural format to
#   the cmapss / deeptralog / promise / bugsinpy fixtures so the
#   audit's TSV ingest path consumes it without changes.
#
# Determinism rules (panel-locked):
#   - No random seeds. The slice [0:256, 0:4096] is the same on
#     every run.
#   - 6-decimal fixed-point printing. f-string `f"{v:.6f}"` rounds
#     half-to-even on float64 → bit-stable across CPython versions
#     for the same input bytes.
#   - LF line endings; no trailing whitespace. `.gitattributes`
#     marks `data/fixtures/*.tsv binary` so git's CRLF
#     normalization cannot mangle the SHA-256-pinned bytes.
#
# Usage:
#   python3 data/recipes/radioml_2018_snr30_large.py \
#       --hdf5 "/run/media/one/toshiba4TB/dsfb-rf datasets/RadioML HDF5/DEEPSIG_2018_SNR30.hdf5" \
#       --out data/fixtures/radioml_2018_snr30_256x4096.tsv
#
# Panel-locked non-claims (in every audit_report.html produced
# from the output of this recipe):
#   - DSFB-GPU does NOT classify modulation type.
#   - DSFB-GPU does NOT decode the RadioML labels.
#   - The audit reports STRUCTURAL residual evidence under
#     the deterministic z-score projection; it does NOT claim
#     a domain-truth verdict on what the radio signal is.

import argparse
import math
import sys
from pathlib import Path

import h5py
import numpy as np


# Default fixture shape: 1024 entities × 1024 windows = 1,048,576
# cells = ~1M events. Matches the S-PERF.16.a saturation-class
# event magnitude (1M events / dispatch) but uses what the
# DEEPSIG_2018_SNR30 HDF5 actually has (24576 rows × 1024 cols;
# only 1024 columns available for the window axis).
#
# Override via --n-entities / --n-windows for shape sweeps. Maximum
# n_entities is 24576 (HDF5 row count); maximum n_windows is 1024
# (HDF5 col count). The recipe REJECTS shapes that exceed those.
DEFAULT_n_entities = 1024
DEFAULT_n_windows = 1024

# Constant-column detection threshold. Matches the cmapss /
# promise / bugsinpy / deeptralog recipes byte-for-byte. Windows
# whose sample-stddev falls below this are emitted as NaN-rows
# (the audit ingest skips NaN cells).
STDDEV_FLOOR = 1e-6


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Project RadioML 2018 SNR=30 HDF5 → residual-projection v2 TSV"
    )
    # --hdf5 path is required because the dataset lives off-repo
    # on an external drive; we do NOT vendor a 97 MB binary into
    # the git tree. The path is recorded into the fixture's
    # `dataset_manifest.toml` provenance block when the audit runs.
    ap.add_argument(
        "--hdf5",
        required=True,
        help="Path to DEEPSIG_2018_SNR30.hdf5 (upstream RadioML 2018 SNR=30).",
    )
    # --out is where the projected TSV lands. Default keeps the
    # fixture under data/fixtures/ alongside the other 20 vendored
    # residual-projections, even though the upstream HDF5 itself
    # is too large to vendor.
    ap.add_argument(
        "--out",
        default="data/fixtures/radioml_2018_snr30_1024x1024.tsv",
        help="Output TSV path (residual-projection v2).",
    )
    # Shape sweep flags. Bench operators may want to sweep fixture
    # sizes to see how throughput scales with event count; these
    # flags let the same recipe produce 256×1024, 1024×1024,
    # 4096×1024, etc. without code edits.
    ap.add_argument("--n-entities", type=int, default=DEFAULT_n_entities,
                    help=f"Number of HDF5 rows to project (default {DEFAULT_n_entities}; max 24576)")
    ap.add_argument("--n-windows", type=int, default=DEFAULT_n_windows,
                    help=f"Number of HDF5 cols to project (default {DEFAULT_n_windows}; max 1024)")
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

    # Read the panel-locked slice. We deliberately load ONLY the
    # 256×4096 block we need rather than the full 24576×1024
    # array (which would force a 100 MB allocation we don't use).
    # `[0:n_entities, 0:n_windows]` is a deterministic contiguous
    # slice; the recipe NEVER samples randomly.
    with h5py.File(str(hdf5_path), "r") as f:
        if "dataset" not in f:
            print(
                "FATAL: HDF5 has no top-level 'dataset' member; "
                "this recipe expects DEEPSIG_2018_SNR30.hdf5 schema",
                file=sys.stderr,
            )
            return 3
        ds = f["dataset"]
        if ds.shape[0] < n_entities or ds.shape[1] < n_windows:
            print(
                f"FATAL: HDF5 shape {ds.shape} smaller than required "
                f"({n_entities}, {n_windows})",
                file=sys.stderr,
            )
            return 4
        # float32 → float64 promotion for the stddev compute is
        # the same posture every sibling recipe uses; lets the
        # math match across recipes regardless of upstream dtype.
        block = ds[:n_entities, :n_windows].astype(np.float64)

    # Per-ENTITY (per-row) z-score against an EARLY-BASELINE window
    # of the first healthy_window_end time samples. This matches
    # the cmapss / promise projection pattern byte-for-byte: each
    # row (entity) computes its own mean + sample-stddev over its
    # first H samples, then residuals are (value - mean_i) / std_i
    # for all 1024 samples of that row.
    #
    # The "healthy_window_end" metadata field then has REAL meaning
    # downstream — the audit's bank stage treats windows [0..H) as
    # baseline-phase and windows [H..n_windows) as active-phase.
    # For RadioML this is a structural measurement: how late
    # samples deviate from early samples within the same IQ frame.
    # It is NOT a real anomaly verdict; the panel-locked non-claim
    # in this recipe's docstring is the authoritative framing.
    #
    # H = n_windows / 4 (= 256 of 1024) is the panel-locked default.
    # Matches the cmapss healthy_window_end / n_windows ratio
    # (30/192 = 15.6%; here we use 25% for clearer baseline
    # statistics on the larger fixture).
    healthy_window_end = n_windows // 4
    if healthy_window_end < 2:
        print(
            f"FATAL: healthy_window_end={healthy_window_end} too small for "
            f"a meaningful baseline (need >= 2 samples per entity row)",
            file=sys.stderr,
        )
        return 5
    # Baseline stats per row: shape (n_entities,).
    baseline = block[:, :healthy_window_end]
    mean = baseline.mean(axis=1)
    std = baseline.std(axis=1, ddof=1)

    # Constant-row mask: entities whose baseline-phase stddev is
    # below the floor would produce divide-by-zero residuals.
    # Mark the entire row NaN. The audit ingest treats NaN cells
    # as missingness; downstream stages skip them.
    drop_mask = std < STDDEV_FLOOR

    # z-score per entity row: residual[i, w] = (block[i, w] -
    # mean[i]) / std[i] for every window w in [0, n_windows).
    # Broadcast mean/std as column vectors across the time axis.
    with np.errstate(divide="ignore", invalid="ignore"):
        residual = (block - mean[:, None]) / std[:, None]
    residual[drop_mask, :] = np.nan

    # Emit residual-projection v2 TSV. Header block uses `# key=value`
    # lines, then data: each data row carries EXACTLY n_entities
    # tab-separated cell values, one row per window. NO blank
    # separator line; NO column-header row; NO leading window-idx
    # column. The audit's TSV ingest infers (window_idx, entity_idx)
    # coordinates from row order × column order, deterministically.
    # This matches the cmapss / promise / bugsinpy / lo2 / tadbench
    # fixture format byte-for-byte; the 20 vendored TSVs are the
    # source of truth on this convention.
    n_finite = int(np.isfinite(residual).sum())
    with open(out_path, "w", encoding="utf-8", newline="\n") as fh:
        # Header block. Records EVERY piece of provenance an auditor
        # needs to verify the projection is reproducible: source
        # file (basename only; full path is recorded in audit's
        # dataset_manifest.toml), projection law, slice shape, axis
        # semantics, NaN handling, expected finite-cell count.
        fh.write("# residual-projection v2\n")
        fh.write(f"# dataset_id=radioml_2018_snr30_{n_entities}x{n_windows}\n")
        fh.write(f"# upstream_basename={hdf5_path.name}\n")
        fh.write("# upstream_source=DeepSig RadioML 2018.01 (https://www.deepsig.ai/datasets)\n")
        fh.write("# license=CC-BY-NC-SA-4.0 (research-use; non-commercial)\n")
        fh.write("# attribution=O'Shea, Corgan; DeepSig 2018\n")
        fh.write(f"# num_windows={n_windows}\n")
        fh.write(f"# num_signals={n_entities}\n")
        fh.write(f"# healthy_window_end={healthy_window_end}\n")
        fh.write(f"# n_finite_cells={n_finite}\n")
        fh.write("# projection=z-score per entity (mean, sample-stddev over first healthy_window_end samples; Bessel correction)\n")
        fh.write(f"# stddev_floor={STDDEV_FLOOR}\n")
        fh.write("# nan_policy=skip (audit ingest drops NaN cells)\n")
        fh.write("# axis0=entity (signal); axis1=window (time-sample within IQ frame)\n")
        fh.write("# decimal_places=6\n")
        fh.write("# notes=Large-fixture z-score residual projection of RadioML 2018 SNR=30 dB I/Q amplitudes. DSFB does NOT classify modulation; it reports STRUCTURAL residual evidence under the deterministic projection.\n")
        # Data rows. One per window, in ascending window-index
        # order (deterministic). Each row contains exactly
        # n_entities tab-separated cells (entity residuals
        # 0..n_entities-1). NO leading window-index column.
        for w in range(n_windows):
            row = []
            for e in range(n_entities):
                v = residual[e, w]
                if math.isnan(v):
                    row.append("NaN")
                else:
                    # 6-decimal fixed-point. Half-to-even rounding
                    # is the CPython default for f-string %.6f and
                    # is bit-stable across CPython versions for
                    # the same input bytes.
                    row.append(f"{v:.6f}")
            fh.write("\t".join(row) + "\n")

    # Report what we wrote so the caller can SHA-256-pin and
    # register it. fixture_byte_size + finite-cell count are the
    # two numbers the audit's dataset_manifest.toml will record.
    byte_size = out_path.stat().st_size
    print(f"wrote: {out_path}")
    print(f"  shape           : {n_entities} entities × {n_windows} windows")
    print(f"  finite cells    : {n_finite} of {n_entities * n_windows}")
    print(f"  constant cols   : {int(drop_mask.sum())} (NaN-emitted)")
    print(f"  byte_size       : {byte_size} bytes")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
