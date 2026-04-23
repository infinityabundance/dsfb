#!/usr/bin/env python3
"""
DSFB-RF Real-Dataset Figure Bank — 80 figures (fig_69 … fig_148)
================================================================

Renders 10 figures per slice × 8 real-world slices loaded from
`figure_data_real.json` (produced by `cargo run --release --example
generate_figures_real --features std,serde,real_figures`).

**Positioning (load-bearing, every caption):** DSFB does **not** compete
with, replace, or detect-earlier-than the upstream chains feeding it
(matched filter, CFAR, AGC, PLL, channel estimator, scheduler,
beamformer, beam-tracker). DSFB is an *observer* that **augments**
those chains by structuring the residuals they already compute and
usually discard.  Every figure caption must:

1. Name the upstream producer of the residual.
2. Frame DSFB as the structural interpreter, not an adversary.
3. End with a provenance stamp: `[<slice> <provenance>, N=<n>]`.
4. Carry one slice-specific *non-claim* (not a benchmark / not a
   reproduction / not a replacement / no "earlier than" framing).

Usage:
    python3 scripts/figures_real.py \
        --data dsfb-rf-output/figure_data_real.json \
        --out  dsfb-rf-output/dsfb-rf-real-<ts>/figs
"""

# ─── stdlib / third-party ──────────────────────────────────────────────
import argparse
import json
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle
from mpl_toolkits.mplot3d import Axes3D  # noqa: F401
import numpy as np

# Import shared style primitives from the 67-figure renderer so both
# banks share a single visual identity.
sys.path.insert(0, str(Path(__file__).parent))
from figures_all import (  # type: ignore  # noqa: E402
    C_ADMISSIBLE, C_BOUNDARY, C_VIOLATION, C_DSFB, C_COMPARATOR,
    C_NEUTRAL, C_HIGHLIGHT, grammar_color,
)

plt.rcParams.update({
    "font.family":        "serif",
    "font.size":          9,
    "axes.titlesize":     9,
    "axes.labelsize":     9,
    "legend.fontsize":    7,
    "xtick.labelsize":    7,
    "ytick.labelsize":    7,
    "axes.linewidth":     0.8,
    "lines.linewidth":    1.0,
    "grid.linewidth":     0.4,
    "grid.alpha":         0.25,
    "grid.linestyle":     ":",
    "axes.grid":          True,
    "figure.dpi":         150,
    "savefig.dpi":        150,
    "savefig.bbox":       "tight",
    "savefig.pad_inches": 0.05,
})


# ═══════════════════════════════════════════════════════════════════════
# Caption + save helpers
# ═══════════════════════════════════════════════════════════════════════

def stamp(common: dict) -> str:
    return f"[{common['name']} {common['provenance']}, N={common['n']}]"


def caption(fig, common: dict, short_title: str, non_claim: str | None = None) -> None:
    """Render the standard augment-not-replace caption under the plot."""
    line1 = f"Upstream: {common['upstream_producer']}."
    line2 = "DSFB role: structural interpreter of the residual stream."
    line3 = stamp(common)
    if non_claim:
        line3 = f"Non-claim: {non_claim}  {line3}"
    fig.text(0.5, -0.02,
             f"{short_title}\n{line1}  {line2}\n{line3}",
             ha="center", va="top", fontsize=6.5, wrap=True,
             color="#444444")


def save(fig, idx: int, slice_name: str, figname: str, out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)
    stub = f"fig_{idx:03d}_{slice_name}_{figname}"
    for ext in ("pdf", "png"):
        fig.savefig(out_dir / f"{stub}.{ext}", format=ext)
    plt.close(fig)
    print(f"  Saved fig {idx}: {stub}")


def pick_non_claim(common: dict, idx_mod: int = 0) -> str:
    nc = common.get("non_claims") or []
    if not nc:
        return "Not a benchmark reproduction."
    return nc[idx_mod % len(nc)]


# ═══════════════════════════════════════════════════════════════════════
# Primitive plot-blocks reused across slices
# ═══════════════════════════════════════════════════════════════════════

def _safe_rho(common: dict) -> float:
    v = common.get("rho")
    try:
        fv = float(v) if v is not None else float("nan")
    except (TypeError, ValueError):
        fv = float("nan")
    return fv


def _safe_arr(seq) -> np.ndarray:
    """Coerce a JSON sequence (possibly containing None/NaN) into a float
    array, replacing non-finite entries with NaN (matplotlib handles NaN)."""
    out = np.array([float(x) if x is not None else float("nan") for x in seq],
                   dtype=float)
    return out


def _post_cal(common: dict, stream_key: str) -> np.ndarray:
    return _safe_arr(common[stream_key])


def plot_grammar_timeline(ax, common: dict, title_suffix: str = "") -> None:
    """Coloured grammar-state rugplot across post-calibration samples."""
    states = common["grammar_states"]
    norms = _safe_arr(common["norms"])
    hw = common["healthy_window_size"]
    post = norms[hw:]
    colors = [grammar_color(s) for s in states]
    ax.plot(post, color=C_DSFB, lw=0.6, alpha=0.85, zorder=2)
    # State rug along the bottom
    for k, c in enumerate(colors):
        ax.axvspan(k - 0.5, k + 0.5, ymin=0, ymax=0.06,
                   color=c, alpha=0.55, lw=0)
    rho = _safe_rho(common)
    if np.isfinite(rho):
        ax.axhline(rho, color=C_VIOLATION, lw=0.7, ls="--", alpha=0.7,
                   label=f"rho = {rho:.3f}")
        ax.legend(loc="best", framealpha=0.75)
    ax.set_xlabel("post-calibration sample index")
    ax.set_ylabel("residual norm")
    ax.set_title(f"Grammar state timeline on residual{title_suffix}")


