#!/usr/bin/env python3
"""
DSFB-RF Publication Figure Generator
=====================================
Reads  paper/figure_data.json  (produced by  cargo run --features std,serde
--example generate_figures) and renders 20 publication-quality figures for the
paper and the elite panel briefing.

Usage:
    cd paper
    python3 figures.py                    # all 20 figures
    python3 figures.py --fig 2 8 13       # specific figures only
    python3 figures.py --dpi 300          # print-resolution (default 150)

Output: paper/figs/fig_XX_*.pdf  +  paper/figs/fig_XX_*.png
"""

# ─── stdlib / third-party ─────────────────────────────────────────────────
import argparse
import json
import math
import os
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as pe
import matplotlib.ticker as ticker
from matplotlib.colors import ListedColormap, LinearSegmentedColormap
from matplotlib.gridspec import GridSpec
from matplotlib.lines import Line2D
import matplotlib.transforms as transforms
import numpy as np
from mpl_toolkits.mplot3d import Axes3D  # noqa: F401

# ─── global style (IEEE-friendly) ─────────────────────────────────────────
plt.rcParams.update({
    "font.family":        "serif",
    "font.serif":         ["Times New Roman", "DejaVu Serif"],
    "font.size":          9,
    "axes.titlesize":     9,
    "axes.labelsize":     9,
    "legend.fontsize":    8,
    "xtick.labelsize":    8,
    "ytick.labelsize":    8,
    "axes.linewidth":     0.8,
    "lines.linewidth":    1.2,
    "grid.linewidth":     0.4,
    "grid.alpha":         0.4,
    "axes.grid":          True,
    "figure.dpi":         150,
    "savefig.dpi":        150,
    "savefig.bbox":       "tight",
    "savefig.pad_inches": 0.04,
    "text.usetex":        False,
})

# Palette — colour-blind safe + IEEE-print compatible
C_ADMISSIBLE  = "#2ca02c"   # green
C_BOUNDARY    = "#ff7f0e"   # orange
C_VIOLATION   = "#d62728"   # red
C_DSFB        = "#1f77b4"   # blue
C_COMPARATOR  = "#9467bd"   # purple
C_NEUTRAL     = "#7f7f7f"   # grey
C_HIGHLIGHT   = "#e377c2"   # pink

GRAMMAR_COLOR = {
    "Admissible":                   C_ADMISSIBLE,
    "Boundary(SustainedOutwardDrift)": C_BOUNDARY,
    "Boundary(AbruptSlewViolation)":   C_BOUNDARY,
    "Boundary(RecurrentBoundaryGrazing)": C_BOUNDARY,
    "Violation":                    C_VIOLATION,
}


def grammar_color(state: str) -> str:
    for k, v in GRAMMAR_COLOR.items():
        if k in state:
            return v
    return C_NEUTRAL


