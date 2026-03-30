# External Validation Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Data Description

- source_kind: `unreal_native`
- captures: `5`
- real_external_data_provided: `true`
- synthetic vs real: `real external data`

## Pipeline Description

- External replay uses the same DSFB host-minimum supervisory logic and the same minimum GPU kernel as the internal suite.
- Differences: imported buffers replace synthetic scene generation, and Demo B uses an allocation proxy because no live renderer samples are present.
- ROI contract: ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.

## GPU Execution Summary

- measured_gpu: `true`
- kernel: `dsfb_host_minimum`
- capture `frame_0001`: adapter = `llvmpipe (LLVM 22.1.1, 256 bits)`, total_ms = 19.4802, dispatch_ms = 19.3927, readback_ms = 0.0869
- capture `frame_0002`: adapter = `llvmpipe (LLVM 22.1.1, 256 bits)`, total_ms = 19.5371, dispatch_ms = 19.4499, readback_ms = 0.0867
- capture `frame_0003`: adapter = `llvmpipe (LLVM 22.1.1, 256 bits)`, total_ms = 18.9144, dispatch_ms = 18.8265, readback_ms = 0.0871
- capture `frame_0004`: adapter = `llvmpipe (LLVM 22.1.1, 256 bits)`, total_ms = 18.3180, dispatch_ms = 18.2410, readback_ms = 0.0767
- capture `frame_0005`: adapter = `llvmpipe (LLVM 22.1.1, 256 bits)`, total_ms = 18.4018, dispatch_ms = 18.3200, readback_ms = 0.0814

## Demo A Results

- `frame_0001`: ROI source = `fixed_alpha_local_contrast_0p15`, ROI pixels = 14159, ROI coverage = 38.41%, metric_source = `real_reference`
  - Fixed alpha baseline: full-frame MAE = 0.11153, ROI MAE = 0.28734, non-ROI MAE = 0.00190, max error = 0.60685, temporal accumulation = 0.11153, intervention rate = 0.00000
  - Strong heuristic clamp: full-frame MAE = 0.00413, ROI MAE = 0.00844, non-ROI MAE = 0.00145, max error = 0.30008, temporal accumulation = 0.00413, intervention rate = 0.33339
  - DSFB host minimum: full-frame MAE = 0.02148, ROI MAE = 0.05302, non-ROI MAE = 0.00182, max error = 0.30050, temporal accumulation = 0.02148, intervention rate = 0.21345
  - DSFB + strong heuristic: full-frame MAE = 0.00333, ROI MAE = 0.00635, non-ROI MAE = 0.00145, max error = 0.30008, temporal accumulation = 0.00333, intervention rate = 0.33373
- `frame_0002`: ROI source = `fixed_alpha_local_contrast_0p15`, ROI pixels = 27619, ROI coverage = 74.92%, metric_source = `real_reference`
  - Fixed alpha baseline: full-frame MAE = 0.34025, ROI MAE = 0.45358, non-ROI MAE = 0.00170, max error = 0.60852, temporal accumulation = 0.34025, intervention rate = 0.00000
  - Strong heuristic clamp: full-frame MAE = 0.00222, ROI MAE = 0.00265, non-ROI MAE = 0.00094, max error = 0.20031, temporal accumulation = 0.00222, intervention rate = 0.69737
  - DSFB host minimum: full-frame MAE = 0.02962, ROI MAE = 0.03903, non-ROI MAE = 0.00150, max error = 0.29211, temporal accumulation = 0.02962, intervention rate = 0.64758
  - DSFB + strong heuristic: full-frame MAE = 0.00175, ROI MAE = 0.00203, non-ROI MAE = 0.00094, max error = 0.19184, temporal accumulation = 0.00175, intervention rate = 0.69752
