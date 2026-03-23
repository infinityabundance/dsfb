# Demo B External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

NO REAL EXTERNAL DATA PROVIDED

Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`.

## Capture `capture_0`

- regime: `aliasing_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `734`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 30720 | 0.64560 | 0.22047 | 2.000 | 2.000 |
| Gradient magnitude | 30720 | 0.45574 | 0.20538 | 4.441 | 1.877 |
| Contrast-based | 30720 | 0.49373 | 0.20104 | 3.623 | 1.919 |
| Variance proxy | 30720 | 0.43597 | 0.19146 | 4.497 | 1.875 |
| Combined heuristic | 30720 | 0.43391 | 0.18755 | 4.499 | 1.875 |
| DSFB imported trust | 30720 | 0.40582 | 0.19027 | 5.135 | 1.843 |
| Hybrid trust + variance | 30720 | 0.41843 | 0.19116 | 4.798 | 1.860 |

Aliasing vs variance discussion:
- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from 0.64560 for uniform to 0.40582, while combined heuristic reached 0.43391.
- DSFB imported trust wins on this imported capture under equal total budget.

## What Is Proven

- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors.
- Budget equality is enforced across all compared policies.

## What Is Not Proven

- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth.
- It does not replace real engine-side fixed-budget sampling experiments.

## Remaining Blockers

- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading.
