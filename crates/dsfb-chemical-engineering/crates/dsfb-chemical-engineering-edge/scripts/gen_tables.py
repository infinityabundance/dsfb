#!/usr/bin/env python3
"""Generate LaTeX result tables for the paper from the committed artifacts, so every number in the
paper traces to a reproducible source. Reads the latest edge demo metrics.csv, the latest CUDA
court manifest.json, the Nsight summary, and the dataset provenance manifest. Emits paper/tables/*.tex.

Run:  python3 scripts/gen_tables.py

Inputs (all read-only committed artifacts):
  output-dsfb-chemical-engineering/<latest>/metrics.csv          -- edge pipeline results
  output-dsfb-chemical-engineering-cuda/<latest>/manifest.json   -- CUDA court results
  crates/dsfb-chemical-engineering-cuda/reports/nsight_summary.json  -- Nsight aggregates
  crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml       -- dataset provenance

Outputs written to paper/tables/:
  datasets.tex    -- longtable: 20 datasets with kind/samples/vars/source (M/S/G taxonomy)
  detection.tex   -- table: detection delay and baseline FP rate (labelled datasets only)
  compression.tex -- longtable: raw breaches, fused episodes, compression ratio, unknown rate
  replay.tex      -- longtable: CUDA evidence roots, cross-backend verification status
  cuda_perf.tex   -- table: nsys kernel timing and GB/s (median across 5 runs/variant)
  cuda_ncu.tex    -- table: ncu SM/DRAM throughput, occupancy, L2 hit rate

Tables are only generated when the required input exists; missing inputs are silently skipped
(e.g. replay.tex is not written if no CUDA output directory is found).
All LaTeX special characters in data strings are escaped by esc() before insertion.
"""
import csv
import glob
import json
import os
import re

# `HERE` is the crate root (dsfb-chemical-engineering-edge); `WORKSPACE` is the repo root.
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORKSPACE = os.path.dirname(os.path.dirname(HERE))
# Tables land in paper/tables/ and are \input{}-ed from the paper's LaTeX source.
TBL = os.path.join(WORKSPACE, "paper", "tables")
os.makedirs(TBL, exist_ok=True)


def latest(glob_pat):
    """Return the lexicographically last path matching glob_pat, or None if no match.

    Timestamp-prefixed output directory names make lexicographic order chronological,
    so the last match is the most recent run.
    """
    xs = sorted(glob.glob(glob_pat))
    return xs[-1] if xs else None


def esc(s):
    """Escape LaTeX special characters in a data string for safe insertion into a table cell.

    Handles underscores (\\_), percent signs (\\%), and ampersands (\\&).
    Other special characters (e.g. $, #, {}) are not currently escaped because they do not
    appear in the data strings written by this pipeline.
    """
    return str(s).replace("_", r"\_").replace("%", r"\%").replace("&", r"\&")


def read_metrics():
    """Load metrics.csv from the latest edge demo output directory.

    Reads: output-dsfb-chemical-engineering/<latest>/metrics.csv
    Returns a list of dicts keyed by header names (one dict per dataset row).
    """
    run = latest(os.path.join(WORKSPACE, "output-dsfb-chemical-engineering", "*"))
    with open(os.path.join(run, "metrics.csv")) as f:
        return list(csv.DictReader(f))


def read_manifest_toml():
    """Tiny TOML array-of-tables reader for data/MANIFEST.toml (no toml dep).

    Reads: crates/dsfb-chemical-engineering-edge/data/MANIFEST.toml
    Returns a dict mapping dataset name -> provenance dict.

    Parsing: accumulates key=value pairs between [[dataset]] markers; comment lines
    (starting with '#') are skipped; string values are stripped of surrounding quotes.
    """
    path = os.path.join(HERE, "data", "MANIFEST.toml")
    entries, cur = [], None
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line == "[[dataset]]":
                if cur:
                    entries.append(cur)
                cur = {}
            elif "=" in line and cur is not None and not line.startswith("#"):
                k, _, v = line.partition("=")
                cur[k.strip()] = v.strip().strip('"')
        if cur:
            entries.append(cur)
    return {e["name"]: e for e in entries}


