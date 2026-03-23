# External Replay Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report covers the file-based external buffer replay path. It demonstrates that the crate is external-capable, not externally validated.

Source kind: `davis_2017_real_video`. Externally validated: `true`. Real external data provided: `true`.

## Required Buffers

- `current_color`
- `reprojected_history`
- `motion_vectors`
- `current_depth`
- `reprojected_depth`
- `current_normals`
- `reprojected_normals`

## Accepted Formats

- `png_rgb8`
- `json_rgb_f32`
- `exr_rgb32f`
- `json_scalar_f32`
- `exr_r32f`
- `raw_r32f` with inline width/height/channels = 1
- `json_vec2_f32`
- `exr_rg32f`
- `raw_rg32f` with inline width/height/channels >= 2
- `json_vec3_f32`
- `raw_rgb32f` with inline width/height/channels >= 3
- `json_mask_bool`
- `raw_mask_u8` with inline width/height/channels = 1
- `json_metadata`

## Normalization Conventions

- sRGB frames converted to normalized RGB in [0,1]
- deterministic block-matching proxy in pixel offsets to the previous frame; derived-low-confidence
- segmentation-guided relative-depth proxy in [0,1]; derived-low-confidence and explicitly non-metric
- normals derived from the relative-depth proxy; derived-low-confidence

## Imported Capture Summary

- Resolution: 854x480
- Frame index: 79
- History frame index: 78
- Mean trust: 0.8583
- Mean alpha: 0.2044
- Mean intervention: 0.1420

## How An Engine Team Would Use This

- Export one frame pair using the buffer names and normalization described in the manifest.
- Set `source.kind` to `files` and point the buffer paths at the exported assets.
- Run `cargo run --release -- run-external-replay --manifest <manifest> --output <dir>`.
- Alias: `cargo run --release -- replay-external --manifest <manifest> --output <dir>`.
- Inspect `external_trust.png`, `external_alpha.png`, and `external_intervention.png` plus the generated report.

## What Is Not Proven

- This report does not claim any real engine capture has been validated unless the metadata says so.
- The example manifest included in the crate is synthetic compatibility data, not field data.

## Remaining Blockers

- A real renderer still needs to export buffers into this schema.
- Real production captures and engine motion vectors are still required for external validation.
- If the GPU external report is unmeasured on the evaluator machine, imported-capture GPU timing still remains future work there.

## Manifest Notes

- DAVIS provides native real-video color and native segmentation masks.
- Motion vectors, depth, and normals are explicit derived proxies rather than native buffers.
- Demo A metrics remain proxy-only because the mapped path has no renderer-quality ground truth.
