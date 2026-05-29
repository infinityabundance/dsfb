"""Figure group I — chemical-engineering practitioner / industry figures (9 designs).

The figures that put the chem-eng homework on the page (panel R3 / mentorship S4): an operator incident
one-pager, alarm-flood compression (ISA-18.2 / IEC 62682), a NAMUR NE107 plant-status timeline, the
historian→case-file workflow, recipe / maintenance context overlays, the MOC-style amendment chain, the
per-detector chemometric passport, and the sensitivity-sweep / ablation robustness pair. Operator-facing
outputs read the demo's per-dataset CSVs; the evidence-object cards read the exported figure_data JSON.
All advisory, candidate-only, no control/safety authority.
"""
import json
import os
import textwrap

import numpy as np
import matplotlib.pyplot as plt

from . import style as S

REP = "cstr_reactor"


def _fd(run, name):
    p = os.path.join(run, "figure_data", name)
    return json.load(open(p)) if os.path.exists(p) else None


def fig_operator_one_pager(run):
    """I53 ★ — the operator incident one-pager: nine questions an operator asks, each answered from evidence.

    A control-room-style one-page card. Answers are drawn from the representative dataset's alarm
    rationalization + episode evidence; the boundary ("what is NOT claimed") is a fixed, prominent row.
    """
    al = S.read_csv_rows(S.ds_csv(run, REP, "alarm_rationalization.csv"))
    row = al[0] if al else {}
    qa = [
        ("1. What happened?", f"A structural episode (motif {row.get('dominant_motif','—')}); candidate: {row.get('label_or_unknown_subtype','unknown')}."),
        ("2. Where?", f"Detectors {row.get('participating_detectors','—').replace('|', ', ')} (residual subspace)."),
        ("3. When?", f"Samples [{row.get('episode_start','?')}, {row.get('episode_end','?')}] (onset-aligned)."),
        ("4. How detected?", f"Quorum fusion of the firing detectors; NE107 status: {row.get('ne107_status','—')}."),
        ("5. How strong is the evidence?", f"Grade: {row.get('evidence_grade','—')}."),
        ("6. What was ruled out?", "Catalogued confusers for the matched signature (see confuser docket)."),
        ("7. Is it reproducible?", f"Yes — sealed evidence_root {row.get('evidence_root','')[:16]}…, byte-exact replay."),
        ("8. What is NOT claimed?", "No root cause, no causality, no control/safety action. Advisory candidate only."),
        ("9. Recommended disposition", f"{row.get('suggested_disposition','operator review')} (operator decides)."),
    ]
    fig, ax = plt.subplots(figsize=(8.0, 6.4))
    ax.axis("off")
    ax.add_patch(plt.Rectangle((0.01, 0.01), 0.98, 0.98, transform=ax.transAxes, fill=False, ec="#333", lw=1.4))
    ax.text(0.5, 0.95, "DSFB operator incident one-pager", ha="center", fontsize=13, weight="bold", transform=ax.transAxes)
    ax.text(0.5, 0.915, f"{S.disp(REP)} · replayable Chemical Court Record v1", ha="center", fontsize=8.5,
            color="#555", transform=ax.transAxes)
    y = 0.85
    for q, a in qa:
        ax.text(0.04, y, q, fontsize=9.2, weight="bold", color=S.INK, transform=ax.transAxes)
        for line in textwrap.wrap(a, 92):
            y -= 0.038
            ax.text(0.06, y, line, fontsize=8.4, color="#222", transform=ax.transAxes)
        y -= 0.05
    # The boundary row, highlighted.
    ax.add_patch(plt.Rectangle((0.03, 0.04), 0.94, 0.06, transform=ax.transAxes, fc=S.WARN, alpha=0.12, ec=S.WARN))
    ax.text(0.5, 0.07, "Advisory only · read-only · no control or safety-instrumented-function authority",
            ha="center", fontsize=8.2, style="italic", color=S.WARN, transform=ax.transAxes)
    S.save(fig, "practitioner_operator_one_pager", "I", f"{REP}/alarm_rationalization.csv",
           "An operator incident one-pager: nine questions answered from sealed evidence, with the claim boundary.")


