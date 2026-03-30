# Demo B External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`.

ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.

## Capture `frame_0001`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `14159`
- ROI coverage: `38.41%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.27572 | 0.11694 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.27835 | 0.11595 | 2.111 | 1.931 |
| Contrast-based | 73728 | 0.27415 | 0.11393 | 2.171 | 1.893 |
| Variance proxy | 73728 | 0.19513 | 0.08761 | 3.450 | 1.096 |
| Combined heuristic | 73728 | 0.22313 | 0.09562 | 2.796 | 1.504 |
| DSFB imported trust | 73728 | 0.19726 | 0.08817 | 3.431 | 1.108 |
| Hybrid trust + variance | 73728 | 0.19252 | 0.08805 | 3.542 | 1.038 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.27572 for uniform to 0.19726, versus 0.22313 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## Capture `frame_0002`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `27619`
- ROI coverage: `74.92%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.38515 | 0.29184 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.40248 | 0.30383 | 1.960 | 2.119 |
| Contrast-based | 73728 | 0.40861 | 0.30825 | 1.944 | 2.166 |
| Variance proxy | 73728 | 0.35498 | 0.27051 | 2.334 | 1.002 |
| Combined heuristic | 73728 | 0.37702 | 0.28542 | 2.114 | 1.660 |
| DSFB imported trust | 73728 | 0.35576 | 0.27115 | 2.335 | 1.000 |
| Hybrid trust + variance | 73728 | 0.35531 | 0.27080 | 2.335 | 1.001 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.38515 for uniform to 0.35576, versus 0.37702 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## Capture `frame_0003`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `9205`
- ROI coverage: `24.97%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.25888 | 0.07558 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.25727 | 0.07354 | 2.164 | 1.946 |
| Contrast-based | 73728 | 0.25310 | 0.07207 | 2.233 | 1.922 |
| Variance proxy | 73728 | 0.15668 | 0.05244 | 4.411 | 1.198 |
| Combined heuristic | 73728 | 0.20053 | 0.06007 | 2.962 | 1.680 |
| DSFB imported trust | 73728 | 0.16317 | 0.05307 | 4.161 | 1.281 |
| Hybrid trust + variance | 73728 | 0.15700 | 0.05071 | 4.445 | 1.186 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.25888 for uniform to 0.16317, versus 0.20053 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## Capture `frame_0004`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `17073`
- ROI coverage: `46.31%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.23506 | 0.12029 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.25249 | 0.12556 | 1.991 | 2.008 |
| Contrast-based | 73728 | 0.25338 | 0.12554 | 2.023 | 1.980 |
| Variance proxy | 73728 | 0.17178 | 0.09324 | 3.096 | 1.055 |
| Combined heuristic | 73728 | 0.19330 | 0.09923 | 2.638 | 1.449 |
| DSFB imported trust | 73728 | 0.17477 | 0.09582 | 3.126 | 1.029 |
| Hybrid trust + variance | 73728 | 0.17344 | 0.09559 | 3.143 | 1.014 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.23506 for uniform to 0.17477, versus 0.19330 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## Capture `frame_0005`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `25211`
- ROI coverage: `68.39%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.34649 | 0.24310 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.37150 | 0.25859 | 1.927 | 2.157 |
| Contrast-based | 73728 | 0.37840 | 0.26302 | 1.911 | 2.192 |
| Variance proxy | 73728 | 0.30063 | 0.21425 | 2.462 | 1.000 |
| Combined heuristic | 73728 | 0.32338 | 0.22653 | 2.311 | 1.327 |
| DSFB imported trust | 73728 | 0.30011 | 0.21389 | 2.462 | 1.001 |
| Hybrid trust + variance | 73728 | 0.30030 | 0.21403 | 2.462 | 1.000 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.34649 for uniform to 0.30011, versus 0.32338 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## What Is Proven

- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors.
- Budget equality is enforced across all compared policies.

## What Is Not Proven

- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth.
- It does not replace real engine-side fixed-budget sampling experiments.

## Remaining Blockers

- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading.
