"""Operator-facing figures: evidence packets, confuser adjudication, ladder.

Three figures:
  1. evidence_cards.png       — 3-card panel showing F-11 episode evidence packets.
  2. confuser_adjudication.png — F-11 ep0 top vs declared confuser, margin-gate decision.
  3. anti_hallucination_ladder.png — Phase 0/5.6/7/8 progression bar chart.
"""
from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch
import numpy as np

from . import _style as S


# ----------------------------------------------------------------------
# Evidence-packet "cards" — 3 side-by-side cards, one per F-11 episode.
# ----------------------------------------------------------------------
# Per-episode evidence packets — verbatim from the Phase-2 tier-affinity
# diagnostic output captured in tests/fusion_compare.rs stdout.
F11_EPISODES = [
    {
        "id": 0,
        "label": "Episode 0",
        "disposition": "Named",
        "top_motif":     "DeploymentRegressionSlew",
        "top_score":     12920.22,
        "runner_up":     "CircuitBreakerOpenShift",
        "runner_score":  9261.07,
        "confuser":      "CircuitBreakerOpenShift",
        "confuser_score":9261.07,
        "margin":        0.283,
        "margin_vs_conf":0.283,
        "tier_cf":       0.750,
        "primary_tier_witness": True,
        "named_witness_fired":  True,
    },
    {
        "id": 1,
        "label": "Episode 1",
        "disposition": "Named",
        "top_motif":     "AuthenticationFailureSpike",
        "top_score":     2.29,
        "runner_up":     "EpisodicTransientSpike",
        "runner_score":  1.85,
        "confuser":      "EpisodicTransientSpike",
        "confuser_score":1.85,
        "margin":        0.192,
        "margin_vs_conf":0.192,
        "tier_cf":       0.500,
        "primary_tier_witness": True,
        "named_witness_fired":  True,
    },
    {
        "id": 2,
        "label": "Episode 2  ← ConfuserAmbiguous",
        "disposition": "ConfuserAmbiguous",
        "top_motif":     "AuthenticationFailureSpike",
        "top_score":     2.70,
        "runner_up":     "EpisodicTransientSpike",
        "runner_score":  2.53,
        "confuser":      "EpisodicTransientSpike",
        "confuser_score":2.53,
        "margin":        0.065,
        "margin_vs_conf":0.065,
        "tier_cf":       0.333,
        "primary_tier_witness": True,
        "named_witness_fired":  False,
    },
]


