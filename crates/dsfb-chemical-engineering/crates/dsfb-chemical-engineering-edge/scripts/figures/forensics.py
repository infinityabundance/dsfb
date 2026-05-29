"""Figure group E — disagreement forensics, negative witnesses, confusers (5 designs).

The interpretability core: which detectors fired and which stayed *silent* (and that silence is itself
evidence), how much they disagreed, and which look-alike faults were ruled out. Read from the demo's
`episode_evidence.csv` (per-episode detectors_fired/silent, families, consensus, disagreement entropy) and
the exported `confuser_docket.json`. Honest: silence is reported as a negative witness, not hidden.
"""
import json
import os

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.colors import ListedColormap

from . import style as S

REP = "tennessee_eastman_idv01"  # many episodes → a legible participation matrix


def _episodes(run, dataset):
    """Per-episode evidence rows for one dataset from episode_evidence.csv."""
    return [r for r in S.read_csv_rows(S.ds_csv(run, dataset, "episode_evidence.csv"))]


def _split(field):
    """Split a `|`-delimited CSV list field into a clean list (empty → [])."""
    return [x for x in field.split("|") if x] if field else []


def fig_participation_matrix(run):
    """E32 ★ — the detector-participation matrix per episode: who fired (filled) vs stayed silent (blank).

    Rows = fused episodes, columns = the detector roster; a cell is filled if that detector fired in that
    episode. Silence is information — a detector that never fires for a fault is a negative witness.
    """
    eps = _episodes(run, REP)
    if not eps:
        S.log("  [E] SKIP forensics_participation: no episodes")
        return
    detectors = sorted(set().union(*[set(_split(e["detectors_fired"]) + _split(e["detectors_silent"])) for e in eps]))
    M = np.zeros((len(eps), len(detectors)))
    for i, e in enumerate(eps):
        fired = set(_split(e["detectors_fired"]))
        for j, d in enumerate(detectors):
            M[i, j] = 1.0 if d in fired else 0.0
    fig, ax = plt.subplots(figsize=(8.0, max(2.6, 0.32 * len(eps) + 1.4)))
    ax.imshow(M, aspect="auto", cmap=ListedColormap(["#eef2f6", S.ACCENT]), vmin=0, vmax=1)
    ax.set_xticks(range(len(detectors))); ax.set_xticklabels(detectors, rotation=55, ha="right", fontsize=6.5)
    ax.set_yticks(range(len(eps)))
    ax.set_yticklabels([f"[{e['episode_start']},{e['episode_end']}]" for e in eps], fontsize=6.8)
    ax.set_ylabel("fused episode")
    ax.set_title(f"Detector participation per episode — {S.disp(REP)} (filled = fired; blank = silent)", fontsize=10)
    handles = [plt.Line2D([0], [0], marker="s", ls="", color=S.ACCENT, label="fired"),
               plt.Line2D([0], [0], marker="s", ls="", color="#eef2f6", mec="#999", label="silent (negative witness)")]
    ax.legend(handles=handles, frameon=False, fontsize=7.5, loc="upper center", bbox_to_anchor=(0.5, -0.16), ncol=2)
    S.save(fig, "forensics_participation", "E", f"{REP}/episode_evidence.csv",
           "Which detectors fired vs stayed silent per episode — silence is recorded as a negative witness.")


def fig_negative_witnesses(run):
    """E33 — negative-witness frequency: how often each detector stays silent across episodes.

    A detector that is silent in most/all episodes is a persistent negative witness — its silence narrows the
    subspace the fault lives in. (e.g. `sensor_bias` silent throughout a process-structure fault.)
    """
    eps = _episodes(run, REP)
    if not eps:
        return
    detectors = sorted(set().union(*[set(_split(e["detectors_fired"]) + _split(e["detectors_silent"])) for e in eps]))
    silent_freq = {d: 0 for d in detectors}
    for e in eps:
        for d in _split(e["detectors_silent"]):
            silent_freq[d] = silent_freq.get(d, 0) + 1
    order = sorted(detectors, key=lambda d: silent_freq[d])
    vals = [100.0 * silent_freq[d] / len(eps) for d in order]
    fig, ax = plt.subplots(figsize=(7.2, 4.6))
    colors = [S.WARN if v >= 99.9 else (S.CB["orange"] if v >= 50 else S.MUTE) for v in vals]
    ax.barh(order, vals, color=colors)
    ax.set_xlabel("% of episodes in which the detector is silent")
    ax.set_title(f"Negative witnesses — persistent detector silence ({S.disp(REP)})")
    ax.axvline(100, color="#333", lw=0.7, ls=":")
    S.figure_caption(ax, "A detector silent in (near-)all episodes is a negative witness: its silence constrains the fault subspace.")
    S.save(fig, "forensics_negative_witnesses", "E", f"{REP}/episode_evidence.csv",
           "How often each detector stays silent across episodes — persistent silence is a negative witness.")


