# Demo A External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below.

## Capture `ambush_5_mixed_frame_0047`

- ROI source: `manifest_mask`
- ROI pixels: `44646`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `not_a_larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.08241 | 0.16425 | 0.15606 | 0.00000 |
| Strong heuristic | 0.06025 | 0.13688 | 0.12922 | 0.59988 |
| DSFB host minimum | 0.06302 | 0.13798 | 0.13049 | 0.33439 |

## Capture `ambush_5_point_frame_0047`

- ROI source: `manifest_mask`
- ROI pixels: `13394`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `point_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `not_a_larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.07654 | 0.15852 | 0.15606 | 0.00000 |
| Strong heuristic | 0.07062 | 0.13103 | 0.12922 | 0.59988 |
| DSFB host minimum | 0.07079 | 0.13233 | 0.13049 | 0.33439 |

## Capture `ambush_5_region_frame_0047`

- ROI source: `manifest_mask`
- ROI pixels: `80364`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.07577 | 0.17369 | 0.15606 | 0.00000 |
| Strong heuristic | 0.05355 | 0.14582 | 0.12922 | 0.59988 |
| DSFB host minimum | 0.05990 | 0.14598 | 0.13049 | 0.33439 |

## Capture `market_6_mixed_frame_0008`

- ROI source: `manifest_mask`
- ROI pixels: `44646`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `not_a_larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.04857 | 0.06879 | 0.06676 | 0.00000 |
| Strong heuristic | 0.05467 | 0.06012 | 0.05957 | 0.94747 |
| DSFB host minimum | 0.04988 | 0.06025 | 0.05921 | 0.32315 |

## Capture `market_6_region_frame_0008`

- ROI source: `manifest_mask`
- ROI pixels: `66970`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.04695 | 0.07026 | 0.06676 | 0.00000 |
| Strong heuristic | 0.05217 | 0.06088 | 0.05957 | 0.94747 |
| DSFB host minimum | 0.04784 | 0.06122 | 0.05921 | 0.32315 |

## What Is Proven

- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha and a strong heuristic baseline.
- ROI and non-ROI behavior remain separated on imported data.

## What Is Not Proven

- Without an optional reference frame, ROI MAE and non-ROI MAE are proxy quantities rather than true reconstruction error.
- Even with a reference frame, this does not replace longer engine-side sequences.

## Remaining Blockers

- Real engine capture sequences and longer temporal windows still need evaluation.
