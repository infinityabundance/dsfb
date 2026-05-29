"""Figure group B — the detector atlas, heuristics bank, and fault-signature catalogue (6 designs).

The authority surface made visible. The 57-detector / 11-family census comes from the committed atlas TOML
(executed + catalogued prior-art surface); the rich per-detector fields, the H1–H6 heuristics, and the
F1–F12 fault signatures come from the atlas-authority JSON dumped by `figure_export.rs`. These show the
*breadth* of the catalogued prior-art surface — central to the defensive-publication strategy.
"""
import json
import os
import textwrap

import numpy as np
import matplotlib.pyplot as plt

from . import style as S


def _atlas_json(run, name):
    """Load an exported atlas-authority JSON dump (returns [] if absent)."""
    p = os.path.join(run, "figure_data", name)
    return json.load(open(p)) if os.path.exists(p) else []


def _toml_family_census():
    """Parse chemometric_atlas.toml into {family: (executed, total)} (the 57-detector / 11-family surface)."""
    fam, exe = {}, {}
    cur = {}
    with open(S.ATLAS_TOML) as f:
        for line in f:
            line = line.strip()
            if line == "[[detector]]":
                if cur.get("primitive_family"):
                    pf = cur["primitive_family"]
                    fam[pf] = fam.get(pf, 0) + 1
                    if cur.get("implementation_status") == "Executed":
                        exe[pf] = exe.get(pf, 0) + 1
                cur = {}
            elif "=" in line:
                k, _, v = line.partition("=")
                cur[k.strip()] = v.strip().strip('"')
        if cur.get("primitive_family"):
            pf = cur["primitive_family"]
            fam[pf] = fam.get(pf, 0) + 1
            if cur.get("implementation_status") == "Executed":
                exe[pf] = exe.get(pf, 0) + 1
    return {k: (exe.get(k, 0), fam[k]) for k in fam}


def fig_census(run):
    """B11 — the authority 'at a glance': the headline counts of the catalogued prior-art surface.

    A clean infographic of the breadth: detectors (executed / catalogued), primitive families, fault
    signatures (executed / catalogued), heuristics, and unknown-taxonomy classes. This is the
    defensive-publication breadth in one panel.
    """
    census = _toml_family_census()
    total_det = sum(t for _, t in census.values())
    exec_det = sum(e for e, _ in census.values())
    n_fam = len(census)
    fs = _atlas_json(run, "atlas_fault_signatures.json")
    he = _atlas_json(run, "atlas_heuristics.json")
    ut = _atlas_json(run, "atlas_unknown_taxonomy.json")
    fs_exec = sum(1 for f in fs if f.get("implementation_status") == "Executed")
    cards = [
        (f"{total_det}", "chemometric\ndetectors", S.ACCENT),
        (f"{exec_det}", "executed\n(rest catalogued)", S.OK),
        (f"{n_fam}", "primitive\nfamilies", S.CB["skyblue"]),
        (f"{len(fs)}", "fault signatures\n(F1–F12)", S.CB["orange"]),
        (f"{fs_exec}", "fault sigs\nexecuted", S.OK),
        (f"{len(he)}", "process\nheuristics", S.CB["purple"]),
        (f"{len(ut)}", "unknown\nclasses", S.MUTE),
    ]
    fig, ax = plt.subplots(figsize=(8.0, 3.0))
    ax.axis("off")
    n = len(cards)
    for i, (big, lab, col) in enumerate(cards):
        x = (i + 0.5) / n
        ax.add_patch(plt.Rectangle((i / n + 0.01, 0.15), 1 / n - 0.02, 0.7, transform=ax.transAxes,
                                   fc=col, alpha=0.14, ec=col, lw=1.2))
        ax.text(x, 0.62, big, transform=ax.transAxes, ha="center", fontsize=22, weight="bold", color=col)
        ax.text(x, 0.30, lab, transform=ax.transAxes, ha="center", fontsize=8, color="#222")
    ax.set_title("The DSFB chemometric authority at a glance (catalogued prior-art surface)", fontsize=11)
    S.figure_caption(ax, "Counts from the committed atlas (chemometric_atlas.toml + atlas authority crate). Breadth is the prior-art strategy.")
    S.save(fig, "atlas_census", "B", "chemometric_atlas.toml + atlas authority JSON",
           "Headline authority counts: detectors, families, fault signatures, heuristics, unknown classes.")