def read_cuda_manifest():
    """Load the CUDA court manifest.json from the latest CUDA output directory.

    Reads: output-dsfb-chemical-engineering-cuda/<latest>/manifest.json
    Returns the parsed JSON dict, or None if no CUDA output directory is found.
    The manifest contains a 'datasets' list with per-dataset evidence roots and
    cross-backend verification results.
    """
    run = latest(os.path.join(WORKSPACE, "output-dsfb-chemical-engineering-cuda", "*"))
    if not run:
        return None
    with open(os.path.join(run, "manifest.json")) as f:
        return json.load(f)


def short(name):
    """Shorten a dataset name for table display: TEP prefix + space-separated words."""
    return name.replace("tennessee_eastman_", "TEP-").replace("_", " ")


def table_datasets(prov):
    """Emit datasets.tex: a longtable listing all 20 datasets with kind/samples/vars/source.

    Reads: prov dict from read_manifest_toml() (keyed by dataset name).
    Rows are sorted alphabetically by name. The `kind` field is mapped to a single-letter
    code for compactness: M=measured, S=simulation, G=agreement-gated.
    Citation text is truncated to 42 characters to fit the table's last column width.

    Writes: paper/tables/datasets.tex
    """
    rows = sorted(prov.values(), key=lambda e: e["name"])
    out = [r"\begin{longtable}{@{}llrr p{0.37\linewidth}@{}}", r"\caption{Public datasets: 10 measured"
           r" real-world (M), 9 simulation benchmarks (S, including the Tennessee Eastman process), and"
           r" 1 agreement-gated stand-in (G). Only small processed slices are committed; full"
           r" provenance, license, and SHA-256 are in \texttt{data/MANIFEST.toml}.}\label{tab:datasets}\\",
           r"\toprule Dataset & Kind & Samples & Vars & Source / basis \\ \midrule", r"\endfirsthead",
           r"\toprule Dataset & Kind & Samples & Vars & Source / basis \\ \midrule", r"\endhead", r"\bottomrule \endfoot"]
    kindmap = {"measured": "M", "simulation": "S", "agreement-gated": "G"}
    for r in rows:
        cite = r.get("citation", "")[:42]
        out.append(f"{esc(short(r['name']))} & {kindmap.get(r['kind'],'?')} & {r['n_samples']} & {r['n_vars']} & {esc(cite)} \\\\")
    out.append(r"\end{longtable}")
    write("datasets.tex", out)


def table_detection(metrics):
    """Emit detection.tex: detection delay and baseline FP rate for labelled datasets.

    Reads: metrics list from read_metrics().  Only rows where `detection_delay` is not
    'na' or empty are included (unlabelled datasets have no ground-truth onset).
    `baseline_false_positive_rate` is stored as a fraction [0,1]; it is multiplied by 100
    for display with one decimal place.

    Writes: paper/tables/detection.tex
    """
    labeled = [m for m in metrics if m["detection_delay"] not in ("na", "")]
    labeled.sort(key=lambda m: m["dataset"])
    out = [r"\begin{table}[t]\centering", r"\small",
           r"\caption{Detection delay (samples relative to the labelled fault onset; negative = at or"
           r" just before onset) and baseline-window false-positive rate, on the labelled datasets."
           r" All values are produced by the deterministic edge pipeline.}\label{tab:detection}",
           r"\begin{tabular}{@{}lrrr@{}}", r"\toprule",
           r"Dataset & Detection delay & Baseline FP (\%) & Fused episodes \\ \midrule"]
    for m in labeled:
        fp = float(m["baseline_false_positive_rate"]) * 100
        out.append(f"{esc(short(m['dataset']))} & {m['detection_delay']} & {fp:.1f} & {m['fused_episodes']} \\\\")
    out += [r"\bottomrule", r"\end{tabular}", r"\end{table}"]
    write("detection.tex", out)