def save(fig, idx: int, name: str, out_dir: Path, dpi: int = 150):
    out_dir.mkdir(parents=True, exist_ok=True)
    stub = f"fig_{idx:02d}_{name}"
    for ext in ("pdf", "png"):
        path = out_dir / f"{stub}.{ext}"
        fig.savefig(path, dpi=dpi, format=ext)
    print(f"  Saved fig {idx:02d}: {stub}.*")
    plt.close(fig)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 1 — Semiotic Manifold Partition
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig01(d, out_dir, dpi):
    pts  = d["fig01_semiotic_manifold"]["points"]
    norm  = np.array([p["norm"]  for p in pts], dtype=float)
    drift = np.array([p["drift"] for p in pts], dtype=float)
    slew  = np.array([p["slew"]  for p in pts], dtype=float)
    regime = [p["regime"] for p in pts]
    colors = [grammar_color(r) for r in regime]
    labels = sorted(set(regime), key=lambda x: ["Admissible","Boundary","Violation"].index(x) if x in ["Admissible","Boundary","Violation"] else 3)

    fig = plt.figure(figsize=(4.5, 3.8))
    ax  = fig.add_subplot(111, projection="3d")

    for lab in labels:
        mask = np.array([r == lab for r in regime])
        ax.scatter(norm[mask], drift[mask], slew[mask],
                   c=grammar_color(lab), s=14, alpha=0.80,
                   label=lab, depthshade=False)

    # Envelope plane (ρ = 0.10)
    rho = 0.10
    _s = np.linspace(-0.015, 0.015, 4)
    _d = np.linspace(-0.015, 0.015, 4)
    S, Dr = np.meshgrid(_s, _d)
    R = np.full_like(S, rho)
    ax.plot_surface(R, Dr, S, alpha=0.12, color=C_VIOLATION, zorder=0)

    ax.set_xlabel("‖r(k)‖", labelpad=4)
    ax.set_ylabel("ṙ(k) drift", labelpad=4)
    ax.set_zlabel("r̈(k) slew", labelpad=4)
    ax.set_title("Fig. 1 — Semiotic Manifold $\\mathcal{M}_{\\mathrm{sem}}$\nGrammar Partition in $(\\|r\\|, \\dot r, \\ddot r)$ space")
    ax.legend(loc="upper left", framealpha=0.7)
    fig.tight_layout()
    save(fig, 1, "semiotic_manifold", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 2 — Review Surface Compression
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig02(d, out_dir, dpi):
    datasets = d["fig02_compression"]["datasets"]
    fig, axes = plt.subplots(1, 2, figsize=(7.0, 3.5))

    for ax, ds in zip(axes, datasets):
        comps = ds["comparators"]
        names     = [c["name"].replace("(","(\n") for c in comps]
        precision = np.array([c["precision"] for c in comps])
        episodes  = np.array([c["episodes"]  for c in comps])

        colors = [C_DSFB if "DSFB" in n else C_NEUTRAL for n in names]

        x = np.arange(len(names))
        bars = ax.bar(x, precision, color=colors, edgecolor="black", linewidth=0.6, zorder=3)

        # Annotate episode count
        for bar, ep in zip(bars, episodes):
            ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 0.01,
                    f"{ep:,}", ha="center", va="bottom", fontsize=6.5, rotation=0)

        ax.set_xticks(x)
        ax.set_xticklabels(names, fontsize=6.5)
        ax.set_ylim(0, 1.05)
        ax.set_ylabel("Episode Precision")
        label = ds["dataset"][:30]
        ax.set_title(f"Fig. 2 — {label}\n"
                     f"Compression {ds['compression_ratio']:.0f}× | "
                     f"Precision gain {ds['precision_gain']:.1f}× | Recall {ds['recall']*100:.1f}%",
                     fontsize=7.5)
        ax.axhline(0.73, color=C_DSFB, lw=0.8, ls="--", label="DSFB precision")
        ax.grid(axis="y", zorder=0)

    fig.suptitle("Review Surface Compression — DSFB vs. Scalar Comparators\n"
                 "(RadioML 2018.01a synthetic  +  ORACLE real USRP B200)",
                 fontsize=9, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.90])
    save(fig, 2, "compression_comparison", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 3 — Observer-of-Observer Structural Blindspot (Theorem OoT)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig03(d, out_dir, dpi):
    oot    = d["fig03_oot_blindspot"]
    traj   = np.array(oot["trajectory"])
    l_alarm = np.array(oot["luenberger_alarm"])
    grammar = oot["dsfb_grammar"]
    k = np.arange(len(traj))

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 3.8), sharex=True)

    # Top: trajectory + Luenberger threshold
    ax1.plot(k, traj, color=C_DSFB, lw=1.3, label="‖r(k)‖")
    ax1.axhline(0.10, color=C_VIOLATION, lw=1.0, ls="--",
                label="Luenberger threshold δ = 0.10")
    ax1.axhline(0.005, color=C_NEUTRAL, lw=0.7, ls=":",
                label="‖L·r(k)‖ = 0.5·‖r(k)‖")
    # Shade DSFB detection region
    first_boundary = next((i for i, g in enumerate(grammar)
                           if "Boundary" in g or "Violation" in g), len(k))
    ax1.axvspan(first_boundary, len(k), color=C_BOUNDARY, alpha=0.12,
                label=f"DSFB Boundary detected (k={first_boundary})")
    ax1.set_ylabel("Residual Norm")
    ax1.legend(fontsize=7, ncol=2)
    ax1.set_title("Fig. 3 — Observer-of-Observer Structural Blindspot (Theorem OoT)\n"
                  "$\\mathcal{T}_{\\mathrm{blind}}$: small ‖r‖ but persistent $\\dot r>0$")

    # Bottom: Grammar state timeline
    colors = [grammar_color(g) for g in grammar]
    for i, (g, c) in enumerate(zip(grammar, colors)):
        ax2.barh(0, 1, left=i, height=0.7, color=c, edgecolor="none")
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Admissible")
    patch_B = mpatches.Patch(color=C_BOUNDARY,   label="Boundary")
    patch_V = mpatches.Patch(color=C_VIOLATION,  label="Violation")
    ax2.legend(handles=[patch_A, patch_B, patch_V], loc="lower right",
               fontsize=7, ncol=3)
    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))

    fig.tight_layout()
    save(fig, 3, "oot_blindspot", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 4 — DSFB Pipeline DAG
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig04(d, out_dir, dpi):
    stages = d["fig04_pipeline"]["stages"]
    edges  = d["fig04_pipeline"]["edges"]

    fig, ax = plt.subplots(figsize=(6.5, 2.5))
    ax.set_xlim(-0.5, len(stages) - 0.5)
    ax.set_ylim(-1, 1.4)
    ax.axis("off")
    ax.set_title("Fig. 4 — DSFB-RF Deterministic Pipeline\n"
                 "(no_std · no_alloc · zero unsafe — compiler-enforced at every stage)",
                 fontsize=8.5)

    box_colors = ["#aec7e8", "#c5b0d5", "#98df8a", "#ffbb78",
                  "#c49c94", "#f7b6d2", "#dbdb8d"]
    w, h = 0.82, 0.48

    for s in stages:
        x = s["id"]
        col = box_colors[s["id"] % len(box_colors)]
        rect = mpatches.FancyBboxPatch((x - w/2, -h/2), w, h,
            boxstyle="round,pad=0.04", facecolor=col,
            edgecolor="#333333", linewidth=0.8, zorder=3)
        ax.add_patch(rect)
        ax.text(x, 0.06, s["name"], ha="center", va="center",
                fontsize=7.5, fontweight="bold", zorder=4)
        ax.text(x, -0.12, s["module"], ha="center", va="center",
                fontsize=6, color="#444444", zorder=4)
        ax.text(x, -0.28, s["output_type"], ha="center", va="center",
                fontsize=6.5, color="#222222",
                fontstyle="italic", zorder=4)
        ax.text(x, 0.44, s["theorem"], ha="center", va="center",
                fontsize=6, color="#666666", zorder=4)

    for (a, b) in edges:
        x1, x2 = a + w/2 + 0.01, b - w/2 - 0.01
        ax.annotate("", xy=(x2, 0), xytext=(x1, 0),
                    arrowprops=dict(arrowstyle="->", color="#333333",
                                   lw=1.1), zorder=2)

    fig.tight_layout()
    save(fig, 4, "pipeline_dag", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 5 — Finite-Time Envelope Exit (Theorem 1)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig05(d, out_dir, dpi):
    data  = d["fig05_envelope_exit"]
    rho   = data["rho"]
    curves = data["curves"]

    fig, ax = plt.subplots(figsize=(4.5, 3.0))

    cmap = plt.cm.get_cmap("coolwarm", len(curves))
    n_obs = len(curves[0]["trajectory"])
    k = np.arange(n_obs)

    for i, c in enumerate(curves):
        traj = np.array(c["trajectory"])
        k_star = c["k_star"]
        col = cmap(i / (len(curves) - 1))
        ax.plot(k, traj, color=col, lw=1.2, label=c["label"])
        # Mark theoretical k*
        ks_int = int(min(k_star, n_obs - 1))
        if ks_int < n_obs:
            ax.axvline(ks_int, color=col, lw=0.6, ls=":", alpha=0.7)

    ax.axhline(rho, color=C_VIOLATION, lw=1.4, ls="--",
               label=f"Envelope boundary ρ = {rho}")
    ax.fill_between(k, rho, ax.get_ylim()[1] if ax.get_ylim()[1] > rho else rho * 1.15,
                    alpha=0.08, color=C_VIOLATION)

    ax.set_xlabel("Observation k")
    ax.set_ylabel("Residual norm ‖r(k)‖")
    ax.set_ylim(-0.005, rho * 1.20)
    ax.legend(fontsize=7, loc="upper left", ncol=1)
    ax.set_title("Fig. 5 — Finite-Time Envelope Exit Bound (Theorem 1)\n"
                 "$k^* \\leq \\rho/\\alpha$ — computable without a noise model")
    fig.tight_layout()
    save(fig, 5, "envelope_exit_theorem1", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 6 — Lyapunov Exponent Time Series
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig06(d, out_dir, dpi):
    lp = d["fig06_lyapunov"]
    k       = np.array(lp["k"])
    lam     = np.array(lp["lambda"])
    grammar = lp["grammar"]
    stability = lp["stability"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 3.8), sharex=True)

    # Top: λ(k)
    ax1.plot(k, lam, color=C_DSFB, lw=1.2, label="λ(k) — FTLE")
    ax1.axhline(0.01, color=C_VIOLATION, lw=0.9, ls="--",
                label="λ_crit = 0.01 (ExponentialDivergence)")
    ax1.axhline(0.001, color=C_BOUNDARY, lw=0.7, ls=":",
                label="ε = 0.001 (MarginalDivergence)")
    ax1.axhline(0.0, color=C_NEUTRAL, lw=0.6, ls="-", alpha=0.5)
    ax1.fill_between(k, lam, 0, where=(lam > 0.01),
                     color=C_VIOLATION, alpha=0.18,
                     label="Exponential divergence zone")
    ax1.fill_between(k, lam, 0, where=((lam > 0.001) & (lam <= 0.01)),
                     color=C_BOUNDARY, alpha=0.14,
                     label="Marginal divergence zone")
    ax1.set_ylabel("Lyapunov exp. λ(k)")
    ax1.legend(fontsize=6.5, ncol=2)
    ax1.set_title("Fig. 6 — Finite-Time Lyapunov Exponent vs. Grammar State\n"
                  "Healthy (λ<0) → slow drift (λ>0) → jamming onset (λ spike) → recovery")

    # Bottom: Grammar timeline
    for i, g in enumerate(grammar):
        ax2.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Admissible (λ<0)")
    patch_B = mpatches.Patch(color=C_BOUNDARY,   label="Boundary (λ>0)")
    patch_V = mpatches.Patch(color=C_VIOLATION,  label="Violation")
    ax2.legend(handles=[patch_A, patch_B, patch_V], loc="lower right",
               fontsize=7, ncol=3)
    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))

    # Regime labels
    for label, start, end in [("Healthy\n(λ < 0)", 0, 30),
                               ("Thermal drift\n(λ > 0)", 30, 70),
                               ("Jamming\nonset", 70, 95),
                               ("Recovery", 95, len(k))]:
        ax1.text((start + end) / 2, ax1.get_ylim()[1] * 0.88, label,
                 ha="center", fontsize=6.5, color="#333333",
                 bbox=dict(boxstyle="round,pad=0.15", facecolor="white", alpha=0.7))

    fig.tight_layout()
    save(fig, 6, "lyapunov_time_series", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 7 — GUM Uncertainty Budget Waterfall
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig07(d, out_dir, dpi):
    g = d["fig07_gum"]
    contribs = g["contributors"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.4, 3.2), gridspec_kw={"width_ratios": [2, 1]})

    # Left: variance budget pie
    names  = [c["name"] for c in contribs]
    values = np.array([c["value"] ** 2 for c in contribs])  # variance
    fracs  = values / values.sum()
    type_colors = [C_DSFB if c["kind"] == "A" else C_BOUNDARY for c in contribs]

    wedges, texts, autotexts = ax1.pie(
        fracs, labels=names, autopct="%1.1f%%",
        colors=type_colors + [C_VIOLATION, C_NEUTRAL, C_HIGHLIGHT],
        startangle=140, textprops={"fontsize": 7}
    )
    for at in autotexts:
        at.set_fontsize(7)
    ax1.set_title("Variance Budget\n(Type A + Type B contributors)")

    # Right: waterfall — u_A, u_B, u_c, U=k·u_c, ρ_GUM
    summaries = [
        ("$u_A$ (Type A)", g["u_a"],         C_DSFB),
        ("$u_B$ (Type B)", g["u_b_combined"], C_BOUNDARY),
        ("$u_c$ combined", g["u_c"],          C_NEUTRAL),
        (f"$U = k{g['coverage_k']:.0f}{chr(183)}u_c$", g["expanded_u"], C_VIOLATION),
        ("$\\rho_{\\mathrm{GUM}} = \\mu + U$",        g["rho_gum"],    C_ADMISSIBLE),
    ]
    labels = [s[0] for s in summaries]
    vals   = [s[1] for s in summaries]
    cols   = [s[2] for s in summaries]

    bars = ax2.barh(range(len(labels)), vals, color=cols,
                    edgecolor="#333333", linewidth=0.6)
    ax2.set_yticks(range(len(labels)))
    ax2.set_yticklabels(labels, fontsize=7.5)
    ax2.set_xlabel("Norm units (‖r‖)")
    ax2.set_title(f"GUM Budget Summary\n"
                  f"ρ_GUM = {g['rho_gum']:.4f}  (k={g['coverage_k']:.0f}, WSS-verified)")
    for i, (bar, val) in enumerate(zip(bars, vals)):
        ax2.text(val + max(vals) * 0.02, i, f"{val:.5f}",
                 va="center", fontsize=7)
    ax2.invert_yaxis()
    ax2.grid(axis="x")

    fig.suptitle("Fig. 7 — GUM/JCGM 100:2008 Uncertainty Budget for Admissibility Envelope\n"
                 "Type A (statistical) + Type B (NF, ADC quantisation, thermal, LO) contributors",
                 fontweight="bold", fontsize=8.5)
    fig.tight_layout(rect=[0, 0, 1, 0.88])
    save(fig, 7, "gum_uncertainty_budget", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 8 — Semiotic Horizon Detection Heatmap
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig08(d, out_dir, dpi):
    hz  = d["fig08_horizon"]
    snr = np.array(hz["snr_levels"])
    alp = np.array(hz["alpha_levels"])
    det = np.array(hz["detection_rate"])    # shape (SNR, alpha)

    fig, ax = plt.subplots(figsize=(5.5, 3.8))

    cmap = LinearSegmentedColormap.from_list(
        "horizon", [(0.0, C_VIOLATION), (0.5, "#ffdd88"), (1.0, C_ADMISSIBLE)])
    im = ax.imshow(det, origin="lower", aspect="auto", cmap=cmap,
                   vmin=0.0, vmax=1.0,
                   extent=[np.log10(alp[0]), np.log10(alp[-1]),
                            snr[0], snr[-1]])

    cbar = fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
    cbar.set_label("Detection rate (0=miss, 1=detect)", fontsize=8)

    ax.set_xlabel("log₁₀(drift rate α)")
    ax.set_ylabel("SNR (dB)")
    ax.axhline(-10, color="white", lw=1.2, ls="--", label="SNR floor (−10 dB)")
    ax.legend(fontsize=7, loc="upper left")
    ax.set_title("Fig. 8 — Semiotic Horizon: Detection Surface in (SNR, α) space\n"
                 "Zone of Success (green) / Zone of Failure (red) — maps CRLB-limited boundary")

    # Annotate failure zone
    ax.text(np.log10(alp[0]) * 0.98, -14, "Sub-SNR floor:\nforced Admissible\n(L10 non-claim honored)",
            fontsize=6.5, color="white", ha="left", va="bottom",
            bbox=dict(boxstyle="round", facecolor="#333333", alpha=0.6))

    fig.tight_layout()
    save(fig, 8, "semiotic_horizon_heatmap", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 9 — Physics-of-Failure Mapping
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig09(d, out_dir, dpi):
    phys = d["fig09_physics"]
    nodes = phys["nodes"]
    edges = phys["edges"]

    fig, ax = plt.subplots(figsize=(7.0, 3.6))
    ax.axis("off")
    ax.set_title("Fig. 9 — Physics-of-Failure Map: Grammar State → Candidate Physical Mechanism\n"
                 "(candidate hypotheses — physical attribution requires deployment calibration data)",
                 fontsize=8.5)

    # Grammar nodes: left column; mechanism nodes: right column
    grammar_nodes = [n for n in nodes if n["kind"] == "grammar"]
    mech_nodes    = [n for n in nodes if n["kind"] == "mechanism"]

    gx, mx = 0.14, 0.72
    n_g, n_m = len(grammar_nodes), len(mech_nodes)

    gy = np.linspace(0.85, 0.15, n_g)
    my = np.linspace(0.90, 0.10, n_m)

    pos = {}
    for i, n in enumerate(grammar_nodes):
        pos[n["id"]] = (gx, gy[i])
    for i, n in enumerate(mech_nodes):
        pos[n["id"]] = (mx, my[i])

    # Edges (gradient width by weight)
    for e in edges:
        x0, y0 = pos[e["from"]]
        x1, y1 = pos[e["to"]]
        ax.plot([x0 + 0.09, x1 - 0.12], [y0, y1],
                color="#888888", lw=1.0 + 2.5 * e["weight"],
                alpha=0.35, solid_capstyle="round")
        mx_pos = (x0 + 0.09 + x1 - 0.12) / 2
        my_pos = (y0 + y1) / 2
        ax.text(mx_pos, my_pos, f"{e['weight']:.2f}",
                ha="center", va="center", fontsize=5.5, color="#444444")

    # Grammar node boxes
    GRAMMAR_BOX_COLORS = {
        "grammar":   "#c5b0d5",
        "mechanism": "#aec7e8",
    }
    for n in nodes:
        x, y = pos[n["id"]]
        w, h = (0.20, 0.07) if n["kind"] == "grammar" else (0.22, 0.08)
        rect = mpatches.FancyBboxPatch((x - w/2, y - h/2), w, h,
            boxstyle="round,pad=0.015",
            facecolor=GRAMMAR_BOX_COLORS[n["kind"]],
            edgecolor="#333333", linewidth=0.7, zorder=3,
            transform=ax.transAxes, clip_on=False)
        ax.add_patch(rect)
        ax.text(x, y, n["label"], ha="center", va="center",
                fontsize=6.5, fontweight="bold" if n["kind"] == "grammar" else "normal",
                transform=ax.transAxes, zorder=4, wrap=True)

    # Column labels
    ax.text(gx, 0.97, "Grammar States", ha="center", fontsize=8, fontweight="bold",
            transform=ax.transAxes, color="#333333")
    ax.text(mx, 0.97, "Candidate Physical Mechanisms", ha="center", fontsize=8, fontweight="bold",
            transform=ax.transAxes, color="#333333")

    for x in [gx, mx]:
        ax.plot([x, x], [0.02, 0.94], transform=ax.transAxes,
                color="#cccccc", lw=0.6, ls="--", zorder=0, clip_on=False)

    fig.tight_layout()
    save(fig, 9, "physics_failure_mapping", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 10 — DSA Score Build-Up Time Series
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig10(d, out_dir, dpi):
    ds = d["fig10_dsa"]
    k       = np.array(ds["k"])
    dsa     = np.array(ds["dsa_score"])
    tau     = ds["tau"]
    grammar = ds["grammar"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 3.8), sharex=True,
                                   gridspec_kw={"height_ratios": [3, 1]})

    # DSA composite score
    ax1.plot(k, dsa, color=C_DSFB, lw=1.4, label="DSA(k) composite")
    ax1.axhline(tau, color=C_VIOLATION, lw=1.0, ls="--",
                label=f"Threshold τ = {tau}")
    ax1.fill_between(k, dsa, tau, where=(dsa >= tau),
                     color=C_VIOLATION, alpha=0.20, label="DSA ≥ τ (Policy active)")
    ax1.fill_between(k, 0, dsa, where=(dsa < tau),
                     color=C_DSFB, alpha=0.10)
    ax1.set_ylabel("DSA Score")
    ax1.legend(fontsize=7.5)
    ax1.set_title("Fig. 10 — Deterministic Structural Accumulator (DSA) Score\n"
                  "DSA(k) = w₁·b + w₂·d + w₃·s + w₄·e + w₅·μ  →  Policy escalation")

    # Grammar timeline
    for i, g in enumerate(grammar):
        ax2.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Admissible")
    patch_B = mpatches.Patch(color=C_BOUNDARY, label="Boundary")
    patch_V = mpatches.Patch(color=C_VIOLATION, label="Violation")
    ax2.legend(handles=[patch_A, patch_B, patch_V], fontsize=7, ncol=3,
               loc="lower right")

    fig.tight_layout()
    save(fig, 10, "dsa_score_buildup", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 11 — Competitive Differentiation Matrix
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig11(d, out_dir, dpi):
    cm = d["fig11_competitive"]
    methods = cm["methods"]
    caps    = cm["capabilities"]
    mat     = np.array(cm["matrix"], dtype=float)   # rows=capabilities, cols=methods

    fig, ax = plt.subplots(figsize=(7.0, 3.8))

    # Color map: 0=white/red, 1=green, 2=partial/orange
    cell_colors = []
    for row in mat:
        color_row = []
        for v in row:
            if v == 1:
                color_row.append(C_ADMISSIBLE)
            elif v == 2:
                color_row.append(C_BOUNDARY)
            else:
                color_row.append("#eeeeee")
        cell_colors.append(color_row)

    n_cap, n_meth = mat.shape
    for i in range(n_cap):
        for j in range(n_meth):
            rect = mpatches.FancyBboxPatch(
                (j + 0.05, n_cap - 1 - i + 0.05), 0.90, 0.90,
                boxstyle="round,pad=0.02",
                facecolor=cell_colors[i][j], edgecolor="#888888",
                linewidth=0.5, zorder=2)
            ax.add_patch(rect)
            sym = "✓" if mat[i, j] == 1 else ("○" if mat[i, j] == 2 else "–")
            ax.text(j + 0.5, n_cap - 1 - i + 0.5, sym,
                    ha="center", va="center",
                    fontsize=10 if mat[i, j] > 0 else 9,
                    color="white" if mat[i, j] == 1 else
                          "#333333" if mat[i, j] == 2 else "#aaaaaa",
                    fontweight="bold" if mat[i, j] == 1 else "normal")

    ax.set_xticks([j + 0.5 for j in range(n_meth)])
    ax.set_xticklabels(methods, fontsize=7.5)
    ax.set_yticks([n_cap - 1 - i + 0.5 for i in range(n_cap)])
    ax.set_yticklabels(caps, fontsize=7.5)
    ax.set_xlim(0, n_meth)
    ax.set_ylim(0, n_cap)
    ax.grid(False)
    ax.set_title("Fig. 11 — Competitive Differentiation Matrix\n"
                 "✓=provided · ○=partial · –=absent   (DSFB=last column, rightmost)",
                 fontsize=8.5)

    # Highlight DSFB column
    rect_h = mpatches.FancyBboxPatch(
        (n_meth - 1, 0), 1, n_cap,
        boxstyle="round,pad=0.0",
        facecolor="none", edgecolor=C_DSFB,
        linewidth=2.0, zorder=5)
    ax.add_patch(rect_h)

    legend_elements = [
        mpatches.Patch(facecolor=C_ADMISSIBLE, label="✓ Provided"),
        mpatches.Patch(facecolor=C_BOUNDARY,   label="○ Partial"),
        mpatches.Patch(facecolor="#eeeeee",     edgecolor="#888888", label="– Absent"),
        Line2D([0],[0], color=C_DSFB, lw=2.0, label="DSFB (this work)"),
    ]
    ax.legend(handles=legend_elements, loc="lower right", fontsize=7)
    ax.axis("on")
    for spine in ax.spines.values():
        spine.set_linewidth(0.6)

    fig.tight_layout()
    save(fig, 11, "competitive_matrix", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 12 — WSS Pre-condition Verification
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig12(d, out_dir, dpi):
    wss = d["fig12_wss"]["scenarios"]
    n_sc = len(wss)

    fig, axes = plt.subplots(2, n_sc, figsize=(8.0, 4.0),
                              gridspec_kw={"height_ratios": [2, 1]})

    for j, sc in enumerate(wss):
        norms = np.array(sc["norms"])
        k     = np.arange(len(norms))
        col   = C_ADMISSIBLE if sc["is_wss"] else C_VIOLATION

        ax_main = axes[0, j] if n_sc > 1 else axes[0]
        ax_bar  = axes[1, j] if n_sc > 1 else axes[1]

        ax_main.plot(k, norms, color=col, lw=1.1)
        ax_main.set_title(sc["name"].replace("(", "\n("), fontsize=7.5)
        ax_main.set_ylabel("‖r(k)‖" if j == 0 else "")
        ax_main.set_xlabel("k")
        verdict = "WSS: PASS" if sc["is_wss"] else "WSS: FAIL"
        ax_main.text(0.97, 0.94, verdict, transform=ax_main.transAxes,
                     ha="right", va="top", fontsize=8, fontweight="bold",
                     color=col,
                     bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.8))

        # Metric bar
        metrics = ["Mean dev.", "Var. dev.", "|lag-1|"]
        vals    = [sc["mean_deviation"], sc["variance_deviation"], abs(sc["lag1_autocorr"])]
        thresholds = [0.20, 0.50, 0.70]
        bar_colors = [C_VIOLATION if v > t else C_ADMISSIBLE
                      for v, t in zip(vals, thresholds)]
        ax_bar.bar(metrics, vals, color=bar_colors, edgecolor="#333333", lw=0.5)
        for t_val, metric in zip(thresholds, metrics):
            x_pos = metrics.index(metric)
            ax_bar.hlines(t_val, x_pos - 0.4, x_pos + 0.4,
                          color="#333333", lw=1.0, ls="--")
        ax_bar.set_ylim(0, 1.05)
        ax_bar.set_ylabel("Metric" if j == 0 else "")
        ax_bar.tick_params(labelsize=6.5)

    fig.suptitle("Fig. 12 — WSS Pre-condition Verification (Wiener-Khinchin)\n"
                 "GUM requires stationary calibration window before ρ = μ+kσ is valid",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.91])
    save(fig, 12, "wss_verification", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 13 — Episode Precision-Recall Frontier
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig13(d, out_dir, dpi):
    pr = d["fig13_precision_recall"]["methods"]

    fig, ax = plt.subplots(figsize=(4.5, 3.5))

    for m in pr:
        p, r = m["precision"], m["recall"]
        ep   = m["episodes"]
        col  = m["color"]
        ms   = 90 if m["is_dsfb"] else 55
        mk   = "D" if m["is_dsfb"] else "o"
        ax.scatter(r, p, s=ms, c=col, marker=mk, zorder=4,
                   edgecolors="#333333" if m["is_dsfb"] else "none",
                   linewidths=0.8 if m["is_dsfb"] else 0)
        ax.annotate(f" {m['name']}\n({ep:,} ep.)",
                    (r, p), fontsize=6.5,
                    xytext=(5 if p < 0.5 else -5, 1),
                    textcoords="offset points",
                    va="bottom", ha="left" if p < 0.5 else "right")

    # Iso-F1 contours
    ps = np.linspace(0.01, 1.0, 200)
    for f1 in [0.01, 0.05, 0.10, 0.30, 0.50, 0.70]:
        rs = f1 * ps / (2 * ps - f1 + 1e-12)
        valid = (rs >= 0) & (rs <= 1.0)
        ax.plot(rs[valid], ps[valid], "--", color="#aaaaaa", lw=0.7, alpha=0.6)
        if np.any(valid):
            idx = np.argmin(np.abs(rs[valid] - 0.78))
            ax.text(rs[valid][idx], ps[valid][idx], f"F₁={f1:.2f}",
                    fontsize=5.5, color="#888888")

    ax.set_xlim(0.88, 1.005)
    ax.set_ylim(0.0, 1.05)
    ax.set_xlabel("Recall")
    ax.set_ylabel("Episode Precision")
    ax.legend(handles=[
        Line2D([0],[0], marker="D", color="white", markeredgecolor="#333333", ms=8, label="DSFB"),
        Line2D([0],[0], marker="o", color=C_NEUTRAL, ms=7, label="Comparators"),
    ], fontsize=7.5, loc="lower left")
    ax.set_title("Fig. 13 — Episode Precision–Recall Frontier\n"
                 "DSFB: 102.2× precision gain on RadioML, 76.8× on ORACLE")
    fig.tight_layout()
    save(fig, 13, "precision_recall_frontier", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 14 — Multi-Channel Corroboration (Lemma 6)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig14(d, out_dir, dpi):
    corr  = d["fig14_corroboration"]
    m_v   = np.array(corr["m_values"])
    f_ep  = np.array(corr["false_ep_rate"])

    fig, ax = plt.subplots(figsize=(4.5, 3.0))

    ax.semilogy(m_v, f_ep, "o-", color=C_DSFB, lw=1.6, ms=7, label="False ep. rate")
    for x, y in zip(m_v, f_ep):
        ax.annotate(f"{y:.3f}", (x, y), textcoords="offset points",
                    xytext=(4, 4), fontsize=7)

    ax.axhline(0.05, color=C_NEUTRAL, ls="--", lw=0.8, alpha=0.7,
               label="5% reference")
    ax.set_xlabel("Required corroboration count m")
    ax.set_ylabel("False episode rate P(X ≥ m)")
    ax.legend(fontsize=7.5)
    ax.set_title("Fig. 14 — Multi-Channel Corroboration (Lemma 6)\n"
                 "$P(X\\geq m) = \\binom{M}{m}p_f^m(1-p_f)^{M-m}$, M=6, $p_f$=0.046")
    ax.set_xticks(m_v)
    fig.tight_layout()
    save(fig, 14, "corroboration_lemma6", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 15 — Memory Footprint (no_std Bare-Metal)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig15(d, out_dir, dpi):
    mem = d["fig15_memory"]["modules"]
    names  = [m["module"] for m in mem]
    sizes  = np.array([m["bytes"] for m in mem])
    colors = [C_DSFB if "total" in m["module"].lower() else C_ADMISSIBLE for m in mem]

    fig, ax = plt.subplots(figsize=(5.5, 3.2))
    bars = ax.barh(range(len(names)), sizes, color=colors,
                   edgecolor="#333333", linewidth=0.7)
    ax.set_yticks(range(len(names)))
    ax.set_yticklabels(names, fontsize=8)
    ax.set_xlabel("Stack bytes (W=10, K=4, M=8)")
    ax.set_title("Fig. 15 — Memory Footprint: no_std / no_alloc / zero-unsafe\n"
                 "All structures stack-allocated — suitable for Cortex-M4F / RISC-V FPGA softcore")
    ax.invert_yaxis()

    for bar, sz in zip(bars, sizes):
        ax.text(sz + 5, bar.get_y() + bar.get_height() / 2,
                f"{sz} B", va="center", fontsize=8)

    # Reference line: 4 KB stack budget
    ax.axvline(4096, color=C_VIOLATION, ls="--", lw=1.0, label="4 KB stack budget")
    ax.axvline(1024, color=C_BOUNDARY,  ls=":", lw=0.8, label="1 KB reference")
    ax.legend(fontsize=7.5)
    fig.tight_layout()
    save(fig, 15, "memory_footprint", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 16 — Complexity Entropy Regime Transition
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig16(d, out_dir, dpi):
    cs = d["fig16_complexity"]
    k      = np.array(cs["k"])
    ent    = np.array(cs["entropy"])
    comp   = np.array(cs["complexity"])
    regime = cs["regime"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 3.8), sharex=True)

    ax1.plot(k, comp, color=C_DSFB, lw=1.3, label="Normalised complexity H/H_max")
    ax1.axhline(0.30, color=C_BOUNDARY, ls="--", lw=0.8, label="Low → Transitional (0.30)")
    ax1.axhline(0.70, color=C_VIOLATION, ls="--", lw=0.8, label="Transitional → High (0.70)")
    ax1.fill_between(k, 0.30, 0.70, alpha=0.08, color=C_BOUNDARY)
    ax1.fill_between(k, 0.70, 1.05, alpha=0.08, color=C_VIOLATION)
    ax1.set_ylim(0, 1.05)
    ax1.set_ylabel("Complexity score")
    ax1.legend(fontsize=7, loc="upper left")
    ax1.set_title("Fig. 16 — MDL/Kolmogorov Complexity (Shannon Entropy Estimator)\n"
                  "Rising complexity = nominal model losing descriptive power = Boundary approaching")

    # Regime timeline
    regime_map = {"LowComplexity": C_ADMISSIBLE,
                  "TransitionalComplexity": C_BOUNDARY,
                  "HighComplexity": C_VIOLATION}
    reg_colors = [regime_map.get(r, C_NEUTRAL) for r in regime]
    for i, c in enumerate(reg_colors):
        ax2.barh(0, 1, left=i, height=0.7, color=c, edgecolor="none")
    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Regime")
    ax2.set_xlim(0, len(k))
    patch_L = mpatches.Patch(color=C_ADMISSIBLE, label="LowComplexity")
    patch_T = mpatches.Patch(color=C_BOUNDARY,   label="TransitionalComplexity")
    patch_H = mpatches.Patch(color=C_VIOLATION,  label="HighComplexity")
    ax2.legend(handles=[patch_L, patch_T, patch_H], fontsize=7, ncol=3,
               loc="lower right")

    # Regime labels on ax1
    for label, start, end in [("Nominal\n(low H)", 0, 30),
                               ("Drift onset\n(rising H)", 30, 60),
                               ("High\ncomplexity", 60, 80),
                               ("Recovery", 80, len(k))]:
        ax1.text((start + end) / 2, 0.94, label,
                 ha="center", fontsize=6.5, color="#333333",
                 bbox=dict(boxstyle="round,pad=0.1", facecolor="white", alpha=0.7))

    fig.tight_layout()
    save(fig, 16, "complexity_entropy_regime", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 17 — Grammar FSM Hysteresis
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig17(d, out_dir, dpi):
    fsm    = d["fig17_fsm"]
    k      = np.array(fsm["k"])
    confs  = np.array(fsm["confirmations"])
    states = fsm["committed_state"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 3.4), sharex=True,
                                   gridspec_kw={"height_ratios": [1, 2]})

    # Severity bar
    ax1.step(k, confs, where="mid", color=C_DSFB, lw=1.3, label="Grammar severity (0-2)")
    ax1.set_ylim(-0.1, 2.4)
    ax1.set_yticks([0, 1, 2])
    ax1.set_yticklabels(["Admiss.", "Boundary", "Violation"], fontsize=7.5)
    ax1.set_ylabel("Severity")
    ax1.set_title("Fig. 17 — Grammar FSM Hysteresis Gate\n"
                  "2 consecutive confirmations required before state transition is committed")

    # State timeline with hysteresis annotations
    colors = [grammar_color(s) for s in states]
    for i, c in enumerate(colors):
        ax2.barh(0, 1, left=i, height=0.7, color=c, edgecolor="none")

    # Annotate the transient spike dismissal and the legitimate confirmation
    ax2.annotate("Transient spike\n(dismissed by\nhysteresis)",
                 xy=(4.5, 0.35), xytext=(7.5, 0.52),
                 arrowprops=dict(arrowstyle="->", color="#333333", lw=0.8),
                 fontsize=7, ha="center",
                 bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.8))
    ax2.annotate("Confirmed after\n2+ observations",
                 xy=(10.5, 0.35), xytext=(13.5, 0.52),
                 arrowprops=dict(arrowstyle="->", color=C_BOUNDARY, lw=0.8),
                 fontsize=7, ha="center",
                 bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.8))

    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Committed state")
    ax2.set_xlim(0, len(k))
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Admissible")
    patch_B = mpatches.Patch(color=C_BOUNDARY, label="Boundary")
    patch_V = mpatches.Patch(color=C_VIOLATION, label="Violation")
    ax2.legend(handles=[patch_A, patch_B, patch_V], fontsize=7, ncol=3)

    fig.tight_layout()
    save(fig, 17, "grammar_fsm_hysteresis", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 18 — Standards Alignment Matrix
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig18(d, out_dir, dpi):
    sm = d["fig18_standards"]
    stds    = sm["standards"]
    aspects = sm["aspects"]
    cov     = np.array(sm["coverage"])

    fig, ax = plt.subplots(figsize=(7.0, 3.8))

    n_std, n_asp = cov.shape
    for i in range(n_std):
        for j in range(n_asp):
            v = cov[i, j]
            col = C_ADMISSIBLE if v == 1 else "#f0f0f0"
            rect = mpatches.FancyBboxPatch(
                (j + 0.05, n_std - 1 - i + 0.05), 0.90, 0.90,
                boxstyle="round,pad=0.03",
                facecolor=col, edgecolor="#cccccc",
                linewidth=0.5, zorder=2)
            ax.add_patch(rect)
            sym = "✓" if v == 1 else "·"
            ax.text(j + 0.5, n_std - 1 - i + 0.5, sym,
                    ha="center", va="center",
                    fontsize=13 if v == 1 else 8,
                    color="white" if v == 1 else "#aaaaaa",
                    fontweight="bold" if v == 1 else "normal")

    ax.set_xticks([j + 0.5 for j in range(n_asp)])
    ax.set_xticklabels(aspects, fontsize=8)
    ax.set_yticks([n_std - 1 - i + 0.5 for i in range(n_std)])
    ax.set_yticklabels(stds, fontsize=8)
    ax.set_xlim(0, n_asp)
    ax.set_ylim(0, n_std)
    ax.grid(False)
    ax.set_title("Fig. 18 — Standards Alignment Coverage Matrix\n"
                 "ITU-R SM.1048-5 · MIL-STD-461G · 3GPP TS 36.141 · VITA 49.2 · SigMF · GUM · SOSA/MORA",
                 fontsize=8.5)
    for spine in ax.spines.values():
        spine.set_linewidth(0.6)

    patch_c = mpatches.Patch(color=C_ADMISSIBLE, label="✓ Covered (clause-depth)")
    patch_n = mpatches.Patch(color="#f0f0f0", edgecolor="#cccccc", label="· Not applicable")
    ax.legend(handles=[patch_c, patch_n], loc="lower right", fontsize=7.5)

    fig.tight_layout()
    save(fig, 18, "standards_alignment", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 19 — Architectural Integration (Read-Only Tap)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig19(d, out_dir, dpi):
    arch = d["fig19_architecture"]
    layers = arch["layers"]

    fig, ax = plt.subplots(figsize=(7.0, 3.6))
    ax.axis("off")
    ax.set_title("Fig. 19 — DSFB-RF Non-Intrusive Architecture\n"
                 "Read-only tap via immutable  &[f32]  — upstream receiver UNCHANGED",
                 fontsize=9)

    n_layers = len(layers)
    layer_h  = 0.18
    layer_gap = 0.04
    total_h  = n_layers * (layer_h + layer_gap)
    start_y  = (1.0 - total_h) / 2

    for i, layer in enumerate(layers):
        y = 1.0 - start_y - (i + 1) * (layer_h + layer_gap)
        x = 0.04
        w = 0.92
        rect = mpatches.FancyBboxPatch((x, y), w, layer_h,
            boxstyle="round,pad=0.01",
            facecolor=layer["color"], edgecolor="#444444",
            linewidth=1.1 + (0.5 if "DSFB" in layer["name"] else 0),
            transform=ax.transAxes, clip_on=False)
        ax.add_patch(rect)

        # Layer name
        ax.text(x + 0.01, y + layer_h / 2, layer["name"],
                transform=ax.transAxes, va="center", ha="left",
                fontsize=8, fontweight="bold", color="#111111")

        # Modules on the right
        mod_str = "  ·  ".join(layer["modules"])
        ax.text(x + w - 0.01, y + layer_h / 2, mod_str,
                transform=ax.transAxes, va="center", ha="right",
                fontsize=6.5, color="#333333")

        # Arrow between layers
        if i < n_layers - 1:
            arrow_x = 0.5
            arrow_y = y - layer_gap / 2
            if "DSFB" in layer["name"]:
                label = "immutable &[f32] tap\n(read-only · zero write path)"
                col   = C_ADMISSIBLE
            else:
                label = ""
                col   = "#888888"
            ax.annotate("", xy=(arrow_x, arrow_y - layer_gap * 0.1),
                        xytext=(arrow_x, arrow_y + layer_gap * 0.7),
                        xycoords="axes fraction",
                        arrowprops=dict(arrowstyle="->,head_width=0.20",
                                        color=col, lw=1.3),
                        annotation_clip=False)
            if label:
                ax.text(arrow_x + 0.02, arrow_y + layer_gap * 0.3, label,
                        transform=ax.transAxes, fontsize=7, color=col,
                        ha="left", va="center",
                        bbox=dict(boxstyle="round,pad=0.15",
                                  facecolor="white", alpha=0.85))

    # "Remove DSFB → receiver unaffected" callout
    ax.text(0.50, 0.02,
            "If DSFB is removed: upstream receiver behavior is UNCHANGED · "
            "Zero calibration · Zero restart · Zero risk",
            transform=ax.transAxes, ha="center", va="bottom", fontsize=7,
            fontstyle="italic", color="#333333",
            bbox=dict(boxstyle="round,pad=0.2", facecolor="#fffde7", alpha=0.9))

    fig.tight_layout()
    save(fig, 19, "architecture_integration", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 20 — Policy Escalation Logic
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig20(d, out_dir, dpi):
    pl = d["fig20_policy"]
    k       = np.array(pl["k"])
    dsa     = np.array(pl["dsa"])
    grammar = pl["grammar"]
    policy  = pl["policy"]
    tau     = pl["tau"]

    POLICY_COLOR = {
        "Silent":   C_ADMISSIBLE,
        "Watch":    "#aec7e8",
        "Review":   C_BOUNDARY,
        "Escalate": C_VIOLATION,
    }
    POLICY_LEVEL = {"Silent": 0, "Watch": 1, "Review": 2, "Escalate": 3}

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(5.5, 4.5), sharex=True,
                                         gridspec_kw={"height_ratios": [3, 1, 1]})

    # DSA trace
    ax1.plot(k, dsa, color=C_DSFB, lw=1.4, label="DSA(k)")
    ax1.axhline(tau, color=C_VIOLATION, ls="--", lw=1.0, label=f"τ = {tau}")
    ax1.fill_between(k, dsa, tau, where=(dsa >= tau),
                     color=C_VIOLATION, alpha=0.15)
    ax1.set_ylabel("DSA Score")
    ax1.legend(fontsize=7.5)
    ax1.set_title("Fig. 20 — Policy Escalation Logic\n"
                  "Silent → Watch → Review → Escalate (persistence K=4 + DSA ≥ τ + corroboration m=1)")

    # Grammar timeline
    for i, g in enumerate(grammar):
        ax2.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    ax2.set_yticks([])
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))

    # Policy timeline
    pol_levels = [POLICY_LEVEL.get(p, 0) for p in policy]
    pol_colors = [POLICY_COLOR.get(p, C_NEUTRAL) for p in policy]
    for i, (lv, pc) in enumerate(zip(pol_levels, pol_colors)):
        ax3.barh(0, 1, left=i, height=0.7, color=pc, edgecolor="none")

    # Annotate policy transitions
    prev = None
    for i, p in enumerate(policy):
        if p != prev and p != "Silent":
            ax3.text(i + 0.5, 0.5, p[0], ha="center", va="center",
                     fontsize=6, color="white", fontweight="bold")
        prev = p

    ax3.set_yticks([])
    ax3.set_xlabel("Observation k")
    ax3.set_ylabel("Policy")
    ax3.set_xlim(0, len(k))

    legend_elems = [
        mpatches.Patch(color=C_ADMISSIBLE, label="Silent"),
        mpatches.Patch(color="#aec7e8",    label="Watch"),
        mpatches.Patch(color=C_BOUNDARY,   label="Review"),
        mpatches.Patch(color=C_VIOLATION,  label="Escalate"),
    ]
    ax3.legend(handles=legend_elems, fontsize=7, ncol=4, loc="lower right")

    fig.tight_layout()
    save(fig, 20, "policy_escalation_logic", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Entry point
# ═══════════════════════════════════════════════════════════════════════════
FIGURES = {
    1:  plot_fig01,   2:  plot_fig02,   3:  plot_fig03,
    4:  plot_fig04,   5:  plot_fig05,   6:  plot_fig06,
    7:  plot_fig07,   8:  plot_fig08,   9:  plot_fig09,
    10: plot_fig10,   11: plot_fig11,   12: plot_fig12,
    13: plot_fig13,   14: plot_fig14,   15: plot_fig15,
    16: plot_fig16,   17: plot_fig17,   18: plot_fig18,
    19: plot_fig19,   20: plot_fig20,
}


def main():
    parser = argparse.ArgumentParser(description="DSFB-RF figure generator")
    parser.add_argument("--data",  default="figure_data.json",
                        help="Input JSON data file (default: figure_data.json)")
    parser.add_argument("--out",   default="figs",
                        help="Output directory (default: figs/)")
    parser.add_argument("--dpi",   type=int, default=150,
                        help="Figure DPI (default: 150; use 300 for print)")
    parser.add_argument("--fig",   type=int, nargs="*",
                        help="Specific figure numbers (default: all)")
    args = parser.parse_args()

    data_path = Path(args.data)
    if not data_path.exists():
        print(f"ERROR: {data_path} not found.")
        print("Run:  cargo run --features std,serde --example generate_figures")
        sys.exit(1)

    with open(data_path, "r") as fh:
        data = json.load(fh)

    out_dir = Path(args.out)
    selected = args.fig if args.fig else sorted(FIGURES.keys())

    print(f"Rendering {len(selected)} figures → {out_dir}/")
    print(f"DPI = {args.dpi}")
    print()

    for idx in selected:
        if idx not in FIGURES:
            print(f"  WARNING: fig {idx} not defined, skipping.")
            continue
        try:
            FIGURES[idx](data, out_dir, args.dpi)
        except Exception as e:
            print(f"  ERROR in fig {idx}: {e}")
            import traceback
            traceback.print_exc()

    print()
    print(f"Done. {len(selected)} figure(s) written to {out_dir}/")


if __name__ == "__main__":
    main()
