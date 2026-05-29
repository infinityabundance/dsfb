# CUDA evidence-kernel V2 — design, grounded in measured Nsight counters

**Status:** design + measured baseline (P70). V1 (`evidence_kernel`, one-thread-per-lane, serial SHA-256
over samples) remains the byte-exact reference. Any V2 is admissible only behind the
**digest-equivalence law** (below) unless it is explicitly a new, opt-in evidence *format*.

All numbers here were **measured on the target hardware** — NVIDIA GeForce RTX 4080 SUPER (CC 8.9, CUDA
13.2) — with Nsight Systems (`nsys`, wall-clock) and Nsight Compute (`ncu`, microarchitectural counters,
GPU performance-counter access enabled via root). Medians over 5 runs per size variant. Raw artifacts:
`crates/dsfb-chemical-engineering-cuda/reports/{ncu_*.csv,nsys_*.txt,NSIGHT_SUMMARY.md}`.

## 1. Measured baseline (same 64 MB total work; pure aspect-ratio sweep)

| variant | lanes × samples | grid (blocks) | kernel ms (nsys) | evidence GB/s | SM tput % | DRAM tput % | warps_active % | L2 hit % |
|---|---|---|---|---|---|---|---|---|
| g | 4096 × 2048 | 32 | 7.10 | 9.46 | 10.24 | 1.83 | 8.28 | 97.9 |
| a | 2048 × 4096 | 16 | 14.05 | 4.78 | 5.16 | 1.19 | 8.28 | 97.4 |
| b | 1024 × 8192 | 8 | 27.75 | 2.42 | 2.60 | 0.85 | 8.28 | 96.3 |

Memory roofline (separate `roofline_kernel`, CUDA-event median): **636.8 GB/s ≈ 88% of the ~736 GB/s
device peak** — so the hardware and our measurement path can saturate memory; the evidence kernel simply
does not.

## 2. Diagnosis — the kernel is grid-starved, not memory- or ALU-bound

Three independent facts from the counters pin the cause:

1. **Kernel time scales exactly inversely with lane count** (7.1 → 14.1 → 27.8 ms as lanes halve
   4096 → 2048 → 1024, total work fixed). Time ∝ samples-per-lane = serial work per thread.
2. **`warps_active` is flat at 8.28% across every variant.** 8.28% = 4 warps / 48-max — i.e. each *active*
   SM holds exactly **one** 128-thread block (4 warps). The grid (8/16/32 blocks) is always smaller than
   the 80 SMs, so no SM ever receives a second block. `launch__occupancy_limit_warps = 12 blocks/SM` shows
   nothing structural (registers/shared memory) caps occupancy — the kernel *could* fill an SM to 48 warps
   (100%); it just never gets enough blocks.
3. **DRAM ≤ 1.83%, L2 hit ~97%.** Definitively not memory-bound; the working set is L2-resident.

**Total resident warps = lanes ÷ 32, full stop.** To fill the device (80 SMs × 12 blocks × 4 warps ≈ 3840
warps) you would need ≈123k lanes. Real datasets carry hundreds of lanes, so the kernel structurally
under-fills the GPU for every realistic single-run size. This is the empirical basis for the paper's
honest framing: the kernel is the **baseline deterministic-sealing** contract, chosen for auditable
determinism, **not** a throughput-optimized design.

## 3. The digest-equivalence law (the invariant any drop-in V2 must satisfy)

> For identical input, a V2 kernel MUST reproduce, byte-for-byte, V1's **per-lane SHA-256 digest**, the
> **Merkle/lane digest tree**, the **`evidence_root`**, and the **replay verdict**. Only wall-clock time
> may differ. A variant that changes any sealed byte is **not** a V2 of this kernel — it is a new,
> separately-versioned evidence *format*.

This is the law `DigestEquivalenceHarnessV1` enforces: it runs V1 and the candidate over the same inputs
and asserts every sealed artifact is identical, gating each benched variant before it is kept.

## 4. V2-A — lane-batching across datasets/runs (digest-IDENTICAL; the real win)