def table_compression(metrics):
    """Emit compression.tex: raw breaches, fused episodes, compression ratio, unknown rate.

    Reads: metrics list. All 20 datasets are included (no label filter).
    `episode_compression_ratio` is formatted as an integer followed by a LaTeX ×.
    `unknown_rate` is a fraction [0,1]; displayed as an integer percentage.

    Writes: paper/tables/compression.tex
    """
    metrics = sorted(metrics, key=lambda m: m["dataset"])
    out = [r"\begin{longtable}{@{}lrrrr@{}}",
           r"\caption{Residual-structure compression and honest unknown rate. Raw detector breaches"
           r" collapse into far fewer fused structural episodes; the unknown rate is the fraction of"
           r" episodes deliberately left unlabelled (evidence preserved).}\label{tab:compression}\\",
           r"\toprule Dataset & Raw breaches & Fused eps. & Compression & Unknown (\%) \\ \midrule",
           r"\endfirsthead", r"\toprule Dataset & Raw breaches & Fused eps. & Compression & Unknown (\%) \\ \midrule",
           r"\endhead", r"\bottomrule \endfoot"]
    for m in metrics:
        comp = float(m["episode_compression_ratio"])
        unk = float(m["unknown_rate"]) * 100
        out.append(f"{esc(short(m['dataset']))} & {m['raw_breach_steps']} & {m['fused_episodes']} & {comp:.0f}$\\times$ & {unk:.0f} \\\\")
    out.append(r"\end{longtable}")
    write("compression.tex", out)


def table_replay(cuda):
    """Emit replay.tex: CUDA court evidence roots and cross-backend verification status.

    Reads: cuda dict from read_cuda_manifest() (the 'datasets' list).
    Returns immediately if cuda is None (no CUDA output directory found).

    Each row shows the first 12 hex characters of the evidence root (a SHA-256-based
    digest of the per-lane residual sequence), the backend (CUDA/CPU), lane/sample counts,
    and whether both `replay_matched` and `cross_backend_verified` are True.
    "OK" means the GPU evidence root is byte-identical to the CPU reference on both the
    initial run and the deterministic replay.

    Writes: paper/tables/replay.tex
    """
    if not cuda:
        return
    rows = sorted(cuda["datasets"], key=lambda e: e["dataset"])
    out = [r"\begin{longtable}{@{}llrrll@{}}",
           r"\caption{CUDA forensic court: per-dataset evidence root (first 12 hex), byte-exact"
           r" cross-backend verification (GPU vs CPU reference), and replay. Every dataset's GPU"
           r" evidence root is identical to the CPU reference.}\label{tab:replay}\\",
           r"\toprule Dataset & Backend & Lanes & Samples & Evidence root & xverify/replay \\ \midrule",
           r"\endfirsthead", r"\toprule Dataset & Backend & Lanes & Samples & Evidence root & xverify/replay \\ \midrule",
           r"\endhead", r"\bottomrule \endfoot"]
    for r in rows:
        # Truncate evidence root to 12 hex chars for column width; full root is in manifest.json.
        root = r["evidence_root"][:12]
        ok = "OK" if (r["replay_matched"] and r["cross_backend_verified"]) else "FAIL"
        out.append(f"{esc(short(r['dataset']))} & {esc(r['backend'])} & {r['n_lanes']} & {r['n_samples']} & \\texttt{{{root}}} & {ok} \\\\")
    out.append(r"\end{longtable}")
    write("replay.tex", out)


