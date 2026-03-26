# Demo A External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below.

## Capture `frame_0001`

- ROI source: `manifest_mask`
- ROI pixels: `20714`
- ground_truth_available: `false`
- metric_source: `proxy_current_vs_history`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`
- ground truth unavailable -> proxy metrics used

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.36410 | 0.00003 | 0.20460 | 0.00000 |
| Strong heuristic | 0.00323 | 0.00015 | 0.00188 | 0.51604 |
| DSFB host minimum | 0.03604 | 0.00004 | 0.02027 | 0.43167 |

## What Is Proven

- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha and a strong heuristic baseline.
- ROI and non-ROI behavior remain separated on imported data.

## What Is Not Proven

- Without an optional reference frame, ROI MAE and non-ROI MAE are proxy quantities rather than true reconstruction error.
- Even with a reference frame, this does not replace longer engine-side sequences.

## Remaining Blockers

- Real engine capture sequences and longer temporal windows still need evaluation.
