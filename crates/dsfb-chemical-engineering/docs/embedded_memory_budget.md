# Embedded memory budget — `dsfb-chemical-engineering-core` (P85)

A concrete, honest memory + execution profile for the `no_std` fixed-point core, so an embedded engineer can
size it before trying it. **Scope (non-claim):** this is a *static memory budget + a QEMU smoke run*, not a
real-time / WCET certification, not a controller, and not a safety-instrumented function. The core is
advisory, read-only, `#![forbid(unsafe_code)]`, **no heap**, and not claimed bit-identical to the edge float
pipeline (it is the embedded sibling grammar, calibrated independently).

## Memory model

Everything is stack- or statically-allocated with a **statically-known size**: the only state is a
const-generic ring buffer of width `N` (the drift window) plus a one-sample slew memory, an envelope, and a
one-state classifier. There is **no allocation** on any path, so there is no heap, no fragmentation, and no
allocation-failure mode. Engineering values are scaled integers (`round(value × SCALE)`, `SCALE = 1_000_000`)
held in `i64`; the only widening is a transient `i128` in the grazing-band comparison (overflow guard).

## Per-type footprint (nominal field sizes, 64-bit `i64`/`usize`)

| Type | Composition | Bytes |
|---|---|---|
| `FixedTriple` | `r, delta, sigma` (3 × `i64`) | 24 |
| `FixedEnvelope` | `r_min/r_max/delta_min/delta_max/sigma_min/sigma_max/band_scaled` (7 × `i64`) | 56 |
| `RingBuffer<N>` | `buf: [i64; N]` + `len, head: usize` + `sum: i64` | **8·N + 24** |
| `GrammarClassifier` | `prev_state` (1-byte enum, aligned) | 8 |
| `DsfbCore<N>` | `FixedEnvelope` + `RingBuffer<N>` + `prev_r: Option<i64>` (16) + `GrammarClassifier` | **8·N + 104** |

So one monitored channel at the default window `N = 8` is **≈ 168 bytes** (`56 + 88 + 16 + 8`); `N = 16` is
≈ 232 bytes, `N = 32` ≈ 360 bytes. (These are field-sum sizes; the compiler may pad by a few bytes for
alignment. The point is the **sub-kilobyte-per-channel** order of magnitude.)

### Worked budget on the QEMU smoke target (`lm3s6965evb`, Cortex-M3)

`memory.x`: **FLASH 256 KiB**, **RAM 64 KiB**.

| Channels (`N=8`) | RAM for cores | % of 64 KiB |
|---|---|---|
| 1 | ~168 B | 0.3 % |
| 16 | ~2.6 KiB | 4 % |
| 64 | ~10.5 KiB | 16 % |
| 256 | ~42 KiB | 64 % |

A realistic plant unit (tens of channels) sits in **single-digit kilobytes**, leaving the bulk of RAM for the
application. The processing per sample is a handful of integer add/sub/compare + one integer divide (the
windowed mean) + one `i128` multiply (the grazing test) — no FPU, no division in the hot envelope test.

## Overflow / panic policy

- `#![forbid(unsafe_code)]`, `#![cfg_attr(not(test), no_std)]`, **no `alloc`** — fixed-capacity buffers only.
- Envelope classification works in doubled integer coordinates; the grazing-band check promotes to `i128`
  to avoid 64-bit multiply overflow, then compares — exact, no rounding.
- `panic = "abort"` for the embedded/release profile (no unwinding machinery; smaller image).
- A bad reading (the fixed-point analogue of a non-finite float) is signalled by `valid = false` → the core
  emits `SensorFault` and leaves the ring / previous-sample / classifier memory untouched (no poisoning).

## Reproduce (QEMU Cortex-M3 smoke run)

```fish
rustup target add thumbv7m-none-eabi
cd crates/dsfb-chemical-engineering-core/qemu-smoke
cargo run --release   # builds for thumbv7m-none-eabi + launches qemu-system-arm (lm3s6965evb) via .cargo/config.toml
```

Verified output (a fixed residual sequence quiet → spike → recovery fed through `DsfbCore::<8>`, emitting one
grammar token per sample via semihosting, then a checksum and a clean QEMU exit):

```
DSFB-core QEMU smoke (Cortex-M3, no_std, fixed-point):
  step 0 r_scaled=100000 -> NOM
  step 1 r_scaled=-100000 -> NOM
  step 2 r_scaled=2500000 -> NOM
  step 3 r_scaled=2500000 -> NOM
  step 4 r_scaled=2500000 -> NOM
  step 5 r_scaled=8000000 -> EV
  step 6 r_scaled=0 -> CP
  step 7 r_scaled=-100000 -> DA
  step 8 r_scaled=0 -> DA
  step 9 r_scaled=50000 -> DA
token-checksum=15794140667410667713
OK
```

The deterministic `token-checksum` (`15794140667410667713`) is the smoke-run fingerprint — stable across runs
on the pinned toolchain. Build artifacts land in the shared root `target/qemu-smoke/` (the crate's
`.cargo/config.toml` redirects `target-dir` there rather than nesting a `target/` inside the crate).
