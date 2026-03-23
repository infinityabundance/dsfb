# Custom Renderer: TAA Buffer Export Playbook

## Overview

This playbook covers exporting the required temporal buffer set from any custom real-time
renderer. The requirements are renderer-agnostic: any pipeline that maintains a TAA history
buffer and outputs per-pixel motion vectors can provide the required data.

---

## Required Buffers

| Buffer | Capture Point | Format |
|--------|--------------|--------|
| `current_color` | After lighting/shading, before TAA blend | EXR RGB32F preferred |
| `history_color` | TAA history input (prev resolved, reprojected) | EXR RGB32F preferred |
| `motion_vectors` | Per-pixel MV pass | EXR RG32F preferred |
| `current_depth` | Depth pass (must be linear) | EXR R32F preferred |
| `current_normals` | G-buffer / normal pass | EXR RGB32F preferred |

---

## Capture Points

The critical constraint is **capture point ordering**:

```
Frame N rendering:
  1. Depth pre-pass          → capture current_depth here
  2. G-buffer / normal pass  → capture current_normals here
  3. Motion vector pass      → capture motion_vectors here
  4. Lighting / shading      → capture current_color HERE (before TAA)
  5. TAA pass
     ├── Input: current_color + history_color → capture history_color HERE (TAA input)
     └── Output: resolved_color (do NOT use this as current_color)
```

If `current_color` is captured after TAA output instead of before TAA input, the DSFB
system will measure post-AA error, which is not meaningful for supervision evaluation.

---

## Format Specifications

### Color buffers (current_color, history_color)
- **Format:** OpenEXR with 32-bit float RGB channels (no alpha needed)
- **Color space:** Linear light, no gamma correction, no tonemapping
- **Range:** [0, ∞) — HDR content may exceed 1.0
- **Not acceptable:** Gamma-encoded (sRGB), tonemapped (ACES, Reinhard), or LDR PNG

If only PNG is available, use 16-bit PNG with values scaled to [0, 1] (clipped HDR is acceptable
for evaluation but must be noted in the metadata).

### Depth buffer (current_depth)
- **Format:** EXR R32F (single channel float)
- **Convention:** Monotonically increasing with distance. Larger value = further from camera.
- **Acceptable conventions:**
  - Linear view-space Z (e.g., `gl_FragCoord.w` → `1/w` in NDC → linear Z via near/far)
  - Normalized linear depth: `(z - near) / (far - near)` — acceptable if consistent
  - Log depth: acceptable but must be linearized before export
- **Not acceptable:** Raw reversed-Z NDC depth (where near=1, far=0) without conversion

Conversion from reversed-Z NDC to linear depth (OpenGL convention):
```python
# near, far = clip plane distances
linear_z = (2.0 * near * far) / (far + near - (2.0 * ndc_depth - 1.0) * (far - near))
```

Conversion from DirectX reversed-Z NDC to linear depth:
```python
linear_z = near / ndc_depth  # for infinite far plane
# or:
linear_z = (near * far) / (far - ndc_depth * (far - near))
```

### Motion vectors (motion_vectors)
- **Format:** EXR RG32F (two-channel float)
- **Convention:** Per-pixel offset from current pixel to corresponding pixel in previous frame,
  in **pixel units**. Positive X = samples from a pixel further right in the history frame.
  Positive Y = samples from a pixel further down in the history frame.
- **Coordinate system:**
  - X channel: `history_uv.x = (current_x + mv_x) / width`
  - Y channel: `history_uv.y = (current_y + mv_y) / height`

Converting from NDC/clip-space motion vectors to pixel offsets:
```python
import numpy as np
mv_ndc = load_exr("mv_ndc.exr")   # [H, W, 2], range [-1, 1]
H, W = mv_ndc.shape[:2]
mv_pixels = mv_ndc * np.array([W / 2.0, H / 2.0])
save_exr("motion_vectors.exr", mv_pixels)
```

Converting from UV-space motion vectors:
```python
mv_uv = load_exr("mv_uv.exr")   # [H, W, 2], range [-1, 1] or [0, 1] delta
mv_pixels = mv_uv * np.array([W, H])
```

### Normals (current_normals)
- **Format:** EXR RGB32F
- **Convention:** View-space unit vectors. Camera-facing surface = positive Z.
  X right, Y up, Z toward camera (right-handed view space).
- **Channel encoding:** Raw float values, not octahedral or spherical harmonic encoded.
- **Range:** Each component in [-1, 1], magnitude = 1.0

Converting from world-space normals:
```glsl
// In a blit shader (or CPU equivalent):
vec3 view_normal = normalize(mat3(view_matrix) * world_normal);
// Output view_normal as RGB32F
```

---

## Metadata JSON

Create a `metadata.json` file alongside each frame's buffers:
```json
{
  "frame_index": 1,
  "history_frame_index": 0,
  "width": 1920,
  "height": 1080,
  "source_kind": "engine_native",
  "real_external_data": true,
  "scene_name": "your_scene_name",
  "depth_convention": "linear_view_space_z",
  "normal_convention": "view_space_unit_vectors",
  "motion_vector_convention": "pixel_offset_to_prev"
}
```

---

## File Naming Convention

```
data/engine_native/
  frame_000/
    current_color.exr
    history_color.exr
    motion_vectors.exr
    current_depth.exr
    current_normals.exr
    metadata.json
```

One `frame_000` directory = one captured frame pair (current + history).
Multiple frame pairs can be added as `frame_001/`, `frame_002/`, etc.

---

## Minimal Export Checklist

Before running import, verify:
- [ ] `current_color.exr` is linear HDR, captured before TAA
- [ ] `history_color.exr` is the reprojected previous frame (or labeled derived-low-confidence)
- [ ] `motion_vectors.exr` has two channels, values in pixel offsets
- [ ] `current_depth.exr` has one channel, larger = further from camera
- [ ] `current_normals.exr` has three channels, view-space unit vectors
- [ ] `metadata.json` exists with correct width, height, frame indices
- [ ] All buffers have the same width and height

---

## Validation: Quick Check Before Running Import

```bash
python3 - <<'EOF'
import struct, sys

def check_exr(path, channels_expected):
    with open(path, 'rb') as f:
        magic = struct.unpack('<I', f.read(4))[0]
        if magic != 20000630:
            print(f"FAIL: {path} is not a valid EXR file")
            return
    print(f"OK: {path} is EXR ({channels_expected}ch expected)")

check_exr("data/engine_native/frame_000/current_color.exr", 3)
check_exr("data/engine_native/frame_000/motion_vectors.exr", 2)
check_exr("data/engine_native/frame_000/current_depth.exr", 1)
check_exr("data/engine_native/frame_000/current_normals.exr", 3)
print("Basic format check complete")
EOF
```

---

## After Exporting

1. Update `examples/engine_native_capture_manifest.json`:
   - Set `source.engine_type` to `"custom"`
   - Set `source.capture_tool` to your renderer name/version
   - Update all buffer paths

2. Run import and replay:
```bash
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native

cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

3. Check the import report for any rejected buffers:
```bash
cat generated/engine_native/engine_native_import_report.md
```

4. Validate:
```bash
cargo run --release -- validate-final \
  --output generated/final_bundle
```

---

## What Is NOT Acceptable

- Synthesized or placeholder buffer values (random noise, constant fills, upscaled proxies)
- Post-tonemapped color
- Reversed-Z depth without linearization
- World-space normals without view-space transformation
- Motion vectors that are velocity (in world units per second) rather than pixel offsets
- Mismatched resolution between any two buffers
- Reusing the same frame as both current and history (no temporal pair)
