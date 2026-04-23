# Architecture — `dsfb-rf`

This document sketches the internal module graph of the crate and the
data flow through a single observation step. It exists so a new reviewer
can orient themselves before diving into `src/`.

## Positioning

DSFB is **not** a detector, classifier, or replacement for existing RF
processing chains. DSFB is an **observer** that structures the residual
streams an upstream producer (matched filter, AGC, PLL, channel
estimator, tracking loop, beamformer, scheduler telemetry) has already
computed and usually discards. The engine consumes that residual and
emits a human-readable grammar (state, sign-tuple, DSA score, episode,
envelope judgment) that augments the producer's output.

## Module Graph

```
                        ┌─────────────────────┐
                        │  caller: upstream   │
                        │  RF chain           │
                        │  (AGC / PLL / …)    │
                        └──────────┬──────────┘
                                   │ residual stream r(k)
                                   ▼
              ┌────────────────────────────────────┐
              │  src/engine.rs  DsfbRfEngine       │
              │  - holds calibration + state       │
              │  - single `observe(n, ctx)` entry  │
              └──────────┬────────────────┬────────┘
                         │                │
               ┌─────────▼──────────┐ ┌──▼─────────────┐
               │ src/sign.rs         │ │ src/envelope.rs │
               │ SignTuple           │ │ Admissibility   │
               │ (‖r‖, ṙ, r̈)         │ │ bound, ρ        │
               └─────────┬──────────┘ └──┬─────────────┘
                         ▼                ▼
                    ┌─────────────────────────┐
                    │ src/grammar.rs          │
                    │ Grammar FSM K=4         │
                    │ Admissible / Boundary / │
                    │ Violation               │
                    └──────────┬──────────────┘
                               ▼
                    ┌─────────────────────────┐
                    │ src/dsa.rs              │
                    │ DSA score + EWMA        │
                    └──────────┬──────────────┘
                               ▼
                    ┌─────────────────────────┐
                    │ src/pipeline.rs         │
                    │ Stage III calibration   │
                    │ + episode tracking      │
                    └──────────┬──────────────┘
                               ▼
                        caller consumes
                        observation result
```

All side-car modules — `complexity`, `lyapunov`, `tda`, `attractor`,
`fisher_geometry`, `detectability`, `stationarity`, `uncertainty`,
`rg_flow`, `quantum_noise`, `impairment`, `physics`, `dna`, `disturbance`,
`pragmatic`, `calibration`, `swarm_consensus` — are additive analyses over
the same residual; none are required for the core observer path.

## Determinism Invariants

1. The grammar FSM is deterministic in `(sig, envelope, waveform_state)`.
2. The DSA accumulator is deterministic in `(sig, state, motif_fired)`.
3. The envelope threshold `ρ` is computed once from the healthy-window
   pass and is immutable thereafter.
4. All fixed-point math is Q16.16 with bounded saturating intermediates.
5. No floating-point reductions cross platform boundaries except via the
   typed `SignTuple`/`AdmissibilityEnvelope` types.

## Feature Surface

| Feature         | Adds                                        |
|-----------------|---------------------------------------------|
| default         | `no_std` core observer                      |
| alloc           | Vec-backed dev-helpers                      |
| std             | `alloc` + std formatting + I/O helpers      |
| serde           | `std` + Serialize/Deserialize derives       |
| paper_lock      | `serde` + Table 1 bit-exact gate binary     |
| hdf5_loader     | `paper_lock` + RadioML HDF5 reader          |
| real_figures    | `serde` + `hdf5_loader` + `csv` — 80-figure |
|                 | real-dataset figure bank                    |
| experimental    | Research-stage modules excluded from Table 1|

The default build surface is unchanged from v1.0.0 consumers. Everything
above `default` is opt-in.

## External Boundaries

- **Only native boundary**: `hdf5-metno` (system `libhdf5`), gated behind
  `hdf5_loader`. Used exclusively by the `paper-lock` calibration
  binary, never by library consumers.
- **No network, no filesystem, no syscalls in core library code**.
  Serialisation (`serde`, `csv`) happens only under `std`-enabled
  feature flags on typed data the caller provides.

## Tree Map

```
crates/dsfb-rf/
├── src/                     library source (frozen at v1.0.0)
├── examples/                demo binaries; not part of library surface
├── tests/                   integration tests + proptest harnesses
├── fuzz/                    cargo-fuzz harness stubs
├── benches/                 Criterion benchmarks
├── scripts/                 Python figure renderers + slice preparation
├── colab/                   Colab reproducibility notebook
├── data/slices/             small real-dataset slices (≤ 2 MB each)
├── docs/                    design notes and API preconditions
├── paper/                   LaTeX sources (local-only, gitignored)
└── gr-dsfb/                 GNU Radio out-of-tree module
```
