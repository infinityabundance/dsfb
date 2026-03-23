# Unreal Engine: TAA Buffer Export Playbook

## Overview

This playbook covers exporting the exact temporal buffer set required by the DSFB engine-native
capture pipeline from Unreal Engine 5.x (also applicable to UE 4.27+).

The goal is one exported frame pair: (frame N-1, frame N) with all required buffers at the same
resolution, captured after shading but before TAA resolve.

---

## Required Buffers

| Buffer | UE Source | Format |
|--------|-----------|--------|
| `current_color` | `SceneColor` (pre-TAA) | EXR RGB32F |
| `history_color` | TAA history buffer | EXR RGB32F |
| `motion_vectors` | `VelocityBuffer` | EXR RG32F |
| `current_depth` | `SceneDepth` (linear) | EXR R32F |
| `current_normals` | `GBufferB` (world normal → view space) | EXR RGB32F |
| `history_depth` | optional — see below | EXR R32F |
| `history_normals` | optional — see below | EXR RGB32F |
| `roi_mask` | optional — any mask texture | EXR R32F |

---

## Method 1: High Resolution Screenshot (HighResBatch) + Custom Pass

This is the recommended approach for a single frame capture without a plugin.

### Step 1: Enable velocity buffer output
In your project's `DefaultEngine.ini` or in-editor console:
```ini
r.BasePassOutputsVelocity=1
r.MotionBlurQuality=0
```
Or in the editor console (`~`):
```
r.BasePassOutputsVelocity 1
```

### Step 2: Enable SceneCapture2D component for buffers
In a Blueprint or a custom actor:
1. Add a `SceneCapture2D` component
2. Set `CaptureSource = FinalColorHDR` for color (or `SceneColor` for pre-tonemapped)
3. Set `ShowFlags.TemporalAA = false` to capture pre-TAA input

For the `VelocityBuffer` (motion vectors), use a separate `SceneCapture2D` with:
```
ShowFlags.MotionBlur = true
CaptureSource = FinalColorHDR  (using a custom post-process material sampling the velocity buffer)
```

### Step 3: Export via MovieRenderQueue (recommended)
1. Open **Window → Movie Render Queue**
2. Add your level sequence
3. Under **Settings**, add:
   - `Image Sequence Output (EXR)` — 32-bit float, linear color space
   - `Custom Render Pass` for: SceneColor, VelocityBuffer, SceneDepth, GBufferB
4. Enable `Deferred Rendering` mode
5. Set `Anti-Aliasing → Method = None` (to capture raw TAA input, not output)
6. Set `Output Resolution` to your target resolution
7. Render two consecutive frames (frame index N-1 and N)

**Output files will appear in your project's `Saved/MovieRenders/` directory.**

### Step 4: Export history_color (optional but recommended)
Unreal does not directly expose the TAA history buffer via standard capture. Options:
- **Option A**: Use the previous frame's resolved output as a proxy.
  This is acceptable but is labeled `derived-low-confidence`.
- **Option B**: Write a custom TAA plugin that exports the history buffer before blend.
  This requires engine source access and is the only way to get `native` quality history.

If history is unavailable, it is automatically classified as `derived-low-confidence` in the
import report. This does not block evaluation but reduces the quality of the temporal analysis.

### Step 5: Convert depth to linear
UE's `SceneDepth` in EXR is already in linear depth (view-space Z). Verify by checking that
near values are small and far values are large. If you see reversed values (near = large),
you have reversed-Z — convert with `linear_depth = far * near / (far - depth * (far - near))`.

### Step 6: Convert normals to view space
GBufferB contains encoded world-space normals. Decode and transform:
```hlsl
// Decode from GBufferB.xyz (UE octahedral or simple encoding)
float3 WorldNormal = DecodeNormal(GBufferB.xyz);
float3 ViewNormal = mul(float4(WorldNormal, 0), View.TranslatedWorldToView).xyz;
```
Export the view-space normal as EXR RGB32F.

### Step 7: Convert motion vectors to pixel offsets
UE's `VelocityBuffer` stores motion vectors in NDC space (range [-1, 1] for full resolution).
Convert to pixel offsets:
```python
# In Python post-processing:
import numpy as np
mv_ndc = load_exr("velocity.exr")  # [H, W, 2], range [-1, 1]
width, height = mv_ndc.shape[1], mv_ndc.shape[0]
mv_pixels = mv_ndc * np.array([width / 2, height / 2])
# Now mv_pixels[y, x] = (dx_to_prev, dy_to_prev) in pixel units
```

---

## Method 2: Render Document / Sequencer with BufferVisualization

For a quicker but less precise approach:
1. Open the **Sequencer** with your scene
2. Add a `Movie Render Queue` job
3. Under Custom Render Passes, select:
   - `BufferVisualization/SceneColor`
   - `BufferVisualization/Velocity`
   - `BufferVisualization/SceneDepth`
   - `BufferVisualization/WorldNormal`
4. Export as EXR 32-bit

**Note:** BufferVisualization exports may include post-processing effects. Verify the exports
are truly pre-TAA for `current_color`.

---

## File Naming Convention

Name exported files to match the manifest schema:
```
data/engine_native/frame_000/current_color.exr
data/engine_native/frame_000/history_color.exr    (or frame N-1 resolved output)
data/engine_native/frame_000/motion_vectors.exr   (converted to pixel offsets)
data/engine_native/frame_000/current_depth.exr    (linear depth)
data/engine_native/frame_000/current_normals.exr  (view-space unit normals)
data/engine_native/frame_000/metadata.json
```

Metadata JSON:
```json
{
  "frame_index": 1,
  "history_frame_index": 0,
  "width": 1920,
  "height": 1080,
  "source_kind": "engine_native",
  "real_external_data": true,
  "scene_name": "your_scene_name"
}
```

---

## After Exporting

1. Update `examples/engine_native_capture_manifest.json`:
   - Set `source.engine_type` to `"unreal"`
   - Set `source.engine_version` to your UE version (e.g., `"5.3"`)
   - Update all buffer paths to point to the exported files

2. Run the import:
```bash
cd crates/dsfb-computer-graphics
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

3. Run the full replay:
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

4. Validate:
```bash
cargo run --release -- validate-final \
  --output generated/final_bundle
```

---

## What Is NOT Acceptable

- Placeholder or synthesized buffer values
- Post-tonemapped color (after ACES/LUT — must be pre-tonemapped linear)
- Reversed-Z depth without conversion
- World-space normals without view-space transformation
- Motion vectors in UV offset space without pixel offset conversion
- Using the TAA output (resolved frame) as `current_color` — must be TAA *input*

---

## Remaining Blockers (Post-Export)

- history_color at native quality requires engine source access (only available as derived proxy without it)
- Ground-truth reference frames require a separate no-TAA render pass
