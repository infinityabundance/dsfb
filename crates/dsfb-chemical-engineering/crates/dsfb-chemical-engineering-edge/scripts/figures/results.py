"""Figure group F — results & metrics (9 designs), from metrics.csv + METRICS_DEFINITIONS.toml.

Distinct from the paper's original six summary charts: these are new framings (a cross-dataset scorecard
heatmap, two-metric scatters, the TEP head-to-head waterfall, the SWaT scope panel, the BATADAL physics-
selectivity bar) that surface the empirical core. Every number traces to a committed artifact — metrics.csv
(per-dataset) or METRICS_DEFINITIONS.toml (the headline-rate protocols) — so each figure is reproducible
and the denominators are disclosed, not asserted.
"""
import os
import re

import numpy as np
import matplotlib.pyplot as plt

from . import style as S


def _metrics(run):
    """Load metrics.csv as a list of dicts (one row per dataset)."""
    return S.read_csv_rows(os.path.join(run, "metrics.csv"))


def _short(name):
    """Compact dataset label for dense axes (TEP-1 etc.)."""
    return (name.replace("tennessee_eastman_idv", "TEP-").replace("_", " "))


def fig_scorecard(run):
    """F40 ★ — a cross-dataset scorecard heatmap (dataset × normalised metric).

    Four metrics per dataset — compression ratio, unknown rate, baseline FP rate, fused-episode count —
    each column min–max normalised so the heatmap reads as a relative scorecard across the 20 datasets.
    Raw values are annotated so nothing is hidden behind the normalisation.
    """
    m = _metrics(run)
    m.sort(key=lambda r: r["dataset"])
    names = [_short(r["dataset"]) for r in m]
    cols = [("compression", "episode_compression_ratio", "{:.0f}×"),
            ("unknown rate", "unknown_rate", "{:.0%}"),
            ("baseline FP", "baseline_false_positive_rate", "{:.0%}"),
            ("episodes", "fused_episodes", "{:.0f}")]
    raw = np.array([[S.fnum(r[c]) for _, c, _ in cols] for r in m])
    norm = np.zeros_like(raw)
    for j in range(raw.shape[1]):
        col = raw[:, j]
        lo, hi = np.nanmin(col), np.nanmax(col)
        norm[:, j] = (col - lo) / (hi - lo) if hi > lo else 0.0
    fig, ax = plt.subplots(figsize=(6.8, 8.0))
    im = ax.imshow(norm, aspect="auto", cmap="cividis")
    ax.set_xticks(range(len(cols))); ax.set_xticklabels([c[0] for c in cols], rotation=20, ha="right")
    ax.set_yticks(range(len(names))); ax.set_yticklabels(names, fontsize=7.5)
    for i in range(raw.shape[0]):
        for j, (_, _, fmt) in enumerate(cols):
            v = raw[i, j]
            txt = fmt.format(v) if np.isfinite(v) else "—"
            ax.text(j, i, txt, ha="center", va="center", fontsize=6.8,
                    color=("white" if norm[i, j] < 0.55 else "black"))
    ax.set_title("Cross-dataset scorecard (raw values; colour = per-column normalised)", fontsize=10.5)
    fig.colorbar(im, ax=ax, fraction=0.03, pad=0.03, label="per-column min–max")
    S.figure_caption(ax, "Each column normalised independently; raw values annotated. Higher baseline FP is expected for non-stationary sets (reported honestly).")
    S.save(fig, "results_scorecard", "F", "metrics.csv",
           "A cross-dataset scorecard heatmap over compression, unknown rate, baseline FP, and episode count.")


def fig_detection_quality(run):
    """F37 — detection delay vs baseline false-positive rate, one point per labelled dataset (two-metric view).

    Combines the two headline detection metrics into one plane: the desirable corner is bottom-left (fast,
    clean). Makes the accuracy/latency trade-off legible across datasets in a single figure.
    """
    m = [r for r in _metrics(run) if r["detection_delay"] not in ("na", "")]
    x = [S.fnum(r["baseline_false_positive_rate"]) * 100 for r in m]
    y = [S.fnum(r["detection_delay"]) for r in m]
    fig, ax = plt.subplots(figsize=(7.0, 4.6))
    ax.scatter(x, y, s=36, color=S.ACCENT, zorder=3)
    for xi, yi, r in zip(x, y, m):
        ax.annotate(_short(r["dataset"]), (xi, yi), fontsize=6.5, xytext=(3, 3), textcoords="offset points")
    ax.axhline(0, color="#999", lw=0.8)
    ax.set_xlabel("baseline-window false-positive rate (%)")
    ax.set_ylabel("detection delay (samples vs labelled onset)")
    ax.set_title("Detection quality: delay vs baseline false-positive rate")
    S.figure_caption(ax, "Bottom-left is best (fast + clean). Negative delay = detection at/just before the labelled onset.")
    S.save(fig, "results_detection_quality", "F", "metrics.csv",
           "Detection delay vs baseline false-positive rate per labelled dataset — the latency/cleanliness plane.")


