# Demo A External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

NO REAL EXTERNAL DATA PROVIDED

Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below.

## Capture `capture_0`

- ROI source: `manifest_mask`
- ROI pixels: `734`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `point_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `not_a_larger_roi_case`

| Method | ROI MAE | non-ROI MAE | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.22364 | 0.05403 | 0.06214 | 0.00000 |
| Strong heuristic | 0.06005 | 0.01232 | 0.01460 | 0.28934 |
| DSFB host minimum | 0.02654 | 0.01752 | 0.01795 | 0.23019 |

## What Is Proven

- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha and a strong heuristic baseline.
- ROI and non-ROI behavior remain separated on imported data.

## What Is Not Proven

- Without an optional reference frame, ROI MAE and non-ROI MAE are proxy quantities rather than true reconstruction error.
- Even with a reference frame, this does not replace longer engine-side sequences.

## Remaining Blockers

- Real engine capture sequences and longer temporal windows still need evaluation.
