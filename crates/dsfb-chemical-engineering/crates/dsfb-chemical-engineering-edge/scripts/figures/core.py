"""Figure group A — the core DSFB method made visible (10 designs).

These are the figures that let a reader *see* how a raw residual channel becomes a sealed structural
episode: the residual triple, the admissibility envelope, the grammar state machine, the quorum fusion,
the episode lifecycle, and the conceptual scaffolding (claim-boundary badges, certainty tiers, the
read-only non-interference architecture). Data-driven figures read one representative dataset's
`residual_streams.csv`; the conceptual diagrams are drawn deterministically with matplotlib patches
(no external layout engine, so layout is fixed and reproducible).
"""
import os

import numpy as np
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch
import matplotlib.pyplot as plt

from . import style as S

# Representative dataset for the per-dataset core figures: the canonical Tennessee Eastman IDV(1) step
# disturbance. Chosen because its residual triple is well-scaled (r∈[0,157], σ∈[−12,10]) so the structure
# is legible on a linear axis, and it is the fault the paper leads with. (cstr_reactor's SPE is degenerate
# — it explodes to ~1.4e35 — so it is a poor choice for showing residual *structure*.)
REP = "tennessee_eastman_idv01"
REP_DET = "pca_spe_q"  # the SPE/Q residual-energy detector (always present)


def _streams(run, dataset, detector):
    """Load (time, r, delta, sigma, grammar_state) for one detector from residual_streams.csv."""
    rows = [r for r in S.read_csv_rows(S.ds_csv(run, dataset, "residual_streams.csv"))
            if r["detector_id"] == detector]
    t = np.array([int(r["time_index"]) for r in rows])
    r_ = np.array([S.fnum(r["r"]) for r in rows])
    d = np.array([S.fnum(r["delta"]) for r in rows])
    s = np.array([S.fnum(r["sigma"]) for r in rows])
    g = [r["grammar_state"] for r in rows]
    return t, r_, d, s, g


def _episodes(run, dataset):
    """Return [(start,end), …] fused-episode spans from dsfb_episodes.csv."""
    return [(int(r["start_index"]), int(r["end_index"]))
            for r in S.read_csv_rows(S.ds_csv(run, dataset, "dsfb_episodes.csv"))]


def _onset(run, dataset):
    """Recover the labelled onset sample (is_onset==1) from the trace CSV, or None if unlabelled."""
    tpath = os.path.join(run, "figures", "trace_data", f"trace_{dataset}.csv")
    for r in S.read_csv_rows(tpath):
        if r.get("is_onset") == "1":
            return int(r["time_index"])
    return None


def _shade_episodes(ax, eps, color="#E69F00", alpha=0.16, label=None):
    """Shade fused-episode spans on a timeline axis (one labelled, rest unlabelled)."""
    for i, (a, b) in enumerate(eps):
        ax.axvspan(a, b, color=color, alpha=alpha, label=(label if i == 0 else None))


def fig_residual_triple(run):
    """A1 ★ — the DSFB residual triple (r, drift δ, slew σ) in three stacked panels for one detector.

    This is the atomic object of the whole framework: every sample becomes (residual, windowed-mean drift,
    first-difference slew). Episode spans shaded; labelled onset marked. Shows *what DSFB actually consumes*.
    """
    t, r_, d, s, g = _streams(run, REP, REP_DET)
    eps, onset = _episodes(run, REP), _onset(run, REP)
    fig, ax = plt.subplots(3, 1, figsize=(7.2, 4.6), sharex=True)
    for a, y, lab, col in zip(ax, (r_, d, s), ("residual  r", "drift  δ", "slew  σ"),
                              (S.INK, S.ACCENT, S.WARN)):
        a.plot(t, y, lw=0.9, color=col)
        a.set_ylabel(lab, fontsize=9.5)
        _shade_episodes(a, eps)
        if onset is not None:
            a.axvline(onset, color=S.OK, ls="--", lw=1.0)
    ax[0].set_title(f"DSFB residual triple — {S.disp(REP)} (detector: SPE/Q)", fontsize=10.5)
    ax[-1].set_xlabel("sample index")
    if onset is not None:
        ax[0].text(onset, ax[0].get_ylim()[1] * 0.9, " labelled onset", color=S.OK, fontsize=7.5)
    S.figure_caption(ax[-1], "Drift δ = windowed mean of r; slew σ = first difference of r. Orange band = fused episode.")
    S.save(fig, "core_residual_triple", "A", f"{REP}/residual_streams.csv",
           "The DSFB residual triple (r, drift, slew) — the atomic object the grammar classifies.")


