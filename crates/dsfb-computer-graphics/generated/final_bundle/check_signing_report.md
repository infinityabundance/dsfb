# Check-Signing Evidence Report

> "The experiment is intended to demonstrate behavioral differences rather than establish optimal performance."

This report maps each reviewer objection to current evidence. It does not claim objections are fully closed where they are not. It states precisely what the evidence shows and what it does not show.

## Objection 1: No real engine data

**Evidence available:**
- Engine-realistic synthetic capture at 1920×1080 with GPU-measured dispatch timing (see `generated/engine_realistic/engine_realistic_validation_report.md`)
- DAVIS external replay (3 captures, real video, proxy structural signals): `generated/external_davis/`
- Sintel external replay (5 captures, ground-truth reference, synthetic renderer): `generated/external_sintel/` (if run)
- Engine-native infrastructure complete: manifest schema, CLI (`run-engine-native-replay`), playbooks for Unreal/Unity/custom renderer

**What this closes:** Pipeline execution on non-trivial data at production resolution (1080p) with measured GPU timing.

**What remains open:** Real renderer reprojection error, real production content, real TAA scheduling. Closes when a real engine capture is provided.

**Exact next step:** `cargo run --release -- run-engine-native-replay --manifest examples/engine_native_capture_manifest.json --output generated/engine_native`

## Objection 2: Demo B not renderer-integrated

**Evidence available:**
- `docs/demo_b_production_integration.md` — exact integration hook in Vulkan/DX12, what a renderer team would need, what would be measured
- Demo B trust signal validated on engine-realistic synthetic specular-flicker region
- Fixed-budget allocation policy comparison is complete in internal suite

**What this closes:** Exact integration design with code-level specificity. Trust signal validated on specular content.

**What remains open:** In-renderer measurement of ROI vs non-ROI sample distribution on real content. Closes when renderer team provides access to pre-denoiser sample budget allocation code.

**Exact next step:** Share `docs/demo_b_production_integration.md` with renderer team and provide `trust_out` buffer format spec.

## Objection 3: Show me Unreal or internal renderer buffers

**Evidence available:**
- `docs/unreal_export_playbook.md` — step-by-step Unreal RenderDoc export procedure
- `docs/unity_export_playbook.md` — step-by-step Unity export procedure
- `docs/custom_renderer_export_playbook.md` — custom renderer integration
- `examples/engine_native_capture_manifest.json` — manifest format for engine-native capture
- Engine-realistic synthetic buffers at 1080p follow the same schema

**What this closes:** Complete playbook and manifest infrastructure — a renderer team can provide buffers with no code changes to this crate.

**What remains open:** Actual Unreal or internal renderer buffers. Closes when a capture is provided.

**Exact next step:** Follow `docs/unreal_export_playbook.md` on any Unreal project with TAA enabled.

## Objection 4: Show me where this sits in the frame graph

**Evidence available:**
- `docs/frame_graph_position.md` — complete pass ordering, resource dependencies, barrier requirements (`srcStageMask`/`dstStageMask`), Unreal RDG pseudocode
- `docs/async_compute_analysis.md` — barrier semantics, async scheduling analysis, no-stall proof

**What this closes:** Complete technical specification of frame graph insertion, barrier requirements, and async compatibility.

**What remains open:** Actual RenderDoc/PIX capture showing the pass in a live frame timeline. Closes when real engine capture is provided.

## Objection 5: Show me it doesn't stall async

**Evidence available:**
- `docs/async_compute_analysis.md` — explicit analysis showing no CPU sync point required in production
- Minimum kernel has no render targets, no framebuffer attachments, pure compute
- All dependencies (current_color, history, depth, normals) are GPU-resident inputs
- Outputs (trust, alpha, intervention) are GPU-resident and consumed by the subsequent resolve pass
- `pollster::block_on()` in the wgpu evaluation harness is evaluation-only, not production

**What this closes:** Architectural proof that no CPU stall is required.

**What remains open:** Measured async overlap confirmation via GPU trace (NSight/PIX). Closes when real engine capture is provided with profiling.

## Objection 6: 4K dispatch proof

**Evidence available:**
- wgpu binding limit raised to `u32::MAX` — removes 134 MB binding cap
- 4K synthetic probe (3840×2160 zero-filled buffers) executed via `run-gpu-path` — see `generated/final_bundle/gpu_execution_report.md` for `gpu_4k_synthetic_probe` row
- 8×8 workgroup tiling is resolution-independent by design (dispatch is `ceil(W/8) × ceil(H/8)`)

**What this closes:** Architecture limit removed; dispatch feasibility tested at 4K resolution with real wgpu path.

**What remains open:** 4K with real engine buffers (real content, production memory pressure, real adapter). Closes when real 4K engine capture is provided.

## Objection 7: Motion disagreement in cost model despite no benefit

**Evidence available:**
- Motion vectors binding completely removed from minimum GPU kernel (`@group(0) @binding(2)` dropped)
- `let _unused_motion` line removed
- Minimum kernel binding count reduced from 9 to 8 bindings
- Motion disagreement remains as `motion_augmented` optional extension only, reported separately
- Cost model updated to reflect minimum-path-only binding count

