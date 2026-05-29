#!/usr/bin/env python3
"""Render publication-quality figures for the paper from the committed numeric traces.

Inputs (read-only):
  * the latest output-dsfb-chemical-engineering/<stamp>/ run (or --run <dir>):
      figures/trace_data/trace_<dataset>.csv, grammar_<dataset>_<det>.csv, metrics.csv
  * crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml (dataset provenance)
  * crates/dsfb-chemical-engineering-edge/corpus/chemometric_atlas.toml (detector atlas)

Outputs: paper/figures/*.png and *.pdf (committed; the paper \\includegraphics{} these).

Figures are deterministic given the same run. No network, no randomness.
Run:  python3 scripts/gen_figures.py [--run <output_dir>]

Output files written to paper/figures/:
  residual_timeline_<dataset>.{png,pdf}  -- SPE/T² timeline + grammar strip (4 datasets)
  detection_delay.{png,pdf}              -- horizontal bar chart of detection delays
  episode_compression.{png,pdf}          -- compression ratio (log scale) across datasets
  unknown_coverage.{png,pdf}             -- labelled vs unknown episode stacked bar
  baseline_false_positive.{png,pdf}      -- baseline-window false-positive rate (%)
  atlas_families.{png,pdf}               -- detector count by primitive family

All figures use the serif font family, 10 pt, with light grid and no top/right spines,
matching the paper's LaTeX style.  The Agg backend is selected explicitly so the script
runs without a display server (headless CI, Colab, etc.).
"""
import csv
import glob
import os
import sys

import matplotlib

# Use the Agg (non-interactive raster) backend before importing pyplot so the
# script does not require a display server; safe on headless CI and Colab.
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

# `HERE` is the crate root (dsfb-chemical-engineering-edge); `WORKSPACE` is the repo root.
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORKSPACE = os.path.dirname(os.path.dirname(HERE))
# Figures land in paper/figures/ and are tracked by git; LaTeX \includegraphics{} references them.
FIGDIR = os.path.join(WORKSPACE, "paper", "figures")
os.makedirs(FIGDIR, exist_ok=True)

# Global matplotlib style: serif font to match LaTeX body text, subtle grid, clean spines.
plt.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.grid": True,
    "grid.alpha": 0.25,
    "axes.spines.top": False,
    "axes.spines.right": False,
    "figure.dpi": 140,
})

# Colour palette for the DSFB grammar state tokens, used in the residual timeline strip.
# Token meanings: NOM=nominal, DA=distributional anomaly, SS=structural shift,
# EV=extreme value, BG=background, RC=recovery, CP=changepoint, SF=sensor fault.
GRAMMAR_COLORS = {
    "NOM": "#dfe7ef", "DA": "#f4a259", "SS": "#e76f51", "EV": "#9b2226",
    "BG": "#a8c686", "RC": "#8ecae6", "CP": "#5a189a", "SF": "#000000",
}


def latest_run():
    """Return the path of the most-recently created edge demo output directory.

    Reads: output-dsfb-chemical-engineering/* (sorted lexicographically; timestamps in
    directory names make lexicographic order equivalent to chronological order).
    Exits with an error message if no output directory is found.
    """
    runs = sorted(glob.glob(os.path.join(WORKSPACE, "output-dsfb-chemical-engineering", "*")))
    if not runs:
        sys.exit("no output run found; run the edge demo first")
    return runs[-1]


def read_csv_rows(path):
    """Read a CSV file and return its rows as a list of dicts keyed by header names."""
    with open(path) as f:
        return list(csv.DictReader(f))


def save(fig, name):
    """Save a matplotlib figure to FIGDIR as both PNG (raster) and PDF (vector).

    bbox_inches="tight" trims excess whitespace so \\includegraphics{} frames the figure
    closely in the paper without manual width adjustments.  The figure is closed after
    saving to release memory.
    """
    for ext in ("png", "pdf"):
        fig.savefig(os.path.join(FIGDIR, f"{name}.{ext}"), bbox_inches="tight")
    plt.close(fig)
    print(f"  wrote {name}.png/.pdf")


