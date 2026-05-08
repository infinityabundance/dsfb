"""Publication-quality figure style for DSFB-Debug.

Imported by every figure-rendering module. Sets matplotlib rcParams to
match Nature / Science / IEEE TVCG visual conventions:

- Sans-serif font (system default; Inter / Helvetica Neue / DejaVu Sans).
- 8 pt axis labels, 7 pt tick labels, 9 pt titles.
- Restrained palette: 1 hue for primary signal, 1 hue for reference,
  gray for context. Categorical palettes from cmcrameri.
- Thin, light grid lines or none.
- 300 dpi PNG, A4-friendly aspect ratios.
- bbox_inches='tight' on every save, so captions never clip into
  margins.
"""
from __future__ import annotations

import matplotlib as mpl
import matplotlib.pyplot as plt
try:
    import cmcrameri.cm  # noqa: F401  — registers the cmc.* colormaps
except ImportError:
    pass

# Preferred sans-serif stack; matplotlib falls back through these.
_FONT_STACK = [
    "Inter",
    "Helvetica Neue",
    "Helvetica",
    "Arial",
    "DejaVu Sans",
    "sans-serif",
]

# DSFB-Debug semantic palette.
# Primary brand hue (deep teal) for DSFB-Debug curves / typed episodes.
DSFB_PRIMARY = "#0F4C5C"
# Reference / baseline hue (soft slate) for SOTA reference lines.
DSFB_REFERENCE = "#4F6173"
# Accent for confusers / runner-ups (warm amber).
DSFB_ACCENT = "#E36414"
# Highlight for the load-bearing result (vivid teal).
DSFB_HIGHLIGHT = "#2A9D8F"
# Muted gray for context / non-data ink.
DSFB_GRAY = "#7C7E80"
# Background gray for shaded "below all baselines" zones.
DSFB_BG = "#F2EFE8"
# Ladder colour ramp (Phase 0 -> Phase 8 strictness).
DSFB_LADDER = ["#9CCC65", "#FFB300", "#F4511E", "#C62828"]

# Three-state grammar palette (Admissible/Boundary/Violation).
GRAMMAR_PALETTE = {
    "Admissible": "#F2EFE8",
    "Boundary":   "#E36414",
    "Violation":  "#9A031E",
}

# Four-state policy palette.
POLICY_PALETTE = {
    "Silent":   "#FFFFFF",
    "Watch":    "#A8DADC",
    "Review":   "#E36414",
    "Escalate": "#9A031E",
}


def install():
    """Apply DSFB-Debug rcParams to matplotlib globally.

    Idempotent — call at top of `render.py`. After this returns, every
    `plt.figure()` / `fig.savefig()` produces publication-quality output.
    """
    rcparams = {
        # Font selection
        "font.family": "sans-serif",
        "font.sans-serif": _FONT_STACK,
        "font.size": 9,

        # Axes / labels
        "axes.labelsize": 9,
        "axes.titlesize": 11,
        "axes.titleweight": "bold",
        "axes.labelweight": "regular",
        "axes.spines.top": False,
        "axes.spines.right": False,
        "axes.linewidth": 0.8,
        "axes.edgecolor": "#444",
        "axes.titlepad": 8,
        "axes.labelpad": 6,

        # Ticks
        "xtick.labelsize": 8,
        "ytick.labelsize": 8,
        "xtick.major.width": 0.6,
        "ytick.major.width": 0.6,
        "xtick.color": "#222",
        "ytick.color": "#222",
        "xtick.direction": "out",
        "ytick.direction": "out",

        # Grid (very light, behind data)
        "axes.grid": True,
        "grid.color": "#E5E5E5",
        "grid.linewidth": 0.5,
        "grid.linestyle": "-",
        "axes.axisbelow": True,

        # Legend
        "legend.frameon": False,
        "legend.fontsize": 8,
        "legend.labelspacing": 0.4,
        "legend.handlelength": 1.4,
        "legend.handletextpad": 0.6,

        # Saving
        "savefig.dpi": 300,
        "savefig.bbox": "tight",
        "savefig.pad_inches": 0.08,
        "figure.dpi": 100,
        "figure.facecolor": "white",
        "axes.facecolor": "white",

        # Lines
        "lines.linewidth": 1.6,
        "lines.markersize": 4.0,
        "lines.markeredgewidth": 0.8,

        # Patches
        "patch.linewidth": 0.6,
        "patch.edgecolor": "#444",
    }
    mpl.rcParams.update(rcparams)


def figsize(kind: str = "single") -> tuple[float, float]:
    """Return (width, height) in inches for common journal column widths.

    - "single"   : 88 mm wide, 65 mm tall (single-column journal).
    - "double"   : 180 mm wide, 110 mm tall (double-column journal).
    - "wide"     : 180 mm × 90 mm (lower aspect for forest plots).
    - "square"   : 110 mm × 110 mm (heatmaps).
    - "tall"     : 88 mm × 130 mm (vertical legends).
    - "twocol"   : 180 mm × 100 mm (two-column comparison).
    """
    mm_per_in = 25.4
    table = {
        "single": (88, 65),
        "double": (180, 110),
        "wide":   (180, 90),
        "square": (110, 110),
        "tall":   (88, 130),
        "twocol": (180, 100),
    }
    w, h = table.get(kind, (180, 110))
    return (w / mm_per_in, h / mm_per_in)


def add_caption(fig, text: str, y: float = -0.04, fontsize: int = 8):
    """Place a wrapped caption beneath the figure.

    Caption is positioned outside the axes area and wraps automatically;
    `bbox_inches='tight'` on save ensures no margin clipping.
    """
    fig.text(
        0.0, y, text,
        ha="left", va="top",
        fontsize=fontsize, color="#333",
        wrap=True,
        transform=fig.transFigure,
    )


def annotate_value(ax, x, y, label, dx=0, dy=0, **kwargs):
    """Direct-label a data point — preferred over legend lookups."""
    ax.annotate(
        label, xy=(x, y), xytext=(x + dx, y + dy),
        fontsize=8, color="#222", ha="left", va="center",
        **kwargs,
    )
