#!/usr/bin/env python3
"""
scripts/figures_real.py — publication-quality figure generator for
the dsfb-robotics paper.

Reads `data/processed/<slug>.csv` (one residual norm per row) directly,
runs the residual through the DSFB grammar offline (replicating the
canonical `paper-lock` pipeline), and renders six figure types per
dataset:

  1. Grammar-state timeline      — sample-by-sample Admissible (green)
                                    / Boundary (yellow) / Violation (red)
                                    shading with the residual norm
                                    overlaid; this is the headline
                                    "structure emerging" view.
  2. Residual-on-envelope        — residual norm vs. sample index,
                                    with ρ, boundary_frac·ρ, and the
                                    healthy-window calibration prefix
                                    overlaid.
  3. Side-by-side comparison     — upper panel: incumbent threshold
                                    monitor on the same residual scalar;
                                    lower panel: DSFB grammar timeline.
                                    Shows what the threshold *misses* in
                                    the trajectory structure.
  4. Compression histogram       — per-dataset review-surface
                                    compression ratio (across-dataset
                                    summary).
  5. Semiotic-manifold 3D scatter — the (‖r‖, ṙ, r̈) tuple as a 3-D
                                    point cloud coloured by grammar
                                    state. The literal visual of the
                                    "structure emerges from residuals"
                                    thesis.
  6. Per-joint 3D scatter        — for kinematic arms only, one panel
                                    per joint of the (τ_meas, τ_pred,
                                    residual) triplet. Reveals which
                                    joint(s) carry the residual.

Output: publication-quality PDFs in `paper/figures/`. PNG renders
also written for the Colab notebook to display inline. Same DSFB
parameters as `paper-lock`: W=8, K=4, ρ from Stage III calibration,
hysteresis=2, boundary_frac=0.5.
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from dataclasses import dataclass
from pathlib import Path

import numpy as np


CRATE_ROOT = Path(__file__).resolve().parent.parent
PROCESSED_DIR = CRATE_ROOT / "data" / "processed"
DEFAULT_OUT_DIR = CRATE_ROOT / "paper" / "figures"

# Canonical DSFB parameters (must match `paper_lock::PAPER_LOCK_*`).
W = 8
K = 4
BOUNDARY_FRAC = 0.5
DELTA_S = 0.05

# Stable, alphabetised dataset slug list (must include all preprocessed CSVs).
SLUGS = [
    "aloha_static",
    "aloha_static_pingpong_test",
    "aloha_static_screw_driver",
    "aloha_static_tape",
    "anymal_parkour",
    "cheetah3",
    "cwru",
    "dlr_justin",
    "droid",
    "femto_st",
    "icub3_sorrentino",
    "icub_pushrecovery",
    "ims",
    "kuka_lwr",
    "mobile_aloha",
    "openx",
    "panda_gaz",
    "so100",
    "unitree_g1",
    "ur10_kufieta",
]

# Datasets with per-joint CSVs (kinematic arms). Joint counts must
# match the per-joint CSV headers.
ARM_DATASETS = {
    "kuka_lwr": ("KUKA LWR-IV+ (Simionato 7R)", 7, "joint accel residual (m/s²)"),
    "panda_gaz": ("Franka Panda (Gaz 2019)", 7, "torque deviation (N·m)"),
    "dlr_justin": ("Panda DLR-class (Giacomuzzo 2024)", 7, "τ_meas − τ_interp (N·m)"),
    "ur10_kufieta": ("UR10 (Polydoros 2015)", 6, "torque deviation (N·m)"),
}

# Grammar colour palette — consistent across every figure.
COL_ADM = "#4caf50"
COL_BND = "#ffca28"
COL_VIO = "#e53935"
COL_LINE = "#212121"

# Family colours for the cross-dataset histogram.
FAMILY_COLOR = {
    "PHM": "#546e7a",
    "Kinematics": "#1e88e5",
    "Balancing": "#8e24aa",
}

DATASET_FAMILY = {
    "aloha_static": "Kinematics", "aloha_static_pingpong_test": "Kinematics",
    "aloha_static_screw_driver": "Kinematics", "aloha_static_tape": "Kinematics",
    "anymal_parkour": "Balancing", "cheetah3": "Balancing", "cwru": "PHM",
    "dlr_justin": "Kinematics", "droid": "Kinematics", "femto_st": "PHM",
    "icub3_sorrentino": "Balancing", "icub_pushrecovery": "Balancing",
    "ims": "PHM", "kuka_lwr": "Kinematics", "mobile_aloha": "Kinematics",
    "openx": "Kinematics", "panda_gaz": "Kinematics", "so100": "Kinematics",
    "unitree_g1": "Balancing", "ur10_kufieta": "Kinematics",
}


# ---------------------------------------------------------------
# DSFB pipeline (Python port of crate's grammar FSM)
# ---------------------------------------------------------------
@dataclass
class Episode:
    index: int
    norm: float           # ‖r‖
    drift: float          # ṙ
    slew: float           # r̈
    grammar: str          # "Admissible" | "Boundary" | "Violation"


def calibrate_envelope(residuals: np.ndarray) -> tuple[float, int]:
    """Stage III calibration: ρ = μ + 3σ over the first 20 % finite samples."""
    n = len(residuals)
    if n == 0:
        return float("inf"), 0
    cal_n = max(1, n // 5)
    cal = residuals[:cal_n]
    finite = cal[np.isfinite(cal)]
    if finite.size == 0:
        return float("inf"), cal_n
    mu = float(np.mean(np.abs(finite)))
    sigma = float(np.std(np.abs(finite)))
    return mu + 3.0 * sigma, cal_n


def run_dsfb(residuals: np.ndarray) -> tuple[list[Episode], float]:
    """Replay the canonical DSFB engine offline. Returns (episodes,
    envelope_radius). Mirrors `dsfb_robotics::engine::DsfbRoboticsEngine`."""
    rho, _ = calibrate_envelope(residuals)
    eps: list[Episode] = []
    norms_window: list[float] = []
    prev_drift = 0.0
    pending = "Admissible"
    confirms = 0
    committed = "Admissible"
    boundary_hits = [False] * K
    hit_head = 0
    hit_count = 0

    for i, r in enumerate(residuals):
        below_floor = not np.isfinite(r)
        norm = abs(r) if np.isfinite(r) else 0.0
        norms_window.append(norm)
        if len(norms_window) > W:
            norms_window.pop(0)
        if below_floor or len(norms_window) < 2:
            drift = 0.0
            slew = 0.0
            prev_drift = 0.0
        else:
            diffs = np.diff(norms_window)
            drift = float(np.mean(diffs))
            slew = drift - prev_drift
            prev_drift = drift

        # Compute raw state.
        if norm > rho:
            raw = "Violation"
        elif norm > BOUNDARY_FRAC * rho:
            if drift > 0:
                raw = "Boundary"
            elif abs(slew) > DELTA_S:
                raw = "Boundary"
            else:
                raw = check_grazing(boundary_hits, hit_count)
        else:
            raw = check_grazing(boundary_hits, hit_count)

        # Update grazing history.
        is_approach = (
            norm > BOUNDARY_FRAC * rho
            and norm <= rho
        )
        boundary_hits[hit_head] = is_approach
        hit_head = (hit_head + 1) % K
        if hit_count < K:
            hit_count += 1

        # 2-confirmation hysteresis.
        if raw == pending:
            if confirms < 2:
                confirms += 1
            if confirms >= 2:
                committed = raw
        else:
            pending = raw
            confirms = 1

        eps.append(Episode(index=i, norm=norm, drift=drift, slew=slew, grammar=committed))
    return eps, rho


def check_grazing(hits: list[bool], hit_count: int) -> str:
    if hit_count >= K and sum(hits) >= K:
        return "Boundary"
    return "Admissible"


# ---------------------------------------------------------------
# CSV loaders
# ---------------------------------------------------------------
def _load_csv_norms(path: Path) -> np.ndarray:
    if not path.is_file():
        return np.array([], dtype=np.float64)
    arr: list[float] = []
    with path.open("r") as f:
        r = csv.reader(f)
        next(r, None)  # header
        for row in r:
            if not row:
                continue
            tok = row[0].strip().lower()
            if tok in ("nan",):
                arr.append(float("nan"))
            elif tok in ("inf", "+inf", "infinity"):
                arr.append(float("inf"))
            elif tok in ("-inf", "-infinity"):
                arr.append(float("-inf"))
            else:
                try:
                    arr.append(float(tok))
                except ValueError:
                    arr.append(float("nan"))
    return np.array(arr, dtype=np.float64)


def load_residuals(slug: str) -> np.ndarray:
    return _load_csv_norms(PROCESSED_DIR / f"{slug}.csv")


def load_per_joint(slug: str) -> np.ndarray | None:
    path = PROCESSED_DIR / f"{slug}_per_joint.csv"
    if not path.is_file():
        return None
    return np.loadtxt(path, delimiter=",", skiprows=1, dtype=np.float64)


def color_for(grammar: str) -> str:
    if grammar == "Admissible":
        return COL_ADM
    if grammar == "Boundary":
        return COL_BND
    return COL_VIO


# ---------------------------------------------------------------
# Figure renderers
# ---------------------------------------------------------------
def render_grammar_timeline(slug: str, residuals: np.ndarray, eps: list[Episode],
                            rho: float, out: Path) -> None:
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(8.0, 3.6), dpi=300)
    n = len(eps)
    if n == 0:
        plt.close(fig)
        return
    # Downsample for very long traces.
    stride = max(1, n // 4096)
    idx = np.arange(0, n, stride)
    norm = np.array([eps[i].norm for i in idx])
    grammars = [eps[i].grammar for i in idx]
    for i, g in enumerate(grammars):
        ax.axvspan(idx[i] - stride / 2, idx[i] + stride / 2,
                   color=color_for(g), alpha=0.25, lw=0)
    ax.plot(idx, norm, color=COL_LINE, lw=1.0)
    ax.axhline(rho, color=COL_VIO, ls="--", lw=0.9, label=f"ρ={rho:.3g}")
    ax.axhline(BOUNDARY_FRAC * rho, color=COL_BND, ls="--", lw=0.9,
               label=f"{BOUNDARY_FRAC}·ρ={BOUNDARY_FRAC * rho:.3g}")
    ax.set_xlim(0, n)
    ax.set_xlabel("sample index k")
    ax.set_ylabel("residual norm ‖r(k)‖")
    ax.set_title(f"{slug} — grammar-state timeline ({n:,} real samples)")
    legend_proxies = [
        plt.Rectangle((0, 0), 1, 1, fc=COL_ADM, alpha=0.6, label="Admissible"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_BND, alpha=0.6, label="Boundary"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_VIO, alpha=0.6, label="Violation"),
    ]
    ax.legend(handles=legend_proxies, loc="upper left", framealpha=0.85, fontsize=8)
    ax.grid(alpha=0.18)
    fig.tight_layout()
    _save(fig, out, "grammar_timeline")
    plt.close(fig)


def render_envelope(slug: str, residuals: np.ndarray, eps: list[Episode],
                    rho: float, cal_n: int, out: Path) -> None:
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(8.0, 3.6), dpi=300)
    n = len(eps)
    if n == 0:
        plt.close(fig)
        return
    stride = max(1, n // 4096)
    idx = np.arange(0, n, stride)
    norm = np.array([eps[i].norm for i in idx])
    ax.plot(idx, norm, color=COL_LINE, lw=1.2, label="‖r(k)‖")
    ax.axhline(rho, color=COL_VIO, ls="--", lw=1.4, label=f"ρ = μ + 3σ = {rho:.3g}")
    ax.axhline(BOUNDARY_FRAC * rho, color=COL_BND, ls="--", lw=1.4,
               label=f"{BOUNDARY_FRAC}·ρ = {BOUNDARY_FRAC * rho:.3g}")
    ax.axvspan(0, cal_n, color="#2196f3", alpha=0.10, label=f"healthy window (k<{cal_n})")
    ax.set_xlim(0, n)
    ax.set_xlabel("sample index k")
    ax.set_ylabel("residual norm ‖r(k)‖")
    ax.set_title(f"{slug} — residual on admissibility envelope ({n:,} real samples)")
    ax.legend(loc="upper left", framealpha=0.85, fontsize=8)
    ax.grid(alpha=0.18)
    fig.tight_layout()
    _save(fig, out, "residual_on_envelope")
    plt.close(fig)


def render_comparison(slug: str, residuals: np.ndarray, eps: list[Episode],
                      rho: float, out: Path) -> None:
    """T1.2: side-by-side incumbent-threshold-alarm vs DSFB grammar."""
    import matplotlib.pyplot as plt
    n = len(eps)
    if n == 0:
        return
    fig, (ax_top, ax_bot) = plt.subplots(2, 1, figsize=(8.0, 5.4), dpi=300, sharex=True)
    stride = max(1, n // 4096)
    idx = np.arange(0, n, stride)
    norm = np.array([eps[i].norm for i in idx])

    # Upper panel: incumbent (3σ-threshold) view of the same residual.
    ax_top.plot(idx, norm, color=COL_LINE, lw=1.0)
    ax_top.axhline(rho, color=COL_VIO, ls="--", lw=1.2,
                   label=f"3σ threshold (ρ={rho:.3g})")
    above = norm > rho
    ax_top.fill_between(idx, 0, norm, where=above, color=COL_VIO, alpha=0.18,
                        label="incumbent ALARM")
    ax_top.set_ylabel("‖r(k)‖")
    ax_top.set_title(f"{slug} — incumbent threshold monitor (top) vs DSFB grammar (bottom) "
                     f"on the SAME residual trace")
    ax_top.legend(loc="upper left", framealpha=0.85, fontsize=8)
    ax_top.grid(alpha=0.18)

    # Lower panel: DSFB grammar shading on the same trace.
    grammars = [eps[i].grammar for i in idx]
    for i, g in enumerate(grammars):
        ax_bot.axvspan(idx[i] - stride / 2, idx[i] + stride / 2,
                       color=color_for(g), alpha=0.25, lw=0)
    ax_bot.plot(idx, norm, color=COL_LINE, lw=1.0)
    ax_bot.axhline(rho, color=COL_VIO, ls="--", lw=0.9)
    ax_bot.axhline(BOUNDARY_FRAC * rho, color=COL_BND, ls="--", lw=0.9)
    ax_bot.set_xlim(0, n)
    ax_bot.set_xlabel("sample index k")
    ax_bot.set_ylabel("‖r(k)‖")
    legend_proxies = [
        plt.Rectangle((0, 0), 1, 1, fc=COL_ADM, alpha=0.6, label="Admissible"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_BND, alpha=0.6, label="Boundary"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_VIO, alpha=0.6, label="Violation"),
    ]
    ax_bot.legend(handles=legend_proxies, loc="upper left", framealpha=0.85, fontsize=8)
    ax_bot.grid(alpha=0.18)
    fig.tight_layout()
    _save(fig, out, "comparison")
    plt.close(fig)


def render_hero_augmentation(slug: str, residuals: np.ndarray, eps: list[Episode],
                             rho: float, out: Path) -> None:
    """Side-by-side augmentation hero panel.

    Top panel: a horizontal band representing the Gaz-style scalar
    summary (one number $\\sigma_{\\text{noise}}$ collapsed across the
    entire trajectory) drawn over the faintly-overlaid residual
    stream. Bottom panel: the DSFB grammar timeline on the same
    residual. Together: incumbent collapses to one scalar, DSFB
    structures the same residual into a per-timestep grammar timeline.
    """
    import matplotlib.pyplot as plt
    n = len(eps)
    if n == 0:
        return
    sigma_noise = float(np.std(residuals))
    fig, (ax_top, ax_bot) = plt.subplots(2, 1, figsize=(8.0, 5.4), dpi=300, sharex=True)
    stride = max(1, n // 4096)
    idx = np.arange(0, n, stride)
    norm = np.array([eps[i].norm for i in idx])

    ax_top.plot(idx, norm, color=COL_LINE, lw=0.7, alpha=0.45, label="‖r(k)‖ (faint)")
    ax_top.axhspan(0.0, sigma_noise, color="#1e88e5", alpha=0.20,
                   label=f"σ_noise = {sigma_noise:.3g} (the one Gaz scalar)")
    ax_top.axhline(sigma_noise, color="#1e88e5", lw=1.6)
    ax_top.set_ylabel("‖r(k)‖")
    ax_top.set_title(f"{slug} — incumbent: one σ_noise scalar (top)  vs.  "
                     f"DSFB: per-timestep grammar (bottom) on the SAME residual")
    ax_top.legend(loc="upper left", framealpha=0.9, fontsize=8)
    ax_top.grid(alpha=0.18)
    ax_top.set_ylim(bottom=0)

    grammars = [eps[i].grammar for i in idx]
    for i, g in enumerate(grammars):
        ax_bot.axvspan(idx[i] - stride / 2, idx[i] + stride / 2,
                       color=color_for(g), alpha=0.25, lw=0)
    ax_bot.plot(idx, norm, color=COL_LINE, lw=1.0)
    ax_bot.axhline(rho, color=COL_VIO, ls="--", lw=0.9, label=f"ρ={rho:.3g}")
    ax_bot.axhline(BOUNDARY_FRAC * rho, color=COL_BND, ls="--", lw=0.9,
                   label=f"{BOUNDARY_FRAC}·ρ={BOUNDARY_FRAC * rho:.3g}")
    ax_bot.set_xlim(0, n)
    ax_bot.set_xlabel("sample index k")
    ax_bot.set_ylabel("‖r(k)‖")
    legend_proxies = [
        plt.Rectangle((0, 0), 1, 1, fc=COL_ADM, alpha=0.6, label="Admissible"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_BND, alpha=0.6, label="Boundary"),
        plt.Rectangle((0, 0), 1, 1, fc=COL_VIO, alpha=0.6, label="Violation"),
    ]
    ax_bot.legend(handles=legend_proxies, loc="upper left", framealpha=0.9, fontsize=8)
    ax_bot.grid(alpha=0.18)
    fig.tight_layout()
    _save(fig, out, "hero_augmentation")
    plt.close(fig)


def render_semiotic_manifold(slug: str, eps: list[Episode], out: Path) -> None:
    """T1.4.5: 3D scatter of (‖r‖, ṙ, r̈) coloured by grammar state."""
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d import Axes3D  # noqa: F401
    n = len(eps)
    if n == 0:
        return
    # Downsample for tractability (3D scatter of 50k points is slow + noisy).
    stride = max(1, n // 6000)
    sel = list(range(0, n, stride))
    norms = np.array([eps[i].norm for i in sel])
    drifts = np.array([eps[i].drift for i in sel])
    slews = np.array([eps[i].slew for i in sel])
    cols = [color_for(eps[i].grammar) for i in sel]

    fig = plt.figure(figsize=(7.2, 6.0), dpi=300)
    ax = fig.add_subplot(111, projection="3d")
    ax.scatter(norms, drifts, slews, c=cols, s=4, alpha=0.55, lw=0)
    ax.set_xlabel("‖r(k)‖")
    ax.set_ylabel("ṙ(k)")
    ax.set_zlabel("r̈(k)")
    ax.set_title(f"{slug} — semiotic manifold σ(k) = (‖r‖, ṙ, r̈)\n"
                 f"({len(sel):,}-point downsample of {n:,} real samples; "
                 f"colour = grammar state)")
    legend_proxies = [
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_ADM,
                   markersize=8, label="Admissible"),
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_BND,
                   markersize=8, label="Boundary"),
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_VIO,
                   markersize=8, label="Violation"),
    ]
    ax.legend(handles=legend_proxies, loc="upper left", framealpha=0.85, fontsize=9)
    fig.tight_layout()
    _save(fig, out, "semiotic_manifold_3d")
    plt.close(fig)


def render_per_joint_3d(slug: str, eps: list[Episode], out: Path) -> None:
    """T1.4 + T1.4.5: per-joint (τ_meas, τ_pred, residual) 3D scatter for arms.

    Since we only have the per-joint *residuals* in the processed CSV (not
    the raw τ_meas/τ_pred), we instead render per-joint (k, joint_residual,
    grammar_severity) as a 3-D scatter — the joint-axis-resolved analogue
    that shows which joint carries the residual at each k.
    """
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d import Axes3D  # noqa: F401
    pj = load_per_joint(slug)
    if pj is None or pj.size == 0 or slug not in ARM_DATASETS:
        return
    label, n_joints, units = ARM_DATASETS[slug]
    n = len(eps)
    if n == 0:
        return
    n_use = min(n, pj.shape[0])
    stride = max(1, n_use // 2000)
    sel = list(range(0, n_use, stride))

    fig = plt.figure(figsize=(8.4, 6.4), dpi=300)
    ax = fig.add_subplot(111, projection="3d")
    severity = {"Admissible": 0, "Boundary": 1, "Violation": 2}
    for i in sel:
        sev = severity[eps[i].grammar]
        col = color_for(eps[i].grammar)
        for j in range(n_joints):
            ax.scatter(i, j, pj[i, j], c=col, s=3, alpha=0.45, lw=0)
        del sev
    ax.set_xlabel("sample index k")
    ax.set_ylabel("joint index")
    ax.set_zlabel(units)
    ax.set_title(f"{slug} — per-joint residual decomposition\n"
                 f"({label}, {n_joints} joints; colour = grammar state)")
    legend_proxies = [
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_ADM,
                   markersize=8, label="Admissible"),
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_BND,
                   markersize=8, label="Boundary"),
        plt.Line2D([0], [0], marker="o", color="w", markerfacecolor=COL_VIO,
                   markersize=8, label="Violation"),
    ]
    ax.legend(handles=legend_proxies, loc="upper left", framealpha=0.85, fontsize=9)
    ax.set_yticks(list(range(n_joints)))
    fig.tight_layout()
    _save(fig, out, "per_joint_3d")
    plt.close(fig)


def render_compression_histogram(reports: dict, out: Path) -> None:
    import matplotlib.pyplot as plt
    if not reports:
        return
    slugs = sorted(reports.keys())
    ratios = [reports[s]["compression_ratio"] for s in slugs]
    families = [DATASET_FAMILY.get(s, "Unknown") for s in slugs]
    cols = [FAMILY_COLOR.get(f, "#9e9e9e") for f in families]

    fig, ax = plt.subplots(figsize=(11.0, 4.5), dpi=300)
    xs = np.arange(len(slugs))
    ax.bar(xs, ratios, color=cols, edgecolor=COL_LINE, lw=0.6)
    ax.set_xticks(xs)
    ax.set_xticklabels(slugs, rotation=42, ha="right", fontsize=8)
    ax.set_ylim(0.0, 1.0)
    ax.set_ylabel("compression ratio  (reviewed / total samples)")
    ax.set_title(f"per-dataset review-surface compression on real-world residuals "
                 f"({len(slugs)} datasets)")
    proxies = [plt.Rectangle((0, 0), 1, 1, fc=c, label=k) for k, c in FAMILY_COLOR.items()]
    ax.legend(handles=proxies, loc="upper right", framealpha=0.9, fontsize=9)
    ax.grid(axis="y", alpha=0.18)
    fig.tight_layout()
    _save(fig, out, "_all_compression_histogram")
    plt.close(fig)


def _save(fig, out_dir: Path, name: str) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    pdf_path = out_dir / f"{name}.pdf"
    png_path = out_dir / f"{name}.png"
    fig.savefig(pdf_path, dpi=300, bbox_inches="tight")
    fig.savefig(png_path, dpi=180, bbox_inches="tight")


# ---------------------------------------------------------------
# Per-dataset aggregate computation
# ---------------------------------------------------------------
def aggregate_eps(eps: list[Episode]) -> dict:
    n = len(eps)
    adm = sum(1 for e in eps if e.grammar == "Admissible")
    bnd = sum(1 for e in eps if e.grammar == "Boundary")
    vio = sum(1 for e in eps if e.grammar == "Violation")
    reviewed = bnd + vio
    return {
        "total_samples": n,
        "admissible": adm,
        "boundary": bnd,
        "violation": vio,
        "compression_ratio": (reviewed / n) if n > 0 else 0.0,
        "max_residual_norm_sq": max((e.norm ** 2 for e in eps), default=0.0),
    }


# ---------------------------------------------------------------
# Driver
# ---------------------------------------------------------------
def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only", nargs="+", default=None, help="Subset of slugs.")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT_DIR,
                        help=f"Output directory (default: {DEFAULT_OUT_DIR}).")
    parser.add_argument("--no-3d", action="store_true",
                        help="Skip 3D scatter plots (faster).")
    args = parser.parse_args(argv)

    try:
        import matplotlib  # noqa
    except ImportError:
        print("matplotlib required: pip install --user --break-system-packages matplotlib",
              file=sys.stderr)
        return 64
    import matplotlib
    matplotlib.use("Agg")

    slugs = args.only if args.only else SLUGS
    args.out.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    reports: dict = {}

    for slug in slugs:
        if slug not in SLUGS:
            print(f"  skip {slug}: not in SLUGS list", file=sys.stderr)
            continue
        residuals = load_residuals(slug)
        if residuals.size == 0:
            print(f"  skip {slug}: no residuals", file=sys.stderr)
            continue
        eps, rho = run_dsfb(residuals)
        _, cal_n = calibrate_envelope(residuals)
        agg = aggregate_eps(eps)
        reports[slug] = agg
        out_dir = args.out / slug
        print(
            f"[figures_real] {slug}: n={agg['total_samples']:>6d} "
            f"adm={agg['admissible']:>5d} bnd={agg['boundary']:>5d} "
            f"vio={agg['violation']:>5d} compression={agg['compression_ratio']:.3f}",
            file=sys.stderr,
        )
        render_grammar_timeline(slug, residuals, eps, rho, out_dir)
        render_envelope(slug, residuals, eps, rho, cal_n, out_dir)
        # Comparison figures only for the three exemplars selected in T1.2.
        if slug in {"cwru", "kuka_lwr", "icub_pushrecovery"}:
            render_comparison(slug, residuals, eps, rho, out_dir)
        # Side-by-side hero augmentation panel for the canonical exemplar.
        # Use the literal Gaz-2019 published-θ residual (not the early-window-nominal
        # proxy) so the figure caption's "literal published-model residual" claim
        # is mechanically true.
        if slug == "panda_gaz":
            published_path = PROCESSED_DIR / "panda_gaz_published.csv"
            if published_path.is_file():
                published_residuals = _load_csv_norms(published_path)
                if published_residuals.size > 0:
                    eps_pub, rho_pub = run_dsfb(published_residuals)
                    render_hero_augmentation(slug, published_residuals, eps_pub,
                                             rho_pub, out_dir)
                else:
                    render_hero_augmentation(slug, residuals, eps, rho, out_dir)
            else:
                render_hero_augmentation(slug, residuals, eps, rho, out_dir)
        if not args.no_3d:
            render_semiotic_manifold(slug, eps, out_dir)
            if slug in ARM_DATASETS:
                render_per_joint_3d(slug, eps, out_dir)

    # Cross-dataset summary.
    render_compression_histogram(reports, args.out)

    # Manifest.
    manifest = {
        "figures_manifest_version": "2",
        "source": "scripts/figures_real.py",
        "mode": "real-data",
        "datasets": sorted(reports.keys()),
        "params": {"W": W, "K": K, "boundary_frac": BOUNDARY_FRAC, "delta_s": DELTA_S},
        "aggregates": reports,
    }
    (args.out / "FIGURES_MANIFEST.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    )
    print(f"[figures_real] wrote figures for {len(reports)} datasets to {args.out}",
          file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
