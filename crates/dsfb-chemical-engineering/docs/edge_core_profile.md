# EdgeCoreProfileV1 — a `no_std`, no-heap, fixed-point profile of the DSFB core (design)

**Status: design only (P55).** Nothing here is built; this documents *how* the DSFB residual-semiotics
core would run on a microcontroller, what is already proven, and what would have to change. It is a
prior-art disclosure of the embedded profile, not a claim that the shipped crate is `no_std` (it is
explicitly std-only — `#![forbid(unsafe_code)]`, dependency-light, but std).

## 1. What "the core" is (vs the shell)

The `dsfb-chemical-engineering-edge` crate splits cleanly into a **compute core** and an **I/O shell**:

| layer | modules (today) | character |
|---|---|---|
| **core** — the DSFB grammar | `dsfb_core` (ResidualTriple: drift δ = causal windowed mean, slew σ = first difference), `detectors` (the executed chemometric scores), `fusion` (deterministic quorum), `heuristics`/`balance` (the bank + balance witnesses) | pure, causal, streaming, fixed working set |
| **shell** — everything that touches the outside world | `data`/`datasets` (CSV), `court_record`/`report`/`figures` (JSON/HTML/PNG), `cli`, `historian` | heap + filesystem + formatting |

The core is what an MCU runs online; the shell is the desktop/server reporting tier. Only the core needs
the `no_std` profile.

## 2. What is already proven (the existence proof)

The CUDA evidence contract (`crates/dsfb-chemical-engineering-cuda/src/evidence.rs`) **already implements
the DSFB core in pure integer fixed-point**, with **no heap and a fixed working set**, and it is gated
byte-for-byte against the CPU reference:

- residuals quantised to `i64` at `SCALE = 1e6` (round-half-away-from-zero, no FMA — `dsfb_quantize`);
- causal drift δ from a **fixed `DRIFT_WINDOW = 16` integer ring buffer** with an incrementally-maintained
  `ring_sum` (no recomputation, no allocation);
- slew σ, one-sided exceedance, and running peaks all **exact integer** arithmetic;
- per-sample state is a handful of `i64`s + the 16-slot ring — i.e. **O(window), not O(stream)**.

So the grammar's hot path is already shown to be integer, allocation-free, and bounded. EdgeCoreProfileV1
is the extraction of that proven shape into a reusable `no_std` core crate; it is not a new algorithm.

## 3. What changes vs the shipped std core

The shipped `dsfb_core` uses `f64` (drift/slew in engineering units) and `String` channel names; the
detectors use `Vec`. The `no_std` profile would:

1. **Fixed-point throughout.** Carry residuals/drift/slew as `i64` at a fixed scale (the evidence path's
   `1e6`, or a per-deployment scale), exactly as the CUDA contract does. The structural decisions
   (envelope membership, drift/slew sign + magnitude class) are threshold comparisons that are identical
   under a fixed scale — determinism is preserved, and the integer form is the *reference* the GPU already
   matches.
2. **No heap.** Replace `Vec`/`String` with `heapless` fixed-capacity types (or plain arrays): a
   `RingBuffer<i64, W>`, a fixed `[DetectorState; N_DET]`, channel identifiers as `u16` indices into a
   `&'static` name table (the atlas already keys detectors/heuristics by `&'static str`). No per-sample
   allocation; the working set is a compile-time constant.
3. **`#![no_std]` + `core`/`alloc`-free core.** The grammar needs only integer arithmetic and the ring;
   no `std::collections`, no formatting, no I/O. The atlas authority crate is **already `no_std`**, so the
   core can depend on it directly for the detector/heuristic/fault-signature records.
4. **Streaming API.** `fn step(&mut self, residual_q: i64, dt_ticks: u32) -> Option<StructuralEvent>` —
   one sample in, bounded work out, no look-ahead (the grammar is already causal).

## 4. Bounded-memory budget (target: RP2040-class)

Order-of-magnitude budget for a single-channel core (W = 16, fixed-point `i64`):

| item | size |
|---|---|
| drift ring buffer | `16 × 8 B` = 128 B |
| running scalars (ring_sum, prev_q, peaks, counters) | ~64 B |
| per-detector state (executed detectors, fixed N) | N × tens of B |
| envelope / heuristic thresholds | `&'static` (flash, not RAM) |

A handful of channels fits comfortably in **single-digit KB of SRAM** — well inside an RP2040's 264 KB,
with no dynamic allocation and no GC pause. The expensive, optional layer is the **SHA-256 evidence seal**
(`sha256.cuh` has the device form; a `no_std` `sha2` works on MCUs): sealing is opt-in, since an online
monitor may forward residual triples to the shell for sealing rather than seal on-device.

## 5. Deployment route (design)

1. Extract `dsfb-chemical-engineering-core` (`no_std`, `heapless`, depends only on the `no_std` atlas).
2. Host-side equivalence gate: the fixed-point core must reproduce the CUDA evidence contract's integer
   outputs byte-for-byte on the synthetic battery (the same `DigestEquivalenceHarnessV1` discipline) —
   the GPU path is the existing reference.
3. **QEMU** (`thumbv6m-none-eabi`) smoke run of the streaming `step` loop over a recorded residual trace.
4. **Cortex-M / RP2040** target: feed a live or replayed residual stream; emit structural events over
   UART/RTT; optionally seal on-device or forward to the shell.

## 6. Honest bounds

- This is a **design**; the `no_std` core crate is not built. The claim is only that the hot path is
  *already* integer/allocation-free/bounded (proven by the CUDA evidence contract) and therefore portable.
- The detector **bank** that needs matrix work (PCA/PLS) is not all MCU-friendly; the embedded profile
  targets the **drift–slew–envelope grammar + the lightweight detectors + fusion + heuristics**, with the
  heavier multivariate detectors staying on the shell. This is disclosed, not hidden.
- Determinism is preserved by construction (fixed scale + integer arithmetic + causal ring); the embedded
  core's outputs are the same structural episodes the desktop core produces, by the same equivalence
  discipline used for CPU↔GPU.
