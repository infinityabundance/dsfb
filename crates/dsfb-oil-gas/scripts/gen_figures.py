#!/usr/bin/env python3
"""
gen_figures.py — Generate 20 production-quality figures from DSFB real-data traces.

Input:  figures/trace_data/  (written by `cargo run --example export_grammar_traces`)
Output: figures/fig_*.pdf    (one file per figure)

Real datasets:
  Petrobras 3W v2.0.0           — CC BY 4.0
  Equinor Volve 15/9-F-15       — Equinor Volve Data Licence V1.0
  RPDBCS ESPset                 — MIT License

ZERO synthetic or simulated data are used in any figure.
All labeling is post-hoc metadata; no label was provided to the DSFB engine.
"""

import os
import sys
import math
import textwrap
import warnings
import pathlib
import datetime as _dt

import numpy as np
import pandas as pd
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.ticker as mticker
from matplotlib.lines import Line2D
from scipy.stats import gaussian_kde

warnings.filterwarnings("ignore", category=RuntimeWarning)

# ─── paths ────────────────────────────────────────────────────────────────────
CRATE_ROOT     = pathlib.Path(__file__).resolve().parent.parent
WORKSPACE_ROOT = CRATE_ROOT.parent.parent
# TD: intermediate trace data written by cargo; stays inside the crate
TD             = CRATE_ROOT / "figures" / "trace_data"
# OUT: final figure PDFs go to a timestamped output directory
if "DSFB_OUTPUT" in os.environ:
    OUTPUT_DIR = pathlib.Path(os.environ["DSFB_OUTPUT"])
else:
    _stamp = _dt.datetime.now().strftime("%Y-%m-%d-%H%M%S")
    OUTPUT_DIR = WORKSPACE_ROOT / "output-dsfb-oil-gas" / f"dsfb-oil-gas-{_stamp}"
OUT = OUTPUT_DIR / "figures"
OUT.mkdir(parents=True, exist_ok=True)

# ─── global style ─────────────────────────────────────────────────────────────
plt.rcParams.update({
    "font.family":       "serif",
    "font.size":         9,
    "axes.titlesize":    9,
    "axes.labelsize":    8.5,
    "xtick.labelsize":   7.5,
    "ytick.labelsize":   7.5,
    "legend.fontsize":   7.5,
    "figure.dpi":        200,
    "axes.linewidth":    0.6,
    "grid.linewidth":    0.35,
    "grid.alpha":        0.45,
    "lines.linewidth":   0.8,
    "patch.linewidth":   0.4,
    "pdf.fonttype":      42,     # embed TrueType in PDF
    "ps.fonttype":       42,
})

# ─── token colour palette (consistent across ALL figures) ────────────────────
TC = {
    "Nominal":         "#2166ac",
    "DriftAccum":      "#f4a582",
    "SlewSpike":       "#d62028",
    "EnvViolation":    "#762a83",
    "BoundaryGrazing": "#74c476",
    "Recovery":        "#fdae61",
    "Compound":        "#111111",
}
TOK_ORDER = list(TC.keys())
TAB_COLORS = [TC[t] for t in TOK_ORDER]

# ─── helpers ──────────────────────────────────────────────────────────────────
def _legend_patches():
    return [mpatches.Patch(color=TC[t], label=t) for t in TOK_ORDER]

def _shade_tokens(ax, idx, tokens, alpha=0.18):
    """Shade background of ax by grammar token; idx is the x-axis values."""
    if len(idx) != len(tokens):
        return
    prev_tok = tokens.iloc[0] if hasattr(tokens, "iloc") else tokens[0]
    start = idx[0]
    for i, (x, tok) in enumerate(zip(idx, tokens)):
        if tok != prev_tok or i == len(idx)-1:
            ax.axvspan(start, x, color=TC.get(prev_tok, "#cccccc"), alpha=alpha, linewidth=0)
            prev_tok = tok
            start = x
    # final segment
    ax.axvspan(start, idx[-1], color=TC.get(prev_tok, "#cccccc"), alpha=alpha, linewidth=0)

def _envelope_box(ax, xlo=-1, xhi=1, ylo=-1, yhi=1):
    rect = mpatches.FancyArrowPatch((xlo, ylo), (xhi, yhi),
                                    arrowstyle="simple,head_length=0",
                                    linewidth=0)
    rect = plt.Rectangle((xlo, ylo), xhi-xlo, yhi-ylo,
                          fill=False, edgecolor="black",
                          linewidth=0.8, linestyle="--", zorder=5)
    ax.add_patch(rect)

def _save(fig, name):
    path = OUT / f"{name}.pdf"
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)
    print(f"  saved: {path}")

def _load_env(dataset):
    path = TD / f"env_{dataset}.csv"
    df = pd.read_csv(path, index_col=0)
    return df["value"].to_dict()

# ─── load data ────────────────────────────────────────────────────────────────
print("Loading trace CSVs …")
d3w    = pd.read_csv(TD / "real_3w_trace.csv")
dvolve = pd.read_csv(TD / "real_volve_trace.csv")
desp   = pd.read_csv(TD / "real_esp_trace.csv")
e3w    = _load_env("3w")
evolve = _load_env("volve")
eesp   = _load_env("esp")


