# External Validation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Data Description

- source_kind: `mpi_sintel_final_pass`
- captures: `5`
- real_external_data_provided: `true`
- synthetic vs real: `real external data`

## Pipeline Description

- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite.
- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present.

## GPU Execution Summary

- measured_gpu: `true`
- kernel: `dsfb_host_minimum`
- capture `ambush_5_mixed_frame_0047`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.7045, dispatch_ms = 3.4956, readback_ms = 1.2071
- capture `ambush_5_point_frame_0047`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.2031, dispatch_ms = 3.5167, readback_ms = 0.6838
- capture `ambush_5_region_frame_0047`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.1064, dispatch_ms = 3.4947, readback_ms = 0.6099
- capture `market_6_mixed_frame_0008`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.0621, dispatch_ms = 3.5045, readback_ms = 0.5555
- capture `market_6_region_frame_0008`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 4.1113, dispatch_ms = 3.4776, readback_ms = 0.6310

## Demo A Results

- `ambush_5_mixed_frame_0047`: ROI source = `manifest_mask`, ROI pixels = 44646, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.08241, non-ROI MAE = 0.16425, temporal accumulation = 0.15606, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.06025, non-ROI MAE = 0.13688, temporal accumulation = 0.12922, intervention rate = 0.59988
  - DSFB host minimum: ROI MAE = 0.06302, non-ROI MAE = 0.13798, temporal accumulation = 0.13049, intervention rate = 0.33439
- `ambush_5_point_frame_0047`: ROI source = `manifest_mask`, ROI pixels = 13394, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.07654, non-ROI MAE = 0.15852, temporal accumulation = 0.15606, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.07062, non-ROI MAE = 0.13103, temporal accumulation = 0.12922, intervention rate = 0.59988
  - DSFB host minimum: ROI MAE = 0.07079, non-ROI MAE = 0.13233, temporal accumulation = 0.13049, intervention rate = 0.33439
- `ambush_5_region_frame_0047`: ROI source = `manifest_mask`, ROI pixels = 80364, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.07577, non-ROI MAE = 0.17369, temporal accumulation = 0.15606, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.05355, non-ROI MAE = 0.14582, temporal accumulation = 0.12922, intervention rate = 0.59988
  - DSFB host minimum: ROI MAE = 0.05990, non-ROI MAE = 0.14598, temporal accumulation = 0.13049, intervention rate = 0.33439
- `market_6_mixed_frame_0008`: ROI source = `manifest_mask`, ROI pixels = 44646, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.04857, non-ROI MAE = 0.06879, temporal accumulation = 0.06676, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.05467, non-ROI MAE = 0.06012, temporal accumulation = 0.05957, intervention rate = 0.94747
  - DSFB host minimum: ROI MAE = 0.04988, non-ROI MAE = 0.06025, temporal accumulation = 0.05921, intervention rate = 0.32315
- `market_6_region_frame_0008`: ROI source = `manifest_mask`, ROI pixels = 66970, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.04695, non-ROI MAE = 0.07026, temporal accumulation = 0.06676, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.05217, non-ROI MAE = 0.06088, temporal accumulation = 0.05957, intervention rate = 0.94747
  - DSFB host minimum: ROI MAE = 0.04784, non-ROI MAE = 0.06122, temporal accumulation = 0.05921, intervention rate = 0.32315

## Demo B Results

- `ambush_5_mixed_frame_0047`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.07109, global error = 0.11330, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.06921, global error = 0.11400, ROI mean spp = 2.048
  - Contrast-based: ROI error = 0.06838, global error = 0.11484, ROI mean spp = 2.087
  - Variance proxy: ROI error = 0.07658, global error = 0.11431, ROI mean spp = 1.730
  - Combined heuristic: ROI error = 0.07056, global error = 0.11275, ROI mean spp = 1.972
  - DSFB imported trust: ROI error = 0.07646, global error = 0.11237, ROI mean spp = 1.492
  - Hybrid trust + variance: ROI error = 0.07414, global error = 0.10647, ROI mean spp = 1.525
- `ambush_5_point_frame_0047`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.06632, global error = 0.11330, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.06407, global error = 0.11400, ROI mean spp = 2.060
  - Contrast-based: ROI error = 0.06329, global error = 0.11484, ROI mean spp = 2.088
  - Variance proxy: ROI error = 0.06913, global error = 0.11431, ROI mean spp = 1.828
  - Combined heuristic: ROI error = 0.06499, global error = 0.11275, ROI mean spp = 2.003
  - DSFB imported trust: ROI error = 0.07270, global error = 0.11237, ROI mean spp = 1.394
  - Hybrid trust + variance: ROI error = 0.07126, global error = 0.10647, ROI mean spp = 1.423
- `ambush_5_region_frame_0047`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.06518, global error = 0.11330, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.06447, global error = 0.11400, ROI mean spp = 2.007
  - Contrast-based: ROI error = 0.06405, global error = 0.11484, ROI mean spp = 2.028
  - Variance proxy: ROI error = 0.07174, global error = 0.11431, ROI mean spp = 1.670
  - Combined heuristic: ROI error = 0.06565, global error = 0.11275, ROI mean spp = 1.925
  - DSFB imported trust: ROI error = 0.07469, global error = 0.11237, ROI mean spp = 1.371
  - Hybrid trust + variance: ROI error = 0.06925, global error = 0.10647, ROI mean spp = 1.527
- `market_6_mixed_frame_0008`: regime = `aliasing_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.07889, global error = 0.11500, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.09439, global error = 0.11346, ROI mean spp = 1.450
  - Contrast-based: ROI error = 0.09869, global error = 0.11313, ROI mean spp = 1.309
  - Variance proxy: ROI error = 0.07886, global error = 0.11470, ROI mean spp = 2.000
  - Combined heuristic: ROI error = 0.08485, global error = 0.11088, ROI mean spp = 1.754
  - DSFB imported trust: ROI error = 0.09007, global error = 0.11434, ROI mean spp = 1.600
  - Hybrid trust + variance: ROI error = 0.08013, global error = 0.11246, ROI mean spp = 1.950
- `market_6_region_frame_0008`: regime = `aliasing_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.07922, global error = 0.11500, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.09378, global error = 0.11346, ROI mean spp = 1.475
  - Contrast-based: ROI error = 0.09799, global error = 0.11313, ROI mean spp = 1.337
  - Variance proxy: ROI error = 0.07934, global error = 0.11470, ROI mean spp = 1.994
  - Combined heuristic: ROI error = 0.08524, global error = 0.11088, ROI mean spp = 1.748
  - DSFB imported trust: ROI error = 0.09103, global error = 0.11434, ROI mean spp = 1.573
  - Hybrid trust + variance: ROI error = 0.08087, global error = 0.11246, ROI mean spp = 1.931

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
