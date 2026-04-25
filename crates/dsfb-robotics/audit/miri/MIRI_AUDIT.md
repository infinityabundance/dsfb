# Miri audit — dsfb-robotics

This document describes the three Miri configurations the crate is
audited under, the rationale for each, and the acceptance criterion
(all three must be clean — no undefined behaviour reports, no
alias-model violations). Reports go alongside this file as
`miri_<config>.txt`.

Miri is a Rust MIR interpreter with instrumentation for detecting
undefined behaviour, strict provenance violations, and memory-model
infractions. It complements Kani (symbolic bounded verification) and
cargo-fuzz (random-input stress) by running the test suite under a
UB-detecting interpreter.

## Configurations

### 1. `miri_nostd_strict.txt` — `no_std` + strict provenance

```bash
MIRIFLAGS="-Zmiri-strict-provenance" \
  cargo +nightly miri test \
    --manifest-path crates/dsfb-robotics/Cargo.toml \
    --no-default-features \
    --lib
```

Rationale: audits the `no_std` + `no_alloc` core under the strictest
provenance model (no int-to-pointer casts). Load-bearing because the
core is the surface consumers compile against on bare-metal targets
(Cortex-M4F, RISC-V 32-bit) without an allocator. Strict provenance
is a forward-compatibility signal for future compiler changes.

### 2. `miri_std_stacked.txt` — `std + serde` stacked borrows

```bash
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation" \
  cargo +nightly miri test \
    --manifest-path crates/dsfb-robotics/Cargo.toml \
    --features std,serde \
    --lib
```

Rationale: audits the host-side `std + serde` build under the default
(stacked-borrows) alias model. Exercises the JSON-serialisation paths
of `PaperLockReport`, `Aggregate`, and the per-dataset adapters. The
isolation flag is disabled so serde-json's filesystem-less code paths
can run.

### 3. `miri_std_tree.txt` — `std + serde` tree borrows

```bash
MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-disable-isolation -Zmiri-tree-borrows" \
  cargo +nightly miri test \
    --manifest-path crates/dsfb-robotics/Cargo.toml \
    --features std,serde \
    --lib
```

Rationale: same test surface as config 2 but under the stricter
tree-borrows alias model (a tighter specification than stacked
borrows). This catches a class of reborrowing patterns that pass
stacked borrows but would be UB under the stricter model. Passing
config 3 is evidence that the crate is ready for the eventual Rust
alias-model tightening.

## Acceptance

- All three configurations must be clean — no `error: Undefined
  Behavior` lines in any `miri_*.txt`.
- Each report must contain a trailing `test result: ok.` line for the
  configuration's feature matrix.
- `RUN_MANIFEST.json` must list all three with SHA-256 hashes so
  reviewers can confirm they are the committed reports.

## Why single-threaded is enough

DSFB has no `Arc`, no `Mutex`, no atomic, no `unsafe`, and no
interior mutability. The observer API is `&mut self, &[f64], &mut
[Episode]` — single-owner. Miri's thread-scheduling exploration adds
no coverage over its single-threaded UB detection for this crate;
loom (`tests/concurrency_observer.rs`) is the right tool for
concurrency hazards if any are ever introduced.

## Re-running

Use `scripts/run_audit.sh`; it invokes all three Miri configurations
sequentially and tees output here. Miri is slow (≈ 10× stock test);
expect a full run to take several minutes.
