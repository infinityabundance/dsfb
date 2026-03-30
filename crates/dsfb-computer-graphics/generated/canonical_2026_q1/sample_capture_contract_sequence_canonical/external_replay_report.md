# External Replay Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report covers the file-based external buffer replay path. It demonstrates that the crate is external-capable, not externally validated.

Source kind: `unreal_native`. Externally validated: `true`. Real external data provided: `true`.

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

- linear_rgb_from_scene_capture_png
- pixel_offset_to_prev; normalized into pixel offsets to the previous frame
- monotonic_visualized_depth
- world_space_unit

## Imported Capture Summary

- Resolution: 256x144
- Frame index: 1
- History frame index: 0
- Mean trust: 0.7866
- Mean alpha: 0.2678
- Mean intervention: 0.2135

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

- provenance_label=unreal_native
- real_engine_capture=true
- no synthetic fallback is implemented in this path
