# Safety Posture

`dsfb-database` is a **read-only, non-intrusive** observer over residual
trajectories in SQL database telemetry. This document names the safety
invariants the crate relies on and the review surface we commit to keep
auditable.

## Non-Interference Invariants

1. The crate never issues `INSERT`, `UPDATE`, `DELETE`, `DROP`, `TRUNCATE`,
   `ALTER`, or any DDL/DML against the observed system. All adapters consume
   pre-extracted dumps, CSV exports, or immutable file handles.
2. The crate never opens a network connection. File I/O is local, and only
   `std::fs` read paths are used on the observation side. Figure writes go to
   an operator-chosen `out/` directory.
3. The crate never forks a child process. Tooling scripts (Colab, paper
   regeneration) are separate entry points outside the crate's binary.
4. The crate does not spawn worker threads. Control flow is single-threaded;
   `rayon` and `tokio` are intentionally absent from the dependency graph.

## Memory & Control-Flow Invariants

1. `#![forbid(unsafe_code)]` is declared at the root (`src/lib.rs`) and the
   binary (`src/main.rs`). Any `unsafe` block is a compile-time error.
2. No FFI: the crate has zero `extern "C"` sites, zero `#[repr(C)]` types,
   and no `bindgen`. The transitive dependency graph is audited quarterly for
   surprise `unsafe`.
3. Panics on control paths are treated as bugs. `.expect(...)` is permitted
   only when accompanied by a comment naming the mathematical invariant that
   makes the extraction total.
4. Loop bounds: all `for` loops iterate over bounded iterators (`.iter()`,
   `in 0..N`, `.enumerate()`, etc.) or over finite adapter inputs.
5. Allocation: per-call allocation is bounded by input size. Steady-state
   processing inside the motif state machine (`src/grammar/motifs.rs`) does
   no heap allocation after construction.

## Determinism Invariants

Deterministic reproducibility is a safety property of this crate, because
the paper's claims are pinned against hashed streams and would detect silent
drift in parser or grammar logic.

1. Residual streams (`ResidualSample` tuples) produced from a fixed seed and
   input are bit-identical across runs on the same toolchain. Verified by
   `tests/deterministic_replay.rs`.
2. Episode streams (`Episode` tuples) produced from a fixed seed are
   bit-identical. Verified by the four `paper_*_fingerprint_is_pinned`
   tests.
3. Non-claim block text is verbatim-locked. Verified by
   `tests/non_claim_lock.rs`.

## Review Surface

| Area                        | File                              | Review trigger                                                                 |
|-----------------------------|-----------------------------------|--------------------------------------------------------------------------------|
| Residual construction       | `src/residual/*.rs`               | Any change requires running `cargo test --release --test deterministic_replay` |
| Motif state machine         | `src/grammar/motifs.rs`           | State transitions must be explicit (no `_ =>` catch-alls)                      |
| Adapter parsing             | `src/adapters/*.rs`               | Any change requires rerunning fingerprint locks                                |
| Display layer               | `src/report/plots.rs`             | Free to iterate — no fingerprint dependency                                    |
| Non-claims & IP notice      | `src/non_claims.rs`, `Cargo.toml` | Verbatim-locked; review required to change                                     |

## Out-of-Scope Hazards

- Real-time guarantees. The crate is not designed for WCET-bounded execution.
- Safety-of-life or regulated domains (DAL, SIL, ASIL). The crate is a
  research observer; no certification claim is made.
- Adversarial host environments. The crate trusts the local filesystem and
  the Rust toolchain.
