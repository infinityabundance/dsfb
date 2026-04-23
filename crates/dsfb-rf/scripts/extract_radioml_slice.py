#!/usr/bin/env python3
"""
Extract a stratified, size-bounded slice of real RadioML IQ data for the repo.

WHY
---
The full RadioML 2018.01a GOLD_XYZ_OSC.0001_1024.hdf5 is ~20 GB — too large to
track in git. The canonical paper evaluation uses the full file (see the paper
§Stage III and `docs/radioml_oracle_protocol.md`). This slice is a *smoke-test
asset* that lets `examples/radioml_hdf5.rs` and the `hdf5_loader` feature run
on REAL IQ samples in CI and on any fresh clone — without shipping 20 GB.

WHAT IS PRODUCED
----------------
  data/slices/radioml_2018_slice.hdf5
    - Schema identical to the parent GOLD_XYZ_OSC file: X, Y, Z datasets
    - X: {N, 1024, 2}  float32    — real/imag IQ samples
    - Y: {N, 24}       int64      — one-hot modulation-class labels
    - Z: {N, 1}        int64      — SNR labels in dB
    - Stratified across 24 modulations × 5 SNRs × 2 captures = 240 captures

  data/slices/deepsig_2018_snr30_slice.hdf5
    - Schema identical to the legacy DEEPSIG_2018_SNR30.hdf5: single `dataset`
    - {N, 1024}        float32    — amplitude envelopes at +30 dB SNR
    - 100 captures from the head of the file

  data/slices/SLICE_MANIFEST.json
    - Parent file path, parent SHA-256, parent size, slice SHA-256, slice size
    - Stratification plan (SNRs + modulations × captures)
    - License / provenance: DeepSig RadioML 2018.01a, CC BY-NC-SA 4.0 derivative

ACADEMIC HONESTY
----------------
The slice is NOT the paper's evaluation dataset. Headline precision / recall /
compression-ratio numbers in Table 1 still require the full 20 GB file. Running
the example on the slice yields smoke-test-level traces — enough to demonstrate
the HDF5 ingest path works, NOT to replicate paper metrics. This distinction is
reinforced by the banner in `examples/radioml_hdf5.rs` and in REPRODUCE.md.

USAGE
-----
    python3 scripts/extract_radioml_slice.py \
        --gold   "data/RadioML HDF5/GOLD_XYZ_OSC.0001_1024.hdf5" \
        --deepsig "data/RadioML HDF5/DEEPSIG_2018_SNR30.hdf5" \
        --out    data/slices/

Re-run after any schema or stratification change. Do not hand-edit the slice
files — regenerate via this script so the SLICE_MANIFEST.json provenance chain
stays sound.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from pathlib import Path

import h5py
import numpy as np

# --- Stratification policy -----------------------------------------------------
# Modulations: RadioML 2018.01a uses 24 classes. We include *all* classes so the
# slice covers the full modulation space; narrowing here would violate the
# broad prior-art posture of the companion paper.
N_MODULATIONS: int = 24
# SNRs (dB) to stratify — chosen to span the operating regime claimed in the
# paper: two below the SNR floor (-10 dB per L10), zero, and two above.
SNRS_DB: list[int] = [-10, 0, 10, 20, 30]
# Captures per (modulation, SNR) cell. 2 keeps the slice under ~2 MB while
# giving each cell a non-trivial sample.
CAPTURES_PER_CELL: int = 2

# DEEPSIG legacy slice: 100 captures from the head.
DEEPSIG_SLICE_N: int = 100


def sha256_of_file(path: Path, chunk_size: int = 4 * 1024 * 1024) -> str:
    """SHA-256 of a file on disk, chunked for memory safety on 20 GB inputs."""
    h = hashlib.sha256()
    with path.open("rb") as fh:
        while True:
            buf = fh.read(chunk_size)
            if not buf:
                break
            h.update(buf)
    return h.hexdigest()


def extract_gold(gold_path: Path, out_path: Path) -> dict:
    """Stratified extraction from the canonical 2018.01a GOLD_XYZ_OSC file."""
    if not gold_path.exists():
        raise FileNotFoundError(f"GOLD parent file missing: {gold_path}")

    with h5py.File(gold_path, "r") as src:
        for k in ("X", "Y", "Z"):
            if k not in src:
                raise RuntimeError(f"GOLD file missing expected dataset '{k}'")
        X_src, Y_src, Z_src = src["X"], src["Y"], src["Z"]
        n_total, n_samp, n_iq = X_src.shape
        n_classes = Y_src.shape[1]
        if n_classes != N_MODULATIONS:
            raise RuntimeError(
                f"Expected {N_MODULATIONS} classes in Y, got {n_classes}"
            )

        # Z is (N,1) int64; flatten to shape (N,).
        z_flat = Z_src[:, 0]
        # Y is (N, 24) one-hot int64; argmax gives class index.
        #   Loading all of Y at once is ~490 MB — fine in RAM on modern hosts.
        y_cls = np.argmax(Y_src[:], axis=1)

        selected_indices: list[int] = []
        strat_plan: list[dict] = []

        for cls in range(N_MODULATIONS):
            for snr in SNRS_DB:
                mask = (y_cls == cls) & (z_flat == snr)
                matches = np.flatnonzero(mask)
                if matches.size == 0:
                    # Not every (class, SNR) cell exists in the parent file;
                    # record and continue instead of failing — avoids silent
                    # over-fitting to an assumed grid.
                    strat_plan.append(
                        {
                            "class": int(cls),
                            "snr_db": int(snr),
                            "available": 0,
                            "selected": 0,
                        }
                    )
                    continue
                # Deterministic selection: first CAPTURES_PER_CELL indices.
                take = matches[:CAPTURES_PER_CELL]
                selected_indices.extend(int(i) for i in take)
                strat_plan.append(
                    {
                        "class": int(cls),
                        "snr_db": int(snr),
                        "available": int(matches.size),
                        "selected": int(take.size),
                    }
                )

        # Sorted ascending for reproducible HDF5 write order.
        selected_indices.sort()
        n_out = len(selected_indices)

        # Build output buffers.
        X_out = np.empty((n_out, n_samp, n_iq), dtype=X_src.dtype)
        Y_out = np.empty((n_out, n_classes), dtype=Y_src.dtype)
        Z_out = np.empty((n_out, 1), dtype=Z_src.dtype)

        for j, i in enumerate(selected_indices):
            X_out[j] = X_src[i]
            Y_out[j] = Y_src[i]
            Z_out[j] = Z_src[i]

    out_path.parent.mkdir(parents=True, exist_ok=True)
    with h5py.File(out_path, "w") as dst:
        dst.create_dataset("X", data=X_out, compression="gzip", compression_opts=4)
        dst.create_dataset("Y", data=Y_out, compression="gzip", compression_opts=4)
        dst.create_dataset("Z", data=Z_out, compression="gzip", compression_opts=4)
        dst.attrs["parent_path"] = str(gold_path.name)
        dst.attrs["stratification"] = json.dumps(
            {
                "modulations": N_MODULATIONS,
                "snrs_db": SNRS_DB,
                "captures_per_cell": CAPTURES_PER_CELL,
                "total_captures": n_out,
            }
        )
        dst.attrs["disclaimer"] = (
            "Smoke-test slice of RadioML 2018.01a GOLD_XYZ_OSC; not the paper's "
            "evaluation dataset. See REPRODUCE.md in dsfb-rf crate root."
        )

    return {
        "n_captures": n_out,
        "strat_plan": strat_plan,
        "parent_shape_total": int(n_total),
        "sample_len": int(n_samp),
    }


def extract_deepsig(deepsig_path: Path, out_path: Path, n: int) -> dict:
    """Head-slice of the legacy DEEPSIG_2018_SNR30 single-SNR file."""
    if not deepsig_path.exists():
        raise FileNotFoundError(f"DEEPSIG parent file missing: {deepsig_path}")
    with h5py.File(deepsig_path, "r") as src:
        if "dataset" not in src:
            raise RuntimeError("DEEPSIG file missing expected 'dataset' key")
        ds = src["dataset"]
        n_total, n_samp = ds.shape
        if n > n_total:
            raise RuntimeError(f"Requested {n} captures but parent has {n_total}")
        sliced = ds[:n]
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with h5py.File(out_path, "w") as dst:
        dst.create_dataset(
            "dataset", data=sliced, compression="gzip", compression_opts=4
        )
        dst.attrs["parent_path"] = str(deepsig_path.name)
        dst.attrs["head_slice_n"] = n
        dst.attrs["snr_db_nominal"] = 30
        dst.attrs["disclaimer"] = (
            "Head slice of DEEPSIG_2018_SNR30 legacy file. Not the paper's "
            "evaluation dataset."
        )
    return {"n_captures": n, "parent_shape_total": int(n_total), "sample_len": int(n_samp)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--gold", type=Path, required=True)
    parser.add_argument("--deepsig", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True, help="output directory")
    args = parser.parse_args()

    out_dir: Path = args.out
    out_dir.mkdir(parents=True, exist_ok=True)
    gold_out = out_dir / "radioml_2018_slice.hdf5"
    deepsig_out = out_dir / "deepsig_2018_snr30_slice.hdf5"
    manifest_out = out_dir / "SLICE_MANIFEST.json"

    print(f"[slice] hashing parents...", flush=True)
    gold_sha = sha256_of_file(args.gold)
    gold_size = args.gold.stat().st_size
    deepsig_sha = sha256_of_file(args.deepsig)
    deepsig_size = args.deepsig.stat().st_size

    print(f"[slice] extracting GOLD slice -> {gold_out}", flush=True)
    gold_info = extract_gold(args.gold, gold_out)

    print(f"[slice] extracting DEEPSIG slice -> {deepsig_out}", flush=True)
    deepsig_info = extract_deepsig(args.deepsig, deepsig_out, DEEPSIG_SLICE_N)

    manifest = {
        "schema_version": 1,
        "generated_by": "scripts/extract_radioml_slice.py",
        "purpose": (
            "Smoke-test slice of real RadioML IQ data for the dsfb-rf crate. "
            "Not the paper's evaluation dataset."
        ),
        "license": "DeepSig RadioML 2018.01a derivative; CC BY-NC-SA 4.0",
        "parents": {
            "gold": {
                "filename": args.gold.name,
                "size_bytes": gold_size,
                "sha256": gold_sha,
            },
            "deepsig": {
                "filename": args.deepsig.name,
                "size_bytes": deepsig_size,
                "sha256": deepsig_sha,
            },
        },
        "slices": {
            "radioml_2018_slice.hdf5": {
                "strategy": "stratified by class x SNR",
                "modulations": N_MODULATIONS,
                "snrs_db": SNRS_DB,
                "captures_per_cell": CAPTURES_PER_CELL,
                "n_captures": gold_info["n_captures"],
                "stratification_plan": gold_info["strat_plan"],
                "sha256": sha256_of_file(gold_out),
                "size_bytes": gold_out.stat().st_size,
            },
            "deepsig_2018_snr30_slice.hdf5": {
                "strategy": "head slice",
                "n_captures": deepsig_info["n_captures"],
                "snr_db_nominal": 30,
                "sha256": sha256_of_file(deepsig_out),
                "size_bytes": deepsig_out.stat().st_size,
            },
        },
    }

    manifest_out.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"[slice] manifest -> {manifest_out}", flush=True)
    print(
        "[slice] done. "
        f"gold_slice={manifest['slices']['radioml_2018_slice.hdf5']['size_bytes']:,} B  "
        f"deepsig_slice={manifest['slices']['deepsig_2018_snr30_slice.hdf5']['size_bytes']:,} B"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
