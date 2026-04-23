#!/usr/bin/env python3
"""
DSFB-RF Unified Publication Figure Generator (All 40 Figures)
==============================================================
Reads  dsfb-rf-output/figure_data_all.json  (produced by
  cargo run --features std,serde --example generate_figures_all)
and renders all 51 publication-quality figures for the paper.

Figures 1–20  : original Phase-1 engine results
Figures 21–40 : Phase-4 engine results (attractor, TDA, pragmatic,
                DNA fingerprinting, CRLB, Arrhenius, …)
Figures 41–51 : Phase-5/6 (Landauer, Fisher-Rao, relativistic Doppler,
                quantum noise, BFT swarm, RG flow / TDA)

Usage:
    python3 figures_all.py                      # all 51 figures
    python3 figures_all.py --fig 2 8 13 25 32   # specific figures only
    python3 figures_all.py --dpi 300            # print-resolution (default 150)

Output: dsfb-rf-output/figs/fig_XX_*.pdf  +  dsfb-rf-output/figs/fig_XX_*.png
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
    "Admissible":                         C_ADMISSIBLE,
    "Boundary(SustainedOutwardDrift)":    C_BOUNDARY,
    "Boundary(AbruptSlewViolation)":      C_BOUNDARY,
    "Boundary(RecurrentBoundaryGrazing)": C_BOUNDARY,
    "Violation":                          C_VIOLATION,
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
    pts   = d["fig01_semiotic_manifold"]["points"]
    norm  = np.array([p["norm"]  for p in pts], dtype=float)
    drift = np.array([p["drift"] for p in pts], dtype=float)
    slew  = np.array([p["slew"]  for p in pts], dtype=float)
    regime = [p["regime"] for p in pts]
    colors = [grammar_color(r) for r in regime]
    labels = sorted(set(regime), key=lambda x: ["Admissible","Boundary","Violation"].index(x)
                    if x in ["Admissible","Boundary","Violation"] else 3)

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
    ax.set_title("Fig. 1 — Semiotic Manifold $\\mathcal{M}_{\\mathrm{sem}}$\n"
                 "Grammar Partition in $(\\|r\\|, \\dot r, \\ddot r)$ space")
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

    cmap = matplotlib.colormaps["coolwarm"].resampled(len(curves))
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

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(7.2, 4.0), sharex=True)

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
    ax1.legend(fontsize=6.5, loc="upper left", bbox_to_anchor=(1.02, 1.0),
               frameon=True, framealpha=0.85, borderaxespad=0.0)
    ax1.set_title("Fig. 6 — Finite-Time Lyapunov Exponent vs. Grammar State\n"
                  "Healthy (λ<0) → slow drift (λ>0) → jamming onset (λ spike) → recovery")

    # Bottom: Grammar timeline
    for i, g in enumerate(grammar):
        ax2.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Admissible (λ<0)")
    patch_B = mpatches.Patch(color=C_BOUNDARY,   label="Boundary (λ>0)")
    patch_V = mpatches.Patch(color=C_VIOLATION,  label="Violation")
    ax2.legend(handles=[patch_A, patch_B, patch_V],
               loc="upper left", bbox_to_anchor=(1.02, 1.0),
               fontsize=7, ncol=1, frameon=True, framealpha=0.85,
               borderaxespad=0.0)
    ax2.set_yticks([])
    ax2.set_xlabel("Observation k")
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))

    # Regime labels (lowered to 78% of top to avoid overlapping the λ(k) trace)
    ymin, ymax = ax1.get_ylim()
    ylabel = ymin + 0.78 * (ymax - ymin)
    for label, start, end in [("Healthy\n(λ < 0)", 0, 30),
                               ("Thermal drift\n(λ > 0)", 30, 70),
                               ("Jamming\nonset", 70, 95),
                               ("Recovery", 95, len(k))]:
        ax1.text((start + end) / 2, ylabel, label,
                 ha="center", fontsize=6.5, color="#333333",
                 bbox=dict(boxstyle="round,pad=0.15", facecolor="white", alpha=0.8))

    fig.tight_layout(rect=[0, 0, 0.78, 1.0])
    save(fig, 6, "lyapunov_time_series", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 7 — GUM Uncertainty Budget Waterfall
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig07(d, out_dir, dpi):
    g = d["fig07_gum"]
    contribs = g["contributors"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.4, 3.2),
                                   gridspec_kw={"width_ratios": [2, 1]})

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
        ("$\\rho_{\\mathrm{GUM}} = \\mu + U$",          g["rho_gum"],    C_ADMISSIBLE),
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
    ax.text(np.log10(alp[0]) * 0.98, -14,
            "Sub-SNR floor:\nforced Admissible\n(L10 non-claim honored)",
            fontsize=6.5, color="white", ha="left", va="bottom",
            bbox=dict(boxstyle="round", facecolor="#333333", alpha=0.6))

    fig.tight_layout()
    save(fig, 8, "semiotic_horizon_heatmap", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 9 — Physics-of-Failure Mapping
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig09(d, out_dir, dpi):
    phys  = d["fig09_physics"]
    nodes = phys["nodes"]
    edges = phys["edges"]

    fig, ax = plt.subplots(figsize=(7.0, 3.6))
    ax.axis("off")
    ax.set_title("Fig. 9 — Physics-of-Failure Map: Grammar State → Candidate Physical Mechanism\n"
                 "(candidate hypotheses — physical attribution requires deployment calibration data)",
                 fontsize=8.5)

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
            sym = "+" if mat[i, j] == 1 else ("o" if mat[i, j] == 2 else "-")
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
                 "+=provided  o=partial  -=absent   (DSFB=last column, rightmost)",
                 fontsize=8.5)

    rect_h = mpatches.FancyBboxPatch(
        (n_meth - 1, 0), 1, n_cap,
        boxstyle="round,pad=0.0",
        facecolor="none", edgecolor=C_DSFB,
        linewidth=2.0, zorder=5)
    ax.add_patch(rect_h)

    legend_elements = [
        mpatches.Patch(facecolor=C_ADMISSIBLE, label="+ Provided"),
        mpatches.Patch(facecolor=C_BOUNDARY,   label="o Partial"),
        mpatches.Patch(facecolor="#eeeeee",     edgecolor="#888888", label="- Absent"),
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

    fig, axes = plt.subplots(2, n_sc, figsize=(9.5, 4.0),
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
    fig.subplots_adjust(top=0.88, left=0.07, right=0.98, wspace=0.38, hspace=0.45)
    save(fig, 12, "wss_verification", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 13 — Episode Precision-Recall Frontier
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig13(d, out_dir, dpi):
    pr = d["fig13_precision_recall"]["methods"]

    fig, ax = plt.subplots(figsize=(6.6, 4.0))

    for m in pr:
        p, r = m["precision"], m["recall"]
        ep   = m["episodes"]
        col  = m["color"]
        ms   = 90 if m["is_dsfb"] else 55
        mk   = "D" if m["is_dsfb"] else "o"
        ax.scatter(r, p, s=ms, c=col, marker=mk, zorder=4,
                   edgecolors="#333333" if m["is_dsfb"] else "none",
                   linewidths=0.8 if m["is_dsfb"] else 0)
        # Adaptive annotation placement: label on the LEFT of points
        # near the right axis, on the RIGHT otherwise.
        if r > 0.95:
            dx, ha = -7, "right"
        else:
            dx, ha = 7, "left"
        ax.annotate(f"{m['name']}\n({ep:,} ep.)",
                    (r, p), fontsize=7.0,
                    xytext=(dx, 4),
                    textcoords="offset points",
                    va="bottom", ha=ha)

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

    ax.set_xlim(0.0, 1.02)
    ax.set_ylim(0.0, 1.05)
    ax.set_xlabel("Recall")
    ax.set_ylabel("Episode Precision")
    ax.legend(handles=[
        Line2D([0],[0], marker="D", color="white", markeredgecolor="#333333", ms=8, label="DSFB"),
        Line2D([0],[0], marker="o", color=C_NEUTRAL, ms=7, label="Comparators"),
    ], fontsize=7.5, loc="lower left")
    ax.set_title("Fig. 13 — Episode Precision–Recall Frontier\n"
                 "DSFB: 102.2× precision gain on RadioML, 76.8× on ORACLE")
    fig.tight_layout(pad=0.8)
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

    ax1.step(k, confs, where="mid", color=C_DSFB, lw=1.3, label="Grammar severity (0-2)")
    ax1.set_ylim(-0.1, 2.4)
    ax1.set_yticks([0, 1, 2])
    ax1.set_yticklabels(["Admiss.", "Boundary", "Violation"], fontsize=7.5)
    ax1.set_ylabel("Severity")
    ax1.set_title("Fig. 17 — Grammar FSM Hysteresis Gate\n"
                  "2 consecutive confirmations required before state transition is committed")

    colors = [grammar_color(s) for s in states]
    for i, c in enumerate(colors):
        ax2.barh(0, 1, left=i, height=0.7, color=c, edgecolor="none")

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
            sym = "+" if v == 1 else "·"
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

    patch_c = mpatches.Patch(color=C_ADMISSIBLE, label="+ Covered (clause-depth)")
    patch_n = mpatches.Patch(facecolor="#f0f0f0", edgecolor="#cccccc", label="· Not applicable")
    ax.legend(handles=[patch_c, patch_n], loc="lower right", fontsize=7.5)

    fig.tight_layout()
    save(fig, 18, "standards_alignment", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 19 — Architectural Integration (Read-Only Tap)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig19(d, out_dir, dpi):
    arch   = d["fig19_architecture"]
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

        ax.text(x + 0.01, y + layer_h / 2, layer["name"],
                transform=ax.transAxes, va="center", ha="left",
                fontsize=8, fontweight="bold", color="#111111")

        mod_str = "  ·  ".join(layer["modules"])
        ax.text(x + w - 0.01, y + layer_h / 2, mod_str,
                transform=ax.transAxes, va="center", ha="right",
                fontsize=6.5, color="#333333")

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

    ax1.plot(k, dsa, color=C_DSFB, lw=1.4, label="DSA(k)")
    ax1.axhline(tau, color=C_VIOLATION, ls="--", lw=1.0, label=f"τ = {tau}")
    ax1.fill_between(k, dsa, tau, where=(dsa >= tau),
                     color=C_VIOLATION, alpha=0.15)
    ax1.set_ylabel("DSA Score")
    ax1.legend(fontsize=7.5)
    ax1.set_title("Fig. 20 — Policy Escalation Logic\n"
                  "Silent → Watch → Review → Escalate "
                  "(persistence K=4 + DSA ≥ τ + corroboration m=1)")

    for i, g in enumerate(grammar):
        ax2.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    ax2.set_yticks([])
    ax2.set_ylabel("Grammar")
    ax2.set_xlim(0, len(k))

    pol_levels = [POLICY_LEVEL.get(p, 0) for p in policy]
    pol_colors = [POLICY_COLOR.get(p, C_NEUTRAL) for p in policy]
    for i, (lv, pc) in enumerate(zip(pol_levels, pol_colors)):
        ax3.barh(0, 1, left=i, height=0.7, color=pc, edgecolor="none")

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
# Fig 21 — Permutation Entropy vs Shannon Entropy
# Dataset anchor: IQEngine ORACLE USRP B200
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig21(d, out_dir, dpi):
    pe = d["fig21_perm_entropy"]
    k           = np.array(pe["k"])
    pe_wss      = np.array(pe["pe_wss"])
    pe_periodic = np.array(pe["pe_periodic"])
    pe_drifting = np.array(pe["pe_drifting"])
    sh_wss      = np.array(pe["sh_wss"])
    sh_periodic = np.array(pe["sh_periodic"])
    sh_drifting = np.array(pe["sh_drifting"])
    regime_per  = pe["regime_periodic"]
    W           = pe["window_w"]

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(6.0, 5.2), sharex=True,
                                         gridspec_kw={"height_ratios": [2, 2, 1]})

    # Top: PE curves
    ax1.plot(k, pe_wss,      color=C_ADMISSIBLE, lw=1.3, label="PE — WSS noise")
    ax1.plot(k, pe_periodic, color=C_DSFB,       lw=1.3, label="PE — periodic tone")
    ax1.plot(k, pe_drifting, color=C_VIOLATION,  lw=1.3, label="PE — drifting carrier")
    ax1.axhline(0.70, color=C_BOUNDARY, lw=0.8, ls="--", label="PE threshold (0.70)")
    ax1.set_ylim(0, 1.05)
    ax1.set_ylabel(f"Norm. PE  (W={W})")
    ax1.legend(fontsize=6.5, ncol=2)
    ax1.set_title("Fig. 21 — Permutation Entropy vs. Shannon Entropy\n"
                  "IQEngine ORACLE USRP B200 · m=3 ordinal patterns")

    # Middle: Shannon H curves
    ax2.plot(k, sh_wss,      color=C_ADMISSIBLE, lw=1.1, ls="--", label="Shannon H — WSS")
    ax2.plot(k, sh_periodic, color=C_DSFB,       lw=1.1, ls="--", label="Shannon H — periodic")
    ax2.plot(k, sh_drifting, color=C_VIOLATION,  lw=1.1, ls="--", label="Shannon H — drifting")
    ax2.set_ylim(0, 1.05)
    ax2.set_ylabel("Norm. Shannon H")
    ax2.legend(fontsize=6.5, ncol=2)

    # Bottom: Grammar timeline for periodic stream
    for i, g in enumerate(regime_per):
        ax3.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    ax3.set_yticks([])
    ax3.set_xlabel("Window index k")
    ax3.set_ylabel("Regime")
    ax3.set_xlim(0, len(k))
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Low PE (ordered)")
    patch_B = mpatches.Patch(color=C_BOUNDARY,   label="Boundary")
    patch_V = mpatches.Patch(color=C_VIOLATION,  label="High PE (random)")
    ax3.legend(handles=[patch_A, patch_B, patch_V], fontsize=6.5, ncol=3)

    fig.tight_layout()
    save(fig, 21, "perm_entropy_comparison", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 22 — Reverse Arrangements Test (trend detection)
# Dataset anchor: NIST POWDER-RENEW
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig22(d, out_dir, dpi):
    rat = d["fig22_rat"]
    windows = rat["windows"]
    n = len(windows)

    fig, axes = plt.subplots(2, n, figsize=(8.0, 4.2),
                              gridspec_kw={"height_ratios": [3, 1]})

    for j, w in enumerate(windows):
        norms = np.array(w["norms"])
        k     = np.arange(len(norms))
        col   = C_VIOLATION if w["has_trend"] else C_ADMISSIBLE
        ax_top = axes[0, j]
        ax_bot = axes[1, j]

        ax_top.plot(k, norms, color=col, lw=1.1)
        verdict = f"|Z|={abs(w['z_score']):.2f}\n{'TREND' if w['has_trend'] else 'WSS'}"
        ax_top.text(0.96, 0.93, verdict, transform=ax_top.transAxes,
                    ha="right", va="top", fontsize=7.5, fontweight="bold", color=col,
                    bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.85))
        ax_top.set_title(w["name"].replace(" ", "\n", 1), fontsize=7.5)
        ax_top.set_ylabel("‖r(k)‖" if j == 0 else "")

        # Z-score bar
        z = w["z_score"]
        bar_col = C_VIOLATION if abs(z) > 1.96 else C_ADMISSIBLE
        ax_bot.bar(["Z"], [abs(z)], color=bar_col, edgecolor="#333333", lw=0.5)
        ax_bot.axhline(1.96, color=C_BOUNDARY, lw=0.8, ls="--")
        ax_bot.axhline(2.576, color=C_VIOLATION, lw=0.7, ls=":")
        ax_bot.set_ylim(0, max(3.0, abs(z) * 1.1))
        ax_bot.tick_params(labelsize=6.5)
        ax_bot.set_ylabel("|Z|" if j == 0 else "")

    fig.suptitle("Fig. 22 — Reverse Arrangements Test: Monotonic Trend Detection\n"
                 "NIST POWDER-RENEW calibration windows · |Z|>1.96 → trend (95%CI) · |Z|>2.576 → strict (99%)",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.90])
    save(fig, 22, "rat_trend_detection", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 23 — CRLB Floor vs ρ across SNR
# Dataset anchor: NIST POWDER-RENEW USRP X310
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig23(d, out_dir, dpi):
    cs  = d["fig23_crlb_sweep"]
    pts = cs["points"]
    snr  = np.array([p["snr_db"]        for p in pts])
    rho_floor = np.array([p["rho_floor"]   for p in pts])
    margin    = np.array([p["margin"]      for p in pts])
    alerts    = [p["alert"] for p in pts]
    rho_test  = cs["rho_test"]
    n_obs     = cs["n_obs"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 4.0), sharex=True)

    ax1.plot(snr, rho_floor, color=C_DSFB, lw=1.5, label="ρ_CRLB floor")
    ax1.axhline(rho_test, color=C_VIOLATION, lw=1.1, ls="--",
                label=f"ρ_test = {rho_test:.3f}")
    ax1.fill_between(snr, rho_floor, rho_test,
                     where=(rho_floor > rho_test),
                     color=C_VIOLATION, alpha=0.15, label="CRLB violates ρ_test")
    ax1.fill_between(snr, rho_floor, rho_test,
                     where=(rho_floor <= rho_test),
                     color=C_ADMISSIBLE, alpha=0.10, label="CRLB safely below ρ_test")
    ax1.set_ylabel("ρ_CRLB floor")
    ax1.legend(fontsize=7, ncol=2)
    ax1.set_title(f"Fig. 23 — CRLB Envelope Floor vs. SNR\n"
                  f"NIST POWDER-RENEW · N={n_obs} obs · ρ_test={rho_test:.3f}")

    # Margin panel
    ax2.bar(snr, margin, width=np.diff(snr).mean() * 0.8,
            color=[C_ADMISSIBLE if m > 0 else C_VIOLATION for m in margin],
            edgecolor="#333333", linewidth=0.4)
    ax2.axhline(0, color="#333333", lw=0.8)
    ax2.set_ylabel("Margin (ρ_test − ρ_floor)")
    ax2.set_xlabel("SNR (dB)")

    # Mark alert points
    for snr_v, al in zip(snr, alerts):
        if al:
            ax2.annotate("!", (snr_v, 0), xytext=(0, -12),
                         textcoords="offset points", ha="center",
                         fontsize=9, color=C_VIOLATION)

    fig.tight_layout()
    save(fig, 23, "crlb_snr_sweep", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 24 — Arrhenius PA Drift Curves
# Dataset anchor: Kayali 1999 JPL-96-25  (GaAs pHEMT / GaN HEMT)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig24(d, out_dir, dpi):
    arr  = d["fig24_arrhenius"]
    T    = np.array(arr["temperatures_c"])
    dg   = np.array(arr["drift_gaas"])
    dn   = np.array(arr["drift_gan"])
    afg  = np.array(arr["af_gaas"])
    afn  = np.array(arr["af_gan"])
    ocxo_tau1 = arr["avar_ocxo_tau1"]
    tcxo_tau1 = arr["avar_tcxo_tau1"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.2, 3.2))

    # Left: drift rate vs temperature
    ax1.semilogy(T, dg, "o-", color=C_DSFB,       lw=1.4, ms=5, label="GaAs pHEMT (E_a=1.6 eV)")
    ax1.semilogy(T, dn, "s-", color=C_ADMISSIBLE,  lw=1.4, ms=5, label="GaN HEMT   (E_a=1.8 eV)")
    ax1.axhline(ocxo_tau1, color=C_NEUTRAL,   lw=0.8, ls=":", label=f"OCXO σ_y(τ=1) = {ocxo_tau1:.2e}")
    ax1.axhline(tcxo_tau1, color=C_HIGHLIGHT, lw=0.8, ls=":", label=f"TCXO σ_y(τ=1) = {tcxo_tau1:.2e}")
    ax1.set_xlabel("Junction temperature (°C)")
    ax1.set_ylabel("Normalised drift rate (a.u.)")
    ax1.legend(fontsize=6.5)
    ax1.set_title("Drift rate vs. T_junction\n(Arrhenius E_a model)")

    # Right: AF (acceleration factor) vs temperature relative to 25 °C
    T_ref_idx = np.argmin(np.abs(T - 25.0))
    ax2.plot(T, afg, "o-", color=C_DSFB,      lw=1.4, ms=5, label="GaAs pHEMT AF")
    ax2.plot(T, afn, "s-", color=C_ADMISSIBLE, lw=1.4, ms=5, label="GaN HEMT AF")
    ax2.axhline(1.0, color="#333333", lw=0.7, ls="--", alpha=0.6, label="AF = 1 (25 °C)")
    ax2.set_xlabel("Junction temperature (°C)")
    ax2.set_ylabel("Acceleration factor AF")
    ax2.legend(fontsize=6.5)
    ax2.set_title("Thermal acceleration factor\n(ref. 25 °C)")

    fig.suptitle("Fig. 24 — Arrhenius Physics-of-Failure: PA Thermal Drift\n"
                 "Kayali 1999 JPL-96-25 model · GaAs pHEMT vs. GaN HEMT",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.87])
    save(fig, 24, "arrhenius_pa_drift", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 25 — Delay-Embedding Phase Portraits
# Dataset anchor: DARPA SC2 / Colosseum
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig25(d, out_dir, dpi):
    pp  = d["fig25_phase_portraits"]
    tau = pp["tau"]
    scenarios = pp["scenarios"]
    n = len(scenarios)

    fig, axes = plt.subplots(1, n, figsize=(3.2 * n, 3.4))
    if n == 1:
        axes = [axes]

    cmap_list = [C_ADMISSIBLE, C_DSFB, C_VIOLATION]
    for ax, sc, col in zip(axes, scenarios, cmap_list):
        x  = np.array(sc["x_now"])
        xd = np.array(sc["x_delayed"])
        ax.scatter(xd, x, s=6, alpha=0.55, color=col, edgecolors="none")
        ax.set_xlabel(f"‖r(k-{tau})‖", fontsize=8)
        ax.set_ylabel("‖r(k)‖", fontsize=8)
        state = sc["attractor_state"].split("::")[-1]
        ax.set_title(f"{sc['label']}\nD₂≈{sc['d2_estimate']:.2f} · {state}", fontsize=7.5)
        # mark centroid
        ax.plot(np.mean(xd), np.mean(x), "x", color="#333333", ms=8, mew=1.5)

    fig.suptitle(f"Fig. 25 — Phase Portraits (τ={tau}): DARPA SC2 / Colosseum\n"
                 "Delay-embedding reconstructs attractor geometry from ‖r(k)‖",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.88])
    save(fig, 25, "phase_portraits", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 26 — Grassberger-Procaccia Correlation Dimension
# Dataset anchor: IQEngine ORACLE USRP B200
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig26(d, out_dir, dpi):
    gp = d["fig26_gp_d2"]
    curves = gp["curves"]

    fig, ax = plt.subplots(figsize=(5.2, 3.6))
    palette = [C_ADMISSIBLE, C_DSFB, C_VIOLATION, C_BOUNDARY, C_COMPARATOR, C_NEUTRAL]

    for i, cv in enumerate(curves):
        lr = np.array(cv["log_r"])
        lc = np.array(cv["log_cr"])
        col = palette[i % len(palette)]
        ax.plot(lr, lc, lw=1.3, color=col, label=f"{cv['label']}  (D₂≈{cv['d2']:.2f})")
        # Annotate slope
        mid = len(lr) // 2
        slope = cv["d2"]
        ax.annotate(f"slope={slope:.2f}",
                    (lr[mid], lc[mid]),
                    xytext=(6, 0), textcoords="offset points",
                    fontsize=6, color=col)

    ax.set_xlabel("log r")
    ax.set_ylabel("log C(r)")
    ax.legend(fontsize=7, loc="upper left")
    ax.set_title("Fig. 26 — Grassberger-Procaccia Correlation Dimension D₂\n"
                 "IQEngine ORACLE USRP B200 · slope = D₂ in scaling region")
    fig.tight_layout()
    save(fig, 26, "gp_correlation_dimension", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 27 — TDA Persistence Diagram
# Dataset anchor: NIST POWDER-RENEW urban interference
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig27(d, out_dir, dpi):
    tda = d["fig27_tda_persistence"]
    r   = tda["radius_used"]
    en  = tda["events_noise"]
    ec  = tda["events_cluster"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(6.8, 3.4))

    def _plot_pd(ax, events, title, betti0, innov):
        births = np.array([e["birth"] for e in events])
        deaths = np.array([e["death"] for e in events])
        pers   = deaths - births
        ax.scatter(births, deaths, s=30, color=C_DSFB, alpha=0.75,
                   edgecolors="#333333", linewidths=0.4, zorder=3)
        # Diagonal (birth == death)
        lo, hi = 0.0, max(deaths.max(), births.max()) * 1.05 if len(deaths) else 1.0
        ax.plot([lo, hi], [lo, hi], color="#aaaaaa", lw=0.7, ls="--")
        ax.set_xlabel("Birth radius")
        ax.set_ylabel("Death radius")
        ax.set_title(f"{title}\nβ₀={betti0} · innov={innov:.4f}")
        ax.text(0.96, 0.06, f"r_used={r:.4f}", transform=ax.transAxes,
                ha="right", fontsize=6.5, color=C_NEUTRAL)

    _plot_pd(ax1, en, "WSS noise (H₀ persistence)",
             tda["betti0_noise"], tda["innovation_noise"])
    _plot_pd(ax2, ec, "2-cluster jammer onset (H₀ persistence)",
             tda["betti0_cluster"], tda["innovation_cluster"])

    fig.suptitle("Fig. 27 — TDA Vietoris-Rips Persistence Diagram (H₀)\n"
                 "NIST POWDER-RENEW · birth/death of connected components",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.88])
    save(fig, 27, "tda_persistence_diagram", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 28 — Betti₀ vs Filtration Radius
# Dataset anchor: DARPA SC2 / Colosseum interference environments
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig28(d, out_dir, dpi):
    bs = d["fig28_betti0_sweep"]
    radii   = np.array(bs["radii"])
    b0_wgn  = np.array(bs["betti0_wgn"])
    b0_fhss = np.array(bs["betti0_fhss"])
    b0_jam  = np.array(bs["betti0_jammer"])
    n_pts   = bs["n_points"]

    fig, ax = plt.subplots(figsize=(5.5, 3.4))
    ax.plot(radii, b0_wgn,  color=C_ADMISSIBLE, lw=1.4, label="WGN (uniform cluster)")
    ax.plot(radii, b0_fhss, color=C_DSFB,       lw=1.4, label="FHSS interference")
    ax.plot(radii, b0_jam,  color=C_VIOLATION,  lw=1.4, label="Jammer onset (2-cluster)")
    ax.set_xlabel("Filtration radius r")
    ax.set_ylabel("β₀ (connected components)")
    ax.set_yscale("log")
    ax.legend(fontsize=7.5)
    ax.set_title(f"Fig. 28 — Betti₀ vs. Filtration Radius\n"
                 f"DARPA SC2 Colosseum · N={n_pts} points per class")
    ax.text(0.97, 0.97,
            "Plateau height reflects cluster structure;\n"
            "jammer class shows later merging → larger β₀",
            transform=ax.transAxes, ha="right", va="top",
            fontsize=6.5, color="#333333",
            bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.8))
    fig.tight_layout()
    save(fig, 28, "betti0_sweep", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 29 — Pragmatic Gate SOSA Efficiency
# Dataset anchor: DARPA SC2 / SOSA backplane
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig29(d, out_dir, dpi):
    pg = d["fig29_pragmatic_gate"]
    k      = np.array(pg["k"])
    ent    = np.array(pg["entropy"])
    emits  = np.array(pg["emit_flags"])
    cumeff = np.array(pg["cumulative_efficiency_pct"])
    eff_adm = pg["admissible_efficiency_pct"]
    sc_k    = pg["state_change_k"]

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(5.5, 4.8), sharex=True,
                                         gridspec_kw={"height_ratios": [2, 1, 2]})

    # Entropy
    ax1.plot(k, ent, color=C_DSFB, lw=1.3, label="PE entropy H(k)")
    ax1.axvline(sc_k, color=C_VIOLATION, lw=1.0, ls="--",
                label=f"State-change emit (k={sc_k})")
    ax1.set_ylabel("Entropy")
    ax1.legend(fontsize=7)
    ax1.set_title("Fig. 29 — Pragmatic Gate: SOSA Event-Centric Efficiency\n"
                  "DARPA SC2 Colosseum · gate emits only at state changes")

    # Emit flags
    ax2.fill_between(k, 0, emits.astype(float),
                     step="mid", color=C_BOUNDARY, alpha=0.65, label="Emit")
    ax2.set_yticks([0, 1])
    ax2.set_yticklabels(["suppress", "emit"], fontsize=7)
    ax2.set_ylabel("Gate")

    # Cumulative efficiency
    ax3.plot(k, cumeff, color=C_ADMISSIBLE, lw=1.5, label="Cumulative suppression %")
    ax3.axhline(eff_adm, color=C_NEUTRAL, lw=0.8, ls=":",
                label=f"Admissible-phase avg = {eff_adm:.1f}%")
    ax3.set_ylim(0, 100)
    ax3.set_ylabel("Suppress. %")
    ax3.set_xlabel("Observation k")
    ax3.legend(fontsize=7)

    fig.tight_layout()
    save(fig, 29, "pragmatic_gate_efficiency", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 30 — Hardware DNA Allan Variance Fingerprints
# Dataset anchor: IQEngine hardware diversity (RTL-SDR → USRP X310)
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig30(d, out_dir, dpi):
    dna = d["fig30_dna_fingerprints"]
    taus        = np.array(dna["taus"])
    avar_ocxo   = np.array(dna["avar_ocxo"])
    avar_tcxo   = np.array(dna["avar_tcxo"])
    avar_mems   = np.array(dna["avar_mems"])
    avar_spoof  = np.array(dna["avar_spoofed"])
    sim_mat     = np.array(dna["sim_matrix"])
    auth_thr    = dna["auth_threshold"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.4, 3.4))

    # Left: ADEV curves
    ax1.loglog(taus, np.sqrt(avar_ocxo),  color=C_ADMISSIBLE, lw=1.3, label="OCXO")
    ax1.loglog(taus, np.sqrt(avar_tcxo),  color=C_DSFB,       lw=1.3, label="TCXO")
    ax1.loglog(taus, np.sqrt(avar_mems),  color=C_VIOLATION,  lw=1.3, label="MEMS")
    ax1.loglog(taus, np.sqrt(avar_spoof), color=C_HIGHLIGHT,  lw=1.2, ls="--",
               label="Spoofed (simulated clone)")
    ax1.set_xlabel("Averaging time τ (samples)")
    ax1.set_ylabel("Allan deviation σ_y(τ)")
    ax1.legend(fontsize=7)
    ax1.set_title("ADEV fingerprints\n(IQEngine hardware classes)")

    # Right: cosine similarity heatmap
    labels_hw = ["OCXO", "TCXO", "MEMS", "Spoofed"]
    n = len(labels_hw)
    cmap_sim = LinearSegmentedColormap.from_list(
        "sim", [(0.0, C_VIOLATION), (auth_thr, "#ffdd88"), (1.0, C_ADMISSIBLE)])
    im = ax2.imshow(sim_mat, vmin=0.0, vmax=1.0, cmap=cmap_sim, aspect="auto")
    ax2.set_xticks(range(n))
    ax2.set_yticks(range(n))
    ax2.set_xticklabels(labels_hw, fontsize=8)
    ax2.set_yticklabels(labels_hw, fontsize=8)
    for i in range(n):
        for j in range(n):
            ax2.text(j, i, f"{sim_mat[i, j]:.2f}", ha="center", va="center",
                     fontsize=8, color="white" if sim_mat[i, j] < 0.6 else "#111111")
    fig.colorbar(im, ax=ax2, fraction=0.046, pad=0.04).set_label(
        f"Cosine similarity (auth τ={auth_thr:.2f})", fontsize=7)
    ax2.set_title(f"Similarity matrix\n(auth threshold={auth_thr:.2f})")

    fig.suptitle("Fig. 30 — Hardware DNA: Allan Variance Fingerprinting\n"
                 "IQEngine RTL-SDR → USRP X310 · OCXO/TCXO/MEMS/Spoofed",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.88])
    save(fig, 30, "dna_fingerprints", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 31 — CRLB Margin vs N Observations
# Dataset anchor: NIST POWDER-RENEW
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig31(d, out_dir, dpi):
    cm = d["fig31_crlb_margin"]
    curves    = cm["curves"]
    rho_test  = cm["rho_test"]
    alert_thr = cm["alert_threshold"]

    fig, ax = plt.subplots(figsize=(5.2, 3.5))
    palette = [C_ADMISSIBLE, C_DSFB, C_VIOLATION]

    for cv, col in zip(curves, palette):
        n_vals  = np.array(cv["n_vals"])
        margins = np.array(cv["margins"])
        ax.plot(n_vals, margins, "o-", color=col, lw=1.3, ms=5,
                label=f"SNR = {cv['snr_db']:+.0f} dB")
        # shade alert zone
        ax.fill_between(n_vals, margins, alert_thr,
                        where=(margins < alert_thr),
                        color=col, alpha=0.08)

    ax.axhline(alert_thr, color="#333333", lw=0.9, ls="--",
               label=f"Alert threshold = {alert_thr:.4f}")
    ax.axhline(0.0, color="#888888", lw=0.6, ls=":")
    ax.set_xscale("log")
    ax.set_xlabel("N (calibration window observations)")
    ax.set_ylabel("Margin  (ρ_test − ρ_CRLB floor)")
    ax.legend(fontsize=7.5)
    ax.set_title(f"Fig. 31 — CRLB Margin vs. Calibration Window Size N\n"
                 f"NIST POWDER-RENEW · ρ_test={rho_test:.3f} · alert if margin < {alert_thr:.4f}")
    fig.tight_layout()
    save(fig, 31, "crlb_margin_vs_n", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 32 — Koopman Mode Proxy VM-Ratio Time Series
# Dataset anchor: DARPA SC2 interference mode classes
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig32(d, out_dir, dpi):
    kd = d["fig32_koopman_proxy"]
    scenarios = kd["scenarios"]
    W         = kd["window_w"]
    n = len(scenarios)

    fig, axes = plt.subplots(n, 1, figsize=(6.0, 2.5 * n), sharex=False)
    if n == 1:
        axes = [axes]
    palette = [C_ADMISSIBLE, C_DSFB, C_VIOLATION]

    for ax, sc, col in zip(axes, scenarios, palette):
        k       = np.array(sc["k"])
        vm      = np.array(sc["vm_ratio"])
        norms   = np.array(sc["norms"])
        states  = sc["attractor_states"]

        ax2 = ax.twinx()
        ax2.plot(k, norms, color="#cccccc", lw=0.9, zorder=1)
        ax2.set_ylabel("‖r(k)‖", fontsize=7, color="#999999")
        ax2.tick_params(axis="y", labelsize=6.5, colors="#999999")

        ax.plot(k, vm, color=col, lw=1.3, zorder=3,
                label=f"VM ratio  (mean={sc['mean_vm']:.3f})")
        ax.axhline(sc["mean_vm"], color=col, lw=0.7, ls="--", alpha=0.6)
        ax.set_ylabel("VM ratio (Koopman proxy)")
        ax.set_xlabel("Window index")
        ax.legend(fontsize=7, loc="upper left")
        ax.set_title(f"{sc['label']}  (W={W})", fontsize=8)

    fig.suptitle("Fig. 32 — Koopman-Mode Proxy: VM Ratio × 3 SC2 Modes\n"
                 "DARPA SC2 Colosseum · VM = var(delayed)/var(norms) → attractor structure",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save(fig, 32, "koopman_vm_ratio", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 33 — Bit-Exactness Q16.16 vs f32
# Dataset anchor: IQEngine hardware diversity
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig33(d, out_dir, dpi):
    be = d["fig33_bit_exactness"]
    k         = np.array(be["k"])
    nf        = np.array(be["norms_f32"])
    nq        = np.array(be["norms_q16"])
    err       = np.array(be["abs_errors"])
    bound     = be["bound"]
    pct_ok    = be["pct_below_bound"]
    agree     = np.array(be["grammar_agree"])

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(5.5, 4.8), sharex=True,
                                         gridspec_kw={"height_ratios": [2, 2, 1]})

    # Norm traces
    ax1.plot(k, nf, color=C_DSFB,       lw=1.3, label="f32 path")
    ax1.plot(k, nq, color=C_VIOLATION,  lw=1.0, ls="--", label="Q16.16 path")
    ax1.set_ylabel("‖r(k)‖")
    ax1.legend(fontsize=7.5)
    ax1.set_title("Fig. 33 — Bit-Exactness: f32 vs. Q16.16 Fixed-Point\n"
                  "IQEngine ORACLE USRP B200 · compiler-enforced determinism")

    # Absolute error (log)
    ax2.semilogy(k, err, color=C_BOUNDARY, lw=1.1, label="|error|")
    ax2.axhline(bound, color=C_VIOLATION, lw=0.9, ls="--",
                label=f"Bound 2⁻¹⁴ = {bound:.2e}")
    ax2.set_ylabel("|f32 − Q16.16|")
    ax2.legend(fontsize=7)
    ax2.text(0.97, 0.95, f"{pct_ok*100:.1f}% samples below bound",
             transform=ax2.transAxes, ha="right", va="top", fontsize=7.5,
             color=C_ADMISSIBLE, fontweight="bold",
             bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.85))

    # Grammar agreement bar
    ax3.fill_between(k, 0, agree.astype(float), step="mid",
                     color=C_ADMISSIBLE, alpha=0.7, label="Agree")
    ax3.fill_between(k, agree.astype(float), 1, step="mid",
                     color=C_VIOLATION, alpha=0.6, label="Disagree")
    ax3.set_yticks([0, 1])
    ax3.set_yticklabels(["disagree", "agree"], fontsize=7)
    ax3.set_xlabel("Observation k")
    ax3.set_ylabel("Grammar")
    ax3.legend(fontsize=6.5, ncol=2, loc="lower right")

    fig.tight_layout()
    save(fig, 33, "bit_exactness_q16", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 34 — Allan Deviation Oscillator Classes
# Dataset anchor: IQEngine hardware characterisation
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig34(d, out_dir, dpi):
    ad = d["fig34_allan_deviation"]
    taus       = np.array(ad["taus"])
    avar_ocxo  = np.array(ad["avar_ocxo"])
    avar_tcxo  = np.array(ad["avar_tcxo"])
    avar_mems  = np.array(ad["avar_mems"])

    fig, ax = plt.subplots(figsize=(5.0, 3.4))
    ax.loglog(taus, np.sqrt(avar_ocxo), color=C_ADMISSIBLE, lw=1.5,
              label="OCXO (Oven-Controlled)")
    ax.loglog(taus, np.sqrt(avar_tcxo), color=C_DSFB,       lw=1.5,
              label="TCXO (Temp-Compensated)")
    ax.loglog(taus, np.sqrt(avar_mems), color=C_VIOLATION,  lw=1.5,
              label="MEMS (low-cost)")

    # Annotate dominant noise regions for OCXO
    t_mid = taus[len(taus) // 2]
    ax.annotate("White FM\n(slope −½)", xy=(taus[2], np.sqrt(avar_ocxo[2])),
                xytext=(taus[2] * 0.5, np.sqrt(avar_ocxo[2]) * 4),
                fontsize=6.5, color=C_ADMISSIBLE,
                arrowprops=dict(arrowstyle="->", lw=0.7, color=C_ADMISSIBLE))

    ax.set_xlabel("Averaging time τ (samples)")
    ax.set_ylabel("Allan deviation σ_y(τ)")
    ax.legend(fontsize=7.5, loc="lower left")
    ax.set_title("Fig. 34 — Allan Deviation: Oscillator Class Discrimination\n"
                 "IQEngine hardware characterisation · OCXO / TCXO / MEMS noise floors")
    ax.grid(True, which="both", alpha=0.3)
    fig.tight_layout()
    save(fig, 34, "allan_deviation_classes", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 35 — PE on Cyclostationary Jammer
# Dataset anchor: DARPA SC2 Colosseum
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig35(d, out_dir, dpi):
    pc = d["fig35_pe_cyclostationary"]
    k           = np.array(pc["k"])
    norms_wgn   = np.array(pc["norms_wgn"])
    norms_jam   = np.array(pc["norms_jammer"])
    pe_wgn      = np.array(pc["pe_wgn"])
    pe_jam      = np.array(pc["pe_jammer"])
    regime_jam  = pc["regime_jammer"]
    onset_k     = pc["jammer_onset_k"]
    det_k       = pc["detection_k"]
    lead        = pc["pe_lead_samples"]

    fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(5.5, 5.0), sharex=True,
                                         gridspec_kw={"height_ratios": [2, 2, 1]})

    # Norm traces
    ax1.plot(k, norms_wgn, color=C_ADMISSIBLE, lw=1.1, label="WGN baseline")
    ax1.plot(k, norms_jam, color=C_VIOLATION,  lw=1.2, label="Cyclostationary jammer")
    ax1.axvline(onset_k, color=C_BOUNDARY, lw=1.0, ls="--", label=f"Jammer on (k={onset_k})")
    ax1.set_ylabel("‖r(k)‖")
    ax1.legend(fontsize=7)
    ax1.set_title("Fig. 35 — PE on Cyclostationary Jammer: DARPA SC2 Colosseum\n"
                  "PE detects periodicity reduction before grammar Boundary transition")

    # PE traces
    ax2.plot(k, pe_wgn, color=C_ADMISSIBLE, lw=1.1, label="PE — WGN")
    ax2.plot(k, pe_jam, color=C_VIOLATION,  lw=1.3, label="PE — jammer")
    ax2.axhline(0.70, color=C_BOUNDARY, lw=0.8, ls="--", label="PE threshold (0.70)")
    if det_k is not None:
        ax2.axvline(det_k, color=C_VIOLATION, lw=0.9, ls=":",
                    label=f"PE detect k={det_k} (lead={lead} samp.)")
    ax2.set_ylim(0, 1.05)
    ax2.set_ylabel("Norm. PE")
    ax2.legend(fontsize=6.5, ncol=2)

    # Grammar timeline
    for i, g in enumerate(regime_jam):
        ax3.barh(0, 1, left=i, height=0.7, color=grammar_color(g), edgecolor="none")
    ax3.set_yticks([])
    ax3.set_xlabel("Window index k")
    ax3.set_ylabel("Regime")
    ax3.set_xlim(0, len(k))
    patch_A = mpatches.Patch(color=C_ADMISSIBLE, label="Low PE")
    patch_B = mpatches.Patch(color=C_BOUNDARY,   label="Boundary")
    patch_V = mpatches.Patch(color=C_VIOLATION,  label="High PE")
    ax3.legend(handles=[patch_A, patch_B, patch_V], fontsize=6.5, ncol=3)

    fig.tight_layout()
    save(fig, 35, "pe_cyclostationary", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 36 — SOSA Backplane Event-Centric vs Naive
# Dataset anchor: DARPA SC2 / SOSA
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig36(d, out_dir, dpi):
    bp = d["fig36_backplane"]
    k           = np.array(bp["k"])
    naive       = np.array(bp["naive_cumsum"])
    pragmatic   = np.array(bp["pragmatic_cumsum"])
    savings_pct = np.array(bp["savings_pct"])
    trans_ks    = bp["transition_ks"]
    final_sav   = bp["final_savings_pct"]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(5.5, 4.0), sharex=True,
                                   gridspec_kw={"height_ratios": [2, 1]})

    ax1.plot(k, naive,     color=C_VIOLATION,  lw=1.4, label="Naive (every sample)")
    ax1.plot(k, pragmatic, color=C_ADMISSIBLE, lw=1.4, label="PragmaticGate (event-centric)")
    for tk in trans_ks:
        ax1.axvline(tk, color=C_BOUNDARY, lw=0.7, ls="--", alpha=0.7)
    ax1.set_ylabel("Cumulative messages")
    ax1.legend(fontsize=7.5)
    ax1.set_title("Fig. 36 — SOSA Backplane Load: Naive vs. Event-Centric\n"
                  "DARPA SC2 · PragmaticGate emits only at state transitions")
    ax1.text(0.97, 0.06, f"Final saving: {final_sav:.1f}%",
             transform=ax1.transAxes, ha="right", va="bottom",
             fontsize=8, fontweight="bold", color=C_ADMISSIBLE,
             bbox=dict(boxstyle="round,pad=0.2", facecolor="white", alpha=0.85))

    ax2.fill_between(k, savings_pct, color=C_ADMISSIBLE, alpha=0.55,
                     label="Savings %")
    ax2.set_ylim(0, 100)
    ax2.set_ylabel("Savings %")
    ax2.set_xlabel("Observation k")
    ax2.legend(fontsize=7)

    fig.tight_layout()
    save(fig, 36, "sosa_backplane_savings", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 37 — Hardware DNA Authentication: Genuine vs Spoofed
# Dataset anchor: IQEngine TCXO Grade B
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig37(d, out_dir, dpi):
    da = d["fig37_dna_auth"]
    taus         = np.array(da["taus"])
    reg          = np.array(da["registered_avar"])
    genuine_sims = da["genuine_sims"]
    spoofed_sims = da["spoofed_sims"]
    thr          = da["auth_threshold"]
    gpr          = da["genuine_pass_rate"]
    srr          = da["spoof_reject_rate"]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.4, 3.4))

    # Left: registered ADEV + individual trial variability
    ax1.loglog(taus, np.sqrt(reg), color=C_DSFB, lw=2.0, zorder=3,
               label="Registered ADEV template")
    ax1.set_xlabel("τ (samples)")
    ax1.set_ylabel("σ_y(τ)")
    ax1.set_title("Registered TCXO template\n(IQEngine ORACLE USRP B200)")
    ax1.legend(fontsize=7.5)

    # Right: distribution of cosine similarities
    g_arr = np.array(genuine_sims)
    s_arr = np.array(spoofed_sims)
    bins  = np.linspace(0.0, 1.0, 25)
    ax2.hist(g_arr, bins=bins, color=C_ADMISSIBLE, alpha=0.70,
             label=f"Genuine  (n={len(g_arr)})")
    ax2.hist(s_arr, bins=bins, color=C_VIOLATION,  alpha=0.70,
             label=f"Spoofed  (n={len(s_arr)})")
    ax2.axvline(thr, color="#333333", lw=1.2, ls="--",
                label=f"Auth threshold = {thr:.2f}")
    ax2.set_xlabel("Cosine similarity to template")
    ax2.set_ylabel("Trial count")
    ax2.legend(fontsize=7)
    ax2.text(0.97, 0.97,
             f"Genuine pass:  {gpr*100:.1f}%\nSpoof reject: {srr*100:.1f}%",
             transform=ax2.transAxes, ha="right", va="top",
             fontsize=7.5, fontweight="bold",
             bbox=dict(boxstyle="round,pad=0.25", facecolor="white", alpha=0.88))
    ax2.set_title("Authentication distribution\n(40 genuine + 40 spoofed trials)")

    fig.suptitle("Fig. 37 — Hardware DNA Authentication: Genuine vs. Spoofed\n"
                 "IQEngine ORACLE · TCXO Grade B registered template · cosine-similarity threshold",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.88])
    save(fig, 37, "dna_auth_genuine_spoof", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 39 — Multi-Mode Attractor Reconstruction
# Dataset anchors: IQEngine / DARPA SC2 / NIST POWDER-RENEW
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig39(d, out_dir, dpi):
    ma = d["fig39_multi_attractor"]
    tau       = ma["tau"]
    scenarios = ma["scenarios"]
    n = len(scenarios)

    fig, axes = plt.subplots(1, n, figsize=(3.2 * n, 3.4))
    if n == 1:
        axes = [axes]
    palette = [C_ADMISSIBLE, C_DSFB, C_VIOLATION]

    for ax, sc, col in zip(axes, scenarios, palette):
        x  = np.array(sc["x_now"])
        xd = np.array(sc["x_delayed"])
        ax.scatter(xd, x, s=7, alpha=0.55, color=col, edgecolors="none")
        ax.set_xlabel(f"‖r(k-{tau})‖", fontsize=8)
        ax.set_ylabel("‖r(k)‖", fontsize=8)
        state = sc["attractor_state"].split("::")[-1]
        ax.set_title(f"{sc['label']}\n{sc['dataset_ref'][:28]}\n"
                     f"D₂≈{sc['d2_estimate']:.2f} · {state}", fontsize=7)
        ax.plot(np.mean(xd), np.mean(x), "x", color="#333333", ms=8, mew=1.5)

    fig.suptitle(f"Fig. 39 — Multi-Mode Attractor Reconstruction (τ={tau})\n"
                 "IQEngine · DARPA SC2 · NIST POWDER-RENEW — three distinct attractor geometries",
                 fontsize=8.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.86])
    save(fig, 39, "multi_attractor", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Fig 40 — Capability Radar Chart
# Provenance: scores derived from engine benchmarks documented in paper §Eval
# ═══════════════════════════════════════════════════════════════════════════
def plot_fig40(d, out_dir, dpi):
    cr   = d["fig40_capability_radar"]
    axes_data = cr["axes"]
    prov = cr["provenance"]
    n    = len(axes_data)

    labels      = [a["label"]        for a in axes_data]
    dsfb_vals   = [a["dsfb_score"]   for a in axes_data]
    typical_vals= [a["typical_score"]for a in axes_data]
    ml_vals     = [a["ml_score"]     for a in axes_data]

    # Close the radar loop
    angles = [2 * math.pi * i / n for i in range(n)]
    angles += angles[:1]
    dsfb_v   = dsfb_vals   + dsfb_vals[:1]
    typical_v= typical_vals+ typical_vals[:1]
    ml_v     = ml_vals     + ml_vals[:1]

    fig, ax = plt.subplots(figsize=(4.8, 4.8), subplot_kw=dict(polar=True))
    ax.set_theta_offset(math.pi / 2)
    ax.set_theta_direction(-1)

    ax.plot(angles, dsfb_v,    color=C_DSFB,       lw=2.0, label="DSFB-RF (this work)")
    ax.fill(angles, dsfb_v,    color=C_DSFB,       alpha=0.18)
    ax.plot(angles, typical_v, color=C_NEUTRAL,    lw=1.4, ls="--", label="Typical SDR monitor")
    ax.fill(angles, typical_v, color=C_NEUTRAL,    alpha=0.08)
    ax.plot(angles, ml_v,      color=C_COMPARATOR, lw=1.4, ls=":",  label="ML/DL classifier")
    ax.fill(angles, ml_v,      color=C_COMPARATOR, alpha=0.08)

    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(labels, size=7.5)
    ax.set_ylim(0, 1.0)
    ax.set_yticks([0.25, 0.50, 0.75, 1.0])
    ax.set_yticklabels(["0.25", "0.50", "0.75", "1.0"], size=6)

    ax.legend(loc="upper right", bbox_to_anchor=(1.28, 1.12), fontsize=7.5)
    ax.set_title("Fig. 40 — Capability Radar\n(scores relative to axis definitions — see §Eval)",
                 pad=18, fontsize=8.5)

    # Provenance footnote
    fig.text(0.5, 0.01, prov, ha="center", va="bottom", fontsize=6,
             fontstyle="italic", color="#555555",
             wrap=True)

    fig.tight_layout()
    save(fig, 40, "capability_radar", out_dir, dpi)


# ── Phase-5: Calibration sensitivity, TRL, SBIR ───────────────────────────

def plot_fig41(d, out_dir, dpi):
    """Fig 41 – rho perturbation sweep: precision & recall vs rho scale."""
    import matplotlib.pyplot as plt
    import numpy as np

    sweep = d["fig41_rho_sweep"]
    cells = sweep["cells"]
    nom_idx = sweep["nominal_idx"]

    rho   = [c["rho_scale"]  for c in cells]
    prec  = [c["precision"]  for c in cells]
    rec   = [c["recall"]     for c in cells]
    frate = [c["false_rate"] for c in cells]

    fig, ax1 = plt.subplots(figsize=(7, 4))
    ax2 = ax1.twinx()

    ax1.plot(rho, prec,  "b-o", markersize=5, linewidth=1.8, label="Precision", zorder=3)
    ax2.plot(rho, rec,   "r--s", markersize=5, linewidth=1.8, label="Recall",    zorder=3)
    ax1.plot(rho, frate, "g:^", markersize=4, linewidth=1.4, label="False Rate", zorder=3)

    ax1.axvline(rho[nom_idx], color="gray", linestyle="--", linewidth=1.2,
                label="Nominal (Table IV)", zorder=2)
    ax1.scatter([rho[nom_idx]], [prec[nom_idx]],  s=70, color="blue",  zorder=5)
    ax2.scatter([rho[nom_idx]], [rec[nom_idx]],   s=70, color="red",   zorder=5)

    ax1.set_xlabel(r"$\rho$ scale (relative to $\rho_\mathrm{nom}$)", fontsize=10)
    ax1.set_ylabel("Precision / False Rate", color="blue", fontsize=10)
    ax2.set_ylabel("Recall", color="red", fontsize=10)
    ax1.set_title(
        r"Fig 41 – $\rho$ Perturbation Sweep $\pm$15\% (Stage III RadioML 2018.01a)"
        "\nAnchored: 73.6\\% prec / 95.1\\% recall at $\\rho_{\\mathrm{nom}}$",
        fontsize=9,
    )
    ax1.tick_params(axis="y", labelcolor="blue")
    ax2.tick_params(axis="y", labelcolor="red")
    ax1.set_ylim(0, 1.0)
    ax2.set_ylim(0, 1.0)

    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax1.legend(lines1 + lines2, labels1 + labels2, fontsize=8, loc="lower left")

    ax1.text(0.98, 0.04,
             "Model estimates; only $\\rho_{\\mathrm{nom}}$ (s=1.0) is measured.",
             transform=ax1.transAxes, fontsize=7, ha="right", color="gray")

    fig.tight_layout()
    save(fig, 41, "rho_sweep", out_dir, dpi)


def plot_fig42(d, out_dir, dpi):
    """Fig 42 – W_pred x W_obs calibration grid heatmap (precision)."""
    import matplotlib.pyplot as plt
    import numpy as np

    grid_data = d["fig42_wpred_grid"]
    cells = grid_data["cells"]
    nom_w_obs  = grid_data["nominal_w_obs"]
    nom_w_pred = grid_data["nominal_w_pred"]

    w_obs_vals  = sorted(set(c["w_obs"]  for c in cells))
    w_pred_vals = sorted(set(c["w_pred"] for c in cells))

    prec_grid = np.zeros((len(w_obs_vals), len(w_pred_vals)))
    for c in cells:
        i = w_obs_vals.index(c["w_obs"])
        j = w_pred_vals.index(c["w_pred"])
        prec_grid[i, j] = c["precision"]

    fig, ax = plt.subplots(figsize=(6, 4))
    im = ax.imshow(prec_grid, aspect="auto", origin="lower",
                   cmap="viridis", vmin=0.5, vmax=1.0)
    plt.colorbar(im, ax=ax, label="Episode Precision")

    ax.set_xticks(range(len(w_pred_vals)))
    ax.set_xticklabels([str(v) for v in w_pred_vals])
    ax.set_yticks(range(len(w_obs_vals)))
    ax.set_yticklabels([str(v) for v in w_obs_vals])
    ax.set_xlabel("Prediction Horizon $W_{\\mathrm{pred}}$ (obs)", fontsize=10)
    ax.set_ylabel("Observation Window $W$ (obs)", fontsize=10)
    ax.set_title(
        "Fig 42 – $W_{\\mathrm{pred}} \\times W$ Calibration Grid\n"
        "(Deferred Table XIV; RadioML 2018.01a)", fontsize=9,
    )

    # Highlight nominal cell
    if nom_w_obs in w_obs_vals and nom_w_pred in w_pred_vals:
        ni = w_obs_vals.index(nom_w_obs)
        nj = w_pred_vals.index(nom_w_pred)
        ax.add_patch(plt.Rectangle((nj - 0.5, ni - 0.5), 1, 1,
                                   fill=False, edgecolor="white",
                                   linewidth=2.5, label="Nominal"))
        ax.legend(fontsize=8, loc="upper right")

    # Annotate cells
    for c in cells:
        i = w_obs_vals.index(c["w_obs"])
        j = w_pred_vals.index(c["w_pred"])
        ax.text(j, i, f"{c['precision']:.2f}", ha="center", va="center",
                fontsize=8, color="white" if c["precision"] < 0.75 else "black")

    fig.tight_layout()
    save(fig, 42, "wpred_grid", out_dir, dpi)


def plot_fig43(d, out_dir, dpi):
    """Fig 43 – W x K x tau configuration landscape (3 subplots, one per tau)."""
    import matplotlib.pyplot as plt
    import numpy as np

    grid_data = d["fig43_config_grid"]
    cells = grid_data["cells"]
    nom_idx  = grid_data["nominal_idx"]
    best_idx = grid_data["best_idx"]

    tau_vals = sorted(set(c["tau"]     for c in cells))
    w_vals   = sorted(set(c["w"]       for c in cells))
    k_vals   = sorted(set(c["k"]       for c in cells))

    fig, axes = plt.subplots(1, len(tau_vals), figsize=(10, 3.5), sharey=True)
    fig.suptitle(
        "Fig 43 – $W \\times K \\times \\tau$ Configuration Landscape\n"
        "(Colour = $F$-score = Precision $\\times$ Recall; RadioML 2018.01a)",
        fontsize=9,
    )

    for ax, tau in zip(axes, tau_vals):
        fscore_grid = np.zeros((len(k_vals), len(w_vals)))
        for c in cells:
            if abs(c["tau"] - tau) < 0.01:
                ki = k_vals.index(c["k"])
                wi = w_vals.index(c["w"])
                fscore_grid[ki, wi] = c["f_score"]

        im = ax.imshow(fscore_grid, aspect="auto", origin="lower",
                       cmap="plasma", vmin=0.5, vmax=1.0)
        ax.set_xticks(range(len(w_vals)))
        ax.set_xticklabels([str(v) for v in w_vals], fontsize=8)
        ax.set_yticks(range(len(k_vals)))
        ax.set_yticklabels([str(v) for v in k_vals], fontsize=8)
        ax.set_xlabel("$W$ (obs)", fontsize=9)
        ax.set_title(f"$\\tau$ = {tau}", fontsize=9)

        # Annotate values
        for ki, k_v in enumerate(k_vals):
            for wi, w_v in enumerate(w_vals):
                fs = fscore_grid[ki, wi]
                ax.text(wi, ki, f"{fs:.2f}", ha="center", va="center",
                        fontsize=7, color="white" if fs < 0.7 else "black")

        # Mark nominal and best
        nom_c  = cells[nom_idx]
        best_c = cells[best_idx]
        for cell, color, marker in [(nom_c, "cyan", "o"), (best_c, "yellow", "*")]:
            if abs(cell["tau"] - tau) < 0.01:
                ki = k_vals.index(cell["k"])
                wi = w_vals.index(cell["w"])
                ax.plot(wi, ki, marker=marker, color=color, markersize=10,
                        markeredgecolor="black", markeredgewidth=0.8)

    axes[0].set_ylabel("$K$ (threshold)", fontsize=9)
    # Shared colorbar
    fig.subplots_adjust(right=0.88, wspace=0.15)
    cbar_ax = fig.add_axes([0.90, 0.15, 0.02, 0.65])
    fig.colorbar(im, cax=cbar_ax, label="F-score")

    save(fig, 43, "config_grid", out_dir, dpi)


def plot_fig44(d, out_dir, dpi):
    """Fig 44 – TRL staircase chart."""
    import matplotlib.pyplot as plt
    import numpy as np

    trl = d["fig44_trl_staircase"]
    comps = trl["components"]
    sys_trl = trl["system_trl"]

    names     = [c["name"] for c in comps]
    current   = [c["trl_current"] for c in comps]
    target    = [c["trl_target"]  for c in comps]

    y = np.arange(len(names))
    fig, ax = plt.subplots(figsize=(9, 5))

    bars_cur = ax.barh(y, current, height=0.4, color="#2166ac",
                       label="Current TRL (Phase I validated)", align="center")
    bars_gap = ax.barh(y, [t - c for t, c in zip(target, current)],
                       left=current, height=0.4, color="#d1e5f0",
                       edgecolor="#2166ac", linewidth=0.8,
                       label="Target TRL (Phase II)", align="center")

    ax.set_yticks(y)
    ax.set_yticklabels(names, fontsize=7.5)
    ax.set_xlabel("Technology Readiness Level (TRL)", fontsize=10)
    ax.set_xticks(range(1, 10))
    ax.set_xlim(0, 9.5)
    ax.axvline(sys_trl, color="#d6604d", linestyle="--", linewidth=1.5,
               label=f"System TRL = {sys_trl} (conservative)")
    ax.axvline(4, color="gray", linestyle=":", linewidth=1.0, alpha=0.7,
               label="Phase I target: TRL 4")
    ax.axvline(6, color="green", linestyle=":", linewidth=1.0, alpha=0.7,
               label="Phase II target: TRL 6")

    for bar in bars_cur:
        w = bar.get_width()
        if w > 0:
            ax.text(w - 0.1, bar.get_y() + bar.get_height() / 2,
                    str(int(w)), ha="right", va="center", fontsize=8,
                    color="white", fontweight="bold")

    ax.set_title(
        "Fig 44 – Component TRL Staircase (Table X, de Beer 2026)\n"
        "Solid: achieved; Hatch: target. Conservative system-level TRL claim.",
        fontsize=9,
    )
    ax.legend(fontsize=8, loc="lower right")
    fig.tight_layout()
    save(fig, 44, "trl_staircase", out_dir, dpi)


def plot_fig45(d, out_dir, dpi):
    """Fig 45 – Phase I SBIR deliverable Gantt chart."""
    import matplotlib.pyplot as plt
    import numpy as np

    timeline = d["fig45_sbir_deliverables"]
    delivs   = timeline["deliverables"]
    milestones = timeline["milestones"]
    total_months = timeline["total_months"]

    fig, ax = plt.subplots(figsize=(9, 4.5))

    colors = ["#4393c3", "#74add1", "#abd9e9", "#e0f3f8", "#fee090"]
    y_pos  = list(range(len(delivs)))

    for i, deliv in enumerate(delivs):
        start = deliv["month_start"] - 1
        dur   = deliv["month_end"] - deliv["month_start"] + 1
        bar = ax.barh(i, dur, left=start, height=0.6,
                      color=colors[i % len(colors)],
                      edgecolor="white", linewidth=0.8, align="center")
        ax.text(start + dur / 2, i, f"D{deliv['id']}",
                ha="center", va="center", fontsize=9, fontweight="bold",
                color="#1a1a2e")

    # Milestone markers
    ms_months = [1, 3, 6]
    ms_labels  = ["M1", "M3", "M6"]
    for ms_m, ms_l in zip(ms_months, ms_labels):
        ax.axvline(ms_m - 0.5, color="#d6604d", linestyle="--",
                   linewidth=1.2, alpha=0.8)
        ax.text(ms_m - 0.5, len(delivs) - 0.1, ms_l,
                ha="center", va="bottom", fontsize=8, color="#d6604d",
                fontweight="bold")

    ax.set_yticks(y_pos)
    ax.set_yticklabels([f"D{d_['id']}: {d_['title'][:50]}" for d_ in delivs],
                       fontsize=7.5)
    ax.set_xlabel("Month (Phase I base period)", fontsize=10)
    ax.set_xticks(range(total_months + 1))
    ax.set_xticklabels([str(m) for m in range(total_months + 1)])
    ax.set_xlim(-0.3, total_months + 0.3)
    ax.set_title(
        "Fig 45 – Phase I SBIR Deliverable Timeline (de Beer 2026, §20)\n"
        "D1–D5: six-month base period. M1/M3/M6: Go/No-go decision points.",
        fontsize=9,
    )
    ax.invert_yaxis()
    fig.tight_layout()
    save(fig, 45, "sbir_timeline", out_dir, dpi)


# ═══════════════════════════════════════════════════════════════════════════
# Phase-6 figures (fig46 – fig51)
# ═══════════════════════════════════════════════════════════════════════════

# ── Fig 46 – Landauer Thermodynamic Audit ─────────────────────────────────
def plot_fig46(d, out_dir, dpi):
    """Fig 46 – Landauer structural entropy cost vs observation level."""
    audit = d["fig46_landauer_audit"]
    entries = audit["entries"]

    CLASS_COLORS = {
        "SubThermal":     "#1f77b4",
        "Thermal":        "#2ca02c",
        "MildBurden":     "#ff7f0e",
        "ModerateBurden": "#d62728",
        "SevereBurden":   "#9467bd",
    }

    obs   = [e["obs_sigma_sq"]  for e in entries]
    nats  = [e["excess_nats"]   for e in entries]
    ejoul = [e["energy_joules"] for e in entries]
    pwatt = [e["power_watts"]   for e in entries]
    clss  = [e["class_label"]   for e in entries]
    idx   = [e["idx"]           for e in entries]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4))

    # Left: excess_nats vs obs_sigma_sq (log-log)
    for cls, col in CLASS_COLORS.items():
        xs = [obs[i]  for i in range(len(entries)) if clss[i] == cls]
        ys = [nats[i] for i in range(len(entries)) if clss[i] == cls]
        if xs:
            ax1.scatter(xs, ys, c=col, s=40, label=cls, zorder=3)
    ax1.set_xscale("log")
    ax1.set_yscale("log")
    ax1.set_xlabel(r"$\sigma^2_\mathrm{obs}$ (W)", fontsize=9)
    ax1.set_ylabel("Excess entropy $\\Delta H$ (nats)", fontsize=9)
    ax1.set_title("Structural entropy burden vs observation power", fontsize=9)
    ax1.legend(fontsize=7, loc="upper left")
    ax1.axhline(0.1, color="gray", linestyle=":", linewidth=0.8, label="MildBurden threshold")
    ax1.axhline(1.0, color="gray", linestyle="--", linewidth=0.8, label="ModerateBurden threshold")

    # Right: energy and power vs window index, colored by class
    colors = [CLASS_COLORS.get(c, C_NEUTRAL) for c in clss]
    ax2r = ax2.twinx()
    ax2.bar(idx, ejoul, color=colors, alpha=0.7, width=0.7, zorder=2)
    ax2r.plot(idx, pwatt, "k--", linewidth=1.2, label="Power (W)", zorder=3)
    ax2.set_xlabel("Window index", fontsize=9)
    ax2.set_ylabel("Erasure energy (J)", fontsize=9)
    ax2r.set_ylabel("Structural power (W)", fontsize=9)
    ax2.set_title(
        f"Erasure energy per window  "
        f"[cumul.={audit['cumulative_energy']:.2e} J]",
        fontsize=9)
    ax2r.legend(fontsize=7, loc="upper left")

    # Legend patches for class colors
    patches = [mpatches.Patch(color=v, label=k) for k, v in CLASS_COLORS.items()]
    ax2.legend(handles=patches, fontsize=7, loc="upper left")

    fig.suptitle(
        "Fig 46 – Landauer Thermodynamic Audit\n"
        r"$E_\mathrm{erase} = k_B T \Delta H_\mathrm{struct}$"
        "  (Landauer 1961, §thermodynamics)",
        fontsize=9)
    fig.tight_layout()
    save(fig, 46, "landauer_audit", out_dir, dpi)


# ── Fig 47 – Fisher-Rao Geodesic Drift Path ───────────────────────────────
def plot_fig47(d, out_dir, dpi):
    """Fig 47 – Fisher-Rao manifold drift: geodesic path in mu-sigma space."""
    drift = d["fig47_fisher_rao_drift"]
    steps = drift["steps"]

    CLASS_COLORS = {
        "Linear":      C_ADMISSIBLE,
        "Settling":    C_BOUNDARY,
        "NonLinear":   "#ff7f0e",
        "Oscillatory": C_VIOLATION,
    }

    mus   = [0.0] + [s["mu"]                for s in steps]
    sigs  = [0.05] + [s["sigma"]            for s in steps]
    frd   = [s["fr_distance"]               for s in steps]
    cum   = [s["cumulative_length"]         for s in steps]
    cls   = ["Linear"] + [s["drift_class_label"] for s in steps]
    idxs  = list(range(len(steps)))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4.5))

    # Left: μ-σ manifold path
    for i in range(1, len(mus)):
        c = CLASS_COLORS.get(cls[i], C_NEUTRAL)
        ax1.annotate("", xy=(mus[i], sigs[i]), xytext=(mus[i-1], sigs[i-1]),
                     arrowprops=dict(arrowstyle="->", color=c, lw=1.2))
    sc = ax1.scatter(mus, sigs, c=[CLASS_COLORS.get(c, C_NEUTRAL) for c in cls],
                     s=30, zorder=3)
    ax1.scatter([mus[0]], [sigs[0]], s=80, marker="*", c="gold", zorder=5,
                label="Start")
    ax1.scatter([mus[-1]], [sigs[-1]], s=80, marker="X", c="black", zorder=5,
                label="End")

    legend_handles = [
        mpatches.Patch(color=v, label=k) for k, v in CLASS_COLORS.items()
    ] + [
        plt.Line2D([0], [0], marker="*", color="w", markerfacecolor="gold",
                   markersize=10, label="Start"),
        plt.Line2D([0], [0], marker="X", color="w", markerfacecolor="black",
                   markersize=8, label="End"),
    ]
    ax1.legend(handles=legend_handles, fontsize=7, loc="upper left")
    ax1.set_xlabel(r"$\mu$ (channel mean)", fontsize=9)
    ax1.set_ylabel(r"$\sigma$ (channel std dev)", fontsize=9)
    ax1.set_title(
        r"Gaussian manifold drift path (Fisher-Rao metric)"
        f"\nPeak step: {drift['peak_distance']:.4f}  |  "
        f"Path length: {drift['total_length']:.4f}",
        fontsize=9)

    # Right: geodesic step distance and cumulative length
    ax2.bar(idxs, frd, color=C_DSFB, alpha=0.6, width=0.7, label="Step distance")
    ax2r = ax2.twinx()
    ax2r.plot(idxs, cum, "r-", linewidth=1.5, label="Cumulative length")
    ax2.set_xlabel("Step index", fontsize=9)
    ax2.set_ylabel("FR step distance", color=C_DSFB, fontsize=9)
    ax2r.set_ylabel("Cumulative path length", color="red", fontsize=9)
    ax2.tick_params(axis="y", labelcolor=C_DSFB)
    ax2r.tick_params(axis="y", labelcolor="red")

    # Phase labels
    for x, label in [(5, "Phase 1\nLinear"), (15, "Phase 2\nSettling"),
                      (25, "Phase 3\nReversal")]:
        ax2.axvline(x, color="gray", linestyle=":", linewidth=0.8)
        ax2.text(x, max(frd) * 0.95, label, fontsize=7, ha="center",
                 color="gray")

    lines1, labels1 = ax2.get_legend_handles_labels()
    lines2, labels2 = ax2r.get_legend_handles_labels()
    ax2.legend(lines1 + lines2, labels1 + labels2, fontsize=8)
    ax2.set_title("Geodesic step distance and cumulative path", fontsize=9)

    fig.suptitle(
        "Fig 47 – Fisher-Rao Geodesic Drift on Gaussian Manifold\n"
        r"$d_\mathrm{FR}(p_1,p_2)=2\sqrt{2}\,|\!\operatorname{arcsinh}(\frac{\mu_1-\mu_2}{2\sigma_m})|$  (§manifold)",
        fontsize=9)
    fig.tight_layout()
    save(fig, 47, "fisher_rao_drift", out_dir, dpi)


# ── Fig 48 – Relativistic Doppler Sweep ──────────────────────────────────
def plot_fig48(d, out_dir, dpi):
    """Fig 48 – Relativistic β, γ, and Doppler offset vs Mach number."""
    sweep = d["fig48_doppler_sweep"]
    pts   = sweep["points"]
    f0    = sweep["f0_hz"]

    mach  = [p["mach"]                   for p in pts]
    beta  = [p["beta"]                   for p in pts]
    gamma = [p["gamma"]                  for p in pts]
    d_hz  = [p["doppler_hz"]             for p in pts]
    cl_hz = [p["classical_doppler_hz"]   for p in pts]
    res   = [p["residual_hz"]            for p in pts]
    sig   = [p["correction_significant"] for p in pts]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4.5))

    # Left: beta and gamma vs Mach
    ax1.semilogy(mach, beta, "b-o", markersize=4, linewidth=1.6, label=r"$\beta = v/c$")
    ax1r = ax1.twinx()
    ax1r.plot(mach, gamma, "r--s", markersize=4, linewidth=1.4, label=r"$\gamma$ (Lorentz factor)")
    ax1.axvline(next((m for m, s in zip(mach, sig) if s), 30),
                color="purple", linestyle="--", linewidth=1.0,
                label=r"$\beta > 3\times10^{-5}$ threshold")
    ax1.set_xlabel("Mach number (ISA sea-level)", fontsize=9)
    ax1.set_ylabel(r"$\beta$", color="blue", fontsize=10)
    ax1r.set_ylabel(r"$\gamma$", color="red", fontsize=10)
    ax1.tick_params(axis="y", labelcolor="blue")
    ax1r.tick_params(axis="y", labelcolor="red")
    ax1.set_title(r"Lorentz factors $\beta,\gamma$ vs Mach  (f₀ = 10 GHz)", fontsize=9)
    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax1r.get_legend_handles_labels()
    ax1.legend(lines1 + lines2, labels1 + labels2, fontsize=7, loc="upper left")

    # Right: Doppler offsets vs Mach (log scale for residual)
    ax2.plot(mach, d_hz,  "b-",  linewidth=1.6, label="Relativistic Doppler (Hz)")
    ax2.plot(mach, cl_hz, "g--", linewidth=1.2, label="Classical Doppler (Hz)")
    ax2r2 = ax2.twinx()
    res_abs = [abs(r) for r in res]
    ax2r2.semilogy(mach[1:], res_abs[1:], "r:", linewidth=1.4,
                   label="Relativistic residual |δf| (Hz)")
    ax2.set_xlabel("Mach number", fontsize=9)
    ax2.set_ylabel("Doppler offset (Hz)", fontsize=9)
    ax2r2.set_ylabel("|Residual| (Hz, log scale)", color="red", fontsize=9)
    ax2r2.tick_params(axis="y", labelcolor="red")
    ax2.set_title(
        f"Doppler offset and relativistic residual\n"
        f"[f₀ = {f0/1e9:.0f} GHz, threshold: β > 3×10⁻⁵]",
        fontsize=9)
    lines1, labels1 = ax2.get_legend_handles_labels()
    lines2, labels2 = ax2r2.get_legend_handles_labels()
    ax2.legend(lines1 + lines2, labels1 + labels2, fontsize=7, loc="upper left")

    fig.suptitle(
        "Fig 48 – Relativistic Doppler Correction (Mach 0–30, ISA Sea Level)\n"
        r"$f_r = f_0\sqrt{\frac{1+\beta}{1-\beta}}$  (hypersonic platform; §high_dynamics)",
        fontsize=9)
    fig.tight_layout()
    save(fig, 48, "doppler_sweep", out_dir, dpi)


# ── Fig 49 – Quantum Noise Regime Map ─────────────────────────────────────
def plot_fig49(d, out_dir, dpi):
    """Fig 49 – Quantum-to-thermal ratio R_QT vs temperature (2K–500K)."""
    qmap = d["fig49_quantum_regime"]
    pts  = qmap["temp_sweep"]
    f0   = qmap["carrier_hz"]
    bw   = qmap["bandwidth_hz"]

    REGIME_COLORS = {
        "DeepThermal":      C_ADMISSIBLE,
        "TransitionRegime": C_BOUNDARY,
        "QuantumLimited":   C_VIOLATION,
        "BelowSQL":         "#9467bd",
    }

    temp   = [p["temp_k"]        for p in pts]
    rqt    = [p["r_qt"]          for p in pts]
    regime = [p["regime_label"]  for p in pts]
    sql_m  = [p["sql_margin"]    for p in pts]
    nph    = [p["thermal_photons"] for p in pts]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4.5))

    # Left: R_QT vs temperature (log-log), colored by regime
    prev_reg = None
    seg_x, seg_y, seg_col = [], [], None
    for t, r, reg in zip(temp, rqt, regime):
        if reg != prev_reg:
            if seg_x:
                ax1.plot(seg_x, seg_y, color=seg_col, linewidth=2.0)
            seg_x, seg_y = [t], [r]
            seg_col = REGIME_COLORS.get(reg, C_NEUTRAL)
            prev_reg = reg
        else:
            seg_x.append(t); seg_y.append(r)
    if seg_x:
        ax1.plot(seg_x, seg_y, color=seg_col, linewidth=2.0)

    colors = [REGIME_COLORS.get(r, C_NEUTRAL) for r in regime]
    ax1.scatter(temp, rqt, c=colors, s=35, zorder=4)
    ax1.axhline(1.0,  color="black", linestyle="--", linewidth=0.9,
                label=r"$R_{QT}=1$ (SQL boundary)")
    ax1.axhline(0.01, color="gray",  linestyle=":",  linewidth=0.8,
                label=r"$R_{QT}=0.01$ (deep thermal)")
    ax1.set_xscale("log"); ax1.set_yscale("log")
    ax1.set_xlabel("Receiver temperature T (K)", fontsize=9)
    ax1.set_ylabel(r"$R_{QT} = \hbar\omega / k_B T$", fontsize=9)
    ax1.set_title(
        f"Quantum-to-thermal ratio\n[f₀={f0/1e9:.0f} GHz, BW={bw/1e6:.0f} MHz]",
        fontsize=9)
    patches = [mpatches.Patch(color=v, label=k) for k, v in REGIME_COLORS.items()]
    ax1.legend(handles=patches + [
        plt.Line2D([0],[0], color="black", linestyle="--", label="SQL"),
        plt.Line2D([0],[0], color="gray",  linestyle=":",  label="DeepThermal"),
    ], fontsize=7, loc="upper right")

    # Right: SQL margin and thermal photon number vs temperature
    ax2.loglog(temp, sql_m, "b-o", markersize=4, linewidth=1.6,
               label="SQL margin (floor / shot noise)")
    ax2r = ax2.twinx()
    ax2r.loglog(temp, nph, "r--s", markersize=4, linewidth=1.4,
                label=r"Thermal photon number $\bar{n}$")
    ax2.axhline(1.0, color="gray", linestyle="--", linewidth=0.8)
    ax2.set_xlabel("Temperature T (K)", fontsize=9)
    ax2.set_ylabel("SQL margin (×)", color="blue", fontsize=9)
    ax2r.set_ylabel(r"$\bar{n}_{th}$", color="red", fontsize=9)
    ax2.tick_params(axis="y", labelcolor="blue")
    ax2r.tick_params(axis="y", labelcolor="red")
    ax2.set_title("Noise floor margin and photon occupation", fontsize=9)
    lines1, labels1 = ax2.get_legend_handles_labels()
    lines2, labels2 = ax2r.get_legend_handles_labels()
    ax2.legend(lines1 + lines2, labels1 + labels2, fontsize=7)

    fig.suptitle(
        r"Fig 49 – Quantum Noise Regime Map ($R_{QT}$ vs $T$)"
        "\n[SQL = Standard Quantum Limit; §quantum_noise — non-claim calibration reference]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 49, "quantum_regime", out_dir, dpi)


# ── Fig 50 – Swarm BFT Consensus vs Byzantine Scale ─────────────────────
def plot_fig50(d, out_dir, dpi):
    """Fig 50 – MAD-robust BFT consensus: quarantine rate vs Byzantine DSA scale."""
    swarm = d["fig50_swarm_consensus"]
    scens = swarm["scenarios"]

    byz_scale = [s["byzantine_dsa_scale"]  for s in scens]
    quarantined = [s["votes_quarantined"]  for s in scens]
    p_admin     = [s["p_admissible"]       for s in scens]
    modal       = [s["modal_state_label"]  for s in scens]
    quorum      = [s["quorum_reached"]     for s in scens]
    consensus   = [s["consensus_dsa_score"] for s in scens]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4.5))

    # Left: quarantined votes vs Byzantine scale
    bar_colors = [C_VIOLATION if q >= 1 else C_NEUTRAL for q in quarantined]
    ax1.bar(range(len(byz_scale)), quarantined, color=bar_colors, alpha=0.8)
    ax1.set_xticks(range(len(byz_scale)))
    ax1.set_xticklabels([f"{s:.0f}×" for s in byz_scale], fontsize=8)
    ax1.set_xlabel("Byzantine DSA scale factor", fontsize=9)
    ax1.set_ylabel("Votes quarantined (MAD-KS filter)", fontsize=9)
    ax1.set_yticks([0, 1])
    ax1.set_title(
        f"Quarantine decisions  [N={swarm['n_honest_nodes']} honest, "
        f"{swarm['n_byzantine_nodes']} Byzantine, BFT-f={swarm['bft_f']}]",
        fontsize=9)
    # Annotate threshold
    ax1.axhline(0.5, color="gray", linestyle="--", linewidth=0.8,
                label="Quarantine threshold")
    ax1.legend(fontsize=8)
    ax1.text(0.98, 0.95,
             "MAD robust z-score  KS filter\n"
             r"$z_\mathrm{rob} = |x-\tilde{x}|/(1.48\cdot\hat{\sigma}_\mathrm{MAD})$",
             transform=ax1.transAxes, fontsize=7, ha="right", va="top", color="gray")

    # Right: p_admissible and consensus DSA score vs scale
    modal_colors = [
        C_ADMISSIBLE if m == "Admissible" else
        C_BOUNDARY   if m == "Boundary"   else C_VIOLATION
        for m in modal
    ]
    ax2.plot(range(len(byz_scale)), p_admin, "b-o", markersize=6,
             linewidth=1.6, label=r"$p_\mathrm{admissible}$", zorder=3)
    for i, (x, y, col) in enumerate(zip(range(len(byz_scale)), p_admin, modal_colors)):
        ax2.scatter([x], [y], c=[col], s=60, zorder=4)
    ax2r = ax2.twinx()
    ax2r.plot(range(len(byz_scale)), consensus, "r--s", markersize=5,
              linewidth=1.4, label="Consensus DSA score", zorder=3)
    ax2.axhline(0.5, color="gray", linestyle=":", linewidth=0.8)
    ax2.set_xticks(range(len(byz_scale)))
    ax2.set_xticklabels([f"{s:.0f}×" for s in byz_scale], fontsize=8)
    ax2.set_xlabel("Byzantine DSA scale factor", fontsize=9)
    ax2.set_ylabel(r"$p_\mathrm{admissible}$", color="blue", fontsize=9)
    ax2r.set_ylabel("Consensus DSA score", color="red", fontsize=9)
    ax2.tick_params(axis="y", labelcolor="blue")
    ax2r.tick_params(axis="y", labelcolor="red")
    ax2.set_ylim(0, 1.05)
    ax2.set_title("Grammar consensus and DSA score", fontsize=9)
    lines1, labels1 = ax2.get_legend_handles_labels()
    lines2, labels2 = ax2r.get_legend_handles_labels()
    ax2.legend(lines1 + lines2, labels1 + labels2, fontsize=7)

    fig.suptitle(
        "Fig 50 – BFT Semiotic Swarm Consensus vs Byzantine Node DSA Scale\n"
        "[Lamport-Shostak-Pease 1982; MAD-robust KS filter; §swarm_consensus]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 50, "swarm_consensus", out_dir, dpi)


# ── Fig 51 – RG Flow Survival Curve ───────────────────────────────────────
def plot_fig51(d, out_dir, dpi):
    """Fig 51 – Renormalisation-Group flow of Betti-0 vs coarse-graining scale."""
    rg    = d["fig51_rg_flow"]
    scs   = rg["scales"]

    eps   = [s["epsilon"]             for s in scs]
    b0    = [s["betti0_surviving"]    for s in scs]
    fmrg  = [s["features_merged"]     for s in scs]
    mpers = [s["mean_persistence"]    for s in scs]
    innov = [s["innovation_fraction"] for s in scs]

    fig, axes = plt.subplots(1, 3, figsize=(13, 4.5))
    ax1, ax2, ax3 = axes

    # Left: Betti-0 survival curve
    ax1.step(eps, b0, where="post", color=C_DSFB, linewidth=2.0,
             label=r"$\beta_0(\varepsilon)$ surviving")
    ax1.fill_between(eps, b0, step="post", alpha=0.2, color=C_DSFB)
    # Annotate stable_at
    if rg["stable_at"] is not None:
        ax1.axvline(rg["stable_at"], color="green", linestyle="--", linewidth=1.0,
                    label=f"Stable at ε={rg['stable_at']:.2f}")
    # Power-law guide β_RG
    beta_rg = rg["beta_rg"]
    if len(eps) >= 2 and eps[0] > 0:
        guide_y = [b0[0] * (e / eps[0]) ** (-beta_rg) for e in eps]
        ax1.plot(eps, guide_y, "r:", linewidth=1.0,
                 label=fr"$\varepsilon^{{-{beta_rg:.2f}}}$ guide")
    ax1.set_xlabel(r"Coarse-graining scale $\varepsilon$", fontsize=9)
    ax1.set_ylabel(r"Betti-0 count $\beta_0$", fontsize=9)
    ax1.set_title(
        f"Betti-0 RG survival curve\n"
        f"Class: {rg['class_label']}  |  "
        r"$\beta_{RG}$" + f"={beta_rg:.2f}",
        fontsize=9)
    ax1.legend(fontsize=7)

    # Centre: features merged per step
    ax2.bar(range(len(eps)), fmrg, color=C_VIOLATION, alpha=0.7, width=0.6)
    ax2.set_xticks(range(len(eps)))
    ax2.set_xticklabels([f"{e:.2f}" for e in eps], rotation=45, fontsize=7)
    ax2.set_xlabel(r"$\varepsilon$ level", fontsize=9)
    ax2.set_ylabel("Features merged", fontsize=9)
    ax2.set_title("Coarse-graining events per scale step", fontsize=9)

    # Right: mean persistence and innovation fraction
    ax3.plot(range(len(eps)), mpers, "b-o", markersize=5, linewidth=1.6,
             label="Mean persistence")
    ax3r = ax3.twinx()
    ax3r.plot(range(len(eps)), innov, "g--^", markersize=5, linewidth=1.4,
              label="Innovation fraction")
    ax3.set_xticks(range(len(eps)))
    ax3.set_xticklabels([f"{e:.2f}" for e in eps], rotation=45, fontsize=7)
    ax3.set_xlabel(r"$\varepsilon$ level", fontsize=9)
    ax3.set_ylabel("Mean persistence", color="blue", fontsize=9)
    ax3r.set_ylabel("Innovation fraction", color="green", fontsize=9)
    ax3.tick_params(axis="y", labelcolor="blue")
    ax3r.tick_params(axis="y", labelcolor="green")
    ax3.set_title("Persistence and topological innovation", fontsize=9)
    lines1, labels1 = ax3.get_legend_handles_labels()
    lines2, labels2 = ax3r.get_legend_handles_labels()
    ax3.legend(lines1 + lines2, labels1 + labels2, fontsize=7)

    fig.suptitle(
        r"Fig 51 – Renormalisation-Group Flow: Betti-0 Survival $\beta_0(\varepsilon)$"
        f"\n[N={rg['n_events']} events; class: {rg['class_label']}; "
        r"$\beta_{RG}$" + f"={beta_rg:.2f}; §rg_flow]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 51, "rg_flow", out_dir, dpi)



# ─── Phase 7: Kani, SWaP-C, datasets, cycle manifest, panel scorecard ────────

def plot_fig52(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 52: Kani formal-verification coverage heatmap."""
    d = data.get("fig52_kani_coverage", {})
    modules = d.get("modules", [])
    if not modules:
        return
    labels   = [m["module_name"] for m in modules]
    proof_ns = [len(m["proof_names"]) for m in modules]
    lines    = [m["lines_covered_est"] for m in modules]
    panic_ok = [1 if m["panic_freedom"] else 0 for m in modules]
    bound_ok = [1 if m["bounds_proved"] else 0 for m in modules]

    matrix = [panic_ok, bound_ok]
    row_labels = ["Panic-free", "Bounds proved"]

    fig, axes = plt.subplots(1, 2, figsize=(10, 3.8),
                             gridspec_kw={"width_ratios": [3, 2]})
    ax0, ax1 = axes

    # Heatmap
    from matplotlib.colors import ListedColormap
    cmap = ListedColormap(["#d32f2f", "#388e3c"])
    im = ax0.imshow(matrix, aspect="auto", cmap=cmap, vmin=0, vmax=1)
    ax0.set_xticks(range(len(labels)))
    ax0.set_xticklabels(labels, rotation=30, ha="right", fontsize=8)
    ax0.set_yticks(range(len(row_labels)))
    ax0.set_yticklabels(row_labels, fontsize=8)
    for i, row in enumerate(matrix):
        for j, val in enumerate(row):
            ax0.text(j, i, "✓" if val else "✗", ha="center", va="center",
                     fontsize=14, color="white")
    ax0.set_title("Kani Property Coverage\n(✓ = proved)", fontsize=9)

    # Bar: harnesses per module
    bars = ax1.barh(labels, proof_ns, color="#1976d2")
    ax1.set_xlabel("# proof harnesses", fontsize=8)
    ax1.set_title("Harnesses per module", fontsize=9)
    for bar, n in zip(bars, proof_ns):
        ax1.text(bar.get_width() + 0.05, bar.get_y() + bar.get_height() / 2,
                 str(n), va="center", fontsize=8)
    ax1.set_xlim(0, max(proof_ns) + 1.5)

    tot = d.get("total_harnesses", 0)
    kv  = d.get("kani_min_ver", "?")
    fig.suptitle(
        f"Fig 52 – Kani Formal Panic-Freedom Coverage\n"
        f"[{tot} harnesses; Kani ≥ {kv}; run: cargo kani --features std; §XIX Kani]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 52, "kani_coverage", out_dir, dpi)


def plot_fig53(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 53: SWaP-C grouped bar comparison."""
    d = data.get("fig53_swap_c_bar", {})
    systems = d.get("systems", [])
    latency = d.get("latency_ns_per_sample", [])
    ram     = d.get("static_ram_bytes", [])
    power   = d.get("power_mw_active", [])
    if not systems:
        return

    labels_short = [s.split("\n")[0] for s in systems]

    import numpy as np
    x = np.arange(len(labels_short))
    width = 0.25

    fig, axes = plt.subplots(1, 3, figsize=(12, 4))
    colors = ["#1565c0", "#f57c00", "#c62828"]

    # Latency
    ax = axes[0]
    bars = ax.bar(x, latency, color=colors)
    ax.set_yscale("log")
    ax.set_xlabel("System")
    ax.set_ylabel("Latency (ns / sample, log)")
    ax.set_title("Latency per Sample")
    ax.set_xticks(x)
    ax.set_xticklabels(labels_short, fontsize=7)
    for bar, v in zip(bars, latency):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() * 1.5,
                f"{v:.0f} ns", ha="center", va="bottom", fontsize=7)

    # RAM
    ax = axes[1]
    bars = ax.bar(x, ram, color=colors)
    ax.set_yscale("log")
    ax.set_xlabel("System")
    ax.set_ylabel("Static RAM (bytes, log)")
    ax.set_title("Static RAM Footprint")
    ax.set_xticks(x)
    ax.set_xticklabels(labels_short, fontsize=7)
    for bar, v in zip(bars, ram):
        def fmt_bytes(b):
            if b < 1024: return f"{b} B"
            if b < 1e6:  return f"{b/1024:.0f} KB"
            if b < 1e9:  return f"{b/1e6:.0f} MB"
            return f"{b/1e9:.0f} GB"
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() * 1.5,
                fmt_bytes(v), ha="center", va="bottom", fontsize=7)

    # Power
    ax = axes[2]
    bars = ax.bar(x, power, color=colors)
    ax.set_yscale("log")
    ax.set_xlabel("System")
    ax.set_ylabel("Active Power (mW, log)")
    ax.set_title("Estimated Active Power")
    ax.set_xticks(x)
    ax.set_xticklabels(labels_short, fontsize=7)
    for bar, v in zip(bars, power):
        lbl = f"{v:.0f} mW" if v < 1000 else f"{v/1000:.0f} W"
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() * 1.5,
                lbl, ha="center", va="bottom", fontsize=7)

    fig.suptitle(
        "Fig 53 – SWaP-C Efficiency Comparison\n"
        "[DSFB 27 ns / 4 KB / ~2 mW vs Typical SDR 800 ns / 64 MB / 2.5 W "
        "vs GPU-CNN 15 ms / 24 GB / 375 W; §XIX-A]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 53, "swap_c_bar", out_dir, dpi)


def plot_fig54(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 54: RadioML episode rate vs SNR by modulation class."""
    d = data.get("fig54_radioml_episodes", {})
    entries = d.get("entries", [])
    if not entries:
        return

    import numpy as np
    # Group by modulation
    from collections import defaultdict
    by_mod = defaultdict(list)
    for e in entries:
        by_mod[e["modulation"]].append((e["snr_db"], e["episode_rate"], e["false_alarm_rate"]))

    fig, axes = plt.subplots(1, 2, figsize=(12, 4.5))
    ax_ep, ax_fa = axes

    cmap = plt.get_cmap("tab10")
    mods = sorted(by_mod.keys())

    for i, mod in enumerate(mods):
        pts = sorted(by_mod[mod])
        snrs = [p[0] for p in pts]
        eps  = [p[1] for p in pts]
        fas  = [p[2] for p in pts]
        color = cmap(i % 10)
        ax_ep.plot(snrs, eps, marker="o", markersize=4, label=mod, color=color)
        ax_fa.plot(snrs, fas, marker="s", markersize=4, label=mod, color=color)

    ax_ep.set_xlabel("SNR (dB)")
    ax_ep.set_ylabel("Episode detection rate")
    ax_ep.set_title("Episode Detection Rate vs SNR\n(structural boundary events)")
    ax_ep.legend(fontsize=6, ncol=2)
    ax_ep.grid(True, alpha=0.3)

    ax_fa.set_xlabel("SNR (dB)")
    ax_fa.set_ylabel("False alarm rate")
    ax_fa.set_title("False Alarm Rate vs SNR")
    ax_fa.legend(fontsize=6, ncol=2)
    ax_fa.grid(True, alpha=0.3)

    ds  = d.get("dataset", "RadioML2018.01a")
    ww  = d.get("window_w", 8)
    rho = d.get("threshold_rho", 0.3)
    fig.suptitle(
        f"Fig 54 – RadioML 2018.01a Structural Episode Detection\n"
        f"[{ds[:60]}; W={ww}; ρ={rho:.2f}; §L5]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 54, "radioml_episodes", out_dir, dpi)


def plot_fig55(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 55: CRAWDAD WiFi lead-time bar + table."""
    d = data.get("fig55_crawdad_lead", {})
    entries = d.get("entries", [])
    if not entries:
        return

    channels = [e["channel"] for e in entries]
    leads    = [e["lead_time_ms"] for e in entries]
    dsas     = [e["dsa_at_onset"] for e in entries]
    states   = [e["grammar_state"] for e in entries]

    med = d.get("median_lead_ms", 0)
    p95 = d.get("p95_lead_ms", 0)

    # Wider figure with no suptitle — avoids blank-top margin issue
    fig, (ax_bar, ax_tbl) = plt.subplots(
        1, 2,
        figsize=(12, 4.8),
        gridspec_kw={"width_ratios": [1.2, 1]},
    )

    # Colour: positive lead (green) vs zero/negative (amber)
    bar_colors = ["#388e3c" if v > 0 else "#f57c00" for v in leads]
    x_labels   = [f"Ch {c}" for c in channels]
    bars = ax_bar.bar(x_labels, leads, color=bar_colors, edgecolor="white", linewidth=0.5)

    ax_bar.set_ylabel("Structural precursor lead time (ms)", fontsize=9)
    ax_bar.set_xlabel("WiFi channel", fontsize=9)
    ax_bar.set_title(
        "Fig 55 — CRAWDAD WiFi Structural Precursor Lead Time\n"
        f"median = {med:.1f} ms  ·  p95 = {p95:.1f} ms  ·  §L5",
        fontsize=9, pad=6,
    )
    ax_bar.tick_params(axis="x", labelsize=8)
    ax_bar.axhline(med, linestyle="--", linewidth=0.9, color="gray",
                   label=f"Median {med:.1f} ms")
    ax_bar.axhline(0, linestyle="-", linewidth=0.5, color="black", alpha=0.4)
    ax_bar.legend(fontsize=8, loc="upper right")

    # Value labels on bars
    y_max = max(leads) if leads else 1.0
    for bar, v in zip(bars, leads):
        va  = "bottom" if v >= 0 else "top"
        off = y_max * 0.02 if v >= 0 else -y_max * 0.02
        ax_bar.text(
            bar.get_x() + bar.get_width() / 2,
            v + off,
            f"{v:.1f}",
            ha="center", va=va, fontsize=7.5,
        )

    # ── Table panel ────────────────────────────────────────────────────
    ax_tbl.axis("off")
    rows = [
        [f"Ch {c}", f"{l:.1f} ms", f"{ds:.2f}", st[:24]]
        for c, l, ds, st in zip(channels, leads, dsas, states)
    ]
    tbl = ax_tbl.table(
        cellText=rows,
        colLabels=["Channel", "Lead (ms)", "DSA∣onset", "Grammar state"],
        cellLoc="center",
        loc="center",
    )
    tbl.auto_set_font_size(False)
    tbl.set_fontsize(7.5)
    tbl.scale(1, 1.35)
    ax_tbl.set_title(
        f"[CRAWDAD {d.get('dataset','')[:45]}]",
        fontsize=6.5, pad=4,
    )

    fig.tight_layout()
    save(fig, 55, "crawdad_lead_time", out_dir, dpi)


def plot_fig56(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 56: IQ Engine / ORACLE hardware diversity scatter."""
    d = data.get("fig56_iqengine_coverage", {})
    entries = d.get("entries", [])
    if not entries:
        return

    import numpy as np
    labels   = [e["hardware_id"][-7:] for e in entries]  # last 7 chars
    means    = [e["dsa_mean"] for e in entries]
    stds     = [e["dsa_std"] for e in entries]
    densities= [e["episode_density"] for e in entries]

    fig, axes = plt.subplots(1, 2, figsize=(10, 4))
    ax_sc, ax_bar = axes

    sc = ax_sc.scatter(means, stds, c=densities, cmap="viridis",
                       s=100, edgecolors="k", linewidths=0.5, zorder=3)
    for i, lbl in enumerate(labels):
        ax_sc.annotate(lbl, (means[i], stds[i]),
                       textcoords="offset points", xytext=(5, 5), fontsize=7)
    plt.colorbar(sc, ax=ax_sc, label="Episode density")
    ax_sc.set_xlabel("DSA score mean")
    ax_sc.set_ylabel("DSA score std")
    ax_sc.set_title("Per-hardware DSA Statistics\n(ORACLE USRP B200 corpus)")
    ax_sc.grid(True, alpha=0.3)

    x = np.arange(len(labels))
    w = 0.35
    ax_bar.bar(x - w/2, means, w, label="DSA mean", color="#1976d2")
    ax_bar.bar(x + w/2, stds, w, label="DSA std", color="#f57c00", alpha=0.8)
    ax_bar.set_xticks(x)
    ax_bar.set_xticklabels(labels, rotation=35, ha="right", fontsize=7)
    ax_bar.set_ylabel("DSA value")
    ax_bar.set_title("Mean/Std per hardware unit")
    ax_bar.legend(fontsize=8)
    ax_bar.grid(True, alpha=0.3)

    ds = d.get("dataset", "ORACLE")
    fig.suptitle(
        f"Fig 56 – IQ Engine / ORACLE Corpus Hardware Diversity\n"
        f"[{ds[:70]}; §L5]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 56, "iqengine_coverage", out_dir, dpi)


def plot_fig57(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 57: Cycle-count manifest stacked / grouped bar."""
    d = data.get("fig57_cycle_manifest", {})
    stages    = d.get("stages", [])
    platforms = d.get("platforms", [])
    latency   = d.get("latency_ns", [])
    if not stages or not latency:
        return

    import numpy as np
    # Show latency_ns as grouped bars (stages × platforms)
    n_stages    = len(stages)
    n_platforms = len(platforms)
    x = np.arange(n_stages)
    width = 0.25
    colors = ["#1565c0", "#00695c", "#827717"]

    fig, ax = plt.subplots(figsize=(13, 5))
    for pi in range(n_platforms):
        offsets = x + (pi - n_platforms / 2 + 0.5) * width
        vals = [latency[si][pi] if si < len(latency) else 0
                for si in range(n_stages)]
        bars = ax.bar(offsets, vals, width, label=platforms[pi],
                      color=colors[pi % len(colors)], alpha=0.85)

    ax.set_xticks(x)
    ax.set_xticklabels(stages, rotation=30, ha="right", fontsize=7)
    ax.set_ylabel("Latency (ns) — instruction-model estimate")
    ax.set_title("Pipeline Stage Latency by Target Platform\n(Phase II measured budgets pending)")
    ax.legend(fontsize=8)
    ax.grid(True, axis="y", alpha=0.3)

    notes = d.get("notes", "")
    fig.suptitle(
        "Fig 57 – Cycle-Count Manifest: DSFB-RF Pipeline Stages\n"
        f"[{notes[:90]}]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 57, "cycle_manifest", out_dir, dpi)


def plot_fig58(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 58: Long-duration empirical stability trace (1M samples)."""
    d = data.get("fig58_stability_trace", {})
    scores = d.get("dsa_scores", [])
    if not scores:
        return

    import numpy as np
    interval = d.get("subsample_interval", 5000)
    n_total  = d.get("n_samples", len(scores) * interval)
    t_axis   = [i * interval for i in range(len(scores))]

    mean_d = d.get("mean_dsa", 0.0)
    std_d  = d.get("std_dsa", 0.0)
    drift  = d.get("max_abs_drift", 0.0)

    fig, axes = plt.subplots(2, 1, figsize=(12, 6), sharex=True)
    ax_trace, ax_hist = axes

    ax_trace.plot(t_axis, scores, lw=0.6, color="#1976d2", alpha=0.8)
    ax_trace.axhline(mean_d, linestyle="--", color="orange",
                     label=f"mean={mean_d:.4f}")
    ax_trace.axhline(mean_d + std_d, linestyle=":", color="red",
                     label=f"±1σ={std_d:.4f}")
    ax_trace.axhline(mean_d - std_d, linestyle=":", color="red")
    ax_trace.set_ylabel("DSA score")
    ax_trace.set_title(f"DSA Score Trace: {n_total:,} samples "
                       f"(subsampled ×{interval})\nmax |drift| = {drift:.6f}")
    ax_trace.legend(fontsize=7)
    ax_trace.grid(True, alpha=0.3)

    ax_hist.hist(scores, bins=40, color="#1976d2", alpha=0.7, density=True)
    ax_hist.axvline(mean_d, color="orange", linestyle="--")
    ax_hist.set_xlabel("Sample index (subsampled)")
    ax_hist.set_ylabel("Density")
    ax_hist.set_title("DSA Score Distribution across 1M-sample run")
    ax_hist.grid(True, alpha=0.3)

    fig.suptitle(
        f"Fig 58 – Long-Duration Empirical Stability: {n_total:,} Observations\n"
        f"[mean={mean_d:.4f}; std={std_d:.6f}; max_drift={drift:.6f}; §XIX-F]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 58, "stability_trace", out_dir, dpi)


def plot_fig59(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 59: Observer non-interference null test."""
    d = data.get("fig59_non_interference", {})
    snr_before = d.get("snr_before_db", [])
    snr_after  = d.get("snr_after_db", [])
    delta      = d.get("delta_db", [])
    if not delta:
        return

    import numpy as np
    max_p  = d.get("max_perturbation_db", 0.0)
    floor  = d.get("measurement_floor_db", -150.0)
    trials = d.get("n_trials", len(delta))
    verdict= d.get("verdict", "")

    fig, axes = plt.subplots(1, 3, figsize=(12, 4))

    # Before vs after overlay
    ax0 = axes[0]
    ax0.plot(snr_before, label="SNR before", color="#1976d2", lw=1.5)
    ax0.plot(snr_after, label="SNR after", color="#f57c00", linestyle="--", lw=1.5)
    ax0.set_xlabel("Trial")
    ax0.set_ylabel("SNR (dB)")
    ax0.set_title("SNR Before vs After\nDSFB-RF Observation")
    ax0.legend(fontsize=8)
    ax0.grid(True, alpha=0.3)

    # Delta
    ax1 = axes[1]
    ax1.plot(delta, color="#388e3c", lw=1.0)
    ax1.axhline(0, color="k", linestyle="--", lw=0.5)
    ax1.set_xlabel("Trial")
    ax1.set_ylabel("ΔSNR (dB)")
    ax1.set_title(f"Perturbation per Trial\n(max = {max_p:.2e} dB)")
    ax1.grid(True, alpha=0.3)

    # Histogram of delta
    ax2 = axes[2]
    ax2.hist(delta, bins=20, color="#1976d2", alpha=0.8)
    ax2.set_xlabel("ΔSNR (dB)")
    ax2.set_ylabel("Count")
    ax2.set_title("Perturbation Distribution\n(all zeros: Copy ABI guarantee)")
    ax2.grid(True, alpha=0.3)

    fig.suptitle(
        f"Fig 59 – Observer Non-Interference Null Test\n"
        f"[{trials} trials; max_perturbation = {max_p:.2e} dB; "
        f"floor = {floor} dB; verdict: {verdict[:40]}; §NON-INTRUSION]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 59, "non_interference", out_dir, dpi)


def plot_fig60(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 60: Formal proof hierarchy network diagram."""
    d = data.get("fig60_proof_hierarchy", {})
    nodes = d.get("nodes", [])
    if not nodes:
        return

    import numpy as np

    # Layout: levels 0..3, roughly evenly spaced
    level_counts = {}
    for n in nodes:
        lv = n["level"]
        level_counts[lv] = level_counts.get(lv, 0) + 1

    level_index = {}
    pos = {}
    for n in nodes:
        lv = n["level"]
        if lv not in level_index:
            level_index[lv] = 0
        idx = level_index[lv]
        count = level_counts[lv]
        x = idx - (count - 1) / 2.0
        y = 3 - lv  # level 0 at top
        pos[n["id"]] = (x, y)
        level_index[lv] += 1

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.set_aspect("equal")
    ax.axis("off")

    # Draw edges
    id_to_node = {n["id"]: n for n in nodes}
    for n in nodes:
        for dep in n["depends_on"]:
            if dep in pos:
                x0, y0 = pos[n["id"]]
                x1, y1 = pos[dep]
                ax.annotate("",
                    xy=(x1, y1), xytext=(x0, y0),
                    arrowprops=dict(arrowstyle="->", color="gray",
                                    lw=0.8, alpha=0.7))

    # Draw nodes
    type_colors = {
        "Kani":                 "#1565c0",
        "Axiom":                "#4a148c",
        "Language Guarantee":   "#00695c",
        "Design Property":      "#f57c00",
        "Compositional Proof":  "#b71c1c",
        "Unit Test":            "#558b2f",
    }
    for n in nodes:
        x, y = pos[n["id"]]
        color = type_colors.get(n["proof_type"], "#666")
        marker = "o" if n["proved"] else "x"
        ax.scatter(x, y, s=600, c=color, marker=marker, zorder=4,
                   edgecolors="k", linewidths=0.8)
        ax.text(x, y - 0.22, n["label"], ha="center", va="top",
                fontsize=6.5, wrap=True)
        ax.text(x, y + 0.12, n["proof_type"], ha="center", va="bottom",
                fontsize=5, color=color)

    # Legend
    for pt, col in type_colors.items():
        ax.scatter([], [], c=col, s=80, label=pt)
    ax.legend(loc="lower center", ncol=3, fontsize=7,
              bbox_to_anchor=(0.5, -0.12))

    ax.set_ylim(-0.6, 3.6)
    total = d.get("total_proved", 0)
    fig.suptitle(
        f"Fig 60 – Formal Proof Hierarchy: Property Dependency Graph\n"
        f"[{total}/{len(nodes)} properties proved; L0=Axioms, L3=Engine; "
        f"src/kani_proofs.rs; §XIX]",
        fontsize=9)
    fig.tight_layout()
    save(fig, 60, "proof_hierarchy", out_dir, dpi)


def plot_fig61(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 61: Structural precognition lead-time CDF."""
    d = data.get("fig61_lead_cdf", {})
    bins = d.get("lead_time_bins_ms", [])
    cdf  = d.get("cdf_values", [])
    if not bins or not cdf:
        return

    med = d.get("median_lead_ms", 0)
    p5  = d.get("p5_lead_ms", 0)
    p95 = d.get("p95_lead_ms", 0)
    fa  = d.get("false_alarm_fraction", 0)

    fig, ax = plt.subplots(figsize=(8, 4.5))
    ax.plot(bins, cdf, lw=2.0, color="#1976d2", label="CDF")
    ax.fill_between(bins, cdf, alpha=0.15, color="#1976d2")
    ax.axvline(p5,  linestyle=":", color="#f57c00", label=f"p5 = {p5:.1f} ms")
    ax.axvline(med, linestyle="--", color="orange", lw=1.5,
               label=f"median = {med:.1f} ms")
    ax.axvline(p95, linestyle=":", color="red", label=f"p95 = {p95:.1f} ms")
    ax.axhline(0.5, color="gray", lw=0.5, linestyle="--")
    ax.set_xlabel("Structural precognition lead time (ms)")
    ax.set_ylabel("CDF")
    ax.set_title(f"Structural Precognition Lead-Time CDF\n"
                 f"(time between 1st Grammar-Boundary and 1st Envelope-Violation)")
    ax.legend(fontsize=8)
    ax.grid(True, alpha=0.3)
    ax.set_ylim(0, 1.05)

    ds = d.get("dataset_anchor", "")
    fig.suptitle(
        f"Fig 61 – Structural Precognition Lead-Time CDF\n"
        f"[{ds}; p5={p5:.1f} ms; med={med:.1f} ms; "
        f"p95={p95:.1f} ms; FA={fa:.4f}; §V-A Theorem 1]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 61, "lead_time_cdf", out_dir, dpi)


def plot_fig62(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 62: Panel defence scorecard matrix."""
    d = data.get("fig62_panel_scorecard", {})
    entries = d.get("entries", [])
    if not entries:
        return

    ids         = [e["id"] for e in entries]
    criticisms  = [e["criticism_short"].replace("\n", "\n") for e in entries]
    responses   = [e["response_short"].replace("\n", "\n") for e in entries]
    ev_types    = [e["evidence_type"].replace("\n", "\n") for e in entries]
    artifacts   = [e["crate_artifact"] for e in entries]
    sections    = [e["paper_section"] for e in entries]
    confs       = [e["confidence"] for e in entries]

    import numpy as np

    fig, axes = plt.subplots(1, 2, figsize=(14, 5),
                             gridspec_kw={"width_ratios": [3, 1]})
    ax_tbl, ax_bar = axes

    # Table
    ax_tbl.axis("off")
    rows = [
        [ids[i], criticisms[i][:25], responses[i][:25],
         ev_types[i][:20], artifacts[i][:28], f"{confs[i]:.2f}"]
        for i in range(len(ids))
    ]
    tbl = ax_tbl.table(
        cellText=rows,
        colLabels=["ID", "Criticism", "Response", "Evidence", "Artifact", "Conf"],
        cellLoc="center",
        loc="center",
        bbox=[0, 0, 1, 1])
    tbl.auto_set_font_size(False)
    tbl.set_fontsize(7)
    # Color rows by confidence
    for i, conf in enumerate(confs):
        color = plt.cm.RdYlGn(conf)
        for j in range(6):
            tbl[i + 1, j].set_facecolor((*color[:3], 0.25))

    # Horizontal confidence bars
    y_pos = np.arange(len(ids))
    colors = [plt.cm.RdYlGn(c) for c in confs]
    ax_bar.barh(y_pos, confs, color=colors, edgecolor="k", linewidth=0.5)
    ax_bar.set_xlim(0, 1.05)
    ax_bar.set_yticks(y_pos)
    ax_bar.set_yticklabels(ids, fontsize=8)
    ax_bar.set_xlabel("Self-assessed confidence")
    ax_bar.set_title("Confidence per\npanel criticism")
    ax_bar.axvline(0.9, color="gray", linestyle="--", lw=0.7, label="0.9")
    ax_bar.legend(fontsize=7)
    ax_bar.grid(True, axis="x", alpha=0.3)
    for i, c in enumerate(confs):
        ax_bar.text(c + 0.01, i, f"{c:.2f}", va="center", fontsize=7)

    overall = d.get("overall_confidence", 0)
    verdict  = d.get("verdict", "")
    fig.suptitle(
        f"Fig 62 – Panel Defence Scorecard (XIX-A through XIX-F + Kani + SWaP-C)\n"
        f"[overall confidence = {overall:.3f}; §XIX; {verdict[:60]}]",
        fontsize=8)
    fig.tight_layout()
    save(fig, 62, "panel_scorecard", out_dir, dpi)


# ─── Phase 8: Swarm Governance & Operational Realism ─────────────────────────

def plot_fig63(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 63 – Swarm Scenario A: BFT false-positive suppression (LNA thermal runaway)."""
    d = data.get("fig63_swarm_scenario_a", {})
    nodes = d.get("nodes", [])
    n_steps = d.get("n_time_steps", 80)
    ms_step = d.get("ms_per_step", 2.0)
    cons_dsa = d.get("consensus_dsa", [])
    quarantined = d.get("dsfb_quarantined", [])
    t_ms = [i * ms_step for i in range(n_steps)]

    fig, axes = plt.subplots(2, 1, figsize=(11, 7), sharex=True)
    colours = ["#2196F3", "#4CAF50", "#FF9800", "#9C27B0", "#F44336"]
    labels   = ["UAV-0 (Nominal)", "UAV-1 (Nominal)", "UAV-2 (Nominal)",
                "UAV-3 (Nominal)", "UAV-4 (LNA Runaway)"]

    ax_dsa, ax_sev = axes

    for ni, node in enumerate(nodes):
        dsa = node.get("dsa_scores", [])[:n_steps]
        lw = 2.0 if node["failure_mode"] == "LNA_Thermal_Runaway" else 1.2
        ls = "--" if node["failure_mode"] == "LNA_Thermal_Runaway" else "-"
        ax_dsa.plot(t_ms[:len(dsa)], dsa, color=colours[ni % len(colours)],
                    lw=lw, ls=ls, alpha=0.85, label=labels[ni % len(labels)])

    if cons_dsa:
        ax_dsa.plot(t_ms[:len(cons_dsa)], cons_dsa, "k-", lw=2.0, alpha=0.5,
                    label="BFT Consensus DSA")

    ax_dsa.axhline(2.0, color="crimson", lw=1.0, ls=":", alpha=0.6, label="τ = 2.0 (Stage-III)")
    ax_dsa.set_ylabel("DSA Score", fontsize=9)
    ax_dsa.legend(fontsize=7, ncol=3, loc="upper left")
    ax_dsa.set_title(
        "DSFB Swarm Scenario A — BFT False-Positive Suppression\n"
        "Node #4 LNA thermal runaway: DSA spikes, 4/5 peers remain Admissible; "
        "BFT consensus stays Admissible (bft_f=1)",
        fontsize=8)

    # Grammar severity per node
    sev_labels = {0: "Admissible", 1: "Boundary", 2: "Violation"}
    for ni, node in enumerate(nodes):
        sev = node.get("grammar_severities", [])[:n_steps]
        ax_sev.step(t_ms[:len(sev)], sev, where="post",
                    color=colours[ni % len(colours)], lw=1.4 if ni < 4 else 2.0,
                    alpha=0.75, label=f"UAV-{ni}")
    ax_sev.set_yticks([0, 1, 2])
    ax_sev.set_yticklabels(["Admissible", "Boundary", "Violation"], fontsize=8)
    ax_sev.set_xlabel(f"Time (ms)  [Δt={ms_step} ms/step]", fontsize=9)
    ax_sev.set_ylabel("Grammar State", fontsize=9)
    ax_sev.legend(fontsize=7, ncol=5)

    # Mark governance outcome
    tag_str = ", ".join(
        f"UAV-{n['node_id']}→{n['final_governance_tag']}"
        for n in nodes if n["node_id"] in quarantined
    )
    std_fires = d.get("standard_alarm_fires", False)
    prov = d.get("provenance", "")[:120]
    fig.text(0.01, 0.01,
             f"Governance: {tag_str or 'None quarantined'}  |  "
             f"Std alarm fires: {std_fires}  |  "
             f"Prov: {prov}",
             fontsize=5.5, wrap=True,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.6))
    fig.tight_layout(rect=[0, 0.06, 1, 1])
    save(fig, 63, "swarm_scenario_a", out_dir, dpi)


def plot_fig64(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 64 – Swarm Scenario B: silent LO drift detection."""
    d = data.get("fig64_swarm_scenario_b", {})
    nodes = d.get("nodes", [])
    n_steps = d.get("n_time_steps", 100)
    ms_step = d.get("ms_per_step", 2.0)
    lo_id   = d.get("lo_node_id", 2)
    std_thr = d.get("standard_alarm_threshold", 5.0)
    std_at  = d.get("standard_alarm_fires_at")
    dsfb_at = d.get("dsfb_lo_precursor_at")
    clock_class = d.get("lo_clock_class", "Unknown")
    t_ms = [i * ms_step for i in range(n_steps)]

    fig, axes = plt.subplots(2, 2, figsize=(12, 7))
    colours = ["#2196F3", "#4CAF50", "#F44336", "#FF9800", "#9C27B0"]

    # Top-left: DSA time series for all nodes
    ax0 = axes[0, 0]
    for ni, node in enumerate(nodes):
        dsa = node.get("dsa_scores", [])[:n_steps]
        lw = 2.0 if node["node_id"] == lo_id else 1.0
        ax0.plot(t_ms[:len(dsa)], dsa, color=colours[ni % len(colours)], lw=lw,
                 alpha=0.85, label=f"UAV-{ni}{' (LO)' if node['node_id']==lo_id else ''}")
    ax0.axhline(std_thr, color="crimson", lw=1.2, ls="--", label=f"Std alarm @ DSA>{std_thr:.0f}")
    if dsfb_at is not None:
        ax0.axvline(dsfb_at * ms_step, color="darkgreen", lw=1.5, ls=":",
                    label=f"DSFB precursor t={dsfb_at*ms_step:.0f}ms")
    if std_at is not None:
        ax0.axvline(std_at * ms_step, color="crimson", lw=1.5, ls="-.",
                    label=f"Std alarm t={std_at*ms_step:.0f}ms")
    elif std_at is None:
        ax0.text(0.97, 0.97, "Std alarm: SILENT\n(never fires)",
                 transform=ax0.transAxes, ha="right", va="top", fontsize=7,
                 color="crimson", bbox=dict(boxstyle="round", fc="mistyrose", alpha=0.7))
    ax0.set_title("DSA Time Series (all nodes)", fontsize=8)
    ax0.set_xlabel(f"Time (ms)", fontsize=8)
    ax0.set_ylabel("DSA Score", fontsize=8)
    ax0.legend(fontsize=6.5, ncol=2)

    # Top-right: grammar severity of LO node
    ax1 = axes[0, 1]
    lo_node_data = next((n for n in nodes if n["node_id"] == lo_id), None)
    if lo_node_data:
        sev = lo_node_data.get("grammar_severities", [])[:n_steps]
        ax1.step(t_ms[:len(sev)], sev, where="post", color=colours[lo_id], lw=2.0)
        ax1.fill_between(t_ms[:len(sev)], 0, sev,
                         step="post", alpha=0.25, color=colours[lo_id])
        if dsfb_at is not None:
            ax1.axvline(dsfb_at * ms_step, color="darkgreen", lw=1.5, ls=":",
                        label=f"DSFB detects @ t={dsfb_at*ms_step:.0f}ms")
            ax1.legend(fontsize=7)
    ax1.set_yticks([0, 1, 2])
    ax1.set_yticklabels(["Admissible", "Boundary", "Violation"], fontsize=7)
    ax1.set_title(f"UAV-{lo_id} Grammar State (LO Drift Node)\n"
                  f"Clock class: {clock_class}", fontsize=8)
    ax1.set_xlabel("Time (ms)", fontsize=8)

    # Bottom-left: detection timeline comparison
    ax2 = axes[1, 0]
    events = [
        ("DSFB LO Precursor",  dsfb_at * ms_step if dsfb_at is not None else None, "darkgreen"),
        ("Standard Alarm",     std_at * ms_step  if std_at  is not None else None, "crimson"),
    ]
    y_pos = [1, 0]
    for yi, (label, t_val, col) in zip(y_pos, events):
        if t_val is not None:
            ax2.barh(yi, t_val, height=0.35, color=col, alpha=0.75, label=f"{label}: {t_val:.0f} ms")
        else:
            ax2.barh(yi, n_steps * ms_step, height=0.35, color=col, alpha=0.2,
                     label=f"{label}: NEVER FIRES")
            ax2.text(n_steps * ms_step * 0.5, yi, "SILENT", ha="center", va="center",
                     fontsize=9, color=col, fontweight="bold")
    ax2.set_yticks([0, 1])
    ax2.set_yticklabels(["Standard Alarm", "DSFB LO Precursor"], fontsize=8)
    ax2.set_xlabel("Detection latency (ms)", fontsize=8)
    ax2.set_title("Detection Timeline Comparison", fontsize=8)
    ax2.legend(fontsize=7)

    # Bottom-right: governance tags summary
    ax3 = axes[1, 1]
    ax3.axis("off")
    tag_rows = [(f"UAV-{n['node_id']}", n["failure_mode"], n["final_governance_tag"])
                for n in nodes]
    tbl = ax3.table(
        cellText=tag_rows,
        colLabels=["Node", "Failure Mode", "Governance Tag"],
        cellLoc="center", loc="center",
    )
    tbl.auto_set_font_size(False)
    tbl.set_fontsize(7.5)
    tbl.scale(1, 1.4)
    # Colour rows by governance tag
    for row_i, row in enumerate(tag_rows):
        tag = row[2]
        colour = "#ffe0e0" if "Precursor" in tag else "#e8f5e9"
        for col_j in range(3):
            tbl[(row_i + 1, col_j)].set_facecolor(colour)
    ax3.set_title("Per-node Governance Tags", fontsize=8, pad=2)

    prov = d.get("provenance", "")[:130]
    fig.suptitle(
        f"Fig 64 – Swarm Scenario B: Silent LO Drift Detection\n"
        f"DSFB detects RecurrentBoundaryGrazing + LoInstabilityPrecursor "
        f"{'before' if (dsfb_at or 999) < (std_at or 9999) else 'while'} "
        f"standard alarm stays silent  |  §XX-B",
        fontsize=8)
    fig.text(0.01, 0.005, f"Prov: {prov}", fontsize=5, wrap=True,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.5))
    fig.tight_layout(rect=[0, 0.03, 1, 0.93])
    save(fig, 64, "swarm_scenario_b", out_dir, dpi)


def plot_fig65(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 65 – Combined governance report (Scenarios A + B)."""
    d = data.get("fig65_governance_report", {})
    rows = d.get("rows", [])

    fig, (ax_tbl, ax_bar) = plt.subplots(1, 2, figsize=(14, 6),
                                          gridspec_kw={"width_ratios": [2, 1]})

    # Left panel: table
    ax_tbl.axis("off")
    col_headers = ["Scenario", "Node", "DSA", "Rob-Z", "Sev", "Governance Tag",
                   "Std Alarm", "Action?"]
    cell_data = []
    for r in rows:
        cell_data.append([
            r.get("scenario", ""),
            str(r.get("node_id", "")),
            f"{r.get('final_dsa', 0):.2f}",
            f"{r.get('robust_z', 0):.2f}",
            str(r.get("local_grammar_sev", 0)),
            r.get("governance_tag", ""),
            r.get("standard_alarm", ""),
            "[*]" if r.get("requires_action") else "--",
        ])
    if cell_data:
        tbl = ax_tbl.table(cellText=cell_data, colLabels=col_headers,
                            cellLoc="center", loc="center")
        tbl.auto_set_font_size(False)
        tbl.set_fontsize(7)
        tbl.scale(1, 1.3)
        tag_col = 5
        alarm_col = 6
    for row_i, r in enumerate(rows):
        tag = r.get("governance_tag", "")
        alarm = r.get("standard_alarm", "")
        tag_fc  = "#ffe0e0" if r.get("requires_action") else "#e8f5e9"
        alrm_fc = "#ffd0d0" if "FIRES" in alarm else "#f0f0f0"
        tbl[(row_i + 1, tag_col)].set_facecolor(tag_fc)
        tbl[(row_i + 1, alarm_col)].set_facecolor(alrm_fc)
    ax_tbl.set_title("Per-node Governance Report — Scenarios A + B", fontsize=9)

    # Right panel: summary counts
    n_flagged  = d.get("n_flagged", 0)
    n_total    = d.get("n_total", 0)
    fp_supp    = d.get("false_positives_suppressed", 0)
    si_det     = d.get("silent_threats_detected", 0)
    categories = ["Total Nodes", "DSFB Flagged", "False Positives\nSuppressed",
                  "Silent Threats\nDetected Early"]
    counts     = [n_total, n_flagged, fp_supp, si_det]
    bar_colours = ["#90A4AE", "#FF7043", "#66BB6A", "#AB47BC"]
    bars = ax_bar.barh(categories, counts, color=bar_colours, height=0.5, alpha=0.85)
    for bar, v in zip(bars, counts):
        ax_bar.text(v + 0.05, bar.get_y() + bar.get_height() / 2,
                    str(v), va="center", fontsize=10, fontweight="bold")
    ax_bar.set_xlabel("Count", fontsize=9)
    ax_bar.set_xlim(0, max(counts) + 2)
    ax_bar.set_title("Governance Summary", fontsize=9)
    ax_bar.invert_yaxis()

    prov = d.get("provenance", "")[:130]
    fig.suptitle(
        "Fig 65 – Combined Swarm Governance Report (Scenarios A + B)\n"
        "BFT consensus + typed tags vs conventional DSA > threshold alarm  |  §XX-C Table XII",
        fontsize=8)
    fig.text(0.01, 0.005, f"Prov: {prov}", fontsize=5,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.5))
    fig.tight_layout(rect=[0, 0.04, 1, 0.92])
    save(fig, 65, "governance_report", out_dir, dpi)


def plot_fig66(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 66 – Honest Bounds: physics limits and DSFB response (§XX Table XI)."""
    d = data.get("fig66_honest_bounds", {})
    entries = d.get("entries", [])

    fig, ax = plt.subplots(figsize=(13, 6))
    ax.axis("off")

    col_headers = ["Threat Class", "SNR Condition", "DSFB Behaviour",
                   "Honest Acknowledgment", "Mitigated?"]
    cell_data = []
    for e in entries:
        cell_data.append([
            e.get("threat_class", ""),
            e.get("snr_condition", ""),
            # Wrap at ~55 chars for readability
            "\n".join(
                e.get("dsfb_behaviour", "")[i:i+55]
                for i in range(0, min(165, len(e.get("dsfb_behaviour",""))), 55)
            ),
            "\n".join(
                e.get("honest_acknowledgment", "")[i:i+55]
                for i in range(0, min(165, len(e.get("honest_acknowledgment",""))), 55)
            ),
            "Yes" if e.get("mitigation_available") else "No",
        ])

    if cell_data:
        tbl = ax.table(cellText=cell_data, colLabels=col_headers,
                        cellLoc="left", loc="center",
                        colWidths=[0.18, 0.10, 0.24, 0.28, 0.08])
        tbl.auto_set_font_size(False)
        tbl.set_fontsize(6.5)
        tbl.scale(1, 4.2)
        for row_i, e in enumerate(entries):
            mit = e.get("mitigation_available", False)
            fc_last = "#e8f5e9" if mit else "#fff3e0"
            tbl[(row_i + 1, 4)].set_facecolor(fc_last)
            tbl[(row_i + 1, 0)].set_facecolor("#e3f2fd")
        # Bold header row
        for col_j in range(len(col_headers)):
            tbl[(0, col_j)].set_facecolor("#37474F")
            tbl[(0, col_j)].set_text_props(color="white", fontweight="bold")

    crate_note = d.get("crate_note", "")
    prov   = d.get("provenance", "")
    ax.set_title(
        "Fig 66 – Honest Bounds: Physics Limits and DSFB Operating Constraints\n"
        f"§XX Table XI  |  {crate_note[:100]}",
        fontsize=8, pad=6)
    fig.text(0.01, 0.005, f"Prov: {prov}", fontsize=5.5,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.5))
    fig.tight_layout(rect=[0, 0.03, 1, 0.93])
    save(fig, 66, "honest_bounds", out_dir, dpi)


def plot_fig67(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 67 – Allan deviation oscillator classification benchmark (IEEE Std 1139-2008)."""
    d = data.get("fig67_allan_bench", {})
    curves = d.get("curves", [])
    tau_units = d.get("tau_units", "s")

    fig, ax = plt.subplots(figsize=(9, 6))
    colours  = ["#1565C0", "#2E7D32", "#B71C1C"]
    markers  = ["o", "s", "^"]
    alpha_slopes = {-1.0: "slope -1 (White FM)", -0.5: "slope -0.5 (Flicker FM)",
                    0.5:  "slope +0.5 (Rand Walk FM)"}

    for ci, curve in enumerate(curves):
        taus    = curve.get("taus", [])
        sigma_y = curve.get("sigma_y", [])
        label   = curve.get("oscillator_class", f"Oscillator {ci}")
        cls     = curve.get("classified_as", "?")
        alpha_v = curve.get("slope_alpha", 0.0)
        if not taus or not sigma_y:
            continue
        ax.loglog(taus, sigma_y,
                  color=colours[ci % len(colours)],
                  marker=markers[ci % len(markers)],
                  markersize=5, lw=1.8, label=f"{label}\n  → classified: {cls}")
        # Reference slope line
        t0, sy0 = taus[0], sigma_y[0]
        slope_taus = [t0, taus[-1]]
        slope_sy   = [sy0, sy0 * (taus[-1] / t0) ** alpha_v]
        ax.loglog(slope_taus, slope_sy, ls="--", lw=0.8,
                  color=colours[ci % len(colours)], alpha=0.5,
                  label=alpha_slopes.get(alpha_v, f"slope {alpha_v}"))

    ax.set_xlabel(f"Integration time τ  [{tau_units}]", fontsize=10)
    ax.set_ylabel("Allan Deviation σ_y(τ)", fontsize=10)
    ax.legend(fontsize=7.5, loc="upper right")
    ax.grid(True, which="both", ls=":", alpha=0.4)
    ax.set_title(
        "Fig 67 – Allan Deviation Oscillator Classification Benchmark\n"
        "Three canonical IEEE Std 1139-2008 classes; "
        "classified by dsfb_rf::heuristics::classify_clock_instability()  |  §XX-B",
        fontsize=8)
    ref = d.get("reference", "")
    prov = d.get("provenance", "")[:130]
    fig.text(0.01, 0.005,
             f"Ref: {ref}  |  Prov: {prov}",
             fontsize=5.5,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.5))
    fig.tight_layout(rect=[0, 0.04, 1, 1])
    save(fig, 67, "allan_dev_bench", out_dir, dpi)


def plot_fig68(data: dict, out_dir: Path, dpi: int) -> None:
    """Fig 68 – Non-intrusion manifest: stack breakdown and governance chain."""
    d = data.get("fig68_non_intrusion", {})
    comps = d.get("components", [])
    total_bytes   = d.get("total_bytes", 504)
    heap_bytes    = d.get("heap_alloc_bytes", 0)
    unsafe_blocks = d.get("unsafe_blocks", 0)
    gov_chain     = d.get("governance_chain", [])
    checklist     = d.get("integration_checklist", [])

    fig, axes = plt.subplots(1, 3, figsize=(15, 6),
                              gridspec_kw={"width_ratios": [1.2, 1.4, 1.4]})

    # Left: horizontal bar chart of stack components
    ax0 = axes[0]
    comp_names = [c.get("component", "") for c in comps]
    comp_bytes = [c.get("bytes", 0) for c in comps]
    hot_mask   = [c.get("hot_path", False) for c in comps]
    colours_bc = ["#EF5350" if h else "#42A5F5" for h in hot_mask]
    bars = ax0.barh(comp_names, comp_bytes, color=colours_bc, height=0.55, alpha=0.85)
    for bar, b in zip(bars, comp_bytes):
        ax0.text(b + 0.5, bar.get_y() + bar.get_height() / 2,
                 f"{b}B", va="center", fontsize=7)
    ax0.set_xlabel("Bytes (field-sum)", fontsize=8)
    ax0.set_title(
        f"Stack Memory Breakdown\nTotal: {total_bytes}B  |  Heap: {heap_bytes}B  |  "
        f"Unsafe: {unsafe_blocks}\n"
        f"  ■ Hot path (red)   ■ Cold path (blue)",
        fontsize=7.5)
    ax0.invert_yaxis()
    ax0.set_xlim(0, max(comp_bytes) * 1.3 if comp_bytes else 250)
    # Pie inset
    ax_pie = ax0.inset_axes([0.5, 0.01, 0.45, 0.28])
    ax_pie.pie(comp_bytes, colors=colours_bc, startangle=90,
               wedgeprops=dict(linewidth=0.4, edgecolor="white"))
    ax_pie.set_title(f"{total_bytes}B total", fontsize=6)

    # Centre: governance chain text
    ax1 = axes[1]
    ax1.axis("off")
    chain_text = "\n\n".join(f"  {step}" for step in gov_chain)
    ax1.text(0.05, 0.95, "Governance Chain\n" + "─" * 40 + "\n\n" + chain_text,
             transform=ax1.transAxes, va="top", ha="left", fontsize=7.5,
             fontfamily="monospace",
             bbox=dict(boxstyle="round,pad=0.5", fc="#F5F5F5", alpha=0.9))
    ax1.set_title("Read-Only Governance Flow", fontsize=8)

    # Right: integration checklist
    ax2 = axes[2]
    ax2.axis("off")
    check_text = "\n\n".join(f"  [x] {item}" for item in checklist)
    ax2.text(0.05, 0.95, "Integration Checklist\n" + "─" * 38 + "\n\n" + check_text,
             transform=ax2.transAxes, va="top", ha="left", fontsize=7.5,
             fontfamily="monospace",
             bbox=dict(boxstyle="round,pad=0.5", fc="#E8F5E9", alpha=0.9))
    ax2.set_title("Deployment Checklist", fontsize=8)

    prov = d.get("provenance", "")[:130]
    fig.suptitle(
        "Fig 68 – Non-Intrusion Manifest\n"
        f"504-byte stack · 0-byte heap · 0 unsafe blocks · read-only tap · §XIX-A §XX-D",
        fontsize=8)
    fig.text(0.01, 0.005, f"Prov: {prov}", fontsize=5,
             bbox=dict(boxstyle="round,pad=0.2", fc="lightyellow", alpha=0.5))
    fig.tight_layout(rect=[0, 0.04, 1, 0.91])
    save(fig, 68, "non_intrusion_manifest", out_dir, dpi)


FIGURES: dict = {
    1:  plot_fig01,   2:  plot_fig02,   3:  plot_fig03,
    4:  plot_fig04,   5:  plot_fig05,   6:  plot_fig06,
    7:  plot_fig07,   8:  plot_fig08,   9:  plot_fig09,
    10: plot_fig10,   11: plot_fig11,   12: plot_fig12,
    13: plot_fig13,   14: plot_fig14,   15: plot_fig15,
    16: plot_fig16,   17: plot_fig17,   18: plot_fig18,
    19: plot_fig19,   20: plot_fig20,
    21: plot_fig21,   22: plot_fig22,   23: plot_fig23,
    24: plot_fig24,   25: plot_fig25,   26: plot_fig26,
    27: plot_fig27,   28: plot_fig28,   29: plot_fig29,
    30: plot_fig30,   31: plot_fig31,   32: plot_fig32,
    33: plot_fig33,   34: plot_fig34,   35: plot_fig35,
    36: plot_fig36,   37: plot_fig37,
    39: plot_fig39,   40: plot_fig40,
    # Phase-5: calibration sensitivity + SBIR positioning
    41: plot_fig41,   42: plot_fig42,   43: plot_fig43,
    44: plot_fig44,   45: plot_fig45,
    # Phase-6: thermodynamics, manifolds, relativity, quantum noise, BFT, RG
    46: plot_fig46,   47: plot_fig47,   48: plot_fig48,
    49: plot_fig49,   50: plot_fig50,   51: plot_fig51,
    # Fig 38 reuses fig19 architecture data (no separate plot function)
    # Phase-7: Kani, SWaP-C, datasets, cycle manifest, panel scorecard
    52: plot_fig52,  53: plot_fig53,  54: plot_fig54,
    55: plot_fig55,  56: plot_fig56,  57: plot_fig57,
    58: plot_fig58,  59: plot_fig59,  60: plot_fig60,
    61: plot_fig61,  62: plot_fig62,
    # Phase-8: Swarm governance & operational realism
    63: plot_fig63,  64: plot_fig64,  65: plot_fig65,
    66: plot_fig66,  67: plot_fig67,  68: plot_fig68,
}


def main():
    parser = argparse.ArgumentParser(description="DSFB-RF unified figure generator (68 figures)")
    parser.add_argument("--data",  default="dsfb-rf-output/figure_data_all.json",
                        help="Input JSON data file (default: dsfb-rf-output/figure_data_all.json)")
    parser.add_argument("--out",   default="dsfb-rf-output/figs",
                        help="Output directory (default: dsfb-rf-output/figs/)")
    parser.add_argument("--dpi",   type=int, default=150,
                        help="Figure DPI (default: 150; use 300 for print)")
    parser.add_argument("--fig",   type=int, nargs="*",
                        help="Specific figure numbers (default: all)")
    args = parser.parse_args()

    data_path = Path(args.data)
    if not data_path.exists():
        print(f"ERROR: {data_path} not found.")
        print("Run:  cargo run --features std,serde --example generate_figures_all")
        sys.exit(1)

    with open(data_path, "r") as fh:
        data = json.load(fh)

    out_dir  = Path(args.out)
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
