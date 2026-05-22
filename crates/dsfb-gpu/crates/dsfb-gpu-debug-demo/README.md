# dsfb-gpu-debug-demo

CLI package for the `dsfb-gpu-debug` proof surface.

The binary name is `dsfb-gpu-debug`. It can generate deterministic
fixtures, run the CPU reference path, run the GPU path when the `cuda`
feature is enabled, compare case files, and emit audit artifacts.

## Features

- `default`: CPU and audit commands, with GPU commands reporting
  `GpuError::CudaUnavailable` when reached.
- `cuda`: forwards to `dsfb-gpu-debug-cuda/cuda` and requires `nvcc`
  plus a compatible CUDA environment at build time.

## Scope

This crate is a delivery and reproducibility CLI. The load-bearing
semantic authority remains in `dsfb-gpu-debug-core`; GPU dispatch lives
in `dsfb-gpu-debug-cuda`.

## Publish order

Publish after both `dsfb-gpu-debug-core = 0.1.0` and
`dsfb-gpu-debug-cuda = 0.1.0` are visible on crates.io.