def plot_sign_scatter3d(fig, common: dict, title: str,
                        color_series=None, color_label: str = ""):
    raw = common["sign_tuples"]
    # Each row is [norm, drift, slew]; guard for None.
    sign = np.array(
        [[float(x) if x is not None else float("nan") for x in row] for row in raw],
        dtype=float,
    )
    ax = fig.add_subplot(111, projection="3d")
    if color_series is None:
        color_series = [grammar_color(s) for s in common["grammar_states"]]
        sc = ax.scatter(sign[:, 0], sign[:, 1], sign[:, 2],
                        c=color_series, s=4, alpha=0.65)
    else:
        sc = ax.scatter(sign[:, 0], sign[:, 1], sign[:, 2],
                        c=color_series, s=4, alpha=0.65, cmap="viridis")
        cb = fig.colorbar(sc, ax=ax, shrink=0.6, pad=0.08)
        cb.set_label(color_label, fontsize=7)
    ax.set_xlabel("||r||");  ax.set_ylabel("drift");  ax.set_zlabel("slew")
    ax.set_title(title)


def plot_dsa_trace(ax, common: dict) -> None:
    dsa = _safe_arr(common["dsa_scores"])
    ewma = _safe_arr(common["ewma_trace"])
    ax.plot(dsa, color=C_DSFB, lw=0.8, label="DSA score (DSFB)")
    ax2 = ax.twinx()
    ax2.plot(ewma, color=C_COMPARATOR, lw=0.6, alpha=0.55, label="EWMA (context)")
    ewma_thr = common.get("ewma_threshold")
    try:
        if ewma_thr is not None and np.isfinite(float(ewma_thr)):
            ax2.axhline(float(ewma_thr), color=C_COMPARATOR, lw=0.6, ls=":",
                        alpha=0.6)
    except (TypeError, ValueError):
        pass
    ax.set_xlabel("post-calibration sample")
    ax.set_ylabel("DSA (DSFB)")
    ax2.set_ylabel("EWMA (context)")
    ax.set_title("DSA score — context: EWMA comparator (not an adversary)")
    lines1, labels1 = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, labels1 + labels2, loc="best", framealpha=0.75)


def plot_envelope_calibration(ax, common: dict) -> None:
    healthy = _safe_arr(common["healthy_norms"])
    healthy_clean = healthy[np.isfinite(healthy)]
    if healthy_clean.size:
        ax.hist(healthy_clean, bins=24, color=C_ADMISSIBLE, alpha=0.75,
                edgecolor="white", lw=0.4, label="healthy-window samples")
    rho = _safe_rho(common)
    if np.isfinite(rho):
        ax.axvline(rho, color=C_VIOLATION, lw=1.2, label=f"rho = {rho:.3f}")
    if healthy_clean.size:
        mu = float(healthy_clean.mean())
        ax.axvline(mu, color=C_NEUTRAL, lw=0.8, ls="--", label=f"μ = {mu:.3f}")
    ax.set_xlabel("||r|| (residual norm)")
    ax.set_ylabel("count")
    ax.set_title("Envelope calibration from healthy window")
    ax.legend(loc="best", framealpha=0.75)


def plot_compression(ax, common: dict) -> None:
    raw = common["raw_boundary_count"]
    dsfb = common["dsfb_episode_count"]
    ax.bar(["raw 3σ events\n(context)", "DSFB episodes"], [raw, dsfb],
           color=[C_COMPARATOR, C_DSFB], alpha=0.8, width=0.55)
    ax.set_ylabel("events / episodes")
    ax.set_title(
        f"Review-surface compression  (ratio ≈ {(raw/max(1,dsfb)):.1f}×)"
    )
    for i, v in enumerate([raw, dsfb]):
        ax.text(i, v, f" {v}", ha="center", va="bottom", fontsize=8)


def plot_perm_entropy(ax, common: dict) -> None:
    pe = _safe_arr(common["perm_entropy"])
    ax.plot(pe, color=C_DSFB, lw=0.8)
    ax.axhline(0.5, color=C_NEUTRAL, lw=0.5, ls=":")
    ax.set_xlabel("post-calibration sample")
    ax.set_ylabel("normalised permutation entropy")
    ax.set_title("Permutation entropy profile — complexity of the residual")


