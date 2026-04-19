#!/usr/bin/env python3
"""Render public_trace_far.csv into paper/tables/public_trace_far.tex.

Input CSV columns:
  detector, dataset, n_seeds, far_per_hour_mean,
  far_per_hour_ci95_lo, far_per_hour_ci95_hi

Emits a detector x dataset matrix of FAR/hr with 95 % bootstrap CIs.
Rows: {DSFB, ADWIN, BOCPD, PELT}. Columns: {Snowset, SQLShare, CEB,
JOB, TPC-DS}.

Usage: python3 to_tex.py <csv> <tex>
"""
import csv
import sys
from pathlib import Path

DETECTOR_ORDER = ["dsfb-database", "adwin", "bocpd", "pelt"]
DETECTOR_LABEL = {
    "dsfb-database": r"\textbf{DSFB}",
    "adwin": "ADWIN",
    "bocpd": "BOCPD",
    "pelt": "PELT",
}
DATASET_ORDER = ["snowset", "sqlshare", "ceb", "job", "tpcds"]
DATASET_LABEL = {
    "snowset": "Snowset",
    "sqlshare": "SQLShare",
    "ceb": "CEB",
    "job": "JOB",
    "tpcds": "TPC-DS",
}


def fmt_cell(m, lo, hi):
    return f"{m:.0f} ({lo:.0f}, {hi:.0f})"


def main():
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    dst.parent.mkdir(parents=True, exist_ok=True)

    grid = {}
    n_seeds = None
    with src.open() as f:
        for r in csv.DictReader(f):
            key = (r["detector"], r["dataset"])
            grid[key] = (
                float(r["far_per_hour_mean"]),
                float(r["far_per_hour_ci95_lo"]),
                float(r["far_per_hour_ci95_hi"]),
            )
            n_seeds = int(r["n_seeds"])

    col_spec = "l " + " ".join(["c"] * len(DATASET_ORDER))
    lines = []
    lines.append(r"\begin{table}[htbp]")
    lines.append(r"\centering")
    lines.append(r"\scriptsize")
    lines.append(r"\renewcommand{\arraystretch}{1.2}")
    lines.append(r"\begin{tabularx}{\linewidth}{" + col_spec + r"}")
    lines.append(r"\toprule")
    header = r"\textbf{Detector} & " + " & ".join(
        r"\textbf{" + DATASET_LABEL[d] + "}" for d in DATASET_ORDER
    ) + r" \\"
    lines.append(header)
    lines.append(r"\midrule")
    for det in DETECTOR_ORDER:
        cells = [DETECTOR_LABEL[det]]
        for ds in DATASET_ORDER:
            trip = grid.get((det, ds))
            cells.append(fmt_cell(*trip) if trip else "---")
        lines.append(" & ".join(cells) + r" \\")
    lines.append(r"\bottomrule")
    lines.append(r"\end{tabularx}")
    n_label = n_seeds if n_seeds is not None else "n"
    lines.append(
        r"\caption{Public-trace false-alarm-per-hour bake-off. "
        r"FAR/hr with $95$~\% percentile-bootstrap CI ($B = 1000$, "
        rf"$\alpha = 0.05$) across $n = {n_label}$ exemplar seeds per "
        r"(detector, corpus) pair. These corpora are published "
        r"workload traces without fault annotations, so every emitted "
        r"episode is counted as a false alarm by construction — the "
        r"numbers are a \emph{workload-stress upper bound} on each "
        r"detector's false-alarm rate, not a detection-quality claim. "
        r"Detection quality is reported in "
        r"Table~\ref{tab:live-eval} and "
        r"Table~\ref{tab:baseline-tuned} on the planted-fault "
        r"protocol. Numbers are emitted byte-deterministically from "
        r"\texttt{public\_trace\_far.csv} by \texttt{to\_tex.py}; the "
        r"per-seed raw file is "
        r"\texttt{public\_trace\_far\_per\_seed.csv}.}"
    )
    lines.append(r"\label{tab:public-trace-far}")
    lines.append(r"\end{table}")
    dst.write_text("\n".join(lines) + "\n")
    print(f"wrote {dst}")


if __name__ == "__main__":
    main()
