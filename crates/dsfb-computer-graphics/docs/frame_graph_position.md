# Frame Graph Position

## Where DSFB supervision sits in the render pipeline

The DSFB supervisory pass executes between the TAA reprojection pass and the TAA resolve pass.

## Pass Ordering

1. GBuffer pass — produces `current_color` (pre-TAA SceneColor), depth, normals
2. Velocity pass — produces `motion_vectors` (VelocityBuffer)
3. TAA reprojection — reprojects history into current-frame space, produces `reprojected_history`
4. **[DSFB supervision pass — this crate]** — reads from outputs of steps 1–3, writes `trust`/`alpha`/`intervention`
5. TAA resolve — blends current + `reprojected_history` using DSFB-modulated `alpha`

## Resource Dependencies

**Inputs (SRV / read-only):**

- `current_color`: written by GBuffer pass (UAV write → SRV read transition required)
- `reprojected_history`: written by TAA reprojection (UAV write → SRV read transition required)
- `motion_vectors`: written by Velocity pass (UAV write → SRV read transition required)
- `depth_pairs`: current depth from GBuffer, reprojected depth from TAA reprojection
- `normal_pairs`: current normals from GBuffer, reprojected normals from TAA reprojection

**Outputs (UAV / read-write):**

- `trust_out`: consumed by TAA resolve and optionally by logging / routing passes
- `alpha_out`: consumed by TAA resolve as per-pixel blend weight
- `intervention_out`: consumed by diagnostic logging only (not required for resolve)

## Barrier Requirements (Vulkan / DX12)

**Before dispatch:**

- Pipeline barrier: `srcStageMask` = `ALL_GRAPHICS` (or `COMPUTE_SHADER` if reprojection is compute), `dstStageMask` = `COMPUTE_SHADER`
- Access transition: `current_color`, `reprojected_history`, `depth_pairs`, `normal_pairs` from `SHADER_READ` → `SHADER_READ` (no transition needed if already in read state)
- `motion_vectors`: same as above

**After dispatch:**

- Pipeline barrier: `srcStageMask` = `COMPUTE_SHADER`, `dstStageMask` = `FRAGMENT_SHADER` or `COMPUTE_SHADER` (whichever runs TAA resolve)
- `trust_out`, `alpha_out`: UAV write → SRV read transition for the resolve pass

## Async Compute Compatibility

The DSFB supervision pass has no dependency on shadow map generation, SSAO, bloom, or any post-processing pass. Its only dependencies are GBuffer outputs, velocity, and the reprojected history. This means it is a valid async-compute candidate that can overlap with:

- Shadow map rendering (no shared resources)
- Environment probes (no shared resources)
- SSAO/GTAO computation (no shared resources)

The GPU kernel uses only storage buffers with no render targets or framebuffer attachments. It has no implicit sync points with the graphics queue unless the runtime inserts one due to resource state transitions. In Vulkan, explicit barriers are required as described above; in DX12, `ResourceBarrier()` calls handle the same semantics.

## No CPU Stall Required

The minimum kernel does not require CPU-side readback during normal operation. The current wgpu evaluation implementation uses `pollster::block_on()` for parity validation only — this sync point is for correctness testing and is not part of the production integration path. In a shipping engine, `trust_out` and `alpha_out` are consumed on the GPU by the subsequent resolve pass without any CPU involvement.

## Frame Graph Pseudocode (Unreal RDG Style)

```cpp
// After GBuffer and Velocity passes, before TAA resolve:
FRDGBufferRef TrustBuffer = GraphBuilder.CreateBuffer(
    FRDGBufferDesc::CreateStructuredDesc(sizeof(float), NumPixels), TEXT("DSFB.Trust"));
FRDGBufferRef AlphaBuffer = GraphBuilder.CreateBuffer(
    FRDGBufferDesc::CreateStructuredDesc(sizeof(float), NumPixels), TEXT("DSFB.Alpha"));

AddPass(GraphBuilder, RDG_EVENT_NAME("DSFB.Supervision"),
    [SceneColor, HistoryBuffer, VelocityBuffer, DepthBuffer, NormalBuffer,
     TrustBuffer, AlphaBuffer](FRHICommandList& RHICmdList) {
        // Dispatch DSFB_HOST_MINIMUM compute shader
        // 8x8 workgroups, ceil(Width/8) x ceil(Height/8) dispatch
    });

// Existing TAA resolve now reads AlphaBuffer instead of using fixed alpha:
TAAResolvePass.SetInput("PerPixelAlpha", AlphaBuffer);
```
