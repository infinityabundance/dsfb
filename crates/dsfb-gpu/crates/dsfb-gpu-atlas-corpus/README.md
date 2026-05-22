[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu-atlas-corpus

`dsfb-gpu-atlas-corpus` is the literature court for DSFB-GPU-Atlas. It
is not a detector zoo and not a learned ranking system. It is a
deterministic, provenance-bound corpus and canonicalisation surface for
detector witnesses: what a detector is called, where it came from, how
aliases collapse, which constraints apply, and which claims remain
unmeasured.

The crate is host-only and has no external Rust dependencies. It reuses
the audited SHA-256 implementation from `dsfb-gpu-debug-core` and emits
versioned court artifacts whose hashes are defined over canonical bytes,
not over rendered prose.

## What

This crate provides:

- a T.10-frozen seed corpus and canonical detector IDs;
- deterministic source, formula, parameter, implementation, and
  semantic-role identity surfaces;
- a deduplication court with explicit decision variants and reason
  codes;
- genealogy, fusion, L-band, usefulness-ledger, passport,
  admissibility, challenge, contraindication, coverage-hole, activation,
  and proposal surfaces;
- `corpus_hash_v1`, the canonical hash anchor used by the registry
  crate.

## Where

This crate lives at `crates/dsfb-gpu/crates/dsfb-gpu-atlas-corpus` in
the [DSFB repository](https://github.com/infinityabundance/dsfb). It is
the Atlas corpus dependency for
[`dsfb-gpu-atlas-registry`](https://crates.io/crates/dsfb-gpu-atlas-registry)
and shares hash infrastructure with
[`dsfb-gpu-debug-core`](https://crates.io/crates/dsfb-gpu-debug-core).
The execution-side crates are
[`dsfb-gpu-debug-cuda`](https://crates.io/crates/dsfb-gpu-debug-cuda)
and
[`dsfb-gpu-debug-demo`](https://crates.io/crates/dsfb-gpu-debug-demo).

The Colab notebook exercises the DSFB-GPU replay/audit surface. This
corpus crate supplies court authority and anchors used by later Atlas
surfaces; it is not itself a CUDA runtime.

## Why

Detector acceleration without detector identity is weak evidence. The
Atlas corpus answers the prior question: which deterministic witness is
being invoked, under what name, with what provenance, with what aliases,
and under which claim boundary? The goal is not to inflate detector
counts. The goal is to make detector authority reviewable before it is
used by execution surfaces.

## Mathematical Contract

`corpus_hash_v1` is a SHA-256 commitment to a canonical byte projection
of the corpus court material. It is not the hash of TXT, JSON, or README
rendering. The material writer uses:

```text
domain = "DSFB-GPU-ATLAS:LITERATURE-CORPUS:v1\0"
strings = u32_be(length) || utf8_bytes
integers = big-endian fixed-width bytes
records = sorted by canonical_id
```

The projection includes canonical record fields, court decisions, source
references, genealogy edges, witness roles, fusion axes, L-band states,
lifecycle states, and usefulness-ledger rows. If the structural payload
changes, the hash changes. If only a rendered report changes, the hash
does not.

L-band is an honesty marker, not a quality score. Usefulness rows remain
unscored unless backed by named evidence. Proposal modules record
reviewable corpus amendments; they do not silently mutate the frozen
seed corpus.

## Code

Verify the corpus:

```sh
cargo run -p dsfb-gpu-atlas-corpus --bin dsfb-corpus -- verify
```

Run crate tests:

```sh
cargo test -p dsfb-gpu-atlas-corpus
```

Use the hash anchor:

```rust
use dsfb_gpu_atlas_corpus::corpus_hash::compute_corpus_hash_v1;

let corpus_hash = compute_corpus_hash_v1();
println!("{}", corpus_hash.to_hex());
```

## Claim Boundary

This crate records deterministic witness authority. It does not claim
learned detector usefulness, medical diagnosis, root-cause certainty,
production CUDA performance, benchmark portability, probabilistic
inference, or that every literature detector has been fully ratified for
every task.

## Publish Order

Publish after `dsfb-gpu-debug-core = 0.1.1` is visible on crates.io.
`dsfb-gpu-atlas-registry` depends on this crate.

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