The single drop-in optimisation the data supports: **concatenate the lanes of all datasets/runs of a
workload into one kernel launch.** The grid grows from ~8–32 blocks toward hundreds, filling more SMs and
lifting `sm_throughput` proportionally (it tracks blocks-launched ÷ 80). Each lane's serial SHA is
untouched, so every per-lane digest — and therefore `evidence_root` — is byte-for-byte identical.
**Passes digest-equivalence by construction.** It speeds up the *aggregate* (e.g. 20-dataset) workload,
which is the realistic batch case; it cannot change the cost of any single lane.

Expected effect (from the measured linear SM-tput↔grid relation): a 20-dataset batch with, say, 8× the
lanes of variant g moves the grid from 32 toward ~256 blocks, i.e. toward filling all 80 SMs — a roughly
linear throughput gain until the grid reaches ~960 blocks, after which the device is full and the serial
per-lane SHA becomes the wall. **Confirm with a re-profiled `ncu` run on the batched grid** (Nsight
handoff, §6).

**Status: BUILT + GATED + measured (nsys).** `evidence_kernel_batched` (kernels.cu) + the
`run_evidence_batched` FFI + the `profile-batched` CLI are implemented; both V1 and V2-A share one
`__device__ dsfb_lane_evidence`, so V2-A is digest-identical *by construction*. The
`DigestEquivalenceHarnessV1` V2-A gate passes on the GPU (every battery lane, in one batched launch,
byte-identical to the CPU reference), and `golden_evidence` confirms the refactor left V1's frozen roots
unchanged. Measured `nsys` wall-clock on the RTX 4080 SUPER, same 64 MB total work:

| kernel | lanes | grid (blocks) | kernel ms (nsys) |
|---|---|---|---|
| V1 `evidence_kernel` (b) | 1024 | 8 | 27.75 |
| V1 `evidence_kernel` (g) | 4096 | 32 | 7.10 |
| **V2-A `evidence_kernel_batched`** | 16384 | 128 | **3.14** |

V2-A at 128 blocks (grid > 80 SMs) is **~2.3× faster than V1's best variant** — the predicted occupancy
win, realised by growing the grid rather than touching the per-lane SHA.

**`ncu` counter confirmation (measured, user sudo run; `evidence_kernel_batched`, 512 samples/lane):**

| grid (blocks) | warps_active % | SM tput % | DRAM % | L2 hit % |
|---|---|---|---|---|
| 32 (= V1 variant g) | 8.32 | 10.70 | 1.80 | 98.0 |
| 128 | 13.40 | 23.79 | 3.42 | 98.6 |
| 256 | 26.16 | 25.13 | 3.77 | 98.6 |
| 512 | **51.63** | 27.08 | 17.46 | 98.7 |

Two measured conclusions: (1) **V2-A breaks the occupancy ceiling** — at grid = 32 it equals V1
(8.32 ≈ 8.28%, same per-lane work), but past 80 SMs occupancy climbs **8% → 52%**, which the per-dataset
V1 path can never reach. (2) **It surfaces the next wall**: SM throughput rises 10.7 → 23.8% then
**plateaus ~25–27%** even as occupancy doubles to 52% — once the SMs are populated, the binding constraint
is the **serial SHA-256 dependency chain per lane**, not occupancy. This is the empirical justification for
V2-B (§5): parallelising the hash *within* a lane is the only lever left once the grid is full.

## 5. V2-B — Merkle-segment intra-lane format (NON-equivalent; opt-in `evidence_root_v2`)

The deep single-run case (few lanes, many samples — variant b) is **unrecoverable under
digest-equivalence**: total warps = lanes ÷ 32 is fixed, and the only way to add threads is to split a
lane's sample stream across them. SHA-256 is serial over its message, so any such split changes the
per-lane digest, hence `evidence_root` — it **fails** the law.