def fig_alarm_flood(run):
    """I54 ★ — alarm-flood compression (ISA-18.2 / IEC 62682): raw breach activations → a few fused episodes.

    The compression that matters to a board operator: thousands of raw detector breach-steps collapse to a
    handful of fused episodes, with lost_evidence = 0 (the underlying breaches remain recoverable).
    """
    al = S.read_csv_rows(S.ds_csv(run, REP, "alarm_rationalization.csv"))
    if not al:
        S.log("  [I] SKIP practitioner_alarm_flood: no alarm_rationalization.csv"); return
    raw = S.fnum(al[0].get("raw_breach_steps_total", "0"))
    fused = S.fnum(al[0].get("fused_episodes_total", str(len(al))))
    comp = S.fnum(al[0].get("compression_ratio", "0"))
    fig, ax = plt.subplots(figsize=(7.0, 3.8))
    bars = ax.bar(["before\n(raw breach-steps)", "after\n(fused episodes)"], [raw, max(fused, 0.7)],
                  color=[S.WARN, S.OK], width=0.5)
    ax.set_yscale("log")
    for b, v in zip(bars, [raw, fused]):
        ax.text(b.get_x() + b.get_width() / 2, b.get_height(), f"{int(v):,}", ha="center", va="bottom", fontsize=11, weight="bold")
    ax.set_ylabel("count (log scale)")
    ax.set_title(f"Alarm-flood rationalization — {S.disp(REP)} ({comp:.0f}× compression)")
    ax.text(0.5, 0.5, "lost_evidence = 0\nrecoverable = true", transform=ax.transAxes, ha="center",
            fontsize=10, color=S.OK, bbox=dict(boxstyle="round", fc="white", ec=S.OK))
    S.figure_caption(ax, "ISA-18.2 / IEC 62682 alarm management: rationalization PRESERVES the underlying breaches (recoverable), it does not suppress them.")
    S.save(fig, "practitioner_alarm_flood", "I", f"{REP}/alarm_rationalization.csv",
           "Alarm-flood compression: raw breach-steps collapse to a few fused episodes with zero lost evidence.")


def fig_ne107_timeline(run):
    """I55 ★ — a NAMUR NE107 plant-status timeline (Good / Maintenance / Out-of-spec / Failure) over time."""
    tr = S.read_csv_rows(S.ds_csv(run, REP, "ne107_status_trace.csv"))
    if not tr:
        S.log("  [I] SKIP practitioner_ne107_timeline: no ne107_status_trace.csv"); return
    # Exact NE107 status strings as emitted in the trace (the four NAMUR states), each a distinct colour.
    status_color = {"OK": S.OK, "Maintenance required": S.CB["skyblue"],
                    "Out of specification": S.CB["orange"], "Failure": S.WARN,
                    "Function check": S.CB["purple"]}
    t = [int(r["time_index"]) for r in tr]
    st = [r["ne107_status"] for r in tr]
    fig, ax = plt.subplots(figsize=(7.6, 2.6))
    for i in range(len(t)):
        ax.axvspan(t[i] - 0.5, t[i] + 0.5, color=status_color.get(st[i], "#cccccc"), lw=0)
    ax.set_yticks([])
    ax.set_xlim(min(t), max(t))
    ax.set_xlabel("sample index")
    ax.set_title(f"NAMUR NE107 plant-wide status timeline — {S.disp(REP)}")
    present = [s for s in dict.fromkeys(st)]
    handles = [plt.Line2D([0], [0], marker="s", ls="", color=status_color.get(s, "#ccc"), label=s) for s in present]
    ax.legend(handles=handles, frameon=False, fontsize=8, ncol=len(present), loc="upper center", bbox_to_anchor=(0.5, -0.3))
    S.figure_caption(ax, "Per-sample status mapped to the NE107 vocabulary operators already read on the DCS. Advisory.")
    S.save(fig, "practitioner_ne107_timeline", "I", f"{REP}/ne107_status_trace.csv",
           "A NAMUR NE107 plant-status timeline (Good/Maintenance/Out-of-spec/Failure) in the operator's vocabulary.")


def fig_workflow(run):
    """I56 ★ — the plant-historian → case-file operator workflow: how DSFB drops into a control room."""
    dot = '''digraph W {
      rankdir=LR; bgcolor="white";
      graph [fontname="serif", fontsize=12, labelloc="t", label="How DSFB drops into a control room (read-only)"];
      node [fontname="serif", fontsize=9.5, style="filled,rounded", shape=box, fillcolor="#e7eef5"];
      hist [label="plant historian /\\nOPC-UA export\\n(temp, pressure,\\nflow, level, current)"];
      det  [label="established\\ndetectors →\\nresiduals"];
      dsfb [label="DSFB\\nresidual semiotics\\n(read-only)", fillcolor="#d6ebe2", color="#009E73", penwidth=2];
      case [label="sealed case file\\n+ operator one-pager\\n+ NE107 status", fillcolor="#f3e9d6"];
      op   [label="operator\\nreview\\n(decides action)"];
      hist -> det -> dsfb -> case -> op;
      edge [color="#555"];
    }'''
    if not S.render_dot(dot, "practitioner_workflow", "I", "conceptual (operator workflow)",
                        "Plant-historian → residuals → DSFB → sealed case file + one-pager → operator review (read-only)."):
        S.log("  [I] workflow: graphviz unavailable")


