# External Validation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Data Description

- source_kind: `synthetic_compat`
- captures: `1`
- real_external_data_provided: `false`
- synthetic vs real: `synthetic compatibility export`

NO REAL EXTERNAL DATA PROVIDED

## Pipeline Description

- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite.
- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present.

## GPU Execution Summary

- measured_gpu: `true`
- kernel: `dsfb_host_minimum`
- capture `capture_0`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 0.3605, dispatch_ms = 0.3082, readback_ms = 0.0505

## Demo A Results

- `capture_0`: ROI source = `manifest_mask`, ROI pixels = 734, metric_source = `real_reference`
  - Fixed alpha baseline: ROI MAE = 0.22364, non-ROI MAE = 0.05403, temporal accumulation = 0.06214, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.06005, non-ROI MAE = 0.01232, temporal accumulation = 0.01460, intervention rate = 0.28934
  - DSFB host minimum: ROI MAE = 0.02654, non-ROI MAE = 0.01752, temporal accumulation = 0.01795, intervention rate = 0.23019

## Demo B Results

- `capture_0`: regime = `aliasing_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.64560, global error = 0.22047, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.45574, global error = 0.20538, ROI mean spp = 4.441
  - Contrast-based: ROI error = 0.49373, global error = 0.20104, ROI mean spp = 3.623
  - Variance proxy: ROI error = 0.43597, global error = 0.19146, ROI mean spp = 4.497
  - Combined heuristic: ROI error = 0.43391, global error = 0.18755, ROI mean spp = 4.499
  - DSFB imported trust: ROI error = 0.40582, global error = 0.19027, ROI mean spp = 5.135
  - Hybrid trust + variance: ROI error = 0.41843, global error = 0.19116, ROI mean spp = 4.798

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
