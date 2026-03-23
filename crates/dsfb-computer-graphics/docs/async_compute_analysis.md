# Async Compute Analysis

## Does the DSFB Supervision Pass Stall Async?

**Short answer:** No CPU stall is required in production. The `pollster::block_on()` call in the wgpu evaluation harness is a correctness-testing sync point, not a production requirement.

## Resource Transition States

### Before the DSFB Supervision Dispatch

All input buffers must be in a readable state before the compute dispatch begins.

**Vulkan resource transitions (before dispatch):**

```
VkImageMemoryBarrier / VkBufferMemoryBarrier:
  srcStageMask:  VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT (or COMPUTE if TAA reprojection is compute)
  dstStageMask:  VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT
  srcAccessMask: VK_ACCESS_SHADER_WRITE_BIT
  dstAccessMask: VK_ACCESS_SHADER_READ_BIT

Buffers requiring this transition:
  - current_color      (written by GBuffer/shading pass)
  - reprojected_history (written by TAA reprojection compute)
  - motion_vectors     (written by velocity pass)
  - depth_pairs        (written by GBuffer depth pass)
  - normal_pairs       (written by GBuffer normal pass)
```

**DX12 equivalent:**
```
ResourceBarrier(D3D12_RESOURCE_STATE_UNORDERED_ACCESS → D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE)
for each input buffer listed above.
```

### After the DSFB Supervision Dispatch

Output buffers must transition to a readable state before the TAA resolve pass consumes them.

**Vulkan resource transitions (after dispatch):**

```
VkBufferMemoryBarrier:
  srcStageMask:  VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT
  dstStageMask:  VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT (or COMPUTE_SHADER for compute resolve)
  srcAccessMask: VK_ACCESS_SHADER_WRITE_BIT
  dstAccessMask: VK_ACCESS_SHADER_READ_BIT

Buffers requiring this transition:
  - trust_out    (consumed by TAA resolve as blend weight source)
  - alpha_out    (consumed by TAA resolve as per-pixel blend weight)
```

**Note:** `intervention_out` is consumed only by diagnostic logging, not the resolve pass. Its barrier timing is less critical.

## Why No CPU Stall Is Needed in Production

The DSFB supervision kernel:

1. **Reads only GPU-resident inputs** — all inputs (current color, history, depth, normals, motion vectors) are produced by preceding GPU passes and remain on the GPU.
2. **Writes only GPU-resident outputs** — `trust_out` and `alpha_out` are consumed by the subsequent TAA resolve pass on the GPU without any CPU readback.
3. **Has no render targets** — the kernel is a pure storage buffer compute pass with no framebuffer attachments. No implicit GPU-CPU synchronization is triggered.
4. **The wgpu `pollster::block_on()` is evaluation-only** — in the crate's test harness, `pollster::block_on()` blocks the CPU thread until the GPU completes, then reads back outputs for CPU-side parity validation. This sync point exists only to verify correctness. In a production engine, the resolve pass reads `alpha_out` directly without CPU involvement.

**Explicit statement:** The wgpu harness blocks for parity validation. A production integration dispatches the kernel, inserts barriers, and continues without CPU involvement until the frame is complete.

## Scheduling on Async Compute Queue (DX12/Vulkan)

The DSFB supervision pass qualifies for async compute scheduling because:

- **No render target writes** — does not need to be on the graphics queue
- **No dependency on shadow maps, SSAO, or bloom** — can start as soon as GBuffer, velocity, and TAA reprojection are complete
- **Dependency chain is short** — GBuffer → Velocity → TAA reprojection → [async: DSFB supervision] → TAA resolve

**Candidate overlap:**

```
Graphics queue:   [GBuffer] → [Velocity] → [TAA reprojection] → ... → [TAA resolve]
                                                  ↓ signal
Async compute:                                [DSFB supervision] → signal →
```

The async compute queue begins after receiving a signal from the TAA reprojection pass. After DSFB supervision completes, it signals the graphics queue so TAA resolve can proceed.

This overlap eliminates the serialization cost of inserting a compute pass on the graphics queue between TAA reprojection and TAA resolve.

## What This Analysis Does NOT Prove

This analysis proves the architectural absence of CPU stall requirements. It does not prove:

- Measured async overlap via GPU trace (requires NSight/PIX profiling on a real engine)
- That a specific driver or runtime will not insert implicit synchronization
- Optimal queue scheduling for a specific engine's frame graph topology

Engine-integrated GPU profiling with NSight or PIX is required to confirm the async overlap in practice.

## The wgpu Evaluation Harness

The current evaluation harness uses `pollster::block_on()` which is a blocking CPU synchronization primitive:

```rust
pub fn try_execute_host_minimum_kernel(...) -> Result<Option<GpuKernelResult>> {
    pollster::block_on(try_execute_host_minimum_kernel_async(...))
}
```

This sync point is present only for:
1. Correctness validation — comparing GPU outputs against CPU reference
2. Timing measurement — measuring end-to-end dispatch time in isolation

It is not part of the production integration path and would be removed in an engine integration where the resolve pass consumes outputs directly on the GPU.

## What Is Not Proven

- Measured async overlap confirmation via GPU trace (NSight/PIX required)
- That a specific driver will not insert implicit synchronization
- Optimal queue scheduling for a specific engine's frame graph topology

## Remaining Blockers

- Real engine capture with GPU profiling (NSight/PIX) to confirm async overlap in practice
- Renderer team participation for in-queue scheduling validation