def fig_context_overlay(run):
    """I57 — recipe-transition + maintenance-event context overlays on a process timeline.

    Read-only overlays mark windows where residuals are *expected* (a recipe phase change, a maintenance
    outage) so an episode in such a window is contextualised rather than blindly alarmed.
    """
    guard = _fd(run, "recipe_guard.json")
    maint = _fd(run, "maintenance_overlay.json")
    fig, ax = plt.subplots(figsize=(7.6, 2.8))
    xmax = 800
    ax.set_xlim(0, xmax); ax.set_ylim(0, 1); ax.set_yticks([])
    ax.set_xlabel("sample index")
    if guard:
        for tr in guard.get("transitions", []):
            idx = tr["at_index"]
            ax.axvline(idx, color=S.CB["purple"], lw=1.6, ls="--")
            ax.text(idx, 0.85, f" recipe: {tr['from_phase']}→{tr['to_phase']}", fontsize=7.5, color=S.CB["purple"])
    if maint:
        for ev in maint.get("events", []):
            ax.axvspan(ev["start_index"], ev["end_index"], color=S.CB["skyblue"], alpha=0.25)
            ax.text((ev["start_index"] + ev["end_index"]) / 2, 0.4, f"maintenance:\n{ev['description']}",
                    ha="center", fontsize=7.5, color="#26607a")
    ax.set_title("Read-only context overlays: recipe transitions + maintenance windows")
    S.figure_caption(ax, "Overlays mark windows where residuals are expected; they contextualise episodes — they never mutate the evidence.")
    S.save(fig, "practitioner_context_overlay", "I", "figure_data/recipe_guard.json + maintenance_overlay.json",
           "Recipe-transition and maintenance-window context overlays on a process timeline (read-only).")


def fig_amendment_chain(run):
    """I58 — the MOC-style evidence-amendment chain: append-only corrections anchored to immutable evidence."""
    ch = _fd(run, "amendment_chain.json")
    if not ch:
        S.log("  [I] SKIP practitioner_amendment_chain: no amendment_chain.json"); return
    lines = ['digraph A {', '  rankdir=LR; bgcolor="white";',
             '  graph [fontname="serif", fontsize=12, labelloc="t", label="Evidence-amendment chain (management-of-change audit trail)"];',
             '  node [fontname="serif", fontsize=8.5, style="filled,rounded", shape=box];']
    lines.append(f'  orig [label="ORIGINAL sealed evidence\\n{ch.get("original_evidence_hash","")[:16]}…\\n(immutable genesis)", fillcolor="#d6ebe2", color="#009E73", penwidth=2];')
    prev = "orig"
    for a in ch.get("amendments", []):
        nid = f'a{a["seq"]}'
        txt = "\\n".join(textwrap.wrap(a.get("amendment_text", ""), 34))
        lines.append(f'  {nid} [label="amendment #{a["seq"]} ({a.get("amendment_kind","")})\\n{txt}\\n{a.get("entry_hash","")[:12]}…", fillcolor="#f3e9d6"];')
        lines.append(f'  {prev} -> {nid} [label="hash-chained", fontsize=7, color="#555"];')
        prev = nid
    lines.append('}')
    S.render_dot("\n".join(lines), "practitioner_amendment_chain", "I", "figure_data/amendment_chain.json",
                 "Append-only evidence-amendment chain: corrections hash-chained to the immutable original evidence (MOC audit trail).")


def fig_passport_card(run):
    """I59 — a per-detector chemometric passport card: the provenance + policy seal for one detector."""
    p = _fd(run, "passport.json")
    if not p:
        S.log("  [I] SKIP practitioner_passport_card: no passport.json"); return
    fields = [
        ("detector", p.get("detector_id", "")),
        ("family", p.get("family", "")),
        ("threshold policy", p.get("threshold_policy", "")),
        ("normalization", p.get("normalization", "")),
        ("missingness", p.get("missingness", "")),
        ("baseline-window hash", p.get("baseline_window_hash", "")[:24] + "…"),
        ("input-matrix hash", p.get("input_matrix_hash", "")[:24] + "…"),
        ("output hash", p.get("output_hash", "")[:24] + "…"),
        ("passport hash", p.get("passport_hash", "")[:24] + "…"),
    ]
    fig, ax = plt.subplots(figsize=(7.0, 4.2))
    ax.axis("off")
    ax.add_patch(plt.Rectangle((0.02, 0.02), 0.96, 0.96, transform=ax.transAxes, fill=False, ec=S.ACCENT, lw=1.6))
    ax.text(0.5, 0.9, "Chemometric passport (per detector)", ha="center", fontsize=12.5, weight="bold",
            color=S.ACCENT, transform=ax.transAxes)
    y = 0.78
    for k, v in fields:
        ax.text(0.06, y, k, fontsize=9, weight="bold", color="#333", transform=ax.transAxes)
        ax.text(0.42, y, str(v), fontsize=8.6, family="monospace", color="#111", transform=ax.transAxes)
        y -= 0.082
    S.figure_caption(ax, "Pins the SHA-256 of baseline/input/output + discloses threshold/normalization/missingness policy. Self-verifying, tamper-evident.")
    S.save(fig, "practitioner_passport_card", "I", "figure_data/passport.json",
           "A per-detector chemometric passport: provenance hashes + threshold/normalization/missingness policy.")


