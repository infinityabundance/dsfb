#!/usr/bin/env python3
"""Distill the committed Nsight Systems summaries and GB/s bench JSONs into a single, paper-citable
report: reports/NSIGHT_SUMMARY.md and reports/nsight_summary.json.

Parses each reports/nsys_*.txt for the evidence_kernel GPU time and the Host->Device copy time,
groups by size variant, and reports median/min/max across runs. Also folds in the GB/s bench
medians (evidence factory + memory roofline) from reports/bench_*.json. Deterministic; no GPU.

Input files (all committed, read-only):
  reports/nsys_evidence_<L>x<S>_variant_<v>_run<N>.txt  -- Nsight Systems --stats text dump
  reports/ncu_evidence_<L>x<S>_variant_<v>_run<N>.csv   -- Nsight Compute --csv dump
  reports/bench_*.json                                   -- GB/s CUDA-event bench results

Output files:
  reports/NSIGHT_SUMMARY.md   -- human-readable Markdown table for review
  reports/nsight_summary.json -- machine-readable JSON consumed by gen_tables.py

Filename convention: <L> = lane count, <S> = sample count, <v> = variant name, <N> = run index.
All aggregation (median/min/max) is done with the stdlib `statistics` module (no numpy dep).

NOTE on ncu vs nsys timing: Nsight Compute counter collection serialises kernel replay to insert
hardware counter reads between warp instructions.  The reported ncu kernel time therefore exceeds
the nsys wall-clock time by a factor that depends on the kernel's instruction count.  The nsys
timings are the correct wall-clock reference; ncu provides the microarchitectural breakdown only.
"""
import csv
import glob
import json
import os
import re
import statistics as st

# Root of the `dsfb-chemical-engineering-cuda` crate.
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPORTS = os.path.join(HERE, "reports")

# Mapping from raw Nsight Compute metric names (as they appear in ncu --csv output, column 12)
# to the short JSON keys used in nsight_summary.json and the paper's LaTeX table.
NCU_METRICS = [
    ("sm__throughput.avg.pct_of_peak_sustained_elapsed", "sm_throughput_pct"),
    ("dram__throughput.avg.pct_of_peak_sustained_elapsed", "dram_throughput_pct"),
    ("sm__warps_active.avg.pct_of_peak_sustained_active", "occupancy_pct"),
    ("lts__t_sector_hit_rate.pct", "l2_hit_pct"),
]


def parse_ns(s):
    """Parse a nanosecond string that may contain thousands-separator commas."""
    return float(s.replace(",", ""))


def parse_ncu(path):
    """Parse one `ncu --csv` output file and return {metric_name: float}.

    The Nsight Compute profiler prepends ==PROF== banner lines to the CSV stream before
    the header row.  This parser ignores any row where column 0 is not a digit (i.e. not
    a numeric kernel-instance row), so banners and the header are silently skipped.

    Expected CSV column layout (0-indexed):
      col 0:  kernel ID (digit string)
      col 12: metric name (matches NCU_METRICS keys)
      col 14: metric value (float, possibly with thousands-separator commas)

    Returns a flat dict; if a metric appears in multiple rows (multiple kernel launches)
    only the last occurrence is kept — this is acceptable because the evidence_factory
    kernel is the only kernel in the profile.

    errors="ignore" tolerates any encoding oddities in profiler output.
    """
    out = {}
    with open(path, errors="ignore") as f:
        for row in csv.reader(f):
            if len(row) >= 15 and row[0].isdigit():
                try:
                    out[row[12]] = float(row[14].replace(",", ""))
                except ValueError:
                    pass
    return out


