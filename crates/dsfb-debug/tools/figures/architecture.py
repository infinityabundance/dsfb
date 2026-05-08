"""Architecture / infrastructure figures.

Three figures:
  1. ml_vs_dsfb.png       — two-column comparison (ML black box vs DSFB-Debug forensic packet)
  2. tier_breakdown.png   — 27-tier detector counts grouped by mathematical family
  3. motif_affinity.png   — 32-motif × 27-tier affinity heatmap with FULL readable labels
"""
from __future__ import annotations

import json
from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch
import numpy as np

from . import _style as S


# ----------------------------------------------------------------------
# Figure: ML vs DSFB side-by-side (replaces previous pipeline figure)
# ----------------------------------------------------------------------
def render_ml_vs_dsfb(out_path: Path):
    fig, axes = plt.subplots(1, 2, figsize=(13.0, 7.5),
                              gridspec_kw={"wspace": 0.18})
    fig.suptitle(
        "How DSFB-Debug differs from learned anomaly detection",
        fontsize=14, fontweight="bold", y=0.99,
    )

    # ----- Left panel: ML anomaly detector -------------------------
    ax = axes[0]
    ax.set_xlim(0, 10)
    ax.set_ylim(0, 10)
    ax.set_aspect("auto")
    ax.set_xticks([]); ax.set_yticks([])
    for s in ["top", "right", "left", "bottom"]:
        ax.spines[s].set_visible(False)
    ax.set_title("Learned anomaly detector\n(Donut, Bagel, OmniAnomaly,\nDeepTraLog, TraceCRL, …)",
                 fontsize=10, color=S.DSFB_GRAY, loc="center", pad=10)

    boxes = [
        (1, 8.5, 8, 0.9, "Production telemetry\n(traces, logs, metrics)", "#EAEAEA"),
        (1, 7.0, 8, 0.9, "Learned model\n(neural net / VAE / GNN — opaque)", "#D6D3CC"),
        (1, 5.5, 8, 0.9, "Anomaly score (single scalar)", "#C9C7C0"),
    ]
    for x, y, w, h, txt, c in boxes:
        ax.add_patch(FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.05",
                                     facecolor=c, edgecolor="#666", linewidth=0.7))
        ax.text(x + w/2, y + h/2, txt, ha="center", va="center", fontsize=9)
    # Down-arrows
    for y0, y1 in [(8.5, 7.9), (7.0, 6.4)]:
        ax.add_patch(FancyArrowPatch((5, y0), (5, y1), arrowstyle="-|>",
                                      mutation_scale=12, color="#666", linewidth=0.8))

    # Cost annotations
    notes = [
        "× requires training corpus",
        "× retrains when prod changes",
        "× silent drift; explainability gap",
        "× GPU bill at inference scale",
    ]
    for i, n in enumerate(notes):
        ax.text(1, 4.3 - i * 0.55, n, fontsize=8.5, color="#9A031E",
                family="sans-serif")

    ax.text(5, 0.5, "→ One number, no structure, no audit trail.",
            ha="center", fontsize=9, style="italic", color="#444")

    # ----- Right panel: DSFB-Debug ---------------------------------
    ax = axes[1]
    ax.set_xlim(0, 10)
    ax.set_ylim(0, 10)
    ax.set_aspect("auto")
    ax.set_xticks([]); ax.set_yticks([])
    for s in ["top", "right", "left", "bottom"]:
        ax.spines[s].set_visible(False)
    ax.set_title("DSFB-Debug\n(deterministic detector-field\nsemiotics, ML-free)",
                 fontsize=10, color=S.DSFB_PRIMARY, loc="center", fontweight="bold",
                 pad=10)

    boxes = [
        (0.5, 8.6, 9, 0.7, "Residual matrix (window × signal)", "#E0EAEC"),
        (0.5, 7.6, 9, 0.7, "205 deterministic detectors / 27 axes", "#C9DDE0"),
        (0.5, 6.6, 9, 0.7, "Tier-coded witness field (per cell)", "#A8C8CC"),
        (0.5, 5.6, 9, 0.7, "DSFB structural episodes (Theorem 9)", "#85ADB3"),
        (0.5, 4.6, 9, 0.7, "32-motif bank · routed evidence", "#5C8E96"),
        (0.5, 3.6, 9, 0.7, "9-axis fusion + anti-hallucination ladder", "#3F7280"),
        (0.5, 2.4, 9, 1.0, "Forensic evidence packet\n(top motif, runner-up, confuser, 3 margins, witnesses, root cause)", "#2A9D8F"),
    ]
    for x, y, w, h, txt, c in boxes:
        ax.add_patch(FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.04",
                                     facecolor=c, edgecolor="#444", linewidth=0.8))
        text_color = "white" if c in ("#3F7280", "#2A9D8F", "#5C8E96") else "#1a1a1a"
        ax.text(x + w/2, y + h/2, txt, ha="center", va="center",
                fontsize=8.5, color=text_color)
    # Down-arrows
    for y0, y1 in [(8.6, 8.3), (7.6, 7.3), (6.6, 6.3),
                    (5.6, 5.3), (4.6, 4.3), (3.6, 3.4)]:
        ax.add_patch(FancyArrowPatch((5, y0), (5, y1), arrowstyle="-|>",
                                      mutation_scale=12, color="#222", linewidth=0.8))

    notes = [
        "✓ no training corpus, no retrain",
        "✓ deterministic (Theorem 9 replay)",
        "✓ no_std + zero deps + edge-deployable",
        "✓ full audit trail per episode",
    ]
    for i, n in enumerate(notes):
        ax.text(0.5, 1.6 - i * 0.32, n, fontsize=8.5, color="#0F4C5C")

    plt.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# Figure: per-tier detector breakdown grouped by family
