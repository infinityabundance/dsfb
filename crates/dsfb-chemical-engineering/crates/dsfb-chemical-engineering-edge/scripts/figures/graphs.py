"""Figure group D — topology, propagation, and provenance graphs (6 designs; panel rec R5).

These render the *actual* DOT emitted by the crate's graph objects (topology / causal-non-claim / residual
provenance), exported to `<run>/figure_data/*.dot` by `figure_export.rs`, so the figures show the canonical
crate output rather than a Python mock-up. A small style preamble is injected for house consistency
(serif, filled nodes) while preserving the crate's node shapes and labels. The two data-driven figures
(propagation onset alignment, residence-time correlation) read the exported JSON/CSV.
"""
import json
import os
import textwrap

import numpy as np
import matplotlib.pyplot as plt

from . import style as S


def _data(run, name):
    """Path to an exported figure_data artifact."""
    return os.path.join(run, "figure_data", name)


def _render_dot_file(run, fname, fid, caption, wrap_graph_label=False):
    """Read a crate-emitted .dot file, inject a house-style preamble, and render it via graphviz.

    The crate's `to_dot()` defines the node shapes/labels/edges (faithful); we add graph/node/edge default
    attributes for serif fonts + filled nodes. If `wrap_graph_label`, the (possibly very long) graph-level
    label — e.g. the causal-non-claim disclaimer — is hard-wrapped so the rendered image is not absurdly wide.
    """
    path = _data(run, fname)
    if not os.path.exists(path):
        S.log(f"  [D] SKIP {fid}: {fname} not found (run `dsfb-chem-edge figures` to export it)")
        return
    src = open(path).read()
    if wrap_graph_label:
        # Wrap the graph label string: find label="…"; (first occurrence) and rewrap to ~70-char lines.
        import re
        m = re.search(r'label="([^"]*)";', src)
        if m:
            wrapped = "\\n".join(textwrap.wrap(m.group(1), 72))
            src = src[:m.start()] + f'label="{wrapped}";' + src[m.end():]
    # Inject defaults right after the opening "digraph NAME {".
    brace = src.index("{") + 1
    preamble = ('\n  graph [fontname="serif", fontsize=12, labelloc="t"];'
                '\n  node [fontname="serif", fontsize=10, style="filled", fillcolor="#e7eef5"];'
                '\n  edge [fontname="serif", fontsize=9, color="#555555"];')
    src = src[:brace] + preamble + src[brace:]
    S.render_dot(src, fid, "D", f"figure_data/{fname} (crate to_dot)", caption)


def fig_topology(run):
    """D26 ★ — the process topology graph (feed → reactor → separator) with declared residence times."""
    _render_dot_file(run, "topology.dot", "graph_topology",
                     "Process-unit topology with declared residence times — the crate's ProcessTopologyGraphV1.to_dot().")


def fig_causal_non_claim(run):
    """D29 — the causal-non-claim graph: precedence + topology edges with the sealed NO-CAUSAL-CLAIM disclaimer."""
    _render_dot_file(run, "causal_non_claim.dot", "graph_causal_non_claim",
                     "Precedence + topology edges (dashed) under a sealed NO-CAUSAL-CLAIM disclaimer — not a causal proof.",
                     wrap_graph_label=True)


def fig_provenance(run):
    """D30 ★ — the residual provenance DAG: raw → residual → detector → episode → label → court_root."""
    _render_dot_file(run, "provenance.dot", "graph_provenance",
                     "Residual provenance DAG (raw→residual→detector→episode→label→evidence_root) — ResidualProvenanceGraphV1.")


def fig_propagation(run):
    """D27 ★ — fault-propagation onset alignment: upstream vs downstream onset, observed vs declared lag.

    A two-track timeline from propagation_witness.json: the reactor onset, the separator onset, the declared
    residence lag, and whether the observed lag is consistent — with the mandatory non-causal disclaimer.
    """
    path = _data(run, "propagation_witness.json")
    if not os.path.exists(path):
        S.log("  [D] SKIP graph_propagation: propagation_witness.json not found")
        return
    w = json.load(open(path))
    up_on = w.get("upstream_onset", 100)
    dn_on = w.get("downstream_onset", 105)
    declared = w.get("declared_residence_lag", w.get("declared_lag", 5))
    observed = w.get("observed_lag", dn_on - up_on)
    consistent = w.get("lag_consistent", True)
    fig, ax = plt.subplots(figsize=(7.2, 3.0))
    ax.hlines(1, 0, max(dn_on, up_on) + 20, color="#ccc", lw=1)
    ax.hlines(0, 0, max(dn_on, up_on) + 20, color="#ccc", lw=1)
    ax.plot([up_on], [1], marker="o", ms=11, color=S.WARN)
    ax.plot([dn_on], [0], marker="o", ms=11, color=S.WARN)
    ax.annotate("", xy=(dn_on, 0.1), xytext=(up_on, 0.9),
                arrowprops=dict(arrowstyle="-|>", color=S.ACCENT, lw=1.4))
    ax.text((up_on + dn_on) / 2, 0.5,
            f"  observed lag = {observed}\n  declared τ = {declared}\n  {'consistent' if consistent else 'inconsistent'}",
            fontsize=8.5, color=(S.OK if consistent else S.WARN))
    ax.text(up_on, 1.12, f"reactor onset @{up_on}", fontsize=8, ha="center")
    ax.text(dn_on, -0.18, f"separator onset @{dn_on}", fontsize=8, ha="center")
    ax.set_yticks([0, 1]); ax.set_yticklabels(["separator\n(downstream)", "reactor\n(upstream)"], fontsize=8)
    ax.set_ylim(-0.5, 1.5)
    ax.set_xlabel("sample index")
    ax.set_title("Fault-propagation candidate: onset lag vs declared residence time")
    S.figure_caption(ax, S.DISCLAIMER["noncausal"])
    S.save(fig, "graph_propagation", "D", "figure_data/propagation_witness.json",
           "Upstream→downstream onset alignment: observed lag vs declared residence, consistency flagged.")