def fig_compression_ladder(run):
    """F38 — the residual-structure compression ladder: raw breach-steps → fused episodes, per dataset.

    Two horizontal bars per dataset on a shared log axis show how many raw detector breach-steps collapse
    into how few fused episodes — the core "compression" claim made concrete per dataset.
    """
    m = [r for r in _metrics(run) if S.fnum(r["raw_breach_steps"]) > 0]
    m.sort(key=lambda r: S.fnum(r["episode_compression_ratio"]))
    names = [_short(r["dataset"]) for r in m]
    raw = [S.fnum(r["raw_breach_steps"]) for r in m]
    ep = [max(S.fnum(r["fused_episodes"]), 0.7) for r in m]  # floor so zero-episode bars are visible on log
    y = np.arange(len(names))
    fig, ax = plt.subplots(figsize=(7.2, 6.2))
    ax.barh(y + 0.2, raw, height=0.38, color=S.MUTE, label="raw detector breach-steps")
    ax.barh(y - 0.2, ep, height=0.38, color=S.OK, label="fused episodes")
    ax.set_yticks(y); ax.set_yticklabels(names, fontsize=7.5)
    ax.set_xscale("log")
    ax.set_xlabel("count (log scale)")
    ax.set_title("Compression ladder: raw breach-steps → fused episodes")
    ax.legend(frameon=False, fontsize=8, loc="lower right")
    S.save(fig, "results_compression_ladder", "F", "metrics.csv",
           "Per-dataset compression: raw detector breach-steps collapsing into a few fused episodes.")


def fig_episode_census(run):
    """F39 — labelled vs unknown fused episodes per dataset, with the honest unknown rate annotated.

    A stacked bar (heuristic-labelled + unknown-with-preserved-evidence) per dataset; the unknown share is
    the framework's honesty dial — it prefers *unknown* to a confident wrong label.
    """
    m = [r for r in _metrics(run) if S.fnum(r["fused_episodes"]) > 0]
    m.sort(key=lambda r: r["dataset"])
    names = [_short(r["dataset"]) for r in m]
    lab = [int(S.fnum(r["labeled_episodes"])) for r in m]
    unk = [int(S.fnum(r["unknown_episodes"])) for r in m]
    y = np.arange(len(names))
    fig, ax = plt.subplots(figsize=(7.2, 5.6))
    ax.barh(y, lab, color=S.OK, label="heuristic-labelled")
    ax.barh(y, unk, left=lab, color=S.MUTE, label="unknown (evidence preserved)")
    for yi, (l, u) in zip(y, zip(lab, unk)):
        tot = l + u
        if tot:
            ax.text(tot, yi, f"  {u}/{tot} unk", va="center", fontsize=6.8, color="#555")
    ax.set_yticks(y); ax.set_yticklabels(names, fontsize=7.5)
    ax.set_xlabel("fused episodes")
    ax.set_title("Explanation coverage vs honest unknown rate")
    ax.legend(frameon=False, fontsize=8, loc="lower right")
    S.save(fig, "results_episode_census", "F", "metrics.csv",
           "Labelled vs unknown fused episodes per dataset — the framework's deliberate honest-unknown behaviour.")


