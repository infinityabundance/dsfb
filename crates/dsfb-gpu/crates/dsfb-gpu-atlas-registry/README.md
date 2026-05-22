[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu-atlas-registry

`dsfb-gpu-atlas-registry` is the deterministic detector algebra and
S1.2 registry generator for DSFB-GPU-Atlas. It turns a frozen corpus
anchor into explicit `DetectorSpec` records by applying a bounded,
auditable parameter grid. The result is registry authority, not runtime
execution and not an empirical usefulness claim.

This crate is host-only. It does not link CUDA, execute detector
kernels, or admit episodes. It says which detector specifications are
well-formed, corpus-bound, and hashable for later execution surfaces.

## What

This crate provides:

- detector algebra types for family, transform, window, statistic,
  comparator, gate, parameterization, numeric mode, implementation kind,
  and corpus binding;
- deterministic canonical detector names;
- the S1.2 generator over the T.10-frozen corpus anchor;
- `registry_hash_v2` over canonical registry bytes;
- verifiers that reject malformed specs and stale corpus bindings;
- the `dsfb-registry-emit` binary for canonical registry artifacts.

## Where

This crate lives at `crates/dsfb-gpu/crates/dsfb-gpu-atlas-registry` in
the [DSFB repository](https://github.com/infinityabundance/dsfb). It
depends on:

- [`dsfb-gpu-atlas-corpus`](https://crates.io/crates/dsfb-gpu-atlas-corpus)
  for the live `corpus_hash_v1` and canonical corpus IDs;
- [`dsfb-gpu-debug-core`](https://crates.io/crates/dsfb-gpu-debug-core)
  for the audited SHA-256 implementation.

The execution-side crates are
[`dsfb-gpu-debug-cuda`](https://crates.io/crates/dsfb-gpu-debug-cuda)
and
[`dsfb-gpu-debug-demo`](https://crates.io/crates/dsfb-gpu-debug-demo).

## Why

A detector registry should be reproducible before it is accelerated. The
registry crate makes detector specification a deterministic algebra
instead of a loose list of names. Every generated spec carries the live
corpus hash, a canonical name, a bounded parameterization, and an
implementation-kind honesty marker. That gives later CUDA and case-file
layers a registry they can cite without pretending that registry
generation is the same as measured detector usefulness.

## Mathematical Contract

The S1.2 grammar is:

```text
Detector =
  LiteraturePrimitive
  x ParameterGrid
  x Transform
  x Window
  x Comparator
  x Gate
  -> DetectorSpec
```

At S1.2 the grid is deliberately small:

```text
54 corpus primitives x 3 grid points = 162 DetectorSpec records
```

The three grid points are:

```text
W32,  persistence=2, comparator=High
W64,  persistence=3, comparator=TwoSided
W128, persistence=5, comparator=TwoSided
```

Every generated spec carries:

```text
primitive_id = corpus canonical_id
corpus_binding_status = HashFrozenT10
source_corpus_hash = compute_corpus_hash_v1()
numeric_mode = Q16_16
implementation_kind = ScalarCpu
```

`registry_hash_v2` is SHA-256 over canonical registry bytes under its
own domain separator. The verifier rejects a registry-bound spec if its
source corpus hash does not match the live corpus hash.

## Code

Emit registry artifacts:

```sh
cargo run -p dsfb-gpu-atlas-registry --bin dsfb-registry-emit
```

Run tests:

```sh
cargo test -p dsfb-gpu-atlas-registry
```

Use the generator:

```rust
use dsfb_gpu_atlas_registry::{compute_registry_hash_v2, generate_s1_2_specs};

let specs = generate_s1_2_specs();
let registry_hash = compute_registry_hash_v2(&specs);
assert_eq!(specs.len(), 162);
```

## Claim Boundary

This crate spells and verifies registry authority. It does not execute
detector kernels, admit episodes, generate an unbounded detector count,
claim GPU implementation for generated specs, claim measured usefulness,
or claim production performance.

## Publish Order

Publish after both `dsfb-gpu-debug-core = 0.1.1` and
`dsfb-gpu-atlas-corpus = 0.1.1` are visible on crates.io.

## Citation

de Beer, R. (2026). DSFB-GPU: Clear-Box Pure Deterministic Inference
CUDA Acceleration for Replayable Trace-Event Verdicts A Prior-Art
Architecture for non-probabilistic, non-stochastic, non-weighted,
GPU-Accelerated Residual Signs, Detector Motifs, Bank-Governed Fusion,
and Byte-Exact Case Files Without Probabilistic Models (1.1). Zenodo.
https://doi.org/10.5281/zenodo.20346478

## IP Notice

DSFB-GPU
Copyright 2026 Invariant Forge LLC
This product includes software developed by Invariant Forge LLC.
Apache 2.0 (reference implementation).
Background IP: Invariant Forge LLC.
Commercial deployment requires separate written license.
Contact: licensing@invariantforge.net.
