# Nsight + throughput summary (RTX 4080 SUPER)

All figures measured locally on an NVIDIA GeForce RTX 4080 SUPER (driver 595.71.05,
CUDA 13.2). Nsight Systems (`nsys`) kernel/memory timings are medians over 5 runs per
size variant; GB/s for the evidence factory is `input_bytes / kernel_time`.

## Evidence-factory kernel (nsys, 5 runs/variant)

| variant | lanes×samples | MB | kernel ms (median) | min | max | evidence GB/s | H2D ms |
|---|---|---|---|---|---|---|---|
| a | 2048×4096 | 64 | 14.053 | 14.018 | 14.239 | 4.78 | 5.067 |
| b | 1024×8192 | 64 | 27.752 | 27.506 | 27.879 | 2.42 | 5.068 |
| g | 4096×2048 | 64 | 7.098 | 6.905 | 7.287 | 9.46 | 4.923 |

**Memory roofline (CUDA-event, median of bench sweeps): 636.8 GB/s** (~88% of the device's ~736 GB/s peak).

The evidence kernel is SHA-256-compute-bound (one thread per lane, sequential over
samples), so its effective bandwidth sits well below the memory roofline — the honest
cost of cryptographically sealing every residual and its noise into a replayable digest.

## Nsight Compute (ncu) microarchitectural metrics

Counters captured with GPU performance-counter access enabled; medians over 5 runs
per variant. The kernel is compute-bound: SMs and DRAM run far below peak while the
L2 hit rate is high. (ncu serialises kernel replay for counter collection, so its
absolute kernel time exceeds the `nsys` timings above — use `nsys` for wall-clock.)

| variant | lanes×samples | SM tput % | DRAM tput % | occupancy % | L2 hit % |
|---|---|---|---|---|---|
| a | 2048×4096 | 5.16 | 1.19 | 8.28 | 97.41 |
| b | 1024×8192 | 2.6 | 0.85 | 8.28 | 96.28 |
| g | 4096×2048 | 10.24 | 1.83 | 8.28 | 97.91 |

## V2 evidence-kernel optimizations (digest-equivalence-gated)

Measured GPU optimizations behind the digest-equivalence law — each gated byte-for-byte
against the CPU reference across an adversarial battery. Full before/after counters and the
rationale are in `docs/cuda_evidence_kernel_v2_design.md`; the V1 figures above are the
baseline-sealing reference these build on.

- **V2-A lane batching** (digest-IDENTICAL): one launch over many datasets' lanes; achieved
  occupancy 8.3% -> 52% by clearing the 80-SM grid wall. Same `evidence_root` as V1.
- **V2-B segment-parallel** (opt-in `evidence_root_v2`, Merkle-segment, `SEGMENT_SIZE=256`):
  deep 1024x8192 kernel 29.75 ms / 2.6% SM -> 1.65 ms / 49% SM (~18x), including two
  digest-preserving micro-opts (funnel-shift rotate + unrolled rounds; chunked SHA buffering).
- **Pinned H2D** (`cudaHostRegister`): transfer ~13.5 -> 26.7 GB/s (~2x). End-to-end deep ~8x.
- **Stream-overlap** (`cudaMemcpyAsync` + multi-stream): built + measured 2.7x SLOWER (chunking
  collapses each chunk below the occupancy knee) and reverted — disclosed as a negative result.
