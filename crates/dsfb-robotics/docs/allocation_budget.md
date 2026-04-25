# `dsfb-robotics` Allocation Budget

This document satisfies NASA/JPL Power-of-Ten Rule 3 (*no dynamic
allocation after initialization*) for the `dsfb-robotics` crate.
P10-3 is reported as `not applied` by `dsfb-gray` because the crate
ships a feature-gated CLI (`paper-lock`) that legitimately allocates
strings and `Vec<f64>` for JSON serialisation. This document is the
**formal exemption record**: every flagged allocation site is listed
here with its purpose, lifetime, and steady-state behaviour.

## Architectural split

The crate has two distinct surfaces:

| Surface | Built when | Allocation policy |
|---|---|---|
| **Core engine** (`engine.rs`, `sign.rs`, `grammar.rs`, `envelope.rs`, `episode.rs`, `math.rs`, `kinematics.rs`, `balancing.rs`, plus all `datasets/*.rs` adapters) | Always (default, `no_std` + `no_alloc`) | **Zero heap allocation.** Stack-only; verified by `cargo build --no-default-features --target thumbv7em-none-eabihf` and `riscv32imac-unknown-none-elf` succeeding (see [`scripts/build_embedded.sh`](../scripts/build_embedded.sh)). |
| **CLI driver** (`paper_lock.rs`, `main.rs`) | `--features paper_lock` (which implies `std + serde + serde_json`) | Bounded initialisation-time allocation; no steady-state allocation in the streaming hot path. Steady-state operation lives entirely inside the no-alloc engine. |

The `dsfb-gray` scanner cannot mechanically distinguish initialisation
allocations from steady-state allocations, so it flags the CLI as
`P10-3: not applied`. This document **declares the steady-state
allocation budget** and provides a Valgrind-massif protocol for empirical
verification.

## Steady-state allocation budget

For a single `paper-lock <slug>` invocation (release build, no `--emit-episodes`):

| Site | Allocation | Lifetime | Why this is initialisation, not steady-state |
|---|---|---|---|
| [`paper_lock.rs:373`](../src/paper_lock.rs) | `String` for raw CSV contents (`std::fs::File::read_to_string`) | per-run, freed at end | One-shot read of `data/processed/<slug>.csv` at startup; not on the hot per-sample path. |
| [`paper_lock.rs:387`](../src/paper_lock.rs) (parser) | `Vec<f64>` for parsed residual stream | per-run, freed at end | One reservation sized to the input length, populated once. The streaming engine consumes the slice without further allocation. |
| [`paper_lock.rs:262`](../src/paper_lock.rs) (`calibrated_envelope`) | `Vec<f64>` of length `cal_len` (≤ 20% of input) | per-run, freed when the envelope is computed | Initialisation buffer for the Stage~III calibration window. Not touched after the envelope is fixed. |
| [`paper_lock.rs:192`](../src/paper_lock.rs) (`build_report`) | `Vec<Episode>` of length `n` (input residual count) | per-run, freed when JSON is emitted | The output buffer the engine writes into. Sized once at run start. The engine never reallocates this buffer; the Power-of-Ten 3 spirit (no growth, no churn) is preserved. |
| [`paper_lock.rs:439`](../src/paper_lock.rs) (`serialize_report`) | `String` for pretty-printed JSON | per-run, freed when written to stdout | Single allocation by `serde_json::to_string_pretty`; size proportional to output JSON. |
| [`main.rs:60`](../src/main.rs) | `Vec<String>` for `argv` (length capped at 256 by `take(256)`) | process-lifetime | OS argv collection; bounded by the explicit `take(256)` per ITER-UNB rule. |
| [`main.rs:261`](../src/main.rs) (`alloc_format`) | small `String` for error messages | per-error | Pre-sized via `String::with_capacity(len)` from the joined-piece total. No growth. |

**No site allocates inside the per-sample streaming loop.** The
streaming hot path is `DsfbRoboticsEngine::observe(&mut self,
residuals: &[f64], out: &mut [Episode], ...)` which is `no_std` +
`no_alloc` by construction (verified by the `thumbv7em-none-eabihf`
build).

## Empirical verification protocol

A reviewer can verify the steady-state-allocation claim with Valgrind massif:

```bash
cd crates/dsfb-robotics
cargo build --release --features std,paper_lock --bin paper-lock
valgrind --tool=massif --pages-as-heap=yes \
    --massif-out-file=audit/allocations/panda_gaz.massif \
    target/release/paper-lock panda_gaz > /dev/null
ms_print audit/allocations/panda_gaz.massif | head -100
```

Expected shape: three or four step increases during initialisation
(file read, residual parse, calibration buffer, episode buffer
allocation, serde JSON output buffer), then a flat plateau, then a
final freeing step at process exit. **No upward staircase during the
streaming-loop section** — that would be the P10-3 violation
the rule guards against.

The audit directory `audit/allocations/` is committed empty; the
Valgrind step is reproducer-side because Valgrind is a heavy
dependency we don't pin in CI.

## Why we don't refactor to remove the CLI allocations

Removing all allocation from `paper_lock.rs` would require:

1. Streaming the input CSV one f64 at a time without a full `Vec`
   (possible but loses the simple per-run determinism guarantee).
2. Streaming the JSON output one field at a time without a full
   `String` (would require a hand-rolled JSON serialiser; serde-json
   does not support no-alloc emission).
3. Stack-allocating the episode buffer (would require a const-generic
   maximum sample count baked into the binary, defeating the dataset-
   agnostic design).

All three changes would compromise either reproducibility or
ergonomics for marginal gain. The core engine remains rigorously
zero-allocation; the CLI's bounded initialisation-time allocations
are the right trade-off for the read-only-observer-tool surface.

## Cross-reference

- `dsfb-gray` finding `P10-3` evidence list: see the SARIF output at
  `audit/dsfb-gray-*/dsfb_robotics_scan.sarif.json` after running the
  audit.
- Power-of-Ten Rule 3 source: Holzmann, G.J. (2006). *The Power of
  Ten — Rules for Developing Safety Critical Code*. IEEE Computer 39(6):
  95–97.
- Companion `feature_matrix.md` documents the `cfg(feature = ...)`
  surface that gates the CLI allocations behind the optional
  `paper_lock` feature.
