# External Handoff

This crate supports a stable file-based import path for external engine buffers.

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

## Accepted Formats

- `png_rgb8`
- `json_rgb_f32`
- `json_scalar_f32`
- `json_vec2_f32`
- `json_vec3_f32`
- `json_mask_bool`
- `json_metadata`

## Run Command

```bash
cd crates/dsfb-computer-graphics
cargo run --release -- import-external --manifest examples/external_capture_manifest.json --output generated/external_demo
```

## External-Capable vs Externally Validated

- External-capable means the crate can ingest a stable external buffer manifest and run the supervisory pass.
- Externally validated means real engine-exported data was supplied and evaluated.

This crate is external-capable. It is not externally validated by default.