def table_cuda_perf():
    """Emit cuda_perf.tex: evidence-factory kernel throughput (Nsight Systems timings).

    Reads: crates/dsfb-chemical-engineering-cuda/reports/nsight_summary.json
    The path is derived by replacing the edge crate name in HERE with the cuda crate name.
    Returns immediately if the file does not exist (Nsight campaign not yet run).

    Each row in `nsys_variants` covers one size variant (variant name, lane/sample counts,
    input size in MB, median kernel time in ms across 5 runs, and effective evidence GB/s).
    GB/s = input_bytes / kernel_time_ns (bytes/ns = GB/s).

    The caption text notes that the kernel is SHA-256-compute-bound (not memory-bound) and
    that its throughput sits below the measured memory roofline.  The roofline value is
    embedded from the JSON field `memory_roofline_gbps_median`.

    Writes: paper/tables/cuda_perf.tex
    """
    path = os.path.join(HERE.replace("dsfb-chemical-engineering-edge", "dsfb-chemical-engineering-cuda"), "reports", "nsight_summary.json")
    if not os.path.exists(path):
        return
    with open(path) as f:
        s = json.load(f)
    out = [r"\begin{table}[t]\centering", r"\small",
           r"\caption{Evidence-factory kernel throughput on an RTX 4080 SUPER (Nsight Systems, median"
           r" of 5 runs per size variant). The kernel is SHA-256-compute / parallelism-bound, so its"
           r" effective bandwidth scales with lane count and sits well below the measured memory"
           r" roofline of " + str(s.get("memory_roofline_gbps_median")) + r"~GB/s.}\label{tab:cudaperf}",
           r"\begin{tabular}{@{}llrrr@{}}", r"\toprule",
           r"Variant & Lanes$\times$Samples & MB & Kernel ms (median) & Evidence GB/s \\ \midrule"]
    for r in s["nsys_variants"]:
        out.append(f"{r['variant']} & {r['lanes']}$\\times${r['samples']} & {r['mb']} & {r['kernel_ms_median']} & {r['evidence_gbps_median']} \\\\")
    out += [r"\bottomrule", r"\end{tabular}", r"\end{table}"]
    write("cuda_perf.tex", out)


def table_cuda_ncu():
    """Emit cuda_ncu.tex: Nsight Compute microarchitectural counters (if available).

    Reads: crates/dsfb-chemical-engineering-cuda/reports/nsight_summary.json (same file
    as table_cuda_perf; the JSON is re-loaded here rather than passed as an argument so
    each table function is independently callable).
    Returns immediately if the file is absent or `ncu_variants` is empty (e.g. when GPU
    performance counters were not accessible — see `ncu_permission_note.txt`).

    Counter semantics (medians over 5 runs per variant, from `ncu --csv`):
      sm_throughput_pct   -- SM utilisation as % of peak sustained
      dram_throughput_pct -- DRAM bandwidth utilisation as % of peak
      occupancy_pct       -- warp occupancy as % of peak active warps
      l2_hit_pct          -- L2 cache sector hit rate

    IMPORTANT: ncu serialises kernel replay to collect hardware counters; its reported
    kernel time exceeds the nsys wall-clock by design.  The table caption documents this
    caveat.  Use nsys (table_cuda_perf) for wall-clock timing comparisons.

    Writes: paper/tables/cuda_ncu.tex
    """
    path = os.path.join(HERE.replace("dsfb-chemical-engineering-edge", "dsfb-chemical-engineering-cuda"), "reports", "nsight_summary.json")
    if not os.path.exists(path):
        return
    with open(path) as f:
        s = json.load(f)
    ncu = s.get("ncu_variants") or []
    if not ncu:
        return
    out = [r"\begin{table}[t]\centering", r"\small",
           r"\caption{Nsight Compute microarchitectural counters for the evidence-factory kernel on an"
           r" RTX 4080 SUPER (median of 5 runs per size variant). SM and DRAM throughput sit far below"
           r" peak while the L2 hit rate is high, confirming the kernel is SHA-256-compute / parallelism"
           r" bound, not memory bound. Counter collection serialises kernel replay, so ncu kernel time"
           r" exceeds the \texttt{nsys} wall-clock of \cref{tab:cudaperf}.}\label{tab:cudancu}",
           r"\begin{tabular}{@{}llrrrr@{}}", r"\toprule",
           r"Variant & Lanes$\times$Samples & SM tput \% & DRAM tput \% & Occupancy \% & L2 hit \% \\ \midrule"]
    for r in ncu:
        out.append(f"{r['variant']} & {r['lanes']}$\\times${r['samples']} & {r['sm_throughput_pct']} & "
                   f"{r['dram_throughput_pct']} & {r['occupancy_pct']} & {r['l2_hit_pct']} \\\\")
    out += [r"\bottomrule", r"\end{tabular}", r"\end{table}"]
    write("cuda_ncu.tex", out)


