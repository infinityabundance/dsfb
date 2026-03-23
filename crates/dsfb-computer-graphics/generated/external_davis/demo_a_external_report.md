# Demo A External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below.

## Capture `dance-twirl_frame_0079`

- ROI source: `manifest_mask`
- ROI pixels: `54219`
- ground_truth_available: `false`
- metric_source: `proxy_current_vs_history`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`
- ground truth unavailable -> proxy metrics used

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.04913 | 0.00841 | 0.01380 | 0.00000 |
| Strong heuristic | 0.02146 | 0.04102 | 0.03843 | 0.16463 |
| DSFB host minimum | 0.02877 | 0.03509 | 0.03426 | 0.14198 |

## Capture `soapbox_frame_0069`

- ROI source: `manifest_mask`
- ROI pixels: `99358`
- ground_truth_available: `false`
- metric_source: `proxy_current_vs_history`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`
- ground truth unavailable -> proxy metrics used

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.06254 | 0.01058 | 0.02316 | 0.00000 |
| Strong heuristic | 0.02035 | 0.07045 | 0.05831 | 0.25879 |
| DSFB host minimum | 0.02977 | 0.05494 | 0.04882 | 0.20289 |

## Capture `camel_frame_0020`

- ROI source: `manifest_mask`
- ROI pixels: `63015`
- ground_truth_available: `false`
- metric_source: `proxy_current_vs_history`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`
- ground truth unavailable -> proxy metrics used

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.02066 | 0.00537 | 0.00772 | 0.00000 |
| Strong heuristic | 0.01321 | 0.01727 | 0.01664 | 0.07675 |
| DSFB host minimum | 0.01443 | 0.01440 | 0.01441 | 0.06533 |

## What Is Proven

- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha and a strong heuristic baseline.
- ROI and non-ROI behavior remain separated on imported data.

## What Is Not Proven

- Without an optional reference frame, ROI MAE and non-ROI MAE are proxy quantities rather than true reconstruction error.
- Even with a reference frame, this does not replace longer engine-side sequences.

## Remaining Blockers

- Real engine capture sequences and longer temporal windows still need evaluation.