def fig_admissibility_envelope(run):
    """A2 ★ — the admissibility envelope in (drift δ, slew σ) space.

    Each sample is a point in (δ, σ); colour = grammar token. The baseline-window points define the
    calibrated *interior* (shaded box from their δ/σ extent); points outside are flagged structure. Honest:
    the box is the empirical baseline extent, not a fabricated analytic boundary.
    """
    t, r_, d, s, g = _streams(run, REP, REP_DET)
    # baseline window size from metrics.csv (n_baseline) — the interior is calibrated from these samples.
    nb = 0
    for m in S.read_csv_rows(os.path.join(run, "metrics.csv")):
        if m["dataset"] == REP:
            nb = int(m.get("n_baseline", "0") or 0)
    fig, ax = plt.subplots(figsize=(6.6, 5.0))
    # Post-onset / structural points first, NOM (interior) points drawn on top so the nominal cluster shows.
    order = sorted(range(len(g)), key=lambda i: 0 if g[i] in ("NOM", "BG") else -1)
    cols = [S.GRAMMAR_COLORS.get(g[i], "#cccccc") for i in order]
    ax.scatter([d[i] for i in order], [s[i] for i in order], c=cols, s=11, edgecolors="none", alpha=0.85)
    # The calibrated interior is the baseline cluster near the origin. For SPE/Q on this dataset the baseline
    # (δ,σ) extent is tight, so we draw its bounding box (always visible) and annotate it rather than a hull.
    if nb > 0 and nb <= len(d):
        bd = np.array([d[i] for i in range(nb) if np.isfinite(d[i])])
        bs = np.array([s[i] for i in range(nb) if np.isfinite(s[i])])
        if len(bd) and len(bs):
            x0, x1, y0, y1 = bd.min(), bd.max(), bs.min(), bs.max()
            pad_x = max((x1 - x0) * 0.15, abs(x1) * 0.05, 0.5)
            pad_y = max((y1 - y0) * 0.15, 0.5)
            ax.add_patch(plt.Rectangle((x0 - pad_x, y0 - pad_y), (x1 - x0) + 2 * pad_x, (y1 - y0) + 2 * pad_y,
                                       fill=True, color=S.OK, alpha=0.12, zorder=0))
            ax.add_patch(plt.Rectangle((x0 - pad_x, y0 - pad_y), (x1 - x0) + 2 * pad_x, (y1 - y0) + 2 * pad_y,
                                       fill=False, ec=S.OK, lw=1.5, ls="--"))
            ax.annotate("calibrated interior\n(baseline extent)", xy=(x1, y1), xytext=(x1 + (ax.get_xlim()[1]) * 0.18, y1 + 3),
                        fontsize=7.5, color=S.OK, arrowprops=dict(arrowstyle="-|>", color=S.OK, lw=1.0))
    ax.set_xlabel("drift  δ")
    ax.set_ylabel("slew  σ")
    ax.set_title(f"Admissibility envelope in (δ, σ) — {S.disp(REP)}", fontsize=10.5)
    handles = [plt.Line2D([0], [0], marker="o", ls="", color=c, label=k) for k, c in S.GRAMMAR_COLORS.items()]
    # Legend to the right keeps the bottom clear for the on-figure honesty caption.
    ax.legend(handles=handles, fontsize=7, loc="center left", bbox_to_anchor=(1.01, 0.5),
              frameon=False, title="grammar token")
    S.figure_caption(ax, "Points coloured by grammar token. Nominal samples cluster in the baseline-calibrated interior; structural states spread outward.")
    S.save(fig, "core_admissibility_envelope", "A", f"{REP}/residual_streams.csv + n_baseline",
           "Samples in (drift, slew) space coloured by grammar token, with the baseline-calibrated interior.")


