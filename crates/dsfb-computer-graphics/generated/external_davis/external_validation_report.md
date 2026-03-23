# External Validation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Data Description

- source_kind: `davis_2017_real_video`
- captures: `3`
- real_external_data_provided: `true`
- synthetic vs real: `real external data`

## Pipeline Description

- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite.
- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present.

## GPU Execution Summary

- measured_gpu: `true`
- kernel: `dsfb_host_minimum`
- capture `dance-twirl_frame_0079`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.5546, dispatch_ms = 3.4461, readback_ms = 1.1068
- capture `soapbox_frame_0069`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.0781, dispatch_ms = 3.5221, readback_ms = 0.5539
- capture `camel_frame_0020`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.0195, dispatch_ms = 3.4535, readback_ms = 0.5642

## Demo A Results

- `dance-twirl_frame_0079`: ROI source = `manifest_mask`, ROI pixels = 54219, metric_source = `proxy_current_vs_history`
  - Fixed alpha baseline: ROI MAE = 0.04913, non-ROI MAE = 0.00841, temporal accumulation = 0.01380, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.02146, non-ROI MAE = 0.04102, temporal accumulation = 0.03843, intervention rate = 0.16463
  - DSFB host minimum: ROI MAE = 0.02877, non-ROI MAE = 0.03509, temporal accumulation = 0.03426, intervention rate = 0.14198
- `soapbox_frame_0069`: ROI source = `manifest_mask`, ROI pixels = 99358, metric_source = `proxy_current_vs_history`
  - Fixed alpha baseline: ROI MAE = 0.06254, non-ROI MAE = 0.01058, temporal accumulation = 0.02316, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.02035, non-ROI MAE = 0.07045, temporal accumulation = 0.05831, intervention rate = 0.25879
  - DSFB host minimum: ROI MAE = 0.02977, non-ROI MAE = 0.05494, temporal accumulation = 0.04882, intervention rate = 0.20289
- `camel_frame_0020`: ROI source = `manifest_mask`, ROI pixels = 63015, metric_source = `proxy_current_vs_history`
  - Fixed alpha baseline: ROI MAE = 0.02066, non-ROI MAE = 0.00537, temporal accumulation = 0.00772, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.01321, non-ROI MAE = 0.01727, temporal accumulation = 0.01664, intervention rate = 0.07675
  - DSFB host minimum: ROI MAE = 0.01443, non-ROI MAE = 0.01440, temporal accumulation = 0.01441, intervention rate = 0.06533

## Demo B Results

- `dance-twirl_frame_0079`: regime = `aliasing_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.17679, global error = 0.19889, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.17441, global error = 0.19342, ROI mean spp = 1.970
  - Contrast-based: ROI error = 0.17682, global error = 0.18889, ROI mean spp = 1.854
  - Variance proxy: ROI error = 0.17680, global error = 0.18521, ROI mean spp = 1.774
  - Combined heuristic: ROI error = 0.16815, global error = 0.17977, ROI mean spp = 1.802
  - DSFB imported trust: ROI error = 0.17020, global error = 0.18716, ROI mean spp = 1.901
  - Hybrid trust + variance: ROI error = 0.17649, global error = 0.18769, ROI mean spp = 1.824
- `soapbox_frame_0069`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.17879, global error = 0.18474, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.16284, global error = 0.18108, ROI mean spp = 2.242
  - Contrast-based: ROI error = 0.15760, global error = 0.17903, ROI mean spp = 2.310
  - Variance proxy: ROI error = 0.16733, global error = 0.16495, ROI mean spp = 1.832
  - Combined heuristic: ROI error = 0.15402, global error = 0.16361, ROI mean spp = 2.061
  - DSFB imported trust: ROI error = 0.16658, global error = 0.16627, ROI mean spp = 1.846
  - Hybrid trust + variance: ROI error = 0.16990, global error = 0.16706, ROI mean spp = 1.818
- `camel_frame_0020`: regime = `aliasing_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.10395, global error = 0.17502, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.11209, global error = 0.16566, ROI mean spp = 1.447
  - Contrast-based: ROI error = 0.10926, global error = 0.16150, ROI mean spp = 1.469
  - Variance proxy: ROI error = 0.11026, global error = 0.16651, ROI mean spp = 1.508
  - Combined heuristic: ROI error = 0.10652, global error = 0.15854, ROI mean spp = 1.475
  - DSFB imported trust: ROI error = 0.09703, global error = 0.16865, ROI mean spp = 2.163
  - Hybrid trust + variance: ROI error = 0.10636, global error = 0.16689, ROI mean spp = 1.633

