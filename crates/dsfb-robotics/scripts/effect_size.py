#!/usr/bin/env python3
"""Compute Cohen's d for slate-wide bimodality.

Reads the per-dataset paper-lock JSON outputs at
``audit/json_outputs/<slug>.json``, partitions the slate by Violation
rate ``V/N`` (zero-V cluster vs non-zero-V cluster), and reports
Cohen's d on the V-rate axis (the load-bearing bimodality test for
the silent-augment vs structure-rich split). Also reports the
all-slate compression-ratio descriptive statistics referenced from
paper §10.X (Effect Size: Bimodality of the Slate).

Outputs ``audit/effect_size/cluster_assignments.csv`` for inspection.

Run from the dsfb-robotics crate root:

    python3 scripts/effect_size.py

"""
from __future__ import annotations

import csv
import glob
import json
import math
import os
import sys
from pathlib import Path


def crate_root() -> Path:
    here = Path(__file__).resolve().parent.parent
    if (here / "audit" / "json_outputs").is_dir():
        return here
    return Path.cwd()


def load_rows(root: Path):
    rows = []
    for path in sorted(glob.glob(str(root / "audit" / "json_outputs" / "*.json"))):
        slug = os.path.basename(path).replace(".json", "")
        with open(path) as f:
            d = json.load(f)
        agg = d["aggregate"]
        a = int(agg["admissible"])
        b = int(agg["boundary"])
        v = int(agg["violation"])
        n = a + b + v
        comp = (b + v) / n if n else 0.0
        vrate = v / n if n else 0.0
        rows.append({"slug": slug, "A": a, "B": b, "V": v, "N": n,
                     "compression": comp, "v_rate": vrate})
    return rows


def stats(xs):
    n = len(xs)
    if n == 0:
        return {"n": 0, "mean": 0.0, "std": 0.0, "min": 0.0, "max": 0.0}
    m = sum(xs) / n
    s = math.sqrt(sum((x - m) ** 2 for x in xs) / (n - 1)) if n > 1 else 0.0
    return {"n": n, "mean": m, "std": s, "min": min(xs), "max": max(xs)}


def cohens_d(xs0, xs1):
    """Pooled-SD Cohen's d for two independent samples."""
    n0, n1 = len(xs0), len(xs1)
    if n0 < 2 or n1 < 2:
        return float("nan")
    m0 = sum(xs0) / n0
    m1 = sum(xs1) / n1
    s0 = math.sqrt(sum((x - m0) ** 2 for x in xs0) / (n0 - 1))
    s1 = math.sqrt(sum((x - m1) ** 2 for x in xs1) / (n1 - 1))
    sp = math.sqrt(((n0 - 1) * s0 ** 2 + (n1 - 1) * s1 ** 2) / (n0 + n1 - 2))
    if sp == 0.0:
        return float("inf") if m1 != m0 else 0.0
    return (m1 - m0) / sp


def main() -> int:
    root = crate_root()
    rows = load_rows(root)
    if not rows:
        print("error: no audit/json_outputs/*.json found; run paper-lock first.",
              file=sys.stderr)
        return 1

    zero_v = [r for r in rows if r["V"] == 0]
    nonzero_v = [r for r in rows if r["V"] > 0]

    s0 = stats([r["v_rate"] for r in zero_v])
    s1 = stats([r["v_rate"] for r in nonzero_v])
    d_v = cohens_d([r["v_rate"] for r in zero_v],
                   [r["v_rate"] for r in nonzero_v])
    s_comp = stats([r["compression"] for r in rows])
    cv = s_comp["std"] / s_comp["mean"] if s_comp["mean"] else 0.0

    print(f"V-rate zero cluster   : n={s0['n']} mean={s0['mean']:.4f} std={s0['std']:.4f}")
    print(f"V-rate nonzero cluster: n={s1['n']} mean={s1['mean']:.4f} std={s1['std']:.4f} "
          f"range=[{s1['min']:.5f},{s1['max']:.4f}]")
    print(f"Cohen's d (V-rate, nonzero - zero): {d_v:.3f}")
    print()
    print(f"All-slate compression: n={s_comp['n']} mean={s_comp['mean']:.3f} "
          f"std={s_comp['std']:.3f} range=[{s_comp['min']:.3f},{s_comp['max']:.3f}] "
          f"CV={cv:.3f}")

    out_dir = root / "audit" / "effect_size"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_csv = out_dir / "cluster_assignments.csv"
    with open(out_csv, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["slug", "A", "B", "V", "N", "compression", "v_rate", "cluster"])
        for r in rows:
            cluster = "zero_v" if r["V"] == 0 else "nonzero_v"
            w.writerow([r["slug"], r["A"], r["B"], r["V"], r["N"],
                        f"{r['compression']:.6f}", f"{r['v_rate']:.6f}", cluster])
    print(f"\nWrote {out_csv}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
