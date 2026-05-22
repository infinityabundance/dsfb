[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu-debug-core

`dsfb-gpu-debug-core` is the semantic authority layer for DSFB-GPU:
the part that makes acceleration auditable instead of magical. CUDA may
produce deterministic witness bytes, but this crate defines the numeric
domain, the canonical stage records, the SHA-256 hash chain, the
detector motifs, the bank admission rule, and the replayable case-file
surface. If a verdict is admitted, it passes through this crate.

The design goal is narrow and evidence-first: produce byte-stable
deterministic artifacts that can be replayed, compared, and challenged.
It is not a learned model, not a probability engine, and not a benchmark
claim.

## What

This crate provides the host reference path and the public data types for
the DSFB-GPU debug pipeline:

- `Q16`: signed Q16.16 fixed-point arithmetic with saturating integer
  operations and deterministic round-half-to-even multiplication.
- `TraceEvent`, `WindowFeature`, `ResidualCell`, `SignCell`,
  `DetectorCell`, `ConsensusCell`, and `CandidateInterval`: canonical
  stage records.
- `sha256` and `Sha256`: a dependency-free SHA-256 implementation used
  by the case-file and Atlas crates.
- `CANONICAL_BANK`: the bank-governed episode admission surface.
- `CaseFile` and `IntermediateHashes`: the replayable verdict record and
  the stage-by-stage hash chain.

## Where

This crate lives at `crates/dsfb-gpu/crates/dsfb-gpu-debug-core` in the
[DSFB repository](https://github.com/infinityabundance/dsfb). It is the
lowest dependency in the DSFB-GPU crate set:

- [`dsfb-gpu-debug-cuda`](https://crates.io/crates/dsfb-gpu-debug-cuda)
  mirrors the evidence-production path on CUDA.
- [`dsfb-gpu-debug-demo`](https://crates.io/crates/dsfb-gpu-debug-demo)
  exposes the CLI that runs and compares CPU/GPU case files.
- [`dsfb-gpu-atlas-corpus`](https://crates.io/crates/dsfb-gpu-atlas-corpus)
  reuses the audited SHA-256 code for corpus hashes.
- [`dsfb-gpu-atlas-registry`](https://crates.io/crates/dsfb-gpu-atlas-registry)
  reuses the same hash surface for registry hashes.

The public Colab notebook is a replay surface for the wider DSFB-GPU
audit gauntlet, not a substitute for reading the crate contracts.

## Why

The core problem is not "can a GPU compute faster?" The problem is
"can accelerated evidence be admitted without losing semantic custody?"
This crate keeps the final authority on the CPU side. The GPU can emit
residuals, detector witnesses, candidate summaries, and digests; it
cannot mint an admitted `Episode`. Episode admission is guarded by a
bank-private token, so semantic bypass is a type-system boundary rather
than a convention.

## Mathematical Contract

The numeric carrier is Q16.16:

```text
raw(q) = integer stored in i32
value(q) = raw(q) / 2^16
```

Residual latency uses the explicit conversion:

```text
residual_latency_q_raw =
  ((mean_latency_us - baseline_latency_us) * 65536) / 1000
```

with `i64` widening, truncation toward zero on the integer divide, and
final saturation to `i32`. The sign stage then computes:

```text
norm_w  = |residual_latency_w| + |residual_error_w|
drift_w = drift_{w-1} + alpha * (norm_w - drift_{w-1})
slew_w  = norm_w - norm_{w-1}
```

where `alpha` is contract-locked as a raw Q16.16 value. Detector cells
are closed-form threshold decisions over residual/sign grids. Consensus
keeps only the GPU-admissible axes: residual magnitude, drift
persistence, slew shock, temporal locality, and detector consensus.
Entity locality, topology, semantic admissibility, and confuser
suppression remain bank-side.

The case-file chain is:

```text
H_0 = SHA256(input_catalog_bytes)
H_i = SHA256(label_i || material_i || H_{i-1})
```

for contract, bank, detector registry, kernel sequence, window,
residual, sign, detector, consensus, candidate, and episode material.
Any byte change in an earlier stage cascades into the final case-file
hash.

## Code

```rust
use dsfb_gpu_debug_core::{
    build_cpu, fixture, Contract, Q16,
};

let _one = Q16::ONE;
let events = fixture::synthesize(fixture::DEFAULT_SEED);
let contract = Contract::canonical();
let case_file = build_cpu(&events, &contract);
```

Run the crate-level checks from `crates/dsfb-gpu`:

```sh
cargo test -p dsfb-gpu-debug-core --features std
```

The default build is `no_std` and dependency-free. The `std` feature
enables allocating pipeline drivers and case-file emission; `demo`
enables support used by the CLI crate.

## Claim Boundary

This crate establishes deterministic reference semantics and replayable
case-file construction. It does not claim neural inference, statistical
prediction, probabilistic confidence, learned usefulness, medical or
safety diagnosis, or production CUDA performance.

## Publish Order

Publish this crate first. The other DSFB-GPU crates depend on
`dsfb-gpu-debug-core = 0.1.1`.

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