def fig_sunburst(run):
    """B12 ★ — a two-ring sunburst of the detector atlas: primitive family (inner) → executed/catalogued (outer)."""
    census = _toml_family_census()
    fams = sorted(census, key=lambda k: -census[k][1])
    totals = [census[f][1] for f in fams]
    fig, ax = plt.subplots(figsize=(6.6, 6.2), subplot_kw=dict(aspect="equal"))
    # palette across families (colourblind-safe cycle).
    palette = [S.CB[k] for k in ("blue", "orange", "green", "skyblue", "vermillion", "purple", "yellow", "black", "grey")]
    fam_colors = [palette[i % len(palette)] for i in range(len(fams))]
    # Inner ring: families.
    ax.pie(totals, radius=1.0, colors=fam_colors, startangle=90, counterclock=False,
           wedgeprops=dict(width=0.42, edgecolor="white"),
           labels=[f"{f}\n({census[f][1]})" for f in fams], labeldistance=1.08, textprops={"fontsize": 7})
    # Outer ring: executed (dark) vs catalogued (light) within each family.
    outer_vals, outer_cols = [], []
    for f, c in zip(fams, fam_colors):
        e, t = census[f]
        outer_vals += [e, t - e]
        outer_cols += [c, "#e9edf1"]
    ax.pie(outer_vals, radius=1.0 - 0.42, colors=outer_cols, startangle=90, counterclock=False,
           wedgeprops=dict(width=0.30, edgecolor="white"))
    ax.set_title(f"Detector atlas: {sum(totals)} detectors across {len(fams)} families\n(inner=family, outer ring: solid=executed, pale=catalogued)", fontsize=10)
    S.save(fig, "atlas_sunburst", "B", "chemometric_atlas.toml",
           "Two-ring sunburst of the detector atlas: primitive family then executed-vs-catalogued split.")


def fig_detector_response(run):
    """B13 — the detector-response heatmap: signed margin per (detector, sample) for the representative dataset.

    A diverging heatmap (blue = below threshold, red = breach) over the executed detector bank shows, at a
    glance, when and which detectors cross their thresholds — the raw input to grammar + fusion.
    """
    rows = S.read_csv_rows(S.ds_csv(run, "tennessee_eastman_idv01", "detector_outputs.csv"))
    if not rows:
        S.log("  [B] SKIP atlas_detector_response: no detector_outputs.csv")
        return
    dets = sorted({r["detector_id"] for r in rows})
    times = sorted({int(r["time_index"]) for r in rows})
    tindex = {t: i for i, t in enumerate(times)}
    M = np.full((len(dets), len(times)), np.nan)
    didx = {d: i for i, d in enumerate(dets)}
    for r in rows:
        M[didx[r["detector_id"]], tindex[int(r["time_index"])]] = S.fnum(r["signed_margin"])
    # Per-detector normalisation: cumulative detectors (CUSUM, Page-Hinkley) reach O(1e4) and would wash out
    # the O(1) detectors on a shared scale. Scale each row by its own 98th-percentile |margin| (clip to ±1)
    # so every detector's breach *pattern* is equally legible. Sign preserved (breach = positive = red).
    for i in range(M.shape[0]):
        s = np.nanpercentile(np.abs(M[i]), 98)
        if s and np.isfinite(s):
            M[i] = np.clip(M[i] / s, -1.0, 1.0)
    fig, ax = plt.subplots(figsize=(8.0, 4.2))
    im = ax.imshow(M, aspect="auto", cmap="RdBu_r", vmin=-1, vmax=1,
                   extent=[times[0], times[-1], len(dets) - 0.5, -0.5])
    ax.set_yticks(range(len(dets))); ax.set_yticklabels(dets, fontsize=7)
    ax.set_xlabel("sample index")
    ax.set_title("Detector breach pattern (red = breach, blue = below threshold) — Tennessee Eastman IDV(1)")
    fig.colorbar(im, ax=ax, fraction=0.025, pad=0.02, label="per-detector normalised signed margin")
    S.figure_caption(ax, "Each detector row scaled by its own 98th-percentile |margin| (sign preserved) so every detector's breach pattern is visible.")
    S.save(fig, "atlas_detector_response", "B", "tennessee_eastman_idv01/detector_outputs.csv",
           "Per-detector breach pattern over time (row-normalised signed margin): when and which detectors breach.")


def fig_heuristic_flowchart(run):
    """B14 ★ — the H1–H6 heuristic bank as a decision flowchart (drift/slew/admissibility → candidate label)."""
    he = _atlas_json(run, "atlas_heuristics.json")
    if not he:
        S.log("  [B] SKIP atlas_heuristics: no heuristics JSON")
        return
    lines = ['digraph H {', '  rankdir=LR; bgcolor="white";',
             '  graph [fontname="serif", fontsize=12, labelloc="t", label="Chemical heuristics bank (H1–H6): residual conditions → candidate label"];',
             '  node [fontname="serif", fontsize=8, style="filled,rounded", shape=box];']
    for h in he:
        hid = h.get("heuristic_id", "")
        cond = " ∧ ".join(x for x in [h.get("drift_condition", ""), h.get("slew_condition", ""), h.get("admissibility_condition", "")]
                          if x and x != "not required")
        cond_w = "\\n".join(textwrap.wrap(cond, 52)) or "(structural)"
        lbl = "\\n".join(textwrap.wrap(h.get("episode_label", h.get("name", "")), 34))
        lines.append(f'  {hid}_c [label="{hid} conditions:\\n{cond_w}", fillcolor="#e7eef5"];')
        lines.append(f'  {hid}_l [label="candidate:\\n{lbl}", fillcolor="#f3e9d6", shape=note];')
        lines.append(f'  {hid}_c -> {hid}_l [color="#555"];')
    lines.append('}')
    S.render_dot("\n".join(lines), "atlas_heuristic_flowchart", "B", "figure_data/atlas_heuristics.json",
                 "The H1–H6 heuristic bank: each rule's drift/slew/admissibility conditions map to a candidate label.")