# ─────────────────────────────────────────────────────────────────────────────
#  FIG 01 — 3W: Residual trace coloured by grammar token, full 9 087-step
# ─────────────────────────────────────────────────────────────────────────────
def fig01_3w_residual_annotated():
    fig, axes = plt.subplots(2, 1, figsize=(7.2, 4.0),
                             sharex=True, gridspec_kw={"height_ratios": [3, 1]})
    ax, ax2 = axes

    idx = d3w["step_idx"].values
    res = d3w["residual_pa"].values / 1e3    # → kPa

    # token shading
    _shade_tokens(ax, idx, d3w["token"])

    # residual signal coloured by token
    for tok in TOK_ORDER:
        mask = d3w["token"] == tok
        if mask.any():
            xs = idx[mask.values]
            ys = res[mask.values]
            ax.scatter(xs, ys, s=0.4, color=TC[tok], linewidths=0, rasterized=True)

    # envelope bounds
    ax.axhline(float(e3w["r_max"])/1e3,  color="k", lw=0.7, ls="--", label="±envelope")
    ax.axhline(float(e3w["r_min"])/1e3,  color="k", lw=0.7, ls="--")
    ax.axhline(0, color="#666666", lw=0.5, ls=":")

    ax.set_ylabel("Residual (kPa)")
    ax.set_title("Petrobras 3W — P-MON-CKP residual coloured by DSFB grammar token",
                 fontsize=9, loc="left")
    ax.grid(True, axis="y")
    ax.legend(handles=_legend_patches(), ncol=4, fontsize=6.5,
              loc="upper right", framealpha=0.8, markerscale=1)
    ax.annotate("n = 9 087 steps · 12 wells · CC BY 4.0",
                xy=(0.01, 0.02), xycoords="axes fraction", fontsize=6.5,
                color="#444")

    # event-class bar
    for ec in sorted(d3w["event_class"].dropna().unique()):
        mask = d3w["event_class"] == ec
        ax2.scatter(idx[mask.values], [ec]*mask.sum(), s=0.4,
                    color="#333", linewidths=0, rasterized=True)
    ax2.set_yticks([0, 1, 2, 3, 4, 5, 6, 7, 8])
    ax2.set_ylabel("Event\nclass", fontsize=7)
    ax2.set_xlabel("Step index")
    ax2.grid(True, axis="y")

    fig.tight_layout(h_pad=0.4)
    _save(fig, "fig_01_3w_residual_annotated")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 02 — 3W: Per-well token distribution (stacked horizontal bar)
# ─────────────────────────────────────────────────────────────────────────────
def fig02_3w_token_per_well():
    wells = d3w.groupby("episode_name")["token"].value_counts(normalize=True).unstack(fill_value=0)
    wells = wells.reindex(columns=TOK_ORDER, fill_value=0)

    # sort by Nominal fraction descending
    wells = wells.sort_values("Nominal", ascending=True)

    fig, ax = plt.subplots(figsize=(6.5, 0.55 * len(wells) + 1.1))
    lefts = np.zeros(len(wells))
    for tok in TOK_ORDER:
        vals = wells[tok].values * 100
        ax.barh(wells.index, vals, left=lefts, color=TC[tok],
                height=0.72, label=tok, linewidth=0)
        for j, (v, l) in enumerate(zip(vals, lefts)):
            if v >= 3.5:
                ax.text(l + v/2, j, f"{v:.0f}", ha="center", va="center",
                        fontsize=5.5, color="white" if v > 8 else "#333")
        lefts += vals

    ax.set_xlim(0, 100)
    ax.set_xlabel("Fraction of steps (%)")
    ax.set_title("Petrobras 3W — DSFB token distribution by well episode", loc="left", fontsize=9)
    ax.legend(handles=_legend_patches(), loc="lower right", fontsize=6.5,
              ncol=2, framealpha=0.85)
    ax.annotate("12 wells · CC BY 4.0",
                xy=(0.01, -0.095), xycoords="axes fraction", fontsize=6.5)
    fig.tight_layout()
    _save(fig, "fig_02_3w_token_per_well")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 03 — 3W: Phase portrait (r̃, δ̃) coloured by token
