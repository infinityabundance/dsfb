# Unity: TAA Buffer Export Playbook

## Overview

This playbook covers exporting the required temporal buffer set from Unity 2022+ (URP and HDRP).
Both render pipelines are covered. HDRP is preferred because it exposes more TAA internals.

---

## Required Buffers

| Buffer | Unity Source | Pipeline | Format |
|--------|-------------|----------|--------|
| `current_color` | Color texture before TAA | URP / HDRP | EXR RGB32F |
| `history_color` | TAA history (prev frame) | HDRP preferred | EXR RGB32F |
| `motion_vectors` | MotionVectorTexture | URP / HDRP | EXR RG32F |
| `current_depth` | CameraDepthTexture (linear) | URP / HDRP | EXR R32F |
| `current_normals` | NormalTexture (view space) | HDRP / URP | EXR RGB32F |

---

## Method 1: Custom Renderer Feature (URP)

### Step 1: Create a custom `ScriptableRendererFeature`
```csharp
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.Universal;
using System.IO;

public class BufferExportFeature : ScriptableRendererFeature
{
    private BufferExportPass _pass;

    public override void Create()
    {
        _pass = new BufferExportPass();
        _pass.renderPassEvent = RenderPassEvent.BeforeRenderingPostProcessing;
    }

    public override void AddRenderPasses(ScriptableRenderer renderer, ref RenderingData renderingData)
    {
        renderer.EnqueuePass(_pass);
    }
}

public class BufferExportPass : ScriptableRenderPass
{
    public override void Execute(ScriptableRenderContext context, ref RenderingData renderingData)
    {
        var camera = renderingData.cameraData.camera;
        if (!camera.CompareTag("MainCamera")) return;

        // Export color (before TAA)
        SaveRenderTexture(renderingData.cameraData.renderer.cameraColorTarget,
            "current_color");
        // Export depth
        SaveRenderTexture(renderingData.cameraData.renderer.cameraDepthTarget,
            "current_depth");
    }

    private void SaveRenderTexture(RenderTargetIdentifier target, string name)
    {
        // Blit to a readable RT and save as EXR
        // Implementation: use AsyncGPUReadback for non-blocking capture
        var desc = new RenderTextureDescriptor(Screen.width, Screen.height,
            RenderTextureFormat.ARGBFloat, 0);
        var rt = RenderTexture.GetTemporary(desc);
        // ... blit and save
    }
}
```

### Step 2: Export MotionVectorTexture
In URP, motion vectors are accessible via:
```csharp
Shader.GetGlobalTexture("_MotionVectorTexture")
```
Motion vectors in Unity are stored as (x, y) in clip space ([-1, 1]). Convert to pixel offsets:
```csharp
// In shader or CPU post-processing:
float2 mv_pixels = mv_clip * float2(width * 0.5, height * 0.5);
```

### Step 3: Export depth as linear
```csharp
// CameraDepthTexture is in native depth format (may be reversed-Z)
// Convert to linear eye depth:
float linear = LinearEyeDepth(depth, _ZBufferParams);
```

---

## Method 2: HDRP Custom Pass (Recommended for Full Buffer Access)

HDRP's `CustomPass` API provides cleaner access to G-buffer and temporal history.

### Step 1: Create a `CustomPass` at `BeforeRendering`
```csharp
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.HighDefinition;

class TemporalBufferExport : CustomPass
{
    protected override void Execute(CustomPassContext ctx)
    {
        // Access G-Buffer
        ctx.hdCamera.GetCameraDepthNormalsBuffer(out var depth, out var normals);

        // Access color before TAA
        var colorBuffer = ctx.cameraColorBuffer;

        // Access motion vectors
        var motionVectors = ctx.cameraMotionVectorsBuffer;

        // Export all buffers to disk
        ExportBuffer(colorBuffer, "current_color.exr", ctx.cmd);
        ExportBuffer(depth, "current_depth.exr", ctx.cmd);
        ExportBuffer(normals, "current_normals.exr", ctx.cmd);
        ExportBuffer(motionVectors, "motion_vectors.exr", ctx.cmd);
    }
}
```

### Step 2: Access TAA history (HDRP)
HDRP does not expose the TAA history buffer via public API. Options:
- **Option A**: Use the previous frame's resolved output (accessible via `PreviousViewProjectionMatrix`)
  as a proxy for history. Label this `derived-low-confidence`.
- **Option B**: Modify HDRP source (available in Package Manager as editable) to expose
  `m_PingPong[0]` from the TAA implementation.

### Step 3: Export normals in view space
HDRP normals are stored in world space in the G-buffer. Convert at export:
```csharp
// In a blit shader:
float3 worldNormal = DecodeNormal(normalBuffer.rgb);
float3 viewNormal = mul((float3x3)UNITY_MATRIX_V, worldNormal);
// Output viewNormal as EXR
```

---

## File Naming Convention

```
data/engine_native/frame_000/current_color.exr
data/engine_native/frame_000/history_color.exr
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
   - Set `source.engine_type` to `"unity"`
   - Set `source.engine_version` to your Unity version and pipeline (e.g., `"2023.1 HDRP 15"`)
   - Update buffer paths

2. Run import and replay:
```bash
cargo run --release -- import-engine-native \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native

cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

---

## What Is NOT Acceptable

- Post-processed color (after tonemapping / ACES)
- Depth in clip space without linear conversion
- World-space normals without view-space transformation
- Motion vectors in UV space without pixel offset conversion
- Fabricated or placeholder buffer data
