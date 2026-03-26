# Demo B External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`.

## Capture `frame_0001`

- regime: `variance_limited`
- metric_source: `allocation_proxy_without_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `20714`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 73728 | 0.33786 | 0.19147 | 2.000 | 2.000 |
| Gradient magnitude | 73728 | 0.34501 | 0.19509 | 2.079 | 1.899 |
| Contrast-based | 73728 | 0.34714 | 0.19627 | 2.100 | 1.872 |
| Variance proxy | 73728 | 0.27266 | 0.15551 | 2.780 | 1.000 |
| Combined heuristic | 73728 | 0.27688 | 0.15729 | 2.746 | 1.044 |
| DSFB imported trust | 73728 | 0.27466 | 0.15656 | 2.779 | 1.001 |
| Hybrid trust + variance | 73728 | 0.27329 | 0.15586 | 2.780 | 1.000 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.33786 for uniform to 0.27466, versus 0.27688 for the combined heuristic.
- DSFB imported trust wins on this imported capture under equal total budget.

## What Is Proven

- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors.
- Budget equality is enforced across all compared policies.

## What Is Not Proven

- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth.
- It does not replace real engine-side fixed-budget sampling experiments.

## Remaining Blockers

- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading.
