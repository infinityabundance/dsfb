#!/usr/bin/env python3
"""Render tuned_summary.csv into paper/tables/baseline_tuned.tex.

Input CSV columns:
  baseline, best_config, f1_train, f1_test_mean, f1_test_ci95_lo,
  f1_test_ci95_hi, f1_test_n

Emits a compact tabularx: four rows (DSFB, ADWIN, BOCPD, PELT) with
frozen config, training-split F1, test-split F1 with 95 % bootstrap
CI, and test-split sample count.

Usage: python3 to_tex.py <csv> <tex>
"""
import csv
import sys
from pathlib import Path

ORDER = ["dsfb-database", "adwin", "bocpd", "pelt"]
LABEL = {
    "dsfb-database": r"\textbf{DSFB}",
    "adwin": "ADWIN",
    "bocpd": "BOCPD",
    "pelt": "PELT",
}


def fmt_cfg(b, s):
    if b == "dsfb-database":
        return r"\emph{defaults (not tuned)}"
    return r"\texttt{" + s.replace("_", r"\_") + "}"


def main():
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    dst.parent.mkdir(parents=True, exist_ok=True)

    rows = {}
    with src.open() as f:
        for r in csv.DictReader(f):
            rows[r["baseline"]] = r

    lines = []
    lines.append(r"\begin{table}[htbp]")
    lines.append(r"\centering")
    lines.append(r"\scriptsize")
    lines.append(r"\renewcommand{\arraystretch}{1.2}")
    lines.append(r"\begin{tabularx}{\linewidth}{l X c c c}")
    lines.append(r"\toprule")
    lines.append(
        r"\textbf{Detector} & \textbf{Frozen config} & "
        r"\textbf{F$_1$ (train)} & "
        r"\textbf{F$_1$ (test, mean [CI])} & "
        r"\textbf{n (test)} \\"
    )
    lines.append(r"\midrule")
    for b in ORDER:
        r = rows.get(b)
        if r is None:
            continue
        train_f1 = float(r["f1_train"])
        test_m = float(r["f1_test_mean"])
        test_lo = float(r["f1_test_ci95_lo"])
        test_hi = float(r["f1_test_ci95_hi"])
        n = int(r["f1_test_n"])
        lines.append(
            f"{LABEL[b]} & {fmt_cfg(b, r['best_config'])} & "
            f"{train_f1:.3f} & "
            f"{test_m:.3f} ({test_lo:.3f}, {test_hi:.3f}) & "
            f"{n} \\\\"
        )
    lines.append(r"\bottomrule")
    lines.append(r"\end{tabularx}")
    lines.append(
        r"\caption{Held-out baseline tuning. For each published "
        r"baseline, the hyperparameter config with the best macro-"
        r"F$_1$ on the training split (replication \texttt{r01} of "
        r"every fault class) is frozen and evaluated on the test "
        r"split (all other replications). Mean and $95$~\% "
        r"percentile-bootstrap CI ($B = 1000$) reported across test "
        r"tapes. DSFB is evaluated at \texttt{spec/motifs.yaml} "
        r"defaults — deliberately not re-tuned on the real tapes so "
        r"the comparison is \emph{baselines at best training config} "
        r"vs.\ \emph{DSFB as published}. The full grid is in "
        r"\texttt{src/bin/baseline\_tune.rs}; per-tape F$_1$ in "
        r"\texttt{experiments/baseline\_tune/out/per\_fault.csv}. "
        r"Numbers are emitted byte-deterministically from "
        r"\texttt{tuned\_summary.csv} by \texttt{to\_tex.py}.}"
    )
    lines.append(r"\label{tab:baseline-tuned}")
    lines.append(r"\end{table}")
    dst.write_text("\n".join(lines) + "\n")
    print(f"wrote {dst}")


if __name__ == "__main__":
    main()