def fig_tep_head_to_head(run):
    """F41 ★ — the Tennessee Eastman IDV(1) head-to-head waterfall: breach-steps → activations → episodes.

    The single most quotable result: 10041 raw detector breach-steps → 102 ISA-18.2 alarm activations →
    6 fused episodes (1674× over breach-steps; 17× over activations). Numbers from METRICS_DEFINITIONS.toml.
    """
    # Transcribed from data/METRICS_DEFINITIONS.toml [tep_idv01.head_to_head] (reproducible via the recipe).
    stages = [("raw breach-steps\n(14 detectors × 960)", 10041, S.MUTE),
              ("ISA-18.2 alarm\nactivations", 102, S.CB["orange"]),
              ("DSFB fused\nepisodes", 6, S.OK)]
    fig, ax = plt.subplots(figsize=(7.0, 4.4))
    x = np.arange(len(stages))
    vals = [s[1] for s in stages]
    ax.bar(x, vals, color=[s[2] for s in stages], width=0.6, zorder=3)
    ax.set_yscale("log")
    for xi, (lab, v, _) in zip(x, stages):
        ax.text(xi, v * 1.15, f"{v:,}", ha="center", fontsize=10, weight="bold")
    ax.set_xticks(x); ax.set_xticklabels([s[0] for s in stages], fontsize=8.5)
    ax.set_ylabel("count (log scale)")
    ax.set_title("Tennessee Eastman IDV(1): 1674× compression over breach-steps, 17× over alarms")
    # Annotate the compression arrows between stages.
    ax.annotate("1674×", xy=(0.5, 600), fontsize=10, color=S.WARN, ha="center")
    ax.annotate("17×", xy=(1.5, 25), fontsize=10, color=S.WARN, ha="center")
    S.figure_caption(ax, "Breach-steps = Σ per-(detector,sample) breach flags; activations = rising edges; from METRICS_DEFINITIONS.toml.")
    S.save(fig, "results_tep_head_to_head", "F", "METRICS_DEFINITIONS.toml [tep_idv01]",
           "TEP IDV(1) head-to-head: raw breach-steps → alarm activations → fused episodes (1674× / 17×).")


def fig_swat_scope(run):
    """F42 ★ — SWaT T101: within-scope recall, out-of-scope quiet, and the normal-run false-positive rate.

    The physics-witness scope story: of the attacks the closure criterion says are *in scope* (touch a T101
    balance term), the witness catches 5/5; of the 30 out-of-scope segments it correctly stays quiet on 73%;
    normal-run false-positive rate is 4.4% (119/2700 blocks). Numbers from METRICS_DEFINITIONS.toml.
    """
    fig, ax = plt.subplots(1, 2, figsize=(7.6, 3.6), gridspec_kw={"width_ratios": [1.3, 1]})
    # Left: recall + out-of-scope quiet as stacked proportion bars.
    ax[0].barh([1], [5], color=S.OK)
    ax[0].text(5, 1, "  5/5 in-scope caught (100%)", va="center", fontsize=8)
    ax[0].barh([0], [30 * 0.73], color=S.CB["skyblue"], label="correctly quiet")
    ax[0].barh([0], [30 * 0.27], left=[30 * 0.73], color=S.MUTE, label="fired (out of scope)")
    ax[0].text(30, 0, "  73% of 30 out-of-scope quiet", va="center", fontsize=8)
    ax[0].set_yticks([0, 1]); ax[0].set_yticklabels(["out-of-scope\nsegments", "in-scope\nattacks"], fontsize=8)
    ax[0].set_xlabel("segments")
    ax[0].set_title("SWaT T101 — scope-conditioned detection", fontsize=9.5)
    # Right: the normal-run FP rate as a big number + denominator.
    ax[1].axis("off")
    ax[1].text(0.5, 0.62, "4.4%", ha="center", fontsize=30, color=S.ACCENT, weight="bold")
    ax[1].text(0.5, 0.40, "normal-run false positives\n(119 / 2700 blocks)", ha="center", fontsize=8.5)
    ax[1].set_title("false-positive rate", fontsize=9.5)
    S.figure_caption(ax[0], "In-scope = attacks touching a T101 balance term (LIT101/FIT101/FIT201). From METRICS_DEFINITIONS.toml. " + S.DISCLAIMER["candidate"])
    S.save(fig, "results_swat_scope", "F", "METRICS_DEFINITIONS.toml [swat_t101]",
           "SWaT T101: 5/5 in-scope recall, 73% of out-of-scope segments correctly quiet, 4.4% normal-run FP.")


