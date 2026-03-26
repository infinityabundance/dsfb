# DATASET_SCHEMA

Schema version:

- `dsfb_unreal_native_v1`

Required top-level fields:

- `schema_version`
- `dataset_kind = "unreal_native"`
- `provenance_label = "unreal_native"`
- `dataset_id`
- `description`
- `engine`
- `contract`
- `frames`

Required engine fields:

- `engine_name = "unreal_engine"`
- `engine_version`
- `capture_tool`
- `real_engine_capture = true`

Required contract fields:

- `color_space` must declare a linear replay color space
- `tonemap = "disabled"`, `pre_tonemap_capture`, or `scene_capture_png_linearized`
- `depth_convention = "monotonic_linear_depth"` or `monotonic_visualized_depth`
- `normal_space = "view_space_unit"` or `world_space_unit`
- `motion_vector_convention = "pixel_offset_to_prev"` or `ndc_to_prev`
- `coordinate_space = "screen_space_current_to_previous"`

Required per-frame buffers:

- `current_color`
- `previous_color`
- `motion_vectors`
- `current_depth`
- `previous_depth`
- `current_normals`
- `previous_normals`
- `metadata`

Optional per-frame buffers:

- `history_color`
- `history_depth`
- `history_normals`
- `host_output`
- `reference_color`
- `roi_mask`
- `disocclusion_mask`
- `reactive_mask`

Metadata requirements:

- `frame_index`
- `history_frame_index`
- `width`
- `height`
- `source_kind = "unreal_native"`
- `provenance_label = "unreal_native"`
- `real_external_data = true`

Hard failures:

1. Wrong schema version.
2. Wrong dataset kind or provenance label.
3. `real_engine_capture = false`.
4. Missing required buffer.
5. Metadata indices do not match the manifest.
6. Buffer extents do not match metadata.
7. Non-finite depth or motion values.
8. Unsupported motion-vector convention.
9. Any attempt to run Unreal-native mode on synthetic or pending input.

Accepted buffer formats:

- Color: `exr_rgb32f`, `png_rgb8`, `json_rgb_f32`, `raw_rgb32f`
- Scalar: `exr_r32f`, `json_scalar_f32`, `raw_r32f`
- Motion: `exr_rg32f`, `json_vec2_f32`, `raw_rg32f`
- Normals: `exr_rgb32f`, `json_vec3_f32`, `raw_rgb32f`
- Mask: `exr_r32f`, `json_scalar_f32`, `json_mask_bool`, `raw_mask_u8`, `raw_r32f`

Canonical example:

- [`examples/unreal_native_capture_manifest.json`](/home/one/dsfb/crates/dsfb-computer-graphics/examples/unreal_native_capture_manifest.json)

Checked-in sample specifics:

- raw Unreal evidence is retained under [`data/unreal_native/sample_capture/frame_0001/raw`](/home/one/dsfb/crates/dsfb-computer-graphics/data/unreal_native/sample_capture/frame_0001/raw)
- the canonical sample materializes `json_rgb_f32`, `json_scalar_f32`, `json_vec3_f32`, `json_vec2_f32`, and `json_mask_bool`
- `motion_vectors.json` is metadata-derived for this minimal sample
- `current_normals.json` and `previous_normals.json` are metadata-derived for this minimal sample
- `current_depth.json` and `previous_depth.json` are decoded from a real Unreal depth-visualization export and therefore use `depth_convention = "monotonic_visualized_depth"`
