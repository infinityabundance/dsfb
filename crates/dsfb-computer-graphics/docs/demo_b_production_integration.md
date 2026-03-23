# Demo B Production Integration

> "The experiment is intended to demonstrate behavioral differences rather than establish optimal performance."

This document specifies exactly how Demo B (fixed-budget allocation) would be integrated into a production renderer. It provides code-level specificity for the renderer team.

## 1. What Demo B Measures

Demo B measures whether the DSFB `trust_out` signal correctly identifies high-difficulty pixels (low trust, high hazard) and allows a renderer to concentrate sample budget on those pixels under a fixed total budget constraint.

**The core question:** Given a fixed total number of samples to allocate across all pixels in a frame, does DSFB-guided allocation reduce reconstruction error compared to uniform allocation?

**Internal evidence:** On the internal suite, DSFB-guided allocation reduces ROI reconstruction error by 10–25% vs uniform allocation under equal total budget. Strong heuristic baselines are competitive on some scenarios.

**What production integration adds:** Real pre-denoiser sample budget allocation, real sample distribution measurement, real reconstruction quality measurement on live rendering.

## 2. The Integration Hook

The DSFB supervision pass outputs `trust_out` (per-pixel, float32, range [0,1]) and `alpha_out` (per-pixel, float32). The Demo B integration hook is in the **pre-denoiser sample budget allocation** pass.

**Integration point in frame graph:**

```
GBuffer → Velocity → TAA reprojection → DSFB supervision → [Demo B hook] → Denoiser → TAA resolve
```

The Demo B hook reads `trust_out` and redistributes the pre-denoiser sample budget using one of the allocation policies below.

## 3. Allocation Policies

### Policy A: Uniform (baseline)

```
per_pixel_budget[i] = total_budget / num_pixels
```

### Policy B: DSFB-guided (DSFB trust-proportional inverse)

```
hazard[i] = 1.0 - trust_out[i]
weight[i] = 1.0 + hazard[i] * DSFB_BUDGET_SCALE_FACTOR
per_pixel_budget[i] = total_budget * weight[i] / sum(weight)
```

Where `DSFB_BUDGET_SCALE_FACTOR` controls the maximum sample redistribution ratio (default: 3.0, meaning high-hazard pixels can receive up to 3× the uniform budget).

### Policy C: Strong heuristic (edge/gradient-based)

```
gradient[i] = spatial_gradient_magnitude(current_color, i)
weight[i] = 1.0 + gradient[i] * GRADIENT_BUDGET_SCALE_FACTOR
per_pixel_budget[i] = total_budget * weight[i] / sum(weight)
```

### Policy D: Variance-based

```
variance[i] = temporal_variance(current_color[i], reprojected_history[i])
weight[i] = 1.0 + variance[i] * VARIANCE_BUDGET_SCALE_FACTOR
per_pixel_budget[i] = total_budget * weight[i] / sum(weight)
```

## 4. What the Renderer Team Needs to Provide

| Item | Description | Format |
|------|-------------|--------|
| Pre-denoiser sample count buffer | Per-pixel sample count before denoiser runs | uint32 storage buffer, same layout as trust_out |
| Post-denoiser reconstruction quality | Per-pixel RMSE or SSIM vs ground truth | float32 storage buffer |
| ROI mask | Which pixels constitute the "important" region | bool/uint8 buffer or JSON mask |
| Total budget constraint | Fixed total sample count per frame | scalar (uint32) |
| Comparison frame pair | Same scene with uniform vs DSFB-guided allocation | Two rendered frames |

## 5. Expected Measurement Output

The Demo B production integration would produce:

```
Policy          | ROI RMSE | Non-ROI RMSE | Total samples | Budget efficiency (ROI RMSE × budget)
Uniform         | baseline  | baseline     | N             | baseline
DSFB-guided     | reduced   | slightly +   | N (same)      | should be lower
Strong heuristic| ?         | ?            | N (same)      | compare
Variance-based  | ?         | ?            | N (same)      | compare
```

**The key metric:** ROI RMSE reduction under equal total budget. If DSFB-guided allocation achieves lower ROI RMSE than uniform with the same total budget, Demo B is confirmed in production.

## 6. What This Document Does NOT Prove

- It does not prove DSFB-guided allocation will win on all scene types (strong heuristics are competitive on some scenarios)
- It does not provide measured production results (renderer team integration required)
- It does not specify optimal `DSFB_BUDGET_SCALE_FACTOR` (scene-dependent tuning required)

## Vulkan/DX12 Integration Notes

The `trust_out` buffer is a `VK_BUFFER_USAGE_STORAGE_BUFFER_BIT` (Vulkan) or `D3D12_RESOURCE_STATE_UNORDERED_ACCESS` (DX12) buffer. After DSFB supervision completes:

```
// Vulkan barrier
VkBufferMemoryBarrier barrier = {
    .srcStageMask = VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,  // DSFB supervision
    .dstStageMask = VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT,  // Budget allocation pass
    .srcAccessMask = VK_ACCESS_SHADER_WRITE_BIT,
    .dstAccessMask = VK_ACCESS_SHADER_READ_BIT,
    .buffer = trust_out_buffer,
    ...
};
```

The budget allocation pass then reads `trust_out` as a read-only storage buffer and writes to the per-pixel sample count buffer.

## What Is Not Proven

- Production results (renderer team integration required)
- Optimal scale factor for real scene content
- Competitive win on all scene types (strong heuristics remain competitive on some scenarios)

## Remaining Blockers

- Renderer team access to pre-denoiser sample budget allocation code
- One rendered frame pair (uniform vs DSFB-guided allocation) with equal total budget
- Ground-truth reference frame for reconstruction quality measurement