## Scaling / Coverage Summary

- attempted_1080p: `true`
- attempted_4k: `true`
- realism_stress_case: `true`
- larger_roi_case: `true`
- mixed_regime_case: `false`
- coverage_status: `partial`
- missing coverage labels: mixed_regime_case

## What Is Proven

- The crate can ingest external buffers through a strict manifest and run the DSFB host-minimum supervisory layer on them.
- The same GPU kernel can execute on imported buffers, with explicit measured-vs-unmeasured disclosure.
- ROI vs non-ROI reporting survives the external path, and Demo B keeps equal budgets across stronger heuristic baselines.

## What Is Not Proven

- This report does not prove production-scene generalization.
- It does not prove engine integration unless real exported buffers are supplied.
- Demo B on imported captures remains an allocation proxy, not a renderer-integrated sampling benchmark.

## Remaining Blockers

- real external engine captures
- engine-side GPU profiling on imported buffers
- renderer-integrated Demo B replay with per-sample budgets

## Next Required Experiment

Export one real frame pair plus an ROI/mask disclosure from an engine into the external schema, run `run-external-replay` on the target GPU, and compare fixed alpha, strong heuristic, and DSFB on the same imported capture.

## Signal Quality Assessment

This section documents the confidence level of each structural signal used in the DAVIS external replay.

| Signal | Source in DAVIS replay | Confidence | Notes |
|--------|----------------------|------------|-------|
| current_color | Extracted from DAVIS frame directly | High | Real video pixels; no derivation |
| reprojected_history | Bilinear warp of previous frame using derived motion | Medium | Warp uses derived motion; real history buffer would differ |
| motion_vectors | Block-matching optical flow (ECC + Lucas-Kanade) | Low-Medium | Not real renderer motion vectors; subpixel precision limited |
| depth (current) | Relative depth from edge/gradient proxy | Low | Not real depth buffer; edge magnitude used as proxy |
| depth (reprojected) | Derived from previous-frame depth proxy | Low | Doubly-derived; not real reprojected depth buffer |
| normal (current) | Computed from depth proxy gradient | Low | Derived from low-confidence depth; not real GBuffer normals |
| normal (reprojected) | Computed from reprojected depth proxy gradient | Very Low | Doubly-derived from two low-confidence sources |

**Summary:** DAVIS provides real video content (high confidence) and bilinear reprojection (medium confidence), but structural signals (depth, normals) are derived proxies with low confidence. The depth and normal gates in the DSFB minimum kernel operate on these proxies rather than real GBuffer buffers.

**Implication for gate accuracy:** The depth_gate and normal_gate computations in the minimum kernel receive low-confidence inputs on DAVIS captures. This means the structural disagreement gate may fire on spurious depth/normal mismatches from derivation error rather than real geometry discontinuities. Results should be interpreted as "proxy-validated" rather than "structurally validated."

## What Sintel Closes vs What Engine Capture Closes

| Signal gap | Sintel closes? | Engine capture closes? |
|------------|---------------|----------------------|
| Real video content | No (synthetic renderer) | Yes |
| Ground-truth depth | Yes (MPI-Sintel ground truth) | Yes |
| Ground-truth normals | Yes (MPI-Sintel ground truth) | Yes |
| Ground-truth motion vectors | Yes (MPI-Sintel ground truth) | Yes |
| Real TAA history buffer | No (warped from ground-truth) | Yes |
| Real specular BRDF | No (Blender materials) | Yes |
| Real subpixel TAA jitter | No (not present in Sintel) | Yes |
| Real reprojection noise | No (clean warp) | Yes |
| Real disocclusion events | Partial (depth-change events) | Yes |
| Real engine scheduler context | No | Yes |

Sintel narrows the structural signal gap (depth, normals, motion vectors) relative to DAVIS, but does not close the real-engine gap for history buffer, specular structure, TAA jitter, or reprojection noise. The engine-realistic synthetic bridge (see `generated/engine_realistic/`) further narrows the TAA jitter and reprojection noise gap synthetically.