def fig_residual_timeline(run, dataset, title):
    """Render a two-panel residual timeline figure for one dataset.

    Reads (from the run directory):
      figures/trace_data/trace_<dataset>.csv
        columns: time_index, spe, t2, in_episode, is_onset  (empty spe/t2 → NaN)
      figures/trace_data/grammar_<dataset>_pca_spe_q.csv  (optional)
        columns: time_index, state_token

    Top panel: SPE/Q (left y-axis, dark blue) and Hotelling T² (right y-axis, red) vs
    sample index.  Fused episode spans are shaded in orange.  The labelled fault onset
    (is_onset==1) is marked with a vertical dashed line.

    Bottom panel: grammar state strip for the `pca_spe_q` detector. Each sample is
    coloured by its grammar token from GRAMMAR_COLORS. If the grammar CSV is absent
    (e.g. detector not run for this dataset), the bottom panel is left empty.

    The figure is only generated if the trace CSV exists; missing datasets are silently
    skipped so a partial run does not abort figure generation.

    Writes: paper/figures/residual_timeline_<dataset>.{png,pdf}
    """
    tpath = os.path.join(run, "figures", "trace_data", f"trace_{dataset}.csv")
    if not os.path.exists(tpath):
        return
    rows = read_csv_rows(tpath)
    t = np.array([int(r["time_index"]) for r in rows])
    # Empty strings in spe/t2 columns indicate samples where the statistic was not computed.
    spe = np.array([float(r["spe"]) if r["spe"] else np.nan for r in rows])
    t2 = np.array([float(r["t2"]) if r["t2"] else np.nan for r in rows])
    inep = np.array([int(r["in_episode"]) for r in rows])
    onset = next((int(r["time_index"]) for r in rows if r["is_onset"] == "1"), None)

    fig, ax = plt.subplots(2, 1, figsize=(7.2, 4.0), sharex=True, gridspec_kw={"height_ratios": [2, 1]})
    ax[0].plot(t, spe, lw=0.9, color="#1d3557", label="SPE / Q (residual energy)")
    ax0b = ax[0].twinx()
    ax0b.plot(t, t2, lw=0.7, color="#e63946", alpha=0.7, label="Hotelling $T^2$")
    ax0b.set_ylabel("$T^2$", color="#e63946")
    ax[0].set_ylabel("SPE / Q", color="#1d3557")
    ax[0].set_title(title)
    # Shade fused episodes: scan for 0→1 and 1→0 transitions in `in_episode` to build spans.
    in_seg = False
    for i in range(len(t)):
        if inep[i] and not in_seg:
            seg_start = t[i]
            in_seg = True
        elif not inep[i] and in_seg:
            ax[0].axvspan(seg_start, t[i], color="#f4a259", alpha=0.18)
            in_seg = False
    if in_seg:
        # Episode extends to the end of the trace; close the span.
        ax[0].axvspan(seg_start, t[-1], color="#f4a259", alpha=0.18)
    if onset is not None:
        for a in (ax[0], ax[1]):
            a.axvline(onset, color="#2a9d8f", ls="--", lw=1.1)
        ax[0].text(onset, ax[0].get_ylim()[1] * 0.92, " labelled onset", color="#2a9d8f", fontsize=8)

    # Grammar strip for pca_spe_q: one coloured span per sample, 1-sample wide.
    gpath = os.path.join(run, "figures", "trace_data", f"grammar_{dataset}_pca_spe_q.csv")
    if os.path.exists(gpath):
        g = read_csv_rows(gpath)
        gi = np.array([int(r["time_index"]) for r in g])
        gs = [r["state_token"] for r in g]
        for i in range(len(gi)):
            ax[1].axvspan(gi[i] - 0.5, gi[i] + 0.5, color=GRAMMAR_COLORS.get(gs[i], "#cccccc"), lw=0)
        ax[1].set_yticks([])
        ax[1].set_ylabel("grammar\n(SPE/Q)", fontsize=8)
    ax[1].set_xlabel("sample index")
    # Legend for grammar colours placed below the strip to avoid covering the data.
    handles = [plt.Line2D([0], [0], color=c, lw=6) for c in GRAMMAR_COLORS.values()]
    ax[1].legend(handles, list(GRAMMAR_COLORS.keys()), ncol=8, fontsize=6.5, loc="upper center",
                 bbox_to_anchor=(0.5, -0.45), frameon=False)
    save(fig, f"residual_timeline_{dataset}")


def fig_detection_delay(metrics):
    """Render a horizontal bar chart of per-dataset detection delay.

    Reads: metrics.csv rows where `detection_delay` is not 'na' or empty.
    Unlabelled datasets (no ground-truth onset) are excluded from this figure.

    Colour encoding by delay magnitude (in samples relative to labelled onset):
      green  (#2a9d8f): ≤ 5 samples (fast detection)
      yellow (#e9c46a): 6-40 samples (moderate)
      red    (#e76f51): > 40 samples (slow detection)
    Negative delay values indicate detection at or just before the labelled onset.

    Writes: paper/figures/detection_delay.{png,pdf}
    """
    labeled = [m for m in metrics if m["detection_delay"] not in ("na", "")]
    labeled.sort(key=lambda m: m["dataset"])
    names = [m["dataset"].replace("tennessee_eastman_", "TEP-").replace("_", " ") for m in labeled]
    delay = [int(m["detection_delay"]) for m in labeled]
    fig, ax = plt.subplots(figsize=(7.2, 3.6))
    colors = ["#2a9d8f" if d <= 5 else ("#e9c46a" if d <= 40 else "#e76f51") for d in delay]
    ax.barh(names, delay, color=colors)
    ax.axvline(0, color="k", lw=0.8)
    ax.set_xlabel("detection delay (samples relative to labelled onset; negative = at/just before)")
    ax.set_title("DSFB fused-episode detection delay vs. labelled fault onset")
    save(fig, "detection_delay")


