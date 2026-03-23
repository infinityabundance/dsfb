# Demo B External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Regime labels used in this report: `aliasing_limited`, `variance_limited`, `mixed_regime`.

## Capture `dance-twirl_frame_0079`

- regime: `aliasing_limited`
- metric_source: `allocation_proxy_without_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `54219`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 819840 | 0.17679 | 0.19889 | 2.000 | 2.000 |
| Gradient magnitude | 819840 | 0.17441 | 0.19342 | 1.970 | 2.005 |
| Contrast-based | 819840 | 0.17682 | 0.18889 | 1.854 | 2.022 |
| Variance proxy | 819840 | 0.17680 | 0.18521 | 1.774 | 2.034 |
| Combined heuristic | 819840 | 0.16815 | 0.17977 | 1.802 | 2.030 |
| DSFB imported trust | 819840 | 0.17020 | 0.18716 | 1.901 | 2.015 |
| Hybrid trust + variance | 819840 | 0.17649 | 0.18769 | 1.824 | 2.027 |

Aliasing vs variance discussion:
- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from 0.17679 for uniform to 0.17020, while combined heuristic reached 0.16815.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `soapbox_frame_0069`

- regime: `variance_limited`
- metric_source: `allocation_proxy_without_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `99358`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 819840 | 0.17879 | 0.18474 | 2.000 | 2.000 |
| Gradient magnitude | 819840 | 0.16284 | 0.18108 | 2.242 | 1.923 |
| Contrast-based | 819840 | 0.15760 | 0.17903 | 2.310 | 1.901 |
| Variance proxy | 819840 | 0.16733 | 0.16495 | 1.832 | 2.054 |
| Combined heuristic | 819840 | 0.15402 | 0.16361 | 2.061 | 1.980 |
| DSFB imported trust | 819840 | 0.16658 | 0.16627 | 1.846 | 2.049 |
| Hybrid trust + variance | 819840 | 0.16990 | 0.16706 | 1.818 | 2.058 |

Aliasing vs variance discussion:
- This imported capture is primarily variance-limited. Variance-like heuristics should remain competitive, and the imported trust result changed ROI proxy error from 0.17879 for uniform to 0.16658, versus 0.15402 for the combined heuristic.
- The strongest heuristic proxy wins on this imported capture; that result is surfaced rather than hidden.

## Capture `camel_frame_0020`

- regime: `aliasing_limited`
- metric_source: `allocation_proxy_without_reference`
- fixed_budget_equal: `true`
- ROI source: `manifest_mask`
- ROI pixels: `63015`

| Policy | total samples | ROI error | global error | ROI mean spp | non-ROI mean spp |
| --- | ---: | ---: | ---: | ---: | ---: |
| Uniform | 819840 | 0.10395 | 0.17502 | 2.000 | 2.000 |
| Gradient magnitude | 819840 | 0.11209 | 0.16566 | 1.447 | 2.100 |
| Contrast-based | 819840 | 0.10926 | 0.16150 | 1.469 | 2.096 |
| Variance proxy | 819840 | 0.11026 | 0.16651 | 1.508 | 2.089 |
| Combined heuristic | 819840 | 0.10652 | 0.15854 | 1.475 | 2.095 |
| DSFB imported trust | 819840 | 0.09703 | 0.16865 | 2.163 | 1.970 |
| Hybrid trust + variance | 819840 | 0.10636 | 0.16689 | 1.633 | 2.067 |

Aliasing vs variance discussion:
- This imported capture is primarily aliasing-limited. DSFB imported trust changed ROI proxy error from 0.10395 for uniform to 0.09703, while combined heuristic reached 0.10652.
- DSFB imported trust wins on this imported capture under equal total budget.

## What Is Proven

- Imported captures can drive fixed-budget allocation policies, including DSFB imported trust and stronger heuristic competitors.
- Budget equality is enforced across all compared policies.

## What Is Not Proven

- This is still an allocation proxy because imported captures do not provide a live renderer or per-sample ground truth.
- It does not replace real engine-side fixed-budget sampling experiments.

## Remaining Blockers

- A renderer-integrated fixed-budget replay still needs to confirm the allocation story on real per-sample shading.