def fig_fault_signature_matrix(run):
    """B15 ★ — the F1–F12 fault-signature catalogue as a feature matrix (executed vs catalogued).

    Rows = fault mechanisms; columns = a few catalogued attributes (residual motif, #cheap sensors,
    #confusers, #exhibiting datasets); a left status stripe marks executed vs catalogued. Shows the breadth
    of the fault-signature prior-art surface.
    """
    fs = _atlas_json(run, "atlas_fault_signatures.json")
    if not fs:
        S.log("  [B] SKIP atlas_fault_signature_matrix: no fault signatures JSON")
        return
    rows = []
    for f in fs:
        rows.append([
            f.get("fault_id", ""),
            f.get("name", ""),
            (f.get("residual_motif", "") or "")[:22],
            str(len(f.get("cheap_sensors", []))),
            str(len(f.get("confuser_faults", []))),
            str(len(f.get("exhibiting_datasets", []))),
            "yes" if f.get("implementation_status") == "Executed" else "—",
        ])
    headers = ["id", "mechanism", "residual motif", "#sensors", "#confusers", "#datasets", "exec"]
    fig, ax = plt.subplots(figsize=(8.0, 0.34 * len(rows) + 1.0))
    ax.axis("off")
    tbl = ax.table(cellText=rows, colLabels=headers, loc="center", cellLoc="left")
    tbl.auto_set_font_size(False); tbl.set_fontsize(7.2); tbl.scale(1, 1.25)
    for j in range(len(headers)):
        tbl[0, j].set_facecolor("#e7eef5"); tbl[0, j].set_text_props(weight="bold")
    for i, f in enumerate(fs):
        col = "#d6ebe2" if f.get("implementation_status") == "Executed" else "#f4f6f8"
        tbl[i + 1, 0].set_facecolor(col)
        tbl[i + 1, 6].set_facecolor(col)
    ax.set_title(f"Fault-signature catalogue F1–F{len(fs)} (green = executed via a witness; rest catalogued)", fontsize=10.5)
    S.save(fig, "atlas_fault_signature_matrix", "B", "figure_data/atlas_fault_signatures.json",
           "The F1–F12 fault-signature catalogue as a feature matrix, executed vs catalogued.")


def fig_detector_input_matrix(run):
    """B16 — the detector × input-requirements matrix for the 18 executed detectors.

    Which inputs each executed detector needs (per-variable residuals, baseline window, scores, …). Makes
    the data dependencies of the executed bank explicit.
    """
    dets = _atlas_json(run, "atlas_detectors.json")
    if not dets:
        S.log("  [B] SKIP atlas_detector_input_matrix: no detectors JSON")
        return
    reqs = sorted(set().union(*[set(d.get("input_requirements", [])) for d in dets]))
    M = np.zeros((len(dets), len(reqs)))
    for i, d in enumerate(dets):
        for r in d.get("input_requirements", []):
            M[i, reqs.index(r)] = 1.0
    from matplotlib.colors import ListedColormap
    fig, ax = plt.subplots(figsize=(8.0, 4.4))
    ax.imshow(M, aspect="auto", cmap=ListedColormap(["#eef2f6", S.CB["skyblue"]]), vmin=0, vmax=1)
    ax.set_yticks(range(len(dets))); ax.set_yticklabels([d.get("detector_id", "") for d in dets], fontsize=7)
    ax.set_xticks(range(len(reqs)))
    ax.set_xticklabels([textwrap.fill(r, 14) for r in reqs], rotation=40, ha="right", fontsize=7)
    ax.set_title("Executed-detector input requirements (filled = required)", fontsize=10.5)
    S.save(fig, "atlas_detector_input_matrix", "B", "figure_data/atlas_detectors.json",
           "Input requirements per executed detector — the data dependencies of the executed bank.")


def render_all(run):
    """Render every group-B figure (atlas TOML census + atlas-authority JSON dumps + detector outputs)."""
    S.log("group B — detector atlas / heuristics / fault signatures")
    fig_census(run)
    fig_sunburst(run)
    fig_detector_response(run)
    fig_heuristic_flowchart(run)
    fig_fault_signature_matrix(run)
    fig_detector_input_matrix(run)
