"""Per-fixture figures (replaces per-fixture grammar-state matrix etc.).

Two figures per fixture:
  1. residual_smallmult.png — per-signal residual time-series small multiples,
     with grammar-state shown as background colour bands and episode windows
     highlighted.
  2. summary_card.png       — single panel with key metrics + episode list.
"""
from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np

from . import _style as S


def render_per_fixture(fixture: dict, out_dir: Path):
    """Render the two per-fixture figures."""
    out_dir.mkdir(parents=True, exist_ok=True)
    render_residual_smallmult(fixture, out_dir / "01_residual_smallmult.png")
    render_summary_card(fixture, out_dir / "02_summary_card.png")
    render_episode_evidence(fixture, out_dir / "03_episode_evidence.png")


def render_residual_smallmult(fixture: dict, out_path: Path):
    """Per-signal residual time-series, small multiples, grammar-state bands."""
    nw = fixture["num_windows"]
    ns = fixture["num_signals"]
    raw_matrix = fixture.get("residual_matrix") or []
    # Replace JSON nulls (NaN) with np.nan
    matrix = np.array([np.nan if v is None else float(v) for v in raw_matrix],
                       dtype=float).reshape(nw, ns) if raw_matrix else None
    if matrix is None or matrix.size == 0:
        # Empty fixture — just emit a placeholder
        fig, ax = plt.subplots(figsize=S.figsize("single"))
        ax.text(0.5, 0.5, f"{fixture['manifest_name']}: no data", ha="center")
        ax.axis("off")
        fig.savefig(out_path, dpi=300, bbox_inches="tight")
        plt.close(fig)
        return

    # Build per-signal grammar-state timeline from eval_grid
    grammar_per_sig: list[list[str]] = [["Admissible"] * nw for _ in range(ns)]
    for ev in fixture.get("eval_grid", []):
        s = ev["s"]; w = ev["w"]
        if s < ns and w < nw:
            grammar_per_sig[s][w] = ev["grammar"]

    channels = fixture.get("channels", [])
    if not channels or len(channels) < ns:
        channels = [f"signal_{i}" for i in range(ns)]

    # Layout: nrows × ncols small multiples
    ncols = 4 if ns >= 8 else max(1, ns)
    nrows = (ns + ncols - 1) // ncols
    fig, axes = plt.subplots(nrows, ncols,
                              figsize=(min(ncols * 3.0, 11), max(nrows * 1.4, 2.2)),
                              squeeze=False, sharex=True)

    for idx in range(nrows * ncols):
        r, c = divmod(idx, ncols)
        ax = axes[r][c]
        if idx >= ns:
            ax.axis("off")
            continue
        signal_data = matrix[:, idx]
        # Mask NaN (was_imputed)
        finite_mask = np.isfinite(signal_data)
        ax.plot(np.where(finite_mask)[0], signal_data[finite_mask],
                color=S.DSFB_PRIMARY, linewidth=0.9)

        # Grammar-state background bands
        states = grammar_per_sig[idx]
        for w_idx, st in enumerate(states):
            if st == "Boundary":
                ax.axvspan(w_idx, w_idx + 1, color=S.GRAMMAR_PALETTE["Boundary"],
                           alpha=0.35, linewidth=0)
            elif st == "Violation":
                ax.axvspan(w_idx, w_idx + 1, color=S.GRAMMAR_PALETTE["Violation"],
                           alpha=0.35, linewidth=0)

        # Title with channel name
        chname = channels[idx]
        if len(chname) > 32:
            chname = chname[:29] + "…"
        ax.set_title(chname, fontsize=7.5, loc="left", pad=2)
        ax.tick_params(labelsize=6.5)
        ax.grid(False)
        for sp in ["top", "right"]:
            ax.spines[sp].set_visible(False)

    # Healthy-window separator on bottom row only
    healthy_end = fixture.get("healthy_window_end", 0)
    for c in range(ncols):
        axes[nrows - 1][c].axvline(healthy_end, color="#222",
                                    linewidth=0.7, linestyle=":", alpha=0.5)
        axes[nrows - 1][c].set_xlabel("Window", fontsize=7.5)

    # Big title above
    fig.suptitle(
        f"{fixture['manifest_name']}  —  per-signal residuals with grammar-state bands "
        f"(orange = Boundary, red = Violation; dotted line = healthy-window end)",
        fontsize=10, fontweight="bold", y=0.995,
    )

    fig.tight_layout(rect=[0, 0, 1, 0.97])
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


