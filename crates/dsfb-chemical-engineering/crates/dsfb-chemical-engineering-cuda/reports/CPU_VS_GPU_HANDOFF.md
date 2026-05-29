# CPU-vs-GPU end-to-end timing — handoff (Wave 5)

> The evidence-factory **end-to-end** throughput on CPU vs GPU, at realistic lane counts, framed honestly:
> the GPU's value here is **auditability + determinism**, not raw throughput (risk-register R4). The
> byte-exact `evidence_root` is identical across both backends — that invariant is the point; timing is
> secondary. CPU numbers were measured in-sandbox; **GPU numbers are reproduced by the user** (the sandbox
> cannot reach the GPU — see the Nsight handoff loop below).

## Measured comparison (same harness, same sizes, `bench_evidence`)
| Workload (lanes × samples) | Bytes | CPU-reference GB/s | GPU (RTX 4080 SUPER) GB/s | GPU end-to-end speedup |
|---|---|---|---|---|
| 1024 × 4096   | 32 MB  | 0.285 | 2.111 | **7.4×** |
| 2048 × 8192   | 128 MB | 0.298 | 4.744 | **15.9×** |
| 4096 × 16384  | 512 MB | 0.289 | 9.599 | **33.2×** |
| (roofline: 1 × 67.1 M streaming) | 512 MB | — | 636.8 | DRAM ceiling |

- CPU-reference: median of 5 runs/size, this machine, `cuda=false` build — `reports/bench_20260525T212325Z.json`.
- GPU: median of 5 runs/size from the P5 campaign on the user's RTX 4080 SUPER — `reports/bench_20260523T070807Z.json`.
- The end-to-end GPU speedup **grows with size** (7.4× → 33×) as the kernel amortises launch + transfer
  overhead, but at 9.6 GB/s the evidence kernel still runs **~66× below the 637 GB/s memory roofline** — the
  per-sample SHA-256 work, not bandwidth, is the limiter. This is exactly R4: at realistic sizes the GPU is
  not a throughput story; it is a *deterministic, hash-sealed, byte-reproducible* evidence factory that
  happens to also be faster than the CPU reference.

## Reproduce it (fish) — user runs the GPU side, pastes the numbers back
```fish
# CPU reference (runs anywhere; no GPU):
cargo run --release -p dsfb-chemical-engineering-cuda --bin dsfb-chem-cuda -- bench

# GPU (needs nvcc + the NVIDIA GPU):
set -x CUDA_HOME /opt/cuda ; set -x PATH /opt/cuda/bin $PATH
cargo run --release -p dsfb-chemical-engineering-cuda --features cuda --bin dsfb-chem-cuda -- bench

# Nsight Systems end-to-end timeline (transfers + launches + kernel), per the existing harness:
bash crates/dsfb-chemical-engineering-cuda/scripts/run_nsight.sh
python3 crates/dsfb-chemical-engineering-cuda/scripts/summarize_nsight.py
```
Paste the GPU `bench_*.json` medians + the `nsys` summary back here and I will fold the refreshed numbers into
this table, `reports/NSIGHT_SUMMARY.md`, and the paper's GPU section. (The `ncu` per-counter profile remains a
root-gated step the user runs — the standing Nsight handoff.)

## Catalogued next optimisation (not executed)
**CUDA Graphs / persistent kernel** to cut per-launch overhead at the small/medium sizes where launch cost
dominates the 7× end-to-end gap. This is a *catalogued* target, not executed: any such change must reproduce
the byte-exact `evidence_root` (gated by `DigestEquivalenceHarnessV1`) before it is accepted — a speed change
that altered the evidence root would be rejected. Tuning is selected via the Nsight handoff, never blind.

## Non-claims
GPU throughput here is **workload- and size-dependent** and **not the primary value claim**; the value is the
deterministic, cross-backend-identical `evidence_root` and the auditable execution attestation. No real-time
guarantee is made. The CPU reference is the correctness oracle the GPU must match byte-for-byte.