def fig_drift_slew_decomposition(run):
    """A5 — residual decomposed into drift (slow, persistent) and slew (fast, transient) components.

    Two overlaid panels make the central intuition explicit: a slow bias shows up in δ; a sharp step or
    spike shows up in σ. Different fault structures live in different components.
    """
    t, r_, d, s, g = _streams(run, REP, REP_DET)
    eps = _episodes(run, REP)
    fig, ax = plt.subplots(2, 1, figsize=(7.2, 3.8), sharex=True)
    ax[0].plot(t, d, lw=1.0, color=S.ACCENT)
    ax[0].fill_between(t, 0, d, color=S.ACCENT, alpha=0.12)
    ax[0].set_ylabel("drift δ\n(persistent)", fontsize=8.5)
    ax[1].plot(t, s, lw=0.8, color=S.WARN)
    ax[1].set_ylabel("slew σ\n(transient)", fontsize=8.5)
    for a in ax:
        _shade_episodes(a, eps)
    ax[0].set_title(f"Drift / slew decomposition — {S.disp(REP)} (SPE/Q)", fontsize=10.5)
    ax[-1].set_xlabel("sample index")
    S.save(fig, "core_drift_slew", "A", f"{REP}/residual_streams.csv",
           "Residual decomposed into a persistent drift component and a transient slew component.")


def fig_multidetector_grammar(run):
    """A4 — multi-detector grammar timeline: each executed detector's per-sample grammar token as a strip.

    Stacking the strips shows *agreement and disagreement structure* across the detector bank at a glance —
    the raw material the quorum fusion consumes.
    """
    rows = S.read_csv_rows(S.ds_csv(run, REP, "residual_streams.csv"))
    dets = sorted({r["detector_id"] for r in rows})
    fig, ax = plt.subplots(figsize=(7.2, max(2.6, 0.32 * len(dets) + 1.2)))
    for yi, det in enumerate(dets):
        drows = [r for r in rows if r["detector_id"] == det]
        for r in drows:
            ax.add_patch(plt.Rectangle((int(r["time_index"]) - 0.5, yi - 0.45), 1.0, 0.9,
                                       color=S.GRAMMAR_COLORS.get(r["grammar_state"], "#cccccc"), lw=0))
    ax.set_yticks(range(len(dets)))
    ax.set_yticklabels(dets, fontsize=7)
    ax.set_ylim(-0.6, len(dets) - 0.4)
    if rows:
        ax.set_xlim(0, max(int(r["time_index"]) for r in rows))
    ax.set_xlabel("sample index")
    ax.set_title(f"Per-detector grammar timelines — {S.disp(REP)}", fontsize=10.5)
    ax.grid(False)
    handles = [plt.Line2D([0], [0], marker="s", ls="", color=c, label=k) for k, c in S.GRAMMAR_COLORS.items()]
    ax.legend(handles=handles, fontsize=6.5, ncol=8, loc="upper center", bbox_to_anchor=(0.5, -0.18), frameon=False)
    S.save(fig, "core_multidetector_grammar", "A", f"{REP}/residual_streams.csv",
           "Per-detector grammar-token timelines — the agreement/disagreement structure fusion consumes.")


def fig_episode_lifecycle(run):
    """A7 — the lifecycle of a single fused episode: evidence (breach count) accruing onset→peak→close.

    Picks the longest fused episode in the representative dataset and plots, per sample within it, how many
    detectors are in a non-nominal grammar state — the evidence accrual that the episode seals.
    """
    eps = _episodes(run, REP)
    if not eps:
        return
    a, b = max(eps, key=lambda e: e[1] - e[0])
    rows = S.read_csv_rows(S.ds_csv(run, REP, "residual_streams.csv"))
    span = list(range(a, b + 1))
    nonnom = []
    for ti in span:
        toks = [r["grammar_state"] for r in rows if int(r["time_index"]) == ti]
        nonnom.append(sum(1 for tok in toks if tok not in ("NOM", "BG")))
    fig, ax = plt.subplots(figsize=(7.2, 3.2))
    ax.fill_between(span, 0, nonnom, color=S.WARN, alpha=0.25)
    ax.plot(span, nonnom, color=S.WARN, lw=1.4)
    ax.set_xlabel("sample index (within episode)")
    ax.set_ylabel("detectors in non-nominal state")
    ax.set_title(f"Episode lifecycle — {S.disp(REP)} episode [{a}, {b}]", fontsize=10.5)
    S.figure_caption(ax, "Evidence = count of detectors in a non-nominal grammar state per sample; the episode seals this accrual.")
    S.save(fig, "core_episode_lifecycle", "A", f"{REP}/dsfb_episodes.csv + residual_streams.csv",
           "Within-episode evidence accrual: detectors entering non-nominal grammar states onset→peak→close.")