def render_summary_card(fixture: dict, out_path: Path):
    """One-panel metrics + episode list summary card."""
    fig, ax = plt.subplots(figsize=S.figsize("double"))
    ax.set_xlim(0, 10); ax.set_ylim(0, 8)
    ax.set_aspect("auto")
    ax.set_xticks([]); ax.set_yticks([])
    for s in ["top", "right", "left", "bottom"]:
        ax.spines[s].set_visible(False)

    m = fixture["metrics"]
    fu = fixture.get("fusion", {})

    title = fixture["manifest_name"]
    ax.text(0.1, 7.6, title, fontsize=14, fontweight="bold", color=S.DSFB_PRIMARY)
    ax.text(0.1, 7.1, f"{m['total_windows']} windows × {m['total_signals']} signals · "
                       f"{m['raw_anomaly_count']} raw alerts · {m['dsfb_episode_count']} typed episodes",
            fontsize=9, color="#444")

    # Metrics in two columns
    col1 = [
        ("RSCR", f"{m['rscr']:.2f}×"),
        ("Clean-window FP rate (DSFB structural)",
         f"{m['clean_window_false_episode_rate']:.4f}"),
        ("Investigation load reduction", f"{m['investigation_load_reduction_pct']:.1f}%"),
    ]
    col2 = [
        ("Detectors fused", f"{fu.get('detectors_used', '-')}"),
        ("Fusion FP rate (default config)",
         f"{fu.get('fusion_clean_window_fp_rate', 0):.4f}"),
        ("Theorem 9 deterministic replay",
         "✓ holds" if fu.get("deterministic_replay_holds", False) else "✗ FAILED"),
    ]
    y = 6.3
    for k, v in col1:
        ax.text(0.1, y, k, fontsize=8.5, color="#666")
        ax.text(0.1, y - 0.3, v, fontsize=11, fontweight="bold", color="#222")
        y -= 0.85
    y = 6.3
    for k, v in col2:
        ax.text(5.1, y, k, fontsize=8.5, color="#666")
        ax.text(5.1, y - 0.3, v, fontsize=11, fontweight="bold", color="#222")
        y -= 0.85

    # Episodes (if any)
    eps = fixture.get("episodes", [])
    ax.text(0.1, 3.5, f"Closed structural episodes: {len(eps)}",
            fontsize=10, fontweight="bold", color=S.DSFB_PRIMARY)
    if not eps:
        ax.text(0.1, 3.0,
                "No typed episodes — engine correctly silent on this fixture\n"
                "(steady-state slice or endoductive validator).",
                fontsize=9, color="#444", style="italic")
    else:
        y = 3.0
        for i, ep in enumerate(eps[:6]):
            motif = ep.get("matched_motif")
            motif_str = motif if motif else "Unknown"
            sig = ep.get("signature", {})
            ax.text(0.1, y,
                    f"  ep{ep['id']}  windows {ep['start_window']}-{ep['end_window']}  "
                    f"{ep['contributing_signals']} sigs  "
                    f"peak_slew {sig.get('peak_slew', 0):.3f}  "
                    f"→ {motif_str}",
                    fontsize=8.5, color="#222", family="DejaVu Sans Mono")
            y -= 0.40

    fig.tight_layout()
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


def render_episode_evidence(fixture: dict, out_path: Path):
    """Per-episode signature card (slew/duration/contributing/drift_dir/peak_state)."""
    eps = fixture.get("episodes", [])
    if not eps:
        # Skip — no episodes
        fig, ax = plt.subplots(figsize=S.figsize("single"))
        ax.text(0.5, 0.5,
                f"{fixture['manifest_name']}\nno episodes",
                ha="center", va="center", fontsize=11)
        ax.axis("off")
        fig.savefig(out_path, dpi=300, bbox_inches="tight")
        plt.close(fig)
        return

    fig, axes = plt.subplots(1, len(eps), figsize=(min(len(eps) * 3.5, 11), 3.5),
                              squeeze=False)
    for ax, ep in zip(axes[0], eps):
        sig = ep.get("signature", {})
        # Bar chart of normalised features
        labels = ["peak_slew", "duration", "signals", "drift_dir", "peak_state"]
        max_slew = max(abs(e["signature"]["peak_slew"]) for e in eps)
        max_dur = max(e["signature"]["duration_windows"] for e in eps) or 1
        max_sig = max(e["contributing_signals"] for e in eps) or 1

        drift_map = {"None": 0.0, "Negative": 0.33, "Oscillatory": 0.66, "Positive": 1.0}
        state_map = {"Admissible": 0.0, "Boundary": 0.5, "Violation": 1.0}
        vals = [
            abs(sig.get("peak_slew", 0)) / (max_slew if max_slew > 0 else 1),
            sig.get("duration_windows", 0) / max_dur,
            ep["contributing_signals"] / max_sig,
            drift_map.get(sig.get("drift_direction", "None"), 0),
            state_map.get(ep.get("peak_grammar", "Admissible"), 0),
        ]
        colors = [S.DSFB_PRIMARY, S.DSFB_HIGHLIGHT, S.DSFB_ACCENT, "#9A031E", "#5F0F40"]
        bars = ax.barh(labels[::-1], vals[::-1], color=colors[::-1],
                        edgecolor="white", linewidth=0.5, height=0.6)
        ax.set_xlim(0, 1.05)
        ax.set_xlabel("Normalised", fontsize=8)
        motif = ep.get("matched_motif") or "Unknown"
        ax.set_title(f"ep{ep['id']}  →  {motif}", fontsize=9, loc="left",
                     color=S.DSFB_PRIMARY)
        ax.tick_params(labelsize=7.5)
        ax.grid(axis="x", linewidth=0.3, alpha=0.5)
        ax.grid(axis="y", visible=False)

    fig.suptitle(f"{fixture['manifest_name']}  —  per-episode structural signature",
                  fontsize=10, fontweight="bold", y=0.995)
    fig.tight_layout(rect=[0, 0, 1, 0.95])
    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)
