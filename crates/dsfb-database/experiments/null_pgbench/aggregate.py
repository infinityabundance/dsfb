#!/usr/bin/env python3
"""Aggregate per-replication no-fault bake-off CSVs.

For each detector, sums episode counts across all motifs (every
emission is a false alarm under the empty-windows ground truth) and
divides by tape duration. Reports per-detector mean / min / max /
sample-stddev FAR/hr across the N replications.

Output columns:
  detector, n_reps, fp_total_mean, far_per_hour_mean,
  far_per_hour_min, far_per_hour_max, far_per_hour_stddev,
  tape_duration_s_mean

The §44 (adversarial workload) paragraph in paper/dsfb-database.tex
cites the per-detector FAR/hr as the no-fault floor against which
the fault-class FAR/hr in Table 2 is interpreted.
"""

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--in", dest="in_dir", required=True,
                   help="Path to experiments/null_pgbench/out/")
    p.add_argument("--out", required=True,
                   help="Output summary_far.csv path.")
    return p.parse_args()


def tape_duration_s(tape_path: Path) -> float:
    if not tape_path.exists():
        return 0.0
    last_t = 0.0
    first_t = None
    with tape_path.open() as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                s = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = float(s.get("t", 0.0))
            if first_t is None:
                first_t = t
            last_t = t
    if first_t is None:
        return 0.0
    return max(0.0, last_t - first_t)


def main():
    a = parse_args()
    in_dir = Path(a.in_dir)
    per_det_fp = defaultdict(list)
    per_det_far = defaultdict(list)
    per_det_dur = defaultdict(list)

    for rep_dir in sorted(in_dir.glob("r*")):
        bakeoff = rep_dir / "bakeoff.csv"
        tape = rep_dir / "live.tape.jsonl"
        if not bakeoff.exists():
            continue
        dur = tape_duration_s(tape)
        if dur <= 0.0:
            continue
        with bakeoff.open() as f:
            lines = [l for l in f if not l.startswith("#")]
        det_fp = defaultdict(int)
        for r in csv.DictReader(lines):
            det_fp[r["detector"]] += int(r.get("fp", 0))
        for det, fp in det_fp.items():
            per_det_fp[det].append(fp)
            per_det_far[det].append((fp / dur) * 3600.0)
            per_det_dur[det].append(dur)

    out_path = Path(a.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    cols = ["detector", "n_reps", "fp_total_mean", "far_per_hour_mean",
            "far_per_hour_min", "far_per_hour_max",
            "far_per_hour_stddev", "tape_duration_s_mean"]
    with out_path.open("w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        for det in sorted(per_det_far):
            fars = per_det_far[det]
            durs = per_det_dur[det]
            w.writerow({
                "detector": det,
                "n_reps": len(fars),
                "fp_total_mean": f"{sum(per_det_fp[det]) / len(fars):.3f}",
                "far_per_hour_mean": f"{sum(fars) / len(fars):.1f}",
                "far_per_hour_min": f"{min(fars):.1f}",
                "far_per_hour_max": f"{max(fars):.1f}",
                "far_per_hour_stddev": (
                    f"{statistics.pstdev(fars):.1f}" if len(fars) > 1 else "0.0"
                ),
                "tape_duration_s_mean": f"{sum(durs) / len(durs):.1f}",
            })
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