# ── Conceptual diagrams (drawn deterministically; no data dependency) ──────────────────────────────────
def _box(ax, xy, w, h, text, fc, ec="#333333", fontsize=8.5, tc="#111111"):
    """Draw a rounded box with centred wrapped text; returns its centre for arrow anchoring."""
    x, y = xy
    ax.add_patch(FancyBboxPatch((x, y), w, h, boxstyle="round,pad=0.02,rounding_size=0.04",
                                fc=fc, ec=ec, lw=1.1))
    ax.text(x + w / 2, y + h / 2, text, ha="center", va="center", fontsize=fontsize, color=tc, wrap=True)
    return (x + w / 2, y + h / 2)


def _arrow(ax, p, q, color="#333333"):
    ax.add_patch(FancyArrowPatch(p, q, arrowstyle="-|>", mutation_scale=12, lw=1.1, color=color,
                                 shrinkA=2, shrinkB=2))


def fig_pipeline_architecture(run=None):
    """A10 ★ — the read-only non-interference architecture: DSFB sits *beside* the plant, writing nothing.

    The single most important figure for a plant audience: data flows in; DSFB emits a case file; nothing
    flows back to the process. Removing DSFB restores the pre-deployment baseline exactly.
    """
    fig, ax = plt.subplots(figsize=(7.4, 3.6))
    ax.set_xlim(0, 10); ax.set_ylim(0, 4); ax.axis("off")
    p_plant = _box(ax, (0.2, 1.4), 1.9, 1.2, "Plant\n(sensors, DCS,\nhistorian)", "#e7eef5")
    p_resid = _box(ax, (2.6, 1.4), 1.9, 1.2, "Established\ndetectors →\nresiduals", "#e7eef5")
    p_dsfb = _box(ax, (5.0, 1.0), 2.1, 2.0, "DSFB\nresidual semiotics\n(read-only)", "#d6ebe2", ec=S.OK)
    p_case = _box(ax, (7.6, 1.4), 2.1, 1.2, "Sealed case file\n(evidence_root,\nreplayable)", "#f3e9d6", ec=S.WARN)
    _arrow(ax, (2.1, 2.0), (2.6, 2.0)); _arrow(ax, (4.5, 2.0), (5.0, 2.0)); _arrow(ax, (7.1, 2.0), (7.6, 2.0))
    # The crucial non-edge: NO write-back to the plant. Draw a dashed back-arrow crossed out with a real
    # 'X' marker (a matplotlib marker, not a Unicode glyph — DejaVu Serif lacks ✗ and renders tofu).
    ax.annotate("no write-back to the process\n(removing DSFB restores the baseline exactly)",
                xy=(6.05, 0.95), xytext=(6.05, 0.2), ha="center", fontsize=7.5, color=S.WARN,
                arrowprops=dict(arrowstyle="-|>", color=S.WARN, ls="dashed", lw=1.0))
    ax.plot([6.05], [0.6], marker="X", ms=13, color=S.WARN, mec="white", mew=0.8)
    ax.set_title("Read-only non-interference architecture", fontsize=11)
    S.save(fig, "core_architecture", "A", "conceptual",
           "DSFB sits beside the plant as a read-only layer; it emits a sealed case file and never writes back.")


