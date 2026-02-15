#!/usr/bin/env python3
"""Plot DSFB simulation outputs from sim-dsfb.csv and print summary statistics."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
from typing import Dict, Optional

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Read sim-dsfb.csv, plot DSFB metrics, and print summary statistics."
    )
    parser.add_argument(
        "--csv",
        default=None,
        help="Path to sim-dsfb.csv (default: auto-detect latest output-dsfb run)",
    )
    parser.add_argument(
        "--impulse-start",
        type=float,
        default=3.0,
        help="Impulse window start time for peak error metric",
    )
    parser.add_argument(
        "--impulse-end",
        type=float,
        default=4.0,
        help="Impulse window end time for peak error metric",
    )
    parser.add_argument(
        "--zoom-start",
        type=float,
        default=3.6,
        help="Zoom plot start time",
    )
    parser.add_argument(
        "--zoom-end",
        type=float,
        default=4.0,
        help="Zoom plot end time",
    )
    parser.add_argument(
        "--save-plots",
        action="store_true",
        help="Save plots as PNG files in --outdir",
    )
    parser.add_argument(
        "--outdir",
        default=".",
        help="Output directory for PNG files when --save-plots is set",
    )
    return parser.parse_args()


def load_data(csv_path: str) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    missing = [c for c in REQUIRED_COLUMNS if c not in df.columns]
    if missing:
        raise ValueError(f"CSV missing required columns: {', '.join(missing)}")
    return df


def resolve_csv_path(cli_csv: Optional[str]) -> str:
    if cli_csv:
        return cli_csv

    run_candidates = sorted(Path("output-dsfb").glob("*/sim-dsfb.csv"))
    if run_candidates:
        return str(run_candidates[-1])

    run_candidates = sorted(Path("output-dsfb").glob("*/sim.csv"))
    if run_candidates:
        return str(run_candidates[-1])

    static_candidates = [
        Path("output-dsfb/sim-dsfb.csv"),
        Path("sim-dsfb.csv"),
        Path("output-dsfb/sim.csv"),
        Path("sim.csv"),
        Path("out/sim.csv"),
    ]
    for candidate in static_candidates:
        if candidate.exists():
            return str(candidate)

    raise FileNotFoundError(
        "Could not find DSFB simulation CSV. Provide --csv or generate output-dsfb/<timestamp>/sim-dsfb.csv "
        "with: cargo run --release -p dsfb --example drift_impulse"
    )


def rms(series: pd.Series) -> float:
    values = series.to_numpy(dtype=float)
    return float(np.sqrt(np.mean(values**2)))


def compute_summary(
    df: pd.DataFrame, impulse_start: float, impulse_end: float
) -> Dict[str, Dict[str, float]]:
    impulse_mask = (df["t"] >= impulse_start) & (df["t"] <= impulse_end)
    if not impulse_mask.any():
        raise ValueError(
            "No rows found inside the impulse window. "
            "Adjust --impulse-start/--impulse-end."
        )

    return {
        "mean": {
            "rms": rms(df["err_mean"]),
            "peak_impulse": float(df.loc[impulse_mask, "err_mean"].max()),
        },
        "freqonly": {
            "rms": rms(df["err_freqonly"]),
            "peak_impulse": float(df.loc[impulse_mask, "err_freqonly"].max()),
        },
        "dsfb": {
            "rms": rms(df["err_dsfb"]),
            "peak_impulse": float(df.loc[impulse_mask, "err_dsfb"].max()),
        },
    }


def print_summary(stats: Dict[str, Dict[str, float]], start: float, end: float) -> None:
    print("\nSummary Statistics")
    print("==================")
    print(f"Peak impulse window: t in [{start:.3f}, {end:.3f}]")
    print(f"{'method':<10} {'rms_error':>12} {'peak_impulse_error':>20}")
    for method in ("mean", "freqonly", "dsfb"):
        print(
            f"{method:<10} "
            f"{stats[method]['rms']:>12.6f} "
            f"{stats[method]['peak_impulse']:>20.6f}"
        )


def maybe_save(fig: plt.Figure, path: str, enabled: bool) -> None:
    if enabled:
        fig.savefig(path, dpi=200, bbox_inches="tight")


def plot_all(
    df: pd.DataFrame,
    zoom_start: float,
    zoom_end: float,
    save_plots: bool,
    outdir: str,
) -> None:
    if save_plots:
        os.makedirs(outdir, exist_ok=True)

    # 1) Estimation errors over full time.
    fig1, ax1 = plt.subplots(figsize=(10, 4.8))
    ax1.plot(df["t"], df["err_mean"], label="err_mean", linewidth=1.7)
    ax1.plot(df["t"], df["err_freqonly"], label="err_freqonly", linewidth=1.7)
    ax1.plot(df["t"], df["err_dsfb"], label="err_dsfb", linewidth=1.9)
    ax1.set_title("Estimation Error vs Time")
    ax1.set_xlabel("t")
    ax1.set_ylabel("Absolute Error")
    ax1.grid(True, alpha=0.3)
    ax1.legend()
    maybe_save(fig1, os.path.join(outdir, "estimation_error_vs_time.png"), save_plots)

    # 2) Zoomed estimation error.
    fig2, ax2 = plt.subplots(figsize=(10, 4.8))
    ax2.plot(df["t"], df["err_mean"], label="err_mean", linewidth=1.7)
    ax2.plot(df["t"], df["err_freqonly"], label="err_freqonly", linewidth=1.7)
    ax2.plot(df["t"], df["err_dsfb"], label="err_dsfb", linewidth=1.9)
    ax2.set_xlim(zoom_start, zoom_end)
    ax2.set_title(f"Zoomed Estimation Error (t in [{zoom_start:.2f}, {zoom_end:.2f}])")
    ax2.set_xlabel("t")
    ax2.set_ylabel("Absolute Error")
    ax2.grid(True, alpha=0.3)
    ax2.legend()
    maybe_save(fig2, os.path.join(outdir, "estimation_error_zoom.png"), save_plots)

    # 3) Trust weight w2.
    fig3, ax3 = plt.subplots(figsize=(10, 4.8))
    ax3.plot(df["t"], df["w2"], label="w2", color="tab:blue", linewidth=1.8)
    ax3.set_title("Trust Weight w2 vs Time")
    ax3.set_xlabel("t")
    ax3.set_ylabel("w2")
    ax3.grid(True, alpha=0.3)
    ax3.legend()
    maybe_save(fig3, os.path.join(outdir, "trust_weight_w2.png"), save_plots)

    # 4) EMA residual s2.
    fig4, ax4 = plt.subplots(figsize=(10, 4.8))
    ax4.plot(df["t"], df["s2"], label="s2", color="tab:orange", linewidth=1.8)
    ax4.set_title("EMA Residual s2 vs Time")
    ax4.set_xlabel("t")
    ax4.set_ylabel("s2")
    ax4.grid(True, alpha=0.3)
    ax4.legend()
    maybe_save(fig4, os.path.join(outdir, "ema_residual_s2.png"), save_plots)

    plt.show()


def main() -> int:
    args = parse_args()
    csv_path = resolve_csv_path(args.csv)
    df = load_data(csv_path)

    stats = compute_summary(df, args.impulse_start, args.impulse_end)
    print_summary(stats, args.impulse_start, args.impulse_end)
    print(f"\nUsing CSV: {os.path.abspath(csv_path)}")

    plot_all(df, args.zoom_start, args.zoom_end, args.save_plots, args.outdir)

    if args.save_plots:
        print(f"\nSaved PNG files to: {os.path.abspath(args.outdir)}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
