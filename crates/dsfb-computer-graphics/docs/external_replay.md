# External Replay

This crate supports a file-based external replay path for one frame pair plus history. The goal is to let an engine or renderer team drop buffers into a stable schema and run the same supervisory code without rewriting the crate.

## Required Buffers

- current color
- reprojected history
- motion vectors
- current depth
- reprojected depth
- current normals
- reprojected normals
- metadata

Optional:

- debug or ROI-like mask
- reference or ground-truth sidecar data if the external team has it
- variance sidecar data if the external team has it

## Accepted Formats

- `png_rgb8`
- `json_rgb_f32`
- `exr_rgb32f`
- `raw_rgb32f`
- `json_scalar_f32`
- `exr_r32f`
- `raw_r32f`
- `json_vec2_f32`
- `exr_rg32f`
- `raw_rg32f`
- `json_vec3_f32`
- `exr_rgb32f`
- `raw_rgb32f`
- `json_mask_bool`
- `raw_mask_u8`
- `json_metadata`

## Layout Expectations

- All required buffers must share the same width and height.
- Motion vectors are expressed in pixel units toward the previous frame.
- Colors are linear RGB in `[0, 1]`.
- Normals must use one consistent view-space basis across current and reprojected buffers.
- Depth must remain monotonic so disagreement is interpretable.

## Run Commands

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_real
```

Alias:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- replay-external --manifest examples/external_capture_manifest.json --output generated/external_real
```

The external replay bundle now also writes:

- `gpu_external_report.md`
- `gpu_external_metrics.json`
- `demo_a_external_report.md`
- `demo_b_external_report.md`
- `demo_b_external_metrics.json`
- `external_validation_report.md`
- `scaling_report.md`
- `scaling_metrics.json`
- `memory_bandwidth_report.md`
- `integration_scaling_report.md`
- `figures/`

## What This Proves

- The crate is external-capable.
- An evaluator can supply buffers through a stable manifest and obtain trust, alpha, and intervention outputs.
- The same imported or external-ready buffers can now drive GPU replay, scaling analysis, bandwidth accounting, and integration notes.

## What This Does Not Prove

- It does not prove real engine correctness until a real engine actually exports into this schema.
- It does not prove production robustness or GPU scheduling quality on those imported captures.
