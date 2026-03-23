# Engine-Native Capture Schema

## Format Version

`dsfb_engine_native_v1`

This schema is a first-class extension of `dsfb_external_capture_v1`. Engine-native manifests
use `source.kind = "engine_native"` instead of `"files"` or `"synthetic_compat"`.

---

## Manifest Structure

```json
{
  "format_version": "dsfb_engine_native_v1",
  "description": "...",
  "source": {
    "kind": "engine_native",
    "engine_type": "unreal | unity | custom | pending",
    "engine_version": "optional string",
    "capture_tool": "optional string",
    "capture_note": "optional string"
  },
  "captures": [...],
  "normalization": {...},
  "notes": [...]
}
```

When `engine_type = "pending"` or buffer files are absent on disk, all downstream reports carry
`ENGINE_NATIVE_CAPTURE_MISSING=true` and the pipeline generates pending placeholder reports.

---

## Required Buffers

### `current_color`
- **Semantic:** Current frame linear HDR color, captured before TAA resolve
- **Capture point:** After shading, before TAA blend
- **Accepted formats:** `exr_rgb32f`, `png_rgb8`, `json_rgb_f32`, `raw_rgb32f`
- **Channel convention:** Linear RGB, [0, ∞), no gamma, no tonemapping
- **Required:** YES

### `history_color` (alias: `reprojected_history`)
- **Semantic:** Previous frame's resolved color, reprojected to current frame coordinates
- **Capture point:** TAA pass input (the history buffer before blend)
- **Accepted formats:** `exr_rgb32f`, `png_rgb8`, `json_rgb_f32`, `raw_rgb32f`
- **Channel convention:** Linear RGB in current frame's coordinate system
- **Required:** YES

### `motion_vectors`
- **Semantic:** Per-pixel screen-space motion vectors
- **Capture point:** Motion vector / velocity pass
- **Accepted formats:** `exr_rg32f`, `json_vec2_f32`, `raw_rg32f`
- **Channel convention:**
  - Channel 0 (X): Pixel offset in X from current to previous frame
  - Channel 1 (Y): Pixel offset in Y from current to previous frame
  - Positive X: pixel is sampled from further right in history
  - Positive Y: pixel is sampled from further down in history
- **Coordinate convention:** `pixel_offset_to_prev`
- **Required:** YES

### `current_depth` (alias: `current_depth`)
- **Semantic:** Per-pixel depth for the current frame
- **Accepted formats:** `exr_r32f`, `json_scalar_f32`, `raw_r32f`
- **Depth convention:** Monotonically increasing with distance. Larger = further from camera.
  The absolute scale does not matter; only relative ordering within the frame pair matters.
- **Not acceptable:** Raw reversed-Z NDC without linearization
- **Required:** YES

### `history_depth` (alias: `reprojected_depth`)
- **Semantic:** Previous frame depth, reprojected to current frame coordinates
- **Accepted formats:** `exr_r32f`, `json_scalar_f32`, `raw_r32f`
- **Same depth convention as `current_depth`**
- **Required:** NO (optional)
- **Absence handling:** Proxy derived from `current_depth` using motion vector extrapolation
- **Quality label when absent:** `derived-low-confidence`

### `current_normals`
- **Semantic:** Per-pixel view-space surface normals for the current frame
- **Accepted formats:** `exr_rgb32f`, `json_vec3_f32`, `raw_rgb32f`
- **Normal convention:** Unit vectors in view space. Camera-facing = positive Z.
  X = right, Y = up, Z = toward camera (right-handed).
- **Range:** Each component [-1, 1], magnitude = 1.0
- **Not acceptable:** Octahedral encoded, spherical harmonics, world-space without transformation
- **Required:** YES

### `history_normals` (alias: `reprojected_normals`)
- **Semantic:** Previous frame normals, reprojected to current frame coordinates
- **Accepted formats:** `exr_rgb32f`, `json_vec3_f32`, `raw_rgb32f`
- **Same normal convention as `current_normals`**
- **Required:** NO (optional)
- **Absence handling:** Approximate from `current_normals`
- **Quality label when absent:** `derived-low-confidence`

---

## Optional Buffers

### `optional_mask` (roi_mask)
- **Semantic:** Per-pixel region-of-interest mask
- **Format:** `exr_r32f`, `json_scalar_f32` — values in [0, 1]
- **Convention:** 1.0 = inside ROI (thin structures, high-detail regions), 0.0 = outside ROI
- **Absence handling:** Derived from depth+normal discontinuities
- **Quality label when absent:** `derived-low-confidence`

### Jitter, exposure, camera matrices
- Provided via `metadata.json` (`json_metadata` format)
- Used for improved reprojection quality validation, not required for core evaluation

---

## Normalization Conventions

Normalization strings are informational — they document what the exporter has already done.
The pipeline does NOT re-normalize; it expects values in the stated conventions.

```json
"normalization": {
  "color": "linear RGB, pre-tonemapped, in [0, ∞)",
  "motion_vectors": "pixel offsets to the previous frame; positive x samples from a pixel further right in history",
  "depth": "monotonic depth with larger values further from camera",
  "normals": "unit vectors in view space; camera-facing = positive z"
}
```

---

## Hard Rejection Conditions

The import pipeline rejects (returns error) if:
1. Any two required buffers have mismatched width or height
2. `motion_vectors` has only one channel instead of two
3. `current_depth` has negative values (indicates reversed-Z without conversion)
4. Any required buffer path exists but file is empty or unreadable

---

## Quality Labels

| Label | Meaning |
|-------|---------|
| `native` | Buffer exported directly from the renderer at the correct capture point |
| `derived-high-confidence` | Buffer derived from native data with a well-constrained algorithm |
| `derived-low-confidence` | Buffer approximated with limited information (e.g., depth extrapolation) |
| `unavailable` | Buffer not provided and cannot be derived |

These labels appear in the import report and resolved manifest. They do not block evaluation
but affect the confidence classification of Demo A and Demo B results.

---

## Schema Relationships

```
dsfb_external_capture_v1 (existing)
  └── ExternalCaptureManifest
        ├── source.kind = "files"            → generic file-based capture
        ├── source.kind = "synthetic_compat" → synthetic scenario compatibility
        └── source.kind = "engine_native"    → this schema (first-class engine path)
              ├── engine_type: unreal | unity | custom | pending
              └── all ExternalBufferSet fields apply with engine-native naming
```

---

## See Also

- `examples/engine_native_capture_manifest.json` — canonical manifest template
- `examples/engine_native_buffer_schema.json` — detailed buffer format spec
- `docs/unreal_export_playbook.md` — Unreal Engine export steps
- `docs/unity_export_playbook.md` — Unity export steps
- `docs/custom_renderer_export_playbook.md` — generic custom renderer steps