# ----------------------------------------------------------------------
TIER_FAMILIES = [
    ("Parametric / robust",      ["A", "B", "EXTRA"],            "#1F77B4"),
    ("Model + non-parametric",   ["C", "D"],                       "#2A9D8F"),
    ("Debug-specific / burst",   ["E", "F", "M"],                  "#E36414"),
    ("Drift / change-point",     ["G", "N", "O", "P", "Q"],        "#9A031E"),
    ("Distribution / forecast",  ["H", "I", "J"],                  "#5F0F40"),
    ("Frequency / multivariate", ["K", "L"],                       "#0F4C5C"),
    ("Depth / count / info",     ["R", "S", "T"],                  "#7C7E80"),
    ("Dynamical / Phase-5 wave", ["U", "V", "X", "Y", "Z", "AA"],  "#264653"),
]

TIER_COUNTS = {
    "A": 3, "B": 3, "C": 5, "D": 5, "E": 3, "F": 4, "EXTRA": 5,
    "G": 9, "H": 10, "I": 10, "J": 10, "K": 10, "L": 9,
    "M": 18, "N": 8, "O": 10, "P": 9, "Q": 10, "R": 8,
    "S": 3, "T": 6, "U": 8,
    "V": 8, "X": 8, "Y": 8, "Z": 8, "AA": 11,
}


def render_tier_breakdown(out_path: Path):
    fig, ax = plt.subplots(figsize=S.figsize("wide"))
    ax.set_title("Detector ensemble — 205 deterministic detectors across 27 mathematical axes",
                 loc="left")

    # Build the bars: ordered by family, coloured by family
    rows = []
    for fam_name, tiers, col in TIER_FAMILIES:
        for t in tiers:
            rows.append((t, TIER_COUNTS[t], col, fam_name))

    y_positions = list(range(len(rows)))
    counts = [r[1] for r in rows]
    colors = [r[2] for r in rows]
    labels = [r[0] for r in rows]

    bars = ax.barh(y_positions, counts, color=colors, edgecolor="white", linewidth=0.4)
    ax.set_yticks(y_positions)
    ax.set_yticklabels(labels, fontsize=8)
    ax.invert_yaxis()
    ax.set_xlabel("Detector count (sum = 205)")
    ax.set_xlim(0, max(counts) * 1.15)
    # Value labels
    for bar, n in zip(bars, counts):
        ax.text(n + 0.3, bar.get_y() + bar.get_height() / 2,
                str(n), va="center", fontsize=7.5, color="#222")

    # Family legend (right side)
    legend_handles = [mpatches.Patch(color=col, label=fam)
                      for fam, _, col in TIER_FAMILIES]
    ax.legend(handles=legend_handles, loc="lower right", title="Mathematical family",
              title_fontsize=8, fontsize=7.5, frameon=True,
              facecolor="white", edgecolor="#CCC")
    ax.grid(axis="x", linewidth=0.4)
    ax.grid(axis="y", visible=False)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# Figure: 32-motif × 27-tier affinity heatmap with FULL labels
# ----------------------------------------------------------------------
TIER_BIT_NAMES = [
    "A", "B", "C", "D", "E", "F", "EXTRA",
    "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U",
    "V", "W", "X", "Y", "Z", "AA",
]
TIER_BITS = [
    1 << 0, 1 << 1, 1 << 2, 1 << 3, 1 << 4, 1 << 5, 1 << 6,
    1 << 7, 1 << 8, 1 << 9, 1 << 10, 1 << 11, 1 << 12, 1 << 13,
    1 << 14, 1 << 15, 1 << 16, 1 << 17, 1 << 18, 1 << 19, 1 << 20, 1 << 21,
    1 << 22, 1 << 23, 1 << 24, 1 << 25, 1 << 26, 1 << 27,
]


def render_motif_affinity(motif_data: list[dict], out_path: Path):
    """`motif_data`: list of {"motif": str, "affinity_tiers": int}."""
    motif_names = [m["motif"] for m in motif_data]
    n_motifs = len(motif_data)
    n_tiers = len(TIER_BIT_NAMES)

    matrix = np.zeros((n_motifs, n_tiers), dtype=int)
    for i, m in enumerate(motif_data):
        bits = m["affinity_tiers"]
        for j, b in enumerate(TIER_BITS):
            if bits & b:
                matrix[i, j] = 1

    # Tall figure: 32 rows + readable margins
    height_in = max(8.5, 0.20 * n_motifs + 1.5)
    fig, ax = plt.subplots(figsize=(7.0, height_in))

    # Use a custom 2-color colormap
    from matplotlib.colors import ListedColormap
    cmap = ListedColormap(["#F2EFE8", S.DSFB_PRIMARY])
    ax.imshow(matrix, cmap=cmap, aspect="auto", interpolation="nearest")

    ax.set_xticks(range(n_tiers))
    ax.set_xticklabels(TIER_BIT_NAMES, fontsize=7.5, rotation=0)
    ax.set_yticks(range(n_motifs))
    ax.set_yticklabels(motif_names, fontsize=7.5)
    ax.set_xlabel("Detector tier", fontsize=9)
    ax.set_ylabel("Motif (32 hand-curated)", fontsize=9)
    ax.set_title("Routed Evidence Principle: motif × axis affinity matrix",
                 loc="left", fontsize=11)

    # Subtle gridlines between cells for readability
    ax.set_xticks([x - 0.5 for x in range(1, n_tiers)], minor=True)
    ax.set_yticks([y - 0.5 for y in range(1, n_motifs)], minor=True)
    ax.grid(which="minor", color="#FFFFFF", linewidth=0.6)
    ax.grid(which="major", visible=False)
    ax.tick_params(which="minor", length=0)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)