def fig_grammar_fsm(run=None):
    """A3 ★ — the grammar finite-state machine, rendered with graphviz for a clean professional layout.

    Nominal ⇄ structural states (distributional anomaly, structural shift, extreme value, change-point,
    recovery) with the transitions DSFB's deterministic grammar uses. graphviz `dot` lays out the edges
    without the arrowhead-occlusion of a hand-placed matplotlib graph.
    """
    nodes = {
        "NOM": ("Nominal", "#dfe7ef", "#111111"),
        "DA": ("Distributional\\nanomaly", "#E69F00", "#111111"),
        "SS": ("Structural\\nshift", "#D55E00", "#111111"),
        "EV": ("Extreme\\nvalue", "#9b2226", "#ffffff"),
        "CP": ("Change-\\npoint", "#CC79A7", "#111111"),
        "RC": ("Recovery", "#56B4E9", "#111111"),
    }
    edges = [("NOM", "DA"), ("NOM", "SS"), ("DA", "EV"), ("SS", "CP"), ("EV", "RC"),
             ("CP", "RC"), ("RC", "NOM"), ("DA", "SS")]
    lines = ['digraph G {', '  rankdir=LR; bgcolor="white";',
             '  labelloc="t"; fontname="serif"; fontsize=14;',
             '  label="DSFB grammar state machine (drift / slew / admissibility transitions)";',
             '  node [shape=box, style="rounded,filled", fontname="serif", fontsize=11, width=1.1, height=0.6];',
             '  edge [color="#555555", penwidth=1.2, arrowsize=0.8];']
    for k, (lab, fc, tc) in nodes.items():
        lines.append(f'  {k} [label="{lab}", fillcolor="{fc}", fontcolor="{tc}"];')
    for a, b in edges:
        lines.append(f'  {a} -> {b};')
    lines.append('  labeljust="l";')
    lines.append('}')
    dot = "\n".join(lines)
    ok = S.render_dot(dot, "core_grammar_fsm", "A", "conceptual (dsfb_core grammar)",
                      "The deterministic grammar state machine: nominal and structural states with transitions.")
    if not ok:
        _fsm_matplotlib_fallback()


def _fsm_matplotlib_fallback():
    """networkx/matplotlib fallback for the grammar FSM when graphviz `dot` is unavailable."""
    import networkx as nx
    g = nx.DiGraph()
    names = {"NOM": "Nominal", "DA": "Distrib.\nanomaly", "SS": "Structural\nshift",
             "EV": "Extreme\nvalue", "CP": "Change-\npoint", "RC": "Recovery"}
    g.add_edges_from([("NOM", "DA"), ("NOM", "SS"), ("DA", "EV"), ("SS", "CP"),
                      ("EV", "RC"), ("CP", "RC"), ("RC", "NOM"), ("DA", "SS")])
    # Deterministic multipartite layout: column index per state (no randomness).
    layer = {"NOM": 0, "DA": 1, "SS": 1, "EV": 2, "CP": 2, "RC": 3}
    nx.set_node_attributes(g, layer, "layer")
    pos = nx.multipartite_layout(g, subset_key="layer")
    fig, ax = plt.subplots(figsize=(7.2, 4.0))
    ax.axis("off")
    nx.draw_networkx_nodes(g, pos, ax=ax, node_size=2600,
                           node_color=[S.GRAMMAR_COLORS.get(n, "#eee") for n in g.nodes])
    nx.draw_networkx_edges(g, pos, ax=ax, arrows=True, arrowsize=16,
                           connectionstyle="arc3,rad=0.08", edge_color="#555")
    nx.draw_networkx_labels(g, pos, labels=names, ax=ax, font_size=7.5)
    ax.set_title("DSFB grammar state machine (drift/slew/admissibility transitions)", fontsize=10.5)
    S.save(fig, "core_grammar_fsm", "A", "conceptual (dsfb_core grammar)",
           "The deterministic grammar state machine: nominal and structural states with their transitions.")


def fig_quorum_fusion(run=None):
    """A6 ★ — quorum fusion schematic: per-detector timelines → quorum vote → one fused episode."""
    fig, ax = plt.subplots(figsize=(7.2, 3.8))
    ax.set_xlim(0, 10); ax.set_ylim(0, 5); ax.axis("off")
    for i, lab in enumerate(["detector 1", "detector 2", "detector 3", "detector k"]):
        y = 4.3 - i * 0.95
        _box(ax, (0.2, y - 0.3), 1.9, 0.6, lab, "#e7eef5", fontsize=7.5)
        _arrow(ax, (2.1, y), (3.9, 2.4))
    _box(ax, (3.9, 1.9), 2.0, 1.0, "quorum vote\n(≥ q detectors,\n≥ d families)", "#d6ebe2", ec=S.OK, fontsize=7.5)
    _arrow(ax, (5.9, 2.4), (7.4, 2.4))
    _box(ax, (7.4, 1.9), 2.2, 1.0, "fused episode\n(+ disagreement\nfingerprint)", "#f3e9d6", ec=S.WARN, fontsize=7.5)
    ax.set_title("Deterministic quorum fusion", fontsize=11)
    S.figure_caption(ax, "Fusion is a fixed threshold rule (no learning); sub-quorum runs are recorded as rejected candidates.")
    S.save(fig, "core_quorum_fusion", "A", "conceptual (fusion)",
           "Per-detector grammar timelines combined by a fixed quorum rule into one fused episode.")