def fig_compression(metrics):
    """Render a horizontal bar chart of residual-structure compression ratios.

    Reads: metrics.csv `episode_compression_ratio` column (raw detector breaches / fused episodes).
    Bars are sorted by ascending compression ratio. Log scale is used because the ratio spans
    multiple orders of magnitude across datasets. Datasets with many short breaches collapsing
    into few fused episodes show the highest ratios.

    Writes: paper/figures/episode_compression.{png,pdf}
    """
    metrics = sorted(metrics, key=lambda m: float(m["episode_compression_ratio"]))
    names = [m["dataset"].replace("tennessee_eastman_", "TEP-").replace("_", " ") for m in metrics]
    comp = [float(m["episode_compression_ratio"]) for m in metrics]
    fig, ax = plt.subplots(figsize=(7.2, 5.0))
    ax.barh(names, comp, color="#457b9d")
    ax.set_xscale("log")
    ax.set_xlabel("episode compression ratio (raw detector breaches / fused episodes, log scale)")
    ax.set_title("Residual-structure compression across datasets")
    save(fig, "episode_compression")


def fig_unknown_coverage(metrics):
    """Render a stacked horizontal bar chart of labelled vs unknown fused episodes.

    Reads: metrics.csv `labeled_episodes` and `unknown_episodes` columns.
    The two segments per bar show:
      green (#2a9d8f): episodes that received a heuristic structural label
      grey  (#adb5bd): episodes left as 'unknown' (evidence preserved, diagnosis deferred)
    This figure documents the pipeline's honest unknown rate — the DSFB framework deliberately
    preserves evidence rather than forcing a diagnosis when structural evidence is ambiguous.

    Writes: paper/figures/unknown_coverage.{png,pdf}
    """
    metrics = sorted(metrics, key=lambda m: m["dataset"])
    names = [m["dataset"].replace("tennessee_eastman_", "TEP-").replace("_", " ") for m in metrics]
    labeled = [int(m["labeled_episodes"]) for m in metrics]
    unknown = [int(m["unknown_episodes"]) for m in metrics]
    fig, ax = plt.subplots(figsize=(7.2, 5.0))
    ax.barh(names, labeled, color="#2a9d8f", label="heuristic-labelled")
    ax.barh(names, unknown, left=labeled, color="#adb5bd", label="unknown (evidence preserved)")
    ax.set_xlabel("fused episodes")
    ax.set_title("Explanation coverage vs. honest unknown rate")
    ax.legend(loc="lower right", frameon=False)
    save(fig, "unknown_coverage")


def fig_baseline_fp(metrics):
    """Render a horizontal bar chart of baseline-window false-positive rates.

    Reads: metrics.csv `baseline_false_positive_rate` column (stored as a fraction 0.0–1.0;
    multiplied by 100 for display as a percentage).

    Colour encoding by FP rate:
      green  (#2a9d8f): < 10%  (low; typical for stationary process data)
      yellow (#e9c46a): 10-35% (moderate; often batch/phase-varying data)
      red    (#e76f51): ≥ 35%  (high; expected for non-stationary or heterogeneous tabular sets)

    The title text honestly notes that higher FP rates are expected and not suppressed for
    non-stationary datasets (batch processes, heterogeneous tabular, drift benchmarks).

    Writes: paper/figures/baseline_false_positive.{png,pdf}
    """
    metrics = sorted(metrics, key=lambda m: float(m["baseline_false_positive_rate"]))
    names = [m["dataset"].replace("tennessee_eastman_", "TEP-").replace("_", " ") for m in metrics]
    fp = [float(m["baseline_false_positive_rate"]) * 100 for m in metrics]
    fig, ax = plt.subplots(figsize=(7.2, 5.0))
    colors = ["#2a9d8f" if v < 10 else ("#e9c46a" if v < 35 else "#e76f51") for v in fp]
    ax.barh(names, fp, color=colors)
    ax.set_xlabel("baseline-window false-positive rate (%)")
    ax.set_title("Baseline coverage: low for stationary process data; higher for\nnon-stationary batch / heterogeneous / tabular sets (reported honestly)")
    save(fig, "baseline_false_positive")


