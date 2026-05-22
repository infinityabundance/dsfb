# dsfb-gpu-debug-core

Deterministic CPU reference and semantic authority for the
`dsfb-gpu-debug` stack.

This crate owns the non-GPU parts that must remain auditable:
fixed-point arithmetic, canonical trace/event structures, SHA-256
hashing, detector motifs, bank-governed episode admission, and
case-file construction. The CUDA crate may produce evidence bytes, but
this crate decides what can be admitted into a replayable case file.

## Scope

- Default build is `no_std` and dependency-free.
- The `std` feature enables allocating pipeline drivers and case-file
  emission.
- The `demo` feature enables additional support used by the CLI crate.

## Non-claims

This crate is not a neural inference framework, a probabilistic model,
or a performance claim. It is the deterministic reference path used to
check byte identity and semantic non-bypass behavior.

## Publish order

Publish this crate first. The other `dsfb-gpu` crates depend on
`dsfb-gpu-debug-core = 0.1.0`.
