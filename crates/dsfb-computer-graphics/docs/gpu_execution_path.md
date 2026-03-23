# GPU Execution Path

This crate now contains a real GPU-executable minimum supervisory kernel implemented with `wgpu` and WGSL in [`src/gpu.rs`](/home/one/dsfb/crates/dsfb-computer-graphics/src/gpu.rs).

## Scope

- Inputs: current color, reprojected history, motion vectors, current depth, reprojected depth, current normals, reprojected normals
- Outputs: trust, alpha, intervention
- Kernel target: minimum host-realistic DSFB supervision path
- Explicitly excluded from the minimum kernel: motion disagreement extension

## Current Status

- The path is GPU-executable when a usable adapter is present.
- The crate generates `gpu_execution_report.md` and `gpu_execution_metrics.json`.
- If no GPU adapter is available in the current environment, the report must say so explicitly instead of implying measured performance.

## Run Command

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-gpu-path --output generated/gpu_path
```

## What This Proves

- The repository no longer stops at CPU-only architectural prose.
- An external evaluator can run an actual compute kernel on a GPU host without re-architecting the crate.

## What This Does Not Prove

- It does not prove performance on a specific GPU unless that hardware was actually measured.
- It does not prove production renderer integration or production-scale throughput.
