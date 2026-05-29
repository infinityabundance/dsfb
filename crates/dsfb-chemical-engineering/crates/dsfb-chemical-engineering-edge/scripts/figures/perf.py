"""Figure group C — performance & the CUDA evidence court, from measured data (9 designs).

These figures turn the committed Nsight/bench measurements into pictures (panel recommendation R4): the
memory roofline showing the kernel is SHA-256 *compute*-bound (not bandwidth-bound), the Nsight Compute
counter heatmap, throughput bars, the V1→V2 optimization deltas, and the conceptual CUDA pipeline / Merkle
/ digest-equivalence-law diagrams (graphviz). All numbers are read from
`crates/dsfb-chemical-engineering-cuda/reports/` (nsight_summary.json, bench_*.json) or, where only the
design doc records them, transcribed with an explicit measured-on-hardware provenance note. No GPU is
needed to *render* these — only to have *measured* them (done on an RTX 4080 SUPER, CUDA 13.2).
"""
import glob
import json
import os

import numpy as np
import matplotlib.pyplot as plt

from . import style as S


def _nsight():
    """Load the Nsight summary JSON (nsys variants, ncu counters, memory roofline)."""
    return json.load(open(os.path.join(S.CUDA_REPORTS, "nsight_summary.json")))


def _bench():
    """Load all bench_*.json throughput summaries (one per benchmark run)."""
    out = []
    for p in sorted(glob.glob(os.path.join(S.CUDA_REPORTS, "bench_*.json"))):
        out.append(json.load(open(p)))
    return out


def fig_roofline(run=None):
    """C17 ★ — the memory roofline: measured evidence throughput vs the ~636.8 GB/s memory ceiling.

    The point: SHA-256 sealing is *compute*-bound. Measured evidence throughput (2.4–9.5 GB/s) is a tiny
    fraction of the device memory roofline, so the kernel is hashing-bound, not bandwidth-bound — which is
    why the V2 work targets occupancy, not memory traffic.
    """
    d = _nsight()
    roof = d["memory_roofline_gbps_median"]
    vs = d["nsys_variants"]
    labels = [f"{v['lanes']}×{v['samples']}" for v in vs]
    gbps = [v["evidence_gbps_median"] for v in vs]
    fig, ax = plt.subplots(figsize=(7.2, 3.8))
    y = np.arange(len(labels))
    ax.barh(y, gbps, color=S.ACCENT, height=0.5, zorder=3)
    ax.axvline(roof, color=S.WARN, lw=2.0, zorder=2)
    # Label the roofline vertically along the line so it never collides with the title.
    ax.text(roof, (len(labels) - 1) / 2.0, f"memory roofline ≈ {roof:g} GB/s", color=S.WARN,
            fontsize=8, va="center", ha="right", rotation=90, backgroundcolor="white")
    for yi, g in zip(y, gbps):
        ax.text(g * 1.1, yi, f"{g:g} GB/s ({100 * g / roof:.2f}% of roofline)", va="center", fontsize=7.5)
    ax.set_yticks(y); ax.set_yticklabels(labels)
    ax.set_xscale("log")
    ax.set_xlabel("evidence throughput (GB/s, log scale)")
    ax.set_ylabel("workload (lanes × samples)")
    ax.set_title("Evidence sealing is SHA-256 compute-bound, not memory-bound")
    S.figure_caption(ax, S.DISCLAIMER["measured"])
    S.save(fig, "perf_roofline", "C", "cuda/reports/nsight_summary.json",
           "Measured evidence throughput is a small fraction of the memory roofline — the kernel is hashing-bound.")


def fig_ncu_heatmap(run=None):
    """C18 ★ — Nsight Compute counter heatmap: SM%, DRAM%, occupancy%, L2-hit% across the three workloads."""
    d = _nsight()
    vs = d["ncu_variants"]
    cols = [f"{v['lanes']}×{v['samples']}" for v in vs]
    metrics = [("SM throughput %", "sm_throughput_pct"), ("DRAM throughput %", "dram_throughput_pct"),
               ("occupancy %", "occupancy_pct"), ("L2 hit %", "l2_hit_pct")]
    M = np.array([[v[k] for v in vs] for _, k in metrics])
    fig, ax = plt.subplots(figsize=(6.6, 3.8))
    im = ax.imshow(M, aspect="auto", cmap="cividis")
    ax.set_xticks(range(len(cols))); ax.set_xticklabels(cols)
    ax.set_yticks(range(len(metrics))); ax.set_yticklabels([m for m, _ in metrics])
    for i in range(M.shape[0]):
        for j in range(M.shape[1]):
            ax.text(j, i, f"{M[i, j]:.1f}", ha="center", va="center",
                    color=("white" if M[i, j] < M.max() * 0.55 else "black"), fontsize=8.5)
    ax.set_title("Nsight Compute counters (V1 evidence kernel)")
    fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04, label="% of peak")
    S.figure_caption(ax, "Low SM/DRAM with ~97% L2 hit and 8.3% occupancy → compute-bound, occupancy-limited. " + S.DISCLAIMER["measured"])
    S.save(fig, "perf_ncu_heatmap", "C", "cuda/reports/nsight_summary.json (ncu_variants)",
           "Nsight Compute counters across the three workloads: compute-bound, occupancy-limited V1 kernel.")