def fig_atlas_families():
    """Parse chemometric_atlas.toml and render a stacked bar chart by primitive family.

    Reads: corpus/chemometric_atlas.toml
    The file begins with a schema header (schema_version, corpus_name, detector_count) that
    does NOT have a `primitive_family` field.  The parser accumulates key=value pairs into
    `cur` between [[detector]] markers; a block is only counted when it has a `primitive_family`
    key — this correctly skips the file-header block without miscounting.

    The corpus is 57 detectors across 11 primitive families.  Each detector is classified as
    either "Executed" (run by the edge demo pipeline) or "Catalogued" (defined prior-art surface,
    not run in the default demo).  The stacked bar shows both segments per family.

    Parser assumption: key=value lines use `=` as separator; string values may or may not be
    quoted.  The strip().strip('"') pattern handles both quoted and unquoted values.  This is
    a lightweight custom reader — no toml library dependency.

    The flush-end block (after the loop) handles the last [[detector]] block in the file,
    which has no trailing [[detector]] marker to trigger the flush inside the loop.

    Writes: paper/figures/atlas_families.{png,pdf}
    """
    path = os.path.join(HERE, "corpus", "chemometric_atlas.toml")
    fam, exe = {}, {}
    cur = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line == "[[detector]]":
                if cur.get("primitive_family"):  # skip the file-header block (schema_version, etc.)
                    pf = cur["primitive_family"]
                    fam[pf] = fam.get(pf, 0) + 1
                    if cur.get("implementation_status") == "Executed":
                        exe[pf] = exe.get(pf, 0) + 1
                cur = {}
            elif "=" in line:
                k, _, v = line.partition("=")
                cur[k.strip()] = v.strip().strip('"')
        # Flush the final block after end-of-file (no trailing [[detector]] marker).
        if cur.get("primitive_family"):
            pf = cur["primitive_family"]
            fam[pf] = fam.get(pf, 0) + 1
            if cur.get("implementation_status") == "Executed":
                exe[pf] = exe.get(pf, 0) + 1
    # Sort families by descending total count for visual clarity.
    order = sorted(fam, key=lambda k: -fam[k])
    total = [fam[k] for k in order]
    executed = [exe.get(k, 0) for k in order]
    catalog = [t - e for t, e in zip(total, executed)]
    fig, ax = plt.subplots(figsize=(7.2, 4.2))
    ax.bar(order, executed, color="#2a9d8f", label="executed")
    ax.bar(order, catalog, bottom=executed, color="#adb5bd", label="catalogued (prior-art surface)")
    ax.set_ylabel("detectors")
    ax.set_title(f"Chemometric detector atlas: {sum(total)} detectors across {len(order)} primitive families")
    ax.set_xticklabels(order, rotation=40, ha="right", fontsize=8)
    ax.legend(frameon=False)
    save(fig, "atlas_families")


def main():
    """Entry point: locate the run directory, load metrics.csv, and render all figures.

    --run <dir>  overrides the automatic latest-run selection.

    Four representative residual timelines are rendered, chosen to cover different dataset
    characteristics:
      tennessee_eastman_idv01  -- canonical multivariate simulator fault (step disturbance)
      cstr_reactor             -- clean simulation with a clear thermal-excursion signature
      gas_sensor_array_drift   -- long-horizon real sensor drift (no sharp onset)
      penicillin_fedbatch      -- batch process with phase structure complicating baseline

    Then five cross-dataset summary figures are rendered from metrics.csv.
    The atlas families figure is rendered from the committed corpus TOML (no run dependency).
    """
    run = latest_run()
    if "--run" in sys.argv:
        run = sys.argv[sys.argv.index("--run") + 1]
    print(f"rendering figures from {run} -> {FIGDIR}")
    metrics = read_csv_rows(os.path.join(run, "metrics.csv"))
    # Representative residual timelines: canonical real fault, clean sim, hard real drift.
    fig_residual_timeline(run, "tennessee_eastman_idv01", "Tennessee Eastman IDV(1): step disturbance (onset @ 160)")
    fig_residual_timeline(run, "cstr_reactor", "CSTR thermal-excursion fault (cooling fouling, onset @ 480)")
    fig_residual_timeline(run, "gas_sensor_array_drift", "Gas sensor array: long-horizon sensor drift (batch 1 -> late batches)")
    fig_residual_timeline(run, "penicillin_fedbatch", "Fed-batch penicillin: aeration fault across batch phases")
    fig_detection_delay(metrics)
    fig_compression(metrics)
    fig_unknown_coverage(metrics)
    fig_baseline_fp(metrics)
    fig_atlas_families()
    print("done.")


if __name__ == "__main__":
    main()
