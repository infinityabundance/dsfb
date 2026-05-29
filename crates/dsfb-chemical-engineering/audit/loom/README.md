# loom — concurrency permutation testing (applicability assessment: N/A by design)

[`loom`](https://github.com/tokio-rs/loom) is a model checker for **concurrent** Rust: it exhaustively permutes
thread interleavings and the C11 memory-ordering choices around `Arc`/`Mutex`/`RwLock`/atomics to surface data
races, deadlocks, and ordering bugs that ordinary tests miss. It tests code *that has shared-state concurrency*.

## Honest finding: DSFB-Chemical has no in-scope concurrency surface

The DSFB pipeline is **deterministic and single-threaded by construction** — that determinism is a load-bearing
property (the verify-replay digest gate and the `no_std` embedded profile both depend on it). A scan of all
first-party crate sources finds **zero** shared-state concurrency primitives:

```fish
grep -rEl "std::thread|std::sync::(Mutex|RwLock|Arc)|Atomic[UIB]|rayon|crossbeam|tokio::spawn|async fn" crates/*/src
# → (no matches)
```

There is no `Arc`, `Mutex`, `RwLock`, atomic, `thread::spawn`, `async`/`await`, `rayon`, or `crossbeam` anywhere in
`edge`, `cuda` (host side), `atlas`, `corpus`, or `core`. loom has **nothing to permute** — not because the audit
was skipped, but because the design admits no data race: no shared mutable state crosses a thread boundary.

This is reported as a **positive structural property**, the same way `cargo-geiger`'s `unsafe` count is: the
absence of a concurrency surface means the entire class of bugs loom hunts (races, deadlocks, torn reads, ordering
violations) is **structurally unreachable** in the current code, with no runtime cost or scheduler dependence.

## What this does NOT certify
- It does not certify that DSFB *could never* become concurrent. The GPU path (`--features cuda`) runs work on the
  device, but the host orchestration is sequential and the device↔host contract is the byte-exact `evidence_root`
  digest, not shared memory — so loom (a *CPU thread-interleaving* checker) would not model it regardless.
- It is an applicability assessment, not a loom run: there is no loom test because there is no concurrent unit to
  put under one.

## If concurrency is ever introduced (the standing recipe)
Should a future change add a shared-state primitive (e.g. a parallel multi-channel residual stage), wrap the type's
imports behind `#[cfg(loom)] use loom::sync::...;` and add a `#[test]` under `loom::model(|| { ... })`, then:

```fish
RUSTFLAGS="--cfg loom" cargo test --test loom_models --release
```

Until then, this folder documents the deliberate single-threaded posture as the honest result.