def fig_throughput_bars(run=None):
    """C19 — measured evidence throughput per workload with the min–max spread over the 5 benchmark runs."""
    bench = _bench()
    # Aggregate per (lanes,samples): collect medians/min/max across the bench files.
    agg = {}
    for b in bench:
        for s in b.get("summaries", []):
            key = (s["n_lanes"], s["n_samples"])
            agg.setdefault(key, []).append(s)
    keys = sorted(agg)
    labels = [f"{l}×{n}" for (l, n) in keys]
    med = [np.median([s["gbps_median"] for s in agg[k]]) for k in keys]
    lo = [min(s["gbps_min"] for s in agg[k]) for k in keys]
    hi = [max(s["gbps_max"] for s in agg[k]) for k in keys]
    fig, ax = plt.subplots(figsize=(7.2, 3.6))
    x = np.arange(len(keys))
    err = [np.array(med) - np.array(lo), np.array(hi) - np.array(med)]
    ax.bar(x, med, yerr=err, color=S.ACCENT, capsize=4, width=0.55)
    for xi, m in zip(x, med):
        ax.text(xi, m, f"{m:.1f}", ha="center", va="bottom", fontsize=8)
    ax.set_xticks(x); ax.set_xticklabels(labels)
    ax.set_ylabel("evidence GB/s")
    ax.set_xlabel("workload (lanes × samples)")
    ax.set_title("Evidence-factory throughput (median; whiskers = min–max over runs)")
    S.figure_caption(ax, S.DISCLAIMER["measured"])
    S.save(fig, "perf_throughput", "C", "cuda/reports/bench_*.json",
           "Measured evidence throughput per workload with the min–max spread across benchmark runs.")


def fig_v1_v2_speedup(run=None):
    """C20 ★ — the V2 optimization deltas (digest-equivalence-gated), measured on the deep 1024×8192 kernel.

    Numbers transcribed from the committed measured results in docs/cuda_evidence_kernel_v2_design.md /
    reports/NSIGHT_SUMMARY.md: V2-B segment-parallel takes the deep kernel 29.75 ms → 1.65 ms (~18×) and
    SM throughput 2.6% → 49%; V2-A lane-batching lifts occupancy 8.3% → 52% with a byte-identical
    evidence_root. The whole point of the digest-equivalence law: only timing changed.
    """
    # (metric, V1, V2, unit) — measured on RTX 4080 SUPER, from the committed V2 design doc / NSIGHT_SUMMARY.
    rows = [("kernel time (ms)", 29.75, 1.65, "ms"),
            ("SM throughput (%)", 2.6, 49.0, "%"),
            ("occupancy (%)", 8.3, 52.0, "%")]
    fig, ax = plt.subplots(figsize=(7.2, 3.8))
    y = np.arange(len(rows))
    h = 0.36
    ax.barh(y + h / 2, [r[1] for r in rows], height=h, color=S.MUTE, label="V1 baseline")
    ax.barh(y - h / 2, [r[2] for r in rows], height=h, color=S.OK, label="V2 (digest-identical / opt-in)")
    for yi, r in zip(y, rows):
        ax.text(max(r[1], r[2]) * 1.02, yi, f"{r[1]:g} → {r[2]:g} {r[3]}", va="center", fontsize=7.5)
    ax.set_yticks(y); ax.set_yticklabels([r[0] for r in rows])
    ax.set_xlabel("value (note: lower is better for kernel time; higher for SM/occupancy)")
    ax.set_title("V2 evidence-kernel optimization (deep 1024×8192; ~18× kernel speedup)")
    ax.legend(frameon=False, fontsize=8, loc="lower right")
    S.figure_caption(ax, "Digest-equivalence-gated: evidence_root byte-identical to V1; only timing changed. " + S.DISCLAIMER["measured"])
    S.save(fig, "perf_v1_v2_speedup", "C", "docs/cuda_evidence_kernel_v2_design.md (measured)",
           "V2 optimization deltas on the deep kernel — ~18× faster with a byte-identical evidence_root.")


