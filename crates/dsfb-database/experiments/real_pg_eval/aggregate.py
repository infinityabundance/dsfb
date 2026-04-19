#!/usr/bin/env python3
"""Aggregate per-replication bakeoff CSVs under OUT_DIR into summary.csv.

Supports two output layouts:
  * Legacy single-fault:  OUT_DIR/r{01..N}/bakeoff.csv
  * Multi-fault:          OUT_DIR/<fault>/r{01..N}/bakeoff.csv

Emits OUT_DIR/summary.csv with columns
  detector, motif, fault, n,
  {precision,recall,f1,ttd_median_s,ttd_p95_s,
   false_alarm_per_hour}_{mean,ci95_lo,ci95_hi,stddev}

Percentile-bootstrap 95 % CI with B=1000, seed 42.
"""
import csv
import glob
import os
import random
import statistics
from pathlib import Path

OUT = Path(os.environ.get("OUT_DIR")
           or Path(__file__).resolve().parent / "out")


def bootstrap_ci(vals, B=1000, alpha=0.05, seed=42):
    if len(vals) < 2:
        v = float(vals[0]) if vals else 0.0
        return (v, v, v)
    rng = random.Random(seed)
    vals = list(vals)
    boots = []
    for _ in range(B):
        s = [vals[rng.randrange(len(vals))] for _ in range(len(vals))]
        boots.append(sum(s) / len(s))
    boots.sort()
    lo = boots[int(alpha / 2 * B)]
    hi = boots[int((1 - alpha / 2) * B)]
    return (sum(vals) / len(vals), lo, hi)


def rows_for(prefix, fault):
    out = []
    rep_dirs = sorted(glob.glob(str(prefix / "r*")))
    for rep_dir in rep_dirs:
        p = Path(rep_dir) / "bakeoff.csv"
        if not p.exists():
            continue
        with p.open() as f:
            lines = [l for l in f if not l.startswith("#")]
        for r in csv.DictReader(lines):
            r["fault"] = fault
            out.append(r)
    return out


def discover_fault_dirs():
    faults = []
    for entry in sorted(OUT.iterdir()) if OUT.exists() else []:
        if entry.is_dir() and (entry / "provenance.txt").exists() \
                and any(entry.glob("r*/bakeoff.csv")):
            faults.append(entry.name)
    return faults


def main():
    rows = []
    fault_dirs = discover_fault_dirs()
    if fault_dirs:
        for f in fault_dirs:
            rows.extend(rows_for(OUT / f, f))
    else:
        rows.extend(rows_for(OUT, "drop_constraint"))

    if not rows:
        print("no bakeoff rows found under", OUT)
        return

    METRICS = [
        "precision", "recall", "f1",
        "ttd_median_s", "ttd_p95_s", "false_alarm_per_hour",
    ]

    by_key = {}
    for r in rows:
        k = (r["detector"], r["motif"], r.get("fault", "drop_constraint"))
        by_key.setdefault(k, []).append(r)

    summary = []
    for (det, motif, fault), rs in sorted(by_key.items()):
        row = {"detector": det, "motif": motif,
               "fault": fault, "n": len(rs)}
        for m in METRICS:
            vals = [float(r[m]) for r in rs]
            mean, lo, hi = bootstrap_ci(vals)
            row[f"{m}_mean"] = f"{mean:.6f}"
            row[f"{m}_ci95_lo"] = f"{lo:.6f}"
            row[f"{m}_ci95_hi"] = f"{hi:.6f}"
            row[f"{m}_stddev"] = (
                f"{statistics.pstdev(vals):.6f}"
                if len(vals) > 1 else "0"
            )
        summary.append(row)

    out_path = OUT / "summary.csv"
    cols = ["detector", "motif", "fault", "n"]
    for m in METRICS:
        cols += [f"{m}_mean", f"{m}_ci95_lo",
                 f"{m}_ci95_hi", f"{m}_stddev"]
    with out_path.open("w") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        for row in summary:
            w.writerow(row)
    print(f"wrote {out_path}: {len(summary)} rows")


if __name__ == "__main__":
    main()