def plot_fisher_rao_drift(ax, common: dict) -> None:
    norms = _post_cal(common, "norms")[common["healthy_window_size"]:]
    # Chunk into windows of 64 and report (μ, σ) per window.
    n = len(norms);  w = max(32, n // 40)
    mus, sigmas = [], []
    for i in range(0, n - w, w):
        seg = norms[i:i + w]
        mus.append(seg.mean())
        sigmas.append(max(1e-6, seg.std()))
    mus = np.array(mus);  sigmas = np.array(sigmas)
    ax.plot(mus, sigmas, "-", color=C_DSFB, lw=0.8, alpha=0.85)
    ax.scatter(mus, sigmas, c=np.arange(len(mus)), cmap="viridis",
               s=10, zorder=3)
    ax.set_xlabel("μ (residual-norm mean per window)")
    ax.set_ylabel("σ (residual-norm std per window)")
    ax.set_title("Fisher-Rao drift — statistical-manifold geometry")


def plot_detectability(ax, common: dict) -> None:
    d = common["detectability"]
    # serde_json serialises NaN/∞ as null — coerce to 0 for display.
    def _f(x):
        try:
            return float(x) if x is not None else 0.0
        except (TypeError, ValueError):
            return 0.0
    vals = [_f(d.get("delta_0")), _f(d.get("alpha")), _f(d.get("kappa"))]
    ax.bar(["Δ₀", "α", "κ"], vals,
           color=[C_DSFB, C_ADMISSIBLE, C_BOUNDARY], alpha=0.8, width=0.45)
    tau = d.get("tau_upper")
    sat = d.get("bound_satisfied")
    sub = "τ_upper = ∞ (no upper bound)" if tau is None else f"τ_upper = {float(tau):.2f}"
    if sat is not None:
        sub += f"   bound satisfied: {sat}"
    if d.get("delta_0") is None:
        sub += "   (Δ₀ non-finite; post-cal max < ρ)"
    ax.set_title(f"Detectability bound  ·  {sub}")
    ax.set_ylabel("value")


def plot_attractor(ax, common: dict, tau: int = 8) -> None:
    norms = _post_cal(common, "norms")[common["healthy_window_size"]:]
    if len(norms) <= 2 * tau:
        tau = max(1, len(norms) // 8)
    x = norms[:-2 * tau];  y = norms[tau:-tau];  z = norms[2 * tau:]
    ax.plot(x, y, color=C_DSFB, lw=0.3, alpha=0.8)
    sc = ax.scatter(x, y, c=z, s=2, cmap="viridis", alpha=0.7)
    plt.colorbar(sc, ax=ax, shrink=0.75, label=f"‖r[k+2τ]‖  (τ={tau})")
    ax.set_xlabel("||r[k]||");  ax.set_ylabel(f"||r[k+τ]||")
    ax.set_title("Delay-embedded phase portrait")


def plot_tda_barcode(ax, common: dict) -> None:
    """Lightweight TDA-style barcode: persistence of residual-norm
    excursions above rho across sliding windows.  Not a full Rips/PH
    computation — descriptive only."""
    norms = _post_cal(common, "norms")[common["healthy_window_size"]:]
    rho = _safe_rho(common)
    if not np.isfinite(rho):
        ax.text(0.5, 0.5, "rho unavailable — envelope calibration did not converge",
                ha="center", va="center", transform=ax.transAxes,
                color=C_NEUTRAL, fontsize=8)
        ax.set_title("Super-rho persistence — (skipped)")
        return
    bars = []  # (birth, death)
    i = 0
    while i < len(norms):
        if np.isfinite(norms[i]) and norms[i] > rho:
            j = i
            while j < len(norms) and np.isfinite(norms[j]) and norms[j] > rho:
                j += 1
            bars.append((i, j))
            i = j
        else:
            i += 1
    # Plot
    for k, (a, b) in enumerate(bars[:40]):
        ax.plot([a, b], [k, k], color=C_VIOLATION, lw=1.4, alpha=0.75)
    if not bars:
        ax.text(0.5, 0.5, "no super-rho excursions in the slice",
                ha="center", va="center", transform=ax.transAxes,
                color=C_NEUTRAL, fontsize=8)
    ax.set_xlabel("post-calibration sample")
    ax.set_ylabel("persistence bar index")
    ax.set_title("Super-rho persistence — sliding-window residual excursions")


# ═══════════════════════════════════════════════════════════════════════
# RadioML (figs 69 – 78)
# ═══════════════════════════════════════════════════════════════════════

MOD_NAMES = [
    "OOK","4ASK","8ASK","BPSK","QPSK","8PSK","16PSK","32PSK",
    "16APSK","32APSK","64APSK","128APSK","16QAM","32QAM","64QAM",
    "128QAM","256QAM","AM-SSB-WC","AM-SSB-SC","AM-DSB-WC","AM-DSB-SC",
    "FM","GMSK","OQPSK",
]

def radioml_figures(data, out_dir):
    block = data.get("radioml")
    if not block or block.get("skipped"):
        print(f"[SKIP] radioml: {block.get('reason') if block else 'missing'}")
        return
    common = block["common"]
    per_mod = block["per_mod_norms"]
    fr = np.array(block["fisher_rao_matrix"])
    mod_class = block["mod_class"]
    snr_db = block["snr_db"]
    name = common["name"]

    # fig_69 — per-modulation residual envelope, top-6 by N
    sizes = [len(x) for x in per_mod]
    top = np.argsort(sizes)[::-1][:6]
    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    for i in top:
        arr = np.asarray(per_mod[i])
        if arr.size == 0:
            continue
        ax.hist(arr, bins=24, alpha=0.35,
                label=MOD_NAMES[i] if i < len(MOD_NAMES) else f"mod_{i}")
    ax.set_xlabel("||r|| (amplitude-template residual)")
    ax.set_ylabel("count")
    ax.set_title("Per-modulation envelope — top 6 mods")
    ax.legend(loc="best", framealpha=0.75, ncol=2)
    caption(fig, common, "Exhibit: what the demodulator was quietly producing.",
            pick_non_claim(common, 0))
    save(fig, 69, name, "permod_envelope", out_dir)

    # fig_70 — grammar state distribution heatmap (mod × state)
    states = common["grammar_states"]
    unique = ["Admissible", "Boundary", "Violation"]
    counts = np.zeros((len(per_mod), 3))
    n_post = len(states)
    hw = common["healthy_window_size"]
    # map state stream back onto capture index via caps×samp/len heuristic;
    # approximate: one state per capture tail.
    for k, s in enumerate(states):
        cap = min(k * len(per_mod) // max(1, n_post), len(mod_class) - 1)
        mi = mod_class[cap]
        if "Admissible" in s: counts[mi][0] += 1
        elif "Violation" in s: counts[mi][2] += 1
        else: counts[mi][1] += 1
    row_sum = counts.sum(axis=1, keepdims=True)
    row_sum[row_sum == 0] = 1.0
    norm_mat = counts / row_sum
    fig, ax = plt.subplots(figsize=(6.4, 3.6))
    im = ax.imshow(norm_mat, aspect="auto", cmap="viridis")
    ax.set_yticks(range(len(per_mod)))
    ax.set_yticklabels(
        [MOD_NAMES[i] if i < len(MOD_NAMES) else f"m{i}" for i in range(len(per_mod))],
        fontsize=6)
    ax.set_xticks([0, 1, 2]); ax.set_xticklabels(unique)
    ax.set_title("Grammar-state fingerprint (row-normalised)")
    plt.colorbar(im, ax=ax, shrink=0.85, label="fraction")
    caption(fig, common, "Structural fingerprint of the discarded residual, per mod.",
            pick_non_claim(common, 1))
    save(fig, 70, name, "grammar_fingerprint", out_dir)

    # fig_71 — SNR × mod sweep (mean residual norm per bin)
    uniq_snr = sorted(set(snr_db))
    # Pair per-mod residuals with their capture SNR.
    mod_caps = {i: [c for c, m in enumerate(mod_class) if m == i] for i in range(len(per_mod))}
    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    for i in top:
        means = []
        caps_i = mod_caps[i]
        snrs_i = [snr_db[c] for c in caps_i]
        for s in uniq_snr:
            sel = [per_mod[i][k] for k, cs in enumerate(snrs_i) if cs == s]
            means.append(float(np.mean(sel)) if sel else np.nan)
        ax.plot(uniq_snr, means, "o-", label=MOD_NAMES[i] if i < len(MOD_NAMES) else f"m{i}")
    ax.set_xlabel("SNR (dB)")
    ax.set_ylabel("mean ||r||")
    ax.set_title("SNR × modulation residual mean (context sweep)")
    ax.legend(loc="best", framealpha=0.75, ncol=2)
    caption(fig, common, "Residual structure shift with SNR.",
            pick_non_claim(common, 2))
    save(fig, 71, name, "snr_mod_sweep", out_dir)

    # fig_72 — review-surface compression
    fig, ax = plt.subplots(figsize=(5.6, 3.2))
    plot_compression(ax, common)
    caption(fig, common, "Raw 3σ context vs. DSFB episode count.",
            pick_non_claim(common, 3))
    save(fig, 72, name, "review_compression", out_dir)

    # fig_73 — sign-tuple 3D by mod family (colour by mod)
    fig = plt.figure(figsize=(5.8, 4.0))
    # Map each post-cal sample to a capture approximation → colour by mod.
    n_post = len(common["grammar_states"])
    colors = [mod_class[min(k * len(mod_class) // max(1, n_post), len(mod_class) - 1)]
              for k in range(n_post)]
    plot_sign_scatter3d(fig, common,
                        "Sign-tuple 3D coloured by modulation",
                        color_series=colors, color_label="modulation idx")
    caption(fig, common, "Sign-tuple manifold of the demodulator residual.",
            pick_non_claim(common, 0))
    save(fig, 73, name, "sign_manifold", out_dir)

    # fig_74 — DSA recurrence-style heatmap: per-mod DSA mean histogram
    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    mod_dsa = [[] for _ in per_mod]
    dsa = common["dsa_scores"]
    for k, v in enumerate(dsa):
        cap = min(k * len(mod_class) // max(1, len(dsa)), len(mod_class) - 1)
        mod_dsa[mod_class[cap]].append(v)
    avg = np.array([np.mean(d) if d else 0.0 for d in mod_dsa])
    order = np.argsort(avg)[::-1]
    labels = [MOD_NAMES[i] if i < len(MOD_NAMES) else f"m{i}" for i in order]
    ax.bar(range(len(order)), avg[order], color=C_DSFB, alpha=0.75)
    ax.set_xticks(range(len(order)))
    ax.set_xticklabels(labels, rotation=60, ha="right", fontsize=6)
    ax.set_ylabel("mean DSA score")
    ax.set_title("DSA mean per modulation — structural density of residual")
    caption(fig, common, "DSA-motif density across modulations.",
            pick_non_claim(common, 1))
    save(fig, 74, name, "dsa_permod", out_dir)

    # fig_75 — Fisher-Rao distance matrix
    fig, ax = plt.subplots(figsize=(5.4, 4.6))
    im = ax.imshow(fr, cmap="magma")
    ax.set_title("Fisher-Rao distance matrix (per-mod residual distributions)")
    ax.set_xticks([]); ax.set_yticks([])
    plt.colorbar(im, ax=ax, shrink=0.85, label="d_FR")
    caption(fig, common, "Statistical-manifold geometry of the residual.",
            pick_non_claim(common, 2))
    save(fig, 75, name, "fisher_rao_matrix", out_dir)

    # fig_76 — TDA persistence
    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    plot_tda_barcode(ax, common)
    caption(fig, common, "Sliding-window persistence on the residual.",
            pick_non_claim(common, 3))
    save(fig, 76, name, "tda_persistence", out_dir)

    # fig_77 — Attractor phase portrait
    fig, ax = plt.subplots(figsize=(5.8, 4.0))
    plot_attractor(ax, common, tau=8)
    caption(fig, common, "Delay-embedded residual portrait.",
            pick_non_claim(common, 0))
    save(fig, 77, name, "attractor", out_dir)

    # fig_78 — False-episode rate on clean windows (negative control)
    fig, ax = plt.subplots(figsize=(5.6, 3.2))
    raw = common["raw_boundary_count"]; n = common["n"]; ep = common["dsfb_episode_count"]
    raw_rate = raw / max(1, n); dsfb_rate = ep / max(1, n)
    ax.bar(["raw 3σ rate\n(context)", "DSFB episode rate"],
           [raw_rate, dsfb_rate], color=[C_COMPARATOR, C_DSFB], alpha=0.8)
    ax.set_ylabel("events / sample")
    ax.set_title("Negative-control rate on the full slice")
    caption(fig, common, "DSFB does not hallucinate structure on clean residuals.",
            pick_non_claim(common, 1))
    save(fig, 78, name, "negative_control", out_dir)


# ═══════════════════════════════════════════════════════════════════════
# ORACLE (figs 79 – 88)
# ═══════════════════════════════════════════════════════════════════════

def oracle_figures(data, out_dir):
    _standard_slice_bank(data, "oracle", 79, out_dir,
                         figure_titles=[
                             "Raw IQ context — input to AGC",
                             "Grammar-state timeline over WiFi session",
                             "Envelope calibration — ρ from healthy window",
                             "DSA score — context: EWMA comparator",
                             "Sign-tuple 3D — WiFi session",
                             "Fisher-Rao drift path",
                             "Super-rho persistence (sliding)",
                             "Attractor reconstruction — τ=8 delay embed",
                             "Detectability-bound parameters",
                             "Review-surface compression",
                         ])


def powder_figures(data, out_dir):
    _standard_slice_bank(data, "powder", 89, out_dir,
                         figure_titles=[
                             "LTE residual envelope calibration",
                             "Grammar-state timeline — LTE session",
                             "Residual trace — channel-estimator output",
                             "Cross-burst residual distribution",
                             "Fisher-Rao drift across LTE bursts",
                             "Super-rho persistence on LTE residual",
                             "Attractor reconstruction under fading",
                             "Detectability bound vs link margin",
                             "Grammar distribution WiFi↔LTE observer reach",
                             "W_pred window-length sensitivity (context)",
                         ])


def tampere_figures(data, out_dir):
    _standard_slice_bank(data, "tampere_gnss", 99, out_dir,
                         figure_titles=[
                             "GNSS envelope — CW/noise-floor calibration",
                             "Grammar-state timeline — weak-SNR L1 C/A",
                             "Boundary episodes on innovation stream",
                             "Tracking-loop innovation + sign-tuple",
                             "Fisher-Rao geometry — innovation manifold",
                             "Super-rho persistence — innovation stream",
                             "Attractor basin — weak-SNR innovation",
                             "Detectability bound — near-threshold exhibit",
                             "Permutation-entropy profile",
                             "Episode-duration CDF",
                         ])


def coloran_figures(data, out_dir):
    _standard_slice_bank(data, "coloran", 109, out_dir,
                         figure_titles=[
                             "Raw KPI traces — scheduler input",
                             "KPI residual stream — DSFB input",
                             "Grammar-state timeline on KPI residual",
                             "Envelope calibration on KPI residual",
                             "DSA score + 3σ context line",
                             "Sign-tuple 3D on KPI residual",
                             "Fisher-Rao drift on KPI residual",
                             "Super-rho persistence on KPI residual",
                             "Reverse-arrangements trend scan",
                             "Review-surface compression — KPI events",
                         ])


def coloran_commag_figures(data, out_dir):
    _standard_slice_bank(data, "coloran_commag", 119, out_dir,
                         figure_titles=[
                             "Scheduling-policy timeline (input label)",
                             "Per-policy KPI residual",
                             "Grammar timeline segmented by policy",
                             "Envelope calibration cross-policy",
                             "DSA score per-policy",
                             "Fisher-Rao geodesic between per-policy residuals",
                             "Attractor basin reconstruction per policy",
                             "Detectability bound per policy",
                             "Grammar occupancy at policy-switch boundaries",
                             "Review-surface compression by policy",
                         ])


def deepbeam_figures(data, out_dir):
    _standard_slice_bank(data, "deepbeam", 129, out_dir,
                         figure_titles=[
                             "Raw mmWave IQ scatter — beamformer input",
                             "Gain (RF front-end) dynamics",
                             "Beam-index evolution — beam-steering context",
                             "Envelope calibration on beamformer residual",
                             "Grammar-state timeline on residual",
                             "DSA score under beam-steering",
                             "Sign-tuple 3D coloured by beam-pair idx",
                             "Fisher-Rao drift on single-pair residual",
                             "Attractor basin — single-pair residual",
                             "Gain-vs-residual joint plot",
                         ])


def deepsense_figures(data, out_dir):
    block = data.get("deepsense_6g")
    if not block or block.get("skipped"):
        print(f"[SKIP] deepsense_6g: {block.get('reason') if block else 'missing'}")
        return
    common = block["common"]
    name = common["name"]
    power = np.asarray(block["mmwave_power"], dtype=float).reshape(
        block["n_steps"], block["n_beams"])
    best = np.asarray(block["best_beam_index"], dtype=int)
    alt = np.asarray(block["altitude"], dtype=float)
    speed = np.asarray(block["speed"], dtype=float)
    pitch = np.asarray(block["pitch"], dtype=float)
    roll = np.asarray(block["roll"], dtype=float)
    corr = np.asarray(block["beam_correlation"], dtype=float)
    margin = np.asarray(block["margin_raw"], dtype=float)

    # Honesty prelude (printed once)
    print("  [honesty] DeepSense Scenario 23 is power-only — no IQ, no sign-tuple on waveform.")

    # fig_139 — power heatmap
    fig, ax = plt.subplots(figsize=(6.2, 3.6))
    im = ax.imshow(power.T, aspect="auto", cmap="viridis", origin="lower")
    ax.set_xlabel("time step"); ax.set_ylabel("beam index")
    ax.set_title("64-beam power heatmap (beam-tracker input matrix)")
    plt.colorbar(im, ax=ax, shrink=0.85, label="power")
    caption(fig, common, "The input matrix the beam-tracker already owns.",
            pick_non_claim(common, 0))
    save(fig, 139, name, "power_heatmap", out_dir)

    # fig_140 — best-beam timeline
    fig, ax = plt.subplots(figsize=(6.0, 3.2))
    ax.plot(best, color=C_DSFB, lw=0.8)
    ax.set_xlabel("time step"); ax.set_ylabel("best_beam_index")
    ax.set_title("Best-beam timeline — the beam-tracker's own decision stream")
    caption(fig, common, "DSFB does not override the beam-tracker.",
            pick_non_claim(common, 1))
    save(fig, 140, name, "best_beam_timeline", out_dir)

    # fig_141 — top-3 beam power distribution
    means = power.mean(axis=0)
    top3 = np.argsort(means)[::-1][:3]
    fig, ax = plt.subplots(figsize=(5.8, 3.2))
    for b in top3:
        ax.hist(power[:, b], bins=30, alpha=0.55, label=f"beam {b}")
    ax.set_xlabel("power"); ax.set_ylabel("count")
    ax.set_title("Power distribution — top-3 beams (by mean)")
    ax.legend(loc="best", framealpha=0.75)
    caption(fig, common, "Descriptive distribution of beam power.",
            pick_non_claim(common, 2))
    save(fig, 141, name, "top3_power", out_dir)

    # fig_142 — best-vs-2nd-best margin histogram (the scalar residual)
    fig, ax = plt.subplots(figsize=(5.8, 3.2))
    ax.hist(margin, bins=40, color=C_DSFB, alpha=0.8, edgecolor="white", lw=0.3)
    ax.axvline(common["rho"], color=C_VIOLATION, lw=1.0, ls="--",
               label=f"rho = {common['rho']:.3f}")
    ax.set_xlabel("best − 2nd-best power margin")
    ax.set_ylabel("count")
    ax.set_title("Scalar margin residual DSFB will structure")
    ax.legend(loc="best", framealpha=0.75)
    caption(fig, common, "The scalar residual (tracker's discarded secondary information).",
            pick_non_claim(common, 0))
    save(fig, 142, name, "margin_hist", out_dir)

    # fig_143 — altitude vs mean-power
    fig, ax = plt.subplots(figsize=(5.6, 3.2))
    mean_p = power.mean(axis=1)
    ax.scatter(alt, mean_p, s=8, c=C_DSFB, alpha=0.6)
    ax.set_xlabel("altitude"); ax.set_ylabel("mean beam power")
    ax.set_title("Altitude × mean-power (descriptive)")
    caption(fig, common, "Descriptive correlation, not causal.",
            pick_non_claim(common, 1))
    save(fig, 143, name, "altitude_power", out_dir)

    # fig_144 — speed vs beam-switching rate
    switches = np.abs(np.diff(best.astype(int)))
    fig, ax = plt.subplots(figsize=(5.6, 3.2))
    ax.scatter(speed[:-1], switches, s=6, c=C_DSFB, alpha=0.55)
    ax.set_xlabel("speed"); ax.set_ylabel("|Δ beam index| per step")
    ax.set_title("Speed × beam-switching rate (descriptive)")
    caption(fig, common, "Descriptive correlation, not a beam-selection benchmark.",
            pick_non_claim(common, 2))
    save(fig, 144, name, "speed_switching", out_dir)

    # fig_145 — pitch / roll × beam-index drift
    fig, ax = plt.subplots(figsize=(6.0, 3.2))
    ax.scatter(pitch, best, s=6, c=C_DSFB, alpha=0.5, label="pitch")
    ax.scatter(roll, best, s=6, c=C_COMPARATOR, alpha=0.5, label="roll")
    ax.set_xlabel("UAV pitch / roll")
    ax.set_ylabel("best_beam_index")
    ax.set_title("Manoeuvre × beam context (descriptive)")
    ax.legend(loc="best", framealpha=0.75)
    caption(fig, common, "Descriptive manoeuvre context.",
            pick_non_claim(common, 0))
    save(fig, 145, name, "pitch_roll_beam", out_dir)

    # fig_146 — multibeam correlation matrix
    fig, ax = plt.subplots(figsize=(5.2, 4.6))
    im = ax.imshow(corr, cmap="coolwarm", vmin=-1, vmax=1)
    ax.set_xlabel("beam"); ax.set_ylabel("beam")
    ax.set_title("64×64 beam correlation matrix")
    plt.colorbar(im, ax=ax, shrink=0.85, label="corr")
    caption(fig, common, "Descriptive multibeam correlation structure.",
            pick_non_claim(common, 1))
    save(fig, 146, name, "beam_correlation", out_dir)

    # fig_147 — UAV 3D trajectory (use distance/height if present, else altitude)
    distance = np.asarray(block.get("distance", []), dtype=float)
    height = np.asarray(block.get("height", []), dtype=float)
    fig = plt.figure(figsize=(6.2, 4.2))
    ax = fig.add_subplot(111, projection="3d")
    if distance.size == len(alt) and height.size == len(alt):
        sc = ax.scatter(distance, height, alt, c=best, s=6, cmap="viridis",
                        alpha=0.7)
    else:
        t = np.arange(len(alt))
        sc = ax.scatter(t, speed, alt, c=best, s=6, cmap="viridis", alpha=0.7)
    ax.set_title("UAV 3D trajectory coloured by best beam")
    plt.colorbar(sc, ax=ax, shrink=0.6, label="best beam")
    caption(fig, common, "Geographic context, not a selection benchmark.",
            pick_non_claim(common, 2))
    save(fig, 147, name, "uav_trajectory", out_dir)

    # fig_148 — grammar timeline on scalar margin residual
    fig, ax = plt.subplots(figsize=(6.4, 3.6))
    plot_grammar_timeline(ax, common, " (scalar margin)")
    caption(fig, common,
            "DSFB structuring the beam-tracker's discarded secondary info.",
            pick_non_claim(common, 0))
    save(fig, 148, name, "grammar_margin", out_dir)


# ═══════════════════════════════════════════════════════════════════════
# Shared 10-figure template for IQ-style slices (ORACLE, POWDER, Tampere,
# DeepBeam) and KPI slices (ColO-RAN / ColO-RAN-commag).  Each slice
# renders 10 figures; ordering is deterministic so zip output is stable.
# ═══════════════════════════════════════════════════════════════════════

def _standard_slice_bank(data, slice_key, base_idx, out_dir, figure_titles):
    block = data.get(slice_key)
    if not block or block.get("skipped"):
        print(f"[SKIP] {slice_key}: {block.get('reason') if block else 'missing'}")
        return
    common = block["common"]
    name = common["name"]

    # Slice-specific raw context (fig_0): raw norms as context.
    fig, ax = plt.subplots(figsize=(6.2, 3.2))
    ax.plot(common["norms"][:8000], color=C_NEUTRAL, lw=0.5, alpha=0.85)
    hw = common["healthy_window_size"]
    ax.axvline(hw, color=C_BOUNDARY, lw=0.7, ls="--",
               label=f"end of healthy window (k={hw})")
    ax.set_xlabel("sample index")
    ax.set_ylabel("||r||")
    ax.set_title(figure_titles[0])
    ax.legend(loc="best", framealpha=0.75)
    caption(fig, common, "Context view of the residual stream.",
            pick_non_claim(common, 0))
    save(fig, base_idx + 0, name, "raw_context", out_dir)

    # fig_1: grammar timeline
    fig, ax = plt.subplots(figsize=(6.2, 3.2))
    plot_grammar_timeline(ax, common)
    caption(fig, common, figure_titles[1], pick_non_claim(common, 1))
    save(fig, base_idx + 1, name, "grammar_timeline", out_dir)

    # fig_2: envelope calibration
    fig, ax = plt.subplots(figsize=(5.8, 3.2))
    plot_envelope_calibration(ax, common)
    caption(fig, common, figure_titles[2], pick_non_claim(common, 2))
    save(fig, base_idx + 2, name, "envelope_cal", out_dir)

    # fig_3: DSA + EWMA context
    fig, ax = plt.subplots(figsize=(6.2, 3.4))
    plot_dsa_trace(ax, common)
    caption(fig, common, figure_titles[3], pick_non_claim(common, 3 % max(1, len(common.get("non_claims", [])))))
    save(fig, base_idx + 3, name, "dsa_ewma", out_dir)

    # fig_4: sign-tuple 3D
    fig = plt.figure(figsize=(5.8, 4.2))
    plot_sign_scatter3d(fig, common, figure_titles[4])
    caption(fig, common, figure_titles[4], pick_non_claim(common, 0))
    save(fig, base_idx + 4, name, "sign_scatter", out_dir)

    # fig_5: Fisher-Rao drift
    fig, ax = plt.subplots(figsize=(5.8, 3.4))
    plot_fisher_rao_drift(ax, common)
    caption(fig, common, figure_titles[5], pick_non_claim(common, 1))
    save(fig, base_idx + 5, name, "fisher_rao_drift", out_dir)

    # fig_6: TDA persistence
    fig, ax = plt.subplots(figsize=(6.0, 3.4))
    plot_tda_barcode(ax, common)
    caption(fig, common, figure_titles[6], pick_non_claim(common, 2))
    save(fig, base_idx + 6, name, "tda_persistence", out_dir)

    # fig_7: Attractor phase portrait
    fig, ax = plt.subplots(figsize=(5.8, 4.0))
    plot_attractor(ax, common, tau=8)
    caption(fig, common, figure_titles[7], pick_non_claim(common, 3 % max(1, len(common.get("non_claims", [])))))
    save(fig, base_idx + 7, name, "attractor", out_dir)

    # fig_8: Detectability bound
    fig, ax = plt.subplots(figsize=(5.4, 3.2))
    plot_detectability(ax, common)
    caption(fig, common, figure_titles[8], pick_non_claim(common, 0))
    save(fig, base_idx + 8, name, "detectability", out_dir)

    # fig_9: review compression
    fig, ax = plt.subplots(figsize=(5.6, 3.2))
    plot_compression(ax, common)
    caption(fig, common, figure_titles[9], pick_non_claim(common, 1))
    save(fig, base_idx + 9, name, "compression", out_dir)


# ═══════════════════════════════════════════════════════════════════════
# fig_149 — demodulation-threshold scan across 24 RadioML modulations
# ═══════════════════════════════════════════════════════════════════════

def plot_fig149(data, out_dir):
    """Demodulation-threshold scan — DSFB first-boundary SNR per modulation.

    Two-panel figure:
      - Left: SNR at which the class-local admissibility envelope first
        flags the amplitude-template demodulator residual, for each of
        the 24 RadioML modulations.
      - Right: the JCGM 100:2008 (GUM) Type-A sample variance
        s^2_{||r||} of the same calibration set, which is the
        finite-sample Fisher-information floor 1/I(rho) referenced in
        the paper's Fisher-information subsection.

    The right panel is a direct visual companion to the Fisher-info
    tie-back: classes with lower s^2_{||r||} have tighter envelopes
    and (generically) later threshold crossings on the left. Not a
    modulation-recognition benchmark; DSFB observes what the
    demodulator quietly produces at each SNR.
    """
    block = data.get("radioml")
    if not block or block.get("skipped"):
        print(f"[SKIP] fig_149: radioml block missing")
        return
    common = block["common"]
    name = common["name"]
    fig_data = block.get("fig_149", [])
    if not fig_data:
        print("[SKIP] fig_149: no per-class threshold data emitted")
        return

    labels = [MOD_NAMES[i] if i < len(MOD_NAMES) else f"m{i}"
              for i in range(len(fig_data))]
    thresholds = [d.get("threshold_snr_db") for d in fig_data]
    s2_r = [d.get("s2_r_unbiased") for d in fig_data]

    fig, (ax_l, ax_r) = plt.subplots(
        1, 2, figsize=(9.6, 5.4),
        gridspec_kw={"width_ratios": [1.6, 1.0], "wspace": 0.04},
        sharey=True,
    )

    # Left panel — threshold-crossing SNR per class.
    for y, (lbl, snr) in enumerate(zip(labels, thresholds)):
        if snr is None:
            ax_l.plot(-21, y, "x", color="#888888", markersize=6, alpha=0.6)
            ax_l.annotate("floor", xy=(-21, y), xytext=(5, 0),
                          textcoords="offset points", fontsize=6,
                          va="center", color="#888888")
        else:
            ax_l.plot(snr, y, "o", color=C_DSFB, markersize=6, alpha=0.9)
            ax_l.annotate(f"{snr:+.0f} dB", xy=(snr, y), xytext=(5, 0),
                          textcoords="offset points", fontsize=6, va="center")
    ax_l.set_yticks(range(len(labels)))
    ax_l.set_yticklabels(labels, fontsize=7)
    ax_l.set_xlabel("SNR at first DSFB boundary crossing (dB)")
    ax_l.set_xlim(-22, 32)
    ax_l.set_ylim(-0.5, len(labels) - 0.5)
    ax_l.invert_yaxis()
    ax_l.grid(True, axis="x", alpha=0.3)
    ax_l.set_title("Threshold-crossing SNR (per class)")

    # Right panel — JCGM Type-A sample variance s^2_{||r||} per class,
    # i.e. the finite-sample Fisher-information floor 1/I(rho).
    finite = [v for v in s2_r if isinstance(v, (int, float))
              and v is not None and v == v]  # reject NaN
    if not finite:
        ax_r.text(0.5, 0.5, "no calibration data",
                  transform=ax_r.transAxes, ha="center", va="center",
                  fontsize=8, color="#888888")
    else:
        vmax = max(finite) * 1.10 if max(finite) > 0 else 1.0
        for y, v in enumerate(s2_r):
            if v is None or not isinstance(v, (int, float)) or v != v:
                ax_r.plot(0, y, "x", color="#888888", markersize=5, alpha=0.5)
                continue
            ax_r.barh(y, v, height=0.65, color=C_DSFB, alpha=0.75,
                      edgecolor="none")
        ax_r.set_xlim(0, vmax)
    ax_r.set_xlabel(r"$s^2_{\|r\|}$ (JCGM Type A, cal set)")
    ax_r.grid(True, axis="x", alpha=0.3)
    ax_r.set_title(r"Fisher-info floor $1/\widehat{I}(\rho)$")

    caption(fig, common,
            "Left: per-class SNR at which DSFB's class-local envelope first "
            "flags the demodulator residual. Right: JCGM Type-A sample "
            r"variance $s^2_{\|r\|}$ of the calibration set (upper-quartile "
            r"SNR captures), i.e.\ the finite-sample Fisher-information "
            r"floor $1/\widehat{I}(\rho)$ from the paper's §IV.F.3' "
            "Fisher-info tie-back.",
            "Not a modulation-recognition benchmark; DSFB observes what the "
            "amplitude-template demodulator quietly produces. The right-panel "
            "values are emission-only and do not change envelope calibration.")
    save(fig, 149, name, "demod_threshold_scan_fisher", out_dir)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()

    print("═" * 64)
    print(" DSFB-RF │ figures_real.py │ 80+1 real-dataset figures")
    print("═" * 64)
    print(f"  loading: {args.data}")
    data = json.loads(args.data.read_text())

    radioml_figures(data, args.out)
    oracle_figures(data, args.out)
    powder_figures(data, args.out)
    tampere_figures(data, args.out)
    coloran_figures(data, args.out)
    coloran_commag_figures(data, args.out)
    deepbeam_figures(data, args.out)
    deepsense_figures(data, args.out)
    plot_fig149(data, args.out)

    pdfs = sorted(args.out.glob("*.pdf"))
    print(f"\nFigures written: {len(pdfs)} PDFs in {args.out}")


if __name__ == "__main__":
    main()