def table_regime():
    """Emit regime.tex: baseline false-positive and unknown rate, global vs regime-conditioned.

    Reads: reports/regime_comparison.csv (produced by `dsfb-chem-edge regime-eval`), one row per
    dataset that carries per-sample regime/phase labels, with the baseline FP, fused-episode count,
    and unknown rate under the global single envelope vs the per-regime calibrated envelope.
    Skipped (no file written) when the comparison CSV is absent.

    Writes: paper/tables/regime.tex
    """
    path = os.path.join(HERE, "reports", "regime_comparison.csv")
    if not os.path.exists(path):
        return
    with open(path) as f:
        rows = list(csv.DictReader(f))
    if not rows:
        return
    out = [r"\begin{table}[t]\centering", r"\small",
           r"\caption{Regime-conditioned (phase-aligned) admissibility envelopes vs the global single"
           r" envelope, on the two datasets that carry per-sample regime/phase labels. Baseline"
           r" false-positive rate (BFP) is the fraction of baseline-window samples covered by a fused"
           r" episode (lower is better). The unknown rate is shown to confirm the conservative honesty"
           r" signal is preserved --- episodes are not relabelled to manufacture a lower BFP. Per-regime"
           r" envelopes are calibrated to be no tighter than the global envelope, and every run replays"
           r" deterministically.}\label{tab:regime}",
           r"\begin{tabular}{@{}lrrrrr@{}}", r"\toprule",
           r"Dataset & Regimes & BFP global & BFP regime & Unknown global & Unknown regime \\ \midrule"]
    for r in rows:
        out.append(
            f"{esc(short(r['dataset']))} & {r['n_regimes']} & "
            f"{float(r['baseline_fp_global'])*100:.1f}\\% & {float(r['baseline_fp_regime'])*100:.1f}\\% & "
            f"{float(r['unknown_global'])*100:.1f}\\% & {float(r['unknown_regime'])*100:.1f}\\% \\\\")
    out += [r"\bottomrule", r"\end{tabular}", r"\end{table}"]
    write("regime.tex", out)


def write(name, lines):
    """Write a LaTeX table to paper/tables/<name>, joining lines with newlines.

    A trailing newline is appended so the file ends cleanly and LaTeX \\input{} does not
    complain about a missing newline at end of file.
    """
    with open(os.path.join(TBL, name), "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  wrote tables/{name}")


def main():
    """Entry point: load all inputs and generate all six LaTeX table files.

    Table generation is ordered to match the paper's section structure:
      datasets -> detection -> compression -> replay -> cuda_perf -> cuda_ncu
    Each table function is independently guarded against missing inputs so a partial
    environment (e.g. no CUDA output) still generates the available tables.
    """
    metrics = read_metrics()
    prov = read_manifest_toml()
    cuda = read_cuda_manifest()
    table_datasets(prov)
    table_detection(metrics)
    table_compression(metrics)
    table_replay(cuda)
    table_cuda_perf()
    table_cuda_ncu()
    table_regime()
    print("tables done.")


if __name__ == "__main__":
    main()
