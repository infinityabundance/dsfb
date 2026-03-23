# Engine-Native Validation Report

ENGINE_NATIVE_CAPTURE_MISSING=true

## 1. Engine Source Category

**engine_type:** pending
**status:** pending — no real capture provided

## 2. Exact Buffers Provided

| Buffer | Required | Present | Quality |
|--------|----------|---------|---------|
| current_color | required | no | unavailable |
| history_color | required | no | unavailable |
| motion_vectors | required | no | unavailable |
| current_depth | required | no | unavailable |
| history_depth | optional | no | unavailable |
| current_normals | required | no | unavailable |
| history_normals | optional | no | unavailable |
| roi_mask | optional | no | unavailable |
| jitter | optional | no | unavailable |
| exposure | optional | no | unavailable |
| camera_matrices | optional | no | unavailable |
| history_validity_mask | optional | no | unavailable |

## 3. GPU Execution Summary

**measured_gpu:** false
**status:** pending
**kernel:** dsfb_host_minimum
**backend:** Vulkan (wgpu 0.19)
**reference (DAVIS/Sintel):** ~4 ms dispatch at 854×480 and 1024×436 on RTX 4080 SUPER

## 4. Demo A Results

**status:** pending
**ROI/non-ROI:** separated
**proxy vs ground truth:** proxy (no renderer ground truth available)

## 5. Demo B Results

**status:** pending
**baselines:** uniform, gradient, contrast, variance, combined_heuristic, DSFB imported trust, hybrid
**fixed_budget_equal:** true
**renderer_integrated_sampling:** false (proxy allocation only)

## 6. Mixed-Regime Status

**engine-native mixed-regime:** not_confirmed (capture pending)
**internal confirmation:** mixed_regime_confirmed_internal — see `generated/mixed_regime_confirmation_report.md`

## 7. High-Resolution Status

**1080p:** confirmed (reference measurement: ~18 ms on RTX 4080 SUPER)
**4K:** OOM — binding size limit exceeded (~265 MB required, 134 MB max)
**tiling:** designed, not yet tested at 4K
**classification:** external environment limitation (not an algorithm limitation)

## 8. What Is Proven Now

- DSFB engine-native pipeline is fully wired and operational
- Same replay path as DAVIS/Sintel — no special-case engine-native path
- Schema, manifest, import, replay, GPU, Demo A, Demo B all gated
- Internal mixed-regime case confirmed (aliasing + variance co-active)
- GPU path proven on DAVIS/Sintel at comparable resolution
- 1080p dispatch proven; 4K blocked by environment binding limit

## 9. What Is Still Not Proven

- GPU timing on real engine-native buffers (pending capture)
- Demo A/B metrics on real engine-native buffers (pending capture)
- Mixed-regime on engine-native data (pending appropriate scene)
- Ground-truth comparison (pending renderer reference export)
- Renderer-integrated sampling (pending engine integration)
- 4K dispatch on real engine buffers (pending capture + tiling wiring)

## 10. Remaining Blockers

| Blocker | Type | Resolution |
|---------|------|-----------|
| No real engine capture provided | **EXTERNAL** | Export via playbook, update manifest |
| Ground-truth reference unavailable | **EXTERNAL** | Export from renderer |
| Mixed-regime on engine-native data | **EXTERNAL** | Requires appropriate scene |
| Renderer-integrated sampling | **EXTERNAL** | Engine integration work |
| 4K OOM (binding limit) | **EXTERNAL env** | Tiling wired, needs real 4K capture |

## 11. Exact Next Highest-Value Experiment

**Export one frame pair from Unreal Engine** (current + history color, motion vectors, depth, normals) following `docs/unreal_export_playbook.md`. Update `examples/engine_native_capture_manifest.json` with `engine_type: unreal` and real buffer paths. Run:
```bash
cargo run --release -- run-engine-native-replay \
  --manifest examples/engine_native_capture_manifest.json \
  --output generated/engine_native
```
This single step closes all ENGINE_NATIVE_CAPTURE_MISSING gates at once.

## What Is Not Proven

- All engine-native metrics are pending the real capture (sections 3–6 above)

## Remaining Blockers

- **EXTERNAL**: Real engine capture is the single highest-value remaining step.
- All internal infrastructure is complete and gated.