- `frame_0003`: ROI source = `fixed_alpha_local_contrast_0p15`, ROI pixels = 9205, ROI coverage = 24.97%, metric_source = `real_reference`
  - Fixed alpha baseline: full-frame MAE = 0.07257, ROI MAE = 0.28668, non-ROI MAE = 0.00132, max error = 0.60450, temporal accumulation = 0.07257, intervention rate = 0.00000
  - Strong heuristic clamp: full-frame MAE = 0.00296, ROI MAE = 0.00860, non-ROI MAE = 0.00108, max error = 0.31922, temporal accumulation = 0.00296, intervention rate = 0.21853
  - DSFB host minimum: full-frame MAE = 0.01444, ROI MAE = 0.05388, non-ROI MAE = 0.00131, max error = 0.25549, temporal accumulation = 0.01444, intervention rate = 0.13224
  - DSFB + strong heuristic: full-frame MAE = 0.00235, ROI MAE = 0.00615, non-ROI MAE = 0.00108, max error = 0.24858, temporal accumulation = 0.00235, intervention rate = 0.21878
- `frame_0004`: ROI source = `fixed_alpha_local_contrast_0p15`, ROI pixels = 17073, ROI coverage = 46.31%, metric_source = `real_reference`
  - Fixed alpha baseline: full-frame MAE = 0.10623, ROI MAE = 0.22578, non-ROI MAE = 0.00309, max error = 0.59634, temporal accumulation = 0.10623, intervention rate = 0.00000
  - Strong heuristic clamp: full-frame MAE = 0.00528, ROI MAE = 0.00852, non-ROI MAE = 0.00249, max error = 0.25255, temporal accumulation = 0.00528, intervention rate = 0.29559
  - DSFB host minimum: full-frame MAE = 0.02105, ROI MAE = 0.04196, non-ROI MAE = 0.00302, max error = 0.23677, temporal accumulation = 0.02105, intervention rate = 0.20294
  - DSFB + strong heuristic: full-frame MAE = 0.00440, ROI MAE = 0.00662, non-ROI MAE = 0.00248, max error = 0.21349, temporal accumulation = 0.00440, intervention rate = 0.29603
- `frame_0005`: ROI source = `fixed_alpha_local_contrast_0p15`, ROI pixels = 25211, ROI coverage = 68.39%, metric_source = `real_reference`
  - Fixed alpha baseline: full-frame MAE = 0.27106, ROI MAE = 0.39490, non-ROI MAE = 0.00314, max error = 0.60392, temporal accumulation = 0.27106, intervention rate = 0.00000
  - Strong heuristic clamp: full-frame MAE = 0.00398, ROI MAE = 0.00466, non-ROI MAE = 0.00252, max error = 0.30322, temporal accumulation = 0.00398, intervention rate = 0.56103
  - DSFB host minimum: full-frame MAE = 0.02714, ROI MAE = 0.03824, non-ROI MAE = 0.00312, max error = 0.36979, temporal accumulation = 0.02714, intervention rate = 0.50715
  - DSFB + strong heuristic: full-frame MAE = 0.00345, ROI MAE = 0.00388, non-ROI MAE = 0.00251, max error = 0.30322, temporal accumulation = 0.00345, intervention rate = 0.56137

## Demo B Results

- `frame_0001`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.27572, global error = 0.11694, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.27835, global error = 0.11595, ROI mean spp = 2.111
  - Contrast-based: ROI error = 0.27415, global error = 0.11393, ROI mean spp = 2.171
  - Variance proxy: ROI error = 0.19513, global error = 0.08761, ROI mean spp = 3.450
  - Combined heuristic: ROI error = 0.22313, global error = 0.09562, ROI mean spp = 2.796
  - DSFB imported trust: ROI error = 0.19726, global error = 0.08817, ROI mean spp = 3.431
  - Hybrid trust + variance: ROI error = 0.19252, global error = 0.08805, ROI mean spp = 3.542
