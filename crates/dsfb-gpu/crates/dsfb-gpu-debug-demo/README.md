[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/infinityabundance/dsfb/blob/main/crates/dsfb-gpu/notebooks/dsfb_gpu_debug_colab.ipynb)

# dsfb-gpu-debug-demo

`dsfb-gpu-debug-demo` is the command-line replay surface for DSFB-GPU.
The binary is named `dsfb-gpu-debug`. It generates fixtures, runs the
CPU reference path, runs the CUDA path when enabled, compares case
files, measures selected pipeline paths, and emits audit artifacts.

This crate is intentionally a delivery crate. It does not own the
semantic rules and it does not make the GPU authoritative. It wires the
reference court in `dsfb-gpu-debug-core` to the CUDA evidence factory in
`dsfb-gpu-debug-cuda` so a user can reproduce, compare, and inspect the
case-file chain from the terminal.

## What

The CLI exposes:

- `generate-fixture`: produce the canonical synthetic trace fixture.
- `run-cpu`: run the deterministic CPU reference pipeline.
- `run-gpu`: run the CUDA path when built with `--features cuda`.
- `compare`: compare two case files and report the verdict.
- `bench`: measure selected CPU/GPU pipeline paths.
- `bench-gpu-scale`: emit the headline money-table benchmark report
  used by this workspace's local audit trail.
- `s-real-audit`: run the sealed S-REAL audit driver over declared
  datasets.

## Where

This crate lives at `crates/dsfb-gpu/crates/dsfb-gpu-debug-demo` in the
[DSFB repository](https://github.com/infinityabundance/dsfb). It depends
on:

- [`dsfb-gpu-debug-core`](https://crates.io/crates/dsfb-gpu-debug-core)
  for deterministic semantics and case files;
- [`dsfb-gpu-debug-cuda`](https://crates.io/crates/dsfb-gpu-debug-cuda)
  for optional CUDA dispatch.

The wider Atlas crates are
[`dsfb-gpu-atlas-corpus`](https://crates.io/crates/dsfb-gpu-atlas-corpus)
and
[`dsfb-gpu-atlas-registry`](https://crates.io/crates/dsfb-gpu-atlas-registry).
The public Colab notebook runs the audit-oriented replay flow on a
Colab GPU runtime and reports divergence honestly when bytes do not
match the committed seal.

## Why

Reproducibility needs an operator surface. A library can define
deterministic semantics, but a reviewer still needs commands that build,
run, compare, and preserve the artifacts. This crate is that surface: a
thin CLI over the core and CUDA crates, with exit codes that automation
can distinguish.

## Mathematical Contract

The CLI does not invent a separate mathematical model. It invokes the
same deterministic pipeline:

```text
trace events
  -> window features
  -> residual Q16.16 fields
  -> drift/slew signs
  -> detector motif masks
  -> consensus axes
  -> candidate intervals
  -> bank-admitted episodes
  -> case-file hash chain
```

For CPU/GPU comparison, equality is byte-level over the declared case
file mode and hash chain. A mismatch is not rounded away as numerical
tolerance; it is a verdict condition surfaced by `compare`.

## Code

Show commands:

```sh
cargo run -p dsfb-gpu-debug-demo --bin dsfb-gpu-debug -- help
```

Run the CPU path:

```sh
cargo run -p dsfb-gpu-debug-demo --bin dsfb-gpu-debug -- generate-fixture --out target/dsfb-gpu/fixture.json
cargo run -p dsfb-gpu-debug-demo --bin dsfb-gpu-debug -- run-cpu --fixture target/dsfb-gpu/fixture.json --out target/dsfb-gpu/cpu.case.json
```

Build with CUDA dispatch:

```sh
cargo run -p dsfb-gpu-debug-demo --features cuda --bin dsfb-gpu-debug -- run-gpu --fixture target/dsfb-gpu/fixture.json --out target/dsfb-gpu/gpu.case.json
```

Run tests:

```sh
cargo test -p dsfb-gpu-debug-demo
```

## Features

- `default`: CPU and audit commands build; GPU commands report
  `GpuError::CudaUnavailable` when reached.
- `cuda`: forwards to `dsfb-gpu-debug-cuda/cuda` and requires `nvcc`
  plus a compatible CUDA environment at build time.

## Claim Boundary

This crate is a CLI and reproducibility wrapper. It does not claim new
detector mathematics, learned usefulness, probabilistic inference,
medical or safety diagnosis, benchmark portability, or independent
semantic authority beyond the core and CUDA crates it invokes.

## Publish Order

Publish after both `dsfb-gpu-debug-core = 0.1.1` and
`dsfb-gpu-debug-cuda = 0.1.1` are visible on crates.io.

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