**What this closes:** The minimum path no longer pays for motion disagreement. Cost model correctly reflects minimum binding set.

**What remains open:** Nothing for this objection. It is closed.

## Objection 8: Real engine memory access pattern

**Evidence available:**
- LDS-optimized kernel uses `var<workgroup> tile: array<f32, 100>` — 10×10 luma cache for 8×8 workgroup
- Color texture reads for neighborhood gates reduced from 16/pixel to ~1.6/pixel
- Engine-realistic 1080p dispatch timing: see `generated/engine_realistic/engine_realistic_validation_report.md`

**What this closes:** LDS optimization directly addresses the cache-thrash concern for the 3×3 neighborhood gates. Dispatch timing at 1080p is measured.

**What remains open:** NSight L1/L2 cache counter data on the target GPU with real engine buffer layout. Closes when real engine capture is provided with hardware profiling.

## Objection 9: DAVIS = weak structural signals

**Evidence available:**
- Signal quality assessment in `generated/external_davis/external_validation_report.md` — documents which signals are derived-low-confidence (block-matching motion, relative-depth, doubly-derived normals)
- Sintel provides ground-truth depth, normals, and motion vectors as the stronger structural signal dataset
- Table in DAVIS report: "What Sintel closes vs what engine capture closes"

**What this closes:** Honest disclosure of DAVIS signal quality and implications for gate accuracy. Sintel partially closes the structural signal gap.

**What remains open:** Real engine structural signals (real reprojection error, real subpixel motion, real specular structure). Closes when real engine capture is provided.

## Objection 10: Sintel = proxy renderer, not real pipeline

**Evidence available:**
- Engine-realistic synthetic scene simulates real-engine artifacts that Sintel lacks: TAA jitter, reprojection noise at depth discontinuities, specular flickering, subpixel motion vector noise, disocclusion events
- Explicit per-artifact mapping in `generated/engine_realistic/engine_realistic_validation_report.md` (Scene Design table)

**What this closes:** Narrower gap between synthetic evidence and real-engine behavior via engine-realistic bridge.

**What remains open:** Real engine TAA history buffer, real production content, real pipeline scheduling. Closes when real engine capture is provided.

## Objection 11: Mixed regime not fully demonstrated

**Evidence available:**
- Internal confirmation computed from actual pixel signals on `noisy_reprojection` scenario: aliasing enrichment 2.31×, variance enrichment 3.63× — see `generated/mixed_regime_confirmation_report.md`
- Confirmed from pixel-level signal computation on actual scenario data, not from architectural claims
- Engine-realistic scene contains both aliasing-pressure (thin geometry at disocclusion band) and variance-pressure (specular flickering, subpixel reprojection noise) co-active in the same ROI frame

**What this closes:** Internal confirmation is from measured signal values. Engine-realistic synthetic closes "show me both signals simultaneously" at 1080p.

**What remains open:** Engine-native mixed-regime confirmation. Closes when a thin-geometry-under-motion real engine capture is provided.

## Objection 12: Won't sign until we see it on a real pipeline

**Summary table:**

| Item | Internal closes? | External closes? | Status |
|------|-----------------|------------------|--------|
| Frame graph position | ✓ (docs/frame_graph_position.md) | — | Closed |
| Async stall proof | ✓ (docs/async_compute_analysis.md) | PIX/NSight trace | Architecture closed |
| Motion disagree removed | ✓ (kernel binding dropped) | — | Closed |
| 4K dispatch | ✓ (limit fix + probe) | Real 4K capture | Architecture closed |
| LDS optimization | ✓ (workgroup tile) | NSight benchmark delta | Code closed |
| GPU timing 1080p | ✓ (engine-realistic) | Engine capture | Measured (synthetic) |
| DAVIS signal quality | ✓ (documented) | Real engine depth/normals | Documented |
| Sintel proxy gap | ✓ (engine-realistic narrows it) | Real engine capture | Partially closed |
| Mixed regime | ✓ (internal + engine-realistic) | Engine thin-geometry capture | Partially closed |
| Real engine buffers | — | ✓ one capture | Open |
| Demo B in renderer | — | ✓ renderer team | Open |
| Real memory pattern | — | ✓ NSight profile | Open |

**The single remaining step:** One real engine capture following `docs/unreal_export_playbook.md` or `docs/unity_export_playbook.md`. All internal infrastructure is complete and gated. The pipeline is waiting.

```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```

## What Is Not Proven

- Real engine reprojection error on production content
- Async overlap measurement in a live engine frame graph (NSight/PIX required)
- Demo B in-renderer integration (renderer team participation required)
- Real production memory access pattern (NSight/PIX profiling required)

## Remaining Blockers

All internal items are closed. The remaining blockers are:

1. Real engine capture (Unreal, Unity, or custom renderer following the playbooks)
2. Engine-integrated GPU profiling (NSight/PIX) to confirm async overlap
3. Demo B in-renderer integration (renderer team participation required)
