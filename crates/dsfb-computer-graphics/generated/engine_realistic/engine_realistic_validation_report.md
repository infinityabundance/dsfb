# Engine-Realistic Synthetic Bridge Report

> "The experiment is intended to demonstrate behavioral differences rather than establish optimal performance."

**SYNTHETIC_ENGINE_REALISTIC=true**
**ENGINE_NATIVE_CAPTURE_MISSING=true**

This report documents a synthetically generated scene designed to mimic real-engine TAA frame structure at 1920×1080. It is NOT a real engine capture. It uses synthetic geometry and procedural motion to approximate real-engine artifacts.

## Scene Design

The engine-realistic synthetic scene simulates the following real-engine artifacts:

| Artifact | Simulation Method | Why |
|----------|------------------|-----|
| GBuffer-realistic depth | Perspective projection with 3 layers (bg z=100, mg z=20, fg z=5) + sine noise | Matches real depth buffer structure with discontinuities at material edges |
| GBuffer-realistic normals | View-space normals consistent with depth; foreground curved 30–45° off axis | Matches GBuffer normal encoding for curved surfaces |
| Subpixel motion vectors | Layer-based pan (bg: 3px, mg: 1px, fg: 5px) + Halton ±0.3px jitter | Simulates real motion vector imprecision |
| Reprojection noise | Per-pixel noise N(0, 0.5px) at edges, N(0, 0.1px) in flat regions | Creates realistic residual concentration at edges |
| TAA jitter | 2×2 Halton subpixel shift on current frame | Simulates raw TAA-jittered input |
| Specular flickering | 40×40 pixel highlight in midground, period=3.0 frames | Creates high-frequency temporal variation |
| Thin geometry | 2 vertical 1px lines + 1 diagonal 1px line at foreground boundary | Aliasing-pressure structures for Demo A |
| Disocclusion event | Foreground moves right at frame 5 revealing 50px+ background band | Onset event for Demo A ROI |
| Ground-truth reference | Current-frame color without TAA jitter | Used for Demo A error measurement |

Resolution: 1920×1080
Frame index (onset): 5
ROI pixels: 31185 / 2073600 (1.5%)

## What This Closes

| Panel Objection | Evidence Provided | Closure Status |
|-----------------|------------------|----------------|
| No real engine data | 1080p synthetic scene with GPU-measured dispatch timing | Narrows gap; real capture still required |
| Show me 4K dispatch | wgpu limit raised, 4K probe executed — see gpu_execution_report.md | Architecture closed |
| Show me where it sits in frame graph | docs/frame_graph_position.md: pass ordering, barriers, RDG pseudocode | Documentation closed |
| Show me it doesn't stall async | docs/async_compute_analysis.md: no CPU sync in production | Architecture closed |
| Motion disagree in cost model | Removed from minimum kernel; binding dropped | Code closed |
| LDS optimization | var<workgroup> tile added, color reads reduced ~1.6/pixel for gates | Code closed |
| Mixed regime | Both aliasing (thin geometry) and variance (specular flicker) in same ROI | Synthetic confirmation |
| Demo B not in renderer | docs/demo_b_production_integration.md: exact integration hook | Documentation closed |
| DAVIS weak signals | Signal quality assessment added to external_validation_report.md | Documentation closed |

## What This Does NOT Close

- **Real engine reprojection error**: Synthetic reprojection noise does not replicate real TAA history buffer jitter and blend artifacts.
- **Real production content**: Synthetic geometry is not real scene content.
- **Real pipeline scheduling**: Synthetic data does not verify async queue overlap in a live engine frame graph.
- **Real specular structure**: Procedural flickering does not replicate real BRDF specular behavior.

## GPU Timing at 1080p

GPU dispatch at 1920×1080: 12.090 ms (adapter: NVIDIA GeForce RTX 4080 SUPER)

## Demo A Results

DSFB supervision on 1920×1080 engine-realistic capture.
ROI pixel count: 31185 (1.5% of frame).
Mean DSFB trust in ROI: 0.2032 (low trust = intervention, expected).
Mean DSFB trust outside ROI: 0.9862 (high trust = no intervention, expected).
Trust enrichment (low trust concentration in ROI vs non-ROI): 57.61×.
SYNTHETIC_ENGINE_REALISTIC=true. ENGINE_NATIVE_CAPTURE_MISSING=true.

## Demo B Results

Demo B (fixed-budget allocation) on the specular-flicker region (high-frequency midground highlight).
The specular region has high temporal variance, which DSFB correctly identifies as a hard region.
DSFB allocates more samples to the specular ROI vs uniform allocation under equal total budget.
Quantitative Demo B results available via `run-demo-b` on the internal suite.
Engine-realistic Demo B integration: trust signal validates correctly on simulated specular content.
SYNTHETIC_ENGINE_REALISTIC=true. ENGINE_NATIVE_CAPTURE_MISSING=true.

## Frame Graph Analysis

The DSFB supervision pass is positioned between TAA reprojection and TAA resolve. See `docs/frame_graph_position.md` for complete barrier specifications, async compatibility analysis, and Unreal RDG pseudocode.

The supervision pass has no CPU stall requirement in production. See `docs/async_compute_analysis.md` for the explicit no-stall analysis.

## LDS Optimization Impact

The GPU kernel now uses `var<workgroup> tile: array<f32, 100>` for 8×8 workgroup shared memory caching of the 3×3 neighborhood gates. This reduces color texture reads from 16/pixel to approximately 1.6/pixel for the `neighborhood_gate` and `local_contrast_gate` computations.

## What Is Not Proven

- Real engine reprojection error (synthetic noise does not replicate real TAA history buffer jitter)
- Real production content generalization (synthetic geometry only)
- Real pipeline scheduling (no live engine frame graph measurement)

## Remaining Blockers

- One real engine capture via `docs/unreal_export_playbook.md` or `docs/unity_export_playbook.md`
- NSight/PIX profiling to confirm async overlap
- Real TAA history buffer reprojection error measurement
