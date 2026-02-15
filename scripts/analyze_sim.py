#!/usr/bin/env python3
"""Analyze DSFB drift-impulse simulation CSV and generate verification artifacts."""

from __future__ import annotations

import argparse
import csv
import json
import math
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List

REQUIRED_COLUMNS = [
    "t",
    "phi_true",
    "phi_mean",
    "phi_freqonly",
    "phi_dsfb",
    "err_mean",
    "err_freqonly",
    "err_dsfb",
    "w2",
    "s2",
]

ESTIMATORS = ("mean", "freqonly", "dsfb")
ERR_COL = {
    "mean": "err_mean",
    "freqonly": "err_freqonly",
    "dsfb": "err_dsfb",
}


@dataclass
class RecoveryResult:
    steps: int
    recovered: bool


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Compute metrics and plots for DSFB drift-impulse simulation output"
    )
    parser.add_argument(
        "--csv",
        default="sim-dsfb.csv",
        help="Path to simulation CSV (default: sim-dsfb.csv)",
    )
    parser.add_argument(
        "--outdir",
        default=None,
        help=(
            "Directory for metrics and plots. "
            "If omitted, writes to output-dsfb/analysis/<YYYYMMDD_HHMMSS>."
        ),
    )
    parser.add_argument(
        "--impulse-start", type=int, default=300, help="Impulse start step index"
    )
    parser.add_argument(
        "--impulse-duration", type=int, default=100, help="Impulse duration in steps"
    )
    parser.add_argument(
        "--baseline-factor",
        type=float,
        default=1.10,
        help="Recovery threshold multiplier on pre-impulse baseline RMS",
    )
    parser.add_argument(
        "--baseline-margin",
        type=float,
        default=0.005,
        help="Additive recovery threshold margin",
    )
    parser.add_argument(
        "--hold-steps",
        type=int,
        default=10,
        help="Consecutive steps under threshold required to count as recovered",
    )
    parser.add_argument(
        "--no-plots", action="store_true", help="Compute metrics only (skip PNG plots)"
    )
    parser.add_argument(
        "--show",
        action="store_true",
        help="Display plots interactively (in addition to saving PNG files)",
    )
    parser.add_argument(
        "--tolerance",
        type=float,
        default=1e-3,
        help="Absolute tolerance for optional expected metric checks",
    )

    for metric in ("rms", "peak", "recovery"):
        for estimator in ESTIMATORS:
            parser.add_argument(
                f"--expect-{metric}-{estimator}",
                type=float,
                default=None,
                help=f"Expected {metric} value for {estimator}",
            )

    args = parser.parse_args()

    if args.impulse_start < 0:
        parser.error("--impulse-start must be >= 0")
    if args.impulse_duration <= 0:
        parser.error("--impulse-duration must be > 0")
    if args.hold_steps <= 0:
        parser.error("--hold-steps must be > 0")

    return args