To parallelise within a lane you must *redefine the digest* as a **Merkle tree over fixed-size
sample-segments**: hash each segment independently (parallel across threads), then combine segment digests
up a tree to the per-lane root. This is a legitimate, disclosable design — but it is a **new evidence
format** (`evidence_root_v2`), not a drop-in: a V1 court record and a V2 court record of the same data
have different roots **by design**. It would ship opt-in, clearly labelled, with its own frozen reference
and its own replay gate; it never silently replaces V1.

**Status: BUILT + GATED + measured.** `lane_evidence_v2_cpu` (authoritative format), the GPU
`evidence_kernel_v2_segmented` (one thread per (lane, segment), with a `DRIFT_WINDOW`-sample causal halo
warm-up so each segment reconstructs the drift state at its boundary), the host per-lane combine (concat
segment digests → SHA-256 = `lane_root_v2`; reduce partial summaries), the FFI, and the
`profile-v2-segmented` CLI are implemented. The `DigestEquivalenceHarnessV1` V2-B gate passes on the GPU
at `seg=4` (partial + full halo warm-up, many segments) and `seg=512` — GPU == CPU Merkle-segment
reference byte-for-byte. Measured `ncu` on the **deep** case (1024×8192 — the shape V1 is worst at and
V2-A cannot help; `gpu__time_duration`):

| seg | segments | grid | time | warps % | SM % | DRAM % |
|---|---|---|---|---|---|---|
| V1 (reference) | — | 8 | 29.75 ms | 8.28 | 2.60 | 0.83 |
| 1024 | 8192 | 64 | 3.73 ms | 8.32 | 21.19 | 3.4 |
| **512 (then-`SEGMENT_SIZE`; pre-SHA-opt)** | 16384 | 128 | **3.17 ms** | 13.44 | 25.01 | 3.8 |
| 256 | 32768 | 256 | 3.16 ms | 26.17 | 25.16 | 4.0 |
| 128 | 65536 | 512 | 3.09 ms | 51.79 | 25.90 | 17.73 |

Three measured conclusions: (1) **V2-B cracks the deep case — 29.75 → 3.17 ms ≈ 9.4×** — the win V2-A
structurally cannot deliver (deep = 8 blocks, nothing to batch). (2) **The lever is the shortened SHA
chain, not occupancy**: at `seg=1024`, occupancy is still 8.32% (= V1) yet SM throughput already jumps
2.60 → 21.19% (~8×), purely from cutting each thread's chain from 8192 to 1024 updates. (3)
**`SEGMENT_SIZE` optimum (pre-SHA-opt = 512)**: at this point SM throughput maxed ~25% by 512 and smaller
segments added no throughput (25.0 → 25.9%) while halo re-reads ballooned DRAM — so 512 was the knee.
**This was re-evaluated after the §6b SHA micro-opt**, which gave the kernel headroom to convert occupancy
into throughput; the re-sweep moved the knee to **256** (see §6b). The serial-SHA ceiling (then ~25–27% SM,
~35% after §6b) is the SHA-256 dependency chain; full memory-roofline throughput would need a different
sealing primitive, out of scope for V2.

## 6. Nsight handoff protocol (measured workflow, for reproducing/extending this)

GPU performance counters are root-restricted (`ERR_NVGPUCTRPERM`) by default. `nsys` wall-clock does **not**
need the permission; `ncu` counters do. Commands are **fish-shell** (the dev environment). Binary:
`./target/release/dsfb-chem-cuda` built with `--features cuda` (nvcc auto-found at `/opt/cuda`).

Full campaign (3 variants × 5 runs, both tools), as root:

```fish
cd /home/one/dsfb-chemical-engineering
sudo bash crates/dsfb-chemical-engineering-cuda/scripts/run_nsight.sh 5
sudo chown -R one:one crates/dsfb-chemical-engineering-cuda/reports   # hand the files back
```

Single `ncu` counter run (one variant, absolute path so no `sudo cd` trap):

