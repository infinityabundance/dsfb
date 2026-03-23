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

## Accepted Formats

- `png_rgb8`
- `json_rgb_f32`
- `json_scalar_f32`
- `json_vec2_f32`
- `json_vec3_f32`
- `json_mask_bool`
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
cargo run --release -- run-external-replay --manifest examples/external_capture_manifest.json --output generated/external_replay
```

Alias:

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- replay-external --manifest examples/external_capture_manifest.json --output generated/external_replay
```

## What This Proves

- The crate is external-capable.
- An evaluator can supply buffers through a stable manifest and obtain trust, alpha, and intervention outputs.

## What This Does Not Prove

- It does not prove real engine correctness until a real engine actually exports into this schema.
- It does not prove production robustness or GPU scheduling quality on those imported captures.
