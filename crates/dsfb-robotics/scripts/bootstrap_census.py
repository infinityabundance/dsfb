#!/usr/bin/env python3
"""Stationary block bootstrap on the per-dataset DSFB grammar census.

Drives the Rust `paper-lock` binary directly via subprocess (with
`--csv-path` pointing at a per-replicate temp CSV) so the bootstrap
census is bit-exact against the canonical engine — no Python FSM
divergence at boundary decisions.

For each dataset:
  1. Load the residual stream (preferring `<slug>_published.csv` over
     `<slug>.csv`, mirroring paper-lock's preference order).
  2. Run paper-lock once on the original stream → point estimate.
  3. Apply stationary block bootstrap (Politis-Romano 1994) with
     mean block length L = W = 8 to draw N_REPLICATES resamples.
  4. For each resample: write to a temp CSV, run paper-lock on it,
     parse JSON, record census.
  5. Compute mean and 95% percentile-bootstrap interval per quantity.
  6. Emit `audit/bootstrap/<slug>_ci.json`.

Parallelised across CPU cores via multiprocessing.Pool to keep total
runtime under ~10 minutes on a workstation.
"""

from __future__ import annotations

import json
import multiprocessing as mp
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np

CRATE_ROOT = Path(__file__).resolve().parent.parent
PROCESSED_ROOT = CRATE_ROOT / "data" / "processed"
OUT_ROOT = CRATE_ROOT / "audit" / "bootstrap"

PAPER_LOCK_BIN = CRATE_ROOT / "target" / "release" / "paper-lock"

ALL_SLUGS = [
    "cwru", "ims", "kuka_lwr", "femto_st", "panda_gaz", "dlr_justin",
    "ur10_kufieta", "cheetah3", "icub_pushrecovery", "droid", "openx",
    "anymal_parkour", "unitree_g1", "aloha_static", "icub3_sorrentino",
    "mobile_aloha", "so100", "aloha_static_tape", "aloha_static_screw_driver",
    "aloha_static_pingpong_test",
]

N_REPLICATES = 1000
BLOCK_LENGTH = 8  # = W (drift window)
RNG_SEED = 20260425


def load_residual_stream(slug: str) -> tuple[np.ndarray, str]:
    """Match paper-lock's preference: <slug>_published.csv > <slug>.csv."""
    pub = PROCESSED_ROOT / f"{slug}_published.csv"
    base = PROCESSED_ROOT / f"{slug}.csv"
    if pub.is_file():
        target, source = pub, "published-theta"
    elif base.is_file():
        target, source = base, "early-window-nominal"
    else:
        raise FileNotFoundError(f"no residual CSV for {slug}: {pub} / {base}")
    rows: list[float] = []
    with target.open() as fh:
        first = fh.readline().strip()
        try:
            rows.append(float(first.split(",")[-1]))
        except ValueError:
            pass  # header
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                rows.append(float(line.split(",")[-1]))
            except ValueError:
                continue
    return np.asarray(rows, dtype=np.float64), source


def stationary_block_bootstrap(
    stream: np.ndarray,
    rng: np.random.Generator,
    block_length: int,
) -> np.ndarray:
    """Politis-Romano 1994 stationary bootstrap. Mean block length L."""
    n = len(stream)
    if n == 0:
        return stream.copy()
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
    rep = json.loads(out.stdout)
    return rep["aggregate"]


def _worker(args: tuple[str, np.ndarray, int]) -> dict:
    slug, stream, seed = args
    rng = np.random.default_rng(seed)
    rs = stationary_block_bootstrap(stream, rng, BLOCK_LENGTH)
    fd, tmp_path_str = tempfile.mkstemp(prefix=f"{slug}_resample_", suffix=".csv")
    os.close(fd)
    tmp_path = Path(tmp_path_str)
    try:
        write_csv(rs, tmp_path)
        return census_via_paper_lock(slug, tmp_path)
    finally:
        try:
            tmp_path.unlink()
        except OSError:
            pass


def bootstrap_one(slug: str, n_workers: int) -> dict:
    stream, source = load_residual_stream(slug)
    if len(stream) < 16:
        raise ValueError(f"{slug}: residual stream too short ({len(stream)})")

    # Point estimate via the actual production binary on the original stream.
    point_path = CRATE_ROOT / f"/tmp/{slug}_point.csv".lstrip("/")
    fd, tmp_path_str = tempfile.mkstemp(prefix=f"{slug}_point_", suffix=".csv")
    os.close(fd)
    point_path = Path(tmp_path_str)
    try:
        write_csv(stream, point_path)
        point = census_via_paper_lock(slug, point_path)
    finally:
        try:
            point_path.unlink()
        except OSError:
            pass

    seeds = [(slug, stream, RNG_SEED ^ (hash(slug) % (2**32)) ^ i)
             for i in range(N_REPLICATES)]
    samples = {k: [] for k in ("admissible", "boundary", "violation",
                                "compression_ratio", "max_residual_norm_sq")}

    if n_workers > 1:
        with mp.Pool(n_workers) as pool:
            for c in pool.imap_unordered(_worker, seeds, chunksize=8):
                for key in samples:
                    samples[key].append(c[key])
    else:
        for s in seeds:
            c = _worker(s)
            for key in samples:
                samples[key].append(c[key])

    out = {
        "dataset": slug,
        "residual_source": source,
        "n_replicates": N_REPLICATES,
        "block_length": BLOCK_LENGTH,
        "rng_seed": RNG_SEED,
        "engine": "rust paper-lock binary (subprocess, --csv-path)",
        "point_estimate": point,
        "ci": {},
    }
    for key, vals in samples.items():
        arr = np.asarray(vals, dtype=np.float64)
        out["ci"][key] = {
            "mean": float(np.mean(arr)),
            "stddev": float(np.std(arr, ddof=1)),
            "ci_lo_2_5": float(np.percentile(arr, 2.5)),
            "ci_hi_97_5": float(np.percentile(arr, 97.5)),
        }
    return out


def main() -> int:
    targets = sys.argv[1:] or ALL_SLUGS
    n_workers = max(1, (os.cpu_count() or 1) - 1)
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    if not PAPER_LOCK_BIN.is_file():
        print(f"ERROR: {PAPER_LOCK_BIN} missing; build via:")
        print(f"  cargo build --manifest-path {CRATE_ROOT}/Cargo.toml --release --features std,paper_lock --bin paper-lock")
        return 2
    failures = []
    for slug in targets:
        print(f"== {slug} ==", flush=True)
        try:
            ci = bootstrap_one(slug, n_workers)
        except Exception as exc:
            print(f"FAIL {slug}: {exc}")
            failures.append(slug)
            continue
        out_path = OUT_ROOT / f"{slug}_ci.json"
        with out_path.open("w") as fh:
            json.dump(ci, fh, indent=2, sort_keys=False)
            fh.write("\n")
        b = ci["ci"]["boundary"]
        v = ci["ci"]["violation"]
        c = ci["ci"]["compression_ratio"]
        print(
            f"  Boundary {b['mean']:>7.1f} [{b['ci_lo_2_5']:.1f},{b['ci_hi_97_5']:.1f}]  "
            f"Violation {v['mean']:>7.1f} [{v['ci_lo_2_5']:.1f},{v['ci_hi_97_5']:.1f}]  "
            f"compression {c['mean']:.3f} [{c['ci_lo_2_5']:.3f},{c['ci_hi_97_5']:.3f}]"
        )
    print(f"\nemitted {len(targets) - len(failures)} bundles to {OUT_ROOT}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
