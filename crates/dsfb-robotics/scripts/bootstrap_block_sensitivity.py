#!/usr/bin/env python3
"""Block-length sensitivity for the stationary block bootstrap.

Default `bootstrap_census.py` uses block length L = W = 8. A reviewer
could reasonably ask: how robust are the reported CIs to the choice
of L? This script runs the bootstrap with L ∈ {4, 8, 16, 32} on a
single exemplar dataset (panda_gaz, the literal published-θ̂ row)
and emits the CI half-widths for each block length.

Stable CI half-widths across L → bootstrap robust to autocorrelation
modelling assumption. Drift in CI half-widths → the bootstrap is
detecting genuine block-length-sensitive structure in the residual
stream's autocorrelation, which is itself an interesting empirical
finding worth surfacing.

Output: `audit/bootstrap/panda_gaz_block_sensitivity.json`.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np

CRATE_ROOT = Path(__file__).resolve().parent.parent
PROCESSED_ROOT = CRATE_ROOT / "data" / "processed"
OUT_PATH = CRATE_ROOT / "audit" / "bootstrap" / "panda_gaz_block_sensitivity.json"
PAPER_LOCK_BIN = CRATE_ROOT / "target" / "release" / "paper-lock"

SLUG = "panda_gaz"
N_REPLICATES = 500   # fewer than the main bootstrap to keep the multi-L run tractable
BLOCK_LENGTHS = [4, 8, 16, 32]
RNG_SEED = 20260425


def load_residual_stream(slug: str) -> np.ndarray:
    pub = PROCESSED_ROOT / f"{slug}_published.csv"
    base = PROCESSED_ROOT / f"{slug}.csv"
    target = pub if pub.is_file() else base
    rows: list[float] = []
    with target.open() as fh:
        first = fh.readline().strip()
        try:
            rows.append(float(first.split(",")[-1]))
        except ValueError:
            pass
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(float(line.split(",")[-1]))
            except ValueError:
                continue
    return np.asarray(rows, dtype=np.float64)


def stationary_block_bootstrap(
    stream: np.ndarray, rng: np.random.Generator, block_length: int
) -> np.ndarray:
    n = len(stream)
    out = np.empty(n, dtype=stream.dtype)
    p = 1.0 / float(block_length)
    i = 0
    while i < n:
        start = int(rng.integers(0, n))
        block_len = int(rng.geometric(p))
        block_len = min(block_len, n - i)
        for k in range(block_len):
            out[i + k] = stream[(start + k) % n]
        i += block_len
    return out


def write_csv(stream: np.ndarray, path: Path) -> None:
    with path.open("w") as fh:
        fh.write("residual_norm\n")
        for v in stream:
            fh.write(f"{v:.17g}\n")


def census_via_paper_lock(slug: str, csv_path: Path) -> dict:
    out = subprocess.run(
        [str(PAPER_LOCK_BIN), "--csv-path", str(csv_path), slug],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(out.stdout)["aggregate"]


def replicate(slug: str, stream: np.ndarray, seed: int, block_length: int) -> dict:
    rng = np.random.default_rng(seed)
    rs = stationary_block_bootstrap(stream, rng, block_length)
    fd, p = tempfile.mkstemp(prefix=f"{slug}_blk{block_length}_", suffix=".csv")
    os.close(fd)
    path = Path(p)
    try:
        write_csv(rs, path)
        return census_via_paper_lock(slug, path)
    finally:
        try:
            path.unlink()
        except OSError:
            pass


def main() -> int:
    if not PAPER_LOCK_BIN.is_file():
        print(f"ERROR: missing {PAPER_LOCK_BIN}; run cargo build --release first")
        return 2
    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    stream = load_residual_stream(SLUG)
    point = census_via_paper_lock(SLUG, PROCESSED_ROOT / f"{SLUG}_published.csv")

    out = {
        "dataset": SLUG,
        "n_replicates_per_block_length": N_REPLICATES,
        "block_lengths": BLOCK_LENGTHS,
        "rng_seed": RNG_SEED,
        "point_estimate": point,
        "ci_per_block_length": {},
    }

    for L in BLOCK_LENGTHS:
        print(f"  L={L} ({N_REPLICATES} replicates)...", flush=True)
        boundary, violation, compression = [], [], []
        for i in range(N_REPLICATES):
            seed = RNG_SEED ^ (L * 0xDEAD_BEEF) ^ i
            c = replicate(SLUG, stream, seed, L)
            boundary.append(c["boundary"])
            violation.append(c["violation"])
            compression.append(c["compression_ratio"])

        def stats(arr):
            arr = np.asarray(arr, dtype=np.float64)
            return {
                "mean": float(np.mean(arr)),
                "stddev": float(np.std(arr, ddof=1)),
                "ci_lo_2_5": float(np.percentile(arr, 2.5)),
                "ci_hi_97_5": float(np.percentile(arr, 97.5)),
                "ci_half_width": float(
                    (np.percentile(arr, 97.5) - np.percentile(arr, 2.5)) / 2.0
                ),
            }

        out["ci_per_block_length"][str(L)] = {
            "boundary": stats(boundary),
            "violation": stats(violation),
            "compression_ratio": stats(compression),
        }
        b = out["ci_per_block_length"][str(L)]["boundary"]
        c = out["ci_per_block_length"][str(L)]["compression_ratio"]
        print(
            f"    boundary  mean {b['mean']:>7.1f}  CI ±{b['ci_half_width']:>5.1f}  |  "
            f"compression mean {c['mean']:.3f}  CI ±{c['ci_half_width']:.4f}"
        )

    with OUT_PATH.open("w") as fh:
        json.dump(out, fh, indent=2)
        fh.write("\n")
    print(f"\nemitted {OUT_PATH.relative_to(CRATE_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
