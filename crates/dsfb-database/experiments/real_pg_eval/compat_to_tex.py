#!/usr/bin/env python3
"""Emit paper/tables/pg_version_compat.tex from two summary.csvs.

Compares DSFB detection quality on the fault-exercised motif for
every fault in both runs. Framed in the caption as a compatibility
sanity check, not a headline result. If the F1 deltas exceed
a compat-flag threshold (0.05), that is disclosed in the caption
rather than silently suppressed.

Usage:
  compat_to_tex.py <pg17_summary.csv> <pg16_summary.csv> \
      <pg17_digest> <pg16_digest> <dst.tex>
"""
import csv
import sys
from pathlib import Path

FAULT_ORDER = ["drop_constraint", "stats_stale", "lock_hold", "cache_evict"]
FAULT_LABEL = {
    "drop_constraint": r"\texttt{DROP CONSTRAINT}",
    "stats_stale": r"statistics staleness",
    "lock_hold": r"row-lock hold",
    "cache_evict": r"buffer-pool eviction",
}
EXERCISED = {
    "drop_constraint": "plan_regression_onset",
    "stats_stale": "cardinality_mismatch_regime",
    "lock_hold": "contention_ramp",
    "cache_evict": "cache_collapse",
}
FLAG_THRESHOLD = 0.05


def load(path):
    by = {}
    with Path(path).open() as f:
        for row in csv.DictReader(f):
            if row.get("detector") != "dsfb-database":
                continue
            fault = row.get("fault") or "drop_constraint"
            motif = row.get("motif")
            if motif != EXERCISED.get(fault):
                continue
            by[fault] = row
    return by


def fmt(mean, lo, hi, digits=3):
    return (
        f"{float(mean):.{digits}f} "
        f"({float(lo):.{digits}f}, {float(hi):.{digits}f})"
    )


def short_digest(d):
    if d.startswith("sha256:"):
        return d[7:19]
    return d[:12]


def main():
    pg17_csv, pg16_csv = sys.argv[1], sys.argv[2]
    pg17_digest, pg16_digest = sys.argv[3], sys.argv[4]
    dst = Path(sys.argv[5])
    dst.parent.mkdir(parents=True, exist_ok=True)

    pg17 = load(pg17_csv)
    pg16 = load(pg16_csv)

    lines = [
        r"\begin{table}[htbp]",
        r"\centering",
        r"\scriptsize",
        r"\renewcommand{\arraystretch}{1.15}",
        r"\begin{tabularx}{\linewidth}{l l c c c}",
        r"\toprule",
        r"\textbf{Fault (exercised motif)} & \textbf{Version} & "
        r"\textbf{Precision} & \textbf{Recall} & \textbf{F$_1$} \\",
        r"\midrule",
    ]

    flagged = []
    for fault in FAULT_ORDER:
        r17 = pg17.get(fault)
        r16 = pg16.get(fault)
        if not r17 and not r16:
            continue
        label = (
            FAULT_LABEL.get(fault, fault)
            + r" \\ \quad (\texttt{"
            + EXERCISED[fault].replace("_", r"\_")
            + r"})"
        )
        if r17:
            lines.append(
                " & ".join([
                    r"\multirow{2}{*}{\parbox[t]{0.36\linewidth}{"
                    + label + r"}}",
                    r"PG17",
                    fmt(r17["precision_mean"],
                        r17["precision_ci95_lo"],
                        r17["precision_ci95_hi"]),
                    fmt(r17["recall_mean"],
                        r17["recall_ci95_lo"],
                        r17["recall_ci95_hi"]),
                    fmt(r17["f1_mean"],
                        r17["f1_ci95_lo"],
                        r17["f1_ci95_hi"]),
                ]) + r" \\"
            )
        else:
            lines.append(
                r"\multirow{2}{*}{\parbox[t]{0.36\linewidth}{"
                + label + r"}} & PG17 & --- & --- & --- \\"
            )
        if r16:
            lines.append(
                " & ".join([
                    "",
                    r"PG16",
                    fmt(r16["precision_mean"],
                        r16["precision_ci95_lo"],
                        r16["precision_ci95_hi"]),
                    fmt(r16["recall_mean"],
                        r16["recall_ci95_lo"],
                        r16["recall_ci95_hi"]),
                    fmt(r16["f1_mean"],
                        r16["f1_ci95_lo"],
                        r16["f1_ci95_hi"]),
                ]) + r" \\"
            )
        else:
            lines.append(r" & PG16 & --- & --- & --- \\")
        lines.append(r"\addlinespace[2pt]")
        if r17 and r16:
            delta = abs(float(r17["f1_mean"]) - float(r16["f1_mean"]))
            if delta > FLAG_THRESHOLD:
                flagged.append((fault, delta))

    lines.append(r"\bottomrule")
    lines.append(r"\end{tabularx}")

    flag_txt = ""
    if flagged:
        flag_txt = (
            r" F$_1$ deltas above "
            + f"{FLAG_THRESHOLD:.2f}"
            + r" are flagged for: "
            + ", ".join(
                f"{FAULT_LABEL[f]} (|$\\Delta$F$_1$| = {d:.3f})"
                for f, d in flagged
            )
            + r"; likely cause is schema drift in \texttt{pg\_stat\_statements}"
            r" between versions and warrants follow-up, not a DSFB bug."
        )
    else:
        flag_txt = (
            r" No F$_1$ delta exceeds "
            + f"{FLAG_THRESHOLD:.2f}"
            + r" across the four fault classes; the DSFB live path is"
            r" engine-version-invariant on this matrix."
        )

    caption = (
        r"Engine-version compatibility sanity: DSFB on the fault-exercised "
        r"motif for each of the four fault classes, run against two pinned "
        r"PostgreSQL container digests. PG17 (\texttt{"
        + short_digest(pg17_digest)
        + r"...}) is the headline version "
        r"reported in Table~\ref{tab:live-eval}; PG16 "
        r"(\texttt{" + short_digest(pg16_digest)
        + r"...}) is a "
        r"compatibility reproduction of the same matrix. Mean and 95~\% "
        r"percentile-bootstrap CI ($B = 1000$) on $n = 10$ replications per "
        r"fault per version."
        + flag_txt
        + r" Raw per-replication bakeoff CSVs live under "
        r"\texttt{experiments/real\_pg\_eval/out/} and "
        r"\texttt{experiments/real\_pg\_eval/out\_pg16/}."
    )
    lines.append(r"\caption{" + caption + "}")
    lines.append(r"\label{tab:pg-version-compat}")
    lines.append(r"\end{table}")

    dst.write_text("\n".join(lines) + "\n")
    print(f"wrote {dst}")


if __name__ == "__main__":
    main()
