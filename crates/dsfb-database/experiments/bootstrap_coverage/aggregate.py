#!/usr/bin/env python3
"""Render the bootstrap-coverage CSV as a paper figure.

Input: coverage.csv (columns: distribution, n, n_mc, B, alpha_nominal,
       true_mean, empirical_coverage, mean_ci_lo, mean_ci_hi)
Output: PNG at the path supplied as argv[2].

The figure plots empirical coverage vs sample size for each of the
three source distributions, with a horizontal dashed line at the
nominal 0.95 level. The §39 paragraph in paper/dsfb-database.tex cites
the n=10 row directly; this figure is the visual companion.

Determinism: the only randomness in the pipeline lives inside
bootstrap_coverage.rs, which is seeded; the renderer is a pure function
of the CSV.
"""

import csv
import sys
from collections import defaultdict
from pathlib import Path

try:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
except ImportError:
    print("matplotlib not installed; install with: pip install matplotlib", file=sys.stderr)
    sys.exit(0)


def main():
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} <coverage.csv> <out.png>", file=sys.stderr)
        sys.exit(2)
    in_path, out_path = sys.argv[1], sys.argv[2]
    Path(out_path).parent.mkdir(parents=True, exist_ok=True)

    by_dist = defaultdict(list)
    nominal = 0.95
    with open(in_path, newline="") as f:
        for row in csv.DictReader(f):
            by_dist[row["distribution"]].append(
                (int(row["n"]), float(row["empirical_coverage"]))
            )
            nominal = 1.0 - float(row["alpha_nominal"])

    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    markers = {
        "beta_like_f1": "o",
        "gamma_like_ttd": "s",
        "lognormal_like_far": "^",
    }
    labels = {
        "beta_like_f1": "Beta(8,2) — F1-like (bounded)",
        "gamma_like_ttd": "Gamma(2, 0.3) — TTD-like (right-skew)",
        "lognormal_like_far": "log-Normal(2, 1) — FAR/hr-like (heavy tail)",
    }
    for dist, points in by_dist.items():
        points.sort()
        xs = [n for n, _ in points]
        ys = [c for _, c in points]
        ax.plot(
            xs, ys,
            marker=markers.get(dist, "."),
            label=labels.get(dist, dist),
        )

    ax.axhline(nominal, linestyle="--", color="grey",
               label=f"nominal level = {nominal:.2f}")
    ax.set_xscale("log")
    ax.set_xlabel("sample size n (log scale)")
    ax.set_ylabel("empirical coverage of 95% percentile-bootstrap CI")
    ax.set_ylim(0.6, 1.0)
    ax.set_title(
        "Percentile-bootstrap CI coverage at small n\n"
        "(Monte Carlo, 2000 iterations per point)"
    )
    ax.legend(loc="lower right", fontsize=8)
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(out_path, dpi=160)
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