def fig_nsys_gantt(run=None):
    """C25 — an nsys-style timeline: H2D copy then kernel, per workload (median ms), drawn as a gantt."""
    d = _nsight()
    vs = d["nsys_variants"]
    fig, ax = plt.subplots(figsize=(7.2, 3.4))
    for i, v in enumerate(vs):
        h2d = v["h2d_ms_median"]
        ker = v["kernel_ms_median"]
        ax.barh(i, h2d, left=0, color=S.CB["skyblue"], height=0.55, label=("H2D copy" if i == 0 else None))
        ax.barh(i, ker, left=h2d, color=S.ACCENT, height=0.55, label=("evidence kernel" if i == 0 else None))
        ax.text(h2d + ker, i, f"  {h2d:.1f}+{ker:.1f} ms", va="center", fontsize=7.5)
    ax.set_yticks(range(len(vs)))
    ax.set_yticklabels([f"{v['lanes']}×{v['samples']}" for v in vs])
    ax.set_xlabel("time (ms)")
    ax.set_title("Per-workload timeline: H2D transfer + evidence kernel (nsys median)")
    ax.legend(frameon=False, fontsize=8, loc="lower right")
    S.figure_caption(ax, S.DISCLAIMER["measured"])
    S.save(fig, "perf_nsys_gantt", "C", "cuda/reports/nsight_summary.json (nsys_variants)",
           "nsys-style timeline of host-to-device transfer and the evidence kernel per workload.")


# ── Conceptual CUDA diagrams (graphviz) ────────────────────────────────────────────────────────────────
def fig_pipeline(run=None):
    """C21 ★ — the CUDA evidence pipeline: raw lane → fixed-point → SHA-256 → Merkle → evidence_root."""
    dot = '''digraph P {
      rankdir=LR; bgcolor="white"; fontname="serif"; fontsize=13; labelloc="t";
      label="CUDA evidence factory — deterministic sealing pipeline";
      node [shape=box, style="rounded,filled", fontname="serif", fontsize=10, fillcolor="#e7eef5"];
      raw   [label="raw residual\\nlane (f64)"];
      fp    [label="fixed-point\\nquantise\\n(SCALE=1e6)", fillcolor="#d6ebe2"];
      rec   [label="evidence record\\nraw|q|e|d|s"];
      sha   [label="on-GPU\\nSHA-256\\n(per lane)", fillcolor="#f3e9d6"];
      mk    [label="Merkle tree\\n(per dataset)", fillcolor="#f3e9d6"];
      root  [label="evidence_root\\n(byte-exact = CPU)", fillcolor="#d6ebe2", color="#009E73", penwidth=2];
      raw -> fp -> rec -> sha -> mk -> root;
      edge [fontname="serif", fontsize=8, color="#555"];
    }'''
    if not S.render_dot(dot, "perf_pipeline", "C", "conceptual (cuda evidence contract)",
                        "The deterministic CUDA evidence pipeline: fixed-point → SHA-256 → Merkle → evidence_root."):
        _flow_fallback("perf_pipeline", ["raw lane", "fixed-point", "record", "SHA-256", "Merkle", "evidence_root"],
                       "CUDA evidence factory pipeline")


def fig_merkle(run=None):
    """C22 — the per-dataset Merkle evidence tree: lane digests combine pairwise up to the evidence_root."""
    dot = '''digraph M {
      bgcolor="white"; fontname="serif"; fontsize=13; labelloc="t";
      label="Merkle evidence tree (lane digests → evidence_root)";
      node [shape=box, style="rounded,filled", fontname="serif", fontsize=9, fillcolor="#e7eef5"];
      root [label="evidence_root", fillcolor="#d6ebe2", color="#009E73", penwidth=2];
      n01 [label="H(0,1)"]; n23 [label="H(2,3)"];
      l0 [label="lane 0\\ndigest", fillcolor="#f3e9d6"]; l1 [label="lane 1\\ndigest", fillcolor="#f3e9d6"];
      l2 [label="lane 2\\ndigest", fillcolor="#f3e9d6"]; l3 [label="lane 3\\ndigest", fillcolor="#f3e9d6"];
      root -> n01; root -> n23; n01 -> l0; n01 -> l1; n23 -> l2; n23 -> l3;
      edge [color="#555"];
    }'''
    if not S.render_dot(dot, "perf_merkle", "C", "conceptual (evidence Merkle)",
                        "Per-dataset Merkle tree combining lane digests up to the evidence_root."):
        _flow_fallback("perf_merkle", ["lane digests", "pairwise H", "evidence_root"], "Merkle evidence tree")