def render_evidence_cards(out_path: Path):
    fig, axes = plt.subplots(1, 3, figsize=(11.5, 6.0))
    fig.suptitle("F-11 forensic evidence packets — operator-facing per-episode output",
                 fontsize=12, fontweight="bold", y=0.99)

    for ax, ep in zip(axes, F11_EPISODES):
        ax.set_xlim(0, 10); ax.set_ylim(0, 10)
        ax.set_aspect("equal")
        ax.set_xticks([]); ax.set_yticks([])
        for s in ["top", "right", "left", "bottom"]:
            ax.spines[s].set_visible(False)
        # Card outline
        is_amb = ep["disposition"] == "ConfuserAmbiguous"
        card_color = "#FFF6E5" if is_amb else "#EAF6F4"
        edge_color = S.DSFB_ACCENT if is_amb else S.DSFB_PRIMARY
        ax.add_patch(FancyBboxPatch(
            (0.2, 0.2), 9.6, 9.6, boxstyle="round,pad=0.1",
            facecolor=card_color, edgecolor=edge_color, linewidth=1.5))
        # Header band
        header_color = S.DSFB_ACCENT if is_amb else S.DSFB_PRIMARY
        ax.add_patch(FancyBboxPatch(
            (0.2, 8.6), 9.6, 1.2, boxstyle="round,pad=0.1",
            facecolor=header_color, edgecolor=header_color))
        ax.text(5, 9.2, ep["label"], ha="center", va="center",
                fontsize=11, fontweight="bold", color="white")

        # Disposition
        ax.text(0.6, 8.0, "disposition", fontsize=8, color="#666", style="italic")
        ax.text(0.6, 7.6, ep["disposition"], fontsize=11, fontweight="bold",
                color=edge_color)

        # Top / runner-up / confuser block
        rows = [
            ("top motif",     ep["top_motif"],      f"score {ep['top_score']:.2f}"),
            ("runner-up",     ep["runner_up"],      f"score {ep['runner_score']:.2f}"),
            ("declared confuser", ep["confuser"],   f"score {ep['confuser_score']:.2f}"),
        ]
        y = 7.0
        for k, v, sc in rows:
            ax.text(0.6, y, k + ":", fontsize=7.5, color="#666")
            ax.text(0.6, y - 0.32, v, fontsize=8.5, color="#222", fontweight="bold")
            ax.text(9.4, y - 0.32, sc, fontsize=7.5, color="#666", ha="right")
            y -= 0.85

        # Margins
        ax.text(0.6, y, "margins:", fontsize=7.5, color="#666")
        y -= 0.32
        ax.text(0.6, y, f"top vs runner-up:  {ep['margin']:.3f}",
                fontsize=8, color="#222")
        y -= 0.30
        thr_warn = ep["margin_vs_conf"] < 0.10
        ax.text(0.6, y, f"top vs confuser:   {ep['margin_vs_conf']:.3f}"
                + ("   ← below 0.10 gate" if thr_warn else ""),
                fontsize=8, color="#9A031E" if thr_warn else "#222",
                fontweight="bold" if thr_warn else "regular")
        y -= 0.40

        # tier_consensus_factor as a small bar
        ax.text(0.6, y, f"tier_consensus_factor: {ep['tier_cf']:.3f}",
                fontsize=8, color="#222")
        y -= 0.42
        # bar
        bar_w = 6.5
        bar_h = 0.20
        ax.add_patch(mpatches.Rectangle((0.6, y), bar_w, bar_h, facecolor="#E0DDD7",
                                         edgecolor="none"))
        ax.add_patch(mpatches.Rectangle(
            (0.6, y), bar_w * ep["tier_cf"], bar_h, facecolor=edge_color, edgecolor="none"))
        y -= 0.55

        # Witness flags
        ax.text(0.6, y, "witness gates:", fontsize=7.5, color="#666")
        y -= 0.32
        for k, v in [("primary tier witness fired", ep["primary_tier_witness"]),
                      ("named-detector witness fired", ep["named_witness_fired"])]:
            mark = "✓" if v else "✗"
            color = "#2A9D8F" if v else "#9A031E"
            ax.text(0.6, y, f"{mark}  {k}", fontsize=8, color=color)
            y -= 0.30

    fig.tight_layout(rect=[0, 0, 1, 0.96])
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# Confuser-pair adjudication (NEW)
# ----------------------------------------------------------------------
def render_confuser_adjudication(out_path: Path):
    """F-11 episode 0: typed motif vs declared confuser scoring + gate decision."""
    fig, ax = plt.subplots(figsize=S.figsize("double"))

    motifs = ["DeploymentRegressionSlew\n(matched motif)",
              "CircuitBreakerOpenShift\n(declared confuser)"]
    scores = [12920.22, 9261.07]
    colors = [S.DSFB_PRIMARY, S.DSFB_ACCENT]

    bars = ax.barh(motifs, scores, color=colors, edgecolor="white", linewidth=0.5,
                    height=0.5)
    for b, s in zip(bars, scores):
        ax.text(s + 200, b.get_y() + b.get_height()/2,
                f"{s:,.0f}", va="center", fontsize=9, fontweight="bold")

    # Margin annotation
    ax.annotate(
        "",
        xy=(scores[0], 0.4), xytext=(scores[1], 0.4),
        arrowprops=dict(arrowstyle="<->", color="#222", linewidth=1.2),
    )
    margin = (scores[0] - scores[1]) / scores[0]
    ax.text((scores[0] + scores[1]) / 2, 0.55,
            f"margin_vs_confuser = {margin:.3f}",
            ha="center", fontsize=9, color="#222", fontweight="bold")
    ax.text((scores[0] + scores[1]) / 2, 0.75,
            f"(gate: 0.10  →  PASS  →  Named({{matched motif}}))",
            ha="center", fontsize=8.5, color=S.DSFB_PRIMARY)

    ax.set_xlim(0, 14500)
    ax.set_xlabel("Bank score (sum of routed-evidence terms)")
    ax.set_title("F-11 episode 0 — confuser-pair adjudication (Phase 5.6 gate)",
                 loc="left", fontsize=11)
    ax.invert_yaxis()
    ax.grid(axis="x", linewidth=0.4, alpha=0.6)
    ax.grid(axis="y", visible=False)
    for s in ["top", "right"]:
        ax.spines[s].set_visible(False)

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


