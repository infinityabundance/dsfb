#!/usr/bin/env python3
"""Render summary.csv into paper/tables/live_eval_mean_ci.tex.

Supports two schemas:

  * Legacy single-fault: rows are keyed by (detector, motif).
    Renders one table for the DROP CONSTRAINT fault with one row
    per detector for the exercised motif plus one collapsed row
    per detector aggregating FAR/hr across the non-exercised
    motifs.

  * Multi-fault: rows are additionally keyed by `fault`. Renders
    one contiguous tabularx grouped by fault; for each fault, the
    exercised motif is identified from the fault class and each
    detector contributes one detailed row + one non-exercised-
    motif FAR/hr summary row.

Fault → exercised motif mapping lives in EXERCISED_MOTIF_BY_FAULT
below. The mapping is one-to-one by design.

Invoked by experiments/real_pg_eval/run.sh aggregation step, or
run manually:

    python3 experiments/real_pg_eval/summary_to_tex.py \
        experiments/real_pg_eval/out/summary.csv \
        paper/tables/live_eval_mean_ci.tex
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
EXERCISED_MOTIF_BY_FAULT = {
    "drop_constraint": "plan_regression_onset",
    "stats_stale": "cardinality_mismatch_regime",
    "lock_hold": "contention_ramp",
    "cache_evict": "cache_collapse",
}
FAULT_ORDER = ["drop_constraint", "stats_stale", "lock_hold", "cache_evict"]
FAULT_LABEL = {
    "drop_constraint": r"\texttt{DROP CONSTRAINT} (plan regression)",
    "stats_stale": r"statistics staleness (cardinality mismatch)",
    "lock_hold": r"row-lock hold (contention ramp)",
    "cache_evict": r"buffer-pool eviction (cache collapse)",
}
MOTIF_LABEL = {
    "plan_regression_onset": r"\texttt{plan\_regression\_onset}",
    "cardinality_mismatch_regime": r"\texttt{cardinality\_mismatch\_regime}",
    "contention_ramp": r"\texttt{contention\_ramp}",
    "cache_collapse": r"\texttt{cache\_collapse}",
    "workload_phase_transition": r"\texttt{workload\_phase\_transition}",
}
NON_EXERCISED_LABEL = r"\emph{non-exercised motifs}"


def fmt(mean, lo, hi, digits=3):
    return f"{float(mean):.{digits}f} ({float(lo):.{digits}f}, {float(hi):.{digits}f})"


def fmt_s(mean, lo, hi):
    return f"{float(mean):.2f} ({float(lo):.2f}, {float(hi):.2f})"


def fmt_far(mean, lo, hi):
    return f"{float(mean):.0f} ({float(lo):.0f}, {float(hi):.0f})"


def fmt_far_summary(mean_of_means, min_of_means, max_of_means):
    return (
        f"{float(mean_of_means):.0f} "
        f"[min {float(min_of_means):.0f}, "
        f"max {float(max_of_means):.0f}]"
    )


def load(src):
    rows = []
    with src.open() as f:
        for row in csv.DictReader(f):
            rows.append(row)
    return rows


def render_fault_block(rows_for_fault, fault_key):
    """Emit rows (list[str]) for one fault: 4 detectors × 2 rows each."""
    exercised = EXERCISED_MOTIF_BY_FAULT[fault_key]
    by_key = {}
    for row in rows_for_fault:
        by_key[(row["detector"], row["motif"])] = row

    out = []
    dash = r"\textit{---}"
    for det in DETECTOR_ORDER:
        r = by_key.get((det, exercised))
        if r is None:
            continue
        detail_cells = [
            DETECTOR_LABEL[det],
            MOTIF_LABEL[exercised],
            fmt(r["precision_mean"], r["precision_ci95_lo"], r["precision_ci95_hi"]),
            fmt(r["recall_mean"], r["recall_ci95_lo"], r["recall_ci95_hi"]),
            fmt(r["f1_mean"], r["f1_ci95_lo"], r["f1_ci95_hi"]),
            fmt_s(
                r["ttd_median_s_mean"],
                r["ttd_median_s_ci95_lo"],
                r["ttd_median_s_ci95_hi"],
            ),
            fmt_s(
                r["ttd_p95_s_mean"],
                r["ttd_p95_s_ci95_lo"],
                r["ttd_p95_s_ci95_hi"],
            ),
            fmt_far(
                r["false_alarm_per_hour_mean"],
                r["false_alarm_per_hour_ci95_lo"],
                r["false_alarm_per_hour_ci95_hi"],
            ),
        ]
        out.append(" & ".join(detail_cells) + r" \\")

        ne_means = []
        for motif in MOTIF_LABEL:
            if motif == exercised:
                continue
            row = by_key.get((det, motif))
            if row is None:
                continue
            ne_means.append(float(row["false_alarm_per_hour_mean"]))
        if ne_means:
            summary_cells = [
                "",
                NON_EXERCISED_LABEL,
                dash, dash, dash, dash, dash,
                fmt_far_summary(
                    sum(ne_means) / len(ne_means),
                    min(ne_means),
                    max(ne_means),
                ),
            ]
            out.append(" & ".join(summary_cells) + r" \\")
    return out


def main():
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    dst.parent.mkdir(parents=True, exist_ok=True)

    rows = load(src)
    has_fault = any("fault" in r and r["fault"] for r in rows)

    lines = []
    lines.append(r"\begin{table}[htbp]")
    lines.append(r"\centering")
    lines.append(r"\scriptsize")
    lines.append(r"\renewcommand{\arraystretch}{1.15}")
    lines.append(r"\begin{tabularx}{\linewidth}{l l c c c c c c}")
    lines.append(r"\toprule")
    lines.append(
        r"\textbf{Detector} & \textbf{Motif} & \textbf{Precision} & "
        r"\textbf{Recall} & \textbf{F$_1$} & "
        r"\textbf{TTD median (s)} & \textbf{TTD p95 (s)} & "
        r"\textbf{FAR / hr} \\"
    )
    lines.append(r"\midrule")

    if not has_fault:
        rendered = render_fault_block(rows, "drop_constraint")
        lines.extend(rendered)
        lines.append(r"\addlinespace")
    else:
        observed = sorted(
            {r["fault"] for r in rows if r.get("fault")},
            key=lambda f: FAULT_ORDER.index(f) if f in FAULT_ORDER else 999,
        )
        for fault in observed:
            rows_for = [r for r in rows if r.get("fault") == fault]
            lines.append(
                r"\multicolumn{8}{l}{\emph{Fault: " +
                FAULT_LABEL.get(fault, fault) + r"}} \\"
            )
            lines.extend(render_fault_block(rows_for, fault))
            lines.append(r"\addlinespace")

    lines.append(r"\bottomrule")
    lines.append(r"\end{tabularx}")

    if has_fault:
        per_fault_n = {}
        for r in rows:
            f = r.get("fault") or "drop_constraint"
            try:
                per_fault_n[f] = max(per_fault_n.get(f, 0), int(r["n"]))
            except (KeyError, ValueError):
                pass
        n_str = ", ".join(
            f"{FAULT_LABEL.get(f, f)}: $n = {per_fault_n[f]}$"
            for f in observed
        )
        deferred = [f for f in FAULT_ORDER
                    if f not in per_fault_n]
        deferred_str = (
            r" Per-fault replication counts: " + n_str + "."
        )
        if deferred:
            deferred_str += (
                r" Replications for "
                + ", ".join(FAULT_LABEL.get(f, f) for f in deferred)
                + r" are deferred to future work "
                + r"(\S\ref{sec:limitations}); the harness in "
                + r"\texttt{experiments/real\_pg\_eval/run.sh} covers "
                + r"all four fault classes and the directory "
                + r"\texttt{experiments/real\_pg\_eval/faults/} "
                + r"contains the pinned fault scripts."
            )
        caption = (
            r"Per-fault, per-detector mean and 95~\% bootstrap CI "
            r"($B = 1000$) on pgbench scale-10 "
            r"(\S\ref{sec:live-eval})."
            + deferred_str +
            r" Each fault class exercises exactly one motif; "
            r"detection-quality columns apply to that motif. The "
            r"\emph{non-exercised motifs} row per (detector, fault) "
            r"aggregates mean / min / max FAR/hr across the four "
            r"motifs the fault does not exercise. Detailed per-"
            r"replication breakdowns are in "
            r"\texttt{experiments/real\_pg\_eval/out/<fault>/r*/bakeoff.csv}. "
            r"Numbers are emitted byte-deterministically from "
            r"\texttt{experiments/real\_pg\_eval/out/summary.csv} by "
            r"\texttt{summary\_to\_tex.py}."
        )
    else:
        caption = (
            r"Per-detector mean and 95~\% bootstrap CI "
            r"($B = 1000$) across $n = 10$ replications of the "
            r"planted \texttt{DROP CONSTRAINT} fault on pgbench "
            r"scale-10 (\S\ref{sec:live-eval}). The exercised motif "
            r"\texttt{plan\_regression\_onset} carries the detection-"
            r"quality columns; the four non-exercised motifs "
            r"(\texttt{cache\_collapse}, "
            r"\texttt{cardinality\_mismatch\_regime}, "
            r"\texttt{contention\_ramp}, "
            r"\texttt{workload\_phase\_transition}) are collapsed "
            r"into one per-detector summary row that reports "
            r"mean / min / max across the four per-motif FAR/hr means; "
            r"detection-quality cells are elided (\textit{---}) on "
            r"that row as they are zero by construction under the "
            r"single-fault protocol. The detailed per-motif-per-"
            r"replication FAR/hr breakdown is in "
            r"\texttt{experiments/real\_pg\_eval/out/r\{01..10\}/bakeoff.csv}. "
            r"Numbers are emitted byte-deterministically from "
            r"\texttt{experiments/real\_pg\_eval/out/summary.csv} by "
            r"\texttt{summary\_to\_tex.py}."
        )
    lines.append(r"\caption{" + caption + "}")
    lines.append(r"\label{tab:live-eval}")
    lines.append(r"\end{table}")

    dst.write_text("\n".join(lines) + "\n")
    print(f"wrote {dst}")


if __name__ == "__main__":
    main()