```fish
sudo env DSFB_BENCH_LANES=4096 DSFB_BENCH_SAMPLES=2048 \
  ncu --target-processes all --kernel-name evidence_kernel --launch-count 1 \
      --metrics gpu__time_duration.sum,sm__throughput.avg.pct_of_peak_sustained_elapsed,dram__throughput.avg.pct_of_peak_sustained_elapsed,sm__warps_active.avg.pct_of_peak_sustained_active,lts__t_sector_hit_rate.pct,launch__occupancy_limit_warps \
      /home/one/dsfb-chemical-engineering/target/release/dsfb-chem-cuda profile
```

Permanent counter enable (run once as root, then reboot — `ncu` then works without sudo):

```fish
echo 'options nvidia NVreg_RestrictProfilingToAdminUsers=0' | sudo tee /etc/modprobe.d/nvidia-profiling.conf
sudo mkinitcpio -P
```

## 6b. SHA-256 micro-optimization (digest-preserving, measured)

The per-thread SHA-256 compression — not memory — is the shared ~25% SM ceiling. Two **bit-identical**
implementation changes in `sha256.cuh` attack it without touching the output digest: the `dsfb_rotr`
rotate now uses the single-instruction `__funnelshift_r` (SHF.R) hardware rotate, and the message-schedule
(W[16..63]) and 64-round compression loops are `#pragma unroll`ed (the rotating `h=g;g=f;…` assignment
becomes register renaming). The `DigestEquivalenceHarnessV1` gate + `golden_evidence` confirm every digest
is unchanged — it is purely a speed change.

Measured `ncu` before→after (deep 1024×8192):

| config | `gpu__time_duration` | SM throughput |
|---|---|---|
| V2-B seg=512 — before | 3.17 ms | 25.01% |
| V2-B seg=512 — after | **2.52 ms** (1.26×) | **30.96%** (past the plateau) |
| V1 deep — before / after | 29.75 → 30.61 ms | 2.60 → 2.50% |