def fig_digest_equivalence(run=None):
    """C23 — the digest-equivalence law: every kernel variant yields the SAME evidence_root; only timing differs."""
    dot = '''digraph D {
      rankdir=LR; bgcolor="white"; fontname="serif"; fontsize=13; labelloc="t";
      label="Digest-equivalence law — same evidence, different timing";
      node [shape=box, style="rounded,filled", fontname="serif", fontsize=10];
      v1 [label="V1 reference\\n(per-lane)", fillcolor="#e7eef5"];
      va [label="V2-A\\nlane-batched", fillcolor="#e7eef5"];
      vb [label="V2-B\\nsegment-parallel", fillcolor="#e7eef5"];
      cpu [label="CPU reference", fillcolor="#e7eef5"];
      root [label="identical\\nevidence_root\\n+ Merkle + replay", shape=box,
            style="rounded,filled", fillcolor="#d6ebe2", color="#009E73", penwidth=2];
      v1 -> root; va -> root; vb -> root; cpu -> root [style=dashed];
      edge [color="#555"];
    }'''
    if not S.render_dot(dot, "perf_digest_equivalence", "C", "conceptual (digest-equivalence harness)",
                        "The digest-equivalence law: all kernel variants reproduce the identical evidence_root."):
        _flow_fallback("perf_digest_equivalence", ["V1", "V2-A", "V2-B", "CPU"], "Digest-equivalence law")


def fig_gpu_cpu_parity(run=None):
    """C24 — GPU↔CPU byte-exact parity: the evidence_root computed on the device equals the CPU reference."""
    fig, ax = plt.subplots(figsize=(7.0, 2.6))
    ax.axis("off")
    ax.text(0.5, 0.85, "evidence_root  (GPU)  ==  evidence_root  (CPU)", ha="center", fontsize=12, family="monospace")
    ax.text(0.5, 0.55, "byte-for-byte identical on every dataset (gpu_cpu_parity gate)", ha="center", fontsize=9, color="#333")
    ax.text(0.5, 0.30, "fixed-point determinism (SCALE=1e6) + --fmad=false + on-GPU SHA-256", ha="center", fontsize=8, color="#555")
    ax.add_patch(plt.Rectangle((0.06, 0.12), 0.88, 0.78, fill=False, ec=S.OK, lw=1.6))
    ax.text(0.5, 0.02, "Determinism is exact and hardware-independent; only wall-clock timing is hardware-specific.",
            ha="center", fontsize=7, style="italic", color="#555")
    ax.set_title("GPU ↔ CPU evidence parity", fontsize=11)
    S.save(fig, "perf_gpu_cpu_parity", "C", "conceptual (gpu_cpu_parity gate)",
           "The CUDA court's evidence_root is byte-identical to the CPU reference on every dataset.")


def _flow_fallback(fid, steps, title):
    """matplotlib left-to-right box-flow fallback when graphviz `dot` is unavailable."""
    fig, ax = plt.subplots(figsize=(7.2, 2.4))
    ax.set_xlim(0, len(steps)); ax.set_ylim(0, 1); ax.axis("off")
    for i, s in enumerate(steps):
        ax.add_patch(plt.Rectangle((i + 0.05, 0.3), 0.8, 0.4, fc="#e7eef5", ec="#333"))
        ax.text(i + 0.45, 0.5, s, ha="center", va="center", fontsize=8)
        if i < len(steps) - 1:
            ax.annotate("", xy=(i + 1.05, 0.5), xytext=(i + 0.85, 0.5), arrowprops=dict(arrowstyle="-|>", color="#333"))
    ax.set_title(title, fontsize=11)
    S.save(fig, fid, "C", "conceptual (graphviz fallback)", title)


def render_all(run):
    """Render every group-C figure (data-driven from cuda/reports + conceptual graphviz diagrams)."""
    S.log("group C — performance & CUDA evidence court")
    fig_roofline(run)
    fig_ncu_heatmap(run)
    fig_throughput_bars(run)
    fig_v1_v2_speedup(run)
    fig_nsys_gantt(run)
    fig_pipeline(run)
    fig_merkle(run)
    fig_digest_equivalence(run)
    fig_gpu_cpu_parity(run)