def fig_badge_taxonomy(run=None):
    """A8 — the claim-boundary badge taxonomy every episode carries (honesty made structural)."""
    badges = [
        ("STRUCTURE_ONLY", "structural episode; no mechanism claimed"),
        ("CANDIDATE_FAULT", "matched a heuristic; candidate, not proof"),
        ("NEAR_MISS", "sub-quorum; recorded, not admitted"),
        ("SENSOR_QUALITY", "consistent with instrument fault"),
        ("CONTROL_CONTEXT_REQUIRED", "needs control logs to disambiguate"),
        ("PHYSICS_WITNESS_REQUIRED", "needs a balance witness to confirm"),
    ]
    fig, ax = plt.subplots(figsize=(7.2, 3.8))
    ax.set_xlim(0, 10); ax.set_ylim(0, len(badges) + 0.5); ax.axis("off")
    for i, (b, desc) in enumerate(badges):
        y = len(badges) - i - 0.5
        _box(ax, (0.2, y - 0.32), 3.4, 0.64, b, "#eef2f6", fontsize=7.8)
        ax.text(3.8, y, desc, va="center", fontsize=8, color="#333")
    ax.set_title("Claim-boundary badges — every episode wears one", fontsize=11)
    S.save(fig, "core_badge_taxonomy", "A", "conceptual (court_record badges)",
           "The claim-boundary badge taxonomy: each episode's epistemic status is structural, not narrative.")


def fig_certainty_tiers(run=None):
    """A9 — the Tier-1/2/3 certainty hierarchy that governs every claim in the paper."""
    tiers = [
        ("Tier 1 — proven by sealed artifacts", S.OK,
         "byte-exact replay (6/6); GPU↔CPU evidence_root identical; atlas/corpus hashes frozen"),
        ("Tier 2 — evidence-motivated interpretation", S.CB["orange"],
         "fused episodes coincide with documented onsets; baseline FP reported honestly"),
        ("Tier 3 — newly demonstrated, bounded", S.WARN,
         "regime-conditioned envelopes (penicillin 54%→39%); effect requires aligned regime labels"),
    ]
    fig, ax = plt.subplots(figsize=(7.2, 3.4))
    ax.set_xlim(0, 10); ax.set_ylim(0, 3.4); ax.axis("off")
    for i, (t, c, desc) in enumerate(tiers):
        y = 2.7 - i * 1.0
        ax.add_patch(FancyBboxPatch((0.3, y - 0.36), 9.4, 0.72, boxstyle="round,pad=0.02",
                                    fc=c, ec="#333", alpha=0.18, lw=1.0))
        ax.text(0.55, y + 0.12, t, fontsize=9, weight="bold", color="#111", va="center")
        ax.text(0.55, y - 0.16, desc, fontsize=7.3, color="#333", va="center")
    ax.set_title("Certainty hierarchy: what is proven vs interpreted vs newly bounded", fontsize=10.5)
    S.save(fig, "core_certainty_tiers", "A", "conceptual (paper claim hierarchy)",
           "The three-tier certainty hierarchy separating sealed proof from bounded interpretation.")


def render_all(run):
    """Render every group-A figure for the given run directory."""
    S.log("group A — core method made visible")
    fig_residual_triple(run)
    fig_admissibility_envelope(run)
    fig_drift_slew_decomposition(run)
    fig_multidetector_grammar(run)
    fig_episode_lifecycle(run)
    fig_pipeline_architecture(run)
    fig_grammar_fsm(run)
    fig_quorum_fusion(run)
    fig_badge_taxonomy(run)
    fig_certainty_tiers(run)