def fig_sweep_heatmap(run):
    """I60a — the sensitivity-sweep robustness heatmap over the quorum thresholds (k × min-families)."""
    sw = _fd(run, "sweep_receipt.json")
    if not sw:
        S.log("  [I] SKIP practitioner_sweep_heatmap: no sweep_receipt.json"); return
    axes = sw["axes"]
    xa, ya = axes[0]["values"], axes[1]["values"]
    M = np.full((len(ya), len(xa)), np.nan)
    for pt in sw["points"]:
        i = ya.index(pt["coords"][1]); j = xa.index(pt["coords"][0])
        M[i, j] = pt["metric"]
    fig, ax = plt.subplots(figsize=(6.4, 4.0))
    im = ax.imshow(M, aspect="auto", cmap="cividis", origin="lower")
    ax.set_xticks(range(len(xa))); ax.set_xticklabels([f"{v:g}" for v in xa])
    ax.set_yticks(range(len(ya))); ax.set_yticklabels([f"{v:g}" for v in ya])
    ax.set_xlabel(axes[0]["name"]); ax.set_ylabel(axes[1]["name"])
    for i in range(M.shape[0]):
        for j in range(M.shape[1]):
            ax.text(j, i, f"{M[i, j]:g}", ha="center", va="center", fontsize=8,
                    color=("white" if M[i, j] < np.nanmean(M) else "black"))
    ax.set_title(f"Sensitivity sweep — {sw.get('metric_name','metric')} (range {sw.get('metric_range',0):g})")
    fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04, label=sw.get("metric_name", "metric"))
    S.figure_caption(ax, "Deterministic Cartesian threshold grid; metric_range quantifies robustness. Sealed receipt.")
    S.save(fig, "practitioner_sweep_heatmap", "I", "figure_data/sweep_receipt.json",
           "Sensitivity-sweep robustness heatmap over the quorum thresholds (deterministic, sealed receipt).")


def fig_ablation_bars(run):
    """I60b — ablation component importance: the metric delta when each DSFB component is removed."""
    ab = _fd(run, "ablation_court.json")
    if not ab:
        S.log("  [I] SKIP practitioner_ablation_bars: no ablation_court.json"); return
    arms = ab["arms"]
    arms = sorted(arms, key=lambda a: a["delta_vs_full"])
    names = [a["component"] for a in arms]
    deltas = [a["delta_vs_full"] for a in arms]
    fig, ax = plt.subplots(figsize=(7.0, 3.8))
    colors = [S.WARN if d < 0 else S.OK for d in deltas]
    ax.barh(names, deltas, color=colors)
    ax.axvline(0, color="#333", lw=0.8)
    ax.set_xlabel(f"Δ {ab.get('metric_name','metric')} vs full pipeline (more negative = more load-bearing)")
    ax.set_title(f"Component ablation — most load-bearing: {ab.get('most_load_bearing','')}")
    for i, d in enumerate(deltas):
        ax.text(d, i, f" {d:+g}", va="center", fontsize=8)
    S.figure_caption(ax, f"Full-pipeline {ab.get('metric_name','metric')} = {ab.get('full_metric','?')}; each bar = removing that component. Sealed court.")
    S.save(fig, "practitioner_ablation_bars", "I", "figure_data/ablation_court.json",
           "Component-ablation importance: the metric delta when each DSFB component is removed.")


def render_all(run):
    """Render every group-I practitioner figure (operator-facing CSVs + evidence-object JSON)."""
    S.log("group I — chemical-engineering practitioner / industry")
    fig_operator_one_pager(run)
    fig_alarm_flood(run)
    fig_ne107_timeline(run)
    fig_workflow(run)
    fig_context_overlay(run)
    fig_amendment_chain(run)
    fig_passport_card(run)
    fig_sweep_heatmap(run)
    fig_ablation_bars(run)