def fig_residence_alignment(run):
    """D28 — residence-time alignment: upstream vs downstream signal overlaid at the declared lag, with r."""
    apath, spath = _data(run, "residence_alignment.json"), _data(run, "residence_series.csv")
    if not (os.path.exists(apath) and os.path.exists(spath)):
        S.log("  [D] SKIP graph_residence_alignment: residence data not found")
        return
    a = json.load(open(apath))
    rows = S.read_csv_rows(spath)
    t = np.array([int(r["t"]) for r in rows])
    up = np.array([S.fnum(r["upstream"]) for r in rows])
    dn = np.array([S.fnum(r["downstream"]) for r in rows])
    lag = a.get("lag_samples", 3)
    corr = a.get("aligned_correlation", float("nan"))
    fig, ax = plt.subplots(2, 1, figsize=(7.2, 3.8), sharex=True)
    ax[0].plot(t, up, color=S.ACCENT, lw=1.0, label="upstream (reactor)")
    ax[0].plot(t, dn, color=S.WARN, lw=1.0, alpha=0.8, label="downstream (separator)")
    ax[0].legend(frameon=False, fontsize=7.5, loc="upper right")
    ax[0].set_title(f"Residence-time alignment — at-lag Pearson r = {corr:.3f} (lag {lag} samples)", fontsize=10)
    # Shift downstream back by the lag to show the at-lag overlay.
    ax[1].plot(t, up, color=S.ACCENT, lw=1.0, label="upstream")
    ax[1].plot(t - lag, dn, color=S.WARN, lw=1.0, alpha=0.8, label=f"downstream shifted −{lag}")
    ax[1].legend(frameon=False, fontsize=7.5, loc="upper right")
    ax[1].set_xlabel("sample index")
    ax[1].set_ylabel("at-lag overlay", fontsize=8)
    S.figure_caption(ax[1], "Correlation evaluated at the declared residence lag; advisory alignment, not a causal claim.")
    S.save(fig, "graph_residence_alignment", "D", "figure_data/residence_alignment.json + residence_series.csv",
           "Upstream/downstream signals overlaid at the declared residence lag, with the at-lag correlation.")


def fig_provenance_flow(run):
    """D31 — provenance ledger flow: counts of samples at each provenance stage (raw→…→episode→label).

    A layered flow built from the demo's residual_provenance_ledger.csv: how many samples are baseline vs
    in-episode vs labelled — the population view that complements the single-chain provenance DAG.
    """
    # Use the representative dataset's provenance ledger from the demo output.
    led = S.read_csv_rows(S.ds_csv(run, "cstr_reactor", "residual_provenance_ledger.csv"))
    if not led:
        led = S.read_csv_rows(S.ds_csv(run, "tennessee_eastman_idv01", "residual_provenance_ledger.csv"))
    if not led:
        S.log("  [D] SKIP graph_provenance_flow: no provenance ledger found")
        return
    # The ledger columns vary; count rows by a coarse stage classification that is robust to schema.
    n = len(led)
    cols = led[0].keys()
    in_ep = sum(1 for r in led if any(("episode" in k.lower() and str(r[k]).strip() not in ("", "0", "-1", "na")) for k in cols))
    baseline = sum(1 for r in led if any(("baseline" in k.lower() and str(r[k]).strip().lower() in ("1", "true", "yes")) for k in cols))
    stages = [("raw samples", n, S.MUTE), ("baseline", baseline or max(n - in_ep, 0), S.CB["skyblue"]),
              ("in episode", in_ep, S.WARN)]
    fig, ax = plt.subplots(figsize=(7.0, 3.0))
    x = np.arange(len(stages))
    ax.bar(x, [s[1] for s in stages], color=[s[2] for s in stages], width=0.55)
    for xi, s in zip(x, stages):
        ax.text(xi, s[1], f"{s[1]}", ha="center", va="bottom", fontsize=9)
    ax.set_xticks(x); ax.set_xticklabels([s[0] for s in stages])
    ax.set_ylabel("samples")
    ax.set_title("Residual provenance population (from the provenance ledger)")
    S.figure_caption(ax, "Per-sample provenance from residual_provenance_ledger.csv; complements the single-chain provenance DAG.")
    S.save(fig, "graph_provenance_flow", "D", "cstr_reactor/residual_provenance_ledger.csv",
           "Population view of the provenance ledger: raw → baseline → in-episode sample counts.")


def render_all(run):
    """Render every group-D figure (crate DOT + data-driven alignment/propagation)."""
    S.log("group D — topology / propagation / provenance graphs")
    fig_topology(run)
    fig_propagation(run)
    fig_residence_alignment(run)
    fig_causal_non_claim(run)
    fig_provenance(run)
    fig_provenance_flow(run)
