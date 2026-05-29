# Flux — refinement types for Rust (INSTALLED + RUN in-sandbox)

[Flux](https://github.com/flux-rs/flux) adds **refinement types** (liquid types) to Rust: you annotate signatures
with logical predicates and Flux discharges them at compile time with an SMT solver. Compile-time, automatic (no
manual proof), catching index-out-of-bounds, arithmetic-range, and precondition violations as **type errors**.

## What was actually done (2026-05-27)
Flux was **installed and run in this sandbox** (it is NOT a handoff):
```fish
git clone --depth 1 https://github.com/flux-rs/flux; and cd flux; and cargo xtask install   # installs cargo-flux + flux
cd crates/dsfb-chemical-engineering-core; and cargo flux                                     # run it
```
Installed version: `cargo-flux 4d329f2 (2026-05-23)`. Real captured output: [`run_core.txt`](run_core.txt) —

```
    Checking dsfb-chemical-engineering-core v0.1.0
    Finished `flux` profile [unoptimized + debuginfo] target(s)
```

**Result: Flux checks the `no_std` core crate with 0 refinement errors** — the crate type-checks under the Flux
refinement checker as-is. This is a real run on the real authority crate, not a scaffold.

## Honest scope of this result
The clean check is meaningful but *minimal*: the core carries no `#[flux::sig]` / `#[spec]` refinement annotations
yet, so Flux verified only the base typing, not added refinements. The next, stronger step (genuine future work, not
a blocker) is to refine the core's integer invariants and have Flux prove them — e.g. `band_scaled ∈ [0, SCALE)`,
the `RingBuffer<N>` write index `< N`, and `classify_axis`'s `hi > lo` precondition. The annotation form (from the
Flux test-suite) is:
```rust
use flux_attrs::*;
#[spec(fn(i: usize{i < N}) -> ...)]   // a checked refinement; a violation becomes a compile error
```
These are added behind `#[cfg_attr(flux, ...)]` so the normal/`no_std` build is unaffected.

## What it does NOT certify
Flux proves only the **refinements written** — an unrefined parameter carries no guarantee. It reasons about
decidable arithmetic/array fragments, not full functional correctness (that is Creusot/Lean/Coq). Float refinement
is limited, so the edge float path stays a Kani + cargo-fuzz target.
