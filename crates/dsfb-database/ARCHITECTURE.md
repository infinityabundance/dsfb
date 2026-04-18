# Architecture

`dsfb-database` implements the `telemetry → residuals → motif episodes →
figures/reports` pipeline for SQL database observability. This document
describes the module layout and the data-flow contract between layers.

## High-Level Data Flow

```
             ┌───────────────────────────────────────────────────┐
raw trace ──▶│  adapters/ (Snowset, SQLShare, CEB, JOB, TPC-DS)  │──▶ ResidualStream
             └───────────────────────────────────────────────────┘
                                   │
                                   ▼
             ┌───────────────────────────────────────────────────┐
             │       residual/ (five structural channels)        │
             │   plan_regression, cardinality, contention,       │
             │       cache_io, workload_phase                    │
             └───────────────────────────────────────────────────┘
                                   │
                                   ▼
             ┌───────────────────────────────────────────────────┐
             │       grammar/ (five motif state machines)        │
             │   drift/slew thresholds + minimum-dwell rule      │
             └───────────────────────────────────────────────────┘
                                   │
                                   ▼
             ┌───────────────────────────────────────────────────┐
             │   report/ (PNG overlays, CSV episodes, summary)   │
             └───────────────────────────────────────────────────┘
```

Every arrow carries immutable data by value or by `&`. No mutable reference
crosses a module boundary.

## Module Contracts

### `src/adapters/*`

- **Input:** a `&Path` to an operator-supplied dump plus any CLI
  configuration. Adapters never issue database queries; they read pre-extracted
  CSVs, JSON, or YAML.
- **Output:** a fully-populated `ResidualStream` whose samples are sorted by
  monotonic time and tagged with one of the five `ResidualClass` variants.
- **Forbidden:** panic on malformed input, network I/O, mutation of
  operator-supplied files.
- **Fingerprint sensitivity:** yes — any change to sample values, ordering,
  or channel labels will break `paper_*_fingerprint_is_pinned` tests.

### `src/residual/*`

- Five files, one per residual class. Each owns its construction formula and
  default thresholds.
- `mod.rs` exposes the `ResidualSample`, `ResidualClass`, and
  `ResidualStream` types and the cross-class iterator.
- **Fingerprint sensitivity:** high.

### `src/grammar/*`

- `motifs.rs`: the five motif state machines, each an explicit
  `enum MotifState` with `advance(&mut self, sample) -> Option<Transition>`.
- `envelope.rs`: admissibility envelopes (drift / slew pairs) used to
  classify residuals as admissible, drift-violating, or slew-violating.
- `mod.rs`: `Episode` and `MotifEngine`; the engine produces episodes from
  a residual stream using the state machines.
- `replay.rs`: deterministic replay harness used by
  `tests/deterministic_replay.rs`.
- **Fingerprint sensitivity:** highest. The `Episode` stream is the object
  hashed by `paper_episode_fingerprint_is_pinned`.

### `src/report/*`

- `plots.rs`: all PNG emission helpers. Functions return `Result<bool>` so
  callers can skip silently when the data cannot support an honest figure
  (see the figure-quality remediation note in
  `/home/one/.claude/plans/steady-singing-pine.md`).
- `mod.rs`: CSV and summary-report writing.
- **Fingerprint sensitivity:** none. PNG bytes are not hashed.

### `src/metrics.rs`

- Precision/recall evaluation against ground-truth perturbation windows
  (TPC-DS tier only).
- Never called on real-world runs; ground truth is not available.

### `src/perturbation/`

- Deterministic TPC-DS perturbation generator. Seeds are CLI-configurable
  and embedded in artefact provenance.

### `src/non_claims.rs`

- Verbatim-locked array of strings stating what the crate does *not* claim.
- Tested by `tests/non_claim_lock.rs` to prevent accidental drift.

### `src/main.rs`

- CLI surface (`clap`-derived). Three subcommands:
  - `reproduce` — regenerate the TPC-DS controlled-perturbation tier
  - `run` — ingest a real-world dataset via one of the adapters
  - `exemplar` — emit a small demonstration artefact bundle

## Cross-Cutting Invariants

1. **Determinism.** All randomness is sourced from `rand_pcg::Pcg64` seeded
   from the CLI. `Instant::now()` is used only in throughput reports that
   are explicitly labelled non-deterministic; it is not used for sample
   timestamps.
2. **No shared mutable state.** The runtime-core modules have zero `static
   mut`, zero `lazy_static!`, and zero interior mutability (`RefCell`,
   `Cell`, `Mutex`, `RwLock`).
3. **Forbidden-unsafe.** `#![forbid(unsafe_code)]` at both crate roots.
4. **Bounded loops.** Every `for` loop iterates over a sized collection or
   a fixed numeric range.

## Evaluation Harness

Pinned tests under `tests/` guard the architecture:

| Test                                      | What it locks                                                     |
|-------------------------------------------|-------------------------------------------------------------------|
| `tests/deterministic_replay.rs`           | residual and episode streams (SHA-256)                            |
| `tests/non_claim_lock.rs`                 | non-claim block text verbatim                                     |
| `tests/adapter_roundtrip.rs`              | CSV adapters parse and produce residuals without error            |
| `tests/spec_validation.rs`                | YAML spec round-trips through the grammar                         |
| `tests/property_threshold.rs`             | proptest: drift/slew threshold invariants                         |
| `tests/stress_sweep.rs`                   | deterministic stress-scale regression                             |

See also [SAFETY.md](SAFETY.md) for the safety posture and
[SECURITY.md](SECURITY.md) for the security threat model.