The change helps the **throughput-bound** kernels (V2-B, V2-A) — V2-B breaks the ~25% plateau to ~31% — but
is **neutral for the occupancy-bound V1 deep case** (8 warps can't hide the dependent-chain latency, so
faster instructions don't show; that case needs *threads*, which is precisely what V2-A/V2-B add). Net:
V2-B deep is now ~11.8× the V1 deep baseline (was 9.4×).

**Re-sweep after the SHA-opt → `SEGMENT_SIZE` lowered 512 → 256.** The faster rounds gave headroom to turn
occupancy into throughput, so (unlike the pre-opt sweep) shrinking `seg` now helps — deep 1024×8192:

| seg | grid | time | SM % | warps % | halo (16/seg) |
|---|---|---|---|---|---|
| 512 | 128 | 2.56 ms | 30.4 | 13.3 | 3.1% |
| **256 (new `SEGMENT_SIZE`)** | 256 | **2.40 ms** | 32.6 | 25.7 | 6.25% |
| 128 | 512 | 2.31 ms | 34.2 | 50.1 | 12.5% |
| 64 | 1024 | 2.24 ms | 35.6 | 52.7 | 25% |

**256 is the knee**: ~6% faster than 512 at 32.6% SM with a still-modest 6.25% halo. Below it the fixed
16-sample warm-up becomes pure waste (12.5% at 128, 25% at 64) and segment count / host-combine + D2H
transfer (not in `gpu__time_duration`) double each step — ~3% more kernel time isn't worth it. With
`SEGMENT_SIZE=256` the deep case is **~12.4× the V1 baseline** (29.75 → 2.40 ms) at that point.

## 6c. Chunked SHA-256 buffering (digest-preserving, measured) — the biggest single win

`dsfb_sha256_update` originally buffered the 40 bytes/sample **one byte at a time with a per-byte
boundary branch**. Restructuring it to fill up to the next 64-byte block boundary in one inner loop (one
branch per chunk, not per byte) is byte-for-byte identical buffering — same bytes, same block firing
points, same digest (gates confirm). Measured `ncu` before→after (deep 1024×8192):

| config | `gpu__time_duration` | SM throughput |
|---|---|---|
| V2-B seg=256 — before | 2.40 ms | 32.6% |
| V2-B seg=256 — after | **1.64 ms** (1.46×) | **49.35%** |
| V1 deep — before / after | 30.61 → 26.26 ms (1.17×) | 2.50 → 3.02% |

The per-byte branch was throttling the evidence feed far more than expected: removing it nearly **doubled**
V2-B SM throughput (32.6 → 49.35%) and even the occupancy-bound V1 gained ~14%. **Cumulative deep case:
29.75 → 1.64 ms ≈ 18×**, SM throughput 2.6% → 49% (from the original 25% plateau).

A final post-buffer-opt `seg` re-sweep **re-confirms `SEGMENT_SIZE=256`**: seg 256→128→64 gives
1.65→1.60→1.57 ms / 49.2→51.1→52.5% SM — i.e. sub-256 now buys ~3% then ~2% while halo doubles each step
(6.25→12.5→25%). The curve has flattened into the **SHA-256 compute ceiling** (~52% SM even at seg=64,
51% occupancy), so 256 stays the knee. This is the end of the V2 perf series: the remaining ceiling is the
SHA-256 compression primitive itself; beyond it would need a different primitive (out of scope for V2 — by
the digest-equivalence law for V2-A, and as a deliberate format choice for V2-B). Net across the series,
all digest-preserving and gated: V1 deep **29.75 → 1.65 ms ≈ 18×**, SM throughput **2.6% → 49%**.

## 6d. Transfer path: pinned H2D (digest-irrelevant, measured) — the new bottleneck once the kernel shrank

With the kernel down to ~1.6 ms, `nsys` end-to-end showed the **host→device transfer now dominates**: H2D
**5.00 ms** vs kernel 1.58 ms (≈3.2×) for the 64 MB residual matrix — the 67 MB chunk moved at **~13.5 GB/s**,
pageable-memory speed (≈half of PCIe 4.0 x16). Pinning the input buffer in place with `cudaHostRegister`
(best-effort, with a plain-copy fallback; applied in all three host wrappers — transfer speed only, no byte
changes, gates unaffected) roughly **halves** it:

| | H2D (67 MB) | bandwidth |
|---|---|---|
| pageable (before) | 5.00 ms | ~13.5 GB/s |
| pinned (after) | **2.55 ms** | **~26.7 GB/s** |

End-to-end deep case: kernel 1.58 + H2D 2.55 + D2H 0.09 ≈ **4.2 ms (was ~6.7 ms)**; the transfer is no
longer 3× the kernel.

**Stream-overlap (`cudaMemcpyAsync` + multi-stream) — explored and REJECTED (measured net-negative).** A
fully-built, gated, digest-identical streamed wrapper (pin input → chunk lanes → round-robin async-H2D /
kernel / async-D2H over N streams) was prototyped to hide the transfer behind compute. Measured at 16384
lanes × 512 (4 streams, 16 chunks): **11.3 ms end-to-end vs ~4.2 ms serial — 2.7× worse.** Cause, straight
from the counters: 16 chunks of 1024 lanes = **8 blocks each**, which collapses right back to the V1-deep
occupancy wall — each chunk kernel takes ~1.49 ms (≈ the *whole* 128-block kernel's 1.67 ms), so the 16
launches sum to ~24 ms of kernel work. The occupancy lost to chunking dwarfs the ~2.5 ms of transfer it
hides. Overlap only pays off when each chunk *still* has ≥~128 blocks (>~100k lanes); this project's real
workloads (hundreds of lanes) never reach that, so the code was reverted. The remaining honest lever is
allocating the input **pinned upstream** (no per-call `cudaHostRegister` overhead) — noted as a future item;
`cudaHostRegister` already captures most of the bandwidth at zero plumbing cost.

## 7. Honest bounds

- V2-A helps **aggregate batch** throughput; it does **not** speed up a single small-lane run.
- The deep single-run case is only recoverable by **changing the evidence format** (V2-B), which is a
  disclosed tradeoff, not a free lunch.
- All gains must be **re-measured** with `ncu` on the new grid; no throughput claim is made from
  geometry alone. The V1 kernel stays the canonical reference regardless of what V2 ships.