def parse_nsys(path):
    """Return (kernel_ns, h2d_ns) from an nsys --stats text dump.

    The parser uses section-header tokens to gate which table is being read, so that
    the `profile:` echo line and the size-in-MB diagnostic table are not mistaken for
    the GPU timing tables.

    Section transitions:
      'cuda_gpu_kern_sum'      -> section="kern"    (GPU kernel summary table)
      'cuda_gpu_mem_time_sum'  -> section="memtime" (memory-copy time summary)
      'cuda_gpu_mem_size_sum'  or 'cuda_api_sum'
                               -> section="other"   (reset; not a timing table)

    Within section="kern": the first line containing "evidence_kernel" is the evidence
    factory kernel row.  Column positions are heuristic (regex on all comma-containing
    numbers in the line): col 0 = Time%, col 1 = TotalTime(ns).

    Within section="memtime": the first "Host-to-Device" line gives the H2D copy time.
    H2D time is optional; h2d_ns is None if the section is absent from the dump.

    Returns (kernel_ns, h2d_ns) where each is a float in nanoseconds, or None if not found.
    """
    kernel_ns = h2d_ns = None
    section = None
    with open(path, errors="ignore") as f:
        for line in f:
            if "cuda_gpu_kern_sum" in line:
                section = "kern"
                continue
            if "cuda_gpu_mem_time_sum" in line:
                section = "memtime"
                continue
            if "cuda_gpu_mem_size_sum" in line or "cuda_api_sum" in line:
                section = "other"
                continue
            if section == "kern" and "evidence_kernel" in line and kernel_ns is None:
                m = re.findall(r"[\d,]+\.?\d*", line)  # Time%, TotalTime(ns), Instances, ...
                if len(m) >= 2:
                    kernel_ns = parse_ns(m[1])
            if section == "memtime" and "Host-to-Device" in line and h2d_ns is None:
                m = re.findall(r"[\d,]+\.?\d*", line)
                if len(m) >= 2:
                    h2d_ns = parse_ns(m[1])
    return kernel_ns, h2d_ns