def read_csv(csv_path: str) -> Dict[str, List[float]]:
    if not os.path.exists(csv_path):
        raise FileNotFoundError(f"CSV not found: {csv_path}")

    data: Dict[str, List[float]] = {col: [] for col in REQUIRED_COLUMNS}

    with open(csv_path, "r", newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError("CSV has no header row")

        missing = [col for col in REQUIRED_COLUMNS if col not in reader.fieldnames]
        if missing:
            raise ValueError(
                "CSV is missing required columns: " + ", ".join(missing)
            )

        for line_no, row in enumerate(reader, start=2):
            if row is None:
                continue
            for col in REQUIRED_COLUMNS:
                raw = row.get(col, "")
                try:
                    value = float(raw)
                except (TypeError, ValueError) as exc:
                    raise ValueError(
                        f"Invalid float at line {line_no}, column '{col}': {raw!r}"
                    ) from exc
                data[col].append(value)

    if not data["t"]:
        raise ValueError("CSV contains no data rows")

    return data


def rms(values: List[float]) -> float:
    return math.sqrt(sum(v * v for v in values) / len(values))


def peak_in_window(values: List[float], start: int, duration: int) -> float:
    end = min(start + duration, len(values))
    if start >= len(values):
        raise ValueError(
            f"Impulse start index {start} is beyond available data length {len(values)}"
        )
    return max(values[start:end])


def recovery_steps(
    values: List[float],
    impulse_end: int,
    threshold: float,
    hold_steps: int,
) -> RecoveryResult:
    if impulse_end >= len(values):
        return RecoveryResult(steps=0, recovered=False)

    last_start = len(values) - hold_steps
    for idx in range(impulse_end, max(impulse_end, last_start) + 1):
        window = values[idx : idx + hold_steps]
        if len(window) < hold_steps:
            break
        if all(v <= threshold for v in window):
            return RecoveryResult(steps=idx - impulse_end, recovered=True)

    return RecoveryResult(steps=len(values) - impulse_end, recovered=False)


def compute_metrics(data: Dict[str, List[float]], args: argparse.Namespace) -> Dict[str, Dict[str, float]]:
    impulse_end = args.impulse_start + args.impulse_duration

    metrics: Dict[str, Dict[str, float]] = {
        "rms": {},
        "peak": {},
        "baseline_rms": {},
        "recovery_threshold": {},
        "recovery_steps": {},
        "recovery_recovered": {},
    }

    for estimator in ESTIMATORS:
        err = data[ERR_COL[estimator]]

        metrics["rms"][estimator] = rms(err)
        metrics["peak"][estimator] = peak_in_window(
            err, args.impulse_start, args.impulse_duration
        )

        baseline_segment = err[: args.impulse_start]
        if not baseline_segment:
            raise ValueError(
                "Cannot compute baseline: impulse starts at step 0. "
                "Use --impulse-start > 0."
            )

        baseline = rms(baseline_segment)
        threshold = baseline * args.baseline_factor + args.baseline_margin
        recovery = recovery_steps(err, impulse_end, threshold, args.hold_steps)

        metrics["baseline_rms"][estimator] = baseline
        metrics["recovery_threshold"][estimator] = threshold
        metrics["recovery_steps"][estimator] = float(recovery.steps)
        metrics["recovery_recovered"][estimator] = 1.0 if recovery.recovered else 0.0

    return metrics


def maybe_compare_expected(metrics: Dict[str, Dict[str, float]], args: argparse.Namespace) -> int:
    checks = []
    for metric_name in ("rms", "peak", "recovery_steps"):
        metric_prefix = "recovery" if metric_name == "recovery_steps" else metric_name
        for estimator in ESTIMATORS:
            arg_name = f"expect_{metric_prefix}_{estimator}"
            expected = getattr(args, arg_name)
            if expected is None:
                continue
            actual = metrics[metric_name][estimator]
            ok = abs(actual - expected) <= args.tolerance
            checks.append(
                {
                    "metric": metric_name,
                    "estimator": estimator,
                    "expected": expected,
                    "actual": actual,
                    "delta": actual - expected,
                    "ok": ok,
                }
            )

    if not checks:
        return 0

    print("\nExpected-value checks:")
    all_ok = True
    for check in checks:
        status = "PASS" if check["ok"] else "FAIL"
        all_ok = all_ok and check["ok"]
        print(
            "  "
            f"{status} {check['metric']}.{check['estimator']}: "
            f"actual={check['actual']:.6f}, expected={check['expected']:.6f}, "
            f"delta={check['delta']:+.6f}"
        )

    return 0 if all_ok else 2


def write_metrics_files(metrics: Dict[str, Dict[str, float]], outdir: str) -> None:
    os.makedirs(outdir, exist_ok=True)

    json_path = os.path.join(outdir, "metrics.json")
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(metrics, f, indent=2, sort_keys=True)

    csv_path = os.path.join(outdir, "metrics_summary.csv")
    with open(csv_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        writer.writerow(["metric", "mean", "freqonly", "dsfb"])

        for metric in (
            "rms",
            "peak",
            "baseline_rms",
            "recovery_threshold",
            "recovery_steps",
            "recovery_recovered",
        ):
            writer.writerow(
                [
                    metric,
                    f"{metrics[metric]['mean']:.10f}",
                    f"{metrics[metric]['freqonly']:.10f}",
                    f"{metrics[metric]['dsfb']:.10f}",
                ]
            )


def maybe_generate_plots(
    data: Dict[str, List[float]],
    outdir: str,
    impulse_start: int,
    impulse_duration: int,
    show: bool,
) -> None:
    # Avoid cache-dir permission issues in restricted environments.
    os.environ.setdefault("MPLCONFIGDIR", "/tmp/matplotlib")

    try:
        import matplotlib.pyplot as plt
    except ImportError as exc:
        raise RuntimeError(
            "matplotlib is required for plotting. Install it (for example: pip install matplotlib), "
            "or rerun with --no-plots."
        ) from exc

    t = data["t"]
    impulse_end = min(impulse_start + impulse_duration, len(t))

    os.makedirs(outdir, exist_ok=True)

    def shade_impulse(ax) -> None:
        if impulse_start < len(t):
            x0 = t[impulse_start]
            x1 = t[impulse_end - 1] if impulse_end > impulse_start else x0
            ax.axvspan(x0, x1, alpha=0.12, color="red", label="impulse window")

    fig1, ax1 = plt.subplots(figsize=(12, 5))
    ax1.plot(t, data["phi_true"], label="phi_true", linewidth=2.0, color="black")
    ax1.plot(t, data["phi_mean"], label="phi_mean", linewidth=1.2)
    ax1.plot(t, data["phi_freqonly"], label="phi_freqonly", linewidth=1.2)
    ax1.plot(t, data["phi_dsfb"], label="phi_dsfb", linewidth=1.6)
    shade_impulse(ax1)
    ax1.set_title("True vs Estimated phi")
    ax1.set_xlabel("t")
    ax1.set_ylabel("phi")
    ax1.grid(True, alpha=0.25)
    ax1.legend(loc="best")
    fig1.tight_layout()
    fig1.savefig(os.path.join(outdir, "phi_estimates.png"), dpi=180)

    fig2, ax2 = plt.subplots(figsize=(12, 5))
    ax2.plot(t, data["err_mean"], label="err_mean", linewidth=1.2)
    ax2.plot(t, data["err_freqonly"], label="err_freqonly", linewidth=1.2)
    ax2.plot(t, data["err_dsfb"], label="err_dsfb", linewidth=1.6)
    shade_impulse(ax2)
    ax2.set_title("Estimation Error Over Time")
    ax2.set_xlabel("t")
    ax2.set_ylabel("absolute error")
    ax2.grid(True, alpha=0.25)
    ax2.legend(loc="best")
    fig2.tight_layout()
    fig2.savefig(os.path.join(outdir, "estimation_errors.png"), dpi=180)

    fig3, ax3 = plt.subplots(figsize=(12, 5))
    ax3.plot(t, data["w2"], label="w2 (trust channel 2)", color="tab:blue", linewidth=1.5)
    ax3.set_xlabel("t")
    ax3.set_ylabel("w2", color="tab:blue")
    ax3.tick_params(axis="y", labelcolor="tab:blue")
    ax3.grid(True, alpha=0.25)

    ax3b = ax3.twinx()
    ax3b.plot(t, data["s2"], label="s2 (EMA residual channel 2)", color="tab:orange", linewidth=1.4)
    ax3b.set_ylabel("s2", color="tab:orange")
    ax3b.tick_params(axis="y", labelcolor="tab:orange")

    shade_impulse(ax3)
    lines1, labels1 = ax3.get_legend_handles_labels()
    lines2, labels2 = ax3b.get_legend_handles_labels()
    ax3.legend(lines1 + lines2, labels1 + labels2, loc="best")
    ax3.set_title("Trust Weight and EMA Residual (Channel 2)")
    fig3.tight_layout()
    fig3.savefig(os.path.join(outdir, "trust_weight_and_ema.png"), dpi=180)

    if show:
        plt.show()

    plt.close(fig1)
    plt.close(fig2)
    plt.close(fig3)


def print_summary(metrics: Dict[str, Dict[str, float]], dt: float) -> None:
    print("\nMetrics from sim-dsfb.csv")
    print("====================")

    print("\nRMS error")
    for estimator in ESTIMATORS:
        print(f"  {estimator:8s}: {metrics['rms'][estimator]:.6f}")

    print("\nPeak error during impulse")
    for estimator in ESTIMATORS:
        print(f"  {estimator:8s}: {metrics['peak'][estimator]:.6f}")

    print("\nRecovery time to near-baseline")
    for estimator in ESTIMATORS:
        steps = int(round(metrics["recovery_steps"][estimator]))
        seconds = steps * dt
        threshold = metrics["recovery_threshold"][estimator]
        recovered = metrics["recovery_recovered"][estimator] > 0.5
        suffix = "" if recovered else " (not fully recovered in horizon)"
        print(
            f"  {estimator:8s}: {steps:4d} steps ({seconds:.4f} s), "
            f"threshold={threshold:.6f}{suffix}"
        )


def main() -> int:
    args = parse_args()

    data = read_csv(args.csv)
    dt = data["t"][1] - data["t"][0] if len(data["t"]) > 1 else 1.0

    if args.outdir:
        outdir = args.outdir
    else:
        csv_parent = Path(args.csv).resolve().parent
        if (
            csv_parent.name not in ("output-dsfb", "")
            and csv_parent.parent.name == "output-dsfb"
        ):
            outdir = str(csv_parent / "analysis")
        else:
            stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
            outdir = os.path.join("output-dsfb", "analysis", stamp)

    metrics = compute_metrics(data, args)
    write_metrics_files(metrics, outdir)
    print_summary(metrics, dt)

    if not args.no_plots:
        maybe_generate_plots(
            data,
            outdir,
            args.impulse_start,
            args.impulse_duration,
            args.show,
        )
        print(f"\nPlots written to: {outdir}")
    else:
        print("\nPlot generation skipped (--no-plots).")

    print(f"Metrics files written to: {outdir}")

    return maybe_compare_expected(metrics, args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # pragma: no cover - script-level guard
        print(f"ERROR: {exc}", file=sys.stderr)
        raise