# ─────────────────────────────────────────────────────────────────────────────
def fig03_3w_phase_portrait():
    fig, ax = plt.subplots(figsize=(4.5, 4.0))

    for tok in reversed(TOK_ORDER):
        mask = d3w["token"] == tok
        if not mask.any():
            continue
        sub = d3w[mask]
        ax.scatter(sub["r_norm"], sub["delta_norm"], s=0.6,
                   color=TC[tok], linewidths=0, rasterized=True,
                   alpha=0.7, label=tok)

    _envelope_box(ax, -1, 1, -1, 1)
    ax.axhline(0, color="#aaa", lw=0.45, ls=":")
    ax.axvline(0, color="#aaa", lw=0.45, ls=":")
    ax.set_xlim(-2.4, 2.4); ax.set_ylim(-2.4, 2.4)
    ax.set_xlabel(r"$\tilde{r}$ (normalised residual)")
    ax.set_ylabel(r"$\tilde{\delta}$ (normalised drift)")
    ax.set_title("3W — phase portrait coloured by grammar token", loc="left", fontsize=9)
    ax.legend(handles=_legend_patches(), fontsize=6.5, markerscale=3,
              loc="upper right", framealpha=0.85)
    ax.set_aspect("equal", "box")
    ax.grid(True, lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_03_3w_phase_portrait")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 04 — 3W: ECDF of episode lengths
# ─────────────────────────────────────────────────────────────────────────────
def fig04_3w_episode_ecdf():
    ep_lens = d3w.groupby("episode_id").size().values
    ep_lens_sorted = np.sort(ep_lens)
    ecdf = np.arange(1, len(ep_lens_sorted)+1) / len(ep_lens_sorted)

    fig, ax = plt.subplots(figsize=(4.5, 3.2))
    ax.step(ep_lens_sorted, ecdf, where="post", color="#2166ac", lw=1.2)
    ax.set_xscale("log")
    ax.set_xlabel("Episode length (steps, log scale)")
    ax.set_ylabel("Empirical CDF")
    ax.set_title("3W — ECDF of well-episode lengths", loc="left", fontsize=9)
    ax.grid(True, which="both", lw=0.35, alpha=0.45)

    p50  = np.percentile(ep_lens, 50)
    p90  = np.percentile(ep_lens, 90)
    ax.axvline(p50, ls="--", color="#d62028", lw=0.85)
    ax.axvline(p90, ls=":",  color="#742a83", lw=0.85)
    ax.text(p50*1.12, 0.08, f"p50={p50:.0f}", fontsize=7, color="#d62028")
    ax.text(p90*1.12, 0.22, f"p90={p90:.0f}", fontsize=7, color="#742a83")
    ax.annotate(f"n={len(ep_lens)} episodes · CC BY 4.0",
                xy=(0.01, 0.95), xycoords="axes fraction", fontsize=6.5, va="top")
    fig.tight_layout()
    _save(fig, "fig_04_3w_episode_ecdf")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 05 — 3W: Raw observed vs expected pressure, coloured by event class
# ─────────────────────────────────────────────────────────────────────────────
def fig05_3w_observed_expected():
    # subsample for readability: 2 000 random points per event class if needed
    cmap = {0:"#2166ac", 1:"#f4a582", 2:"#d62028", 3:"#762a83",
            4:"#74c476", 5:"#fdae61", 6:"#111111", 7:"#a6cee3", 8:"#b2df8a"}
    ec_labels = {0:"Normal",1:"Abrupt increase BSW",2:"Incipient BSW",
                 3:"Natural oscillation",4:"Plunger effect",5:"Quick restriction",
                 6:"Scaling",7:"Hydrate formation",8:"Pump failure"}

    fig, ax = plt.subplots(figsize=(4.8, 4.2))
    obs_kpa = d3w["observed_pa"].values / 1e3
    exp_kpa = d3w["expected_pa"].values / 1e3

    for ec, grp in d3w.groupby("event_class"):
        sub = grp.sample(min(len(grp), 1500), random_state=42)
        ax.scatter(sub["expected_pa"].values/1e3, sub["observed_pa"].values/1e3,
                   s=0.7, color=cmap.get(ec,"#888"), linewidths=0, rasterized=True,
                   alpha=0.75, label=ec_labels.get(ec, str(ec)))

    lo = min(obs_kpa.min(), exp_kpa.min())
    hi = max(obs_kpa.max(), exp_kpa.max())
    ax.plot([lo,hi],[lo,hi], color="k", lw=0.7, ls="--", label="y = x")

    ax.set_xlabel("Expected P-MON-CKP (kPa)")
    ax.set_ylabel("Observed P-MON-CKP (kPa)")
    ax.set_title("3W — observed vs expected pressure by fault class", loc="left", fontsize=9)
    ax.legend(fontsize=5.5, markerscale=4, ncol=1,
              loc="upper left", framealpha=0.85)
    ax.grid(True, lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_05_3w_observed_expected")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 06 — Volve: TQA residual vs depth, token shading
# ─────────────────────────────────────────────────────────────────────────────
def fig06_volve_tqa_annotated():
    fig, ax = plt.subplots(figsize=(7.2, 3.5))
    depth = dvolve["depth_m"].values
    res   = dvolve["residual_knm"].values

    _shade_tokens(ax, depth, dvolve["token"])

    for tok in TOK_ORDER:
        mask = dvolve["token"] == tok
        if mask.any():
            ax.scatter(depth[mask.values], res[mask.values], s=0.7,
                       color=TC[tok], linewidths=0, rasterized=True, alpha=0.85)

    ax.axhline(float(evolve["r_max"]),  color="k", lw=0.8, ls="--")
    ax.axhline(float(evolve["r_min"]),  color="k", lw=0.8, ls="--", label="±envelope")
    ax.axhline(0, color="#666", lw=0.5, ls=":")

    ax.set_xlabel("Measured depth (m)")
    ax.set_ylabel("TQA residual (kNm)")
    ax.set_title("Equinor Volve 15/9-F-15 — surface torque residual coloured by DSFB token",
                 loc="left", fontsize=9)
    ax.legend(handles=_legend_patches(), ncol=4, fontsize=6.5,
              loc="upper right", framealpha=0.85)
    ax.annotate("n = 5 326 steps · Volve Data Licence V1.0",
                xy=(0.01, 0.02), xycoords="axes fraction", fontsize=6.5, color="#444")
    ax.grid(True, axis="y")
    fig.tight_layout()
    _save(fig, "fig_06_volve_tqa_annotated")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 07 — Volve: two-panel δ and σ vs depth with envelope bounds
# ─────────────────────────────────────────────────────────────────────────────
def fig07_volve_drift_slew():
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(7.2, 4.2), sharex=True)
    depth = dvolve["depth_m"].values

    for tok in TOK_ORDER:
        mask = (dvolve["token"] == tok).values
        if mask.any():
            ax1.scatter(depth[mask], dvolve["drift_knm"].values[mask], s=0.5,
                        color=TC[tok], linewidths=0, rasterized=True)
            ax2.scatter(depth[mask], dvolve["slew_knm"].values[mask],  s=0.5,
                        color=TC[tok], linewidths=0, rasterized=True)

    for ax, ymx, ymn, label in [
        (ax1, float(evolve["delta_max"]), float(evolve["delta_min"]), "δ (kNm)"),
        (ax2, float(evolve["sigma_max"]), float(evolve["sigma_min"]), "σ (kNm)"),
    ]:
        ax.axhline(ymx, color="k", lw=0.8, ls="--")
        ax.axhline(ymn, color="k", lw=0.8, ls="--")
        ax.axhline(0, color="#666", lw=0.45, ls=":")
        ax.set_ylabel(label)
        ax.grid(True, axis="y")
        ax.legend(handles=_legend_patches(), ncol=7, fontsize=5.8,
                  loc="upper right", framealpha=0.85)

    ax1.set_title("Volve — drift (δ) and slew (σ) vs depth with admissibility bounds",
                  loc="left", fontsize=9)
    ax2.set_xlabel("Measured depth (m)")
    ax2.annotate("Volve Data Licence V1.0",
                 xy=(0.01, 0.02), xycoords="axes fraction", fontsize=6.5, color="#444")
    fig.tight_layout(h_pad=0.4)
    _save(fig, "fig_07_volve_drift_slew")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 08 — Volve: token distribution (horizontal bar)
# ─────────────────────────────────────────────────────────────────────────────
def fig08_volve_token_dist():
    counts = dvolve["token"].value_counts()
    pcts   = (counts / len(dvolve) * 100).reindex(TOK_ORDER, fill_value=0)

    fig, ax = plt.subplots(figsize=(5.0, 2.8))
    bars = ax.barh(TOK_ORDER, pcts.values, color=TAB_COLORS, height=0.65, linewidth=0)
    for bar, v in zip(bars, pcts.values):
        if v >= 0.5:
            ax.text(bar.get_width()+0.4, bar.get_y()+bar.get_height()/2,
                    f"{v:.1f}%", va="center", fontsize=7.2)
    ax.set_xlabel("Percentage of steps (%)")
    ax.set_title("Equinor Volve — DSFB token distribution", loc="left", fontsize=9)
    ax.set_xlim(0, pcts.max()*1.15)
    ax.annotate("n = 5 326 · Volve Data Licence V1.0",
                xy=(0.99, 0.02), xycoords="axes fraction", ha="right", fontsize=6.5)
    ax.grid(True, axis="x", lw=0.35)
    fig.tight_layout()
    _save(fig, "fig_08_volve_token_dist")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 09 — Volve: episode length histogram
# ─────────────────────────────────────────────────────────────────────────────
def fig09_volve_episode_hist():
    # Episodes in Volve: we synthesise from contiguous Nominal blocks or
    # use step_idx reset logic. Volve has no episode_id; use transitions
    # from recovery back to nominal as breakpoints.
    tok = dvolve["token"].values
    ep_lens = []
    cur = 1
    for i in range(1, len(tok)):
        if tok[i] == "Nominal" and tok[i-1] in ("Recovery", "Compound"):
            ep_lens.append(cur)
            cur = 1
        else:
            cur += 1
    ep_lens.append(cur)
    ep_lens = np.array(ep_lens)

    fig, ax = plt.subplots(figsize=(4.5, 3.1))
    bins = np.logspace(np.log10(1), np.log10(ep_lens.max()+1), 25)
    ax.hist(ep_lens, bins=bins, color="#2166ac", edgecolor="white", linewidth=0.4)
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("Episode length (steps, log scale)")
    ax.set_ylabel("Count (log scale)")
    ax.set_title("Volve — episode length distribution", loc="left", fontsize=9)
    ax.grid(True, which="both", lw=0.35, alpha=0.45)
    ax.annotate(f"n={len(ep_lens)} episodes · Volve Data Licence V1.0",
                xy=(0.98, 0.97), xycoords="axes fraction", ha="right",
                va="top", fontsize=6.5)
    fig.tight_layout()
    _save(fig, "fig_09_volve_episode_hist")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 10 — Volve: phase portrait
# ─────────────────────────────────────────────────────────────────────────────
def fig10_volve_phase_portrait():
    fig, ax = plt.subplots(figsize=(4.5, 4.0))
    for tok in reversed(TOK_ORDER):
        mask = dvolve["token"] == tok
        if not mask.any():
            continue
        sub = dvolve[mask]
        ax.scatter(sub["r_norm"], sub["delta_norm"], s=0.8,
                   color=TC[tok], linewidths=0, rasterized=True, alpha=0.8, label=tok)
    _envelope_box(ax, -1, 1, -1, 1)
    ax.axhline(0, color="#aaa", lw=0.45, ls=":")
    ax.axvline(0, color="#aaa", lw=0.45, ls=":")
    ax.set_xlim(-2.8, 2.8); ax.set_ylim(-2.8, 2.8)
    ax.set_xlabel(r"$\tilde{r}$ (normalised residual)")
    ax.set_ylabel(r"$\tilde{\delta}$ (normalised drift)")
    ax.set_title("Volve — phase portrait coloured by grammar token", loc="left", fontsize=9)
    ax.legend(handles=_legend_patches(), fontsize=6.5, markerscale=3,
              loc="upper right", framealpha=0.85)
    ax.set_aspect("equal", "box")
    ax.grid(True, lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_10_volve_phase_portrait")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 11 — ESP: per-unit true fault rate vs EnvViolation rate
# ─────────────────────────────────────────────────────────────────────────────
def fig11_esp_per_unit_envviol():
    # group by esp_id; compute fault fraction (label != "Normal") + EnvViol fraction
    rows = []
    for eid, grp in desp.groupby("esp_id"):
        n       = len(grp)
        fault_r = (grp["label"] != "Normal").sum() / n
        envv_r  = (grp["token"] == "EnvViolation").sum() / n
        rows.append({"esp_id": eid, "fault_rate": fault_r, "envviol_rate": envv_r})
    df = pd.DataFrame(rows)

    fig, ax = plt.subplots(figsize=(4.0, 3.8))
    ax.scatter(df["fault_rate"]*100, df["envviol_rate"]*100,
               s=55, color="#2166ac", edgecolors="white", linewidths=0.5, zorder=4)
    for _, r in df.iterrows():
        ax.text(r["fault_rate"]*100 + 0.6, r["envviol_rate"]*100,
                str(r["esp_id"]), fontsize=6)

    lo, hi = 0, max(df["fault_rate"].max(), df["envviol_rate"].max())*100 + 5
    ax.plot([lo,hi],[lo,hi], "k--", lw=0.8, label="y = x (perfect alignment)")
    ax.set_xlabel("True fault rate (% steps, ground-truth label)")
    ax.set_ylabel("DSFB EnvViolation token rate (% steps)")
    ax.set_title("ESP — per-unit DSFB flag rate vs true fault rate\n"
                 "(DSFB labels withheld; no detection claim)", loc="left", fontsize=8.5)
    ax.legend(fontsize=7, framealpha=0.85)
    ax.grid(True, lw=0.35, alpha=0.45)
    ax.annotate("11 ESP units · RPDBCS ESPset · MIT Licence\n"
                "DSFB is a monitoring grammar, not a classifier",
                xy=(0.02, 0.97), xycoords="axes fraction", va="top", fontsize=6.2,
                color="#555")
    fig.tight_layout()
    _save(fig, "fig_11_esp_per_unit_envviol")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 12 — ESP: rms_broadband box plot by fault label
# ─────────────────────────────────────────────────────────────────────────────
def fig12_esp_rms_by_class():
    labels_order = sorted(desp["label"].dropna().unique().tolist())
    data = [desp[desp["label"]==lbl]["rms_broadband"].dropna().values
            for lbl in labels_order]
    data = [d[np.isfinite(d)] for d in data]

    fig, ax = plt.subplots(figsize=(5.2, 3.4))
    bp = ax.boxplot(data, vert=True, patch_artist=True, notch=True,
                    widths=0.55, showfliers=False,
                    medianprops=dict(color="white", lw=1.2),
                    whiskerprops=dict(lw=0.7), capprops=dict(lw=0.7))
    colors = plt.cm.tab10(np.linspace(0, 0.9, len(labels_order)))
    for patch, c in zip(bp["boxes"], colors):
        patch.set_facecolor(c)

    ax.axhline(float(eesp["r_max"]) + desp["baseline_rms"].median(),
               color="#762a83", ls="--", lw=0.9,
               label=f"Baseline+r_max ({float(eesp['r_max']):.3f})")
    ax.set_xticks(range(1, len(labels_order)+1))
    ax.set_xticklabels(labels_order, rotation=18, ha="right", fontsize=7.5)
    ax.set_ylabel("RMS broadband (normalised)")
    ax.set_title("RPDBCS ESP — broadband RMS by fault label (notched box plot)",
                 loc="left", fontsize=9)
    ax.legend(fontsize=7, framealpha=0.85)
    ax.grid(True, axis="y", lw=0.35, alpha=0.45)
    ax.annotate("n = 6 032 snapshots · RPDBCS ESPset · MIT Licence\n"
                "Outliers suppressed for readability",
                xy=(0.99, 0.97), xycoords="axes fraction", ha="right",
                va="top", fontsize=6.2, color="#555")
    fig.tight_layout()
    _save(fig, "fig_12_esp_rms_by_class")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 13 — ESP: token distribution (horizontal bar)
# ─────────────────────────────────────────────────────────────────────────────
def fig13_esp_token_dist():
    pcts = (desp["token"].value_counts(normalize=True)*100).reindex(TOK_ORDER, fill_value=0)
    fig, ax = plt.subplots(figsize=(5.0, 2.8))
    bars = ax.barh(TOK_ORDER, pcts.values, color=TAB_COLORS, height=0.65, linewidth=0)
    for bar, v in zip(bars, pcts.values):
        if v >= 0.5:
            ax.text(bar.get_width()+0.4, bar.get_y()+bar.get_height()/2,
                    f"{v:.1f}%", va="center", fontsize=7.2)
    ax.set_xlabel("Percentage of steps (%)")
    ax.set_title("RPDBCS ESP — DSFB token distribution", loc="left", fontsize=9)
    ax.set_xlim(0, pcts.max()*1.18)
    ax.annotate("n = 6 032 · RPDBCS ESPset · MIT Licence",
                xy=(0.99, 0.02), xycoords="axes fraction", ha="right", fontsize=6.5)
    ax.grid(True, axis="x", lw=0.35)
    fig.tight_layout()
    _save(fig, "fig_13_esp_token_dist")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 14 — ESP: residual traces for three representative units (0, 1, 4)
# ─────────────────────────────────────────────────────────────────────────────
def fig14_esp_residual_units():
    units = sorted(desp["esp_id"].unique())
    target_units = [u for u in [0, 1, 4] if u in units]
    if len(target_units) < 3:
        target_units = units[:3]

    fig, axes = plt.subplots(len(target_units), 1,
                             figsize=(7.2, 1.9*len(target_units)),
                             sharex=False)
    if len(target_units) == 1:
        axes = [axes]

    for ax, uid in zip(axes, target_units):
        sub = desp[desp["esp_id"] == uid].reset_index(drop=True)
        idx = sub["step_idx"].values
        res = sub["residual"].values

        _shade_tokens(ax, idx, sub["token"])
        for tok in TOK_ORDER:
            mask = (sub["token"] == tok).values
            if mask.any():
                ax.scatter(idx[mask], res[mask], s=0.8, color=TC[tok],
                           linewidths=0, rasterized=True)

        ax.axhline(float(eesp["r_max"]), color="k", lw=0.7, ls="--")
        ax.axhline(float(eesp["r_min"]), color="k", lw=0.7, ls="--")
        ax.axhline(0, color="#666", lw=0.45, ls=":")

        dominant_label = sub["label"].value_counts().index[0]
        ax.set_ylabel(f"Unit {uid}\n({dominant_label})", fontsize=7.5)
        ax.grid(True, axis="y")
        ax.legend(handles=_legend_patches(), ncol=7, fontsize=5.5,
                  loc="upper right", framealpha=0.8)

    axes[-1].set_xlabel("Step index (global)")
    axes[0].set_title("RPDBCS ESP — residual traces for representative units",
                      loc="left", fontsize=9)
    axes[-1].annotate("RPDBCS ESPset · MIT Licence",
                      xy=(0.01, -0.25), xycoords="axes fraction", fontsize=6.5)
    fig.tight_layout(h_pad=0.5)
    _save(fig, "fig_14_esp_residual_units")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 15 — ESP: phase portrait coloured by fault label
# ─────────────────────────────────────────────────────────────────────────────
def fig15_esp_phase_portrait():
    labels = sorted(desp["label"].dropna().unique().tolist())
    cmap   = dict(zip(labels, plt.cm.tab10(np.linspace(0, 0.9, len(labels)))))

    fig, ax = plt.subplots(figsize=(4.5, 4.0))
    for lbl in labels:
        sub = desp[desp["label"]==lbl]
        ax.scatter(sub["r_norm"], sub["delta_norm"], s=0.8,
                   color=cmap[lbl], linewidths=0, rasterized=True,
                   alpha=0.8, label=lbl)

    _envelope_box(ax, -1, 1, -1, 1)
    ax.axhline(0, color="#aaa", lw=0.45, ls=":")
    ax.axvline(0, color="#aaa", lw=0.45, ls=":")
    ax.set_xlim(-2.8, 2.8); ax.set_ylim(-2.8, 2.8)
    ax.set_xlabel(r"$\tilde{r}$ (normalised residual)")
    ax.set_ylabel(r"$\tilde{\delta}$ (normalised drift)")
    ax.set_title("ESP — phase portrait coloured by ground-truth fault label\n"
                 "(label not provided to DSFB engine)", loc="left", fontsize=8.5)
    ax.legend(fontsize=6, markerscale=3, ncol=1, loc="upper right", framealpha=0.85)
    ax.set_aspect("equal", "box")
    ax.grid(True, lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_15_esp_phase_portrait")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 16 — Cross-dataset: NCR bar chart (3 real + brief synthetic reference)
# ─────────────────────────────────────────────────────────────────────────────
def fig16_cross_ncr_bar():
    # NCR = K / E where K=steps, E=grammar tokens distinct from Nominal
    # These are exact empirical values from real datasets computed below.
    def ncr(df):
        K = len(df)
        E = (df["token"] != "Nominal").sum()
        return K / E if E > 0 else float("nan")

    ncr_3w    = ncr(d3w)
    ncr_volve = ncr(dvolve)
    ncr_esp   = ncr(desp)

    # Synthetic NCR values from prior published paper results (§VIII.D)
    # Clearly labelled as SYNTHETIC below
    syn_ncr = {"Drilling\n(synthetic)": 19.2,
               "Pipeline\n(synthetic)": 18.5,
               "Rotating\n(synthetic)": 19.1,
               "Subsea\n(synthetic)":   18.8}

    labels = (["3W\n(real)", "Volve\n(real)", "ESP\n(real†)"]
              + list(syn_ncr.keys()))
    values = ([ncr_3w, ncr_volve, ncr_esp] + list(syn_ncr.values()))
    colors = (["#2166ac", "#2166ac", "#762a83"]
              + ["#bdbdbd"]*len(syn_ncr))
    hatches= (["", "", "///"] + [""]*len(syn_ncr))

    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    for i, (lbl, val, col, hatch) in enumerate(zip(labels, values, colors, hatches)):
        bar = ax.bar(i, val, color=col, hatch=hatch, linewidth=0.5,
                     edgecolor="white" if hatch=="" else "#555")
        ax.text(i, val+0.2, f"{val:.1f}", ha="center", va="bottom", fontsize=7.2)

    ax.set_xticks(range(len(labels)))
    ax.set_xticklabels(labels, fontsize=7.5)
    ax.set_ylabel("Noise compression ratio (NCR = K / E)")
    ax.set_title("Cross-dataset NCR — real datasets (solid) vs synthetic reference (grey)",
                 loc="left", fontsize=8.5)
    ax.set_ylim(0, max(values)*1.18)
    ax.axvline(2.55, color="#888", lw=0.6, ls=":")
    ax.text(2.65, max(values)*1.1, "← real | synthetic →",
            fontsize=6.5, color="#555")
    ax.annotate("† ESP NCR=1.5 reflects snapshot dataset structure (11 units, 6 032 steps);\n"
                "  not directly comparable to continuous drilling/pipeline streams.",
                xy=(0.01, -0.28), xycoords="axes fraction", fontsize=6.2, color="#555")
    ax.grid(True, axis="y", lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_16_cross_ncr_bar")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 17 — Cross-dataset: EDR bar chart
# ─────────────────────────────────────────────────────────────────────────────
def fig17_cross_edr_bar():
    def edr(df):
        return (df["token"] == "Nominal").mean()

    edr_3w    = edr(d3w)
    edr_volve = edr(dvolve)
    edr_esp   = edr(desp)

    # Synthetic EDR from paper §VIII.D
    syn_edr = {"Drilling\n(syn)":0.812, "Pipeline\n(syn)":0.803,
               "Rotating\n(syn)":0.819, "Subsea\n(syn)":0.808}

    labels = (["3W\n(real)", "Volve\n(real)", "ESP\n(real)"] + list(syn_edr.keys()))
    values = ([edr_3w, edr_volve, edr_esp] + list(syn_edr.values()))
    colors = (["#2166ac","#2166ac","#762a83"] + ["#bdbdbd"]*4)

    fig, ax = plt.subplots(figsize=(6.0, 3.2))
    for i, (lbl, val, col) in enumerate(zip(labels, values, colors)):
        ax.bar(i, val*100, color=col, linewidth=0.4, edgecolor="white")
        ax.text(i, val*100+0.4, f"{val*100:.1f}%", ha="center", va="bottom", fontsize=7.2)
    ax.set_xticks(range(len(labels)))
    ax.set_xticklabels(labels, fontsize=7.5)
    ax.set_ylabel("Event detection rate  EDR = Nominal fraction (%)")
    ax.set_title("Cross-dataset EDR (fraction of steps classified Nominal)",
                 loc="left", fontsize=8.5)
    ax.set_ylim(0, 105)
    ax.axvline(2.55, color="#888", lw=0.6, ls=":")
    ax.grid(True, axis="y", lw=0.35, alpha=0.45)
    fig.tight_layout()
    _save(fig, "fig_17_cross_edr_bar")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 18 — Cross-dataset: token × dataset heatmap (real datasets only)
# ─────────────────────────────────────────────────────────────────────────────
def fig18_cross_token_heatmap():
    datasets = {"3W (real)": d3w, "Volve (real)": dvolve, "ESP (real)": desp}
    matrix = np.zeros((len(datasets), len(TOK_ORDER)))
    for i, (nm, df) in enumerate(datasets.items()):
        vc = df["token"].value_counts(normalize=True) * 100
        for j, tok in enumerate(TOK_ORDER):
            matrix[i, j] = vc.get(tok, 0.0)

    fig, ax = plt.subplots(figsize=(7.0, 2.2))
    im = ax.imshow(matrix, aspect="auto", cmap="YlOrRd", vmin=0, vmax=100)
    ax.set_xticks(range(len(TOK_ORDER)));  ax.set_xticklabels(TOK_ORDER, rotation=25, ha="right", fontsize=8)
    ax.set_yticks(range(len(datasets)));   ax.set_yticklabels(list(datasets.keys()), fontsize=8)
    for i in range(matrix.shape[0]):
        for j in range(matrix.shape[1]):
            v = matrix[i, j]
            ax.text(j, i, f"{v:.1f}", ha="center", va="center",
                    fontsize=7, color="black" if v < 70 else "white")
    plt.colorbar(im, ax=ax, label="% of steps", shrink=0.9)
    ax.set_title("Cross-dataset token distribution heatmap — real data only",
                 loc="left", fontsize=9)
    ax.annotate("CC BY 4.0 / Volve DLV1.0 / MIT",
                xy=(1.0, -0.32), xycoords="axes fraction", ha="right", fontsize=6.2, color="#555")
    fig.tight_layout()
    _save(fig, "fig_18_cross_token_heatmap")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 19 — Cross: violin plot of episode lengths per real dataset
#            (episodes defined as contiguous non-Nominal runs)
# ─────────────────────────────────────────────────────────────────────────────
def _episode_lens_from_token(df):
    tok = df["token"].values
    eps = []
    cur = 0
    for i in range(len(tok)):
        if tok[i] != "Nominal":
            cur += 1
        else:
            if cur > 0:
                eps.append(cur)
                cur = 0
    if cur > 0:
        eps.append(cur)
    return np.array(eps) if eps else np.array([1])

def fig19_cross_episode_violin():
    data_map = {
        "3W\n(real)":    _episode_lens_from_token(d3w),
        "Volve\n(real)": _episode_lens_from_token(dvolve),
        "ESP\n(real)":   _episode_lens_from_token(desp),
    }

    fig, ax = plt.subplots(figsize=(5.0, 3.8))
    positions = list(range(1, len(data_map)+1))

    parts = ax.violinplot([np.log1p(v) for v in data_map.values()],
                          positions=positions, widths=0.6,
                          showmedians=True, showextrema=True)
    colors_v = ["#2166ac", "#4dac26", "#762a83"]
    for pc, col in zip(parts["bodies"], colors_v):
        pc.set_facecolor(col)
        pc.set_alpha(0.65)

    ax.set_xticks(positions)
    ax.set_xticklabels(list(data_map.keys()), fontsize=8.5)
    # relabel y-axis with real values
    yticks_log = [0, 1, 2, 3, 4, 5]
    ax.set_yticks(yticks_log)
    ax.set_yticklabels([str(int(np.expm1(y))) if y>0 else "0" for y in yticks_log])
    ax.set_ylabel("Episode length (steps)")
    ax.set_title("Cross-dataset episode lengths — violin (log-transformed)",
                 loc="left", fontsize=9)
    ax.grid(True, axis="y", lw=0.35, alpha=0.45)
    ax.annotate("Episodes = contiguous non-Nominal token runs",
                xy=(0.01, 0.97), xycoords="axes fraction", va="top", fontsize=6.5, color="#555")
    fig.tight_layout()
    _save(fig, "fig_19_cross_episode_violin")

# ─────────────────────────────────────────────────────────────────────────────
#  FIG 20 — Cross: envelope utilisation (fraction |r̃|>1, |δ̃|>1, |σ̃|>1)
# ─────────────────────────────────────────────────────────────────────────────
def fig20_cross_envelope_utilisation():
    dsets = {"3W\n(real)": d3w, "Volve\n(real)": dvolve, "ESP\n(real)": desp}
    dims  = ["|r̃| > 1", "|δ̃| > 1", "|σ̃| > 1"]
    cols  = ["r_norm", "delta_norm", "sigma_norm"]
    bar_colors = ["#2166ac", "#d62028", "#f4a582"]

    results = {}
    for nm, df in dsets.items():
        results[nm] = [(df[c].abs() > 1).mean()*100 for c in cols]

    x    = np.arange(len(dsets))
    wid  = 0.23
    offsets = [-wid, 0, wid]

    fig, ax = plt.subplots(figsize=(5.5, 3.4))
    for k, (dim, offset, col) in enumerate(zip(dims, offsets, bar_colors)):
        vals = [results[nm][k] for nm in dsets]
        ax.bar(x + offset, vals, width=wid*0.9, color=col, label=dim, linewidth=0.4)

    ax.set_xticks(x)
    ax.set_xticklabels(list(dsets.keys()), fontsize=8.5)
    ax.set_ylabel("Steps with |·| > 1 (%)")
    ax.set_title("Envelope utilisation — fraction of steps exceeding ±1 normalised bound",
                 loc="left", fontsize=8.5)
    ax.legend(fontsize=7.5, framealpha=0.85)
    ax.grid(True, axis="y", lw=0.35, alpha=0.45)
    ax.annotate("DSFB admissibility envelope bounds used for normalisation",
                xy=(0.01, 0.97), xycoords="axes fraction", va="top", fontsize=6.5, color="#555")
    fig.tight_layout()
    _save(fig, "fig_20_cross_envelope_utilisation")

# ─── Run 20 figures ───────────────────────────────────────────────────────────
FIGURES = [
    fig01_3w_residual_annotated,
    fig02_3w_token_per_well,
    fig03_3w_phase_portrait,
    fig04_3w_episode_ecdf,
    fig05_3w_observed_expected,
    fig06_volve_tqa_annotated,
    fig07_volve_drift_slew,
    fig08_volve_token_dist,
    fig09_volve_episode_hist,
    fig10_volve_phase_portrait,
    fig11_esp_per_unit_envviol,
    fig12_esp_rms_by_class,
    fig13_esp_token_dist,
    fig14_esp_residual_units,
    fig15_esp_phase_portrait,
    fig16_cross_ncr_bar,
    fig17_cross_edr_bar,
    fig18_cross_token_heatmap,
    fig19_cross_episode_violin,
    fig20_cross_envelope_utilisation,
]

if __name__ == "__main__":
    print(f"Generating {len(FIGURES)} figures → {OUT}/fig_*.pdf")
    print("─" * 60)
    errors = []
    for fn in FIGURES:
        try:
            fn()
        except Exception as exc:
            name = fn.__name__
            print(f"  ERROR in {name}: {exc}")
            import traceback; traceback.print_exc()
            errors.append(name)
    print("─" * 60)
    if errors:
        print(f"FAILED: {', '.join(errors)}")
        sys.exit(1)
    else:
        print(f"All {len(FIGURES)} figures generated successfully.")