def fig_disagreement_entropy(run):
    """E35 — disagreement-entropy per episode over time: how divided the detector bank was at each episode."""
    eps = _episodes(run, REP)
    if not eps:
        return
    x = [int(e["episode_start"]) for e in eps]
    y = [S.fnum(e["disagreement_entropy"]) for e in eps]
    cons = [S.fnum(e["consensus_strength"]) for e in eps]
    fig, ax = plt.subplots(figsize=(7.2, 3.6))
    ax.plot(x, y, marker="o", color=S.WARN, lw=1.2, label="disagreement entropy")
    ax.plot(x, cons, marker="s", color=S.OK, lw=1.0, alpha=0.8, label="consensus strength")
    ax.set_xlabel("episode start (sample index)")
    ax.set_ylabel("normalised value")
    ax.set_title(f"Detector disagreement vs consensus per episode — {S.disp(REP)}")
    ax.legend(frameon=False, fontsize=8, loc="upper right")
    S.figure_caption(ax, "High disagreement entropy + low consensus marks structurally ambiguous episodes (often emitted as unknown).")
    S.save(fig, "forensics_disagreement_entropy", "E", f"{REP}/episode_evidence.csv",
           "Per-episode detector disagreement entropy alongside consensus strength.")


def fig_witness_diversity(run):
    """E36 — witness diversity per episode: how many detector families fired, coloured by evidence grade.

    More independent families firing = stronger, less correlated evidence. The bar height is the family
    count; the colour is the episode's evidence grade.
    """
    eps = _episodes(run, REP)
    if not eps:
        return
    labels = [f"[{e['episode_start']},{e['episode_end']}]" for e in eps]
    nfam = [len(_split(e["families_fired"])) for e in eps]
    grades = [e.get("evidence_grade", "") for e in eps]
    grade_color = {"EVIDENCE_STRONG": S.OK, "EVIDENCE_MODERATE": S.CB["orange"],
                   "EVIDENCE_THIN_SUPPORT": S.MUTE, "EVIDENCE_SHORT_WINDOW": S.WARN}
    colors = [grade_color.get(g, S.MUTE) for g in grades]
    fig, ax = plt.subplots(figsize=(7.2, 4.0))
    ax.bar(range(len(eps)), nfam, color=colors)
    ax.set_xticks(range(len(eps))); ax.set_xticklabels(labels, rotation=55, ha="right", fontsize=6.5)
    ax.set_ylabel("detector families fired")
    ax.set_title(f"Witness diversity per episode (colour = evidence grade) — {S.disp(REP)}")
    handles = [plt.Line2D([0], [0], marker="s", ls="", color=c, label=g.replace("EVIDENCE_", ""))
               for g, c in grade_color.items()]
    ax.legend(handles=handles, frameon=False, fontsize=7, loc="upper right", ncol=2)
    S.figure_caption(ax, "Independent families firing = stronger, less-correlated evidence; grade encodes support strength + window length.")
    S.save(fig, "forensics_witness_diversity", "E", f"{REP}/episode_evidence.csv",
           "Number of detector families firing per episode, coloured by the episode's evidence grade.")


def fig_confuser_tree(run):
    """E34 ★ — the confuser elimination tree: matched fault → discriminating signature → confusers ruled out.

    Rendered from the exported confuser_docket.json (which cites only the catalogued confusers of the matched
    fault signature — it invents none). graphviz tree.
    """
    path = os.path.join(run, "figure_data", "confuser_docket.json")
    if not os.path.exists(path):
        S.log("  [E] SKIP forensics_confuser_tree: confuser_docket.json not found")
        return
    d = json.load(open(path))
    sig = d.get("discriminating_signature", "")
    # Wrap the long signature label.
    import textwrap
    sig_w = "\\n".join(textwrap.wrap(sig, 46))
    lines = ['digraph confuser {', '  rankdir=LR; bgcolor="white";',
             '  graph [fontname="serif", fontsize=12, labelloc="t", label="Confuser docket — what was ruled out, and why"];',
             '  node [fontname="serif", fontsize=9, style="filled,rounded", shape=box];']
    lines.append(f'  matched [label="MATCHED (candidate): {d.get("matched_fault_id","")}\\n{d.get("matched_fault_name","")}", fillcolor="#d6ebe2", color="#009E73", penwidth=2];')
    lines.append(f'  sig [label="discriminating signature:\\n{sig_w}", fillcolor="#f3e9d6", shape=note];')
    lines.append('  matched -> sig [style=bold, label="distinguished by"];')
    for i, c in enumerate(d.get("confusers", [])):
        cw = "\\n".join(textwrap.wrap(c, 28))
        lines.append(f'  c{i} [label="ruled out:\\n{cw}", fillcolor="#eef2f6"];')
        lines.append(f'  matched -> c{i} [style=dashed, color="#D55E00", label="not this"];')
    lines.append('}')
    dot = "\n".join(lines)
    S.render_dot(dot, "forensics_confuser_tree", "E", "figure_data/confuser_docket.json",
                 "Confuser elimination: the matched candidate fault, its discriminating signature, and the catalogued look-alikes ruled out.")


def render_all(run):
    """Render every group-E figure from episode_evidence.csv + the exported confuser docket."""
    S.log("group E — disagreement forensics / negative witnesses / confusers")
    fig_participation_matrix(run)
    fig_negative_witnesses(run)
    fig_disagreement_entropy(run)
    fig_witness_diversity(run)
    fig_confuser_tree(run)
