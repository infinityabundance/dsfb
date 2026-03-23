# Demo B External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`.

## Capture `ambush_5_mixed_frame_0047`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `44646`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 892928 | 0.07109 | 0.11330 | 2.000 | 2.000 |
| Gradient magnitude | 892928 | 0.06921 | 0.11400 | 2.048 | 1.995 |
| Contrast-based | 892928 | 0.06838 | 0.11484 | 2.087 | 1.990 |
| Variance proxy | 892928 | 0.07658 | 0.11431 | 1.730 | 2.030 |
| Combined heuristic | 892928 | 0.07056 | 0.11275 | 1.972 | 2.003 |
| DSFB imported trust | 892928 | 0.07646 | 0.11237 | 1.492 | 2.056 |
| Hybrid trust + variance | 892928 | 0.07414 | 0.10647 | 1.525 | 2.053 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.07109 for uniform to 0.07646, versus 0.07056 for the combined heuristic.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `ambush_5_point_frame_0047`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `13394`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 892928 | 0.06632 | 0.11330 | 2.000 | 2.000 |
| Gradient magnitude | 892928 | 0.06407 | 0.11400 | 2.060 | 1.998 |
| Contrast-based | 892928 | 0.06329 | 0.11484 | 2.088 | 1.997 |
| Variance proxy | 892928 | 0.06913 | 0.11431 | 1.828 | 2.005 |
| Combined heuristic | 892928 | 0.06499 | 0.11275 | 2.003 | 2.000 |
| DSFB imported trust | 892928 | 0.07270 | 0.11237 | 1.394 | 2.019 |
| Hybrid trust + variance | 892928 | 0.07126 | 0.10647 | 1.423 | 2.018 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.06632 for uniform to 0.07270, versus 0.06499 for the combined heuristic.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `ambush_5_region_frame_0047`

- regime: `variance_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `80364`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 892928 | 0.06518 | 0.11330 | 2.000 | 2.000 |
| Gradient magnitude | 892928 | 0.06447 | 0.11400 | 2.007 | 1.998 |
| Contrast-based | 892928 | 0.06405 | 0.11484 | 2.028 | 1.994 |
| Variance proxy | 892928 | 0.07174 | 0.11431 | 1.670 | 2.072 |
| Combined heuristic | 892928 | 0.06565 | 0.11275 | 1.925 | 2.016 |
| DSFB imported trust | 892928 | 0.07469 | 0.11237 | 1.371 | 2.138 |
| Hybrid trust + variance | 892928 | 0.06925 | 0.10647 | 1.527 | 2.104 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.06518 for uniform to 0.07469, versus 0.06565 for the combined heuristic.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `market_6_mixed_frame_0008`

- regime: `aliasing_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `44646`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 892928 | 0.07889 | 0.11500 | 2.000 | 2.000 |
| Gradient magnitude | 892928 | 0.09439 | 0.11346 | 1.450 | 2.061 |
| Contrast-based | 892928 | 0.09869 | 0.11313 | 1.309 | 2.077 |
| Variance proxy | 892928 | 0.07886 | 0.11470 | 2.000 | 2.000 |
| Combined heuristic | 892928 | 0.08485 | 0.11088 | 1.754 | 2.027 |
| DSFB imported trust | 892928 | 0.09007 | 0.11434 | 1.600 | 2.044 |
| Hybrid trust + variance | 892928 | 0.08013 | 0.11246 | 1.950 | 2.006 |

Aliasing vs variance discussion:
- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from 0.07889 for uniform to 0.09007, while combined heuristic reached 0.08485.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `market_6_region_frame_0008`

- regime: `aliasing_limited`
- metric_source: `allocation_proxy_with_optional_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `66970`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 892928 | 0.07922 | 0.11500 | 2.000 | 2.000 |
| Gradient magnitude | 892928 | 0.09378 | 0.11346 | 1.475 | 2.093 |
| Contrast-based | 892928 | 0.09799 | 0.11313 | 1.337 | 2.117 |
| Variance proxy | 892928 | 0.07934 | 0.11470 | 1.994 | 2.001 |
| Combined heuristic | 892928 | 0.08524 | 0.11088 | 1.748 | 2.044 |
| DSFB imported trust | 892928 | 0.09103 | 0.11434 | 1.573 | 2.075 |
| Hybrid trust + variance | 892928 | 0.08087 | 0.11246 | 1.931 | 2.012 |

Aliasing vs variance discussion:
- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from 0.07922 for uniform to 0.09103, while combined heuristic reached 0.08524.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## What Is Proven

- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors.
- Budget equality is enforced across all compared policies.

## What Is Not Proven

- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth.
- It does not replace real engine-side fixed-budget sampling experiments.

## Remaining Blockers

- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading.