- `frame_0002`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.38515, global error = 0.29184, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.40248, global error = 0.30383, ROI mean spp = 1.960
  - Contrast-based: ROI error = 0.40861, global error = 0.30825, ROI mean spp = 1.944
  - Variance proxy: ROI error = 0.35498, global error = 0.27051, ROI mean spp = 2.334
  - Combined heuristic: ROI error = 0.37702, global error = 0.28542, ROI mean spp = 2.114
  - DSFB imported trust: ROI error = 0.35576, global error = 0.27115, ROI mean spp = 2.335
  - Hybrid trust + variance: ROI error = 0.35531, global error = 0.27080, ROI mean spp = 2.335
- `frame_0003`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.25888, global error = 0.07558, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.25727, global error = 0.07354, ROI mean spp = 2.164
  - Contrast-based: ROI error = 0.25310, global error = 0.07207, ROI mean spp = 2.233
  - Variance proxy: ROI error = 0.15668, global error = 0.05244, ROI mean spp = 4.411
  - Combined heuristic: ROI error = 0.20053, global error = 0.06007, ROI mean spp = 2.962
  - DSFB imported trust: ROI error = 0.16317, global error = 0.05307, ROI mean spp = 4.161
  - Hybrid trust + variance: ROI error = 0.15700, global error = 0.05071, ROI mean spp = 4.445
- `frame_0004`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.23506, global error = 0.12029, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.25249, global error = 0.12556, ROI mean spp = 1.991
  - Contrast-based: ROI error = 0.25338, global error = 0.12554, ROI mean spp = 2.023
  - Variance proxy: ROI error = 0.17178, global error = 0.09324, ROI mean spp = 3.096
  - Combined heuristic: ROI error = 0.19330, global error = 0.09923, ROI mean spp = 2.638
  - DSFB imported trust: ROI error = 0.17477, global error = 0.09582, ROI mean spp = 3.126
  - Hybrid trust + variance: ROI error = 0.17344, global error = 0.09559, ROI mean spp = 3.143
- `frame_0005`: regime = `variance_limited`, fixed_budget_equal = `true`
  - Uniform: ROI error = 0.34649, global error = 0.24310, ROI mean spp = 2.000
  - Gradient magnitude: ROI error = 0.37150, global error = 0.25859, ROI mean spp = 1.927
  - Contrast-based: ROI error = 0.37840, global error = 0.26302, ROI mean spp = 1.911
  - Variance proxy: ROI error = 0.30063, global error = 0.21425, ROI mean spp = 2.462
  - Combined heuristic: ROI error = 0.32338, global error = 0.22653, ROI mean spp = 2.311
  - DSFB imported trust: ROI error = 0.30011, global error = 0.21389, ROI mean spp = 2.462
  - Hybrid trust + variance: ROI error = 0.30030, global error = 0.21403, ROI mean spp = 2.462

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
- The same GPU kernel can execute on imported buffers, with explicit measured-vs-unmeasured disclosure and isolated scaled-resolution probes when the standalone binary is available.
- ROI vs non-ROI reporting survives the external path, Demo B keeps equal budgets across stronger heuristic baselines, and Demo A now includes an explicit DSFB + strong heuristic hybrid.

## What Is Not Proven

- This report does not prove production-scene generalization.
- It does not prove engine integration unless real exported buffers are supplied.
- Demo B on imported captures remains an allocation proxy, not a renderer-integrated sampling benchmark.
- The trust trajectory is now measured across an ordered five-frame real Unreal-native sequence, but that short sequence is still not enough to claim broad temporal calibration.

## Remaining Blockers

- engine-side GPU profiling on imported buffers
- renderer-integrated Demo B replay with per-sample budgets

## Next Required Experiment

Move from the current five-frame exported Unreal-native sequence to a longer production-representative engine capture, preserve the same fixed ROI contract and baseline ladder, and confirm the trust trajectory plus scaled GPU timings on the target evaluation hardware.
