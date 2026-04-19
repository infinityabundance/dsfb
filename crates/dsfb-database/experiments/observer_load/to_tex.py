#!/usr/bin/env python3
"""Render the observer self-load delta table + overlaid CDF figure.

Input CSV columns:
  condition, rep, n_tx, p50_us, p95_us, p99_us, p99_9_us

The CDF figure reads per-replication raw transaction logs from
out/{condition}_r{rep}/tx_log.raw so the sample size is the pooled
per-condition distribution.

Usage: to_tex.py <csv> <tex_out> <png_out>
"""
import csv
import random
import sys
from pathlib import Path


def bootstrap_ci(vals, B=1000, alpha=0.05, seed=42):
    if len(vals) < 2:
        v = float(vals[0]) if vals else 0.0
        return (v, v, v)
    rng = random.Random(seed)
    boots = []
    for _ in range(B):
        s = [vals[rng.randrange(len(vals))] for _ in range(len(vals))]
        boots.append(sum(s) / len(s))
    boots.sort()
    lo = boots[int(alpha / 2 * B)]
    hi = boots[int((1 - alpha / 2) * B)]
    return (sum(vals) / len(vals), lo, hi)


def fmt_us(mean, lo, hi):
    return f"{mean:.0f} ({lo:.0f}, {hi:.0f})"


def render_tex(csv_path, tex_path):
    by_cond = {"without_scraper": [], "with_scraper": []}
    with csv_path.open() as f:
        for row in csv.DictReader(f):
            by_cond[row["condition"]].append(row)

    metrics = [("p50_us", "p50 (\\textmu s)"),
               ("p95_us", "p95 (\\textmu s)"),
               ("p99_us", "p99 (\\textmu s)"),
               ("p99_9_us", "p99.9 (\\textmu s)")]

    stats = {}
    for cond in by_cond:
        stats[cond] = {}
        for key, _ in metrics:
            vals = [float(r[key]) for r in by_cond[cond]]
            stats[cond][key] = bootstrap_ci(vals) if vals else (0.0, 0.0, 0.0)

    lines = []
    lines.append(r"\begin{table}[htbp]")
    lines.append(r"\centering")
    lines.append(r"\scriptsize")
    lines.append(r"\renewcommand{\arraystretch}{1.2}")
    lines.append(r"\begin{tabularx}{\linewidth}{l c c c}")
    lines.append(r"\toprule")
    lines.append(
        r"\textbf{Percentile} & \textbf{Without scraper} & "
        r"\textbf{With scraper} & \textbf{Delta (us)} \\"
    )
    lines.append(r"\midrule")
    for key, label in metrics:
        w_mean, w_lo, w_hi = stats["without_scraper"][key]
        s_mean, s_lo, s_hi = stats["with_scraper"][key]
        delta = s_mean - w_mean
        lines.append(
            f"{label} & {fmt_us(w_mean, w_lo, w_hi)} & "
            f"{fmt_us(s_mean, s_lo, s_hi)} & "
            f"{delta:+.0f} \\\\"
        )
    lines.append(r"\bottomrule")
    lines.append(r"\end{tabularx}")
    lines.append(
        r"\caption{Observer self-load: per-transaction pgbench "
        r"latency percentiles with and without the "
        r"\texttt{dsfb-database live} scraper running at "
        r"500~ms poll cadence, across $n = 5$ "
        r"replications of pgbench \texttt{-c 16 -j 4 -T 90} on a "
        r"pinned postgres:17 container. Mean and $95$~\% percentile-"
        r"bootstrap CI ($B = 1000$, $\alpha = 0.05$). The delta "
        r"column reports the incremental latency attributable to the "
        r"scraper at the listed percentile; numbers are emitted "
        r"byte-deterministically from "
        r"\texttt{experiments/observer\_load/out/pgbench\_latency\_deltas.csv} "
        r"by \texttt{to\_tex.py}.}"
    )
    lines.append(r"\label{tab:observer-self-load}")
    lines.append(r"\end{table}")
    tex_path.write_text("\n".join(lines) + "\n")
    print(f"wrote {tex_path}")


def render_cdf(csv_path, png_path):
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except Exception as e:
        print(f"matplotlib unavailable ({e}); skipping CDF figure")
        return

    out_dir = csv_path.parent
    plt.figure(figsize=(6.5, 4.0))
    colors = {"without_scraper": "#2b7bba", "with_scraper": "#d94a38"}
    labels = {"without_scraper": "without scraper",
              "with_scraper": "with scraper (500 ms poll)"}
    plotted = set()
    for sub in sorted(out_dir.iterdir()):
        if not sub.is_dir():
            continue
        name = sub.name
        if "_r" not in name:
            continue
        cond = name.rsplit("_r", 1)[0]
        if cond not in colors:
            continue
        raw = sub / "tx_log.raw"
        if not raw.exists():
            continue
        lats = []
        with raw.open() as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 3:
                    try:
                        lats.append(int(parts[2]))
                    except ValueError:
                        pass
        if not lats:
            continue
        lats.sort()
        ys = [(i + 1) / len(lats) for i in range(len(lats))]
        plt.plot(lats, ys, color=colors[cond],
                 alpha=0.4, linewidth=0.9,
                 label=labels[cond] if cond not in plotted else None)
        plotted.add(cond)
    plt.xscale("log")
    plt.xlim(left=100)
    plt.xlabel("per-transaction latency (us, log scale)")
    plt.ylabel("empirical CDF")
    plt.title("pgbench per-transaction latency: with vs. without scraper")
    plt.grid(True, alpha=0.3, which="both")
    if plotted:
        plt.legend(loc="lower right", frameon=False)
    plt.tight_layout()
    plt.savefig(png_path, dpi=140)
    plt.close()
    print(f"wrote {png_path}")


def main():
    csv_path = Path(sys.argv[1])
    tex_path = Path(sys.argv[2])
    png_path = Path(sys.argv[3])
    tex_path.parent.mkdir(parents=True, exist_ok=True)
    png_path.parent.mkdir(parents=True, exist_ok=True)
    render_tex(csv_path, tex_path)
    render_cdf(csv_path, png_path)


if __name__ == "__main__":
    main()
