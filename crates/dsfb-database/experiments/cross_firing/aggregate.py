#!/usr/bin/env python3
"""Build the cross-firing matrix from the per-replication bakeoff CSVs.

The §13 ¶2 prose-only observation in the paper says DSFB over-fires
on `contention_ramp` during the `drop_constraint` cascade. This script
turns that observation into a measured matrix:

  rows    = planted fault (drop_constraint, stats_stale, lock_hold,
            cache_evict)
  cols    = emitted motif (plan_regression_onset, …,
            workload_phase_transition)
  values  = mean false-positives per replication, per (detector,
            planted_fault, emitted_motif)

The "ground-truth" emitted motif for each fault — drop_constraint →
plan_regression_onset, stats_stale → cardinality_mismatch_regime,
lock_hold → contention_ramp, cache_evict → cache_collapse — is
highlighted on the LaTeX table; off-diagonal cells are the
cross-firing.

Output:
  --csv  experiments/cross_firing/out/cross_firing.csv
  --tex  paper/tables/cross_firing.tex
"""

import argparse
import csv
import statistics
from collections import defaultdict
from pathlib import Path


GT_FOR_FAULT = {
    "drop_constraint": "plan_regression_onset",
    "stats_stale": "cardinality_mismatch_regime",
    "lock_hold": "contention_ramp",
    "cache_evict": "cache_collapse",
}

MOTIFS_ORDER = [
    "plan_regression_onset",
    "cardinality_mismatch_regime",
    "contention_ramp",
    "cache_collapse",
    "workload_phase_transition",
]

DETECTORS = ["dsfb-database", "adwin", "bocpd", "pelt"]


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--pg-out", required=True,
                   help="Path to experiments/real_pg_eval/out/")
    p.add_argument("--csv", required=True,
                   help="Output cross-firing CSV.")
    p.add_argument("--tex", required=True,
                   help="Output cross-firing LaTeX table.")
    return p.parse_args()


def collect(pg_out):
    rows = []
    pg_out = Path(pg_out)
    for fault_dir in sorted(pg_out.iterdir()):
        if not fault_dir.is_dir() or fault_dir.name not in GT_FOR_FAULT:
            continue
        fault = fault_dir.name
        for rep_dir in sorted(fault_dir.glob("r*")):
            bakeoff = rep_dir / "bakeoff.csv"
            if not bakeoff.exists():
                continue
            with bakeoff.open() as f:
                lines = [l for l in f if not l.startswith("#")]
            for r in csv.DictReader(lines):
                rows.append({
                    "fault": fault,
                    "rep": rep_dir.name,
                    "detector": r["detector"],
                    "motif": r["motif"],
                    "fp": int(r["fp"]),
                    "tp": int(r["tp"]),
                })
    return rows


def aggregate(rows):
    # Mean (TP if motif is the GT; else FP) per (detector, fault, motif).
    cells = defaultdict(list)
    for r in rows:
        gt_motif = GT_FOR_FAULT[r["fault"]]
        if r["motif"] == gt_motif:
            v = r["tp"]
        else:
            v = r["fp"]
        cells[(r["detector"], r["fault"], r["motif"])].append(v)
    out = []
    for (det, fault, motif), vs in cells.items():
        out.append({
            "detector": det,
            "fault": fault,
            "motif": motif,
            "n_reps": len(vs),
            "mean_count": (sum(vs) / len(vs)) if vs else 0.0,
            "max_count": max(vs) if vs else 0,
            "min_count": min(vs) if vs else 0,
            "stddev": statistics.pstdev(vs) if len(vs) > 1 else 0.0,
            "is_gt": motif == GT_FOR_FAULT[fault],
        })
    return out


def write_csv(out_rows, path):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    cols = ["detector", "fault", "motif", "n_reps", "mean_count",
            "max_count", "min_count", "stddev", "is_gt"]
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        for r in sorted(out_rows, key=lambda r: (r["detector"],
                                                 r["fault"],
                                                 r["motif"])):
            w.writerow({**r,
                        "mean_count": f"{r['mean_count']:.3f}",
                        "stddev": f"{r['stddev']:.3f}",
                        "is_gt": "1" if r["is_gt"] else "0"})


def write_tex(out_rows, path):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    by_det = defaultdict(dict)
    for r in out_rows:
        by_det[r["detector"]][(r["fault"], r["motif"])] = r

    lines = []
    lines.append("% Cross-firing matrix — Pass-2 N1 deliverable.")
    lines.append("% Rows: planted fault. Columns: emitted motif.")
    lines.append("% Bold cells are the ground-truth motif for that fault.")
    lines.append("% Off-diagonal entries quantify cross-firing.")
    lines.append("\\begin{tabular}{l l " + "r " * len(MOTIFS_ORDER) + "}")
    lines.append("\\toprule")
    motif_short = {
        "plan_regression_onset": "PR",
        "cardinality_mismatch_regime": "CM",
        "contention_ramp": "CR",
        "cache_collapse": "CC",
        "workload_phase_transition": "WP",
    }
    header = ["Detector", "Planted fault"] + [motif_short[m] for m in MOTIFS_ORDER]
    lines.append(" & ".join(header) + " \\\\")
    lines.append("\\midrule")
    for det in DETECTORS:
        if det not in by_det:
            continue
        for fault in sorted(GT_FOR_FAULT):
            row_cells = [det.replace("_", "\\_"),
                         fault.replace("_", "\\_")]
            for m in MOTIFS_ORDER:
                key = (fault, m)
                cell = by_det[det].get(key)
                v = cell["mean_count"] if cell else 0.0
                if cell and cell["is_gt"]:
                    row_cells.append(f"\\textbf{{{v:.1f}}}")
                else:
                    row_cells.append(f"{v:.1f}")
            lines.append(" & ".join(row_cells) + " \\\\")
        lines.append("\\midrule")
    lines.append("\\bottomrule")
    lines.append("\\end{tabular}")
    Path(path).write_text("\n".join(lines) + "\n")


def main():
    args = parse_args()
    rows = collect(args.pg_out)
    if not rows:
        print(f"no bakeoff CSVs found under {args.pg_out}", flush=True)
        return
    out_rows = aggregate(rows)
    write_csv(out_rows, args.csv)
    write_tex(out_rows, args.tex)
    print(f"wrote {args.csv} ({len(out_rows)} cells)")
    print(f"wrote {args.tex}")


if __name__ == "__main__":
    main()
