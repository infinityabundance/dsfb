# dsfb-gpu-debug-cuda

CUDA FFI bridge and kernel dispatch for `dsfb-gpu-debug`.

The default build does not require CUDA. With default features, public
GPU entry points return `GpuError::CudaUnavailable`, which lets CPU and
Atlas verification build on ordinary hosts. Enabling the `cuda` feature
uses `nvcc` from `build.rs` to compile the kernels.

## Features

- `default`: Rust shim only, no CUDA toolkit required.
- `cuda`: compile and link the CUDA kernels under the workspace `cuda/`
  directory.

## Scope

The GPU side produces deterministic witness bytes, candidate summaries,
and stage digests under explicit numeric and hashing contracts. It does
not admit episodes, assign final meaning, or bypass the CPU-side bank
authority in `dsfb-gpu-debug-core`.

## Publish order

Publish after `dsfb-gpu-debug-core = 0.1.0` is visible on crates.io.