# ----------------------------------------------------------------------
# Anti-Hallucination Ladder progression (the REAL one)
# ----------------------------------------------------------------------
def render_anti_hallucination_ladder(out_path: Path):
    """F-11 typed/ambiguous/confuser_ambiguous counts at Phase 0/5.6/7/8."""
    phases = ["Phase 0\n(raw consensus)", "Phase 5.6\n(confuser gate)",
              "Phase 7\n(tier witness)",  "Phase 8\n(named witness)"]
    typed_count       = [3, 2, 2, 2]
    confuser_amb      = [0, 1, 1, 1]
    silent_demoted    = [0, 0, 0, 0]   # F-11: no further demotions at Phase 8

    fig, ax = plt.subplots(figsize=S.figsize("double"))
    x = np.arange(len(phases))
    w = 0.55
    typed_bar    = ax.bar(x, typed_count,    color=S.DSFB_HIGHLIGHT, width=w,
                           label="Typed (Named)", edgecolor="white", linewidth=0.5)
    conf_bar     = ax.bar(x, confuser_amb,   bottom=typed_count, color=S.DSFB_ACCENT, width=w,
                           label="ConfuserAmbiguous", edgecolor="white", linewidth=0.5)
    silent_bar   = ax.bar(x, silent_demoted, bottom=[t+c for t, c in zip(typed_count, confuser_amb)],
                           color=S.DSFB_GRAY, width=w,
                           label="Demoted to silent", edgecolor="white", linewidth=0.5)

    # Direct labels per bar segment
    for xi, t in zip(x, typed_count):
        if t > 0:
            ax.text(xi, t/2, str(t), ha="center", va="center",
                    color="white", fontsize=11, fontweight="bold")
    for xi, t, c in zip(x, typed_count, confuser_amb):
        if c > 0:
            ax.text(xi, t + c/2, str(c), ha="center", va="center",
                    color="white", fontsize=11, fontweight="bold")

    ax.set_xticks(x)
    ax.set_xticklabels(phases, fontsize=8.5)
    ax.set_ylabel("Closed structural episodes (F-11)")
    ax.set_ylim(0, 4)
    ax.set_title("Anti-Hallucination Ladder progression — recall vs precision tradeoff on F-11",
                 loc="left", fontsize=11)
    ax.legend(loc="upper right", fontsize=8)
    ax.grid(axis="y", linewidth=0.4, alpha=0.5)
    ax.grid(axis="x", visible=False)

    # Caption-side annotation
    ax.text(-0.15, -0.20,
            "All four ladder rungs preserve the same three closed structural episodes; the strictness change is in the typed/ambiguous decision for episode 2 (margin-vs-confuser 0.065 < 0.10 default gate).",
            fontsize=8, color="#444", transform=ax.transAxes, wrap=True,
            ha="left", va="top")

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)
