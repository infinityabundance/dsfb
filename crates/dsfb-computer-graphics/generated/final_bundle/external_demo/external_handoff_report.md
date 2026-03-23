# External Handoff Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report covers the file-based external buffer import path. It demonstrates that the crate is external-capable, not externally validated.

Source kind: `synthetic_compat`. Externally validated: `false`.

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
- `json_scalar_f32`
- `json_vec2_f32`
- `json_vec3_f32`
- `json_mask_bool`
- `json_metadata`

## Normalization Conventions

- linear RGB in [0,1]
- pixel offsets to the previous frame
- monotonic depth with larger disagreement indicating less trust
- unit vectors in a consistent view-space basis

## Imported Capture Summary

- Resolution: 160x96
- Frame index: 6
- History frame index: 5
- Mean trust: 0.7698
- Mean alpha: 0.2826
- Mean intervention: 0.2302

## How An Engine Team Would Use This

- Export one frame pair using the buffer names and normalization described in the manifest.
- Set `source.kind` to `files` and point the buffer paths at the exported assets.
- Run `cargo run --release -- import-external --manifest <manifest> --output <dir>`.
- Inspect `external_trust.png`, `external_alpha.png`, and `external_intervention.png` plus the generated report.

## What Is Not Proven

- This report does not claim any real engine capture has been validated unless the metadata says so.
- The example manifest included in the crate is synthetic compatibility data, not field data.

## Remaining Blockers

- A real renderer still needs to export buffers into this schema.
- Real production captures and engine motion vectors are still required for external validation.
- GPU measurements on imported captures remain future work.

## Manifest Notes

- Switch source.kind from synthetic_compat to files when real engine exports are available.
- This example is external-capable but not externally validated.
