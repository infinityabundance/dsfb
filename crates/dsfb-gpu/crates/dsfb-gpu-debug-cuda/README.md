[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu-debug-cuda

`dsfb-gpu-debug-cuda` is the CUDA evidence factory for DSFB-GPU. It
exists to accelerate deterministic witness production while preserving
the CPU-side court boundary: kernels may compute stage bytes and
digests, but they do not admit episodes, assign final meaning, or bypass
the bank authority in `dsfb-gpu-debug-core`.

The default build is deliberately ordinary-host friendly. Without the
`cuda` feature, this crate compiles as a Rust shim and reports
`GpuError::CudaUnavailable` from GPU entry points. With `cuda`, `build.rs`
invokes `nvcc` and links the kernels under the workspace `cuda/`
directory.

## What

This crate provides:

- a small Rust FFI boundary around CUDA kernels;
- GPU dispatch for deterministic evidence-production stages;
- pinned-host and workspace abstractions used by throughput paths;
- explicit `GpuError` reporting for unavailable CUDA, kernel failure,
  and invalid inputs;
- non-CUDA stubs so CPU and Atlas crates can build without an NVIDIA
  toolkit.

## Where

This crate lives at `crates/dsfb-gpu/crates/dsfb-gpu-debug-cuda` in the
[DSFB repository](https://github.com/infinityabundance/dsfb). It depends
on [`dsfb-gpu-debug-core`](https://crates.io/crates/dsfb-gpu-debug-core)
for semantic types, numeric contracts, and case-file construction. The
CLI surface is
[`dsfb-gpu-debug-demo`](https://crates.io/crates/dsfb-gpu-debug-demo).
The Atlas court and registry crates are separate host-only surfaces:
[`dsfb-gpu-atlas-corpus`](https://crates.io/crates/dsfb-gpu-atlas-corpus)
and
[`dsfb-gpu-atlas-registry`](https://crates.io/crates/dsfb-gpu-atlas-registry).

## Why

DSFB-GPU uses CUDA as an acceleration mechanism, not as an oracle. The
GPU is allowed to produce evidence bytes that can be checked against the
CPU reference path. It is not allowed to decide what the evidence means.
That split is the point: high-throughput evidence production remains
subordinate to a replayable, inspectable, CPU-side admission court.

## Mathematical Contract

The CUDA path mirrors the core arithmetic contract:

```text
Q16.16 value = raw_i32 / 65536
```

Stage computations are integer and contract-driven. The load-bearing
equivalence target is not floating-point closeness; it is byte identity
for the stage material and digest chain under the declared mode. The
device side may compute:

- residual fields;
- drift/slew signs;
- detector motif bitsets;
- consensus axes 1, 2, 3, 4, and 7;
- candidate summaries and stage digests on throughput paths.

The bank-reserved axes stay host-side:

```text
axis 5: entity locality
axis 6: topological adjacency
axis 8: semantic admissibility
axis 9: confuser suppression
```

This is the semantic non-bypass boundary. A GPU result can be evidence
for a later court decision; it is not itself an admitted episode.

## Code

Default build, no CUDA toolkit required:

```sh
cargo test -p dsfb-gpu-debug-cuda
```

CUDA build, requiring `nvcc` and a compatible NVIDIA environment:

```sh
cargo test -p dsfb-gpu-debug-cuda --features cuda
```

Minimal Rust surface:

```rust
use dsfb_gpu_debug_cuda::{pipeline_available, GpuError};

match pipeline_available() {
    Ok(()) => println!("CUDA feature enabled"),
    Err(GpuError::CudaUnavailable) => println!("built without CUDA"),
    Err(e) => println!("GPU error: {e:?}"),
}
```

## Claim Boundary

This crate claims a bounded CUDA dispatch and FFI surface for
deterministic evidence production. It does not claim peak bandwidth,
optimal occupancy, optimal multi-GPU scaling, production CUDA
performance, probabilistic inference, learned detector usefulness, or
semantic authority.

## Publish Order

Publish after `dsfb-gpu-debug-core = 0.1.1` is visible on crates.io.

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