def fig_batadal_selectivity(run):
    """F43 ★ — BATADAL T1 physics selectivity: the volume-balance witness fires only on inflow-manipulation.

    The most persuasive physics result: of the labelled BATADAL attacks, the T1 volume-balance witness
    fires on exactly the two that manipulate T1's inflow pump (closure breaks 1.80× / 1.68×) and stays quiet
    on the three that target other tanks (0.5–0.9×). Normal-year FP = 0.1% (5/3958). Spatial selectivity
    from physics, not tuning. Numbers from docs/balance_witness_criterion.md + METRICS_DEFINITIONS.toml.
    """
    # The two in-scope ratios (1.80, 1.68) are the MEASURED closure breaks from the committed criterion doc;
    # the three other-tank attacks are documented only as a band (0.5–0.9×, correctly sub-threshold), so they
    # are drawn as a shaded "quiet band" rather than invented per-attack heights (academic honesty).
    fired = [("Oct 9–11\n(T1 inflow)", 1.80), ("Oct 30–Nov 1\n(T1 inflow)", 1.68)]
    fig, ax = plt.subplots(figsize=(7.2, 4.0))
    x = np.arange(len(fired) + 1)
    ax.bar(x[:2], [a[1] for a in fired], color=S.WARN, width=0.6, zorder=3)
    # The three other-tank attacks: a shaded band over their documented 0.5–0.9× range (no fabricated heights).
    ax.bar([x[2]], [0.9 - 0.5], bottom=[0.5], color=S.MUTE, width=0.6, zorder=3)
    ax.text(x[2], 0.95, "3 other-tank attacks\n0.5–0.9× band\n(correctly quiet)", ha="center", va="bottom", fontsize=7.2)
    ax.axhline(1.0, color="#333", lw=0.9, ls="--")
    ax.text(x[-1], 1.0, " closure = 1 (balanced)", fontsize=7.5, va="bottom", ha="right", color="#333")
    ax.set_xticks(x); ax.set_xticklabels([fired[0][0], fired[1][0], "other tanks\n(×3)"], fontsize=7.5)
    ax.set_ylabel("closure-break ratio (vs normal)")
    ax.set_ylim(0, 2.0)
    ax.set_title("BATADAL T1: volume balance fires only on T1-inflow manipulation (0.1% normal-year FP)")
    handles = [plt.Line2D([0], [0], marker="s", ls="", color=S.WARN, label="in-scope (T1 inflow) — fires (measured 1.80, 1.68×)"),
               plt.Line2D([0], [0], marker="s", ls="", color=S.MUTE, label="other tank — correctly quiet (0.5–0.9× band)")]
    ax.legend(handles=handles, frameon=False, fontsize=7.5, loc="upper right")
    S.figure_caption(ax, "In-scope ratios measured (docs/balance_witness_criterion.md); other-tank attacks shown as their documented quiet band. Physics selectivity, not tuning. " + S.DISCLAIMER["candidate"])
    S.save(fig, "results_batadal_selectivity", "F", "balance_witness_criterion.md + METRICS_DEFINITIONS.toml",
           "BATADAL T1 volume balance fires on the two T1-inflow attacks and stays quiet on the three others.")


def fig_dataset_summary(run):
    """F44 — a compact rendered table of the per-dataset run summary (samples, vars, episodes, compression).

    A clean tabular figure (not a chart) giving the at-a-glance run census across all datasets — useful as a
    reference panel and an auditable cross-check against the manifest.
    """
    m = sorted(_metrics(run), key=lambda r: r["dataset"])
    headers = ["dataset", "kind", "samples", "vars", "episodes", "compr.", "unknown"]
    rows = []
    for r in m:
        rows.append([_short(r["dataset"]), r.get("kind", ""), r["n_samples"], r["n_vars"],
                     r["fused_episodes"], f"{S.fnum(r['episode_compression_ratio']):.0f}×",
                     f"{S.fnum(r['unknown_rate']):.0%}"])
    fig, ax = plt.subplots(figsize=(7.4, 0.32 * len(rows) + 1.0))
    ax.axis("off")
    tbl = ax.table(cellText=rows, colLabels=headers, loc="center", cellLoc="center")
    tbl.auto_set_font_size(False); tbl.set_fontsize(7.0); tbl.scale(1, 1.15)
    for j in range(len(headers)):
        tbl[0, j].set_facecolor("#e7eef5"); tbl[0, j].set_text_props(weight="bold")
    ax.set_title("Per-dataset run summary (cross-check against manifest.json)", fontsize=10.5)
    S.save(fig, "results_dataset_summary", "F", "metrics.csv",
           "A compact per-dataset run-summary table: samples, variables, episodes, compression, unknown rate.")


def render_all(run):
    """Render every group-F figure from metrics.csv + METRICS_DEFINITIONS.toml."""
    S.log("group F — results & metrics")
    fig_scorecard(run)
    fig_detection_quality(run)
    fig_compression_ladder(run)
    fig_episode_census(run)
    fig_tep_head_to_head(run)
    fig_swat_scope(run)
    fig_batadal_selectivity(run)
    fig_dataset_summary(run)
