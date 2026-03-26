# External Validation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Data Description

- source_kind: `unreal_native`
- captures: `1`
- real_external_data_provided: `true`
- synthetic vs real: `real external data`

## Pipeline Description

- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite.
- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present.

## GPU Execution Summary

- measured_gpu: `true`
- kernel: `dsfb_host_minimum`
- capture `frame_0001`: adapter = `NVIDIA GeForce RTX 4080 SUPER`, total_ms = 0.4814, dispatch_ms = 0.3996, readback_ms = 0.0808

## Demo A Results

- `frame_0001`: ROI source = `manifest_mask`, ROI pixels = 20714, metric_source = `proxy_current_vs_history`
  - Fixed alpha baseline: ROI MAE = 0.36410, non-ROI MAE = 0.00003, temporal accumulation = 0.20460, intervention rate = 0.00000
  - Strong heuristic: ROI MAE = 0.00323, non-ROI MAE = 0.00015, temporal accumulation = 0.00188, intervention rate = 0.51604
  - DSFB host minimum: ROI MAE = 0.03604, non-ROI MAE = 0.00004, temporal accumulation = 0.02027, intervention rate = 0.43167

## Demo B Results

- `frame_0001`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.33786, global error = 0.19147, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.34501, global error = 0.19509, ROI mean spp = 2.079
  - Contrast-based: ROI error = 0.34714, global error = 0.19627, ROI mean spp = 2.100
  - Variance proxy: ROI error = 0.27266, global error = 0.15551, ROI mean spp = 2.780
  - Combined heuristic: ROI error = 0.27688, global error = 0.15729, ROI mean spp = 2.746
  - DSFB imported trust: ROI error = 0.27466, global error = 0.15656, ROI mean spp = 2.779
  - Hybrid trust + variance: ROI error = 0.27329, global error = 0.15586, ROI mean spp = 2.780

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
