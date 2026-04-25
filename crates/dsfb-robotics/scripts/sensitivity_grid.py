#!/usr/bin/env python3
"""Parameter-sensitivity grid for the DSFB FSM.

Sweeps the four canonical FSM parameters across a grid and records the
grammar census at each cell:

  W       ∈ {4, 6, 8, 12, 16}
  K       ∈ {2, 3, 4, 6}
  beta    ∈ {0.3, 0.4, 0.5, 0.6, 0.7}
  delta_s ∈ {0.02, 0.05, 0.10}

For each cell the parametric Python FSM (validated against the Rust
binary on canonical params W=8 K=4 β=0.5 δ_s=0.05) is run on the target
dataset's residual stream, and the census {Admissible, Boundary,
Violation, compression_ratio, peak_norm_sq} is stored.

Output: `audit/sensitivity/<slug>_grid.csv` — one row per (W,K,β,δ_s)
cell with the resulting census, plus an `audit/sensitivity/<slug>_summary.json`
summarising the elasticity of compression_ratio to each parameter.

Default target dataset: panda_gaz (the published-θ̂ exemplar).
"""

from __future__ import annotations

import csv
import json
import sys
from itertools import product
from pathlib import Path

import numpy as np

CRATE_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(CRATE_ROOT / "scripts"))

from dsfb_fsm_parametric import FsmParams, load_residual_stream, run_fsm  # noqa: E402

PROCESSED_ROOT = CRATE_ROOT / "data" / "processed"
OUT_ROOT = CRATE_ROOT / "audit" / "sensitivity"

W_GRID = [4, 6, 8, 12, 16]
K_GRID = [2, 3, 4, 6]
BETA_GRID = [0.3, 0.4, 0.5, 0.6, 0.7]
DELTA_S_GRID = [0.02, 0.05, 0.10]

CANONICAL = (8, 4, 0.5, 0.05)


def grid_for(slug: str) -> list[dict]:
    pub = PROCESSED_ROOT / f"{slug}_published.csv"
    base = PROCESSED_ROOT / f"{slug}.csv"
    target = pub if pub.is_file() else base
    if not target.is_file():
        raise FileNotFoundError(f"{slug}: residual CSV missing")
    stream = load_residual_stream(str(target))
    rows: list[dict] = []
    for W, K, beta, delta_s in product(W_GRID, K_GRID, BETA_GRID, DELTA_S_GRID):
        params = FsmParams(W=W, K=K, boundary_frac=beta, delta_s=delta_s)
        c = run_fsm(stream, params)
        rows.append({
            "W": W, "K": K, "beta": beta, "delta_s": delta_s,
            "is_canonical": (W, K, beta, delta_s) == CANONICAL,
            "admissible": c["admissible"],
            "boundary": c["boundary"],
            "violation": c["violation"],
            "compression_ratio": c["compression_ratio"],
            "max_residual_norm_sq": c["max_residual_norm_sq"],
        })
    return rows


def elasticity(rows: list[dict]) -> dict:
    """For each parameter, vary it while holding others at canonical;
    report the (max - min) compression spread across that 1-D slice."""
    canonical_row = next(r for r in rows if r["is_canonical"])
    out: dict[str, dict] = {}
    for param, grid in [
        ("W", W_GRID), ("K", K_GRID),
        ("beta", BETA_GRID), ("delta_s", DELTA_S_GRID),
    ]:
        slice_rows = [
            r for r in rows
            if all(r[p] == canonical_row[p] for p in ("W", "K", "beta", "delta_s") if p != param)
        ]
        compressions = [r["compression_ratio"] for r in slice_rows]
        out[param] = {
            "grid": grid,
            "compression_spread": float(max(compressions) - min(compressions)),
            "compression_min": float(min(compressions)),
            "compression_max": float(max(compressions)),
            "min_at": next(r[param] for r in slice_rows
                            if r["compression_ratio"] == min(compressions)),
            "max_at": next(r[param] for r in slice_rows
                            if r["compression_ratio"] == max(compressions)),
        }
    out["canonical"] = {
        "W": canonical_row["W"], "K": canonical_row["K"],
        "beta": canonical_row["beta"], "delta_s": canonical_row["delta_s"],
        "compression_ratio": canonical_row["compression_ratio"],
    }
    return out


def main() -> int:
    targets = sys.argv[1:] or ["panda_gaz"]
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    failures = []
    for slug in targets:
        print(f"== {slug} ({len(W_GRID)*len(K_GRID)*len(BETA_GRID)*len(DELTA_S_GRID)} cells) ==",
              flush=True)
        try:
            rows = grid_for(slug)
        except Exception as exc:
            print(f"FAIL {slug}: {exc}")
            failures.append(slug)
            continue
        csv_path = OUT_ROOT / f"{slug}_grid.csv"
        with csv_path.open("w", newline="") as fh:
            w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
            w.writeheader()
            w.writerows(rows)
        summary = elasticity(rows)
        with (OUT_ROOT / f"{slug}_summary.json").open("w") as fh:
            json.dump(summary, fh, indent=2)
            fh.write("\n")
        print(f"  emitted {csv_path.name}; canonical compression "
              f"{summary['canonical']['compression_ratio']:.3f}; "
              f"W spread {summary['W']['compression_spread']:.3f}, "
              f"K spread {summary['K']['compression_spread']:.3f}, "
              f"beta spread {summary['beta']['compression_spread']:.3f}, "
              f"delta_s spread {summary['delta_s']['compression_spread']:.3f}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