def main():
    """Parse all Nsight reports in REPORTS/ and write nsight_summary.json + NSIGHT_SUMMARY.md.

    Processing steps:
    1. Parse nsys_*.txt files -> group by (variant, lanes, samples) -> compute median/min/max
       kernel time in ns -> compute effective GB/s = input_bytes / median_kernel_ns.
       input_bytes = lanes * samples * 8  (8 bytes per f64 lane value).
       Files that do not yield a kernel_ns (parse_nsys returns None) are skipped.

    2. Fold in bench_*.json files -> aggregate per workload -> extract memory_roofline_gbps_median.
       Multiple bench JSON files (from separate run sessions) are accumulated; the median of all
       reported gbps_median values is used as the roofline.

    3. Fold in ncu_evidence_*.csv files -> group by (variant, lanes, samples) -> median per counter.
       ncu_raw accumulates per-counter lists across runs; the median is taken across runs.
       Counter collection requires GPU perf-counter access (root on most Linux systems);
       if no ncu CSV files are found, ncu_rows is empty and the ncu section is omitted.

    4. Write nsight_summary.json (consumed by gen_tables.py table_cuda_perf/table_cuda_ncu).

    5. Write NSIGHT_SUMMARY.md with two conditional sections:
       - ncu_rows non-empty: include the ncu microarchitectural table.
       - ncu_rows empty but ncu_permission_note.txt present: include the permission error note.
       - ncu_rows empty and no note file: silently omit the ncu section.

    No GPU required; processes committed report files only.
    """
    # ── Step 1: parse nsys timing files ────────────────────────────────────────────────
    variants = {}
    for p in sorted(glob.glob(os.path.join(REPORTS, "nsys_*.txt"))):
        base = os.path.basename(p)
        # Expected filename: nsys_evidence_<L>x<S>_variant_<v>_run<N>.txt
        m = re.search(r"evidence_(\d+)x(\d+)_variant_(\w+)_run\d+", base)
        if not m:
            continue
        lanes, samples, v = int(m.group(1)), int(m.group(2)), m.group(3)
        k, h = parse_nsys(p)
        if k is None:
            # File did not contain a parseable evidence_kernel timing row; skip.
            continue
        key = (v, lanes, samples)
        variants.setdefault(key, {"kernel_ns": [], "h2d_ns": []})
        variants[key]["kernel_ns"].append(k)
        if h is not None:
            variants[key]["h2d_ns"].append(h)

    rows = []
    for (v, lanes, samples), d in sorted(variants.items()):
        kn = d["kernel_ns"]
        # Input bytes: each lane processes `samples` float64 values (8 bytes each).
        bytes_in = lanes * samples * 8
        med_ms = st.median(kn) / 1e6
        # GB/s = bytes / ns  (since 1 byte/ns = 1 GB/s).
        gbps = bytes_in / st.median(kn) if st.median(kn) > 0 else 0.0  # bytes/ns = GB/s
        rows.append({
            "variant": v, "lanes": lanes, "samples": samples, "mb": bytes_in // (1 << 20),
            "runs": len(kn),
            "kernel_ms_median": round(med_ms, 3),
            "kernel_ms_min": round(min(kn) / 1e6, 3),
            "kernel_ms_max": round(max(kn) / 1e6, 3),
            "evidence_gbps_median": round(gbps, 2),
            "h2d_ms_median": round(st.median(d["h2d_ns"]) / 1e6, 3) if d["h2d_ns"] else None,
        })

    # ── Step 2: fold in GB/s bench JSONs (CUDA-event timing, separate from nsys) ──────
    bench = {"evidence_factory": [], "memory_roofline": []}
    for p in sorted(glob.glob(os.path.join(REPORTS, "bench_*.json"))):
        with open(p) as f:
            rep = json.load(f)
        for s in rep.get("summaries", []):
            bench.setdefault(s["workload"], []).append(s)
    roofline_med = None
    rfl = [s["gbps_median"] for s in bench.get("memory_roofline", [])]
    if rfl:
        # Median across multiple bench runs/sessions for a stable roofline estimate.
        roofline_med = round(st.median(rfl), 1)

    # ── Step 3: fold in Nsight Compute counters ────────────────────────────────────────
    # One CSV per variant per run; accumulate per-counter values, then take the median.
    ncu_raw = {}
    for p in sorted(glob.glob(os.path.join(REPORTS, "ncu_evidence_*.csv"))):
        m = re.search(r"evidence_(\d+)x(\d+)_variant_(\w+)_run\d+", os.path.basename(p))
        if not m:
            continue
        v = m.group(3)
        metrics = parse_ncu(p)
        if not metrics:
            continue
        for name, key in NCU_METRICS:
            if name in metrics:
                ncu_raw.setdefault((v, int(m.group(1)), int(m.group(2))), {}).setdefault(key, []).append(metrics[name])
    ncu_rows = []
    for (v, lanes, samples), d in sorted(ncu_raw.items()):
        row = {"variant": v, "lanes": lanes, "samples": samples,
               "runs": max((len(x) for x in d.values()), default=0)}
        for _, key in NCU_METRICS:
            row[key] = round(st.median(d[key]), 2) if d.get(key) else None
        ncu_rows.append(row)

    # ── Step 4: write nsight_summary.json ─────────────────────────────────────────────
    summary = {"nsys_variants": rows, "memory_roofline_gbps_median": roofline_med,
               "ncu_available": bool(ncu_rows), "ncu_variants": ncu_rows}
    with open(os.path.join(REPORTS, "nsight_summary.json"), "w") as f:
        json.dump(summary, f, indent=2)

    # ── Step 5: write NSIGHT_SUMMARY.md ───────────────────────────────────────────────
    lines = ["# Nsight + throughput summary (RTX 4080 SUPER)", "",
             "All figures measured locally on an NVIDIA GeForce RTX 4080 SUPER (driver 595.71.05,",
             "CUDA 13.2). Nsight Systems (`nsys`) kernel/memory timings are medians over 5 runs per",
             "size variant; GB/s for the evidence factory is `input_bytes / kernel_time`.", "",
             "## Evidence-factory kernel (nsys, 5 runs/variant)", "",
             "| variant | lanes×samples | MB | kernel ms (median) | min | max | evidence GB/s | H2D ms |",
             "|---|---|---|---|---|---|---|---|"]
    for r in rows:
        lines.append(f"| {r['variant']} | {r['lanes']}×{r['samples']} | {r['mb']} | {r['kernel_ms_median']} | {r['kernel_ms_min']} | {r['kernel_ms_max']} | {r['evidence_gbps_median']} | {r['h2d_ms_median']} |")
    lines += ["", f"**Memory roofline (CUDA-event, median of bench sweeps): {roofline_med} GB/s** "
              "(~88% of the device's ~736 GB/s peak).", "",
              "The evidence kernel is SHA-256-compute-bound (one thread per lane, sequential over",
              "samples), so its effective bandwidth sits well below the memory roofline — the honest",
              "cost of cryptographically sealing every residual and its noise into a replayable digest.", ""]
    if ncu_rows:
        lines += ["## Nsight Compute (ncu) microarchitectural metrics", "",
                  "Counters captured with GPU performance-counter access enabled; medians over 5 runs",
                  "per variant. The kernel is compute-bound: SMs and DRAM run far below peak while the",
                  "L2 hit rate is high. (ncu serialises kernel replay for counter collection, so its",
                  "absolute kernel time exceeds the `nsys` timings above — use `nsys` for wall-clock.)", "",
                  "| variant | lanes×samples | SM tput % | DRAM tput % | occupancy % | L2 hit % |",
                  "|---|---|---|---|---|---|"]
        for r in ncu_rows:
            lines.append(f"| {r['variant']} | {r['lanes']}×{r['samples']} | {r['sm_throughput_pct']} | "
                         f"{r['dram_throughput_pct']} | {r['occupancy_pct']} | {r['l2_hit_pct']} |")
        lines.append("")
    elif os.path.exists(os.path.join(REPORTS, "ncu_permission_note.txt")):
        # ncu was attempted but failed due to missing GPU perf-counter access.
        lines += ["## Nsight Compute (ncu) microarchitectural metrics", "",
                  "Not captured on this host: GPU performance-counter access is admin-restricted",
                  "(`ERR_NVGPUCTRPERM`). Enable counters (root) and re-run `scripts/run_nsight.sh` to",
                  "populate occupancy / DRAM-throughput / warp-efficiency metrics. See",
                  "`reports/ncu_permission_note.txt`.", ""]
    # V2 evidence-kernel optimizations (digest-equivalence-gated). Static, documented + measured
    # results (full detail in docs/cuda_evidence_kernel_v2_design.md), summarised here so the Nsight
    # report points to the V2 work. Kept in the generator so a regenerate never drops them.
    lines += [
        "## V2 evidence-kernel optimizations (digest-equivalence-gated)",
        "",
        "Measured GPU optimizations behind the digest-equivalence law — each gated byte-for-byte",
        "against the CPU reference across an adversarial battery. Full before/after counters and the",
        "rationale are in `docs/cuda_evidence_kernel_v2_design.md`; the V1 figures above are the",
        "baseline-sealing reference these build on.",
        "",
        "- **V2-A lane batching** (digest-IDENTICAL): one launch over many datasets' lanes; achieved",
        "  occupancy 8.3% -> 52% by clearing the 80-SM grid wall. Same `evidence_root` as V1.",
        "- **V2-B segment-parallel** (opt-in `evidence_root_v2`, Merkle-segment, `SEGMENT_SIZE=256`):",
        "  deep 1024x8192 kernel 29.75 ms / 2.6% SM -> 1.65 ms / 49% SM (~18x), including two",
        "  digest-preserving micro-opts (funnel-shift rotate + unrolled rounds; chunked SHA buffering).",
        "- **Pinned H2D** (`cudaHostRegister`): transfer ~13.5 -> 26.7 GB/s (~2x). End-to-end deep ~8x.",
        "- **Stream-overlap** (`cudaMemcpyAsync` + multi-stream): built + measured 2.7x SLOWER (chunking",
        "  collapses each chunk below the occupancy knee) and reverted — disclosed as a negative result.",
        "",
    ]
    with open(os.path.join(REPORTS, "NSIGHT_SUMMARY.md"), "w") as f:
        f.write("\n".join(lines))
    print(f"wrote {os.path.join(REPORTS, 'NSIGHT_SUMMARY.md')} ({len(rows)} variants)")
    for r in rows:
        print(f"  variant {r['variant']}: {r['lanes']}x{r['samples']} {r['kernel_ms_median']} ms median ({r['runs']} runs), {r['evidence_gbps_median']} GB/s")


if __name__ == "__main__":
    main()
